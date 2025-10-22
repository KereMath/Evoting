use wasm_bindgen::prelude::*;
use pure_bip39::{Mnemonic, EntropyBits, Language};
use sha2::{Sha512, Digest};
use serde::{Deserialize, Serialize};
use curve25519_dalek::scalar::Scalar;
use curve25519_dalek::ristretto::RistrettoPoint;
use curve25519_dalek::constants::RISTRETTO_BASEPOINT_POINT;
use getrandom::getrandom;

#[derive(Serialize, Deserialize)]
pub struct DIDResult {
    pub mnemonic: String,
    pub did: String,
}

#[derive(Serialize, Deserialize)]
pub struct DIDRecovery {
    pub did: String,
    pub valid: bool,
}

/// Generate a new DID with 12-word BIP39 mnemonic (128-bit entropy)
/// Returns: { mnemonic: "word1 word2 ...", did: "sha512_hash" }
#[wasm_bindgen]
pub fn generate_did() -> Result<JsValue, JsValue> {
    // Generate 12-word mnemonic (128-bit entropy)
    let mnemonic = Mnemonic::generate(EntropyBits::Bits128, Language::English)
        .map_err(|e| JsValue::from_str(&format!("Failed to generate mnemonic: {}", e)))?;

    let mnemonic_phrase = mnemonic.phrase();

    // Calculate DID as SHA-512 hash of mnemonic
    let did = calculate_did(&mnemonic_phrase);

    let result = DIDResult {
        mnemonic: mnemonic_phrase,
        did,
    };

    serde_wasm_bindgen::to_value(&result)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Recover DID from existing 12-word mnemonic
/// Input: "word1 word2 word3 ... word12"
/// Returns: { did: "sha512_hash", valid: true/false }
#[wasm_bindgen]
pub fn recover_did(mnemonic_phrase: &str) -> Result<JsValue, JsValue> {
    // Validate mnemonic
    let is_valid = Mnemonic::validate(mnemonic_phrase, Language::English);

    if !is_valid {
        let result = DIDRecovery {
            did: String::new(),
            valid: false,
        };
        return serde_wasm_bindgen::to_value(&result)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)));
    }

    // Calculate DID
    let did = calculate_did(mnemonic_phrase);

    let result = DIDRecovery {
        did,
        valid: true,
    };

    serde_wasm_bindgen::to_value(&result)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Calculate DID as SHA-512(mnemonic_phrase)
fn calculate_did(mnemonic_phrase: &str) -> String {
    let mut hasher = Sha512::new();
    hasher.update(mnemonic_phrase.as_bytes());
    let hash = hasher.finalize();
    hex::encode(hash)
}

/// Generate o-value with 12-word BIP39 mnemonic (for blind signature blinding)
/// Returns: { mnemonic: "word1 word2 ...", o_value: "sha512_hash" }
#[wasm_bindgen]
pub fn generate_o_value() -> Result<JsValue, JsValue> {
    // Generate 12-word mnemonic (128-bit entropy)
    let mnemonic = Mnemonic::generate(EntropyBits::Bits128, Language::English)
        .map_err(|e| JsValue::from_str(&format!("Failed to generate mnemonic: {}", e)))?;

    let mnemonic_phrase = mnemonic.phrase();

    // Calculate o-value as SHA-512 hash of mnemonic
    let o_value = calculate_o_value(&mnemonic_phrase);

    let result = DIDResult {
        mnemonic: mnemonic_phrase,
        did: o_value,  // Reuse DIDResult struct (did field holds o_value)
    };

    serde_wasm_bindgen::to_value(&result)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Recover o-value from existing 12-word mnemonic
/// Input: "word1 word2 word3 ... word12"
/// Returns: { did: "sha512_hash", valid: true/false }
#[wasm_bindgen]
pub fn recover_o_value(mnemonic_phrase: &str) -> Result<JsValue, JsValue> {
    // Validate mnemonic
    let is_valid = Mnemonic::validate(mnemonic_phrase, Language::English);

    if !is_valid {
        let result = DIDRecovery {
            did: String::new(),
            valid: false,
        };
        return serde_wasm_bindgen::to_value(&result)
            .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)));
    }

    // Calculate o-value
    let o_value = calculate_o_value(mnemonic_phrase);

    let result = DIDRecovery {
        did: o_value,  // Reuse DIDRecovery struct (did field holds o_value)
        valid: true,
    };

    serde_wasm_bindgen::to_value(&result)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}

