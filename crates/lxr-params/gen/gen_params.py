#!/usr/bin/env python3
# SPDX-License-Identifier: GPL-3.0-only
"""Generate crates/lxr-params/src/generated.rs from the LXR firmware source.

Everything in the generated file is read from the firmware C source; nothing is retyped:

* Core/Inc/Parameters.h          enum ParamEnums (indices, names, END_OF_SOUND_PARAMETERS)
* Core/Inc/menu/menu.h           NamesEnum, shortNamesEnum, catNamesEnum, longNamesEnum, Datatypes
* Core/Inc/menu/MenuText.h       shortNames / catNames / longNames texts, MENU_* lists and their texts
* Core/Src/menu/menu.c           valueNames[] and parameter_dtypes[]
* Core/Inc/menu/menuPages.h      menuPages[] (which parameter is shown with which text on which page)
* Core/Src/menu/Cc2Text.c        modTargets[], modTargetVoiceOffsets[]
* Core/Src/Preset/initKit.c      initKit[] (default values)
* Core/Src/Soundengine/MIDI/MidiParser.c   midiParser_cc2voiceNr(), cases handled by midiParser_ccHandler()
* Core/Inc/Soundengine/MIDI/MidiMessages.h CC / CC2 enums

The only additions made here (and marked as such in the output) are
* the voice/section names given to the menu pages (taken from the comments in menuPages.h),
* spelled-out versions of the 8-character LCD texts for the readable `name` field (EXPAND below),
* stable host parameter IDs (lowercase enum name without the PAR_ prefix).

Usage: gen_params.py <firmware root (contains Core/)> <output .rs>
"""
import re
import sys
from pathlib import Path

ROOT = Path(sys.argv[1])
OUT = Path(sys.argv[2])
CORE = ROOT / "Core"


def read(p):
    return (CORE / p).read_text(encoding="latin-1")


def strip_comments(s):
    """Remove C comments in one left-to-right pass (string literals are kept intact)."""
    tok = re.compile(r'"(?:[^"\\\n]|\\.)*"|/\*.*?\*/|//[^\n]*', re.S)
    return tok.sub(lambda m: m.group(0) if m.group(0).startswith('"') else " ", s)


def preprocess(s, defines):
    """Minimal #if/#else/#endif evaluation (only `#if NAME` / `#if 0|1`)."""
    out = []
    stack = []  # list of (active, taken)
    for line in s.split("\n"):
        st = line.strip()
        m = re.match(r"#\s*if\s+(\w+)", st)
        if m:
            tok = m.group(1)
            val = int(tok) if tok.isdigit() else int(defines.get(tok, 0))
            parent = all(a for a, _ in stack)
            stack.append((parent and bool(val), bool(val)))
            continue
        if re.match(r"#\s*ifn?def\b", st):
            parent = all(a for a, _ in stack)
            stack.append((parent, True))
            continue
        if re.match(r"#\s*else\b", st):
            a, taken = stack.pop()
            parent = all(x for x, _ in stack)
            stack.append((parent and not taken, True))
            continue
        if re.match(r"#\s*endif\b", st):
            stack.pop()
            continue
        if all(a for a, _ in stack):
            out.append(line)
    return "\n".join(out)


DEFINES = {}  # USE_CODEC_EQ is not defined anywhere in the firmware -> 0


def parse_enum(text, enum_name, known=None):
    """Parse `enum <enum_name> { ... }` (or `typedef enum { ... } enum_name`)."""
    t = strip_comments(preprocess(text, DEFINES))
    m = re.search(r"enum\s+" + enum_name + r"\s*\{(.*?)\}", t, re.S)
    if m:
        body = m.group(1)
    else:
        end = re.search(r"\}\s*" + enum_name + r"\s*;", t)
        assert end, enum_name
        start = [x.end() for x in re.finditer(r"typedef\s+enum\s*\{", t[: end.start()])][-1]
        body = t[start: end.start()]
    values = {}
    order = []
    cur = -1
    env = dict(known or {})
    for item in body.split(","):
        item = item.strip()
        if not item:
            continue
        if "=" in item:
            name, expr = [x.strip() for x in item.split("=", 1)]
            expr = expr.strip()
            if re.fullmatch(r"0x[0-9a-fA-F]+", expr):
                cur = int(expr, 16)
            elif re.fullmatch(r"\d+", expr):
                cur = int(expr)
            else:
                cur = env[expr] if expr in env else values[expr]
        else:
            name = item
            cur += 1
        values[name] = cur
        order.append((name, cur))
    return values, order


def c_array_body(text, decl_regex):
    t = strip_comments(preprocess(text, DEFINES))
    m = re.search(decl_regex + r"\s*=\s*\{", t, re.S)
    assert m, decl_regex
    i = m.end()
    depth = 1
    j = i
    while depth:
        if t[j] == "{":
            depth += 1
        elif t[j] == "}":
            depth -= 1
        j += 1
    return t[i:j - 1]


