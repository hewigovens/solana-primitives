//! Compute Budget program instructions for legacy and v0 transactions.
//!
//! v1 transactions carry these settings in their message config instead, and
//! the runtime ignores Compute Budget instructions in them.

use crate::instructions::program_ids::compute_budget_program;
use crate::instructions::system::is_advance_nonce_instruction;
use crate::types::Instruction;

/// Compute budget instruction discriminant for requesting a heap frame.
pub const REQUEST_HEAP_FRAME_DISCRIMINANT: u8 = 1;
/// Compute budget instruction discriminant for setting compute unit limit.
pub const SET_COMPUTE_UNIT_LIMIT_DISCRIMINANT: u8 = 2;
/// Compute budget instruction discriminant for setting compute unit price.
pub const SET_COMPUTE_UNIT_PRICE_DISCRIMINANT: u8 = 3;
/// Compute budget instruction discriminant for setting the loaded accounts data size limit.
pub const SET_LOADED_ACCOUNTS_DATA_SIZE_LIMIT_DISCRIMINANT: u8 = 4;

/// Compute Budget program instructions.
///
/// Encoded as Borsh: a one-byte discriminant followed by the little-endian
/// value. Discriminant 0 is a retired variant that the runtime rejects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComputeBudgetInstruction {
    /// Request a transaction-wide program heap size in bytes (a multiple of 1024).
    RequestHeapFrame(u32),
    /// Set the transaction-wide compute unit limit.
    SetComputeUnitLimit(u32),
    /// Set the compute unit price in micro-lamports per compute unit.
    SetComputeUnitPrice(u64),
    /// Set the transaction-wide limit on loaded account data, in bytes.
    SetLoadedAccountsDataSizeLimit(u32),
}

impl ComputeBudgetInstruction {
    /// Serialize the instruction data.
    pub fn serialize(&self) -> Vec<u8> {
        let mut data = vec![self.discriminant()];
        match self {
            Self::RequestHeapFrame(value)
            | Self::SetComputeUnitLimit(value)
            | Self::SetLoadedAccountsDataSizeLimit(value) => {
                data.extend_from_slice(&value.to_le_bytes())
            }
            Self::SetComputeUnitPrice(value) => data.extend_from_slice(&value.to_le_bytes()),
        }
        data
    }

    /// Parse instruction data the way the runtime does: bytes after the value are ignored.
    pub fn parse(data: &[u8]) -> Option<Self> {
        let (&discriminant, value) = data.split_first()?;
        let u32_value = || value.first_chunk().copied().map(u32::from_le_bytes);
        match discriminant {
            REQUEST_HEAP_FRAME_DISCRIMINANT => u32_value().map(Self::RequestHeapFrame),
            SET_COMPUTE_UNIT_LIMIT_DISCRIMINANT => u32_value().map(Self::SetComputeUnitLimit),
            SET_COMPUTE_UNIT_PRICE_DISCRIMINANT => value
                .first_chunk()
                .copied()
                .map(u64::from_le_bytes)
                .map(Self::SetComputeUnitPrice),
            SET_LOADED_ACCOUNTS_DATA_SIZE_LIMIT_DISCRIMINANT => {
                u32_value().map(Self::SetLoadedAccountsDataSizeLimit)
            }
            _ => None,
        }
    }

    fn discriminant(&self) -> u8 {
        match self {
            Self::RequestHeapFrame(_) => REQUEST_HEAP_FRAME_DISCRIMINANT,
            Self::SetComputeUnitLimit(_) => SET_COMPUTE_UNIT_LIMIT_DISCRIMINANT,
            Self::SetComputeUnitPrice(_) => SET_COMPUTE_UNIT_PRICE_DISCRIMINANT,
            Self::SetLoadedAccountsDataSizeLimit(_) => {
                SET_LOADED_ACCOUNTS_DATA_SIZE_LIMIT_DISCRIMINANT
            }
        }
    }

    fn into_instruction(self) -> Instruction {
        Instruction {
            program_id: compute_budget_program(),
            accounts: vec![],
            data: self.serialize(),
        }
    }
}

/// Request a specific heap frame size
pub fn request_heap_frame(bytes: u32) -> Instruction {
    ComputeBudgetInstruction::RequestHeapFrame(bytes).into_instruction()
}

/// Set a specific compute unit price, in micro-lamports per compute unit
pub fn set_compute_unit_price(micro_lamports: u64) -> Instruction {
    ComputeBudgetInstruction::SetComputeUnitPrice(micro_lamports).into_instruction()
}

/// Set a specific compute unit limit
pub fn set_compute_unit_limit(units: u32) -> Instruction {
    ComputeBudgetInstruction::SetComputeUnitLimit(units).into_instruction()
}

