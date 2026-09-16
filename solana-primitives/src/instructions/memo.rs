use crate::instructions::program_ids::memo_program;
use crate::types::{AccountMeta, Instruction, Pubkey};

/// Create a Memo program instruction; each signer must sign the transaction.
pub fn memo(memo_text: &str, signers: &[&Pubkey]) -> Instruction {
    Instruction {
        program_id: memo_program(),
        accounts: signers
            .iter()
            .map(|signer| AccountMeta::new_signer(**signer))
            .collect(),
        data: memo_text.as_bytes().to_vec(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::key;

    #[test]
    fn memo_accounts_are_readonly_signers() {
        let (a, b) = (key("a"), key("b"));
        assert_eq!(
            memo("hi", &[&a, &b]),
            Instruction {
                program_id: Pubkey::from_str_const("MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr"),
                accounts: vec![AccountMeta::new_signer(a), AccountMeta::new_signer(b)],
                data: b"hi".to_vec(),
            }
        );
    }
}
