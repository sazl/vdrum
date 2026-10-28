"""Generate the lxr-core skeleton (types, consts, Lxr globals, function stubs) from the model."""
import sys, os, re, json, struct
from collections import defaultdict, OrderedDict
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from model import *  # noqa

CRATE = sys.argv[1]  # crates/lxr-core
SRCDIR = CRATE + '/src/'
ELF_PATH = '/home/user/lxr-ref/lxr_v1.7_gcc11.3.elf'
from elftools.elf.elffile import ELFFile
ELF = ELFFile(open(ELF_PATH, 'rb'))

MACROS = json.load(open(S + 'macros.json')); MACROS.pop('__conflicts__', None)
MVALS = json.load(open(S + 'macro_values.json'))

REPORT = defaultdict(list)  # notes for the final report / status doc

# ============================================================== module mapping
def snake(n):
    s = re.sub(r'(?<=[a-z0-9])(?=[A-Z])', '_', n).lower()
    s = s.replace('__', '_')
    if s[0].isdigit():
        s = {'1_pole_lp': 'one_pole_lp'}.get(s, '_' + s)
    return s


def module_of(path):
    p = os.path.normpath(path)
    if p == CMSIS:
        return 'cmsis'
    if not p.startswith('Core/'):
        return None
    p = re.sub(r'^Core/(Inc|Src)/', '', p)
    p = re.sub(r'\.[ch]$', '', p)
    if p == 'Parameters':
        return 'parameters'
    if p in ('frontPanelParser', 'frontPanelParser_ui'):
        return 'front_panel_parser'
    if p == 'menu/menu':
        return 'menu'
    if p.startswith('menu/'):
        return 'menu::' + snake(p[5:])
    if p in ('Preset/presetManager', 'Preset/PresetManager'):
        return 'preset'
    if p in ('Preset/InitKit', 'Preset/initKit'):
        return 'preset::init_kit'
    if p.startswith('Preset/'):
        return 'preset::' + snake(p[7:])
    if p == 'Soundengine/Sequencer/sequencer':
        return 'sequencer'
    for pre, mod in [('Soundengine/DSPAudio/FX/VerbTools/', 'dsp::fx::verb_tools::'), ('Soundengine/DSPAudio/FX/', 'dsp::fx::'),
                     ('Soundengine/DSPAudio/', 'dsp::'), ('Soundengine/MIDI/', 'midi::'), ('Soundengine/Sequencer/', 'sequencer::'),
                     ('Soundengine/USB/', ''), ('Soundengine/', '')]:
        if p.startswith(pre):
            return mod + snake(p[len(pre):])
    return 'external'


# files that are not part of the engine: their items go to `external` (hardware / UI)
EXTERNAL_MODS = {'external', 'audio_codec_manager', 'cs4344_cs5343', 'trigger_out', 'usb_manager'}


def mod_of(path):
    m = module_of(path)
    if m in ('audio_codec_manager', 'cs4344_cs5343', 'trigger_out', 'usb_manager'):
        return 'external'
    return m


GROUP_BY_MOD = {}
P1 = ['dsp::one_pole_lp', 'dsp::buffer_tools', 'dsp::decay', 'dsp::distortion', 'dsp::dither', 'dsp::lfo', 'dsp::oscillator',
      'dsp::random', 'dsp::resonant_filter', 'dsp::slope_eg2', 'dsp::snap_eg', 'dsp::square_root_lut', 'dsp::tilt_eq',
      'dsp::transient_generator', 'dsp::transient_tables', 'dsp::wavetable', 'dsp::samples', 'dsp::automation_node',
      'dsp::modulation_node', 'dsp::median_filter', 'cmsis']
P4 = ['dsp::drum_voice', 'dsp::snare', 'dsp::hi_hat', 'dsp::cymbal_voice', 'dsp::mixer']


def group_of(mod):
    if mod in P1:
        return 'P1'
    if mod in P4:
        return 'P4'
    if mod == 'fx_bus' or mod.startswith('dsp::fx'):
        return 'P2'
    return 'P3'


