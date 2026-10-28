"""Second half of the generator: globals, functions, consts, modules, docs."""
import sys, os; sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from gen import *  # noqa

# ============================================================== ELF memory image
SECTIONS = []
for sec in ELF.iter_sections():
    if sec['sh_flags'] & 2 and sec['sh_size'] > 0:  # SHF_ALLOC
        SECTIONS.append((sec['sh_addr'], sec['sh_addr'] + sec['sh_size'], sec.name, sec['sh_type']))


def init_bytes(addr, size):
    for a, b, n, tp in SECTIONS:
        if a <= addr < b:
            if tp == 'SHT_NOBITS':
                return bytes(size), n
            sec = ELF.get_section_by_name(n)
            d = sec.data()[addr - a: addr - a + size]
            assert len(d) == size, (hex(addr), size, n)
            return d, n
    raise Exception('no section for %x' % addr)


def fbits32(b):
    v = struct.unpack('<I', b)[0]
    f = struct.unpack('<f', b)[0]
    return 'f32::from_bits(0x%08x)' % v if v else '0.0'


def fbits64(b):
    v = struct.unpack('<Q', b)[0]
    return 'f64::from_bits(0x%016x)' % v if v else '0.0'


INT_FMT = {'u8': '<B', 'i8': '<b', 'u16': '<H', 'i16': '<h', 'u32': '<I', 'i32': '<i', 'u64': '<Q', 'i64': '<q'}


def decode(T, k, b):
    """Rust expression for the value of DWARF type k stored in bytes b."""
    rt = rty(T, k, need=False)
    if not any(b):
        return zero_expr(rt)
    t = T[str(k)]
    tg = t['tag']
    if tg in ('DW_TAG_const_type', 'DW_TAG_volatile_type', 'DW_TAG_typedef'):
        u = T[str(t['type'])]
        if u['tag'] == 'DW_TAG_enumeration_type' or (tg == 'DW_TAG_typedef' and u['tag'] not in ('DW_TAG_structure_type', 'DW_TAG_union_type')):
            pass
        return decode(T, t['type'], b)
    if tg == 'DW_TAG_base_type':
        p = PRIM[t['name']]
        if p == 'f32':
            return fbits32(b[:4])
        if p == 'f64':
            return fbits64(b[:8])
        if p == 'bool':
            return 'true' if b[0] else 'false'
        return str(struct.unpack(INT_FMT[p], b[:PRIM_SIZE[p]])[0])
    if tg == 'DW_TAG_enumeration_type':
        p = enum_prim(T, t)
        return str(struct.unpack(INT_FMT[p], b[:PRIM_SIZE[p]])[0])
    if tg == 'DW_TAG_pointer_type':
        return '0x%08x' % struct.unpack('<I', b[:4])[0]
    if tg == 'DW_TAG_array_type':
        et = t['type']
        es = type_size(T, et)
        dims = t['dims']
        def rec(off, ds):
            if len(ds) == 1:
                items = [decode(T, et, b[off + i * es: off + (i + 1) * es]) for i in range(ds[0])]
            else:
                stride = es
                for d in ds[1:]:
                    stride *= d
                items = [rec(off + i * stride, ds[1:]) for i in range(ds[0])]
            if len(set(items)) == 1 and len(items) > 1:
                return '[%s; %d]' % (items[0], len(items))
            return '[' + ', '.join(items) + ']'
        return rec(0, dims)
    if tg in ('DW_TAG_structure_type', 'DW_TAG_union_type'):
        n = struct_rname(T, k)
        if n in HAND_DECODE:
            return HAND_DECODE[n](b)
        parts = []
        for m in t['members']:
            ms = type_size(T, m['type'])
            parts.append('%s: %s' % (rid(m['name']), decode(T, m['type'], b[m['offset']:m['offset'] + ms])))
        return '%s { %s }' % (n, ', '.join(parts))
    raise Exception(tg)


# element types of the tables (e.g. ModTarg) are C types of the crate as well
for g in sorted(TABLES):
    rty(REF_TYPES, GLOBALS[g]['type'])

# ============================================================== globals -> Lxr fields
EXCLUDED_GLOBALS = {'htim2': 'HAL timer handle (TIM_HandleTypeDef), hardware only',
                    'hrng': 'HAL RNG handle (RNG_HandleTypeDef); replaced by `rng_state`, see rng.rs'}
