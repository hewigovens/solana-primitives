use crate::instructions::program_ids::{recent_blockhashes_sysvar, rent_sysvar, system_program};
use crate::types::{AccountMeta, Instruction, Pubkey};

/// Serialized size of a nonce account (`nonce::state::Versions`).
pub const NONCE_STATE_SIZE: u64 = 80;

/// System program instructions.
///
/// [`SystemInstruction::serialize`] produces the program's bincode encoding:
/// a `u32` LE variant index, fixed-width LE integers, raw 32-byte pubkeys, and
/// strings as a `u64` LE byte length followed by the UTF-8 bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SystemInstruction {
    /// Create a new account.
    ///
    /// 0. `[WRITE, SIGNER]` Funding account
    /// 1. `[WRITE, SIGNER]` New account
    CreateAccount {
        lamports: u64,
        space: u64,
        owner: Pubkey,
    },
    /// Assign account to a program.
    ///
    /// 0. `[WRITE, SIGNER]` Assigned account
    Assign { owner: Pubkey },
    /// Transfer lamports.
    ///
    /// 0. `[WRITE, SIGNER]` Funding account
    /// 1. `[WRITE]` Recipient account
    Transfer { lamports: u64 },
    /// Create a new account at an address derived from a base pubkey and a seed.
    ///
    /// 0. `[WRITE, SIGNER]` Funding account
    /// 1. `[WRITE]` Created account
    /// 2. `[SIGNER]` (optional) Base account, omitted when it is the funding account
    CreateAccountWithSeed {
        base: Pubkey,
        seed: String,
        lamports: u64,
        space: u64,
        owner: Pubkey,
    },
    /// Consume a stored nonce, replacing it with a successor.
    ///
    /// 0. `[WRITE]` Nonce account
    /// 1. `[]` RecentBlockhashes sysvar
    /// 2. `[SIGNER]` Nonce authority
    AdvanceNonceAccount,
    /// Withdraw lamports from a nonce account.
    ///
    /// 0. `[WRITE]` Nonce account
    /// 1. `[WRITE]` Recipient account
    /// 2. `[]` RecentBlockhashes sysvar
    /// 3. `[]` Rent sysvar
    /// 4. `[SIGNER]` Nonce authority
    WithdrawNonceAccount(u64),
    /// Initialize a nonce account with the given authority.
    ///
    /// 0. `[WRITE]` Nonce account
    /// 1. `[]` RecentBlockhashes sysvar
    /// 2. `[]` Rent sysvar
    InitializeNonceAccount(Pubkey),
    /// Change the nonce authority.
    ///
    /// 0. `[WRITE]` Nonce account
    /// 1. `[SIGNER]` Nonce authority
    AuthorizeNonceAccount(Pubkey),
    /// Allocate space in an account without funding it.
    ///
    /// 0. `[WRITE, SIGNER]` New account
    Allocate { space: u64 },
    /// Allocate space for and assign an account derived from a base pubkey and a seed.
    ///
    /// 0. `[WRITE]` Allocated account
    /// 1. `[SIGNER]` Base account
    AllocateWithSeed {
        base: Pubkey,
        seed: String,
        space: u64,
        owner: Pubkey,
    },
    /// Assign an account derived from a base pubkey and a seed to a program.
    ///
    /// 0. `[WRITE]` Assigned account
    /// 1. `[SIGNER]` Base account
    AssignWithSeed {
        base: Pubkey,
        seed: String,
        owner: Pubkey,
    },
    /// Transfer lamports from an account derived from a base pubkey and a seed.
    ///
    /// 0. `[WRITE]` Funding account
    /// 1. `[SIGNER]` Base for the funding account
    /// 2. `[WRITE]` Recipient account
    TransferWithSeed {
        lamports: u64,
        from_seed: String,
        from_owner: Pubkey,
    },
    /// One-time idempotent upgrade of a legacy nonce account.
    ///
    /// 0. `[WRITE]` Nonce account
    UpgradeNonceAccount,
}

impl SystemInstruction {
    fn index(&self) -> u32 {
        match self {
            Self::CreateAccount { .. } => 0,
            Self::Assign { .. } => 1,
            Self::Transfer { .. } => 2,
            Self::CreateAccountWithSeed { .. } => 3,
            Self::AdvanceNonceAccount => 4,
            Self::WithdrawNonceAccount(_) => 5,
            Self::InitializeNonceAccount(_) => 6,
            Self::AuthorizeNonceAccount(_) => 7,
            Self::Allocate { .. } => 8,
            Self::AllocateWithSeed { .. } => 9,
            Self::AssignWithSeed { .. } => 10,
            Self::TransferWithSeed { .. } => 11,
            Self::UpgradeNonceAccount => 12,
        }
    }

