#!/usr/bin/env python3
"""Summarise an lxr-emu raw render: per-channel peak/RMS and the first non-silent block.

usage: raw_stats.py <file.raw> [blocks_per_row]
Raw format: per block DAC1[128] then DAC2[128] int16 LE (L/R interleaved).
"""
import struct
import sys

data = open(sys.argv[1], 'rb').read()
per = int(sys.argv[2]) if len(sys.argv) > 2 else 0
n = len(data) // 512
chans = {'out1L': [], 'out1R': [], 'out2L': [], 'out2R': []}
for b in range(n):
    s = struct.unpack('<256h', data[b * 512:(b + 1) * 512])
    d1, d2 = s[:128], s[128:]
    chans['out1L'] += d1[0::2]
    chans['out1R'] += d1[1::2]
    chans['out2L'] += d2[0::2]
    chans['out2R'] += d2[1::2]
print(f'{n} blocks')
for k, v in chans.items():
    nz = next((i for i, x in enumerate(v) if x), None)
    peak = max(abs(x) for x in v) if v else 0
    rms = (sum(x * x for x in v) / len(v)) ** 0.5 if v else 0
    print(f'{k}: peak {peak} rms {rms:.1f} first non-zero sample {nz}')
if per:
    for b in range(0, n, per):
        seg = chans['out1L'][b * 64:(b + per) * 64]
        print(b, max(abs(x) for x in seg))
