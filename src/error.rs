//! Error types for the Bittensor SDK

use thiserror::Error;

/// Result type alias using the SDK's Error type
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for the Bittensor SDK
#[derive(Error, Debug)]
pub enum Error {
    /// Chain connection errors
    #[error("Chain connection error: {0}")]
    Connection(String),

    /// Chain query errors
    #[error("Chain query error: {0}")]
    Query(String),

    /// Decoding errors
    #[error("Decoding error: {0}")]
    Decode(String),

    /// Encoding errors
    #[error("Encoding error: {0}")]
    Encode(String),

    /// Invalid address errors
    #[error("Invalid SS58 address: {0}")]
    InvalidAddress(String),

    /// Subnet not found
    #[error("Subnet {0} not found")]
    SubnetNotFound(u16),

    /// Neuron not found
    #[error("Neuron with UID {uid} not found on subnet {netuid}")]
    NeuronNotFound { netuid: u16, uid: u16 },

    /// Hotkey not registered
    #[error("Hotkey {hotkey} not registered on subnet {netuid}")]
    HotkeyNotRegistered { netuid: u16, hotkey: String },

    /// Metagraph sync error
    #[error("Metagraph sync error: {0}")]
    MetagraphSync(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Network error
    #[error("Network error: {0}")]
    Network(String),

    /// Timeout error
    #[error("Operation timed out: {0}")]
    Timeout(String),

    /// Invalid parameter
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// Not implemented
    #[error("Not implemented: {0}")]
    NotImplemented(String),

    /// Generic error wrapper
    #[error("{0}")]
    Other(String),
}

impl Error {
    pub fn connection(msg: impl Into<String>) -> Self {
        Error::Connection(msg.into())
    }

    pub fn query(msg: impl Into<String>) -> Self {
        Error::Query(msg.into())
    }

    pub fn decode(msg: impl Into<String>) -> Self {
        Error::Decode(msg.into())
    }

    pub fn encode(msg: impl Into<String>) -> Self {
        Error::Encode(msg.into())
    }

    pub fn invalid_address(msg: impl Into<String>) -> Self {
        Error::InvalidAddress(msg.into())
    }

    pub fn metagraph_sync(msg: impl Into<String>) -> Self {
        Error::MetagraphSync(msg.into())
    }

    pub fn other(msg: impl Into<String>) -> Self {
        Error::Other(msg.into())
    }
}

impl From<anyhow::Error> for Error {
    fn from(e: anyhow::Error) -> Self {
        Error::Other(e.to_string())
    }
}

impl From<subxt::Error> for Error {
    fn from(e: subxt::Error) -> Self {
        Error::Query(e.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Self {
        Error::Serialization(e.to_string())
    }
}
