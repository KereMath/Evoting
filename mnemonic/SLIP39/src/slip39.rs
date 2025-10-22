use rand::Rng;
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::cipher::{EncryptedSecret, FeistelCipher};
use crate::error::{Result, Slip39Error};
use crate::shamir::ShamirSecretSharing;
use crate::share::Share;

#[derive(Debug, Clone, Zeroize, ZeroizeOnDrop)]
pub struct MasterSecret {
    pub data: Vec<u8>,
}

impl MasterSecret {
    pub fn new(data: Vec<u8>) -> Result<Self> {
        if data.len() != 16 && data.len() != 32 {
            return Err(Slip39Error::InvalidEntropySize(data.len() * 8));
        }

        Ok(MasterSecret { data })
    }

    pub fn generate(bits: usize) -> Result<Self> {
        if bits != 128 && bits != 256 {
            return Err(Slip39Error::InvalidEntropySize(bits));
        }

        let mut rng = rand::thread_rng();
        let mut data = vec![0u8; bits / 8];
        rng.fill(&mut data[..]);

        Self::new(data)
    }

    pub fn bit_length(&self) -> usize {
        self.data.len() * 8
    }
}

#[derive(Debug, Clone)]
pub struct GroupConfig {
    pub member_threshold: u8,

    pub member_count: u8,
}

impl GroupConfig {
    pub fn new(member_threshold: u8, member_count: u8) -> Result<Self> {
        if member_threshold == 0 || member_threshold > 16 {
            return Err(Slip39Error::InvalidThreshold(format!(
                "Member threshold must be 1-16: {}",
                member_threshold
            )));
        }

        if member_count == 0 || member_count > 16 {
            return Err(Slip39Error::InvalidGroupConfig(format!(
                "Member count must be 1-16: {}",
                member_count
            )));
        }

        if member_threshold > member_count {
            return Err(Slip39Error::InvalidThreshold(format!(
                "Member threshold ({}) cannot exceed member count ({})",
                member_threshold, member_count
            )));
        }

        Ok(GroupConfig {
            member_threshold,
            member_count,
        })
    }
}

#[derive(Debug)]
pub struct Slip39 {
    identifier: u16,

    extendable: bool,

    iteration_exponent: u8,

    group_threshold: u8,

    groups: Vec<GroupConfig>,
}

impl Slip39 {
    pub fn new(
        group_threshold: u8,
        groups: Vec<GroupConfig>,
        iteration_exponent: Option<u8>,
    ) -> Result<Self> {
        if group_threshold == 0 || group_threshold > 16 {
            return Err(Slip39Error::InvalidThreshold(format!(
                "Group threshold must be 1-16: {}",
                group_threshold
            )));
        }

        if groups.is_empty() || groups.len() > 16 {
            return Err(Slip39Error::InvalidGroupConfig(format!(
                "Group count must be 1-16: {}",
                groups.len()
            )));
        }

        if group_threshold > groups.len() as u8 {
            return Err(Slip39Error::InvalidThreshold(format!(
                "Group threshold ({}) cannot exceed group count ({})",
                group_threshold,
                groups.len()
            )));
        }

        let iteration_exponent = iteration_exponent.unwrap_or(0);
        if iteration_exponent > 15 {
            return Err(Slip39Error::EncryptionError(
                "Iteration exponent must be 0-15".to_string(),
            ));
        }

        let mut rng = rand::thread_rng();
        let identifier = rng.gen::<u16>() & 0x7FFF;

        Ok(Slip39 {
            identifier,
            extendable: false,
            iteration_exponent,
            group_threshold,
            groups,
        })
    }

    pub fn new_single_group(threshold: u8, share_count: u8) -> Result<Self> {
        let group = GroupConfig::new(threshold, share_count)?;
        Self::new(1, vec![group], None)
    }

    pub fn generate_shares(
        &self,
        master_secret: &MasterSecret,
        passphrase: &[u8],
    ) -> Result<Vec<Vec<Share>>> {
        let cipher = FeistelCipher::new(passphrase, self.identifier, self.extendable);
        let encrypted = cipher.encrypt(&master_secret.data, self.iteration_exponent)?;

        let group_count = self.groups.len() as u8;

        let group_shares = if group_count > 1 {
            let x_coords: Vec<u8> = (0..group_count).collect();
            ShamirSecretSharing::split(
                &encrypted.data,
                self.group_threshold,
                group_count,
                &x_coords,
            )?
        } else {
            vec![encrypted.data.clone()]
        };

        let mut all_shares = Vec::new();

        for (group_idx, group_config) in self.groups.iter().enumerate() {
            let group_share = &group_shares[group_idx];

            let x_coords: Vec<u8> = (0..group_config.member_count).collect();

            let member_share_values = ShamirSecretSharing::split(
                group_share,
                group_config.member_threshold,
                group_config.member_count,
                &x_coords,
            )?;

            let mut group_member_shares = Vec::new();

            for (member_idx, share_value) in member_share_values.iter().enumerate() {
                let share = Share::new(
                    self.identifier,
                    self.extendable,
                    self.iteration_exponent,
                    group_idx as u8,
                    self.group_threshold,
                    group_count,
                    member_idx as u8,
                    group_config.member_threshold,
                    share_value.clone(),
                )?;

                group_member_shares.push(share);
            }

            all_shares.push(group_member_shares);
        }

        Ok(all_shares)
    }

