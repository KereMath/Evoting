//! Feistel Cipher for SLIP-39
//!
//! SLIP-39 uses a 4-round Feistel network for encrypting the master secret.
//! Each round uses PBKDF2-HMAC-SHA256 as the round function.

use hmac::Hmac;
use pbkdf2::pbkdf2;
use sha2::Sha256;
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::error::{Result, Slip39Error};

/// Salt for PBKDF2 in encryption
const ENCRYPTION_SALT: &[u8] = b"shamir";

/// Number of Feistel rounds
const FEISTEL_ROUNDS: usize = 4;

/// Base iteration exponent (2^iteration_exponent iterations)
const BASE_ITERATION_EXPONENT: u32 = 10000;

/// Encrypted master secret with metadata
#[derive(Debug, Clone, Zeroize, ZeroizeOnDrop)]
pub struct EncryptedSecret {
    /// Encrypted data
    pub data: Vec<u8>,

    /// Iteration exponent (0-15)
    /// Actual iterations = BASE_ITERATION_EXPONENT * 2^iteration_exponent
    pub iteration_exponent: u8,
}

impl EncryptedSecret {
    /// Create a new encrypted secret
    pub fn new(data: Vec<u8>, iteration_exponent: u8) -> Result<Self> {
        if iteration_exponent > 15 {
            return Err(Slip39Error::EncryptionError(
                "Iteration exponent must be 0-15".to_string(),
            ));
        }

        Ok(EncryptedSecret {
            data,
            iteration_exponent,
        })
    }

    /// Get the actual number of PBKDF2 iterations
    pub fn iterations(&self) -> u32 {
        BASE_ITERATION_EXPONENT * (1 << self.iteration_exponent)
    }
}

/// Feistel cipher for SLIP-39 encryption/decryption
#[derive(Debug)]
pub struct FeistelCipher {
    passphrase: Vec<u8>,
}

impl FeistelCipher {
    /// Create a new Feistel cipher with a passphrase
    pub fn new(passphrase: &[u8]) -> Self {
        FeistelCipher {
            passphrase: passphrase.to_vec(),
        }
    }

    /// Encrypt a master secret using the Feistel network
    ///
    /// # Arguments
    /// * `secret` - The master secret to encrypt (must be even length)
    /// * `iteration_exponent` - Iteration exponent for PBKDF2 (0-15)
    ///
    /// # Returns
    /// Encrypted secret with metadata
    pub fn encrypt(&self, secret: &[u8], iteration_exponent: u8) -> Result<EncryptedSecret> {
        if secret.len() % 2 != 0 {
            return Err(Slip39Error::EncryptionError(
                "Secret length must be even".to_string(),
            ));
        }

        if iteration_exponent > 15 {
            return Err(Slip39Error::EncryptionError(
                "Iteration exponent must be 0-15".to_string(),
            ));
        }

        let iterations = BASE_ITERATION_EXPONENT * (1 << iteration_exponent);
        let half_len = secret.len() / 2;

        // Split secret into left and right halves
        let mut left = secret[..half_len].to_vec();
        let mut right = secret[half_len..].to_vec();

        // Feistel rounds (encryption: forward rounds)
        // In each round: L' = R, R' = L XOR F(R)
        for round in 0..FEISTEL_ROUNDS {
            let round_key = self.derive_round_key(&right, round as u32, iterations)?;

            // Compute R' = L XOR F(R)
            let mut new_right = Vec::with_capacity(half_len);
            for ((&l, &r), &k) in left.iter().zip(right.iter()).zip(round_key.iter()) {
                new_right.push(l ^ k); // F(R) is just the round key derived from R
            }

            // Swap: L' = R, R' = new_right
            left = right;
            right = new_right;
        }

        // Concatenate encrypted halves
        let mut encrypted = Vec::with_capacity(secret.len());
        encrypted.extend_from_slice(&left);
        encrypted.extend_from_slice(&right);

        EncryptedSecret::new(encrypted, iteration_exponent)
    }

    /// Decrypt an encrypted secret using the Feistel network
    ///
    /// # Arguments
    /// * `encrypted` - The encrypted secret
    ///
    /// # Returns
    /// Decrypted master secret
    pub fn decrypt(&self, encrypted: &EncryptedSecret) -> Result<Vec<u8>> {
        if encrypted.data.len() % 2 != 0 {
            return Err(Slip39Error::EncryptionError(
                "Encrypted data length must be even".to_string(),
            ));
        }

        let iterations = encrypted.iterations();
        let half_len = encrypted.data.len() / 2;

        // Split encrypted data into left and right halves
        let mut left = encrypted.data[..half_len].to_vec();
        let mut right = encrypted.data[half_len..].to_vec();

        // Feistel rounds (decryption: reverse rounds)
        // Reverse of encryption: R = L', L = R' XOR F(L')
        for round in (0..FEISTEL_ROUNDS).rev() {
            let round_key = self.derive_round_key(&left, round as u32, iterations)?;

            // Compute L = R' XOR F(L')
            let mut new_left = Vec::with_capacity(half_len);
            for ((&r, &l), &k) in right.iter().zip(left.iter()).zip(round_key.iter()) {
                new_left.push(r ^ k);
            }

            // Swap: R = L', L = new_left
            right = left;
            left = new_left;
        }

        // Concatenate decrypted halves
        let mut decrypted = Vec::with_capacity(encrypted.data.len());
        decrypted.extend_from_slice(&left);
        decrypted.extend_from_slice(&right);

        Ok(decrypted)
    }

