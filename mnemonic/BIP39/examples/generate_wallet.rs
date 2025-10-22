use bip39_wallet::{prelude::*, wallet::Wallet};
use bitcoin::Network;

fn main() -> Result<()> {

    let mnemonic = Mnemonic::generate(EntropyBits::Bits128, Language::English)?;

    println!("Generated mnemonic:");
    println!("{}", mnemonic.phrase());
    println!("\nWord count: {}", mnemonic.word_count());

    let seed = mnemonic.to_seed("");
    println!("\nSeed (hex): {}", seed.to_hex());

    let wallet = Wallet::from_seed(&seed, Network::Bitcoin)?;

    println!("\nFirst 5 Bitcoin addresses:");
    let addresses = wallet.generate_addresses(5, 0)?;
    for (i, addr) in addresses.iter().enumerate() {
        println!("  Address {}: {}", i, addr);
    }

    Ok(())
}