    /// The serialized size of the instruction data in bytes.
    pub fn size(&self) -> usize {
        const TAG: usize = 4;
        const U64: usize = 8;
        const KEY: usize = 32;
        let string = |s: &str| U64 + s.len();
        TAG + match self {
            Self::CreateAccount { .. } => U64 + U64 + KEY,
            Self::Assign { .. } => KEY,
            Self::Transfer { .. } => U64,
            Self::CreateAccountWithSeed { seed, .. } => KEY + string(seed) + U64 + U64 + KEY,
            Self::AdvanceNonceAccount | Self::UpgradeNonceAccount => 0,
            Self::WithdrawNonceAccount(_) => U64,
            Self::InitializeNonceAccount(_) | Self::AuthorizeNonceAccount(_) => KEY,
            Self::Allocate { .. } => U64,
            Self::AllocateWithSeed { seed, .. } => KEY + string(seed) + U64 + KEY,
            Self::AssignWithSeed { seed, .. } => KEY + string(seed) + KEY,
            Self::TransferWithSeed { from_seed, .. } => U64 + string(from_seed) + KEY,
        }
    }

    /// Serialize the instruction data.
    pub fn serialize(&self) -> Vec<u8> {
        fn put_string(data: &mut Vec<u8>, s: &str) {
            data.extend_from_slice(&(s.len() as u64).to_le_bytes());
            data.extend_from_slice(s.as_bytes());
        }

        let mut data = Vec::with_capacity(self.size());
        data.extend_from_slice(&self.index().to_le_bytes());
        match self {
            Self::CreateAccount {
                lamports,
                space,
                owner,
            } => {
                data.extend_from_slice(&lamports.to_le_bytes());
                data.extend_from_slice(&space.to_le_bytes());
                data.extend_from_slice(owner.as_bytes());
            }
            Self::Assign { owner } => data.extend_from_slice(owner.as_bytes()),
            Self::Transfer { lamports } | Self::WithdrawNonceAccount(lamports) => {
                data.extend_from_slice(&lamports.to_le_bytes());
            }
            Self::CreateAccountWithSeed {
                base,
                seed,
                lamports,
                space,
                owner,
            } => {
                data.extend_from_slice(base.as_bytes());
                put_string(&mut data, seed);
                data.extend_from_slice(&lamports.to_le_bytes());
                data.extend_from_slice(&space.to_le_bytes());
                data.extend_from_slice(owner.as_bytes());
            }
            Self::AdvanceNonceAccount | Self::UpgradeNonceAccount => {}
            Self::InitializeNonceAccount(authority) | Self::AuthorizeNonceAccount(authority) => {
                data.extend_from_slice(authority.as_bytes());
            }
            Self::Allocate { space } => data.extend_from_slice(&space.to_le_bytes()),
            Self::AllocateWithSeed {
                base,
                seed,
                space,
                owner,
            } => {
                data.extend_from_slice(base.as_bytes());
                put_string(&mut data, seed);
                data.extend_from_slice(&space.to_le_bytes());
                data.extend_from_slice(owner.as_bytes());
            }
            Self::AssignWithSeed { base, seed, owner } => {
                data.extend_from_slice(base.as_bytes());
                put_string(&mut data, seed);
                data.extend_from_slice(owner.as_bytes());
            }
            Self::TransferWithSeed {
                lamports,
                from_seed,
                from_owner,
            } => {
                data.extend_from_slice(&lamports.to_le_bytes());
                put_string(&mut data, from_seed);
                data.extend_from_slice(from_owner.as_bytes());
            }
        }
        data
    }
}

/// Instruction data prefix of `SystemInstruction::AdvanceNonceAccount`.
const ADVANCE_NONCE_ACCOUNT_DATA: [u8; 4] = [4, 0, 0, 0];

/// Whether System program instruction data is `AdvanceNonceAccount`.
pub(crate) fn is_advance_nonce_instruction_data(data: &[u8]) -> bool {
    data.starts_with(&ADVANCE_NONCE_ACCOUNT_DATA)
}

