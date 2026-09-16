use crate::error::{Result, SolanaError};
use crate::types::Pubkey;
use curve25519_dalek::edwards::CompressedEdwardsY;
use sha2::{Digest, Sha256};

/// Maximum number of seeds allowed in a PDA, including the bump seed.
pub const MAX_SEEDS: usize = 16;
/// Maximum length of a seed in bytes
pub const MAX_SEED_LEN: usize = 32;
/// Domain separator appended to PDA preimages.
const PDA_MARKER: &[u8; 21] = b"ProgramDerivedAddress";

/// Find a program address and bump seed for the given seeds.
///
/// Tries bump seeds from 255 down to 1 and returns the first address that is
/// off the Ed25519 curve, matching `Pubkey::find_program_address`. Bump 0 is
/// never searched, so a program that wants it must call
/// [`create_program_address`] directly.
pub fn find_program_address(program_id: &Pubkey, seeds: &[&[u8]]) -> Result<(Pubkey, u8)> {
    validate_seeds(seeds)?;
    for bump in (1..=u8::MAX).rev() {
        if let Some(address) = derive_program_address(program_id, seeds, bump) {
            return Ok((address, bump));
        }
    }
    Err(SolanaError::NoViableBumpSeed)
}

/// Create a program address from seeds and a bump seed.
///
/// Fails if the derived address is on the Ed25519 curve.
pub fn create_program_address(
    program_id: &Pubkey,
    seeds: &[&[u8]],
    bump_seed: u8,
) -> Result<Pubkey> {
    validate_seeds(seeds)?;
    derive_program_address(program_id, seeds, bump_seed).ok_or(SolanaError::InvalidSeeds)
}

/// Derive an account address from a base pubkey, a seed string, and an owner,
/// matching `Pubkey::create_with_seed`.
pub fn create_with_seed(base: &Pubkey, seed: &str, owner: &Pubkey) -> Result<Pubkey> {
    if seed.len() > MAX_SEED_LEN {
        return Err(SolanaError::InvalidSeeds);
    }
    if owner.as_bytes().ends_with(PDA_MARKER) {
        return Err(SolanaError::IllegalOwner);
    }
    let hash = Sha256::new()
        .chain_update(base.as_bytes())
        .chain_update(seed.as_bytes())
        .chain_update(owner.as_bytes())
        .finalize();
    Ok(Pubkey::new(hash.into()))
}

/// Seeds exclude the bump, which takes one of the [`MAX_SEEDS`] slots.
fn validate_seeds(seeds: &[&[u8]]) -> Result<()> {
    if seeds.len() >= MAX_SEEDS || seeds.iter().any(|seed| seed.len() > MAX_SEED_LEN) {
        return Err(SolanaError::InvalidSeeds);
    }
    Ok(())
}

/// `sha256(seeds || bump || program_id || PDA_MARKER)`, or `None` if it is on the curve.
fn derive_program_address(program_id: &Pubkey, seeds: &[&[u8]], bump: u8) -> Option<Pubkey> {
    let mut hasher = Sha256::new();
    for seed in seeds {
        hasher.update(seed);
    }
    let hash: [u8; 32] = hasher
        .chain_update([bump])
        .chain_update(program_id.as_bytes())
        .chain_update(PDA_MARKER)
        .finalize()
        .into();
    (!is_on_curve(&hash)).then(|| Pubkey::new(hash))
}

