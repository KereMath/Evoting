# Cryptocurrency Mnemonic Standards - Rust Implementation

Bu repository **4 farklı mnemonic/wallet standardı** içeriyor:
- **PURE_BIP39**: Minimal BIP-39 implementasyonu (~600 LOC)
- **BIP39**: HD wallet desteği ile tam implementasyon (~1200 LOC)
- **SHAMIR_SSS**: Shamir Secret Sharing entegrasyonu (~600 LOC)
- **SLIP39**: 🆕 Modern SLIP-39 standard (group-based secret sharing) (~2000 LOC)

---

## Repository Yapısı

```
mnemonic/
├── PURE_BIP39/          # Temel BIP-39 implementasyonu
│   ├── src/             # 6 kaynak dosya
│   └── examples/        # interactive.rs CLI tool
├── BIP39/               # HD Wallet implementasyonu
│   ├── src/             # 9 kaynak dosya
│   └── examples/        # cli_tool.rs, generate_wallet.rs
├── SHAMIR_SSS/          # Secret sharing implementasyonu
│   ├── src/             # 5 kaynak dosya
│   └── examples/        # shamir_demo.rs CLI tool
└── SLIP39/              # 🆕 SLIP-39 modern standard
    ├── src/             # 8 kaynak dosya (rs1024, cipher, shamir, slip39...)
    ├── examples/        # basic_demo.rs, group_shares.rs
    ├── wordlists/       # 1024-word wordlist (vs BIP-39's 2048)
    └── README.md        # Detailed documentation
```

**Test Coverage**: 88+ test (13 + 14 + 24 + 37+) - tümü başarılı

---

## 🆕 SLIP-39 Highlights

**SLIP-39**, BIP-39'un modern alternatifi ve Trezor tarafından önerilen yeni standard:

### Avantajları:
✅ **Group Support**: 2-level threshold (örn: "2 family members + 1 lawyer")
✅ **Built-in Encryption**: Feistel cipher (PBKDF2-HMAC-SHA256)
✅ **Stronger Checksum**: RS1024 Reed-Solomon (3-word error detection)
✅ **Shorter Wordlist**: 1024 words (BIP-39: 2048)
✅ **No External Crypto**: 100% native Rust (GF256, GF1024, Lagrange)

### Matematiksel İşlemler

| Matematiksel İşlem | Dosya | Fonksiyon/Satır | Açıklama |
|-------------------|-------|-----------------|----------|
| **polymod(values) over GF(1024)** | `rs1024.rs` | `polymod()` (167-186) | Reed-Solomon polynomial |
| **checksum = polymod(data) ^ 1** | `rs1024.rs` | `compute_checksum()` (203-223) | 3-word checksum |
| **F(R, K) = PBKDF2(K, R, c)** | `cipher.rs` | `round_function()` (35-52) | Feistel round |
| **f(x) = s + a₁x + ... + a_{t-1}x^{t-1}** | `shamir.rs` | `split()` (259-323) | Polynomial over GF(256) |
| **s = Σⱼ yⱼ · Lⱼ(0) mod 256** | `shamir.rs` | `reconstruct()` (325-406) | Lagrange interpolation |

### Quick Example:
```rust
use slip39::{Slip39, MasterSecret};

// 3-of-5 shares: need any 3 out of 5 to recover
let slip39 = Slip39::new_single_group(3, 5)?;
let master = MasterSecret::generate(128)?;
let shares = slip39.generate_shares(&master, b"passphrase")?;

// Distribute shares...
// Reconstruct from any 3
let reconstructed = Slip39::reconstruct_secret(&shares[0][0..3], b"passphrase")?;
```

**See [SLIP39/README.md](SLIP39/README.md) for full documentation**

---

## Matematiksel Altyapı ve Algoritma Detayları

### PURE_BIP39: Temel Kriptografik Operasyonlar

Bu implementasyon BIP-39 standardının core fonksiyonlarını içeriyor.

**Entropi Üretimi** (`entropy.rs`)

