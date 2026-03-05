#[cfg(test)]
mod tests {
    use crate::Pubkey;
    use crate::builder::{InstructionBuilder, TransactionBuilder};
    use crate::instructions::{
        program_ids::{system_program, token_program},
        system::{create_account, transfer},
        token::transfer_checked,
    };
    use crate::types::instruction::AccountMeta;
    use crate::types::{
        AddressLookupTableAccount, Instruction, SignatureBytes, VersionedTransaction,
    };
    use base64::Engine;
    use base64::engine::general_purpose::STANDARD;

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

        // Build the instruction using builder methods
        let builder_ix = InstructionBuilder::new(program_id)
            .account(source, false, true)
            .account(mint, false, false)
            .account(dest, false, true)
            .account(owner, true, false)
            .build();

        let ix = transfer_checked(&source, &mint, &dest, &owner, amount, decimals);

        // Check program ID
        assert_eq!(builder_ix.program_id, token_program());
        assert_eq!(ix.program_id, token_program());
    }

    #[test]
    fn test_transaction_builder() {
        let recent_blockhash = "9U2ogLjDt479wubHbEtPLGBF84DijmWggA4KoXSwcivd";
        let recent_blockhash_bytes = bs58::decode(recent_blockhash).into_vec().unwrap();
        let fee_payer: Pubkey = "A21o4asMbFHYadqXdLusT9Bvx9xaC5YV9gcaidjqtdXC"
            .parse()
            .unwrap();
        let program_id =
            Pubkey::from_base58("J88B7gmadHzTNGiy54c9Ms8BsEXNdB2fntFyhKpk3qoT").unwrap();
        let data = hex::decode("a3265ce2f3698dc400000070000000000100000014000000514bcb1f9aabb904e6106bd1052b66d2706dbbb701000000006c000000000a00000085fba93ee29c604fa858a351688c01290841eafb19c63a70a475d3c7bc3bef9f000000000000000000008489b9cc07af97add00300000000000000000000000000001e83d2972d3dca3a330d60c2777ee5b8d25683c63fa359116985609830f42054050004002d16000000f0314f0cffdf8d00b6a7ce61f86164ca47c1b8b1bc2e").unwrap();
        let instruction = InstructionBuilder::new(program_id)
            .data(data)
            .accounts(vec![
                AccountMeta::new_readonly(
                    "ACLMuTFvDAb3oecQQGkTVqpUbhCKHG3EZ9uNXHK1W9ka"
                        .parse()
                        .unwrap(),
                ),
                AccountMeta::new_writable(
                    "3tJ67qa2GDfvv2wcMYNUfN5QBZrFpTwcU8ASZKMvCTVU"
                        .parse()
                        .unwrap(),
                ),
                AccountMeta::new_signer_writable(
                    "A21o4asMbFHYadqXdLusT9Bvx9xaC5YV9gcaidjqtdXC"
                        .parse()
                        .unwrap(),
                ),
                AccountMeta::new_writable(
                    "E8p6aiwuSDWEzQnjGjkNiMZrd1rpSsntWsaZCivdFz51"
                        .parse()
                        .unwrap(),
                ),
                AccountMeta::new_writable(
                    "FmAcjWaRFUxGWBfGT7G3CzcFeJFsewQ4KPJVG4f6fcob"
                        .parse()
                        .unwrap(),
                ),
                AccountMeta::new_readonly(system_program()),
            ]);

        let mut tx_builder =
            TransactionBuilder::new(fee_payer, recent_blockhash_bytes.try_into().unwrap());
        tx_builder.add_instruction(instruction.build());

        let transaction = tx_builder.build().unwrap();
        println!("Transaction: {transaction:#?}");

        // Use the new method on Transaction to serialize
        let tx_wire_bytes = transaction
            .serialize_legacy()
            .expect("Failed to serialize transaction with wire format");

        let base64_tx = STANDARD.encode(&tx_wire_bytes);
        println!("Base64 transaction: {base64_tx}");

        // Deserialize and verify
        let deserialized_vt = VersionedTransaction::deserialize_with_version(&tx_wire_bytes)
            .expect("Failed to deserialize wire bytes into VersionedTransaction");

        match deserialized_vt {
            VersionedTransaction::Legacy {
                signatures: deserialized_signatures,
                message: deserialized_legacy_message,
            } => {
                assert_eq!(
                    deserialized_signatures, transaction.signatures,
                    "Signatures mismatch after round trip"
                );
                assert_eq!(
                    deserialized_legacy_message.header, transaction.message.header,
                    "Message header mismatch"
                );
                assert_eq!(
                    deserialized_legacy_message.account_keys, transaction.message.account_keys,
                    "Account keys mismatch"
                );
                assert_eq!(
                    deserialized_legacy_message.recent_blockhash,
                    transaction.message.recent_blockhash,
                    "Recent blockhash mismatch"
                );
                assert_eq!(
                    deserialized_legacy_message.instructions, transaction.message.instructions,
                    "Instructions mismatch"
                );
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

    #[test]
    fn test_versioned_transaction_builder_without_lookup_tables() {
        let fee_payer = payer_pubkey();
        let recipient = random_pubkey();
        let recent_blockhash = test_blockhash();

        let transfer_ix = transfer(&fee_payer, &recipient, 123);

        let mut builder = TransactionBuilder::new(fee_payer, recent_blockhash);
        builder.add_instruction(transfer_ix);

        let transaction = builder.build_v0(&[]).unwrap();
        let wire_bytes = transaction.serialize().unwrap();
        let parsed = VersionedTransaction::deserialize_with_version(&wire_bytes).unwrap();

        match parsed {
            VersionedTransaction::V0 {
                signatures,
                message,
            } => {
                assert_eq!(signatures.len(), 1);
                assert_eq!(message.header.num_required_signatures, 1);
                assert!(message.address_table_lookups.is_empty());
                assert_eq!(message.instructions.len(), 1);
            }
            _ => panic!("expected v0 transaction"),
        }
    }

    #[test]
    fn test_versioned_transaction_builder_with_lookup_table() {
        let fee_payer = payer_pubkey();
        let recent_blockhash = test_blockhash();
        let looked_up_account = Pubkey::new([42u8; 32]);
        let program_id = Pubkey::new([7u8; 32]);

        let instruction = InstructionBuilder::new(program_id)
            .account(fee_payer, true, true)
            .account(looked_up_account, false, true)
            .data(vec![1, 2, 3])
            .build();

        let lookup_table = AddressLookupTableAccount::new(
            Pubkey::new([99u8; 32]),
            vec![looked_up_account, Pubkey::new([11u8; 32])],
        );

        let mut builder = TransactionBuilder::new(fee_payer, recent_blockhash);
        builder.add_instruction(instruction);

        let transaction = builder.build_v0(&[lookup_table]).unwrap();
        let wire_bytes = transaction.serialize().unwrap();
        let parsed = VersionedTransaction::deserialize_with_version(&wire_bytes).unwrap();

        match parsed {
            VersionedTransaction::V0 {
                signatures,
                message,
            } => {
                assert_eq!(signatures.len(), 1);
                assert_eq!(message.address_table_lookups.len(), 1);
                assert_eq!(message.address_table_lookups[0].writable_indexes, vec![0]);
                assert_eq!(
                    message.address_table_lookups[0].readonly_indexes,
                    Vec::<u8>::new()
                );
                assert!(!message.account_keys.contains(&looked_up_account));
                assert_eq!(message.instructions.len(), 1);
                assert_eq!(message.instructions[0].data, vec![1, 2, 3]);
            }
            _ => panic!("expected v0 transaction"),
        }
    }

    #[test]
    fn test_add_instructions_helper() {
        let fee_payer = payer_pubkey();
        let recent_blockhash = test_blockhash();
        let recipient = random_pubkey();

        let ix1 = transfer(&fee_payer, &recipient, 1);
        let ix2 = transfer(&fee_payer, &recipient, 2);

        let mut builder = TransactionBuilder::new(fee_payer, recent_blockhash);
        builder.add_instructions(vec![ix1, ix2]);

        let tx = builder.build().unwrap();
        assert_eq!(tx.message.instructions.len(), 2);
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

    #[test]
    fn test_v0_builder_real_world_regression_case() {
        // https://solscan.io/tx/2dUtuLXqDEVXppXc6FDP4RRupp2VuHoki8fmR5WqF6aPwAZfcc2wEaRDmjYhhmdDGx6df7kX2ddDhRnfVJvB6egr
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

        let wire = STANDARD.decode(REAL_TX_BASE64).unwrap();
        let mut expected_tx = VersionedTransaction::deserialize_with_version(&wire).unwrap();
        for signature in expected_tx.signatures_mut() {
            *signature = SignatureBytes::default();
        }

        let (fee_payer, recent_blockhash) = match &expected_tx {
            VersionedTransaction::V0 { message, .. } => {
                (message.account_keys[0], message.recent_blockhash)
            }
            _ => panic!("expected V0 transaction fixture"),
        };

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

        let mut builder = TransactionBuilder::new(fee_payer, recent_blockhash);
        builder.add_instructions(instructions);
        let rebuilt_tx = builder.build_v0(&lookup_tables).unwrap();

        assert_eq!(
            rebuilt_tx.serialize().unwrap(),
            expected_tx.serialize().unwrap()
        );
    }
}
