pub mod builder;
pub mod error;
#[cfg(feature = "jsonrpc")]
pub mod jsonrpc;
pub mod types;

pub use builder::{InstructionBuilder, TransactionBuilder};
pub use types::*;

#[cfg(feature = "jsonrpc")]
pub use jsonrpc::RpcClient;

use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SolanaError {
    #[error("Invalid public key length")]
    InvalidPubkeyLength,
    #[error("Invalid signature length")]
    InvalidSignatureLength,
    #[error("Invalid instruction data")]
    InvalidInstructionData,
    #[error("RPC error: {0}")]
    RpcError(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
}

pub type Result<T> = std::result::Result<T, SolanaError>;

/// A Solana public key (32 bytes)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, BorshSerialize, BorshDeserialize)]
pub struct Pubkey([u8; 32]);

impl Ord for Pubkey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialOrd for Pubkey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Serialize for Pubkey {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_base58())
    }
}

impl<'de> Deserialize<'de> for Pubkey {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = <String as Deserialize>::deserialize(deserializer)?;
        Pubkey::from_base58(&s).map_err(serde::de::Error::custom)
    }
}

impl Pubkey {
    pub fn new(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub fn from_base58(s: &str) -> Result<Self> {
        let bytes = bs58::decode(s)
            .into_vec()
            .map_err(|_| SolanaError::InvalidPubkeyLength)?;

        if bytes.len() != 32 {
            return Err(SolanaError::InvalidPubkeyLength);
        }

        Ok(Self(bytes.try_into().unwrap()))
    }

    pub fn to_base58(&self) -> String {
        bs58::encode(&self.0).into_string()
    }

    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// Represents a Solana signature (64 bytes)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, BorshSerialize, BorshDeserialize)]
pub struct SignatureBytes([u8; 64]);

impl Serialize for SignatureBytes {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_base58())
    }
}

impl<'de> Deserialize<'de> for SignatureBytes {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = <String as Deserialize>::deserialize(deserializer)?;
        SignatureBytes::from_base58(&s).map_err(serde::de::Error::custom)
    }
}

impl SignatureBytes {
    pub fn new(bytes: [u8; 64]) -> Self {
        Self(bytes)
    }

    pub fn from_base58(s: &str) -> Result<Self> {
        let bytes = bs58::decode(s)
            .into_vec()
            .map_err(|_| SolanaError::InvalidSignatureLength)?;

        if bytes.len() != 64 {
            return Err(SolanaError::InvalidSignatureLength);
        }

        Ok(Self(bytes.try_into().unwrap()))
    }

    pub fn to_base58(&self) -> String {
        bs58::encode(&self.0).into_string()
    }

    pub fn as_bytes(&self) -> &[u8; 64] {
        &self.0
    }
}

/// Represents a Solana instruction
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct Instruction {
    /// The program ID that will process this instruction
    pub program_id: Pubkey,
    /// The accounts that will be read from or written to
    pub accounts: Vec<AccountMeta>,
    /// The instruction data
    pub data: Vec<u8>,
}

/// Metadata about an account in an instruction
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct AccountMeta {
    /// The account's public key
    pub pubkey: Pubkey,
    /// Whether the account is a signer
    pub is_signer: bool,
    /// Whether the account is writable
    pub is_writable: bool,
}

/// The header of a Solana message
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct MessageHeader {
    /// Number of signatures required for this transaction
    pub num_required_signatures: u8,
    /// Number of read-only signed accounts
    pub num_readonly_signed_accounts: u8,
    /// Number of read-only unsigned accounts
    pub num_readonly_unsigned_accounts: u8,
}

/// A Solana message that contains the transaction details
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct Message {
    /// The message header
    pub header: MessageHeader,
    /// All account public keys referenced in the transaction
    pub account_keys: Vec<Pubkey>,
    /// The recent blockhash
    pub recent_blockhash: [u8; 32],
    /// The instructions to execute
    pub instructions: Vec<CompiledInstruction>,
}

/// A compiled instruction that references accounts by their indices
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct CompiledInstruction {
    /// Index into the account keys array indicating the program to execute
    pub program_id_index: u8,
    /// Indices into the account keys array indicating which accounts to pass to the program
    pub accounts: Vec<u8>,
    /// The instruction data
    pub data: Vec<u8>,
}

/// A complete Solana transaction
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct Transaction {
    /// The signatures for this transaction
    pub signatures: Vec<SignatureBytes>,
    /// The message containing the transaction details
    pub message: Message,
}

// Helper function to convert a compact array to bytes
pub fn compact_array_to_bytes<T: BorshSerialize>(items: &[T]) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    items
        .serialize(&mut &mut bytes)
        .map_err(|e| SolanaError::SerializationError(e.to_string()))?;
    Ok(bytes)
}

// Helper function to convert bytes to a compact array
pub fn bytes_to_compact_array<T: BorshDeserialize>(bytes: &[u8]) -> Result<Vec<T>> {
    let mut bytes = bytes;
    Vec::<T>::deserialize(&mut bytes).map_err(|e| SolanaError::SerializationError(e.to_string()))
}