    /// Derive a round key using PBKDF2-HMAC-SHA256
    fn derive_round_key(&self, data: &[u8], round: u32, iterations: u32) -> Result<Vec<u8>> {
        // Prepare salt: ENCRYPTION_SALT || round_index || data
        let mut salt = Vec::with_capacity(ENCRYPTION_SALT.len() + 1 + data.len());
        salt.extend_from_slice(ENCRYPTION_SALT);
        salt.push(round as u8);
        salt.extend_from_slice(data);

        // Derive key using PBKDF2
        let mut key = vec![0u8; data.len()];
        pbkdf2::<Hmac<Sha256>>(&self.passphrase, &salt, iterations, &mut key)
            .map_err(|e| Slip39Error::EncryptionError(format!("PBKDF2 failed: {}", e)))?;

        Ok(key)
    }
}

impl Drop for FeistelCipher {
    fn drop(&mut self) {
        self.passphrase.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feistel_encryption_decryption() {
        let secret = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];
        let passphrase = b"test passphrase";
        let iteration_exponent = 0;

        let cipher = FeistelCipher::new(passphrase);

        // Encrypt
        let encrypted = cipher.encrypt(&secret, iteration_exponent).unwrap();

        // Encrypted should be different from original
        assert_ne!(encrypted.data, secret);

        // Decrypt
        let decrypted = cipher.decrypt(&encrypted).unwrap();

        // Decrypted should match original
        assert_eq!(decrypted, secret);
    }

    #[test]
    fn test_feistel_empty_passphrase() {
        let secret = vec![0x01, 0x02, 0x03, 0x04];
        let passphrase = b"";
        let cipher = FeistelCipher::new(passphrase);

        let encrypted = cipher.encrypt(&secret, 0).unwrap();
        let decrypted = cipher.decrypt(&encrypted).unwrap();

        assert_eq!(decrypted, secret);
    }

    #[test]
    fn test_feistel_different_passphrases() {
        let secret = vec![0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08];

        let cipher1 = FeistelCipher::new(b"password1");
        let cipher2 = FeistelCipher::new(b"password2");

        let encrypted1 = cipher1.encrypt(&secret, 0).unwrap();
        let encrypted2 = cipher2.encrypt(&secret, 0).unwrap();

        // Different passphrases should produce different ciphertexts
        assert_ne!(encrypted1.data, encrypted2.data);
    }

    #[test]
    fn test_feistel_iteration_exponents() {
        let secret = vec![0x01, 0x02, 0x03, 0x04];
        let cipher = FeistelCipher::new(b"test");

        // Test different iteration exponents
        for exp in 0..=15 {
            let encrypted = cipher.encrypt(&secret, exp).unwrap();
            assert_eq!(encrypted.iteration_exponent, exp);

            let expected_iterations = BASE_ITERATION_EXPONENT * (1 << exp);
            assert_eq!(encrypted.iterations(), expected_iterations);

            let decrypted = cipher.decrypt(&encrypted).unwrap();
            assert_eq!(decrypted, secret);
        }
    }

    #[test]
    fn test_feistel_odd_length_fails() {
        let secret = vec![0x01, 0x02, 0x03]; // Odd length
        let cipher = FeistelCipher::new(b"test");

        let result = cipher.encrypt(&secret, 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_feistel_invalid_iteration_exponent() {
        let secret = vec![0x01, 0x02, 0x03, 0x04];
        let cipher = FeistelCipher::new(b"test");

        let result = cipher.encrypt(&secret, 16); // Invalid: > 15
        assert!(result.is_err());
    }

    #[test]
    fn test_feistel_deterministic() {
        let secret = vec![0x01, 0x02, 0x03, 0x04];
        let cipher = FeistelCipher::new(b"test");

        let encrypted1 = cipher.encrypt(&secret, 0).unwrap();
        let encrypted2 = cipher.encrypt(&secret, 0).unwrap();

        // Same inputs should produce same output
        assert_eq!(encrypted1.data, encrypted2.data);
    }
}
