//! Enhanced instruction builder

use crate::{AccountMeta, Instruction, Pubkey};

/// A builder for constructing Solana instructions
#[derive(Debug)]
pub struct InstructionBuilder {
    /// The program ID that will process this instruction
    program_id: Pubkey,
    /// The accounts that will be read from or written to
    accounts: Vec<AccountMeta>,
    /// The instruction data
    data: Vec<u8>,
}

impl InstructionBuilder {
    /// Create a new instruction builder
    pub fn new(program_id: Pubkey) -> Self {
        Self {
            program_id,
            accounts: Vec::new(),
            data: Vec::new(),
        }
    }

    /// Add an account to the instruction
    pub fn account(mut self, pubkey: Pubkey, is_signer: bool, is_writable: bool) -> Self {
        self.accounts.push(AccountMeta {
            pubkey,
            is_signer,
            is_writable,
        });
        self
    }

    /// Add multiple accounts to the instruction
    pub fn accounts(mut self, accounts: Vec<AccountMeta>) -> Self {
        self.accounts.extend(accounts);
        self
    }

    /// Set the instruction data
    pub fn data(mut self, data: Vec<u8>) -> Self {
        self.data = data;
        self
    }

    /// Build the instruction
    pub fn build(self) -> Instruction {
        Instruction {
            program_id: self.program_id,
            accounts: self.accounts,
            data: self.data,
        }
    }

    /// Get the current program ID
    pub fn program_id(&self) -> &Pubkey {
        &self.program_id
    }

    /// Get the current accounts
    pub fn get_accounts(&self) -> &[AccountMeta] {
        &self.accounts
    }

    /// Get the current data
    pub fn get_data(&self) -> &[u8] {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::token::transfer_checked;

    fn token_program() -> Pubkey {
        Pubkey::from_base58("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA").unwrap()
    }

    fn mint_pubkey() -> Pubkey {
        Pubkey::from_base58("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap()
    }

    fn token_pubkey() -> Pubkey {
        Pubkey::from_base58("4q2wPZuZwQTB1dEU9sMGsJK1d8NSL1hpBjTGHBsLQNDh").unwrap()
    }

    fn authority_pubkey() -> Pubkey {
        Pubkey::from_base58("Hozo7TadHq6PMMiGLGNvgk79Hvj5VTAM7Ny2bamQ2m8q").unwrap()
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

    #[test]
    fn test_instruction_builder() {
        let program_id = token_program();
        let source = token_pubkey();
        let dest = random_pubkey();
        let owner = authority_pubkey();
        let mint = mint_pubkey();
        let amount = 1_000_000;
        let decimals = 6;

        // Build the instruction using the builder
        let builder = InstructionBuilder::new(program_id)
            .account(source, false, true)
            .account(mint, false, false)
            .account(dest, false, true)
            .account(owner, true, false)
            .data(vec![12, 0, 0, 0, 64, 66, 15, 0, 0, 0, 0, 0, decimals]);

        // Build using helper function for comparison
        let ix = transfer_checked(&source, &mint, &dest, &owner, amount, decimals);

        // Check program ID
        assert_eq!(builder.program_id(), &token_program());
        assert_eq!(ix.program_id, token_program());

        // Check accounts length
        assert_eq!(builder.get_accounts().len(), 4);
        assert_eq!(ix.accounts.len(), 4);
    }

    #[test]
    fn test_instruction_builder_empty() {
        let program_id = Pubkey::new([1; 32]);
        let builder = InstructionBuilder::new(program_id);

        assert_eq!(builder.program_id(), &program_id);
        assert!(builder.get_accounts().is_empty());
        assert!(builder.get_data().is_empty());

        let instruction = builder.build();
        assert_eq!(instruction.program_id, program_id);
        assert!(instruction.accounts.is_empty());
        assert!(instruction.data.is_empty());
    }

    #[test]
    fn test_instruction_builder_multiple_accounts() {
        let program_id = Pubkey::new([1; 32]);
        let account1 = Pubkey::new([2; 32]);
        let account2 = Pubkey::new([3; 32]);

        let accounts = vec![
            AccountMeta {
                pubkey: account1,
                is_signer: true,
                is_writable: false,
            },
            AccountMeta {
                pubkey: account2,
                is_signer: false,
                is_writable: true,
            },
        ];

        let builder = InstructionBuilder::new(program_id)
            .accounts(accounts.clone())
            .data(vec![1, 2, 3, 4]);

        assert_eq!(builder.get_accounts().len(), 2);
        assert_eq!(builder.get_accounts()[0], accounts[0]);
        assert_eq!(builder.get_accounts()[1], accounts[1]);
        assert_eq!(builder.get_data(), &[1, 2, 3, 4]);
    }
}