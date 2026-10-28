#!/usr/bin/env python3
"""Match the functions of a firmware ELF (linked with -Wl,--emit-relocs) against a
raw firmware binary (e.g. the shipped v1.71 payload) with relocated bytes masked.

usage: function_match.py <fw.elf> <shipped.bin> [out.json]

For every STT_FUNC symbol of at least 8 bytes, the function bytes are taken from
the ELF's load image (LMA), every byte covered by a relocation is masked, and the
masked pattern is searched in the shipped binary (loaded at 0x08008000). A function
"matches" when exactly one position in the shipped binary has identical unmasked
bytes. Output: {name: {ship_addr (LMA in the shipped image), size, our_addr (VMA)}
| {ambiguous: [...]} | {unmatched: true}}.

The module is also imported by tools/symbol_map.py.
"""
import json
import sys

from elftools.elf.elffile import ELFFile
from elftools.elf.relocation import RelocationSection

APP_BASE = 0x08008000


class Elf:
    """ELF with load image (by LMA), VMA->LMA mapping, symbols and relocations."""

    def __init__(self, path):
        self.f = ELFFile(open(path, 'rb'))
        self.segs = [(s['p_vaddr'], s['p_paddr'], s['p_filesz'], s['p_memsz'])
                     for s in self.f.iter_segments() if s['p_type'] == 'PT_LOAD']
        # load image indexed by LMA - APP_BASE
        end = max(pa + fs for va, pa, fs, ms in self.segs if fs)
        img = bytearray(end - APP_BASE)
        for seg in self.f.iter_segments():
            if seg['p_type'] != 'PT_LOAD' or not seg['p_filesz']:
                continue
            o = seg['p_paddr'] - APP_BASE
            data = seg.data()
            if o < 0:  # segment starting below the application (ELF headers)
                data = data[-o:]
                o = 0
            img[o:o + len(data)] = data
        self.image = bytes(img)
        self.relocs = []  # (vma, type, sym_index)
        for sec in self.f.iter_sections():
            if isinstance(sec, RelocationSection):
                tgt = self.f.get_section(sec['sh_info'])
                if not tgt['sh_flags'] & 2:  # SHF_ALLOC
                    continue
                for r in sec.iter_relocations():
                    self.relocs.append((r['r_offset'], r['r_info_type'], r['r_info_sym']))
        self.relocs.sort()
        self.symtab = self.f.get_section_by_name('.symtab')
        self.syms = list(self.symtab.iter_symbols())

    def v2off(self, a):
        """VMA -> offset into the load image (None if not file-backed)."""
        for va, pa, fs, ms in self.segs:
            if va <= a < va + fs:
                o = pa + (a - va) - APP_BASE
                return o if o >= 0 else None
        return None

    def word(self, vma):
        o = self.v2off(vma)
        return int.from_bytes(self.image[o:o + 4], 'little')

    def half(self, vma):
        o = self.v2off(vma)
        return int.from_bytes(self.image[o:o + 2], 'little')

    def functions(self, min_size=8):
        out = set()
        for s in self.syms:
            if s['st_info']['type'] == 'STT_FUNC' and s['st_size'] >= min_size:
                out.add((s.name, s['st_value'] & ~1, s['st_size']))
        return sorted(out, key=lambda x: x[1])

    def masked_set(self):
        m = set()
        for a, t, _ in self.relocs:
            for k in range(4):
                m.add(a + k)
        return m


def match_functions(elf, ship, min_size=8):
    masked = elf.masked_set()
    res = {}
    for name, a, sz in elf.functions(min_size):
        o = elf.v2off(a)
        if o is None:
            continue
        body = elf.image[o:o + sz]
        mask = [(a + i) in masked for i in range(sz)]
        # longest unmasked run as search key
        best = (0, 0)
        i = 0
        while i < sz:
            if mask[i]:
                i += 1
                continue
            j = i
            while j < sz and not mask[j]:
                j += 1
            if j - i > best[1] - best[0]:
                best = (i, j)
            i = j
        seg = body[best[0]:best[1]]
        hits = []
        if len(seg) >= 4:
            p = ship.find(seg)
            while p >= 0 and len(hits) < 50:
                st = p - best[0]
                if st >= 0 and st + sz <= len(ship) and all(mask[k] or ship[st + k] == body[k] for k in range(sz)):
                    hits.append(st)
                p = ship.find(seg, p + 1)
        hits = sorted(set(hits))
        if len(hits) == 1:
            res[name] = {'ship_addr': hex(APP_BASE + hits[0]), 'size': sz, 'our_addr': hex(a)}
        elif len(hits) > 1:
            res[name] = {'ambiguous': [hex(APP_BASE + h) for h in hits[:5]], 'size': sz}
        else:
            res[name] = {'unmatched': True, 'size': sz}
    return res


def main():
    elf = Elf(sys.argv[1])
    ship = open(sys.argv[2], 'rb').read()
    res = match_functions(elf, ship)
    n = len(res)
    m = sum('ship_addr' in v for v in res.values())
    amb = sum('ambiguous' in v for v in res.values())
    print(f'funcs {n} unique {m} ambiguous {amb} unmatched {n - m - amb}')
    if len(sys.argv) > 3:
        json.dump(res, open(sys.argv[3], 'w'), indent=1)


if __name__ == '__main__':
    main()
