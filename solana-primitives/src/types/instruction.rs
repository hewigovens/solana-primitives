use super::pubkey::Pubkey;
use borsh::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};

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
