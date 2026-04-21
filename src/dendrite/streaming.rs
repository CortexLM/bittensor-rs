//! Streaming support for Dendrite requests
//!
//! This module provides types and utilities for streaming responses
//! from Axon servers, allowing for incremental processing of large
//! or continuous data streams.

use futures::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Trait for synapses that support streaming responses
///
/// Types implementing this trait can process response chunks incrementally
/// rather than waiting for the complete response.
pub trait StreamingSynapse: Send {
    /// The type of each chunk produced by the stream
    type Chunk: Send;

    /// Process a chunk of data from the response stream
    ///
    /// # Arguments
    ///
    /// * `chunk` - Raw bytes from the response stream
    ///
    /// # Returns
    ///
    /// `Some(chunk)` if a complete chunk was parsed, `None` if more data is needed
    fn process_chunk(&mut self, chunk: &[u8]) -> Option<Self::Chunk>;

    /// Check if the stream is complete
    ///
    /// Returns `true` when no more chunks are expected
    fn is_complete(&self) -> bool;

    /// Get the name of this streaming synapse
    fn name(&self) -> &str;

    /// Called when the stream ends (either normally or due to error)
    ///
    /// Default implementation does nothing
    fn on_stream_end(&mut self) {}
}

/// A streaming response wrapper that implements the Stream trait
///
/// This wraps an async bytes stream and a StreamingSynapse to produce
/// parsed chunks as they arrive.
pub struct StreamingResponse<S, B>
where
    S: StreamingSynapse,
    B: Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Unpin,
{
    /// The synapse processor
    synapse: S,
    /// The underlying byte stream
    byte_stream: B,
    /// Buffer for incomplete chunks
    buffer: Vec<u8>,
    /// Whether the stream has completed
    completed: bool,
}

impl<S, B> StreamingResponse<S, B>
where
    S: StreamingSynapse,
    B: Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Unpin,
{
    /// Create a new StreamingResponse
    ///
    /// # Arguments
    ///
    /// * `synapse` - The streaming synapse processor
    /// * `byte_stream` - The underlying byte stream from the HTTP response
    pub fn new(synapse: S, byte_stream: B) -> Self {
        Self {
            synapse,
            byte_stream,
            buffer: Vec::with_capacity(4096),
            completed: false,
        }
    }

    /// Get a reference to the underlying synapse
    pub fn synapse(&self) -> &S {
        &self.synapse
    }

    /// Get a mutable reference to the underlying synapse
    pub fn synapse_mut(&mut self) -> &mut S {
        &mut self.synapse
    }

    /// Check if the stream has completed
    pub fn is_completed(&self) -> bool {
        self.completed
    }
}

