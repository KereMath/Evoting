# Manuel Kullanım Kılavuzu - Shamir SSS

Bu dokümanda PURE_BIP39 ve SHAMIR_SSS'i elle nasıl kullanacağını adım adım öğreneceksin.

---

## 🎮 İnteraktif CLI Tool (En Kolay Yol!)

### Başlat

```bash
cd SHAMIR_SSS
cargo run --example cli_tool
```

### Ana Menü

```
🔐 Shamir Secret Sharing - Interactive CLI Tool
=================================================

📋 Main Menu:
  1. Generate new BIP39 mnemonic (PURE_BIP39)
  2. Split mnemonic into shares (SHAMIR_SSS)
  3. Save shares to files
  4. Load shares and reconstruct mnemonic
  5. Full workflow (all steps)
  6. Exit
```

---

## 📝 Adım Adım Manuel Test

### Senaryo 1: Yeni Mnemonic Üret ve Böl

#### Adım 1: Mnemonic Üret
```bash
cargo run --example cli_tool
# Seçim: 1

# Kelime sayısı seç:
# 1 = 12 kelime
# 2 = 15 kelime
# 3 = 18 kelime
# 4 = 21 kelime
# 5 = 24 kelime (en güvenli)

# Seçim: 5
```

**Çıktı:**
```
✅ Mnemonic generated successfully!

┌─────────────────────────────────────────────┐
│ YOUR MNEMONIC PHRASE (WRITE IT DOWN!)      │
└─────────────────────────────────────────────┘

abandon ability able about above absent absorb abstract
absurd abuse access accident account accuse achieve acid
acoustic acquire across act action actor actress actual

📊 Details:
  - Word count: 24
  - Entropy: 256 bits
  - Language: English

💾 Mnemonic saved to: temp_mnemonic.txt
```

#### Adım 2: Shamir'e Böl
```bash
# Ana menüden seçim: 2

# Load mnemonic from temp_mnemonic.txt? (y/n): y

# Enter threshold: 3
# Enter total shares: 5
```

**Çıktı:**
```
✅ Successfully created 5 shares!

Share #1/5:
  ID: 1
  Data: 9f03eeb9480d9d35...c8
  Full length: 72 characters

Share #2/5:
  ID: 2
  Data: cc2d6adba4981b1c...a2
  Full length: 72 characters

...

💾 Shares saved to: temp_shares.json
```

#### Adım 3: Dosyalara Kaydet
```bash
# Ana menüden seçim: 3
```

**Çıktı:**
```
✅ Saved: shares/share_1.json
✅ Saved: shares/share_2.json
✅ Saved: shares/share_3.json
✅ Saved: shares/share_4.json
✅ Saved: shares/share_5.json

📁 All shares saved to 'shares/' directory
```

**Dosya Yapısı:**
```
SHAMIR_SSS/
├── shares/
│   ├── share_1.json  ← Kasada sakla
│   ├── share_2.json  ← Bankada sakla
│   ├── share_3.json  ← Arkadaşta sakla
│   ├── share_4.json  ← Avukatta sakla
│   └── share_5.json  ← Bulutta (şifreli) sakla
├── temp_mnemonic.txt
└── temp_shares.json
```

#### Adım 4: Geri Oluştur (Reconstruct)
```bash
# Ana menüden seçim: 4

# Found 5 share files
# Share numbers (e.g., 1,2,3): 1,3,5
```

**Çıktı:**
```
⏳ Loading 3 shares...
  ✅ Loaded share #1
  ✅ Loaded share #3
  ✅ Loaded share #5

⏳ Reconstructing mnemonic...

✅ RECONSTRUCTION SUCCESSFUL!

┌─────────────────────────────────────────────┐
│ RECOVERED MNEMONIC PHRASE                   │
└─────────────────────────────────────────────┘

abandon ability able about above absent absorb abstract
absurd abuse access accident account accuse achieve acid
acoustic acquire across act action actor actress actual

🎉 PERFECT MATCH! Recovered mnemonic matches original!
✨ Seeds also match - perfect reconstruction!

💾 Recovered mnemonic saved to: recovered_mnemonic.txt
```

---

### Senaryo 2: Var Olan Mnemonic'i Böl

