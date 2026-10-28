"""Third part of the generator: writes the Rust files and docs/PORTING_STATUS.md."""
import sys, os; sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from gen_main import *  # noqa

LINK_DESC = {'flash': 'out-of-line in flash', 'itcm': 'out-of-line in ITCM', 'clone': 'out-of-line as a GCC clone (.part/.isra/.constprop)',
             'inlined': 'inlined into its callers only', 'gc': 'not linked (removed by --gc-sections)',
             'absent': 'not in the reference ELF (file not linked or function optimised away)'}

# tables: C global name -> tables module
TABLE_MOD = {'sine_table': 'wavetable', 'sawTable': 'wavetable', 'triTable': 'wavetable', 'recTable': 'wavetable',
             'transientData': 'transient_tables', 'transientVolumeTable': 'transient_tables',
             'table_1n914': 'distortion', 'ecc83Table': 'distortion', 'MidiNoteFrequencies': 'midi_note_numbers',
             'initKit': 'init_kit', 'modTargets': 'cc2_text', 'parameterOrder': 'parameters'}
TABLE_CMOD = {'sine_table': 'dsp::wavetable', 'sawTable': 'dsp::wavetable', 'triTable': 'dsp::wavetable', 'recTable': 'dsp::wavetable',
              'transientData': 'dsp::transient_tables', 'transientVolumeTable': 'dsp::transient_tables',
              'table_1n914': 'dsp::fx::distortion', 'ecc83Table': 'dsp::fx::distortion', 'MidiNoteFrequencies': 'midi::midi_note_numbers',
              'initKit': 'preset::init_kit', 'modTargets': 'menu::cc2_text', 'parameterOrder': 'parameters'}
for g in TABLES:
    assert g[0] in TABLE_MOD, g

# ============================================================== module set
MOD_FILES = defaultdict(set)
for fl in open(S + 'files.txt').read().split():
    if in_full(fl) or fl in PARTIAL_FILES:
        MOD_FILES[mod_of(fl)].add(fl)
for d in O0.values():
    pass
# headers of in-scope dirs
for root in ['Core/Inc/Soundengine', 'Core/Inc/Preset']:
    for dp, dn, fns in os.walk(SRC + root):
        for fn in fns:
            fl = os.path.relpath(os.path.join(dp, fn), SRC)
            m = mod_of(fl)
            if m and m != 'external':
                MOD_FILES[m].add(fl)
MOD_FILES['parameters'].add('Core/Inc/Parameters.h')
MOD_FILES['parameters'].add('Core/Src/Parameters.c')
MOD_FILES['front_panel_parser'] |= {'Core/Src/frontPanelParser.c', 'Core/Inc/frontPanelParser_ui.h'}
MOD_FILES['menu'].add('Core/Inc/menu/menu.h')
MOD_FILES['menu::cc2_text'].add('Core/Src/menu/Cc2Text.c')
MOD_FILES['cmsis'].add(CMSIS)
for it in FUNC_ITEMS:
    MOD_FILES[it['mod']].add(it['file'])
for m in list(CONST_ITEMS) + list(ENUM_ITEMS) + list(STRUCT_ITEMS) + list(FN_MACROS):
    MOD_FILES[m]
for m in list(MOD_FILES):
    if m in ('audio_codec_manager', 'cs4344_cs5343', 'trigger_out', 'usb_manager'):
        MOD_FILES['external'] |= MOD_FILES.pop(m)
MOD_FILES['external']

# fields of Lxr grouped by defining file
# ============================================================== prelude names
PRELUDE = OrderedDict()  # name -> path
DUP = set()


def prelude_add(name, path):
    if name in PRELUDE and PRELUDE[name] != path:
        DUP.add(name)
    else:
        PRELUDE[name] = path


for mod, items in CONST_ITEMS.items():
    for n, _ in items:
        prelude_add(rid(n), 'crate::%s::%s' % (mod, rid(n)))
for mod, items in ENUM_ITEMS.items():
    for code in items:
        for m in re.finditer(r'^pub (?:type|const) (\S+?)[: ]', code, re.M):
            prelude_add(m.group(1), 'crate::%s::%s' % (mod, m.group(1)))
for mod, items in STRUCT_ITEMS.items():
    for code in items:
        for m in re.finditer(r'^pub (?:struct|type) (\w+)', code, re.M):
            prelude_add(m.group(1), 'crate::%s::%s' % (mod, m.group(1)))
