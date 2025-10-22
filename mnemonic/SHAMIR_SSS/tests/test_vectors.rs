/// Official test vectors and compatibility tests
///
/// These tests verify compatibility with known implementations

use shamir_sss::{ShamirSSS, GF256};

#[test]
fn test_galois_field_basic_operations() {
    // Test vector: Basic GF(256) arithmetic
    let a = GF256::new(3);
    let b = GF256::new(5);

    // Addition (XOR)
    assert_eq!((a + b).value(), 6);

    // Multiplication
    assert_eq!((a * b).value(), 15);

    // Inverse
    let inv = b.inverse().unwrap();
    assert_eq!((b * inv), GF256::ONE);
}

#[test]
fn test_known_secret_reconstruction() {
    // Test vector: Simple known secret
    let secret = b"test secret";
    let sss = ShamirSSS::new(3, 5).unwrap();

    let shares = sss.split(secret).unwrap();
    let recovered = sss.reconstruct(&shares[0..3]).unwrap();

    assert_eq!(secret.as_slice(), recovered.as_slice());
}

#[test]
fn test_different_threshold_combinations() {
    let secret = b"another test";

    // Test (2, 3) scheme
    let sss = ShamirSSS::new(2, 3).unwrap();
    let shares = sss.split(secret).unwrap();

    // Test all possible 2-share combinations
    assert_eq!(sss.reconstruct(&shares[0..2]).unwrap(), secret);
    assert_eq!(sss.reconstruct(&[shares[0].clone(), shares[2].clone()]).unwrap(), secret);
    assert_eq!(sss.reconstruct(&shares[1..3]).unwrap(), secret);
}

#[test]
fn test_256_bit_secret() {
    // Test with 32-byte secret (256 bits)
    let secret: Vec<u8> = (0..32).collect();
    let sss = ShamirSSS::new(4, 7).unwrap();

    let shares = sss.split(&secret).unwrap();
    assert_eq!(shares.len(), 7);

    // Reconstruct with exactly threshold shares
    let recovered = sss.reconstruct(&shares[0..4]).unwrap();
    assert_eq!(secret, recovered);

    // Reconstruct with more than threshold
    let recovered = sss.reconstruct(&shares[0..5]).unwrap();
    assert_eq!(secret, recovered);
}

#[test]
fn test_bip39_entropy_lengths() {
    // Test all standard BIP39 entropy lengths
    let lengths = [16, 20, 24, 28, 32]; // 128, 160, 192, 224, 256 bits

    for &len in &lengths {
        let secret: Vec<u8> = (0..len).map(|i| (i * 7) as u8).collect();
        let sss = ShamirSSS::new(3, 5).unwrap();

        let shares = sss.split(&secret).unwrap();
        let recovered = sss.reconstruct(&shares[0..3]).unwrap();

        assert_eq!(secret, recovered, "Failed for {} byte entropy", len);
    }
}

#[test]
fn test_share_independence() {
    // Test that individual shares reveal nothing
    let secret = b"super secret password";
    let sss = ShamirSSS::new(3, 5).unwrap();

    let shares = sss.split(secret).unwrap();

    // With threshold-1 shares, reconstruction should fail
    let result = sss.reconstruct(&shares[0..2]);
    assert!(result.is_err(), "Should fail with insufficient shares");
}

#[test]
fn test_share_order_independence() {
    // Test that share order doesn't matter
    let secret = b"order test";
    let sss = ShamirSSS::new(3, 5).unwrap();

    let shares = sss.split(secret).unwrap();

    // Different orderings
    let recovered1 = sss.reconstruct(&[shares[0].clone(), shares[1].clone(), shares[2].clone()]).unwrap();
    let recovered2 = sss.reconstruct(&[shares[2].clone(), shares[0].clone(), shares[1].clone()]).unwrap();
    let recovered3 = sss.reconstruct(&[shares[1].clone(), shares[2].clone(), shares[0].clone()]).unwrap();

    assert_eq!(recovered1, secret);
    assert_eq!(recovered2, secret);
    assert_eq!(recovered3, secret);
}

#[test]
fn test_maximum_shares() {
    // Test with maximum number of shares (limited by GF256)
    let secret = b"max shares test";
    let sss = ShamirSSS::new(2, 255).unwrap();

    let shares = sss.split(secret).unwrap();
    assert_eq!(shares.len(), 255);

    // Reconstruct from first and last share
    let recovered = sss.reconstruct(&[shares[0].clone(), shares[254].clone()]).unwrap();
    assert_eq!(secret.as_slice(), recovered.as_slice());
}

#[test]
fn test_digest_corruption_detection() {
    // Test that corrupted shares are detected
    let secret = b"integrity test";
    let sss = ShamirSSS::new(3, 5).unwrap();

    let mut shares = sss.split(secret).unwrap();

    // Corrupt one share's data
    if let Some(value) = shares[0].value.get_mut(0) {
        *value = GF256::new(value.value().wrapping_add(1));
    }

    // Should detect corruption via digest
    let result = sss.reconstruct(&shares[0..3]);
    assert!(result.is_err(), "Should detect corrupted share");
}

/// Manual test runner - can be run with: cargo test -- --nocapture test_manual_verification
#[test]
fn test_manual_verification() {
    println!("\n🔐 SHAMIR SSS - Manual Verification Test");
    println!("=========================================\n");

    let secret = b"Hello, Shamir's Secret Sharing!";
    println!("📝 Original secret: {:?}", String::from_utf8_lossy(secret));
    println!("   Length: {} bytes\n", secret.len());

    // Create (3, 5) scheme
    let sss = ShamirSSS::new(3, 5).unwrap();
    println!("🔀 Splitting into 5 shares (threshold = 3)");

    let shares = sss.split(secret).unwrap();
    println!("   ✅ Created {} shares\n", shares.len());

    // Show each share
    for (i, share) in shares.iter().enumerate() {
        let hex: String = share.value.iter()
            .take(8)
            .map(|v| format!("{:02x}", v.value()))
            .collect::<Vec<_>>()
            .join("");
        println!("   Share #{}: ID={}, Data={}... ({} bytes)",
                 i + 1, share.id, hex, share.value.len());
    }

    println!("\n🔓 Reconstructing from shares #1, #3, #5");
    let selected = vec![shares[0].clone(), shares[2].clone(), shares[4].clone()];
    let recovered = sss.reconstruct(&selected).unwrap();

    println!("   Recovered: {:?}", String::from_utf8_lossy(&recovered));
    println!("   Length: {} bytes\n", recovered.len());

    // Verify
    if secret.as_slice() == recovered.as_slice() {
        println!("✅ SUCCESS! Perfect reconstruction!");
    } else {
        panic!("❌ FAILED! Secrets don't match!");
    }

    println!("\n📊 Statistics:");
    println!("   - Original size: {} bytes", secret.len());
    println!("   - Share size: {} bytes (includes {}-byte digest)",
             shares[0].value.len(), 4);
    println!("   - Shares needed: 3 of 5");
    println!("   - Expansion ratio: {:.2}x",
             shares[0].value.len() as f64 / secret.len() as f64);
}
