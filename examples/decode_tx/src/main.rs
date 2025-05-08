use base64::Engine;
use solana_primitives::types::VersionedTransaction;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // https://solscan.io/tx/24iGSgbnUUpL49Hp6ci46CrtujrMbDgZxJYcL6w3ySJudT3c5swGKnNygv4R1G3vYPwPYS9Emr3fimeEJQszDkzV
    let tx = "ATU3t4TX00FNb4aqeojTYpIOZFQFED4pdoviPCBxUsEXwKHRbHBmmgsyEsDyjXREHGYAWlq13q4WzF5JDOvsBwKAAQAICwzd9xRZ1DCfrYBfTdgj38msCBn1maih1JKu7vCovzzacU2yFtM8/6xky5bNOdOL9vYNQWxOAtjDd8qJeKlPPJIXZyoBm/f1Y2xMsGa3gRLrL4a5Ba4vtFtLcOU1pWTT4wVKU1qZKSEGTSTocWDaOHx8NbXdvJK7geQfqEBBBUSNAwZGb+UhFzL/7K26csOb57yM5bvF9xJrLEObOkAAAAAGxoMMVxwvPwb8Y6KPKojQO9MxPeq3HTSRmeyRYk5jPwlU276eyWDJinopP+ITNpZv4YDRUa5LgXlWH4mFSlP2fhcCyQN7oqc4HZ+5zm3xXQ/FTXS7cxGfivEvGQwOFb4pA2InHkw7ace1KQX3P6I/rhhBsIayFI80eInx/pUv4R7U/fA1on6uX4cAPWh+6q5kflQbDzfTC/LJrf1AdS22d3DH2Q4dNNF2yrr6HjJmXJlYvanqTxxULDNngUZsJSq7g2zxeB7onMepv2C8TaiFnhOPYyeceZ5GEIyJx6r0FAYDADExNzQ2NzAxMDg2MTA0fDMzODYzMjQ4M3wxNzQ2NzAxMDg2MTEzfDMzODYzMjQ4M3xwBAAFApZUAgAEAAkDUMMAAAAAAAAFAgEAGFj16I5t+vX/F0jWAAAAAAAjixxoAAAAAAYMCwIABwgJCgwNDg8QDu7hX57jZwjCAQEBAQAABgwLAgAHCAkKDA0ODxBcPD8yewzFPL4CAAAAAQEAAACE1xcAAAAABAExPRcAAAABAAADAAEtixxoAAAAAAAAAAAAAAEBAQA0kqIPAAAAAPJZLkAXAAAAAQAAAwABLYscaAAAAAAAAAAAAAAB3E42Kl7pkP+vCgX6bK+0TFMtWHz2EUnYgJSxOMI1cDcABoNbXAMFAQ==";

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
                println!("  Data (bs58): {}", bs58::encode(&instruction.data).into_string());
                println!("  Data (utf8): {}", String::from_utf8_lossy(&instruction.data));
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
                println!("  Data (bs58): {}", bs58::encode(&instruction.data).into_string());
                println!("  Data (utf8): {}", String::from_utf8_lossy(&instruction.data));
            }

            if !message.address_table_lookups.is_empty() {
                println!(
                    "\nAddress Table Lookups: {}",
                    message.address_table_lookups.len()
                );
                for (i, lookup) in message.address_table_lookups.iter().enumerate() {
                    println!("\nLookup Table {}:", i + 1);
                    println!("  Table Key: {}", lookup.account_key.to_base58());
                    println!("  Writable Indexes: {:?}", lookup.writable_indexes);
                    println!("  Readonly Indexes: {:?}", lookup.readonly_indexes);
                }
            }
        }
    }
}
