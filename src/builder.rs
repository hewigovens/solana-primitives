use crate::{
    AccountMeta, CompiledInstruction, Instruction, Message, MessageHeader, Pubkey, Result,
    SignatureBytes, Transaction,
};
use std::collections::HashMap;

/// A builder for constructing Solana transactions
#[derive(Debug)]
pub struct TransactionBuilder {
    /// The instructions to include in the transaction
    instructions: Vec<Instruction>,
    /// The recent blockhash
    recent_blockhash: [u8; 32],
    /// A map of account public keys to their metadata
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
            instructions: Vec::new(),
            recent_blockhash,
            account_metas,
        }
    }

    /// Add an instruction to the transaction
    pub fn add_instruction(&mut self, instruction: Instruction) -> &mut Self {
        // Add all accounts from the instruction to our account metas
        for account_meta in &instruction.accounts {
            self.account_metas
                .entry(account_meta.pubkey)
                .or_insert_with(|| account_meta.clone());
        }
        self.instructions.push(instruction);
        self
    }

    /// Build the transaction
    pub fn build(self) -> Result<Transaction> {
        // Convert account metas to a vector and sort them
        let mut account_keys: Vec<Pubkey> = self.account_metas.keys().copied().collect();
        account_keys.sort();

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

/// A builder for constructing Solana instructions
#[derive(Debug)]
pub struct InstructionBuilder {
    /// The program ID that will process this instruction
    program_id: Pubkey,
    /// The accounts that will be read from or written to
    accounts: Vec<AccountMeta>,
    /// The instruction data
    data: Vec<u8>,
}

impl InstructionBuilder {
    /// Create a new instruction builder
    pub fn new(program_id: Pubkey) -> Self {
        Self {
            program_id,
            accounts: Vec::new(),
            data: Vec::new(),
        }
    }

    /// Add an account to the instruction
    pub fn account(mut self, pubkey: Pubkey, is_signer: bool, is_writable: bool) -> Self {
        self.accounts.push(AccountMeta {
            pubkey,
            is_signer,
            is_writable,
        });
        self
    }

    /// Set the instruction data
    pub fn data(mut self, data: Vec<u8>) -> Self {
        self.data = data;
        self
    }

    /// Build the instruction
    pub fn build(self) -> Instruction {
        Instruction {
            program_id: self.program_id,
            accounts: self.accounts,
            data: self.data,
        }
    }
}
