use rand::Rng;
use zeroize::Zeroize;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use once_cell::sync::Lazy;

use crate::error::{Result, Slip39Error};

const DIGEST_LENGTH_BYTES: usize = 4;

const DIGEST_INDEX: u8 = 254;
const SECRET_INDEX: u8 = 255;

static EXP_TABLE: Lazy<[u8; 256]> = Lazy::new(|| {
    let mut exp = [0u8; 256];
    let mut poly: u16 = 1;
    for i in 0..255 {
        exp[i] = poly as u8;
        poly = (poly << 1) ^ poly;
        if poly & 0x100 != 0 {
            poly ^= 0x11B;
        }
    }
    exp[255] = exp[0];
    exp
});

static LOG_TABLE: Lazy<[u8; 256]> = Lazy::new(|| {
    let mut log = [0u8; 256];
    for (i, &exp_val) in EXP_TABLE.iter().enumerate().take(255) {
        log[exp_val as usize] = i as u8;
    }
    log
});

#[derive(Debug, Clone, Copy, PartialEq, Eq, Zeroize)]
pub struct GF256(pub u8);

const IRREDUCIBLE_POLY: u16 = 0x11B;

impl GF256 {
    pub const ZERO: Self = GF256(0);

    pub const ONE: Self = GF256(1);

    #[inline]
    pub const fn new(value: u8) -> Self {
        GF256(value)
    }

    #[inline]
    pub const fn value(self) -> u8 {
        self.0
    }

    #[inline]
    pub fn add(self, other: Self) -> Self {
        GF256(self.0 ^ other.0)
    }

    #[inline]
    pub fn sub(self, other: Self) -> Self {
        self.add(other)
    }

    pub fn mul(self, other: Self) -> Self {
        if self.0 == 0 || other.0 == 0 {
            return GF256::ZERO;
        }

        let mut result = 0u16;
        let mut a = self.0 as u16;
        let mut b = other.0 as u16;

        for _ in 0..8 {
            if b & 1 != 0 {
                result ^= a;
            }
            b >>= 1;

            let carry = a & 0x80;
            a <<= 1;

            if carry != 0 {
                a ^= IRREDUCIBLE_POLY;
            }
        }

        GF256((result & 0xFF) as u8)
    }

    pub fn div(self, other: Self) -> Option<Self> {
        if other.0 == 0 {
            return None;
        }
        Some(self.mul(other.inverse()?))
    }

    pub fn inverse(self) -> Option<Self> {
        if self.0 == 0 {
            return None;
        }

        let mut t = 0u16;
        let mut new_t = 1u16;
        let mut r = IRREDUCIBLE_POLY;
        let mut new_r = self.0 as u16;

        while new_r != 0 {
            let quotient = gf256_divide_poly(r, new_r);

            let temp_t = t;
            t = new_t;
            new_t = temp_t ^ gf256_multiply_poly(quotient, new_t);

            let temp_r = r;
            r = new_r;
            new_r = temp_r ^ gf256_multiply_poly(quotient, new_r);
        }

        Some(GF256((t & 0xFF) as u8))
    }

    pub fn pow(self, exp: u32) -> Self {
        if exp == 0 {
            return GF256::ONE;
        }

        let mut result = GF256::ONE;
        let mut base = self;
        let mut e = exp;

        while e > 0 {
            if e & 1 != 0 {
                result = result.mul(base);
            }
            base = base.mul(base);
            e >>= 1;
        }

        result
    }
}

fn gf256_divide_poly(dividend: u16, divisor: u16) -> u16 {
    if divisor == 0 {
        return 0;
    }

    let mut quotient = 0u16;
    let mut remainder = dividend;
    let divisor_degree = 15 - divisor.leading_zeros();

    while remainder != 0 {
        let remainder_degree = 15 - remainder.leading_zeros();
        if remainder_degree < divisor_degree {
            break;
        }

        let shift = remainder_degree - divisor_degree;
        quotient ^= 1u16 << shift;
        remainder ^= divisor << shift;
    }

    quotient
}

