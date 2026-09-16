//! Instruction lists used to generate [`super::vectors`].

use super::{key, signer};
use crate::instructions::program_ids::{recent_blockhashes_sysvar, token_program};
use crate::instructions::{associated_token, compute_budget, memo, system, token};
use crate::types::{AccountMeta, AddressLookupTableAccount, Instruction};

/// One SOL transfer from `payer`.
pub fn simple() -> Vec<Instruction> {
    vec![system::transfer(
        &signer("payer").pubkey,
        &key("recipient"),
        1_000_000,
    )]
}

/// Mint creation with an ATA, a custom instruction that merges account roles, and a memo.
pub fn complex_without_compute_budget() -> Vec<Instruction> {
    let payer = signer("payer").pubkey;
    let mint = signer("new_account").pubkey;
    let cosigner = signer("cosigner").pubkey;
    vec![
        system::create_account(&payer, &mint, 1_461_600, 82, &token_program()),
        token::initialize_mint(
            &mint,
            &key("mint_authority"),
            Some(&key("freeze_authority")),
            6,
        ),
        associated_token::create_associated_token_account_idempotent(
            &payer,
            &key("wallet"),
            &mint,
            &token_program(),
        ),
        Instruction {
            program_id: key("program"),
            accounts: vec![
                AccountMeta::new_signer(cosigner),
                AccountMeta::new_writable(key("recipient")),
                AccountMeta::new_readonly(key("program")),
                AccountMeta::new_writable(payer),
                AccountMeta::new_readonly(key("recipient")),
            ],
            data: vec![1, 2, 3],
        },
        memo::memo("hello", &[&cosigner]),
    ]
}

/// [`complex_without_compute_budget`] preceded by compute budget instructions.
pub fn complex() -> Vec<Instruction> {
    let mut instructions = vec![
        compute_budget::set_compute_unit_limit(200_000),
        compute_budget::set_compute_unit_price(5_000),
    ];
    instructions.extend(complex_without_compute_budget());
    instructions
}

/// A durable-nonce transfer.
pub fn nonce() -> Vec<Instruction> {
    let payer = signer("payer").pubkey;
    vec![
        system::advance_nonce_account(&key("nonce"), &payer),
        system::transfer(&payer, &key("recipient"), 42),
    ]
}

/// Instructions whose keys are partly available in [`lookup_tables`].
pub fn v0_alt() -> Vec<Instruction> {
    let payer = signer("payer").pubkey;
    vec![
        compute_budget::set_compute_unit_price(10_000),
        system::transfer(&payer, &key("alt_writable_a"), 5),
        Instruction {
            program_id: key("program"),
            accounts: vec![
                AccountMeta::new_readonly(key("alt_readonly_a")),
                AccountMeta::new_writable(key("alt_writable_b")),
                AccountMeta::new_signer(signer("cosigner").pubkey),
                AccountMeta::new_readonly(key("static_ro")),
                AccountMeta::new_readonly(key("alt_both")),
                AccountMeta::new_writable(key("dup_in_table")),
            ],
            data: vec![9, 9],
        },
    ]
}

/// Tables that also list a program, a signer, keys present in two tables or twice
/// in one table, and a table with no usable keys.
pub fn lookup_tables() -> Vec<AddressLookupTableAccount> {
    vec![
        AddressLookupTableAccount::new(
            key("table1"),
            vec![
                key("filler0"),
                key("alt_readonly_a"),
                key("program"),
                key("alt_both"),
                key("alt_writable_a"),
                signer("cosigner").pubkey,
            ],
        ),
        AddressLookupTableAccount::new(
            key("table2"),
            vec![
                key("alt_both"),
                key("dup_in_table"),
                key("alt_writable_b"),
                key("dup_in_table"),
            ],
        ),
        AddressLookupTableAccount::new(key("table3"), vec![key("unused")]),
    ]
}

/// A durable-nonce transfer whose nonce account is also in [`nonce_lookup_table`].
pub fn v0_nonce() -> Vec<Instruction> {
    let payer = signer("payer").pubkey;
    vec![
        system::advance_nonce_account(&key("nonce"), &payer),
        system::transfer(&payer, &key("alt_writable_a"), 7),
    ]
}

pub fn nonce_lookup_table() -> AddressLookupTableAccount {
    AddressLookupTableAccount::new(
        key("table_nonce"),
        vec![
            key("nonce"),
            key("alt_writable_a"),
            recent_blockhashes_sysvar(),
        ],
    )
}
