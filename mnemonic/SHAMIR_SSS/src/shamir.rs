
use crate::error::{Result, ShamirError};
use crate::galois::GF256;
use hmac::{Hmac, Mac};
use rand::RngCore;
use sha2::Sha256;
use zeroize::{Zeroize, ZeroizeOnDrop};

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Zeroize, ZeroizeOnDrop)]
pub struct Share {

    pub id: u8,

    #[zeroize(skip)]
    pub value: Vec<GF256>,
}

impl Share {
    pub fn new(id: u8, value: Vec<GF256>) -> Self {
        Share { id, value }
    }
}

pub struct ShamirSSS {

    threshold: usize,

    total_shares: usize,

    digest_bytes: usize,
}

impl ShamirSSS {

    pub fn new(threshold: usize, total_shares: usize) -> Result<Self> {
        Self::with_digest_bytes(threshold, total_shares, 4)
    }

    pub fn with_digest_bytes(
        threshold: usize,
        total_shares: usize,
        digest_bytes: usize,
    ) -> Result<Self> {

        if threshold < 2 {
            return Err(ShamirError::InvalidThreshold(
                "Threshold must be at least 2".to_string(),
            ));
        }

        if threshold > total_shares {
            return Err(ShamirError::InvalidThreshold(format!(
                "Threshold ({}) cannot exceed total shares ({})",
                threshold, total_shares
            )));
        }

        if total_shares > 255 {
            return Err(ShamirError::InvalidShareCount(
                "Cannot create more than 255 shares (GF256 limitation)".to_string(),
            ));
        }

        Ok(ShamirSSS {
            threshold,
            total_shares,
            digest_bytes,
        })
    }

    pub fn split(&self, secret: &[u8]) -> Result<Vec<Share>> {
        let secret_len = secret.len();

        let mut secret_gf: Vec<GF256> = secret.iter().map(|&b| GF256::new(b)).collect();

        let digest = self.generate_digest(&secret_gf)?;

        let mut shares = vec![Vec::new(); self.total_shares];

        for &byte_gf in &secret_gf {
            let byte_shares = self.split_byte(byte_gf)?;
            for (i, share_val) in byte_shares.into_iter().enumerate() {
                shares[i].push(share_val);
            }
        }

        for &byte_gf in &digest {
            let byte_shares = self.split_byte(byte_gf)?;
            for (i, share_val) in byte_shares.into_iter().enumerate() {
                shares[i].push(share_val);
            }
        }

        secret_gf.iter_mut().for_each(|v| *v = GF256::ZERO);

        Ok(shares
            .into_iter()
            .enumerate()
            .map(|(i, value)| Share::new((i + 1) as u8, value))
            .collect())
    }

    fn split_byte(&self, secret_byte: GF256) -> Result<Vec<GF256>> {

        let mut coefficients = vec![secret_byte];

        let mut rng = rand::thread_rng();
        for _ in 1..self.threshold {
            let mut random_byte = 0u8;
            loop {
                rng.fill_bytes(std::slice::from_mut(&mut random_byte));

                if random_byte != 0 {
                    break;
                }
            }
            coefficients.push(GF256::new(random_byte));
        }

        let mut shares = Vec::with_capacity(self.total_shares);
        for x in 1..=self.total_shares {
            let x_gf = GF256::new(x as u8);
            let y = evaluate_polynomial(&coefficients, x_gf);
            shares.push(y);
        }

        Ok(shares)
    }

