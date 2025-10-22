#!/usr/bin/env python3
"""
Debug test vector with Trezor's python-shamir-mnemonic
"""
import sys

# First, let's manually implement the cipher to see exact values
import hashlib

mnemonic = "duckling enlarge academic academic agency result length solution fridge kidney coal piece deal husband erode duke ajar critical decision keyboard"
expected_master = bytes.fromhex("bb54aac4b89dc868ba37d9cc21b2cece")
passphrase = b"TREZOR"

print("="*60)
print("MANUAL CIPHER DEBUG")
print("="*60)
print(f"Expected master secret: {expected_master.hex()}")
print(f"Passphrase: {passphrase}")

# Parse share to get identifier and encrypted value
# We need to manually get the word indices first
# But let's try with shamir_mnemonic if available

try:
    from shamir_mnemonic import shamir
    from shamir_mnemonic import share as share_module
    
    print("\n" + "="*60)
    print("USING TREZOR LIBRARY")
    print("="*60)
    
    # Parse the mnemonic
    parsed_share = share_module.Share.from_mnemonic(mnemonic)
    
    print(f"\nParsed Share:")
    print(f"  Identifier: {parsed_share.identifier}")
    print(f"  Iteration exp: {parsed_share.iteration_exponent}")
    print(f"  Extendable: {parsed_share.extendable}")
    print(f"  Group index: {parsed_share.group_index}")
    print(f"  Group threshold: {parsed_share.group_threshold}")
    print(f"  Member threshold: {parsed_share.member_threshold}")
    print(f"  Share value: {parsed_share.share_value.hex()}")
    
    # Now decrypt the share value
    from shamir_mnemonic import cipher
    
    decrypted = cipher.decrypt(
        parsed_share.share_value,
        passphrase,
        parsed_share.iteration_exponent,
        parsed_share.identifier,
        parsed_share.extendable
    )
    
    print(f"\nDecrypted from share value:")
    print(f"  Result: {decrypted.hex()}")
    print(f"  Expected: {expected_master.hex()}")
    print(f"  Match: {decrypted == expected_master}")
    
    # Also try encrypting the expected master
    encrypted = cipher.encrypt(
        expected_master,
        passphrase,
        parsed_share.iteration_exponent,
        parsed_share.identifier,
        parsed_share.extendable
    )
    
    print(f"\nEncrypted expected master:")
    print(f"  Result: {encrypted.hex()}")
    print(f"  Share value: {parsed_share.share_value.hex()}")
    print(f"  Match: {encrypted == parsed_share.share_value}")
    
except ImportError as e:
    print(f"\nTrezor library not available: {e}")
    print("Install with: pip install shamir-mnemonic")
    sys.exit(1)

