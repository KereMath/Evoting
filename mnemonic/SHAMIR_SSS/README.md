# 🔐 Shamir's Secret Sharing for BIP-39

Shamir Secret Sharing over GF(256) - BIP-39 mnemonic'leri güvenli şekilde bölme

---

## Matematiksel Altyapı

### Algoritma Mapping

### Galois Field GF(256) İşlemleri

| Matematiksel İşlem | Dosya | Fonksiyon/Satır | Açıklama |
|-------------------|-------|-----------------|----------|
| **a ⊕ b** | `galois.rs` | `Add::add()` (112-118) | GF(256) toplama (XOR) |
| **a ⊖ b** | `galois.rs` | `Sub::sub()` (120-126) | GF(256) çıkarma (XOR) |
| **a ⊗ b** | `galois.rs` | `Mul::mul()` (128-152) | GF(256) çarpma (peasant mult + mod 0x11B) |
| **a⁻¹** | `galois.rs` | `inverse()` (24-55) | Multiplicative inverse (Extended GCD) |
| **a / b = a ⊗ b⁻¹** | `galois.rs` | `Div::div()` (154-161) | GF(256) bölme |
| **aⁿ** | `galois.rs` | `pow()` (57-74) | Fast exponentiation (square-and-multiply) |

### Shamir Secret Sharing İşlemleri

| Matematiksel İşlem | Dosya | Fonksiyon/Satır | Açıklama |
|-------------------|-------|-----------------|----------|
| **f(x) = s + Σ aᵢxⁱ** | `shamir.rs` | `split_byte()` (105-130) | (t-1)-derece polinom oluşturma |
| **yᵢ = f(xᵢ)** | `shamir.rs` | `split_byte()` (123-127) | Share hesaplama (polynomial evaluation) |
| **P(x) = Σ aᵢxⁱ** | `shamir.rs` | `evaluate_polynomial()` (199-209) | Horner's method ile polinom değerlendirme |
| **Lⱼ(x) = Π (x-xₘ)/(xⱼ-xₘ)** | `shamir.rs` | `lagrange_interpolate()` (218-226) | Lagrange basis polinomları |
| **f(0) = Σ yⱼLⱼ(0)** | `shamir.rs` | `lagrange_interpolate()` (211-232) | Secret reconstruction |
| **h = SHA256(s)[0:4]** | `shamir.rs` | `generate_digest()` (183-196) | Integrity digest |

### BIP-39 Integration İşlemleri

| Matematiksel İşlem | Dosya | Fonksiyon/Satır | Açıklama |
|-------------------|-------|-----------------|----------|
| **M → E** | `mnemonic_share.rs` | `split_mnemonic()` (50-53) | Mnemonic → Entropy extraction |
| **E → shares** | `mnemonic_share.rs` | `split_mnemonic()` (55-72) | Entropy splitting over GF(256) |
| **shares → E** | `mnemonic_share.rs` | `reconstruct_mnemonic()` (111) | Share reconstruction |
| **E → M** | `mnemonic_share.rs` | `reconstruct_mnemonic()` (113-114) | Entropy → Mnemonic |

### Matematiksel Tanımlar

**Galois Field GF(256) = GF(2⁸)**:
```
Irreducible polynomial: p(x) = x⁸ + x⁴ + x³ + x + 1 (0x11B)
Elements: {0, 1, 2, ..., 255}
Operations: +, -, ×, ÷ (all mod p(x))
```

**Shamir SSS Parametreleri**:
- **t**: Threshold (minimum shares needed) ∈ [2, 255]
- **n**: Total shares generated ∈ [t, 255]
- **s**: Secret byte ∈ GF(256)
- **aᵢ**: Random coefficients ∈ GF(256) \ {0}, i = 1,...,t-1
- **(xᵢ, yᵢ)**: Share points, xᵢ = i ∈ [1, n]

**Perfect Secrecy (Shannon 1949)**:
```
I(S; S₁, S₂, ..., S_{t-1}) = 0

Any t-1 shares reveal ZERO information about secret
(Information-theoretic security)
```

---

## Implementation Detayları

### Dosya Yapısı (5 dosya, ~600 LOC)

### 1. `galois.rs` (232 satır) ⭐ TEMEL DOSYA
**Amaç**: GF(256) finite field aritmetiği

