#!/usr/bin/env python3
"""Analyze test results and create a summary"""

with open('../test_output.txt', 'r', encoding='utf-8') as f:
    content = f.read()

lines = content.split('\n')

print("=" * 70)
print("SLIP-39 Official Test Vectors - Complete Results")
print("=" * 70)

# Parse valid vectors
valid_results = []
for i, line in enumerate(lines):
    if line.startswith("Testing valid vector"):
        vec_num = int(line.split("vector ")[1].split(":")[0])
        # Look ahead for Pass/Fail
        passed = False
        for j in range(i, min(i+30, len(lines))):
            if "✅ Pass" in lines[j]:
                passed = True
                break
            if f"Testing valid vector {vec_num+1}:" in lines[j] or f"Testing invalid vector" in lines[j]:
                break
        desc = line.split(f"{vec_num}: ")[1] if f"{vec_num}: " in line else ""
        valid_results.append((vec_num, desc, passed))

# Parse invalid vectors
invalid_results = []
for i, line in enumerate(lines):
    if line.startswith("Testing invalid vector"):
        vec_num = int(line.split("vector ")[1].split(":")[0])
        # Look ahead for rejection
        rejected = False
        for j in range(i, min(i+10, len(lines))):
            if "✅ Correctly rejected" in lines[j]:
                rejected = True
                break
            if f"Testing" in lines[j] and j > i:
                break
        desc = line.split(f"{vec_num}: ")[1] if f"{vec_num}: " in line else ""
        invalid_results.append((vec_num, desc, rejected))

print(f"\n{'VALID VECTORS':<60} {'Result':<10}")
print("-" * 70)
valid_passed = 0
for num, desc, passed in sorted(valid_results):
    status = "[PASS]" if passed else "[FAIL]"
    if passed:
        valid_passed += 1
    print(f"{num:2d}. {desc:<53} {status}")

print(f"\n{'INVALID VECTORS':<60} {'Result':<10}")
print("-" * 70)
invalid_rejected = 0
for num, desc, rejected in sorted(invalid_results):
    status = "[REJECT]" if rejected else "[ACCEPT]"
    if rejected:
        invalid_rejected += 1
    print(f"{num:2d}. {desc:<51} {status}")

print("\n" + "=" * 70)
print(f"SUMMARY:")
print(f"  Valid vectors:   {valid_passed}/{len(valid_results)} passed")
print(f"  Invalid vectors: {invalid_rejected}/{len(invalid_results)} correctly rejected")
print(f"  TOTAL:           {valid_passed + invalid_rejected}/{len(valid_results) + len(invalid_results)} correct")

# Check for test result line
for line in lines:
    if "test result:" in line:
        print(f"\nCargo test result: {line.strip()}")
        break

print("=" * 70)
