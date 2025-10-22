use bip39_wallet::{prelude::*, wallet::Wallet};
use bitcoin::Network;

fn main() -> Result<()> {

    let mnemonic_phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

    println!("Recovering wallet from mnemonic:");
    println!("{}", mnemonic_phrase);

    let mnemonic = Mnemonic::from_phrase(mnemonic_phrase, Language::English)?;

    println!("\n Mnemonic is valid!");
    println!("Word count: {}", mnemonic.word_count());
    println!("Entropy: {} bits", mnemonic.entropy().bits() as usize);

    let seed = mnemonic.to_seed("");
    println!("\nSeed (hex): {}...", &seed.to_hex()[..32]);

    let wallet = Wallet::from_seed(&seed, Network::Bitcoin)?;

    let path = HDPath::bitcoin();
    let address = wallet.get_address(&path)?;

    println!("\nDefault address (m/44'/0'/0'/0/0):");
    println!("  {}", address);

    Ok(())
}
