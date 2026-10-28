#!/usr/bin/env python3
"""Generate crates/lxr-core/src/tables/*.rs from the reference ELF.

usage: python3 -I tools/gen_tables.py <lxr_v1.7_gcc11.3.elf> <crates/lxr-core/src/tables>

Every constant data table of the engine is emitted as a Rust `static` with the exact element type, taken
from the ELF bytes (the initial image of `.rodata`, `.data` and `.dtcm`) and the DWARF type of the symbol,
never from the C literals. Floats are emitted as `f32::from_bits` / `f64::from_bits` so that they are
bit exact. Also writes `checksums.rs` (length + FNV-1a 64 of the little-endian bytes of each table) that
the tests in `tables/mod.rs` check against the generated data.

Requires pyelftools (run python with -I).
"""
import os
import struct
import sys

from elftools.elf.elffile import ELFFile

# (symbol, rust module, doc).  The DWARF type of the symbol gives the element type and the dimensions.
TABLES = [
    ('sine_table', 'wavetable', 'Core/Src/Soundengine/DSPAudio/wavetable.c'),
    ('sawTable', 'wavetable', 'Core/Src/Soundengine/DSPAudio/wavetable.c'),
    ('triTable', 'wavetable', 'Core/Src/Soundengine/DSPAudio/wavetable.c'),
    ('recTable', 'wavetable', 'Core/Src/Soundengine/DSPAudio/wavetable.c'),
    ('transientData', 'transient_tables', 'Core/Src/Soundengine/DSPAudio/transientTables.c'),
    ('transientVolumeTable', 'transient_tables', 'Core/Src/Soundengine/DSPAudio/transientTables.c'),
    ('table_1n914', 'distortion', 'Core/Src/Soundengine/DSPAudio/FX/Distortion.c'),
    ('ecc83Table', 'distortion', 'Core/Src/Soundengine/DSPAudio/FX/Distortion.c'),
    ('MidiNoteFrequencies', 'midi_note_numbers', 'Core/Inc/Soundengine/MIDI/MidiNoteNumbers.h (static const, one copy linked)'),
    ('initKit', 'init_kit', 'Core/Src/Preset/initKit.c'),
    ('modTargets', 'cc2_text', 'Core/Src/menu/Cc2Text.c'),
    ('parameterOrder', 'parameters', 'Core/Src/Parameters.c (not const in C, but never written)'),
]
# Rust paths of struct element types
STRUCT_PATHS = {'ModTarg': 'crate::menu::ModTarg'}

PRIM = {'unsigned char': 'u8', 'char': 'u8', 'signed char': 'i8', 'short int': 'i16', 'short unsigned int': 'u16',
        'int': 'i32', 'unsigned int': 'u32', 'long int': 'i32', 'long unsigned int': 'u32', 'float': 'f32', 'double': 'f64'}
FMT = {'u8': 'B', 'i8': 'b', 'i16': 'h', 'u16': 'H', 'i32': 'i', 'u32': 'I', 'f32': 'f', 'f64': 'd'}
SIZE = {'u8': 1, 'i8': 1, 'i16': 2, 'u16': 2, 'i32': 4, 'u32': 4, 'f32': 4, 'f64': 8}


def fnv1a64(data):
    h = 0xcbf29ce484222325
    for b in data:
        h ^= b
        h = (h * 0x100000001b3) & 0xffffffffffffffff
    return h