/// Set a limit on the total size of account data the transaction loads
pub fn set_loaded_accounts_data_size_limit(bytes: u32) -> Instruction {
    ComputeBudgetInstruction::SetLoadedAccountsDataSizeLimit(bytes).into_instruction()
}

/// Parse compute unit limit from one compute budget instruction payload.
pub fn parse_compute_unit_limit_data(data: &[u8]) -> Option<u32> {
    match ComputeBudgetInstruction::parse(data)? {
        ComputeBudgetInstruction::SetComputeUnitLimit(units) => Some(units),
        _ => None,
    }
}

/// Parse compute unit price from one compute budget instruction payload.
pub fn parse_compute_unit_price_data(data: &[u8]) -> Option<u64> {
    match ComputeBudgetInstruction::parse(data)? {
        ComputeBudgetInstruction::SetComputeUnitPrice(micro_lamports) => Some(micro_lamports),
        _ => None,
    }
}

/// Heap requests must be a multiple of 1024 within this range.
const HEAP_FRAME_BYTES: std::ops::RangeInclusive<u32> = 32 * 1024..=256 * 1024;

/// Parse Compute Budget instruction data the way the runtime does, keeping each
/// instruction's position.
///
/// Returns `None` when the runtime would fail the transaction: an instruction
/// that doesn't parse, a request kind that appears twice, or an invalid heap size.
pub(crate) fn parse_compute_budget_requests<'a>(
    instructions: impl IntoIterator<Item = (usize, &'a [u8])>,
) -> Option<Vec<(usize, ComputeBudgetInstruction)>> {
    let mut requests: Vec<(usize, ComputeBudgetInstruction)> = Vec::new();
    for (index, data) in instructions {
        let request = ComputeBudgetInstruction::parse(data)?;
        if let ComputeBudgetInstruction::RequestHeapFrame(bytes) = request
            && (!bytes.is_multiple_of(1024) || !HEAP_FRAME_BYTES.contains(&bytes))
        {
            return None;
        }
        if requests
            .iter()
            .any(|(_, seen)| seen.discriminant() == request.discriminant())
        {
            return None;
        }
        requests.push((index, request));
    }
    Some(requests)
}

/// The Compute Budget requests in a list, in order.
///
/// Returns `None` when the runtime would fail the transaction: a Compute Budget
/// instruction that doesn't parse, a request kind that appears twice, or an
/// invalid heap size.
pub fn compute_budget_instructions(
    instructions: &[Instruction],
) -> Option<Vec<ComputeBudgetInstruction>> {
    let program_id = compute_budget_program();
    let requests = instructions
        .iter()
        .enumerate()
        .filter(|(_, instruction)| instruction.program_id == program_id)
        .map(|(index, instruction)| (index, instruction.data.as_slice()));
    parse_compute_budget_requests(requests)
        .map(|requests| requests.into_iter().map(|(_, request)| request).collect())
}

/// The compute unit limit the runtime would apply, if one is requested.
pub fn get_compute_unit_limit(instructions: &[Instruction]) -> Option<u32> {
    compute_budget_instructions(instructions)?
        .into_iter()
        .find_map(|instruction| match instruction {
            ComputeBudgetInstruction::SetComputeUnitLimit(units) => Some(units),
            _ => None,
        })
}

/// The compute unit price the runtime would apply, if one is requested.
pub fn get_compute_unit_price(instructions: &[Instruction]) -> Option<u64> {
    compute_budget_instructions(instructions)?
        .into_iter()
        .find_map(|instruction| match instruction {
            ComputeBudgetInstruction::SetComputeUnitPrice(micro_lamports) => Some(micro_lamports),
            _ => None,
        })
}

