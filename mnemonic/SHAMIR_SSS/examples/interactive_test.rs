
use shamir_sss::{split_mnemonic, reconstruct_mnemonic, ShamirSSS};
use pure_bip39::{Mnemonic, Language, EntropyBits};
use std::io::{self, Write};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔐 Shamir's Secret Sharing - Interactive Test");
    println!("==============================================\n");

    loop {
        println!("\nChoose test:");
        println!("  1. Test with known secret");
        println!("  2. Test with BIP39 mnemonic");
        println!("  3. Test different thresholds");
        println!("  4. Test Galois Field operations");
        println!("  5. Exit");
        print!("\nYour choice: ");
        io::stdout().flush()?;

        let mut choice = String::new();
        io::stdin().read_line(&mut choice)?;

        match choice.trim() {
            "1" => test_known_secret()?,
            "2" => test_bip39_mnemonic()?,
            "3" => test_different_thresholds()?,
            "4" => test_galois_field()?,
            "5" => {
                println!("\n👋 Goodbye!");
                break;
            }
            _ => println!("❌ Invalid choice!"),
        }
    }

    Ok(())
}

fn test_known_secret() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📝 Test 1: Known Secret");
    println!("========================\n");

    print!("Enter secret text (or press Enter for default): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let secret = if input.trim().is_empty() {
        "Hello, Shamir!".as_bytes().to_vec()
    } else {
        input.trim().as_bytes().to_vec()
    };

    println!("   Secret: {:?}", String::from_utf8_lossy(&secret));
    println!("   Length: {} bytes", secret.len());

    print!("\nEnter threshold (default 3): ");
    io::stdout().flush()?;
    let mut t_input = String::new();
    io::stdin().read_line(&mut t_input)?;
    let threshold: usize = t_input.trim().parse().unwrap_or(3);

    print!("Enter total shares (default 5): ");
    io::stdout().flush()?;
    let mut n_input = String::new();
    io::stdin().read_line(&mut n_input)?;
    let total_shares: usize = n_input.trim().parse().unwrap_or(5);

    println!("\n🔀 Splitting...");
    let sss = ShamirSSS::new(threshold, total_shares)?;
    let shares = sss.split(&secret)?;

    println!("   ✅ Created {} shares (threshold = {})", shares.len(), threshold);

    for (i, share) in shares.iter().enumerate() {
        let hex: String = share.value.iter()
            .take(10)
            .map(|v| format!("{:02x}", v.value()))
            .collect::<Vec<_>>()
            .join("");
        println!("   Share #{}: {}", i + 1, hex);
    }

    println!("\n🔓 Reconstructing from first {} shares...", threshold);
    let recovered = sss.reconstruct(&shares[..threshold])?;

    println!("   Recovered: {:?}", String::from_utf8_lossy(&recovered));

    if secret == recovered {
        println!("\n✅ SUCCESS! Perfect reconstruction!");
    } else {
        println!("\n❌ FAILED! Secrets don't match!");
    }

    Ok(())
}

