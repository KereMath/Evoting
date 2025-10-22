use thiserror::Error;

pub type Result<T> = std::result::Result<T, Bip39Error>;

#[derive(Error, Debug)]
pub enum Bip39Error {
    #[error("Invalid entropy size: {0} bits. Must be 128, 160, 192, 224, or 256")]
    InvalidEntropySize(usize),

    #[error("Invalid mnemonic length: {0} words. Must be 12, 15, 18, 21, or 24")]
    InvalidMnemonicLength(usize),

    #[error("Word '{0}' not found in wordlist")]
    WordNotFound(String),

    #[error("Invalid checksum - mnemonic is corrupted or invalid")]
    InvalidChecksum,

    #[error("Wordlist has {0} words, expected 2048")]
    InvalidWordlist(usize),

    #[error("Invalid seed phrase: {0}")]
    InvalidSeedPhrase(String),

    #[error("HD derivation error: {0}")]
    DerivationError(String),

    #[error("Invalid derivation path: {0}")]
    InvalidPath(String),

    #[error("Cryptographic error: {0}")]
    CryptoError(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Hex decode error: {0}")]
    HexError(#[from] hex::FromHexError),

    #[error("UTF-8 error: {0}")]
    Utf8Error(#[from] std::string::FromUtf8Error),

    #[error("Random generation error: {0}")]
    RandomError(String),
}
