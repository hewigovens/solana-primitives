use crate::instructions::program_ids::{rent_sysvar, token_program};
use crate::types::{AccountMeta, Instruction, Pubkey};

/// Token program instruction types
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
    /// Initialize a new token account, asserting the token mint
    InitializeAccount2 {
        /// The authority/multisignature to mint tokens
        owner: Pubkey,
    },
    /// Syncronize the closing of Token accounts
    SyncNative,
    /// Initialize a new token account, asserting the token mint and owner
    InitializeAccount3 {
        /// The authority/multisignature to mint tokens
        owner: Pubkey,
    },
    /// Initialize a multisignature account with an owner
    InitializeMultisig2 {
        /// The number of signers (M) required to validate this multisignature account
        m: u8,
    },
    /// Initialize a new mint, asserting the mint authority
    InitializeMint2 {
        /// Number of base 10 digits to the right of the decimal place
        decimals: u8,
        /// The authority/multisignature to mint tokens
        mint_authority: Pubkey,
        /// The freeze authority/multisignature of the mint
        freeze_authority: Option<Pubkey>,
    },
}

/// Authority types
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

impl TokenInstruction {
    /// Serialize the token instruction
    pub fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::new();
        match self {
            Self::InitializeMint {
                decimals,
                mint_authority,
                freeze_authority,
            } => {
                data.push(0); // Initialize mint instruction
                data.push(*decimals);
                data.extend_from_slice(mint_authority.as_bytes());
                data.push(freeze_authority.is_some() as u8);
                if let Some(freeze_authority) = freeze_authority {
                    data.extend_from_slice(freeze_authority.as_bytes());
                }
            }
            Self::InitializeAccount => {
                data.push(1); // Initialize account instruction
            }
            Self::InitializeMultisig { m } => {
                data.push(2); // Initialize multisig instruction
                data.push(*m);
            }
            Self::Transfer { amount } => {
                data.push(3); // Transfer instruction
                data.extend_from_slice(&amount.to_le_bytes());
            }
            Self::Approve { amount } => {
                data.push(4); // Approve instruction
                data.extend_from_slice(&amount.to_le_bytes());
            }
            Self::Revoke => {
                data.push(5); // Revoke instruction
            }
            Self::SetAuthority {
                authority_type,
                new_authority,
            } => {
                data.push(6); // Set authority instruction
                data.push(authority_type.into()); // Authority type
                data.push(new_authority.is_some() as u8);
                if let Some(new_authority) = new_authority {
                    data.extend_from_slice(new_authority.as_bytes());
                }
            }
            Self::MintTo { amount } => {
                data.push(7); // Mint to instruction
                data.extend_from_slice(&amount.to_le_bytes());
            }
            Self::Burn { amount } => {
                data.push(8); // Burn instruction
                data.extend_from_slice(&amount.to_le_bytes());
            }
            Self::CloseAccount => {
                data.push(9); // Close account instruction
            }
            Self::FreezeAccount => {
                data.push(10); // Freeze account instruction
            }
            Self::ThawAccount => {
                data.push(11); // Thaw account instruction
            }
            Self::TransferChecked { amount, decimals } => {
                data.push(12); // Transfer checked instruction
                data.extend_from_slice(&amount.to_le_bytes());
                data.push(*decimals);
            }
            Self::ApproveChecked { amount, decimals } => {
                data.push(13); // Approve checked instruction
                data.extend_from_slice(&amount.to_le_bytes());
                data.push(*decimals);
            }
            Self::MintToChecked { amount, decimals } => {
                data.push(14); // Mint to checked instruction
                data.extend_from_slice(&amount.to_le_bytes());
                data.push(*decimals);
            }
            Self::BurnChecked { amount, decimals } => {
                data.push(15); // Burn checked instruction
                data.extend_from_slice(&amount.to_le_bytes());
                data.push(*decimals);
            }
            Self::InitializeAccount2 { owner } => {
                data.push(16); // Initialize account 2 instruction
                data.extend_from_slice(owner.as_bytes());
            }
            Self::SyncNative => {
                data.push(17); // Sync native instruction
            }
            Self::InitializeAccount3 { owner } => {
                data.push(18); // Initialize account 3 instruction
                data.extend_from_slice(owner.as_bytes());
            }
            Self::InitializeMultisig2 { m } => {
                data.push(19); // Initialize multisig 2 instruction
                data.push(*m);
            }
            Self::InitializeMint2 {
                decimals,
                mint_authority,
                freeze_authority,
            } => {
                data.push(20); // Initialize mint 2 instruction
                data.push(*decimals);
                data.extend_from_slice(mint_authority.as_bytes());
                data.push(freeze_authority.is_some() as u8);
                if let Some(freeze_authority) = freeze_authority {
                    data.extend_from_slice(freeze_authority.as_bytes());
                }
            }
        }
        data
    }
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
    let account_metas = vec![
        AccountMeta {
            pubkey: *mint,
            is_signer: false,
            is_writable: true,
        },
        AccountMeta {
            pubkey: rent_sysvar(),
            is_signer: false,
            is_writable: false,
        },
    ];

    let instruction = TokenInstruction::InitializeMint {
        decimals,
        mint_authority: *mint_authority,
        freeze_authority: freeze_authority.cloned(),
    };

    Instruction {
        program_id: *token_program_id,
        accounts: account_metas,
        data: instruction.serialize(),
    }
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
    let account_metas = vec![
        AccountMeta {
            pubkey: *account,
            is_signer: false,
            is_writable: true,
        },
        AccountMeta {
            pubkey: *mint,
            is_signer: false,
            is_writable: false,
        },
        AccountMeta {
            pubkey: *owner,
            is_signer: false,
            is_writable: false,
        },
        AccountMeta {
            pubkey: rent_sysvar(),
            is_signer: false,
            is_writable: false,
        },
    ];

    let instruction = TokenInstruction::InitializeAccount;

    Instruction {
        program_id: *token_program_id,
        accounts: account_metas,
        data: instruction.serialize(),
    }
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
    let account_metas = vec![
        AccountMeta {
            pubkey: *source,
            is_signer: false,
            is_writable: true,
        },
        AccountMeta {
            pubkey: *destination,
            is_signer: false,
            is_writable: true,
        },
        AccountMeta {
            pubkey: *owner,
            is_signer: true,
            is_writable: false,
        },
    ];

    let instruction = TokenInstruction::Transfer { amount };

    Instruction {
        program_id: *token_program_id,
        accounts: account_metas,
        data: instruction.serialize(),
    }
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
    let account_metas = vec![
        AccountMeta {
            pubkey: *mint,
            is_signer: false,
            is_writable: true,
        },
        AccountMeta {
            pubkey: *destination,
            is_signer: false,
            is_writable: true,
        },
        AccountMeta {
            pubkey: *authority,
            is_signer: true,
            is_writable: false,
        },
    ];

    let instruction = TokenInstruction::MintTo { amount };

    Instruction {
        program_id: *token_program_id,
        accounts: account_metas,
        data: instruction.serialize(),
    }
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
    let account_metas = vec![
        AccountMeta {
            pubkey: *account,
            is_signer: false,
            is_writable: true,
        },
        AccountMeta {
            pubkey: *mint,
            is_signer: false,
            is_writable: true,
        },
        AccountMeta {
            pubkey: *authority,
            is_signer: true,
            is_writable: false,
        },
    ];

    let instruction = TokenInstruction::Burn { amount };

    Instruction {
        program_id: *token_program_id,
        accounts: account_metas,
        data: instruction.serialize(),
    }
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
    let account_metas = vec![
        AccountMeta {
            pubkey: *account,
            is_signer: false,
            is_writable: true,
        },
        AccountMeta {
            pubkey: *destination,
            is_signer: false,
            is_writable: true,
        },
        AccountMeta {
            pubkey: *owner,
            is_signer: true,
            is_writable: false,
        },
    ];

    let instruction = TokenInstruction::CloseAccount;

    Instruction {
        program_id: *token_program_id,
        accounts: account_metas,
        data: instruction.serialize(),
    }
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
    let accounts = vec![
        AccountMeta {
            pubkey: *source,
            is_signer: false,
            is_writable: true,
        },
        AccountMeta {
            pubkey: *mint,
            is_signer: false,
            is_writable: false,
        },
        AccountMeta {
            pubkey: *destination,
            is_signer: false,
            is_writable: true,
        },
        AccountMeta {
            pubkey: *owner,
            is_signer: true,
            is_writable: false,
        },
    ];

    let data = TokenInstruction::TransferChecked { amount, decimals }.serialize();

    Instruction {
        program_id: *token_program_id,
        accounts,
        data,
    }
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
    let accounts = vec![
        AccountMeta {
            pubkey: *mint,
            is_signer: false,
            is_writable: true,
        },
        AccountMeta {
            pubkey: *destination,
            is_signer: false,
            is_writable: true,
        },
        AccountMeta {
            pubkey: *authority,
            is_signer: true,
            is_writable: false,
        },
    ];

    let data = TokenInstruction::MintToChecked { amount, decimals }.serialize();

    Instruction {
        program_id: *token_program_id,
        accounts,
        data,
    }
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
    let accounts = vec![
        AccountMeta {
            pubkey: *account,
            is_signer: false,
            is_writable: true,
        },
        AccountMeta {
            pubkey: *mint,
            is_signer: false,
            is_writable: true,
        },
        AccountMeta {
            pubkey: *authority,
            is_signer: true,
            is_writable: false,
        },
    ];

    let data = TokenInstruction::BurnChecked { amount, decimals }.serialize();

    Instruction {
        program_id: *token_program_id,
        accounts,
        data,
    }
}

