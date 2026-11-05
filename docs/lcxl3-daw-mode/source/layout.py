"""Proposed VDrum <-> Launch Control XL 3 DAW-mode layouts, revision 2 (single source for images and tables).

Control keys:
  K<r><c>  encoder, row r (1..3), column c (1..8)
  F<c>     fader, column c
  T<c>     top button row (DAW CC 37..44, printed 1..8 on the panel)
  B<c>     bottom button row (DAW CC 45..52, printed 9..16 on the panel)
  side:    PAGE, TRACK, REC, PLAY, SHIFT, MODE, SOLO, MUTE, LOGO

Entry: (label, detail, section, kind)
  kind: 'enc' | 'fader' | 'cycle' | 'toggle' | 'hold' | 'press' | 'nav' | 'spare'

Revision 2: closed and open hi-hat are separate voices (open hat = voice 7, new parameter IDs ending
in 7 / _oh, marked NEW_IDS); 16 mode slots; Shift + button 1..16 selects a mode.
"""

DAW_CC = {}
for c in range(1, 9):
    DAW_CC[f"K1{c}"] = 12 + c
    DAW_CC[f"K2{c}"] = 20 + c
    DAW_CC[f"K3{c}"] = 28 + c
    DAW_CC[f"F{c}"] = 4 + c
    DAW_CC[f"T{c}"] = 36 + c
    DAW_CC[f"B{c}"] = 44 + c
DAW_CC.update({"PAGE": "106/107", "TRACK": "103/102", "REC": 118, "PLAY": 116,
               "SHIFT": "63 (ch 7)", "MODE": "n/a", "SOLO": 65, "MUTE": 66, "LOGO": 104})

SECTIONS = {
    "OSC": ("Oscillator", (244, 162, 89)),
    "AMP": ("Amp EG", (91, 186, 111)),
    "MOD": ("Pitch mod EG", (242, 208, 75)),
    "FM": ("FM / noise / metal", (171, 130, 222)),
    "TRANS": ("Transient", (232, 106, 166)),
    "FILTER": ("Filter", (95, 165, 230)),
    "LFO": ("LFO", (53, 201, 194)),
    "VELO": ("Velocity mod", (214, 172, 128)),
    "MIX": ("Mix", (190, 198, 210)),
    "FX": ("FX bus", (238, 99, 82)),
    "MOMENT": ("Momentary effect (hold)", (255, 196, 64)),
    "SEQ": ("Sequencer", (140, 220, 120)),
    "STEP": ("Step parameter (while a step is held)", (250, 240, 150)),
    "MACRO": ("Macro / morph", (200, 150, 255)),
    "GLOBAL": ("Global / MIDI", (150, 160, 185)),
    "DEVICE": ("Controller setting", (120, 130, 150)),
    "PERF": ("Performance action", (255, 255, 255)),
    "PAD": ("Trigger pad", (235, 235, 235)),
    "EDIT": ("Edit action", (180, 190, 205)),
    "NAV": ("Navigation", (70, 76, 88)),
    "SPARE": ("Unassigned", (55, 58, 64)),
}

TRACKS = ["D1", "D2", "D3", "SD", "CY", "CH", "OH"]
VOICE_COLOURS = {
    "D1": (255, 120, 90), "D2": (255, 170, 70), "D3": (240, 215, 80), "SD": (120, 210, 110),
    "CY": (80, 200, 220), "CH": (110, 150, 255), "OH": (170, 120, 255), "FX": (238, 99, 82),
}
COLOURED_BY_TRACK = {"PERF", "PAD", "SEQ_T"}  # sections drawn in the column's track colour

MODE_NAMES = {1: "Drum 1", 2: "Drum 2", 3: "Drum 3", 4: "Snare", 5: "Cymbal",
              6: "Closed hi-hat", 7: "Open hi-hat", 8: "Sequencer", "8h": "Sequencer, step held",
              9: "Mix + FX", 10: "Perform", 11: "LFO + Mod", 12: "Morph + Macro", 16: "Global"}
FILE_NAMES = {1: "drum-1", 2: "drum-2", 3: "drum-3", 4: "snare", 5: "cymbal", 6: "closed-hi-hat",
              7: "open-hi-hat", 8: "sequencer", "8h": "sequencer-step-held", 9: "mix-fx",
              10: "perform", 11: "lfo-mod", 12: "morph-macro", 16: "global"}

