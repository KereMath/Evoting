#!/usr/bin/env python3
"""
Compare our Rust interpolation with Python Trezor implementation byte-by-byte
"""

import shamir_mnemonic as slip39

# Parse the mnemonics
mnemonics = [
    "shadow pistol academic always adequate wildlife fancy gross oasis cylinder mustang wrist rescue view short owner flip making coding armed",
    "shadow pistol academic acid actress prayer class unknown daughter sweater depict flip twice unkind craft early superior advocate guest smoking",
]

# Parse shares
shares = [slip39.Share.from_mnemonic(m) for m in mnemonics]

print("=== Share Information ===")
for i, share in enumerate(shares):
    print(f"Share {i}: index={share.index}, value_len={len(share.value)}")
    print(f"  Value (hex): {share.value.hex()}")
    print(f"  First 8 bytes: {share.value[:8].hex()}")

# Now use internal functions to reconstruct
from shamir_mnemonic.shamir import _interpolate, RawShare, SECRET_INDEX, DIGEST_INDEX

raw_shares = [RawShare(s.index, s.value) for s in shares]

print(f"\n=== Interpolation at x={SECRET_INDEX} (SECRET_INDEX) ===")
secret = _interpolate(raw_shares, SECRET_INDEX)
print(f"Encrypted secret: {secret.hex()}")

print(f"\n=== Interpolation at x={DIGEST_INDEX} (DIGEST_INDEX) ===")
digest_share = _interpolate(raw_shares, DIGEST_INDEX)
print(f"Digest share: {digest_share.hex()}")

# Now decrypt
passphrase = b"TREZOR"
from shamir_mnemonic.shamir import EncryptedMasterSecret
identifier = shares[0].identifier
iteration_exponent = shares[0].iteration_exponent
extendable = shares[0].extendable

ems = EncryptedMasterSecret(identifier, extendable, iteration_exponent, secret)
master_secret = ems.decrypt(passphrase)

print(f"\n=== Decryption ===")
print(f"Master secret: {master_secret.hex()}")
print(f"Expected:      b43ceb7e57a0ea8766221624d01b0864")
print(f"Match: {master_secret.hex() == 'b43ceb7e57a0ea8766221624d01b0864'}")
