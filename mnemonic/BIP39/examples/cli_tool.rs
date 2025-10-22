use bip39_wallet::{prelude::*, wallet::Wallet};
use bitcoin::Network;
use std::fs;
use std::io::{self, Write as IoWrite};

type CliResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> CliResult<()> {
    println!("🔐 BIP-39 Wallet - Interactive CLI Tool");
    println!("=========================================\n");

    loop {
        print_menu();
        let choice = get_input("Enter your choice: ")?;

        match choice.trim() {
            "1" => generate_new_wallet()?,
            "2" => recover_from_mnemonic()?,
            "3" => validate_mnemonic()?,
            "4" => generate_addresses()?,
            "5" => derive_custom_path()?,
            "6" => export_wallet()?,
            "7" => import_wallet()?,
            "8" => full_workflow()?,
            "9" => {
                println!("\n👋 Goodbye!\n");
                break;
            }
            _ => println!("❌ Invalid choice! Please try again.\n"),
        }
    }

    Ok(())
}

fn print_menu() {
    println!("📋 Main Menu:");
    println!("  1. Generate new wallet");
    println!("  2. Recover wallet from mnemonic");
    println!("  3. Validate mnemonic phrase");
    println!("  4. Generate addresses from existing wallet");
    println!("  5. Derive custom HD path");
    println!("  6. Export wallet to file");
    println!("  7. Import wallet from file");
    println!("  8. Full workflow (generate + export + addresses)");
    println!("  9. Exit");
    println!();
}

fn get_input(prompt: &str) -> CliResult<String> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn get_entropy_bits() -> CliResult<EntropyBits> {
    println!("\n📏 Select entropy strength:");
    println!("  1. 128 bits (12 words) - Standard");
    println!("  2. 160 bits (15 words)");
    println!("  3. 192 bits (18 words)");
    println!("  4. 224 bits (21 words)");
    println!("  5. 256 bits (24 words) - Maximum security");

    let choice = get_input("Your choice: ")?;

    match choice.as_str() {
        "1" => Ok(EntropyBits::Bits128),
        "2" => Ok(EntropyBits::Bits160),
        "3" => Ok(EntropyBits::Bits192),
        "4" => Ok(EntropyBits::Bits224),
        "5" => Ok(EntropyBits::Bits256),
        _ => Err("Invalid entropy choice".into()),
    }
}

fn get_network() -> CliResult<Network> {
    println!("\n🌐 Select network:");
    println!("  1. Bitcoin Mainnet");
    println!("  2. Bitcoin Testnet");

    let choice = get_input("Your choice: ")?;

    match choice.as_str() {
        "1" => Ok(Network::Bitcoin),
        "2" => Ok(Network::Testnet),
        _ => Err("Invalid network choice".into()),
    }
}

fn generate_new_wallet() -> CliResult<()> {
    println!("\n🆕 Generate New Wallet");
    println!("─────────────────────────");

    let bits = get_entropy_bits()?;
    let network = get_network()?;

    let passphrase = get_input("\n🔒 Enter passphrase (press Enter for no passphrase): ")?;

    println!("\n⏳ Generating mnemonic...");
    let mnemonic = Mnemonic::generate(bits, Language::English)?;

    println!("\n✅ Wallet Generated Successfully!\n");
    println!("╔═══════════════════════════════════════════════════════╗");
    println!("║           🔑 YOUR MNEMONIC PHRASE                     ║");
    println!("╠═══════════════════════════════════════════════════════╣");

    let phrase = mnemonic.phrase();
    let words: Vec<&str> = phrase.split_whitespace().collect();
    for (i, word) in words.iter().enumerate() {
        if i % 3 == 0 && i > 0 {
            println!("║                                                       ║");
        }
        print!("║ {:2}. {:<15}", i + 1, word);
        if (i + 1) % 3 == 0 {
            println!("║");
        } else if i == words.len() - 1 {
            for _ in 0..(3 - (i + 1) % 3) {
                print!("                 ");
            }
            println!("║");
        }
    }

    println!("╚═══════════════════════════════════════════════════════╝");

    println!("\n📊 Wallet Information:");
    println!("  • Word count: {} words", mnemonic.word_count());
    println!("  • Entropy: {} bits", bits as u32);
    println!("  • Network: {:?}", network);
    println!("  • Language: English");
    if !passphrase.is_empty() {
        println!("  • Passphrase: *** (protected)");
    }

    let seed = mnemonic.to_seed(&passphrase);
    println!("\n🌱 Seed (first 32 chars):");
    println!("  {}", &seed.to_hex()[..64]);

    let wallet = Wallet::from_seed(&seed, network)?;

    println!("\n🏠 First 3 Addresses:");
    let addresses = wallet.generate_addresses(3, 0)?;
    for (i, addr) in addresses.iter().enumerate() {
        println!("  {}. {}", i + 1, addr);
    }

    println!("\n⚠️  SECURITY WARNINGS:");
    println!("  • Write down these words on paper and store securely");
    println!("  • NEVER share your mnemonic with anyone");
    println!("  • Anyone with these words can access your funds");
    println!("  • Make multiple backups in different secure locations");

    let save = get_input("\n💾 Save this wallet? (y/n): ")?;
    if save.to_lowercase() == "y" {
        save_wallet_to_file(&mnemonic, network, &passphrase)?;
    }

    println!();
    Ok(())
}