for g in sorted(TABLES):
    prelude_add(g[0], 'crate::tables::%s::%s' % (TABLE_MOD[g[0]], g[0]))
for n in DUP:
    REPORT['prelude_dup'].append(n)
    del PRELUDE[n]

# ============================================================== writers
def write(path, text):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, 'w') as f:
        f.write(text)


def mod_path(mod):
    parts = mod.split('::')
    return parts


def fn_doc(it):
    d = ['/// C: `%s`' % clean_doc(it['cdecl'])]
    d.append('///')
    d.append('/// Source: `%s:%s`. Reference build: %s. Group %s.' % (it['file'], it['line'], LINK_DESC[it['link']], it['group']))
    if it['globals']:
        d.append('/// Globals: %s.' % ', '.join('`%s`' % g for g in it['globals']))
    if it['tables']:
        d.append('/// Tables: %s.' % ', '.join('`%s`' % g for g in it['tables']))
    cl = [c for c in it['calls'] if c in FUNC_BY_KEY]
    if cl:
        d.append('/// Calls: %s.' % ', '.join('`%s`' % rust_path(FUNC_BY_KEY[c]) for c in cl))
    for pn, pt, note in it['params']:
        if note == 'place':
            d.append('///')
            d.append('/// `%s` is a place accessor (see `crate::place`): the C pointer may point into `Lxr` and this' % pn)
            d.append('/// function can also reach the pointee through globals, so a plain `&mut` would not be faithful.')
        elif note:
            d.append('/// `%s`: %s.' % (pn, note))
    if it['external']:
        d.append('///')
        d.append('/// Hardware / UI function outside the engine, called from engine code. Decide its behaviour')
        d.append('/// (no-op, fixed value, or an emulation of its effect on engine state) when porting the caller.')
    return '\n'.join(d)


FUNC_BY_KEY = {it['key']: it for it in FUNC_ITEMS}


def rust_path(it):
    if it['skip']:
        return 'Lxr::%s' % it['name'] if it['method'] else it['name']
    if it['method']:
        return 'Lxr::%s' % it['name']
    return '%s::%s' % (it['mod'], it['name'])


def fn_sig(it, indent=''):
    ps = (['&mut self'] if it['method'] else []) + ['%s: %s' % (n, t) for n, t, _ in it['params']]
    ret = ' -> %s' % it['ret'] if it['ret'] else ''
    return '%spub fn %s(%s)%s' % (indent, rid(it['name']), ', '.join(ps), ret)


def emit_fn(it, indent=''):
    doc = '\n'.join(indent + l for l in fn_doc(it).split('\n'))
    body = '%s    todo!("port: %s:%s")' % (indent, it['file'], it['line'])
    return '%s\n%s {\n%s\n%s}' % (doc, fn_sig(it, indent), body, indent)


def emit_module(mod):
    files = sorted(MOD_FILES.get(mod, ()))
    out = []
    head = '//! Port of %s.' % ', '.join('`%s`' % f for f in files) if files else '//! (no C files)'
    out.append(head)
    out.append('//!')
    if mod == 'external':
        out.append('//! Functions, constants and types of hardware / UI files that engine code uses. They are not part of the')
        out.append('//! sound engine; each stub must be given a deliberate behaviour (usually a no-op) by the group that ports')
        out.append('//! the caller (P3).')
    else:
        out.append('//! Porting group: %s. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.' % group_of(mod))
    out.append('//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the')
    out.append('//! reference ELF. Function bodies are `todo!()` until ported.')
    out.append('')
    out.append('use crate::prelude::*;')
    out.append('')
    tabs = sorted(n for n, m in TABLE_CMOD.items() if m == mod)
    if tabs:
        out.append('// ---- tables (generated by tools/gen_tables.py from the reference ELF)')
        out.append('pub use crate::tables::%s::{%s};' % (TABLE_MOD[tabs[0]], ', '.join(tabs)))
        out.append('')
    if CONST_ITEMS.get(mod):
        out.append('// ---- #define constants')
        for n, code in CONST_ITEMS[mod]:
            out.append(code)
        out.append('')
    if FN_MACROS.get(mod):
        out.append('// ---- C macros that are not constants (function-like or not constant expressions); not translated:')
        for n, fl, line, src in FN_MACROS[mod]:
            out.append('// %s:%d: %s' % (os.path.normpath(fl), line, src))
        out.append('')
    if ENUM_ITEMS.get(mod):
        out.append('// ---- enums')
        for code in ENUM_ITEMS[mod]:
            out.append(code)
        out.append('')
    if STRUCT_ITEMS.get(mod):
        out.append('// ---- types')
        for code in STRUCT_ITEMS[mod]:
            out.append(code)
            out.append('')
    frees = [it for it in FUNC_ITEMS if it['mod'] == mod and not it['method'] and not it['skip']]
    meths = [it for it in FUNC_ITEMS if it['mod'] == mod and it['method'] and not it['skip']]
    if frees:
        out.append('// ---- functions without access to globals')
        for it in frees:
            out.append(emit_fn(it))
            out.append('')
    if meths:
        out.append('// ---- functions that use globals (methods on `Lxr`)')
        out.append('impl Lxr {')
        for it in meths:
            out.append(emit_fn(it, '    '))
            out.append('')
        if out[-1] == '':
            out.pop()
        out.append('}')
        out.append('')
    return '\n'.join(out).rstrip() + '\n'


