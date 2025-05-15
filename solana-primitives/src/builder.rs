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
                    existing_meta.is_writable = existing_meta.is_writable || account_meta.is_writable;
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
            if *pubkey == self.fee_payer { // Already added
                continue;
            }
            if meta.is_signer {
                if meta.is_writable {
                    writable_signers.push(*pubkey);
                } else {
                    readonly_signers.push(*pubkey);
                }
            } else {
                if meta.is_writable {
                    writable_non_signers.push(*pubkey);
                } else {
                    readonly_non_signers.push(*pubkey);
                }
            }
        }

        // Sort within categories for deterministic output
        writable_signers.sort();
        readonly_signers.sort();
        writable_non_signers.sort();
        readonly_non_signers.sort();

        // Append categorized keys to final_account_keys, ensuring no duplicates from previous categories
        for key in writable_signers {
            if processed_keys.insert(key) { // insert returns true if value was newly inserted
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
    use crate::instructions::{system::create_account, token::transfer_checked};
    use crate::{types::instruction::AccountMeta, Pubkey};
    use base64::engine::general_purpose::STANDARD;
    use base64::Engine;

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
        use crate::types::VersionedTransaction; // Added import for deserialization
        let recent_blockhash = "9U2ogLjDt479wubHbEtPLGBF84DijmWggA4KoXSwcivd";
        let recent_blockhash_bytes = bs58::decode(recent_blockhash).into_vec().unwrap();
        let fee_payer: Pubkey = "A21o4asMbFHYadqXdLusT9Bvx9xaC5YV9gcaidjqtdXC".parse().unwrap();
        let program_id = Pubkey::from_base58("J88B7gmadHzTNGiy54c9Ms8BsEXNdB2fntFyhKpk3qoT").unwrap();
        let data = hex::decode("a3265ce2f3698dc400000070000000000100000014000000514bcb1f9aabb904e6106bd1052b66d2706dbbb701000000006c000000000a00000085fba93ee29c604fa858a351688c01290841eafb19c63a70a475d3c7bc3bef9f000000000000000000008489b9cc07af97add00300000000000000000000000000001e83d2972d3dca3a330d60c2777ee5b8d25683c63fa359116985609830f42054050004002d16000000f0314f0cffdf8d00b6a7ce61f86164ca47c1b8b1bc2e").unwrap();
        let mut instruction = InstructionBuilder::new(program_id).data(data);

        instruction.accounts.extend_from_slice(&[
            AccountMeta {
                pubkey: "ACLMuTFvDAb3oecQQGkTVqpUbhCKHG3EZ9uNXHK1W9ka".parse().unwrap(),
                is_signer: false,
                is_writable: false,
            },
            AccountMeta {
                pubkey: "3tJ67qa2GDfvv2wcMYNUfN5QBZrFpTwcU8ASZKMvCTVU".parse().unwrap(),
                is_signer: false,
                is_writable: true,
            },
            AccountMeta {
                pubkey: "A21o4asMbFHYadqXdLusT9Bvx9xaC5YV9gcaidjqtdXC".parse().unwrap(),
                is_signer: true,
                is_writable: true,
            },
            AccountMeta {
                pubkey: "E8p6aiwuSDWEzQnjGjkNiMZrd1rpSsntWsaZCivdFz51".parse().unwrap(),
                is_signer: false,
                is_writable: true
            },
            AccountMeta {
                pubkey: "FmAcjWaRFUxGWBfGT7G3CzcFeJFsewQ4KPJVG4f6fcob".parse().unwrap(),
                is_signer: false,
                is_writable: true
            },
            AccountMeta {
                pubkey: "11111111111111111111111111111111".parse().unwrap(),
                is_signer: false,
                is_writable: false
            }
        ]);

        let mut tx_builder = TransactionBuilder::new(fee_payer, recent_blockhash_bytes.try_into().unwrap());
        tx_builder.add_instruction(instruction.build());

        let transaction = tx_builder.build().unwrap(); 
        println!("Transaction: {:#?}", transaction);

        // Use the new method on Transaction to serialize
        let tx_wire_bytes = transaction.serialize_legacy().expect("Failed to serialize transaction with wire format");

        let base64_tx = STANDARD.encode(&tx_wire_bytes);
        println!("Base64 transaction: {}", base64_tx);

        // Deserialize and verify
        let deserialized_vt = VersionedTransaction::deserialize_with_version(&tx_wire_bytes)
            .expect("Failed to deserialize wire bytes into VersionedTransaction");

        match deserialized_vt {
            VersionedTransaction::Legacy { signatures: deserialized_signatures, message: deserialized_legacy_message } => {
                assert_eq!(deserialized_signatures, transaction.signatures, "Signatures mismatch after round trip");
                assert_eq!(deserialized_legacy_message.header, transaction.message.header, "Message header mismatch");
                assert_eq!(deserialized_legacy_message.account_keys, transaction.message.account_keys, "Account keys mismatch");
                assert_eq!(deserialized_legacy_message.recent_blockhash, transaction.message.recent_blockhash, "Recent blockhash mismatch");
                assert_eq!(deserialized_legacy_message.instructions, transaction.message.instructions, "Instructions mismatch");
            }
            _ => panic!("Deserialized transaction is not the expected Legacy variant"),
        }
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