def string_list(text, name):
    """const char name[][N] = { {count}?, {"txt"}, ... } -> (count or None, [texts])"""
    body = c_array_body(text, r"const\s+char\s+" + name + r"\s*\[\s*\]\s*\[\s*\d+\s*\]")
    items = re.findall(r"\{\s*(\"(?:[^\"\\]|\\.)*\"|\d+)\s*\}", body)
    count = None
    texts = []
    for it in items:
        if it.startswith('"'):
            texts.append(it[1:-1])
        else:
            assert count is None and not texts
            count = int(it)
    return count, texts


def split_top(body):
    """split a brace initializer body into top level items (as strings)"""
    items, depth, cur = [], 0, ""
    for ch in body:
        if ch == "{":
            depth += 1
            if depth == 1:
                cur = ""
                continue
        if ch == "}":
            depth -= 1
            if depth == 0:
                items.append(cur)
                cur = ""
                continue
        if depth >= 1:
            cur += ch
    return items


# ---------------------------------------------------------------------------- Parameters.h
par_vals, par_order = parse_enum(read("Inc/Parameters.h"), "ParamEnums")
END = par_vals["END_OF_SOUND_PARAMETERS"]
NUM_PARAMS = par_vals["NUM_PARAMS"]
# canonical name per index: the later declared alias wins (PAR_OSC_WAVE_DRUM1 over PAR_MOD_WHEEL,
# PAR_ROLL over END_OF_SOUND_PARAMETERS, PAR_BPM over PAR_BEGINNING_OF_GLOBALS)
par_name = {}
for name, v in par_order:
    if name in ("END_OF_SOUND_PARAMETERS", "NUM_PARAMS", "PAR_BEGINNING_OF_GLOBALS"):
        continue
    par_name[v] = name
assert sorted(par_name) == list(range(NUM_PARAMS)), "ParamEnums not contiguous"

# ---------------------------------------------------------------------------- menu.h enums
menu_h = read("Inc/menu/menu.h")
text_vals, text_order = parse_enum(menu_h, "NamesEnum")
short_vals, short_order = parse_enum(menu_h, "shortNamesEnum")
cat_vals, cat_order = parse_enum(menu_h, "catNamesEnum")
long_vals, long_order = parse_enum(menu_h, "longNamesEnum")
dtype_vals, dtype_order = parse_enum(menu_h, "Datatypes")
page_vals, page_order = parse_enum(menu_h, "PageNames")
NUM_NAMES = text_vals["NUM_NAMES"]

menutext_h = read("Inc/menu/MenuText.h")
menu_ids = {m.group(1): int(m.group(2)) for m in re.finditer(r"#define\s+(MENU_\w+)\s+(\d+)", menutext_h)}
_, SHORT_NAMES = string_list(menutext_h, "shortNames")
_, CAT_NAMES = string_list(menutext_h, "catNames")
_, LONG_NAMES = string_list(menutext_h, "longNames")
assert len(SHORT_NAMES) == len(short_order), (len(SHORT_NAMES), len(short_order))
assert len(CAT_NAMES) == len(cat_order)
assert len(LONG_NAMES) == len(long_order)

menu_c = read("Src/menu/menu.c")

# getMaxEntriesForMenu / getMenuItemNameForValue: which list belongs to which MENU_ id
menu_list_of = {}
# MENU_WAVEFORM returns waveformNames[0][0] + menu_numSamples (user samples in flash; VDrum has none)
for m in re.finditer(r"case\s+(MENU_\w+)\s*:\s*return\s+(?:\(uint8_t\)\s*\()?(\w+)\[0\]\[0\]", strip_comments(menu_c)):
    menu_list_of[m.group(1)] = m.group(2)
MENU_LISTS = {}
for menu_name, arr in menu_list_of.items():
    count, texts = string_list(menutext_h, arr)
    # getMaxEntriesForMenu returns the count entry, not the number of texts (rollRateNames has 6
    # as count but 13 texts): the count is what limits the value.
    MENU_LISTS[menu_name] = (arr, count, texts)
assert set(MENU_LISTS) == set(menu_ids), set(menu_ids) ^ set(MENU_LISTS)

# valueNames
vn_body = c_array_body(menu_c, r"const\s+Name\s+valueNames\s*\[\s*NUM_NAMES\s*\]")
VALUE_NAMES = []
for it in split_top(vn_body):
    a, b, c = [x.strip() for x in it.split(",")]
    VALUE_NAMES.append((short_vals[a], cat_vals[b], long_vals[c]))
assert len(VALUE_NAMES) == NUM_NAMES, (len(VALUE_NAMES), NUM_NAMES)

