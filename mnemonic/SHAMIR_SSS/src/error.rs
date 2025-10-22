use thiserror::Error;

#[derive(Error, Debug)]
pub enum ShamirError {
    #[error("Invalid threshold: {0}")]
    InvalidThreshold(String),

    #[error("Invalid number of shares: {0}")]
    InvalidShareCount(String),

    #[error("Not enough shares for reconstruction (have {have}, need {need})")]
    InsufficientShares { have: usize, need: usize },

    #[error("Invalid share format: {0}")]
    InvalidShareFormat(String),

    #[error("Share digest verification failed")]
    DigestVerificationFailed,

    #[error("Galois field error: {0}")]
    GaloisFieldError(String),

    #[error("Mnemonic error: {0}")]
    MnemonicError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("BIP39 error: {0}")]
    Bip39Error(#[from] pure_bip39::Bip39Error),

    #[error("Invalid field element: {0}")]
    InvalidFieldElement(String),

    #[error("Reconstruction failed: {0}")]
    ReconstructionFailed(String),
}

pub type Result<T> = std::result::Result<T, ShamirError>;
