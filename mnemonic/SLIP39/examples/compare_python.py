#!/usr/bin/env python3
"""
Compare our interpolation with Python's reference implementation.
This uses the exact Python code from Trezor's slip39.py
"""

# Python reference GF(256) tables
def _precompute_exp_log():
    exp = [0] * 255
    log = [0] * 256

    poly = 1
    for i in range(255):
        exp[i] = poly
        log[poly] = i

        # Multiply poly by (x + 1)
        poly = (poly << 1) ^ poly

        # Reduce poly by x^8 + x^4 + x^3 + x + 1
        if poly & 0x100:
            poly ^= 0x11B

    return exp, log

EXP_TABLE, LOG_TABLE = _precompute_exp_log()

def interpolate(shares, x):
    """
    Returns f(x) given the Shamir shares (x_1, y_1), ... , (x_k, y_k).
    """
    x_coordinates = set(share[0] for share in shares)

    if x in x_coordinates:
        for share in shares:
            if share[0] == x:
                return share[1]

    # Use logarithms for efficient computation in GF(256)
    log_prod = sum(LOG_TABLE[share[0] ^ x] for share in shares)

    result = bytes(
        _interpolate_byte(i, shares, x, log_prod) for i in range(len(shares[0][1]))
    )
    return result

def _interpolate_byte(byte_index, shares, x, log_prod):
    """
    Interpolates a single byte.
    """
    result = 0
    for share in shares:
        x_i, share_value = share[0], share[1]
        y_i = share_value[byte_index]

        if y_i == 0:
            continue

        # Compute log of Lagrange basis polynomial at x
        log_basis = log_prod - LOG_TABLE[x_i ^ x]
        for other_share in shares:
            x_j = other_share[0]
            log_basis -= LOG_TABLE[x_i ^ x_j]

        # Normalize to 0..254 range
        log_basis = log_basis % 255

        # y_i * L_i(x)
        result ^= EXP_TABLE[(LOG_TABLE[y_i] + log_basis) % 255]

    return result

# Test with vector 4 values
print("=== Python Reference Implementation ===")
print(f"First 10 EXP: {EXP_TABLE[:10]}")
print(f"First 10 LOG: {LOG_TABLE[:10]}")
print(f"LOG[1]={LOG_TABLE[1]}, EXP[0]={EXP_TABLE[0]}")
print(f"LOG[2]={LOG_TABLE[2]}, EXP[1]={EXP_TABLE[1]}")
print(f"LOG[3]={LOG_TABLE[3]}, EXP[25]={EXP_TABLE[25]}")

# Share values from our test (first 4 bytes)
share1 = (3, bytes.fromhex("08fb14b6"))  # member_index=2, x=3
share2 = (1, bytes.fromhex("06ab48fe"))  # member_index=0, x=1

shares = [share1, share2]
x = 255

print(f"\n=== Test Vector 4: Interpolate at x={x} ===")
print(f"Share 1: x={share1[0]}, first 4 bytes: {share1[1].hex()}")
print(f"Share 2: x={share2[0]}, first 4 bytes: {share2[1].hex()}")

# Compute log_prod
log_prod = sum(LOG_TABLE[share[0] ^ x] for share in shares)
print(f"\nlog_prod calculation:")
for share in shares:
    diff = share[0] ^ x
    print(f"  {share[0]} ^ {x} = {diff}, LOG[{diff}] = {LOG_TABLE[diff]}")
print(f"log_prod = {log_prod}")

# Interpolate first byte manually
print(f"\n=== Manual byte 0 interpolation ===")
byte_index = 0

for i, share in enumerate(shares):
    x_i = share[0]
    y_i = share[1][byte_index]

    print(f"\n--- Share {i+1}: x={x_i}, y={y_i} ---")

    if y_i == 0:
        print("  y_i is 0, skipping")
        continue

    log_basis = log_prod - LOG_TABLE[x_i ^ x]
    print(f"  log_basis = {log_prod} - LOG[{x_i ^ x}] = {log_prod} - {LOG_TABLE[x_i ^ x]} = {log_basis}")

    for j, other_share in enumerate(shares):
        x_j = other_share[0]
        diff = x_i ^ x_j
        print(f"  Subtract LOG[{x_i} ^ {x_j}] = LOG[{diff}] = {LOG_TABLE[diff]}")
        log_basis -= LOG_TABLE[diff]

    print(f"  After all subtractions: log_basis = {log_basis}")
    log_basis = log_basis % 255
    print(f"  After mod 255: log_basis = {log_basis}")

    log_y = LOG_TABLE[y_i]
    log_result = (log_y + log_basis) % 255
    contrib = EXP_TABLE[log_result]

    print(f"  LOG[y={y_i}] = {log_y}")
    print(f"  log_result = ({log_y} + {log_basis}) % 255 = {log_result}")
    print(f"  contribution = EXP[{log_result}] = {contrib}")

# Now do it properly with the function
result = interpolate(shares, x)
print(f"\n=== Python interpolate() result ===")
print(f"First 4 bytes: {result.hex()}")
print(f"First byte decimal: {result[0]}")
print(f"Expected: 0xb4 (180)")