/// Calculate o-value as SHA-512(mnemonic_phrase)
fn calculate_o_value(mnemonic_phrase: &str) -> String {
    let mut hasher = Sha512::new();
    hasher.update(mnemonic_phrase.as_bytes());
    let hash = hasher.finalize();
    hex::encode(hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    // BIP39 official test vectors from https://github.com/trezor/python-mnemonic/blob/master/vectors.json
    // Covers all entropy sizes: 128, 160, 192, 224, 256 bits
    const TEST_VECTORS_128: &[(&str, &str)] = &[
        // 128-bit (12 words)
        (
            "00000000000000000000000000000000",
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
        ),
        (
            "7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f",
            "legal winner thank year wave sausage worth useful legal winner thank yellow"
        ),
        (
            "80808080808080808080808080808080",
            "letter advice cage absurd amount doctor acoustic avoid letter advice cage above"
        ),
        (
            "ffffffffffffffffffffffffffffffff",
            "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong"
        ),
    ];

    const TEST_VECTORS_160: &[(&str, &str)] = &[
        // 160-bit (15 words) - 20 bytes = 40 hex chars
        (
            "0000000000000000000000000000000000000000",
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon address"
        ),
        (
            "7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f",
            "legal winner thank year wave sausage worth useful legal winner thank year wave sausage wise"
        ),
        (
            "8080808080808080808080808080808080808080",
            "letter advice cage absurd amount doctor acoustic avoid letter advice cage absurd amount doctor accident"
        ),
        (
            "ffffffffffffffffffffffffffffffffffffffff",
            "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrist"
        ),
    ];

    const TEST_VECTORS_192: &[(&str, &str)] = &[
        // 192-bit (18 words)
        (
            "000000000000000000000000000000000000000000000000",
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon agent"
        ),
        (
            "7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f",
            "legal winner thank year wave sausage worth useful legal winner thank year wave sausage worth useful legal will"
        ),
        (
            "808080808080808080808080808080808080808080808080",
            "letter advice cage absurd amount doctor acoustic avoid letter advice cage absurd amount doctor acoustic avoid letter always"
        ),
        (
            "ffffffffffffffffffffffffffffffffffffffffffffffff",
            "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo when"
        ),
    ];

    const TEST_VECTORS_224: &[(&str, &str)] = &[
        // 224-bit (21 words) - 28 bytes = 56 hex chars
        // Note: 224-bit is rarely used, only testing boundary cases
        (
            "00000000000000000000000000000000000000000000000000000000",
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon admit"
        ),
        (
            "ffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo veteran"
        ),
    ];

    const TEST_VECTORS_256: &[(&str, &str)] = &[
        // 256-bit (24 words)
        (
            "0000000000000000000000000000000000000000000000000000000000000000",
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art"
        ),
        (
            "7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f",
            "legal winner thank year wave sausage worth useful legal winner thank year wave sausage worth useful legal winner thank year wave sausage worth title"
        ),
        (
            "8080808080808080808080808080808080808080808080808080808080808080",
            "letter advice cage absurd amount doctor acoustic avoid letter advice cage absurd amount doctor acoustic avoid letter advice cage absurd amount doctor acoustic bless"
        ),
        (
            "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
            "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo vote"
        ),
    ];

    // Additional edge case vectors
    const TEST_VECTORS_EDGE: &[(&str, &str)] = &[
        // Pattern tests
        (
            "9e885d952ad362caeb4efe34a8e91bd2",
            "ozone drill grab fiber curtain grace pudding thank cruise elder eight picnic"
        ),
        (
            "6610b25967cdcca9d59875f5cb50b0ea75433311869e930b",
            "gravity machine north sort system female filter attitude volume fold club stay feature office ecology stable narrow fog"
        ),
        (
            "68a79eaca2324873eacc50cb9c6eca8cc68ea5d936f98787c60c7ebc74e6ce7c",
            "hamster diagram private dutch cause delay private meat slide toddler razor book happy fancy gospel tennis maple dilemma loan word shrug inflict delay length"
        ),
        (
            "c0ba5a8e914111210f2bd131f3d5e08d",
            "scheme spot photo card baby mountain device kick cradle pact join borrow"
        ),
        (
            "6d9be1ee6ebd27a258115aad99b7317b9c8d28b6d76431c3",
            "horn tenant knee talent sponsor spell gate clip pulse soap slush warm silver nephew swap uncle crack brave"
        ),
    ];

    #[test]
    fn test_bip39_vectors_128bit() {
        use pure_bip39::Entropy;
        println!("\n🔬 Testing 128-bit (12-word) BIP39 vectors...");

        for &(entropy_hex, expected_mnemonic) in TEST_VECTORS_128 {
            let entropy = Entropy::from_hex(entropy_hex).unwrap();
            let mnemonic = Mnemonic::from_entropy(entropy, Language::English).unwrap();
            assert_eq!(mnemonic.phrase(), expected_mnemonic,
                "BIP39 128-bit test vector failed for entropy: {}", entropy_hex);
        }
        println!("✅ All {} 128-bit vectors passed!", TEST_VECTORS_128.len());
    }

    #[test]
    fn test_bip39_vectors_160bit() {
        use pure_bip39::Entropy;
        println!("\n🔬 Testing 160-bit (15-word) BIP39 vectors...");

        for &(entropy_hex, expected_mnemonic) in TEST_VECTORS_160 {
            let entropy = Entropy::from_hex(entropy_hex).unwrap();
            let mnemonic = Mnemonic::from_entropy(entropy, Language::English).unwrap();
            assert_eq!(mnemonic.phrase(), expected_mnemonic,
                "BIP39 160-bit test vector failed for entropy: {}", entropy_hex);
        }
        println!("✅ All {} 160-bit vectors passed!", TEST_VECTORS_160.len());
    }

    #[test]
    fn test_bip39_vectors_192bit() {
        use pure_bip39::Entropy;
        println!("\n🔬 Testing 192-bit (18-word) BIP39 vectors...");

        for &(entropy_hex, expected_mnemonic) in TEST_VECTORS_192 {
            let entropy = Entropy::from_hex(entropy_hex).unwrap();
            let mnemonic = Mnemonic::from_entropy(entropy, Language::English).unwrap();
            assert_eq!(mnemonic.phrase(), expected_mnemonic,
                "BIP39 192-bit test vector failed for entropy: {}", entropy_hex);
        }
        println!("✅ All {} 192-bit vectors passed!", TEST_VECTORS_192.len());
    }

    #[test]
    fn test_bip39_vectors_224bit() {
        use pure_bip39::Entropy;
        println!("\n🔬 Testing 224-bit (21-word) BIP39 vectors...");

        for &(entropy_hex, expected_mnemonic) in TEST_VECTORS_224 {
            let entropy = Entropy::from_hex(entropy_hex).unwrap();
            let mnemonic = Mnemonic::from_entropy(entropy, Language::English).unwrap();
            assert_eq!(mnemonic.phrase(), expected_mnemonic,
                "BIP39 224-bit test vector failed for entropy: {}", entropy_hex);
        }
        println!("✅ All {} 224-bit vectors passed!", TEST_VECTORS_224.len());
    }

    #[test]
    fn test_bip39_vectors_256bit() {
        use pure_bip39::Entropy;
        println!("\n🔬 Testing 256-bit (24-word) BIP39 vectors...");

        for &(entropy_hex, expected_mnemonic) in TEST_VECTORS_256 {
            let entropy = Entropy::from_hex(entropy_hex).unwrap();
            let mnemonic = Mnemonic::from_entropy(entropy, Language::English).unwrap();
            assert_eq!(mnemonic.phrase(), expected_mnemonic,
                "BIP39 256-bit test vector failed for entropy: {}", entropy_hex);
        }
        println!("✅ All {} 256-bit vectors passed!", TEST_VECTORS_256.len());
    }

    #[test]
    fn test_bip39_vectors_edge_cases() {
        use pure_bip39::Entropy;
        println!("\n🔬 Testing edge case BIP39 vectors...");

        for &(entropy_hex, expected_mnemonic) in TEST_VECTORS_EDGE {
            let entropy = Entropy::from_hex(entropy_hex).unwrap();
            let mnemonic = Mnemonic::from_entropy(entropy, Language::English).unwrap();
            assert_eq!(mnemonic.phrase(), expected_mnemonic,
                "BIP39 edge case test vector failed for entropy: {}", entropy_hex);
        }
        println!("✅ All {} edge case vectors passed!", TEST_VECTORS_EDGE.len());
    }

    #[test]
    fn test_mnemonic_validation() {
        // Valid mnemonic
        assert!(Mnemonic::validate(
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
            Language::English
        ));

        // Invalid checksum
        assert!(!Mnemonic::validate(
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon",
            Language::English
        ));

        // Invalid word count
        assert!(!Mnemonic::validate(
            "abandon abandon abandon",
            Language::English
        ));
    }

    #[test]
    fn test_did_deterministic() {
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let did1 = calculate_did(mnemonic);
        let did2 = calculate_did(mnemonic);
        assert_eq!(did1, did2, "DID should be deterministic");
        assert_eq!(did1.len(), 128, "SHA-512 hash should be 128 hex characters");
    }

    #[test]
    fn test_did_uniqueness() {
        let mnemonic1 = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let mnemonic2 = "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong";
        let did1 = calculate_did(mnemonic1);
        let did2 = calculate_did(mnemonic2);
        assert_ne!(did1, did2, "Different mnemonics should produce different DIDs");
    }
}

// ============================================================================
// PrepareBlindSign Implementation (Algorithm 4 from TIAC)
// ============================================================================

#[derive(Serialize, Deserialize)]
pub struct KoRProof {
    pub c: String,   // Challenge (hex)
    pub s1: String,  // Response s1 (hex)
    pub s2: String,  // Response s2 (hex)
    pub s3: String,  // Response s3 (hex)
}

#[derive(Serialize, Deserialize)]
pub struct PrepareBlindSignResult {
    pub com: String,      // Commitment com (hex)
    pub com_i: String,    // Initial commitment com_i (hex)
    pub h: String,        // Hash point h (hex)
    pub proof: KoRProof,  // Zero-knowledge proof π_s
    pub o: String,        // Blinding factor o (hex) - KEEP SECRET
}

/// Helper: Generate random scalar
fn random_scalar() -> Result<Scalar, String> {
    let mut bytes = [0u8; 32];
    getrandom(&mut bytes).map_err(|e| format!("Random generation failed: {}", e))?;
    Ok(Scalar::from_bytes_mod_order(bytes))
}

/// Helper: Convert hex string to Scalar
fn hex_to_scalar(hex: &str) -> Result<Scalar, String> {
    // Parse hex string to bytes
    let bytes = hex::decode(hex).map_err(|e| format!("Invalid hex: {}", e))?;

    // SHA-512 hash to get uniform distribution, then reduce modulo curve order
    let mut hasher = Sha512::new();
    hasher.update(&bytes);
    let hash = hasher.finalize();

    // Take first 32 bytes for Scalar (Curve25519 uses 256-bit scalars)
    let mut scalar_bytes = [0u8; 32];
    scalar_bytes.copy_from_slice(&hash[0..32]);

    Ok(Scalar::from_bytes_mod_order(scalar_bytes))
}

/// Helper: Convert RistrettoPoint to hex string
fn point_to_hex(point: &RistrettoPoint) -> String {
    hex::encode(point.compress().as_bytes())
}

/// Helper: Convert Scalar to hex string
fn scalar_to_hex(scalar: &Scalar) -> String {
    hex::encode(scalar.as_bytes())
}

/// Helper: Hash to point (deterministic)
fn hash_to_point(data: &[u8]) -> RistrettoPoint {
    let mut hasher = Sha512::new();
    hasher.update(data);
    let hash = hasher.finalize();

    // Use hash as seed for deterministic point generation
    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&hash[0..32]);
    let scalar = Scalar::from_bytes_mod_order(bytes);

    RISTRETTO_BASEPOINT_POINT * scalar
}

