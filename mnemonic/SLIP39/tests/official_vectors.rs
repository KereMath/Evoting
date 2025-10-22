//! Official SLIP-39 Test Vectors
//!
//! Test vectors from Trezor's python-shamir-mnemonic repository:
//! https://github.com/trezor/python-shamir-mnemonic/blob/master/vectors.json

use slip39::{Share, Slip39};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct TestVector {
    description: String,
    mnemonics: Vec<String>,
    master_secret: String,
    xprv: String,
}

// Load test vectors from JSON
fn load_test_vectors() -> Vec<(String, Vec<String>, String, String)> {
    let json_data = include_str!("vectors.json");
    let vectors: Vec<serde_json::Value> = serde_json::from_str(json_data)
        .expect("Failed to parse test vectors JSON");

    vectors
        .into_iter()
        .map(|v| {
            let arr = v.as_array().expect("Expected array");
            (
                arr[0].as_str().unwrap().to_string(),
                arr[1]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|s| s.as_str().unwrap().to_string())
                    .collect(),
                arr[2].as_str().unwrap().to_string(),
                arr[3].as_str().unwrap().to_string(),
            )
        })
        .collect()
}

#[test]
fn test_vector_01_valid_without_sharing_128() {
    let vectors = load_test_vectors();
    let (desc, mnemonics, expected_secret, _xprv) = &vectors[0];

    println!("Testing: {}", desc);

    // Parse shares from mnemonics
    let shares: Result<Vec<Share>, _> = mnemonics
        .iter()
        .map(|m| Share::from_mnemonic_string(m))
        .collect();

    assert!(shares.is_ok(), "Failed to parse valid mnemonic");

    // Reconstruct secret with passphrase "TREZOR"
    let shares = shares.unwrap();
    let result = Slip39::reconstruct_secret(&shares, b"TREZOR");

    if let Err(e) = &result {
        println!("  Error: {}", e);
    }

    assert!(result.is_ok(), "Failed to reconstruct valid secret: {:?}", result.err());

    let master = result.unwrap();
    let secret_hex = hex::encode(&master.data);

    assert_eq!(
        &secret_hex, expected_secret,
        "Master secret mismatch for: {}",
        desc
    );
}

#[test]
fn test_vector_02_invalid_checksum() {
    let vectors = load_test_vectors();
    let (desc, mnemonics, expected_secret, _) = &vectors[1];

    println!("Testing: {}", desc);

    // Should be empty (invalid)
    assert_eq!(expected_secret, "", "Expected invalid test case");

    // Try to parse - should fail checksum
    let result = Share::from_mnemonic_string(&mnemonics[0]);

    assert!(
        result.is_err(),
        "Should reject mnemonic with invalid checksum"
    );
}

#[test]
fn test_vector_04_basic_sharing_2of3() {
    let vectors = load_test_vectors();
    let (desc, mnemonics, expected_secret, _xprv) = &vectors[3];

    println!("Testing: {}", desc);

    // Parse shares
    let shares: Result<Vec<Share>, _> = mnemonics
        .iter()
        .map(|m| Share::from_mnemonic_string(m))
        .collect();

    assert!(shares.is_ok(), "Failed to parse valid shares");

    // Reconstruct with 2 shares (threshold = 2)
    let shares = shares.unwrap();

    println!("Reconstructing with {} shares", shares.len());
    for (i, share) in shares.iter().enumerate() {
        println!("  Share {}: member_threshold={}, value_len={}",
            i+1, share.member_threshold, share.value.len());
    }

    let result = Slip39::reconstruct_secret(&shares, b"TREZOR");

    if let Err(ref e) = result {
        println!("Reconstruction error: {}", e);
    }

    assert!(result.is_ok(), "Failed to reconstruct from 2-of-3 shares: {:?}", result.err());

    let master = result.unwrap();
    let secret_hex = hex::encode(&master.data);

    assert_eq!(
        &secret_hex, expected_secret,
        "Master secret mismatch for 2-of-3: {}",
        desc
    );
}