# parameter_dtypes
dt_body = c_array_body(menu_c, r"const\s+enum\s+Datatypes\s+PROGMEM\s+parameter_dtypes\s*\[\s*NUM_PARAMS\s*\]")
DTYPES = []
for it in [x.strip() for x in dt_body.split(",") if x.strip()]:
    m = re.fullmatch(r"(DTYPE_\w+)\s*(?:\|\s*\(\s*(MENU_\w+)\s*<<\s*8\s*\))?", it)
    assert m, it
    DTYPES.append((m.group(1), m.group(2)))
assert len(DTYPES) == NUM_PARAMS, (len(DTYPES), NUM_PARAMS)

# ---------------------------------------------------------------------------- menuPages.h
mp = read("Inc/menu/menuPages.h")
mp_body = c_array_body(mp, r"const\s+Page\s+menuPages\s*\[\s*NUM_PAGES\s*\]\s*\[\s*NUM_SUB_PAGES\s*\]")
PAGES = []
for page in split_top(mp_body):
    subs = []
    for sub in split_top(page):
        toks = [x.strip() for x in sub.split(",") if x.strip()]
        top = [text_vals[t] for t in toks[:8]]
        bot = [par_vals[t] for t in toks[8:16]]
        top += [0] * (8 - len(top))
        bot += [0] * (8 - len(bot))
        subs.append((top, bot))
    PAGES.append(subs)

# voices and sections of the pages (names from the comments in menuPages.h)
VOICE_OF_PAGE = {
    page_vals["VOICE1_PAGE"]: "Drum1",
    page_vals["VOICE2_PAGE"]: "Drum2",
    page_vals["VOICE3_PAGE"]: "Drum3",
    page_vals["VOICE4_PAGE"]: "Snare",
    page_vals["VOICE5_PAGE"]: "Cymbal",
    page_vals["VOICE6_PAGE"]: "HiHat",
    page_vals["VOICE7_PAGE"]: "HiHat",   # "Voice 6 - HH/Cymbal (FM) - same as voice 6 -> open hi hat"
    page_vals["MENU_MIDI_PAGE"]: "Global",
    page_vals["PERF_PAGE"]: "Global",
    page_vals["FX_MODE_PAGE"]: "Fx",
}
VOICE_SECTIONS = ["Oscillator", "AmpEg", "Modulation", "Fm", "Transient", "Filter", "Lfo", "Mix"]
FX_SECTIONS = ["FxBus", "FxDistortion", "FxRing", "FxCompressor", "FxDelay", "FxReverb"]

first_seen = {}  # param -> (page, sub, slot, text)
for pi, subs in enumerate(PAGES):
    if pi not in VOICE_OF_PAGE:
        continue
    for si, (top, bot) in enumerate(subs):
        for slot in range(8):
            p = bot[slot]
            if p == 0:
                continue
            if p not in first_seen:
                first_seen[p] = (pi, si, slot, top[slot])

# FX parameter labels per FX type (sub page index == fxBus.fxType)
fx_page = PAGES[page_vals["FX_MODE_PAGE"]]
FX_LABELS = []  # per fx sub page: list of (param, text)
for si, (top, bot) in enumerate(fx_page):
    FX_LABELS.append([(bot[k], top[k]) for k in range(8) if bot[k] != 0])

# ---------------------------------------------------------------------------- Cc2Text.c
cc2t = read("Src/menu/Cc2Text.c")
mt_body = c_array_body(cc2t, r"const\s+ModTarg\s+modTargets\s*\[\s*\]")
MOD_TARGETS = []
for it in split_top(mt_body):
    a, b = [x.strip() for x in it.split(",")]
    MOD_TARGETS.append((text_vals[a], par_vals[b]))
off_body = c_array_body(cc2t, r"const\s+ModTargetVoiceOffset\s+modTargetVoiceOffsets\s*\[\s*6\s*\]")
MT_OFFSETS = [tuple(int(x) for x in it.split(",")) for it in split_top(off_body)]
assert len(MT_OFFSETS) == 6

# ---------------------------------------------------------------------------- initKit.c
ik = read("Src/Preset/initKit.c")
ik_body = c_array_body(ik, r"const\s+uint8_t\s+initKit\s*\[\s*INIT_KIT_SIZE\s*\]")
INIT_KIT = [int(x, 16) for x in re.findall(r"0x[0-9a-fA-F]+", ik_body)]
ik_size = int(re.search(r"#define\s+INIT_KIT_SIZE\s+(\d+)", read("Inc/Preset/InitKit.h")).group(1))
assert len(INIT_KIT) == ik_size

# ---------------------------------------------------------------------------- MIDI
mp_c = read("Src/Soundengine/MIDI/MidiParser.c")
mm_h = read("Inc/Soundengine/MIDI/MidiMessages.h")
cc_vals, cc_order = parse_enum(mm_h, "ParamEnumsDefinition")
cc2_vals, cc2_order = parse_enum(mm_h, "Param2EnumsDefinition")
cc_name_of = {}
for n, v in cc_order:
    cc_name_of.setdefault(v, n)
