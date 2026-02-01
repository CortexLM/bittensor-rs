//! Logging system for Bittensor SDK
//!
//! Provides structured logging matching the Python SDK format with support for
//! multiple output formats (text, JSON, compact) and file logging.
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use bittensor_rs::logging::{init_default_logging, init_logging, LoggingConfig, LogFormat};
//!
//! // Initialize with defaults (INFO level, text format)
//! init_default_logging();
//!
//! // Or configure logging explicitly
//! let config = LoggingConfig {
//!     debug: true,
//!     format: LogFormat::Json,
//!     ..Default::default()
//! };
//! init_logging(&config);
//! ```
//!
//! # Logging Macros
//!
//! Use the provided macros for consistent logging:
//!
//! ```rust,ignore
//! bt_info!("Connected to network");
//! bt_debug!(netuid = 1, "Syncing metagraph");
//! bt_warn!("Connection unstable");
//! bt_error!(error = %e, "Failed to submit extrinsic");
//! ```

pub mod format;

use std::io;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Once;

use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;

use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{fmt, EnvFilter};

pub use format::{BittensorFormatter, CompactFormatter, JsonFormatter};

/// Static initialization guard to ensure logging is only initialized once
static INIT: Once = Once::new();

/// Flag indicating whether logging has been initialized
static INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Guard for non-blocking file writer (must be kept alive for duration of program)
static mut FILE_GUARD: Option<WorkerGuard> = None;

/// Log output format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LogFormat {
    /// Human-readable text format with timestamps
    /// Format: `YYYY-MM-DD HH:MM:SS | LEVEL | target | message`
    #[default]
    Text,
    /// JSON format for structured logging and log aggregation
    Json,
    /// Compact format for development: `[LEVEL] message`
    Compact,
}

impl std::fmt::Display for LogFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogFormat::Text => write!(f, "text"),
            LogFormat::Json => write!(f, "json"),
            LogFormat::Compact => write!(f, "compact"),
        }
    }
}

impl std::str::FromStr for LogFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "text" => Ok(LogFormat::Text),
            "json" => Ok(LogFormat::Json),
            "compact" => Ok(LogFormat::Compact),
            _ => Err(format!(
                "Invalid log format '{}'. Valid options: text, json, compact",
                s
            )),
        }
    }
}

/// Logging configuration for the Bittensor SDK
///
/// This configuration extends the basic `config::LoggingConfig` with format options
/// and provides the initialization logic for the tracing subscriber.
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    /// Enable debug-level logging (sets minimum level to DEBUG)
    pub debug: bool,
    /// Enable trace-level logging (sets minimum level to TRACE, overrides debug)
    pub trace: bool,
    /// Enable logging to file in addition to stdout
    pub record_log: bool,
    /// Directory for log files (supports ~ for home directory)
    pub logging_dir: String,
    /// Output format for log messages
    pub format: LogFormat,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            debug: false,
            trace: false,
            record_log: false,
            logging_dir: "~/.bittensor/logs".to_string(),
            format: LogFormat::Text,
        }
    }
}

impl LoggingConfig {
    /// Create a new LoggingConfig with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable debug logging
    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    /// Enable trace logging
    pub fn with_trace(mut self, trace: bool) -> Self {
        self.trace = trace;
        self
    }

    /// Enable file logging
    pub fn with_file_logging(mut self, enabled: bool) -> Self {
        self.record_log = enabled;
        self
    }

    /// Set the logging directory
    pub fn with_logging_dir(mut self, dir: impl Into<String>) -> Self {
        self.logging_dir = dir.into();
        self
    }

    /// Set the log format
    pub fn with_format(mut self, format: LogFormat) -> Self {
        self.format = format;
        self
    }

    /// Load configuration from environment variables
    ///
    /// Supported environment variables:
    /// - `BITTENSOR_LOG_LEVEL`: Set log level (trace, debug, info, warn, error)
    /// - `BITTENSOR_LOG_FORMAT`: Set format (text, json, compact)
    /// - `BITTENSOR_LOG_DIR`: Set logging directory
    /// - `BITTENSOR_DEBUG`: Enable debug mode (any value)
    /// - `RUST_LOG`: Standard tracing filter (takes precedence if set)
    pub fn from_env() -> Self {
        let mut config = Self::default();

        if std::env::var("BITTENSOR_DEBUG").is_ok() || std::env::var("BITTENSOR_TRACE").is_ok() {
            config.debug = true;
        }

        if std::env::var("BITTENSOR_TRACE").is_ok() {
            config.trace = true;
        }

        if let Ok(format) = std::env::var("BITTENSOR_LOG_FORMAT") {
            if let Ok(f) = format.parse() {
                config.format = f;
            }
        }

        if let Ok(dir) = std::env::var("BITTENSOR_LOG_DIR") {
            config.logging_dir = dir;
            config.record_log = true;
        }

        config
    }

