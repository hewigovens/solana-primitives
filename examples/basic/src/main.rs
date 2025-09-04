use base64::Engine;
use solana_primitives::{
    get_public_key, instructions::system::transfer, Pubkey, TransactionBuilder,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Solana Primitives - Transaction Signing Example");
    println!("═══════════════════════════════════════════════════");

    // Generate a test private key from base58 string
    // ⚠️  WARNING: This is for demonstration only!
    // In production, use proper key generation and secure storage
    let private_key_base58 = "3KSeAx7jkVrjJ2PXjhzVMnJfV3zsyT4ADWtpnHxks5eD";
    let private_key = bs58::decode(private_key_base58).into_vec()?;

    // Generate corresponding public key
    let fee_payer_pubkey_bytes = get_public_key(&private_key)?;
    let fee_payer = Pubkey::new(fee_payer_pubkey_bytes);

    let recipient = Pubkey::from_base58("4fYNw3dojWmQ4dXtSGE9epjRGy9uFrCRgbvGgQBNZCQF")?;

    // Use a dummy blockhash for this example (in production, get from RPC)
    let recent_blockhash = [1u8; 32]; // Use non-zero for visual distinction

    println!("\n📝 Building Transaction:");
    println!("   - Fee payer: {}", fee_payer.to_base58());
    println!("   - Recipient: {}", recipient.to_base58());

    // Create a simple SOL transfer instruction
    let transfer_instruction = transfer(&fee_payer, &recipient, 1_000_000); // 0.001 SOL

    // Build transaction
    let mut tx_builder = TransactionBuilder::new(fee_payer, recent_blockhash);
    tx_builder.add_instruction(transfer_instruction);

    let mut transaction = tx_builder.build()?;

    println!("✅ Unsigned transaction created:");
    println!(
        "   - Instructions: {}",
        transaction.message.instructions.len()
    );
    println!(
        "   - Account keys: {}",
        transaction.message.account_keys.len()
    );
    println!(
        "   - Required signatures: {}",
        transaction.message.header.num_required_signatures
    );
    println!("   - Is signed: {}", transaction.is_signed());

    // Validate transaction size
    transaction.validate_size()?;
    println!("✅ Transaction size validation passed");

    println!("\n🔐 Signing Transaction:");

    // Method 1: Sign the entire transaction at once
    let private_keys = [private_key.as_ref()]; // References to private keys
    transaction.sign(&private_keys)?;

    println!("✅ Transaction signed successfully!");
    println!("   - Is signed: {}", transaction.is_signed());
    println!("   - Signature count: {}", transaction.signatures.len());

    // Display the signature (first 16 bytes for brevity)
    if let Some(first_signature) = transaction.signatures.first() {
        let sig_bytes = first_signature.as_bytes();
        println!("   - First signature: {}...", hex::encode(&sig_bytes[..16]));
    }

    println!("\n📦 Final Transaction:");
    // Serialize the complete signed transaction
    let transaction_bytes = borsh::to_vec(&transaction)?;
    println!("   - Serialized size: {} bytes", transaction_bytes.len());
    println!(
        "   - Base64 encoded: {}",
        base64::prelude::BASE64_STANDARD.encode(&transaction_bytes)
    );

    // Demonstrate partial signing (useful for multi-sig scenarios)
    println!("\n🔄 Demonstrating Partial Signing:");

    // Create a new transaction builder for partial signing demo
    let mut partial_tx_builder = TransactionBuilder::new(fee_payer, recent_blockhash);
    partial_tx_builder.add_instruction(transfer(&fee_payer, &recipient, 1_000_000));
    let mut partial_tx = partial_tx_builder.build()?;
    println!(
        "   - Before partial sign - Is signed: {}",
        partial_tx.is_signed()
    );

    // Partial sign with specific keys and their corresponding public keys
    let private_keys_for_partial = [private_key.as_ref()];
    let public_keys_for_partial = [fee_payer];
    partial_tx.partial_sign(&private_keys_for_partial, &public_keys_for_partial)?;

    println!("✅ Partial signing completed!");
    println!(
        "   - After partial sign - Is signed: {}",
        partial_tx.is_signed()
    );

    println!("\n🎯 Summary:");
    println!("   ✓ Transaction built using InstructionBuilder");
    println!("   ✓ Transaction validated for size limits");
    println!("   ✓ Transaction signed with private key");
    println!("   ✓ Partial signing demonstrated");
    println!("   ✓ Transaction serialized for network submission");

    println!("\n📡 Next Steps:");
    println!("   - Submit transaction_bytes to Solana RPC endpoint");
    println!("   - Monitor transaction status on-chain");
    println!("   - Handle success/failure responses");

    Ok(())
}