cc2_name_of = {}
for n, v in cc2_order:
    cc2_name_of.setdefault(v, n)

# midiParser_cc2voiceNr
f = strip_comments(mp_c)
m = re.search(r"uint8_t\s+midiParser_cc2voiceNr\s*\(\s*uint8_t\s+cc\s*\)\s*\{(.*?)\n\}", f, re.S)
CC2VOICE = {}
pending = []
default_voice = None
for tok in re.findall(r"(default\s*:|case\s+\d+\s*:|return\s+\d+\s*;)", m.group(1)):
    if tok.startswith("default"):
        pending.append("default")
    elif tok.startswith("case"):
        pending.append(int(re.search(r"\d+", tok).group(0)))
    else:
        v = int(re.search(r"\d+", tok).group(0))
        for c in pending:
            if c == "default":
                default_voice = v
            else:
                CC2VOICE[c] = v
        pending = []
CC2VOICE_TABLE = [CC2VOICE.get(c, default_voice) for c in range(128)]

# cases handled in midiParser_ccHandler (MIDI_CC part and MIDI_CC2 part)
cc_part, cc2_part = re.split(r"\}\s*//msg.status == MIDI_CC\s*\n\s*else", mp_c[mp_c.index("void midiParser_ccHandler"):], maxsplit=1)
cc2_part = cc2_part[: cc2_part.index("void midiParser_checkMtc")]


def handled_cases(src):
    """case labels whose case body does something other than an immediate `break`"""
    s = strip_comments(src)
    res = set()
    labels = []
    for m in re.finditer(r"case\s+(\w+)\s*:|(\bbreak\s*;)|(\breturn\b)|([^\s])", s):
        if m.group(1):
            labels.append(m.group(1))
        elif m.group(2) or m.group(3):
            labels = []
        elif m.group(4):
            if labels:
                res.update(labels)
                labels = []
    return res


CC_HANDLED = handled_cases(cc_part)
CC2_HANDLED = handled_cases(cc2_part)

# ---------------------------------------------------------------------------- assemble
EXPAND = {
    # VDrum's spelled-out versions of the firmware's 8 character LCD texts (catNames/longNames)
    "Oscilltr": "Oscillator", "Veloc EG": "Amp EG", "Mod EG": "Mod EG", "PitchMod": "Pitch Mod",
    "Nois/Osc": "Noise/Osc", "Mod Osc": "Mod Osc", "Transnt": "Transient", "Parametr": "Parameter",
    "Sequencr": "Sequencer", "Generatr": "Generator", "Fx": "FX",
    "Frequncy": "Frequency", "Overdriv": "Overdrive", "RepeatCt": "Repeat Count",
    "Resnance": "Resonance", "Waveform": "Waveform", "DstParam": "Destination",
    "ClockSnc": "Clock Sync", "Retriggr": "Retrigger", "DstVoice": "Destination Voice",
    "Dcy Clsd": "Decay Closed", "Dcy Open": "Decay Open", "RollRate": "Roll Rate",
    "Prbablty": "Probability", "AudioOut": "Audio Out", "SampleRt": "Sample Rate",
    "NextPatt": "Next Pattern", "Vol mod": "Volume Mod", "AutTrack": "Automation Track",
    "Aut Dest": "Automation Destination", "AutValue": "Automation Value", "Screensv": "Screensaver",
    "X Positn": "X Position", "Y Positn": "Y Position", "Freqcy 1": "Frequency 1",
    "Freqcy 2": "Frequency 2", "PulsWdth": "Pulse Width", "Threshld": "Threshold",
    "PreDelay": "Pre-Delay", "Reflxion": "Reflection", "KitChMde": "Kit Change Mode",
    "In PPQ": "In PPQ", "Out1 PPQ": "Out 1 PPQ", "Out2 PPQ": "Out 2 PPQ", "GateMode": "Gate Mode",
    "PCReset": "PC Reset", "Panning": "Pan",
}
VOICE_LABEL = {"Drum1": "Drum 1", "Drum2": "Drum 2", "Drum3": "Drum 3", "Snare": "Snare",
               "Cymbal": "Cymbal", "HiHat": "Hi-hat", "Fx": "FX", "Global": "Global", "None": ""}
SECTION_LABEL = {"Oscillator": "Oscillator", "AmpEg": "Amp EG", "Modulation": "Modulation", "Fm": "FM",
                 "Transient": "Transient", "Filter": "Filter", "Lfo": "LFO", "Mix": "Mix",
                 "FxBus": "FX Bus", "FxDistortion": "Distortion", "FxRing": "Ring Mod",
                 "FxCompressor": "Compressor", "FxDelay": "Delay", "FxReverb": "Reverb",
                 "Performance": "Performance", "Midi": "MIDI", "None": ""}


