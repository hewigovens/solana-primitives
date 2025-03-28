use std::fmt;

/// A custom error type for Solana operations
#[derive(Debug)]
pub enum SolanaError {
    /// Invalid public key
    InvalidPubkey,
    /// Invalid signature
    InvalidSignature,
    /// Invalid signature index
    InvalidSignatureIndex,
    /// Invalid instruction data
    InvalidInstructionData,
    /// Invalid message
    InvalidMessage,
    /// Invalid transaction
    InvalidTransaction,
    /// RPC error
    #[cfg(feature = "jsonrpc")]
    RpcError(String),
}

impl fmt::Display for SolanaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SolanaError::InvalidPubkey => write!(f, "Invalid public key"),
            SolanaError::InvalidSignature => write!(f, "Invalid signature"),
            SolanaError::InvalidSignatureIndex => write!(f, "Invalid signature index"),
            SolanaError::InvalidInstructionData => write!(f, "Invalid instruction data"),
            SolanaError::InvalidMessage => write!(f, "Invalid message"),
            SolanaError::InvalidTransaction => write!(f, "Invalid transaction"),
            #[cfg(feature = "jsonrpc")]
            SolanaError::RpcError(msg) => write!(f, "RPC error: {}", msg),
        }
    }
}

impl std::error::Error for SolanaError {}

/// A type alias for Result with SolanaError
pub type Result<T> = std::result::Result<T, SolanaError>;