/// Helper: Hash multiple strings to Scalar (for Fiat-Shamir)
fn hash_to_scalar(inputs: &[&str]) -> Scalar {
    let mut hasher = Sha512::new();
    for input in inputs {
        hasher.update(input.as_bytes());
    }
    let hash = hasher.finalize();

    let mut bytes = [0u8; 32];
    bytes.copy_from_slice(&hash[0..32]);
    Scalar::from_bytes_mod_order(bytes)
}

/// Compute KoR (Knowledge of Representation) Proof - Algorithm 5
/// Proves knowledge of (o_i, did, o) such that:
///   com_i = g^{o_i} · h1^{did}
///   com = g^{o} · h^{did}
fn compute_kor_proof(
    g: &RistrettoPoint,      // Generator g1
    h1: &RistrettoPoint,     // Generator h1
    h: &RistrettoPoint,      // Hash point h
    com: &RistrettoPoint,    // Commitment com
    com_i: &RistrettoPoint,  // Initial commitment com_i
    o_i: &Scalar,            // Secret o_i
    did: &Scalar,            // Secret did
    o: &Scalar,              // Secret o
) -> Result<KoRProof, String> {
    // Generate random r1, r2, r3
    let r1 = random_scalar()?;
    let r2 = random_scalar()?;
    let r3 = random_scalar()?;

    // Compute com_i' = g^{r1} · h1^{r2}
    let com_i_prime = g * r1 + h1 * r2;

    // Compute com' = g^{r3} · h^{r2}
    let com_prime = g * r3 + h * r2;

    // Compute challenge c = Hash(g, h, h1, com, com', com_i, com_i')
    let c = hash_to_scalar(&[
        &point_to_hex(g),
        &point_to_hex(h),
        &point_to_hex(h1),
        &point_to_hex(com),
        &point_to_hex(&com_prime),
        &point_to_hex(com_i),
        &point_to_hex(&com_i_prime),
    ]);

    // Compute responses:
    // s1 = r1 - c·o_i
    let s1 = r1 - c * o_i;

    // s2 = r2 - c·did
    let s2 = r2 - c * did;

    // s3 = r3 - c·o
    let s3 = r3 - c * o;

    Ok(KoRProof {
        c: scalar_to_hex(&c),
        s1: scalar_to_hex(&s1),
        s2: scalar_to_hex(&s2),
        s3: scalar_to_hex(&s3),
    })
}