fn gf256_multiply_poly(a: u16, b: u16) -> u16 {
    let mut result = 0u16;
    let mut multiplicand = a;
    let mut multiplier = b;

    while multiplier != 0 {
        if multiplier & 1 != 0 {
            result ^= multiplicand;
        }
        multiplicand <<= 1;
        multiplier >>= 1;
    }

    result
}

#[derive(Debug)]
pub struct ShamirSecretSharing;

impl ShamirSecretSharing {
    fn create_digest(random_part: &[u8], secret: &[u8]) -> Vec<u8> {
        let mut mac = Hmac::<Sha256>::new_from_slice(random_part)
            .expect("HMAC can take key of any size");
        mac.update(secret);
        let result = mac.finalize();
        result.into_bytes()[..DIGEST_LENGTH_BYTES].to_vec()
    }

    pub fn split(
        secret: &[u8],
        threshold: u8,
        share_count: u8,
        x_coords: &[u8],
    ) -> Result<Vec<Vec<u8>>> {
        if threshold < 1 {
            return Err(Slip39Error::InvalidThreshold(
                "Threshold must be at least 1".to_string(),
            ));
        }

        if share_count < threshold {
            return Err(Slip39Error::InvalidThreshold(format!(
                "Share count ({}) must be >= threshold ({})",
                share_count, threshold
            )));
        }

        if x_coords.len() != share_count as usize {
            return Err(Slip39Error::InvalidShareData(format!(
                "X-coordinates length ({}) must match share count ({})",
                x_coords.len(),
                share_count
            )));
        }

        for (i, &x) in x_coords.iter().enumerate() {
            for &other_x in &x_coords[i + 1..] {
                if x == other_x {
                    return Err(Slip39Error::InvalidShareData(
                        "X-coordinates must be unique".to_string(),
                    ));
                }
            }
        }

        if threshold == 1 {
            return Ok(vec![secret.to_vec(); share_count as usize]);
        }

        if secret.len() <= DIGEST_LENGTH_BYTES {
            return Err(Slip39Error::InvalidShareData(format!(
                "Secret must be at least {} bytes for threshold > 1 (got {})",
                DIGEST_LENGTH_BYTES + 1,
                secret.len()
            )));
        }

        let random_share_count = (threshold - 2) as usize;

        let mut rng = rand::thread_rng();
        let mut base_shares: Vec<(u8, Vec<u8>)> = Vec::new();

        for i in 0..random_share_count {
            let random_share: Vec<u8> = (0..secret.len()).map(|_| rng.gen()).collect();
            base_shares.push((i as u8, random_share));
        }

        let random_part: Vec<u8> = (0..secret.len() - DIGEST_LENGTH_BYTES)
            .map(|_| rng.gen())
            .collect();
        let digest = Self::create_digest(&random_part, secret);

        let mut digest_share = digest.clone();
        digest_share.extend_from_slice(&random_part);
        base_shares.push((DIGEST_INDEX, digest_share));

        base_shares.push((SECRET_INDEX, secret.to_vec()));

        let mut shares = Vec::new();
        for &x in x_coords {
            let share_value = Self::interpolate_share(&base_shares, x)?;
            shares.push(share_value);
        }

        Ok(shares)
    }

    fn interpolate_share(base_shares: &[(u8, Vec<u8>)], x: u8) -> Result<Vec<u8>> {
        let secret_len = base_shares[0].1.len();
        let mut result = vec![0u8; secret_len];

        for byte_idx in 0..secret_len {
            let byte_shares: Vec<_> = base_shares
                .iter()
                .map(|(x_coord, share)| (*x_coord, share[byte_idx]))
                .collect();

            result[byte_idx] = Self::interpolate_byte(&byte_shares, x);
        }

        Ok(result)
    }

