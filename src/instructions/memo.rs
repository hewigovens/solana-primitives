use crate::instructions::program_ids::MEMO_PROGRAM_ID;
use crate::types::{AccountMeta, Instruction, Pubkey};

/// Create a memo instruction
pub fn memo(memo_text: &str, signers: &[&Pubkey]) -> Instruction {
    let account_metas = signers
        .iter()
        .map(|signer| AccountMeta {
            pubkey: *(*signer),
            is_signer: true,
            is_writable: false,
        })
        .collect::<Vec<AccountMeta>>();

    Instruction {
        program_id: Pubkey::from_base58(MEMO_PROGRAM_ID).unwrap(),
        accounts: account_metas,
        data: memo_text.as_bytes().to_vec(),
    }
}