def exp(s):
    return EXPAND.get(s, s)


GLOBAL_KEEP = ["PAR_BPM"] + [f"PAR_MIDI_CHAN_{i}" for i in range(1, 8)] + ["PAR_MIDI_CHAN_GLOBAL"] + \
              [f"PAR_MIDI_NOTE{i}" for i in range(1, 8)]

rows = []
for p in list(range(END)) + [par_vals[n] for n in GLOBAL_KEEP]:
    ename = par_name[p]
    dt, menu = DTYPES[p]
    text = None
    voice, section = "None", "None"
    if p in first_seen:
        pi, si, slot, text = first_seen[p]
        voice = VOICE_OF_PAGE[pi]
        if voice == "Fx":
            section = FX_SECTIONS[si]
        elif pi == page_vals["MENU_MIDI_PAGE"]:
            section = "Midi"
        elif pi == page_vals["PERF_PAGE"]:
            section = "Performance"
        else:
            section = VOICE_SECTIONS[si]
    # FX P1..P4 are shared by all FX types: use the generic text of modTargets ("Param 1") and
    # put them on the FX bus section; the per-type meaning is in FX_PARAM_TEXTS.
    if ename in ("PAR_FX_P1", "PAR_FX_P2", "PAR_FX_P3", "PAR_FX_P4"):
        text = [t for (t, q) in MOD_TARGETS if q == p][0]
        section = "FxBus"
    kind = "Sound"
    if p >= END:
        # global settings (MIDI channels/notes appear on the voice MIX pages, BPM on the global page)
        kind = "Global"
        voice = "Global"
        section = "Midi"
    elif text is None:
        # not shown on any menu page: NRPN/bank-select placeholders, PAR_UNUSED_x, PAR_MORPH_KIT_DEST
        kind = "Unused"
    rows.append(dict(p=p, ename=ename, dt=dt, menu=menu, text=text, voice=voice, section=section, kind=kind))

# global MIDI channel / note params appear on the voice mix pages: record the track (voice 1..7;
# track 6 = closed hi-hat on VOICE6_PAGE, track 7 = open hi-hat on VOICE7_PAGE)
TRACK_LABEL = {0: "Drum 1", 1: "Drum 2", 2: "Drum 3", 3: "Snare", 4: "Cymbal", 5: "Closed Hi-hat",
               6: "Open Hi-hat"}
for r in rows:
    if r["kind"] == "Global" and r["p"] in first_seen:
        pi = first_seen[r["p"]][0]
        if pi <= page_vals["VOICE7_PAGE"]:
            r["track"] = pi - page_vals["VOICE1_PAGE"]


def join_words(parts):
    words = []
    for part in parts:
        for wd in part.split():
            if not words or words[-1] != wd:
                words.append(wd)
    return " ".join(words)


# readable names, disambiguated
for r in rows:
    if r["text"] is None:
        r["short"], r["cat"], r["long"] = "", "", ""
        r["name"] = r["ename"]
        continue
    s, c, l = VALUE_NAMES[r["text"]]
    r["short"], r["cat"], r["long"] = SHORT_NAMES[s], CAT_NAMES[c], LONG_NAMES[l]
for r in rows:
    if r["text"] is None:
        continue
    if r["kind"] == "Global":
        if "track" in r:
            r["name"] = join_words([TRACK_LABEL[r["track"]], "MIDI", exp(r["long"])])
        else:
            r["name"] = join_words(["Global", exp(r["cat"]), exp(r["long"])])
        continue
    v = VOICE_LABEL[r["voice"]]
    cat = exp(r["cat"])
    if cat == "Oscillator" and r["section"] in VOICE_SECTIONS and r["section"] != "Oscillator":
        cat = SECTION_LABEL[r["section"]]
    r["name_a"] = join_words([v, cat, exp(r["long"])])
    r["name_b"] = join_words([v, SECTION_LABEL[r["section"]], exp(r["long"])])
from collections import Counter
ca = Counter(r["name_a"] for r in rows if "name_a" in r)
for r in rows:
    if "name_a" in r:
        r["name"] = r["name_a"] if ca[r["name_a"]] == 1 else r["name_b"]
cb = Counter(r["name"] for r in rows)
seen = Counter()
for r in rows:
    if cb[r["name"]] > 1:
        seen[r["name"]] += 1
        r["name"] = r["name"] + " " + str(seen[r["name"]])
assert len(set(r["name"] for r in rows)) == len(rows)