`Entropy::generate()` fonksiyonu (satır 66-71) OsRng kullanarak kriptografik güvenli rastgele byte üretiyor:
```
E ← CSPRNG(n), n ∈ {128, 160, 192, 224, 256} bits
```

Checksum hesaplaması `Entropy::checksum()` metodunda (satır 102-114). SHA-256 hash'in ilk CS biti alınıyor:
```
c = SHA256(E)[0:CS], CS = n/32
```

Birleştirme işlemi `to_bits_with_checksum()` fonksiyonunda (satır 116-128):
```
B = E ∥ c
```

**Mnemonic Dönüşümü** (`mnemonic.rs`)

`from_entropy()` fonksiyonu (satır 25-48) entropy'yi mnemonic phrase'e çeviriyor. Bit stream 11-bit chunklara bölünüp wordlist indekslerine dönüştürülüyor:
```
B → chunks(11 bits) → indices → words
W = {w₁, w₂, ..., wₙ}, n = (|E| + CS) / 11
```

Phrase oluşturma `phrase()` metodunda (satır 101-103). Kelimeler boşlukla birleştiriliyor:
```
M = w₁ ∥ " " ∥ w₂ ∥ ... ∥ wₙ
```

Validasyon `from_phrase()` fonksiyonunda (satır 50-98). NFKD normalization uygulanıp checksum doğrulanıyor:
```
E' = bits(M) mod 2ⁿ
verify(c, SHA256(E')[0:CS])
```

**Seed Derivation** (`seed.rs`)

`from_mnemonic()` fonksiyonu (satır 17-34) PBKDF2-HMAC-SHA512 ile 512-bit seed üretiyor:
```
password = NFKD(mnemonic)
salt = "mnemonic" ∥ NFKD(passphrase)
S = PBKDF2(password, salt, 2048, HMAC-SHA512)
```

**Wordlist Operasyonları** (`wordlist.rs`)

İki yönlü lookup yapısı var. `get_index()` metodu (satır 108-110) HashMap ile O(1) word→index:
```
I(w) → [0, 2047]
```

`get_word()` metodu (satır 104-106) array indexing ile O(1) index→word:
```
W[i] → word
```

Wordlist validation `from_str()` fonksiyonunda (satır 56-86):
- 2048 kelime kontrolü
- Sıralama kontrolü
- Unique 4-char prefix kontrolü (satır 88-102)

**Parametreler**:
```
n: entropi bit sayısı ∈ {128, 160, 192, 224, 256}
CS: checksum bit = n/32
|W|: kelime sayısı = (n + CS)/11 ∈ {12, 15, 18, 21, 24}
|L|: wordlist size = 2048
```

### BIP39: Hierarchical Deterministic Wallet

Bu implementasyon BIP-32 ve BIP-44 standartlarını da kapsıyor.

**Master Key Derivation** (`wallet.rs`)

`from_seed()` fonksiyonu (satır 81-88) seed'den master key türetiyor:
```
I = HMAC-SHA512(key="Bitcoin seed", data=seed)
master_key = parse256(I[0:32])
chain_code = I[32:64]
```

**Child Key Derivation**

`derive()` metodu (satır 100-108) BIP-32 CKD fonksiyonunu implement ediyor:
```
I = HMAC-SHA512(key=c_parent, data)
k_child = parse256(I_L) + k_parent (mod n)
c_child = I_R
```

Hardened derivation için:
```
data = 0x00 ∥ k_parent ∥ index    (index ≥ 2³¹)
```

**Bitcoin Address Generation**

`get_address()` metodu (satır 110-121) P2PKH address üretiyor:

1. Public key hesaplama (satır 129-135):
```
P = k · G  (secp256k1 elliptic curve)
```

2. Hash160 işlemi:
```
H = RIPEMD160(SHA256(P))
```

3. Base58Check encoding:
```
A = Base58Check(version ∥ H)
```

**WIF Private Key Export**

`get_private_key()` metodu (satır 123-127):
```
WIF = Base58Check(0x80 ∥ k ∥ compression_flag)
```

**BIP-44 Path Structure**