# Parameter IDs that do not exist yet (open hi-hat voice, voice fader 7).
NEW_IDS = set()


def vid(v, kind):
    """Parameter id of `kind` for voice v (1..7). Voice 7 (open hat) ids are new."""
    tbl = {
        "coarse": "coarse{v}", "fine": "fine{v}", "pwm": "pwm_{v}", "attack": "veloa{v}",
        "slope": "vol_slope{v}", "trans_wave": "trans{v}_wave", "trans_freq": "trans{v}_freq",
        "trans_vol": "trans{v}_vol", "ftype": "filter_type_{v}", "reso": "reso_{v}",
        "fdrive": "filter_drive_{v}", "cutoff": "filter_freq_{v}", "velvol": "volume_mod_on_off{v}",
        "veldest": "vel_dest_{v}", "velamt": "velo_mod_amt_{v}", "srate": "voice_decimation{v}",
        "out": "audio_out{v}", "pan": "pan{v}", "vol": "vol{v}", "fader": "vdrum_fader_{v}",
        "lfo_rate": "freq_lfo{v}", "lfo_amt": "amount_lfo{v}", "lfo_ofs": "offset_lfo{v}",
        "lfo_wave": "wave_lfo{v}", "lfo_sync": "sync_lfo{v}", "lfo_rtg": "retrigger_lfo{v}",
        "lfo_voice": "voice_lfo{v}", "lfo_dest": "target_lfo{v}",
    }
    if kind == "drive":
        s = {1: "drive1", 2: "drive2", 3: "drive3", 4: "snare_distortion", 5: "cymbal_distortion",
             6: "hat_distortion", 7: "oh_distortion"}[v]
    elif kind == "decay":
        s = {6: "velod6_closed", 7: "velod7"}.get(v, f"velod{v}")
    elif kind == "wave":
        s = {1: "osc_wave_drum1", 2: "osc_wave_drum2", 3: "osc_wave_drum3", 4: "osc_wave_snare",
             5: "wave1_cym", 6: "wave1_hh", 7: "wave1_oh"}[v]
    else:
        s = tbl[kind].format(v=v)
    if v == 7:
        NEW_IDS.add(s)
    return s


def nav(mode):
    m = {
        "PAGE": ("Kit browse", "up / down", "NAV", "nav"),
        "TRACK": ("(unused)", "mode 8 only", "NAV", "nav"),
        "REC": ("Record arm", "pads record", "NAV", "toggle"),
        "PLAY": ("Play", "audition / load", "NAV", "press"),
        "SHIFT": ("Shift", "+ button 1-16", "NAV", "hold"),
        "MODE": ("Device mode", "handled by device", "NAV", "nav"),
        "SOLO": ("Sequencer", "mode 8", "NAV", "nav"),
        "MUTE": ("Perform", "mode 10", "NAV", "nav"),
        "LOGO": ("Back", "last mode", "NAV", "nav"),
    }
    return m


def pads(m):
    for c, t in enumerate(TRACKS, 1):
        m[f"B{c}"] = (f"{t} pad", "", "PAD", "press")
    m["B8"] = ("Accent", "pads at vel 127", "PAD", "hold")


def lfo_row(m, v):
    keys = ["lfo_rate", "lfo_amt", "lfo_ofs", "lfo_wave", "lfo_sync", "lfo_rtg", "lfo_voice", "lfo_dest"]
    names = ["Rate", "Amount", "Offset", "Wave", "Sync", "Retrigger", "Dest voice", "Dest param"]
    for c, (n, k) in enumerate(zip(names, keys), 1):
        m[f"K3{c}"] = (f"LFO {n}", vid(v, k), "LFO", "enc")


