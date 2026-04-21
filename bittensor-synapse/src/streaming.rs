//! Streaming synapse trait for Server-Sent Events (SSE) handling.

use crate::synapse::{Synapse, SynapseError};

/// Extension of the Synapse trait for streaming responses via SSE.
///
/// Streaming synapses process incremental chunks from the server
/// rather than waiting for the complete body.
pub trait StreamingSynapse: Synapse {
    /// The type produced from each SSE chunk.
    type StreamItem;

    /// Process a single SSE data chunk and produce a stream item.
    fn process_chunk(chunk: &[u8]) -> Result<Self::StreamItem, SynapseError>;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terminal_info::TerminalInfo;

    struct TestStreamingSynapse {
        name_val: String,
        timeout_val: f64,
        dendrite_info: TerminalInfo,
        axon_info: TerminalInfo,
        computed_hash: String,
        total: u64,
        header: u64,
    }

    impl Synapse for TestStreamingSynapse {
        type Output = String;

        fn name(&self) -> &str {
            &self.name_val
        }
        fn timeout(&self) -> f64 {
            self.timeout_val
        }
        fn set_timeout(&mut self, t: f64) {
            self.timeout_val = t;
        }
        fn dendrite(&self) -> &TerminalInfo {
            &self.dendrite_info
        }
        fn set_dendrite(&mut self, info: TerminalInfo) {
            self.dendrite_info = info;
        }
        fn axon(&self) -> &TerminalInfo {
            &self.axon_info
        }
        fn set_axon(&mut self, info: TerminalInfo) {
            self.axon_info = info;
        }
        fn computed_body_hash(&self) -> &str {
            &self.computed_hash
        }
        fn set_computed_body_hash(&mut self, h: String) {
            self.computed_hash = h;
        }
        fn total_size(&self) -> u64 {
            self.total
        }
        fn set_total_size(&mut self, s: u64) {
            self.total = s;
        }
        fn header_size(&self) -> u64 {
            self.header
        }
        fn set_header_size(&mut self, s: u64) {
            self.header = s;
        }
    }

    impl StreamingSynapse for TestStreamingSynapse {
        type StreamItem = String;

        fn process_chunk(chunk: &[u8]) -> Result<String, SynapseError> {
            String::from_utf8(chunk.to_vec())
                .map_err(|e| SynapseError::DeserializationFailed(e.to_string()))
        }
    }

    #[test]
    fn process_chunk_valid_utf8() {
        let chunk = b"hello streaming world";
        let result = TestStreamingSynapse::process_chunk(chunk);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "hello streaming world");
    }

    #[test]
    fn process_chunk_invalid_utf8() {
        let chunk = &[0x80, 0x81, 0x82];
        let result = TestStreamingSynapse::process_chunk(chunk);
        assert!(result.is_err());
    }

    #[test]
    fn streaming_synapse_creation() {
        let synapse = TestStreamingSynapse {
            name_val: "StreamingTest".to_string(),
            timeout_val: 30.0,
            dendrite_info: TerminalInfo::default(),
            axon_info: TerminalInfo::default(),
            computed_hash: String::new(),
            total: 0,
            header: 0,
        };
        assert_eq!(synapse.name(), "StreamingTest");
        assert_eq!(synapse.timeout(), 30.0);
        assert_eq!(synapse.computed_body_hash(), "");
    }

    #[test]
    fn process_chunk_empty_data() {
        let chunk = b"";
        let result = TestStreamingSynapse::process_chunk(chunk);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "");
    }

    #[test]
    fn process_chunk_multibyte_utf8() {
        let chunk = "日本語".as_bytes();
        let result = TestStreamingSynapse::process_chunk(chunk);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "日本語");
    }
}
