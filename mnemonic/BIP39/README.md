# 🔐 BIP-39 Full HD Wallet Implementation

BIP-39 + BIP-32 + BIP-44 - Tam teşekküllü hierarchical deterministic wallet

---

## Matematiksel Altyapı

### Algoritma Mapping

### BIP-39 İşlemleri (PURE_BIP39 ile aynı)

| Matematiksel İşlem | Dosya | Fonksiyon/Satır | Açıklama |
|-------------------|-------|-----------------|----------|
| **E ← CSPRNG(n)** | `entropy.rs` | `Entropy::generate()` (66-71) | OsRng ile entropi |
| **c = SHA256(E)[0:CS]** | `entropy.rs` | `Entropy::checksum()` (102-114) | Checksum hesaplama |
| **S = PBKDF2(M, salt, 2048)** | `seed.rs` | `from_mnemonic()` (17-34) | Seed derivation |

### BIP-32 HD Wallet İşlemleri

| Matematiksel İşlem | Dosya | Fonksiyon/Satır | Açıklama |
|-------------------|-------|-----------------|----------|
| **(k_m, c_m) = HMAC-SHA512("Bitcoin seed", S)** | `wallet.rs` | `from_seed()` (81-88) | Master key derivation |
| **I = HMAC-SHA512(c_par, data)** | `wallet.rs` | `derive()` (100-108) | Child key derivation |
| **k_i = parse256(I_L) + k_par (mod n)** | `wallet.rs` | `derive()` (104) | Private key addition |
| **c_i = I_R** | `wallet.rs` | `derive()` (105) | Chain code extraction |
| **P = k·G** | `wallet.rs` | `get_public_key()` (129-135) | EC point multiplication (secp256k1) |
| **A = Base58Check(HASH160(P))** | `wallet.rs` | `get_address()` (110-121) | P2PKH address generation |
| **WIF = Base58Check(0x80 ∥ k)** | `wallet.rs` | `get_private_key()` (123-127) | Private key encoding |

### BIP-44 Path İşlemleri

| Matematiksel İşlem | Dosya | Fonksiyon/Satır | Açıklama |
|-------------------|-------|-----------------|----------|
| **m/44'/c'/a'/ch/i** | `wallet.rs` | `to_derivation_path()` (47-58) | BIP-44 path formatting |
| **hardened(i) = i + 2³¹** | `wallet.rs` | Path string (49) | Hardened derivation |

### Matematiksel Tanımlar

**BIP-32 Parametreleri**:
- **k**: 256-bit private key ∈ [1, n-1]
- **c**: 256-bit chain code
- **n**: secp256k1 order = FFFFFFFF FFFFFFFF FFFFFFFF FFFFFFFE BAAEDCE6 AF48A03B BFD25E8C D0364141
- **G**: Generator point on secp256k1
- **p**: Field prime = 2²⁵⁶ - 2³² - 977

**secp256k1 Elliptic Curve**:
```
y² = x³ + 7 (mod p)
```

**BIP-44 Derivation Path**:
```
m / purpose' / coin_type' / account' / change / address_index

purpose = 44 (BIP-44)
coin_type = 0 (Bitcoin), 60 (Ethereum), ...
' = hardened derivation (i >= 2^31)
```

---

## Implementation Detayları

### Dosya Yapısı (9 dosya, ~1200 LOC)

### Temel Dosyalar (PURE_BIP39 ile aynı)
- `entropy.rs` (174 satır) - Entropi üretimi
- `mnemonic.rs` (231 satır) - Mnemonic işlemleri
- `seed.rs` (59 satır) - PBKDF2 key derivation
- `wordlist.rs` (144 satır) - Wordlist yönetimi
- `error.rs` - Error types
- `lib.rs` (38 satır) - Public API

### HD Wallet Dosyaları (YENI)

---

### 1. `wallet.rs` (222 satır) ⭐ ANA DOSYA
**Amaç**: BIP-32/44 hierarchical deterministic wallet

**Struct'lar**:

```rust
// Satır 15-59: BIP-44 derivation path
pub struct HDPath {
    pub coin: u32,        // 0=BTC, 60=ETH
    pub account: u32,     // Account number
    pub change: u32,      // 0=external, 1=internal
    pub index: u32,       // Address index
}

// Satır 61-66: Extended key pair
pub struct ExtendedKey {
    pub xpriv: ExtendedPrivKey,  // Extended private key
    pub xpub: ExtendedPubKey,    // Extended public key
}

// Satır 68-153: Main wallet struct
pub struct Wallet {
    network: Network,
    master_key: ExtendedPrivKey,
}

// Satır 155-185: Account information
pub struct AccountInfo {
    pub path: String,
    pub address: String,
    pub public_key: String,
    pub private_key: String,
    pub xpub: String,
}
```

**Ana Fonksiyonlar**:

```rust
// Satır 81-88: Master key derivation (BIP-32)
pub fn from_seed(seed: &Seed, network: Network) -> Result<Self> {
    // HMAC-SHA512("Bitcoin seed", seed) → (master_key, chain_code)
    let master_key = ExtendedPrivKey::new_master(network, seed.as_bytes())?;
    Ok(Wallet { network, master_key })
}

// Satır 90-98: Get master extended keys
pub fn master_keys(&self) -> ExtendedKey {
    let secp = Secp256k1::new();
    let xpub = ExtendedPubKey::from_priv(&secp, &self.master_key);
    ExtendedKey { xpriv: self.master_key, xpub }
}

// Satır 100-108: BIP-32 child key derivation
pub fn derive(&self, path: &HDPath) -> Result<ExtendedKey> {
    let secp = Secp256k1::new();
    let derivation_path = path.to_derivation_path()?;

    // CKD (Child Key Derivation)
    let xpriv = self.master_key.derive_priv(&secp, &derivation_path)?;
    let xpub = ExtendedPubKey::from_priv(&secp, &xpriv);

    Ok(ExtendedKey { xpriv, xpub })
}

// Satır 110-121: Generate Bitcoin address (P2PKH)
pub fn get_address(&self, path: &HDPath) -> Result<Address> {
    let key = self.derive(path)?;

    // Public key from extended key
    let public_key = PublicKey {
        compressed: true,
        inner: key.xpub.public_key,
    };

    // Base58Check(HASH160(public_key))
    let address = Address::p2pkh(&public_key, self.network);
    Ok(address)
}

// Satır 123-127: Export private key (WIF format)
pub fn get_private_key(&self, path: &HDPath) -> Result<String> {
    let key = self.derive(path)?;
    let private_key = PrivateKey::new(key.xpriv.private_key, self.network);
    Ok(private_key.to_wif())  // Wallet Import Format
}

// Satır 129-135: Get public key (compressed)
pub fn get_public_key(&self, path: &HDPath) -> Result<PublicKey> {
    let key = self.derive(path)?;
    Ok(PublicKey {
        compressed: true,
        inner: key.xpub.public_key,
    })
}

// Satır 137-152: Batch address generation
pub fn generate_addresses(&self, count: usize, account: u32) -> Result<Vec<Address>> {
    let mut addresses = Vec::with_capacity(count);

    for i in 0..count {
        let path = HDPath {
            coin: 0,
            account,
            change: 0,
            index: i as u32,
        };
        addresses.push(self.get_address(&path)?);
    }

    Ok(addresses)
}

// Satır 171-184: Full account information
pub fn get_account_info(&self, path: &HDPath) -> Result<AccountInfo> {
    let key = self.derive(path)?;
    let address = self.get_address(path)?;
    let private_key = self.get_private_key(path)?;

    Ok(AccountInfo {
        path: format!("m/44'/{}'/{}'/{}/{}",
            path.coin, path.account, path.change, path.index),
        address: address.to_string(),
        public_key: hex::encode(key.xpub.public_key.serialize()),
        private_key,
        xpub: key.xpub.to_string(),
    })
}
```

**HDPath Helper Methods**:

```rust
// Satır 29-36: Bitcoin standard path (m/44'/0'/0'/0/i)
pub fn bitcoin() -> Self {
    HDPath {
        coin: 0,      // BTC
        account: 0,
        change: 0,
        index: 0,
    }
}

// Satır 38-45: Ethereum path (m/44'/60'/0'/0/i)
pub fn ethereum() -> Self {
    HDPath {
        coin: 60,     // ETH
        account: 0,
        change: 0,
        index: 0,
    }
}

// Satır 47-58: Convert to DerivationPath string
pub fn to_derivation_path(&self) -> Result<DerivationPath> {
    let path_str = format!(
        "m/44'/{}'/{}'/{}/{}",
        self.coin,
        self.account,
        self.change,
        self.index
    );

    DerivationPath::from_str(&path_str)
        .map_err(|e| Bip39Error::InvalidPath(e.to_string()))
}
```

