//! SPL Token instructions. Each `*_with_program_id` variant also works with Token-2022.

use crate::instructions::program_ids::{rent_sysvar, token_program};
use crate::types::{AccountMeta, Instruction, Pubkey};

/// SPL Token instructions supported by this crate.
///
/// [`TokenInstruction::serialize`] produces the program's packed encoding: a
/// one-byte tag, little-endian integers, raw 32-byte pubkeys, and optional
/// pubkeys as a one-byte flag followed by the key when present.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenInstruction {
    /// Initialize a new mint
    InitializeMint {
        /// Number of base 10 digits to the right of the decimal place
        decimals: u8,
        /// The authority/multisignature to mint tokens
        mint_authority: Pubkey,
        /// The freeze authority/multisignature of the mint
        freeze_authority: Option<Pubkey>,
    },
    /// Initialize a new account
    InitializeAccount,
    /// Initialize a multisignature account
    InitializeMultisig {
        /// The number of signers (M) required to validate this multisignature account
        m: u8,
    },
    /// Transfer tokens
    Transfer {
        /// The amount of tokens to transfer
        amount: u64,
    },
    /// Approve a delegate
    Approve {
        /// The amount of tokens the delegate is approved for
        amount: u64,
    },
    /// Revoke a delegate's authority
    Revoke,
    /// Set a new authority
    SetAuthority {
        /// The type of authority to update
        authority_type: AuthorityType,
        /// The new authority
        new_authority: Option<Pubkey>,
    },
    /// Mint new tokens to an account
    MintTo {
        /// The amount of new tokens to mint
        amount: u64,
    },
    /// Burn tokens from an account
    Burn {
        /// The amount of tokens to burn
        amount: u64,
    },
    /// Close an account by transferring all its SOL to the destination account
    CloseAccount,
    /// Freeze an account
    FreezeAccount,
    /// Thaw a frozen account
    ThawAccount,
    /// Transfer tokens, asserting the token mint and decimals
    TransferChecked {
        /// The amount of tokens to transfer
        amount: u64,
        /// The amount's decimals
        decimals: u8,
    },
    /// Approve a delegate, asserting the token mint and decimals
    ApproveChecked {
        /// The amount of tokens the delegate is approved for
        amount: u64,
        /// The amount's decimals
        decimals: u8,
    },
    /// Mint new tokens to an account, asserting the token mint and decimals
    MintToChecked {
        /// The amount of tokens to mint
        amount: u64,
        /// The amount's decimals
        decimals: u8,
    },
    /// Burn tokens from an account, asserting the token mint and decimals
    BurnChecked {
        /// The amount of tokens to burn
        amount: u64,
        /// The amount's decimals
        decimals: u8,
    },
    /// Initialize a new token account with the owner in instruction data
    InitializeAccount2 {
        /// The new account's owner
        owner: Pubkey,
    },
    /// Sync a native (wrapped SOL) account's amount with its lamports
    SyncNative,
    /// Initialize a new token account without the Rent sysvar
    InitializeAccount3 {
        /// The new account's owner
        owner: Pubkey,
    },
    /// Initialize a multisignature account without the Rent sysvar
    InitializeMultisig2 {
        /// The number of signers (M) required to validate this multisignature account
        m: u8,
    },
    /// Initialize a new mint without the Rent sysvar
    InitializeMint2 {
        /// Number of base 10 digits to the right of the decimal place
        decimals: u8,
        /// The authority/multisignature to mint tokens
        mint_authority: Pubkey,
        /// The freeze authority/multisignature of the mint
        freeze_authority: Option<Pubkey>,
    },
}

/// Authority types for [`TokenInstruction::SetAuthority`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthorityType {
    /// Authority to mint new tokens
    MintTokens,
    /// Authority to freeze any account associated with the mint
    FreezeAccount,
    /// Owner of a given token account
    AccountOwner,
    /// Authority to close a token account
    CloseAccount,
}

