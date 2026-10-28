"""Dump DWARF info of the LXR reference ELF into a JSON file for code generation."""
import sys, json, os
from elftools.elf.elffile import ELFFile

ELF = sys.argv[2] if len(sys.argv) > 2 else '/home/user/lxr-ref/lxr_v1.7_gcc11.3.elf'
f = open(ELF, 'rb'); e = ELFFile(f)
d = e.get_dwarf_info(relocate_dwarf_sections=(len(sys.argv) > 2))

def name(die):
    a = die.attributes.get('DW_AT_name')
    return a.value.decode() if a else None

def ref(die, attr):
    a = die.attributes.get(attr)
    if not a: return None
    return die.get_DIE_from_attribute(attr)

types = {}  # offset -> description

def tdesc(die):
    """Return a type reference key (offset) and register description."""
    if die is None:
        return None
    off = die.offset
    if off in types:
        return off
    types[off] = None  # placeholder for recursion
    tag = die.tag
    t = {'tag': tag, 'name': name(die)}
    if 'DW_AT_byte_size' in die.attributes:
        t['size'] = die.attributes['DW_AT_byte_size'].value
    if tag == 'DW_TAG_base_type':
        t['encoding'] = die.attributes['DW_AT_encoding'].value
    elif tag in ('DW_TAG_typedef', 'DW_TAG_pointer_type', 'DW_TAG_const_type', 'DW_TAG_volatile_type', 'DW_TAG_restrict_type'):
        t['type'] = tdesc(ref(die, 'DW_AT_type'))
    elif tag in ('DW_TAG_structure_type', 'DW_TAG_union_type'):
        t['decl'] = 'DW_AT_declaration' in die.attributes
        mem = []
        for c in die.iter_children():
            if c.tag == 'DW_TAG_member':
                m = {'name': name(c), 'type': tdesc(ref(c, 'DW_AT_type'))}
                if 'DW_AT_decl_line' in c.attributes:
                    m['line'] = c.attributes['DW_AT_decl_line'].value
                if 'DW_AT_data_member_location' in c.attributes:
                    m['offset'] = c.attributes['DW_AT_data_member_location'].value
                if 'DW_AT_bit_size' in c.attributes:
                    m['bit_size'] = c.attributes['DW_AT_bit_size'].value
                    m['bit_offset'] = c.attributes.get('DW_AT_data_bit_offset', c.attributes.get('DW_AT_bit_offset')).value
                mem.append(m)
        t['members'] = mem
        t['file'], t['line'] = declpos(die)
    elif tag == 'DW_TAG_enumeration_type':
        t['type'] = tdesc(ref(die, 'DW_AT_type'))
        t['enumerators'] = [(name(c), c.attributes['DW_AT_const_value'].value) for c in die.iter_children() if c.tag == 'DW_TAG_enumerator']
        t['file'], t['line'] = declpos(die)
    elif tag == 'DW_TAG_array_type':
        t['type'] = tdesc(ref(die, 'DW_AT_type'))
        dims = []
        for c in die.iter_children():
            if c.tag == 'DW_TAG_subrange_type':
                if 'DW_AT_upper_bound' in c.attributes:
                    ub = c.attributes['DW_AT_upper_bound']
                    dims.append(ub.value + 1 if ub.form != 'DW_FORM_exprloc' else None)
                elif 'DW_AT_count' in c.attributes:
                    dims.append(c.attributes['DW_AT_count'].value)
                else:
                    dims.append(0)
        t['dims'] = dims
    elif tag == 'DW_TAG_subroutine_type':
        t['type'] = tdesc(ref(die, 'DW_AT_type'))
        t['params'] = [tdesc(ref(c, 'DW_AT_type')) for c in die.iter_children() if c.tag == 'DW_TAG_formal_parameter']
    if tag == 'DW_TAG_typedef':
        t['file'], t['line'] = declpos(die)
    types[off] = t
    return off

cu_files = {}
cur_cu = None

def declpos(die):
    fa = die.attributes.get('DW_AT_decl_file')
    la = die.attributes.get('DW_AT_decl_line')
    fn = None
    if fa is not None:
        fn = cu_files[cur_cu.cu_offset][fa.value]
    return fn, (la.value if la else None)

def filenames(cu):
    lp = d.line_program_for_CU(cu)
    hdr = lp.header
    dirs = [x.decode() if isinstance(x, bytes) else x for x in hdr['include_directory']]
    out = {}
    for i, fe in enumerate(hdr['file_entry']):
        n = fe.name.decode() if isinstance(fe.name, bytes) else fe.name
        v5 = hdr['version'] >= 5
        di = fe.dir_index if v5 else fe.dir_index - 1
        dn = dirs[di] if 0 <= di < len(dirs) else ''
        p = os.path.normpath(os.path.join(dn, n))
        if '/lxr-desktop-app/' in p: p = p.split('/lxr-desktop-app/')[1]
        out[i if v5 else i + 1] = p
    return out

