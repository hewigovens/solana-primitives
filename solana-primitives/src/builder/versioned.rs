//! Versioned transaction support

use crate::{
    types::{Instruction, Pubkey, VersionedTransaction},
    error::{Result, SolanaError},
};
use serde::{Deserialize, Serialize};

/// Transaction version enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionVersion {
    /// Legacy transaction format (pre-versioned transactions)
    Legacy,
    /// Versioned transaction format V0
    V0,
}

impl Default for TransactionVersion {
    fn default() -> Self {
        Self::Legacy
    }
}

/// Configuration for compute budget
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComputeBudgetConfig {
    /// Maximum compute units to consume
    pub compute_unit_limit: Option<u32>,
    /// Price per compute unit in micro-lamports
    pub compute_unit_price: Option<u64>,
    /// Heap frame size for the transaction
    pub heap_frame_size: Option<u32>,
}

/// Address lookup table configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AddressLookupTable {
    /// The lookup table account key
    pub key: Pubkey,
    /// The addresses in the lookup table
    pub addresses: Vec<Pubkey>,
}

/// Enhanced transaction builder supporting versioned transactions
#[derive(Debug)]
pub struct VersionedTransactionBuilder {
    /// The fee payer for the transaction
    fee_payer: Pubkey,
    /// The instructions to include in the transaction
    instructions: Vec<Instruction>,
    /// The recent blockhash
    recent_blockhash: Option<[u8; 32]>,
    /// Transaction version
    version: TransactionVersion,
    /// Address lookup tables
    lookup_tables: Vec<AddressLookupTable>,
    /// Compute budget configuration
    compute_budget: Option<ComputeBudgetConfig>,
    /// Whether to automatically optimize the transaction
    auto_optimize: bool,
}

impl VersionedTransactionBuilder {
    /// Create a new versioned transaction builder
    pub fn new(fee_payer: Pubkey, version: TransactionVersion) -> Self {
        Self {
            fee_payer,
            instructions: Vec::new(),
            recent_blockhash: None,
            version,
            lookup_tables: Vec::new(),
            compute_budget: None,
            auto_optimize: false,
        }
    }

    /// Create a new legacy transaction builder
    pub fn new_legacy(fee_payer: Pubkey) -> Self {
        Self::new(fee_payer, TransactionVersion::Legacy)
    }

    /// Create a new V0 transaction builder
    pub fn new_v0(fee_payer: Pubkey) -> Self {
        Self::new(fee_payer, TransactionVersion::V0)
    }

    /// Set the recent blockhash
    pub fn with_recent_blockhash(mut self, recent_blockhash: [u8; 32]) -> Self {
        self.recent_blockhash = Some(recent_blockhash);
        self
    }

    /// Add an instruction to the transaction
    pub fn add_instruction(mut self, instruction: Instruction) -> Self {
        self.instructions.push(instruction);
        self
    }

    /// Add multiple instructions to the transaction
    pub fn add_instructions(mut self, instructions: Vec<Instruction>) -> Self {
        self.instructions.extend(instructions);
        self
    }

    /// Add an address lookup table
    pub fn with_lookup_table(mut self, table: AddressLookupTable) -> Self {
        self.lookup_tables.push(table);
        self
    }

    /// Set compute budget configuration
    pub fn with_compute_budget(mut self, config: ComputeBudgetConfig) -> Self {
        self.compute_budget = Some(config);
        self
    }

    /// Enable automatic optimization
    pub fn auto_optimize(mut self) -> Self {
        self.auto_optimize = true;
        self
    }

    /// Build the versioned transaction
    pub fn build(self) -> Result<VersionedTransaction> {
        let _recent_blockhash = self.recent_blockhash
            .ok_or_else(|| SolanaError::Transaction("Recent blockhash is required".to_string()))?;

        // TODO: Implement actual transaction building logic
        // This is a placeholder implementation
        match self.version {
            TransactionVersion::Legacy => {
                // Build legacy transaction
                Err(SolanaError::Transaction("Legacy transaction building not yet implemented".to_string()))
            }
            TransactionVersion::V0 => {
                // Build V0 transaction
                Err(SolanaError::Transaction("V0 transaction building not yet implemented".to_string()))
            }
        }
    }

    /// Get the current transaction version
    pub fn version(&self) -> TransactionVersion {
        self.version
    }

    /// Get the fee payer
    pub fn fee_payer(&self) -> &Pubkey {
        &self.fee_payer
    }

    /// Get the instructions
    pub fn instructions(&self) -> &[Instruction] {
        &self.instructions
    }

    /// Get the lookup tables
    pub fn lookup_tables(&self) -> &[AddressLookupTable] {
        &self.lookup_tables
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Pubkey;

    #[test]
    fn test_transaction_version_default() {
        assert_eq!(TransactionVersion::default(), TransactionVersion::Legacy);
    }

    #[test]
    fn test_versioned_transaction_builder_creation() {
        let fee_payer = Pubkey::new([1; 32]);
        let builder = VersionedTransactionBuilder::new_legacy(fee_payer);
        
        assert_eq!(builder.version(), TransactionVersion::Legacy);
        assert_eq!(builder.fee_payer(), &fee_payer);
        assert!(builder.instructions().is_empty());
        assert!(builder.lookup_tables().is_empty());
    }

    #[test]
    fn test_compute_budget_config_default() {
        let config = ComputeBudgetConfig::default();
        assert!(config.compute_unit_limit.is_none());
        assert!(config.compute_unit_price.is_none());
        assert!(config.heap_frame_size.is_none());
    }
}