**Dependency**: `bitcoin` crate (BIP-32), `secp256k1`
**Tests**: 2 tests (satır 188-221)

---

### 2. `utils.rs` (helper functions)
**Amaç**: Utility fonksiyonlar

**Fonksiyonlar**:
- Hex encoding/decoding
- Address validation
- Path parsing helpers

---

### 3. `main.rs` (CLI application)
**Amaç**: Command-line interface

**Features**:
- Interactive wallet generation
- Address derivation
- Key export
- Colorized output

---

## 🖥️ CLI Tool Kullanımı

### `examples/cli_tool.rs` (477 satır) - Comprehensive CLI

**Çalıştırma**:
```bash
cd BIP39
cargo run --example cli_tool
```

**Menü Seçenekleri (9 seçenek)**:

```
🔐 BIP-39 Wallet - Interactive CLI Tool

1. Generate new wallet
2. Recover wallet from mnemonic
3. Validate mnemonic phrase
4. Generate addresses from existing wallet
5. Derive custom HD path
6. Export wallet to file
7. Import wallet from file
8. Full workflow (generate + export + addresses)
9. Exit
```

**Örnek 1: Yeni Wallet Oluştur**:
```
Your choice: 1

Select entropy strength:
  1. 12 words (128 bits)
  2. 15 words (160 bits)
  3. 18 words (192 bits)
  4. 21 words (224 bits)
  5. 24 words (256 bits)

Your choice: 5

Select network:
  1. Bitcoin Mainnet
  2. Bitcoin Testnet

Your choice: 1

✅ Wallet Generated Successfully!

📝 YOUR MNEMONIC PHRASE (24 words):
┌──────────────────────────────────────────┐
│ word1 word2 word3 word4 word5 word6      │
│ word7 word8 word9 word10 word11 word12   │
│ word13 word14 word15 word16 word17 w18   │
│ word19 word20 word21 word22 word23 w24   │
└──────────────────────────────────────────┘

⚠️  WRITE THIS DOWN! Store in a safe place!

🌱 Seed (hex):
a1b2c3d4e5f6...

🏦 Master Keys:
  Extended Private Key (xpriv):
  xprv9s21ZrQH143K...

  Extended Public Key (xpub):
  xpub661MyMwAqRbc...

📍 First 5 Addresses (m/44'/0'/0'/0/i):
  Address 0: 1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa
  Address 1: 1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2
  Address 2: 1C3SoftYBC2bbDzCadZxDrfbnobEXLcYHb
  Address 3: 1D4ZrxZzMM4sZCKQJF8c5YvWQCQhC8fZ4Q
  Address 4: 1E5ZrxZzMM4sZCKQJF8c5YvWQCQhC8fZ4Q

💾 Saved to: wallet_bitcoin_20250101_120000.json
```

**Örnek 2: Custom Path Derivation**:
```
Your choice: 5

Enter derivation path details:
Coin type (0=BTC, 60=ETH, 2=LTC): 0
Account: 0
Change (0=external, 1=internal): 0
Index: 42

📍 Derivation Path: m/44'/0'/0'/0/42

📍 Address: 1FxkfJQLJTXpW6QmxGT6oF43ZH959ns8Cq
🔑 Private Key (WIF): L2vjK...
🔓 Public Key (hex): 02a1b2c3...
📘 Extended Public Key: xpub6D...
```

**Örnek 3: Wallet Export**:
```
Your choice: 6

Enter filename (default: wallet.json): my_wallet.json

✅ Wallet exported to: my_wallet.json

File contains:
{
  "mnemonic": "word1 word2 ...",
  "created_at": "2025-01-01T12:00:00Z",
  "network": "bitcoin",
  "master_xpub": "xpub661...",
  "addresses": [
    {
      "path": "m/44'/0'/0'/0/0",
      "address": "1A1zP1...",
      "index": 0
    }
  ]
}
```