GLOBAL_TYPE_OVERRIDE = {
    'morphParameters': ('Option<usize>', 'C: `uint8_t* morphParameters`. Representation: `Some(i)` = `mainParameters[i]` '
                        '(the only values the firmware assigns), `None` = NULL (power-on value; the firmware never assigns it '
                        'without an SD card, so `preset_morph_tick` then reads through a NULL pointer, i.e. ITCM code bytes '
                        'at 0x00000000 + index).'),
    'seq_activeSong': ('Option<usize>', 'C: `uint8_t *seq_activeSong`. Representation: `Some(i)` = `seq_songData[i]` '
                       '(the only values the firmware assigns), `None` = NULL (power-on value, set by `seq_init`).'),
}
BOX_LIMIT = 4096


def gl_field_name(g):
    if '::' in g[0]:
        fn, v = g[0].split('::')
        return '%s_%s' % (fn, v)
    return g[0]


LXR_FIELDS = []  # dicts
for g in sorted(set(used_globals) - TABLES, key=lambda g: GLOBALS[g]['addr']):
    if g[0] in EXCLUDED_GLOBALS:
        continue
    gv = GLOBALS[g]
    T = REF_TYPES
    k = gv['type']
    size = type_size(T, k)
    fname = gl_field_name(g)
    ov = GLOBAL_TYPE_OVERRIDE.get(g[0])
    b, secname = init_bytes(gv['addr'], size)
    if ov:
        rt = ov[0]
        assert not any(b), g
        init = 'None'
    else:
        rt = rty(T, k)
        init = decode(T, k, b)
    boxed = size >= BOX_LIMIT
    decl_file = gv.get('file') or g[1]
    LXR_FIELDS.append(dict(name=fname, key=g, rt=rt, init=init, boxed=boxed, size=size, addr=gv['addr'], sec=secname,
                           cdecl=src_line(decl_file, gv.get('line')) if not gv['local_of'] else '',
                           decl_file=decl_file, decl_line=gv.get('line'), defcu=g[1], local_of=gv['local_of'],
                           note=ov[1] if ov else None, users=sorted(set(f[0] for f in used_globals[g]))))

names = [f['name'] for f in LXR_FIELDS]
assert len(names) == len(set(names)), 'duplicate field names'

# local statics: find the declaration text
for f in LXR_FIELDS:
    if f['local_of']:
        fk = [k for k in FUNCS if k[0] == f['local_of'] and k[1] == f['defcu']]
        if fk:
            fn = FUNCS[fk[0]]
            L = src_lines(fn['file'])
            var = f['key'][0].split('::')[1]
            for i in range((fn['line'] or 1) - 1, (fn['end_line'] or fn['line']) or 0):
                if re.search(r'\bstatic\b.*\b%s\b' % re.escape(var), L[i]):
                    f['cdecl'] = L[i].strip(); f['decl_file'] = fn['file']; f['decl_line'] = i + 1
                    break

# ============================================================== functions
def is_const_pointee(T, k):
    while k is not None:
        t = T[str(k)]
        if t['tag'] == 'DW_TAG_const_type':
            return True
        if t['tag'] in ('DW_TAG_volatile_type', 'DW_TAG_typedef'):
            k = t.get('type'); continue
        return False
    return False


def buf_usage(body, n):
    """'slice' if the pointer parameter is indexed / moved / passed on, 'ref' if only dereferenced"""
    if re.search(r'(?<![\w.>])%s\s*\[' % re.escape(n), body):
        return 'slice'
    if re.search(r'(?<![\w.>])%s\s*(\+\+|--|\+=|-=|\+|-(?!>))' % re.escape(n), body) or re.search(r'(\+\+|--)\s*%s\b' % re.escape(n), body):
        return 'slice'
    if re.search(r'\*\s*%s\b' % re.escape(n), body) and not re.search(r'[(,]\s*%s\s*[,)]' % re.escape(n), body):
        return 'ref'
    return 'slice'