/// Whether `instruction` is a System `AdvanceNonceAccount`, which marks a durable-nonce
/// transaction when it is the first instruction.
pub(crate) fn is_advance_nonce_instruction(instruction: &Instruction) -> bool {
    instruction.program_id == system_program()
        && is_advance_nonce_instruction_data(&instruction.data)
}

/// The nonce account of a durable-nonce instruction list, if any.
pub(crate) fn durable_nonce_account(instructions: &[Instruction]) -> Option<Pubkey> {
    instructions
        .first()
        .filter(|instruction| is_advance_nonce_instruction(instruction))
        .and_then(|instruction| instruction.accounts.first())
        .map(|meta| meta.pubkey)
}

fn system_instruction(instruction: SystemInstruction, accounts: Vec<AccountMeta>) -> Instruction {
    Instruction {
        program_id: system_program(),
        accounts,
        data: instruction.serialize(),
    }
}

/// Create a new account.
pub fn create_account(
    from_pubkey: &Pubkey,
    to_pubkey: &Pubkey,
    lamports: u64,
    space: u64,
    owner: &Pubkey,
) -> Instruction {
    system_instruction(
        SystemInstruction::CreateAccount {
            lamports,
            space,
            owner: *owner,
        },
        vec![
            AccountMeta::new_signer_writable(*from_pubkey),
            AccountMeta::new_signer_writable(*to_pubkey),
        ],
    )
}

/// Create a new account at an address derived with
/// [`create_with_seed`](crate::create_with_seed)`(base, seed, owner)`.
pub fn create_account_with_seed(
    from_pubkey: &Pubkey,
    to_pubkey: &Pubkey,
    base: &Pubkey,
    seed: &str,
    lamports: u64,
    space: u64,
    owner: &Pubkey,
) -> Instruction {
    let mut accounts = vec![
        AccountMeta::new_signer_writable(*from_pubkey),
        AccountMeta::new_writable(*to_pubkey),
    ];
    if base != from_pubkey {
        accounts.push(AccountMeta::new_signer(*base));
    }
    system_instruction(
        SystemInstruction::CreateAccountWithSeed {
            base: *base,
            seed: seed.to_string(),
            lamports,
            space,
            owner: *owner,
        },
        accounts,
    )
}

/// Assign an account to a program.
pub fn assign(pubkey: &Pubkey, owner: &Pubkey) -> Instruction {
    system_instruction(
        SystemInstruction::Assign { owner: *owner },
        vec![AccountMeta::new_signer_writable(*pubkey)],
    )
}

/// Assign an account derived from `base` and `seed` to a program.
pub fn assign_with_seed(pubkey: &Pubkey, base: &Pubkey, seed: &str, owner: &Pubkey) -> Instruction {
    system_instruction(
        SystemInstruction::AssignWithSeed {
            base: *base,
            seed: seed.to_string(),
            owner: *owner,
        },
        vec![
            AccountMeta::new_writable(*pubkey),
            AccountMeta::new_signer(*base),
        ],
    )
}

/// Transfer lamports from one account to another.
pub fn transfer(from_pubkey: &Pubkey, to_pubkey: &Pubkey, lamports: u64) -> Instruction {
    system_instruction(
        SystemInstruction::Transfer { lamports },
        vec![
            AccountMeta::new_signer_writable(*from_pubkey),
            AccountMeta::new_writable(*to_pubkey),
        ],
    )
}

/// Transfer lamports from an account derived from `from_base` and `from_seed`.
pub fn transfer_with_seed(
    from_pubkey: &Pubkey,
    from_base: &Pubkey,
    from_seed: &str,
    from_owner: &Pubkey,
    to_pubkey: &Pubkey,
    lamports: u64,
) -> Instruction {
    system_instruction(
        SystemInstruction::TransferWithSeed {
            lamports,
            from_seed: from_seed.to_string(),
            from_owner: *from_owner,
        },
        vec![
            AccountMeta::new_writable(*from_pubkey),
            AccountMeta::new_signer(*from_base),
            AccountMeta::new_writable(*to_pubkey),
        ],
    )
}

/// Allocate space in an account without funding it.
pub fn allocate(pubkey: &Pubkey, space: u64) -> Instruction {
    system_instruction(
        SystemInstruction::Allocate { space },
        vec![AccountMeta::new_signer_writable(*pubkey)],
    )
}