ALL_MODS = sorted(set(MOD_FILES) | set(it['mod'] for it in FUNC_ITEMS))
for mod in ALL_MODS:
    parts = mod.split('::')
    # leaf modules -> file; a module that has children becomes dir/mod.rs
    has_children = any(m.startswith(mod + '::') for m in ALL_MODS)
    if has_children:
        path = SRCDIR + '/'.join(parts) + '/mod.rs'
    else:
        path = SRCDIR + '/'.join(parts) + '.rs'
    code = emit_module(mod)
    if has_children:
        kids = sorted(set(m[len(mod) + 2:].split('::')[0] for m in ALL_MODS if m.startswith(mod + '::')))
        code = code.replace('use crate::prelude::*;', '\n'.join('pub mod %s;' % k for k in kids) + '\n\nuse crate::prelude::*;', 1)
    write(path, code)

# intermediate parents without own files (dsp, dsp::fx, midi, ...)
parents = set()
for mod in ALL_MODS:
    parts = mod.split('::')
    for i in range(1, len(parts)):
        parents.add('::'.join(parts[:i]))
for p in sorted(parents - set(ALL_MODS)):
    kids = sorted(set(m[len(p) + 2:].split('::')[0] for m in ALL_MODS if m.startswith(p + '::')))
    desc = {'dsp': 'Port of `Core/Src/Soundengine/DSPAudio/` (voices, oscillators, filters, envelopes, mixer).',
            'dsp::fx': 'Port of `Core/Src/Soundengine/DSPAudio/FX/` (master effects of the FX bus).',
            'dsp::fx::verb_tools': 'Port of `Core/Src/Soundengine/DSPAudio/FX/VerbTools/` (not linked in the reference firmware).',
            'midi': 'Port of `Core/Src/Soundengine/MIDI/`.'}.get(p, '')
    write(SRCDIR + p.replace('::', '/') + '/mod.rs', '//! %s\n\n%s\n' % (desc, '\n'.join('pub mod %s;' % k for k in kids)))

TOP = sorted(set(m.split('::')[0] for m in ALL_MODS))

# ============================================================== prelude.rs
pl = ['//! Everything a ported module needs in scope: all C types, enum aliases, `#define` constants and tables,',
      '//! plus `Lxr`, `CPtr` and the FPU helpers. Generated; names that exist in two C files with different',
      '//! values are not re-exported here (use the full path).', '',
      'pub use crate::engine::{Block, Lxr};', 'pub use crate::fpu;', 'pub use crate::fpu::libm;',
      'pub use crate::fpu::{fma, vfma, vfms, vfnma, vfnms};', 'pub use crate::memmap::CPtr;', '']
by_mod = defaultdict(list)
for n, p in PRELUDE.items():
    m, item = p.rsplit('::', 1)
    by_mod[m].append(item)
for m in sorted(by_mod):
    items = sorted(by_mod[m])
    pl.append('pub use %s::{%s};' % (m, ', '.join(items)))
write(SRCDIR + 'prelude.rs', '\n'.join(pl) + '\n')

# ============================================================== engine.rs (Lxr)
def field_rt(f):
    return 'Box<%s>' % f['rt'] if f['boxed'] else f['rt']


def field_init(f):
    if f['boxed']:
        return 'Box::new(%s)' % f['init']
    return f['init']


