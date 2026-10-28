#!/usr/bin/env python3
"""Recover a symbol map (addresses/sizes) for the shipped LXR-02 firmware binary.

usage: symbol_map.py <fw.elf> <shipped.bin> <out.json>

<fw.elf> is the reference build of the firmware source (tools/build_fw.sh, linked
with -Wl,--emit-relocs so that the final executable keeps its relocations).

1. Functions are matched with tools/function_match.py (relocation-masked bytes).
2. For every relocation inside a matched function (literal pool words R_ARM_ABS32,
   BL/B.W R_ARM_THM_CALL/R_ARM_THM_JUMP24), the value in our build points to a symbol
   X (plus an offset) and the bytes at the same place in the shipped binary hold the
   shipped value. shipped_addr(X) = shipped_value - offset. Every reference is a vote;
   all votes for a symbol must agree (otherwise it is reported as a conflict).
3. Calls through linker veneers (flash <-> ITCM) are followed in both images.
4. Relocations inside recovered data objects (.data initialisers, .rodata tables,
   vector table) are processed the same way until nothing new is found.
5. Functions that did not match byte-for-byte are recovered from call sites/pointers.

Addresses are runtime addresses (ITCM functions at their ITCM address, with their
load address in the image as "lma"). Thumb bits are cleared.
"""
import bisect
import json
import sys
from collections import Counter, defaultdict

sys.path.insert(0, __import__('os').path.dirname(__file__))
from function_match import APP_BASE, Elf, match_functions  # noqa: E402

R_ARM_ABS32 = 2
R_ARM_THM_CALL = 10
R_ARM_THM_JUMP24 = 30
ITCM_END = 0x4000
# literal pool bytes after the symbol size of assembler functions
TRAILING_POOL = {'Reset_Handler': 48}


def sext(v, bits):
    return v - (1 << bits) if v & (1 << (bits - 1)) else v


def decode_bl(hw1, hw2, pc):
    """Target of a Thumb-2 BL / B.W (T4) at address pc. None if not such an instruction."""
    if (hw1 & 0xF800) != 0xF000 or (hw2 & 0xD000) not in (0xD000, 0x9000, 0xC000):
        return None
    s = (hw1 >> 10) & 1
    j1 = (hw2 >> 13) & 1
    j2 = (hw2 >> 11) & 1
    i1 = (~(j1 ^ s)) & 1
    i2 = (~(j2 ^ s)) & 1
    imm = (s << 24) | (i1 << 23) | (i2 << 22) | ((hw1 & 0x3FF) << 12) | ((hw2 & 0x7FF) << 1)
    imm = sext(imm, 25)
    if (hw2 & 0xD000) == 0xC000:  # BLX (to ARM) - not used on M profile
        return ((pc + 4) & ~3) + imm
    return pc + 4 + imm


class Ship:
    def __init__(self, data):
        self.data = data
        self.itcm_lma = None  # LMA of runtime address 0 (ITCM)
        self.data_map = []  # (vma_start, vma_end, lma_start) for initialised RAM

    def lma_of(self, runtime):
        if runtime < ITCM_END:
            if self.itcm_lma is None:
                return None
            return self.itcm_lma + runtime
        if APP_BASE <= runtime < APP_BASE + len(self.data):
            return runtime
        for vs, ve, ls in self.data_map:
            if vs <= runtime < ve:
                return ls + (runtime - vs)
        return None

    def word_lma(self, lma):
        o = lma - APP_BASE
        if o < 0 or o + 4 > len(self.data):
            return None
        return int.from_bytes(self.data[o:o + 4], 'little')

    def half_lma(self, lma):
        o = lma - APP_BASE
        if o < 0 or o + 2 > len(self.data):
            return None
        return int.from_bytes(self.data[o:o + 2], 'little')


def is_veneer(read_half, lma):
    """ldr.w pc, [pc, #-0] ; .word target"""
    return read_half(lma) == 0xF85F and read_half(lma + 2) == 0xF000