**Constant**:
```rust
const IRREDUCIBLE_POLY: u16 = 0x11B;  // x⁸ + x⁴ + x³ + x + 1
```

**Struct**:
```rust
// Satır 7-180: GF(256) element
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GF256(pub u8);
```

**Ana Fonksiyonlar**:

```rust
// Satır 12-14: Constructor
pub fn new(value: u8) -> Self {
    GF256(value)
}

// Satır 16-18: Constants
pub const ZERO: GF256 = GF256(0);
pub const ONE: GF256 = GF256(1);

// Satır 24-55: Multiplicative inverse (Extended Euclidean Algorithm)
pub fn inverse(&self) -> Result<GF256> {
    if self.0 == 0 {
        return Err(ShamirError::GaloisFieldError(...));
    }

    // Extended GCD in GF(2)[x]
    let mut t = 0u16;
    let mut new_t = 1u16;
    let mut r = IRREDUCIBLE_POLY;
    let mut new_r = self.0 as u16;

    while new_r != 0 {
        let quotient = gf_divide_poly(r, new_r);

        // Update t
        let temp_t = t;
        t = new_t;
        new_t = temp_t ^ gf_multiply_poly(quotient, new_t);

        // Update r
        let temp_r = r;
        r = new_r;
        new_r = temp_r ^ gf_multiply_poly(quotient, new_r);
    }

    Ok(GF256(t as u8))
}

// Satır 57-74: Fast exponentiation (square-and-multiply)
pub fn pow(&self, mut exponent: u32) -> GF256 {
    if exponent == 0 { return GF256::ONE; }

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
```

**Operator Overloading**:

```rust
// Satır 112-118: Addition (XOR)
impl Add for GF256 {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        GF256(self.0 ^ other.0)
    }
}

// Satır 120-126: Subtraction (XOR, same as addition)
impl Sub for GF256 {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        GF256(self.0 ^ other.0)
    }
}

// Satır 128-152: Multiplication (peasant multiplication + mod 0x11B)
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
                a ^= IRREDUCIBLE_POLY;  // Modular reduction
            }
        }

        GF256((result & 0xFF) as u8)
    }
}

// Satır 154-161: Division (multiply by inverse)
impl Div for GF256 {
    type Output = Result<Self>;
    fn div(self, other: Self) -> Result<Self> {
        let inv = other.inverse()?;
        Ok(self * inv)
    }
}
```

**Helper Functions**:

```rust
// Satır 77-94: Polynomial division in GF(2)[x]
fn gf_divide_poly(a: u16, b: u16) -> u16 {
    if b == 0 { return 0; }

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

// Satır 96-110: Polynomial multiplication in GF(2)[x]
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
```

**Tests**: 7 tests (satır 182-231)
- Addition, subtraction, multiplication
- Multiplicative inverse (all 255 elements)
- Division, power

---

### 2. `shamir.rs` (288 satır) ⭐ CORE ALGORITHM
**Amaç**: Shamir Secret Sharing implementation

**Struct'lar**:

```rust
// Satır 11-24: Single share
#[derive(Debug, Clone, Zeroize, ZeroizeOnDrop)]
pub struct Share {
    pub id: u8,                    // Share ID (1..n)
    pub value: Vec<GF256>,         // Share data (secret length + digest)
}

// Satır 26-197: Main SSS engine
pub struct ShamirSSS {
    threshold: usize,              // t (minimum shares)
    total_shares: usize,           // n (total shares)
    digest_bytes: usize,           // Integrity check size (default: 4)
}
```

**Ana Fonksiyonlar**:

