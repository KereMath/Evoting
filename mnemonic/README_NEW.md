# MPC Wallet - BIP39 Implementation Suite

**Kriptografi ve Deterministik Cüzdan Protokolleri Implementasyonu**

---

## 📋 Bu Repo Ne İçeriyor?

Bu repository, 3 bağımsız kriptografik implementasyon içerir:

1. **PURE_BIP39** - BIP-39 Spesifikasyonu (Mnemonic ↔ Seed)
2. **BIP39** - Tam HD Wallet (BIP-39 + BIP-32 + BIP-44)
3. **SHAMIR_SSS** - Shamir Secret Sharing over GF(256)

Her biri ayrı bir Rust crate olarak organize edilmiş, bağımsız çalışabilir.

---

## 🎓 Matematikçiler ve Kriptologlar İçin

### Implementasyon Haritası: Matematik → Kod

#### 1️⃣ PURE_BIP39: Entropi → Mnemonic → Seed

**Matematiksel Akış:**
```
E ∈ {0,1}^n  →  M ∈ W^m  →  S ∈ {0,1}^512
  entropy      mnemonic       seed
```

**Kod Yapısı:**
```
PURE_BIP39/src/
├── entropy.rs      → Entropi üretimi ve checksum
├── mnemonic.rs     → 11-bit encoding ve mnemonic ops
├── seed.rs         → PBKDF2-HMAC-SHA512 implementasyonu
├── wordlist.rs     → 2048-word dictionary management
├── error.rs        → Error types
└── lib.rs          → Public API
```

**Hangi Dosyada Ne Var:**

| Matematiksel İşlem | Dosya | Fonksiyon/Struct |
|-------------------|-------|------------------|
| E ← CSPRNG(n) | `entropy.rs` | `Entropy::generate()` |
| c = SHA256(E)[0:n/32] | `entropy.rs` | `Entropy::checksum()` |
| B = E ‖ c | `entropy.rs` | `to_bits_with_checksum()` |
| s_i = B[i*11:(i+1)*11] | `mnemonic.rs` | `from_entropy()` |
| w_i = WORDLIST[s_i] | `wordlist.rs` | `Wordlist::get_word()` |
| S = PBKDF2(M, salt, 2048) | `seed.rs` | `Seed::from_mnemonic()` |

**Algoritmalar:**

1. **Entropi Üretimi** (`entropy.rs:73-78`)
   ```rust
   pub fn generate(bits: EntropyBits) -> Result<Self>
   // CSPRNG kullanarak n-bit random entropi
   ```

2. **Checksum** (`entropy.rs:109-122`)
   ```rust
   pub fn checksum(&self) -> Vec<bool>
   // SHA-256(entropy)[0:n/32]
   ```

3. **11-bit Segmentasyon** (`mnemonic.rs:26-50`)
   ```rust
   pub fn from_entropy(entropy: Entropy) -> Result<Self>
   // (entropy + checksum) → 11-bit chunks → word indices
   ```

4. **PBKDF2 KDF** (`seed.rs:18-37`)
   ```rust
   pub fn from_mnemonic(mnemonic: &str, passphrase: &str) -> Self
   // PBKDF2(HMAC-SHA512, mnemonic, "mnemonic"||pass, 2048, 512)
   ```

**Test Coverage:** 13 tests
- `entropy.rs`: 3 tests (generation, checksum, conversion)
- `mnemonic.rs`: 7 tests (roundtrip, validation, errors)
- Integration: 3 tests (BIP-39 compliance)

---

#### 2️⃣ BIP39: Hierarchical Deterministic Wallet

**Matematiksel Akış:**
```
Seed S → Master (k_m, c_m) → CKD → Child keys → Addresses
         BIP-32              secp256k1         Bitcoin
```

**Kod Yapısı:**
```
BIP39/src/
├── entropy.rs      → Entropi (PURE_BIP39 ile aynı)
├── mnemonic.rs     → Mnemonic (PURE_BIP39 ile aynı)
├── seed.rs         → Seed (PURE_BIP39 ile aynı)
├── wordlist.rs     → Wordlist (PURE_BIP39 ile aynı)
├── wallet.rs       → BIP-32 HD derivation + secp256k1
├── utils.rs        → Path parsing, serialization
├── error.rs        → Error types
├── main.rs         → CLI application
└── lib.rs          → Public API

examples/
├── cli_tool.rs          → Interactive wallet tool
├── generate_wallet.rs   → Simple generation
└── recover_wallet.rs    → Recovery example
```

**Hangi Dosyada Ne Var:**

