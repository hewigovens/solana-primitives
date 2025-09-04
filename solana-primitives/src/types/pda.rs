use crate::error::{Result, SolanaError};
use crate::types::Pubkey;
use ed25519_dalek::VerifyingKey;
use sha2::{Digest, Sha256};

/// Maximum number of seeds allowed in a PDA
pub const MAX_SEEDS: usize = 16;
/// Maximum length of a seed in bytes
pub const MAX_SEED_LEN: usize = 32;

/// Find a program address and bump seed for the given seeds
pub fn find_program_address(program_id: &Pubkey, seeds: &[&[u8]]) -> Result<(Pubkey, u8)> {
    // Validate seeds
    if seeds.len() > MAX_SEEDS {
        return Err(SolanaError::InvalidPubkey(format!(
            "too many seeds: {}, max: {}",
            seeds.len(),
            MAX_SEEDS
        )));
    }
    for seed in seeds {
        if seed.len() > MAX_SEED_LEN {
            return Err(SolanaError::InvalidPubkey(format!(
                "seed too long: {}, max: {}",
                seed.len(),
                MAX_SEED_LEN
            )));
        }
    }

    // Try each bump seed until we find a valid PDA
    let mut bump = 255;
    loop {
        let mut hasher = Sha256::new();

        // Hash all seeds
        for seed in seeds {
            hasher.update(seed);
        }

        // Add bump seed
        hasher.update([bump]);

        // Add program ID
        hasher.update(program_id.as_bytes());

        // Add "ProgramDerivedAddress" as a domain separator
        hasher.update(b"ProgramDerivedAddress");

        // Get the hash result
        let hash = hasher.finalize();

        // Convert hash to pubkey
        let mut pubkey_bytes = [0u8; 32];
        pubkey_bytes.copy_from_slice(&hash[..32]);

        // Check if it's on curve
        if !is_on_curve(&pubkey_bytes) {
            // Found a valid PDA
            return Ok((Pubkey::new(pubkey_bytes), bump));
        }

        if bump == 0 {
            return Err(SolanaError::InvalidPubkey(
                "unable to find valid PDA, all bump seeds exhausted".to_string(),
            ));
        }
        bump -= 1;
    }
}

/// Create a program address from seeds and a bump seed
pub fn create_program_address(
    program_id: &Pubkey,
    seeds: &[&[u8]],
    bump_seed: u8,
) -> Result<Pubkey> {
    // Validate seeds
    if seeds.len() > MAX_SEEDS {
        return Err(SolanaError::InvalidPubkey(format!(
            "too many seeds: {}, max: {}",
            seeds.len(),
            MAX_SEEDS
        )));
    }
    for seed in seeds {
        if seed.len() > MAX_SEED_LEN {
            return Err(SolanaError::InvalidPubkey(format!(
                "seed too long: {}, max: {}",
                seed.len(),
                MAX_SEED_LEN
            )));
        }
    }

    let mut hasher = Sha256::new();

    // Hash all seeds
    for seed in seeds {
        hasher.update(seed);
    }

    // Add bump seed
    hasher.update([bump_seed]);

    // Add program ID
    hasher.update(program_id.as_bytes());

    // Add "ProgramDerivedAddress" as a domain separator
    hasher.update(b"ProgramDerivedAddress");

    // Get the hash result
    let hash = hasher.finalize();

    // Convert hash to pubkey
    let mut pubkey_bytes = [0u8; 32];
    pubkey_bytes.copy_from_slice(&hash[..32]);

    // Check if it's on curve
    if is_on_curve(&pubkey_bytes) {
        return Err(SolanaError::InvalidPubkey(
            "resulting address is on curve (invalid PDA)".to_string(),
        ));
    }

    Ok(Pubkey::new(pubkey_bytes))
}

