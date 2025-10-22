use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GF1024(pub u16);

const IRREDUCIBLE_POLY: u16 = 0x409;

const GENERATOR: [u32; 10] = [
    0xE0E040,
    0x1C1C080,
    0x3838100,
    0x7070200,
    0xE0E0009,
    0x1C0C2412,
    0x38086C24,
    0x3090FC48,
    0x21B1F890,
    0x3F3F120,
];

impl GF1024 {
    #[inline]
    pub const fn new(value: u16) -> Self {
        GF1024(value & 0x3FF)
    }

    pub const ZERO: Self = GF1024(0);

    pub const ONE: Self = GF1024(1);

    #[inline]
    pub const fn value(self) -> u16 {
        self.0
    }

    #[inline]
    pub fn add(self, other: Self) -> Self {
        GF1024::new(self.0 ^ other.0)
    }

    #[inline]
    pub fn sub(self, other: Self) -> Self {
        self.add(other)
    }

    pub fn mul(self, other: Self) -> Self {
        if self.0 == 0 || other.0 == 0 {
            return GF1024::ZERO;
        }

        let mut result = 0u16;
        let mut a = self.0;
        let mut b = other.0;

        for _ in 0..10 {
            if b & 1 != 0 {
                result ^= a;
            }
            b >>= 1;

            let carry = a & 0x200;
            a <<= 1;

            if carry != 0 {
                a ^= IRREDUCIBLE_POLY;
            }
        }

        GF1024::new(result)
    }

    pub fn inverse(self) -> Option<Self> {
        if self.0 == 0 {
            return None;
        }

        let mut t = 0u32;
        let mut new_t = 1u32;
        let mut r = IRREDUCIBLE_POLY as u32;
        let mut new_r = self.0 as u32;

        while new_r != 0 {
            let quotient = gf1024_divide_poly(r, new_r);

            let temp_t = t;
            t = new_t;
            new_t = temp_t ^ gf1024_multiply_poly(quotient, new_t);

            let temp_r = r;
            r = new_r;
            new_r = temp_r ^ gf1024_multiply_poly(quotient, new_r);
        }

        if r > 0x3FF {
            None
        } else {
            Some(GF1024::new(t as u16))
        }
    }
}

fn gf1024_divide_poly(dividend: u32, divisor: u32) -> u32 {
    if divisor == 0 {
        return 0;
    }

    let mut quotient = 0u32;
    let mut remainder = dividend;
    let divisor_degree = 31 - divisor.leading_zeros();

    while remainder != 0 {
        let remainder_degree = 31 - remainder.leading_zeros();
        if remainder_degree < divisor_degree {
            break;
        }

        let shift = remainder_degree - divisor_degree;
        quotient |= 1u32 << shift;
        remainder ^= divisor << shift;
    }

    quotient
}

fn gf1024_multiply_poly(a: u32, b: u32) -> u32 {
    let mut result = 0u32;
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

impl fmt::Display for GF1024 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:03x}", self.0)
    }
}

#[derive(Debug)]
pub struct RS1024 {
    customization: Vec<u16>,
}

impl RS1024 {
    pub fn new(customization: &str) -> Self {
        let customization: Vec<u16> = customization
            .chars()
            .map(|c| (c as u8) as u16)
            .collect();

        RS1024 { customization }
    }

    pub fn compute_checksum(&self, data: &[u16]) -> [u16; 3] {
        let mut values = Vec::with_capacity(self.customization.len() + data.len() + 3);

        values.extend_from_slice(&self.customization);

        values.extend_from_slice(data);

        values.extend_from_slice(&[0, 0, 0]);

        let residue = self.polymod(&values) ^ 1;

        [
            ((residue >> 20) & 0x3FF) as u16,
            ((residue >> 10) & 0x3FF) as u16,
            (residue & 0x3FF) as u16,
        ]
    }

    pub fn verify_checksum(&self, data: &[u16]) -> bool {
        if data.len() < 3 {
            return false;
        }

        let mut values = Vec::with_capacity(self.customization.len() + data.len());

        values.extend_from_slice(&self.customization);

        values.extend_from_slice(data);

        self.polymod(&values) == 1
    }

    fn polymod(&self, values: &[u16]) -> u32 {
        let mut chk = 1u32;

        for &value in values {
            let b = ((chk >> 20) & 0x3FF) as u16;
            chk = ((chk & 0xFFFFF) << 10) ^ (value as u32);

            for i in 0..10 {
                if (b >> i) & 1 != 0 {
                    chk ^= GENERATOR[i];
                }
            }
        }

        chk
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gf1024_addition() {
        let a = GF1024::new(0x123);
        let b = GF1024::new(0x321);
        let c = a.add(b);
        assert_eq!(c.0, 0x123 ^ 0x321);
    }

    #[test]
    fn test_gf1024_multiplication() {
        let a = GF1024::new(0x1FF);
        let one = GF1024::ONE;
        assert_eq!(a.mul(one), a);

        let zero = GF1024::ZERO;
        assert_eq!(a.mul(zero), zero);
    }

    #[test]
    fn test_gf1024_inverse() {
        let a = GF1024::new(0x123);
        if let Some(inv) = a.inverse() {
            let product = a.mul(inv);
            assert_eq!(product, GF1024::ONE);
        }
    }

    #[test]
    fn test_rs1024_checksum_basic() {
        let rs = RS1024::new("shamir");
        let data = vec![0x000, 0x001, 0x002, 0x003];
        let checksum = rs.compute_checksum(&data);

        let mut full_data = data.clone();
        full_data.extend_from_slice(&checksum);

        assert!(rs.verify_checksum(&full_data));
    }

    #[test]
    fn test_rs1024_error_detection() {
        let rs = RS1024::new("shamir");
        let data = vec![0x000, 0x001, 0x002, 0x003];
        let checksum = rs.compute_checksum(&data);

        let mut full_data = data.clone();
        full_data.extend_from_slice(&checksum);

        full_data[1] ^= 0x001;

        assert!(!rs.verify_checksum(&full_data));
    }

    #[test]
    fn test_rs1024_customization() {
        let rs1 = RS1024::new("shamir");
        let rs2 = RS1024::new("shamir_extendable");

        let data = vec![0x000, 0x001, 0x002];

        let checksum1 = rs1.compute_checksum(&data);
        let checksum2 = rs2.compute_checksum(&data);

        assert_ne!(checksum1, checksum2);
    }
}