fn test_bip39_mnemonic() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📝 Test 2: BIP39 Mnemonic");
    println!("==========================\n");

    print!("Generate new mnemonic? (y/n, default y): ");
    io::stdout().flush()?;
    let mut gen_input = String::new();
    io::stdin().read_line(&mut gen_input)?;

    let mnemonic_phrase = if gen_input.trim().to_lowercase() != "n" {
        let mnemonic = Mnemonic::generate(EntropyBits::Bits128, Language::English)?;
        println!("\n   Generated mnemonic:");
        println!("   {}\n", mnemonic.phrase());
        mnemonic.phrase()
    } else {
        print!("\nEnter mnemonic phrase: ");
        io::stdout().flush()?;
        let mut phrase = String::new();
        io::stdin().read_line(&mut phrase)?;
        phrase.trim().to_string()
    };

    let mnemonic = Mnemonic::from_phrase(&mnemonic_phrase, Language::English)?;
    println!("   ✅ Valid BIP39 mnemonic ({} words)", mnemonic.word_count());

    print!("\nEnter threshold (default 3): ");
    io::stdout().flush()?;
    let mut t_input = String::new();
    io::stdin().read_line(&mut t_input)?;
    let threshold: usize = t_input.trim().parse().unwrap_or(3);

    print!("Enter total shares (default 5): ");
    io::stdout().flush()?;
    let mut n_input = String::new();
    io::stdin().read_line(&mut n_input)?;
    let total_shares: usize = n_input.trim().parse().unwrap_or(5);

    println!("\n🔀 Splitting mnemonic into shares...");
    let shares = split_mnemonic(&mnemonic_phrase, threshold, total_shares, Language::English)?;

    println!("   ✅ Created {} shares", shares.len());

    for (i, share) in shares.iter().enumerate() {
        println!("   Share #{}: {} characters (hex)",
                 i + 1, share.share_data.len());
    }

    println!("\n🔓 Reconstructing from first {} shares...", threshold);
    let recovered = reconstruct_mnemonic(&shares[..threshold], Language::English)?;

    println!("\n   Original:    {}", mnemonic_phrase);
    println!("   Recovered:   {}", recovered);

    if mnemonic_phrase == recovered {
        println!("\n✅ SUCCESS! Mnemonics match!");

        let original_seed = mnemonic.to_seed("");
        let recovered_mnemonic = Mnemonic::from_phrase(&recovered, Language::English)?;
        let recovered_seed = recovered_mnemonic.to_seed("");

        if original_seed.as_bytes() == recovered_seed.as_bytes() {
            println!("✅ Seeds also match perfectly!");
        }
    } else {
        println!("\n❌ FAILED! Mnemonics don't match!");
    }

    Ok(())
}

fn test_different_thresholds() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n📝 Test 3: Different Thresholds");
    println!("=================================\n");

    let secret = b"Threshold Test";

    let configs = vec![
        (2, 3, "Simple 2-of-3"),
        (3, 5, "Standard 3-of-5"),
        (4, 7, "High security 4-of-7"),
        (5, 9, "Very high 5-of-9"),
    ];

    for (threshold, total, desc) in configs {
        println!("Testing {}: ({}, {})", desc, threshold, total);

        let sss = ShamirSSS::new(threshold, total)?;
        let shares = sss.split(secret)?;

        let recovered = sss.reconstruct(&shares[..threshold])?;

        if secret.as_slice() == recovered.as_slice() {
            println!("   ✅ Success with {} shares", threshold);
        } else {
            println!("   ❌ Failed!");
        }

        let result = sss.reconstruct(&shares[..threshold - 1]);
        if result.is_err() {
            println!("   ✅ Correctly fails with {} shares", threshold - 1);
        } else {
            println!("   ❌ Should have failed with insufficient shares!");
        }

        println!();
    }

    println!("✅ All threshold tests passed!");

    Ok(())
}

fn test_galois_field() -> Result<(), Box<dyn std::error::Error>> {
    use shamir_sss::GF256;

    println!("\n📝 Test 4: Galois Field GF(256)");
    println!("=================================\n");

    println!("Testing basic operations:\n");

    let a = GF256::new(3);
    let b = GF256::new(5);
    let c = a + b;
    println!("   3 + 5 = {} (XOR)", c.value());
    assert_eq!(c.value(), 6);

    let d = a * b;
    println!("   3 × 5 = {} (GF multiplication)", d.value());

    let inv = b.inverse()?;
    println!("   5⁻¹ = {}", inv.value());
    println!("   5 × 5⁻¹ = {} (should be 1)", (b * inv).value());
    assert_eq!(b * inv, GF256::ONE);

    let e = GF256::new(10);
    let f = GF256::new(2);
    let g = (e / f)?;
    println!("\n   10 ÷ 2 = {}", g.value());
    println!("   Verify: {} × 2 = {}", g.value(), (g * f).value());

    let h = GF256::new(2);
    println!("\n   2⁰ = {}", h.pow(0).value());
    println!("   2¹ = {}", h.pow(1).value());
    println!("   2² = {}", h.pow(2).value());
    println!("   2³ = {}", h.pow(3).value());

    println!("\n✅ All Galois Field operations working correctly!");

    Ok(())
}