def voice_common(m, v):
    m["T1"] = ("Osc wave", vid(v, "wave"), "OSC", "cycle")
    m["K11"] = ("Fine", vid(v, "fine"), "OSC", "enc")
    m["K21"] = ("Pulse width", vid(v, "pwm"), "OSC", "enc")
    m["F1"] = ("Coarse", vid(v, "coarse"), "OSC", "fader")
    m["T2"] = ("Copy voice", "", "EDIT", "press")
    m["K12"] = ("Attack", vid(v, "attack"), "AMP", "enc")
    m["K22"] = ("Amp slope", vid(v, "slope"), "AMP", "enc")
    m["F2"] = ("Decay", vid(v, "decay"), "AMP", "fader")
    m["T3"] = ("Paste voice", "1 s", "EDIT", "hold")
    m["T5"] = ("Trans wave", vid(v, "trans_wave"), "TRANS", "cycle")
    m["K15"] = ("Trans freq", vid(v, "trans_freq"), "TRANS", "enc")
    m["F5"] = ("Trans volume", vid(v, "trans_vol"), "TRANS", "fader")
    m["T6"] = ("Filter type", vid(v, "ftype"), "FILTER", "cycle")
    m["K16"] = ("Resonance", vid(v, "reso"), "FILTER", "enc")
    m["K26"] = ("Filter drive", vid(v, "fdrive"), "FILTER", "enc")
    m["F6"] = ("Cutoff", vid(v, "cutoff"), "FILTER", "fader")
    m["T7"] = ("Vel > Vol", vid(v, "velvol"), "VELO", "toggle")
    m["K17"] = ("Vel dest", vid(v, "veldest"), "VELO", "enc")
    m["K27"] = ("Vel amount", vid(v, "velamt"), "VELO", "enc")
    m["F7"] = ("Sample rate", vid(v, "srate"), "MIX", "fader")
    m["T8"] = ("Audio out", vid(v, "out"), "MIX", "cycle")
    m["K18"] = ("Pan", vid(v, "pan"), "MIX", "enc")
    m["K28"] = ("Overdrive", vid(v, "drive"), "MIX", "enc")
    m["F8"] = ("Volume", vid(v, "vol"), "MIX", "fader")
    lfo_row(m, v)
    pads(m)


def drum_mode(v):
    m = nav(v)
    voice_common(m, v)
    m["K13"] = ("Pitch EG decay", f"mod_eg{v}", "MOD", "enc")
    m["K23"] = ("Pitch EG slope", f"pitch_slope{v}", "MOD", "enc")
    m["F3"] = ("Pitch EG amt", f"modamnt{v}", "MOD", "fader")
    m["T4"] = ("FM / Mix", f"mix_mod_{v}", "FM", "toggle")
    m["K14"] = ("FM freq", f"fm_freq{v}", "FM", "enc")
    m["K24"] = ("FM wave", f"mod_wave_drum{v}", "FM", "enc")
    m["F4"] = ("FM amount", f"fmamnt{v}", "FM", "fader")
    m["K25"] = ("(spare)", "", "SPARE", "spare")
    return m


def snare_mode():
    m = nav(4)
    voice_common(m, 4)
    m["K13"] = ("Pitch EG decay", "mod_eg4", "MOD", "enc")
    m["K23"] = ("Pitch EG slope", "pitch_slope4", "MOD", "enc")
    m["F3"] = ("Pitch EG amt", "modamnt4", "MOD", "fader")
    m["T4"] = ("(spare)", "", "SPARE", "spare")
    m["K14"] = ("Noise freq", "noise_freq1", "FM", "enc")
    m["K24"] = ("(spare)", "", "SPARE", "spare")
    m["F4"] = ("Noise/Osc mix", "mix1", "FM", "fader")
    m["K25"] = ("Amp repeat", "repeat4", "AMP", "enc")
    return m


def metal(m, ids, choke):
    f1, f2, g1, g2, w1, w2 = ids
    m["K13"] = ("Metal freq 1", f1, "FM", "enc")
    m["K23"] = ("Metal wave 1", w1, "FM", "enc")
    m["F3"] = ("Metal gain 1", g1, "FM", "fader")
    m["K14"] = ("Metal freq 2", f2, "FM", "enc")
    m["K24"] = ("Metal wave 2", w2, "FM", "enc")
    m["F4"] = ("Metal gain 2", g2, "FM", "fader")
    m["T4"] = choke


def cymbal_mode():
    m = nav(5)
    voice_common(m, 5)
    metal(m, ("mod_osc_f1_cym", "mod_osc_f2_cym", "mod_osc_gain1_cym", "mod_osc_gain2_cym",
              "wave2_cym", "wave3_cym"), ("(spare)", "", "SPARE", "spare"))
    m["K25"] = ("Amp repeat", "repeat5", "AMP", "enc")
    return m


