
use crate::error::{Result, ShamirError};
use crate::shamir::{Share, ShamirSSS};
use pure_bip39::{Language, Mnemonic};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MnemonicShare {

    pub id: u8,

    pub share_data: String,

    pub total_shares: u8,

    pub threshold: u8,
}

impl MnemonicShare {

    pub fn new(id: u8, share_data: String, total_shares: u8, threshold: u8) -> Self {
        Self {
            id,
            share_data,
            total_shares,
            threshold,
        }
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).map_err(Into::into)
    }

    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).map_err(Into::into)
    }

    pub fn phrase(&self) -> &str {
        &self.share_data
    }
}

pub fn split_mnemonic(
    mnemonic_phrase: &str,
    threshold: usize,
    total_shares: usize,
    language: Language,
) -> Result<Vec<MnemonicShare>> {

    let mnemonic = Mnemonic::from_phrase(mnemonic_phrase, language)?;

    let entropy = mnemonic.entropy();
    let entropy_bytes = entropy.as_bytes();

    let sss = ShamirSSS::new(threshold, total_shares)?;

    let shares = sss.split(entropy_bytes)?;

    let mnemonic_shares: Vec<MnemonicShare> = shares
        .into_iter()
        .map(|share| {
            let share_bytes: Vec<u8> = share.value.iter().map(|gf| gf.value()).collect();
            let share_hex = hex::encode(&share_bytes);

            MnemonicShare::new(
                share.id,
                share_hex,
                total_shares as u8,
                threshold as u8,
            )
        })
        .collect();

    Ok(mnemonic_shares)
}

pub fn reconstruct_mnemonic(
    mnemonic_shares: &[MnemonicShare],
    language: Language,
) -> Result<String> {
    if mnemonic_shares.is_empty() {
        return Err(ShamirError::InsufficientShares {
            have: 0,
            need: 1,
        });
    }

    let threshold = mnemonic_shares[0].threshold as usize;
    let total_shares = mnemonic_shares[0].total_shares as usize;

    let sss = ShamirSSS::new(threshold, total_shares)?;

    let shares: Result<Vec<Share>> = mnemonic_shares
        .iter()
        .map(|ms| {

            let share_bytes = hex::decode(&ms.share_data)
                .map_err(|e| ShamirError::InvalidShareFormat(e.to_string()))?;

            let gf_values: Vec<crate::galois::GF256> = share_bytes
                .iter()
                .map(|&b| crate::galois::GF256::new(b))
                .collect();

            Ok(Share::new(ms.id, gf_values))
        })
        .collect();

    let shares = shares?;

    let reconstructed_entropy = sss.reconstruct(&shares)?;

    let entropy = pure_bip39::Entropy::from_bytes(reconstructed_entropy)?;
    let reconstructed_mnemonic = Mnemonic::from_entropy(entropy, language)?;

    Ok(reconstructed_mnemonic.phrase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_and_reconstruct_mnemonic() {
        let original = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let language = Language::English;

        let shares = split_mnemonic(original, 3, 5, language).unwrap();
        assert_eq!(shares.len(), 5);

        let reconstructed = reconstruct_mnemonic(&shares[0..3], language).unwrap();

        let original_mnemonic = Mnemonic::from_phrase(original, language).unwrap();
        let reconstructed_mnemonic = Mnemonic::from_phrase(&reconstructed, language).unwrap();

        assert_eq!(
            original_mnemonic.to_seed("").as_bytes(),
            reconstructed_mnemonic.to_seed("").as_bytes()
        );
    }

    #[test]
    fn test_mnemonic_share_json() {
        let share = MnemonicShare::new(
            1,
            "deadbeef".to_string(),
            5,
            3,
        );

        let json = share.to_json().unwrap();
        let parsed = MnemonicShare::from_json(&json).unwrap();

        assert_eq!(share.id, parsed.id);
        assert_eq!(share.threshold, parsed.threshold);
        assert_eq!(share.total_shares, parsed.total_shares);
        assert_eq!(share.share_data, parsed.share_data);
    }
}