def rng(r):
    dt, menu, p = r["dt"], r["menu"], r["p"]
    if dt in ("DTYPE_0B127", "DTYPE_PM63", "DTYPE_NOTE_NAME"):
        return 0, 127
    if dt == "DTYPE_0B100":
        return 0, 100
    if dt == "DTYPE_0B255":
        return 0, 255
    if dt == "DTYPE_1B16":
        return 1, 16
    if dt == "DTYPE_1B64":
        return 1, 64
    if dt == "DTYPE_0B15":
        return 0, 15
    if dt in ("DTYPE_ON_OFF", "DTYPE_0b1"):
        return 0, 1
    if dt == "DTYPE_VOICE_LFO":
        return 1, 6
    if dt in ("DTYPE_TARGET_SELECTION_VELO", "DTYPE_TARGET_SELECTION_LFO", "DTYPE_AUTOM_TARGET"):
        return 0, len(MOD_TARGETS) - 1
    if dt == "DTYPE_MENU":
        _, count, _ = MENU_LISTS[menu]
        return 0, count - 1
    raise Exception(dt)


GLOBAL_DEFAULTS = {}
# preset_loadDefaultGlobals() in presetManager.c
pm = strip_comments(read("Src/Preset/presetManager.c"))
gd = re.search(r"void\s+preset_loadDefaultGlobals\s*\(\s*\)\s*\{(.*?)\n\}", pm, re.S).group(1)
for m in re.finditer(r"activeParameters\[(\w+)\]\s*=\s*(\d+)\s*;", gd):
    GLOBAL_DEFAULTS[par_vals[m.group(1)]] = int(m.group(2))


def default(r):
    p = r["p"]
    if p >= END:
        return GLOBAL_DEFAULTS.get(p, 0)
    # preset_init(): para[i-8] = initKit[i] for i in 8..END_OF_SOUND_PARAMETERS-1
    if p + 8 < END and p + 8 < len(INIT_KIT):
        return INIT_KIT[p + 8]
    return 0


def midi(r):
    p, en = r["p"], r["ename"]
    cc, nrpn, handled, chan = "None", "None", "false", 0
    if r["kind"] == "Global":
        return cc, nrpn, handled, chan
    if 1 <= p <= 126 and en not in ("PAR_NRPN_DATA_ENTRY_COARSE", "PAR_NRPN_FINE", "PAR_NRPN_COARSE"):
        # frontPanel_ccHandler: parNr = data1 - 1; midiParser_ccHandler switches on data1
        c = p + 1
        cc = f"Some({c})"
        chan = CC2VOICE_TABLE[c]
        handled = "true" if cc_name_of.get(c) in CC_HANDLED else "false"
    elif 128 <= p < END:
        # NRPN number n -> MIDI_CC2 with data1 = n, paramNr = n + 128 (frontParser_parseNrpn)
        n = p - 128
        nrpn = f"Some({n})"
        handled = "true" if cc2_name_of.get(n) in CC2_HANDLED else "false"
        # cross-check the CC2 enum against the parameter enum where the names agree
        c2 = cc2_name_of.get(n)
        if c2 and c2.replace("CC2_", "PAR_") == en:
            pass
    return cc, nrpn, handled, chan


# check CC2 enum vs ParamEnums where names correspond
for n, v in cc2_order:
    pn = n.replace("CC2_", "PAR_")
    if pn in par_vals and par_vals[pn] < END:
        assert par_vals[pn] - 128 == v, (n, v, pn, par_vals[pn])


def rs_str(s):
    return '"' + s.replace("\\", "\\\\").replace('"', '\\"') + '"'


def dtype_rs(dt):
    return {
        "DTYPE_0B255": "Dtype::U0to255", "DTYPE_0B127": "Dtype::U0to127", "DTYPE_PM100": "Dtype::Pm100",
        "DTYPE_MENU": "Dtype::Menu", "DTYPE_PM63": "Dtype::Pm63", "DTYPE_1B16": "Dtype::U1to16",
        "DTYPE_ON_OFF": "Dtype::OnOff", "DTYPE_TARGET_SELECTION_LFO": "Dtype::TargetSelectionLfo",
        "DTYPE_TARGET_SELECTION_VELO": "Dtype::TargetSelectionVelo", "DTYPE_VOICE_LFO": "Dtype::VoiceLfo",
        "DTYPE_AUTOM_TARGET": "Dtype::AutomTarget", "DTYPE_0b1": "Dtype::U0to1",
        "DTYPE_NOTE_NAME": "Dtype::NoteName", "DTYPE_0B15": "Dtype::U0to15", "DTYPE_1B64": "Dtype::U1to64",
        "DTYPE_0B100": "Dtype::U0to100",
    }[dt]


def camel(menu):
    return "".join(w.capitalize() for w in menu[len("MENU_"):].split("_"))


