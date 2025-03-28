use crate::instructions::program_ids::{
    ASSOCIATED_TOKEN_PROGRAM_ID, SYSTEM_PROGRAM_ID, TOKEN_PROGRAM_ID,
};
use crate::types::{AccountMeta, Instruction, Pubkey};

/// Create an associated token account instruction
pub fn create_associated_token_account(
    payer: &Pubkey,
    wallet_address: &Pubkey,
    token_mint_address: &Pubkey,
) -> Instruction {
    let associated_token_address = get_associated_token_address(wallet_address, token_mint_address);

    let account_metas = vec![
        // Funding account
        AccountMeta {
            pubkey: *payer,
            is_signer: true,
            is_writable: true,
        },
        // Associated token account
        AccountMeta {
            pubkey: associated_token_address,
            is_signer: false,
            is_writable: true,
        },
        // Wallet address
        AccountMeta {
            pubkey: *wallet_address,
            is_signer: false,
            is_writable: false,
        },
        // Token mint
        AccountMeta {
            pubkey: *token_mint_address,
            is_signer: false,
            is_writable: false,
        },
        // System program
        AccountMeta {
            pubkey: Pubkey::from_base58(SYSTEM_PROGRAM_ID).unwrap(),
            is_signer: false,
            is_writable: false,
        },
        // Token program
        AccountMeta {
            pubkey: Pubkey::from_base58(TOKEN_PROGRAM_ID).unwrap(),
            is_signer: false,
            is_writable: false,
        },
        // Rent sysvar
        AccountMeta {
            pubkey: Pubkey::from_base58("SysvarRent111111111111111111111111111111111").unwrap(),
            is_signer: false,
            is_writable: false,
        },
    ];

    Instruction {
        program_id: Pubkey::from_base58(ASSOCIATED_TOKEN_PROGRAM_ID).unwrap(),
        accounts: account_metas,
        data: vec![],
    }
}

/// Derive the associated token account address for a wallet address and token mint
pub fn get_associated_token_address(
    wallet_address: &Pubkey,
    token_mint_address: &Pubkey,
) -> Pubkey {
    // This will find a program address that derives from the associated token program, wallet address,
    // and token mint. This is the deterministic way the Solana program finds the PDA.
    // Note: This is a simplified version, actual implementations may do further validation and checks

    // Create seed derivation info
    let token_program_id = Pubkey::from_base58(TOKEN_PROGRAM_ID).unwrap();
    let seeds = [
        wallet_address.as_bytes(),
        token_program_id.as_bytes(),
        token_mint_address.as_bytes(),
    ];

    // In a real implementation this would call find_program_address
    // For demonstration, we'll just use a dummy representation
    // The actual derivation requires a more complex approach
    // Real impl: Pubkey::find_program_address(&seeds, &Pubkey::from_base58(ASSOCIATED_TOKEN_PROGRAM_ID).unwrap()).0

    // This is a dummy implementation - you should replace this with proper PDA derivation
    // Just concatenating bytes for demonstration - NOT the actual algorithm
    let mut bytes = [0u8; 32];
    let mut i = 0;

    for seed in seeds.iter() {
        for byte in seed.iter().take(10) {
            // limit to 10 bytes from each seed
            if i < 32 {
                bytes[i] = *byte;
                i += 1;
            }
        }
    }

    Pubkey::new(bytes)
}