def map_param(key, p, places):
    T = otypes(key)
    f = FUNCS[key]
    n = p['name'] or 'arg'
    k = p['type']
    t = T[str(k)]
    note = None
    if t['tag'] == 'DW_TAG_pointer_type':
        tgt = t.get('type')
        const = is_const_pointee(T, tgt)
        u = strip_cv(T, tgt)
        if u is None:
            return rid(n), 'CPtr', 'void pointer, see memmap::CPtr'
        ut = T[str(u)]
        if ut['tag'] in ('DW_TAG_structure_type', 'DW_TAG_union_type'):
            rn = rty(T, tgt)
            if n in places:
                return rid(n), 'impl Fn(&mut Lxr) -> &mut %s + Copy' % rn, 'place'
            return rid(n), ('&' if const else '&mut ') + rn, None
        if ut['tag'] == 'DW_TAG_array_type':
            return rid(n), ('&' if const else '&mut ') + '[%s]' % rty(T, tgt), None
        if ut['tag'] in ('DW_TAG_pointer_type', 'DW_TAG_subroutine_type'):
            return rid(n), 'CPtr', 'pointer to pointer / function pointer'
        prim = rty(T, tgt)
        use = buf_usage(f['body'], n) if f['body'] else 'slice'
        if use == 'ref':
            return rid(n), ('&' if const else '&mut ') + prim, None
        return rid(n), ('&[%s]' if const else '&mut [%s]') % prim, None
    return rid(n), rty(T, k), None


def map_ret(key):
    T = otypes(key)
    r = FUNCS[key]['ret']
    if r is None:
        return None
    t = T[str(r)]
    if t['tag'] == 'DW_TAG_pointer_type':
        return 'CPtr'
    return rty(T, r)


FUNC_ITEMS = []  # dicts
SKIP_FUNCS = {('GetRngValue', 'Core/Src/Soundengine/DSPAudio/random.c')}  # implemented in rng.rs
for key in sorted(set(in_scope) | set(external), key=lambda k: (k[1], FUNCS[k]['line'] or 0)):
    f = FUNCS[key]
    mod = mod_of(key[1])
    if mod is None:
        mod = 'cmsis'
    places = place_params(key) if key in in_scope else []
    params = [map_param(key, p, places) for p in f['params']]
    is_ext = key in external
    cdecl = c_decl_text(key[1], f['line']) if f['line'] else ''
    if not cdecl or '(' not in cdecl:
        cdecl = '%s %s(%s)' % (ctype(otypes(key), f['ret']), key[0], ', '.join('%s %s' % (ctype(otypes(key), p['type']), p['name']) for p in f['params']))
    FUNC_ITEMS.append(dict(key=key, name=key[0], mod=mod, method=key in needs_self, params=params, ret=map_ret(key),
                           cdecl=cdecl, file=key[1], line=f['line'], link=link_status(key), external=is_ext,
                           group='P3' if is_ext else group_of(mod), static=not f['external'],
                           skip=key in SKIP_FUNCS, calls=sorted(CALLS.get(key, ())),
                           globals=sorted(gl_field_name(g) for g in direct_globals.get(key, ()) if g not in TABLES and g[0] not in EXCLUDED_GLOBALS),
                           tables=sorted(g[0] for g in direct_globals.get(key, ()) if g in TABLES)))

# ============================================================== enums
ALL_ENUMS = OrderedDict()
for T in TDB.tables:
    for k, t in T.items():
        if t and t['tag'] == 'DW_TAG_enumeration_type' and t.get('file'):
            ident = (t['file'], t.get('line'), t['name'])
            if ident not in ALL_ENUMS:
                ALL_ENUMS[ident] = (T, k)
# typedef aliases of enums
ENUM_TYPEDEFS = defaultdict(set)
for T in TDB.tables:
    for k, t in T.items():
        if t and t['tag'] == 'DW_TAG_typedef' and t.get('type') is not None:
            u = T[str(t['type'])]
            if u and u['tag'] == 'DW_TAG_enumeration_type' and u.get('file'):
                ENUM_TYPEDEFS[(u['file'], u.get('line'), u['name'])].add((t['name'], t.get('file')))

# identifiers used by in-scope code (original source)
USED_IDENTS = set()
for key in in_scope:
    f = FUNCS[key]
    USED_IDENTS |= orig_idents(key[1], f['line'], f['end_line'])
for fld in LXR_FIELDS:
    USED_IDENTS |= set(IDENT_RE.findall(fld['cdecl']))


def file_in_scope(fl):
    return scope_hdr(fl) or in_full(fl) or fl in PARTIAL_FILES


EMIT_ENUMS = OrderedDict()
for ident, (T, k) in ALL_ENUMS.items():
    t = T[str(k)]
    fl = os.path.normpath(ident[0])
    if 'cs4344' in fl:
        continue
    used = any(e[0] in USED_IDENTS for e in t['enumerators'])
    if (file_in_scope(fl) and mod_of(fl) != 'external') or used or ident in needed_enums:
        EMIT_ENUMS[ident] = (T, k)

