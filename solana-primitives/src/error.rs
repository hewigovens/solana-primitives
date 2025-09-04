use thiserror::Error;

/// A custom error type for Solana operations
#[derive(Debug, Error)]
pub enum SolanaError {
    #[error("Invalid public key: {0}")]
    InvalidPubkey(String),
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),
    #[error("Invalid instruction data")]
    InvalidInstructionData,
    #[error("Invalid message")]
    InvalidMessage,
    #[error("Invalid transaction")]
    InvalidTransaction,
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Deserialization error: {0}")]
    DeserializationError(String),
    #[error("{0}")]
    GenericError(String),
}

impl From<&str> for SolanaError {
    fn from(s: &str) -> Self {
        SolanaError::GenericError(s.to_string())
    }
}

impl From<String> for SolanaError {
    fn from(s: String) -> Self {
        SolanaError::GenericError(s)
    }
}

/// A type alias for Result with SolanaError
pub type Result<T> = std::result::Result<T, SolanaError>;
