#!/usr/bin/env python3
"""
Test vector 4 with actual Trezor slip39 library
"""

try:
    import shamir_mnemonic as slip39
    print("Using actual Trezor shamir_mnemonic library")
    HAS_TREZOR = True
except ImportError:
    print("Trezor shamir_mnemonic library not available")
    HAS_TREZOR = False
    exit(1)

mnemonics = [
    "shadow pistol academic always adequate wildlife fancy gross oasis cylinder mustang wrist rescue view short owner flip making coding armed",
    "shadow pistol academic acid actress prayer class unknown daughter sweater depict flip twice unkind craft early superior advocate guest smoking",
]

passphrase = b"TREZOR"

try:
    # Check API
    print(f"Available functions: {[x for x in dir(slip39) if not x.startswith('_')]}")

    # Try combine_mnemonics which is the main recovery function
    master_secret = slip39.combine_mnemonics(mnemonics, passphrase)
    print(f"\nMaster Secret: {master_secret.hex()}")
    print(f"Expected:      b43ceb7e57a0ea8766221624d01b0864")
    print(f"Match: {master_secret.hex() == 'b43ceb7e57a0ea8766221624d01b0864'}")
except Exception as e:
    print(f"Error: {e}")
    import traceback
    traceback.print_exc()