Eğer zaten bir mnemonic'in varsa:

```bash
cargo run --example cli_tool
# Seçim: 2

# Load mnemonic from temp_mnemonic.txt? (y/n): n

# Enter your BIP39 mnemonic phrase:
> abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about

# Enter threshold: 3
# Enter total shares: 5
```

---

### Senaryo 3: Hızlı Test (Full Workflow)

Tüm adımları otomatik olarak test et:

```bash
cargo run --example cli_tool
# Seçim: 5
```

**Çıktı:**
```
Step 1/4: Generating mnemonic...
✅ Generated: 24 words

Step 2/4: Splitting into shares (3-of-5)...
✅ Created 5 shares

Step 3/4: Saving shares...
✅ Saved to 'shares/' directory

Step 4/4: Reconstructing from first 3 shares...
✅ Reconstructed

🎉 SUCCESS! Perfect reconstruction!
✨ Seeds match - cryptographically verified!
```

---

## 📂 Dosya Formatları

### temp_mnemonic.txt
```
abandon ability able about above absent absorb abstract absurd abuse access accident account accuse achieve acid acoustic acquire across act action actor actress actual
```

### shares/share_1.json
```json
{
  "id": 1,
  "share_data": "9f03eeb9480d9d35bc56d2614b477dbd366eee2714b54aca7459651aa68189a348e56cf8",
  "total_shares": 5,
  "threshold": 3
}
```

---

## 🧪 Farklı Kombinasyonları Test Et

### Test 1: Minimum Shares (Threshold)
```bash
# 3-of-5 scheme için 3 share kullan
cargo run --example cli_tool
# Seçim: 4
# Share numbers: 1,2,3

✅ Should work (exactly threshold)
```

### Test 2: Fazla Shares
```bash
# 3-of-5 scheme için 4 share kullan
# Share numbers: 1,2,3,4

✅ Should work (more than threshold)
```

### Test 3: Yetersiz Shares (Başarısız Olmalı)
```bash
# 3-of-5 scheme için sadece 2 share kullan
# Share numbers: 1,2

❌ Should fail (insufficient shares)
```

### Test 4: Farklı Kombinasyonlar
```bash
# Herhangi 3 share çalışmalı:
1,2,3 ✅
1,3,5 ✅
2,4,5 ✅
1,2,5 ✅
# vs...
```

---

## 🎯 Gerçek Dünya Senaryosu

### Güvenli Backup Stratejisi

```bash
# 1. Mnemonic üret
cargo run --example cli_tool
# Seçim: 1 → 24 kelime

# 2. 5 parçaya böl (threshold=3)
# Seçim: 2 → 3-of-5

# 3. Dosyalara kaydet
# Seçim: 3

# 4. Shares'leri dağıt:
cp shares/share_1.json ~/Desktop/evdeki_kasa/
cp shares/share_2.json ~/Desktop/banka/
cp shares/share_3.json ~/Desktop/arkadas/
cp shares/share_4.json ~/Desktop/avukat/
cp shares/share_5.json ~/Desktop/bulut_sifreli/

# 5. temp dosyalarını güvenli sil
shred -u temp_mnemonic.txt temp_shares.json
```

### Recovery Senaryosu (10 Yıl Sonra)

```bash
# Diyelim share_2 ve share_4 kayıp
# Sadece share_1, share_3, share_5 var

# 1. Shares klasörü oluştur
mkdir -p shares

# 2. Sahip olduğun shares'leri kopyala
cp ~/evdeki_kasa/share_1.json shares/
cp ~/arkadas/share_3.json shares/
cp ~/bulut_sifreli/share_5.json shares/

# 3. Reconstruct
cargo run --example cli_tool
# Seçim: 4
# Share numbers: 1,3,5

✅ Mnemonic kurtarıldı!
```

---

## 🔍 Manuel Verification

### Seed'leri Karşılaştır

