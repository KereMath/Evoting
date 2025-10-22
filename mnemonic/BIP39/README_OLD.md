# BIP-39 Wallet - Full-Featured Implementation

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-blue.svg)](https://www.rust-lang.org/)
[![BIP-39](https://img.shields.io/badge/BIP--39-compliant-green.svg)](https://github.com/bitcoin/bips/blob/master/bip-0039.mediawiki)
[![BIP-32](https://img.shields.io/badge/BIP--32-compliant-green.svg)](https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki)
[![BIP-44](https://img.shields.io/badge/BIP--44-compliant-green.svg)](https://github.com/bitcoin/bips/blob/master/bip-0044.mediawiki)

A **production-ready, full-featured** implementation of BIP-39 with HD wallet functionality (BIP-32/44) in Rust.

## 🎯 What is This?

This is a **complete wallet solution** that implements:

1. **BIP-39**: Mnemonic phrases and seed generation
2. **BIP-32**: Hierarchical Deterministic (HD) wallet key derivation
3. **BIP-44**: Multi-account hierarchy for deterministic wallets
4. **Bitcoin Integration**: Address generation, key management

## ✨ Features

### Core BIP-39 (Same as PURE_BIP39)
- ✅ Entropy generation (128-256 bits)
- ✅ Mnemonic phrase generation and validation
- ✅ Seed generation with PBKDF2-HMAC-SHA512
- ✅ Passphrase support
- ✅ Memory zeroing for security

### Extended Features (Beyond BIP-39)
- ✅ **HD Wallet Derivation (BIP-32)**
  - Master key generation from seed
  - Child key derivation
  - Extended public/private keys (xprv/xpub)

- ✅ **BIP-44 Account Structure**
  - Standard derivation paths: `m/44'/coin'/account'/change/index`
  - Bitcoin, Ethereum, and other coin support
  - Multiple account management

- ✅ **Bitcoin Integration**
  - Address generation (P2PKH, P2WPKH, P2SH)
  - WIF format private keys
  - Public key export
  - Multiple address generation

- ✅ **CLI Application**
  - Interactive wallet generation
  - Wallet recovery from mnemonic
  - Address generation
  - Colorized output

## 📦 Installation & Building

### Clone and Build

```bash
# Navigate to the project directory
cd BIP39

# Build in debug mode
cargo build

# Build in release mode (recommended)
cargo build --release

# Build and run CLI
cargo run --release
```

### As a Library Dependency

Add to your `Cargo.toml`:

```toml
[dependencies]
bip39-wallet = { path = "../BIP39" }
bitcoin = "0.31"
```

## 🧪 How to Test

### Running All Tests

```bash
cd BIP39

# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_wallet_derivation

# Run integration tests
cargo test --test integration_tests
```

### Expected Output

```
running 14 tests
test entropy::tests::test_entropy_generation ... ok
test entropy::tests::test_checksum_calculation ... ok
test mnemonic::tests::test_from_entropy ... ok
test mnemonic::tests::test_from_phrase ... ok
test mnemonic::tests::test_to_seed ... ok
test wallet::tests::test_wallet_derivation ... ok
test wallet::tests::test_multiple_addresses ... ok
...
test result: ok. 14 passed; 0 failed; 0 ignored
```

## 🚀 Quick Start Examples
