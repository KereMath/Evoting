//! Multi-Group SLIP-39 Demo
//!
//! Demonstrates advanced group-based secret sharing

use slip39::{GroupConfig, MasterSecret, Share, Slip39};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════");
    println!("  SLIP-39 Multi-Group Demo");
    println!("═══════════════════════════════════════════════\n");

    // Generate a 256-bit master secret
    println!("📝 Generating 256-bit master secret...");
    let master_secret = MasterSecret::generate(256)?;
    println!("   Secret (hex): {}...\n", &hex::encode(&master_secret.data)[..32]);

    // Define group structure
    println!("🏢 Creating 2-of-3 Group Scheme:");
    println!("   ├─ Group 0 (Family):  2-of-3 shares");
    println!("   ├─ Group 1 (Friends): 2-of-5 shares");
    println!("   └─ Group 2 (Backup):  3-of-5 shares");
    println!("   (Need ANY 2 groups to recover)\n");

    let groups = vec![
        GroupConfig::new(2, 3)?, // Family: 2-of-3
        GroupConfig::new(2, 5)?, // Friends: 2-of-5
        GroupConfig::new(3, 5)?, // Backup: 3-of-5
    ];

    let slip39 = Slip39::new(2, groups, Some(0))?; // Need 2 groups, low iteration

    // Generate shares
    let passphrase = b"super secret master password";
    println!("🔐 Generating shares...\n");

    let shares = slip39.generate_shares(&master_secret, passphrase)?;

    // Display shares by group
    let group_names = ["Family", "Friends", "Backup"];

    for (group_idx, group_shares) in shares.iter().enumerate() {
        println!("───────────────────────────────────────────────");
        println!("📦 Group {}: {} ({} shares)", group_idx, group_names[group_idx], group_shares.len());
        println!("───────────────────────────────────────────────\n");

        for (member_idx, share) in group_shares.iter().enumerate() {
            let mnemonic = share.to_mnemonic_string()?;
            let words = mnemonic.split_whitespace().collect::<Vec<_>>();

            println!("  Member {}: ({} words)", member_idx + 1, words.len());
            println!("  {}", format_mnemonic_short(&mnemonic));
            println!();
        }
    }

    // Scenario 1: Use Family + Friends groups
    println!("═══════════════════════════════════════════════");
    println!("🔓 Scenario 1: Reconstruct with Family + Friends");
    println!("═══════════════════════════════════════════════\n");

    let mut reconstruction1: Vec<Share> = Vec::new();

    // Take 2 shares from Family (group 0)
    reconstruction1.extend_from_slice(&shares[0][0..2]);
    println!("   ✓ Added 2 shares from Family group");

    // Take 2 shares from Friends (group 1)
    reconstruction1.extend_from_slice(&shares[1][0..2]);
    println!("   ✓ Added 2 shares from Friends group\n");

    let result1 = Slip39::reconstruct_secret(&reconstruction1, passphrase)?;

    if result1.data == master_secret.data {
        println!("   ✅ Success! Secret recovered correctly\n");
    } else {
        println!("   ❌ Error! Reconstruction failed\n");
    }

    // Scenario 2: Use Friends + Backup groups
    println!("═══════════════════════════════════════════════");
    println!("🔓 Scenario 2: Reconstruct with Friends + Backup");
    println!("═══════════════════════════════════════════════\n");

    let mut reconstruction2: Vec<Share> = Vec::new();

    // Take 3 shares from Friends (group 1)
    reconstruction2.extend_from_slice(&shares[1][1..4]);
    println!("   ✓ Added 3 shares from Friends group");

    // Take 3 shares from Backup (group 2)
    reconstruction2.extend_from_slice(&shares[2][0..3]);
    println!("   ✓ Added 3 shares from Backup group\n");

    let result2 = Slip39::reconstruct_secret(&reconstruction2, passphrase)?;

    if result2.data == master_secret.data {
        println!("   ✅ Success! Secret recovered correctly\n");
    } else {
        println!("   ❌ Error! Reconstruction failed\n");
    }

    // Scenario 3: Insufficient - only 1 group
    println!("═══════════════════════════════════════════════");
    println!("⚠️  Scenario 3: Try with only 1 group (should fail)");
    println!("═══════════════════════════════════════════════\n");

    let insufficient: Vec<Share> = shares[0].clone(); // All of family group
    println!("   Using all {} shares from Family group only\n", insufficient.len());

    match Slip39::reconstruct_secret(&insufficient, passphrase) {
        Ok(_) => println!("   ❌ Unexpected success!"),
        Err(e) => println!("   ✅ Correctly rejected: {}\n", e),
    }

    println!("═══════════════════════════════════════════════");
    println!("  Multi-Group Demo Complete!");
    println!("═══════════════════════════════════════════════");
    println!("\n💡 Key Insight:");
    println!("   With this scheme, you can distribute shares to");
    println!("   different trusted parties (family, friends, backup)");
    println!("   and require cooperation between groups to recover");
    println!("   the secret, providing flexible security!\n");

    Ok(())
}

fn format_mnemonic_short(mnemonic: &str) -> String {
    let words: Vec<&str> = mnemonic.split_whitespace().collect();
    if words.len() > 6 {
        format!("    {} {} {} ... {} {} {}",
            words[0], words[1], words[2],
            words[words.len()-3], words[words.len()-2], words[words.len()-1]
        )
    } else {
        format!("    {}", mnemonic)
    }
}
