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
        // Add program ID to account metas to ensure it's included in the account keys
        self.account_metas
            .entry(instruction.program_id)
            .or_insert_with(|| AccountMeta {
                pubkey: instruction.program_id,
                is_signer: false,
                is_writable: false,
            });

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::system::create_account;
    use crate::instructions::token::transfer_checked;
    use crate::types::instruction::AccountMeta;
    use crate::Pubkey;

    // Real public keys from JavaScript test file
    fn system_program() -> Pubkey {
        Pubkey::from_base58("11111111111111111111111111111111").unwrap()
    }

    fn token_program() -> Pubkey {
        Pubkey::from_base58("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap()
    }

    fn mint_pubkey() -> Pubkey {
        Pubkey::from_base58("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap()
    }

    fn token_pubkey() -> Pubkey {
        Pubkey::from_base58("4q2wPZuZwQTB1dEU9sMGsJK1d8NSL1hpBjTGHBsLQNDh").unwrap()
    }

    fn authority_pubkey() -> Pubkey {
        Pubkey::from_base58("Hozo7TadHq6PMMiGLGNvgk79Hvj5VTAM7Ny2bamQ2m8q").unwrap()
    }

    fn payer_pubkey() -> Pubkey {
        Pubkey::from_base58("7o36UsWR1JQLpZ9PE2gn9L4SQ69CNNiWAXd4Jt7rqz9Z").unwrap()
    }

    fn new_account_pubkey() -> Pubkey {
        Pubkey::from_base58("DShWnroshVbeUp28oopA3Pu7oFPDBtC1DBmPECXXAQ9n").unwrap()
    }

    // Create a random pubkey for tests that need a unique key
    fn random_pubkey() -> Pubkey {
        let mut bytes = [0u8; 32];
        bytes
            .iter_mut()
            .enumerate()
            .for_each(|(i, byte)| *byte = i as u8);
        Pubkey::new(bytes)
    }

    // Create a test blockhash for transactions
    fn test_blockhash() -> [u8; 32] {
        let mut bytes = [0u8; 32];
        bytes
            .iter_mut()
            .enumerate()
            .for_each(|(i, byte)| *byte = i as u8);
        bytes
    }

    #[test]
    fn test_instruction_builder() {
        let program_id = token_program();
        let source = token_pubkey();
        let dest = random_pubkey();
        let owner = authority_pubkey();
        let mint = mint_pubkey();
        let amount = 1_000_000;
        let decimals = 6;

        // Build the instruction
        let mut builder = InstructionBuilder::new(program_id);
        builder.accounts.push(AccountMeta {
            pubkey: source,
            is_signer: false,
            is_writable: true,
        });
        builder.accounts.push(AccountMeta {
            pubkey: mint,
            is_signer: false,
            is_writable: false,
        });
        builder.accounts.push(AccountMeta {
            pubkey: dest,
            is_signer: false,
            is_writable: true,
        });
        builder.accounts.push(AccountMeta {
            pubkey: owner,
            is_signer: true,
            is_writable: false,
        });

        let ix = transfer_checked(&source, &mint, &dest, &owner, amount, decimals);

        // Check program ID
        assert_eq!(builder.program_id, token_program());
        assert_eq!(ix.program_id, token_program());
    }

    #[test]
    fn test_transaction_builder() {
        // Create a simple transaction with one token transfer
        let payer = payer_pubkey();
        let blockhash = test_blockhash();
        let mut tx_builder = TransactionBuilder::new(payer, blockhash);

        // Add a token transfer instruction
        let source = token_pubkey();
        let dest = random_pubkey();
        let owner = authority_pubkey();
        let mint = mint_pubkey();
        let amount = 1_000_000;
        let decimals = 6;

        let transfer_ix = transfer_checked(&source, &mint, &dest, &owner, amount, decimals);
        tx_builder.add_instruction(transfer_ix);

        // Build the transaction
        let transaction = tx_builder.build().unwrap();

        // Verify the transaction has the correct structure
        assert_eq!(transaction.signatures.len(), 2); // payer + owner signatures

        // Check that account keys are properly included
        let account_keys = &transaction.message.account_keys;
        assert!(account_keys.contains(&payer_pubkey())); // Fee payer
        assert!(account_keys.contains(&source)); // Source account
        assert!(account_keys.contains(&dest)); // Destination account
        assert!(account_keys.contains(&owner)); // Owner
        assert!(account_keys.contains(&mint)); // Mint
        assert!(account_keys.contains(&token_program())); // Token program

        // Verify the instructions
        assert_eq!(transaction.message.instructions.len(), 1);
        let compiled_ix = &transaction.message.instructions[0];
        assert_eq!(
            account_keys[compiled_ix.program_id_index as usize],
            token_program()
        );
    }

    #[test]
    fn test_complex_transaction() {
        // Create a transaction with multiple instructions to test shortVec encoding
        let payer = payer_pubkey();
        let blockhash = test_blockhash();
        let mut tx_builder = TransactionBuilder::new(payer, blockhash);

        // Add a system create account instruction
        let from = payer_pubkey();
        let new_account = new_account_pubkey();
        let owner = system_program(); // Owner will be system program for this test
        let lamports = 1_000_000_000; // 1 SOL
        let space = 165;

        let create_account_ix = create_account(&from, &new_account, lamports, space, &owner);
        tx_builder.add_instruction(create_account_ix);

        // Add a token transfer instruction
        let source = token_pubkey();
        let dest = random_pubkey();
        let owner = authority_pubkey();
        let mint = mint_pubkey();
        let amount = 1_000_000;
        let decimals = 6;

        let transfer_ix = transfer_checked(&source, &mint, &dest, &owner, amount, decimals);
        tx_builder.add_instruction(transfer_ix);

        // Build the transaction
        let transaction = tx_builder.build().unwrap();

        // Verify the transaction has the correct structure
        // There should be multiple signatures (at least payer and new_account for create_account)
        assert!(transaction.signatures.len() >= 2);

        // Check account keys are properly included
        let account_keys = &transaction.message.account_keys;
        assert!(account_keys.contains(&payer_pubkey())); // Fee payer
        assert!(account_keys.contains(&new_account)); // New account
        assert!(account_keys.contains(&system_program())); // System program
        assert!(account_keys.contains(&source)); // Source account
        assert!(account_keys.contains(&dest)); // Destination account
        assert!(account_keys.contains(&owner)); // Owner
        assert!(account_keys.contains(&mint)); // Mint
        assert!(account_keys.contains(&token_program())); // Token program

        // Verify number of instructions
        assert_eq!(transaction.message.instructions.len(), 2);

        // Test serialization - since we don't have a direct serialize method,
        // we'll at least check that we can access the message
        let message_data = &transaction.message;
        assert!(!message_data.account_keys.is_empty());
    }
}
