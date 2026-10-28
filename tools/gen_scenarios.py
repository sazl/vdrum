#!/usr/bin/env python3
"""Generate the scenario corpus in tests/scenarios/ (deterministic).

usage: gen_scenarios.py [tests/scenarios]

Reads tests/scenarios/params.json (tools/gen_params.py). Scenario format: see
crates/lxr-emu/src/scenario.rs. Groups:
  default_*      boot defaults (all voices on MIDI ch 1, note 48)
  param_NNN_*    one sound parameter at several values, relevant voices triggered with
                 several notes/velocities after each change
  fx_*           every FX type x FX routing with all voices on the FX bus, FX params swept
  lfo_*          every LFO: all mod targets of every target voice, all waveforms
  velmod_*       velocity modulation: all targets of each voice, several velocities
  kit_*          kit loading (random valid kits) incl. loads while voices sound
  midicc_*       DIN MIDI CC (sound parameters through the MIDI CC path), NRPN
  global_*       BPM / LFO sync, decimation, morph (no morph kit)
  fuzz_*         long seeded random event streams
"""
import json
import os
import random
import sys

OUT = sys.argv[1] if len(sys.argv) > 1 else os.path.join(os.path.dirname(__file__), '..', 'tests', 'scenarios')
P = json.load(open(os.path.join(OUT, 'params.json')))
PARAMS = P['params']
EOS = P['END_OF_SOUND_PARAMETERS']
NMT = P['num_mod_targets']
BY_NAME = {n: p for p in PARAMS for n in p['names']}
# modTargetVoiceOffsets (Cc2Text.c): valid mod target index range per voice
VOICE_TARGETS = [(2, 39), (40, 77), (78, 115), (116, 152), (153, 190), (191, 228)]
FX_BUS = 6  # MENU_AUDIO_OUT: St1 St2 L1 R1 L2 R2 FX
# DIN MIDI CC n sets sound parameter n-1. Oscillator waveform parameters above the last
# waveform select "sample playback", which (without samples) leaves the oscillator
# buffer unwritten: the output then depends on uninitialised stack memory, i.e. it is
# not deterministic on the hardware. Such values are excluded from the corpus.
WAVE_CCS = {p['index'] + 1: p['max'] for p in PARAMS if p['menu'] == 'MENU_WAVEFORM' and p['index'] < 127}


def cc_value(cc, v):
    return min(v, WAVE_CCS[cc]) if cc in WAVE_CCS else v


def pidx(name):
    return BY_NAME[name]['index']


def sp(name_or_idx, v):
    i = name_or_idx if isinstance(name_or_idx, int) else pidx(name_or_idx)
    return {'set_parameter': {'param': i, 'value': v}}


def note(voice, n, vel):
    """Note on for voice 0..6 (MIDI channel voice+1 after the prologue)."""
    return {'midi': [0x90 | voice, n, vel]}


def prologue():
    """Voice v listens on MIDI channel v+1, any note (PAR_MIDI_NOTEx = 0)."""
    ev = [sp(f'PAR_MIDI_CHAN_{v}', v) for v in range(1, 8)]
    ev += [sp(f'PAR_MIDI_NOTE{v}', 0) for v in range(1, 8)]
    return ev


class Sc:
    def __init__(self, name, desc=''):
        self.name, self.desc, self.ev, self.t = name, desc, {}, 0

    def at(self, t, *events):
        self.ev.setdefault(t, []).extend(events)

    def dump(self, ticks):
        evs = [{'tick': t, 'events': e} for t, e in sorted(self.ev.items())]
        d = {'name': self.name, 'description': self.desc, 'ticks': ticks, 'events': evs}
        with open(os.path.join(OUT, self.name + '.json'), 'w') as f:
            json.dump(d, f, separators=(',', ':'))
            f.write('\n')
        return ticks


