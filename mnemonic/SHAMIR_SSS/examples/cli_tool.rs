
use shamir_sss::{split_mnemonic, reconstruct_mnemonic, MnemonicShare};
use pure_bip39::{Mnemonic, Language, EntropyBits};
use std::io::{self, Write};
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔐 Shamir Secret Sharing - Interactive CLI Tool");
    println!("=================================================\n");

    loop {
        print_menu();

        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;

        match choice.trim() {
            "1" => generate_mnemonic()?,
            "2" => split_existing_mnemonic()?,
            "3" => save_shares()?,
            "4" => load_and_reconstruct()?,
            "5" => full_workflow()?,
            "6" => {
                println!("\n👋 Goodbye!");
                break;
            }
            _ => println!("❌ Invalid choice! Please try again."),
        }
    }

    Ok(())
}

fn print_menu() {
    println!("\n📋 Main Menu:");
    println!("  1. Generate new BIP39 mnemonic (PURE_BIP39)");
    println!("  2. Split mnemonic into shares (SHAMIR_SSS)");
    println!("  3. Save shares to files");
    println!("  4. Load shares and reconstruct mnemonic");
    println!("  5. Full workflow (all steps)");
    println!("  6. Exit");
    print!("\nYour choice: ");
    io::stdout().flush().unwrap();
}

fn generate_mnemonic() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📝 Step 1: Generate BIP39 Mnemonic");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    println!("Choose word count:");
    println!("  1. 12 words (128 bits)");
    println!("  2. 15 words (160 bits)");
    println!("  3. 18 words (192 bits)");
    println!("  4. 21 words (224 bits)");
    println!("  5. 24 words (256 bits) [Most secure]");
    print!("\nChoice (default 5): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let entropy_bits = match input.trim() {
        "1" => EntropyBits::Bits128,
        "2" => EntropyBits::Bits160,
        "3" => EntropyBits::Bits192,
        "4" => EntropyBits::Bits224,
        _ => EntropyBits::Bits256,
    };

    println!("\n⏳ Generating mnemonic...");
    let mnemonic = Mnemonic::generate(entropy_bits, Language::English)?;

    println!("\n✅ Mnemonic generated successfully!\n");
    println!("┌─────────────────────────────────────────────┐");
    println!("│ YOUR MNEMONIC PHRASE (WRITE IT DOWN!)      │");
    println!("└─────────────────────────────────────────────┘");
    println!("\n{}\n", mnemonic.phrase());
    println!("┌─────────────────────────────────────────────┐");
    println!("│ ⚠️  KEEP THIS SAFE! ⚠️                      │");
    println!("└─────────────────────────────────────────────┘\n");

    println!("📊 Details:");
    println!("  - Word count: {}", mnemonic.word_count());
    println!("  - Entropy: {} bits", mnemonic.entropy().as_bytes().len() * 8);
    println!("  - Language: {:?}", mnemonic.language());

    let seed = mnemonic.to_seed("");
    let seed_hex = hex::encode(&seed.as_bytes()[..32]);
    println!("  - Seed (first 32 bytes): {}...", &seed_hex[..16]);

    fs::write("temp_mnemonic.txt", mnemonic.phrase())?;
    println!("\n💾 Mnemonic saved to: temp_mnemonic.txt");

    Ok(())
}

