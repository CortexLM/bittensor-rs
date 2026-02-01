//! Custom log formatters for Bittensor SDK
//!
//! Provides log formatting that matches Python SDK output style.

use std::fmt;
use tracing::{Event, Level, Subscriber};
use tracing_subscriber::fmt::format::{self, FormatEvent, FormatFields};
use tracing_subscriber::fmt::FmtContext;
use tracing_subscriber::registry::LookupSpan;

/// Bittensor-style log formatter matching Python SDK output format.
///
/// Output format: `YYYY-MM-DD HH:MM:SS | LEVEL | target | message`
///
/// # Example Output
/// ```text
/// 2024-01-15 10:30:45 | INFO  | bittensor::subtensor | Connected to network
/// 2024-01-15 10:30:46 | DEBUG | bittensor::metagraph | Syncing metagraph for netuid 1
/// ```
pub struct BittensorFormatter;

impl<S, N> FormatEvent<S, N> for BittensorFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: format::Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        // Format: YYYY-MM-DD HH:MM:SS | LEVEL | target | message
        let now = chrono::Local::now();
        let level = event.metadata().level();
        let target = event.metadata().target();

        // Use colored level representation for better readability
        let level_str = format_level(*level);

        write!(
            writer,
            "{} | {} | {} | ",
            now.format("%Y-%m-%d %H:%M:%S"),
            level_str,
            target
        )?;

        ctx.field_format().format_fields(writer.by_ref(), event)?;
        writeln!(writer)
    }
}

/// Format log level with fixed width for alignment
fn format_level(level: Level) -> &'static str {
    match level {
        Level::TRACE => "TRACE",
        Level::DEBUG => "DEBUG",
        Level::INFO => "INFO ",
        Level::WARN => "WARN ",
        Level::ERROR => "ERROR",
    }
}

/// JSON log formatter for structured logging.
///
/// Produces newline-delimited JSON (NDJSON) suitable for log aggregation systems.
///
/// # Example Output
/// ```json
/// {"timestamp":"2024-01-15T10:30:45.123456Z","level":"INFO","target":"bittensor::subtensor","message":"Connected to network"}
/// ```
pub struct JsonFormatter;

impl<S, N> FormatEvent<S, N> for JsonFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        _ctx: &FmtContext<'_, S, N>,
        mut writer: format::Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        let now = chrono::Utc::now();
        let level = event.metadata().level();
        let target = event.metadata().target();

        write!(
            writer,
            "{{\"timestamp\":\"{}\",\"level\":\"{}\",\"target\":\"{}\",\"message\":\"",
            now.format("%Y-%m-%dT%H:%M:%S%.6fZ"),
            level,
            escape_json_string(target)
        )?;

        // Capture fields into a string buffer for JSON escaping
        let mut field_visitor = JsonFieldVisitor::new();
        event.record(&mut field_visitor);

        write!(writer, "{}", escape_json_string(&field_visitor.message))?;
        write!(writer, "\"")?;

        // Add additional fields if present
        if !field_visitor.fields.is_empty() {
            for (key, value) in &field_visitor.fields {
                write!(
                    writer,
                    ",\"{}\":\"{}\"",
                    escape_json_string(key),
                    escape_json_string(value)
                )?;
            }
        }

        writeln!(writer, "}}")
    }
}

/// Visitor to extract fields from tracing events for JSON formatting
struct JsonFieldVisitor {
    message: String,
    fields: Vec<(String, String)>,
}

impl JsonFieldVisitor {
    fn new() -> Self {
        Self {
            message: String::new(),
            fields: Vec::new(),
        }
    }
}

impl tracing::field::Visit for JsonFieldVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn fmt::Debug) {
        if field.name() == "message" {
            self.message = format!("{:?}", value);
            // Remove surrounding quotes if present
            if self.message.starts_with('"') && self.message.ends_with('"') {
                self.message = self.message[1..self.message.len() - 1].to_string();
            }
        } else {
            self.fields
                .push((field.name().to_string(), format!("{:?}", value)));
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        if field.name() == "message" {
            self.message = value.to_string();
        } else {
            self.fields
                .push((field.name().to_string(), value.to_string()));
        }
    }

    fn record_i64(&mut self, field: &tracing::field::Field, value: i64) {
        self.fields
            .push((field.name().to_string(), value.to_string()));
    }

    fn record_u64(&mut self, field: &tracing::field::Field, value: u64) {
        self.fields
            .push((field.name().to_string(), value.to_string()));
    }

    fn record_bool(&mut self, field: &tracing::field::Field, value: bool) {
        self.fields
            .push((field.name().to_string(), value.to_string()));
    }
}

/// Escape special characters for JSON string values
fn escape_json_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            c if c.is_control() => {
                result.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => result.push(c),
        }
    }
    result
}

/// Compact log formatter with minimal output.
///
/// Output format: `[LEVEL] message`
///
/// Useful for development and quick debugging where timestamps aren't needed.
///
/// # Example Output
/// ```text
/// [INFO] Connected to network
/// [DEBUG] Syncing metagraph
/// ```
pub struct CompactFormatter;

impl<S, N> FormatEvent<S, N> for CompactFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: format::Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        let level = event.metadata().level();

        write!(writer, "[{}] ", format_level(*level).trim())?;

        ctx.field_format().format_fields(writer.by_ref(), event)?;
        writeln!(writer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_level() {
        assert_eq!(format_level(Level::TRACE), "TRACE");
        assert_eq!(format_level(Level::DEBUG), "DEBUG");
        assert_eq!(format_level(Level::INFO), "INFO ");
        assert_eq!(format_level(Level::WARN), "WARN ");
        assert_eq!(format_level(Level::ERROR), "ERROR");
    }

    #[test]
    fn test_escape_json_string() {
        assert_eq!(escape_json_string("hello"), "hello");
        assert_eq!(escape_json_string("hello\"world"), "hello\\\"world");
        assert_eq!(escape_json_string("line1\nline2"), "line1\\nline2");
        assert_eq!(escape_json_string("path\\to\\file"), "path\\\\to\\\\file");
        assert_eq!(escape_json_string("tab\there"), "tab\\there");
    }

    #[test]
    fn test_json_field_visitor() {
        let visitor = JsonFieldVisitor::new();
        assert!(visitor.message.is_empty());
        assert!(visitor.fields.is_empty());
    }
}