    pub fn reconstruct(&self, shares: &[Share]) -> Result<Vec<u8>> {

        if shares.len() < self.threshold {
            return Err(ShamirError::InsufficientShares {
                have: shares.len(),
                need: self.threshold,
            });
        }

        let share_len = shares[0].value.len();
        if !shares.iter().all(|s| s.value.len() == share_len) {
            return Err(ShamirError::InvalidShareFormat(
                "All shares must have the same length".to_string(),
            ));
        }

        let shares = &shares[..self.threshold];

        let mut reconstructed = Vec::with_capacity(share_len);

        for byte_pos in 0..share_len {

            let points: Vec<(GF256, GF256)> = shares
                .iter()
                .map(|share| (GF256::new(share.id), share.value[byte_pos]))
                .collect();

            let secret_byte = lagrange_interpolate(&points, GF256::ZERO)?;
            reconstructed.push(secret_byte.value());
        }

        if reconstructed.len() < self.digest_bytes {
            return Err(ShamirError::ReconstructionFailed(
                "Reconstructed data too short".to_string(),
            ));
        }

        let digest_start = reconstructed.len() - self.digest_bytes;
        let secret = &reconstructed[..digest_start];
        let digest = &reconstructed[digest_start..];

        let secret_gf: Vec<GF256> = secret.iter().map(|&b| GF256::new(b)).collect();
        let expected_digest = self.generate_digest(&secret_gf)?;

        if digest != expected_digest.iter().map(|g| g.value()).collect::<Vec<_>>() {
            return Err(ShamirError::DigestVerificationFailed);
        }

        Ok(secret.to_vec())
    }

    fn generate_digest(&self, secret: &[GF256]) -> Result<Vec<GF256>> {
        use sha2::{Digest, Sha256};

        let secret_bytes: Vec<u8> = secret.iter().map(|g| g.value()).collect();

        let hash = Sha256::digest(&secret_bytes);

        let mut digest = Vec::new();
        for i in 0..self.digest_bytes.min(hash.len()) {
            digest.push(GF256::new(hash[i]));
        }

        Ok(digest)
    }
}

fn evaluate_polynomial(coefficients: &[GF256], x: GF256) -> GF256 {
    let mut result = GF256::ZERO;
    let mut x_power = GF256::ONE;

    for &coeff in coefficients {
        result = result + (coeff * x_power);
        x_power = x_power * x;
    }

    result
}

fn lagrange_interpolate(points: &[(GF256, GF256)], x: GF256) -> Result<GF256> {
    let mut result = GF256::ZERO;

    for (i, &(xi, yi)) in points.iter().enumerate() {

        let mut li = GF256::ONE;

        for (j, &(xj, _)) in points.iter().enumerate() {
            if i != j {

                let numerator = x - xj;
                let denominator = xi - xj;
                let quotient = (numerator / denominator)?;
                li = li * quotient;
            }
        }

        result = result + (yi * li);
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_and_reconstruct() {
        let sss = ShamirSSS::new(3, 5).unwrap();
        let secret = b"Hello, Shamir!";

        let shares = sss.split(secret).unwrap();
        assert_eq!(shares.len(), 5);

        let reconstructed1 = sss.reconstruct(&shares[0..3]).unwrap();
        assert_eq!(reconstructed1, secret);

        let reconstructed2 = sss.reconstruct(&[shares[0].clone(), shares[2].clone(), shares[4].clone()]).unwrap();
        assert_eq!(reconstructed2, secret);
    }

    #[test]
    fn test_insufficient_shares_fails() {
        let sss = ShamirSSS::new(3, 5).unwrap();
        let secret = b"Test";

        let shares = sss.split(secret).unwrap();
        let result = sss.reconstruct(&shares[0..2]);

        assert!(result.is_err());
    }

    #[test]
    fn test_evaluate_polynomial() {
        let coeffs = vec![GF256(5), GF256(2), GF256(3)];
        let result = evaluate_polynomial(&coeffs, GF256(0));
        assert_eq!(result, GF256(5));
    }

    #[test]
    fn test_lagrange_interpolation() {

        let coeffs = vec![GF256(5), GF256(2), GF256(3)];

        let points: Vec<(GF256, GF256)> = (1..=3)
            .map(|x| {
                let x_gf = GF256(x);
                let y = evaluate_polynomial(&coeffs, x_gf);
                (x_gf, y)
            })
            .collect();

        let result = lagrange_interpolate(&points, GF256::ZERO).unwrap();
        assert_eq!(result, GF256(5));
    }
}