/// Allocate space for and assign an account derived from `base` and `seed`.
pub fn allocate_with_seed(
    pubkey: &Pubkey,
    base: &Pubkey,
    seed: &str,
    space: u64,
    owner: &Pubkey,
) -> Instruction {
    system_instruction(
        SystemInstruction::AllocateWithSeed {
            base: *base,
            seed: seed.to_string(),
            space,
            owner: *owner,
        },
        vec![
            AccountMeta::new_writable(*pubkey),
            AccountMeta::new_signer(*base),
        ],
    )
}

/// Advance a nonce account. A durable-nonce transaction must use this as its first instruction.
pub fn advance_nonce_account(nonce_pubkey: &Pubkey, authorized_pubkey: &Pubkey) -> Instruction {
    system_instruction(
        SystemInstruction::AdvanceNonceAccount,
        vec![
            AccountMeta::new_writable(*nonce_pubkey),
            AccountMeta::new_readonly(recent_blockhashes_sysvar()),
            AccountMeta::new_signer(*authorized_pubkey),
        ],
    )
}

/// Withdraw lamports from a nonce account.
pub fn withdraw_nonce_account(
    nonce_pubkey: &Pubkey,
    authorized_pubkey: &Pubkey,
    to_pubkey: &Pubkey,
    lamports: u64,
) -> Instruction {
    system_instruction(
        SystemInstruction::WithdrawNonceAccount(lamports),
        vec![
            AccountMeta::new_writable(*nonce_pubkey),
            AccountMeta::new_writable(*to_pubkey),
            AccountMeta::new_readonly(recent_blockhashes_sysvar()),
            AccountMeta::new_readonly(rent_sysvar()),
            AccountMeta::new_signer(*authorized_pubkey),
        ],
    )
}

/// Create and initialize a nonce account.
pub fn create_nonce_account(
    from_pubkey: &Pubkey,
    nonce_pubkey: &Pubkey,
    authority_pubkey: &Pubkey,
    lamports: u64,
) -> Vec<Instruction> {
    vec![
        create_account(
            from_pubkey,
            nonce_pubkey,
            lamports,
            NONCE_STATE_SIZE,
            &system_program(),
        ),
        initialize_nonce_account(nonce_pubkey, authority_pubkey),
    ]
}

/// Create and initialize a nonce account at an address derived from `base` and `seed`.
pub fn create_nonce_account_with_seed(
    from_pubkey: &Pubkey,
    nonce_pubkey: &Pubkey,
    base: &Pubkey,
    seed: &str,
    authority_pubkey: &Pubkey,
    lamports: u64,
) -> Vec<Instruction> {
    vec![
        create_account_with_seed(
            from_pubkey,
            nonce_pubkey,
            base,
            seed,
            lamports,
            NONCE_STATE_SIZE,
            &system_program(),
        ),
        initialize_nonce_account(nonce_pubkey, authority_pubkey),
    ]
}

/// Initialize a nonce account. Must be in the same transaction that creates the account.
pub fn initialize_nonce_account(nonce_pubkey: &Pubkey, authority_pubkey: &Pubkey) -> Instruction {
    system_instruction(
        SystemInstruction::InitializeNonceAccount(*authority_pubkey),
        vec![
            AccountMeta::new_writable(*nonce_pubkey),
            AccountMeta::new_readonly(recent_blockhashes_sysvar()),
            AccountMeta::new_readonly(rent_sysvar()),
        ],
    )
}

/// Change the authority of a nonce account.
pub fn authorize_nonce_account(
    nonce_pubkey: &Pubkey,
    authority_pubkey: &Pubkey,
    new_authority_pubkey: &Pubkey,
) -> Instruction {
    system_instruction(
        SystemInstruction::AuthorizeNonceAccount(*new_authority_pubkey),
        vec![
            AccountMeta::new_writable(*nonce_pubkey),
            AccountMeta::new_signer(*authority_pubkey),
        ],
    )
}

