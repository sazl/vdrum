"""Build a model of the in-scope firmware code (functions, globals, types) for skeleton generation.

Inputs (all produced by scripts in this directory):
  dw.json      DWARF dump of the reference ELF (dwdump.py)
  cg.json      call graph / data refs from the reference ELF relocations (cg.py)
  o0json/*     DWARF dumps of the in-scope C files compiled at -O0 with all static/inline functions kept
  pp/*.i       preprocessed in-scope C files
"""
import json, glob, os, re, sys
from collections import defaultdict

S = os.path.dirname(os.path.abspath(__file__)) + '/'
SRC = '/home/user/lxr-ref/lxr-desktop-app/'

REF = json.load(open(S + 'dw.json'))
CG = json.load(open(S + 'cg.json'))

# ---------------------------------------------------------------- scope
FULL_DIRS = ['Core/Src/Soundengine/DSPAudio/']
FULL_FILES = ['Core/Src/Soundengine/FxBus.c', 'Core/Src/Soundengine/Soundengine.c',
              'Core/Src/Soundengine/MIDI/ParameterArray.c', 'Core/Src/Soundengine/MIDI/MidiVoiceControl.c',
              'Core/Src/Soundengine/MIDI/MidiParser.c', 'Core/Src/Soundengine/MIDI/MidiMessages.c',
              'Core/Src/Soundengine/Sequencer/EuklidGenerator.c', 'Core/Src/Soundengine/Sequencer/clockSync.c',
              'Core/Src/Preset/initKit.c']
# files of which only the functions reachable from the roots are in scope
PARTIAL_FILES = ['Core/Src/Soundengine/Sequencer/sequencer.c', 'Core/Src/Preset/presetManager.c',
                 'Core/Src/Soundengine/MIDI/CortexParser.c', 'Core/Src/Soundengine/MIDI/Uart.c',
                 'Core/Src/Soundengine/FIFO.c', 'Core/Src/frontPanelParser.c', 'Core/Src/menu/Cc2Text.c']
# headers whose whole content (types, macros, inline functions) is in scope
SCOPE_HDR_DIRS = ['Core/Inc/Soundengine/', 'Core/Inc/Preset/']
SCOPE_HDRS = ['Core/Inc/Parameters.h']
ROOTS = ['soundengine_init', 'seq_tick', 'soundengine_calcNextSampleBlock', 'soundengine_process',
         'midiParser_parseUartData', 'preset_init', 'preset_loadDrumsetRam', 'preset_sendDrumsetParameters',
         'preset_sendDrumsetParametersImmediate', 'preset_morph_tick', 'uart_processMidi']
# functions that are never followed (hardware, UI); they become stubs in external.rs when called
STOP_PREFIX = ['menu_', 'led_', 'lcd_', 'usb_', 'uart_send', 'trigger_', 'Codec', 'codec_', 'HAL_', 'adc_',
               'USB', 'CDC', 'I2S', 'buttonHandler_', 'uart_clearFrontFifo', 'dio_', 'screensaver_',
               'copyClear_', 'encode_', 'f_', 'frontPanel_parseData', 'frontPanel_ccHandler', 'frontParser_parseNrpn',
               'frontParser_updateTrackLeds']
CMSIS = 'Drivers/CMSIS/Include/cmsis_gcc.h'


def in_full(f):
    return f in FULL_FILES or any(f.startswith(d) for d in FULL_DIRS)


def scope_hdr(f):
    return f in SCOPE_HDRS or any(f.startswith(d) for d in SCOPE_HDR_DIRS)


def stopped(name):
    return any(name.startswith(p) for p in STOP_PREFIX)

# ---------------------------------------------------------------- O0 DWARF: functions + types
O0 = {}
for fn in sorted(glob.glob(S + 'o0json/*.json')):
    d = json.load(open(fn))
    if not d['cus']:
        print('no CU in', fn, file=sys.stderr); continue
    cu = d['cus'][0]
    O0[cu['name']] = d

