use pure_bip39::{Mnemonic, Language, EntropyBits};

fn main() {
    println!("🔐 Simple BIP-39 Demo\n");

    println!("Generating 12-word mnemonic...\n");
    let mnemonic = Mnemonic::generate(EntropyBits::Bits128, Language::English)
        .expect("Failed to generate mnemonic");

    println!("📝 Mnemonic:");
    println!("{}\n", mnemonic.phrase());

    println!("🔑 Entropy (hex):");
    println!("{}\n", mnemonic.entropy().to_hex());

    println!("🌱 Seed (no passphrase):");
    let seed = mnemonic.to_seed("");
    println!("{}\n", seed.to_hex());

    println!("🔒 Seed (with passphrase 'secret'):");
    let seed_with_pass = mnemonic.to_seed("secret");
    println!("{}\n", seed_with_pass.to_hex());

    println!("✅ Done! Notice how the seeds are different!");
}
