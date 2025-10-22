#![forbid(unsafe_code)]

use bip39_wallet::{
    prelude::*,
    wallet::HDPath as WalletPath,
};
use bitcoin::Network;
use clap::{Parser, Subcommand};
use colored::Colorize;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Password};
use indicatif::{ProgressBar, ProgressStyle};
use std::{thread, time::Duration};

#[derive(Parser)]
#[command(name = "bip39-wallet")]
#[command(about = "BIP-39 HD Wallet Generator", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, default_value = "bitcoin")]
    network: String,
}

#[derive(Subcommand)]
enum Commands {

    Generate {

        #[arg(short, long, default_value = "12")]
        words: usize,

        #[arg(short, long, default_value = "english")]
        language: String,

        #[arg(short, long)]
        passphrase: Option<String>,

        #[arg(short = 'n', long, default_value = "5")]
        addresses: usize,

        #[arg(short, long)]
        export: Option<String>,
    },

    Recover {

        #[arg(short, long)]
        mnemonic: Option<String>,

        #[arg(short, long)]
        passphrase: Option<String>,

        #[arg(short = 'n', long, default_value = "5")]
        addresses: usize,
    },

    Validate {

        #[arg(short, long)]
        mnemonic: Option<String>,
    },

    Derive {

        #[arg(short, long)]
        mnemonic: Option<String>,

        #[arg(short, long, default_value = "m/44'/0'/0'/0/0")]
        path: String,

        #[arg(short, long)]
        passphrase: Option<String>,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let network = match cli.network.as_str() {
        "bitcoin" | "mainnet" => Network::Bitcoin,
        "testnet" => Network::Testnet,
        "regtest" => Network::Regtest,
        _ => {
            eprintln!("{}", "Invalid network. Use: bitcoin, testnet, or regtest".red());
            std::process::exit(1);
        }
    };

    match cli.command {
        Commands::Generate { words, language, passphrase, addresses, export } => {
            generate_wallet(words, &language, passphrase, addresses, network, export)?;
        }
        Commands::Recover { mnemonic, passphrase, addresses } => {
            recover_wallet(mnemonic, passphrase, addresses, network)?;
        }
        Commands::Validate { mnemonic } => {
            validate_mnemonic(mnemonic)?;
        }
        Commands::Derive { mnemonic, path, passphrase } => {
            derive_address(mnemonic, &path, passphrase, network)?;
        }
    }

    Ok(())
}

fn generate_wallet(
    words: usize,
    _language: &str,
    passphrase: Option<String>,
    num_addresses: usize,
    network: Network,
    export_path: Option<String>,
) -> anyhow::Result<()> {
    println!("\n{}", "🔐 BIP-39 Wallet Generator".cyan().bold());
    println!("{}", "═".repeat(50).cyan());

    let bits = EntropyBits::from_word_count(words)?;

    let lang = Language::English;

    let passphrase = match passphrase {
        Some(p) => p,
        None => {
            if Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Use a passphrase? (recommended)")
                .default(false)
                .interact()?
            {
                Password::with_theme(&ColorfulTheme::default())
                    .with_prompt("Enter passphrase")
                    .with_confirmation("Confirm passphrase", "Passphrases don't match")
                    .interact()?
            } else {
                String::new()
            }
        }
    };

    let pb = ProgressBar::new(100);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {msg}")
            .unwrap()
            .progress_chars("#>-"),
    );

    pb.set_message("Generating entropy...");
    pb.inc(20);
    thread::sleep(Duration::from_millis(100));

    let mnemonic = Mnemonic::generate(bits, lang)?;
    pb.inc(30);
    pb.set_message("Creating mnemonic...");
    thread::sleep(Duration::from_millis(100));

    let seed = mnemonic.to_seed(&passphrase);
    pb.inc(25);
    pb.set_message("Deriving wallet...");
    thread::sleep(Duration::from_millis(100));

    let wallet = bip39_wallet::wallet::Wallet::from_seed(&seed, network)?;
    pb.inc(25);
    pb.finish_with_message("✅ Wallet generated successfully!");

    println!("\n{}", "📝 MNEMONIC PHRASE (KEEP SAFE!)".yellow().bold());
    println!("{}", "─".repeat(50).yellow());

    for (i, word) in mnemonic.words().iter().enumerate() {
        print!("{:2}. {:12} ", i + 1, word.green().bold());
        if (i + 1) % 4 == 0 {
            println!();
        }
    }

    println!("\n{}", "🔑 SEED INFORMATION".blue().bold());
    println!("{}", "─".repeat(50).blue());
    println!("Entropy: {}", mnemonic.entropy().to_hex().cyan());
    println!("Seed: {}...", &seed.to_hex()[..32].cyan());

    if !passphrase.is_empty() {
        println!("Passphrase: {}", "****** (hidden)".yellow());
    }

    println!("\n{}", "💰 DERIVED ADDRESSES".green().bold());
    println!("{}", "─".repeat(50).green());

    for i in 0..num_addresses {
        let path = WalletPath {
            coin: 0,
            account: 0,
            change: 0,
            index: i as u32,
        };

        let info = wallet.get_account_info(&path)?;

        println!("\n{} {}",
            format!("Address #{}:", i).white().bold(),
            format!("({})", info.path).white().dimmed()
        );
        println!("  Address: {}", info.address.yellow());
        println!("  Public Key: {}", &info.public_key[..20].cyan());

        if i == 0 {

            println!("  Private Key: {}", &info.private_key[..20].red().dimmed());
            println!("  {}", "(full keys hidden for security)".white().dimmed());
        }
    }