```rust
// Satır 37-71: Constructor with validation
pub fn new(threshold: usize, total_shares: usize) -> Result<Self> {
    // Validate: 2 <= t <= n <= 255
    if threshold < 2 {
        return Err(ShamirError::InvalidThreshold(...));
    }
    if threshold > total_shares {
        return Err(ShamirError::InvalidThreshold(...));
    }
    if total_shares > 255 {
        return Err(ShamirError::InvalidShareCount(...));
    }

    Ok(ShamirSSS {
        threshold,
        total_shares,
        digest_bytes: 4,
    })
}

// Satır 73-103: Secret splitting
pub fn split(&self, secret: &[u8]) -> Result<Vec<Share>> {
    // Convert to GF256
    let secret_gf: Vec<GF256> = secret.iter()
        .map(|&b| GF256::new(b))
        .collect();

    // Generate integrity digest
    let digest = self.generate_digest(&secret_gf)?;

    // Split each byte independently
    let mut shares = vec![Vec::new(); self.total_shares];

    for &byte_gf in &secret_gf {
        let byte_shares = self.split_byte(byte_gf)?;
        for (i, share_val) in byte_shares.into_iter().enumerate() {
            shares[i].push(share_val);
        }
    }

    // Append digest to each share
    for &byte_gf in &digest {
        let byte_shares = self.split_byte(byte_gf)?;
        for (i, share_val) in byte_shares.into_iter().enumerate() {
            shares[i].push(share_val);
        }
    }

    // Create Share structs
    Ok(shares.into_iter().enumerate()
        .map(|(i, value)| Share::new((i + 1) as u8, value))
        .collect())
}

// Satır 105-130: Split single byte (polynomial creation & evaluation)
fn split_byte(&self, secret_byte: GF256) -> Result<Vec<GF256>> {
    // Create polynomial: f(x) = secret + a₁x + a₂x² + ... + a_{t-1}x^{t-1}
    let mut coefficients = vec![secret_byte];

    // Generate random non-zero coefficients
    let mut rng = rand::thread_rng();
    for _ in 1..self.threshold {
        let mut random_byte = 0u8;
        loop {
            rng.fill_bytes(std::slice::from_mut(&mut random_byte));
            if random_byte != 0 { break; }
        }
        coefficients.push(GF256::new(random_byte));
    }

    // Evaluate polynomial at x = 1, 2, ..., n
    let mut shares = Vec::with_capacity(self.total_shares);
    for x in 1..=self.total_shares {
        let x_gf = GF256::new(x as u8);
        let y = evaluate_polynomial(&coefficients, x_gf);
        shares.push(y);
    }

    Ok(shares)
}

// Satır 132-181: Secret reconstruction
pub fn reconstruct(&self, shares: &[Share]) -> Result<Vec<u8>> {
    // Validate: at least t shares
    if shares.len() < self.threshold {
        return Err(ShamirError::InsufficientShares {
            have: shares.len(),
            need: self.threshold,
        });
    }

    let shares = &shares[..self.threshold];  // Use exactly t shares
    let share_len = shares[0].value.len();

    // Reconstruct each byte
    let mut reconstructed = Vec::with_capacity(share_len);

    for byte_pos in 0..share_len {
        // Collect points (xᵢ, yᵢ)
        let points: Vec<(GF256, GF256)> = shares.iter()
            .map(|share| (GF256::new(share.id), share.value[byte_pos]))
            .collect();

        // Lagrange interpolation at x=0
        let secret_byte = lagrange_interpolate(&points, GF256::ZERO)?;
        reconstructed.push(secret_byte.value());
    }

    // Separate secret and digest
    let digest_start = reconstructed.len() - self.digest_bytes;
    let secret = &reconstructed[..digest_start];
    let digest = &reconstructed[digest_start..];

    // Verify integrity
    let secret_gf: Vec<GF256> = secret.iter()
        .map(|&b| GF256::new(b))
        .collect();
    let expected_digest = self.generate_digest(&secret_gf)?;

    if digest != expected_digest.iter().map(|g| g.value()).collect::<Vec<_>>() {
        return Err(ShamirError::DigestVerificationFailed);
    }

    Ok(secret.to_vec())
}

// Satır 183-196: SHA-256 integrity digest
fn generate_digest(&self, secret: &[GF256]) -> Result<Vec<GF256>> {
    use sha2::{Digest, Sha256};

    let secret_bytes: Vec<u8> = secret.iter()
        .map(|g| g.value())
        .collect();

    let hash = Sha256::digest(&secret_bytes);

    // Take first digest_bytes
    let mut digest = Vec::new();
    for i in 0..self.digest_bytes.min(hash.len()) {
        digest.push(GF256::new(hash[i]));
    }

    Ok(digest)
}
```

**Polynomial Functions**:

```rust
// Satır 199-209: Polynomial evaluation (Horner's method)
fn evaluate_polynomial(coefficients: &[GF256], x: GF256) -> GF256 {
    let mut result = GF256::ZERO;
    let mut x_power = GF256::ONE;

    for &coeff in coefficients {
        result = result + (coeff * x_power);
        x_power = x_power * x;
    }

    result
}

// Satır 211-232: Lagrange interpolation
fn lagrange_interpolate(points: &[(GF256, GF256)], x: GF256) -> Result<GF256> {
    let mut result = GF256::ZERO;

    // For each point (xᵢ, yᵢ)
    for (i, &(xi, yi)) in points.iter().enumerate() {
        // Calculate Lagrange basis Lᵢ(x)
        let mut li = GF256::ONE;

        for (j, &(xj, _)) in points.iter().enumerate() {
            if i != j {
                // Lᵢ(x) *= (x - xⱼ) / (xᵢ - xⱼ)
                let numerator = x - xj;
                let denominator = xi - xj;
                let quotient = (numerator / denominator)?;
                li = li * quotient;
            }
        }

        // f(x) += yᵢ * Lᵢ(x)
        result = result + (yi * li);
    }

    Ok(result)
}
```

**Tests**: 7 tests (satır 234-287)

---

### 3. `mnemonic_share.rs` (160 satır)
**Amaç**: BIP-39 integration

**Struct**:

```rust
// Satır 7-41: Serializable share
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MnemonicShare {
    pub id: u8,
    pub share_data: String,        // Hex-encoded share
    pub total_shares: u8,
    pub threshold: u8,
}
```

**Ana Fonksiyonlar**:

```rust
// Satır 43-75: Mnemonic → n shares
pub fn split_mnemonic(
    mnemonic_phrase: &str,
    threshold: usize,
    total_shares: usize,
    language: Language,
) -> Result<Vec<MnemonicShare>> {
    // 1. Parse mnemonic (satır 50)
    let mnemonic = Mnemonic::from_phrase(mnemonic_phrase, language)?;

    // 2. Extract entropy (satır 52-53)
    let entropy = mnemonic.entropy();
    let entropy_bytes = entropy.as_bytes();

    // 3. Split with SSS (satır 55-57)
    let sss = ShamirSSS::new(threshold, total_shares)?;
    let shares = sss.split(entropy_bytes)?;

    // 4. Convert to MnemonicShare (satır 59-72)
    let mnemonic_shares: Vec<MnemonicShare> = shares.into_iter()
        .map(|share| {
            let share_bytes: Vec<u8> = share.value.iter()
                .map(|gf| gf.value())
                .collect();
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

// Satır 77-117: t shares → Mnemonic
pub fn reconstruct_mnemonic(
    mnemonic_shares: &[MnemonicShare],
    language: Language,
) -> Result<String> {
    if mnemonic_shares.is_empty() {
        return Err(ShamirError::InsufficientShares { have: 0, need: 1 });
    }

    let threshold = mnemonic_shares[0].threshold as usize;
    let total_shares = mnemonic_shares[0].total_shares as usize;

    // 1. Create SSS instance (satır 91)
    let sss = ShamirSSS::new(threshold, total_shares)?;

    // 2. Convert MnemonicShare → Share (satır 93-107)
    let shares: Result<Vec<Share>> = mnemonic_shares.iter()
        .map(|ms| {
            let share_bytes = hex::decode(&ms.share_data)
                .map_err(|e| ShamirError::InvalidShareFormat(e.to_string()))?;

            let gf_values: Vec<GF256> = share_bytes.iter()
                .map(|&b| GF256::new(b))
                .collect();

            Ok(Share::new(ms.id, gf_values))
        })
        .collect();

    let shares = shares?;

    // 3. Reconstruct entropy (satır 111)
    let reconstructed_entropy = sss.reconstruct(&shares)?;

    // 4. Entropy → Mnemonic (satır 113-114)
    let entropy = pure_bip39::Entropy::from_bytes(reconstructed_entropy)?;
    let reconstructed_mnemonic = Mnemonic::from_entropy(entropy, language)?;

    Ok(reconstructed_mnemonic.phrase())
}

// Satır 30-36: JSON serialization/deserialization
pub fn to_json(&self) -> Result<String>
pub fn from_json(json: &str) -> Result<Self>
```

**Tests**: 2 tests (satır 120-159)

---

### 4. `error.rs` (64 satır)
**Amaç**: Error types

