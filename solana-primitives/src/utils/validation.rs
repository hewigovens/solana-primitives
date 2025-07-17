//! Transaction and data validation utilities

use crate::{
    error::{Result, SolanaError},
    types::{Transaction, VersionedTransaction, Instruction},
};

/// Transaction validation utilities
pub struct TransactionValidator;

impl TransactionValidator {
    /// Validate a transaction for common issues
    pub fn validate_transaction(transaction: &Transaction) -> Result<()> {
        // Check if transaction has instructions
        if transaction.instructions().is_empty() {
            return Err(SolanaError::Transaction("Transaction has no instructions".to_string()));
        }

        // Check if transaction has required signatures
        if transaction.signatures.len() != transaction.num_required_signatures() as usize {
            return Err(SolanaError::Transaction(
                format!(
                    "Transaction signature count mismatch: expected {}, got {}",
                    transaction.num_required_signatures(),
                    transaction.signatures.len()
                )
            ));
        }

        // Check account keys
        if transaction.account_keys().is_empty() {
            return Err(SolanaError::Transaction("Transaction has no account keys".to_string()));
        }

        Ok(())
    }

    /// Validate a versioned transaction
    pub fn validate_versioned_transaction(transaction: &VersionedTransaction) -> Result<()> {
        // Check if transaction has instructions
        if transaction.instructions().is_empty() {
            return Err(SolanaError::Transaction("Transaction has no instructions".to_string()));
        }

        // Check account keys
        if transaction.account_keys().is_empty() {
            return Err(SolanaError::Transaction("Transaction has no account keys".to_string()));
        }

        Ok(())
    }

    /// Validate an instruction
    pub fn validate_instruction(instruction: &Instruction) -> Result<()> {
        // Check if instruction has a valid program ID
        if instruction.program_id.as_bytes().iter().all(|&b| b == 0) {
            return Err(SolanaError::Transaction("Instruction has invalid program ID".to_string()));
        }

        Ok(())
    }

    /// Calculate transaction size in bytes
    pub fn calculate_transaction_size(transaction: &Transaction) -> usize {
        // This is a simplified calculation
        // In a real implementation, you would serialize the transaction and get the actual size
        let signature_size = transaction.signatures.len() * 64;
        let message_size = 32 + // recent blockhash
            3 + // header
            transaction.account_keys().len() * 32 + // account keys
            transaction.instructions().iter().map(|ix| {
                1 + // program_id_index
                ix.accounts.len() + // account indices
                ix.data.len() // instruction data
            }).sum::<usize>();
        
        signature_size + message_size
    }

    /// Check if transaction size is within limits
    pub fn check_transaction_size_limit(transaction: &Transaction) -> Result<()> {
        const MAX_TRANSACTION_SIZE: usize = 1232; // Solana's current limit
        
        let size = Self::calculate_transaction_size(transaction);
        if size > MAX_TRANSACTION_SIZE {
            return Err(SolanaError::Transaction(
                format!("Transaction size {size} exceeds limit of {MAX_TRANSACTION_SIZE}")
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        types::{Message, MessageHeader, SignatureBytes, CompiledInstruction},
        Pubkey,
    };

    fn create_test_transaction() -> Transaction {
        let header = MessageHeader {
            num_required_signatures: 1,
            num_readonly_signed_accounts: 0,
            num_readonly_unsigned_accounts: 1,
        };
        let account_keys = vec![Pubkey::new([0; 32]), Pubkey::new([1; 32])];
        let recent_blockhash = [0u8; 32];
        let instructions = vec![CompiledInstruction {
            program_id_index: 1,
            accounts: vec![0],
            data: vec![],
        }];

        let message = Message {
            header,
            account_keys,
            recent_blockhash,
            instructions,
        };

        Transaction {
            signatures: vec![SignatureBytes::new([0; 64])],
            message,
        }
    }

    #[test]
    fn test_validate_transaction() {
        let transaction = create_test_transaction();
        let result = TransactionValidator::validate_transaction(&transaction);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_empty_transaction() {
        let mut transaction = create_test_transaction();
        transaction.message.instructions.clear();
        
        let result = TransactionValidator::validate_transaction(&transaction);
        assert!(result.is_err());
    }

    #[test]
    fn test_calculate_transaction_size() {
        let transaction = create_test_transaction();
        let size = TransactionValidator::calculate_transaction_size(&transaction);
        assert!(size > 0);
    }

    #[test]
    fn test_check_transaction_size_limit() {
        let transaction = create_test_transaction();
        let result = TransactionValidator::check_transaction_size_limit(&transaction);
        assert!(result.is_ok());
    }
}