//! Debugging and inspection utilities

use crate::{
    types::{Transaction, VersionedTransaction, Instruction, CompiledInstruction},
    utils::SerializationUtils,
};

/// Transaction debugging utilities
pub struct TransactionDebugger;

impl TransactionDebugger {
    /// Print detailed transaction information
    pub fn inspect_transaction(transaction: &Transaction) -> String {
        let mut output = String::new();
        
        output.push_str("=== TRANSACTION INSPECTION ===\n");
        output.push_str(&format!("Signatures: {}\n", transaction.signatures.len()));
        output.push_str(&format!("Required signatures: {}\n", transaction.num_required_signatures()));
        output.push_str(&format!("Readonly signed accounts: {}\n", transaction.num_readonly_signed_accounts()));
        output.push_str(&format!("Readonly unsigned accounts: {}\n", transaction.num_readonly_unsigned_accounts()));
        
        output.push_str("\n--- ACCOUNT KEYS ---\n");
        for (i, key) in transaction.account_keys().iter().enumerate() {
            output.push_str(&format!("  {}: {}\n", i, key.to_base58()));
        }
        
        output.push_str(&format!("\nRecent blockhash: {}\n", 
            SerializationUtils::to_base58(transaction.recent_blockhash())));
        
        output.push_str("\n--- INSTRUCTIONS ---\n");
        for (i, instruction) in transaction.instructions().iter().enumerate() {
            output.push_str(&Self::inspect_compiled_instruction(instruction, i));
        }
        
        output
    }

    /// Print detailed versioned transaction information
    pub fn inspect_versioned_transaction(transaction: &VersionedTransaction) -> String {
        let mut output = String::new();
        
        output.push_str("=== VERSIONED TRANSACTION INSPECTION ===\n");
        
        match transaction {
            VersionedTransaction::Legacy { signatures, message } => {
                output.push_str("Type: Legacy\n");
                output.push_str(&format!("Signatures: {}\n", signatures.len()));
                output.push_str(&format!("Required signatures: {}\n", message.header.num_required_signatures));
                
                output.push_str("\n--- ACCOUNT KEYS ---\n");
                for (i, key) in message.account_keys.iter().enumerate() {
                    output.push_str(&format!("  {}: {}\n", i, key.to_base58()));
                }
                
                output.push_str(&format!("\nRecent blockhash: {}\n", 
                    SerializationUtils::to_base58(&message.recent_blockhash)));
                
                output.push_str("\n--- INSTRUCTIONS ---\n");
                for (i, instruction) in message.instructions.iter().enumerate() {
                    output.push_str(&Self::inspect_compiled_instruction(instruction, i));
                }
            }
            VersionedTransaction::V0 { signatures, message } => {
                output.push_str("Type: V0\n");
                output.push_str(&format!("Signatures: {}\n", signatures.len()));
                output.push_str(&format!("Required signatures: {}\n", message.header.num_required_signatures));
                
                output.push_str("\n--- ACCOUNT KEYS ---\n");
                for (i, key) in message.account_keys.iter().enumerate() {
                    output.push_str(&format!("  {}: {}\n", i, key.to_base58()));
                }
                
                output.push_str(&format!("\nRecent blockhash: {}\n", 
                    SerializationUtils::to_base58(&message.recent_blockhash)));
                
                output.push_str("\n--- INSTRUCTIONS ---\n");
                for (i, instruction) in message.instructions.iter().enumerate() {
                    output.push_str(&Self::inspect_compiled_instruction(instruction, i));
                }
                
                if !message.address_table_lookups.is_empty() {
                    output.push_str("\n--- ADDRESS LOOKUP TABLES ---\n");
                    for (i, lookup) in message.address_table_lookups.iter().enumerate() {
                        output.push_str(&format!("  Lookup {}: {}\n", i, lookup.account_key.to_base58()));
                        output.push_str(&format!("    Writable indexes: {:?}\n", lookup.writable_indexes));
                        output.push_str(&format!("    Readonly indexes: {:?}\n", lookup.readonly_indexes));
                    }
                }
            }
        }
        
        output
    }

