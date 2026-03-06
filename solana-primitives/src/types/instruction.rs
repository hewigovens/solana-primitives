use super::pubkey::Pubkey;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

/// Represents a Solana instruction
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct Instruction {
    /// The program ID that will process this instruction
    #[serde(alias = "programId")]
    pub program_id: Pubkey,
    /// The accounts that will be read from or written to
    #[serde(alias = "keys")]
    pub accounts: Vec<AccountMeta>,
    /// The instruction data
    pub data: Vec<u8>,
}

/// Metadata about an account in an instruction
#[derive(Debug, Clone, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct AccountMeta {
    /// The account's public key
    #[serde(alias = "publicKey")]
    pub pubkey: Pubkey,
    /// Whether the account is a signer
    #[serde(alias = "isSigner")]
    pub is_signer: bool,
    /// Whether the account is writable
    #[serde(alias = "isWritable")]
    pub is_writable: bool,
}

impl AccountMeta {
    /// Create a new AccountMeta with explicit signer/writable flags.
    pub fn new(pubkey: Pubkey, is_signer: bool, is_writable: bool) -> Self {
        Self {
            pubkey,
            is_signer,
            is_writable,
        }
    }

    /// Create a new AccountMeta that is read-only
    pub fn new_readonly(pubkey: Pubkey) -> Self {
        Self::new(pubkey, false, false)
    }

    /// Create a new AccountMeta that is a signer
    pub fn new_signer(pubkey: Pubkey) -> Self {
        Self::new(pubkey, true, false)
    }

    /// Create a new AccountMeta that is writable
    pub fn new_writable(pubkey: Pubkey) -> Self {
        Self::new(pubkey, false, true)
    }

    /// Create a new AccountMeta that is both a signer and writable
    pub fn new_signer_writable(pubkey: Pubkey) -> Self {
        Self::new(pubkey, true, true)
    }
}

/// A compiled instruction that references accounts by their indices
#[derive(Debug, Clone, PartialEq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
pub struct CompiledInstruction {
    /// Index into the account keys array indicating the program to execute
    pub program_id_index: u8,
    /// Indices into the account keys array indicating which accounts to pass to the program
    pub accounts: Vec<u8>,
    /// The instruction data
    pub data: Vec<u8>,
}
