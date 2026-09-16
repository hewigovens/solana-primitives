//! Instruction lists used to generate [`super::vectors`].

use super::{key, signer};
use crate::instructions::program_ids::{recent_blockhashes_sysvar, system_program, token_program};
use crate::instructions::{associated_token, compute_budget, memo, system, token};
use crate::types::{
    AccountMeta, AddressLookupTableAccount, Instruction, Pubkey, TransactionConfig,
};

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

/// Every v1 config field set.
pub fn full_config() -> TransactionConfig {
    TransactionConfig::new()
        .with_priority_fee(12_345)
        .with_compute_unit_limit(300_000)
        .with_loaded_accounts_data_size_limit(65_536)
        .with_heap_size(65_536)
}

/// A v1 config with a gap in the mask (no fee, no loaded accounts limit).
pub fn partial_config() -> TransactionConfig {
    TransactionConfig::new()
        .with_compute_unit_limit(50_000)
        .with_heap_size(32 * 1024)
}

/// One randomly generated compile case.
pub struct RandomCase {
    pub payer: Pubkey,
    pub instructions: Vec<Instruction>,
    pub lookup_tables: Vec<AddressLookupTableAccount>,
    pub config: TransactionConfig,
    pub blockhash: [u8; 32],
}

/// xorshift64, matching the generator used for the upstream digests.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }

    fn chance(&mut self, percent: u64) -> bool {
        self.below(100) < percent
    }
}

/// 500 random instruction lists over a 40-key pool: random roles, programs that
/// are also accounts, a leading durable nonce 20% of the time, up to three
/// lookup tables with repeated entries, and random (often invalid) v1 configs.
pub fn random_cases() -> Vec<RandomCase> {
    let mut rng = Rng(0x9e37_79b9_7f4a_7c15);
    let pool: Vec<Pubkey> = (0..40).map(|i| key(&format!("k{i}"))).collect();
    let pick = |rng: &mut Rng| pool[rng.below(pool.len() as u64) as usize];
    (0..500)
        .map(|case| {
            let payer = pick(&mut rng);
            let mut instructions = Vec::new();
            if rng.chance(20) {
                let nonce = pick(&mut rng);
                let authority = pick(&mut rng);
                instructions.push(system::advance_nonce_account(&nonce, &authority));
            }
            for _ in 0..1 + rng.below(5) {
                let program_id = if rng.chance(10) {
                    system_program()
                } else {
                    pick(&mut rng)
                };
                let accounts = (0..rng.below(7))
                    .map(|_| {
                        let pubkey = pick(&mut rng);
                        let is_signer = rng.chance(25);
                        let is_writable = rng.chance(50);
                        AccountMeta::new(pubkey, is_signer, is_writable)
                    })
                    .collect();
                let data = (0..rng.below(10)).map(|_| rng.next() as u8).collect();
                instructions.push(Instruction {
                    program_id,
                    accounts,
                    data,
                });
            }
            let lookup_tables = (0..rng.below(4))
                .map(|table| {
                    let addresses = (0..rng.below(25)).map(|_| pick(&mut rng)).collect();
                    AddressLookupTableAccount::new(
                        key(&format!("case{case}table{table}")),
                        addresses,
                    )
                })
                .collect();
            let mut config = TransactionConfig::new();
            if rng.chance(50) {
                config = config.with_priority_fee(rng.next());
            }
            if rng.chance(50) {
                config = config.with_compute_unit_limit(rng.next() as u32);
            }
            if rng.chance(50) {
                config = config.with_loaded_accounts_data_size_limit(rng.next() as u32);
            }
            if rng.chance(50) {
                config = config.with_heap_size(rng.next() as u32);
            }
            RandomCase {
                payer,
                instructions,
                lookup_tables,
                config,
                blockhash: crate::crypto::hash_data(format!("case{case}").as_bytes()),
            }
        })
        .collect()
}
