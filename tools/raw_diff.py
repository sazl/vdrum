#!/usr/bin/env python3
"""Compare two lxr-emu raw renders; print the first differing block and the scenario
events up to it.

usage: raw_diff.py <a.raw> <b.raw> [scenario.json]
"""
import json
import struct
import sys

a = open(sys.argv[1], 'rb').read()
b = open(sys.argv[2], 'rb').read()
n = min(len(a), len(b)) // 512
first = None
ndiff = 0
for blk in range(n):
    if a[blk * 512:(blk + 1) * 512] != b[blk * 512:(blk + 1) * 512]:
        ndiff += 1
        if first is None:
            first = blk
print(f'{n} blocks, {ndiff} differ, first {first}')
if first is not None:
    sa = struct.unpack('<256h', a[first * 512:(first + 1) * 512])
    sb = struct.unpack('<256h', b[first * 512:(first + 1) * 512])
    d = [(i, x, y) for i, (x, y) in enumerate(zip(sa, sb)) if x != y]
    print('first samples', d[:8], 'max abs diff', max(abs(x - y) for _, x, y in d))
    if len(sys.argv) > 3:
        sc = json.load(open(sys.argv[3]))
        evs = sorted(((e['tick'], ev) for e in sc['events'] for ev in e['events'] if e['tick'] <= first), key=lambda x: x[0])
        for t, ev in evs[-12:]:
            print(t, json.dumps(ev)[:160])
