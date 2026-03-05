use crate::instructions::program_ids::COMPUTE_BUDGET_PROGRAM_ID;
use crate::types::{Instruction, Pubkey};

/// Compute budget instruction discriminant for setting compute unit limit.
pub const SET_COMPUTE_UNIT_LIMIT_DISCRIMINANT: u8 = 2;
/// Compute budget instruction discriminant for setting compute unit price.
pub const SET_COMPUTE_UNIT_PRICE_DISCRIMINANT: u8 = 3;

/// Compute Budget Instructions
pub enum ComputeBudgetInstruction {
    /// Request a specific transaction-wide compute unit limit
    RequestUnits {
        /// Units to request
        units: u32,
        /// Additional compute unit fee to pay
        additional_fee: u32,
    },
    /// Request a specific transaction-wide heap frame size
    RequestHeapFrame {
        /// Stack size in bytes
        bytes: u32,
    },
    /// Request a specific transaction-wide compute unit price
    SetComputeUnitPrice {
        /// Compute unit price to request (in micro-lamports per compute unit)
        micro_lamports: u64,
    },
    /// Request a specific transaction-wide compute unit limit
    SetComputeUnitLimit {
        /// Units to request
        units: u32,
    },
}

impl ComputeBudgetInstruction {
    /// Serialize the compute budget instruction
    pub fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::new();
        match self {
            Self::RequestUnits {
                units,
                additional_fee,
            } => {
                data.push(0);
                data.extend_from_slice(&units.to_le_bytes());
                data.extend_from_slice(&additional_fee.to_le_bytes());
            }
            Self::RequestHeapFrame { bytes } => {
                data.push(1);
                data.extend_from_slice(&bytes.to_le_bytes());
            }
            Self::SetComputeUnitLimit { units } => {
                data.push(SET_COMPUTE_UNIT_LIMIT_DISCRIMINANT);
                data.extend_from_slice(&units.to_le_bytes());
            }
            Self::SetComputeUnitPrice { micro_lamports } => {
                data.push(SET_COMPUTE_UNIT_PRICE_DISCRIMINANT);
                data.extend_from_slice(&micro_lamports.to_le_bytes());
            }
        }
        data
    }
}

/// Request a specific transaction-wide compute unit limit
pub fn request_units(units: u32, additional_fee: u32) -> Instruction {
    Instruction {
        program_id: Pubkey::from_base58(COMPUTE_BUDGET_PROGRAM_ID).unwrap(),
        accounts: vec![],
        data: ComputeBudgetInstruction::RequestUnits {
            units,
            additional_fee,
        }
        .serialize(),
    }
}

/// Request a specific heap frame size
pub fn request_heap_frame(bytes: u32) -> Instruction {
    Instruction {
        program_id: Pubkey::from_base58(COMPUTE_BUDGET_PROGRAM_ID).unwrap(),
        accounts: vec![],
        data: ComputeBudgetInstruction::RequestHeapFrame { bytes }.serialize(),
    }
}

/// Set a specific compute unit price
pub fn set_compute_unit_price(micro_lamports: u64) -> Instruction {
    Instruction {
        program_id: Pubkey::from_base58(COMPUTE_BUDGET_PROGRAM_ID).unwrap(),
        accounts: vec![],
        data: ComputeBudgetInstruction::SetComputeUnitPrice { micro_lamports }.serialize(),
    }
}

/// Set a specific compute unit limit
pub fn set_compute_unit_limit(units: u32) -> Instruction {
    Instruction {
        program_id: Pubkey::from_base58(COMPUTE_BUDGET_PROGRAM_ID).unwrap(),
        accounts: vec![],
        data: ComputeBudgetInstruction::SetComputeUnitLimit { units }.serialize(),
    }
}

/// Parse compute unit limit from one compute budget instruction payload.
pub fn parse_compute_unit_limit_data(data: &[u8]) -> Option<u32> {
    if data.len() == 5 && data[0] == SET_COMPUTE_UNIT_LIMIT_DISCRIMINANT {
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(&data[1..5]);
        Some(u32::from_le_bytes(bytes))
    } else {
        None
    }
}

/// Parse compute unit price from one compute budget instruction payload.
pub fn parse_compute_unit_price_data(data: &[u8]) -> Option<u64> {
    if data.len() == 9 && data[0] == SET_COMPUTE_UNIT_PRICE_DISCRIMINANT {
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&data[1..9]);
        Some(u64::from_le_bytes(bytes))
    } else {
        None
    }
}

/// Get the first compute unit limit present in a list of instructions.
pub fn get_compute_unit_limit(instructions: &[Instruction]) -> Option<u32> {
    let compute_budget_program = Pubkey::from_base58(COMPUTE_BUDGET_PROGRAM_ID).unwrap();
    instructions.iter().find_map(|instruction| {
        if instruction.program_id == compute_budget_program {
            parse_compute_unit_limit_data(&instruction.data)
        } else {
            None
        }
    })
}

/// Ensure a compute unit price instruction exists at the beginning of the instruction list.
/// Returns true when the instruction was inserted and false when it already existed.
pub fn ensure_compute_unit_price(instructions: &mut Vec<Instruction>, micro_lamports: u64) -> bool {
    let compute_budget_program = Pubkey::from_base58(COMPUTE_BUDGET_PROGRAM_ID).unwrap();
    let has_price = instructions.iter().any(|instruction| {
        instruction.program_id == compute_budget_program
            && parse_compute_unit_price_data(&instruction.data).is_some()
    });

    if has_price {
        return false;
    }

    instructions.insert(0, set_compute_unit_price(micro_lamports));
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::program_ids::{compute_budget_program, system_program};
    use crate::instructions::system::transfer;

    #[test]
    fn test_compute_budget_discriminants() {
        let limit_ix = set_compute_unit_limit(200_000);
        let price_ix = set_compute_unit_price(1_000);

        assert_eq!(
            limit_ix.data[0], SET_COMPUTE_UNIT_LIMIT_DISCRIMINANT,
            "limit instruction must use discriminant 2"
        );
        assert_eq!(
            price_ix.data[0], SET_COMPUTE_UNIT_PRICE_DISCRIMINANT,
            "price instruction must use discriminant 3"
        );
    }

    #[test]
    fn test_get_compute_unit_limit() {
        let payer = Pubkey::new([1u8; 32]);
        let recipient = Pubkey::new([2u8; 32]);
        let mut instructions = vec![transfer(&payer, &recipient, 10)];

        assert_eq!(get_compute_unit_limit(&instructions), None);

        instructions.insert(0, set_compute_unit_limit(250_000));
        assert_eq!(get_compute_unit_limit(&instructions), Some(250_000));
    }

    #[test]
    fn test_ensure_compute_unit_price() {
        let payer = Pubkey::new([1u8; 32]);
        let recipient = Pubkey::new([2u8; 32]);
        let mut instructions = vec![transfer(&payer, &recipient, 10)];

        let inserted = ensure_compute_unit_price(&mut instructions, 5_000);
        assert!(inserted);
        assert_eq!(instructions.len(), 2);
        assert_eq!(instructions[0].program_id, compute_budget_program());
        assert_eq!(
            parse_compute_unit_price_data(&instructions[0].data),
            Some(5_000)
        );

        let inserted_again = ensure_compute_unit_price(&mut instructions, 9_999);
        assert!(!inserted_again);
        assert_eq!(instructions.len(), 2);
        assert_eq!(
            parse_compute_unit_price_data(&instructions[0].data),
            Some(5_000)
        );

        assert_eq!(instructions[1].program_id, system_program());
    }
}