def loc_addr(die):
    a = die.attributes.get('DW_AT_location')
    if a is None or a.form != 'DW_FORM_exprloc':
        return None
    v = a.value
    if v and v[0] == 0x03 and len(v) == 5:
        return int.from_bytes(bytes(v[1:5]), 'little')
    return None

out = {'cus': []}
for cu in d.iter_CUs():
    cur_cu = cu
    top = cu.get_top_DIE()
    cname = name(top)
    if 'Core/' not in cname:
        continue
    cname = cname[cname.index('Core/'):]
    cu_files[cu.cu_offset] = filenames(cu)
    funcs = []; vars_ = []; localstatics = []
    inl = {}
    def walk_locals(die, fname):
        for c in die.iter_children():
            if c.tag == 'DW_TAG_inlined_subroutine':
                o = ref(c, 'DW_AT_abstract_origin')
                while o is not None and 'DW_AT_abstract_origin' in o.attributes:
                    o = ref(o, 'DW_AT_abstract_origin')
                if o is not None:
                    inl.setdefault(fname, set()).add(name(o))
                walk_locals(c, fname)
            if c.tag == 'DW_TAG_variable':
                ad = loc_addr(c)
                if ad is not None:
                    localstatics.append({'func': fname, 'name': name(c), 'type': tdesc(ref(c, 'DW_AT_type')), 'addr': ad})
            if c.tag == 'DW_TAG_lexical_block':
                walk_locals(c, fname)
    for c in top.iter_children():
        if c.tag == 'DW_TAG_subprogram':
            if 'DW_AT_abstract_origin' in c.attributes or 'DW_AT_specification' in c.attributes:
                o = ref(c, 'DW_AT_abstract_origin') or ref(c, 'DW_AT_specification')
                # concrete out-of-line instance of an abstract function
                for fn in funcs:
                    if fn['off'] == o.offset:
                        fn['concrete'] = True
                        if 'DW_AT_low_pc' in c.attributes:
                            fn['low_pc'] = c.attributes['DW_AT_low_pc'].value
                walk_locals(c, name(o))
                continue
            fn = {'off': c.offset, 'name': name(c), 'external': 'DW_AT_external' in c.attributes,
                  'declaration': 'DW_AT_declaration' in c.attributes,
                  'inline': c.attributes['DW_AT_inline'].value if 'DW_AT_inline' in c.attributes else None,
                  'ret': tdesc(ref(c, 'DW_AT_type')),
                  'prototyped': 'DW_AT_prototyped' in c.attributes,
                  'low_pc': c.attributes['DW_AT_low_pc'].value if 'DW_AT_low_pc' in c.attributes else None,
                  'params': [], 'variadic': False}
            fn['file'], fn['line'] = declpos(c)
            for p in c.iter_children():
                if p.tag == 'DW_TAG_formal_parameter':
                    fn['params'].append({'name': name(p), 'type': tdesc(ref(p, 'DW_AT_type'))})
                elif p.tag == 'DW_TAG_unspecified_parameters':
                    fn['variadic'] = True
            if not fn['declaration']:
                walk_locals(c, fn['name'])
            funcs.append(fn)
        elif c.tag == 'DW_TAG_variable':
            v = {'name': name(c), 'external': 'DW_AT_external' in c.attributes,
                 'declaration': 'DW_AT_declaration' in c.attributes,
                 'type': tdesc(ref(c, 'DW_AT_type')), 'addr': loc_addr(c)}
            if 'DW_AT_specification' in c.attributes:
                s = ref(c, 'DW_AT_specification')
                v['name'] = name(s); v['external'] = 'DW_AT_external' in s.attributes
                v['type'] = tdesc(ref(s, 'DW_AT_type'))
                v['file'], v['line'] = declpos(s)
            else:
                v['file'], v['line'] = declpos(c)
            vars_.append(v)
        else:
            if c.tag in ('DW_TAG_structure_type', 'DW_TAG_union_type', 'DW_TAG_enumeration_type', 'DW_TAG_typedef'):
                tdesc(c)
    out['cus'].append({'name': cname, 'funcs': funcs, 'vars': vars_, 'localstatics': localstatics, 'inlined': {k: sorted(v) for k, v in inl.items()}})

out['types'] = {str(k): v for k, v in types.items()}
json.dump(out, open(sys.argv[1], 'w'), indent=0)
print('ok', len(out['cus']), len(types))