fn split_existing_mnemonic() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🔀 Step 2: Split Mnemonic into Shares");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    let mnemonic_phrase = if Path::new("temp_mnemonic.txt").exists() {
        print!("Load mnemonic from temp_mnemonic.txt? (y/n, default y): ");
        io::stdout().flush()?;

        let mut load_choice = String::new();
        io::stdin().read_line(&mut load_choice)?;

        if load_choice.trim().to_lowercase() != "n" {
            fs::read_to_string("temp_mnemonic.txt")?
        } else {
            get_mnemonic_input()?
        }
    } else {
        get_mnemonic_input()?
    };

    println!("\n⏳ Validating mnemonic...");
    let mnemonic = Mnemonic::from_phrase(mnemonic_phrase.trim(), Language::English)?;
    println!("✅ Valid BIP39 mnemonic! ({} words)", mnemonic.word_count());

    print!("\nEnter threshold (minimum shares needed, default 3): ");
    io::stdout().flush()?;
    let mut t_input = String::new();
    io::stdin().read_line(&mut t_input)?;
    let threshold: usize = t_input.trim().parse().unwrap_or(3);

    print!("Enter total number of shares (default 5): ");
    io::stdout().flush()?;
    let mut n_input = String::new();
    io::stdin().read_line(&mut n_input)?;
    let total_shares: usize = n_input.trim().parse().unwrap_or(5);

    if threshold > total_shares {
        return Err("Threshold cannot be greater than total shares!".into());
    }

    if threshold < 2 {
        return Err("Threshold must be at least 2!".into());
    }

    println!("\n⏳ Splitting mnemonic...");
    println!("   Configuration: {}-of-{} scheme", threshold, total_shares);

    let shares = split_mnemonic(&mnemonic.phrase(), threshold, total_shares, Language::English)?;

    println!("\n✅ Successfully created {} shares!\n", shares.len());

    println!("┌─────────────────────────────────────────────┐");
    println!("│ SHARES (Save each to a different location) │");
    println!("└─────────────────────────────────────────────┘\n");

    for (i, share) in shares.iter().enumerate() {
        println!("Share #{}/{}:", i + 1, total_shares);
        println!("  ID: {}", share.id);
        println!("  Data: {}...{}", &share.share_data[..16], &share.share_data[share.share_data.len()-8..]);
        println!("  Full length: {} characters\n", share.share_data.len());
    }

    println!("ℹ️  Reconstruction requires any {} shares", threshold);

    let shares_json = serde_json::to_string_pretty(&shares)?;
    fs::write("temp_shares.json", shares_json)?;
    println!("\n💾 Shares saved to: temp_shares.json");

    Ok(())
}

fn get_mnemonic_input() -> Result<String, Box<dyn std::error::Error>> {
    println!("\nEnter your BIP39 mnemonic phrase:");
    println!("(12-24 words separated by spaces)");
    print!("> ");
    io::stdout().flush()?;

    let mut mnemonic = String::new();
    io::stdin().read_line(&mut mnemonic)?;

    Ok(mnemonic)
}

fn save_shares() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("💾 Step 3: Save Shares to Files");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    if !Path::new("temp_shares.json").exists() {
        return Err("No shares found! Please run Step 2 first.".into());
    }

    let shares_json = fs::read_to_string("temp_shares.json")?;
    let shares: Vec<MnemonicShare> = serde_json::from_str(&shares_json)?;

    println!("Found {} shares. Saving to individual files...\n", shares.len());

    fs::create_dir_all("shares")?;

    for (i, share) in shares.iter().enumerate() {
        let filename = format!("shares/share_{}.json", i + 1);
        let json = share.to_json()?;
        fs::write(&filename, json)?;
        println!("✅ Saved: {}", filename);
    }

    println!("\n📁 All shares saved to 'shares/' directory");
    println!("\n⚠️  IMPORTANT: Store each share in a different secure location!");
    println!("   Examples:");
    println!("   - Share 1: Home safe");
    println!("   - Share 2: Bank deposit box");
    println!("   - Share 3: Trusted friend");
    println!("   - Share 4: Cloud storage (encrypted)");
    println!("   - Share 5: Lawyer's office");

    Ok(())
}

