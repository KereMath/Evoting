# 🔐 PURE BIP-39 Implementation

Saf BIP-39 implementasyonu - Minimum bağımlılık, maksimum basitlik

---

## Matematiksel Altyapı

### Algoritma Mapping

| Matematiksel İşlem | Dosya | Fonksiyon/Satır | Açıklama |
|-------------------|-------|-----------------|----------|
| **E ← CSPRNG(n)** | `entropy.rs` | `Entropy::generate()` (66-71) | OsRng ile kriptografik güvenli rastgele entropi |
| **c = SHA256(E)[0:CS]** | `entropy.rs` | `Entropy::checksum()` (102-114) | SHA-256 hash'in ilk CS biti |
| **B = E ∥ c** | `entropy.rs` | `to_bits_with_checksum()` (116-128) | Entropi + checksum birleştirme |
| **W = {w₁, w₂, ..., wₙ}** | `mnemonic.rs` | `from_entropy()` (25-48) | 11-bit blokları wordlist indekslerine dönüştürme |
| **M = w₁ w₂ ... wₙ** | `mnemonic.rs` | `phrase()` (101-103) | Kelimeleri boşlukla birleştirme |
| **E' = bits(M) mod 2ⁿ** | `mnemonic.rs` | `from_phrase()` (50-98) | Mnemonic'ten entropi geri çıkarma |
| **verify(c, E')** | `mnemonic.rs` | `from_phrase()` (88-92) | Checksum doğrulama |
| **S = PBKDF2(M, salt, 2048)** | `seed.rs` | `from_mnemonic()` (17-34) | PBKDF2-HMAC-SHA512 ile 512-bit seed |
| **NFKD(M)** | `seed.rs` | `from_mnemonic()` (18-19) | Unicode normalizasyonu |
| **I(w) → [0, 2047]** | `wordlist.rs` | `get_index()` (108-110) | Kelime → indeks HashMap lookup |
| **W[i] → word** | `wordlist.rs` | `get_word()` (104-106) | İndeks → kelime array lookup |

### Matematiksel Tanımlar

- **n**: Entropi bit sayısı ∈ {128, 160, 192, 224, 256}
- **CS**: Checksum bit sayısı = n/32
- **|W|**: Kelime sayısı = (n + CS)/11 ∈ {12, 15, 18, 21, 24}
- **Wordlist**: |L| = 2048, sorted, unique 4-char prefixes
- **PBKDF2**: 2048 iterasyon, HMAC-SHA512, salt = "mnemonic" + passphrase

### Güvenlik Özellikleri

```
P(collision) ≤ 2^(-n/2) (Birthday paradox)
P(false positive) = 2^(-CS)
KDF complexity: 2^n × 2048 operations
```

---

## Implementation Detayları

### Dosya Yapısı (6 dosya, ~600 LOC)

### 1. `entropy.rs` (174 satır)
**Amaç**: Kriptografik entropi üretimi ve checksum

**Struct'lar**:
- `EntropyBits` (8-50): Enum - 128/160/192/224/256 bit seçenekleri
  ```rust
  pub enum EntropyBits {
      Bits128 = 128,  // 12 words
      Bits256 = 256,  // 24 words
  }
  ```

- `Entropy` (58-129): Main struct - zeroize ile güvenli
  ```rust
  pub struct Entropy {
      bits: EntropyBits,
      data: Vec<u8>,  // Zeroed on drop
  }
  ```

**Ana Fonksiyonlar**:
```rust
// Satır 66-71: CSPRNG ile entropi üretimi
pub fn generate(bits: EntropyBits) -> Result<Self> {
    let mut data = vec![0u8; bits.byte_count()];
    OsRng.try_fill_bytes(&mut data)?;  // OS-level CSPRNG
    Ok(Entropy { bits, data })
}

// Satır 73-76: Byte array'den entropi oluşturma
pub fn from_bytes(bytes: Vec<u8>) -> Result<Self>

// Satır 78-81: Hex string parsing
pub fn from_hex(hex: &str) -> Result<Self>

// Satır 102-114: SHA-256 checksum (ilk n/32 bit)
pub fn checksum(&self) -> Vec<bool> {
    let hash = Sha256::digest(&self.data);
    // Extract first CS bits
}

// Satır 116-128: Entropi + checksum concatenation
pub fn to_bits_with_checksum(&self) -> Vec<bool>
```

**Dependency**: `rand::rngs::OsRng`, `sha2::Sha256`, `zeroize`
**Tests**: 4 tests (satır 140-173)

---

### 2. `mnemonic.rs` (231 satır)
**Amaç**: Mnemonic phrase oluşturma ve doğrulama

**Struct**:
```rust
pub struct Mnemonic {
    words: Vec<String>,
    entropy: Entropy,
    language: Language,
}
```

**Ana Fonksiyonlar**:
```rust
// Satır 20-23: Yeni mnemonic generate
pub fn generate(bits: EntropyBits, language: Language) -> Result<Self>

// Satır 25-48: Entropy → Mnemonic dönüşümü
// Algorithm: bits → 11-bit chunks → wordlist indices
pub fn from_entropy(entropy: Entropy, language: Language) -> Result<Self> {
    let bits = entropy.to_bits_with_checksum();
    let mut words = Vec::new();

    // 11-bit chunking
    for chunk in bits.chunks(11) {
        let mut index = 0u16;
        for (i, &bit) in chunk.iter().enumerate() {
            if bit { index |= 1 << (10 - i); }
        }
        words.push(wordlist.get_word(index)?);
    }
}

// Satır 50-98: Mnemonic → Entropy (validation ile)
// - NFKD normalization (satır 51)
// - Checksum verification (satır 88-92)
pub fn from_phrase(phrase: &str, language: Language) -> Result<Self>

// Satır 121-123: PBKDF2 seed generation
pub fn to_seed(&self, passphrase: &str) -> Seed {
    Seed::from_mnemonic(&self.phrase(), passphrase)
}

// Satır 125-127: Quick validation (boolean)
pub fn validate(phrase: &str, language: Language) -> bool
```

**Test Vectors**: BIP-39 official vectors (satır 151-172)
```rust
const TEST_VECTORS: &[(&str, &str, &str)] = &[
    // (entropy_hex, mnemonic, seed_with_passphrase_TREZOR)
    ("00000000...", "abandon abandon...", "c55257c360..."),
];
```

**Tests**: 7 tests (satır 174-230)

---

### 3. `seed.rs` (59 satır)
**Amaç**: PBKDF2-HMAC-SHA512 key derivation

**Constants**:
```rust
const PBKDF2_ROUNDS: u32 = 2048;  // BIP-39 standard
const SEED_SIZE: usize = 64;       // 512 bits
```

**Struct**:
```rust
pub struct Seed {
    data: [u8; 64],  // Zeroed on drop
}
```

**Ana Fonksiyon**:
```rust
// Satır 17-34: PBKDF2 implementation
pub fn from_mnemonic(mnemonic: &str, passphrase: &str) -> Self {
    // 1. NFKD normalization (satır 18-19)
    let normalized_mnemonic = mnemonic.nfkd().collect::<String>();
    let normalized_passphrase = passphrase.nfkd().collect::<String>();

    // 2. Prepare parameters
    let password = normalized_mnemonic.as_bytes();
    let salt = format!("mnemonic{}", normalized_passphrase);

    // 3. PBKDF2 execution (satır 26-31)
    pbkdf2::<Hmac<Sha512>>(
        password,
        salt.as_bytes(),
        2048,  // iterations
        &mut seed_data
    );
}
```

**Dependency**: `pbkdf2`, `hmac`, `sha2::Sha512`, `unicode-normalization`
**Tests**: None (tested via mnemonic.rs)

---

### 4. `wordlist.rs` (144 satır)
**Amaç**: BIP-39 wordlist yönetimi

**Enums**:
```rust
pub enum Language {
    English, Japanese, Korean, Spanish,
    ChineseSimplified, ChineseTraditional,
    French, Italian, Czech,
}
```

**Struct**:
```rust
pub struct Wordlist {
    language: Language,
    words: Vec<String>,              // 2048 words
    word_to_index: HashMap<String, u16>,  // Fast lookup
}
```

**Ana Fonksiyonlar**:
```rust
// Satır 56-86: Wordlist parsing & validation
pub fn from_str(content: &str, language: Language) -> Result<Self> {
    let words: Vec<String> = content.lines()
        .map(|w| w.trim().to_string())
        .filter(|w| !w.is_empty())
        .collect();

    // Validation: exactly 2048 words (satır 63-65)
    if words.len() != 2048 { return Err(...); }

    // Validation: sorted order (satır 67-71)
    let mut sorted = words.clone();
    sorted.sort();
    if sorted != words { return Err(...); }

    // Validation: unique 4-char prefixes (satır 73)
    Self::validate_unique_prefixes(&words)?;
}

// Satır 88-102: Prefix uniqueness check
fn validate_unique_prefixes(words: &[String]) -> Result<()> {
    let mut prefixes = HashMap::new();
    for (idx, word) in words.iter().enumerate() {
        let prefix: String = word.chars().take(4).collect();
        if let Some(_) = prefixes.insert(prefix, idx) {
            return Err(...);  // Duplicate prefix!
        }
    }
}

// Satır 104-106: Index → word (O(1))
pub fn get_word(&self, index: usize) -> Option<&str> {
    self.words.get(index).map(|s| s.as_str())
}

// Satır 108-110: Word → index (O(1) HashMap)
pub fn get_index(&self, word: &str) -> Option<u16> {
    self.word_to_index.get(word).copied()
}
```

**Static Storage**:
```rust
// Satır 121-135: Lazy-loaded wordlist cache
static WORDLISTS: Lazy<HashMap<Language, Wordlist>> = Lazy::new(|| {
    let english = include_str!("../data/wordlists/english.txt");
    // Load and cache wordlists
});

// Satır 138-142: Global accessor
pub fn get(language: Language) -> Result<&'static Wordlist>
```

**Dependency**: `once_cell::sync::Lazy`
**Tests**: None (validated during runtime)

---

### 5. `error.rs` (52 satır)
**Amaç**: Error types ve external error conversions

**Main Enum**:
```rust
#[derive(Debug, Error)]
pub enum Bip39Error {
    #[error("Invalid entropy size: {0} bits")]
    InvalidEntropySize(usize),

    #[error("Invalid mnemonic length: {0} words")]
    InvalidMnemonicLength(usize),

    #[error("Word not found in wordlist: {0}")]
    WordNotFound(String),

    #[error("Invalid checksum")]
    InvalidChecksum,

    #[error("Invalid wordlist: {0}")]
    InvalidWordlist(usize),

    // External error conversions
    #[error("Random error: {0}")]
    RandomError(String),

    #[error("Hex decode error: {0}")]
    HexError(#[from] hex::FromHexError),
}

pub type Result<T> = std::result::Result<T, Bip39Error>;
```

**Dependency**: `thiserror`

---

### 6. `lib.rs` (34 satır)
**Amaç**: Public API ve module exports

```rust
// Satır 1-2: Safety guarantees
#![forbid(unsafe_code)]
#![warn(missing_docs)]

// Satır 4-8: Module declarations
pub mod entropy;
pub mod error;
pub mod mnemonic;
pub mod seed;
pub mod wordlist;

// Satır 10-14: Re-exports (public API)
pub use entropy::{Entropy, EntropyBits};
pub use error::{Bip39Error, Result};
pub use mnemonic::Mnemonic;
pub use seed::Seed;
pub use wordlist::{Language, Wordlist};

// Satır 16-18: Metadata
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const BIP39_SPEC_URL: &str = "...";
```

**Tests**: 2 tests (metadata validation)

---

## 🖥️ CLI Tool Kullanımı

### `examples/interactive.rs` (interaktif CLI)

**Çalıştırma**:
```bash
cd PURE_BIP39
cargo run --example interactive
```

**Menü Seçenekleri**:
1. Generate new mnemonic (12/15/18/21/24 kelime)
2. Validate existing mnemonic
3. Generate seed from mnemonic (passphrase ile)
4. Exit

**Örnek Session**:
```
🔐 PURE BIP-39 Interactive Demo

Choose an option:
  1. Generate new mnemonic
  2. Validate existing mnemonic
  3. Generate seed from mnemonic
  4. Exit

Your choice: 1

Select word count:
  1. 12 words (128 bits)
  2. 24 words (256 bits)

Your choice: 2

✅ Mnemonic Generated!
📝 Your mnemonic (24 words):
abandon ability able about above absent absorb abstract
absurd abuse access accident account accuse achieve acid
acoustic acquire across act action actor actress actual

🔑 Entropy (hex):
00000000000000000000000000000000...

💾 Saved to: mnemonic_20250101_120000.txt
```

---

## 🔬 Test Sonuçları

**Toplam**: 13 test
- `entropy.rs`: 4 test
- `mnemonic.rs`: 7 test
- `lib.rs`: 2 test

**Çalıştırma**:
```bash
cargo test
# Expected: test result: ok. 13 passed; 0 failed
```

---

## 📦 Bağımlılıklar

```toml
[dependencies]
sha2 = "0.10"              # SHA-256 hashing (checksum)
pbkdf2 = "0.12"            # Key derivation function
hmac = "0.12"              # HMAC-SHA512 (for PBKDF2)
rand = "0.8"               # CSPRNG (OsRng)
hex = "0.4"                # Hex encoding/decoding
unicode-normalization = "0.1"  # NFKD normalization
zeroize = "1.7"            # Memory zeroing
once_cell = "1.19"         # Lazy static
thiserror = "1.0"          # Error derive macros
```

**Toplam**: 9 bağımlılık (minimum!)

---

## 🔐 Güvenlik

### Memory Safety
- `#![forbid(unsafe_code)]` - Zero unsafe code
- `zeroize` - Automatic memory cleanup
- `ZeroizeOnDrop` - Drop trait implementation

### Randomness
- `OsRng` - OS-level CSPRNG
  - Linux: `/dev/urandom`
  - Windows: `BCryptGenRandom`
  - macOS: `getentropy()`

### Crypto Primitives
- SHA-256 (checksum)
- PBKDF2-HMAC-SHA512 (2048 iterations)
- NFKD Unicode normalization

---

## 📚 Referanslar

- **BIP-39 Spec**: https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki
- **PBKDF2**: RFC 2898
- **HMAC**: RFC 2104
- **SHA-256**: FIPS 180-4
- **Unicode NFKD**: UAX #15

---

## 📄 Lisans

MIT License

**Not**: Eğitim amaçlıdır. Production için security audit önerilir.