RUST_KW = set('as break const continue crate else enum extern false fn for if impl in let loop match mod move mut pub ref return '
              'self Self static struct super trait true type unsafe use where while async await dyn abstract become box do final '
              'macro override priv typeof unsized virtual yield try gen'.split())


def rid(n):
    if n in ('self', 'Self', 'super', 'crate'):
        return n + '_'
    if n in RUST_KW:
        return 'r#' + n
    return n


# ============================================================== types
PRIM = {
    'unsigned char': 'u8', 'char': 'u8', 'signed char': 'i8', 'short int': 'i16', 'short unsigned int': 'u16',
    'int': 'i32', 'unsigned int': 'u32', 'long int': 'i32', 'long unsigned int': 'u32', 'long long int': 'i64',
    'long long unsigned int': 'u64', 'float': 'f32', 'double': 'f64', 'long double': 'f64', '_Bool': 'bool',
    '__fp16': 'u16', 'sizetype': 'u32',
}
PRIM_SIZE = {'u8': 1, 'i8': 1, 'i16': 2, 'u16': 2, 'i32': 4, 'u32': 4, 'i64': 8, 'u64': 8, 'f32': 4, 'f64': 8, 'bool': 1}

# hand-written representations (unions, bitfields): name -> rust source
HAND = OrderedDict()
HAND_DECODE = {}

HAND['ptrValue'] = ('midi::parameter_array', '''/// C: `typedef union { float flt; uint32_t itg; } ptrValue;` (Core/Inc/Soundengine/MIDI/ParameterArray.h:367), 4 bytes.
///
/// Representation: the union's storage as the raw 32 bit pattern (`itg`); `flt` is accessed through
/// `flt()`/`set_flt()` (`f32::from_bits`/`to_bits`), which is exactly what the C union does.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct ptrValue {
    /// `uint32_t itg;`
    pub itg: u32,
}
impl ptrValue {
    pub const ZERO: Self = Self { itg: 0 };
    /// `float flt;` (read)
    #[inline(always)]
    pub fn flt(&self) -> f32 { f32::from_bits(self.itg) }
    /// `float flt;` (write)
    #[inline(always)]
    pub fn set_flt(&mut self, v: f32) { self.itg = v.to_bits() }
}
''')
HAND_DECODE['ptrValue'] = lambda b: 'ptrValue { itg: 0x%08x }' % struct.unpack('<I', b[:4])[0]

HAND['ActiveSteps'] = ('sequencer', '''/// C: `typedef union { uint16_t word[NUM_BARS]; uint64_t quadword; } ActiveSteps;` (Core/Inc/Soundengine/Sequencer/sequencer.h:81), 8 bytes.
///
/// Representation: the storage is `word`; `quadword` is accessed through `quadword()`/`set_quadword()`
/// (little endian, as on the Cortex-M7).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct ActiveSteps {
    /// `uint16_t word[NUM_BARS];`
    pub word: [u16; 4],
}
impl ActiveSteps {
    pub const ZERO: Self = Self { word: [0; 4] };
    /// `uint64_t quadword;` (read)
    #[inline(always)]
    pub fn quadword(&self) -> u64 {
        (self.word[0] as u64) | (self.word[1] as u64) << 16 | (self.word[2] as u64) << 32 | (self.word[3] as u64) << 48
    }
    /// `uint64_t quadword;` (write)
    #[inline(always)]
    pub fn set_quadword(&mut self, v: u64) {
        for i in 0..4 {
            self.word[i] = (v >> (16 * i)) as u16;
        }
    }
}
''')
HAND_DECODE['ActiveSteps'] = lambda b: 'ActiveSteps { word: [%s] }' % ', '.join(str(x) for x in struct.unpack('<4H', b[:8]))

