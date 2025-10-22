use crate::error::{Result, Slip39Error};
use crate::rs1024::RS1024;
use crate::wordlist;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(Debug, Clone, Zeroize, ZeroizeOnDrop)]
pub struct Share {
    pub identifier: u16,

    pub extendable: bool,

    pub iteration_exponent: u8,

    pub group_index: u8,

    pub group_threshold: u8,

    pub group_count: u8,

    pub member_index: u8,

    pub member_threshold: u8,

    pub value: Vec<u8>,
}

impl Share {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        identifier: u16,
        extendable: bool,
        iteration_exponent: u8,
        group_index: u8,
        group_threshold: u8,
        group_count: u8,
        member_index: u8,
        member_threshold: u8,
        value: Vec<u8>,
    ) -> Result<Self> {
        if identifier > 0x7FFF {
            return Err(Slip39Error::InvalidIdentifier(format!(
                "Identifier must be 15-bit: {}",
                identifier
            )));
        }

        if iteration_exponent > 15 {
            return Err(Slip39Error::InvalidShareData(
                "Iteration exponent must be 4-bit (0-15)".to_string(),
            ));
        }

        if group_index > 15 {
            return Err(Slip39Error::InvalidGroupConfig(format!(
                "Group index must be 0-15: {}",
                group_index
            )));
        }

        if group_threshold == 0 || group_threshold > 16 {
            return Err(Slip39Error::InvalidThreshold(format!(
                "Group threshold must be 1-16: {}",
                group_threshold
            )));
        }

        if group_count == 0 || group_count > 16 {
            return Err(Slip39Error::InvalidGroupConfig(format!(
                "Group count must be 1-16: {}",
                group_count
            )));
        }

        if group_threshold > group_count {
            return Err(Slip39Error::InvalidThreshold(format!(
                "Group threshold ({}) cannot exceed group count ({})",
                group_threshold, group_count
            )));
        }

        if member_index > 15 {
            return Err(Slip39Error::InvalidShareData(format!(
                "Member index must be 0-15: {}",
                member_index
            )));
        }

        if member_threshold == 0 || member_threshold > 16 {
            return Err(Slip39Error::InvalidThreshold(format!(
                "Member threshold must be 1-16: {}",
                member_threshold
            )));
        }

        Ok(Share {
            identifier,
            extendable,
            iteration_exponent,
            group_index,
            group_threshold,
            group_count,
            member_index,
            member_threshold,
            value,
        })
    }

    pub fn to_mnemonic(&self) -> Result<Vec<String>> {
        let words_data = self.to_words()?;
        let wordlist = wordlist::get_english_wordlist();

        let mut mnemonic = Vec::with_capacity(words_data.len());
        for &word_index in &words_data {
            let word = wordlist.get_word(word_index)?;
            mnemonic.push(word.to_string());
        }

        Ok(mnemonic)
    }

    pub fn from_mnemonic(words: &[String]) -> Result<Self> {
        let wordlist = wordlist::get_english_wordlist();
        let mut word_indices = Vec::with_capacity(words.len());

        for word in words {
            let index = wordlist.get_index(word)?;
            word_indices.push(index);
        }

        Self::from_words(&word_indices)
    }

    pub fn to_words(&self) -> Result<Vec<u16>> {
        let mut data = Vec::new();

        let id_ext_iter = ((self.identifier as u32) << 5)
            | ((self.extendable as u32) << 4)
            | (self.iteration_exponent as u32);

        data.push(((id_ext_iter >> 10) & 0x3FF) as u16);
        data.push((id_ext_iter & 0x3FF) as u16);

        let group_member_info = ((self.group_index as u32) << 16)
            | ((self.group_threshold.saturating_sub(1) as u32) << 12)
            | ((self.group_count.saturating_sub(1) as u32) << 8)
            | ((self.member_index as u32) << 4)
            | (self.member_threshold.saturating_sub(1) as u32);

        data.push(((group_member_info >> 10) & 0x3FF) as u16);
        data.push((group_member_info & 0x3FF) as u16);

        let value_bits = self.value.len() * 8;
        let padding_bits = (10 - (value_bits % 10)) % 10;

        let mut bit_buffer = 0u32;
        let mut bits_in_buffer = padding_bits;

        for &byte in &self.value {
            bit_buffer = (bit_buffer << 8) | (byte as u32);
            bits_in_buffer += 8;

            while bits_in_buffer >= 10 {
                bits_in_buffer -= 10;
                let word = ((bit_buffer >> bits_in_buffer) & 0x3FF) as u16;
                data.push(word);
            }
        }

        debug_assert_eq!(bits_in_buffer, 0);

        let rs = RS1024::new(if self.extendable {
            "shamir_extendable"
        } else {
            "shamir"
        });

        let checksum = rs.compute_checksum(&data);
        data.extend_from_slice(&checksum);

        Ok(data)
    }

    pub fn from_words(words: &[u16]) -> Result<Self> {
        if words.len() < 20 {
            return Err(Slip39Error::InvalidMnemonicLength(words.len()));
        }

        let id_ext_iter = ((words[0] as u32) << 10) | (words[1] as u32);

        let identifier = ((id_ext_iter >> 5) & 0x7FFF) as u16;
        let extendable = ((id_ext_iter >> 4) & 1) == 1;
        let iteration_exponent = (id_ext_iter & 0x0F) as u8;

        let rs = RS1024::new(if extendable {
            "shamir_extendable"
        } else {
            "shamir"
        });

        if !rs.verify_checksum(words) {
            return Err(Slip39Error::ChecksumFailed);
        }

        let data_words = &words[..words.len() - 3];
        let group_member_info = ((data_words[2] as u32) << 10) | (data_words[3] as u32);

        let group_index = ((group_member_info >> 16) & 0x0F) as u8;
        let group_threshold = (((group_member_info >> 12) & 0x0F) + 1) as u8;
        let group_count = (((group_member_info >> 8) & 0x0F) + 1) as u8;
        let member_index = ((group_member_info >> 4) & 0x0F) as u8;
        let member_threshold = ((group_member_info & 0x0F) + 1) as u8;

        let value_words = &data_words[4..];

        let total_bits = value_words.len() * 10;
        let padding_bits = total_bits % 16;
        let value_bits = total_bits - padding_bits;
        let value_bytes = value_bits / 8;

        let mut value = Vec::new();
        let mut bit_buffer = 0u32;
        let mut bits_in_buffer = 0;

        let mut bits_to_skip = padding_bits;

        for &word in value_words {
            bit_buffer = (bit_buffer << 10) | (word as u32);
            bits_in_buffer += 10;

            if bits_to_skip > 0 {
                if bits_to_skip >= bits_in_buffer {
                    bits_to_skip -= bits_in_buffer;
                    bits_in_buffer = 0;
                    bit_buffer = 0;
                    continue;
                } else {
                    bits_in_buffer -= bits_to_skip;
                    bit_buffer &= (1 << bits_in_buffer) - 1;
                    bits_to_skip = 0;
                }
            }

            while bits_in_buffer >= 8 {
                bits_in_buffer -= 8;
                let byte = ((bit_buffer >> bits_in_buffer) & 0xFF) as u8;
                value.push(byte);
            }
        }

        debug_assert_eq!(value.len(), value_bytes);

        Share::new(
            identifier,
            extendable,
            iteration_exponent,
            group_index,
            group_threshold,
            group_count,
            member_index,
            member_threshold,
            value,
        )
    }

    pub fn to_mnemonic_string(&self) -> Result<String> {
        let words = self.to_mnemonic()?;
        Ok(words.join(" "))
    }

    pub fn from_mnemonic_string(mnemonic: &str) -> Result<Self> {
        let words: Vec<String> = mnemonic
            .split_whitespace()
            .map(|s| s.to_string())
            .collect();

        Self::from_mnemonic(&words)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_share_creation() {
        let share = Share::new(
            12345,
            false,
            0,
            0,
            1,
            1,
            0,
            1,
            vec![0x01, 0x02, 0x03, 0x04],
        );
        assert!(share.is_ok());
    }

    #[test]
    fn test_share_invalid_identifier() {
        let share = Share::new(
            0x8000,
            false,
            0,
            0,
            1,
            1,
            0,
            1,
            vec![0x01],
        );
        assert!(share.is_err());
    }

    #[test]
    fn test_share_roundtrip() {
        let original = Share::new(
            12345,
            false,
            2,
            0,
            2,
            3,
            1,
            2,
            vec![0x01; 16],
        )
        .unwrap();

        let words = original.to_words().unwrap();
        let restored = Share::from_words(&words).unwrap();

        assert_eq!(restored.identifier, original.identifier);
        assert_eq!(restored.extendable, original.extendable);
        assert_eq!(restored.iteration_exponent, original.iteration_exponent);
        assert_eq!(restored.group_index, original.group_index);
        assert_eq!(restored.group_threshold, original.group_threshold);
        assert_eq!(restored.group_count, original.group_count);
        assert_eq!(restored.member_index, original.member_index);
        assert_eq!(restored.member_threshold, original.member_threshold);
    }

    #[test]
    fn test_share_mnemonic_roundtrip() {
        let original = Share::new(
            12345,
            false,
            0,
            0,
            1,
            1,
            0,
            1,
            vec![0xFF; 16],
        )
        .unwrap();

        let mnemonic = original.to_mnemonic().unwrap();
        let restored = Share::from_mnemonic(&mnemonic).unwrap();

        assert_eq!(restored.identifier, original.identifier);
        assert_eq!(restored.value, original.value);
    }

    #[test]
    fn test_share_checksum_detection() {
        let share = Share::new(12345, false, 0, 0, 1, 1, 0, 1, vec![0x01, 0x02]).unwrap();

        let mut words = share.to_words().unwrap();

        words[5] ^= 1;

        let result = Share::from_words(&words);
        assert!(result.is_err());
    }
}
