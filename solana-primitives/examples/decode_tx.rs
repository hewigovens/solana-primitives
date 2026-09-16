use base64::Engine;
use solana_primitives::VersionedTransaction;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // https://solscan.io/tx/24iGSgbnUUpL49Hp6ci46CrtujrMbDgZxJYcL6w3ySJudT3c5swGKnNygv4R1G3vYPwPYS9Emr3fimeEJQszDkzV
    let tx = "ATU3t4TX00FNb4aqeojTYpIOZFQFED4pdoviPCBxUsEXwKHRbHBmmgsyEsDyjXREHGYAWlq13q4WzF5JDOvsBwKAAQAICwzd9xRZ1DCfrYBfTdgj38msCBn1maih1JKu7vCovzzacU2yFtM8/6xky5bNOdOL9vYNQWxOAtjDd8qJeKlPPJIXZyoBm/f1Y2xMsGa3gRLrL4a5Ba4vtFtLcOU1pWTT4wVKU1qZKSEGTSTocWDaOHx8NbXdvJK7geQfqEBBBUSNAwZGb+UhFzL/7K26csOb57yM5bvF9xJrLEObOkAAAAAGxoMMVxwvPwb8Y6KPKojQO9MxPeq3HTSRmeyRYk5jPwlU276eyWDJinopP+ITNpZv4YDRUa5LgXlWH4mFSlP2fhcCyQN7oqc4HZ+5zm3xXQ/FTXS7cxGfivEvGQwOFb4pA2InHkw7ace1KQX3P6I/rhhBsIayFI80eInx/pUv4R7U/fA1on6uX4cAPWh+6q5kflQbDzfTC/LJrf1AdS22d3DH2Q4dNNF2yrr6HjJmXJlYvanqTxxULDNngUZsJSq7g2zxeB7onMepv2C8TaiFnhOPYyeceZ5GEIyJx6r0FAYDADExNzQ2NzAxMDg2MTA0fDMzODYzMjQ4M3wxNzQ2NzAxMDg2MTEzfDMzODYzMjQ4M3xwBAAFApZUAgAEAAkDUMMAAAAAAAAFAgEAGFj16I5t+vX/F0jWAAAAAAAjixxoAAAAAAYMCwIABwgJCgwNDg8QDu7hX57jZwjCAQEBAQAABgwLAgAHCAkKDA0ODxBcPD8yewzFPL4CAAAAAQEAAACE1xcAAAAABAExPRcAAAABAAADAAEtixxoAAAAAAAAAAAAAAEBAQA0kqIPAAAAAPJZLkAXAAAAAQAAAwABLYscaAAAAAAAAAAAAAAB3E42Kl7pkP+vCgX6bK+0TFMtWHz2EUnYgJSxOMI1cDcABoNbXAMFAQ==";
    let tx_bytes = base64::engine::general_purpose::STANDARD.decode(tx)?;
    println!("Transaction bytes: {}", tx_bytes.len());

    let tx = VersionedTransaction::deserialize(&tx_bytes)?;
    print_transaction(&tx);
    Ok(())
}

fn print_transaction(tx: &VersionedTransaction) {
    println!("Version: {:?}", tx.version());
    println!("Signatures: {}", tx.signatures.len());
    for (i, signature) in tx.signatures.iter().enumerate() {
        println!("  {i}: {signature}");
    }

    let header = tx.header();
    println!("\nHeader:");
    println!("  required signatures: {}", header.num_required_signatures);
    println!(
        "  readonly signed accounts: {}",
        header.num_readonly_signed_accounts
    );
    println!(
        "  readonly unsigned accounts: {}",
        header.num_readonly_unsigned_accounts
    );

    println!("\nAccount keys: {}", tx.account_keys().len());
    for (i, key) in tx.account_keys().iter().enumerate() {
        println!("  {i}: {key}");
    }
    println!(
        "\nRecent blockhash: {}",
        bs58::encode(tx.recent_blockhash()).into_string()
    );

    println!("\nInstructions: {}", tx.instructions().len());
    for (i, instruction) in tx.instructions().iter().enumerate() {
        println!("  {i}: program index {}", instruction.program_id_index);
        println!("     account indexes {:?}", instruction.accounts);
        println!(
            "     data (base58) {}",
            bs58::encode(&instruction.data).into_string()
        );
    }

    if let Some(lookups) = tx.address_table_lookups() {
        println!("\nAddress table lookups: {}", lookups.len());
        for lookup in lookups {
            println!("  table {}", lookup.account_key);
            println!("     writable indexes {:?}", lookup.writable_indexes);
            println!("     readonly indexes {:?}", lookup.readonly_indexes);
        }
    }

    if let Some(price) = tx.get_compute_unit_price() {
        println!("\nCompute unit price: {price} micro-lamports");
    }
    if let Some(limit) = tx.get_compute_unit_limit() {
        println!("Compute unit limit: {limit}");
    }
}
