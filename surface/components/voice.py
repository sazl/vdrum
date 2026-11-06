"""Components of the voice modes (1 to 7)."""

from surface_spec import *

from components.common import pads


def lfo_row(m, v):
    """Row 3: the voice's LFO."""
    keys = ["lfo_rate", "lfo_amt", "lfo_ofs", "lfo_wave", "lfo_sync", "lfo_rtg", "lfo_voice", "lfo_dest"]
    names = ["Rate", "Amount", "Offset", "Wave", "Sync", "Retrigger", "Dest voice", "Dest param"]
    for c, (n, key) in enumerate(zip(names, keys)):
        m.set(k(3, c + 1), f"LFO {n}", LFO, param(vid(v, key)))


def voice_common(m, v):
    """The columns every voice shares: osc, amp, transient, filter, velocity, mix, LFO row, pads."""
    m.set(t(1), "Osc wave", OSC, param(vid(v, "wave")))
    m.set(k(1, 1), "Fine", OSC, param(vid(v, "fine")))
    m.set(k(2, 1), "Pulse width", OSC, param(vid(v, "pwm")))
    m.set(f(1), "Coarse", OSC, param(vid(v, "coarse")))
    m.set(t(2), "Copy voice", EDIT, copy_voice())
    m.set(k(1, 2), "Attack", AMP, param(vid(v, "attack")))
    m.set(k(2, 2), "Amp slope", AMP, param(vid(v, "slope")))
    m.set(f(2), "Decay", AMP, param(vid(v, "decay")))
    m.set(t(3), "Paste voice", EDIT, paste_voice())
    m.set(t(5), "Trans wave", TRANS, param(vid(v, "trans_wave")))
    m.set(k(1, 5), "Trans freq", TRANS, param(vid(v, "trans_freq")))
    m.set(f(5), "Trans volume", TRANS, param(vid(v, "trans_vol")))
    m.set(t(6), "Filter type", FILTER, param(vid(v, "ftype")))
    m.set(k(1, 6), "Resonance", FILTER, param(vid(v, "reso")))
    m.set(k(2, 6), "Filter drive", FILTER, param(vid(v, "fdrive")))
    m.set(f(6), "Cutoff", FILTER, param(vid(v, "cutoff")))
    m.set(t(7), "Vel > Vol", VELO, param(vid(v, "velvol")))
    m.set(k(1, 7), "Vel dest", VELO, param(vid(v, "veldest")))
    m.set(k(2, 7), "Vel amount", VELO, param(vid(v, "velamt")))
    m.set(f(7), "Sample rate", MIX, param(vid(v, "srate")))
    m.set(t(8), "Audio out", MIX, param(vid(v, "out")))
    m.set(k(1, 8), "Pan", MIX, param(vid(v, "pan")))
    m.set(k(2, 8), "Overdrive", MIX, param(vid(v, "drive")))
    m.set(f(8), "Volume", MIX, param(vid(v, "vol")))
    lfo_row(m, v)
    pads(m)


def metal(m, ids):
    """Columns 3 and 4 of the cymbal and hi-hats: the two metal oscillators."""
    f1, f2, g1, g2, w1, w2 = ids
    m.set(k(1, 3), "Metal freq 1", FM, param(f1))
    m.set(k(2, 3), "Metal wave 1", FM, param(w1))
    m.set(f(3), "Metal gain 1", FM, param(g1))
    m.set(k(1, 4), "Metal freq 2", FM, param(f2))
    m.set(k(2, 4), "Metal wave 2", FM, param(w2))
    m.set(f(4), "Metal gain 2", FM, param(g2))