    /// Get the effective log level based on configuration
    fn get_level(&self) -> Level {
        if self.trace {
            Level::TRACE
        } else if self.debug {
            Level::DEBUG
        } else {
            Level::INFO
        }
    }

    /// Expand ~ to home directory in paths
    fn expand_path(&self) -> PathBuf {
        let path = &self.logging_dir;
        if let Some(stripped) = path.strip_prefix("~/") {
            if let Some(home) = dirs::home_dir() {
                return home.join(stripped);
            }
        }
        PathBuf::from(path)
    }
}

/// Initialize the logging system with the given configuration.
///
/// This function can only be called once; subsequent calls will be ignored.
/// The logging system uses the `tracing` crate internally and integrates with
/// the standard Rust logging ecosystem.
///
/// # Arguments
///
/// * `config` - The logging configuration specifying level, format, and output options
///
/// # Example
///
/// ```rust,no_run
/// use bittensor_rs::logging::{init_logging, LoggingConfig, LogFormat};
///
/// let config = LoggingConfig {
///     debug: true,
///     format: LogFormat::Text,
///     ..Default::default()
/// };
/// init_logging(&config);
/// ```
pub fn init_logging(config: &LoggingConfig) {
    INIT.call_once(|| {
        init_logging_internal(config);
        INITIALIZED.store(true, Ordering::SeqCst);
    });
}

/// Initialize logging with default configuration (INFO level, text format).
///
/// This is a convenience function for quick setup. For production use,
/// consider using `init_logging` with explicit configuration.
///
/// # Example
///
/// ```rust,no_run
/// use bittensor_rs::logging::init_default_logging;
///
/// init_default_logging();
/// ```
pub fn init_default_logging() {
    init_logging(&LoggingConfig::default());
}

/// Check if logging has been initialized
pub fn is_initialized() -> bool {
    INITIALIZED.load(Ordering::SeqCst)
}

/// Internal initialization logic
fn init_logging_internal(config: &LoggingConfig) {
    // Build environment filter
    // Allow RUST_LOG to override, otherwise use config-based level
    let env_filter = if std::env::var("RUST_LOG").is_ok() {
        EnvFilter::from_default_env()
    } else {
        let level = config.get_level();
        EnvFilter::new(format!("{},hyper=warn,reqwest=warn,h2=warn", level))
    };

    // Setup file appender if configured
    let file_appender = if config.record_log {
        let log_dir = config.expand_path();
        if let Err(e) = std::fs::create_dir_all(&log_dir) {
            eprintln!(
                "Warning: Failed to create log directory {:?}: {}",
                log_dir, e
            );
            None
        } else {
            let file_appender = tracing_appender::rolling::daily(&log_dir, "bittensor.log");
            let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
            // Store the guard to keep the writer alive
            // SAFETY: This is only called once via Once::call_once, so there's no race condition
            unsafe {
                FILE_GUARD = Some(guard);
            }
            Some(non_blocking)
        }
    } else {
        None
    };

    // Build and initialize the subscriber based on format
    match config.format {
        LogFormat::Text => {
            if let Some(file_writer) = file_appender {
                let subscriber = tracing_subscriber::registry()
                    .with(env_filter)
                    .with(
                        fmt::layer()
                            .event_format(BittensorFormatter)
                            .with_writer(io::stdout),
                    )
                    .with(
                        fmt::layer()
                            .event_format(BittensorFormatter)
                            .with_writer(file_writer)
                            .with_ansi(false),
                    );
                subscriber.init();
            } else {
                let subscriber = tracing_subscriber::registry().with(env_filter).with(
                    fmt::layer()
                        .event_format(BittensorFormatter)
                        .with_writer(io::stdout),
                );
                subscriber.init();
            }
        }
        LogFormat::Json => {
            if let Some(file_writer) = file_appender {
                let subscriber = tracing_subscriber::registry()
                    .with(env_filter)
                    .with(fmt::layer().json().with_writer(io::stdout))
                    .with(
                        fmt::layer()
                            .json()
                            .with_writer(file_writer)
                            .with_ansi(false),
                    );
                subscriber.init();
            } else {
                let subscriber = tracing_subscriber::registry()
                    .with(env_filter)
                    .with(fmt::layer().json().with_writer(io::stdout));
                subscriber.init();
            }
        }
        LogFormat::Compact => {
            if let Some(file_writer) = file_appender {
                let subscriber = tracing_subscriber::registry()
                    .with(env_filter)
                    .with(
                        fmt::layer()
                            .event_format(CompactFormatter)
                            .with_writer(io::stdout),
                    )
                    .with(
                        fmt::layer()
                            .event_format(CompactFormatter)
                            .with_writer(file_writer)
                            .with_ansi(false),
                    );
                subscriber.init();
            } else {
                let subscriber = tracing_subscriber::registry().with(env_filter).with(
                    fmt::layer()
                        .event_format(CompactFormatter)
                        .with_writer(io::stdout),
                );
                subscriber.init();
            }
        }
    }
}

