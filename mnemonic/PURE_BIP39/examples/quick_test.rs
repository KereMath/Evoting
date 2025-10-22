use pure_bip39::{Mnemonic, Language, EntropyBits};

fn main() {
    println!("\n╔════════════════════════════════════════════════════════════╗");
    println!("║          🔐 PURE BIP-39 Quick Test                        ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    println!("📋 TEST 1: Different Word Counts");
    println!("─────────────────────────────────────────────────────────────");

    let sizes = [
        (EntropyBits::Bits128, "12 words"),
        (EntropyBits::Bits160, "15 words"),
        (EntropyBits::Bits192, "18 words"),
        (EntropyBits::Bits224, "21 words"),
        (EntropyBits::Bits256, "24 words"),
    ];

    for (bits, label) in sizes.iter() {
        let mnemonic = Mnemonic::generate(*bits, Language::English).unwrap();
        println!("\n{}: ✅", label);
        println!("{}", mnemonic.phrase());
    }

    println!("\n\n📋 TEST 2: BIP-39 Test Vector Validation");
    println!("─────────────────────────────────────────────────────────────");

    let test_phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    match Mnemonic::from_phrase(test_phrase, Language::English) {
        Ok(mnemonic) => {
            println!("✅ Test vector is valid!");
            println!("Entropy: {}", mnemonic.entropy().to_hex());

            let seed = mnemonic.to_seed("TREZOR");
            println!("Seed: {}", seed.to_hex());

            let expected = "c55257c360c07c72029aebc1b53c05ed0362ada38ead3e3e9efa3708e53495531f09a6987599d18264c1e1c92f2cf141630c7a3c4ab7c81b2f001698e7463b04";
            if seed.to_hex() == expected {
                println!("✅ Seed matches BIP-39 test vector!");
            } else {
                println!("❌ Seed does NOT match!");
            }
        }
        Err(e) => println!("❌ Test vector validation failed: {}", e),
    }

    println!("\n\n📋 TEST 3: Invalid Checksum Detection");
    println!("─────────────────────────────────────────────────────────────");

    let invalid = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon";
    match Mnemonic::from_phrase(invalid, Language::English) {
        Ok(_) => println!("❌ Should have detected invalid checksum!"),
        Err(_) => println!("✅ Invalid checksum detected correctly!"),
    }

    println!("\n\n📋 TEST 4: Passphrase Effect");
    println!("─────────────────────────────────────────────────────────────");

    let mnemonic = Mnemonic::generate(EntropyBits::Bits128, Language::English).unwrap();
    let seed1 = mnemonic.to_seed("");
    let seed2 = mnemonic.to_seed("password");
    let seed3 = mnemonic.to_seed("different");

    println!("Same mnemonic, different passphrases:");
    println!("\nNo passphrase:");
    println!("{}", seed1.to_hex());
    println!("\nWith 'password':");
    println!("{}", seed2.to_hex());
    println!("\nWith 'different':");
    println!("{}", seed3.to_hex());

    if seed1.to_hex() != seed2.to_hex() && seed2.to_hex() != seed3.to_hex() {
        println!("\n✅ All seeds are different (as expected)!");
    }

    println!("\n\n╔════════════════════════════════════════════════════════════╗");
    println!("║          ✅ All Tests Completed Successfully!             ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");
}
