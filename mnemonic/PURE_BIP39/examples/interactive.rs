use pure_bip39::{Mnemonic, Language, EntropyBits};
use std::io::{self, Write};

fn main() {
    println!("🔐 PURE BIP-39 Interactive Demo");
    println!("================================\n");

    loop {
        println!("Choose an option:");
        println!("  1. Generate new mnemonic");
        println!("  2. Validate existing mnemonic");
        println!("  3. Generate seed from mnemonic");
        println!("  4. Exit");
        print!("\nYour choice: ");
        io::stdout().flush().unwrap();

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).unwrap();

        match choice.trim() {
            "1" => generate_mnemonic(),
            "2" => validate_mnemonic(),
            "3" => generate_seed(),
            "4" => {
                println!("\n👋 Goodbye!");
                break;
            }
            _ => println!("❌ Invalid choice!\n"),
        }
    }
}

fn generate_mnemonic() {
    println!("\n--- Generate New Mnemonic ---");
    println!("Select word count:");
    println!("  1. 12 words (128 bits)");
    println!("  2. 15 words (160 bits)");
    println!("  3. 18 words (192 bits)");
    println!("  4. 21 words (224 bits)");
    println!("  5. 24 words (256 bits)");
    print!("\nYour choice: ");
    io::stdout().flush().unwrap();

    let mut choice = String::new();
    io::stdin().read_line(&mut choice).unwrap();

    let bits = match choice.trim() {
        "1" => EntropyBits::Bits128,
        "2" => EntropyBits::Bits160,
        "3" => EntropyBits::Bits192,
        "4" => EntropyBits::Bits224,
        "5" => EntropyBits::Bits256,
        _ => {
            println!("❌ Invalid choice!\n");
            return;
        }
    };

    match Mnemonic::generate(bits, Language::English) {
        Ok(mnemonic) => {
            println!("\n✅ Mnemonic Generated Successfully!\n");
            println!("📝 Your mnemonic ({} words):", mnemonic.word_count());
            println!("┌────────────────────────────────────────┐");

            let phrase = mnemonic.phrase();
            let words: Vec<&str> = phrase.split_whitespace().collect();
            for (i, word) in words.iter().enumerate() {
                print!("│ {:2}. {:<15}", i + 1, word);
                if (i + 1) % 2 == 0 {
                    println!("│");
                }
            }
            if words.len() % 2 != 0 {
                println!("                 │");
            }
            println!("└────────────────────────────────────────┘");

            println!("\n🔑 Entropy (hex):");
            println!("{}", mnemonic.entropy().to_hex());

            println!("\n⚠️  IMPORTANT: Write these words down and keep them safe!");
            println!("    Anyone with these words can access your funds.\n");
        }
        Err(e) => {
            println!("❌ Error: {}\n", e);
        }
    }
}

fn validate_mnemonic() {
    println!("\n--- Validate Mnemonic ---");
    println!("Enter your mnemonic phrase (words separated by spaces):");
    print!("> ");
    io::stdout().flush().unwrap();

    let mut phrase = String::new();
    io::stdin().read_line(&mut phrase).unwrap();
    let phrase = phrase.trim();

    match Mnemonic::from_phrase(phrase, Language::English) {
        Ok(mnemonic) => {
            println!("\n✅ Valid Mnemonic!\n");
            println!("Word count: {}", mnemonic.word_count());
            println!("Entropy (hex): {}", mnemonic.entropy().to_hex());
            println!("Language: {:?}", mnemonic.language());
            println!();
        }
        Err(e) => {
            println!("\n❌ Invalid Mnemonic!");
            println!("Error: {}\n", e);
        }
    }
}

fn generate_seed() {
    println!("\n--- Generate Seed from Mnemonic ---");
    println!("Enter your mnemonic phrase:");
    print!("> ");
    io::stdout().flush().unwrap();

    let mut phrase = String::new();
    io::stdin().read_line(&mut phrase).unwrap();
    let phrase = phrase.trim();

    match Mnemonic::from_phrase(phrase, Language::English) {
        Ok(mnemonic) => {
            println!("\n✅ Valid mnemonic!");

            println!("\nEnter passphrase (press Enter for no passphrase):");
            print!("> ");
            io::stdout().flush().unwrap();

            let mut passphrase = String::new();
            io::stdin().read_line(&mut passphrase).unwrap();
            let passphrase = passphrase.trim();

            let seed = mnemonic.to_seed(passphrase);

            println!("\n🌱 Seed Generated (512 bits):");
            println!("{}", seed.to_hex());

            if passphrase.is_empty() {
                println!("\n💡 No passphrase used");
            } else {
                println!("\n🔒 Passphrase used: '{}'", passphrase);
            }
            println!();
        }
        Err(e) => {
            println!("\n❌ Invalid mnemonic: {}\n", e);
        }
    }
}