def main():
    elf = Elf(sys.argv[1])
    ship = Ship(open(sys.argv[2], 'rb').read())
    out_path = sys.argv[3]

    # ---- ELF symbol tables -------------------------------------------------------
    cur_file = None
    syms = []  # (addr, size, key, type)
    by_name = defaultdict(list)
    idx_name = {}
    for idx, s in enumerate(elf.syms):
        t = s['st_info']['type']
        if t == 'STT_FILE':
            cur_file = s.name
            continue
        if not s.name or s.name.startswith('$') or t == 'STT_SECTION':
            continue
        if s['st_shndx'] in ('SHN_UNDEF', 'SHN_ABS') and t != 'STT_NOTYPE':
            continue
        addr = s['st_value'] & ~1 if t == 'STT_FUNC' else s['st_value']
        idx_name[idx] = (s.name, addr)
        local = s['st_info']['bind'] == 'STB_LOCAL'
        by_name[s.name].append((addr, s['st_size'], t, local, cur_file))
    # disambiguate duplicate names (static symbols in several files)
    key_of = {}
    for name, lst in by_name.items():
        uniq = {(a, sz, t) for a, sz, t, l, f in lst}
        for a, sz, t, l, f in lst:
            k = name if len(uniq) == 1 else f'{name}@{f}'
            key_of[(name, a)] = k
            syms.append((a, sz, k, t))
    syms = sorted(set(syms))
    sym_info = {k: (a, sz, t) for a, sz, k, t in syms}
    funcs_at = defaultdict(list)
    exact_at = defaultdict(list)
    objects = []
    for a, sz, k, t in syms:
        if t == 'STT_FUNC':
            funcs_at[a].append(k)
        elif t == 'STT_OBJECT':
            objects.append((a, sz, k))
        if t in ('STT_NOTYPE', 'STT_OBJECT'):
            exact_at[a].append(k)
    objects.sort()
    obj_addrs = [o[0] for o in objects]

    def containing_object(v):
        i = bisect.bisect_right(obj_addrs, v) - 1
        best = []
        while i >= 0 and objects[i][0] > v - 0x100000:
            a, sz, k = objects[i]
            if a <= v < a + max(sz, 1):
                best.append((a, sz, k))
            i -= 1
            if len(best) and objects[i][0] < best[0][0] - 0x10000:
                break
        return best

    # ELF veneers: key -> target address
    def elf_lma(vma):
        o = elf.v2off(vma)
        return None if o is None else o + APP_BASE

    def elf_half(lma):
        o = lma - APP_BASE
        return int.from_bytes(elf.image[o:o + 2], 'little')

    def elf_word(lma):
        o = lma - APP_BASE
        return int.from_bytes(elf.image[o:o + 4], 'little')

    # ---- votes -------------------------------------------------------------------
    votes = defaultdict(Counter)
    evidence = defaultdict(list)
    sources = defaultdict(set)

    ambiguous = []  # ([(key, addr)...], why): candidates for one reference

    def vote(key, ship_addr, why, src):
        votes[key][ship_addr] += 1
        if len(evidence[key]) < 8:
            evidence[key].append(why)
        sources[key].add(src)

    def vote_value(v_elf, v_ship, why, src, rsym=None):
        """v_elf: value in our build, v_ship: value at the same place in the shipped binary.
        rsym: symbol index of the relocation (a named symbol identifies the target exactly;
        section symbols fall back to an address lookup)."""
        if v_ship is None:
            return
        if rsym in idx_name:
            k = key_of[idx_name[rsym]]
            a = sym_info[k][0]
            if sym_info[k][2] == 'STT_FUNC':
                if v_elf & ~1 == a:
                    vote(k, v_ship & ~1, why, src)
                    return
            elif a <= v_elf <= a + max(sym_info[k][1], 0):
                vote(k, v_ship - (v_elf - a), why, src)
                return
        # functions (thumb bit)
        if (v_elf & 1) and (v_elf & ~1) in funcs_at and (v_ship & 1):
            for k in funcs_at[v_elf & ~1]:
                vote(k, v_ship & ~1, why, src)
            return
        if v_elf in funcs_at and not exact_at.get(v_elf):
            for k in funcs_at[v_elf]:
                vote(k, v_ship & ~1, why, src)
            return
        # Section-relative reference: the object containing the address.
        # GCC also emits "array - 1" for pre-increment loops (ldrb rX, [rY, #1]!), which
        # lands in the last bytes of the preceding object. A value within 4 bytes before the
        # start of an object is therefore ambiguous: both candidates get a weak vote, used
        # only when a symbol has no unambiguous reference.
        i = bisect.bisect_right(obj_addrs, v_elf)
        nxt = objects[i] if i < len(objects) and 0 < objects[i][0] - v_elf <= 4 else None
        cont = containing_object(v_elf)
        cands = [(k, v_ship - (v_elf - a)) for a, sz, k in cont]
        if nxt and all(a != v_elf for a, sz, k in cont):
            a, sz, k = nxt
            cands.append((k, v_ship + (a - v_elf)))
            ambiguous.append((cands, why))
            return
        for k, addr in cands:
            vote(k, addr, why, src)

    matches = match_functions(elf, ship.data)
    matched = {}
    for name, r in matches.items():
        if 'ship_addr' in r:
            matched[name] = (int(r['our_addr'], 16), r['size'], int(r['ship_addr'], 16))

    reloc_vmas = [r[0] for r in elf.relocs]

    def relocs_in(vma, size):
        i = bisect.bisect_left(reloc_vmas, vma)
        while i < len(elf.relocs) and elf.relocs[i][0] < vma + size:
            yield elf.relocs[i]
            i += 1

    def process_function(name, vma, size, ship_lma, ship_runtime, with_calls):
        # assembler functions keep their literal pool after the .size'd body
        size += TRAILING_POOL.get(name, 0)
        for off_vma, typ, rsym in relocs_in(vma, size):
            d = off_vma - vma
            if typ == R_ARM_ABS32:
                vote_value(elf.word(off_vma), ship.word_lma(ship_lma + d), f'{name}+{d:#x}', 'literal', rsym)
            elif typ in (R_ARM_THM_CALL, R_ARM_THM_JUMP24) and with_calls and ship_runtime is not None:
                te = decode_bl(elf.half(off_vma), elf.half(off_vma + 2), off_vma)
                h1, h2 = ship.half_lma(ship_lma + d), ship.half_lma(ship_lma + d + 2)
                ts = decode_bl(h1, h2, ship_runtime + d)
                if te is None or ts is None:
                    continue
                follow_call(te, ts, f'{name}+{d:#x}')

    def follow_call(te, ts, why):
        # resolve veneers on both sides
        e_l = elf_lma(te)
        names_e = funcs_at.get(te, [])
        is_ven_e = e_l is not None and is_veneer(elf_half, e_l)
        s_l = ship.lma_of(ts)
        is_ven_s = s_l is not None and is_veneer(ship.half_lma, s_l)
        if is_ven_e:
            for k in names_e:
                vote(k, ts, why, 'call')
            te = elf_word(e_l + 4) & ~1
            names_e = funcs_at.get(te, [])
        if is_ven_s:
            ts = ship.word_lma(s_l + 4) & ~1
        for k in names_e:
            vote(k, ts, why, 'call')

    # Pass 1: literal pools of matched functions (linker symbols, data, pointers)
    for name, (vma, size, lma) in matched.items():
        process_function(name, vma, size, lma, None, False)

    def resolved(k):
        c = votes.get(k)
        if not c:
            return None
        return c.most_common(1)[0][0]

    # Initialised RAM layout from the startup code's literal pool
    sidata, sdata, edata = resolved('_sidata'), resolved('_sdata'), resolved('_edata')
    siccm, sccm, eccm = resolved('_siccm'), resolved('_sccm'), resolved('_eccm')
    if None not in (sidata, sdata, edata):
        ship.data_map.append((sdata, edata, sidata))
    if None not in (siccm, sccm, eccm):
        ship.data_map.append((sccm, eccm, siccm))

    # The ITCM image directly follows the DTCM initialisers (linker script:
    # _siitcm = _siccm + SIZEOF(.dtcm)); checked below against call sites into ITCM.
    lin_base = siccm + (eccm - sccm) if None not in (siccm, sccm, eccm) else None
    ship.itcm_lma = lin_base

    def ship_runtime_of(lma):
        if ship.itcm_lma is not None and lma >= ship.itcm_lma:
            return lma - ship.itcm_lma
        return lma

    # Pass 2: everything incl. calls
    votes.clear()
    ambiguous.clear()
    evidence.clear()
    sources.clear()
    for name, (vma, size, lma) in matched.items():
        rt = ship_runtime_of(lma)
        vote(name, rt, 'bytes', 'match')
        process_function(name, vma, size, lma, rt, True)

    # vector table (same position in both images)
    for off_vma, typ, rsym in relocs_in(APP_BASE, 0x200):
        if typ == R_ARM_ABS32:
            vote_value(elf.word(off_vma), ship.word_lma(off_vma), f'vector+{off_vma - APP_BASE:#x}', 'vector', rsym)

    def merge_weak():
        # an ambiguous reference explained by a candidate's unambiguous address is
        # dropped; otherwise the candidates without other references get the vote
        for cands, why in ambiguous:
            if any(k in votes and votes[k].most_common(1)[0][0] == addr for k, addr in cands):
                continue
            for k, addr in cands:
                if k not in votes:
                    votes[k][addr] += 1
                    evidence[k].append(why + ' (ambiguous boundary reference)')
                    sources[k].add('ambiguous')
        ambiguous.clear()

    merge_weak()
    # data objects: relocations inside recovered objects (iterate to a fixpoint)
    done = set()
    for _ in range(5):
        new = False
        for a, sz, k in objects:
            if k in done or k not in votes or sz == 0:
                continue
            c = votes[k]
            if len(c) != 1:
                continue
            rt = next(iter(c))
            slma = ship.lma_of(rt)
            if slma is None:
                continue
            done.add(k)
            for off_vma, typ, rsym in relocs_in(a, sz):
                if typ == R_ARM_ABS32:
                    vote_value(elf.word(off_vma), ship.word_lma(slma + (off_vma - a)), f'{k}+{off_vma - a:#x}', 'data', rsym)
                    new = True
        if not new:
            break

    merge_weak()

    # Functions that are not byte-identical but were located from call sites (e.g. main):
    # align the sequence of BL targets in our build with the BLs found by a linear
    # decode of the shipped function. Equal-length runs of calls between two calls whose
    # targets agree pair up the unknown targets.
    import difflib

    def ship_bls(start, length):
        out = []
        a = start
        lma0 = ship.lma_of(start)
        if lma0 is None:
            return out
        while a < start + length:
            h1 = ship.half_lma(lma0 + (a - start))
            if h1 is None:
                break
            if (h1 >> 11) in (0x1D, 0x1E, 0x1F):
                h2 = ship.half_lma(lma0 + (a - start) + 2)
                if (h1 & 0xF800) == 0xF000 and (h2 & 0xD000) == 0xD000:
                    out.append(decode_bl(h1, h2, a))
                a += 4
            else:
                a += 2
        return out

    for _ in range(3):
        for k, (a_elf, sz, t) in list(sym_info.items()):
            if t != 'STT_FUNC' or k in matched or k not in votes or len(votes[k]) != 1:
                continue
            start = next(iter(votes[k]))
            e_calls = []
            for off_vma, typ, rsym in relocs_in(a_elf, sz):
                if typ == R_ARM_THM_CALL and (elf.half(off_vma + 2) & 0xD000) == 0xD000:
                    te = decode_bl(elf.half(off_vma), elf.half(off_vma + 2), off_vma)
                    if te is None:
                        continue
                    lst = funcs_at.get(te, [])
                    e_calls.append(lst[0] if lst else hex(te))
            s_calls = ship_bls(start, int(sz * 1.25) + 64)
            known = {}
            for kk, c in votes.items():
                if len(c) == 1 and sym_info.get(kk, (0, 0, ''))[2] == 'STT_FUNC':
                    known.setdefault(next(iter(c)), kk)
            s_names = [known.get(x, hex(x)) for x in s_calls]
            sm = difflib.SequenceMatcher(None, e_calls, s_names, autojunk=False)
            for op, i1, i2, j1, j2 in sm.get_opcodes():
                if op == 'replace' and i2 - i1 == j2 - j1:
                    for ii, jj in zip(range(i1, i2), range(j1, j2)):
                        en, sn = e_calls[ii], s_names[jj]
                        if not en.startswith('0x') and sn.startswith('0x') and en not in votes:
                            vote(en, int(sn, 16), f'{k} call sequence', 'callseq')

    # ---- result ------------------------------------------------------------------
    functions, data, conflicts = {}, {}, {}
    for k, c in sorted(votes.items()):
        a_elf, sz, t = sym_info.get(k, (None, None, None))
        if a_elf is None:
            continue
        addr, n = c.most_common(1)[0]
        if len(c) > 1:
            conflicts[k] = {hex(v): cnt for v, cnt in c.items()}
            conflicts[k]['evidence'] = evidence[k]
        entry = {'addr': hex(addr)}
        if t == 'STT_FUNC':
            if k in matched:
                entry['size'] = sz
                entry['match'] = 'bytes'
            else:
                entry['size'] = None
                entry['elf_size'] = sz
                entry['match'] = 'refs'
            if addr < ITCM_END:
                entry['lma'] = hex(ship.itcm_lma + addr)
            entry['refs'] = sum(c.values()) - (1 if k in matched else 0)
            functions[k] = entry
        else:
            entry['size'] = sz
            entry['refs'] = sum(c.values())
            data[k] = entry
    # sanity check: recovered objects must not overlap (identical addresses = aliases)
    items = sorted((int(v['addr'], 16), v['size'], k) for k, v in list(functions.items()) + list(data.items())
                   if v['size'])
    overlaps = [[k, l] for (a, sz, k), (b, t, l) in zip(items, items[1:]) if a + sz > b and a != b]
    aliases = [[k, l] for (a, sz, k), (b, t, l) in zip(items, items[1:]) if a == b]
    unresolved_funcs = sorted(k for k, r in matches.items() if k not in functions)
    out = {
        'comment': 'Addresses of symbols in the shipped LXR-02 v1.71 firmware (payload of '
                   'LXRV2_update_v1.71.img loaded at 0x08008000), recovered by tools/symbol_map.py '
                   'from the GCC 11.3 reference build. Runtime addresses; ITCM functions have their '
                   'load address as lma. size = null: function not byte-identical, address from call sites.',
        'image_size': len(ship.data),
        'itcm_lma': hex(ship.itcm_lma) if ship.itcm_lma is not None else None,
        'stats': {
            'functions_total_elf': len(matches),
            'functions_byte_matched': len(matched),
            'functions_recovered': len(functions),
            'functions_recovered_by_refs_only': sum(1 for v in functions.values() if v['match'] == 'refs'),
            'functions_not_recovered': len(unresolved_funcs),
            'data_symbols_recovered': len(data),
            'conflicts': len(conflicts),
            'overlaps': len(overlaps),
        },
        'overlaps': overlaps,
        'aliases': aliases,
        'functions': functions,
        'data': data,
        'conflicts': conflicts,
        'not_recovered_functions': unresolved_funcs,
    }
    json.dump(out, open(out_path, 'w'), indent=1)
    print(json.dumps(out['stats'], indent=1))
    print('itcm', out['itcm_lma'])
    for k, v in list(conflicts.items())[:40]:
        print('CONFLICT', k, v)


if __name__ == '__main__':
    main()
