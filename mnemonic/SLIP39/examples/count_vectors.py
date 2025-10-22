#!/usr/bin/env python3
import json

with open('../tests/vectors.json') as f:
    data = json.load(f)

valid = [(i+1, v[0]) for i,v in enumerate(data) if v[2] != '']
invalid = [(i+1, v[0]) for i,v in enumerate(data) if v[2] == '']

print(f'Total test vectors: {len(data)}')
print(f'\nValid vectors: {len(valid)}')
for idx, desc in valid:
    print(f'  {idx}. {desc}')

print(f'\nInvalid vectors: {len(invalid)}')
for idx, desc in invalid:
    print(f'  {idx}. {desc}')
