// Re-export instruction modules
pub mod associated_token;
pub mod compute_budget;
pub mod memo;
pub mod system;
pub mod token;

// Program IDs
pub mod program_ids {
    /// System program ID
    pub const SYSTEM_PROGRAM_ID: &str = "11111111111111111111111111111111";

    /// Token program ID
    pub const TOKEN_PROGRAM_ID: &str = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

    /// Token 2022 program ID
    pub const TOKEN_2022_PROGRAM_ID: &str = "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";

    /// Associated Token program ID
    pub const ASSOCIATED_TOKEN_PROGRAM_ID: &str = "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";

    /// Memo program ID
    pub const MEMO_PROGRAM_ID: &str = "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr";

    /// BPF Loader program ID
    pub const BPF_LOADER_PROGRAM_ID: &str = "BPFLoaderUpgradeab1e11111111111111111111111";

    /// Compute Budget program ID
    pub const COMPUTE_BUDGET_PROGRAM_ID: &str = "ComputeBudget111111111111111111111111111111";
}

// Re-export commonly used types
pub use crate::types::{AccountMeta, Instruction, Pubkey};