| Matematiksel İşlem | Dosya | Fonksiyon |
|-------------------|-------|-----------|
| (k_m, c_m) = HMAC-SHA512("Bitcoin seed", S) | `wallet.rs` | `Wallet::from_seed()` |
| I = HMAC-SHA512(c_par, data) | `wallet.rs` | `derive_child()` (private) |
| k_i = parse256(I_L) + k_par (mod n) | `wallet.rs` | Child key derivation |
| K = k·G (EC point mult) | `wallet.rs` | `get_public_key()` |
| Address = RIPEMD160(SHA256(K)) | `wallet.rs` | `get_address()` |
| m/44'/0'/0'/0/i path parsing | `utils.rs` | `HDPath::from_str()` |

**Algoritmalar:**

1. **Master Key Derivation** (`wallet.rs:58-75`)
   ```rust
   pub fn from_seed(seed: &Seed, network: Network) -> Result<Self>
   // HMAC-SHA512("Bitcoin seed", seed) → (private_key, chain_code)
   ```

2. **Child Key Derivation (BIP-32)** (`wallet.rs:~120-180`)
   ```rust
   fn derive_child(&self, path: &HDPath) -> Result<PrivateKey>
   // CKD function: parent → child via HMAC + EC addition
   ```

3. **Address Generation** (`wallet.rs:200-250`)
   ```rust
   pub fn get_address(&self, path: &HDPath) -> Result<String>
   // Private key → Public key → Hash → Base58Check
   ```

4. **Batch Address Generation** (`wallet.rs:~280`)
   ```rust
   pub fn generate_addresses(&self, count: usize, start: u32) -> Result<Vec<String>>
   // Loop over indices, derive each
   ```

5. **HD Path Parsing** (`utils.rs:30-80`)
   ```rust
   impl HDPath {
       pub fn from_str(path: &str) -> Result<Self>
       // "m/44'/0'/0'/0/0" → struct with components
   }
   ```

**Elliptic Curve:** secp256k1 (via `secp256k1` crate)
- Prime field: p = 2^256 - 2^32 - 977
- Curve: y² = x³ + 7
- Generator G, order n

**Test Coverage:** 14 tests
- Core BIP-39: 7 tests (inherited)
- HD wallet: 2 tests (derivation, addresses)
- Utils: 2 tests (hex, path parsing)
- Integration: 3 tests

**CLI Tool:** `examples/cli_tool.rs` (477 LOC)
- 8 interaktif menu seçeneği
- JSON wallet export/import
- Batch address generation

---

#### 3️⃣ SHAMIR_SSS: Threshold Secret Sharing

**Matematiksel Akış:**
```
Secret s ∈ GF(256)^ℓ → Polynomial f(x) → Shares (i, f(i))
                        Lagrange        → Reconstruct s = f(0)
```

**Kod Yapısı:**
```
SHAMIR_SSS/src/
├── galois.rs           → GF(256) arithmetic
├── shamir.rs           → Polynomial construction, Lagrange
├── mnemonic_share.rs   → BIP39 integration
├── error.rs            → Error types
└── lib.rs              → Public API

examples/
├── cli_tool.rs         → Full workflow tool
├── basic_demo.rs       → Simple example
└── interactive_test.rs → Manual testing
```

**Hangi Dosyada Ne Var:**

| Matematiksel İşlem | Dosya | Fonksiyon/Struct |
|-------------------|-------|------------------|
| GF(256) addition (⊕) | `galois.rs` | `Add` trait impl |
| GF(256) multiplication (⊗) | `galois.rs` | `Mul` trait impl |
| a^(-1) in GF(256) | `galois.rs` | `GF256::inverse()` |
| f(x) = s + a₁x + ... | `shamir.rs` | `ShamirSSS::split()` |
| Shares = f(1),...,f(n) | `shamir.rs` | Loop in `split()` |
| f(0) = Σ y_i·L_i(0) | `shamir.rs` | `ShamirSSS::reconstruct()` |
| L_i(0) = ∏ x_j/(x_j-x_i) | `shamir.rs` | `lagrange_coefficient()` |
| BIP39 entropy split | `mnemonic_share.rs` | `split_mnemonic()` |

**Algoritmalar:**

1. **GF(256) Aritmetiği** (`galois.rs`)

   **Addition** (lines ~90-95):
   ```rust
   impl Add for GF256 {
       fn add(self, other: Self) -> Self {
           GF256::new(self.0 ^ other.0)  // XOR
       }
   }
   ```

   **Multiplication** (lines ~100-125):
   ```rust
   impl Mul for GF256 {
       fn mul(self, other: Self) -> Self {
           // Russian peasant multiplication
           // Reduction with p(x) = x⁸ + x⁴ + x³ + x + 1 (0x11B)
       }
   }
   ```

   **Inverse** (lines ~140-180):
   ```rust
   pub fn inverse(&self) -> Result<GF256> {
       // Extended Euclidean Algorithm over GF(256)
       // Returns a^(-1) such that a ⊗ a^(-1) = 1
   }
   ```

