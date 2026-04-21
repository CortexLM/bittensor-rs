use std::fmt;

/// Core error type for the bittensor-rs SDK.
#[derive(Debug, thiserror::Error)]
pub enum BittensorError {
    /// WebSocket or HTTP RPC failure.
    #[error("RPC error: {0}")]
    Rpc(String),

    /// Signature creation or verification failure.
    #[error("Signing error: {0}")]
    Signing(String),

    /// SCALE codec or serde serialization failure.
    #[error("Codec error: {0}")]
    Codec(String),

    /// Extrinsic submission or finalization failure.
    #[error("Transaction error: {0}")]
    Transaction(String),

    /// Wallet file I/O or decryption failure.
    #[error("Wallet error: {0}")]
    Wallet(String),

    /// Network connectivity or DNS failure.
    #[error("Network error: {0}")]
    Network(String),

    /// Invalid configuration value or missing field.
    #[error("Configuration error: {0}")]
    Config(String),

    /// Balance underflow, overflow, or invalid conversion.
    #[error("Balance error: {0}")]
    Balance(String),

    /// Operation exceeded its deadline.
    #[error("Timeout error: {0}")]
    Timeout(String),

    /// Server-side rate limit exceeded.
    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    /// Authentication or authorization failure.
    #[error("Authentication error: {0}")]
    Authentication(String),

    /// Input validation failure.
    #[error("Validation error: {0}")]
    Validation(String),
}

impl BittensorError {
    /// Classify this error into a retry-oriented category.
    pub fn category(&self) -> ErrorCategory {
        match self {
            Self::Rpc(_) | Self::Network(_) | Self::Timeout(_) => ErrorCategory::Transient,
            Self::RateLimit(_) => ErrorCategory::RateLimit,
            Self::Authentication(_) => ErrorCategory::Auth,
            Self::Config(_) => ErrorCategory::Config,
            Self::Signing(_)
            | Self::Codec(_)
            | Self::Transaction(_)
            | Self::Wallet(_)
            | Self::Balance(_)
            | Self::Validation(_) => ErrorCategory::Permanent,
        }
    }

    /// Returns `true` if the error may succeed on retry.
    pub fn is_retryable(&self) -> bool {
        matches!(self.category(), ErrorCategory::Transient | ErrorCategory::RateLimit)
    }
}

/// Classification of error categories for retry logic.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ErrorCategory {
    /// Transient network or RPC error — worth retrying.
    Transient,
    /// Server rate limit — retry with longer backoff.
    RateLimit,
    /// Authentication failure — do not retry.
    Auth,
    /// Invalid configuration — do not retry.
    Config,
    /// Network-level failure — retry with moderate backoff.
    Network,
    /// Permanent error — do not retry.
    Permanent,
}

impl ErrorCategory {
    /// Return the retry policy for this error category.
    pub fn retry_config(&self) -> RetryConfig {
        match self {
            Self::Transient => RetryConfig {
                max_retries: 3,
                base_delay_ms: 1000,
                max_delay_ms: 30_000,
                backoff_factor: 2.0,
            },
            Self::RateLimit => RetryConfig {
                max_retries: 5,
                base_delay_ms: 5000,
                max_delay_ms: 60_000,
                backoff_factor: 2.0,
            },
            Self::Auth => RetryConfig {
                max_retries: 0,
                base_delay_ms: 0,
                max_delay_ms: 0,
                backoff_factor: 0.0,
            },
            Self::Config => RetryConfig {
                max_retries: 0,
                base_delay_ms: 0,
                max_delay_ms: 0,
                backoff_factor: 0.0,
            },
            Self::Network => RetryConfig {
                max_retries: 3,
                base_delay_ms: 2000,
                max_delay_ms: 30_000,
                backoff_factor: 2.0,
            },
            Self::Permanent => RetryConfig {
                max_retries: 0,
                base_delay_ms: 0,
                max_delay_ms: 0,
                backoff_factor: 0.0,
            },
        }
    }
}

impl fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Transient => write!(f, "transient"),
            Self::RateLimit => write!(f, "rate_limit"),
            Self::Auth => write!(f, "auth"),
            Self::Config => write!(f, "config"),
            Self::Network => write!(f, "network"),
            Self::Permanent => write!(f, "permanent"),
        }
    }
}

/// Retry configuration associated with an error category.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts.
    pub max_retries: u32,
    /// Base delay in milliseconds before the first retry.
    pub base_delay_ms: u64,
    /// Maximum delay in milliseconds between retries.
    pub max_delay_ms: u64,
    /// Exponential backoff multiplier.
    pub backoff_factor: f64,
}

