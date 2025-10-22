use crate::{
    error::{Bip39Error, Result},
    seed::Seed,
};
use bitcoin::{
    bip32::{Xpriv as ExtendedPrivKey, Xpub as ExtendedPubKey, DerivationPath},
    Network,
    Address,
    PublicKey,
    PrivateKey,
};
use secp256k1::Secp256k1;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct HDPath {

    pub coin: u32,

    pub account: u32,

    pub change: u32,

    pub index: u32,
}

impl HDPath {

    pub fn bitcoin() -> Self {
        HDPath {
            coin: 0,
            account: 0,
            change: 0,
            index: 0,
        }
    }

    pub fn ethereum() -> Self {
        HDPath {
            coin: 60,
            account: 0,
            change: 0,
            index: 0,
        }
    }

    pub fn to_derivation_path(&self) -> Result<DerivationPath> {
        let path_str = format!(
            "m/44'/{}'/{}'/{}/{}",
            self.coin,
            self.account,
            self.change,
            self.index
        );

        DerivationPath::from_str(&path_str)
            .map_err(|e| Bip39Error::InvalidPath(e.to_string()))
    }
}

pub struct ExtendedKey {

    pub xpriv: ExtendedPrivKey,

    pub xpub: ExtendedPubKey,
}

pub struct Wallet {
    network: Network,
    master_key: ExtendedPrivKey,
}

impl Drop for Wallet {
    fn drop(&mut self) {

    }
}

impl Wallet {

    pub fn from_seed(seed: &Seed, network: Network) -> Result<Self> {
        let master_key = ExtendedPrivKey::new_master(network, seed.as_bytes())?;

        Ok(Wallet {
            network,
            master_key,
        })
    }

    pub fn master_keys(&self) -> ExtendedKey {
        let secp = Secp256k1::new();
        let xpub = ExtendedPubKey::from_priv(&secp, &self.master_key);

        ExtendedKey {
            xpriv: self.master_key,
            xpub,
        }
    }

    pub fn derive(&self, path: &HDPath) -> Result<ExtendedKey> {
        let secp = Secp256k1::new();
        let derivation_path = path.to_derivation_path()?;

        let xpriv = self.master_key.derive_priv(&secp, &derivation_path)?;
        let xpub = ExtendedPubKey::from_priv(&secp, &xpriv);

        Ok(ExtendedKey { xpriv, xpub })
    }

    pub fn get_address(&self, path: &HDPath) -> Result<Address> {
        let key = self.derive(path)?;

        let public_key = PublicKey {
            compressed: true,
            inner: key.xpub.public_key,
        };

        let address = Address::p2pkh(&public_key, self.network);

        Ok(address)
    }

    pub fn get_private_key(&self, path: &HDPath) -> Result<String> {
        let key = self.derive(path)?;
        let private_key = PrivateKey::new(key.xpriv.private_key, self.network);
        Ok(private_key.to_wif())
    }

    pub fn get_public_key(&self, path: &HDPath) -> Result<PublicKey> {
        let key = self.derive(path)?;
        Ok(PublicKey {
            compressed: true,
            inner: key.xpub.public_key,
        })
    }

    pub fn generate_addresses(&self, count: usize, account: u32) -> Result<Vec<Address>> {
        let mut addresses = Vec::with_capacity(count);

        for i in 0..count {
            let path = HDPath {
                coin: 0,
                account,
                change: 0,
                index: i as u32,
            };

            addresses.push(self.get_address(&path)?);
        }

        Ok(addresses)
    }
}

#[derive(Debug)]
pub struct AccountInfo {

    pub path: String,

    pub address: String,

    pub public_key: String,

    pub private_key: String,

    pub xpub: String,
}

impl Wallet {

    pub fn get_account_info(&self, path: &HDPath) -> Result<AccountInfo> {
        let key = self.derive(path)?;
        let address = self.get_address(path)?;
        let private_key = self.get_private_key(path)?;

        Ok(AccountInfo {
            path: format!("m/44'/{}'/{}'/{}/{}",
                path.coin, path.account, path.change, path.index),
            address: address.to_string(),
            public_key: hex::encode(key.xpub.public_key.serialize()),
            private_key,
            xpub: key.xpub.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mnemonic::Mnemonic;
    use crate::wordlist::Language;
    use crate::entropy::EntropyBits;

    #[test]
    fn test_wallet_derivation() {

        let mnemonic = Mnemonic::generate(EntropyBits::Bits128, Language::English).unwrap();
        let seed = mnemonic.to_seed("");

        let wallet = Wallet::from_seed(&seed, Network::Bitcoin).unwrap();

        let path = HDPath::bitcoin();
        let address = wallet.get_address(&path).unwrap();

        assert!(!address.to_string().is_empty());
    }

    #[test]
    fn test_multiple_addresses() {
        let mnemonic = Mnemonic::generate(EntropyBits::Bits128, Language::English).unwrap();
        let seed = mnemonic.to_seed("");
        let wallet = Wallet::from_seed(&seed, Network::Bitcoin).unwrap();

        let addresses = wallet.generate_addresses(5, 0).unwrap();
        assert_eq!(addresses.len(), 5);

        use std::collections::HashSet;
        let unique: HashSet<_> = addresses.iter().map(|a| a.to_string()).collect();
        assert_eq!(unique.len(), 5);
    }
}