HAND['StepTiming'] = ('preset::seq_step', '''/// C: `typedef struct StepTiming { uint8_t flam:4; uint8_t shift:4; } StepTiming;` (Core/Inc/Preset/SeqStep.h:37), 1 byte.
///
/// Representation: the bit-field storage byte `bits`; GCC (little endian ARM) allocates `flam` in bits 0..3 and
/// `shift` in bits 4..7. Use the accessors.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct StepTiming {
    pub bits: u8,
}
impl StepTiming {
    pub const ZERO: Self = Self { bits: 0 };
    /// `uint8_t flam:4;`
    #[inline(always)]
    pub fn flam(&self) -> u8 { self.bits & 0x0f }
    #[inline(always)]
    pub fn set_flam(&mut self, v: u8) { self.bits = (self.bits & 0xf0) | (v & 0x0f) }
    /// `uint8_t shift:4;`
    #[inline(always)]
    pub fn shift(&self) -> u8 { self.bits >> 4 }
    #[inline(always)]
    pub fn set_shift(&mut self, v: u8) { self.bits = (self.bits & 0x0f) | (v << 4) }
}
''')
HAND_DECODE['StepTiming'] = lambda b: 'StepTiming { bits: %d }' % b[0]

HAND['MidiBits'] = ('midi::midi_messages', '''/// C: `struct MidiBits { enum MidiSource source:1; unsigned sysxbyte:1; unsigned length:2; unsigned :4; };`
/// (Core/Inc/Soundengine/MIDI/MidiMessages.h:25), 4 bytes (the bit-fields live in an `unsigned int` unit).
///
/// Representation: the 32 bit storage unit `bits`; `source` = bit 0, `sysxbyte` = bit 1, `length` = bits 2..3
/// (GCC little endian allocation). Use the accessors.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct MidiBits {
    pub bits: u32,
}
impl MidiBits {
    pub const ZERO: Self = Self { bits: 0 };
    /// `enum MidiSource source:1;`
    #[inline(always)]
    pub fn source(&self) -> u32 { self.bits & 1 }
    #[inline(always)]
    pub fn set_source(&mut self, v: u32) { self.bits = (self.bits & !1) | (v & 1) }
    /// `unsigned sysxbyte:1;`
    #[inline(always)]
    pub fn sysxbyte(&self) -> u32 { (self.bits >> 1) & 1 }
    #[inline(always)]
    pub fn set_sysxbyte(&mut self, v: u32) { self.bits = (self.bits & !2) | ((v & 1) << 1) }
    /// `unsigned length:2;`
    #[inline(always)]
    pub fn length(&self) -> u32 { (self.bits >> 2) & 3 }
    #[inline(always)]
    pub fn set_length(&mut self, v: u32) { self.bits = (self.bits & !0xc) | ((v & 3) << 2) }
}
''')
HAND_DECODE['MidiBits'] = lambda b: 'MidiBits { bits: 0x%x }' % struct.unpack('<I', b[:4])[0]

HAND['midiParser_routing_t'] = ('midi::midi_parser', '''/// C: the anonymous type of `midiParser_routing` (Core/Src/Soundengine/MIDI/MidiParser.c:44):
/// `static union { uint8_t value; struct { unsigned usb2midi:1; unsigned usb2usb:1; unsigned midi2midi:1;
/// unsigned midi2usb:1; unsigned :4; } route; } midiParser_routing;`, 4 bytes.
///
/// Representation: the union storage as a 32 bit word (`value` is its low byte); the `route` bit-fields are bits 0..3.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct midiParser_routing_t {
    pub bits: u32,
}
impl midiParser_routing_t {
    pub const ZERO: Self = Self { bits: 0 };
    /// `uint8_t value;` (read)
    #[inline(always)]
    pub fn value(&self) -> u8 { self.bits as u8 }
    /// `uint8_t value;` (write; only the low byte changes)
    #[inline(always)]
    pub fn set_value(&mut self, v: u8) { self.bits = (self.bits & !0xff) | v as u32 }
    /// `route.usb2midi`
    #[inline(always)]
    pub fn route_usb2midi(&self) -> u32 { self.bits & 1 }
    #[inline(always)]
    pub fn set_route_usb2midi(&mut self, v: u32) { self.bits = (self.bits & !1) | (v & 1) }
    /// `route.usb2usb`
    #[inline(always)]
    pub fn route_usb2usb(&self) -> u32 { (self.bits >> 1) & 1 }
    #[inline(always)]
    pub fn set_route_usb2usb(&mut self, v: u32) { self.bits = (self.bits & !2) | ((v & 1) << 1) }
    /// `route.midi2midi`
    #[inline(always)]
    pub fn route_midi2midi(&self) -> u32 { (self.bits >> 2) & 1 }
    #[inline(always)]
    pub fn set_route_midi2midi(&mut self, v: u32) { self.bits = (self.bits & !4) | ((v & 1) << 2) }
    /// `route.midi2usb`
    #[inline(always)]
    pub fn route_midi2usb(&self) -> u32 { (self.bits >> 3) & 1 }
    #[inline(always)]
    pub fn set_route_midi2usb(&mut self, v: u32) { self.bits = (self.bits & !8) | ((v & 1) << 3) }
}
''')
HAND_DECODE['midiParser_routing_t'] = lambda b: 'midiParser_routing_t { bits: 0x%x }' % struct.unpack('<I', b[:4])[0]

