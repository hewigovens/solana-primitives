//! Transaction v1 ([SIMD-0385]).
//!
//! A v1 transaction is `0x81 || message || signatures`, with exactly
//! `num_required_signatures` signatures and no length prefix. The message is:
//!
//! ```text
//! MessageHeader                 3 x u8
//! TransactionConfigMask         u32 LE
//! LifetimeSpecifier             [u8; 32]
//! NumInstructions               u8
//! NumAddresses                  u8
//! Addresses                     NumAddresses x [u8; 32]
//! ConfigValues                  one value per mask field, in bit order
//! InstructionHeaders            NumInstructions x (program index u8, account count u8, data length u16 LE)
//! InstructionPayloads           per instruction: account indexes, then data
//! ```
//!
//! There are no address lookup tables, and compute budget settings live in
//! [`TransactionConfig`] instead of Compute Budget instructions (which the
//! runtime ignores for v1).
//!
//! [SIMD-0385]: https://github.com/solana-foundation/solana-improvement-documents/blob/main/proposals/0385-transaction-v1.md

use crate::compiler::{CompiledKeys, compile_instructions};
use crate::error::{Result, SanitizeError};
use crate::types::{CompiledInstruction, Instruction, MessageHeader, Pubkey};
use crate::wire;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// First byte of a v1 message and transaction (`0x80 | 1`).
pub const VERSION_PREFIX: u8 = 0x81;
/// Maximum serialized size of a v1 transaction in bytes.
pub const MAX_TRANSACTION_SIZE: usize = 4096;
/// Maximum number of account addresses in a v1 message.
pub const MAX_ADDRESSES: usize = 64;
/// Maximum number of instructions in a v1 message.
pub const MAX_INSTRUCTIONS: usize = 64;
/// Maximum number of signatures in a v1 transaction.
pub const MAX_SIGNATURES: usize = 12;
/// Smallest heap size a v1 transaction may request (32 KiB).
pub const MIN_HEAP_SIZE: u32 = 32 * 1024;
/// Largest heap size a v1 transaction may request (256 KiB).
pub const MAX_HEAP_SIZE: u32 = 256 * 1024;
/// Heap size used when a v1 transaction requests none.
pub const DEFAULT_HEAP_SIZE: u32 = MIN_HEAP_SIZE;

/// Resource requests carried by a v1 message.
///
/// Unset fields fall back to protocol minimums, not the legacy/v0 defaults:
/// a priority fee of 0, a compute unit limit of 0, a loaded accounts data
/// size limit of 0, and a heap of [`DEFAULT_HEAP_SIZE`]. Set the compute unit
/// limit explicitly or the transaction cannot execute.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TransactionConfig {
    /// Total priority fee in lamports (not micro-lamports per compute unit).
    pub priority_fee: Option<u64>,
    /// Maximum compute units.
    pub compute_unit_limit: Option<u32>,
    /// Maximum bytes of account data the transaction may load.
    pub loaded_accounts_data_size_limit: Option<u32>,
    /// Heap size in bytes: a multiple of 1024 within 32..=256 KiB.
    pub heap_size: Option<u32>,
}

impl TransactionConfig {
    /// A config with no requests.
    pub const fn new() -> Self {
        Self {
            priority_fee: None,
            compute_unit_limit: None,
            loaded_accounts_data_size_limit: None,
            heap_size: None,
        }
    }

    #[must_use]
    pub const fn with_priority_fee(mut self, lamports: u64) -> Self {
        self.priority_fee = Some(lamports);
        self
    }

    #[must_use]
    pub const fn with_compute_unit_limit(mut self, units: u32) -> Self {
        self.compute_unit_limit = Some(units);
        self
    }

    #[must_use]
    pub const fn with_loaded_accounts_data_size_limit(mut self, bytes: u32) -> Self {
        self.loaded_accounts_data_size_limit = Some(bytes);
        self
    }

    #[must_use]
    pub const fn with_heap_size(mut self, bytes: u32) -> Self {
        self.heap_size = Some(bytes);
        self
    }