/// Sync native instruction (defaults to the SPL Token program)
pub fn sync_native(account: &Pubkey) -> Instruction {
    sync_native_with_program_id(account, &token_program())
}

/// Sync native instruction using the provided token program
pub fn sync_native_with_program_id(account: &Pubkey, token_program_id: &Pubkey) -> Instruction {
    let accounts = vec![AccountMeta {
        pubkey: *account,
        is_signer: false,
        is_writable: true,
    }];

    let data = TokenInstruction::SyncNative.serialize();

    Instruction {
        program_id: *token_program_id,
        accounts,
        data,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Pubkey;
    use crate::instructions::program_ids::{
        SYSVAR_RENT_ID, TOKEN_2022_PROGRAM_ID, TOKEN_PROGRAM_ID,
    };

    // Use the same public keys as in the JavaScript test file
    fn mint_pubkey() -> Pubkey {
        Pubkey::from_base58("7o36UsWR1JQLpZ9PE2gn9L4SQ69CNNiWAXd4Jt7rqz9Z").unwrap()
    }

    fn token_pubkey() -> Pubkey {
        Pubkey::from_base58("DShWnroshVbeUp28oopA3Pu7oFPDBtC1DBmPECXXAQ9n").unwrap()
    }

    fn authority_pubkey() -> Pubkey {
        Pubkey::from_base58("Hozo7TadHq6PMMiGLGNvgk79Hvj5VTAM7Ny2bamQ2m8q").unwrap()
    }

    fn payer_pubkey() -> Pubkey {
        Pubkey::from_base58("3ECJhLBQ9DAuKBKNjQGLEk3YqoFcF1YvhdayQ2C96eXF").unwrap()
    }

    #[test]
    fn test_transfer() {
        let source = mint_pubkey();
        let destination = token_pubkey();
        let owner = authority_pubkey();
        let amount = 123u64;

        let instruction = transfer(&source, &destination, &owner, amount);

        assert_eq!(
            instruction.program_id,
            Pubkey::from_base58(TOKEN_PROGRAM_ID).unwrap()
        );
        assert_eq!(instruction.accounts.len(), 3);
        assert_eq!(instruction.accounts[0].pubkey, source);
        assert!(instruction.accounts[0].is_writable);
        assert_eq!(instruction.accounts[1].pubkey, destination);
        assert!(instruction.accounts[1].is_writable);
        assert_eq!(instruction.accounts[2].pubkey, owner);
        assert!(instruction.accounts[2].is_signer);

        // Check data - should be [3] (transfer instruction) followed by amount bytes
        let expected_data = {
            let mut data = vec![3];
            data.extend_from_slice(&amount.to_le_bytes());
            data
        };
        assert_eq!(instruction.data, expected_data);

        let token_2022_program = Pubkey::from_base58(TOKEN_2022_PROGRAM_ID).unwrap();
        let instruction =
            transfer_with_program_id(&source, &destination, &owner, amount, &token_2022_program);
        assert_eq!(instruction.program_id, token_2022_program);
        assert_eq!(instruction.data, expected_data);
    }

    #[test]
    fn test_transfer_checked() {
        let source = token_pubkey();
        let mint = mint_pubkey();
        let destination = payer_pubkey();
        let owner = authority_pubkey();
        let amount = 123u64;
        let decimals = 10u8;

        let instruction = transfer_checked(&source, &mint, &destination, &owner, amount, decimals);

        assert_eq!(
            instruction.program_id,
            Pubkey::from_base58(TOKEN_PROGRAM_ID).unwrap()
        );
        assert_eq!(instruction.accounts.len(), 4);
        assert_eq!(instruction.accounts[0].pubkey, source);
        assert!(instruction.accounts[0].is_writable);
        assert_eq!(instruction.accounts[1].pubkey, mint);
        assert!(!instruction.accounts[1].is_writable);
        assert_eq!(instruction.accounts[2].pubkey, destination);
        assert!(instruction.accounts[2].is_writable);
        assert_eq!(instruction.accounts[3].pubkey, owner);
        assert!(instruction.accounts[3].is_signer);

        // Check data - should be [12] (transfer checked instruction) followed by amount bytes and decimals
        let expected_data = {
            let mut data = vec![12];
            data.extend_from_slice(&amount.to_le_bytes());
            data.push(decimals);
            data
        };
        assert_eq!(instruction.data, expected_data);

        let token_2022_program = Pubkey::from_base58(TOKEN_2022_PROGRAM_ID).unwrap();
        let instruction = transfer_checked_with_program_id(
            &source,
            &mint,
            &destination,
            &owner,
            amount,
            decimals,
            &token_2022_program,
        );
        assert_eq!(instruction.program_id, token_2022_program);
        assert_eq!(instruction.data, expected_data);
    }

    #[test]
    fn test_mint_to_checked() {
        let mint = mint_pubkey();
        let token = token_pubkey();
        let mint_authority = authority_pubkey();
        let amount = 123u64;
        let decimals = 10u8;

        let instruction = mint_to_checked(&mint, &token, &mint_authority, amount, decimals);

        assert_eq!(
            instruction.program_id,
            Pubkey::from_base58(TOKEN_PROGRAM_ID).unwrap()
        );
        assert_eq!(instruction.accounts.len(), 3);
        assert_eq!(instruction.accounts[0].pubkey, mint);
        assert!(instruction.accounts[0].is_writable);
        assert_eq!(instruction.accounts[1].pubkey, token);
        assert!(instruction.accounts[1].is_writable);
        assert_eq!(instruction.accounts[2].pubkey, mint_authority);
        assert!(instruction.accounts[2].is_signer);

        // Check data - should be [14] (mint to checked instruction) followed by amount bytes and decimals
        let expected_data = {
            let mut data = vec![14];
            data.extend_from_slice(&amount.to_le_bytes());
            data.push(decimals);
            data
        };
        assert_eq!(instruction.data, expected_data);

        let token_2022_program = Pubkey::from_base58(TOKEN_2022_PROGRAM_ID).unwrap();
        let instruction = mint_to_checked_with_program_id(
            &mint,
            &token,
            &mint_authority,
            amount,
            decimals,
            &token_2022_program,
        );
        assert_eq!(instruction.program_id, token_2022_program);
        assert_eq!(instruction.data, expected_data);
    }

    #[test]
    fn test_burn_checked() {
        let account = mint_pubkey();
        let mint = token_pubkey();
        let authority = authority_pubkey();
        let amount = 123u64;
        let decimals = 10u8;

        let instruction = burn_checked(&account, &mint, &authority, amount, decimals);

        assert_eq!(
            instruction.program_id,
            Pubkey::from_base58(TOKEN_PROGRAM_ID).unwrap()
        );
        assert_eq!(instruction.accounts.len(), 3);
        assert_eq!(instruction.accounts[0].pubkey, account);
        assert!(instruction.accounts[0].is_writable);
        assert_eq!(instruction.accounts[1].pubkey, mint);
        assert!(instruction.accounts[1].is_writable);
        assert_eq!(instruction.accounts[2].pubkey, authority);
        assert!(instruction.accounts[2].is_signer);

        // Check data - should be [15] (burn checked instruction) followed by amount bytes and decimals
        let expected_data = {
            let mut data = vec![15];
            data.extend_from_slice(&amount.to_le_bytes());
            data.push(decimals);
            data
        };
        assert_eq!(instruction.data, expected_data);

        let token_2022_program = Pubkey::from_base58(TOKEN_2022_PROGRAM_ID).unwrap();
        let instruction = burn_checked_with_program_id(
            &account,
            &mint,
            &authority,
            amount,
            decimals,
            &token_2022_program,
        );
        assert_eq!(instruction.program_id, token_2022_program);
        assert_eq!(instruction.data, expected_data);
    }

    #[test]
    fn test_sync_native() {
        let account = token_pubkey();

        let instruction = sync_native(&account);

        assert_eq!(
            instruction.program_id,
            Pubkey::from_base58(TOKEN_PROGRAM_ID).unwrap()
        );
        assert_eq!(instruction.accounts.len(), 1);
        assert_eq!(instruction.accounts[0].pubkey, account);
        assert!(instruction.accounts[0].is_writable);

        // Check data - should be [17] (sync native instruction)
        assert_eq!(instruction.data, vec![17]);

        let token_2022_program = Pubkey::from_base58(TOKEN_2022_PROGRAM_ID).unwrap();
        let instruction = sync_native_with_program_id(&account, &token_2022_program);
        assert_eq!(instruction.program_id, token_2022_program);
        assert_eq!(instruction.accounts.len(), 1);
        assert_eq!(instruction.accounts[0].pubkey, account);
        assert!(instruction.accounts[0].is_writable);
        assert_eq!(instruction.data, vec![17]);
    }

    #[test]
    fn test_initialize_mint() {
        let mint = mint_pubkey();
        let mint_authority = authority_pubkey();
        let decimals = 9u8;
        let rent = Pubkey::from_base58(SYSVAR_RENT_ID).unwrap();

        let instruction = initialize_mint(&mint, &mint_authority, None, decimals);
        assert_eq!(
            instruction.program_id,
            Pubkey::from_base58(TOKEN_PROGRAM_ID).unwrap()
        );
        assert_eq!(instruction.accounts.len(), 2);
        assert_eq!(instruction.accounts[0].pubkey, mint);
        assert!(instruction.accounts[0].is_writable);
        assert_eq!(instruction.accounts[1].pubkey, rent);

        let expected_data = {
            let mut data = vec![0, decimals];
            data.extend_from_slice(mint_authority.as_bytes());
            data.push(0); // no freeze authority
            data
        };
        assert_eq!(instruction.data, expected_data);

        let token_2022_program = Pubkey::from_base58(TOKEN_2022_PROGRAM_ID).unwrap();
        let instruction = initialize_mint_with_program_id(
            &mint,
            &mint_authority,
            None,
            decimals,
            &token_2022_program,
        );
        assert_eq!(instruction.program_id, token_2022_program);
        assert_eq!(instruction.data, expected_data);
    }

    #[test]
    fn test_initialize_account() {
        let account = token_pubkey();
        let mint = mint_pubkey();
        let owner = authority_pubkey();

        let instruction = initialize_account(&account, &mint, &owner);
        assert_eq!(
            instruction.program_id,
            Pubkey::from_base58(TOKEN_PROGRAM_ID).unwrap()
        );
        assert_eq!(instruction.accounts.len(), 4);
        assert_eq!(instruction.accounts[0].pubkey, account);
        assert!(instruction.accounts[0].is_writable);
        assert_eq!(instruction.accounts[1].pubkey, mint);
        assert_eq!(instruction.accounts[2].pubkey, owner);
        assert_eq!(
            instruction.accounts[3].pubkey,
            Pubkey::from_base58(SYSVAR_RENT_ID).unwrap()
        );
        assert_eq!(instruction.data, vec![1]);

        let token_2022_program = Pubkey::from_base58(TOKEN_2022_PROGRAM_ID).unwrap();
        let instruction =
            initialize_account_with_program_id(&account, &mint, &owner, &token_2022_program);
        assert_eq!(instruction.program_id, token_2022_program);
        assert_eq!(instruction.data, vec![1]);
    }

    #[test]
    fn test_mint_to() {
        let mint = mint_pubkey();
        let destination = token_pubkey();
        let authority = authority_pubkey();
        let amount = 123u64;

        let instruction = mint_to(&mint, &destination, &authority, amount);
        assert_eq!(
            instruction.program_id,
            Pubkey::from_base58(TOKEN_PROGRAM_ID).unwrap()
        );
        assert_eq!(instruction.accounts.len(), 3);
        assert_eq!(instruction.accounts[0].pubkey, mint);
        assert!(instruction.accounts[0].is_writable);
        assert_eq!(instruction.accounts[1].pubkey, destination);
        assert!(instruction.accounts[1].is_writable);
        assert_eq!(instruction.accounts[2].pubkey, authority);
        assert!(instruction.accounts[2].is_signer);

        let expected_data = {
            let mut data = vec![7];
            data.extend_from_slice(&amount.to_le_bytes());
            data
        };
        assert_eq!(instruction.data, expected_data);

        let token_2022_program = Pubkey::from_base58(TOKEN_2022_PROGRAM_ID).unwrap();
        let instruction =
            mint_to_with_program_id(&mint, &destination, &authority, amount, &token_2022_program);
        assert_eq!(instruction.program_id, token_2022_program);
        assert_eq!(instruction.data, expected_data);
    }

    #[test]
    fn test_burn() {
        let account = token_pubkey();
        let mint = mint_pubkey();
        let authority = authority_pubkey();
        let amount = 123u64;

        let instruction = burn(&account, &mint, &authority, amount);
        assert_eq!(
            instruction.program_id,
            Pubkey::from_base58(TOKEN_PROGRAM_ID).unwrap()
        );
        assert_eq!(instruction.accounts.len(), 3);
        assert_eq!(instruction.accounts[0].pubkey, account);
        assert!(instruction.accounts[0].is_writable);
        assert_eq!(instruction.accounts[1].pubkey, mint);
        assert!(instruction.accounts[1].is_writable);
        assert_eq!(instruction.accounts[2].pubkey, authority);
        assert!(instruction.accounts[2].is_signer);

        let expected_data = {
            let mut data = vec![8];
            data.extend_from_slice(&amount.to_le_bytes());
            data
        };
        assert_eq!(instruction.data, expected_data);

        let token_2022_program = Pubkey::from_base58(TOKEN_2022_PROGRAM_ID).unwrap();
        let instruction =
            burn_with_program_id(&account, &mint, &authority, amount, &token_2022_program);
        assert_eq!(instruction.program_id, token_2022_program);
        assert_eq!(instruction.data, expected_data);
    }

    #[test]
    fn test_close_account() {
        let account = token_pubkey();
        let destination = payer_pubkey();
        let owner = authority_pubkey();

        let instruction = close_account(&account, &destination, &owner);
        assert_eq!(
            instruction.program_id,
            Pubkey::from_base58(TOKEN_PROGRAM_ID).unwrap()
        );
        assert_eq!(instruction.accounts.len(), 3);
        assert_eq!(instruction.accounts[0].pubkey, account);
        assert!(instruction.accounts[0].is_writable);
        assert_eq!(instruction.accounts[1].pubkey, destination);
        assert!(instruction.accounts[1].is_writable);
        assert_eq!(instruction.accounts[2].pubkey, owner);
        assert!(instruction.accounts[2].is_signer);
        assert_eq!(instruction.data, vec![9]);

        let token_2022_program = Pubkey::from_base58(TOKEN_2022_PROGRAM_ID).unwrap();
        let instruction =
            close_account_with_program_id(&account, &destination, &owner, &token_2022_program);
        assert_eq!(instruction.program_id, token_2022_program);
        assert_eq!(instruction.data, vec![9]);
    }
}