def main():
    elf_path, out_dir = sys.argv[1], sys.argv[2]
    e = ELFFile(open(elf_path, 'rb'))
    syms = {}
    for s in e.get_section_by_name('.symtab').iter_symbols():
        if s['st_info']['type'] == 'STT_OBJECT':
            syms.setdefault(s.name, (s['st_value'], s['st_size']))

    def mem(addr, size):
        for sec in e.iter_sections():
            a = sec['sh_addr']
            if sec['sh_flags'] & 2 and a <= addr < a + sec['sh_size']:
                assert sec['sh_type'] != 'SHT_NOBITS', 'table in NOBITS section'
                d = sec.data()[addr - a: addr - a + size]
                assert len(d) == size
                return d
        raise KeyError(hex(addr))

    # DWARF: find the type of each table variable (by its location address)
    d = e.get_dwarf_info(relocate_dwarf_sections=False)
    want = {syms[n][0]: n for n, _, _ in TABLES}
    vtypes = {}
    for cu in d.iter_CUs():
        for die in cu.get_top_DIE().iter_children():
            if die.tag != 'DW_TAG_variable':
                continue
            loc = die.attributes.get('DW_AT_location')
            if loc is None or loc.form != 'DW_FORM_exprloc' or len(loc.value) != 5 or loc.value[0] != 3:
                continue
            addr = int.from_bytes(bytes(loc.value[1:5]), 'little')
            if addr in want:
                tdie = die
                if 'DW_AT_type' not in die.attributes and 'DW_AT_specification' in die.attributes:
                    tdie = die.get_DIE_from_attribute('DW_AT_specification')
                vtypes[want[addr]] = tdie.get_DIE_from_attribute('DW_AT_type')

    def strip(t):
        while t.tag in ('DW_TAG_const_type', 'DW_TAG_volatile_type', 'DW_TAG_typedef'):
            if t.tag == 'DW_TAG_typedef' and t.get_DIE_from_attribute('DW_AT_type').tag == 'DW_TAG_structure_type':
                return t
            t = t.get_DIE_from_attribute('DW_AT_type')
        return t

    def describe(name):
        """-> (dims, element) where element is a prim name or ('struct', rustname, [(field, prim, offset)], size)"""
        t = strip(vtypes[name])
        assert t.tag == 'DW_TAG_array_type', name
        dims = []
        for c in t.iter_children():
            if c.tag == 'DW_TAG_subrange_type':
                if 'DW_AT_upper_bound' in c.attributes:
                    dims.append(c.attributes['DW_AT_upper_bound'].value + 1)
                else:
                    dims.append(None)  # incomplete (extern T x[]): size from the symbol
        el = strip(t.get_DIE_from_attribute('DW_AT_type'))
        if el.tag == 'DW_TAG_base_type':
            return dims, PRIM[el.attributes['DW_AT_name'].value.decode()]
        if el.tag == 'DW_TAG_typedef':
            sname = el.attributes['DW_AT_name'].value.decode()
            st = el.get_DIE_from_attribute('DW_AT_type')
            fields = []
            for m in st.iter_children():
                if m.tag == 'DW_TAG_member':
                    ft = strip(m.get_DIE_from_attribute('DW_AT_type'))
                    fields.append((m.attributes['DW_AT_name'].value.decode(), PRIM[ft.attributes['DW_AT_name'].value.decode()],
                                   m.attributes['DW_AT_data_member_location'].value))
            return dims, ('struct', sname, fields, st.attributes['DW_AT_byte_size'].value)
        raise Exception('element type of %s: %s' % (name, el.tag))

    def lit(prim, raw):
        v = struct.unpack('<' + FMT[prim], raw)[0]
        if prim == 'f32':
            return 'f32::from_bits(0x%08x)' % struct.unpack('<I', raw)[0]
        if prim == 'f64':
            return 'f64::from_bits(0x%016x)' % struct.unpack('<Q', raw)[0]
        return str(v)

    modules = {}
    checks = []
    for name, mod, src in TABLES:
        addr, size = syms[name]
        raw = mem(addr, size)
        dims, el = describe(name)
        if isinstance(el, tuple):
            esz = el[3]
            ety = STRUCT_PATHS[el[1]]
        else:
            esz = SIZE[el]
            ety = el
        if dims[0] is None:
            inner = 1
            for x in dims[1:]:
                inner *= x
            dims[0] = size // (esz * inner)
        n = 1
        for x in dims:
            n *= x
        assert n * esz == size, (name, dims, esz, size)
        elems = []
        for i in range(n):
            r = raw[i * esz:(i + 1) * esz]
            if isinstance(el, tuple):
                elems.append('%s { %s }' % (ety, ', '.join('%s: %s' % (f, lit(p, r[o:o + SIZE[p]])) for f, p, o in el[2])))
            else:
                elems.append(lit(el, r))

        def nest(items, ds):
            if len(ds) == 1:
                lines = []
                per = 16 if not isinstance(el, tuple) else 2
                for i in range(0, len(items), per):
                    lines.append(', '.join(items[i:i + per]))
                return '[\n' + ',\n'.join(lines) + ',\n]'
            step = len(items) // ds[0]
            return '[\n' + ',\n'.join(nest(items[i * step:(i + 1) * step], ds[1:]) for i in range(ds[0])) + ',\n]'

        rtype = ety
        for x in reversed(dims):
            rtype = '[%s; %d]' % (rtype, x)
        doc = '/// `%s` from `%s`: %s, %d bytes at 0x%08x in the reference ELF.' % (name, src, rtype, size, addr)
        modules.setdefault(mod, []).append('%s\npub static %s: %s = %s;\n' % (doc, name, rtype, nest(elems, dims)))
        checks.append((name, mod, size, fnv1a64(raw)))

    os.makedirs(out_dir, exist_ok=True)
    header = '//! Generated by `tools/gen_tables.py` from the reference ELF. Do not edit.\n\n'
    for mod, items in modules.items():
        with open(os.path.join(out_dir, mod + '.rs'), 'w') as f:
            f.write(header + '#[allow(unused_imports)]\nuse crate::prelude::*;\n\n' + '\n'.join(items))
    with open(os.path.join(out_dir, 'checksums.rs'), 'w') as f:
        f.write(header + '/// (table, module, size in bytes, FNV-1a 64 of the little-endian bytes in the reference ELF)\n')
        f.write('pub const CHECKSUMS: &[(&str, &str, usize, u64)] = &[\n')
        for name, mod, size, h in checks:
            f.write('    ("%s", "%s", %d, 0x%016x),\n' % (name, mod, size, h))
        f.write('];\n')
    print('wrote', sorted(modules))


if __name__ == '__main__':
    main()
