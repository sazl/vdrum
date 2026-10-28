#!/usr/bin/env python3
"""Extract the parameter table (index, name, menu data type, value range) from the
reference ELF (DWARF + .rodata via gdb-multiarch) for the scenario generator.

usage: gen_params.py <fw.elf> <Parameters.h> <out.json>

Value ranges follow menu_encoderChangeParameter (menu.c): DTYPE_0B255 0..255,
DTYPE_0B127/DTYPE_PM63 0..127, DTYPE_1B16 1..16, DTYPE_1B64 1..64, DTYPE_0B15 0..15,
DTYPE_ON_OFF/DTYPE_0b1 0..1, DTYPE_MENU 0..entries-1 (getMaxEntriesForMenu, no SD
samples), DTYPE_VOICE_LFO 1..6, DTYPE_TARGET_SELECTION_* / DTYPE_AUTOM_TARGET
0..getNumModTargets()-1, everything else (default branch) 0..100.
"""
import json
import re
import subprocess
import sys

elf, hdr, out = sys.argv[1:4]
src = open(hdr).read().split('enum ParamEnums')[1].split('};')[0]
names = list(dict.fromkeys(re.findall(r'\b(PAR_\w+|END_OF_SOUND_PARAMETERS|NUM_PARAMS|UNUSED_PAR_GLO_SAT)\b', src)))

MENUS = {
    'MENU_TRANS': 'transientNames', 'MENU_AUDIO_OUT': 'outputNames', 'MENU_AUDIO_OUT_FX': 'outputNamesFx',
    'MENU_FILTER': 'filterTypes', 'MENU_SYNC_RATES': 'syncRateNames', 'MENU_LFO_WAVES': 'lfoWaveNames',
    'MENU_RETRIGGER': 'retriggerNames', 'MENU_FOLLOW': 'followNames', 'MENU_SEQ_QUANT': 'quantisationNames',
    'MENU_MIDI': 'midiModes', 'MENU_NEXT_PATTERN': 'nextPatternNames', 'MENU_WAVEFORM': 'waveformNames',
    'MENU_ROLL_RATES': 'rollRateNames', 'MENU_MIDI_ROUTING': 'midiRoutingNames',
    'MENU_MIDI_FILTERING': 'midiFilterNames', 'MENU_PPQ_OUT': 'ppqOutNames', 'MENU_PPQ_IN': 'ppqInNames',
    'MENU_MIX_FM': 'mixFmNames', 'MENU_FX': 'fxNames', 'MENU_DIST': 'distNames', 'MENU_RATIO': 'ratioNames',
    'MENU_DELAY': 'delayNames', 'MENU_REVERB': 'reverbNames', 'MENU_PAT_MODE': 'patModeNames',
    'MENU_STEP_FLAM': 'stepFlamNames', 'MENU_STEP_SHIFT': 'stepShiftNames',
}


def gdb(exprs):
    cmd = ['gdb-multiarch', '-batch']
    for tag, e in exprs:
        cmd += ['-ex', f'printf "%s %d\\n", "{tag}", (int)({e})']
    cmd.append(elf)
    r = subprocess.run(cmd, capture_output=True, text=True).stdout
    vals = {}
    for line in r.splitlines():
        p = line.split()
        if len(p) == 2:
            vals[p[0]] = int(p[1])
    return vals


idx = gdb([(n, n) for n in names])
num = idx['NUM_PARAMS']
dt = gdb([(f'd{i}', f'parameter_dtypes[{i}]') for i in range(num)])
dnames = ['DTYPE_0B255', 'DTYPE_0B127', 'DTYPE_PM100', 'DTYPE_MENU', 'DTYPE_PM63', 'DTYPE_1B16', 'DTYPE_ON_OFF',
          'DTYPE_TARGET_SELECTION_LFO', 'DTYPE_TARGET_SELECTION_VELO', 'DTYPE_VOICE_LFO', 'DTYPE_AUTOM_TARGET',
          'DTYPE_0b1', 'DTYPE_NOTE_NAME', 'DTYPE_0B15', 'DTYPE_1B64', 'DTYPE_0B100']
dval = gdb([(n, n) for n in dnames])
dby = {v: k for k, v in dval.items()}
# menu ids are #defines in menu/MenuText.h (next to Parameters.h)
import os  # noqa: E402
_mt = open(os.path.join(os.path.dirname(hdr), 'menu', 'MenuText.h')).read()
menu_ids = {m: int(v, 0) for m, v in re.findall(r'#define\s+(MENU_\w+)\s+(\w+)', _mt) if m in MENUS}
menu_max = gdb([(m, f'{t}[0][0]') for m, t in MENUS.items()])
menu_by_id = {v: k for k, v in menu_ids.items()}
# getNumModTargets() = sizeof(modTargets)/sizeof(ModTarg); the array type is incomplete
# in DWARF, so use the symbol size (ModTarg = {uint8_t nameIdx; uint16_t param;} = 4 bytes)
from elftools.elf.elffile import ELFFile  # noqa: E402
_st = ELFFile(open(elf, 'rb')).get_section_by_name('.symtab')
nmt = next(s['st_size'] for s in _st.iter_symbols() if s.name == 'modTargets') // 4

by_index = {}
for n in names:
    if n in ('END_OF_SOUND_PARAMETERS', 'NUM_PARAMS', 'PAR_BEGINNING_OF_GLOBALS'):
        continue
    by_index.setdefault(idx[n], []).append(n)

params = []
for i in range(num):
    raw = dt[f'd{i}'] & 0xFFFF
    t = dby.get(raw & 0xFF, f'DTYPE_{raw & 0xFF}')
    lo, hi = 0, 100
    menu = None
    if t == 'DTYPE_0B255':
        hi = 255
    elif t in ('DTYPE_0B127', 'DTYPE_PM63'):
        hi = 127
    elif t == 'DTYPE_1B16':
        lo, hi = 1, 16
    elif t == 'DTYPE_1B64':
        lo, hi = 1, 64
    elif t == 'DTYPE_0B15':
        hi = 15
    elif t in ('DTYPE_ON_OFF', 'DTYPE_0b1'):
        hi = 1
    elif t == 'DTYPE_VOICE_LFO':
        lo, hi = 1, 6
    elif t in ('DTYPE_TARGET_SELECTION_LFO', 'DTYPE_TARGET_SELECTION_VELO', 'DTYPE_AUTOM_TARGET'):
        hi = nmt - 1
    elif t == 'DTYPE_MENU':
        menu = menu_by_id.get(raw >> 8)
        hi = menu_max[menu] - 1 if menu else 0
    params.append({'index': i, 'names': by_index.get(i, []), 'dtype': t, 'menu': menu, 'min': lo, 'max': hi,
                   'sound': i < idx['END_OF_SOUND_PARAMETERS']})

json.dump({'END_OF_SOUND_PARAMETERS': idx['END_OF_SOUND_PARAMETERS'], 'NUM_PARAMS': num, 'num_mod_targets': nmt,
           'params': params}, open(out, 'w'), indent=1)
print(len(params), 'params; mod targets', nmt)