/// PrepareBlindSign - Algorithm 4 from TIAC
/// Creates a blind signature request with zero-knowledge proof
///
/// Inputs:
///   - did_hex: Digital Identity (SHA-512 hash from mnemonic, hex string)
///   - o_hex: Blinding factor (SHA-512 hash from mnemonic, hex string)
///
/// Returns: PrepareBlindSignResult with commitment, proof, and blinding factors
#[wasm_bindgen]
pub fn prepare_blind_sign(did_hex: &str, o_hex: &str) -> Result<JsValue, JsValue> {
    // Convert DID and o from hex to scalars
    let did_scalar = hex_to_scalar(did_hex)
        .map_err(|e| JsValue::from_str(&format!("Invalid DID: {}", e)))?;

    let o_scalar = hex_to_scalar(o_hex)
        .map_err(|e| JsValue::from_str(&format!("Invalid o-value: {}", e)))?;

    // Generate random o_i (blinding factor for initial commitment)
    let o_i = random_scalar()
        .map_err(|e| JsValue::from_str(&format!("Random generation failed: {}", e)))?;

    // Generators (in real implementation, these would come from crypto parameters)
    let g = RISTRETTO_BASEPOINT_POINT;
    let h1 = hash_to_point(b"h1_generator");

    // Compute com_i = g^{o_i} · h1^{did}
    let com_i = g * o_i + h1 * did_scalar;

    // Compute h = Hash(com_i) - deterministic hash to point
    let h = hash_to_point(com_i.compress().as_bytes());

    // Compute com = g^{o} · h^{did}
    let com = g * o_scalar + h * did_scalar;

    // Compute KoR proof π_s
    let proof = compute_kor_proof(
        &g,
        &h1,
        &h,
        &com,
        &com_i,
        &o_i,
        &did_scalar,
        &o_scalar,
    ).map_err(|e| JsValue::from_str(&format!("Proof generation failed: {}", e)))?;

    // Build result
    let result = PrepareBlindSignResult {
        com: point_to_hex(&com),
        com_i: point_to_hex(&com_i),
        h: point_to_hex(&h),
        proof,
        o: o_hex.to_string(), // Keep original o for later use
    };

    serde_wasm_bindgen::to_value(&result)
        .map_err(|e| JsValue::from_str(&format!("Serialization error: {}", e)))
}
