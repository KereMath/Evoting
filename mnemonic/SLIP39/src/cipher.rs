use hmac::Hmac;
use pbkdf2::pbkdf2;
use sha2::Sha256;
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::error::{Result, Slip39Error};

const CUSTOMIZATION_STRING: &[u8] = b"shamir";
const FEISTEL_ROUNDS: usize = 4;
const BASE_ITERATION_COUNT: u32 = 10000;

#[derive(Debug, Clone, Zeroize, ZeroizeOnDrop)]
pub struct EncryptedSecret {
    pub data: Vec<u8>,
    pub iteration_exponent: u8,
}

impl EncryptedSecret {
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

    pub fn iterations(&self) -> u32 {
        BASE_ITERATION_COUNT * (1 << self.iteration_exponent)
    }
}

#[derive(Debug)]
pub struct FeistelCipher {
    passphrase: Vec<u8>,
    identifier: u16,
    extendable: bool,
}

impl FeistelCipher {
    pub fn new(passphrase: &[u8], identifier: u16, extendable: bool) -> Self {
        FeistelCipher {
            passphrase: passphrase.to_vec(),
            identifier,
            extendable,
        }
    }

    fn get_salt(&self) -> Vec<u8> {
        if self.extendable {
            Vec::new()
        } else {
            let mut salt = CUSTOMIZATION_STRING.to_vec();
            salt.extend_from_slice(&self.identifier.to_be_bytes());
            salt
        }
    }

    fn round_function(&self, round: u8, r: &[u8], iteration_exponent: u8) -> Result<Vec<u8>> {
        let iterations = (BASE_ITERATION_COUNT << iteration_exponent) / FEISTEL_ROUNDS as u32;

        let mut password = vec![round];
        password.extend_from_slice(&self.passphrase);

        let mut salt = self.get_salt();
        salt.extend_from_slice(r);

        let mut output = vec![0u8; r.len()];
        pbkdf2::<Hmac<Sha256>>(&password, &salt, iterations, &mut output)
            .map_err(|e| Slip39Error::EncryptionError(format!("PBKDF2 failed: {}", e)))?;

        Ok(output)
    }

    pub fn encrypt(&self, secret: &[u8], iteration_exponent: u8) -> Result<EncryptedSecret> {
        if secret.len() % 2 != 0 {
            return Err(Slip39Error::EncryptionError(
                "Secret length must be even".to_string(),
            ));
        }

        let half_len = secret.len() / 2;
        let mut l = secret[..half_len].to_vec();
        let mut r = secret[half_len..].to_vec();

        for round in 0..FEISTEL_ROUNDS as u8 {
            let f = self.round_function(round, &r, iteration_exponent)?;
            let new_r: Vec<u8> = l.iter().zip(f.iter()).map(|(&li, &fi)| li ^ fi).collect();
            l = r;
            r = new_r;
        }

        let mut encrypted = Vec::with_capacity(secret.len());
        encrypted.extend_from_slice(&r);
        encrypted.extend_from_slice(&l);

        EncryptedSecret::new(encrypted, iteration_exponent)
    }

    pub fn decrypt(&self, encrypted: &EncryptedSecret) -> Result<Vec<u8>> {
        if encrypted.data.len() % 2 != 0 {
            return Err(Slip39Error::EncryptionError(
                "Encrypted data length must be even".to_string(),
            ));
        }

        let half_len = encrypted.data.len() / 2;
        let mut l = encrypted.data[..half_len].to_vec();
        let mut r = encrypted.data[half_len..].to_vec();

        for round in (0..FEISTEL_ROUNDS as u8).rev() {
            let f = self.round_function(round, &r, encrypted.iteration_exponent)?;
            let new_r: Vec<u8> = l.iter().zip(f.iter()).map(|(&li, &fi)| li ^ fi).collect();
            l = r;
            r = new_r;
        }

        let mut decrypted = Vec::with_capacity(encrypted.data.len());
        decrypted.extend_from_slice(&r);
        decrypted.extend_from_slice(&l);

        Ok(decrypted)
    }
}

impl Drop for FeistelCipher {
    fn drop(&mut self) {
        self.passphrase.zeroize();
    }
}
