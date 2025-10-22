
use crate::error::{Result, ShamirError};
use std::ops::{Add, Sub, Mul, Div};

const IRREDUCIBLE_POLY: u16 = 0x11B;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GF256(pub u8);

impl GF256 {

    pub fn new(value: u8) -> Self {
        GF256(value)
    }

    pub const ZERO: GF256 = GF256(0);

    pub const ONE: GF256 = GF256(1);

    pub fn value(&self) -> u8 {
        self.0
    }

    pub fn inverse(&self) -> Result<GF256> {
        if self.0 == 0 {
            return Err(ShamirError::GaloisFieldError(
                "Cannot compute inverse of zero".to_string()
            ));
        }

        let mut t = 0u16;
        let mut new_t = 1u16;
        let mut r = IRREDUCIBLE_POLY;
        let mut new_r = self.0 as u16;

        while new_r != 0 {
            let quotient = gf_divide_poly(r, new_r);

            let temp_t = t;
            t = new_t;
            new_t = temp_t ^ gf_multiply_poly(quotient, new_t);

            let temp_r = r;
            r = new_r;
            new_r = temp_r ^ gf_multiply_poly(quotient, new_r);
        }

        if r > 0xFF {
            return Err(ShamirError::GaloisFieldError(
                "Inverse computation failed".to_string()
            ));
        }

        Ok(GF256(t as u8))
    }

    pub fn pow(&self, mut exponent: u32) -> GF256 {
        if exponent == 0 {
            return GF256::ONE;
        }

        let mut result = GF256::ONE;
        let mut base = *self;

        while exponent > 0 {
            if exponent & 1 == 1 {
                result = result * base;
            }
            base = base * base;
            exponent >>= 1;
        }

        result
    }
}

fn gf_divide_poly(a: u16, b: u16) -> u16 {
    if b == 0 {
        return 0;
    }

    let mut quotient = 0u16;
    let mut remainder = a;
    let b_degree = 15 - b.leading_zeros();

    while remainder >= b {
        let remainder_degree = 15 - remainder.leading_zeros();
        let shift = remainder_degree - b_degree;
        quotient ^= 1 << shift;
        remainder ^= b << shift;
    }

    quotient
}

fn gf_multiply_poly(a: u16, b: u16) -> u16 {
    let mut result = 0u16;
    let mut a = a;
    let mut b = b;

    while b != 0 {
        if b & 1 == 1 {
            result ^= a;
        }
        a <<= 1;
        b >>= 1;
    }

    result
}

impl Add for GF256 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        GF256(self.0 ^ other.0)
    }
}

impl Sub for GF256 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        GF256(self.0 ^ other.0)
    }
}

impl Mul for GF256 {
    type Output = Self;

    fn mul(self, other: Self) -> Self {
        let mut result = 0u16;
        let mut a = self.0 as u16;
        let mut b = other.0 as u16;

        for _ in 0..8 {
            if b & 1 == 1 {
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
}

impl Div for GF256 {
    type Output = Result<Self>;

    fn div(self, other: Self) -> Result<Self> {
        let inv = other.inverse()?;
        Ok(self * inv)
    }
}

impl std::fmt::Display for GF256 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GF256({})", self.0)
    }
}

impl From<u8> for GF256 {
    fn from(value: u8) -> Self {
        GF256(value)
    }
}

impl From<GF256> for u8 {
    fn from(gf: GF256) -> Self {
        gf.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_addition() {
        assert_eq!(GF256(3) + GF256(5), GF256(6));
        assert_eq!(GF256(255) + GF256(255), GF256(0));
    }

    #[test]
    fn test_subtraction() {
        assert_eq!(GF256(6) - GF256(3), GF256(5));
    }

    #[test]
    fn test_multiplication() {
        assert_eq!(GF256(0) * GF256(5), GF256(0));
        assert_eq!(GF256(1) * GF256(5), GF256(5));
        assert_eq!(GF256(2) * GF256(3), GF256(6));
    }

    #[test]
    fn test_multiplicative_inverse() {
        for i in 1..=255u8 {
            let a = GF256(i);
            let inv = a.inverse().unwrap();
            assert_eq!(a * inv, GF256::ONE);
        }
    }

    #[test]
    fn test_division() {
        let a = GF256(10);
        let b = GF256(5);
        let result = (a / b).unwrap();
        assert_eq!(result * b, a);
    }

    #[test]
    fn test_power() {
        assert_eq!(GF256(2).pow(0), GF256::ONE);
        assert_eq!(GF256(2).pow(1), GF256(2));
        assert_eq!(GF256(2).pow(2), GF256(4));
    }

    #[test]
    fn test_zero_inverse_fails() {
        assert!(GF256::ZERO.inverse().is_err());
    }
}