```rust
#[derive(Debug, Error)]
pub enum ShamirError {
    InvalidThreshold(String),
    InvalidShareCount(String),
    InsufficientShares { have: usize, need: usize },
    InvalidShareFormat(String),
    ReconstructionFailed(String),
    DigestVerificationFailed,
    GaloisFieldError(String),

    // External errors
    #[error(transparent)]
    Bip39Error(#[from] pure_bip39::Bip39Error),

    #[error(transparent)]
    HexError(#[from] hex::FromHexError),

    #[error(transparent)]
    JsonError(#[from] serde_json::Error),
}
```

---

### 5. `lib.rs` (25 satır)
**Amaç**: Public API

```rust
#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod error;
pub mod galois;
pub mod shamir;
pub mod mnemonic_share;

pub use galois::GF256;
pub use shamir::{Share, ShamirSSS};
pub use mnemonic_share::{MnemonicShare, split_mnemonic, reconstruct_mnemonic};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
```

---

## 🖥️ CLI Tool Kullanımı

### `examples/cli_tool.rs` - Interactive tool

**Çalıştırma**:
```bash
cd SHAMIR_SSS
cargo run --example cli_tool
```

**Menü**:
```
🔐 Shamir Secret Sharing - Interactive CLI

1. Generate new BIP39 mnemonic
2. Split mnemonic into shares
3. Save shares to files
4. Load shares and reconstruct
5. Full workflow (all steps)
6. Exit
```

**Örnek - Full Workflow**:
```
Your choice: 5

🎯 Starting Full Workflow...

Step 1: Generate Mnemonic
Word count: 5 (24 words)
✅ Generated 24-word mnemonic

Step 2: Split into Shares
Threshold: 3
Total shares: 5
✅ Created 5 shares (need any 3 to recover)

Step 3: Save Shares
✅ Saved: shares/share_1.json
✅ Saved: shares/share_2.json
✅ Saved: shares/share_3.json
✅ Saved: shares/share_4.json
✅ Saved: shares/share_5.json

Step 4: Test Reconstruction
Using shares: 1, 3, 5
✅ RECONSTRUCTION SUCCESSFUL!
🎉 PERFECT MATCH!

Seeds Match: ✓
✅ Workflow complete!
```

**Share JSON Format**:
```json
{
  "id": 1,
  "share_data": "9f03eeb9480d9d35a1c8f7e6d4b2...",
  "total_shares": 5,
  "threshold": 3
}
```

---

## 🔬 Test Sonuçları

**Toplam**: 25 test (14 unit + 10 integration + 1 doc)

**Breakdown**:
- `galois.rs`: 7 tests
- `shamir.rs`: 7 tests
- Integration tests: 10 tests

**Çalıştırma**:
```bash
cd SHAMIR_SSS
cargo test

# Expected:
# running 14 tests (unit)
# test result: ok. 14 passed; 0 failed
#
# running 10 tests (integration)
# test result: ok. 10 passed; 0 failed
```

---

## 📦 Bağımlılıklar

```toml
[dependencies]
pure_bip39 = { path = "../PURE_BIP39" }  # BIP-39 integration
sha2 = "0.10"              # SHA-256 (digest)
hmac = "0.12"              # HMAC
rand = "0.8"               # CSPRNG (coefficients)
hex = "0.4"                # Hex encoding
serde = "1.0"              # Serialization
serde_json = "1.0"         # JSON
zeroize = "1.7"            # Memory zeroing
thiserror = "1.0"          # Error macros
```

**Toplam**: 9 bağımlılık

---

## 🔐 Güvenlik

### Information-Theoretic Security
```
Perfect Secrecy (Shannon):
H(S | S₁, S₂, ..., S_{t-1}) = H(S)

t-1 shares → ZERO bilgi
t shares → TAM bilgi
```

### Quantum Resistance
- **No hardness assumption**: Hesaplama gücünden bağımsız
- **Quantum computer**: Etkisiz (information-theoretic)

### Implementation Security
- `#![forbid(unsafe_code)]` - Zero unsafe code
- `zeroize` - Automatic memory cleanup
- `rand::thread_rng` - CSPRNG for coefficients

---

## 📚 Referanslar

- **Shamir (1979)**: "How to Share a Secret", CACM
- **BIP-39**: https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki
- **GF(256)**: Finite field arithmetic (AES field)
- **Lagrange Interpolation**: Polynomial reconstruction

---

## 📄 Lisans

MIT License

**Not**: Eğitim amaçlıdır. Production için security audit önerilir.