def values_for(p, k=5):
    lo, hi = p['min'], p['max']
    if hi - lo + 1 <= k + 2:
        return list(range(lo, hi + 1))
    vs = sorted({lo, hi, lo + (hi - lo) // 2, lo + (hi - lo) // 4, lo + 3 * (hi - lo) // 4, lo + 1})
    return vs


def voices_of(p):
    """Voices (0..6) a parameter belongs to; None = all."""
    n = p['names'][0] if p['names'] else ''
    if 'LFO' in n or n.startswith('PAR_FX') or 'DECIMATION_ALL' in n or 'MORPH' in n:
        return None
    for key, v in (('DRUM1', [0]), ('DRUM2', [1]), ('DRUM3', [2]), ('SNARE', [3]), ('CYM', [4]), ('HH', [5, 6]),
                   ('HAT', [5, 6]), ('NOISE_FREQ1', [3]), ('MIX1', [3])):
        if key in n:
            return v
    if n in ('PAR_MOD_OSC_F1', 'PAR_MOD_OSC_F2', 'PAR_MOD_OSC_GAIN1', 'PAR_MOD_OSC_GAIN2'):
        return [5, 6]
    if n.startswith('PAR_VELOD6'):
        return [5] if 'CLOSED' in n else [6]
    digits = [c for c in n if c.isdigit()]
    if digits:
        k = int(digits[-1])
        if n.startswith('PAR_TRANS'):
            k = int(digits[0])
        return {1: [0], 2: [1], 3: [2], 4: [3], 5: [4], 6: [5, 6]}.get(k)
    return None


def gen():
    random.seed(0x1a2b3c)
    total = 0
    count = 0
    # ---- default boot ----
    s = Sc('default_boot_note48', 'boot defaults: one note 48 on ch1 triggers all 7 voices; then other notes')
    s.at(2, note(0, 48, 100))
    s.at(150, note(0, 48, 30))
    s.at(300, note(0, 60, 127), note(1, 48, 127))
    total += s.dump(450)
    count += 1
    s = Sc('default_boot_silence', 'boot defaults, no input')
    total += s.dump(64)
    count += 1

    # ---- per-parameter sweeps ----
    vels = [127, 90, 40, 1]
    for p in PARAMS:
        if not p['sound']:
            continue
        i = p['index']
        name = p['names'][0] if p['names'] else f'P{i}'
        vs = voices_of(p)
        s = Sc(f'param_{i:03d}_{name[4:].lower()}', f'{name}: values {values_for(p)}')
        s.at(0, *prologue())
        if p['dtype'] == 'DTYPE_TARGET_SELECTION_LFO':
            # LFO k targets its voice's own range; make it audible
            k = int(name[-1])
            s.at(0, sp(f'PAR_AMOUNT_LFO{k}', 127), sp(f'PAR_FREQ_LFO{k}', 100))
        if p['dtype'] == 'DTYPE_TARGET_SELECTION_VELO':
            k = int(name[-1])
            s.at(0, sp(f'PAR_VELO_MOD_AMT_{k}', 127))
        if name.startswith('PAR_FX') and name != 'PAR_FX_TYPE':
            s.at(0, *[sp(f'PAR_AUDIO_OUT{k}', FX_BUS) for k in range(1, 7)], sp('PAR_FX_TYPE', 1 + (i % 4)))
        t = 1
        vals = values_for(p)
        if p['dtype'] in ('DTYPE_TARGET_SELECTION_VELO', 'DTYPE_TARGET_SELECTION_LFO'):
            k = int(name[-1]) - 1
            lo, hi = VOICE_TARGETS[k]
            vals = sorted({0, lo, hi, (lo + hi) // 2, lo + 3, hi - 5})
        for j, v in enumerate(vals):
            s.at(t, sp(i, v))
            trig = vs if vs is not None else list(range(7))
            for q, vo in enumerate(trig):
                s.at(t + (q % 4), note(vo, 36 + 7 * ((j + q) % 5), vels[(j + q) % 4]))
                s.at(t + 18 + (q % 4), note(vo, 48 + 5 * ((j + q) % 4), vels[(j + q + 2) % 4]))
            t += 40
        total += s.dump(t + 20)
        count += 1

    # ---- FX: type x routing ----
    fx_types = range(BY_NAME['PAR_FX_TYPE']['max'] + 1)
    fx_routes = range(BY_NAME['PAR_FX_ROUTING']['max'] + 1)
    for ft in fx_types:
        for fr in fx_routes:
            s = Sc(f'fx_type{ft}_route{fr}', f'FX type {ft}, routing {fr}, all voices on the FX bus, P1..P4 swept')
            s.at(0, *prologue(), *[sp(f'PAR_AUDIO_OUT{k}', FX_BUS) for k in range(1, 7)],
                 sp('PAR_FX_TYPE', ft), sp('PAR_FX_ROUTING', fr), sp('PAR_FX_DRY_WET', 100))
            t = 1
            for j in range(4):
                s.at(t, sp('PAR_FX_P1', [0, 127, 64, 20][j]), sp('PAR_FX_P2', [127, 0, 90, 40][j]),
                     sp('PAR_FX_P3', [64, 10, 127, 0][j]), sp('PAR_FX_P4', [30, 127, 0, 100][j]),
                     sp('PAR_FX_DIST_TYPE', j % 3), sp('PAR_FX_RATIO', j % 6), sp('PAR_FX_WAVE', j % 6),
                     sp('PAR_FX_DELAY_TYPE', j % 2))
                for vo in range(7):
                    s.at(t + vo * 3, note(vo, 40 + 3 * vo, 127 - 20 * j))
                t += 50
            total += s.dump(t + 100)
            count += 1

    # ---- LFOs: all targets ----
    for k in range(1, 7):
        s = Sc(f'lfo{k}_all_targets', f'LFO {k}: every target voice and every mod target of it, all waveforms')
        s.at(0, *prologue(), sp(f'PAR_AMOUNT_LFO{k}', 127), sp(f'PAR_FREQ_LFO{k}', 110))
        t = 1
        n = 0
        for tv in range(1, 7):
            s.at(t, sp(f'PAR_VOICE_LFO{k}', tv))
            lo, hi = VOICE_TARGETS[tv - 1]
            for tg in range(lo, hi + 1):
                s.at(t, sp(f'PAR_TARGET_LFO{k}', tg), sp(f'PAR_WAVE_LFO{k}', n % 8),
                     sp(f'PAR_RETRIGGER_LFO{k}', n % 7), sp(f'PAR_SYNC_LFO{k}', (n // 3) % 12),
                     sp(f'PAR_OFFSET_LFO{k}', (n * 37) % 128))
                for vo in ([tv - 1] if tv < 6 else [5, 6]):
                    s.at(t, note(vo, 45 + n % 12, 100))
                n += 1
                t += 6
        total += s.dump(t + 30)
        count += 1

    # ---- velocity modulation ----
    for k in range(1, 7):
        s = Sc(f'velmod{k}_all_targets', f'velocity modulation of voice {k}: every target, several velocities')
        s.at(0, *prologue(), sp(f'PAR_VELO_MOD_AMT_{k}', 127))
        lo, hi = VOICE_TARGETS[k - 1]
        t = 1
        vo = k - 1 if k < 6 else 5
        for tg in [0] + list(range(lo, hi + 1)):
            s.at(t, sp(f'PAR_VEL_DEST_{k}', tg), note(vo, 50, 127))
            s.at(t + 5, note(vo, 50, 30))
            if k == 6:
                s.at(t + 2, note(6, 52, 64))
            t += 10
        s.at(t, sp(f'PAR_VELO_MOD_AMT_{k}', 40))
        for j, vel in enumerate([1, 20, 64, 100, 127]):
            s.at(t + 8 * j, note(vo, 50, vel))
        total += s.dump(t + 60)
        count += 1

    # ---- kits ----
    def random_kit(rng):
        kit = bytearray(b'RND' + bytes([48 + rng.randrange(10) for _ in range(5)]))
        for p in PARAMS[:EOS]:
            v = rng.randint(p['min'], p['max'])
            if p['dtype'] == 'DTYPE_TARGET_SELECTION_VELO':
                lo, hi = VOICE_TARGETS[int(p['names'][0][-1]) - 1]
                v = rng.choice([0, rng.randint(lo, hi)])
            kit.append(v)
        # LFO targets must be in the range of the LFO's target voice
        for k in range(1, 7):
            tv = kit[8 + pidx(f'PAR_VOICE_LFO{k}')]
            lo, hi = VOICE_TARGETS[tv - 1]
            kit[8 + pidx(f'PAR_TARGET_LFO{k}')] = rng.choice([0, rng.randint(lo, hi)])
        # keep volumes audible
        for k in range(1, 7):
            kit[8 + pidx(f'PAR_VOL{k}')] = rng.randint(60, 127)
        return bytes(kit)

    rng = random.Random(7)
    for n in range(6):
        s = Sc(f'kit_random_{n}', 'load random valid kits and play all voices; kit loads while voices sound')
        s.at(0, *prologue())
        t = 1
        for j in range(4):
            s.at(t, {'load_kit': {'hex': random_kit(rng).hex()}})
            for vo in range(7):
                s.at(t + 2 * vo, note(vo, 36 + rng.randrange(36), rng.randint(1, 127)))
            t += 70
            # reload in the middle of sounding notes
            s.at(t - 30, {'load_kit': {'hex': random_kit(rng).hex()}})
        total += s.dump(t + 40)
        count += 1
    s = Sc('kit_short_file', 'kit file shorter than END_OF_SOUND_PARAMETERS (zero padded) and invalid mod targets')
    s.at(0, *prologue())
    kit = bytearray(random_kit(random.Random(99)))
    kit[8 + pidx('PAR_VEL_DEST_1')] = 250
    kit[8 + pidx('PAR_TARGET_LFO2')] = 240
    s.at(1, {'load_kit': {'hex': bytes(kit[:8 + 200]).hex()}})
    for vo in range(7):
        s.at(3 + 3 * vo, note(vo, 48, 100))
    s.at(100, {'load_kit': {'hex': bytes(kit).hex()}})
    for vo in range(7):
        s.at(102 + 3 * vo, note(vo, 55, 70))
    total += s.dump(200)
    count += 1

    # ---- DIN MIDI CC path ----
    for part in range(4):
        s = Sc(f'midicc_sweep_{part}', 'MIDI CCs 1..127 on the voice channels (midiParser_ccHandler via DIN)')
        s.at(0, *prologue())
        t = 1
        for cc in range(1 + part * 32, min(128, 1 + (part + 1) * 32)):
            if cc in (0, 32, 98, 99, 6, 38, 120, 121, 123):
                continue
            for j, v in enumerate([0, 64, 127]):
                ch = (cc + j) % 7
                s.at(t, {'midi': [0xB0 | ch, cc, cc_value(cc, v)]}, note(ch, 40 + j * 5, 110))
                t += 8
        total += s.dump(t + 20)
        count += 1
    s = Sc('midi_running_status_nrpn', 'running status, note off / velocity 0, NRPN, program change, system bytes')
    s.at(0, *prologue())
    s.at(1, {'midi': [0x90, 48, 100, 50, 90, 52, 80]})  # running status
    s.at(10, {'midi': [0x80, 48, 0, 0x90, 50, 0]})  # note off / zero velocity
    s.at(20, {'midi': [0xB0, 99, 0, 0xB0, 98, 37, 0xB0, 6, 100]})  # NRPN
    s.at(30, {'midi': [0xF8, 0x91, 0xFE, 60, 0xF8, 120]})  # realtime inside a message
    s.at(40, {'midi': [0xC0, 3]})  # program change
    s.at(50, {'midi': [0xF0, 0x7D, 1, 2, 3, 0xF7, 0x92, 40, 127]})  # sysex then note
    s.at(60, {'midi': [0xB3, 1, 127, 0x93, 45, 127]})  # mod wheel
    total += s.dump(200)
    count += 1

    # ---- globals ----
    s = Sc('global_bpm_lfo_sync', 'LFOs synced to the internal clock at several BPMs')
    s.at(0, *prologue())
    t = 1
    for j, bpm in enumerate([60, 120, 175, 240]):
        s.at(t, sp('PAR_BPM', bpm))
        for k in range(1, 7):
            s.at(t, sp(f'PAR_SYNC_LFO{k}', 1 + (j * 3 + k) % 11), sp(f'PAR_AMOUNT_LFO{k}', 120))
            s.at(t + k, note(k - 1, 50, 100))
        t += 150
    total += s.dump(t)
    count += 1
    s = Sc('global_decimation', 'per-voice and global sample-rate decimation')
    s.at(0, *prologue())
    t = 1
    for j, v in enumerate([127, 64, 10, 0]):
        s.at(t, sp('PAR_VOICE_DECIMATION_ALL', v), *[sp(f'PAR_VOICE_DECIMATION{k}', (v + 30 * k) % 128)
                                                     for k in range(1, 7)])
        for vo in range(7):
            s.at(t + vo, note(vo, 48, 120))
        t += 60
    total += s.dump(t)
    count += 1
    s = Sc('global_morph_no_target', 'PAR_MORPH without a morph kit: morphParameters is NULL (reads address 0)')
    s.at(0, *prologue())
    s.at(1, sp('PAR_MORPH', 0))
    for vo in range(7):
        s.at(2 + vo, note(vo, 48, 100))
    s.at(60, sp('PAR_MORPH', 128))
    for vo in range(7):
        s.at(61 + vo, note(vo, 48, 100))
    s.at(120, sp('PAR_MORPH', 255))
    for vo in range(7):
        s.at(121 + vo, note(vo, 48, 100))
    total += s.dump(180)
    count += 1
    s = Sc('global_faders', 'fader positions (ADC) per voice')
    s.at(0, *prologue())
    t = 1
    for j, adc in enumerate([0xFFF0, 0x8000, 0x1000, 0x0000, 0xC350]):
        for i in range(6):
            s.at(t, {'fader': {'index': i, 'adc': (adc + 0x1110 * i) & 0xFFF0}})
        for vo in range(7):
            s.at(t + vo, note(vo, 48, 120))
        t += 40
    total += s.dump(t)
    count += 1

    # ---- fuzz ----
    sound = [p for p in PARAMS if p['sound']]
    for n in range(4):
        r = random.Random(1000 + n)
        s = Sc(f'fuzz_{n}', f'random events, seed {1000 + n}')
        s.at(0, *prologue())
        ticks = 3000
        for t in range(1, ticks - 1):
            if r.random() < 0.15:
                s.at(t, note(r.randrange(7), r.randrange(24, 96), r.randint(1, 127)))
            if r.random() < 0.12:
                p = r.choice(sound)
                v = r.randint(p['min'], p['max'])
                if p['dtype'] in ('DTYPE_TARGET_SELECTION_VELO', 'DTYPE_TARGET_SELECTION_LFO'):
                    lo, hi = VOICE_TARGETS[r.randrange(6)]
                    v = r.choice([0, r.randint(lo, hi)])
                s.at(t, sp(p['index'], v))
            if r.random() < 0.03:
                cc = r.randrange(1, 120)
                s.at(t, {'midi': [0xB0 | r.randrange(7), cc, cc_value(cc, r.randrange(128))]})
            if r.random() < 0.002:
                s.at(t, {'load_kit': {'hex': random_kit(r).hex()}})
        total += s.dump(ticks)
        count += 1
    print(f'{count} scenarios, {total} ticks')


if __name__ == '__main__':
    gen()
