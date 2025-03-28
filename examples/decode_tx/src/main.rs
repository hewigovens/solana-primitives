use base64::Engine;
use solana_primitives::types::VersionedTransaction;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // https://explorer.solana.com/tx/5Nnhjv1GVB8T1k8MguUGHQw5zQQQsWET1f1zzj8azRhnVoYQPoZPtkscPCKy6FisP2eVWehjU1EYV8zywqKm5if4
    let tx = "Atrba9P4rJ4tA3fMXioF+LBR5Y397TCaCC7o/JsViIFxDQ+FOpW2/I+DGMtapWPmrRJ3KDEaYa21YbpUcXaygQPKXDfudpRNZKsMsjhhH018U2YKTAJoqu6Jr1jASfnV98/65boYyPzPujo4YMKnIaCjrt1EsvnPNCuoBMXUEzYAAgEECc20MANIMI92j1eVfOiH5WQ691HznE9ZeQfjeXpDNm0eH5z5eohWokD+6H+jjnZ/KFqkCmlEdPrk6HCx+mOgjTAJUM/3r5vR1DjJnZhT6PQK3Z32pIe8MzDmPxe8Ttzy2CTxiTfFaNQeAkRJCefcB5JJGeb/Qxrj4dpxv8Kv9gClJ544V5wdVgmhBbCFO1kSIv6OaEUizyYdqhTUiO8w8XsGp9UXGSxWjuCKhF9z0peIzwNcMUWyGrNE2AYuqUAAAM4BDmCv7bInF71jGS9UFFo/llozu4LSxwKess4eIIJkAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAG3fbh12Whk9nL4UbO63msHLSF7V9bN5E6jPWFfv8AqYUpqsC9KfFD7lsris1C7YZkNRdSH5qix9nMo2igoP0yAgcDAgUBBAQAAAAIBAMGBAAKDJSDxgMPAAAABg==";

    // Decode base64 transaction
    let tx_bytes = base64::engine::general_purpose::STANDARD.decode(tx)?;

    // Print the transaction bytes structure
    println!("Transaction bytes length: {}", tx_bytes.len());
    println!("First byte (signature count): 0x{:02x}", tx_bytes[0]);

    println!("\n==== Method 1: Direct deserialization from Solana wire format ====");

    // Try to decode as versioned transaction using manual deserialization
    match VersionedTransaction::deserialize_with_version(&tx_bytes) {
        Ok(decoded_tx) => {
            println!("✅ Successfully decoded transaction from wire format");
            print_versioned_transaction(&decoded_tx);
        }
        Err(e) => {
            println!("❌ Failed to decode transaction: {}", e);
        }
    }

    println!("\n==== Method 2: Bincode serialization and deserialization ====");

    // Convert wire format to bincode format
    let bincode_bytes = VersionedTransaction::to_bincode_format(&tx_bytes)?;

    println!("Wire format size: {} bytes", tx_bytes.len());
    println!("Bincode format size: {} bytes", bincode_bytes.len());

    // Deserialize from bincode
    match VersionedTransaction::deserialize_bincode(&bincode_bytes) {
        Ok(_bincode_decoded) => {
            println!("✅ Successfully decoded transaction using bincode");
            print_versioned_transaction(&_bincode_decoded);
        }
        Err(e) => {
            println!("❌ Failed to decode with bincode: {}", e);
        }
    }

    Ok(())
}

fn print_versioned_transaction(tx: &VersionedTransaction) {
    match tx {
        VersionedTransaction::Legacy {
            signatures,
            message,
        } => {
            println!("Transaction Type: Legacy");
            println!("Number of signatures: {}", signatures.len());
            for (i, sig) in signatures.iter().enumerate() {
                println!("Signature {}: {}", i + 1, sig.to_base58());
            }

            println!("\nMessage header:");
            println!(
                "  Required signatures: {}",
                message.header.num_required_signatures
            );
            println!(
                "  Readonly signed accounts: {}",
                message.header.num_readonly_signed_accounts
            );
            println!(
                "  Readonly unsigned accounts: {}",
                message.header.num_readonly_unsigned_accounts
            );

            println!("\nAccount keys: {}", message.account_keys.len());
            for (i, key) in message.account_keys.iter().enumerate() {
                println!("  Account {}: {}", i, key.to_base58());
            }

            println!(
                "\nRecent blockhash: {}",
                bs58::encode(&message.recent_blockhash).into_string()
            );

            println!("\nInstructions: {}", message.instructions.len());
            for (i, instruction) in message.instructions.iter().enumerate() {
                println!("\nInstruction {}:", i + 1);
                println!("  Program ID Index: {}", instruction.program_id_index);
                println!("  Account Indices: {:?}", instruction.accounts);
                println!("  Data (hex): {}", hex::encode(&instruction.data));
            }
        }
        VersionedTransaction::V0 {
            signatures,
            message,
        } => {
            println!("Transaction Type: V0");
            println!("Number of signatures: {}", signatures.len());
            for (i, sig) in signatures.iter().enumerate() {
                println!("Signature {}: {}", i + 1, sig.to_base58());
            }

            println!("\nMessage header:");
            println!(
                "  Required signatures: {}",
                message.header.num_required_signatures
            );
            println!(
                "  Readonly signed accounts: {}",
                message.header.num_readonly_signed_accounts
            );
            println!(
                "  Readonly unsigned accounts: {}",
                message.header.num_readonly_unsigned_accounts
            );

            println!("\nAccount keys: {}", message.account_keys.len());
            for (i, key) in message.account_keys.iter().enumerate() {
                println!("  Account {}: {}", i, key.to_base58());
            }

            println!(
                "\nRecent blockhash: {}",
                bs58::encode(&message.recent_blockhash).into_string()
            );

            println!("\nInstructions: {}", message.instructions.len());
            for (i, instruction) in message.instructions.iter().enumerate() {
                println!("\nInstruction {}:", i + 1);
                println!("  Program ID Index: {}", instruction.program_id_index);
                println!("  Account Indices: {:?}", instruction.accounts);
                println!("  Data (hex): {}", hex::encode(&instruction.data));
            }

            if !message.address_table_lookups.is_empty() {
                println!(
                    "\nAddress Table Lookups: {}",
                    message.address_table_lookups.len()
                );
                for (i, lookup) in message.address_table_lookups.iter().enumerate() {
                    println!("\nLookup Table {}:", i + 1);
                    println!("  Table Key: {}", lookup.key.to_base58());
                    println!("  Addresses: {}", lookup.addresses.len());
                    for (j, addr) in lookup.addresses.iter().enumerate() {
                        println!("    Address {}: {}", j, addr.to_base58());
                    }
                }
            }
        }
    }
}