# all function definitions, keyed by (name, file)
FUNCS = {}
for cuname, d in O0.items():
    for f in d['cus'][0]['funcs']:
        if f['declaration'] or not f['file']:
            continue
        key = (f['name'], f['file'])
        if key not in FUNCS:
            f = dict(f)
            f['cu'] = cuname
            FUNCS[key] = f

# functions only known from the reference ELF (files not compiled here: UI, HAL, ...)
for cu in REF['cus']:
    for f in cu['funcs']:
        if f['declaration'] or not f['file'] or not f['name']:
            continue
        key = (f['name'], f['file'])
        if key not in FUNCS:
            f = dict(f)
            f['cu'] = 'REF'
            FUNCS[key] = f

by_name = defaultdict(list)
for (n, fl), f in FUNCS.items():
    by_name[n].append(fl)

# ---------------------------------------------------------------- preprocessed source
LM = re.compile(r'^# (\d+) "([^"]*)"')


class PP:
    def __init__(self, path):
        self.lines = []  # (file, line, text)
        cur_f, cur_l = None, 0
        for raw in open(path, errors='replace'):
            m = LM.match(raw)
            if m:
                cur_l = int(m.group(1)); cur_f = m.group(2)
                continue
            self.lines.append((cur_f, cur_l, raw.rstrip('\n')))
            cur_l += 1
        self.index = {}
        for i, (f, l, _) in enumerate(self.lines):
            self.index.setdefault((f, l), i)

    def body(self, f, line):
        """Return the text of the function definition starting at file:line (signature + body)."""
        i = self.index.get((f, line))
        if i is None:
            return None
        text = '\n'.join(t for _, _, t in self.lines[i:i + 4000])
        # find first '{' at paren depth 0
        depth = 0; j = 0; n = len(text)
        while j < n:
            c = text[j]
            if c == '(':
                depth += 1
            elif c == ')':
                depth -= 1
            elif c == ';' and depth == 0:
                return None
            elif c == '{' and depth == 0:
                break
            j += 1
        sig = text[:j]
        k = j; depth = 0
        while k < n:
            c = text[k]
            if c == '"' or c == "'":
                q = c; k += 1
                while k < n and text[k] != q:
                    if text[k] == '\\':
                        k += 1
                    k += 1
            elif c == '{':
                depth += 1
            elif c == '}':
                depth -= 1
                if depth == 0:
                    off = text.count('\n', 0, k)
                    return sig, text[j:k + 1], self.lines[i + off][1]
            k += 1
        return sig, text[j:], None


PPS = {}
for p in glob.glob(S + 'pp/*.i'):
    PPS[os.path.basename(p)[:-2].replace('_', '/')] = p  # not exact, fixed below
PPS = {}
for cuname in O0:
    PPS[cuname] = S + 'pp/' + cuname.replace('/', '_') + '.i'
_ppcache = {}


def pp(cuname):
    if cuname not in _ppcache:
        _ppcache[cuname] = PP(PPS[cuname])
    return _ppcache[cuname]


for key, f in FUNCS.items():
    if f['cu'] == 'REF':
        f['sig_pp'], f['body'], f['end_line'] = None, '', None
        continue
    b = pp(f['cu']).body(f['file'], f['line'])
    if b is None:
        # the definition may start a line earlier (return type on its own line)
        b = pp(f['cu']).body(f['file'], f['line'] - 1)
    f['sig_pp'], f['body'], f['end_line'] = b if b else (None, '', None)

IDENT = re.compile(r'(?<![\w.>])(?<!->)([A-Za-z_]\w*)')


def idents(text):
    out = set()
    for m in IDENT.finditer(text):
        s = m.start()
        # skip members: preceded by '.' or '->' (with optional spaces)
        k = s - 1
        while k >= 0 and text[k] in ' \t\n':
            k -= 1
        if k >= 0 and (text[k] == '.' or (text[k] == '>' and k > 0 and text[k - 1] == '-')):
            continue
        out.add(m.group(1))
    return out


for key, f in FUNCS.items():
    f['idents'] = idents(f['body']) if f['body'] else set()