/// Whether `bytes` decompress to an Ed25519 point, matching Solana's `bytes_are_curve_point`.
pub(crate) fn is_on_curve(bytes: &[u8; 32]) -> bool {
    CompressedEdwardsY(*bytes).decompress().is_some()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::program_ids::system_program;
    use crate::test_utils::{key, pubkey};
    use hexlit::hex;

    // Vectors from `solana-address` 2.7 (`curve25519` feature). `key(..)` is `sha256(label)`.
    const PDA_PROGRAM: &str = "HimW7bKhxSK4Ti3zkGZC94CLoRYrAb9rkgneaETAqFHW";

    fn find(seeds: &[&[u8]]) -> (String, u8) {
        let (address, bump) = find_program_address(&key("pda_program"), seeds).unwrap();
        (address.to_base58(), bump)
    }

    #[test]
    fn find_program_address_matches_upstream() {
        assert_eq!(key("pda_program").to_base58(), PDA_PROGRAM);
        let long = [7u8; 32];
        let fifteen: Vec<[u8; 1]> = (0..15u8).map(|i| [i]).collect();
        let fifteen: Vec<&[u8]> = fifteen.iter().map(|s| &s[..]).collect();

        let cases: [(&[&[u8]], &str, u8); 5] = [
            (&[], "AGmwQqVG4pDGQR4rkuZu3xeTGN9fFtTp9C95LbUZwnxa", 255),
            (&[b"a"], "FNGma17diVJNi2hk8rqL9EXZfubEzLze3EMmRsRd2qZz", 254),
            (&[&long], "7zfPGCTgMLLm8rvX5ciZtMo24ADErj5byx5yy4WG74y", 255),
            (
                &[b"vault", &long, b""],
                "CbxsgYHyHP4pnjK53oVxah6byhkyhE78z5zs2oGeP9o7",
                255,
            ),
            (
                &fifteen,
                "C28icY5egSkHEvhwZpietpuMax1wnZKSfPmeK1Ruzka1",
                255,
            ),
        ];
        for (seeds, address, bump) in cases {
            assert_eq!(find(seeds), (address.to_string(), bump));
            assert_eq!(
                create_program_address(&key("pda_program"), seeds, bump).unwrap(),
                pubkey(address)
            );
        }

        let (address, bump) = find_program_address(&system_program(), &[b"helloWorld"]).unwrap();
        assert_eq!(
            (address, bump),
            (pubkey("46GZzzetjCURsdFPb7rcnspbEMnCBXe9kpjrsZAkKb6X"), 254)
        );
    }

    #[test]
    fn create_program_address_matches_solana_sdk_tests() {
        // Vectors from `solana-address`'s own `test_create_program_address`, where the
        // last seed plays the role of the bump.
        let program_id = pubkey("BPFLoaderUpgradeab1e11111111111111111111111");
        let seed_pubkey = pubkey("SeedPubey1111111111111111111111111111111111");
        let cases: [(&[&[u8]], u8, &str); 4] = [
            (&[b""], 1, "BwqrghZA2htAcqq8dzP1WDAhTXYTYWj7CHxF5j7TDBAe"),
            (
                &["\u{2609}".as_bytes()],
                0,
                "13yWmRpaTR4r5nAktwLqMpRNr28tnVUZw26rTvPSSB19",
            ),
            // Seeds are hashed without separators: ["Talking", "Squirrels"].
            (
                &[b"Talking", b"Squirrel"],
                b's',
                "2fnQrngrQT4SeLcdToJAD96phoEjNL2man2kfRLCASVk",
            ),
            (
                &[seed_pubkey.as_bytes()],
                1,
                "976ymqVnfE32QFe6NfGDctSvVa36LWnvYxhU6G2232YL",
            ),
        ];
        for (seeds, bump, expected) in cases {
            assert_eq!(
                create_program_address(&program_id, seeds, bump),
                Ok(pubkey(expected))
            );
        }
    }

    #[test]
    fn create_program_address_rejects_on_curve_result() {
        // Upstream `create_program_address([00000000, ff], program)` returns `InvalidSeeds`.
        let seed = [0u8; 4];
        assert_eq!(
            create_program_address(&key("pda_program"), &[&seed], 255),
            Err(SolanaError::InvalidSeeds)
        );
        let (_, bump) = find_program_address(&key("pda_program"), &[&seed]).unwrap();
        assert!(bump < 255);
    }

    #[test]
    fn seed_limits() {
        let program_id = key("pda_program");
        let seeds: Vec<[u8; 1]> = (0..MAX_SEEDS as u8).map(|i| [i]).collect();
        let seeds: Vec<&[u8]> = seeds.iter().map(|s| &s[..]).collect();

        // The bump occupies the last slot, so callers get MAX_SEEDS - 1.
        let invalid = SolanaError::InvalidSeeds;
        assert_eq!(
            find_program_address(&program_id, &seeds),
            Err(invalid.clone())
        );
        assert_eq!(
            create_program_address(&program_id, &seeds, 0),
            Err(invalid.clone())
        );
        assert!(find_program_address(&program_id, &seeds[1..]).is_ok());

        let too_long = [0u8; MAX_SEED_LEN + 1];
        assert_eq!(
            find_program_address(&program_id, &[&too_long]),
            Err(invalid.clone())
        );
        assert_eq!(
            create_program_address(&program_id, &[&too_long], 0),
            Err(invalid)
        );
    }

    #[test]
    fn is_on_curve_matches_upstream_bytes_are_curve_point() {
        let cases: [([u8; 32], bool); 15] = [
            // All-zero bytes decompress (y = 0) and are therefore on the curve.
            ([0; 32], true),
            ([1; 32], true),
            ([0xff; 32], true),
            (
                hex!("5866666666666666666666666666666666666666666666666666666666666666"),
                true,
            ),
            // Non-canonical y encodings are reduced mod p before decompression.
            (
                hex!("edffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7f"),
                true,
            ),
            (
                hex!("eeffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff7f"),
                true,
            ),
            (
                hex!("0000000000000000000000000000000000000000000000000000000000000080"),
                true,
            ),
            (
                hex!("df299516b500f8bd486b00b46686c8c5229a94539ffff63664c0890934d1ef4f"),
                false,
            ),
            (
                hex!("1676e820cd6100968f689719f2af91083c36612aa3ad411cbfbdc30dc489ac2f"),
                false,
            ),
            (
                hex!("7b90b08d7676d79df0ce8c4768991529b25d908dde3b6b99f2ef900163e935a8"),
                false,
            ),
            (
                hex!("e2e7256451e07e1205f11ca96ef1c61b56e9c9eaa5ecb24d98c44bff08b5b5a5"),
                true,
            ),
            (
                hex!("f6866ca10994a5c324e8cdaf99a389f4a83883022861c615063248822d7ea7e8"),
                true,
            ),
            (
                hex!("4bf7fe7a76a5a9a514bffb0f92eeb3f2ddd8bd19f7ad4a3764b2c97f6ea5d4a4"),
                true,
            ),
            (
                hex!("785e43a85294d8ce1d8c1ffc7434c24e0d02613b7371030e4065b2b3ae481b61"),
                false,
            ),
            (
                hex!("c3b721b14c56a235bde44cbf1cac02cd3983c911538af7a3523bab1d3aa33fe7"),
                false,
            ),
        ];
        for (bytes, expected) in cases {
            assert_eq!(is_on_curve(&bytes), expected, "{bytes:02x?}");
        }
    }

    #[test]
    fn create_with_seed_rules() {
        let (base, owner) = (key("base"), key("owner"));
        assert_eq!(
            create_with_seed(&base, "seed", &owner).unwrap(),
            pubkey("3CzqgepHiVmdbc3owKnm2SiTGX4wpiawznXCN3jsP2jp")
        );

        assert_eq!(
            create_with_seed(&base, &"x".repeat(MAX_SEED_LEN + 1), &owner),
            Err(SolanaError::InvalidSeeds)
        );

        let mut marked = [0u8; 32];
        marked[32 - PDA_MARKER.len()..].copy_from_slice(PDA_MARKER);
        assert_eq!(
            create_with_seed(&base, "seed", &Pubkey::new(marked)),
            Err(SolanaError::IllegalOwner)
        );
    }
}
