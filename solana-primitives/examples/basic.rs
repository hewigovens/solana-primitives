use base64::Engine;
use solana_primitives::{
    Pubkey, TransactionBuilder, get_public_key, instructions::system::transfer,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Solana Primitives - Transaction Signing Example");

    // Demonstration key only; never hard-code real keys.
    let private_key = bs58::decode("3KSeAx7jkVrjJ2PXjhzVMnJfV3zsyT4ADWtpnHxks5eD").into_vec()?;
    let fee_payer = Pubkey::new(get_public_key(&private_key)?);
    let recipient = Pubkey::from_base58("4fYNw3dojWmQ4dXtSGE9epjRGy9uFrCRgbvGgQBNZCQF")?;
    // Fetch a real blockhash from RPC (`getLatestBlockhash`) in production.
    let recent_blockhash = [1u8; 32];

    println!("\nBuilding transaction:");
    println!("  fee payer: {fee_payer}");
    println!("  recipient: {recipient}");

    let mut tx_builder = TransactionBuilder::new(fee_payer, recent_blockhash);
    tx_builder.add_instruction(transfer(&fee_payer, &recipient, 1_000_000));
    let mut transaction = tx_builder.build()?;

    println!("  instructions: {}", transaction.message.instructions.len());
    println!("  account keys: {}", transaction.message.account_keys.len());
    println!(
        "  required signatures: {}",
        transaction.message.header.num_required_signatures
    );
    println!("  signed: {}", transaction.is_signed());
    transaction.validate_size()?;

    println!("\nSigning:");
    transaction.sign(&[&private_key])?;
    transaction.verify()?;
    println!("  signed: {}", transaction.is_signed());
    let signature_prefix: String = transaction.signatures[0].as_bytes()[..8]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    println!("  first signature: {signature_prefix}...");

    // These are the bytes to submit with `sendTransaction` (base64 encoding).
    let wire_bytes = transaction.serialize()?;
    println!("\nWire transaction:");
    println!("  size: {} bytes", wire_bytes.len());
    println!(
        "  base64: {}",
        base64::prelude::BASE64_STANDARD.encode(&wire_bytes)
    );

    // Partial signing lets several parties add their signatures independently.
    println!("\nPartial signing:");
    let mut partial_builder = TransactionBuilder::new(fee_payer, recent_blockhash);
    partial_builder.add_instruction(transfer(&fee_payer, &recipient, 1_000_000));
    let mut partial_tx = partial_builder.build()?;
    println!("  before: signed = {}", partial_tx.is_signed());
    partial_tx.partial_sign(&[&private_key])?;
    println!("  after: signed = {}", partial_tx.is_signed());

    Ok(())
}
