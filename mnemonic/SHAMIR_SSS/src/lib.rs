#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod error;
pub mod galois;
pub mod shamir;
pub mod mnemonic_share;

pub use error::{Result, ShamirError};
pub use galois::GF256;
pub use shamir::{Share, ShamirSSS};
pub use mnemonic_share::{MnemonicShare, split_mnemonic, reconstruct_mnemonic};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
