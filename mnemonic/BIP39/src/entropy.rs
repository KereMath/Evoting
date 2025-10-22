use crate::error::{Bip39Error, Result};
use rand::RngCore;
use sha2::{Sha256, Digest};
use zeroize::{Zeroize, ZeroizeOnDrop};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntropyBits {

    Bits128 = 128,

    Bits160 = 160,

    Bits192 = 192,

    Bits224 = 224,

    Bits256 = 256,
}

impl EntropyBits {

    pub fn word_count(&self) -> usize {
        (*self as usize + *self as usize / 32) / 11
    }

    pub fn byte_count(&self) -> usize {
        *self as usize / 8
    }

    pub fn checksum_bits(&self) -> usize {
        *self as usize / 32
    }

    pub fn from_bits(bits: usize) -> Result<Self> {
        match bits {
            128 => Ok(EntropyBits::Bits128),
            160 => Ok(EntropyBits::Bits160),
            192 => Ok(EntropyBits::Bits192),
            224 => Ok(EntropyBits::Bits224),
            256 => Ok(EntropyBits::Bits256),
            _ => Err(Bip39Error::InvalidEntropySize(bits)),
        }
    }

    pub fn from_word_count(words: usize) -> Result<Self> {
        match words {
            12 => Ok(EntropyBits::Bits128),
            15 => Ok(EntropyBits::Bits160),
            18 => Ok(EntropyBits::Bits192),
            21 => Ok(EntropyBits::Bits224),
            24 => Ok(EntropyBits::Bits256),
            _ => Err(Bip39Error::InvalidMnemonicLength(words)),
        }
    }
}

impl fmt::Display for EntropyBits {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} bits ({} words)", *self as usize, self.word_count())
    }
}

#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct Entropy {
    #[zeroize(skip)]
    bits: EntropyBits,
    data: Vec<u8>,
}

impl Entropy {

    pub fn generate(bits: EntropyBits) -> Result<Self> {
        let mut data = vec![0u8; bits.byte_count()];
        Self::fill_random(&mut data)?;

        Ok(Entropy { bits, data })
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self> {
        let bits = EntropyBits::from_bits(bytes.len() * 8)?;
        Ok(Entropy { bits, data: bytes })
    }

    pub fn from_hex(hex: &str) -> Result<Self> {
        let bytes = hex::decode(hex)?;
        Self::from_bytes(bytes)
    }

    fn fill_random(buffer: &mut [u8]) -> Result<()> {
        use rand::rngs::OsRng;

        OsRng.try_fill_bytes(buffer)
            .map_err(|e| Bip39Error::RandomError(e.to_string()))
    }

    pub fn bits(&self) -> EntropyBits {
        self.bits
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    pub fn to_hex(&self) -> String {
        hex::encode(&self.data)
    }

    pub fn checksum(&self) -> Vec<bool> {
        let hash = Sha256::digest(&self.data);
        let checksum_bits = self.bits.checksum_bits();

        let mut checksum = Vec::with_capacity(checksum_bits);
        for i in 0..checksum_bits {
            let byte_index = i / 8;
            let bit_index = 7 - (i % 8);
            checksum.push((hash[byte_index] >> bit_index) & 1 == 1);
        }

        checksum
    }

    pub fn to_bits_with_checksum(&self) -> Vec<bool> {
        let mut bits = Vec::new();

        for byte in &self.data {
            for i in (0..8).rev() {
                bits.push((byte >> i) & 1 == 1);
            }
        }

        bits.extend(self.checksum());

        bits
    }
}

impl fmt::Debug for Entropy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Entropy")
            .field("bits", &self.bits)
            .field("data", &"<REDACTED>")
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entropy_generation() {
        for bits in [EntropyBits::Bits128, EntropyBits::Bits256] {
            let entropy = Entropy::generate(bits).unwrap();
            assert_eq!(entropy.as_bytes().len(), bits.byte_count());
        }
    }

    #[test]
    fn test_entropy_from_hex() {
        let hex = "00000000000000000000000000000000";
        let entropy = Entropy::from_hex(hex).unwrap();
        assert_eq!(entropy.bits(), EntropyBits::Bits128);
        assert_eq!(entropy.to_hex(), hex);
    }

    #[test]
    fn test_checksum_calculation() {

        let entropy = Entropy::from_hex("00000000000000000000000000000000").unwrap();
        let checksum = entropy.checksum();

        assert_eq!(checksum, vec![false, false, true, true]);
    }

    #[test]
    fn test_word_count_mapping() {
        assert_eq!(EntropyBits::Bits128.word_count(), 12);
        assert_eq!(EntropyBits::Bits256.word_count(), 24);
    }
}
