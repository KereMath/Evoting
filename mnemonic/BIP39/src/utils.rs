use crate::error::Result;

pub fn bytes_to_hex(bytes: &[u8]) -> String {
    hex::encode(bytes)
}

pub fn hex_to_bytes(hex: &str) -> Result<Vec<u8>> {
    Ok(hex::decode(hex)?)
}

pub fn is_valid_hex(hex: &str) -> bool {
    hex.len() % 2 == 0 && hex.chars().all(|c| c.is_ascii_hexdigit())
}

pub fn format_btc(satoshis: u64) -> String {
    let btc = satoshis as f64 / 100_000_000.0;
    format!("{:.8} BTC", btc)
}

pub fn parse_derivation_path(path: &str) -> Result<Vec<u32>> {
    let path = path.trim_start_matches("m/");
    let mut components = Vec::new();

    for part in path.split('/') {
        let (index, hardened) = if part.ends_with('\'') || part.ends_with('h') {
            (part.trim_end_matches('\'').trim_end_matches('h'), true)
        } else {
            (part, false)
        };

        let index: u32 = index.parse()
            .map_err(|_| crate::error::Bip39Error::InvalidPath(path.to_string()))?;

        components.push(if hardened { index + 0x80000000 } else { index });
    }

    Ok(components)
}

pub fn secure_random(size: usize) -> Result<Vec<u8>> {
    use rand::RngCore;
    let mut bytes = vec![0u8; size];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_conversion() {
        let bytes = vec![0x00, 0xFF, 0x42];
        let hex = bytes_to_hex(&bytes);
        assert_eq!(hex, "00ff42");

        let decoded = hex_to_bytes(&hex).unwrap();
        assert_eq!(decoded, bytes);
    }

    #[test]
    fn test_path_parsing() {
        let path = "m/44'/0'/0'/0/0";
        let components = parse_derivation_path(path).unwrap();

        assert_eq!(components.len(), 5);
        assert_eq!(components[0], 0x80000000 + 44);
        assert_eq!(components[4], 0);
    }
}
