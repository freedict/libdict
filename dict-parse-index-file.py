#!/usr/bin/env python
codes = {}
for idx, val in enumerate('ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/'):
    codes[val] = idx


def get_index(value):
    index = 0
    for i, char in enumerate(reversed(value)):
        index += codes[char] * (64**i)
    return index

# dummy test
import os
if not os.path.exists('/tmp/german-english.dict'):
    os.system('cp /usr/share/dictd/german-english.dict.dz /tmp/german-english.dict.gz')
    os.system('gunzip /tmp/german-english.dict.gz')
idx = get_index('3fW2')
print(idx)
end = get_index('c')
with open('/tmp/german-english.dict', 'rb') as f:
    data = f.read()
print(data[idx:idx+end])