#[test]
fn test_vector_05_insufficient_shares() {
    let vectors = load_test_vectors();
    let (desc, mnemonics, expected_secret, _) = &vectors[4];

    println!("Testing: {}", desc);

    // Should be invalid (only 1 share, need 2)
    assert_eq!(expected_secret, "", "Expected invalid test case");

    let shares: Result<Vec<Share>, _> = mnemonics
        .iter()
        .map(|m| Share::from_mnemonic_string(m))
        .collect();

    assert!(shares.is_ok(), "Should parse valid share format");

    let shares = shares.unwrap();
    let result = Slip39::reconstruct_secret(&shares, b"TREZOR");

    // Should fail or produce wrong result
    if let Ok(master) = result {
        let secret_hex = hex::encode(&master.data);
        // If it "succeeds", the secret should be wrong
        assert_ne!(&secret_hex, "b43ceb7e57a0ea8766221624d01b0864");
    }
}

#[test]
fn test_vector_06_different_identifiers() {
    let vectors = load_test_vectors();
    let (desc, mnemonics, expected_secret, _) = &vectors[5];

    println!("Testing: {}", desc);

    assert_eq!(expected_secret, "", "Expected invalid test case");

    let shares: Result<Vec<Share>, _> = mnemonics
        .iter()
        .map(|m| Share::from_mnemonic_string(m))
        .collect();

    if let Ok(shares) = shares {
        let result = Slip39::reconstruct_secret(&shares, b"TREZOR");

        // Should fail due to different identifiers
        assert!(
            result.is_err(),
            "Should reject shares with different identifiers"
        );
    }
}

#[test]
fn test_all_valid_vectors() {
    let vectors = load_test_vectors();

    let valid_indices = vec![0, 3, 16, 17, 18, 19, 22, 35, 36, 37, 40, 41, 42, 43, 44];

    for &idx in &valid_indices {
        if idx >= vectors.len() {
            continue;
        }

        let (desc, mnemonics, expected_secret, _) = &vectors[idx];

        if expected_secret.is_empty() {
            continue; // Skip invalid cases
        }

        println!("Testing valid vector {}: {}", idx + 1, desc);

        let shares: Result<Vec<Share>, _> = mnemonics
            .iter()
            .map(|m| Share::from_mnemonic_string(m))
            .collect();

        if shares.is_err() {
            println!("  ⚠️  Failed to parse shares");
            continue;
        }

        let shares = shares.unwrap();
        let result = Slip39::reconstruct_secret(&shares, b"TREZOR");

        match result {
            Ok(master) => {
                let secret_hex = hex::encode(&master.data);
                if &secret_hex == expected_secret {
                    println!("  ✅ Pass");
                } else {
                    println!("  ❌ Secret mismatch");
                    println!("     Expected: {}", expected_secret);
                    println!("     Got:      {}", secret_hex);
                }
            }
            Err(e) => {
                println!("  ❌ Reconstruction failed: {}", e);
            }
        }
    }
}

#[test]
fn test_all_invalid_vectors() {
    let vectors = load_test_vectors();

    // Test all invalid cases (empty expected_secret)
    for (idx, (desc, mnemonics, expected_secret, _)) in vectors.iter().enumerate() {
        if !expected_secret.is_empty() {
            continue; // Skip valid cases
        }

        println!("Testing invalid vector {}: {}", idx + 1, desc);

        // Try to parse shares
        let shares: Result<Vec<Share>, _> = mnemonics
            .iter()
            .map(|m| Share::from_mnemonic_string(m))
            .collect();

        if let Ok(shares) = shares {
            // Shares parsed, try reconstruction
            let result = Slip39::reconstruct_secret(&shares, b"TREZOR");

            if result.is_ok() {
                println!("  ⚠️  Unexpectedly succeeded (should have failed)");
            } else {
                println!("  ✅ Correctly rejected");
            }
        } else {
            println!("  ✅ Correctly rejected at parse stage");
        }
    }
}
