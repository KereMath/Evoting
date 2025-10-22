//! Debug official test vector

use slip39::{Share, Slip39};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mnemonic = "duckling enlarge academic academic agency result length solution fridge kidney coal piece deal husband erode duke ajar critical decision keyboard";

    println!("Testing official vector 1:");
    println!("Mnemonic: {}\n", mnemonic);

    // Test different passphrase encodings
    println!("Passphrase tests:");
    println!("  'TREZOR' as bytes: {:?}", b"TREZOR");
    println!("  'TREZOR' UTF-8: {:?}", "TREZOR".as_bytes());
    println!();

    // Parse share
    println!("1. Parsing share...");
    let share = Share::from_mnemonic_string(mnemonic)?;

    println!("   ✅ Parsed successfully");
    println!("   Identifier: {}", share.identifier);
    println!("   Extendable: {}", share.extendable);
    println!("   Iteration exp: {}", share.iteration_exponent);
    println!("   Group index: {}", share.group_index);
    println!("   Group threshold: {}", share.group_threshold);
    println!("   Group count: {}", share.group_count);
    println!("   Member index: {}", share.member_index);
    println!("   Member threshold: {}", share.member_threshold);
    println!("   Value length: {} bytes", share.value.len());
    println!("   Value (hex): {}\n", hex::encode(&share.value));

    // Manual encryption test - encrypt expected secret
    println!("\n2. Forward test: Encrypt expected secret...");
    use slip39::FeistelCipher;
    use slip39::EncryptedSecret;

    let expected_master = hex::decode("bb54aac4b89dc868ba37d9cc21b2cece").unwrap();
    let cipher = FeistelCipher::new(b"TREZOR", share.identifier, share.extendable);

    println!("   Expected master:  {}", hex::encode(&expected_master));
    println!("   Share value:      {}", hex::encode(&share.value));

    match cipher.encrypt(&expected_master, share.iteration_exponent) {
        Ok(encrypted) => {
            println!("   Encrypted master: {}", hex::encode(&encrypted.data));
            if encrypted.data == share.value {
                println!("   ✅ MATCH! Encryption is correct!");
            } else {
                println!("   ❌ NO MATCH - encryption/decryption or parsing issue");
            }
        }
        Err(e) => {
            println!("   ❌ Encryption failed: {}", e);
        }
    }

    // Manual decryption test
    println!("\n3. Backward test: Decrypt share value...");
    let encrypted = EncryptedSecret::new(share.value.clone(), share.iteration_exponent)?;

    match cipher.decrypt(&encrypted) {
        Ok(decrypted) => {
            println!("   Decrypted (hex):  {}", hex::encode(&decrypted));
            println!("   Expected:         bb54aac4b89dc868ba37d9cc21b2cece");
            if decrypted == expected_master {
                println!("   ✅ MATCH!");
            } else {
                println!("   ❌ NO MATCH!");
            }
        }
        Err(e) => {
            println!("   ❌ Decryption failed: {}", e);
        }
    }

    // Try reconstruction
    println!("\n4. Full reconstruction with passphrase 'TREZOR'...");
    let result = Slip39::reconstruct_secret(&[share], b"TREZOR");

    match result {
        Ok(master) => {
            println!("   ✅ Reconstruction succeeded!");
            println!("   Master secret (hex): {}", hex::encode(&master.data));
            println!("   Expected:            bb54aac4b89dc868ba37d9cc21b2cece");
        }
        Err(e) => {
            println!("   ❌ Reconstruction failed: {}", e);
        }
    }

    Ok(())
}
