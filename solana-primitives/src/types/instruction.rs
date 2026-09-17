use super::pubkey::Pubkey;

/// Represents a Solana instruction
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Instruction {
    /// The program ID that will process this instruction
    #[cfg_attr(feature = "serde", serde(alias = "programId"))]
    pub program_id: Pubkey,
    /// The accounts that will be read from or written to
    #[cfg_attr(feature = "serde", serde(alias = "keys"))]
    pub accounts: Vec<AccountMeta>,
    /// The instruction data
    pub data: Vec<u8>,
}

/// Metadata about an account in an instruction
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AccountMeta {
    /// The account's public key
    #[cfg_attr(feature = "serde", serde(alias = "publicKey"))]
    pub pubkey: Pubkey,
    /// Whether the account is a signer
    #[cfg_attr(feature = "serde", serde(alias = "isSigner"))]
    pub is_signer: bool,
    /// Whether the account is writable
    #[cfg_attr(feature = "serde", serde(alias = "isWritable"))]
    pub is_writable: bool,
}

impl AccountMeta {
    /// Create a new AccountMeta with explicit signer/writable flags.
    pub fn new(pubkey: Pubkey, is_signer: bool, is_writable: bool) -> Self {
        Self {
            pubkey,
            is_signer,
            is_writable,
        }
    }

    /// A read-only account that does not sign.
    pub fn new_readonly(pubkey: Pubkey) -> Self {
        Self::new(pubkey, false, false)
    }

    /// A read-only account that signs.
    pub fn new_signer(pubkey: Pubkey) -> Self {
        Self::new(pubkey, true, false)
    }

    /// A writable account that does not sign.
    pub fn new_writable(pubkey: Pubkey) -> Self {
        Self::new(pubkey, false, true)
    }

    /// A writable account that signs.
    pub fn new_signer_writable(pubkey: Pubkey) -> Self {
        Self::new(pubkey, true, true)
    }
}

/// A compiled instruction that references accounts by their indices
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CompiledInstruction {
    /// Index into the account keys array indicating the program to execute
    pub program_id_index: u8,
    /// Indices into the account keys array indicating which accounts to pass to the program
    pub accounts: Vec<u8>,
    /// The instruction data
    pub data: Vec<u8>,
}

#[cfg(all(test, feature = "serde"))]
mod tests {
    use super::*;

    #[test]
    fn serde_accepts_web3_field_names() {
        let program = Pubkey::new([1; 32]);
        let account = Pubkey::new([2; 32]);
        let instruction = Instruction {
            program_id: program,
            accounts: vec![AccountMeta::new_signer(account)],
            data: vec![1, 2],
        };

        let json = serde_json::to_value(&instruction).unwrap();
        assert_eq!(
            json,
            serde_json::json!({
                "program_id": program.to_string(),
                "accounts": [{"pubkey": account.to_string(), "is_signer": true, "is_writable": false}],
                "data": [1, 2],
            })
        );

        let web3 = serde_json::json!({
            "programId": program.to_string(),
            "keys": [{"pubkey": account.to_string(), "isSigner": true, "isWritable": false}],
            "data": [1, 2],
        });
        assert_eq!(
            serde_json::from_value::<Instruction>(web3).unwrap(),
            instruction
        );
    }
}
