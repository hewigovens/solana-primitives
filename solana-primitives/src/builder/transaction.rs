use crate::{
    AccountMeta, CompiledInstruction, Instruction, Message, MessageHeader, Pubkey, Result,
    SignatureBytes, Transaction,
};
use std::collections::HashMap;

/// A builder for constructing Solana transactions
#[derive(Debug)]
pub struct TransactionBuilder {
    /// The fee payer for the transaction
    fee_payer: Pubkey,
    /// The instructions to include in the transaction
    instructions: Vec<Instruction>,
    /// The recent blockhash
    recent_blockhash: [u8; 32],
    /// A map of account public keys to their metadata, including the fee payer
    account_metas: HashMap<Pubkey, AccountMeta>,
}

impl TransactionBuilder {
    /// Create a new transaction builder
    pub fn new(fee_payer: Pubkey, recent_blockhash: [u8; 32]) -> Self {
        let mut account_metas = HashMap::new();
        account_metas.insert(
            fee_payer,
            AccountMeta {
                pubkey: fee_payer,
                is_signer: true,
                is_writable: true,
            },
        );

        Self {
            fee_payer, // Store the fee_payer
            instructions: Vec::new(),
            recent_blockhash,
            account_metas,
        }
    }

    /// Add an instruction to the transaction
    pub fn add_instruction(&mut self, instruction: Instruction) -> &mut Self {
        // Add program ID to account metas. Program IDs are typically not signers and are read-only (executable).
        self.account_metas
            .entry(instruction.program_id)
            .or_insert_with(|| AccountMeta {
                pubkey: instruction.program_id,
                is_signer: false,
                is_writable: false,
            });

        // Add all accounts from the instruction to our account_metas, merging properties.
        // If an account is used in multiple instructions, its signer/writable status is the OR of all uses.
        for account_meta in &instruction.accounts {
            self.account_metas
                .entry(account_meta.pubkey)
                .and_modify(|existing_meta| {
                    existing_meta.is_signer = existing_meta.is_signer || account_meta.is_signer;
                    existing_meta.is_writable =
                        existing_meta.is_writable || account_meta.is_writable;
                })
                .or_insert_with(|| account_meta.clone());
        }
        self.instructions.push(instruction);
        self
    }

    /// Build the transaction
    pub fn build(self) -> Result<Transaction> {
        let mut final_account_keys = Vec::new();
        // HashSet to track keys already added to final_account_keys to prevent duplicates,
        // though the categorization should handle distinct roles.
        let mut processed_keys = std::collections::HashSet::new();

        // 1. Fee payer first
        final_account_keys.push(self.fee_payer);
        processed_keys.insert(self.fee_payer);

        let mut writable_signers = Vec::new();
        let mut readonly_signers = Vec::new();
        let mut writable_non_signers = Vec::new();
        let mut readonly_non_signers = Vec::new();

        // Categorize all other accounts from account_metas
        for (pubkey, meta) in &self.account_metas {
            if *pubkey == self.fee_payer {
                // Already added
                continue;
            }
            if meta.is_signer {
                if meta.is_writable {
                    writable_signers.push(*pubkey);
                } else {
                    readonly_signers.push(*pubkey);
                }
            } else if meta.is_writable {
                writable_non_signers.push(*pubkey);
            } else {
                readonly_non_signers.push(*pubkey);
            }
        }

        // Sort within categories for deterministic output
        writable_signers.sort();
        readonly_signers.sort();
        writable_non_signers.sort();
        readonly_non_signers.sort();

        // Append categorized keys to final_account_keys, ensuring no duplicates from previous categories
        for key in writable_signers {
            if processed_keys.insert(key) {
                // insert returns true if value was newly inserted
                final_account_keys.push(key);
            }
        }
        for key in readonly_signers {
            if processed_keys.insert(key) {
                final_account_keys.push(key);
            }
        }
        for key in writable_non_signers {
            if processed_keys.insert(key) {
                final_account_keys.push(key);
            }
        }
        for key in readonly_non_signers {
            if processed_keys.insert(key) {
                final_account_keys.push(key);
            }
        }

        let account_keys: Vec<Pubkey> = final_account_keys;

        // Create a map of pubkey to index for quick lookups
        let key_to_index: HashMap<Pubkey, u8> = account_keys
            .iter()
            .enumerate()
            .map(|(i, &key)| (key, i as u8))
            .collect();

        // Compile instructions
        let compiled_instructions: Vec<CompiledInstruction> = self
            .instructions
            .iter()
            .map(|instruction| {
                let program_id_index = key_to_index[&instruction.program_id];
                let accounts: Vec<u8> = instruction
                    .accounts
                    .iter()
                    .map(|meta| key_to_index[&meta.pubkey])
                    .collect();

                CompiledInstruction {
                    program_id_index,
                    accounts,
                    data: instruction.data.clone(),
                }
            })
            .collect();

        // Create message header
        let num_required_signatures = self
            .account_metas
            .values()
            .filter(|meta| meta.is_signer)
            .count() as u8;

        let num_readonly_signed_accounts = self
            .account_metas
            .values()
            .filter(|meta| meta.is_signer && !meta.is_writable)
            .count() as u8;

        let num_readonly_unsigned_accounts = self
            .account_metas
            .values()
            .filter(|meta| !meta.is_signer && !meta.is_writable)
            .count() as u8;

        let header = MessageHeader {
            num_required_signatures,
            num_readonly_signed_accounts,
            num_readonly_unsigned_accounts,
        };

        // Create message
        let message = Message {
            header,
            account_keys,
            recent_blockhash: self.recent_blockhash,
            instructions: compiled_instructions,
        };

        // Create empty signatures vector
        let signatures = vec![SignatureBytes::new([0u8; 64]); num_required_signatures as usize];

        Ok(Transaction {
            signatures,
            message,
        })
    }
}
