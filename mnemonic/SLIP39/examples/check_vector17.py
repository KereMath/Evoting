#!/usr/bin/env python3
import json

with open('../tests/vectors.json') as f:
    vectors = json.load(f)

v = vectors[16]  # Vector 17 (0-indexed)
print('Vector 17:')
print('Description:', v[0])
print('Expected secret:', v[2])
print('Mnemonics count:', len(v[1]))
print('\nMnemonics:')
for i, m in enumerate(v[1], 1):
    print(f'  {i}. {m}')
