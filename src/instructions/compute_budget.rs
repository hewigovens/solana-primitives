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
    /// Set a specific compute unit limit for a single instruction
    /// (deprecated)
    RequestUnitDeprecated {
        /// Units to request
        units: u32,
    },
    /// Request a specific transaction-wide compute unit price
    /// (deprecated)
    SetComputeUnitPriceDeprecated {
        /// Compute unit price to request
        micro_lamports: u32,
    },
    /// Request a specific transaction-wide compute unit limit
    RequestHeapFrame {
        /// Stack size in bytes
        bytes: u32,
    },
    /// Request a specific transaction-wide compute unit price
    /// This replaces the deprecated SetComputeUnitPrice
    SetComputeUnitPrice {
        /// Compute unit price to request (in increments of 0.000001 lamports per compute unit)
        micro_lamports: u64,
    },
    /// Request a specific transaction-wide compute unit limit
    /// This replaces the deprecated RequestUnits
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
            Self::RequestUnitDeprecated { units } => {
                data.push(1);
                data.extend_from_slice(&units.to_le_bytes());
            }
            Self::SetComputeUnitPriceDeprecated { micro_lamports } => {
                data.push(2);
                data.extend_from_slice(&micro_lamports.to_le_bytes());
            }
            Self::RequestHeapFrame { bytes } => {
                data.push(3);
                data.extend_from_slice(&bytes.to_le_bytes());
            }
            Self::SetComputeUnitPrice { micro_lamports } => {
                data.push(4);
                data.extend_from_slice(&micro_lamports.to_le_bytes());
            }
            Self::SetComputeUnitLimit { units } => {
                data.push(5);
                data.extend_from_slice(&units.to_le_bytes());
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
