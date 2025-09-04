// Re-export instruction modules
pub mod associated_token;
pub mod compute_budget;
pub mod memo;
pub mod system;
pub mod token;

// Program IDs
pub mod program_ids {
    use crate::types::Pubkey;

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

    /// Helper function to get System program Pubkey
    pub fn system_program() -> Pubkey {
        Pubkey::from_base58(SYSTEM_PROGRAM_ID).unwrap()
    }

    /// Helper function to get Token program Pubkey
    pub fn token_program() -> Pubkey {
        Pubkey::from_base58(TOKEN_PROGRAM_ID).unwrap()
    }

    /// Helper function to get Token 2022 program Pubkey
    pub fn token_2022_program() -> Pubkey {
        Pubkey::from_base58(TOKEN_2022_PROGRAM_ID).unwrap()
    }

    /// Helper function to get Associated Token program Pubkey
    pub fn associated_token_program() -> Pubkey {
        Pubkey::from_base58(ASSOCIATED_TOKEN_PROGRAM_ID).unwrap()
    }

    /// Helper function to get Memo program Pubkey
    pub fn memo_program() -> Pubkey {
        Pubkey::from_base58(MEMO_PROGRAM_ID).unwrap()
    }

    /// Helper function to get BPF Loader program Pubkey
    pub fn bpf_loader_program() -> Pubkey {
        Pubkey::from_base58(BPF_LOADER_PROGRAM_ID).unwrap()
    }

    /// Helper function to get Compute Budget program Pubkey
    pub fn compute_budget_program() -> Pubkey {
        Pubkey::from_base58(COMPUTE_BUDGET_PROGRAM_ID).unwrap()
    }
}
