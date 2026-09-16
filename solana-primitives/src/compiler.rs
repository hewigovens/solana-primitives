//! Account compilation shared by legacy, v0, and v1 messages.
//!
//! Mirrors the Solana SDK's `CompiledKeys`: every key referenced by the
//! instructions is merged into one role (signer / writable), keys are ordered
//! fee payer first, then writable signers, readonly signers, writable
//! non-signers, and readonly non-signers, each group sorted by key bytes.

use crate::error::CompileError;
use crate::instructions::system::durable_nonce_account;
use crate::types::{
    AddressLookupTableAccount, CompiledInstruction, Instruction, MessageAddressTableLookup,
    MessageHeader, Pubkey,
};
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Default, Clone, Copy)]
struct KeyMeta {
    is_signer: bool,
    is_writable: bool,
    is_invoked: bool,
    is_nonce: bool,
}

impl KeyMeta {
    /// Programs, signers, and the durable nonce account must be static keys.
    fn can_be_looked_up(&self) -> bool {
        !self.is_signer && !self.is_invoked && !self.is_nonce
    }
}

/// Keys loaded from address lookup tables, in lookup order.
#[derive(Debug, Default)]
pub(crate) struct LoadedAddresses {
    pub writable: Vec<Pubkey>,
    pub readonly: Vec<Pubkey>,
}

/// The merged account keys of an instruction list.
#[derive(Debug)]
pub(crate) struct CompiledKeys {
    payer: Pubkey,
    keys: BTreeMap<Pubkey, KeyMeta>,
}

impl CompiledKeys {
    pub fn compile(payer: &Pubkey, instructions: &[Instruction]) -> Self {
        let mut keys = BTreeMap::<Pubkey, KeyMeta>::new();
        for instruction in instructions {
            keys.entry(instruction.program_id).or_default().is_invoked = true;
            for account in &instruction.accounts {
                let meta = keys.entry(account.pubkey).or_default();
                meta.is_signer |= account.is_signer;
                meta.is_writable |= account.is_writable;
            }
        }
        if let Some(nonce) = durable_nonce_account(instructions) {
            keys.entry(nonce).or_default().is_nonce = true;
        }
        let payer_meta = keys.entry(*payer).or_default();
        payer_meta.is_signer = true;
        payer_meta.is_writable = true;
        Self {
            payer: *payer,
            keys,
        }
    }

    /// Move the eligible keys found in `table` out of the static keys.
    ///
    /// Returns `None` if the table provides none of them. Each key uses its
    /// first index in the table.
    pub fn extract_table_lookup(
        &mut self,
        table: &AddressLookupTableAccount,
    ) -> Result<Option<(MessageAddressTableLookup, LoadedAddresses)>, CompileError> {
        let (writable_indexes, writable) =
            self.drain_keys_in_table(&table.addresses, |meta| meta.is_writable)?;
        let (readonly_indexes, readonly) =
            self.drain_keys_in_table(&table.addresses, |meta| !meta.is_writable)?;
        if writable_indexes.is_empty() && readonly_indexes.is_empty() {
            return Ok(None);
        }
        Ok(Some((
            MessageAddressTableLookup {
                account_key: table.key,
                writable_indexes,
                readonly_indexes,
            },
            LoadedAddresses { writable, readonly },
        )))
    }

    fn drain_keys_in_table(
        &mut self,
        addresses: &[Pubkey],
        filter: impl Fn(&KeyMeta) -> bool,
    ) -> Result<(Vec<u8>, Vec<Pubkey>), CompileError> {
        let mut indexes = Vec::new();
        let mut drained = Vec::new();
        for (key, meta) in &self.keys {
            if !meta.can_be_looked_up() || !filter(meta) {
                continue;
            }
            if let Some(index) = addresses.iter().position(|address| address == key) {
                indexes.push(
                    u8::try_from(index)
                        .map_err(|_| CompileError::AddressTableLookupIndexOverflow)?,
                );
                drained.push(*key);
            }
        }
        for key in &drained {
            self.keys.remove(key);
        }
        Ok((indexes, drained))
    }