**Örnek 4: Full Workflow**:
```
Your choice: 8

🎯 Starting Full Workflow...

Step 1: Generate Mnemonic
✅ Generated 24-word mnemonic

Step 2: Create Wallet
✅ Wallet created (Bitcoin Mainnet)

Step 3: Generate Addresses
How many addresses? 10
✅ Generated 10 addresses

Step 4: Export to File
✅ Exported to: workflow_20250101_120000.json

Step 5: Display Summary
━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Wallet Summary
━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Mnemonic: 24 words ✓
Network: Bitcoin Mainnet ✓
Addresses: 10 ✓
Exported: Yes ✓
━━━━━━━━━━━━━━━━━━━━━━━━━━━━

🎉 Workflow Complete!
```

---

## 🔬 Test Sonuçları

**Toplam**: 14 test
- `entropy.rs`: 4 test
- `mnemonic.rs`: 7 test
- `wallet.rs`: 2 test
- `lib.rs`: 1 test

**Çalıştırma**:
```bash
cd BIP39
cargo test

# Expected output:
# running 14 tests
# test result: ok. 14 passed; 0 failed
```

**Test Coverage**:
- ✅ HD key derivation
- ✅ Address generation (P2PKH)
- ✅ Multiple address uniqueness
- ✅ Path validation
- ✅ BIP-39 compliance

---

## 📦 Bağımlılıklar

```toml
[dependencies]
# PURE_BIP39 dependencies
sha2 = "0.10"
pbkdf2 = "0.12"
hmac = "0.12"
rand = "0.8"
hex = "0.4"
unicode-normalization = "0.1"
zeroize = "1.7"
once_cell = "1.19"
thiserror = "1.0"

# HD Wallet dependencies (YENI)
bitcoin = "0.31"           # BIP-32, secp256k1, addresses
secp256k1 = "0.28"         # Elliptic curve operations
bs58 = "0.5"               # Base58 encoding

# CLI dependencies
serde = "1.0"              # Serialization
serde_json = "1.0"         # JSON
chrono = "0.4"             # Timestamps
```

**Toplam**: ~15 bağımlılık

---

## 🔐 Güvenlik

### Elliptic Curve Security
- **secp256k1**: 128-bit security level
- **ECDLP**: Computationally infeasible (best attack: ~2^128 operations)
- **Private key range**: [1, n-1] where n ≈ 2^256

### Derivation Security
- **Hardened derivation**: Prevents xpub → parent xpriv attack
- **Chain code**: 256-bit additional entropy
- **HMAC-SHA512**: Collision resistant

### Address Security
- **HASH160 = RIPEMD160(SHA256(P))**: Double hashing
- **Base58Check**: Checksum prevents typos
- **Compressed public keys**: 33 bytes (prefix + x-coordinate)

---

## 📊 Karşılaştırma: BIP39 vs PURE_BIP39

| Özellik | PURE_BIP39 | BIP39 (Bu Proje) |
|---------|------------|------------------|
| **LOC** | ~600 | ~1200 |
| **Dosya Sayısı** | 6 | 9 |
| **Bağımlılık** | 9 | 15 |
| Mnemonic üretimi | ✅ | ✅ |
| Seed üretimi | ✅ | ✅ |
| HD key derivation | ❌ | ✅ |
| Bitcoin adresleri | ❌ | ✅ |
| BIP-44 paths | ❌ | ✅ |
| CLI tool | ✅ Basic | ✅ Advanced |
| Wallet export | ❌ | ✅ JSON |

**Ne Zaman Hangisi?**

✅ **PURE_BIP39 kullan**:
- Sadece mnemonic/seed gerekiyorsa
- Minimum dependency istiyorsan
- Custom wallet logic yazıyorsan

✅ **BIP39 kullan**:
- Bitcoin adresi üreteceksen
- HD wallet fonksiyonu gerekiyorsa
- Production-ready wallet istiyorsan

---

## 📚 Referanslar

- **BIP-39**: https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki
- **BIP-32**: https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki
- **BIP-44**: https://github.com/bitcoin/bips/blob/master/bip-0044.mediawiki
- **secp256k1**: SEC 2 v2.0 (Standards for Efficient Cryptography)

---

## 📄 Lisans

MIT License

**Not**: Production kullanımı için professional security audit önerilir.
