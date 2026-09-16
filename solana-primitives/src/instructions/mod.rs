//! Instruction constructors for common programs.

pub mod anchor;
pub mod associated_token;
pub mod compute_budget;
pub mod memo;
pub mod system;
pub mod token;

/// Well-known program and sysvar addresses.
///
/// `*_ID` constants are base58 strings; the unsuffixed constants and helper
/// functions are the decoded [`Pubkey`](crate::Pubkey)s.
pub mod program_ids {
    use crate::types::Pubkey;

    macro_rules! program_ids {
        ($($(#[$doc:meta])* $id:ident, $key:ident, $getter:ident = $address:literal;)*) => {
            $(
                $(#[$doc])*
                pub const $id: &str = $address;
                $(#[$doc])*
                pub const $key: Pubkey = Pubkey::from_str_const($id);
                $(#[$doc])*
                pub const fn $getter() -> Pubkey {
                    $key
                }
            )*
        };
    }

    program_ids! {
        /// System program.
        SYSTEM_PROGRAM_ID, SYSTEM_PROGRAM, system_program =
            "11111111111111111111111111111111";
        /// SPL Token program.
        TOKEN_PROGRAM_ID, TOKEN_PROGRAM, token_program =
            "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";
        /// SPL Token-2022 program.
        TOKEN_2022_PROGRAM_ID, TOKEN_2022_PROGRAM, token_2022_program =
            "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb";
        /// Associated Token Account program.
        ASSOCIATED_TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM, associated_token_program =
            "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL";
        /// Memo program (v2).
        MEMO_PROGRAM_ID, MEMO_PROGRAM, memo_program =
            "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr";
        /// Upgradeable BPF loader (loader v3).
        BPF_LOADER_UPGRADEABLE_PROGRAM_ID, BPF_LOADER_UPGRADEABLE_PROGRAM, bpf_loader_upgradeable_program =
            "BPFLoaderUpgradeab1e11111111111111111111111";
        /// Compute Budget program.
        COMPUTE_BUDGET_PROGRAM_ID, COMPUTE_BUDGET_PROGRAM, compute_budget_program =
            "ComputeBudget111111111111111111111111111111";
        /// Rent sysvar.
        SYSVAR_RENT_ID, SYSVAR_RENT, rent_sysvar =
            "SysvarRent111111111111111111111111111111111";
        /// Recent blockhashes sysvar, still required by the nonce instructions.
        SYSVAR_RECENT_BLOCKHASHES_ID, SYSVAR_RECENT_BLOCKHASHES, recent_blockhashes_sysvar =
            "SysvarRecentB1ockHashes11111111111111111111";
    }

    /// The upgradeable BPF loader; see [`BPF_LOADER_UPGRADEABLE_PROGRAM_ID`].
    #[deprecated(since = "0.3.0", note = "use `BPF_LOADER_UPGRADEABLE_PROGRAM_ID`")]
    pub const BPF_LOADER_PROGRAM_ID: &str = BPF_LOADER_UPGRADEABLE_PROGRAM_ID;

    /// The upgradeable BPF loader; see [`bpf_loader_upgradeable_program`].
    #[deprecated(since = "0.3.0", note = "use `bpf_loader_upgradeable_program`")]
    pub const fn bpf_loader_program() -> Pubkey {
        BPF_LOADER_UPGRADEABLE_PROGRAM
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn constants_match_their_base58_strings() {
            for (id, key) in [
                (SYSTEM_PROGRAM_ID, SYSTEM_PROGRAM),
                (TOKEN_PROGRAM_ID, TOKEN_PROGRAM),
                (TOKEN_2022_PROGRAM_ID, TOKEN_2022_PROGRAM),
                (ASSOCIATED_TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM),
                (MEMO_PROGRAM_ID, MEMO_PROGRAM),
                (
                    BPF_LOADER_UPGRADEABLE_PROGRAM_ID,
                    BPF_LOADER_UPGRADEABLE_PROGRAM,
                ),
                (COMPUTE_BUDGET_PROGRAM_ID, COMPUTE_BUDGET_PROGRAM),
                (SYSVAR_RENT_ID, SYSVAR_RENT),
                (SYSVAR_RECENT_BLOCKHASHES_ID, SYSVAR_RECENT_BLOCKHASHES),
            ] {
                assert_eq!(key.to_base58(), id);
            }
            assert_eq!(SYSTEM_PROGRAM, Pubkey::default());
        }
    }
}
