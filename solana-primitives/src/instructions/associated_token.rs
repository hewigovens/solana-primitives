use crate::instructions::program_ids::{
    ASSOCIATED_TOKEN_PROGRAM_ID, SYSTEM_PROGRAM_ID, TOKEN_PROGRAM_ID,
};
use crate::types::{AccountMeta, Instruction, Pubkey, find_program_address};

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
    let token_program_id = Pubkey::from_base58(TOKEN_PROGRAM_ID).unwrap();
    let associated_token_program_id = Pubkey::from_base58(ASSOCIATED_TOKEN_PROGRAM_ID).unwrap();

    let seeds: [&[u8]; 3] = [
        wallet_address.as_bytes(),
        token_program_id.as_bytes(),
        token_mint_address.as_bytes(),
    ];

    find_program_address(&associated_token_program_id, &seeds)
        .expect("Failed to derive associated token address")
        .0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Pubkey;

    fn mint_pubkey() -> Pubkey {
        Pubkey::from_base58("7o36UsWR1JQLpZ9PE2gn9L4SQ69CNNiWAXd4Jt7rqz9Z").unwrap()
    }

    fn owner_pubkey() -> Pubkey {
        Pubkey::from_base58("Hozo7TadHq6PMMiGLGNvgk79Hvj5VTAM7Ny2bamQ2m8q").unwrap()
    }

    fn payer_pubkey() -> Pubkey {
        Pubkey::from_base58("3ECJhLBQ9DAuKBKNjQGLEk3YqoFcF1YvhdayQ2C96eXF").unwrap()
    }

    #[test]
    fn test_create_associated_token_account() {
        let payer = payer_pubkey();
        let wallet_address = owner_pubkey();
        let token_mint_address = mint_pubkey();

        let instruction =
            create_associated_token_account(&payer, &wallet_address, &token_mint_address);

        assert_eq!(
            instruction.program_id,
            Pubkey::from_base58(ASSOCIATED_TOKEN_PROGRAM_ID).unwrap()
        );

        // Check that accounts are in the expected order
        assert_eq!(instruction.accounts.len(), 7);

        // Payer
        assert_eq!(instruction.accounts[0].pubkey, payer);
        assert!(instruction.accounts[0].is_signer);
        assert!(instruction.accounts[0].is_writable);

        // Associated token account
        let associated_token_address =
            get_associated_token_address(&wallet_address, &token_mint_address);
        assert_eq!(instruction.accounts[1].pubkey, associated_token_address);
        assert!(!instruction.accounts[1].is_signer);
        assert!(instruction.accounts[1].is_writable);

        // Wallet address
        assert_eq!(instruction.accounts[2].pubkey, wallet_address);
        assert!(!instruction.accounts[2].is_signer);
        assert!(!instruction.accounts[2].is_writable);

        // Token mint
        assert_eq!(instruction.accounts[3].pubkey, token_mint_address);
        assert!(!instruction.accounts[3].is_signer);
        assert!(!instruction.accounts[3].is_writable);

        // System program
        assert_eq!(
            instruction.accounts[4].pubkey,
            Pubkey::from_base58(SYSTEM_PROGRAM_ID).unwrap()
        );
        assert!(!instruction.accounts[4].is_signer);
        assert!(!instruction.accounts[4].is_writable);

        // Token program
        assert_eq!(
            instruction.accounts[5].pubkey,
            Pubkey::from_base58(TOKEN_PROGRAM_ID).unwrap()
        );
        assert!(!instruction.accounts[5].is_signer);
        assert!(!instruction.accounts[5].is_writable);

        // Rent sysvar
        assert_eq!(
            instruction.accounts[6].pubkey,
            Pubkey::from_base58("SysvarRent111111111111111111111111111111111").unwrap()
        );
        assert!(!instruction.accounts[6].is_signer);
        assert!(!instruction.accounts[6].is_writable);

        // Check data - should be empty
        assert_eq!(instruction.data, Vec::<u8>::new());
    }
}