/// Upgrade a legacy nonce account.
pub fn upgrade_nonce_account(nonce_pubkey: &Pubkey) -> Instruction {
    system_instruction(
        SystemInstruction::UpgradeNonceAccount,
        vec![AccountMeta::new_writable(*nonce_pubkey)],
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::{key, meta};
    use hexlit::hex;

    // Vectors from `solana-system-interface` 3.3 (`bincode` feature).
    const SYSTEM: &str = "11111111111111111111111111111111";
    const RECENT_BLOCKHASHES: &str = "SysvarRecentB1ockHashes11111111111111111111";
    const RENT: &str = "SysvarRent111111111111111111111111111111111";
    const FROM: &str = "8ukkSPFecLhdnkzp6s1h6zLSLkPzDPk1oaMuaM6Rxmjr";
    const TO: &str = "7t7yfuynrNBBRtHpwC9Vn2QwAcMxuUE3Y6nt27LP413E";
    const BASE: &str = "Ef37CudiH2EeQegAn9gGUjKrGCwf5ksMzXnSAPpWtv17";
    const NONCE: &str = "96GzYFvs4dEeswTaQFQhvbGi6NHUcNHhEgXnPrRjyF2s";
    const AUTHORITY: &str = "Af2Y56WUFQuTTTYHMCjMozYsDxvTvSM6YQnyv8E6EK3v";
    fn check(ix: &Instruction, accounts: &[AccountMeta], data: &[u8]) {
        assert_eq!(ix.program_id, crate::test_utils::pubkey(SYSTEM));
        assert_eq!(ix.accounts, accounts);
        assert_eq!(ix.data, data);
    }

    #[test]
    fn create_account_matches_upstream() {
        let ix = create_account(&key("from"), &key("to"), 1_000_000_007, 165, &key("owner"));
        check(
            &ix,
            &[meta(FROM, true, true), meta(TO, true, true)],
            &hex!(
                "00000000 07ca9a3b00000000 a500000000000000"
                "4c1029697ee358715d3a14a2add817c4b01651440de808371f78165ac90dc581"
            ),
        );
    }

    #[test]
    fn create_account_with_seed_matches_upstream() {
        let ix = create_account_with_seed(
            &key("from"),
            &key("to"),
            &key("base"),
            "seed-123",
            42,
            80,
            &key("owner"),
        );
        check(
            &ix,
            &[
                meta(FROM, true, true),
                meta(TO, false, true),
                meta(BASE, true, false),
            ],
            &hex!(
                "03000000"
                "cae662172fd450bb0cd710a769079c05bfc5d8e35efa6576edc7d0377afdd4a2"
                "0800000000000000 736565642d313233"
                "2a00000000000000 5000000000000000"
                "4c1029697ee358715d3a14a2add817c4b01651440de808371f78165ac90dc581"
            ),
        );

        // The base account is only listed separately when it isn't the funder.
        let ix = create_account_with_seed(
            &key("from"),
            &key("to"),
            &key("from"),
            "s",
            42,
            80,
            &key("owner"),
        );
        check(
            &ix,
            &[meta(FROM, true, true), meta(TO, false, true)],
            &hex!(
                "03000000"
                "75857a45899985be4c4d941e90b6b396d6c92a4c7437aaf0bf102089fe21379d"
                "0100000000000000 73"
                "2a00000000000000 5000000000000000"
                "4c1029697ee358715d3a14a2add817c4b01651440de808371f78165ac90dc581"
            ),
        );
    }

    #[test]
    fn assign_matches_upstream() {
        check(
            &assign(&key("from"), &key("owner")),
            &[meta(FROM, true, true)],
            &hex!("01000000 4c1029697ee358715d3a14a2add817c4b01651440de808371f78165ac90dc581"),
        );
        check(
            &assign_with_seed(&key("to"), &key("base"), "assign-seed", &key("owner")),
            &[meta(TO, false, true), meta(BASE, true, false)],
            &hex!(
                "0a000000"
                "cae662172fd450bb0cd710a769079c05bfc5d8e35efa6576edc7d0377afdd4a2"
                "0b00000000000000 61737369676e2d73656564"
                "4c1029697ee358715d3a14a2add817c4b01651440de808371f78165ac90dc581"
            ),
        );
    }

    #[test]
    fn transfer_matches_upstream() {
        check(
            &transfer(&key("from"), &key("to"), 123_456_789),
            &[meta(FROM, true, true), meta(TO, false, true)],
            &hex!("02000000 15cd5b0700000000"),
        );
        check(
            &transfer_with_seed(
                &key("from"),
                &key("base"),
                "transfer-seed",
                &key("owner"),
                &key("to"),
                99,
            ),
            &[
                meta(FROM, false, true),
                meta(BASE, true, false),
                meta(TO, false, true),
            ],
            &hex!(
                "0b000000 6300000000000000"
                "0d00000000000000 7472616e736665722d73656564"
                "4c1029697ee358715d3a14a2add817c4b01651440de808371f78165ac90dc581"
            ),
        );
    }

    #[test]
    fn allocate_matches_upstream() {
        check(
            &allocate(&key("from"), 4096),
            &[meta(FROM, true, true)],
            &hex!("08000000 0010000000000000"),
        );
        check(
            &allocate_with_seed(&key("to"), &key("base"), "alloc-seed", 2048, &key("owner")),
            &[meta(TO, false, true), meta(BASE, true, false)],
            &hex!(
                "09000000"
                "cae662172fd450bb0cd710a769079c05bfc5d8e35efa6576edc7d0377afdd4a2"
                "0a00000000000000 616c6c6f632d73656564"
                "0008000000000000"
                "4c1029697ee358715d3a14a2add817c4b01651440de808371f78165ac90dc581"
            ),
        );
    }

    #[test]
    fn nonce_instructions_match_upstream() {
        let (nonce, authority) = (key("nonce"), key("authority"));

        // AdvanceNonceAccount is a unit variant: the authority is only an account meta.
        check(
            &advance_nonce_account(&nonce, &authority),
            &[
                meta(NONCE, false, true),
                meta(RECENT_BLOCKHASHES, false, false),
                meta(AUTHORITY, true, false),
            ],
            &hex!("04000000"),
        );
        check(
            &withdraw_nonce_account(&nonce, &authority, &key("to"), 5_000),
            &[
                meta(NONCE, false, true),
                meta(TO, false, true),
                meta(RECENT_BLOCKHASHES, false, false),
                meta(RENT, false, false),
                meta(AUTHORITY, true, false),
            ],
            &hex!("05000000 8813000000000000"),
        );

        let create = create_nonce_account(&key("from"), &nonce, &authority, 1_447_680);
        check(
            &create[0],
            &[meta(FROM, true, true), meta(NONCE, true, true)],
            &hex!(
                "00000000 0017160000000000 5000000000000000"
                "0000000000000000000000000000000000000000000000000000000000000000"
            ),
        );
        check(
            &create[1],
            &[
                meta(NONCE, false, true),
                meta(RECENT_BLOCKHASHES, false, false),
                meta(RENT, false, false),
            ],
            &hex!("06000000 8f76fd501bb68ef71f4e276bc28f29bce1003b0c2c9d9478de81b5bfc0cde1e9"),
        );
        check(
            &authorize_nonce_account(&nonce, &authority, &key("new_authority")),
            &[meta(NONCE, false, true), meta(AUTHORITY, true, false)],
            &hex!("07000000 0489533a40c0b90efe0346163b6865cc8fff89463e5600c10b44ab366a248d31"),
        );
        check(
            &upgrade_nonce_account(&nonce),
            &[meta(NONCE, false, true)],
            &hex!("0c000000"),
        );
    }

    #[test]
    fn size_matches_serialized_length() {
        let (base, owner) = (key("base"), key("owner"));
        let seed = "seed-of-some-length".to_string();
        let instructions = [
            SystemInstruction::CreateAccount {
                lamports: 1,
                space: 2,
                owner,
            },
            SystemInstruction::Assign { owner },
            SystemInstruction::Transfer { lamports: 1 },
            SystemInstruction::CreateAccountWithSeed {
                base,
                seed: seed.clone(),
                lamports: 1,
                space: 2,
                owner,
            },
            SystemInstruction::AdvanceNonceAccount,
            SystemInstruction::WithdrawNonceAccount(1),
            SystemInstruction::InitializeNonceAccount(owner),
            SystemInstruction::AuthorizeNonceAccount(owner),
            SystemInstruction::Allocate { space: 1 },
            SystemInstruction::AllocateWithSeed {
                base,
                seed: seed.clone(),
                space: 1,
                owner,
            },
            SystemInstruction::AssignWithSeed {
                base,
                seed: seed.clone(),
                owner,
            },
            SystemInstruction::TransferWithSeed {
                lamports: 1,
                from_seed: seed,
                from_owner: owner,
            },
            SystemInstruction::UpgradeNonceAccount,
        ];
        for instruction in instructions {
            assert_eq!(
                instruction.serialize().len(),
                instruction.size(),
                "{instruction:?}"
            );
        }
    }
}