    fn interpolate_byte(points: &[(u8, u8)], x: u8) -> u8 {
        for &(x_i, y_i) in points {
            if x_i == x {
                return y_i;
            }
        }

        let log_prod: u32 = points
            .iter()
            .map(|&(x_i, _)| {
                let diff = x_i ^ x;
                if diff == 0 {
                    0
                } else {
                    LOG_TABLE[diff as usize] as u32
                }
            })
            .sum();

        let mut result: u8 = 0;

        for &(x_i, y_i) in points {
            let mut log_basis: i32 = log_prod as i32;

            let diff_i = x_i ^ x;
            log_basis -= LOG_TABLE[diff_i as usize] as i32;

            let sum_diffs: i32 = points
                .iter()
                .map(|&(x_j, _)| {
                    let diff = x_i ^ x_j;
                    LOG_TABLE[diff as usize] as i32
                })
                .sum();

            log_basis -= sum_diffs;

            log_basis = log_basis.rem_euclid(255);

            let contribution = if y_i == 0 {
                0
            } else {
                let log_y = LOG_TABLE[y_i as usize] as i32;
                let log_result = (log_y + log_basis).rem_euclid(255);
                EXP_TABLE[log_result as usize]
            };

            result ^= contribution;
        }

        result
    }

    #[allow(dead_code)]
    fn split_byte_old(secret: u8, threshold: u8, x_coords: &[u8]) -> Result<Vec<u8>> {
        let mut rng = rand::thread_rng();

        let mut coeffs = vec![GF256::new(secret)];

        for _ in 1..threshold {
            coeffs.push(GF256::new(rng.gen::<u8>()));
        }

        let mut shares = Vec::with_capacity(x_coords.len());

        for &x in x_coords {
            let y = Self::evaluate_polynomial(&coeffs, GF256::new(x));
            shares.push(y.value());
        }

        Ok(shares)
    }

    fn evaluate_polynomial(coeffs: &[GF256], x: GF256) -> GF256 {
        let mut result = GF256::ZERO;

        for &coeff in coeffs.iter().rev() {
            result = result.mul(x).add(coeff);
        }

        result
    }

    pub fn reconstruct(shares: &[(u8, Vec<u8>)]) -> Result<Vec<u8>> {
        if shares.is_empty() {
            return Err(Slip39Error::InsufficientShares {
                have: 0,
                need: 1,
            });
        }

        let secret_len = shares[0].1.len();
        for (_, share_value) in shares {
            if share_value.len() != secret_len {
                return Err(Slip39Error::IncompatibleShares(
                    "All shares must have the same length".to_string(),
                ));
            }
        }

        if shares.len() == 1 {
            return Ok(shares[0].1.clone());
        }

        let mut secret = vec![0u8; secret_len];

        for byte_idx in 0..secret_len {
            let byte_shares: Vec<(u8, u8)> = shares
                .iter()
                .map(|(x, y)| (*x, y[byte_idx]))
                .collect();

            secret[byte_idx] = Self::interpolate_byte(&byte_shares, SECRET_INDEX);
        }

        let mut digest_share = vec![0u8; secret_len];
        for byte_idx in 0..secret_len {
            let byte_shares: Vec<(u8, u8)> = shares
                .iter()
                .map(|(x, y)| (*x, y[byte_idx]))
                .collect();

            digest_share[byte_idx] = Self::interpolate_byte(&byte_shares, DIGEST_INDEX);
        }

        let reconstructed_digest = &digest_share[..DIGEST_LENGTH_BYTES];
        let random_part = &digest_share[DIGEST_LENGTH_BYTES..];

        let expected_digest = Self::create_digest(random_part, &secret);

        if reconstructed_digest != expected_digest.as_slice() {
            return Err(Slip39Error::DigestVerificationFailed);
        }

        Ok(secret)
    }