# ---------------------------------------------------------------- reference ELF globals
RAM = lambda a: a is not None and (0x20000000 <= a < 0x20080000 or 0x08000000 <= a < 0x08100000)
GLOBALS = {}   # (name, file_of_definition_cu) -> info
gl_by_name = defaultdict(list)
REF_TYPES = REF['types']
for cu in REF['cus']:
    for v in cu['vars']:
        if v['addr'] is None or not RAM(v['addr']):
            continue
        key = (v['name'], cu['name'])
        GLOBALS[key] = dict(v, cu=cu['name'], local_of=None)
        gl_by_name[v['name']].append(key)
    for v in cu['localstatics']:
        if not RAM(v['addr']) or v['name'] is None:
            continue
        key = (v['func'] + '::' + v['name'], cu['name'])
        GLOBALS[key] = dict(v, cu=cu['name'], local_of=v['func'], external=False, declaration=False)
        gl_by_name[key[0]].append(key)

# map from address to global key
gl_by_addr = {g['addr']: k for k, g in GLOBALS.items()}

# ---------------------------------------------------------------- scope: functions
def fkey_lookup(name, prefer_file=None, prefer_cu=None):
    fls = by_name.get(name, [])
    if not fls:
        return None
    if prefer_file in fls:
        return (name, prefer_file)
    if prefer_cu:
        for fl in fls:
            if fl == prefer_cu:
                return (name, fl)
    # prefer non-static
    for fl in fls:
        if FUNCS[(name, fl)]['external']:
            return (name, fl)
    return (name, fls[0])


def callees(key):
    f = FUNCS[key]
    out = set()
    for i in f['idents']:
        if i in by_name and i != key[0]:
            k = fkey_lookup(i, prefer_file=key[1], prefer_cu=f['cu'])
            if k:
                out.add(k)
    return out


in_scope = set()
stack = []
for key, f in FUNCS.items():
    if in_full(key[1]) or (scope_hdr(key[1]) and key[1].endswith('.h')):
        pass  # header inline functions are added only when reachable (below)
    if in_full(key[1]):
        stack.append(key)
for r in ROOTS:
    k = fkey_lookup(r)
    if k:
        stack.append(k)
external = set()
while stack:
    k = stack.pop()
    if k in in_scope or k in external:
        continue
    fl = k[1]
    if stopped(k[0]) or not (in_full(fl) or fl in PARTIAL_FILES or scope_hdr(fl) or fl == CMSIS):
        external.add(k)
        continue
    in_scope.add(k)
    for c in callees(k):
        stack.append(c)

# all inline functions of in-scope headers are in scope as well (BufferTools.h etc.)
for key, f in FUNCS.items():
    if scope_hdr(key[1]) and key[1].endswith('.h') and key not in in_scope:
        in_scope.add(key)
        for c in callees(key):
            if c not in in_scope:
                if stopped(c[0]) or not (in_full(c[1]) or c[1] in PARTIAL_FILES or scope_hdr(c[1]) or c[1] == CMSIS):
                    external.add(c)

# library functions (libm etc.) referenced by in-scope code
LIBM = set()
libm_names = {'expf', 'logf', 'exp', 'fmod', 'sinf', 'cosf', 'powf', 'tanhf', 'fmaf', 'sqrtf', 'fabsf', 'log', 'pow',
              'exp2f', 'log2f', 'log10f', 'floorf', 'ceilf', 'roundf', 'truncf', 'fmodf', 'sin', 'cos', 'tanf',
              'atanf', 'sqrt', 'fabs', 'floor', 'ceil', 'lrintf', 'rintf', 'copysignf', 'fmaxf', 'fminf'}
for k in in_scope:
    for i in FUNCS[k]['idents']:
        if i in libm_names:
            LIBM.add((i, k))

# ---------------------------------------------------------------- linked status (reference ELF)
ref_funcs = {}
for cu in REF['cus']:
    for f in cu['funcs']:
        if f['declaration']:
            continue
        ref_funcs.setdefault((f['name'], f['file']), []).append(f)
elf_syms = CG['funcs']