# ============================================================== macros
def macro_selected():
    sel = set()
    for n, v in MACROS.items():
        fl = os.path.normpath(v[0])
        if 'cs4344' in fl or fl.endswith('stm32f7xx_hal_conf.h') or fl.endswith('main.h'):
            if n not in USED_IDENTS:
                continue
        if n.endswith('_H_') or n.endswith('_H') or v[3] == '' or n in ('true', 'false'):
            continue
        if (file_in_scope(fl) and mod_of(fl) != 'external') or n in USED_IDENTS:
            sel.add(n)
    # closure over macro bodies
    changed = True
    while changed:
        changed = False
        for n in list(sel):
            for i in IDENT_RE.findall(MACROS[n][3]):
                if i in MACROS and i not in sel and not i.endswith('_H_') and MACROS[i][3] != '':
                    sel.add(i); changed = True
    return sel


SEL_MACROS = macro_selected()
CONST_ITEMS = defaultdict(list)  # mod -> list of (name, rust line(s))
CONST_SEEN = {}  # name -> (mod, type, value)


def const_type_value(n):
    r = MVALS['ok'].get(n)
    if not r or not r['type']:
        return None
    t = r['type']; b = bytes.fromhex(r['bytes'])
    if t['tag'] == 'DW_TAG_base_type':
        p = PRIM.get(t['name'])
        if p is None:
            return None
        if p == 'f32':
            v = struct.unpack('<f', b)[0]
            return 'f32', 'f32::from_bits(0x%08x)' % struct.unpack('<I', b)[0], repr(v)
        if p == 'f64':
            v = struct.unpack('<d', b)[0]
            return 'f64', 'f64::from_bits(0x%016x)' % struct.unpack('<Q', b)[0], repr(v)
        if p == 'bool':
            return 'bool', 'true' if b[0] else 'false', None
        v = struct.unpack(INT_FMT[p], b[:PRIM_SIZE[p]])[0]
        return p, str(v) if v >= 0 else '-%d' % -v, None
    if t['tag'] == 'DW_TAG_enumeration_type':
        p = {1: 'u8', 2: 'u16', 4: 'u32'}[t['size']]
        if t.get('enc') in (5,):
            p = p.replace('u', 'i')
        v = struct.unpack(INT_FMT[p], b[:PRIM_SIZE[p]])[0]
        return p, str(v), None
    if t['tag'] == 'DW_TAG_array_type' and r['size']:
        # string literal
        return '&[u8; %d]' % r['size'], 'b"%s"' % ''.join('\\x%02x' % c for c in b), bytes(b).split(b'\0')[0].decode('latin1')
    return None


FN_MACROS = defaultdict(list)
for n in sorted(SEL_MACROS, key=lambda n: (MACROS[n][0], MACROS[n][1])):
    fl, line, params, body, tu = MACROS[n]
    mod = mod_of(os.path.normpath(fl)) or 'external'
    src = '#define %s%s %s' % (n, params or '', body)
    if params is not None:
        FN_MACROS[mod].append((n, fl, line, src))
        continue
    tv = const_type_value(n)
    if tv is None:
        FN_MACROS[mod].append((n, fl, line, src + '    (not a constant expression; not translated)'))
        continue
    ty, val, comment = tv
    if n in CONST_SEEN:
        if CONST_SEEN[n][1:] != (ty, val):
            REPORT['const_conflict'].append((n, CONST_SEEN[n], (mod, ty, val)))
        continue
    CONST_SEEN[n] = (mod, ty, val)
    doc = '/// `%s` (%s:%d)' % (clean_doc(src.strip()), os.path.normpath(fl), line)
    if comment and ty.startswith('f'):
        doc += ' = %s' % comment
    CONST_ITEMS[mod].append((n, '%s\npub const %s: %s = %s;' % (doc, rid(n), ty, val)))