def hat_mode(v):
    m = nav(v)
    voice_common(m, v)
    if v == 6:
        ids = ("mod_osc_f1", "mod_osc_f2", "mod_osc_gain1", "mod_osc_gain2", "wave2_hh", "wave3_hh")
        choke = ("Choke OH", "chokes open hat", "AMP", "toggle")
    else:
        ids = ("mod_osc_f1_oh", "mod_osc_f2_oh", "mod_osc_gain1_oh", "mod_osc_gain2_oh", "wave2_oh", "wave3_oh")
        NEW_IDS.update(ids)
        choke = ("Choke by CH", "same switch", "AMP", "toggle")
    metal(m, ids, choke)
    m["K25"] = ("(spare)", "", "SPARE", "spare")
    return m


def sequencer_mode():
    m = nav(8)
    m["SOLO"] = ("Group (hold)", "top row: tracks", "SEQ", "hold")
    m["MUTE"] = ("Pattern (hold)", "buttons: 1-16", "SEQ", "hold")
    m["PAGE"] = ("Bar", "1-4 up / down", "SEQ", "nav")
    m["TRACK"] = ("Track - / +", "select 1-7", "SEQ", "nav")
    m["PLAY"] = ("Start / stop", "internal clock", "SEQ", "press")
    for c, t in enumerate(TRACKS, 1):
        m[f"K1{c}"] = (f"{t} Fills", "Euclid pulses", "SEQ_T", "enc")
        m[f"K2{c}"] = (f"{t} Rotate", "Euclid offset", "SEQ_T", "enc")
        m[f"K3{c}"] = (f"{t} Length", "1-64 steps", "SEQ_T", "enc")
        m[f"F{c}"] = (f"{t} Density", "play probability", "SEQ_T", "fader")
    m["K18"] = ("Shuffle", "seq_setShuffle", "SEQ", "enc")
    m["K28"] = ("Next pattern", "queued to bar end", "SEQ", "enc")
    m["K38"] = ("Entry velocity", "new steps", "SEQ", "enc")
    m["F8"] = ("Density all", "scales all tracks", "SEQ", "fader")
    for c in range(1, 9):
        m[f"T{c}"] = (f"Step {c}", "group tracks", "SEQ", "toggle")
        m[f"B{c}"] = (f"Step {c + 8}", "group tracks", "SEQ", "toggle")
    return m


def sequencer_held_mode():
    m = sequencer_mode()
    row1 = [("Velocity", "step volume"), ("Probability", "step prob"), ("Note", "step note"),
            ("Flam / ratchet", "stepTiming flam"), ("Micro-shift", "stepTiming shift"),
            ("Lock 1 value", "param1Val"), ("Lock 2 value", "param2Val"), ("Clear locks", "turn left")]
    for c, (n, d) in enumerate(row1, 1):
        m[f"K1{c}"] = (n, d, "STEP", "enc")
    row2 = [("Lock Pitch", "coarse"), ("Lock Decay", "amp decay"), ("Lock Cutoff", "filter freq"),
            ("Lock Reso", "resonance"), ("Lock Drive", "overdrive"), ("Lock Pan", "pan"),
            ("Lock S.rate", "sample rate"), ("Lock FX send", "audio out")]
    for c, (n, d) in enumerate(row2, 1):
        m[f"K2{c}"] = (n, d, "STEP", "enc")
    for c in range(1, 9):
        m[f"T{c}"] = (f"Step {c}", "hold: select", "STEP", "hold")
        m[f"B{c}"] = (f"Step {c + 8}", "hold: select", "STEP", "hold")
    return m


