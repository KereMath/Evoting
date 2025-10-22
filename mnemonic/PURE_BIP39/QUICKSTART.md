# PURE_BIP39 Quick Start Guide

## 🚀 3 Komutla Başla

### 1. Test Et

```bash
cd PURE_BIP39
cargo test
```

**Beklenen:** 17/17 tests passed ✅

### 2. Basit Örnek Çalıştır

```bash
cargo run --example simple
```

**Ne Göreceksin:**
- 12 kelimelik mnemonic
- Entropy (hex format)
- Seed (passphrase'siz)
- Seed (passphrase ile)

### 3. İnteraktif Mod

```bash
cargo run --example interactive
```

**Yapabileceklerin:**
- Yeni mnemonic oluştur (12-24 kelime)
- Mevcut mnemonic'i doğrula
- Seed üret (istersen passphrase ile)

---

## 📋 Tüm Komutlar

### Test Komutları

```bash
# Tüm testler
cargo test

# Detaylı output
cargo test -- --nocapture

# Sadece mnemonic testleri
cargo test mnemonic

# Sadece entropy testleri
cargo test entropy

# Tek bir test
cargo test test_from_entropy

# Doc testleri
cargo test --doc
```

### Örnek Komutları

```bash
# 1. Basit demo (en kolay)
cargo run --example simple

# 2. Hızlı test (tüm özellikler)
cargo run --example quick_test

# 3. İnteraktif mod (eğlenceli)
cargo run --example interactive

# Release mode (daha hızlı)
cargo run --release --example simple
```

### Build Komutları

```bash
# Debug build
cargo build

# Release build (optimize edilmiş)
cargo build --release

# Dokümantasyon
cargo doc --open

# Temizle
cargo clean
```

---

## 💡 Ne Yapar Bu Örnekler?

### `simple` - Temel Kullanım

```
🔐 Simple BIP-39 Demo

📝 Mnemonic:
history boy ketchup topic coast mystery...

🔑 Entropy: 6c2355e7f282cd24...
🌱 Seed (no pass): 09e56d1e5c6904f6...
🔒 Seed (with 'secret'): 72914c10c2fafaf0...
```

**Öğrendiklerin:**
- Mnemonic nasıl oluşturulur
- Entropy nedir
- Passphrase seed'i nasıl değiştirir

---

### `quick_test` - Kapsamlı Test

```
📋 TEST 1: Different Word Counts
12 words: ✅
15 words: ✅
18 words: ✅
21 words: ✅
24 words: ✅

📋 TEST 2: BIP-39 Test Vector
✅ Test vector is valid!
✅ Seed matches!

📋 TEST 3: Invalid Checksum
✅ Invalid checksum detected!

📋 TEST 4: Passphrase Effect
✅ All seeds are different!
```

**Öğrendiklerin:**
- Tüm kelime sayıları (12-24)
- BIP-39 test vektörleri
- Checksum doğrulaması
- Passphrase etkisi

---

### `interactive` - İnteraktif Kullanım

```
Choose an option:
  1. Generate new mnemonic
  2. Validate existing mnemonic
  3. Generate seed from mnemonic
  4. Exit

Your choice: 1

Select word count:
  1. 12 words (128 bits)
  2. 15 words (160 bits)
  3. 18 words (192 bits)
  4. 21 words (224 bits)
  5. 24 words (256 bits)

Your choice: 1

✅ Mnemonic Generated!

┌────────────────────────────────────────┐
│  1. abandon         2. ability         │
│  3. able            4. about           │
│  5. above           6. absent          │
│  7. absorb          8. abstract        │
│  9. absurd         10. abuse           │
│ 11. access         12. accident        │
└────────────────────────────────────────┘
```

**Yapabileceklerin:**
1. **Yeni mnemonic oluştur**
   - 12-24 kelime seç
   - Mnemonic'i gör
   - Entropy'yi gör

2. **Mnemonic doğrula**
   - Kendi mnemonic'ini yaz
   - Geçerli mi kontrol et
   - Entropy'yi öğren

3. **Seed oluştur**
   - Mnemonic gir
   - Passphrase gir (opsiyonel)
   - 512-bit seed al

---

## 🎯 Hızlı Senaryo Örnekleri

### Senaryo 1: İlk Kez Kullanım

```bash
# 1. Test et (çalışıyor mu?)
cargo test

# 2. Basit demo gör
cargo run --example simple

# 3. Eğlenceli mod dene
cargo run --example interactive
# Seç: 1 (Generate new mnemonic)
# Seç: 1 (12 words)
# Mnemonic'ini yaz!
```

### Senaryo 2: Mevcut Mnemonic'i Test Et

```bash
cargo run --example interactive

# Menüden seç: 2 (Validate)
# Mnemonic'ini gir
# Sonucu gör: ✅ veya ❌
```

### Senaryo 3: Farklı Passphrase'leri Dene

```bash
cargo run --example interactive

# Menüden seç: 3 (Generate seed)
# Aynı mnemonic'i gir
# Farklı passphrase'ler dene:
#   - "" (boş)
#   - "password"
#   - "my secret"
# Seed'lerin farklı olduğunu gör!
```

### Senaryo 4: BIP-39 Test Vector'ü Kontrol Et

```bash
cargo run --example quick_test

# Otomatik kontrol eder:
# - Test vector: "abandon abandon..."
# - Expected seed: c55257c360c07c72...
# - Sonuç: ✅ Match!
```

---

## 🔥 Pro Tips

### Tip 1: Release Mode Kullan
```bash
# Daha hızlı çalışır (özellikle seed generation)
cargo run --release --example interactive
```

### Tip 2: Output'u Dosyaya Kaydet
```bash
cargo run --example simple > my_mnemonic.txt
```

### Tip 3: Belirli Testi Çalıştır
```bash
# Sadece to_seed testini çalıştır
cargo test test_to_seed -- --nocapture
```

### Tip 4: Testleri Sırayla Çalıştır
```bash
# Paralel değil, sırayla (daha okunaklı)
cargo test -- --test-threads=1 --nocapture
```

---

## 📊 Örnek Output'ları

### `simple` Output:
```
🔐 Simple BIP-39 Demo

Generating 12-word mnemonic...

📝 Mnemonic:
history boy ketchup topic coast mystery lizard like valley tone divert toilet

🔑 Entropy (hex):
6c2355e7f282cd24e0b40df0fc88ff71

🌱 Seed (no passphrase):
09e56d1e5c6904f614081ca2d3930a777df094fae321e37503a9b142ab2fb978...

🔒 Seed (with passphrase 'secret'):
72914c10c2fafaf0d218a46edc6a9700166ef1f06bc29f89639c63890dbcd12b...

✅ Done! Notice how the seeds are different!
```

### `quick_test` Output (Kısa):
```
📋 TEST 1: Different Word Counts
─────────────────────────────────
12 words: ✅
15 words: ✅
18 words: ✅
21 words: ✅
24 words: ✅

📋 TEST 2: BIP-39 Test Vector
─────────────────────────────────
✅ Test vector is valid!
✅ Seed matches BIP-39 test vector!

📋 TEST 3: Invalid Checksum
─────────────────────────────────
✅ Invalid checksum detected!

📋 TEST 4: Passphrase Effect
─────────────────────────────────
✅ All seeds are different!

╔════════════════════════════════╗
║  ✅ All Tests Completed!      ║
╚════════════════════════════════╝
```

---

## ❓ Sorun Giderme

### Hata: "cargo: command not found"
```bash
# Rust yüklü değil, yükle:
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Hata: Build fails
```bash
# Önce temizle
cargo clean

# Sonra tekrar dene
cargo build
```

### Hata: "example not found"
```bash
# examples/ klasörü var mı kontrol et
ls examples/

# Yoksa:
mkdir -p examples
# README'deki örnekleri kopyala
```

---

## 🎓 Öğrenme Yolu

### 1. Başlangıç (5 dakika)
```bash
cargo test                    # Çalışıyor mu?
cargo run --example simple    # İlk mnemonic
```

### 2. Keşif (10 dakika)
```bash
cargo run --example quick_test    # Tüm özellikler
cargo run --example interactive   # Eğlenceli mod
```

### 3. Derinlemesine (20 dakika)
```bash
cargo test -- --nocapture          # Test detayları
cargo doc --open                   # Dokümantasyon
# Kendi kodunu yaz!
```

---

## 🚀 Hemen Başla!

```bash
cd PURE_BIP39

# En hızlı başlangıç:
cargo run --example interactive

# Veya:
cargo run --example simple

# Veya:
cargo test && echo "✅ Her şey çalışıyor!"
```

**İyi eğlenceler!** 🎉