by_file = OrderedDict()
for f in sorted(LXR_FIELDS, key=lambda f: (f['defcu'], f['addr'])):
    by_file.setdefault(f['defcu'], []).append(f)

e = []
e.append('''//! `Lxr`: the state of the whole engine. Every C global variable that the ported code uses is a field
//! with the C name (function-local `static` variables are named `<function>_<variable>`). Large buffers are
//! boxed and allocated once in the constructor.
//!
//! Generated from the DWARF and the `.data` / `.dtcm` contents of the reference ELF (`lxr_v1.7_gcc11.3.elf`):
//! `new_zeroed()` is the state right after the firmware's startup code (`.data`/`.dtcm` copied from flash,
//! `.bss`/`.dtcmz` zeroed) and before `main()`.

use crate::prelude::*;

/// One TIM2 interrupt worth of audio: `OUTPUT_DMA_SIZE` = 64 stereo frames per DAC, int16, L/R interleaved.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Block {
    pub dac1: [i16; 128],
    pub dac2: [i16; 128],
}

impl Default for Block {
    fn default() -> Self {
        Block { dac1: [0; 128], dac2: [0; 128] }
    }
}

/// The engine state: one field per C global (same name and C type). See the module documentation.
#[derive(Clone)]
pub struct Lxr {''')
for cu, fs in by_file.items():
    e.append('    // ---------------- %s' % cu)
    for f in fs:
        doc = '`%s`' % clean_doc(re.sub(r'\s+', ' ', f['cdecl'])) if f['cdecl'] else '(C global `%s`)' % f['key'][0]
        e.append('    /// %s' % doc)
        loc = '%s:%s' % (f['decl_file'], f['decl_line'])
        e.append('    ///')
        e.append('    /// Declared at `%s`; firmware address 0x%08x (`%s`), %d bytes.' % (loc, f['addr'], f['sec'], f['size']))
        if f['local_of']:
            e.append('    /// Function-local `static` of `%s`.' % f['local_of'])
        if f['note']:
            e.append('    /// %s' % f['note'])
        e.append('    pub %s: %s,' % (rid(f['name']), field_rt(f)))
e.append('    // ---------------- not C globals')
e.append('    /// State of the deterministic replacement of the hardware RNG (`GetRngValue`, see `crate::rng`).')
e.append('    pub rng_state: u32,')
e.append('}')
e.append('')
e.append('impl Lxr {')
e.append('    /// The state after the firmware\'s startup code: every global at its static initial value (generated from the')
e.append('    /// reference ELF), the RNG seeded with `crate::rng::RNG_SEED`. Nothing of `main()` has run yet.')
e.append('    pub fn new_zeroed() -> Self {')
e.append('        Lxr {')
for cu, fs in by_file.items():
    for f in fs:
        e.append('            %s: %s,' % (rid(f['name']), field_init(f)))
e.append('            rng_state: crate::rng::RNG_SEED,')
e.append('        }')
e.append('    }')
e.append('')
e.append('''    /// Power-on: reproduces the firmware start without SD card (see `docs/ENGINE_CONTRACT.md`), then loads
    /// the init kit. Group P3.
    pub fn new() -> Self {
        todo!("port: Core/Src/main.c main() without SD card, then load the init kit")
    }

    /// Feeds one byte of DIN MIDI through the firmware parser (main-loop context, FZ = 0). Group P3.
    pub fn midi_byte(&mut self, b: u8) {
        todo!("port: Core/Src/Soundengine/MIDI/Uart.c uart_processMidi -> midiParser_parseUartData")
    }

    /// Runs one TIM2 interrupt: `seq_tick(); soundengine_calcNextSampleBlock();` with FZ = 1 (use
    /// `crate::fpu::FtzGuard`). Group P3.
    pub fn tick(&mut self) -> Block {
        todo!("port: Core/Src/Soundengine/Soundengine.c soundengine_process (interrupt part)")
    }
}''')
write(SRCDIR + 'engine.rs', '\n'.join(e) + '\n')