# anonymous types: (file, line) -> name
ANON_NAMES = {
    ('Core/Src/Soundengine/MIDI/MidiParser.c', 44): 'midiParser_routing_t',
    ('Core/Src/Soundengine/Sequencer/sequencer.c', 99): 'seq_rndValue_t',
    ('Core/Inc/Soundengine/MIDI/ParameterArray.h', 367): 'ptrValue',
    ('Core/Inc/Soundengine/Sequencer/sequencer.h', 81): 'ActiveSteps',
}

# ---- collect type definitions from all DWARF tables
class TypeDB:
    def __init__(self):
        self.structs = OrderedDict()   # rust name -> dict(def)
        self.enums = OrderedDict()     # (file,line,name) -> def
        self.typedefs = OrderedDict()  # name -> (table, key)
        self.struct_name = {}          # (file, line, tag) -> rust name
        self.tables = []

    def add_table(self, T):
        self.tables.append(T)
        # typedef -> struct naming
        for k, t in T.items():
            if t and t['tag'] == 'DW_TAG_typedef' and t.get('type') is not None:
                u = T[str(t['type'])]
                if u and u['tag'] in ('DW_TAG_structure_type', 'DW_TAG_union_type') and u.get('file'):
                    key = (u['file'], u.get('line'))
                    # first typedef wins (C code uses the typedef name)
                    self.struct_name.setdefault(key, t['name'])


TDB = TypeDB()
for cu, d in O0.items():
    TDB.add_table(d['types'])
TDB.add_table(REF_TYPES)

needed_structs = OrderedDict()  # rust name -> (T, key)
needed_enums = OrderedDict()    # enum identity -> (T, key)
needed_aliases = OrderedDict()  # alias name -> (rust type, doc, file)


def struct_rname(T, k):
    t = T[str(k)]
    key = (t.get('file'), t.get('line'))
    if key in ANON_NAMES:
        return ANON_NAMES[key]
    n = TDB.struct_name.get(key)
    if n:
        return n
    if t['name']:
        return t['name']
    return 'anon_%s_%s' % (snake(os.path.basename(t.get('file') or 'x').split('.')[0]), t.get('line'))


def enum_prim(T, t):
    if t.get('type') is not None:
        b = T[str(t['type'])]
        return PRIM.get(b['name'], 'u32')
    sz = t.get('size', 4)
    neg = any(v < 0 for _, v in t['enumerators'])
    return {1: 'i8' if neg else 'u8', 2: 'i16' if neg else 'u16', 4: 'i32' if neg else 'u32'}[sz]


