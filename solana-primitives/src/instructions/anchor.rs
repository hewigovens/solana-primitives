use sha2::{Digest, Sha256};

/// The first 8 bytes of `sha256("{namespace}:{name}")`.
fn namespaced_discriminator(namespace: &str, name: &str) -> [u8; 8] {
    let hash = Sha256::new()
        .chain_update(namespace)
        .chain_update(b":")
        .chain_update(name)
        .finalize();
    let mut discriminator = [0u8; 8];
    discriminator.copy_from_slice(&hash[..8]);
    discriminator
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
    use hexlit::hex;

    #[test]
    fn test_global_discriminator() {
        assert_eq!(global_discriminator("init_order"), hex!("204c290c27a284db"));
    }

    #[test]
    fn test_account_discriminator() {
        assert_eq!(account_discriminator("Order"), hex!("86addfb94d561c33"));
    }

    #[test]
    fn test_event_discriminator() {
        assert_eq!(event_discriminator("OrderPlaced"), hex!("6082cceaa9dbd8e3"));
    }

    #[test]
    fn test_namespaces_differ_for_same_name() {
        let name = "Foo";
        assert_ne!(global_discriminator(name), account_discriminator(name));
        assert_ne!(global_discriminator(name), event_discriminator(name));
        assert_ne!(account_discriminator(name), event_discriminator(name));
    }
}