    /// The wire mask for the fields that are set.
    pub const fn mask(&self) -> TransactionConfigMask {
        let mut mask = 0;
        if self.priority_fee.is_some() {
            mask |= TransactionConfigMask::PRIORITY_FEE;
        }
        if self.compute_unit_limit.is_some() {
            mask |= TransactionConfigMask::COMPUTE_UNIT_LIMIT;
        }
        if self.loaded_accounts_data_size_limit.is_some() {
            mask |= TransactionConfigMask::LOADED_ACCOUNTS_DATA_SIZE_LIMIT;
        }
        if self.heap_size.is_some() {
            mask |= TransactionConfigMask::HEAP_SIZE;
        }
        TransactionConfigMask(mask)
    }

    fn sanitize(&self) -> std::result::Result<(), SanitizeError> {
        match self.heap_size {
            Some(size)
                if !size.is_multiple_of(1024)
                    || !(MIN_HEAP_SIZE..=MAX_HEAP_SIZE).contains(&size) =>
            {
                Err(SanitizeError::InvalidHeapSize)
            }
            _ => Ok(()),
        }
    }
}

/// Bitmask of the config values present in a v1 message.
///
/// Each bit stands for four bytes of values; the `u64` priority fee uses the
/// bit pair 0–1, and both bits must be set together.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct TransactionConfigMask(pub u32);

impl TransactionConfigMask {
    /// Bits 0 and 1: priority fee (`u64` LE).
    pub const PRIORITY_FEE: u32 = 0b11;
    /// Bit 2: compute unit limit (`u32` LE).
    pub const COMPUTE_UNIT_LIMIT: u32 = 0b100;
    /// Bit 3: loaded accounts data size limit (`u32` LE).
    pub const LOADED_ACCOUNTS_DATA_SIZE_LIMIT: u32 = 0b1000;
    /// Bit 4: heap size (`u32` LE).
    pub const HEAP_SIZE: u32 = 0b1_0000;
    /// Every defined bit.
    pub const KNOWN_BITS: u32 = Self::PRIORITY_FEE
        | Self::COMPUTE_UNIT_LIMIT
        | Self::LOADED_ACCOUNTS_DATA_SIZE_LIMIT
        | Self::HEAP_SIZE;

    /// Whether the mask can be decoded: no unknown bits and no half-set priority fee pair.
    pub const fn is_valid(&self) -> bool {
        let fee_bits = self.0 & Self::PRIORITY_FEE;
        self.0 & !Self::KNOWN_BITS == 0 && (fee_bits == 0 || fee_bits == Self::PRIORITY_FEE)
    }

    pub const fn has_priority_fee(&self) -> bool {
        self.0 & Self::PRIORITY_FEE == Self::PRIORITY_FEE
    }

    pub const fn has_compute_unit_limit(&self) -> bool {
        self.0 & Self::COMPUTE_UNIT_LIMIT != 0
    }

    pub const fn has_loaded_accounts_data_size_limit(&self) -> bool {
        self.0 & Self::LOADED_ACCOUNTS_DATA_SIZE_LIMIT != 0
    }

    pub const fn has_heap_size(&self) -> bool {
        self.0 & Self::HEAP_SIZE != 0
    }
}

/// A v1 message.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageV1 {
    /// The message header, identifying signed and read-only `account_keys`.
    pub header: MessageHeader,
    /// Compute budget and fee requests.
    pub config: TransactionConfig,
    /// A recent blockhash or durable nonce value.
    pub lifetime_specifier: [u8; 32],
    /// All account keys; v1 has no lookup tables.
    pub account_keys: Vec<Pubkey>,
    /// Instructions that will be executed in sequence and committed in one atomic transaction if all succeed.
    pub instructions: Vec<CompiledInstruction>,
}

impl MessageV1 {
    /// Compile `instructions` with `payer` as the fee payer.
    ///
    /// Keys are ordered like the Solana SDK's `v1::Message::try_compile_with_config`.
    /// Compute Budget instructions are compiled as ordinary instructions; set
    /// `config` instead.
    pub fn try_compile(
        payer: &Pubkey,
        instructions: &[Instruction],
        lifetime_specifier: [u8; 32],
        config: TransactionConfig,
    ) -> Result<Self> {
        let (header, account_keys) =
            CompiledKeys::compile(payer, instructions).into_message_components()?;
        let instructions = compile_instructions(instructions, &account_keys, &[])?;
        Ok(Self {
            header,
            config,
            lifetime_specifier,
            account_keys,
            instructions,
        })
    }

