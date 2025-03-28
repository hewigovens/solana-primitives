use thiserror::Error;

/// A custom error type for Solana operations
#[derive(Debug, Error)]
pub enum SolanaError {
    #[error("Invalid public key")]
    InvalidPubkey,
    #[error("Invalid public key length")]
    InvalidPubkeyLength,
    #[error("Invalid signature")]
    InvalidSignature,
    #[error("Invalid signature length")]
    InvalidSignatureLength,
    #[error("Invalid signature index")]
    InvalidSignatureIndex,
    #[error("Invalid instruction data")]
    InvalidInstructionData,
    #[error("Invalid message")]
    InvalidMessage,
    #[error("Invalid transaction")]
    InvalidTransaction,
    #[error("Serialization error: {0}")]
    SerializationError(String),
    /// RPC error
    #[cfg(feature = "jsonrpc")]
    #[error("RPC error: {0}")]
    RpcError(String),
}

/// A type alias for Result with SolanaError
pub type Result<T> = std::result::Result<T, SolanaError>;