fn recover_from_mnemonic() -> CliResult<()> {
    println!("\n🔄 Recover Wallet from Mnemonic");
    println!("───────────────────────────────");

    println!("\nEnter your mnemonic phrase (12-24 words, space-separated):");
    let phrase = get_input("> ")?;

    println!("\n⏳ Validating mnemonic...");
    let mnemonic = Mnemonic::from_phrase(&phrase, Language::English)?;

    println!("✅ Valid mnemonic!");

    let network = get_network()?;
    let passphrase = get_input("\n🔒 Enter passphrase (press Enter if none was used): ")?;

    let seed = mnemonic.to_seed(&passphrase);
    let wallet = Wallet::from_seed(&seed, network)?;

    println!("\n✅ Wallet Recovered Successfully!\n");
    println!("📊 Wallet Information:");
    println!("  • Word count: {} words", mnemonic.word_count());
    println!("  • Network: {:?}", network);
    println!("  • Entropy: {} bits", mnemonic.entropy().bits() as usize);

    println!("\n🏠 First 5 Addresses:");
    let addresses = wallet.generate_addresses(5, 0)?;
    for (i, addr) in addresses.iter().enumerate() {
        println!("  {}. {}", i + 1, addr);
    }

    println!();
    Ok(())
}

fn validate_mnemonic() -> CliResult<()> {
    println!("\n✓ Validate Mnemonic Phrase");
    println!("──────────────────────────");

    println!("\nEnter mnemonic phrase to validate:");
    let phrase = get_input("> ")?;

    println!("\n⏳ Validating...");

    match Mnemonic::from_phrase(&phrase, Language::English) {
        Ok(mnemonic) => {
            println!("\n✅ VALID MNEMONIC!\n");
            println!("📊 Details:");
            println!("  • Word count: {} words", mnemonic.word_count());
            println!("  • Entropy: {} bits", mnemonic.entropy().bits() as usize);
            println!("  • Language: {:?}", mnemonic.language());
            println!("  • Checksum: Valid ✓");

            let seed = mnemonic.to_seed("");
            println!("\n🌱 Seed preview (no passphrase):");
            println!("  {}...", &seed.to_hex()[..64]);
        }
        Err(e) => {
            println!("\n❌ INVALID MNEMONIC!\n");
            println!("Error: {}", e);
            println!("\n💡 Common issues:");
            println!("  • Wrong word order");
            println!("  • Typo in one or more words");
            println!("  • Word not in BIP-39 wordlist");
            println!("  • Invalid checksum");
        }
    }

    println!();
    Ok(())
}