out = []
w = out.append
w("// @generated by crates/lxr-params/gen/gen_params.py from the LXR firmware source. Do not edit.")
w("// SPDX-License-Identifier: GPL-3.0-only")
w("// Derived from the GPL-3.0 LXR firmware (Copyright (C) 2012-2026 Erica Synths).")
w("#![allow(clippy::all)]")
w("use crate::*;")
w("")
w(f"/// `END_OF_SOUND_PARAMETERS` of `enum ParamEnums` (Parameters.h): number of parameters stored in a kit.")
w(f"pub const END_OF_SOUND_PARAMETERS: u16 = {END};")
w(f"/// `NUM_PARAMS` of `enum ParamEnums`.")
w(f"pub const NUM_PARAMS: u16 = {NUM_PARAMS};")
w(f"/// `NUM_NAMES` of `enum NamesEnum` (menu.h).")
w(f"pub const NUM_NAMES: usize = {NUM_NAMES};")
w(f"/// `INIT_KIT_SIZE` (InitKit.h).")
w(f"pub const INIT_KIT_SIZE: usize = {len(INIT_KIT)};")
w("")
w("/// Every name of `enum ParamEnums` with its value, in declaration order (aliases included).")
w("pub static PARAM_ENUM: &[(&str, u16)] = &[")
for n, v in par_order:
    w(f"    ({rs_str(n)}, {v}),")
w("];")
w("")
w("/// `ParamEnums` constants (canonical names without the `PAR_` prefix).")
w("pub mod par {")
for v in range(NUM_PARAMS):
    n = par_name[v]
    rn = n[4:] if n.startswith("PAR_") else n
    w(f"    pub const {rn}: u16 = {v};")
w("}")
w("")
w("/// `shortNames[]` (MenuText.h), indexed by `enum shortNamesEnum`.")
w("pub static SHORT_NAMES: &[&str] = &[" + ", ".join(rs_str(s) for s in SHORT_NAMES) + "];")
w("/// `catNames[]` (MenuText.h), indexed by `enum catNamesEnum`.")
w("pub static CAT_NAMES: &[&str] = &[" + ", ".join(rs_str(s) for s in CAT_NAMES) + "];")
w("/// `longNames[]` (MenuText.h), indexed by `enum longNamesEnum`.")
w("pub static LONG_NAMES: &[&str] = &[" + ", ".join(rs_str(s) for s in LONG_NAMES) + "];")
w("/// `valueNames[]` (menu.c): (shortName, category, longName) per `enum NamesEnum` (TEXT_*).")
w(f"pub static VALUE_NAMES: [(u8, u8, u8); {NUM_NAMES}] = [")
for i, (a, b, c) in enumerate(VALUE_NAMES):
    tn = [n for n, v in text_order if v == i][0]
    w(f"    ({a}, {b}, {c}), // {tn}")
w("];")
w("")
w("/// The `MENU_*` value lists of MenuText.h used by `DTYPE_MENU` parameters.")
w("#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]")
w("pub enum MenuList {")
for mname in sorted(MENU_LISTS, key=lambda k: menu_ids[k]):
    w(f"    /// `{mname}` ({MENU_LISTS[mname][0]})")
    w(f"    {camel(mname)} = {menu_ids[mname]},")
w("}")
w("")
w("impl MenuList {")
w("    /// Number of selectable entries (`getMaxEntriesForMenu`, the count stored in entry 0).")
w("    pub const fn count(self) -> u8 {")
w("        match self {")
for mname in MENU_LISTS:
    w(f"            MenuList::{camel(mname)} => {MENU_LISTS[mname][1]},")
w("        }")
w("    }")
w("    /// The texts of the list (entries 1.. of the C array), as stored in the firmware.")
w("    pub const fn texts(self) -> &'static [&'static str] {")
w("        match self {")
for mname in MENU_LISTS:
    w(f"            MenuList::{camel(mname)} => &[" + ", ".join(rs_str(t) for t in MENU_LISTS[mname][2]) + "],")
w("        }")
w("    }")
w("    /// The firmware's C array name.")
w("    pub const fn c_name(self) -> &'static str {")
w("        match self {")
for mname in MENU_LISTS:
    w(f"            MenuList::{camel(mname)} => {rs_str(MENU_LISTS[mname][0])},")
w("        }")
w("    }")
w("}")
w("")
w("/// `modTargets[]` (Cc2Text.c): (TEXT_* name index, parameter number).")
w(f"pub static MOD_TARGETS: [(u8, u16); {len(MOD_TARGETS)}] = [")
for i, (t, p) in enumerate(MOD_TARGETS):
    w(f"    ({t}, {p}), // {i}: {par_name[p]}")
