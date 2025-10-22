use hmac::Hmac;
use pbkdf2::pbkdf2;
use sha2::Sha512;
use unicode_normalization::UnicodeNormalization;
use zeroize::{Zeroize, ZeroizeOnDrop};

const PBKDF2_ROUNDS: u32 = 2048;

const SEED_SIZE: usize = 64;

#[derive(Clone, Zeroize, ZeroizeOnDrop)]
pub struct Seed {
    data: [u8; SEED_SIZE],
}

impl Seed {

    pub fn from_mnemonic(mnemonic: &str, passphrase: &str) -> Self {

        let normalized_mnemonic = mnemonic.nfkd().collect::<String>();
        let normalized_passphrase = passphrase.nfkd().collect::<String>();

        let password = normalized_mnemonic.as_bytes();
        let salt = format!("mnemonic{}", normalized_passphrase);

        let mut seed_data = [0u8; SEED_SIZE];

        pbkdf2::<Hmac<Sha512>>(
            password,
            salt.as_bytes(),
            PBKDF2_ROUNDS,
            &mut seed_data
        ).expect("PBKDF2 should not fail");

        Seed { data: seed_data }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }

    pub fn to_hex(&self) -> String {
        hex::encode(&self.data)
    }

    pub fn from_bytes(bytes: [u8; SEED_SIZE]) -> Self {
        Seed { data: bytes }
    }

    pub fn from_hex(hex: &str) -> Result<Self, hex::FromHexError> {
        let bytes = hex::decode(hex)?;
        if bytes.len() != SEED_SIZE {
            return Err(hex::FromHexError::InvalidStringLength);
        }

        let mut data = [0u8; SEED_SIZE];
        data.copy_from_slice(&bytes);
        Ok(Seed { data })
    }
}