impl<S, B> Stream for StreamingResponse<S, B>
where
    S: StreamingSynapse + Unpin,
    B: Stream<Item = Result<bytes::Bytes, reqwest::Error>> + Unpin,
{
    type Item = Result<S::Chunk, StreamError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = &mut *self;

        if this.completed || this.synapse.is_complete() {
            this.synapse.on_stream_end();
            return Poll::Ready(None);
        }

        // Try to process any buffered data first
        if !this.buffer.is_empty() {
            if let Some(chunk) = this.synapse.process_chunk(&this.buffer) {
                this.buffer.clear();
                return Poll::Ready(Some(Ok(chunk)));
            }
        }

        // Poll the underlying stream for more data
        match Pin::new(&mut this.byte_stream).poll_next(cx) {
            Poll::Ready(Some(Ok(bytes))) => {
                // Append to buffer
                this.buffer.extend_from_slice(&bytes);

                // Try to process the chunk
                if let Some(chunk) = this.synapse.process_chunk(&this.buffer) {
                    this.buffer.clear();
                    Poll::Ready(Some(Ok(chunk)))
                } else {
                    // Need more data, continue polling
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            }
            Poll::Ready(Some(Err(e))) => {
                this.completed = true;
                this.synapse.on_stream_end();
                Poll::Ready(Some(Err(StreamError::Network(e.to_string()))))
            }
            Poll::Ready(None) => {
                // Stream ended
                this.completed = true;
                this.synapse.on_stream_end();

                // Process any remaining buffered data
                if !this.buffer.is_empty() {
                    if let Some(chunk) = this.synapse.process_chunk(&this.buffer) {
                        this.buffer.clear();
                        return Poll::Ready(Some(Ok(chunk)));
                    }
                }

                Poll::Ready(None)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Errors that can occur during streaming
#[derive(Debug, thiserror::Error)]
pub enum StreamError {
    #[error("Network error: {0}")]
    Network(String),
    #[error("Parse error: {0}")]
    Parse(String),
    #[error("Stream timeout")]
    Timeout,
    #[error("Stream cancelled")]
    Cancelled,
}

/// A simple text streaming synapse implementation
///
/// Processes newline-delimited text chunks
#[derive(Debug)]
pub struct TextStreamingSynapse {
    name: String,
    complete: bool,
    delimiter: u8,
}

impl TextStreamingSynapse {
    /// Create a new text streaming synapse with newline delimiter
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            complete: false,
            delimiter: b'\n',
        }
    }

    /// Set a custom delimiter byte
    pub fn with_delimiter(mut self, delimiter: u8) -> Self {
        self.delimiter = delimiter;
        self
    }
}

impl StreamingSynapse for TextStreamingSynapse {
    type Chunk = String;

    fn process_chunk(&mut self, chunk: &[u8]) -> Option<Self::Chunk> {
        // Look for delimiter
        if let Some(pos) = chunk.iter().position(|&b| b == self.delimiter) {
            let text = String::from_utf8_lossy(&chunk[..pos]).to_string();
            Some(text)
        } else {
            None
        }
    }

    fn is_complete(&self) -> bool {
        self.complete
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn on_stream_end(&mut self) {
        self.complete = true;
    }
}

/// A JSON streaming synapse implementation
///
/// Processes newline-delimited JSON objects (NDJSON/JSON Lines format)
#[derive(Debug)]
pub struct JsonStreamingSynapse<T> {
    name: String,
    complete: bool,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> JsonStreamingSynapse<T>
where
    T: serde::de::DeserializeOwned + Send,
{
    /// Create a new JSON streaming synapse
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            complete: false,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T> StreamingSynapse for JsonStreamingSynapse<T>
where
    T: serde::de::DeserializeOwned + Send,
{
    type Chunk = T;

    fn process_chunk(&mut self, chunk: &[u8]) -> Option<Self::Chunk> {
        // Look for newline-delimited JSON
        if let Some(pos) = chunk.iter().position(|&b| b == b'\n') {
            let json_bytes = &chunk[..pos];
            if json_bytes.is_empty() {
                return None;
            }
            serde_json::from_slice(json_bytes).ok()
        } else {
            // Try to parse the entire buffer as a single JSON object
            serde_json::from_slice(chunk).ok()
        }
    }

    fn is_complete(&self) -> bool {
        self.complete
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn on_stream_end(&mut self) {
        self.complete = true;
    }
}

/// Server-Sent Events (SSE) streaming synapse
///
/// Processes SSE format: `data: <content>\n\n`
#[derive(Debug)]
pub struct SseStreamingSynapse {
    name: String,
    complete: bool,
}

impl SseStreamingSynapse {
    /// Create a new SSE streaming synapse
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            complete: false,
        }
    }
}

/// An SSE event
#[derive(Debug, Clone)]
pub struct SseEvent {
    /// Event type (from `event:` field)
    pub event: Option<String>,
    /// Event data (from `data:` field)
    pub data: String,
    /// Event ID (from `id:` field)
    pub id: Option<String>,
}

impl StreamingSynapse for SseStreamingSynapse {
    type Chunk = SseEvent;

    fn process_chunk(&mut self, chunk: &[u8]) -> Option<Self::Chunk> {
        let text = std::str::from_utf8(chunk).ok()?;

        // Look for double newline which marks end of event
        if let Some(pos) = text.find("\n\n") {
            let event_text = &text[..pos];
            let mut event = None;
            let mut data = String::new();
            let mut id = None;

            for line in event_text.lines() {
                if let Some(value) = line.strip_prefix("event:") {
                    event = Some(value.trim().to_string());
                } else if let Some(value) = line.strip_prefix("data:") {
                    if !data.is_empty() {
                        data.push('\n');
                    }
                    data.push_str(value.trim());
                } else if let Some(value) = line.strip_prefix("id:") {
                    id = Some(value.trim().to_string());
                }
            }

            if !data.is_empty() {
                return Some(SseEvent { event, data, id });
            }
        }

        None
    }

    fn is_complete(&self) -> bool {
        self.complete
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn on_stream_end(&mut self) {
        self.complete = true;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_streaming_synapse() {
        let mut synapse = TextStreamingSynapse::new("test");

        // Incomplete chunk (no newline)
        assert!(synapse.process_chunk(b"hello").is_none());

        // Complete chunk
        assert_eq!(synapse.process_chunk(b"hello\n"), Some("hello".to_string()));

        // Multiple lines, should only return first
        assert_eq!(
            synapse.process_chunk(b"line1\nline2\n"),
            Some("line1".to_string())
        );
    }

    #[test]
    fn test_json_streaming_synapse() {
        use serde::Deserialize;

        #[derive(Debug, Deserialize, PartialEq)]
        struct TestData {
            value: i32,
        }

        let mut synapse: JsonStreamingSynapse<TestData> = JsonStreamingSynapse::new("test");

        // Valid JSON with newline
        let chunk = synapse.process_chunk(br#"{"value": 42}"#.as_slice());
        assert_eq!(chunk, Some(TestData { value: 42 }));

        // NDJSON format
        let chunk = synapse.process_chunk(
            br#"{"value": 100}
"#,
        );
        assert_eq!(chunk, Some(TestData { value: 100 }));
    }

    #[test]
    fn test_sse_streaming_synapse() {
        let mut synapse = SseStreamingSynapse::new("test");

        // Single data event
        let chunk = synapse.process_chunk(b"data: hello world\n\n");
        assert!(chunk.is_some());
        let event = chunk.unwrap();
        assert_eq!(event.data, "hello world");
        assert!(event.event.is_none());

        // Event with type and id
        let chunk = synapse.process_chunk(b"event: message\nid: 123\ndata: test data\n\n");
        assert!(chunk.is_some());
        let event = chunk.unwrap();
        assert_eq!(event.event, Some("message".to_string()));
        assert_eq!(event.id, Some("123".to_string()));
        assert_eq!(event.data, "test data");

        // Incomplete event (no double newline)
        assert!(synapse.process_chunk(b"data: incomplete").is_none());
    }

    #[test]
    fn test_sse_multiline_data() {
        let mut synapse = SseStreamingSynapse::new("test");

        // Multi-line data
        let chunk = synapse.process_chunk(b"data: line1\ndata: line2\n\n");
        assert!(chunk.is_some());
        let event = chunk.unwrap();
        assert_eq!(event.data, "line1\nline2");
    }

    #[test]
    fn test_text_streaming_custom_delimiter() {
        let mut synapse = TextStreamingSynapse::new("test").with_delimiter(b'|');

        assert!(synapse.process_chunk(b"hello\n").is_none());
        assert_eq!(synapse.process_chunk(b"hello|"), Some("hello".to_string()));
    }

    #[test]
    fn test_streaming_synapse_completion() {
        let mut synapse = TextStreamingSynapse::new("test");

        assert!(!synapse.is_complete());
        synapse.on_stream_end();
        assert!(synapse.is_complete());
    }
}
