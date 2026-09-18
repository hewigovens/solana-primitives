//! Solana transaction primitives without the Solana SDK.
//!
//! Build, sign, serialize, and parse legacy, v0, and v1 ([SIMD-0385]) transactions.
//! Byte encodings are checked against the Solana SDK with golden vectors.
//!
//! ```
//! # #[cfg(feature = "signing")]
//! # fn main() -> solana_primitives::Result<()> {
//! use solana_primitives::{
//!     Pubkey, TransactionBuilder, TransactionConfig, VersionedTransaction, get_public_key,
//!     instructions::{compute_budget, system},
//! };
//!
//! let private_key = [7u8; 32];
//! let payer = Pubkey::new(get_public_key(&private_key)?);
//! let recipient = Pubkey::from_str_const("4fYNw3dojWmQ4dXtSGE9epjRGy9uFrCRgbvGgQBNZCQF");
//! let recent_blockhash = [1u8; 32]; // from `getLatestBlockhash`
//!
//! // v1: compute budget and fee live in the message config. Unset limits are zero.
//! let mut builder = TransactionBuilder::new(payer, recent_blockhash);
//! builder.add_instruction(system::transfer(&payer, &recipient, 1_000_000));
//! let config = TransactionConfig::new()
//!     .with_compute_unit_limit(1_000)
//!     .with_loaded_accounts_data_size_limit(64 * 1024)
//!     .with_priority_fee(5_000); // total lamports
//! let mut v1 = builder.build_v1(config)?;
//! v1.sign(&[&private_key])?;
//! let wire = v1.serialize()?;
//! assert_eq!(wire[0], 0x81);
//!
//! // Legacy/v0: compute budget is set with instructions.
//! builder.add_instruction(compute_budget::set_compute_unit_price(10_000));
//! let mut legacy = builder.build()?;
//! legacy.sign(&[&private_key])?;
//!
//! // Parsing is strict and sanitizes, for every version.
//! let parsed = VersionedTransaction::deserialize(&wire)?;
//! assert_eq!(parsed.priority_fee_lamports(), Some(5_000));
//! parsed.verify()?;
//! # Ok(())
//! # }
//! # #[cfg(not(feature = "signing"))]
//! # fn main() {}
//! ```
//!
//! Signing somewhere else (a hardware wallet, a remote signer) works without the
//! `signing` feature: sign [`VersionedTransaction::serialize_message`] and attach
//! the result with [`VersionedTransaction::add_signature`].
//!
//! # Features
//!
//! - `signing` (default): Ed25519 key derivation, signing, and verification
//!   through `ed25519-dalek`. Without it the crate depends only on `bs58`, `sha2`,
//!   and `curve25519-dalek` (for program-derived addresses).
//! - `serde`: Serde for the Rust data model (pubkeys and signatures as base58).
//!   This is not the wire format; use `serialize`/`deserialize` for that.
//! - `borsh`: Borsh for [`Pubkey`] and [`SignatureBytes`].
//!
//! [SIMD-0385]: https://github.com/solana-foundation/solana-improvement-documents/blob/main/proposals/0385-transaction-v1.md

pub mod builder;
mod compiler;
pub mod crypto;
pub mod error;
pub mod instructions;
mod short_vec;
pub mod types;
mod wire;

#[cfg(test)]
mod test_utils;

pub use builder::{InstructionBuilder, InstructionDataBuilder, TransactionBuilder};
pub use crypto::*;
pub use error::{CompileError, DecodeError, EncodeError, Result, SanitizeError, SolanaError};
pub use instructions::*;
pub use types::*;