    fn reconstruct_byte(shares: &[(u8, u8)]) -> Result<u8> {
        let mut result = GF256::ZERO;

        for (j, &(x_j, y_j)) in shares.iter().enumerate() {
            let mut basis = GF256::ONE;

            for (m, &(x_m, _)) in shares.iter().enumerate() {
                if j == m {
                    continue;
                }

                let numerator = GF256::new(x_m);
                let denominator = GF256::new(x_j ^ x_m);

                if let Some(inv) = denominator.inverse() {
                    basis = basis.mul(numerator).mul(inv);
                } else {
                    return Err(Slip39Error::GroupReconstructionFailed(
                        "Division by zero in Lagrange interpolation".to_string(),
                    ));
                }
            }

            result = result.add(GF256::new(y_j).mul(basis));
        }

        Ok(result.value())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gf256_arithmetic() {
        let a = GF256::new(0x53);
        let b = GF256::new(0xCA);

        assert_eq!(a.add(b).value(), 0x53 ^ 0xCA);

        assert_eq!(a.mul(GF256::ONE), a);

        assert_eq!(a.mul(GF256::ZERO), GF256::ZERO);
    }

    #[test]
    fn test_gf256_inverse() {
        for i in 1..=255u8 {
            let a = GF256::new(i);
            if let Some(inv) = a.inverse() {
                assert_eq!(a.mul(inv), GF256::ONE);
            }
        }
    }

    #[test]
    fn test_shamir_split_reconstruct() {
        let secret = vec![0x01; 16];
        let threshold = 3;
        let share_count = 5;
        let x_coords = vec![1, 2, 3, 4, 5];

        let shares = ShamirSecretSharing::split(&secret, threshold, share_count, &x_coords).unwrap();

        let share_subset: Vec<(u8, Vec<u8>)> = shares
            .iter()
            .take(threshold as usize)
            .enumerate()
            .map(|(i, s)| (x_coords[i], s.clone()))
            .collect();

        let reconstructed = ShamirSecretSharing::reconstruct(&share_subset).unwrap();
        assert_eq!(reconstructed, secret);
    }

    #[test]
    fn test_shamir_any_threshold_shares() {
        let secret = vec![0xAA; 16];
        let threshold = 3;
        let share_count = 5;
        let x_coords = vec![1, 2, 3, 4, 5];

        let shares = ShamirSecretSharing::split(&secret, threshold, share_count, &x_coords).unwrap();

        let combinations = vec![
            vec![0, 1, 2],
            vec![0, 2, 4],
            vec![1, 3, 4],
            vec![2, 3, 4],
        ];

        for combo in combinations {
            let share_subset: Vec<(u8, Vec<u8>)> = combo
                .iter()
                .map(|&i| (x_coords[i], shares[i].clone()))
                .collect();

            let reconstructed = ShamirSecretSharing::reconstruct(&share_subset).unwrap();
            assert_eq!(reconstructed, secret);
        }
    }

    #[test]
    fn test_shamir_insufficient_shares() {
        let secret = vec![0x01; 16];
        let threshold = 3;
        let x_coords = vec![1, 2, 3, 4];

        let shares = ShamirSecretSharing::split(&secret, threshold, 4, &x_coords).unwrap();

        let insufficient: Vec<(u8, Vec<u8>)> = shares
            .iter()
            .take(2)
            .enumerate()
            .map(|(i, s)| (x_coords[i], s.clone()))
            .collect();

        let result = ShamirSecretSharing::reconstruct(&insufficient);
        assert!(result.is_err());
        assert!(matches!(result, Err(Slip39Error::DigestVerificationFailed)));
    }

    #[test]
    fn test_shamir_invalid_threshold() {
        let result = ShamirSecretSharing::split(&[0x01], 0, 5, &[1, 2, 3, 4, 5]);
        assert!(result.is_err());

        let result = ShamirSecretSharing::split(&[0x01], 6, 5, &[1, 2, 3, 4, 5]);
        assert!(result.is_err());
    }

    #[test]
    fn test_shamir_zero_x_coordinate() {
        let secret = vec![0x01; 16];
        let result = ShamirSecretSharing::split(&secret, 2, 3, &[0, 1, 2]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_shamir_duplicate_x_coordinates() {
        let result = ShamirSecretSharing::split(&[0x01], 2, 3, &[1, 1, 2]);
        assert!(result.is_err());
    }
}
