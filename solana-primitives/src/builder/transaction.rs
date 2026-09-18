use crate::{
    AddressLookupTableAccount, Instruction, Message, MessageV0, MessageV1, Pubkey, Result,
    TransactionConfig, VersionedMessage, VersionedTransaction,
};

/// Collects instructions and compiles them into a transaction.
///
/// Accounts are merged and ordered the same way for every version: the fee
/// payer first, then writable signers, readonly signers, writable non-signers,
/// and readonly non-signers, each group sorted by key bytes (as the Solana SDK
/// does). Built messages are sanitized, and transactions carry placeholder
/// signatures until signed.
#[derive(Debug, Clone)]
pub struct TransactionBuilder {
    fee_payer: Pubkey,
    recent_blockhash: [u8; 32],
    instructions: Vec<Instruction>,
}

impl TransactionBuilder {
    /// Create a new transaction builder
    pub fn new(fee_payer: Pubkey, recent_blockhash: [u8; 32]) -> Self {
        Self {
            fee_payer,
            recent_blockhash,
            instructions: Vec::new(),
        }
    }

    /// Add an instruction to the transaction
    pub fn add_instruction(&mut self, instruction: Instruction) -> &mut Self {
        self.instructions.push(instruction);
        self
    }

    /// Add multiple instructions to the transaction.
    pub fn add_instructions<I>(&mut self, instructions: I) -> &mut Self
    where
        I: IntoIterator<Item = Instruction>,
    {
        self.instructions.extend(instructions);
        self
    }

    /// Build a legacy transaction.
    pub fn build(&self) -> Result<VersionedTransaction> {
        let message =
            Message::try_compile(&self.fee_payer, &self.instructions, self.recent_blockhash)?;
        Self::finish(message.into())
    }

    /// Build a v0 transaction, loading eligible accounts from `address_lookup_tables`.
    ///
    /// See [`MessageV0::try_compile`] for which accounts are looked up.
    pub fn build_v0(
        &self,
        address_lookup_tables: &[AddressLookupTableAccount],
    ) -> Result<VersionedTransaction> {
        let message = MessageV0::try_compile(
            &self.fee_payer,
            &self.instructions,
            address_lookup_tables,
            self.recent_blockhash,
        )?;
        Self::finish(message.into())
    }

    /// Build a v1 transaction with `config`.
    ///
    /// All accounts are inline. Compute Budget instructions are not translated
    /// into `config` (the runtime ignores them for v1), so set the compute unit
    /// limit, loaded accounts data size limit, and fee there; unset values mean
    /// zero, not the legacy defaults.
    pub fn build_v1(&self, config: TransactionConfig) -> Result<VersionedTransaction> {
        let message = MessageV1::try_compile(
            &self.fee_payer,
            &self.instructions,
            self.recent_blockhash,
            config,
        )?;
        Self::finish(message.into())
    }

