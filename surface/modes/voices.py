"""Modes 1 to 7: one voice each."""

from surface_spec import *

from components.common import unassigned
from components.voice import metal, voice_common


def drum(t0):
    """Drum 1..3 (track 0..2)."""
    v = t0 + 1
    m = layout((voice_common, t0))
    m.set(k(1, 3), "Pitch EG decay", MOD, param(f"mod_eg{v}"))
    m.set(k(2, 3), "Pitch EG slope", MOD, param(f"pitch_slope{v}"))
    m.set(f(3), "Pitch EG amt", MOD, param(f"modamnt{v}"))
    m.set(t(4), "FM / Mix", FM, param(f"mix_mod_{v}"))
    m.set(k(1, 4), "FM freq", FM, param(f"fm_freq{v}"))
    m.set(k(2, 4), "FM wave", FM, param(f"mod_wave_drum{v}"))
    m.set(f(4), "FM amount", FM, param(f"fmamnt{v}"))
    unassigned(m, k(2, 5))
    return m


def snare():
    m = layout((voice_common, 3))
    m.set(k(1, 3), "Pitch EG decay", MOD, param("mod_eg4"))
    m.set(k(2, 3), "Pitch EG slope", MOD, param("pitch_slope4"))
    m.set(f(3), "Pitch EG amt", MOD, param("modamnt4"))
    unassigned(m, t(4))
    m.set(k(1, 4), "Noise freq", FM, param("noise_freq1"))
    unassigned(m, k(2, 4))
    m.set(f(4), "Noise/Osc mix", FM, param("mix1"))
    m.set(k(2, 5), "Amp repeat", AMP, param("repeat4"))
    return m


def cymbal():
    m = layout((voice_common, 4))
    metal(m, ["mod_osc_f1_cym", "mod_osc_f2_cym", "mod_osc_gain1_cym", "mod_osc_gain2_cym", "wave2_cym", "wave3_cym"])
    unassigned(m, t(4))
    m.set(k(2, 5), "Amp repeat", AMP, param("repeat5"))
    return m


def closed_hat():
    m = layout((voice_common, 5))
    metal(m, ["mod_osc_f1", "mod_osc_f2", "mod_osc_gain1", "mod_osc_gain2", "wave2_hh", "wave3_hh"])
    m.set(t(4), "Choke OH", AMP, param("hat_choke"))
    unassigned(m, k(2, 5))
    return m


def open_hat():
    m = layout((voice_common, 6))
    metal(m, ["mod_osc_f1_oh", "mod_osc_f2_oh", "mod_osc_gain1_oh", "mod_osc_gain2_oh", "wave2_oh", "wave3_oh"])
    m.set(t(4), "Choke by CH", AMP, param("hat_choke"))
    unassigned(m, k(2, 5))
    return m
