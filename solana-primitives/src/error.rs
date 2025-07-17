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
    
    // Enhanced error types
    #[error("RPC error: {0}")]
    Rpc(#[from] RpcError),
    
    #[error("Transaction error: {0}")]
    Transaction(String),
    
    #[error("Serialization error: {0}")]
    Serialization(String),
    
    #[error("Cryptographic error: {0}")]
    Crypto(String),
    
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    
    #[error("WebSocket error: {0}")]
    WebSocket(String),
    
    /// Legacy RPC error for backward compatibility
    #[cfg(feature = "rpc")]
    #[error("RPC error: {0}")]
    RpcError(String),
}

/// RPC-specific error types
#[derive(Debug, Error)]
pub enum RpcError {
    #[error("Invalid params: {0}")]
    InvalidParams(String),
    
    #[error("Method not found: {0}")]
    MethodNotFound(String),
    
    #[error("Rate limited")]
    RateLimited,
    
    #[error("Node unhealthy")]
    NodeUnhealthy,
    
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
    
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
}

/// A type alias for Result with SolanaError
pub type Result<T> = std::result::Result<T, SolanaError>;
