use crate::error::{Bip39Error, Result};
use once_cell::sync::Lazy;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {

    English,

    Japanese,

    Korean,

    Spanish,

    ChineseSimplified,

    ChineseTraditional,

    French,

    Italian,

    Czech,
}

impl Language {

    pub fn name(&self) -> &str {
        match self {
            Language::English => "English",
            Language::Japanese => "Japanese",
            Language::Korean => "Korean",
            Language::Spanish => "Spanish",
            Language::ChineseSimplified => "Chinese (Simplified)",
            Language::ChineseTraditional => "Chinese (Traditional)",
            Language::French => "French",
            Language::Italian => "Italian",
            Language::Czech => "Czech",
        }
    }

    pub fn code(&self) -> &str {
        match self {
            Language::English => "en",
            Language::Japanese => "ja",
            Language::Korean => "ko",
            Language::Spanish => "es",
            Language::ChineseSimplified => "zh-hans",
            Language::ChineseTraditional => "zh-hant",
            Language::French => "fr",
            Language::Italian => "it",
            Language::Czech => "cs",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Wordlist {
    language: Language,
    words: Vec<String>,
    word_to_index: HashMap<String, u16>,
}

impl Wordlist {

    pub fn from_str(content: &str, language: Language) -> Result<Self> {
        let words: Vec<String> = content
            .lines()
            .map(|w| w.trim().to_string())
            .filter(|w| !w.is_empty())
            .collect();

        if words.len() != 2048 {
            return Err(Bip39Error::InvalidWordlist(words.len()));
        }

        let word_to_index = words
            .iter()
            .enumerate()
            .map(|(i, w)| (w.clone(), i as u16))
            .collect();

        Ok(Wordlist {
            language,
            words,
            word_to_index,
        })
    }

    pub fn get_word(&self, index: usize) -> Option<&str> {
        self.words.get(index).map(|s| s.as_str())
    }

    pub fn get_index(&self, word: &str) -> Option<u16> {
        self.word_to_index.get(word).copied()
    }

    pub fn words(&self) -> &[String] {
        &self.words
    }

    pub fn language(&self) -> Language {
        self.language
    }
}

static WORDLISTS: Lazy<HashMap<Language, Wordlist>> = Lazy::new(|| {
    let mut map = HashMap::new();

    let english_words = include_str!("../data/wordlists/english.txt");

    if let Ok(wordlist) = Wordlist::from_str(english_words, Language::English) {
        map.insert(Language::English, wordlist);
    }

    #[cfg(feature = "all-languages")]
    {

    }

    map
});

impl Wordlist {

    pub fn get(language: Language) -> Result<&'static Wordlist> {
        WORDLISTS
            .get(&language)
            .ok_or_else(|| Bip39Error::InvalidWordlist(0))
    }
}

pub fn load_wordlist_json(json: &str) -> Result<HashMap<Language, Vec<String>>> {
    let data: HashMap<String, Vec<String>> = serde_json::from_str(json)?;
    let mut result = HashMap::new();

    for (lang_code, words) in data {
        let language = match lang_code.as_str() {
            "en" => Language::English,
            "ja" => Language::Japanese,
            "ko" => Language::Korean,
            "es" => Language::Spanish,
            "zh-hans" => Language::ChineseSimplified,
            "zh-hant" => Language::ChineseTraditional,
            "fr" => Language::French,
            "it" => Language::Italian,
            "cs" => Language::Czech,
            _ => continue,
        };

        if words.len() == 2048 {
            result.insert(language, words);
        }
    }

    Ok(result)
}
