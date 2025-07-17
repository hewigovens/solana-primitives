//! RPC client for Solana

pub mod client;
pub mod request;
pub mod response;
pub mod methods;
pub mod types;

// Re-export the client for easier access
pub use client::RpcClient;
pub use types::*;