impl From<&AuthorityType> for u8 {
    fn from(authority_type: &AuthorityType) -> Self {
        match authority_type {
            AuthorityType::MintTokens => 0,
            AuthorityType::FreezeAccount => 1,
            AuthorityType::AccountOwner => 2,
            AuthorityType::CloseAccount => 3,
        }
    }
}

impl TokenInstruction {
    fn tag(&self) -> u8 {
        match self {
            Self::InitializeMint { .. } => 0,
            Self::InitializeAccount => 1,
            Self::InitializeMultisig { .. } => 2,
            Self::Transfer { .. } => 3,
            Self::Approve { .. } => 4,
            Self::Revoke => 5,
            Self::SetAuthority { .. } => 6,
            Self::MintTo { .. } => 7,
            Self::Burn { .. } => 8,
            Self::CloseAccount => 9,
            Self::FreezeAccount => 10,
            Self::ThawAccount => 11,
            Self::TransferChecked { .. } => 12,
            Self::ApproveChecked { .. } => 13,
            Self::MintToChecked { .. } => 14,
            Self::BurnChecked { .. } => 15,
            Self::InitializeAccount2 { .. } => 16,
            Self::SyncNative => 17,
            Self::InitializeAccount3 { .. } => 18,
            Self::InitializeMultisig2 { .. } => 19,
            Self::InitializeMint2 { .. } => 20,
        }
    }

    /// Serialize the instruction data.
    pub fn serialize(&self) -> Vec<u8> {
        fn put_option_pubkey(data: &mut Vec<u8>, key: &Option<Pubkey>) {
            match key {
                Some(key) => {
                    data.push(1);
                    data.extend_from_slice(key.as_bytes());
                }
                None => data.push(0),
            }
        }

        let mut data = vec![self.tag()];
        match self {
            Self::InitializeMint {
                decimals,
                mint_authority,
                freeze_authority,
            }
            | Self::InitializeMint2 {
                decimals,
                mint_authority,
                freeze_authority,
            } => {
                data.push(*decimals);
                data.extend_from_slice(mint_authority.as_bytes());
                put_option_pubkey(&mut data, freeze_authority);
            }
            Self::InitializeMultisig { m } | Self::InitializeMultisig2 { m } => data.push(*m),
            Self::Transfer { amount }
            | Self::Approve { amount }
            | Self::MintTo { amount }
            | Self::Burn { amount } => data.extend_from_slice(&amount.to_le_bytes()),
            Self::SetAuthority {
                authority_type,
                new_authority,
            } => {
                data.push(authority_type.into());
                put_option_pubkey(&mut data, new_authority);
            }
            Self::TransferChecked { amount, decimals }
            | Self::ApproveChecked { amount, decimals }
            | Self::MintToChecked { amount, decimals }
            | Self::BurnChecked { amount, decimals } => {
                data.extend_from_slice(&amount.to_le_bytes());
                data.push(*decimals);
            }
            Self::InitializeAccount2 { owner } | Self::InitializeAccount3 { owner } => {
                data.extend_from_slice(owner.as_bytes());
            }
            Self::InitializeAccount
            | Self::Revoke
            | Self::CloseAccount
            | Self::FreezeAccount
            | Self::ThawAccount
            | Self::SyncNative => {}
        }
        data
    }

    fn into_instruction(
        self,
        token_program_id: &Pubkey,
        accounts: Vec<AccountMeta>,
    ) -> Instruction {
        Instruction {
            program_id: *token_program_id,
            accounts,
            data: self.serialize(),
        }
    }
}

/// Create and initialize a token mint (defaults to the SPL Token program)
pub fn initialize_mint(
    mint: &Pubkey,
    mint_authority: &Pubkey,
    freeze_authority: Option<&Pubkey>,
    decimals: u8,
) -> Instruction {
    initialize_mint_with_program_id(
        mint,
        mint_authority,
        freeze_authority,
        decimals,
        &token_program(),
    )
}

