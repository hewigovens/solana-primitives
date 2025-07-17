pub mod builder;
pub mod crypto;
pub mod error;
pub mod instructions;
#[cfg(feature = "rpc")]
pub mod rpc;
pub mod types;
pub mod utils;
pub mod borsh_helpers;
pub mod short_vec;
#[cfg(feature = "testing")]
pub mod testing;

// Re-export enhanced builders
pub use builder::{InstructionBuilder, TransactionBuilder, VersionedTransactionBuilder, TransactionVersion};

// Re-export crypto functionality
pub use crypto::*;

// Re-export error types
pub use error::{Result, SolanaError};

// Re-export instruction builders
pub use instructions::*;

// Re-export core types
pub use types::*;

// Re-export utilities
pub use utils::*;

// Re-export legacy helpers for backward compatibility
pub use borsh_helpers::{compact_array_to_bytes, bytes_to_compact_array};
pub use short_vec::{ShortU16, ShortVec, encode_length_to_compact_u16_bytes, decode_compact_u16_len};

// Re-export RPC client
#[cfg(feature = "rpc")]
pub use rpc::RpcClient;

// Re-export testing utilities
#[cfg(feature = "testing")]
pub use testing::*;