```bash
# Original seed
echo "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about" > original.txt

# Recovered seed
cat recovered_mnemonic.txt

# Python ile karşılaştır (opsiyonel)
python3 << 'EOF'
from hashlib import pbkdf2_hmac

def mnemonic_to_seed(mnemonic):
    password = mnemonic.encode('utf-8')
    salt = b'mnemonic'
    return pbkdf2_hmac('sha512', password, salt, 2048, 64)

original = open('original.txt').read().strip()
recovered = open('recovered_mnemonic.txt').read().strip()

seed1 = mnemonic_to_seed(original)
seed2 = mnemonic_to_seed(recovered)

if seed1 == seed2:
    print("✅ Seeds match!")
else:
    print("❌ Seeds don't match!")
EOF
```

---

## 📊 Performance Testing

### Benchmark Different Configurations

```bash
# Test (2, 3)
time cargo run --example cli_tool
# Seçim: 2 → threshold=2, total=3

# Test (3, 5)
time cargo run --example cli_tool
# Seçim: 2 → threshold=3, total=5

# Test (5, 9)
time cargo run --example cli_tool
# Seçim: 2 → threshold=5, total=9

# Test (10, 20)
time cargo run --example cli_tool
# Seçim: 2 → threshold=10, total=20
```

---

## 🛠️ Troubleshooting

### Problem: "No shares found!"

```bash
# Çözüm: Önce shares oluştur
cargo run --example cli_tool
# Seçim: 1 (mnemonic üret)
# Seçim: 2 (split)
# Seçim: 3 (save)
```

### Problem: "Not enough shares!"

```bash
# Hata mesajı: Need at least 3 but got 2

# Çözüm: Daha fazla share kullan
# Share numbers: 1,2,3  (3 tane)
```

### Problem: "Invalid mnemonic!"

```bash
# Çözüm: BIP39 wordlist'inden kelime kullan
# 12, 15, 18, 21, veya 24 kelime olmalı
# Checksum geçerli olmalı
```

---

## 📝 Cheat Sheet

### Hızlı Komutlar

```bash
# Yeni mnemonic + split + save (all in one)
cargo run --example cli_tool
# → Seçim 5

# Sadece mnemonic üret
cargo run --example cli_tool
# → Seçim 1

# Var olan mnemonic'i böl
cargo run --example cli_tool
# → Seçim 2 → n (manual entry)

# Shares'lerden reconstruct
cargo run --example cli_tool
# → Seçim 4 → 1,2,3
```

### Dosya İşlemleri

```bash
# Shares'leri listele
ls -lh shares/

# Share içeriğini gör
cat shares/share_1.json | jq

# Mnemonic'i gör
cat temp_mnemonic.txt

# Hepsini temizle
rm -rf shares/ temp_*.txt recovered_*.txt
```

---

## 🎓 Öğrenme Yolu

### Seviye 1: Başlangıç
1. ✅ Full workflow çalıştır (Seçim 5)
2. ✅ Çıktıları incele
3. ✅ Dosyaları kontrol et

### Seviye 2: Adım Adım
1. ✅ Mnemonic üret (Seçim 1)
2. ✅ Manuel böl (Seçim 2)
3. ✅ Kaydet (Seçim 3)
4. ✅ Reconstruct (Seçim 4)

### Seviye 3: Deneysel
1. ✅ Farklı threshold değerleri dene
2. ✅ Farklı share kombinasyonları test et
3. ✅ Shares'leri sil ve recovery dene
4. ✅ Kendi mnemonic'inle test et

---

## 🔐 Güvenlik Kuralları

1. ⚠️ **temp_mnemonic.txt'yi asla paylaşma**
2. ⚠️ **Her share'i farklı yerde sakla**
3. ⚠️ **Minimum threshold kadar share'e erişim garantile**
4. ⚠️ **Testleri gerçek fonlarla yapma**
5. ⚠️ **Production'da kullanmadan önce audit et**

---

## ✅ Checklist

Manuel test öncesi:
- [ ] `cargo build --example cli_tool` başarılı
- [ ] `shares/` klasörü yok (temiz başlangıç)
- [ ] `temp_*.txt` dosyaları yok

Manuel test sonrası:
- [ ] Mnemonic generate edildi
- [ ] Shares oluşturuldu
- [ ] Dosyalara kaydedildi
- [ ] Reconstruction başarılı
- [ ] Seeds eşleşti ✅

---

**Son Güncelleme:** 2025-01-10
**Versiyon:** 1.0.0
**Status:** ✅ TESTED & WORKING