def mixfx_mode():
    m = nav(9)
    fx = [("FX P1", "fx_p1"), ("FX P2", "fx_p2"), ("FX P3", "fx_p3"), ("FX P4", "fx_p4"),
          ("FX type", "fx_type"), ("FX sub-type", "per FX type"), ("FX output", "fx_routing"),
          ("Global S.rate", "voice_decimation_all")]
    for c, (n, d) in enumerate(fx, 1):
        m[f"K1{c}"] = (n, d, "FX" if c < 8 else "GLOBAL", "enc")
    for c, t in enumerate(TRACKS, 1):
        m[f"K2{c}"] = (f"{t} Pan", vid(c, "pan"), "MIX", "enc")
        m[f"K3{c}"] = (f"{t} Drive", vid(c, "drive"), "MIX", "enc")
        m[f"F{c}"] = (f"{t} Volume", vid(c, "vol"), "MIX", "fader")
        m[f"T{c}"] = (f"{t} > FX", f"audio_out = FX", "FX", "toggle")
    m["K28"] = ("Effect depth", "momentary row", "MOMENT", "enc")
    m["K38"] = ("Effect rate", "stutter / sweep", "MOMENT", "enc")
    m["F8"] = ("FX dry/wet", "fx_dry_wet", "FX", "fader")
    m["T8"] = ("FX on/off", "fx_type <-> off", "FX", "toggle")
    moments = [("Stutter 1/16", "repeat last hits"), ("Stutter 1/32", "repeat last hits"),
               ("Filter sweep", "cutoff down"), ("Reso up", "resonance max"),
               ("Swell", "attack up"), ("Long decay", "decay up"), ("Lo-fi", "sample rate down"),
               ("FX throw", "all to FX, wet")]
    for c, (n, d) in enumerate(moments, 1):
        m[f"B{c}"] = (n, d, "MOMENT", "hold")
    return m


def perform_mode():
    m = nav(10)
    for c, t in enumerate(TRACKS, 1):
        m[f"K1{c}"] = (f"{t} Pitch", vid(c, "coarse"), "PERF", "enc")
        m[f"K2{c}"] = (f"{t} Decay", vid(c, "decay"), "PERF", "enc")
        m[f"K3{c}"] = (f"{t} Cutoff", vid(c, "cutoff"), "PERF", "enc")
        m[f"F{c}"] = (f"{t} Level", vid(c, "fader"), "PERF", "fader")
        m[f"T{c}"] = (f"{t} Trigger", "", "PERF", "press")
        m[f"B{c}"] = (f"{t} Mute", "", "PERF", "toggle")
    m["K18"] = ("FX P1", "fx_p1", "FX", "enc")
    m["K28"] = ("FX P2", "fx_p2", "FX", "enc")
    m["K38"] = ("Global S.rate", "voice_decimation_all", "FX", "enc")
    m["F8"] = ("FX dry/wet", "fx_dry_wet", "FX", "fader")
    m["T8"] = ("FX on/off", "fx_type <-> off", "FX", "toggle")
    m["B8"] = ("Mute all", "", "PERF", "toggle")
    return m


def lfo_mode():
    m = nav(11)
    for c, t in enumerate(TRACKS, 1):
        m[f"K1{c}"] = (f"{t} LFO rate", vid(c, "lfo_rate"), "LFO", "enc")
        m[f"K2{c}"] = (f"{t} LFO amt", vid(c, "lfo_amt"), "LFO", "enc")
        m[f"K3{c}"] = (f"{t} LFO dest", vid(c, "lfo_dest"), "LFO", "enc")
        m[f"F{c}"] = (f"{t} Vel amt", vid(c, "velamt"), "VELO", "fader")
        m[f"T{c}"] = (f"{t} LFO wave", vid(c, "lfo_wave"), "LFO", "cycle")
        m[f"B{c}"] = (f"{t} LFO sync", vid(c, "lfo_sync"), "LFO", "cycle")
    m["K18"] = ("Rate scale", "all LFOs", "LFO", "enc")
    m["K28"] = ("Vel scale", "all voices", "VELO", "enc")
    m["K38"] = ("LFO offset all", "phase", "LFO", "enc")
    m["F8"] = ("Depth scale", "all LFOs", "LFO", "fader")
    m["T8"] = ("Sync to host", "all LFOs", "LFO", "toggle")
    m["B8"] = ("Reset phases", "all LFOs", "LFO", "press")
    return m


def morph_mode():
    m = nav(12)
    for c in range(1, 9):
        m[f"F{c}"] = (f"Macro {c}", "user assigned", "MACRO", "fader")
        m[f"K1{c}"] = (f"Macro {c} depth", "range scale", "MACRO", "enc")
        m[f"T{c}"] = (f"Snapshot {c}", "hold 1 s: store", "MACRO", "press")
        m[f"B{c}"] = (f"Learn {c}", "assign macro", "MACRO", "toggle")
    for c, t in enumerate(TRACKS, 1):
        m[f"K2{c}"] = (f"{t} Morph", "to its target", "MACRO", "enc")
        m[f"K3{c}"] = (f"{t} Target", "snapshot 1-8", "MACRO", "enc")
    m["K28"] = ("Morph all", "PAR_MORPH", "MACRO", "enc")
    m["K38"] = ("Target all", "snapshot 1-8", "MACRO", "enc")
    return m


