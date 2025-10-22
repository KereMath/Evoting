use crate::{
    entropy::{Entropy, EntropyBits},
    error::{Bip39Error, Result},
    seed::Seed,
    wordlist::{Wordlist, Language},
};
use unicode_normalization::UnicodeNormalization;
use zeroize::{Zeroize, ZeroizeOnDrop};
use std::fmt;

#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct Mnemonic {
    words: Vec<String>,
    entropy: Entropy,
    #[zeroize(skip)]
    language: Language,
}

impl Mnemonic {

    pub fn generate(bits: EntropyBits, language: Language) -> Result<Self> {
        let entropy = Entropy::generate(bits)?;
        Self::from_entropy(entropy, language)
    }

    pub fn from_entropy(entropy: Entropy, language: Language) -> Result<Self> {
        let wordlist = Wordlist::get(language)?;
        let bits = entropy.to_bits_with_checksum();

        let mut words = Vec::new();
        for chunk in bits.chunks(11) {
            let mut index = 0u16;
            for (i, &bit) in chunk.iter().enumerate() {
                if bit {
                    index |= 1 << (10 - i);
                }
            }

            let word = wordlist.get_word(index as usize)
                .ok_or_else(|| Bip39Error::WordNotFound(format!("index_{}", index)))?;
            words.push(word.to_string());
        }

        Ok(Mnemonic {
            words,
            entropy,
            language,
        })
    }

    pub fn from_phrase(phrase: &str, language: Language) -> Result<Self> {

        let normalized = phrase.nfkd().collect::<String>();
        let words: Vec<String> = normalized
            .split_whitespace()
            .map(|w| w.to_string())
            .collect();

        let word_count = words.len();
        EntropyBits::from_word_count(word_count)?;

        let wordlist = Wordlist::get(language)?;

        let mut bits = Vec::new();
        for word in &words {
            let index = wordlist.get_index(word)
                .ok_or_else(|| Bip39Error::WordNotFound(word.clone()))?;

            for i in (0..11).rev() {
                bits.push((index >> i) & 1 == 1);
            }
        }

        let total_bits = bits.len();
        let checksum_bits = total_bits / 33;
        let entropy_bits = total_bits - checksum_bits;

        let mut entropy_data = Vec::new();
        for chunk in bits[..entropy_bits].chunks_exact(8) {
            let mut byte = 0u8;
            for (i, &bit) in chunk.iter().enumerate() {
                if bit {
                    byte |= 1 << (7 - i);
                }
            }
            entropy_data.push(byte);
        }

        let entropy = Entropy::from_bytes(entropy_data)?;
        let calculated_bits = entropy.to_bits_with_checksum();

        if bits != calculated_bits {
            return Err(Bip39Error::InvalidChecksum);
        }

        Ok(Mnemonic {
            words,
            entropy,
            language,
        })
    }

    pub fn phrase(&self) -> String {
        self.words.join(" ")
    }

    pub fn words(&self) -> &[String] {
        &self.words
    }

    pub fn word_count(&self) -> usize {
        self.words.len()
    }

    pub fn entropy(&self) -> &Entropy {
        &self.entropy
    }

    pub fn language(&self) -> Language {
        self.language
    }

    pub fn to_seed(&self, passphrase: &str) -> Seed {
        Seed::from_mnemonic(&self.phrase(), passphrase)
    }

    pub fn validate(phrase: &str, language: Language) -> bool {
        Self::from_phrase(phrase, language).is_ok()
    }
}

impl fmt::Display for Mnemonic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.phrase())
    }
}

impl fmt::Debug for Mnemonic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Mnemonic")
            .field("word_count", &self.word_count())
            .field("language", &self.language)
            .field("entropy", &"<REDACTED>")
            .field("words", &"<REDACTED>")
            .finish()
    }
}

pub struct MnemonicBuilder {
    bits: EntropyBits,
    language: Language,
    passphrase: Option<String>,
}

impl Default for MnemonicBuilder {
    fn default() -> Self {
        Self {
            bits: EntropyBits::Bits128,
            language: Language::English,
            passphrase: None,
        }
    }
}

impl MnemonicBuilder {

    pub fn new() -> Self {
        Self::default()
    }

    pub fn bits(mut self, bits: EntropyBits) -> Self {
        self.bits = bits;
        self
    }

    pub fn language(mut self, language: Language) -> Self {
        self.language = language;
        self
    }

    pub fn passphrase(mut self, passphrase: impl Into<String>) -> Self {
        self.passphrase = Some(passphrase.into());
        self
    }

    pub fn build(self) -> Result<(Mnemonic, Option<Seed>)> {
        let mnemonic = Mnemonic::generate(self.bits, self.language)?;

        let seed = self.passphrase.as_ref()
            .map(|p| mnemonic.to_seed(p));

        Ok((mnemonic, seed))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_VECTOR_1: (&str, &str) = (
        "00000000000000000000000000000000",
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about"
    );

    const TEST_VECTOR_2: (&str, &str) = (
        "7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f",
        "legal winner thank year wave sausage worth useful legal winner thank yellow"
    );

    #[test]
    fn test_from_entropy() {
        let entropy = Entropy::from_hex(TEST_VECTOR_1.0).unwrap();
        let mnemonic = Mnemonic::from_entropy(entropy, Language::English).unwrap();
        assert_eq!(mnemonic.phrase(), TEST_VECTOR_1.1);
    }

    #[test]
    fn test_from_phrase() {
        let mnemonic = Mnemonic::from_phrase(TEST_VECTOR_2.1, Language::English).unwrap();
        assert_eq!(mnemonic.entropy().to_hex(), TEST_VECTOR_2.0);
    }

    #[test]
    fn test_roundtrip() {
        let mnemonic1 = Mnemonic::generate(EntropyBits::Bits192, Language::English).unwrap();
        let phrase = mnemonic1.phrase();
        let mnemonic2 = Mnemonic::from_phrase(&phrase, Language::English).unwrap();

        assert_eq!(mnemonic1.phrase(), mnemonic2.phrase());
        assert_eq!(mnemonic1.entropy().to_hex(), mnemonic2.entropy().to_hex());
    }

    #[test]
    fn test_invalid_checksum() {
        let invalid = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon";
        assert!(Mnemonic::from_phrase(invalid, Language::English).is_err());
    }

    #[test]
    fn test_builder() {
        let result = MnemonicBuilder::new()
            .bits(EntropyBits::Bits256)
            .language(Language::English)
            .passphrase("test")
            .build();

        assert!(result.is_ok());
        let (mnemonic, seed) = result.unwrap();
        assert_eq!(mnemonic.word_count(), 24);
        assert!(seed.is_some());
    }
}
