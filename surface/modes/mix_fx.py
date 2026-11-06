"""Mode 9: Mix + FX."""

from surface_spec import *

MOMENT_NAMES = ["Stutter 1/16", "Stutter 1/32", "Filter sweep", "Reso up", "Swell", "Long decay", "Lo-fi", "FX throw"]


def mix_fx():
    m = Layout()
    m.set(k(1, 1), "FX P1", FX, param("fx_p1"))
    m.set(k(1, 2), "FX P2", FX, param("fx_p2"))
    m.set(k(1, 3), "FX P3", FX, param("fx_p3"))
    m.set(k(1, 4), "FX P4", FX, param("fx_p4"))
    m.set(k(1, 5), "FX type", FX, param("fx_type"))
    m.set(k(1, 6), "FX sub-type", FX, fx_sub_type())
    m.set(k(1, 7), "FX output", FX, param("fx_routing"))
    m.set(k(1, 8), "Global S.rate", GLOBAL, param("voice_decimation_all"))
    for c in range(7):
        tr = TRACKS[c]
        m.set(k(2, c + 1), f"{tr} Pan", MIX, param(vid(c, "pan")))
        m.set(k(3, c + 1), f"{tr} Drive", MIX, param(vid(c, "drive")))
        m.set(f(c + 1), f"{tr} Volume", MIX, param(vid(c, "vol")))
        m.set(t(c + 1), f"{tr} > FX", FX, to_fx(c))
    m.set(k(2, 8), "Effect depth", MOMENT, moment_depth())
    m.set(k(3, 8), "Effect rate", MOMENT, moment_rate())
    m.set(f(8), "FX dry/wet", FX, param("fx_dry_wet"))
    m.set(t(8), "FX on/off", FX, fx_on_off())
    for c, n in enumerate(MOMENT_NAMES):
        m.set(b(c + 1), n, MOMENT, moment(c))
    return m
