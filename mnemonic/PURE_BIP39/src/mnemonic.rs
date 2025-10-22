use crate::{
    entropy::{Entropy, EntropyBits},
    error::{Bip39Error, Result},
    seed::Seed,
    wordlist::{Wordlist, Language},
};
use unicode_normalization::UnicodeNormalization;
use zeroize::{Zeroize, ZeroizeOnDrop};
use core::fmt;

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

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_VECTORS: &[(&str, &str, &str)] = &[
        (
            "00000000000000000000000000000000",
            "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about",
            "c55257c360c07c72029aebc1b53c05ed0362ada38ead3e3e9efa3708e53495531f09a6987599d18264c1e1c92f2cf141630c7a3c4ab7c81b2f001698e7463b04"
        ),
        (
            "7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f7f",
            "legal winner thank year wave sausage worth useful legal winner thank yellow",
            "2e8905819b8723fe2c1d161860e5ee1830318dbf49a83bd451cfb8440c28bd6fa457fe1296106559a3c80937a1c1069be3a3a5bd381ee6260e8d9739fce1f607"
        ),
        (
            "80808080808080808080808080808080",
            "letter advice cage absurd amount doctor acoustic avoid letter advice cage above",
            "d71de856f81a8acc65e6fc851a38d4d7ec216fd0796d0a6827a3ad6ed5511a30fa280f12eb2e47ed2ac03b5c462a0358d18d69fe4f985ec81778c1b370b652a8"
        ),
        (
            "ffffffffffffffffffffffffffffffff",
            "zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo zoo wrong",
            "ac27495480225222079d7be181583751e86f571027b0497b5b5d11218e0a8a13332572917f0f8e5a589620c6f15b11c61dee327651a14c34e18231052e48c069"
        ),
    ];

    #[test]
    fn test_from_entropy() {
        for &(entropy_hex, expected_mnemonic, _) in TEST_VECTORS {
            let entropy = Entropy::from_hex(entropy_hex).unwrap();
            let mnemonic = Mnemonic::from_entropy(entropy, Language::English).unwrap();
            assert_eq!(mnemonic.phrase(), expected_mnemonic);
        }
    }

    #[test]
    fn test_from_phrase() {
        for &(expected_entropy, mnemonic_phrase, _) in TEST_VECTORS {
            let mnemonic = Mnemonic::from_phrase(mnemonic_phrase, Language::English).unwrap();
            assert_eq!(mnemonic.entropy().to_hex(), expected_entropy);
        }
    }

    #[test]
    fn test_to_seed() {
        for &(_, mnemonic_phrase, expected_seed) in TEST_VECTORS {
            let mnemonic = Mnemonic::from_phrase(mnemonic_phrase, Language::English).unwrap();
            let seed = mnemonic.to_seed("TREZOR");
            assert_eq!(seed.to_hex(), expected_seed);
        }
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
    fn test_invalid_word_count() {
        let invalid = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon";
        assert!(Mnemonic::from_phrase(invalid, Language::English).is_err());
    }

    #[test]
    fn test_word_counts() {
        assert_eq!(Mnemonic::generate(EntropyBits::Bits128, Language::English).unwrap().word_count(), 12);
        assert_eq!(Mnemonic::generate(EntropyBits::Bits160, Language::English).unwrap().word_count(), 15);
        assert_eq!(Mnemonic::generate(EntropyBits::Bits192, Language::English).unwrap().word_count(), 18);
        assert_eq!(Mnemonic::generate(EntropyBits::Bits224, Language::English).unwrap().word_count(), 21);
        assert_eq!(Mnemonic::generate(EntropyBits::Bits256, Language::English).unwrap().word_count(), 24);
    }
}