def link_status(key):
    name, fl = key
    if name in elf_syms or any(s.split('.')[0] == name for s in ()):
        a = elf_syms[name][0]
        return 'itcm' if a < 0x01000000 else 'flash'
    # out-of-line clones (.part/.isra/.constprop)
    for s in elf_syms:
        if s.split('.')[0] == name:
            return 'clone'
    # inlined somewhere?
    for cu in REF['cus']:
        for fn, inl in cu['inlined'].items():
            if name in inl:
                return 'inlined'
    if key in ref_funcs:
        return 'gc'
    return 'absent'

# ---------------------------------------------------------------- scope: globals
def resolve_global(name, fn_key):
    ks = gl_by_name.get(name, [])
    if not ks:
        return None
    f = FUNCS[fn_key]
    for k in ks:
        if k[1] == f['cu'] and not GLOBALS[k]['external'] and GLOBALS[k]['local_of'] is None:
            return k
    for k in ks:
        if GLOBALS[k]['external']:
            return k
    return None


used_globals = defaultdict(set)  # global key -> set of function keys
for k in in_scope:
    f = FUNCS[k]
    for i in f['idents']:
        g = resolve_global(i, k)
        if g:
            used_globals[g].add(k)
    # function-local statics
    for g, gv in GLOBALS.items():
        if gv['local_of'] == k[0] and g[1] == FUNCS[k]['cu']:
            used_globals[g].add(k)
    # ELF data references
    for d in CG['data'].get(k[0], []):
        if d.startswith('@'):
            continue
        base = d.split('.')[0]
        for gk in gl_by_name.get(d, []) + gl_by_name.get(base, []):
            if GLOBALS[gk]['cu'] == f['cu'] or GLOBALS[gk]['external']:
                used_globals[gk].add(k)

# ---------------------------------------------------------------- C type strings
def ctype(types, k):
    if k is None: return 'void'
    t = types[str(k)]
    tg = t['tag']
    if tg == 'DW_TAG_base_type': return t['name']
    if tg == 'DW_TAG_typedef': return t['name']
    if tg == 'DW_TAG_pointer_type': return ctype(types, t.get('type')) + '*'
    if tg == 'DW_TAG_const_type': return 'const ' + ctype(types, t.get('type'))
    if tg == 'DW_TAG_volatile_type': return 'volatile ' + ctype(types, t.get('type'))
    if tg == 'DW_TAG_restrict_type': return ctype(types, t.get('type'))
    if tg == 'DW_TAG_array_type': return ctype(types, t.get('type')) + ''.join('[%s]' % x for x in t['dims'])
    if tg == 'DW_TAG_structure_type': return 'struct ' + str(t['name'])
    if tg == 'DW_TAG_union_type': return 'union ' + str(t['name'])
    if tg == 'DW_TAG_enumeration_type': return 'enum ' + str(t['name'])
    if tg == 'DW_TAG_subroutine_type': return 'fnptr'
    return tg


def strip_cv(types, k):
    """follow const/volatile/typedef to the underlying type key"""
    while k is not None:
        t = types[str(k)]
        if t['tag'] in ('DW_TAG_const_type', 'DW_TAG_volatile_type', 'DW_TAG_typedef', 'DW_TAG_restrict_type'):
            k = t.get('type')
        else:
            return k
    return None


def otypes(key):
    cu = FUNCS[key]['cu']
    return REF_TYPES if cu == 'REF' else O0[cu]['types']


# ---------------------------------------------------------------- tables (read-only data)
def is_const_type(T, k):
    while k is not None:
        t = T[str(k)]
        if t['tag'] == 'DW_TAG_const_type':
            return True
        if t['tag'] in ('DW_TAG_volatile_type', 'DW_TAG_typedef', 'DW_TAG_array_type'):
            k = t.get('type'); continue
        return False
    return False


WRITE_RE_T = r'(?<![\w.])%s\s*(\[[^\]]*\]\s*)*(\.\w+\s*|->\w+\s*|\[[^\]]*\]\s*)*([-+*/|&^]?=(?!=)|\+\+|--)'