/// Create and initialize a token mint using the provided token program
pub fn initialize_mint_with_program_id(
    mint: &Pubkey,
    mint_authority: &Pubkey,
    freeze_authority: Option<&Pubkey>,
    decimals: u8,
    token_program_id: &Pubkey,
) -> Instruction {
    TokenInstruction::InitializeMint {
        decimals,
        mint_authority: *mint_authority,
        freeze_authority: freeze_authority.copied(),
    }
    .into_instruction(
        token_program_id,
        vec![
            AccountMeta::new_writable(*mint),
            AccountMeta::new_readonly(rent_sysvar()),
        ],
    )
}

/// Create and initialize a token account (defaults to the SPL Token program)
pub fn initialize_account(account: &Pubkey, mint: &Pubkey, owner: &Pubkey) -> Instruction {
    initialize_account_with_program_id(account, mint, owner, &token_program())
}

/// Create and initialize a token account using the provided token program
pub fn initialize_account_with_program_id(
    account: &Pubkey,
    mint: &Pubkey,
    owner: &Pubkey,
    token_program_id: &Pubkey,
) -> Instruction {
    TokenInstruction::InitializeAccount.into_instruction(
        token_program_id,
        vec![
            AccountMeta::new_writable(*account),
            AccountMeta::new_readonly(*mint),
            AccountMeta::new_readonly(*owner),
            AccountMeta::new_readonly(rent_sysvar()),
        ],
    )
}

/// Transfer tokens from one account to another (defaults to the SPL Token program)
pub fn transfer(source: &Pubkey, destination: &Pubkey, owner: &Pubkey, amount: u64) -> Instruction {
    transfer_with_program_id(source, destination, owner, amount, &token_program())
}

/// Transfer tokens from one account to another using the provided token program
pub fn transfer_with_program_id(
    source: &Pubkey,
    destination: &Pubkey,
    owner: &Pubkey,
    amount: u64,
    token_program_id: &Pubkey,
) -> Instruction {
    TokenInstruction::Transfer { amount }.into_instruction(
        token_program_id,
        vec![
            AccountMeta::new_writable(*source),
            AccountMeta::new_writable(*destination),
            AccountMeta::new_signer(*owner),
        ],
    )
}

/// Mint tokens to an account (defaults to the SPL Token program)
pub fn mint_to(
    mint: &Pubkey,
    destination: &Pubkey,
    authority: &Pubkey,
    amount: u64,
) -> Instruction {
    mint_to_with_program_id(mint, destination, authority, amount, &token_program())
}

/// Mint tokens to an account using the provided token program
pub fn mint_to_with_program_id(
    mint: &Pubkey,
    destination: &Pubkey,
    authority: &Pubkey,
    amount: u64,
    token_program_id: &Pubkey,
) -> Instruction {
    TokenInstruction::MintTo { amount }.into_instruction(
        token_program_id,
        vec![
            AccountMeta::new_writable(*mint),
            AccountMeta::new_writable(*destination),
            AccountMeta::new_signer(*authority),
        ],
    )
}

/// Burn tokens from an account (defaults to the SPL Token program)
pub fn burn(account: &Pubkey, mint: &Pubkey, authority: &Pubkey, amount: u64) -> Instruction {
    burn_with_program_id(account, mint, authority, amount, &token_program())
}

/// Burn tokens from an account using the provided token program
pub fn burn_with_program_id(
    account: &Pubkey,
    mint: &Pubkey,
    authority: &Pubkey,
    amount: u64,
    token_program_id: &Pubkey,
) -> Instruction {
    TokenInstruction::Burn { amount }.into_instruction(
        token_program_id,
        vec![
            AccountMeta::new_writable(*account),
            AccountMeta::new_writable(*mint),
            AccountMeta::new_signer(*authority),
        ],
    )
}

/// Close a token account (defaults to the SPL Token program)
pub fn close_account(account: &Pubkey, destination: &Pubkey, owner: &Pubkey) -> Instruction {
    close_account_with_program_id(account, destination, owner, &token_program())
}