def rty(T, k, need=True):
    """Rust type for a DWARF type (struct fields / globals / values). Pointers become CPtr."""
    if k is None:
        return '()'
    t = T[str(k)]
    tg = t['tag']
    if tg == 'DW_TAG_base_type':
        return PRIM[t['name']]
    if tg in ('DW_TAG_const_type', 'DW_TAG_volatile_type', 'DW_TAG_restrict_type'):
        return rty(T, t.get('type'), need)
    if tg == 'DW_TAG_typedef':
        u = T[str(t['type'])] if t.get('type') is not None else None
        if u is None:
            return '()'
        if u['tag'] in ('DW_TAG_structure_type', 'DW_TAG_union_type'):
            return rty(T, t['type'], need)
        if u['tag'] == 'DW_TAG_enumeration_type':
            if need:
                register_enum(T, t['type'], alias=t['name'], alias_file=t.get('file'))
            return t['name']
        return rty(T, t['type'], need)
    if tg in ('DW_TAG_structure_type', 'DW_TAG_union_type'):
        n = struct_rname(T, k)
        if need and n not in HAND and n not in needed_structs:
            needed_structs[n] = (T, k)
            for m in t['members']:
                rty(T, m['type'], need)
        return n
    if tg == 'DW_TAG_enumeration_type':
        if need:
            register_enum(T, k)
        if t['name']:
            return t['name']
        return enum_prim(T, t)
    if tg == 'DW_TAG_array_type':
        inner = rty(T, t['type'], need)
        for dim in reversed(t['dims']):
            inner = '[%s; %d]' % (inner, dim or 0)
        return inner
    if tg in ('DW_TAG_pointer_type', 'DW_TAG_subroutine_type'):
        return 'CPtr'
    raise Exception('type ' + tg)


def register_enum(T, k, alias=None, alias_file=None):
    t = T[str(k)]
    ident = (t.get('file'), t.get('line'), t['name'])
    if ident not in needed_enums:
        needed_enums[ident] = {'T': T, 'k': k, 'aliases': set()}
    if alias:
        needed_enums[ident]['aliases'].add((alias, alias_file))


def type_size(T, k):
    t = T[str(k)]
    tg = t['tag']
    if 'size' in t:
        return t['size']
    if tg in ('DW_TAG_const_type', 'DW_TAG_volatile_type', 'DW_TAG_typedef', 'DW_TAG_restrict_type'):
        return type_size(T, t['type'])
    if tg == 'DW_TAG_array_type':
        n = type_size(T, t['type'])
        for d in t['dims']:
            n *= (d or 0)
        return n
    if tg == 'DW_TAG_pointer_type':
        return 4
    raise Exception(tg)


def zero_expr(rt):
    if rt in ('f32', 'f64'):
        return '0.0'
    if rt == 'bool':
        return 'false'
    if rt in PRIM_SIZE or rt == 'CPtr':
        return '0'
    m = re.match(r'^\[(.*); (\d+)\]$', rt)
    if m:
        return '[%s; %s]' % (zero_expr(m.group(1)), m.group(2))
    if rt.startswith('Option<'):
        return 'None'
    if rt in needed_structs or rt in HAND:
        return rt + '::ZERO'
    return '0'


ENUM_ALIASES = {}

# ============================================================== source helpers
_src_cache = {}


def src_lines(path):
    if path not in _src_cache:
        try:
            _src_cache[path] = open(SRC + path, errors='replace').read().split('\n')
        except FileNotFoundError:
            _src_cache[path] = []
    return _src_cache[path]


def src_line(path, line):
    L = src_lines(path)
    if line and 0 < line <= len(L):
        return L[line - 1].strip()
    return ''


def c_decl_text(path, line):
    """the C declaration (signature) starting at path:line, up to the opening brace"""
    L = src_lines(path)
    out = []
    i = line - 1
    while 0 <= i < len(L) and len(out) < 12:
        s = L[i]
        s = re.sub(r'//.*$', '', s)
        if '{' in s:
            out.append(s.split('{')[0]); break
        out.append(s)
        if ')' in s and s.rstrip().endswith(')') and s.count('(') <= s.count(')') and '(' in ''.join(out):
            # signature complete
            if i + 1 < len(L) and L[i + 1].strip().startswith('{'):
                break
        i += 1
    t = ' '.join(x.strip() for x in out)
    t = re.sub(r'/\*.*?\*/', '', t)
    return re.sub(r'\s+', ' ', t).strip()


def clean_doc(s):
    s = s.replace('*/', '* /').replace('/*', '/ *')
    return s


IDENT_RE = re.compile(r'[A-Za-z_]\w*')


def orig_idents(path, a, b):
    L = src_lines(path)
    txt = '\n'.join(L[(a or 1) - 1:(b or a or 1)])
    txt = re.sub(r'//[^\n]*', '', txt)
    txt = re.sub(r'/\*.*?\*/', '', txt, flags=re.S)
    return set(IDENT_RE.findall(txt))