`to_derivation_path()` metodu (satır 47-58) path string'ini parse ediyor:
```
m / 44' / coin_type' / account' / change / address_index

Örnek:
m/44'/0'/0'/0/0  (Bitcoin first receiving address)
m/44'/60'/0'/0/0 (Ethereum first receiving address)
```

`HDPath::bitcoin()` (satır 29-36) ve `HDPath::ethereum()` (satır 38-45) helper'lar default path'leri sağlıyor.

**Batch Address Generation**

`generate_addresses()` metodu (satır 137-152) batch olarak adres üretiyor:
```rust
for i in 0..count {
    path = HDPath { coin, account, change: 0, index: i }
    addresses.push(derive_and_generate(path))
}
```

**Account Information**

`get_account_info()` metodu (satır 171-184) tam account bilgilerini topluyor:
- Derivation path
- Bitcoin address
- Public key (hex)
- Private key (WIF)
- Extended public key (xpub)

**Parametreler**:
```
secp256k1 curve: y² = x³ + 7 (mod p)
p = 2²⁵⁶ - 2³² - 977
n = FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFE BAAEDCE6 AF48A03B BFD25E8C D0364141
G = generator point (compressed: 33 bytes)
```
---

### SHAMIR_SSS: Secret Sharing over GF(256)

Bu implementasyon Shamir's Secret Sharing'i Galois Field GF(256) üzerinde implement ediyor.

**Galois Field GF(256) Aritmetiği** (`galois.rs`)

Field: GF(2⁸) with irreducible polynomial p(x) = x⁸ + x⁴ + x³ + x + 1 (0x11B)

Toplama (satır 112-118):
```
a ⊕ b = a XOR b
```

Çıkarma (satır 120-126):
```
a ⊖ b = a XOR b  (GF(2) karakteristiğinde a⊕b = a⊖b)
```

Çarpma (satır 128-152) peasant multiplication ile:
```
a ⊗ b = (a · b) mod p(x)
```
İmplementasyon binary polynomial multiplication + modular reduction kullanıyor.

Ters eleman (satır 24-55) Extended Euclidean Algorithm ile:
```
a⁻¹: a ⊗ a⁻¹ = 1 (mod p(x))
```

Extended GCD'yi GF(2)[x] polynomial ring'inde çalıştırıyor.

Bölme (satır 154-161):
```
a / b = a ⊗ b⁻¹
```

Üs alma (satır 57-74) square-and-multiply ile:
```
aⁿ via repeated squaring
```

**Helper fonksiyonlar**:
- `gf_divide_poly()` (satır 77-94): GF(2)[x] polynomial division
- `gf_multiply_poly()` (satır 96-110): GF(2)[x] polynomial multiplication

**Shamir Secret Sharing Algorithm** (`shamir.rs`)

`ShamirSSS::new()` constructor (satır 37-71) parametreleri validate ediyor:
```
2 ≤ t ≤ n ≤ 255
t: threshold (minimum shares)
n: total shares
```

**Secret Splitting** (`split()`, satır 73-103)

Her byte için ayrı polynomial oluşturuluyor:
```
f(x) = s + a₁x + a₂x² + ... + a_{t-1}x^(t-1)
```

Katsayılar `split_byte()` fonksiyonunda (satır 105-130) CSPRNG ile üretiliyor (non-zero).

Share hesaplama:
```
yᵢ = f(xᵢ), xᵢ = i ∈ [1, n]
```

Polynomial evaluation `evaluate_polynomial()` fonksiyonunda (satır 199-209) Horner's method ile:
```
P(x) = a₀ + x(a₁ + x(a₂ + x(...)))
```

**Integrity Digest**

`generate_digest()` fonksiyonu (satır 183-196) SHA-256 kullanıyor:
```
h = SHA256(secret)[0:4 bytes]
```

Digest de aynı şekilde split ediliyor ve her share'e append ediliyor.

**Secret Reconstruction** (`reconstruct()`, satır 132-181)