# ============================================================== memmap.rs
mm = ['''//! C pointers and the firmware memory map.
//!
//! Struct fields of C pointer type are stored as `CPtr`: the address the pointer would hold in the reference
//! firmware (`lxr_v1.7_gcc11.3.elf`), 0 = NULL. This keeps NULL checks, pointer comparisons and the values
//! themselves identical to the firmware (the emulator can compare them directly). Dereferencing goes through
//! `Lxr::cptr_*`, which map an address back to the `Lxr` field (implement with the address and offset tables
//! below).
//!
//! Generated from the DWARF of the reference ELF: `ADDR_<global>` are the addresses of the C globals that are
//! fields of `Lxr`, `size_of_c` / `offset_of_c` the C sizes and member offsets of every C struct.

use crate::prelude::*;

/// A C pointer stored as its address in the reference firmware's memory map (0 = NULL).
pub type CPtr = u32;
''']
for f in sorted(LXR_FIELDS, key=lambda f: f['addr']):
    mm.append('/// `%s` (%d bytes)\npub const ADDR_%s: CPtr = 0x%08x;' % (f['name'], f['size'], f['name'], f['addr']))
for g in sorted(TABLES, key=lambda g: GLOBALS[g]['addr']):
    mm.append('/// table `%s`\npub const ADDR_%s: CPtr = 0x%08x;' % (g[0], g[0], GLOBALS[g]['addr']))
mm.append('')
mm.append('/// C `sizeof` of every C struct type (bytes).')
mm.append('pub mod size_of_c {')
offs = ['/// C `offsetof(<struct>, <member>)` of every C struct type, named `<struct>_<member>`.', 'pub mod offset_of_c {']
for n, (T, k) in sorted(needed_structs.items()):
    t = T[str(k)]
    mm.append('    pub const %s: u32 = %d;' % (n, t.get('size', 0)))
    for m in t['members']:
        offs.append('    pub const %s_%s: u32 = %d;' % (n, m['name'], m.get('offset', 0)))
mm.append('}')
offs.append('}')
mm.append('')
mm.extend(offs)
mm.append('''
impl Lxr {
    /// Reads the `f32` at firmware address `p` (a field of an `Lxr` global). Non-C helper, group P3
    /// (used by ParameterArray / modulationNode code).
    pub fn cptr_read_f32(&self, p: CPtr) -> f32 {
        todo!("non-C helper: map firmware address to Lxr field")
    }
    /// Writes the `f32` at firmware address `p`. Non-C helper, group P3.
    pub fn cptr_write_f32(&mut self, p: CPtr, v: f32) {
        todo!("non-C helper: map firmware address to Lxr field")
    }
    /// Reads the `u8` at firmware address `p`. Non-C helper, group P3.
    pub fn cptr_read_u8(&self, p: CPtr) -> u8 {
        todo!("non-C helper: map firmware address to Lxr field")
    }
    /// Writes the `u8` at firmware address `p`. Non-C helper, group P3.
    pub fn cptr_write_u8(&mut self, p: CPtr, v: u8) {
        todo!("non-C helper: map firmware address to Lxr field")
    }
    /// Reads the `u32` at firmware address `p`. Non-C helper, group P3.
    pub fn cptr_read_u32(&self, p: CPtr) -> u32 {
        todo!("non-C helper: map firmware address to Lxr field")
    }
    /// Writes the `u32` at firmware address `p`. Non-C helper, group P3.
    pub fn cptr_write_u32(&mut self, p: CPtr, v: u32) {
        todo!("non-C helper: map firmware address to Lxr field")
    }
}''')
write(SRCDIR + 'memmap.rs', '\n'.join(mm) + '\n')

# ============================================================== lib.rs module list (written by hand-maintained template)
json.dump({'top': TOP, 'report': REPORT, 'funcs': [{k: v for k, v in it.items() if k != 'calls'} | {'calls': ['%s|%s' % c for c in it['calls']]} for it in FUNC_ITEMS],
           'fields': [{k: v for k, v in f.items() if k not in ('init',)} for f in LXR_FIELDS], 'mods': ALL_MODS,
           'tables': sorted(g[0] for g in TABLES), 'fn_macros': {m: [x[0] for x in v] for m, v in FN_MACROS.items()},
           'libm': sorted(set((x[0], x[1][0]) for x in LIBM))},
          open(S + 'gen_out.json', 'w'), indent=1, default=list)
print('modules', len(ALL_MODS), 'functions', len(FUNC_ITEMS), 'fields', len(LXR_FIELDS), 'structs', len(needed_structs),
      'consts', len(CONST_SEEN))
for k, v in REPORT.items():
    print('REPORT', k, v[:20])
