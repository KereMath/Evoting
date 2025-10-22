use once_cell::sync::Lazy;
use std::collections::HashMap;
use thiserror::Error;

const ENGLISH_WORDLIST: &str = include_str!("../wordlists/english.txt");

#[derive(Debug, Error)]
pub enum WordlistError {
    #[error("Word not found in wordlist: {0}")]
    WordNotFound(String),

    #[error("Invalid index: {0} (must be 0-1023)")]
    InvalidIndex(u16),

    #[error("Wordlist must contain exactly 1024 words, found {0}")]
    InvalidWordlistSize(usize),
}

#[derive(Debug)]
pub struct Wordlist {
    words: Vec<String>,
    word_to_index: HashMap<String, u16>,
}

impl Wordlist {
    pub fn from_str(wordlist_str: &str) -> Result<Self, WordlistError> {
        let words: Vec<String> = wordlist_str
            .lines()
            .map(|line| line.trim().to_lowercase())
            .filter(|line| !line.is_empty())
            .collect();

        if words.len() != 1024 {
            return Err(WordlistError::InvalidWordlistSize(words.len()));
        }

        let word_to_index: HashMap<String, u16> = words
            .iter()
            .enumerate()
            .map(|(i, word)| (word.clone(), i as u16))
            .collect();

        Ok(Wordlist {
            words,
            word_to_index,
        })
    }

    pub fn get_word(&self, index: u16) -> Result<&str, WordlistError> {
        if index >= 1024 {
            return Err(WordlistError::InvalidIndex(index));
        }
        Ok(&self.words[index as usize])
    }

    pub fn get_index(&self, word: &str) -> Result<u16, WordlistError> {
        let normalized = word.trim().to_lowercase();
        self.word_to_index
            .get(&normalized)
            .copied()
            .ok_or_else(|| WordlistError::WordNotFound(word.to_string()))
    }

    pub fn words(&self) -> &[String] {
        &self.words
    }

    pub fn len(&self) -> usize {
        self.words.len()
    }

    pub fn is_empty(&self) -> bool {
        self.words.is_empty()
    }
}

pub static ENGLISH: Lazy<Wordlist> = Lazy::new(|| {
    Wordlist::from_str(ENGLISH_WORDLIST).expect("Failed to load English wordlist")
});

pub fn get_english_wordlist() -> &'static Wordlist {
    &ENGLISH
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wordlist_size() {
        let wordlist = get_english_wordlist();
        assert_eq!(wordlist.len(), 1024);
    }

    #[test]
    fn test_word_lookup() {
        let wordlist = get_english_wordlist();

        assert_eq!(wordlist.get_word(0).unwrap(), "academic");

        let last_word = wordlist.get_word(1023).unwrap();
        assert!(!last_word.is_empty());
    }

    #[test]
    fn test_index_lookup() {
        let wordlist = get_english_wordlist();

        assert_eq!(wordlist.get_index("academic").unwrap(), 0);

        assert_eq!(wordlist.get_index("ACADEMIC").unwrap(), 0);
        assert_eq!(wordlist.get_index("Academic").unwrap(), 0);
    }

    #[test]
    fn test_roundtrip() {
        let wordlist = get_english_wordlist();

        for i in 0..1024 {
            let word = wordlist.get_word(i).unwrap();
            let index = wordlist.get_index(word).unwrap();
            assert_eq!(index, i);
        }
    }

    #[test]
    fn test_invalid_word() {
        let wordlist = get_english_wordlist();
        assert!(wordlist.get_index("notaword123").is_err());
    }

    #[test]
    fn test_invalid_index() {
        let wordlist = get_english_wordlist();
        assert!(wordlist.get_word(1024).is_err());
        assert!(wordlist.get_word(9999).is_err());
    }

    #[test]
    fn test_no_duplicates() {
        let wordlist = get_english_wordlist();
        let mut seen = std::collections::HashSet::new();

        for word in wordlist.words() {
            assert!(seen.insert(word), "Duplicate word found: {}", word);
        }
    }

    #[test]
    fn test_all_lowercase() {
        let wordlist = get_english_wordlist();

        for word in wordlist.words() {
            assert_eq!(word, &word.to_lowercase(), "Word not lowercase: {}", word);
        }
    }
}