Lagrange interpolation `lagrange_interpolate()` fonksiyonunda (satır 211-232):
```
f(x) = Σⱼ yⱼ · Lⱼ(x)

Lⱼ(x) = Πₘ≠ⱼ (x - xₘ) / (xⱼ - xₘ)
```

Secret x=0'da:
```
s = f(0) = Σⱼ yⱼ · Lⱼ(0)
```

Tüm operasyonlar GF(256) üzerinde yapılıyor.

Reconstruction sonrası digest verification:
```
verify(h_reconstructed, SHA256(s_reconstructed)[0:4])
```

**BIP-39 Integration** (`mnemonic_share.rs`)

`split_mnemonic()` fonksiyonu (satır 43-75):
```
Mnemonic → Entropy (satır 50-53)
Entropy → SSS split (satır 55-57)
Shares → hex encoding + JSON (satır 59-72)
```

`reconstruct_mnemonic()` fonksiyonu (satır 77-117):
```
JSON → hex decode (satır 93-107)
SSS reconstruct (satır 111)
Entropy → Mnemonic (satır 113-114)
```

**Parametreler**:
```
Field: GF(256) = GF(2⁸)
Polynomial degree: t-1
Share count: n ∈ [t, 255]
Threshold: t ∈ [2, 255]
```

---

## Implementation Detayları

### PURE_BIP39 Dosya Yapısı

**entropy.rs** (174 satır)

`EntropyBits` enum (satır 8-50) 5 farklı entropi boyutunu define ediyor:
```rust
pub enum EntropyBits {
    Bits128 = 128,  // 12 words
    Bits160 = 160,  // 15 words
    Bits192 = 192,  // 18 words
    Bits224 = 224,  // 21 words
    Bits256 = 256,  // 24 words
}
```

`Entropy` struct (satır 58-129) zeroize-on-drop semantiği ile:
```rust
pub struct Entropy {
    bits: EntropyBits,
    data: Vec<u8>,  // auto-zeroed on drop
}
```

Fonksiyonlar:
- `generate()` (66-71): OsRng ile CSPRNG
- `from_bytes()` (73-76): Byte array'den oluşturma
- `from_hex()` (78-81): Hex string parsing
- `checksum()` (102-114): SHA-256 checksum
- `to_bits_with_checksum()` (116-128): Concatenation

Dependencies: `rand::rngs::OsRng`, `sha2::Sha256`, `zeroize`

**mnemonic.rs** (231 satır)

`Mnemonic` struct (satır 11-17):
```rust
pub struct Mnemonic {
    words: Vec<String>,
    entropy: Entropy,
    language: Language,
}
```

Ana fonksiyonlar:
- `generate()` (20-23): Yeni mnemonic
- `from_entropy()` (25-48): Entropy → Mnemonic (11-bit chunking)
- `from_phrase()` (50-98): Phrase → Mnemonic (NFKD + validation)
- `to_seed()` (121-123): PBKDF2 seed generation
- `validate()` (125-127): Quick boolean check

Test vectors (satır 151-172): BIP-39 official test vectors embedded.

**seed.rs** (59 satır)

Constants:
```rust
const PBKDF2_ROUNDS: u32 = 2048;
const SEED_SIZE: usize = 64;
```

`Seed` struct (satır 11-58):
```rust
pub struct Seed {
    data: [u8; 64],  // auto-zeroed
}
```

`from_mnemonic()` (satır 17-34):
```rust
// 1. NFKD normalize (18-19)
// 2. Prepare salt (21-22)
// 3. PBKDF2 (26-31)
```

Dependencies: `pbkdf2`, `hmac`, `sha2::Sha512`, `unicode-normalization`

**wordlist.rs** (144 satır)

`Language` enum (satır 5-46): 9 dil desteği.

`Wordlist` struct (satır 48-143):
```rust
pub struct Wordlist {
    language: Language,
    words: Vec<String>,                  // 2048 words
    word_to_index: HashMap<String, u16>, // O(1) lookup
}
```

Fonksiyonlar:
- `from_str()` (56-86): Parse + validation
- `validate_unique_prefixes()` (88-102): 4-char prefix check
- `get_word()` (104-106): Index → word (O(1))
- `get_index()` (108-110): Word → index (O(1) HashMap)

