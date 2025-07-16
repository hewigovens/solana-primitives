//! RPC client functionality

pub mod client;
pub mod methods;
pub mod types;

#[cfg(feature = "websocket")]
pub mod websocket;

pub use client::RpcClient;
pub use types::*;

#[cfg(feature = "websocket")]
pub use websocket::WebSocketClient;