2. **Polynomial Construction** (`shamir.rs:73-130`)
   ```rust
   pub fn split(&self, secret: &[u8]) -> Result<Vec<Share>>
   // For each byte s:
   //   1. Pick random a_1,...,a_{t-1} ∈ GF(256)
   //   2. f(x) = s + a_1·x + a_2·x² + ... + a_{t-1}·x^{t-1}
   //   3. Evaluate f(i) for i=1,...,n
   ```

3. **Lagrange Interpolation** (`shamir.rs:180-250`)
   ```rust
   pub fn reconstruct(&self, shares: &[Share]) -> Result<Vec<u8>>
   // For each byte position:
   //   1. Collect (x_i, y_i) from t shares
   //   2. Compute L_i(0) = ∏_{j≠i} x_j/(x_j - x_i) in GF(256)
   //   3. s = Σ y_i · L_i(0)
   ```

4. **BIP39 Integration** (`mnemonic_share.rs:40-90`)
   ```rust
   pub fn split_mnemonic(phrase: &str, t: usize, n: usize) -> Result<Vec<MnemonicShare>>
   // 1. Parse BIP39 mnemonic
   // 2. Extract entropy (NOT seed!)
   // 3. Split entropy bytes over GF(256)
   // 4. Wrap in MnemonicShare struct
   ```

5. **Share Reconstruction** (`mnemonic_share.rs:95-135`)
   ```rust
   pub fn reconstruct_mnemonic(shares: &[MnemonicShare]) -> Result<String>
   // 1. Extract share data
   // 2. Reconstruct entropy via Lagrange
   // 3. Verify SHA256 digest
   // 4. Convert entropy → BIP39 mnemonic
   ```

**Galois Field Details:**
- Irreducible polynomial: p(x) = x⁸ + x⁴ + x³ + x + 1
- Hex: 0x11B
- Field size: 2^8 = 256 elements
- Addition: XOR (⊕)
- Multiplication: mod p(x)

**Security Property:**
```
I(S; S₁,...,S_{t-1}) = 0
∀ subsets of t-1 shares: no information about secret
(Information-theoretic security)
```

**Test Coverage:** 25 tests
- `galois.rs`: 4 tests (field operations, inverse)
- `shamir.rs`: 6 tests (split, reconstruct, threshold)
- `mnemonic_share.rs`: 4 tests (BIP39 integration)
- Test vectors: 10 tests (various configurations)
- Doc test: 1 test

**CLI Tool:** `examples/cli_tool.rs` (472 LOC)
- 6 menu options
- Full workflow automation
- JSON share serialization

---

## 💻 Yazılımcılar İçin: Dosya Yapısı ve Kullanım

### Proje Organizasyonu

```
mnemonic/
│
├── PURE_BIP39/          # Minimal BIP-39 (600 LOC)
│   ├── src/
│   │   ├── entropy.rs   # 4.7KB - Entropi ve checksum
│   │   ├── mnemonic.rs  # 7.9KB - Mnemonic encoding
│   │   ├── seed.rs      # 1.5KB - PBKDF2 KDF
│   │   ├── wordlist.rs  # 3.9KB - Dictionary ops
│   │   ├── error.rs     # 1.2KB - Error types
│   │   └── lib.rs       # 0.8KB - Public API
│   │
│   ├── examples/
│   │   ├── interactive.rs    # 150 LOC - CLI tool
│   │   ├── simple.rs         # Basic usage
│   │   └── quick_test.rs     # Quick test
│   │
│   ├── tests/            # 13 unit tests
│   └── Cargo.toml
│
├── BIP39/               # Full HD Wallet (1200 LOC)
│   ├── src/
│   │   ├── entropy.rs   # 4.7KB - Same as PURE_BIP39
│   │   ├── mnemonic.rs  # 7.1KB - Same as PURE_BIP39
│   │   ├── seed.rs      # 1.5KB - Same as PURE_BIP39
│   │   ├── wordlist.rs  # 4.0KB - Same as PURE_BIP39
│   │   ├── wallet.rs    # 5.4KB - HD derivation ⭐ NEW
│   │   ├── utils.rs     # 1.9KB - Helper functions ⭐ NEW
│   │   ├── main.rs      # 12KB - CLI app ⭐ NEW
│   │   ├── error.rs     # 1.7KB - Extended errors
│   │   └── lib.rs       # 0.8KB - Public API
│   │
│   ├── examples/
│   │   ├── cli_tool.rs         # 477 LOC - Full wallet tool
│   │   ├── generate_wallet.rs  # Simple generation
│   │   └── recover_wallet.rs   # Recovery example
│   │
│   ├── tests/            # 14 unit + integration tests
│   └── Cargo.toml
│
└── SHAMIR_SSS/          # Secret Sharing (600 LOC)
    ├── src/
    │   ├── galois.rs           # 5.0KB - GF(256) math ⭐
    │   ├── shamir.rs           # 8.1KB - SSS core ⭐
    │   ├── mnemonic_share.rs   # 4.4KB - BIP39 integration ⭐
    │   ├── error.rs            # 1.1KB - Error types
    │   └── lib.rs              # 0.5KB - Public API
    │
    ├── examples/
    │   ├── cli_tool.rs         # 472 LOC - Full workflow
    │   ├── basic_demo.rs       # Simple example
    │   └── interactive_test.rs # Manual testing
    │
    ├── tests/
    │   └── test_vectors.rs     # 10 integration tests
    │
    ├── MANUAL_USAGE.md    # Step-by-step guide
    ├── TESTING.md         # Test documentation
    └── Cargo.toml
```