Static storage (satır 121-135): Lazy-loaded cache.

**error.rs** (52 satır)

```rust
#[derive(Debug, Error)]
pub enum Bip39Error {
    InvalidEntropySize(usize),
    InvalidMnemonicLength(usize),
    WordNotFound(String),
    InvalidChecksum,
    // + external error conversions
}
```

**lib.rs** (34 satır)

Safety guarantees:
```rust
#![forbid(unsafe_code)]
#![warn(missing_docs)]
```

Public API exports tüm ana types için.

**Tests**: 13 test (entropy: 4, mnemonic: 7, lib: 2)

---

### BIP39 Dosya Yapısı

PURE_BIP39'daki 6 dosya + HD wallet dosyaları:

**wallet.rs** (222 satır)

`HDPath` struct (satır 15-59):
```rust
pub struct HDPath {
    pub coin: u32,     // 0=BTC, 60=ETH, ...
    pub account: u32,
    pub change: u32,   // 0=external, 1=internal
    pub index: u32,
}
```

Helper metodlar:
- `bitcoin()` (29-36): m/44'/0'/0'/0/0
- `ethereum()` (38-45): m/44'/60'/0'/0/0
- `to_derivation_path()` (47-58): String formatting

`ExtendedKey` struct (satır 61-66):
```rust
pub struct ExtendedKey {
    pub xpriv: ExtendedPrivKey,
    pub xpub: ExtendedPubKey,
}
```

`Wallet` struct (satır 68-153):
```rust
pub struct Wallet {
    network: Network,
    master_key: ExtendedPrivKey,
}
```

Ana metodlar:
- `from_seed()` (81-88): Master key derivation
- `master_keys()` (90-98): xpriv/xpub extraction
- `derive()` (100-108): BIP-32 CKD
- `get_address()` (110-121): P2PKH address
- `get_private_key()` (123-127): WIF export
- `get_public_key()` (129-135): Compressed public key
- `generate_addresses()` (137-152): Batch generation
- `get_account_info()` (171-184): Full account info

`AccountInfo` struct (satır 155-185): Tüm account bilgilerini toplayan wrapper.

Dependencies: `bitcoin`, `secp256k1`, `bs58`

**Tests**: 14 test (entropy: 4, mnemonic: 7, wallet: 2, lib: 1)

---

### SHAMIR_SSS Dosya Yapısı

**galois.rs** (232 satır)

```rust
const IRREDUCIBLE_POLY: u16 = 0x11B;  // AES field polynomial
```

`GF256` struct (satır 7-180):
```rust
pub struct GF256(pub u8);
```

Constants:
```rust
pub const ZERO: GF256 = GF256(0);
pub const ONE: GF256 = GF256(1);
```

Metodlar:
- `new()` (12-14): Constructor
- `inverse()` (24-55): Extended GCD
- `pow()` (57-74): Square-and-multiply

Operator overloads:
- `Add` (112-118): XOR
- `Sub` (120-126): XOR
- `Mul` (128-152): Peasant mult + mod
- `Div` (154-161): Multiply by inverse

Helper fonksiyonlar:
- `gf_divide_poly()` (77-94)
- `gf_multiply_poly()` (96-110)

**Tests**: 7 test (all field operations)

**shamir.rs** (288 satır)

`Share` struct (satır 11-24):
```rust
pub struct Share {
    pub id: u8,
    pub value: Vec<GF256>,
}
```

`ShamirSSS` struct (satır 26-197):
```rust
pub struct ShamirSSS {
    threshold: usize,
    total_shares: usize,
    digest_bytes: usize,  // default: 4
}
```

Ana metodlar:
- `new()` (37-71): Constructor + validation
- `split()` (73-103): Secret → n shares
- `split_byte()` (105-130): Single byte split (polynomial creation)
- `reconstruct()` (132-181): t shares → secret (Lagrange)
- `generate_digest()` (183-196): SHA-256 integrity check