    /// Inspect a compiled instruction
    fn inspect_compiled_instruction(instruction: &CompiledInstruction, index: usize) -> String {
        let mut output = String::new();
        
        output.push_str(&format!("  Instruction {}:\n", index));
        output.push_str(&format!("    Program ID index: {}\n", instruction.program_id_index));
        output.push_str(&format!("    Account indices: {:?}\n", instruction.accounts));
        output.push_str(&format!("    Data length: {} bytes\n", instruction.data.len()));
        
        if !instruction.data.is_empty() {
            output.push_str(&format!("    Data (hex): {}\n", SerializationUtils::to_hex(&instruction.data)));
            output.push_str(&format!("    Data (base58): {}\n", SerializationUtils::to_base58(&instruction.data)));
            
            // Try to display as UTF-8 if possible
            if let Ok(utf8_str) = std::str::from_utf8(&instruction.data) {
                if utf8_str.chars().all(|c| c.is_ascii() && !c.is_control()) {
                    output.push_str(&format!("    Data (UTF-8): {}\n", utf8_str));
                }
            }
        }
        
        output
    }

    /// Inspect a regular instruction
    pub fn inspect_instruction(instruction: &Instruction) -> String {
        let mut output = String::new();
        
        output.push_str("=== INSTRUCTION INSPECTION ===\n");
        output.push_str(&format!("Program ID: {}\n", instruction.program_id.to_base58()));
        output.push_str(&format!("Accounts: {}\n", instruction.accounts.len()));
        
        for (i, account) in instruction.accounts.iter().enumerate() {
            output.push_str(&format!("  Account {}: {}\n", i, account.pubkey.to_base58()));
            output.push_str(&format!("    Signer: {}\n", account.is_signer));
            output.push_str(&format!("    Writable: {}\n", account.is_writable));
        }
        
        output.push_str(&format!("Data length: {} bytes\n", instruction.data.len()));
        
        if !instruction.data.is_empty() {
            output.push_str(&format!("Data (hex): {}\n", SerializationUtils::to_hex(&instruction.data)));
            output.push_str(&format!("Data (base58): {}\n", SerializationUtils::to_base58(&instruction.data)));
        }
        
        output
    }

    /// Get transaction summary
    pub fn transaction_summary(transaction: &Transaction) -> String {
        format!(
            "Transaction: {} signatures, {} accounts, {} instructions, size: ~{} bytes",
            transaction.signatures.len(),
            transaction.account_keys().len(),
            transaction.instructions().len(),
            crate::utils::TransactionValidator::calculate_transaction_size(transaction)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        types::{Message, MessageHeader, SignatureBytes, AccountMeta},
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
            data: vec![1, 2, 3, 4],
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
    fn test_inspect_transaction() {
        let transaction = create_test_transaction();
        let output = TransactionDebugger::inspect_transaction(&transaction);
        
        assert!(output.contains("TRANSACTION INSPECTION"));
        assert!(output.contains("Signatures: 1"));
        assert!(output.contains("ACCOUNT KEYS"));
        assert!(output.contains("INSTRUCTIONS"));
    }

    #[test]
    fn test_inspect_instruction() {
        let instruction = Instruction {
            program_id: Pubkey::new([1; 32]),
            accounts: vec![AccountMeta {
                pubkey: Pubkey::new([2; 32]),
                is_signer: true,
                is_writable: false,
            }],
            data: vec![1, 2, 3],
        };
        
        let output = TransactionDebugger::inspect_instruction(&instruction);
        
        assert!(output.contains("INSTRUCTION INSPECTION"));
        assert!(output.contains("Program ID:"));
        assert!(output.contains("Accounts: 1"));
        assert!(output.contains("Signer: true"));
        assert!(output.contains("Writable: false"));
    }

    #[test]
    fn test_transaction_summary() {
        let transaction = create_test_transaction();
        let summary = TransactionDebugger::transaction_summary(&transaction);
        
        assert!(summary.contains("1 signatures"));
        assert!(summary.contains("2 accounts"));
        assert!(summary.contains("1 instructions"));
        assert!(summary.contains("bytes"));
    }
}