    fn finish(message: VersionedMessage) -> Result<VersionedTransaction> {
        message.sanitize()?;
        Ok(VersionedTransaction::new(message))
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

#[cfg(test)]
mod tests {
    use super::TransactionBuilder;
    use crate::instructions::system::transfer;
    use crate::test_utils::{base64, blockhash, scenarios, signer, vectors};
    use crate::types::{
        AccountMeta, AddressLookupTableAccount, Instruction, MessageV0, SignatureBytes,
        TransactionConfig, VersionedMessage, VersionedTransaction,
    };
    use crate::{CompileError, Pubkey, SanitizeError, SolanaError};

    fn builder(instructions: Vec<Instruction>) -> TransactionBuilder {
        let mut builder = TransactionBuilder::new(signer("payer").pubkey, blockhash());
        builder.add_instructions(instructions);
        builder
    }

    /// Check a built, unsigned transaction against the upstream signed one: same
    /// message, and the same bytes once signed (or once upstream's signatures are
    /// attached, without the `signing` feature).
    fn assert_matches_upstream(tx: VersionedTransaction, signers: &[&str], expected: &[u8]) {
        let upstream = VersionedTransaction::deserialize(expected).unwrap();
        assert_eq!(tx.serialize_message(), upstream.serialize_message());

        let mut attached = tx.clone();
        for (key, signature) in upstream.account_keys().iter().zip(&upstream.signatures) {
            attached.add_signature(key, *signature).unwrap();
        }
        assert_eq!(attached.serialize().unwrap(), expected);

        #[cfg(feature = "signing")]
        {
            let keys: Vec<[u8; 32]> = signers.iter().map(|s| signer(s).private_key).collect();
            let keys: Vec<&[u8]> = keys.iter().map(|k| &k[..]).collect();
            let mut signed = tx;
            signed.sign(&keys).unwrap();
            assert_eq!(signed.serialize().unwrap(), expected);
            assert_eq!(upstream.verify(), Ok(()));
        }
        #[cfg(not(feature = "signing"))]
        let _ = signers;
    }

    #[test]
    fn legacy_matches_upstream() {
        let cases: [(Vec<Instruction>, &[&str], &[u8]); 3] = [
            (scenarios::simple(), &["payer"], &vectors::LEGACY_SIMPLE_TX),
            (
                scenarios::complex(),
                &["payer", "new_account", "cosigner"],
                &vectors::LEGACY_COMPLEX_TX,
            ),
            (scenarios::nonce(), &["payer"], &vectors::LEGACY_NONCE_TX),
        ];
        for (instructions, signers, expected) in cases {
            let tx = builder(instructions).build().unwrap();
            assert!(!tx.is_signed());
            assert_matches_upstream(tx, signers, expected);
        }
    }

    #[test]
    fn v0_matches_upstream() {
        assert_matches_upstream(
            builder(scenarios::simple()).build_v0(&[]).unwrap(),
            &["payer"],
            &vectors::V0_SIMPLE_TX,
        );
        assert_matches_upstream(
            builder(scenarios::v0_alt())
                .build_v0(&scenarios::lookup_tables())
                .unwrap(),
            &["payer", "cosigner"],
            &vectors::V0_ALT_TX,
        );
        assert_matches_upstream(
            builder(scenarios::v0_nonce())
                .build_v0(&[scenarios::nonce_lookup_table()])
                .unwrap(),
            &["payer"],
            &vectors::V0_NONCE_TX,
        );
    }

    #[test]
    fn v1_matches_upstream() {
        let check = |instructions, config, signers: &[&str], expected: &[u8]| {
            let tx = builder(instructions).build_v1(config).unwrap();
            assert_eq!(tx.message.transaction_config(), Some(&config));
            assert_matches_upstream(tx, signers, expected);
        };
        check(
            scenarios::simple(),
            TransactionConfig::new(),
            &["payer"],
            &vectors::V1_SIMPLE_TX,
        );
        check(
            scenarios::complex_without_compute_budget(),
            scenarios::full_config(),
            &["payer", "new_account", "cosigner"],
            &vectors::V1_COMPLEX_TX,
        );
        check(
            scenarios::nonce(),
            scenarios::partial_config(),
            &["payer"],
            &vectors::V1_NONCE_TX,
        );
        check(
            scenarios::simple(),
            TransactionConfig::new().with_priority_fee(u64::MAX),
            &["payer"],
            &vectors::V1_FEE_ONLY_TX,
        );
    }

    #[test]
    fn build_sanitizes_every_version() {
        let payer = signer("payer").pubkey;
        // Programs cannot be the fee payer in any version.
        let pay_to_program = [Instruction {
            program_id: payer,
            accounts: vec![],
            data: vec![],
        }];
        let invalid_program = Err(SanitizeError::InvalidProgramIndex.into());
        assert_eq!(
            builder(pay_to_program.to_vec()).build().map(|_| ()),
            invalid_program
        );
        assert_eq!(
            builder(pay_to_program.to_vec()).build_v0(&[]).map(|_| ()),
            invalid_program
        );
        assert_eq!(
            builder(pay_to_program.to_vec())
                .build_v1(TransactionConfig::new())
                .map(|_| ()),
            invalid_program
        );
    }

    fn lookup_table_from_sparse_entries(
        table_key: &str,
        entries: &[(u8, &str)],
    ) -> AddressLookupTableAccount {
        let max_index = entries.iter().map(|(index, _)| *index).max().unwrap_or(0) as usize;
        let mut addresses: Vec<Pubkey> = (0..=max_index)
            .map(|entry_index| {
                let mut bytes = [0u8; 32];
                bytes[0] = 0xFE;
                bytes[1] = (entry_index & 0xFF) as u8;
                bytes[2] = ((entry_index >> 8) & 0xFF) as u8;
                Pubkey::new(bytes)
            })
            .collect();

        for (index, value) in entries {
            addresses[*index as usize] = Pubkey::from_base58(value).unwrap();
        }

        AddressLookupTableAccount::new(Pubkey::from_base58(table_key).unwrap(), addresses)
    }

    fn account_meta_for_combined_index(index: usize, pubkey: Pubkey) -> AccountMeta {
        let is_signer = index < 2;
        let is_writable = index < 2 || (2..5).contains(&index) || (13..19).contains(&index);
        AccountMeta::new(pubkey, is_signer, is_writable)
    }

    // Derived from `solana decode-transaction <tx_base64> base64 --output json-compact`.
    fn instruction_from_decoded_tx(
        program_id_index: usize,
        account_indexes: &[u8],
        data_base58: &str,
        combined_accounts: &[AccountMeta],
    ) -> Instruction {
        Instruction {
            program_id: combined_accounts[program_id_index].pubkey,
            accounts: account_indexes
                .iter()
                .map(|index| combined_accounts[*index as usize].clone())
                .collect(),
            data: bs58::decode(data_base58).into_vec().unwrap(),
        }
    }

    /// Resolve compiled v0 instructions back to keys and roles through `tables`.
    fn decompile(message: &MessageV0, tables: &[AddressLookupTableAccount]) -> Vec<Instruction> {
        let lookup_keys = |select: fn(&crate::MessageAddressTableLookup) -> &Vec<u8>| {
            message
                .address_table_lookups
                .iter()
                .flat_map(move |lookup| {
                    let table = tables
                        .iter()
                        .find(|table| table.key == lookup.account_key)
                        .unwrap();
                    select(lookup)
                        .iter()
                        .map(|index| table.addresses[usize::from(*index)])
                })
                .collect::<Vec<_>>()
        };
        let writable_loaded = lookup_keys(|lookup| &lookup.writable_indexes);
        let readonly_loaded = lookup_keys(|lookup| &lookup.readonly_indexes);

        let header = message.header;
        let num_static = message.account_keys.len();
        let num_signers = usize::from(header.num_required_signatures);
        let num_writable_signers = num_signers - usize::from(header.num_readonly_signed_accounts);
        let num_writable_static = num_static - usize::from(header.num_readonly_unsigned_accounts);
        let keys: Vec<Pubkey> = message
            .account_keys
            .iter()
            .chain(&writable_loaded)
            .chain(&readonly_loaded)
            .copied()
            .collect();
        let meta = |index: u8| {
            let index = usize::from(index);
            let is_writable = if index < num_static {
                index < num_writable_signers || (num_signers..num_writable_static).contains(&index)
            } else {
                index < num_static + writable_loaded.len()
            };
            AccountMeta::new(keys[index], index < num_signers, is_writable)
        };
        message
            .instructions
            .iter()
            .map(|instruction| Instruction {
                program_id: keys[usize::from(instruction.program_id_index)],
                accounts: instruction
                    .accounts
                    .iter()
                    .map(|&index| meta(index))
                    .collect(),
                data: instruction.data.clone(),
            })
            .collect()
    }

    /// A Jupiter swap built by web3.js (first-seen key order). The rebuilt message orders
    /// keys like the Solana SDK, so it differs in bytes but must load the same accounts
    /// with the same roles.
    ///
    /// https://solscan.io/tx/2dUtuLXqDEVXppXc6FDP4RRupp2VuHoki8fmR5WqF6aPwAZfcc2wEaRDmjYhhmdDGx6df7kX2ddDhRnfVJvB6egr
    #[test]
    fn v0_rebuilds_real_world_transaction() {
        const REAL_TX_BASE64: &str = "AlF6Dlk4UjQD0xek1R2X8/hcORMjfzZ7/Vmql3hZcmM3+wwWrtvNkbqDFGZqJyFQxlNopEYLGJ3Oo/9gTDqylwOaaKU6sUi0z0x/4AIr2bEbk4F0Bb3eQnlZB2Pd4fwON80kvuBSbQPthCRffekiFXCnIXQUNFcuW3YDiZP0o0oBgAIACA2mI04pxqQuMUitv1NuRlK9ZWJWaV1k+p/LfT3tvKJ+fbIxWsd0GlHg175uFfLQ+Y+1DxMT48DDYU+4V77WYfZ1G4LkfQewG7EXCfCqmCEkGyByWhJU1GOFbK7yr0N338lnQQQP5AeqsFBGoH5xsx9hmNdlxN72v4J91uC6Ksvw/j23WlYbqpa0+YWZyJHXFuu3ghb5vWc1zPY3lpthsJywjJclj04kifG7PRApFI4NgwtaE5na/xCEBI572Nvp+FkAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAbd9uHXZaGT2cvhRs7reawctIXtX1s3kTqM9YV+/wCpBHnVW/IxwG7udMVuzmgVB/2xst6j9I5RArHNola8E49RPixdukLDvYMw2r2DTumX5VA1ifoAVfXgkOTnLswDErQ/+if11/ZKdMCbHylYed5LCas238ndUUsyGqezjOXot/ord4dFsTTM4tnKRq1DlX7l6IZI9NWIfD9sANaw+DcFSlNamSkhBk0k6HFg2jh8fDW13bySu4HkH6hAQQVEjeSccqqE9cRgyD3i0H5PnVvX+q6L+uN2Xdbz16thksnaBgUGABAREwYHAQEGAgECDAIAAACApL8HAAAAAAcBAgERCBoHCQECAwQQFBMICAoIFRYNCQ4PAwQUEwcHFyXBIJszQdacgQMBAAAAWQFkAAGApL8HAAAAAGzHtAAAAAAAyAAACwwAAREYGRMQEhoHBQYonVNwIb8yqyWioGpTHjinCEcrRIIzlc3YWKd5g/z9UQsz2pcScMC7SQwAQjB4YzBiZDEyNDczNjVlM2Q2MTMyM2IxYTYyM2YwMzI0MDUzMzU0Yjk2MTJhODkzNTg3YjdlYTMyNjNlN2JhMTNiNAJ5QE4t+Dvx0UlyGT++v3V9s/1gQI0crEMfwbwNXZBmFgPb3xcF3gjcFuH5CleX0p1W2E0BwNC64/nFjEaXTuuVyg5P1Sf64f/vAgMMAwcDAQIA";
        const STATIC_KEYS: [&str; 13] = [
            "CBXuKTC3JAHjCvUeCXF2mXJazBqATDExQRxZi1iqQcDa",
            "CzbDjxK4wqSpBuKfocC9vUgpzfhVEGPh8EihXbQkophA",
            "2rPmeokZcYM8F3roghsoPYNqSYi32QyrNA9Lm7gN8TDa",
            "7x4VcEX8aLd3kFsNWULTp1qFgVtDwyWSxpTGQkoMM6XX",
            "59v2cSbCsnyaWymLnsq6TWzE6cEN5KJYNTBNrcP4smRH",
            "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL",
            "11111111111111111111111111111111",
            "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA",
            "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4",
            "6U91aKa8pmMxkJwBCfPTmUEfZi6dHe7DcFq2ALvB2tbB",
            "D8cy77BBepLMngZx6ZukaTff5hCt1HrWyKk3Hnd9oitf",
            "DPArtTLbEqa6EuXHfL5UFLBZhFjiEXWRudhvXDrjwXUr",
            "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr",
        ];
        const LOADED_WRITABLE: [&str; 6] = [
            "FLckHLGMJy5gEoXWwcE68Nprde1D4araK4TGLw4pQq2n",
            "5pVN5XZB8cYBjNLFrsBCPWkCQBan5K5Mq2dWGzwPgGJV",
            "9t4P5wMwfFkyn92Z7hf463qYKEZf8ERVZsGBEPNp8uJx",
            "H2DG3qk1cRqBUmRNjJ2fsGrGs47NQk5VRBLt1AevW8m2",
            "6GpvpHXBJA7pW8gP9KEXBJ2spNyydmbY5Q4nbdoo5TeT",
            "4nvJ5zWdVspxJiNZzB127U6amPH98SFFkBx2JZrAduia",
        ];
        const LOADED_READONLY: [&str; 8] = [
            "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v",
            "So11111111111111111111111111111111111111112",
            "TessVdML9pBGgG9yGks7o4HewRaXVAMuoVj4x83GLQH",
            "8ekCy2jHHUbW2yeNGFWYJT9Hm9FW7SvZcZK66dSZCDiF",
            "Sysvar1nstructions1111111111111111111111111",
            "Dodg2HifwU8rmaVVyMyUZDGTRbqAJTyVYxXPwcbNpBKc",
            "7uTT8Xi5RWXzy7h9XL244GRgEycDYDhLjr3ZyNdXi8pZ",
            "99vQwtBwYtrqqD9YSXbdum3KBdxPAVxYTaQ3cfnJSrN2",
        ];

        let original = VersionedTransaction::deserialize(&base64(REAL_TX_BASE64)).unwrap();
        let VersionedMessage::V0(original) = original.message else {
            unreachable!()
        };
        let fee_payer = original.account_keys[0];

        let combined_accounts: Vec<AccountMeta> = STATIC_KEYS
            .iter()
            .chain(LOADED_WRITABLE.iter())
            .chain(LOADED_READONLY.iter())
            .copied()
            .map(|value| Pubkey::from_base58(value).unwrap())
            .enumerate()
            .map(|(index, key)| account_meta_for_combined_index(index, key))
            .collect();

        let lookup_tables = vec![
            lookup_table_from_sparse_entries(
                "9AKCoNoAGYLW71TwTHY9e7KrZUWWL3c7VtHKb66NT3EV",
                &[
                    (219, "FLckHLGMJy5gEoXWwcE68Nprde1D4araK4TGLw4pQq2n"),
                    (223, "5pVN5XZB8cYBjNLFrsBCPWkCQBan5K5Mq2dWGzwPgGJV"),
                    (23, "9t4P5wMwfFkyn92Z7hf463qYKEZf8ERVZsGBEPNp8uJx"),
                    (222, "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
                    (8, "So11111111111111111111111111111111111111112"),
                    (220, "TessVdML9pBGgG9yGks7o4HewRaXVAMuoVj4x83GLQH"),
                    (22, "8ekCy2jHHUbW2yeNGFWYJT9Hm9FW7SvZcZK66dSZCDiF"),
                    (225, "Sysvar1nstructions1111111111111111111111111"),
                ],
            ),
            lookup_table_from_sparse_entries(
                "Hm9fUgcn7qwDaiNTFiGh6pNtVATgnaRcmK6Bbx6EMZfP",
                &[
                    (12, "H2DG3qk1cRqBUmRNjJ2fsGrGs47NQk5VRBLt1AevW8m2"),
                    (3, "6GpvpHXBJA7pW8gP9KEXBJ2spNyydmbY5Q4nbdoo5TeT"),
                    (7, "4nvJ5zWdVspxJiNZzB127U6amPH98SFFkBx2JZrAduia"),
                    (1, "Dodg2HifwU8rmaVVyMyUZDGTRbqAJTyVYxXPwcbNpBKc"),
                    (2, "7uTT8Xi5RWXzy7h9XL244GRgEycDYDhLjr3ZyNdXi8pZ"),
                    (0, "99vQwtBwYtrqqD9YSXbdum3KBdxPAVxYTaQ3cfnJSrN2"),
                ],
            ),
        ];

        let instructions = vec![
            instruction_from_decoded_tx(5, &[0, 16, 17, 19, 6, 7], "2", &combined_accounts),
            instruction_from_decoded_tx(6, &[1, 2], "3Bxs4NNfTBw5NH5H", &combined_accounts),
            instruction_from_decoded_tx(7, &[2], "J", &combined_accounts),
            instruction_from_decoded_tx(
                8,
                &[
                    7, 9, 1, 2, 3, 4, 16, 20, 19, 8, 8, 10, 8, 21, 22, 13, 9, 14, 15, 3, 4, 20, 19,
                    7, 7, 23,
                ],
                "7UR2vxkjV6WhbmWvkCZQvQJKVhT964yPqVRoTBAPv678iyHS8LF",
                &combined_accounts,
            ),
            instruction_from_decoded_tx(
                11,
                &[0, 1, 17, 24, 25, 19, 16, 18, 26, 7, 5, 6],
                "8pPpkivb1mTLA5APTUWQU2CsG1oYxcnh5C8fQsYux2BVVxSXuFvWtLx",
                &combined_accounts,
            ),
            instruction_from_decoded_tx(
                12,
                &[],
                "KszMTKrqxdHWZULtjrD9cmodXEnC1UboEfkgMRLSGuuPLDYWo8BrqcbfRddG4w18gsf1sZR69vK1mKhXyvNCTZxwsq",
                &combined_accounts,
            ),
        ];

        let mut builder = TransactionBuilder::new(fee_payer, original.recent_blockhash);
        builder.add_instructions(instructions.clone());
        let rebuilt = builder.build_v0(&lookup_tables).unwrap();
        let VersionedMessage::V0(rebuilt) = rebuilt.message else {
            unreachable!()
        };

        assert_eq!(rebuilt.header, original.header);
        assert_eq!(decompile(&original, &lookup_tables), instructions);
        assert_eq!(decompile(&rebuilt, &lookup_tables), instructions);

        let mut original_static = original.account_keys.clone();
        let mut rebuilt_static = rebuilt.account_keys.clone();
        original_static.sort();
        rebuilt_static.sort();
        assert_eq!(rebuilt_static, original_static);
        assert_eq!(
            rebuilt.address_table_lookups.len(),
            original.address_table_lookups.len()
        );
    }

    #[test]
    fn rejects_account_index_overflow() {
        fn distinct_pubkey(index: u32) -> Pubkey {
            let mut bytes = [0u8; 32];
            bytes[0..4].copy_from_slice(&index.to_le_bytes());
            Pubkey::new(bytes)
        }
        fn builder_with(accounts: Vec<AccountMeta>) -> TransactionBuilder {
            let mut builder = TransactionBuilder::new(distinct_pubkey(0), blockhash());
            builder.add_instruction(Instruction {
                program_id: distinct_pubkey(1),
                accounts,
                data: vec![],
            });
            builder
        }
        let overflow = Err(SolanaError::Compile(CompileError::AccountIndexOverflow));

        // 255 other accounts plus the payer and the program exceed 256 keys.
        let writable = builder_with(
            (2..257)
                .map(|index| AccountMeta::new_writable(distinct_pubkey(index)))
                .collect(),
        );
        assert_eq!(writable.build().map(|_| ()), overflow);
        assert_eq!(writable.build_v0(&[]).map(|_| ()), overflow);

        // 255 signers plus the fee payer would wrap to 0 as a u8.
        let signers = builder_with(
            (1..256)
                .map(|index| AccountMeta::new_signer_writable(distinct_pubkey(index)))
                .collect(),
        );
        assert_eq!(signers.build().map(|_| ()), overflow);
    }

    #[test]
    fn add_instructions_appends_in_order() {
        let payer = signer("payer").pubkey;
        let recipient = Pubkey::new([7; 32]);
        let mut builder = TransactionBuilder::new(payer, blockhash());
        builder
            .add_instruction(transfer(&payer, &recipient, 1))
            .add_instructions([transfer(&payer, &recipient, 2)]);
        let tx = builder.build().unwrap();
        assert_eq!(tx.instructions().len(), 2);
        assert_eq!(tx.instructions()[1].data[4], 2);
        assert_eq!(tx.signatures, vec![SignatureBytes::default()]);
    }
}