fn generate_addresses() -> CliResult<()> {
    println!("\n🏠 Generate Addresses");
    println!("─────────────────────");

    println!("\nEnter your mnemonic phrase:");
    let phrase = get_input("> ")?;

    let mnemonic = Mnemonic::from_phrase(&phrase, Language::English)?;
    let network = get_network()?;

    let passphrase = get_input("\n🔒 Enter passphrase (press Enter if none): ")?;

    let count_str = get_input("\n📊 How many addresses to generate? (1-20): ")?;
    let count: usize = count_str.parse().unwrap_or(5).min(20).max(1);

    let start_str = get_input("Starting index (default 0): ")?;
    let start: u32 = start_str.parse().unwrap_or(0);

    let seed = mnemonic.to_seed(&passphrase);
    let wallet = Wallet::from_seed(&seed, network)?;

    println!("\n✅ Generating {} addresses starting from index {}...\n", count, start);

    let addresses = wallet.generate_addresses(count, start)?;

    println!("╔═══════════════════════════════════════════════════════╗");
    println!("║                  GENERATED ADDRESSES                  ║");
    println!("╠═══════════════════════════════════════════════════════╣");

    for (i, addr) in addresses.iter().enumerate() {
        println!("║ [{:2}] {:44} ║", start as usize + i, addr);
    }

    println!("╚═══════════════════════════════════════════════════════╝");

    let export = get_input("\n💾 Export to file? (y/n): ")?;
    if export.to_lowercase() == "y" {
        use chrono::Local;
        let filename = format!("addresses_{}_{}.txt",
            network.to_string().to_lowercase(),
            Local::now().format("%Y%m%d_%H%M%S")
        );
        let mut content = format!("Generated Addresses - {} Network\n", network);
        content.push_str(&format!("Generated: {}\n", Local::now().format("%Y-%m-%d %H:%M:%S")));
        content.push_str(&format!("Range: {} to {}\n\n", start, start + count as u32 - 1));

        for (i, addr) in addresses.iter().enumerate() {
            content.push_str(&format!("[{}] {}\n", start as usize + i, addr));
        }

        fs::write(&filename, content)?;
        println!("✅ Addresses saved to: {}", filename);
    }

    println!();
    Ok(())
}

fn derive_custom_path() -> CliResult<()> {
    println!("\n🛤️  Derive Custom HD Path");
    println!("─────────────────────────");

    println!("\nEnter your mnemonic phrase:");
    let phrase = get_input("> ")?;

    let mnemonic = Mnemonic::from_phrase(&phrase, Language::English)?;
    let network = get_network()?;

    let passphrase = get_input("\n🔒 Enter passphrase (press Enter if none): ")?;

    println!("\n📋 Select derivation path:");
    println!("  1. Bitcoin (m/44'/0'/0'/0/0)");
    println!("  2. Ethereum (m/44'/60'/0'/0/0)");

    let path_choice = get_input("Your choice: ")?;

    let seed = mnemonic.to_seed(&passphrase);
    let wallet = Wallet::from_seed(&seed, network)?;

    match path_choice.as_str() {
        "1" => {
            let path = HDPath::bitcoin();
            let address = wallet.get_address(&path)?;
            println!("\n✅ Bitcoin address (m/44'/0'/0'/0/0): ");
            println!("   {}", address);
        }
        "2" => {
            let path = HDPath::ethereum();
            let address = wallet.get_address(&path)?;
            println!("\n✅ Ethereum address (m/44'/60'/0'/0/0): ");
            println!("   {}", address);
        }
        _ => println!("❌ Invalid choice"),
    }

    println!();
    Ok(())
}

fn save_wallet_to_file(mnemonic: &Mnemonic, network: Network, passphrase: &str) -> CliResult<()> {
    use chrono::Local;
    let timestamp = Local::now().format("%Y%m%d_%H%M%S");
    let filename = format!("wallet_{}_{}.json", network.to_string().to_lowercase(), timestamp);

    let wallet_data = serde_json::json!({
        "mnemonic": mnemonic.phrase(),
        "word_count": mnemonic.word_count(),
        "network": format!("{:?}", network),
        "has_passphrase": !passphrase.is_empty(),
        "entropy_hex": mnemonic.entropy().to_hex(),
        "created_at": Local::now().to_rfc3339(),
        "warning": "⚠️ KEEP THIS FILE SECURE! Anyone with access can steal your funds!"
    });

    fs::write(&filename, serde_json::to_string_pretty(&wallet_data)?)?;

    println!("✅ Wallet saved to: {}", filename);
    println!("⚠️  IMPORTANT: Store this file in a secure location!");

    Ok(())
}