    /// The fee payer (first account key).
    pub fn fee_payer(&self) -> Option<&Pubkey> {
        self.account_keys.first()
    }

    /// Serialize to wire bytes, including the `0x81` prefix. These are the bytes that get signed.
    pub fn serialize(&self) -> Result<Vec<u8>> {
        let mut out = Vec::new();
        wire::write_v1_message(&mut out, self)?;
        Ok(out)
    }

    /// Check the SIMD-0385 sanitization rules.
    pub fn sanitize(&self) -> Result<()> {
        let num_keys = self.account_keys.len();
        if usize::from(self.header.num_required_signatures) > MAX_SIGNATURES {
            return Err(SanitizeError::TooManySignatures.into());
        }
        if self.instructions.len() > MAX_INSTRUCTIONS {
            return Err(SanitizeError::TooManyInstructions.into());
        }
        if num_keys > MAX_ADDRESSES {
            return Err(SanitizeError::TooManyAccountKeys.into());
        }
        self.header.sanitize(num_keys)?;
        let unique: HashSet<&Pubkey> = self.account_keys.iter().collect();
        if unique.len() != num_keys {
            return Err(SanitizeError::DuplicateAccountKeys.into());
        }
        self.config.sanitize()?;
        for instruction in &self.instructions {
            if instruction.accounts.len() > usize::from(u8::MAX) {
                return Err(SanitizeError::InstructionAccountsTooLarge.into());
            }
            if instruction.data.len() > usize::from(u16::MAX) {
                return Err(SanitizeError::InstructionDataTooLarge.into());
            }
        }
        crate::types::message::sanitize_instructions(&self.instructions, num_keys, num_keys)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::{DecodeError, SolanaError};
    use crate::types::VersionedMessage;

    fn message() -> MessageV1 {
        MessageV1 {
            header: MessageHeader {
                num_required_signatures: 1,
                num_readonly_signed_accounts: 0,
                num_readonly_unsigned_accounts: 1,
            },
            config: TransactionConfig::new().with_compute_unit_limit(200_000),
            lifetime_specifier: [0xab; 32],
            account_keys: vec![
                Pubkey::new([1; 32]),
                Pubkey::new([2; 32]),
                Pubkey::new([3; 32]),
            ],
            instructions: vec![CompiledInstruction {
                program_id_index: 2,
                accounts: vec![0, 1],
                data: vec![0xde, 0xad],
            }],
        }
    }

    fn sanitize_err(message: MessageV1) -> SanitizeError {
        match message.sanitize() {
            Err(SolanaError::Sanitize(err)) => err,
            other => panic!("expected a sanitize error, got {other:?}"),
        }
    }

    #[test]
    fn mask_rules() {
        assert!(TransactionConfigMask(0).is_valid());
        assert!(TransactionConfigMask(0b1_1111).is_valid());
        assert!(!TransactionConfigMask(0b01).is_valid());
        assert!(!TransactionConfigMask(0b10).is_valid());
        assert!(!TransactionConfigMask(0b10_0000).is_valid());
        assert!(!TransactionConfigMask(1 << 31).is_valid());

        let mask = TransactionConfig::new()
            .with_priority_fee(1)
            .with_heap_size(MIN_HEAP_SIZE)
            .mask();
        assert_eq!(mask, TransactionConfigMask(0b1_0011));
        assert!(mask.has_priority_fee() && mask.has_heap_size());
        assert!(!mask.has_compute_unit_limit() && !mask.has_loaded_accounts_data_size_limit());
    }

    #[test]
    fn byte_layout_matches_simd() {
        let message = MessageV1 {
            config: TransactionConfig::new()
                .with_priority_fee(0x0102_0304_0506_0708)
                .with_heap_size(0x0001_0000),
            ..message()
        };
        let bytes = message.serialize().unwrap();

        let mut expected = vec![0x81, 1, 0, 1];
        expected.extend_from_slice(&0b1_0011u32.to_le_bytes());
        expected.extend_from_slice(&[0xab; 32]);
        expected.extend_from_slice(&[1, 3]);
        expected.extend_from_slice(&[1; 32]);
        expected.extend_from_slice(&[2; 32]);
        expected.extend_from_slice(&[3; 32]);
        // Config values in bit order, then instruction headers, then payloads.
        expected.extend_from_slice(&0x0102_0304_0506_0708u64.to_le_bytes());
        expected.extend_from_slice(&0x0001_0000u32.to_le_bytes());
        expected.extend_from_slice(&[2, 2, 2, 0]);
        expected.extend_from_slice(&[0, 1, 0xde, 0xad]);
        assert_eq!(bytes, expected);

        assert_eq!(
            VersionedMessage::deserialize(&bytes),
            Ok(VersionedMessage::V1(message))
        );
    }

    #[test]
    fn decode_rejects_invalid_masks() {
        let bytes = message().serialize().unwrap();
        for mask in [0b01u32, 0b10, 0b10_0000, 1 << 31] {
            let mut bytes = bytes.clone();
            bytes[4..8].copy_from_slice(&mask.to_le_bytes());
            assert_eq!(
                VersionedMessage::deserialize(&bytes),
                Err(DecodeError::InvalidConfigMask(mask).into())
            );
        }
    }

    #[test]
    fn sanitize_rules() {
        use SanitizeError::*;
        assert_eq!(message().sanitize(), Ok(()));

        let mut m = message();
        m.header.num_required_signatures = 13;
        m.account_keys = (0..13).map(|i| Pubkey::new([i; 32])).collect();
        assert_eq!(sanitize_err(m), TooManySignatures);

        let mut m = message();
        m.instructions = vec![m.instructions[0].clone(); 65];
        assert_eq!(sanitize_err(m), TooManyInstructions);
        let mut m = message();
        m.instructions = vec![m.instructions[0].clone(); 64];
        assert_eq!(m.sanitize(), Ok(()));

        let mut m = message();
        m.account_keys = (0..65).map(|i| Pubkey::new([i; 32])).collect();
        assert_eq!(sanitize_err(m), TooManyAccountKeys);
        let mut m = message();
        m.account_keys = (0..64).map(|i| Pubkey::new([i; 32])).collect();
        assert_eq!(m.sanitize(), Ok(()));

        let mut m = message();
        m.header.num_readonly_unsigned_accounts = 3;
        assert_eq!(sanitize_err(m), NotEnoughAccountKeys);

        let mut m = message();
        m.header.num_readonly_signed_accounts = 1;
        assert_eq!(sanitize_err(m), NoWritableFeePayer);

        let mut m = message();
        m.account_keys[2] = m.account_keys[0];
        assert_eq!(sanitize_err(m), DuplicateAccountKeys);

        for heap_size in [
            MIN_HEAP_SIZE - 1024,
            MIN_HEAP_SIZE + 1,
            MAX_HEAP_SIZE + 1024,
        ] {
            let mut m = message();
            m.config.heap_size = Some(heap_size);
            assert_eq!(sanitize_err(m), InvalidHeapSize, "{heap_size}");
        }
        for heap_size in [MIN_HEAP_SIZE, 64 * 1024, MAX_HEAP_SIZE] {
            let mut m = message();
            m.config.heap_size = Some(heap_size);
            assert_eq!(m.sanitize(), Ok(()), "{heap_size}");
        }

        let mut m = message();
        m.instructions[0].program_id_index = 0;
        assert_eq!(sanitize_err(m), InvalidProgramIndex);
        let mut m = message();
        m.instructions[0].program_id_index = 3;
        assert_eq!(sanitize_err(m), InvalidProgramIndex);
        let mut m = message();
        m.instructions[0].accounts = vec![3];
        assert_eq!(sanitize_err(m), InvalidAccountIndex);
        let mut m = message();
        m.instructions[0].accounts = vec![0; 256];
        assert_eq!(sanitize_err(m), InstructionAccountsTooLarge);
        let mut m = message();
        m.instructions[0].data = vec![0; 65_536];
        assert_eq!(sanitize_err(m), InstructionDataTooLarge);
    }
}