/// Log a debug message.
///
/// This macro wraps `tracing::debug!` for consistent usage across the SDK.
///
/// # Example
///
/// ```rust,ignore
/// bt_debug!("Processing request");
/// bt_debug!(netuid = 1, hotkey = %hotkey, "Syncing neuron");
/// ```
#[macro_export]
macro_rules! bt_debug {
    ($($arg:tt)*) => {
        tracing::debug!($($arg)*)
    };
}

/// Log an info message.
///
/// This macro wraps `tracing::info!` for consistent usage across the SDK.
///
/// # Example
///
/// ```rust,ignore
/// bt_info!("Connected to network");
/// bt_info!(network = "finney", "Initialized subtensor");
/// ```
#[macro_export]
macro_rules! bt_info {
    ($($arg:tt)*) => {
        tracing::info!($($arg)*)
    };
}

/// Log a warning message.
///
/// This macro wraps `tracing::warn!` for consistent usage across the SDK.
///
/// # Example
///
/// ```rust,ignore
/// bt_warn!("Connection unstable");
/// bt_warn!(retries = 3, "Request failed, retrying");
/// ```
#[macro_export]
macro_rules! bt_warn {
    ($($arg:tt)*) => {
        tracing::warn!($($arg)*)
    };
}

/// Log an error message.
///
/// This macro wraps `tracing::error!` for consistent usage across the SDK.
///
/// # Example
///
/// ```rust,ignore
/// bt_error!("Failed to connect");
/// bt_error!(error = %e, "Transaction failed");
/// ```
#[macro_export]
macro_rules! bt_error {
    ($($arg:tt)*) => {
        tracing::error!($($arg)*)
    };
}

/// Log a trace message.
///
/// This macro wraps `tracing::trace!` for consistent usage across the SDK.
/// Trace-level messages are only emitted when trace logging is enabled.
///
/// # Example
///
/// ```rust,ignore
/// bt_trace!("Entering function");
/// bt_trace!(bytes = data.len(), "Received data");
/// ```
#[macro_export]
macro_rules! bt_trace {
    ($($arg:tt)*) => {
        tracing::trace!($($arg)*)
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_logging_config_default() {
        let config = LoggingConfig::default();
        assert!(!config.debug);
        assert!(!config.trace);
        assert!(!config.record_log);
        assert_eq!(config.logging_dir, "~/.bittensor/logs");
        assert_eq!(config.format, LogFormat::Text);
    }

    #[test]
    fn test_logging_config_builder() {
        let config = LoggingConfig::new()
            .with_debug(true)
            .with_format(LogFormat::Json)
            .with_logging_dir("/tmp/logs");

        assert!(config.debug);
        assert_eq!(config.format, LogFormat::Json);
        assert_eq!(config.logging_dir, "/tmp/logs");
    }

    #[test]
    fn test_log_format_display() {
        assert_eq!(format!("{}", LogFormat::Text), "text");
        assert_eq!(format!("{}", LogFormat::Json), "json");
        assert_eq!(format!("{}", LogFormat::Compact), "compact");
    }

    #[test]
    fn test_log_format_from_str() {
        assert_eq!("text".parse::<LogFormat>().unwrap(), LogFormat::Text);
        assert_eq!("json".parse::<LogFormat>().unwrap(), LogFormat::Json);
        assert_eq!("compact".parse::<LogFormat>().unwrap(), LogFormat::Compact);
        assert_eq!("TEXT".parse::<LogFormat>().unwrap(), LogFormat::Text);
        assert!("invalid".parse::<LogFormat>().is_err());
    }

    #[test]
    fn test_get_level() {
        let config = LoggingConfig::default();
        assert_eq!(config.get_level(), Level::INFO);

        let config = LoggingConfig::default().with_debug(true);
        assert_eq!(config.get_level(), Level::DEBUG);

        let config = LoggingConfig::default().with_trace(true);
        assert_eq!(config.get_level(), Level::TRACE);

        // trace takes precedence over debug
        let config = LoggingConfig::default().with_debug(true).with_trace(true);
        assert_eq!(config.get_level(), Level::TRACE);
    }

    #[test]
    fn test_expand_path_tilde() {
        let config = LoggingConfig::default();
        let path = config.expand_path();
        // Should have expanded ~ to home dir
        assert!(!path.to_string_lossy().starts_with('~'));
    }

    #[test]
    fn test_expand_path_absolute() {
        let config = LoggingConfig::default().with_logging_dir("/var/log/bittensor");
        let path = config.expand_path();
        assert_eq!(path.to_string_lossy(), "/var/log/bittensor");
    }
}
