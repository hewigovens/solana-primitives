use crate::instructions::program_ids::TOKEN_PROGRAM_ID;
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

/// Create and initialize a token mint
pub fn initialize_mint(
    mint: &Pubkey,
    mint_authority: &Pubkey,
    freeze_authority: Option<&Pubkey>,
    decimals: u8,
) -> Instruction {
    let account_metas = vec![
        AccountMeta {
            pubkey: *mint,
            is_signer: false,
            is_writable: true,
        },
        AccountMeta {
            pubkey: Pubkey::from_base58("SysvarRent111111111111111111111111111111111").unwrap(),
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
        program_id: Pubkey::from_base58(TOKEN_PROGRAM_ID).unwrap(),
        accounts: account_metas,
        data: instruction.serialize(),
    }
}

/// Create and initialize a token account
pub fn initialize_account(account: &Pubkey, mint: &Pubkey, owner: &Pubkey) -> Instruction {
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
            pubkey: Pubkey::from_base58("SysvarRent111111111111111111111111111111111").unwrap(),
            is_signer: false,
            is_writable: false,
        },
    ];

    let instruction = TokenInstruction::InitializeAccount;

    Instruction {
        program_id: Pubkey::from_base58(TOKEN_PROGRAM_ID).unwrap(),
        accounts: account_metas,
        data: instruction.serialize(),
    }
}

/// Transfer tokens from one account to another
pub fn transfer(source: &Pubkey, destination: &Pubkey, owner: &Pubkey, amount: u64) -> Instruction {
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
        program_id: Pubkey::from_base58(TOKEN_PROGRAM_ID).unwrap(),
        accounts: account_metas,
        data: instruction.serialize(),
    }
}

/// Mint tokens to an account
pub fn mint_to(
    mint: &Pubkey,
    destination: &Pubkey,
    authority: &Pubkey,
    amount: u64,
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
        program_id: Pubkey::from_base58(TOKEN_PROGRAM_ID).unwrap(),
        accounts: account_metas,
        data: instruction.serialize(),
    }
}

/// Burn tokens from an account
pub fn burn(account: &Pubkey, mint: &Pubkey, authority: &Pubkey, amount: u64) -> Instruction {
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
        program_id: Pubkey::from_base58(TOKEN_PROGRAM_ID).unwrap(),
        accounts: account_metas,
        data: instruction.serialize(),
    }
}

/// Close a token account
pub fn close_account(account: &Pubkey, destination: &Pubkey, owner: &Pubkey) -> Instruction {
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
        program_id: Pubkey::from_base58(TOKEN_PROGRAM_ID).unwrap(),
        accounts: account_metas,
        data: instruction.serialize(),
    }
}