### Hızlı Başlangıç

#### PURE_BIP39 Kullanımı
```bash
cd PURE_BIP39

# Tests
cargo test            # 13 tests

# CLI
cargo run --example interactive

# Library usage
cargo build --release
```

#### BIP39 Kullanımı
```bash
cd BIP39

# Tests
cargo test            # 14 tests

# CLI Tool (interactive)
cargo run --example cli_tool

# Examples
cargo run --example generate_wallet
cargo run --example recover_wallet
```

#### SHAMIR_SSS Kullanımı
```bash
cd SHAMIR_SSS

# Tests
cargo test            # 25 tests

# CLI Tool (full workflow)
cargo run --example cli_tool

# Simple demo
cargo run --example basic_demo
```

### Dependencies

**PURE_BIP39:**
- `sha2`: SHA-256/512
- `hmac`: HMAC implementation
- `pbkdf2`: Key derivation
- `rand`: CSPRNG
- `hex`: Hex encoding
- `unicode-normalization`: NFKD
- `zeroize`: Memory security

**BIP39 (extends PURE_BIP39):**
- `secp256k1`: Elliptic curve ops
- `bitcoin`: Address generation
- `bs58`: Base58 encoding
- `chrono`: Timestamps
- All PURE_BIP39 deps

**SHAMIR_SSS:**
- `sha2`: Digest verification
- `rand`: Random coefficients
- `hex`: Serialization
- `serde`, `serde_json`: Share format
- `zeroize`: Memory security
- `pure-bip39`: Mnemonic integration (local path)

---

## 🔬 Test İstatistikleri

| Project | Unit Tests | Integration Tests | Doc Tests | Total | Status |
|---------|------------|-------------------|-----------|-------|--------|
| PURE_BIP39 | 13 | 0 | 0 | 13 | ✅ All Pass |
| BIP39 | 14 | 0 | 0 | 14 | ✅ All Pass |
| SHAMIR_SSS | 14 | 10 | 1 | 25 | ✅ All Pass |
| **TOTAL** | **41** | **10** | **1** | **52** | ✅ **100%** |

---

## 📖 Dokümantasyon

Her klasörde detaylı README:
- **PURE_BIP39/README.md** - BIP-39 spec + kod mapping
- **BIP39/README.md** - HD wallet + examples
- **SHAMIR_SSS/README.md** - SSS theory + usage
- **SHAMIR_SSS/MANUAL_USAGE.md** - Step-by-step guide
- **SHAMIR_SSS/TESTING.md** - Test documentation

---

## 🎯 Hangi Klasörü Kullanmalıyım?

| İhtiyaç | Klasör | Sebep |
|---------|--------|-------|
| Sadece mnemonic ↔ seed | PURE_BIP39 | Minimal, 600 LOC |
| Bitcoin adresi üretimi | BIP39 | HD wallet support |
| Mnemonic'i parçalara böl | SHAMIR_SSS | Threshold scheme |
| Production wallet | BIP39 | Complete solution |
| Akademik araştırma | SHAMIR_SSS | Mathematical implementation |

---

## 📚 Referanslar

**Spesifikasyonlar:**
- BIP-39: https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki
- BIP-32: https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki
- BIP-44: https://github.com/bitcoin/bips/blob/master/bip-0044.mediawiki

**Akademik:**
- Shamir, A. (1979). "How to share a secret". CACM 22(11): 612–613

**Standartlar:**
- NIST FIPS 180-4: SHA-256
- RFC 2898: PBKDF2
- SEC 2: secp256k1

---

**Made with ❤️ by METU CENG**
