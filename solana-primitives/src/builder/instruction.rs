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

    /// Add an AccountMeta directly
    pub fn account_meta(mut self, account_meta: AccountMeta) -> Self {
        self.accounts.push(account_meta);
        self
    }

    /// Add multiple accounts at once
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
}

#[cfg(test)]
mod tests {
    use super::InstructionBuilder;
    use crate::Pubkey;
    use crate::instructions::{program_ids::token_program, token::transfer_checked};

    fn mint_pubkey() -> Pubkey {
        Pubkey::from_base58("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v").unwrap()
    }

    fn token_pubkey() -> Pubkey {
        Pubkey::from_base58("4q2wPZuZwQTB1dEU9sMGsJK1d8NSL1hpBjTGHBsLQNDh").unwrap()
    }

    fn authority_pubkey() -> Pubkey {
        Pubkey::from_base58("Hozo7TadHq6PMMiGLGNvgk79Hvj5VTAM7Ny2bamQ2m8q").unwrap()
    }

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

        let builder_ix = InstructionBuilder::new(program_id)
            .account(source, false, true)
            .account(mint, false, false)
            .account(dest, false, true)
            .account(owner, true, false)
            .build();

        let ix = transfer_checked(&source, &mint, &dest, &owner, amount, decimals);

        assert_eq!(builder_ix.program_id, token_program());
        assert_eq!(ix.program_id, token_program());
    }
}
