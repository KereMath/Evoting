#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod entropy;
pub mod error;
pub mod mnemonic;
pub mod seed;
pub mod wallet;
pub mod wordlist;
pub mod utils;

pub use entropy::{Entropy, EntropyBits};
pub use error::{Bip39Error, Result};
pub use mnemonic::Mnemonic;
pub use wordlist::Language;
pub use seed::Seed;
pub use wallet::{Wallet, HDPath, ExtendedKey};
pub use wordlist::Wordlist;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub mod prelude {
    pub use crate::{
        Mnemonic, Language, Entropy, EntropyBits,
        Seed, Wallet, HDPath, Result
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_library_version() {
        assert!(!VERSION.is_empty());
    }
}