def global_mode():
    m = nav(16)
    for c, t in enumerate(TRACKS, 1):
        m[f"K1{c}"] = (f"{t} note", f"midi_note{c}", "GLOBAL", "enc")
        m[f"K2{c}"] = (f"{t} channel", f"midi_chan_{c}", "GLOBAL", "enc")
        m[f"F{c}"] = (f"{t} HW fader", vid(c, "fader"), "MIX", "fader")
    m["K18"] = ("Tempo", "bpm", "GLOBAL", "enc")
    m["K28"] = ("Global ch.", "midi_chan_global", "GLOBAL", "enc")
    dev = [("LED bright", "device CC 111"), ("Screen bright", "device CC 112"),
           ("Popup time", "device CC 113"), ("Enc. curve", "device CC 121"),
           ("Pad velocity", "plugin setting"), ("List speed", "plugin setting"),
           ("Fader mode", "pickup / jump"), ("Clock", "host / internal")]
    for c, (n, d) in enumerate(dev, 1):
        m[f"K3{c}"] = (n, d, "DEVICE", "enc")
    m["F8"] = ("Global S.rate", "voice_decimation_all", "GLOBAL", "fader")
    tops = [("Sum aux", "vdrum_sum_aux", "toggle"), ("HW clock", "vdrum_hardware_clock", "toggle"),
            ("Init kit", "1 s", "hold"), ("Save kit", "1 s", "hold"), ("Compare", "saved kit", "hold"),
            ("Undo", "", "press"), ("Redo", "", "press"), ("Panic", "all voices off", "press")]
    for c, (n, d, k) in enumerate(tops, 1):
        m[f"T{c}"] = (n, d, "GLOBAL", k)
    pads(m)
    return m


MODES = {
    1: drum_mode(1), 2: drum_mode(2), 3: drum_mode(3), 4: snare_mode(), 5: cymbal_mode(),
    6: hat_mode(6), 7: hat_mode(7), 8: sequencer_mode(), "8h": sequencer_held_mode(),
    9: mixfx_mode(), 10: perform_mode(), 11: lfo_mode(), 12: morph_mode(), 16: global_mode(),
}
NEW_IDS.update({"oh_distortion", "vdrum_fader_7", "midi_note7_voice"})
NEW_IDS.discard("midi_note7_voice")

PERF_VOICE_OF_COL = {i + 1: t for i, t in enumerate(TRACKS)}
PERF_VOICE_OF_COL[8] = "FX"

TRACK_TITLES = ["DRUM 1", "DRUM 2", "DRUM 3", "SNARE", "CYMBAL", "CL. HAT", "OP. HAT"]
COLUMN_TITLES = {
    "drum": ["OSC", "AMP", "PITCH EG", "FM", "TRANSIENT", "FILTER", "VELOCITY", "MIX"],
    "snare": ["OSC", "AMP", "PITCH EG", "NOISE", "TRANSIENT", "FILTER", "VELOCITY", "MIX"],
    "metal": ["OSC", "AMP", "METAL 1", "METAL 2", "TRANSIENT", "FILTER", "VELOCITY", "MIX"],
    "tracks_seq": TRACK_TITLES + ["PATTERN"],
    "held": ["STEP / D1", "D2", "D3", "SD", "CY", "CH", "OH", "PATTERN"],
    "mixfx": TRACK_TITLES + ["FX / EFFECTS"],
    "tracks_fx": TRACK_TITLES + ["FX"],
    "tracks_all": TRACK_TITLES + ["ALL"],
    "macro": ["1", "2", "3", "4", "5", "6", "7", "8 / ALL"],
    "global": TRACK_TITLES + ["GLOBAL"],
}
MODE_COLS = {1: "drum", 2: "drum", 3: "drum", 4: "snare", 5: "metal", 6: "metal", 7: "metal",
             8: "tracks_seq", "8h": "tracks_seq", 9: "mixfx", 10: "tracks_fx", 11: "tracks_all",
             12: "macro", 16: "global"}