/// Insert a compute unit price instruction at the start of the list (after a
/// leading `AdvanceNonceAccount`) unless the list already has one.
///
/// Returns whether an instruction was inserted. Nothing is inserted when the
/// existing Compute Budget instructions are invalid (see
/// [`compute_budget_instructions`]).
pub fn ensure_compute_unit_price(instructions: &mut Vec<Instruction>, micro_lamports: u64) -> bool {
    match compute_budget_instructions(instructions) {
        Some(requests)
            if !requests.iter().any(|request| {
                matches!(request, ComputeBudgetInstruction::SetComputeUnitPrice(_))
            }) => {}
        _ => return false,
    }

    // Durable-nonce txs require AdvanceNonceAccount as instruction 0; insert after it.
    let insert_pos = usize::from(
        instructions
            .first()
            .is_some_and(is_advance_nonce_instruction),
    );
    instructions.insert(insert_pos, set_compute_unit_price(micro_lamports));
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::program_ids::COMPUTE_BUDGET_PROGRAM_ID;
    use crate::instructions::system::{advance_nonce_account, transfer};
    use crate::test_utils::{key, pubkey};
    use hexlit::hex;

    #[test]
    fn instructions_match_upstream() {
        // Vectors from `solana-compute-budget-interface` 3.1.
        let cases = [
            (request_heap_frame(65_536), &hex!("0100000100")[..]),
            (set_compute_unit_limit(1_400_000), &hex!("02c05c1500")),
            (
                set_compute_unit_price(u64::MAX - 1),
                &hex!("03feffffffffffffff"),
            ),
            (
                set_loaded_accounts_data_size_limit(123_456),
                &hex!("0440e20100"),
            ),
        ];
        for (instruction, data) in cases {
            assert_eq!(
                instruction,
                Instruction {
                    program_id: pubkey(COMPUTE_BUDGET_PROGRAM_ID),
                    accounts: vec![],
                    data: data.to_vec(),
                }
            );
            assert!(ComputeBudgetInstruction::parse(data).is_some());
        }
    }

    #[test]
    fn parse_follows_the_runtime() {
        assert_eq!(
            ComputeBudgetInstruction::parse(&hex!("02c05c1500")),
            Some(ComputeBudgetInstruction::SetComputeUnitLimit(1_400_000))
        );
        // Trailing bytes are ignored, as Borsh `try_from_slice_unchecked` does.
        assert_eq!(
            ComputeBudgetInstruction::parse(&hex!("02c05c1500ff")),
            Some(ComputeBudgetInstruction::SetComputeUnitLimit(1_400_000))
        );
        for data in [
            &[][..],
            &hex!("00"),
            &hex!("0000000000"),
            &hex!("02c05c15"),
            &hex!("03feffffffffffff"),
            &hex!("05"),
        ] {
            assert_eq!(ComputeBudgetInstruction::parse(data), None, "{data:02x?}");
        }
    }

    #[test]
    fn getters() {
        let payer = key("payer");
        let mut instructions = vec![transfer(&payer, &key("recipient"), 10)];
        assert_eq!(get_compute_unit_limit(&instructions), None);
        assert_eq!(get_compute_unit_price(&instructions), None);

        instructions.extend([set_compute_unit_limit(250_000), set_compute_unit_price(7)]);
        assert_eq!(get_compute_unit_limit(&instructions), Some(250_000));
        assert_eq!(get_compute_unit_price(&instructions), Some(7));
        assert_eq!(
            parse_compute_unit_limit_data(&instructions[1].data),
            Some(250_000)
        );
        assert_eq!(parse_compute_unit_price_data(&instructions[1].data), None);
    }

    #[test]
    fn invalid_requests_have_no_values() {
        let payer = key("payer");
        let transfer = transfer(&payer, &key("recipient"), 10);
        let retired = Instruction {
            data: vec![0],
            ..set_compute_unit_limit(0)
        };
        let invalid_lists = [
            vec![retired, set_compute_unit_limit(5)],
            vec![set_compute_unit_limit(5), set_compute_unit_limit(6)],
            vec![set_compute_unit_limit(5), request_heap_frame(1000)],
            vec![set_compute_unit_limit(5), request_heap_frame(512 * 1024)],
        ];
        for mut instructions in invalid_lists {
            assert_eq!(compute_budget_instructions(&instructions), None);
            assert_eq!(get_compute_unit_limit(&instructions), None);
            let before = instructions.clone();
            assert!(!ensure_compute_unit_price(&mut instructions, 1));
            assert_eq!(instructions, before);
        }

        let valid = vec![
            transfer,
            request_heap_frame(64 * 1024),
            set_compute_unit_limit(5),
        ];
        assert_eq!(
            compute_budget_instructions(&valid),
            Some(vec![
                ComputeBudgetInstruction::RequestHeapFrame(64 * 1024),
                ComputeBudgetInstruction::SetComputeUnitLimit(5),
            ])
        );
    }

    #[test]
    fn ensure_compute_unit_price_inserts_once() {
        let payer = key("payer");
        let transfer = transfer(&payer, &key("recipient"), 10);
        let mut instructions = vec![transfer.clone()];

        assert!(ensure_compute_unit_price(&mut instructions, 5_000));
        assert_eq!(
            instructions,
            vec![set_compute_unit_price(5_000), transfer.clone()]
        );

        assert!(!ensure_compute_unit_price(&mut instructions, 9_999));
        assert_eq!(instructions, vec![set_compute_unit_price(5_000), transfer]);
    }

    #[test]
    fn ensure_compute_unit_price_keeps_advance_nonce_first() {
        let payer = key("payer");
        let advance = advance_nonce_account(&key("nonce"), &payer);
        let transfer = transfer(&payer, &key("recipient"), 10);
        let mut instructions = vec![advance.clone(), transfer.clone()];

        assert!(ensure_compute_unit_price(&mut instructions, 5_000));
        assert_eq!(
            instructions,
            vec![advance, set_compute_unit_price(5_000), transfer]
        );
    }
}