Polynomial fonksiyonlar:
- `evaluate_polynomial()` (199-209): Horner's method
- `lagrange_interpolate()` (211-232): Lagrange interpolation

**Tests**: 7 test (split/reconstruct scenarios)

**mnemonic_share.rs** (160 satır)

`MnemonicShare` struct (satır 7-41):
```rust
#[derive(Serialize, Deserialize)]
pub struct MnemonicShare {
    pub id: u8,
    pub share_data: String,  // hex-encoded
    pub total_shares: u8,
    pub threshold: u8,
}
```

Fonksiyonlar:
- `split_mnemonic()` (43-75): Mnemonic → shares
- `reconstruct_mnemonic()` (77-117): Shares → mnemonic
- `to_json()` (30-32): Serialization
- `from_json()` (34-36): Deserialization

**Tests**: 2 test (round-trip scenarios)

**error.rs** (64 satır)

```rust
pub enum ShamirError {
    InvalidThreshold(String),
    InsufficientShares { have: usize, need: usize },
    DigestVerificationFailed,
    GaloisFieldError(String),
    // + external error conversions
}
```

**lib.rs** (25 satır)

Public API:
```rust
pub use galois::GF256;
pub use shamir::{Share, ShamirSSS};
pub use mnemonic_share::{MnemonicShare, split_mnemonic, reconstruct_mnemonic};
```

**Tests**: 24 test total (14 unit + 10 integration)

---

## CLI Tool Kullanımı

### PURE_BIP39 Interactive Tool

```bash
cd PURE_BIP39
cargo run --example interactive
```

4 seçenek var:
1. Generate new mnemonic (12-24 words)
2. Validate existing mnemonic
3. Generate seed from mnemonic (+ passphrase)
4. Exit

Örnek:
```
Your choice: 1
Select word count: 5 (24 words)

✅ Mnemonic Generated!
📝 Your mnemonic (24 words):
abandon ability able about above absent absorb abstract
absurd abuse access accident account accuse achieve acid
acoustic acquire across act action actor actress actual

🔑 Entropy (hex): 00000000000000000000000000000000...
```

---

### BIP39 HD Wallet CLI

```bash
cd BIP39
cargo run --example cli_tool
```

9 seçenek var:
1. Generate new wallet
2. Recover from mnemonic
3. Validate mnemonic
4. Generate addresses (batch)
5. Derive custom path
6. Export wallet (JSON)
7. Import wallet (JSON)
8. Full workflow
9. Exit

Örnek 1 - Wallet generation:
```
Your choice: 1

Select entropy: 5 (24 words)
Select network: 1 (Bitcoin Mainnet)

✅ Wallet Generated!

📝 Mnemonic (24 words):
[words displayed]

🌱 Seed: a1b2c3d4...

🏦 Master Keys:
  xpriv: xprv9s21ZrQH143K...
  xpub: xpub661MyMwAqRbc...

📍 First 5 Addresses (m/44'/0'/0'/0/i):
  0: 1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa
  1: 1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2
  2: 1C3SoftYBC2bbDzCadZxDrfbnobEXLcYHb
  ...
```

Örnek 2 - Custom derivation:
```
Your choice: 5

Coin type: 0 (Bitcoin)
Account: 0
Change: 0
Index: 42

📍 Path: m/44'/0'/0'/0/42
📍 Address: 1FxkfJQLJTXpW6QmxGT6oF43ZH959ns8Cq
🔑 Private Key (WIF): L2vjK...
🔓 Public Key: 02a1b2c3...
```

---

### SHAMIR_SSS CLI

```bash
cd SHAMIR_SSS
cargo run --example cli_tool
```

6 seçenek var:
1. Generate BIP39 mnemonic
2. Split mnemonic into shares
3. Save shares to files
4. Load and reconstruct
5. Full workflow
6. Exit

Örnek - Full workflow:
```
Your choice: 5

Step 1: Generate Mnemonic
Word count: 5 (24 words)
✅ Generated

Step 2: Split into Shares
Threshold: 3
Total shares: 5
✅ Created 5 shares (need any 3)

Step 3: Save to Files
✅ shares/share_1.json
✅ shares/share_2.json
✅ shares/share_3.json
✅ shares/share_4.json
✅ shares/share_5.json

Step 4: Test Reconstruction
Using shares: 1, 3, 5
✅ Reconstruction successful
🎉 Perfect match!
```

