use crate::instructions::program_ids::COMPUTE_BUDGET_PROGRAM_ID;
use crate::types::{Instruction, Pubkey};

/// Compute Budget Instructions
pub enum ComputeBudgetInstruction {
    /// Request a specific transaction-wide compute unit limit
    RequestUnits {
        /// Units to request
        units: u32,
        /// Additional compute unit fee to pay
        additional_fee: u32,
    },
    /// Request a specific transaction-wide compute unit limit
    RequestHeapFrame {
        /// Stack size in bytes
        bytes: u32,
    },
    /// Request a specific transaction-wide compute unit price
    SetComputeUnitPrice {
        /// Compute unit price to request (in increments of 0.000001 lamports per compute unit)
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
                data.push(2);
                data.extend_from_slice(&units.to_le_bytes());
            }
            Self::SetComputeUnitPrice { micro_lamports } => {
                data.push(3);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_compute_unit_limit_uses_discriminant_2() {
        let ix = ComputeBudgetInstruction::SetComputeUnitLimit { units: 200_000 };
        let data = ix.serialize();
        assert_eq!(data[0], 2);
        assert_eq!(data.len(), 5); // 1 discriminant + 4 u32
        let units = u32::from_le_bytes(data[1..5].try_into().unwrap());
        assert_eq!(units, 200_000);
    }

    #[test]
    fn set_compute_unit_price_uses_discriminant_3() {
        let ix = ComputeBudgetInstruction::SetComputeUnitPrice {
            micro_lamports: 50_000,
        };
        let data = ix.serialize();
        assert_eq!(data[0], 3);
        assert_eq!(data.len(), 9); // 1 discriminant + 8 u64
        let price = u64::from_le_bytes(data[1..9].try_into().unwrap());
        assert_eq!(price, 50_000);
    }

    #[test]
    fn helper_functions_produce_correct_data() {
        let price_ix = set_compute_unit_price(12345);
        assert_eq!(price_ix.data[0], 3);
        assert_eq!(
            u64::from_le_bytes(price_ix.data[1..9].try_into().unwrap()),
            12345
        );

        let limit_ix = set_compute_unit_limit(400_000);
        assert_eq!(limit_ix.data[0], 2);
        assert_eq!(
            u32::from_le_bytes(limit_ix.data[1..5].try_into().unwrap()),
            400_000
        );

        // Both should target the compute budget program
        let cb_program = Pubkey::from_base58(COMPUTE_BUDGET_PROGRAM_ID).unwrap();
        assert_eq!(price_ix.program_id, cb_program);
        assert_eq!(limit_ix.program_id, cb_program);
    }
}
