//! Shared test fixtures.
//!
//! Keys are derived as `sha256(label)` so the same labels can be reproduced
//! in upstream Solana SDK code when generating golden vectors.

use crate::crypto::hash_data;
use crate::types::{AccountMeta, Pubkey};

/// Deterministic pubkey for `label`.
pub fn key(label: &str) -> Pubkey {
    Pubkey::new(hash_data(label.as_bytes()))
}

/// Parse a base58 pubkey literal.
pub fn pubkey(s: &str) -> Pubkey {
    Pubkey::from_base58(s).unwrap()
}

/// Build an [`AccountMeta`] from a base58 key and flags, for comparing
/// against metas printed by upstream helpers.
pub fn meta(s: &str, is_signer: bool, is_writable: bool) -> AccountMeta {
    AccountMeta::new(pubkey(s), is_signer, is_writable)
}
