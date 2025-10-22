
use shamir_sss::{split_mnemonic, reconstruct_mnemonic, MnemonicShare};
use pure_bip39::{Mnemonic, Language, EntropyBits};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔐 Shamir's Secret Sharing - Basic Demo");
    println!("=========================================\n");

    println!("📝 Step 1: Generate BIP39 Mnemonic");
    let mnemonic = Mnemonic::generate(EntropyBits::Bits256, Language::English)?;
    println!("   Mnemonic (24 words):");
    println!("   {}\n", mnemonic.phrase());

    println!("🔀 Step 2: Split into Shamir Shares");
    println!("   Configuration: 5 shares, threshold = 3");
    println!("   (Any 3 shares can recover the mnemonic)\n");

    let shares = split_mnemonic(
        &mnemonic.phrase(),
        3,
        5,
        Language::English
    )?;

    println!("   Generated {} shares:", shares.len());
    for (i, share) in shares.iter().enumerate() {
        println!("   Share #{}: {} bytes", i + 1, share.share_data.len() / 2);
    }
    println!();

    println!("💾 Step 3: Save Shares to JSON");
    for (i, share) in shares.iter().enumerate() {
        let json = share.to_json()?;
        println!("   Share #{} JSON:", i + 1);
        println!("{}", json);
        println!();
    }

    println!("🔓 Step 4: Reconstruct Mnemonic");
    println!("   Using shares #1, #3, #5 (any 3 would work)");

    let selected_shares = vec![
        shares[0].clone(),
        shares[2].clone(),
        shares[4].clone(),
    ];

    let reconstructed = reconstruct_mnemonic(&selected_shares, Language::English)?;
    println!("   Reconstructed:");
    println!("   {}\n", reconstructed);

    println!("✅ Step 5: Verify");
    if mnemonic.phrase() == reconstructed {
        println!("   SUCCESS! Original and reconstructed mnemonics match! 🎉");

        let original_seed = mnemonic.to_seed("");
        let reconstructed_mnemonic = Mnemonic::from_phrase(&reconstructed, Language::English)?;
        let reconstructed_seed = reconstructed_mnemonic.to_seed("");

        if original_seed.as_bytes() == reconstructed_seed.as_bytes() {
            println!("   Seeds also match - perfect reconstruction! ✨");
        }
    } else {
        println!("   ERROR: Mnemonics don't match!");
        return Err("Reconstruction failed".into());
    }

    println!("\n📊 Summary:");
    println!("   - Original mnemonic: {} words", mnemonic.word_count());
    println!("   - Total shares created: {}", shares.len());
    println!("   - Threshold: 3 shares minimum");
    println!("   - Shares used for recovery: 3");
    println!("   - Result: Perfect reconstruction! ✅");

    Ok(())
}