impl RetryConfig {
    /// Compute the delay in milliseconds for the given 1-based attempt number.
    pub fn delay_for_attempt(&self, attempt: u32) -> u64 {
        if attempt == 0 {
            return 0;
        }
        let delay = self.base_delay_ms as f64 * self.backoff_factor.powi(attempt as i32 - 1);
        delay.min(self.max_delay_ms as f64) as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_category_mapping() {
        assert_eq!(BittensorError::Rpc("x".into()).category(), ErrorCategory::Transient);
        assert_eq!(BittensorError::Network("x".into()).category(), ErrorCategory::Transient);
        assert_eq!(BittensorError::Timeout("x".into()).category(), ErrorCategory::Transient);
        assert_eq!(BittensorError::RateLimit("x".into()).category(), ErrorCategory::RateLimit);
        assert_eq!(BittensorError::Authentication("x".into()).category(), ErrorCategory::Auth);
        assert_eq!(BittensorError::Config("x".into()).category(), ErrorCategory::Config);
        assert_eq!(BittensorError::Signing("x".into()).category(), ErrorCategory::Permanent);
        assert_eq!(BittensorError::Codec("x".into()).category(), ErrorCategory::Permanent);
        assert_eq!(BittensorError::Transaction("x".into()).category(), ErrorCategory::Permanent);
        assert_eq!(BittensorError::Wallet("x".into()).category(), ErrorCategory::Permanent);
        assert_eq!(BittensorError::Balance("x".into()).category(), ErrorCategory::Permanent);
        assert_eq!(BittensorError::Validation("x".into()).category(), ErrorCategory::Permanent);
    }

    #[test]
    fn retryable_errors() {
        assert!(BittensorError::Rpc("x".into()).is_retryable());
        assert!(BittensorError::RateLimit("x".into()).is_retryable());
        assert!(BittensorError::Network("x".into()).is_retryable());
        assert!(BittensorError::Timeout("x".into()).is_retryable());
        assert!(!BittensorError::Config("x".into()).is_retryable());
        assert!(!BittensorError::Authentication("x".into()).is_retryable());
        assert!(!BittensorError::Signing("x".into()).is_retryable());
    }

    #[test]
    fn retry_config_transient() {
        let rc = ErrorCategory::Transient.retry_config();
        assert_eq!(rc.max_retries, 3);
        assert_eq!(rc.base_delay_ms, 1000);
        assert_eq!(rc.max_delay_ms, 30_000);
        assert_eq!(rc.backoff_factor, 2.0);
    }

    #[test]
    fn retry_config_rate_limit() {
        let rc = ErrorCategory::RateLimit.retry_config();
        assert_eq!(rc.max_retries, 5);
        assert_eq!(rc.base_delay_ms, 5000);
    }

    #[test]
    fn retry_config_non_retryable() {
        let rc = ErrorCategory::Permanent.retry_config();
        assert_eq!(rc.max_retries, 0);
        assert_eq!(rc.base_delay_ms, 0);
    }

    #[test]
    fn delay_for_attempt_exponential_backoff() {
        let rc = ErrorCategory::Transient.retry_config();
        assert_eq!(rc.delay_for_attempt(0), 0);
        assert_eq!(rc.delay_for_attempt(1), 1000);
        assert_eq!(rc.delay_for_attempt(2), 2000);
        assert_eq!(rc.delay_for_attempt(3), 4000);
        assert_eq!(rc.delay_for_attempt(4), 8000);
    }

    #[test]
    fn delay_capped_at_max() {
        let rc = RetryConfig {
            max_retries: 10,
            base_delay_ms: 1000,
            max_delay_ms: 5000,
            backoff_factor: 3.0,
        };
        assert_eq!(rc.delay_for_attempt(1), 1000);
        assert_eq!(rc.delay_for_attempt(2), 3000);
        assert_eq!(rc.delay_for_attempt(3), 5000);
        assert_eq!(rc.delay_for_attempt(4), 5000);
    }

    #[test]
    fn error_display() {
        let err = BittensorError::Rpc("connection refused".into());
        assert_eq!(format!("{err}"), "RPC error: connection refused");
    }

    #[test]
    fn error_category_display() {
        assert_eq!(format!("{}", ErrorCategory::Transient), "transient");
        assert_eq!(format!("{}", ErrorCategory::RateLimit), "rate_limit");
        assert_eq!(format!("{}", ErrorCategory::Permanent), "permanent");
    }

    #[test]
    fn retry_config_serialization_roundtrip() {
        let rc = ErrorCategory::Transient.retry_config();
        let json = serde_json::to_string(&rc).expect("serialize");
        let deserialized: RetryConfig = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(rc, deserialized);
    }
}