    if let Some(export_path) = export_path {
        export_wallet_json(&mnemonic, &passphrase, network, &export_path)?;
        println!("\n✅ Wallet exported to: {}", export_path.green());
    }

    println!("\n{}", "⚠️  SECURITY WARNINGS".red().bold());
    println!("{}", "─".repeat(50).red());
    println!("• {} Write down your mnemonic phrase on paper", "CRITICAL:".red().bold());
    println!("• Never share your mnemonic or private keys");
    println!("• Store backups in multiple secure locations");
    println!("• Test recovery with small amounts first");

    Ok(())
}

fn recover_wallet(
    mnemonic: Option<String>,
    passphrase: Option<String>,
    num_addresses: usize,
    network: Network,
) -> anyhow::Result<()> {
    println!("\n{}", "🔄 Wallet Recovery".cyan().bold());
    println!("{}", "═".repeat(50).cyan());

    let mnemonic_phrase = match mnemonic {
        Some(m) => m,
        None => {
            Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Enter your mnemonic phrase")
                .interact_text()?
        }
    };

    let passphrase = match passphrase {
        Some(p) => p,
        None => {
            if Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Did you use a passphrase?")
                .default(false)
                .interact()?
            {
                Password::with_theme(&ColorfulTheme::default())
                    .with_prompt("Enter passphrase")
                    .interact()?
            } else {
                String::new()
            }
        }
    };

    let mnemonic = Mnemonic::from_phrase(&mnemonic_phrase, Language::English)?;
    let seed = mnemonic.to_seed(&passphrase);
    let wallet = bip39_wallet::wallet::Wallet::from_seed(&seed, network)?;

    println!("\n✅ {} Wallet recovered successfully!", "SUCCESS:".green().bold());

    println!("\n{}", "💰 RECOVERED ADDRESSES".green().bold());
    println!("{}", "─".repeat(50).green());

    for i in 0..num_addresses {
        let path = WalletPath {
            coin: 0,
            account: 0,
            change: 0,
            index: i as u32,
        };

        let address = wallet.get_address(&path)?;
        println!("Address #{}: {}", i, address.to_string().yellow());
    }

    Ok(())
}

fn validate_mnemonic(mnemonic: Option<String>) -> anyhow::Result<()> {
    println!("\n{}", "✔️  Mnemonic Validator".cyan().bold());
    println!("{}", "═".repeat(50).cyan());

    let mnemonic_phrase = match mnemonic {
        Some(m) => m,
        None => {
            Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Enter mnemonic to validate")
                .interact_text()?
        }
    };

    match Mnemonic::from_phrase(&mnemonic_phrase, Language::English) {
        Ok(mnemonic) => {
            println!("\n✅ {} Valid mnemonic!", "SUCCESS:".green().bold());
            println!("Word count: {}", mnemonic.word_count());
            println!("Entropy: {} bits", mnemonic.entropy().bits());
            println!("Checksum: Valid ✓");
        }
        Err(e) => {
            println!("\n❌ {} Invalid mnemonic!", "ERROR:".red().bold());
            println!("Reason: {}", e.to_string().red());
        }
    }

    Ok(())
}

fn derive_address(
    mnemonic: Option<String>,
    path: &str,
    passphrase: Option<String>,
    network: Network,
) -> anyhow::Result<()> {
    println!("\n{}", "🔍 Address Derivation".cyan().bold());
    println!("{}", "═".repeat(50).cyan());

    let mnemonic_phrase = match mnemonic {
        Some(m) => m,
        None => {
            Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Enter mnemonic")
                .interact_text()?
        }
    };

    let passphrase = passphrase.unwrap_or_default();

    let _path_parts: Vec<&str> = path.trim_start_matches("m/").split('/').collect();
    let path = WalletPath {
        coin: 0,
        account: 0,
        change: 0,
        index: 0,
    };

    let mnemonic = Mnemonic::from_phrase(&mnemonic_phrase, Language::English)?;
    let seed = mnemonic.to_seed(&passphrase);
    let wallet = bip39_wallet::wallet::Wallet::from_seed(&seed, network)?;

    let info = wallet.get_account_info(&path)?;

    println!("\n{}", "📍 DERIVED ADDRESS".green().bold());
    println!("{}", "─".repeat(50).green());
    println!("Path: {}", info.path.cyan());
    println!("Address: {}", info.address.yellow().bold());
    println!("Public Key: {}", info.public_key.cyan());
    println!("xPub: {}", info.xpub.blue());

    Ok(())
}

fn export_wallet_json(
    mnemonic: &Mnemonic,
    passphrase: &str,
    network: Network,
    path: &str,
) -> anyhow::Result<()> {
    use std::fs;
    use serde_json::json;

    let seed = mnemonic.to_seed(passphrase);
    let wallet = bip39_wallet::wallet::Wallet::from_seed(&seed, network)?;

    let mut addresses = vec![];
    for i in 0..10 {
        let path = WalletPath {
            coin: 0,
            account: 0,
            change: 0,
            index: i,
        };

        let info = wallet.get_account_info(&path)?;
        addresses.push(json!({
            "index": i,
            "path": info.path,
            "address": info.address,
            "publicKey": info.public_key,
        }));
    }

    let export_data = json!({
        "version": "1.0.0",
        "network": format!("{:?}", network),
        "mnemonic": mnemonic.phrase(),
        "wordCount": mnemonic.word_count(),
        "hasPassphrase": !passphrase.is_empty(),
        "addresses": addresses,
        "warning": "KEEP THIS FILE SECURE! It contains your wallet seed."
    });

    fs::write(path, serde_json::to_string_pretty(&export_data)?)?;
    Ok(())
}
