use crate::{
    AccountMeta, AddressLookupTableAccount, CompiledInstruction, Instruction, Message,
    MessageAddressTableLookup, MessageHeader, Pubkey, Result, SignatureBytes, SolanaError,
    Transaction, VersionedMessageV0, VersionedTransaction,
};
use std::collections::{HashMap, HashSet};

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

    /// Add multiple instructions to the transaction.
    pub fn add_instructions<I>(&mut self, instructions: I) -> &mut Self
    where
        I: IntoIterator<Item = Instruction>,
    {
        for instruction in instructions {
            self.add_instruction(instruction);
        }
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

    /// Build a V0 versioned transaction.
    pub fn build_v0(
        self,
        address_lookup_tables: &[AddressLookupTableAccount],
    ) -> Result<VersionedTransaction> {
        let mut lookup_map: HashMap<Pubkey, (usize, u8)> = HashMap::new();
        for (table_index, table) in address_lookup_tables.iter().enumerate().rev() {
            for (entry_index, address) in table.addresses.iter().enumerate() {
                if let Ok(entry_index_u8) = u8::try_from(entry_index) {
                    lookup_map.insert(*address, (table_index, entry_index_u8));
                } else {
                    break;
                }
            }
        }

        let program_ids: HashSet<Pubkey> = self
            .instructions
            .iter()
            .map(|instruction| instruction.program_id)
            .collect();

        let mut flags: HashMap<Pubkey, (bool, bool)> = HashMap::new();
        let mut order: Vec<Pubkey> = Vec::new();
        let mut merge = |pubkey: Pubkey, is_signer: bool, is_writable: bool| {
            flags
                .entry(pubkey)
                .and_modify(|(existing_signer, existing_writable)| {
                    *existing_signer |= is_signer;
                    *existing_writable |= is_writable;
                })
                .or_insert_with(|| {
                    order.push(pubkey);
                    (is_signer, is_writable)
                });
        };

        merge(self.fee_payer, true, true);
        for instruction in &self.instructions {
            merge(instruction.program_id, false, false);
            for account_meta in &instruction.accounts {
                merge(
                    account_meta.pubkey,
                    account_meta.is_signer,
                    account_meta.is_writable,
                );
            }
        }

        let mut static_keys: [Vec<Pubkey>; 4] = Default::default();
        let mut lookup_writable: Vec<Vec<(Pubkey, u8)>> =
            vec![Vec::new(); address_lookup_tables.len()];
        let mut lookup_readonly: Vec<Vec<(Pubkey, u8)>> =
            vec![Vec::new(); address_lookup_tables.len()];

        for pubkey in &order {
            let (is_signer, is_writable) = flags
                .get(pubkey)
                .copied()
                .ok_or(SolanaError::InvalidMessage)?;

            if is_signer || program_ids.contains(pubkey) || !lookup_map.contains_key(pubkey) {
                let bucket = match (is_signer, is_writable) {
                    (true, true) => 0,
                    (true, false) => 1,
                    (false, true) => 2,
                    (false, false) => 3,
                };
                static_keys[bucket].push(*pubkey);
            } else {
                let (table_index, entry_index) = lookup_map
                    .get(pubkey)
                    .copied()
                    .ok_or(SolanaError::InvalidMessage)?;
                if is_writable {
                    lookup_writable[table_index].push((*pubkey, entry_index));
                } else {
                    lookup_readonly[table_index].push((*pubkey, entry_index));
                }
            }
        }

        let mut account_keys = Vec::with_capacity(static_keys.iter().map(Vec::len).sum());
        account_keys.push(self.fee_payer);

        let mut other_writable_signers: Vec<Pubkey> = static_keys[0]
            .iter()
            .copied()
            .filter(|pubkey| *pubkey != self.fee_payer)
            .collect();
        other_writable_signers.sort_unstable();
        account_keys.extend(other_writable_signers);

        for bucket in &static_keys[1..] {
            let mut sorted_bucket = bucket.clone();
            sorted_bucket.sort_unstable();
            account_keys.extend(sorted_bucket);
        }

        if account_keys.len() > u8::MAX as usize {
            return Err(SolanaError::InvalidMessage);
        }

        let header = MessageHeader {
            num_required_signatures: (static_keys[0].len() + static_keys[1].len()) as u8,
            num_readonly_signed_accounts: static_keys[1].len() as u8,
            num_readonly_unsigned_accounts: static_keys[3].len() as u8,
        };

        let mut virtual_index_map: HashMap<Pubkey, u8> = HashMap::new();
        let mut next_virtual_index = account_keys.len();
        for (pubkey, _) in lookup_writable
            .iter()
            .flat_map(|entries| entries.iter())
            .chain(lookup_readonly.iter().flat_map(|entries| entries.iter()))
        {
            let virtual_index =
                u8::try_from(next_virtual_index).map_err(|_| SolanaError::InvalidMessage)?;
            virtual_index_map.insert(*pubkey, virtual_index);
            next_virtual_index += 1;
        }

        let address_table_lookups: Vec<MessageAddressTableLookup> = address_lookup_tables
            .iter()
            .enumerate()
            .filter_map(|(table_index, table)| {
                let writable_indexes: Vec<u8> = lookup_writable[table_index]
                    .iter()
                    .map(|(_, entry_index)| *entry_index)
                    .collect();
                let readonly_indexes: Vec<u8> = lookup_readonly[table_index]
                    .iter()
                    .map(|(_, entry_index)| *entry_index)
                    .collect();

                if writable_indexes.is_empty() && readonly_indexes.is_empty() {
                    return None;
                }

                Some(MessageAddressTableLookup::new(
                    table.key,
                    writable_indexes,
                    readonly_indexes,
                ))
            })
            .collect();

        let static_index_map: HashMap<Pubkey, u8> = account_keys
            .iter()
            .enumerate()
            .map(|(index, pubkey)| (*pubkey, index as u8))
            .collect();

        let compiled_instructions: Vec<CompiledInstruction> = self
            .instructions
            .iter()
            .map(|instruction| {
                let program_id_index = static_index_map
                    .get(&instruction.program_id)
                    .copied()
                    .ok_or(SolanaError::InvalidMessage)?;

                let accounts = instruction
                    .accounts
                    .iter()
                    .map(|account_meta| {
                        static_index_map
                            .get(&account_meta.pubkey)
                            .copied()
                            .or_else(|| virtual_index_map.get(&account_meta.pubkey).copied())
                            .ok_or(SolanaError::InvalidMessage)
                    })
                    .collect::<Result<Vec<_>>>()?;

                Ok(CompiledInstruction {
                    program_id_index,
                    accounts,
                    data: instruction.data.clone(),
                })
            })
            .collect::<Result<Vec<_>>>()?;

        let signatures = vec![SignatureBytes::default(); header.num_required_signatures as usize];

        Ok(VersionedTransaction::V0 {
            signatures,
            message: VersionedMessageV0 {
                header,
                account_keys,
                recent_blockhash: self.recent_blockhash,
                instructions: compiled_instructions,
                address_table_lookups,
            },
        })
    }

    /// One-shot helper for compiling a V0 transaction.
    pub fn build_v0_transaction(
        fee_payer: Pubkey,
        recent_blockhash: [u8; 32],
        instructions: &[Instruction],
        address_lookup_tables: &[AddressLookupTableAccount],
    ) -> Result<VersionedTransaction> {
        let mut builder = TransactionBuilder::new(fee_payer, recent_blockhash);
        builder.add_instructions(instructions.iter().cloned());
        builder.build_v0(address_lookup_tables)
    }
}