    pub fn reconstruct_secret(shares: &[Share], passphrase: &[u8]) -> Result<MasterSecret> {
        if shares.is_empty() {
            return Err(Slip39Error::InsufficientShares {
                have: 0,
                need: 1,
            });
        }

        let identifier = shares[0].identifier;
        let iteration_exponent = shares[0].iteration_exponent;
        let group_threshold = shares[0].group_threshold;
        let group_count = shares[0].group_count;

        for share in shares {
            if share.identifier != identifier {
                return Err(Slip39Error::IncompatibleShares(
                    "Shares have different identifiers".to_string(),
                ));
            }

            if share.group_count != group_count || share.group_threshold != group_threshold {
                return Err(Slip39Error::IncompatibleShares(
                    "Shares have incompatible group configuration".to_string(),
                ));
            }
        }

        let mut groups: std::collections::HashMap<u8, Vec<&Share>> =
            std::collections::HashMap::new();

        for share in shares {
            groups
                .entry(share.group_index)
                .or_insert_with(Vec::new)
                .push(share);
        }

        let mut reconstructed_groups: Vec<(u8, Vec<u8>)> = Vec::new();

        for (group_idx, group_shares) in groups.iter() {
            if group_shares.is_empty() {
                continue;
            }

            let member_threshold = group_shares[0].member_threshold;

            if group_shares.len() < member_threshold as usize {
                continue;
            }

            let member_share_pairs: Vec<(u8, Vec<u8>)> = group_shares
                .iter()
                .map(|s| (s.member_index, s.value.clone()))
                .collect();

            let group_share = ShamirSecretSharing::reconstruct(&member_share_pairs)?;
            reconstructed_groups.push((*group_idx, group_share));
        }

        if reconstructed_groups.len() < group_threshold as usize {
            return Err(Slip39Error::InsufficientShares {
                have: reconstructed_groups.len(),
                need: group_threshold as usize,
            });
        }

        let encrypted_data = if group_count > 1 {
            ShamirSecretSharing::reconstruct(&reconstructed_groups)?
        } else {
            reconstructed_groups[0].1.clone()
        };

        let encrypted = EncryptedSecret::new(encrypted_data, iteration_exponent)?;
        let extendable = shares[0].extendable;
        let cipher = FeistelCipher::new(passphrase, identifier, extendable);
        let decrypted = cipher.decrypt(&encrypted)?;

        MasterSecret::new(decrypted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_master_secret_generation() {
        let secret = MasterSecret::generate(128).unwrap();
        assert_eq!(secret.bit_length(), 128);

        let secret = MasterSecret::generate(256).unwrap();
        assert_eq!(secret.bit_length(), 256);
    }

    #[test]
    fn test_single_group_shares() {
        let master_secret = MasterSecret::generate(128).unwrap();
        let passphrase = b"test passphrase";

        let slip39 = Slip39::new_single_group(3, 5).unwrap();
        let shares = slip39.generate_shares(&master_secret, passphrase).unwrap();

        assert_eq!(shares.len(), 1);
        assert_eq!(shares[0].len(), 5);

        let subset: Vec<Share> = shares[0].iter().take(3).cloned().collect();
        let reconstructed = Slip39::reconstruct_secret(&subset, passphrase).unwrap();

        assert_eq!(reconstructed.data, master_secret.data);
    }

    #[test]
    fn test_multi_group_shares() {
        let master_secret = MasterSecret::generate(128).unwrap();
        let passphrase = b"my secret";

        let groups = vec![
            GroupConfig::new(2, 3).unwrap(),
            GroupConfig::new(2, 5).unwrap(),
            GroupConfig::new(3, 5).unwrap(),
        ];

        let slip39 = Slip39::new(2, groups, None).unwrap();
        let shares = slip39.generate_shares(&master_secret, passphrase).unwrap();

        assert_eq!(shares.len(), 3);

        let mut reconstruction_shares = Vec::new();
        reconstruction_shares.extend(shares[0].iter().take(2).cloned());
        reconstruction_shares.extend(shares[2].iter().take(3).cloned());

        let reconstructed =
            Slip39::reconstruct_secret(&reconstruction_shares, passphrase).unwrap();

        assert_eq!(reconstructed.data, master_secret.data);
    }

    #[test]
    fn test_wrong_passphrase() {
        let master_secret = MasterSecret::generate(128).unwrap();
        let slip39 = Slip39::new_single_group(2, 3).unwrap();

        let shares = slip39
            .generate_shares(&master_secret, b"correct")
            .unwrap();

        let subset: Vec<Share> = shares[0].iter().take(2).cloned().collect();

        let result = Slip39::reconstruct_secret(&subset, b"wrong");
        assert!(result.is_ok());

        let wrong_secret = result.unwrap();
        assert_ne!(wrong_secret.data, master_secret.data);
    }

    #[test]
    fn test_insufficient_shares() {
        let master_secret = MasterSecret::generate(128).unwrap();
        let slip39 = Slip39::new_single_group(3, 5).unwrap();

        let shares = slip39.generate_shares(&master_secret, b"test").unwrap();

        let subset: Vec<Share> = shares[0].iter().take(2).cloned().collect();
        let result = Slip39::reconstruct_secret(&subset, b"test");

        if let Ok(reconstructed) = result {
            assert_ne!(reconstructed.data, master_secret.data);
        }
    }
}
