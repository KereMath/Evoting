//! Basic SLIP-39 Demo
//!
//! Demonstrates simple single-group share generation and reconstruction

use slip39::{MasterSecret, Share, Slip39};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════");
    println!("  SLIP-39 Basic Demo");
    println!("═══════════════════════════════════════════════\n");

    // Generate a random 128-bit master secret
    println!("📝 Generating 128-bit master secret...");
    let master_secret = MasterSecret::generate(128)?;
    println!("   Secret (hex): {}\n", hex::encode(&master_secret.data));

    // Create 3-of-5 share configuration
    let threshold = 3;
    let share_count = 5;
    println!("🔒 Creating {}-of-{} share scheme", threshold, share_count);
    println!("   (Need any {} shares to recover)\n", threshold);

    let slip39 = Slip39::new_single_group(threshold, share_count)?;

    // Generate shares with a passphrase
    let passphrase = b"my secure passphrase";
    println!("🔐 Generating shares with passphrase...\n");

    let shares = slip39.generate_shares(&master_secret, passphrase)?;

    // Display all shares
    println!("📋 Generated Shares:");
    println!("───────────────────────────────────────────────\n");

    for (i, share) in shares[0].iter().enumerate() {
        let mnemonic = share.to_mnemonic_string()?;
        let word_count = mnemonic.split_whitespace().count();
        println!("Share {}/{}:", i + 1, share_count);
        println!("  Words: {}", word_count);
        println!("  Mnemonic: {}\n", mnemonic);
    }

    // Reconstruct from subset
    println!("───────────────────────────────────────────────");
    println!("🔓 Reconstructing from shares 1, 3, and 5...\n");

    let reconstruction_shares: Vec<Share> = vec![
        shares[0][0].clone(),
        shares[0][2].clone(),
        shares[0][4].clone(),
    ];

    let reconstructed = Slip39::reconstruct_secret(&reconstruction_shares, passphrase)?;

    println!("   Reconstructed (hex): {}", hex::encode(&reconstructed.data));

    // Verify
    if reconstructed.data == master_secret.data {
        println!("   ✅ Success! Secrets match!\n");
    } else {
        println!("   ❌ Error! Secrets don't match!\n");
    }

    // Test with insufficient shares
    println!("───────────────────────────────────────────────");
    println!("⚠️  Testing with only 2 shares (insufficient)...\n");

    let insufficient: Vec<Share> = vec![shares[0][0].clone(), shares[0][1].clone()];

    match Slip39::reconstruct_secret(&insufficient, passphrase) {
        Ok(result) => {
            if result.data != master_secret.data {
                println!("   ⚠️  Reconstruction produced wrong result (as expected)");
                println!("   This is normal - insufficient shares can't recover the secret\n");
            }
        }
        Err(e) => {
            println!("   ⚠️  Reconstruction failed: {}", e);
            println!("   This is expected behavior with insufficient shares\n");
        }
    }

    // Test with wrong passphrase
    println!("───────────────────────────────────────────────");
    println!("⚠️  Testing with wrong passphrase...\n");

    let wrong_passphrase = b"wrong password";
    match Slip39::reconstruct_secret(&reconstruction_shares, wrong_passphrase) {
        Ok(_) => println!("   ❌ Unexpected success!"),
        Err(e) => println!("   ✅ Correctly rejected: {}\n", e),
    }

    println!("═══════════════════════════════════════════════");
    println!("  Demo Complete!");
    println!("═══════════════════════════════════════════════");

    Ok(())
}
