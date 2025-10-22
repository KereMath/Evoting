use slip39::{Share, Slip39};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Test vector 4: 2-of-3 sharing
    let mnemonics = vec![
        "shadow pistol academic always adequate wildlife fancy gross oasis cylinder mustang wrist rescue view short owner flip making coding armed",
        "shadow pistol academic acid actress prayer class unknown daughter sweater depict flip twice unkind craft early superior advocate guest smoking",
    ];

    let expected = "b43ceb7e57a0ea8766221624d01b0864";

    println!("Testing 2-of-3 reconstruction:");
    println!("Expected: {}", expected);

    // Parse shares
    let shares: Vec<Share> = mnemonics
        .iter()
        .map(|m| Share::from_mnemonic_string(m).unwrap())
        .collect();

    println!("\nShares parsed:");
    for (i, share) in shares.iter().enumerate() {
        println!("  Share {}: member_index={}, value_len={}", i+1, share.member_index, share.value.len());
        println!("    Value: {}", hex::encode(&share.value));
    }

    // Try reconstruction
    match Slip39::reconstruct_secret(&shares, b"TREZOR") {
        Ok(master) => {
            let result = hex::encode(&master.data);
            println!("\n✅ Reconstruction succeeded!");
            println!("  Result:   {}", result);
            println!("  Expected: {}", expected);
            println!("  Match: {}", result == expected);
        }
        Err(e) => {
            println!("\n❌ Reconstruction failed: {}", e);
        }
    }

    Ok(())
}