def written_in_source(name):
    """True if any in-scope (or any compiled) function body writes to the global `name`."""
    rx = re.compile(WRITE_RE_T % re.escape(name))
    rx2 = re.compile(r'(\+\+|--)\s*%s\b' % re.escape(name))
    rx3 = re.compile(r'&\s*%s\b' % re.escape(name))  # address taken: assume written
    for k, f in FUNCS.items():
        b = f['body']
        if name in f['idents'] and (rx.search(b) or rx2.search(b) or rx3.search(b)):
            return True
    return False


TABLES = set()
for g in used_globals:
    gv = GLOBALS[g]
    if is_const_type(REF_TYPES, gv['type']) or g[0] in ('parameterOrder',):
        TABLES.add(g)

# ---------------------------------------------------------------- method vs free function
direct_globals = defaultdict(set)
for g, fs in used_globals.items():
    for f in fs:
        direct_globals[f].add(g)
CALLS = {k: callees(k) for k in in_scope}
needs_self = set(k for k in in_scope if direct_globals.get(k, set()) - TABLES) | set(external)
changed = True
while changed:
    changed = False
    for k in in_scope:
        if k not in needs_self and any(c in needs_self for c in CALLS[k]):
            needs_self.add(k); changed = True


def is_struct_ptr(T, tk):
    t = T[str(tk)]
    if t['tag'] != 'DW_TAG_pointer_type':
        return False
    u = strip_cv(T, t.get('type'))
    return u is not None and T[str(u)]['tag'] in ('DW_TAG_structure_type', 'DW_TAG_union_type')


# transitive globals (excluding tables)
trans_globals = {k: set(direct_globals.get(k, set())) - TABLES for k in in_scope}
changed = True
while changed:
    changed = False
    for k in in_scope:
        for c in CALLS[k]:
            if c in trans_globals and not trans_globals[c] <= trans_globals[k]:
                trans_globals[k] |= trans_globals[c]; changed = True


def type_contains(T, k, target_name, seen=None):
    """does type k (REF types) contain a struct named target_name (by struct tag or typedef name)?"""
    if k is None:
        return False
    t = T[str(k)]
    tg = t['tag']
    if tg == 'DW_TAG_typedef':
        if t['name'] == target_name:
            return True
        return type_contains(T, t.get('type'), target_name)
    if tg in ('DW_TAG_const_type', 'DW_TAG_volatile_type', 'DW_TAG_array_type'):
        return type_contains(T, t.get('type'), target_name)
    if tg in ('DW_TAG_structure_type', 'DW_TAG_union_type'):
        if t['name'] == target_name:
            return True
        return any(type_contains(T, m['type'], target_name) for m in t['members'])
    return False


def ptr_target_names(T, tk):
    """names (typedef + tag) of the struct a pointer type points to"""
    t = T[str(tk)]
    k = t.get('type'); names = []
    while k is not None:
        u = T[str(k)]
        if u['tag'] == 'DW_TAG_typedef':
            names.append(u['name'])
        if u['tag'] in ('DW_TAG_const_type', 'DW_TAG_volatile_type', 'DW_TAG_typedef'):
            k = u.get('type'); continue
        if u['name']:
            names.append(u['name'])
        break
    return names


def place_params(k):
    """struct pointer params of method k whose pointee may be reached by k through globals"""
    if k not in needs_self:
        return []
    T = otypes(k); out = []
    for p in FUNCS[k]['params']:
        if not is_struct_ptr(T, p['type']):
            continue
        names = ptr_target_names(T, p['type'])
        if any(type_contains(REF_TYPES, GLOBALS[g]['type'], n) for g in trans_globals.get(k, ()) for n in names):
            out.append(p['name'])
    return out


if __name__ == '__main__':
    import pprint
    files = defaultdict(list)
    for k in sorted(in_scope):
        files[k[1]].append(k[0])
    for fl in sorted(files):
        print(fl, len(files[fl]))
        print('   ', ' '.join(files[fl]))
    print('EXTERNAL')
    for k in sorted(external, key=lambda x: x[1]):
        print('   ', k)
    print('LIBM', sorted(set(x[0] for x in LIBM)))
    print('GLOBALS', len(used_globals))
    for g in sorted(used_globals, key=lambda g: g[1]):
        gv = GLOBALS[g]
        print('   %-45s %-50s %08x' % (g[0], g[1], gv['addr']))
