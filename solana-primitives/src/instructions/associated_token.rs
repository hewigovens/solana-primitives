use crate::instructions::program_ids::{associated_token_program, system_program, token_program};
use crate::types::{AccountMeta, Instruction, Pubkey, find_program_address};

/// `AssociatedTokenAccountInstruction::Create`.
const CREATE_DISCRIMINATOR: u8 = 0;
/// `AssociatedTokenAccountInstruction::CreateIdempotent`.
const CREATE_IDEMPOTENT_DISCRIMINATOR: u8 = 1;

/// Create an associated token account instruction (defaults to the SPL Token program)
pub fn create_associated_token_account(
    payer: &Pubkey,
    wallet_address: &Pubkey,
    token_mint_address: &Pubkey,
) -> Instruction {
    create_associated_token_account_with_program_id(
        payer,
        wallet_address,
        token_mint_address,
        &token_program(),
    )
}

/// Create an associated token account instruction using the provided token program
pub fn create_associated_token_account_with_program_id(
    payer: &Pubkey,
    wallet_address: &Pubkey,
    token_mint_address: &Pubkey,
    token_program_id: &Pubkey,
) -> Instruction {
    create_instruction(
        payer,
        &get_associated_token_address_with_program_id(
            wallet_address,
            token_mint_address,
            token_program_id,
        ),
        wallet_address,
        token_mint_address,
        token_program_id,
        CREATE_DISCRIMINATOR,
    )
}

/// Create an associated token account idempotent instruction for the given token program
pub fn create_associated_token_account_idempotent(
    payer: &Pubkey,
    wallet_address: &Pubkey,
    token_mint_address: &Pubkey,
    token_program_id: &Pubkey,
) -> Instruction {
    create_associated_token_account_idempotent_with_address(
        payer,
        &get_associated_token_address_with_program_id(
            wallet_address,
            token_mint_address,
            token_program_id,
        ),
        wallet_address,
        token_mint_address,
        token_program_id,
    )
}

/// Create an associated token account idempotent instruction with an explicit associated token account address
pub fn create_associated_token_account_idempotent_with_address(
    payer: &Pubkey,
    associated_token_address: &Pubkey,
    wallet_address: &Pubkey,
    token_mint_address: &Pubkey,
    token_program_id: &Pubkey,
) -> Instruction {
    create_instruction(
        payer,
        associated_token_address,
        wallet_address,
        token_mint_address,
        token_program_id,
        CREATE_IDEMPOTENT_DISCRIMINATOR,
    )
}

/// Accounts: funding (writable, signer), associated account (writable), wallet,
/// mint, System program, token program.
fn create_instruction(
    payer: &Pubkey,
    associated_token_address: &Pubkey,
    wallet_address: &Pubkey,
    token_mint_address: &Pubkey,
    token_program_id: &Pubkey,
    discriminator: u8,
) -> Instruction {
    Instruction {
        program_id: associated_token_program(),
        accounts: vec![
            AccountMeta::new_signer_writable(*payer),
            AccountMeta::new_writable(*associated_token_address),
            AccountMeta::new_readonly(*wallet_address),
            AccountMeta::new_readonly(*token_mint_address),
            AccountMeta::new_readonly(system_program()),
            AccountMeta::new_readonly(*token_program_id),
        ],
        data: vec![discriminator],
    }
}

/// Derive the associated token account address for a wallet address and token mint
pub fn get_associated_token_address(
    wallet_address: &Pubkey,
    token_mint_address: &Pubkey,
) -> Pubkey {
    get_associated_token_address_with_program_id(
        wallet_address,
        token_mint_address,
        &token_program(),
    )
}

/// Derive the associated token account address for a wallet address, token mint, and token program
pub fn get_associated_token_address_with_program_id(
    wallet_address: &Pubkey,
    token_mint_address: &Pubkey,
    token_program_id: &Pubkey,
) -> Pubkey {
    let seeds: [&[u8]; 3] = [
        wallet_address.as_bytes(),
        token_program_id.as_bytes(),
        token_mint_address.as_bytes(),
    ];
    find_program_address(&associated_token_program(), &seeds)
        .expect("three 32-byte seeds are always valid")
        .0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::program_ids::{ASSOCIATED_TOKEN_PROGRAM_ID, token_2022_program};
    use crate::test_utils::{key, meta, pubkey};

    // Vectors from `spl-associated-token-account-interface` 2.0.
    const WALLET: &str = "Hozo7TadHq6PMMiGLGNvgk79Hvj5VTAM7Ny2bamQ2m8q";
    const MINT: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
    const PAYER: &str = "AWxggjuZRmWULwxwPeM6ZZxRtdDdekVq22mFRx2QbW7U";
    const SYSTEM: &str = "11111111111111111111111111111111";
    const TOKEN: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
    const TOKEN_2022: &str = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";
    const TOKEN_ATA: &str = "BKrrNZoFUEaMGhUYq3JmFMLC5mFBMYU3NidJtiYnu2Db";
    const TOKEN_2022_ATA: &str = "8Du77zKsVNiU5dZ6PeG8YG8vmNwfpJGAnu6jw9baYdpp";

    fn expected(ata: &str, token_program: &str, data: u8) -> Instruction {
        Instruction {
            program_id: pubkey(ASSOCIATED_TOKEN_PROGRAM_ID),
            accounts: vec![
                meta(PAYER, true, true),
                meta(ata, false, true),
                meta(WALLET, false, false),
                meta(MINT, false, false),
                meta(SYSTEM, false, false),
                meta(token_program, false, false),
            ],
            data: vec![data],
        }
    }

    #[test]
    fn token_program_instructions_match_upstream() {
        let (payer, wallet, mint) = (key("payer"), pubkey(WALLET), pubkey(MINT));
        assert_eq!(payer, pubkey(PAYER));
        assert_eq!(
            get_associated_token_address(&wallet, &mint),
            pubkey(TOKEN_ATA)
        );
        assert_eq!(
            create_associated_token_account(&payer, &wallet, &mint),
            expected(TOKEN_ATA, TOKEN, 0)
        );
        assert_eq!(
            create_associated_token_account_idempotent(&payer, &wallet, &mint, &token_program()),
            expected(TOKEN_ATA, TOKEN, 1)
        );
    }

    #[test]
    fn token_2022_instructions_match_upstream() {
        let (payer, wallet, mint) = (key("payer"), pubkey(WALLET), pubkey(MINT));
        let program = token_2022_program();
        assert_eq!(
            get_associated_token_address_with_program_id(&wallet, &mint, &program),
            pubkey(TOKEN_2022_ATA)
        );
        assert_eq!(
            create_associated_token_account_with_program_id(&payer, &wallet, &mint, &program),
            expected(TOKEN_2022_ATA, TOKEN_2022, 0)
        );
        assert_eq!(
            create_associated_token_account_idempotent(&payer, &wallet, &mint, &program),
            expected(TOKEN_2022_ATA, TOKEN_2022, 1)
        );
    }
}