fn load_and_reconstruct() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🔓 Step 4: Load Shares and Reconstruct");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    if !Path::new("shares").exists() {
        return Err("No 'shares/' directory found! Please run Step 3 first.".into());
    }

    let entries = fs::read_dir("shares")?;
    let mut share_files: Vec<String> = entries
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "json"))
        .map(|e| e.path().to_string_lossy().to_string())
        .collect();

    share_files.sort();

    if share_files.is_empty() {
        return Err("No share files found in 'shares/' directory!".into());
    }

    println!("Found {} share files:\n", share_files.len());
    for (i, file) in share_files.iter().enumerate() {
        println!("  {}. {}", i + 1, file);
    }

    let first_share_json = fs::read_to_string(&share_files[0])?;
    let first_share: MnemonicShare = serde_json::from_str(&first_share_json)?;
    let threshold = first_share.threshold as usize;

    println!("\nℹ️  Threshold: {} shares needed", threshold);
    println!("\n┌─────────────────────────────────────────────┐");
    println!("│ Select which shares to use (comma-separated)│");
    println!("└─────────────────────────────────────────────┘");
    print!("\nShare numbers (e.g., 1,2,3): ");
    io::stdout().flush()?;

    let mut selection = String::new();
    io::stdin().read_line(&mut selection)?;

    let selected_indices: Vec<usize> = selection
        .trim()
        .split(',')
        .filter_map(|s| s.trim().parse::<usize>().ok())
        .filter(|&i| i > 0 && i <= share_files.len())
        .map(|i| i - 1)
        .collect();

    if selected_indices.len() < threshold {
        return Err(format!(
            "Not enough shares! Need at least {} but got {}",
            threshold,
            selected_indices.len()
        ).into());
    }

    println!("\n⏳ Loading {} shares...", selected_indices.len());

    let mut shares = Vec::new();
    for &idx in &selected_indices {
        let json = fs::read_to_string(&share_files[idx])?;
        let share: MnemonicShare = serde_json::from_str(&json)?;
        println!("  ✅ Loaded share #{}", share.id);
        shares.push(share);
    }

    println!("\n⏳ Reconstructing mnemonic...");
    let recovered = reconstruct_mnemonic(&shares, Language::English)?;

    println!("\n✅ RECONSTRUCTION SUCCESSFUL!\n");
    println!("┌─────────────────────────────────────────────┐");
    println!("│ RECOVERED MNEMONIC PHRASE                   │");
    println!("└─────────────────────────────────────────────┘");
    println!("\n{}\n", recovered);

    if Path::new("temp_mnemonic.txt").exists() {
        let original = fs::read_to_string("temp_mnemonic.txt")?;
        if original.trim() == recovered.trim() {
            println!("🎉 PERFECT MATCH! Recovered mnemonic matches original!");

            let original_mnemonic = Mnemonic::from_phrase(original.trim(), Language::English)?;
            let recovered_mnemonic = Mnemonic::from_phrase(&recovered, Language::English)?;

            let original_seed = original_mnemonic.to_seed("");
            let recovered_seed = recovered_mnemonic.to_seed("");

            if original_seed.as_bytes() == recovered_seed.as_bytes() {
                println!("✨ Seeds also match - perfect reconstruction!");
            }
        } else {
            println!("⚠️  WARNING: Recovered mnemonic differs from original!");
            println!("    Original: {}", original.trim());
            println!("    Recovered: {}", recovered);
        }
    }

    fs::write("recovered_mnemonic.txt", &recovered)?;
    println!("\n💾 Recovered mnemonic saved to: recovered_mnemonic.txt");

    Ok(())
}

fn full_workflow() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🚀 Full Workflow - All Steps");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    println!("Step 1/4: Generating mnemonic...");
    let mnemonic = Mnemonic::generate(EntropyBits::Bits256, Language::English)?;
    println!("✅ Generated: {} words", mnemonic.word_count());
    println!("   Mnemonic: {}", mnemonic.phrase());

    println!("\nStep 2/4: Splitting into shares (3-of-5)...");
    let shares = split_mnemonic(&mnemonic.phrase(), 3, 5, Language::English)?;
    println!("✅ Created {} shares", shares.len());

    println!("\nStep 3/4: Saving shares...");
    fs::create_dir_all("shares")?;
    for (i, share) in shares.iter().enumerate() {
        let filename = format!("shares/share_{}.json", i + 1);
        fs::write(&filename, share.to_json()?)?;
    }
    println!("✅ Saved to 'shares/' directory");

    println!("\nStep 4/4: Reconstructing from first 3 shares...");
    let recovered = reconstruct_mnemonic(&shares[0..3], Language::English)?;
    println!("✅ Reconstructed");

    println!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("🔍 VERIFICATION");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n");

    println!("Original:    {}", mnemonic.phrase());
    println!("Recovered:   {}", recovered);

    if mnemonic.phrase() == recovered {
        println!("\n🎉 SUCCESS! Perfect reconstruction!");

        let original_seed = mnemonic.to_seed("");
        let recovered_mnemonic = Mnemonic::from_phrase(&recovered, Language::English)?;
        let recovered_seed = recovered_mnemonic.to_seed("");

        if original_seed.as_bytes() == recovered_seed.as_bytes() {
            println!("✨ Seeds match - cryptographically verified!");
        }
    } else {
        println!("\n❌ FAILED! Mnemonics don't match!");
        return Err("Reconstruction verification failed!".into());
    }

    println!("\n📊 Summary:");
    println!("  ✅ Mnemonic generated (PURE_BIP39)");
    println!("  ✅ Split into 5 shares (SHAMIR_SSS)");
    println!("  ✅ Saved to files");
    println!("  ✅ Reconstructed from 3 shares");
    println!("  ✅ Verification passed");

    Ok(())
}