    /// The header and ordered static keys.
    pub fn into_message_components(mut self) -> Result<(MessageHeader, Vec<Pubkey>), CompileError> {
        self.keys.remove(&self.payer);
        let group = |signer: bool, writable: bool| {
            self.keys
                .iter()
                .filter(move |(_, meta)| meta.is_signer == signer && meta.is_writable == writable)
                .map(|(key, _)| *key)
        };

        let writable_signers = 1 + group(true, true).count();
        let readonly_signers = group(true, false).count();
        let readonly_non_signers = group(false, false).count();
        let count = |n: usize| u8::try_from(n).map_err(|_| CompileError::AccountIndexOverflow);
        let header = MessageHeader {
            num_required_signatures: count(writable_signers + readonly_signers)?,
            num_readonly_signed_accounts: count(readonly_signers)?,
            num_readonly_unsigned_accounts: count(readonly_non_signers)?,
        };

        let account_keys = std::iter::once(self.payer)
            .chain(group(true, true))
            .chain(group(true, false))
            .chain(group(false, true))
            .chain(group(false, false))
            .collect();
        Ok((header, account_keys))
    }
}

/// Compile `instructions` against `static_keys` followed by the loaded writable
/// keys and then the loaded readonly keys of every lookup, in order.
pub(crate) fn compile_instructions(
    instructions: &[Instruction],
    static_keys: &[Pubkey],
    loaded: &[LoadedAddresses],
) -> Result<Vec<CompiledInstruction>, CompileError> {
    let all_keys = static_keys
        .iter()
        .chain(loaded.iter().flat_map(|lookup| &lookup.writable))
        .chain(loaded.iter().flat_map(|lookup| &lookup.readonly));
    let mut index_of = HashMap::new();
    for (index, key) in all_keys.enumerate() {
        index_of.entry(*key).or_insert(index);
    }
    let index = |key: &Pubkey| -> Result<u8, CompileError> {
        let index = index_of
            .get(key)
            .ok_or(CompileError::UnknownInstructionKey(*key))?;
        u8::try_from(*index).map_err(|_| CompileError::AccountIndexOverflow)
    };

    instructions
        .iter()
        .map(|instruction| {
            Ok(CompiledInstruction {
                program_id_index: index(&instruction.program_id)?,
                accounts: instruction
                    .accounts
                    .iter()
                    .map(|account| index(&account.pubkey))
                    .collect::<Result<_, _>>()?,
                data: instruction.data.clone(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::program_ids::system_program;
    use crate::instructions::system::{advance_nonce_account, transfer};
    use crate::test_utils::{key, scenarios};
    use crate::types::{AccountMeta, Message, MessageV0, MessageV1};
    use hexlit::hex;
    use sha2::{Digest, Sha256};

    fn instruction(program_id: Pubkey, accounts: Vec<AccountMeta>) -> Instruction {
        Instruction {
            program_id,
            accounts,
            data: vec![],
        }
    }

    #[test]
    fn merges_roles_and_orders_keys() {
        let payer = Pubkey::new([5; 32]);
        let [a, b, c, d, program] = [1u8, 2, 3, 4, 9].map(|i| Pubkey::new([i; 32]));
        let instructions = [
            instruction(
                program,
                vec![
                    AccountMeta::new_readonly(d),
                    AccountMeta::new_signer(c),
                    AccountMeta::new_writable(b),
                    AccountMeta::new_readonly(payer),
                ],
            ),
            // Later uses promote earlier roles; the program is also an account.
            instruction(
                program,
                vec![
                    AccountMeta::new_signer_writable(a),
                    AccountMeta::new_writable(d),
                    AccountMeta::new_readonly(program),
                ],
            ),
        ];
        let (header, keys) = CompiledKeys::compile(&payer, &instructions)
            .into_message_components()
            .unwrap();
        assert_eq!(
            header,
            MessageHeader {
                num_required_signatures: 3,
                num_readonly_signed_accounts: 1,
                num_readonly_unsigned_accounts: 1,
            }
        );
        // The payer leads even though its bytes sort after `a`..`d`.
        assert_eq!(keys, vec![payer, a, c, b, d, program]);

        let compiled = compile_instructions(&instructions, &keys, &[]).unwrap();
        assert_eq!(compiled[0].program_id_index, 5);
        assert_eq!(compiled[0].accounts, vec![4, 2, 3, 0]);
        assert_eq!(compiled[1].accounts, vec![1, 4, 5]);
    }

    #[test]
    fn lookup_extraction_keeps_static_keys_static() {
        let payer = key("payer");
        let nonce = key("nonce");
        let program = key("program");
        let signer = key("signer");
        let (writable, readonly, unused) = (key("writable"), key("readonly"), key("unused"));
        let instructions = [
            advance_nonce_account(&nonce, &payer),
            instruction(
                program,
                vec![
                    AccountMeta::new_signer(signer),
                    AccountMeta::new_writable(writable),
                    AccountMeta::new_readonly(readonly),
                ],
            ),
        ];
        let table = AddressLookupTableAccount::new(
            key("table"),
            vec![
                unused, readonly, writable, program, signer, nonce, payer, writable,
            ],
        );

        let mut keys = CompiledKeys::compile(&payer, &instructions);
        let (lookup, loaded) = keys.extract_table_lookup(&table).unwrap().unwrap();
        assert_eq!(lookup.writable_indexes, vec![2]);
        assert_eq!(lookup.readonly_indexes, vec![1]);
        assert_eq!(loaded.writable, vec![writable]);
        assert_eq!(loaded.readonly, vec![readonly]);

        // Nothing eligible is left for a second table.
        assert!(keys.extract_table_lookup(&table).unwrap().is_none());

        let (_, static_keys) = keys.into_message_components().unwrap();
        for static_key in [payer, nonce, program, signer] {
            assert!(static_keys.contains(&static_key));
        }
        assert!(!static_keys.contains(&writable));

        let compiled = compile_instructions(&instructions, &static_keys, &[loaded]).unwrap();
        let n = static_keys.len() as u8;
        assert_eq!(compiled[1].accounts[1..], [n, n + 1]);
    }

    #[test]
    fn index_overflows_are_errors() {
        let payer = key("payer");
        let table = AddressLookupTableAccount::new(
            key("table"),
            (0..=256u32)
                .map(|i| Pubkey::new(crate::crypto::hash_data(&i.to_le_bytes())))
                .collect(),
        );
        let late = table.addresses[256];
        let mut keys = CompiledKeys::compile(&payer, &[transfer(&payer, &late, 1)]);
        assert_eq!(
            keys.extract_table_lookup(&table).map(|_| ()),
            Err(CompileError::AddressTableLookupIndexOverflow)
        );

        let unknown = key("unknown");
        assert_eq!(
            compile_instructions(
                &[transfer(&payer, &unknown, 1)],
                &[payer, system_program()],
                &[]
            ),
            Err(CompileError::UnknownInstructionKey(unknown))
        );
    }

    /// SHA-256 over the 500 messages of each version that `solana-message` 5.0
    /// (`Message::new_with_blockhash`, `v0::Message::try_compile`,
    /// `v1::Message::try_compile_with_config`) produced for `scenarios::random_cases`.
    #[test]
    fn random_instructions_compile_like_upstream() {
        let mut digests = [Sha256::new(), Sha256::new(), Sha256::new()];
        for case in scenarios::random_cases() {
            let legacy = Message::try_compile(&case.payer, &case.instructions, case.blockhash);
            let v0 = MessageV0::try_compile(
                &case.payer,
                &case.instructions,
                &case.lookup_tables,
                case.blockhash,
            );
            let v1 = MessageV1::try_compile(
                &case.payer,
                &case.instructions,
                case.blockhash,
                case.config,
            );
            digests[0].update(legacy.unwrap().serialize().unwrap());
            digests[1].update(v0.unwrap().serialize().unwrap());
            digests[2].update(v1.unwrap().serialize().unwrap());
        }
        let [legacy, v0, v1] = digests.map(|digest| <[u8; 32]>::from(digest.finalize()));
        assert_eq!(
            legacy,
            hex!("0a45a0415ee447108daaa0973a70c418ef9539e3faf6af05acf25947495564a1")
        );
        assert_eq!(
            v0,
            hex!("b694634ecb45447547137f7e3567147d0c76d9d21ef7614b0be6dda8ab6164de")
        );
        assert_eq!(
            v1,
            hex!("04bb0830fcfdaf1568ca537dd4d8dd5120496fc617940fa9b88820c552bde040")
        );
    }
}