Share JSON formatı:
```json
{
  "id": 1,
  "share_data": "9f03eeb9480d9d35a1c8...",
  "total_shares": 5,
  "threshold": 3
}
```

---

## Test Coverage

**PURE_BIP39**: 13 test
- entropy.rs: 4 test
- mnemonic.rs: 7 test
- lib.rs: 2 test

**BIP39**: 14 test
- entropy.rs: 4 test
- mnemonic.rs: 7 test
- wallet.rs: 2 test
- lib.rs: 1 test

**SHAMIR_SSS**: 24 test
- galois.rs: 7 test
- shamir.rs: 7 test
- mnemonic_share.rs: 2 test
- Integration: 10 test

**Total**: 51 test - hepsi başarılı

Test çalıştırma:
```bash
cd PURE_BIP39 && cargo test
cd BIP39 && cargo test
cd SHAMIR_SSS && cargo test
```

---

## Dependencies

**PURE_BIP39** (9 dependency):
```toml
sha2 = "0.10"              # SHA-256
pbkdf2 = "0.12"            # Key derivation
hmac = "0.12"              # HMAC-SHA512
rand = "0.8"               # CSPRNG
hex = "0.4"                # Hex encoding
unicode-normalization = "0.1"
zeroize = "1.7"            # Memory safety
once_cell = "1.19"         # Lazy static
thiserror = "1.0"          # Error macros
```

**BIP39** (15 dependency):
```toml
# PURE_BIP39 dependencies + aşağıdakiler:
bitcoin = "0.31"           # BIP-32, secp256k1
secp256k1 = "0.28"         # Elliptic curve
bs58 = "0.5"               # Base58 encoding
serde = "1.0"              # Serialization
serde_json = "1.0"         # JSON
chrono = "0.4"             # Timestamps
```

**SHAMIR_SSS** (9 dependency):
```toml
pure_bip39 = { path = "../PURE_BIP39" }
sha2 = "0.10"              # SHA-256 digest
hmac = "0.12"              # HMAC
rand = "0.8"               # CSPRNG
hex = "0.4"                # Hex encoding
serde = "1.0"              # Serialization
serde_json = "1.0"         # JSON
zeroize = "1.7"            # Memory safety
thiserror = "1.0"          # Error macros
```

---

## Security Properties

**Memory Safety**:
```rust
#![forbid(unsafe_code)]  // zero unsafe code
```
Tüm sensitive data (entropy, seed, keys) `zeroize` ile otomatik temizleniyor.

**Randomness**:
- OsRng: OS-level CSPRNG
  - Linux: /dev/urandom
  - Windows: BCryptGenRandom
  - macOS: getentropy()

**Cryptographic Primitives**:
- SHA-256: Checksum ve digest
- PBKDF2-HMAC-SHA512: 2048 iteration (BIP-39 standard)
- secp256k1: Bitcoin elliptic curve (128-bit security)
- GF(256): Information-theoretically secure SSS

**Side-Channel Resistance**:
- Constant-time GF(256) operations
- O(1) wordlist lookups (HashMap)

**Shamir SSS Security**:
- Perfect secrecy (Shannon)
- Information-theoretic (quantum-resistant)
- t-1 shares → zero bilgi
- t shares → tam recovery

---

## References

**Standards**:
- BIP-39: https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki
- BIP-32: https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki
- BIP-44: https://github.com/bitcoin/bips/blob/master/bip-0044.mediawiki

**Papers**:
- Shamir, A. (1979): "How to Share a Secret", CACM
- NIST FIPS 180-4: SHA-256
- RFC 2898: PBKDF2
- RFC 2104: HMAC

**Implementation References**:
- rust-bitcoin: BIP-32 reference
- tiny-bip39: Rust BIP-39
- threshold-secret-sharing: SSS crate

---

