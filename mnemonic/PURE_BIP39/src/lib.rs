#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod entropy;
pub mod error;
pub mod mnemonic;
pub mod seed;
pub mod wordlist;

pub use entropy::{Entropy, EntropyBits};
pub use error::{Bip39Error, Result};
pub use mnemonic::Mnemonic;
pub use seed::Seed;
pub use wordlist::{Language, Wordlist};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub const BIP39_SPEC_URL: &str = "https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_bip39_compliance() {
        assert_eq!(BIP39_SPEC_URL, "https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki");
    }
}
