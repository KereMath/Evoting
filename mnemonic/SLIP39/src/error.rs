use thiserror::Error;

#[derive(Debug, Error)]
pub enum Slip39Error {
    #[error("Invalid threshold: {0}")]
    InvalidThreshold(String),

    #[error("Invalid group configuration: {0}")]
    InvalidGroupConfig(String),

    #[error("Insufficient shares: have {have}, need {need}")]
    InsufficientShares {
        have: usize,
        need: usize
    },

    #[error("Share validation failed: {0}")]
    ShareValidationFailed(String),

    #[error("Checksum verification failed")]
    ChecksumFailed,

    #[error("Invalid share identifier: {0}")]
    InvalidIdentifier(String),

    #[error("Incompatible shares: {0}")]
    IncompatibleShares(String),

    #[error("Invalid entropy size: {0} bits (must be 128 or 256)")]
    InvalidEntropySize(usize),

    #[error("Invalid mnemonic length: {0} words (expected 20 or 33)")]
    InvalidMnemonicLength(usize),

    #[error("Encryption error: {0}")]
    EncryptionError(String),

    #[error("Invalid passphrase: {0}")]
    InvalidPassphrase(String),

    #[error("Group reconstruction failed: {0}")]
    GroupReconstructionFailed(String),

    #[error("Invalid share data: {0}")]
    InvalidShareData(String),

    #[error("Digest verification failed - shares may be corrupted or incorrect")]
    DigestVerificationFailed,

    #[error("Wordlist error: {0}")]
    WordlistError(#[from] crate::wordlist::WordlistError),

    #[error("Hex decode error: {0}")]
    HexError(#[from] hex::FromHexError),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, Slip39Error>;