fn export_wallet() -> CliResult<()> {
    println!("\n💾 Export Wallet");
    println!("────────────────");

    println!("\nEnter your mnemonic phrase:");
    let phrase = get_input("> ")?;

    let mnemonic = Mnemonic::from_phrase(&phrase, Language::English)?;
    let network = get_network()?;

    let passphrase = get_input("\n🔒 Enter passphrase (press Enter if none): ")?;

    save_wallet_to_file(&mnemonic, network, &passphrase)?;

    println!();
    Ok(())
}

fn import_wallet() -> CliResult<()> {
    println!("\n📥 Import Wallet from File");
    println!("──────────────────────────");

    let filename = get_input("\nEnter wallet file path: ")?;

    if !std::path::Path::new(&filename).exists() {
        println!("❌ File not found: {}", filename);
        return Ok(());
    }

    println!("\n⏳ Reading wallet file...");
    let content = fs::read_to_string(&filename)?;

    let wallet_data: serde_json::Value = serde_json::from_str(&content)?;

    if let Some(phrase) = wallet_data["mnemonic"].as_str() {
        println!("✅ Wallet loaded successfully!\n");

        if let Some(word_count) = wallet_data["word_count"].as_u64() {
            println!("📊 Wallet Info:");
            println!("  • Word count: {}", word_count);
        }

        if let Some(network) = wallet_data["network"].as_str() {
            println!("  • Network: {}", network);
        }

        if let Some(created) = wallet_data["created_at"].as_str() {
            println!("  • Created: {}", created);
        }

        let recover = get_input("\n🔄 Would you like to recover this wallet now? (y/n): ")?;

        if recover.to_lowercase() == "y" {
            let mnemonic = Mnemonic::from_phrase(phrase, Language::English)?;

            let passphrase = if wallet_data["has_passphrase"].as_bool().unwrap_or(false) {
                get_input("\n🔒 Enter passphrase: ")?
            } else {
                String::new()
            };

            let network = if wallet_data["network"].as_str().unwrap_or("").contains("Testnet") {
                Network::Testnet
            } else {
                Network::Bitcoin
            };

            let seed = mnemonic.to_seed(&passphrase);
            let wallet = Wallet::from_seed(&seed, network)?;

            println!("\n🏠 First 5 Addresses:");
            let addresses = wallet.generate_addresses(5, 0)?;
            for (i, addr) in addresses.iter().enumerate() {
                println!("  {}. {}", i + 1, addr);
            }
        }
    } else {
        println!("❌ Invalid wallet file format");
    }

    println!();
    Ok(())
}

fn full_workflow() -> CliResult<()> {
    println!("\n🎯 Full Workflow");
    println!("════════════════");
    println!("This will: Generate mnemonic → Export to file → Generate addresses\n");

    let bits = get_entropy_bits()?;
    let network = get_network()?;
    let passphrase = get_input("\n🔒 Enter passphrase (press Enter for none): ")?;

    println!("\n⏳ Step 1/3: Generating mnemonic...");
    let mnemonic = Mnemonic::generate(bits, Language::English)?;
    println!("✅ Mnemonic generated ({} words)", mnemonic.word_count());

    println!("\n⏳ Step 2/3: Exporting to file...");
    save_wallet_to_file(&mnemonic, network, &passphrase)?;

    println!("\n⏳ Step 3/3: Generating addresses...");
    let seed = mnemonic.to_seed(&passphrase);
    let wallet = Wallet::from_seed(&seed, network)?;
    let addresses = wallet.generate_addresses(10, 0)?;

    println!("\n✅ Workflow Complete!\n");
    println!("╔═══════════════════════════════════════════════════════╗");
    println!("║                  YOUR MNEMONIC                        ║");
    println!("╚═══════════════════════════════════════════════════════╝");
    println!("{}\n", mnemonic.phrase());

    println!("🏠 First 10 Addresses:");
    for (i, addr) in addresses.iter().enumerate() {
        println!("  {:2}. {}", i + 1, addr);
    }

    println!("\n⚠️  Remember to backup your mnemonic securely!\n");

    Ok(())
}
