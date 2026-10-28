#!/usr/bin/env python3
"""Render scenarios with the shipped image and with the ELF and show where they differ.

usage: compare_builds.py <img> <elf> <scenario.json>... [--fz]
Writes out/<name>.{ship,elf}.raw and runs tools/raw_diff.py on each pair.
"""
import os
import subprocess
import sys

root = os.path.join(os.path.dirname(os.path.abspath(__file__)), '..')
exe = os.path.join(root, 'target', 'release', 'lxr-emu')
img, elf, *rest = sys.argv[1:]
extra = [a for a in rest if a.startswith('--')]
files = [a for a in rest if not a.startswith('--')]
os.makedirs(os.path.join(root, 'out'), exist_ok=True)
for f in files:
    name = os.path.splitext(os.path.basename(f))[0]
    outs = []
    for tag, opt in (('ship', ['--img', img]), ('elf', ['--elf', elf])):
        o = os.path.join(root, 'out', f'{name}.{tag}.raw')
        subprocess.run([exe, 'render', *opt, f, '--out', o, *extra], check=True, capture_output=True)
        outs.append(o)
    print('==', name)
    subprocess.run([sys.executable, os.path.join(root, 'tools', 'raw_diff.py'), *outs, f], check=True)
