use sha2::{Digest, Sha256};

fn namespaced_discriminator(namespace: &str, name: &str) -> [u8; 8] {
    let mut hasher = Sha256::new();
    hasher.update(format!("{namespace}:{name}").as_bytes());
    let hash = hasher.finalize();
    let mut data = [0u8; 8];
    data.copy_from_slice(&hash[..8]);
    data
}

/// Return the 8-byte Anchor instruction discriminator for a global instruction name.
///
/// This is the prefix Anchor prepends to instruction data for entrypoints declared
/// inside `#[program]`.
pub fn global_discriminator(name: &str) -> [u8; 8] {
    namespaced_discriminator("global", name)
}

/// Return the 8-byte Anchor account discriminator for an account type name.
///
/// This is the prefix Anchor writes at the start of every account's data for types
/// declared with `#[account]`. Use it to identify or verify account types when parsing.
pub fn account_discriminator(name: &str) -> [u8; 8] {
    namespaced_discriminator("account", name)
}

/// Return the 8-byte Anchor event discriminator for an event type name.
///
/// This is the prefix Anchor writes at the start of an event payload emitted via
/// `emit!` for types declared with `#[event]`. Use it to identify events in program logs.
pub fn event_discriminator(name: &str) -> [u8; 8] {
    namespaced_discriminator("event", name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_discriminator() {
        assert_eq!(
            global_discriminator("init_order"),
            [0x20, 0x4c, 0x29, 0x0c, 0x27, 0xa2, 0x84, 0xdb]
        );
    }

    #[test]
    fn test_account_discriminator() {
        assert_eq!(
            account_discriminator("Order"),
            [0x86, 0xad, 0xdf, 0xb9, 0x4d, 0x56, 0x1c, 0x33]
        );
    }

    #[test]
    fn test_event_discriminator() {
        assert_eq!(
            event_discriminator("OrderPlaced"),
            [0x60, 0x82, 0xcc, 0xea, 0xa9, 0xdb, 0xd8, 0xe3]
        );
    }

    #[test]
    fn test_namespaces_differ_for_same_name() {
        let name = "Foo";
        assert_ne!(global_discriminator(name), account_discriminator(name));
        assert_ne!(global_discriminator(name), event_discriminator(name));
        assert_ne!(account_discriminator(name), event_discriminator(name));
    }
}