/// Check if a public key is on the ed25519 curve
pub fn is_on_curve(bytes: &[u8; 32]) -> bool {
    // Check if the point is all zeros
    if bytes.iter().all(|&b| b == 0) {
        return false;
    }
    // Try to decompress the point
    VerifyingKey::from_bytes(bytes).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn create_test_program_id() -> Pubkey {
        crate::instructions::program_ids::system_program()
    }

    #[test]
    fn test_find_program_address() {
        let program_id = create_test_program_id();
        let seed = b"test_seed";
        let seeds = [seed.as_ref()];

        let (pda, bump) = find_program_address(&program_id, &seeds).unwrap();

        // Verify the PDA is off curve
        assert!(!is_on_curve(pda.as_bytes()));

        // Verify we can recreate the PDA with the bump
        let recreated_pda = create_program_address(&program_id, &seeds, bump).unwrap();
        assert_eq!(pda, recreated_pda);
    }

    #[test]
    fn test_find_program_address_multiple_seeds() {
        let program_id = create_test_program_id();
        let seed1 = b"seed1";
        let seed2 = b"seed2";
        let seed3 = b"seed3";
        let seeds = [seed1.as_ref(), seed2.as_ref(), seed3.as_ref()];

        let (pda, bump) = find_program_address(&program_id, &seeds).unwrap();

        // Verify the PDA is off curve
        assert!(!is_on_curve(pda.as_bytes()));

        // Verify we can recreate the PDA with the bump
        let recreated_pda = create_program_address(&program_id, &seeds, bump).unwrap();
        assert_eq!(pda, recreated_pda);
    }

    #[test]
    fn test_find_program_address_too_many_seeds() {
        let program_id = create_test_program_id();
        let seed_strings: Vec<String> = (0..MAX_SEEDS + 1).map(|i| format!("seed{i}")).collect();
        let seed_refs: Vec<&[u8]> = seed_strings.iter().map(|s| s.as_bytes()).collect();

        let result = find_program_address(&program_id, &seed_refs);
        assert!(matches!(result, Err(SolanaError::InvalidPubkey(_))));
    }

    #[test]
    fn test_find_program_address_seed_too_long() {
        let program_id = create_test_program_id();
        let seed = [0u8; MAX_SEED_LEN + 1];
        let seeds = [&seed[..]];

        let result = find_program_address(&program_id, &seeds);
        assert!(matches!(result, Err(SolanaError::InvalidPubkey(_))));
    }

    #[test]
    fn test_create_program_address() {
        let program_id = create_test_program_id();
        let seed = b"test_seed";
        let seeds = [seed.as_ref()];
        let bump = 255;

        let pda = create_program_address(&program_id, &seeds, bump).unwrap();

        // Verify the PDA is off curve
        assert!(!is_on_curve(pda.as_bytes()));
    }

    #[test]
    fn test_create_program_address_on_curve() {
        let program_id = create_test_program_id();
        // Try different seeds and bumps to verify we never get a point on the curve
        for i in 0..10 {
            let seed = format!("test_seed_{i}");
            let seeds = [seed.as_bytes()];
            for bump in 0..10 {
                if let Ok(pubkey) = create_program_address(&program_id, &seeds, bump) {
                    assert!(
                        !is_on_curve(pubkey.as_bytes()),
                        "Found point on curve with seed {i} and bump {bump}"
                    );
                }
            }
        }
    }

    #[test]
    fn test_is_on_curve() {
        // Test a valid ed25519 public key (base point)
        let valid_key = [
            0x58, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66,
            0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66, 0x66,
            0x66, 0x66, 0x66, 0x66,
        ];
        assert!(is_on_curve(&valid_key));

        // Test an invalid public key (all zeros)
        let invalid_key = [0u8; 32];
        assert!(!is_on_curve(&invalid_key));
    }

    #[test]
    fn test_pda_deterministic() {
        let program_id = create_test_program_id();
        let seed = b"test_seed";
        let seeds = [seed.as_ref()];

        // Generate PDA twice with same inputs
        let (pda1, bump1) = find_program_address(&program_id, &seeds).unwrap();
        let (pda2, bump2) = find_program_address(&program_id, &seeds).unwrap();

        // Verify results are identical
        assert_eq!(pda1, pda2);
        assert_eq!(bump1, bump2);
    }

    #[test]
    fn test_pda_different_program_ids() {
        let program_id1 = create_test_program_id();
        let program_id2 = Pubkey::new([
            1, 1, 1, 1, 1, 1, 1, 1, 2, 2, 2, 2, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 3, 4, 4, 4, 4, 4,
            4, 4, 4,
        ]);
        let seed = b"test_seed";
        let seeds = [seed.as_ref()];

        let (pda1, _bump1) = find_program_address(&program_id1, &seeds).unwrap();
        let (pda2, _bump2) = find_program_address(&program_id2, &seeds).unwrap();

        // Verify PDAs are different
        assert_ne!(pda1, pda2);
        // Both PDAs should be off curve
        assert!(!is_on_curve(pda1.as_bytes()));
        assert!(!is_on_curve(pda2.as_bytes()));
    }

    #[test]
    fn test_pda_matches_js_example() {
        let program_id = crate::instructions::program_ids::system_program();
        let string = b"helloWorld";
        let seeds = [string.as_ref()];

        let (pda, bump) = find_program_address(&program_id, &seeds).unwrap();

        // Expected values from JS example:
        // PDA: 46GZzzetjCURsdFPb7rcnspbEMnCBXe9kpjrsZAkKb6X
        // Bump: 254
        let expected_pda =
            Pubkey::from_str("46GZzzetjCURsdFPb7rcnspbEMnCBXe9kpjrsZAkKb6X").unwrap();
        let expected_bump = 254;

        assert_eq!(pda, expected_pda, "PDA does not match expected value");
        assert_eq!(
            bump, expected_bump,
            "Bump seed does not match expected value"
        );

        // Verify we can recreate the PDA with the bump
        let recreated_pda = create_program_address(&program_id, &seeds, bump).unwrap();
        assert_eq!(recreated_pda, expected_pda);
    }
}