w("];")
w("/// `modTargetVoiceOffsets[]` (Cc2Text.c): first and last `modTargets` index of each voice.")
w("pub static MOD_TARGET_VOICE_OFFSETS: [(u8, u8); 6] = [" + ", ".join(f"({a}, {b})" for a, b in MT_OFFSETS) + "];")
w("")
w("/// `initKit[]` (initKit.c): 8 byte kit name followed by parameter values.")
w(f"pub static INIT_KIT: [u8; {len(INIT_KIT)}] = [" + ", ".join(str(b) for b in INIT_KIT) + "];")
w("")
w("/// `midiParser_cc2voiceNr()` (MidiParser.c) for every CC number: 0 = no voice (any channel that")
w("/// is set for a voice or as global channel), 1..=6 = `midi_MidiChannels[n]` or the global channel,")
w("/// 7 = all voices (any set channel).")
w("pub static CC_TO_VOICE: [u8; 128] = [" + ", ".join(str(v) for v in CC2VOICE_TABLE) + "];")
w("")
w("/// FX page (menuPages.h, FX_MODE_PAGE): the (parameter, TEXT_*) pairs shown for each FX type")
w("/// (sub page index == `PAR_FX_TYPE` value).")
w("pub static FX_PAGE: &[&[(u16, u8)]] = &[")
for lab in FX_LABELS:
    w("    &[" + ", ".join(f"({p}, {t})" for p, t in lab) + "],")
w("];")
w("")
w("/// The menu pages of the six voices (menuPages.h, VOICE1_PAGE..VOICE7_PAGE): for every sub page the")
w("/// (TEXT_*, parameter) pairs in display order (empty slots and TEXT_SKIP removed).")
w("pub static VOICE_PAGES: &[&[&[(u8, u16)]]] = &[")
for pi in range(page_vals["VOICE1_PAGE"], page_vals["VOICE7_PAGE"] + 1):
    w("    &[")
    for top, bot in PAGES[pi]:
        w("        &[" + ", ".join(f"({t}, {b})" for t, b in zip(top, bot) if b != 0) + "],")
    w("    ],")
w("];")
w("")


def info_rs(r):
    lo, hi = rng(r)
    d = default(r)
    # a default outside the encoder range (PAR_MIDI_CHAN_GLOBAL = 0 = no global channel) stays representable
    lo, hi = min(lo, d), max(hi, d)
    cc, nrpn, handled, chan = midi(r)
    menu = f"Some(MenuList::{camel(r['menu'])})" if r["menu"] else "None"
    text = f"Some({r['text']})" if r["text"] is not None else "None"
    # frontParser_parseNrpn(): PAR_TARGET_LFOx / PAR_VEL_DEST_x NRPN values are parameter numbers that the
    # front panel converts (paramToModTarget) and sends to the engine as CC_LFO_TARGET / CC_VELO_TARGET
    front = "true" if (par_vals["PAR_TARGET_LFO1"] <= r["p"] <= par_vals["PAR_TARGET_LFO6"]
                       or par_vals["PAR_VEL_DEST_1"] <= r["p"] <= par_vals["PAR_VEL_DEST_6"]) else "false"
    track = f"Some({r['track']})" if "track" in r else "None"
    ident = r["ename"][4:].lower() if r["ename"].startswith("PAR_") else r["ename"].lower()
    return (
        f"    ParamInfo {{ index: {r['p']}, enum_name: {rs_str(r['ename'])}, id: {rs_str(ident)}, "
        f"kind: ParamKind::{r['kind']}, voice: Voice::{r['voice']}, section: Section::{r['section']}, "
        f"text: {text}, short_name: {rs_str(r['short'])}, category: {rs_str(r['cat'])}, long_name: {rs_str(r['long'])}, "
        f"name: {rs_str(r['name'])}, description: \"\", dtype: {dtype_rs(r['dt'])}, menu: {menu}, "
        f"min: {lo}, max: {hi}, default: {d}, "
        f"midi: MidiMapping {{ cc: {cc}, cc_voice: {chan}, nrpn: {nrpn}, engine_handles: {handled}, "
        f"front_translates_nrpn: {front} }}, track: {track} }},"
    )


sound = [r for r in rows if r["p"] < END]
glob = [r for r in rows if r["p"] >= END]
w(f"/// All kit (sound) parameters, indexed by their `ParamEnums` value (0..END_OF_SOUND_PARAMETERS).")
w(f"pub static PARAMS: [ParamInfo; {len(sound)}] = [")
for r in sound:
    w(info_rs(r))
w("];")
w("")
w("/// Global settings that change how the engine reacts to MIDI or the clock (not stored in a kit).")
w(f"pub static GLOBAL_PARAMS: [ParamInfo; {len(glob)}] = [")
for r in glob:
    w(info_rs(r))
w("];")
OUT.write_text("\n".join(out) + "\n")
print(f"END_OF_SOUND_PARAMETERS={END} NUM_PARAMS={NUM_PARAMS} sound={len(sound)} "
      f"unused={sum(1 for r in sound if r['kind']=='Unused')} globals={len(glob)} modTargets={len(MOD_TARGETS)}")
