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
        self.accounts
            .push(AccountMeta::new(pubkey, is_signer, is_writable));
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
    use crate::AccountMeta;
    use crate::instructions::{program_ids::token_program, token::transfer_checked};
    use crate::test_utils::key;

    #[test]
    fn builds_the_same_instruction_as_the_helper() {
        let (source, mint, destination, owner) =
            (key("source"), key("mint"), key("destination"), key("owner"));
        let data = transfer_checked(&source, &mint, &destination, &owner, 1_000_000, 6).data;

        let built = InstructionBuilder::new(token_program())
            .account(source, false, true)
            .account_meta(AccountMeta::new_readonly(mint))
            .accounts(vec![
                AccountMeta::new_writable(destination),
                AccountMeta::new_signer(owner),
            ])
            .data(data)
            .build();

        assert_eq!(
            built,
            transfer_checked(&source, &mint, &destination, &owner, 1_000_000, 6)
        );
    }
}