ENUM_ITEMS = defaultdict(list)
for ident, (T, k) in EMIT_ENUMS.items():
    t = T[str(k)]
    fl = os.path.normpath(ident[0])
    mod = mod_of(fl) or 'external'
    prim = enum_prim(T, t)
    lines = []
    tname = None
    aliases = sorted(ENUM_TYPEDEFS.get(ident, ()))
    if t['name']:
        tname = t['name']
    elif aliases:
        tname = aliases[0][0]
    hdr = '/// C: `enum %s` (%s:%s), %d bytes, underlying type `%s`.' % (t['name'] or '{...}', fl, ident[1], t.get('size', 4), prim)
    if tname:
        if tname in ENUM_ALIASES:
            continue
        ENUM_ALIASES[tname] = prim
        lines.append(hdr + '\n/// Representation: integer type alias plus one `pub const` per enumerator.\npub type %s = %s;' % (tname, prim))
        for a, af in aliases:
            if a != tname and a not in ENUM_ALIASES:
                ENUM_ALIASES[a] = prim
                lines.append('/// C: `typedef enum %s %s;`\npub type %s = %s;' % (t['name'] or '{...}', a, a, tname))
    else:
        lines.append('// ' + hdr[4:] + ' (anonymous: enumerators only)')
    cty = tname or prim
    for en, ev in t['enumerators']:
        if en in CONST_SEEN:
            if CONST_SEEN[en][2] != str(ev):
                REPORT['const_conflict'].append((en, CONST_SEEN[en], (mod, cty, ev)))
            continue
        CONST_SEEN[en] = (mod, cty, str(ev))
        lines.append('pub const %s: %s = %d;' % (rid(en), cty, ev))
    ENUM_ITEMS[mod].append('\n'.join(lines))

# ============================================================== structs
# make sure all struct types defined in scope files are emitted
for T in TDB.tables:
    for k, t in list(T.items()):
        if t and t['tag'] in ('DW_TAG_structure_type', 'DW_TAG_union_type') and t.get('file') and not t.get('decl'):
            fl = os.path.normpath(t['file'])
            if file_in_scope(fl) and mod_of(fl) not in ('external', None):
                if t['name'] == 'pair' or (t['name'] is None and (fl, t.get('line')) not in ANON_NAMES and struct_rname(T, k).startswith('anon_')):
                    # anonymous helper structs of unions handled by hand / unused
                    if t['name'] == 'pair':
                        rty(T, k)
                    continue
                rty(T, k)

STRUCT_ITEMS = defaultdict(list)
STRUCT_MOD = {}
emitted = set()
i = 0
order = list(needed_structs.items())
while i < len(order):
    n, (T, k) = order[i]; i += 1
    if n in emitted or n in HAND:
        continue
    emitted.add(n)
    t = T[str(k)]
    fl = os.path.normpath(t.get('file') or '')
    mod = mod_of(fl) or 'external'
    STRUCT_MOD[n] = mod
    kind = 'union' if t['tag'] == 'DW_TAG_union_type' else 'struct'
    if kind == 'union':
        REPORT['unhandled_union'].append(n)
    tag = t['name']
    out = []
    out.append('/// C: `%s %s` (%s:%s), %d bytes.' % (kind, tag or '{...}', fl, t.get('line'), t.get('size', 0)))
    if n != tag and tag:
        out.append('/// C typedef name `%s`; the struct tag `%s` is a type alias below.' % (n, tag))
    out.append('#[derive(Clone, Copy, Debug, PartialEq)]')
    out.append('pub struct %s {' % n)
    zero = []
    for m in t['members']:
        mt = rty(T, m['type'])
        cl = src_line(fl, m.get('line')) if m.get('line') else ''
        cl = re.sub(r'\s+', ' ', cl)
        doc = '    /// `%s`' % clean_doc(cl) if cl else '    /// (C member `%s`)' % m['name']
        if mt == 'CPtr':
            doc += '\n    ///\n    /// C pointer: stored as the address in the reference firmware memory map (`CPtr`, 0 = NULL), see `memmap`.'
        if 'bit_size' in m:
            REPORT['unhandled_bitfield'].append((n, m['name']))
        out.append(doc)
        out.append('    pub %s: %s,' % (rid(m['name']), mt))
        zero.append('%s: %s' % (rid(m['name']), zero_expr(mt)))
    out.append('}')
    if tag and tag != n:
        out.append('/// C struct tag of `%s`.\npub type %s = %s;' % (n, tag, n))
    out.append('impl %s {\n    /// All-zero value (C static zero initialisation).\n    pub const ZERO: Self = Self { %s };\n}' % (n, ', '.join(zero)))
    out.append('impl Default for %s {\n    fn default() -> Self {\n        Self::ZERO\n    }\n}' % n)
    STRUCT_ITEMS[mod].append('\n'.join(out))
    order = list(needed_structs.items())

for n, (mod, code) in HAND.items():
    STRUCT_ITEMS[mod].insert(0, code.rstrip())
    STRUCT_MOD[n] = mod