/// Close a token account using the provided token program
pub fn close_account_with_program_id(
    account: &Pubkey,
    destination: &Pubkey,
    owner: &Pubkey,
    token_program_id: &Pubkey,
) -> Instruction {
    TokenInstruction::CloseAccount.into_instruction(
        token_program_id,
        vec![
            AccountMeta::new_writable(*account),
            AccountMeta::new_writable(*destination),
            AccountMeta::new_signer(*owner),
        ],
    )
}

/// Transfer tokens, asserting the token mint and decimals (defaults to the SPL Token program)
pub fn transfer_checked(
    source: &Pubkey,
    mint: &Pubkey,
    destination: &Pubkey,
    owner: &Pubkey,
    amount: u64,
    decimals: u8,
) -> Instruction {
    transfer_checked_with_program_id(
        source,
        mint,
        destination,
        owner,
        amount,
        decimals,
        &token_program(),
    )
}

/// Transfer tokens, asserting the token mint and decimals, using the provided token program
pub fn transfer_checked_with_program_id(
    source: &Pubkey,
    mint: &Pubkey,
    destination: &Pubkey,
    owner: &Pubkey,
    amount: u64,
    decimals: u8,
    token_program_id: &Pubkey,
) -> Instruction {
    TokenInstruction::TransferChecked { amount, decimals }.into_instruction(
        token_program_id,
        vec![
            AccountMeta::new_writable(*source),
            AccountMeta::new_readonly(*mint),
            AccountMeta::new_writable(*destination),
            AccountMeta::new_signer(*owner),
        ],
    )
}

/// Mint new tokens to an account, asserting the token mint and decimals (defaults to the SPL Token program)
pub fn mint_to_checked(
    mint: &Pubkey,
    destination: &Pubkey,
    authority: &Pubkey,
    amount: u64,
    decimals: u8,
) -> Instruction {
    mint_to_checked_with_program_id(
        mint,
        destination,
        authority,
        amount,
        decimals,
        &token_program(),
    )
}

/// Mint new tokens to an account, asserting the token mint and decimals, using the provided token program
pub fn mint_to_checked_with_program_id(
    mint: &Pubkey,
    destination: &Pubkey,
    authority: &Pubkey,
    amount: u64,
    decimals: u8,
    token_program_id: &Pubkey,
) -> Instruction {
    TokenInstruction::MintToChecked { amount, decimals }.into_instruction(
        token_program_id,
        vec![
            AccountMeta::new_writable(*mint),
            AccountMeta::new_writable(*destination),
            AccountMeta::new_signer(*authority),
        ],
    )
}

/// Burn tokens from an account, asserting the token mint and decimals (defaults to the SPL Token program)
pub fn burn_checked(
    account: &Pubkey,
    mint: &Pubkey,
    authority: &Pubkey,
    amount: u64,
    decimals: u8,
) -> Instruction {
    burn_checked_with_program_id(account, mint, authority, amount, decimals, &token_program())
}

/// Burn tokens from an account, asserting the token mint and decimals, using the provided token program
pub fn burn_checked_with_program_id(
    account: &Pubkey,
    mint: &Pubkey,
    authority: &Pubkey,
    amount: u64,
    decimals: u8,
    token_program_id: &Pubkey,
) -> Instruction {
    TokenInstruction::BurnChecked { amount, decimals }.into_instruction(
        token_program_id,
        vec![
            AccountMeta::new_writable(*account),
            AccountMeta::new_writable(*mint),
            AccountMeta::new_signer(*authority),
        ],
    )
}

/// Sync native instruction (defaults to the SPL Token program)
pub fn sync_native(account: &Pubkey) -> Instruction {
    sync_native_with_program_id(account, &token_program())
}

