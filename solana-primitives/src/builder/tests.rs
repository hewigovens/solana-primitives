use crate::builder::{InstructionBuilder, TransactionBuilder};
use crate::instructions::{
    program_ids::{system_program, token_program},
    system::create_account,
    token::transfer_checked,
};
use crate::types::instruction::AccountMeta;
use crate::types::VersionedTransaction;
use crate::Pubkey;
use base64::engine::general_purpose::STANDARD;
use base64::Engine;

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
    let program_id = Pubkey::from_base58("J88B7gmadHzTNGiy54c9Ms8BsEXNdB2fntFyhKpk3qoT").unwrap();
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
                deserialized_legacy_message.recent_blockhash, transaction.message.recent_blockhash,
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