/// Sync native instruction using the provided token program
pub fn sync_native_with_program_id(account: &Pubkey, token_program_id: &Pubkey) -> Instruction {
    TokenInstruction::SyncNative
        .into_instruction(token_program_id, vec![AccountMeta::new_writable(*account)])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::program_ids::{SYSVAR_RENT_ID, token_2022_program};
    use crate::test_utils::{key, meta};
    use hexlit::hex;

    // Vectors from `spl-token-interface` 3.0. `key(..)` is `sha256(label)`.
    const MINT: &str = "FqUwnBMN1shpeqKVm7W5fN73tvrjVr19TQFFgkoFFzhq";
    const AUTHORITY: &str = "Af2Y56WUFQuTTTYHMCjMozYsDxvTvSM6YQnyv8E6EK3v";
    const ACCOUNT: &str = "BRqsWh4VgYp3M4QAiHdGmUhnDWw2K8Nejj2n8xjPkypw";
    const OWNER: &str = "67vHA8qZGCJKw1UNGUJZME4MwEWDRGWzp7MGvsut43A8";
    const SOURCE: &str = "5Rtvwg6C7fnCFDSaLQmQJYp8kvVxLVeubPTN8o4yapQc";
    const DESTINATION: &str = "DEb5yphxEaPc5BN118svVN4R3GFu9jKs31Gcv5yekjZx";

    /// Check `instruction` against the upstream vector, and that the Token-2022
    /// variant differs only in its program id.
    fn check(
        instruction: Instruction,
        token_2022: Instruction,
        accounts: &[AccountMeta],
        data: &[u8],
    ) {
        let expected = Instruction {
            program_id: token_program(),
            accounts: accounts.to_vec(),
            data: data.to_vec(),
        };
        assert_eq!(instruction, expected);
        assert_eq!(
            token_2022,
            Instruction {
                program_id: token_2022_program(),
                ..expected
            }
        );
    }

    #[test]
    fn initialize_matches_upstream() {
        let t22 = token_2022_program();
        let (mint, authority) = (key("mint"), key("authority"));
        check(
            initialize_mint(&mint, &authority, None, 9),
            initialize_mint_with_program_id(&mint, &authority, None, 9, &t22),
            &[meta(MINT, false, true), meta(SYSVAR_RENT_ID, false, false)],
            &hex!(
                "0009 8f76fd501bb68ef71f4e276bc28f29bce1003b0c2c9d9478de81b5bfc0cde1e9"
                "00"
            ),
        );
        let freeze = key("freeze");
        check(
            initialize_mint(&mint, &authority, Some(&freeze), 6),
            initialize_mint_with_program_id(&mint, &authority, Some(&freeze), 6, &t22),
            &[meta(MINT, false, true), meta(SYSVAR_RENT_ID, false, false)],
            &hex!(
                "0006 8f76fd501bb68ef71f4e276bc28f29bce1003b0c2c9d9478de81b5bfc0cde1e9"
                "01 12dd9774cc96e18b5b1b5d4a4b1e0724c4b3e3f37a3eed1af6a1dac7cfdbc3e3"
            ),
        );

        let (account, owner) = (key("account"), key("owner"));
        check(
            initialize_account(&account, &mint, &owner),
            initialize_account_with_program_id(&account, &mint, &owner, &t22),
            &[
                meta(ACCOUNT, false, true),
                meta(MINT, false, false),
                meta(OWNER, false, false),
                meta(SYSVAR_RENT_ID, false, false),
            ],
            &hex!("01"),
        );
    }

    #[test]
    fn transfers_match_upstream() {
        let t22 = token_2022_program();
        let (source, destination, owner, mint) =
            (key("source"), key("destination"), key("owner"), key("mint"));
        check(
            transfer(&source, &destination, &owner, 123),
            transfer_with_program_id(&source, &destination, &owner, 123, &t22),
            &[
                meta(SOURCE, false, true),
                meta(DESTINATION, false, true),
                meta(OWNER, true, false),
            ],
            &hex!("03 7b00000000000000"),
        );
        check(
            transfer_checked(&source, &mint, &destination, &owner, u64::MAX, 18),
            transfer_checked_with_program_id(
                &source,
                &mint,
                &destination,
                &owner,
                u64::MAX,
                18,
                &t22,
            ),
            &[
                meta(SOURCE, false, true),
                meta(MINT, false, false),
                meta(DESTINATION, false, true),
                meta(OWNER, true, false),
            ],
            &hex!("0c ffffffffffffffff 12"),
        );
    }

    #[test]
    fn mint_and_burn_match_upstream() {
        let t22 = token_2022_program();
        let (mint, destination, authority, account) = (
            key("mint"),
            key("destination"),
            key("authority"),
            key("account"),
        );
        let mint_accounts = [
            meta(MINT, false, true),
            meta(DESTINATION, false, true),
            meta(AUTHORITY, true, false),
        ];
        check(
            mint_to(&mint, &destination, &authority, 1_000_000),
            mint_to_with_program_id(&mint, &destination, &authority, 1_000_000, &t22),
            &mint_accounts,
            &hex!("07 40420f0000000000"),
        );
        check(
            mint_to_checked(&mint, &destination, &authority, 5, 2),
            mint_to_checked_with_program_id(&mint, &destination, &authority, 5, 2, &t22),
            &mint_accounts,
            &hex!("0e 0500000000000000 02"),
        );

        let burn_accounts = [
            meta(ACCOUNT, false, true),
            meta(MINT, false, true),
            meta(AUTHORITY, true, false),
        ];
        check(
            burn(&account, &mint, &authority, 77),
            burn_with_program_id(&account, &mint, &authority, 77, &t22),
            &burn_accounts,
            &hex!("08 4d00000000000000"),
        );
        check(
            burn_checked(&account, &mint, &authority, 6, 3),
            burn_checked_with_program_id(&account, &mint, &authority, 6, 3, &t22),
            &burn_accounts,
            &hex!("0f 0600000000000000 03"),
        );
    }

    #[test]
    fn account_management_matches_upstream() {
        let t22 = token_2022_program();
        let (account, destination, owner) = (key("account"), key("destination"), key("owner"));
        check(
            close_account(&account, &destination, &owner),
            close_account_with_program_id(&account, &destination, &owner, &t22),
            &[
                meta(ACCOUNT, false, true),
                meta(DESTINATION, false, true),
                meta(OWNER, true, false),
            ],
            &hex!("09"),
        );
        check(
            sync_native(&account),
            sync_native_with_program_id(&account, &t22),
            &[meta(ACCOUNT, false, true)],
            &hex!("11"),
        );
    }

    #[test]
    fn serializes_remaining_variants() {
        let key = Pubkey::new([7; 32]);
        let cases = [
            (TokenInstruction::InitializeMultisig { m: 2 }, vec![2, 2]),
            (
                TokenInstruction::Approve { amount: 1 },
                [&[4][..], &1u64.to_le_bytes()].concat(),
            ),
            (TokenInstruction::Revoke, vec![5]),
            (
                TokenInstruction::SetAuthority {
                    authority_type: AuthorityType::CloseAccount,
                    new_authority: None,
                },
                vec![6, 3, 0],
            ),
            (
                TokenInstruction::SetAuthority {
                    authority_type: AuthorityType::AccountOwner,
                    new_authority: Some(key),
                },
                [&[6, 2, 1][..], key.as_bytes()].concat(),
            ),
            (TokenInstruction::FreezeAccount, vec![10]),
            (TokenInstruction::ThawAccount, vec![11]),
            (
                TokenInstruction::ApproveChecked {
                    amount: 1,
                    decimals: 9,
                },
                [&[13][..], &1u64.to_le_bytes(), &[9]].concat(),
            ),
            (
                TokenInstruction::InitializeAccount2 { owner: key },
                [&[16][..], key.as_bytes()].concat(),
            ),
            (
                TokenInstruction::InitializeAccount3 { owner: key },
                [&[18][..], key.as_bytes()].concat(),
            ),
            (TokenInstruction::InitializeMultisig2 { m: 3 }, vec![19, 3]),
            (
                TokenInstruction::InitializeMint2 {
                    decimals: 6,
                    mint_authority: key,
                    freeze_authority: None,
                },
                [&[20, 6][..], key.as_bytes(), &[0]].concat(),
            ),
        ];
        for (instruction, data) in cases {
            assert_eq!(instruction.serialize(), data, "{instruction:?}");
        }
    }
}
