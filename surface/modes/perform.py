"""Mode 10: Perform."""

from surface_spec import *


def perform():
    m = Layout()
    for c in range(7):
        tr = TRACKS[c]
        m.set(k(1, c + 1), f"{tr} Pitch", perf(c), param(vid(c, "coarse")))
        m.set(k(2, c + 1), f"{tr} Decay", perf(c), param(vid(c, "decay")))
        m.set(k(3, c + 1), f"{tr} Cutoff", perf(c), param(vid(c, "cutoff")))
        m.set(f(c + 1), f"{tr} Level", perf(c), voice_fader(c))
        m.set(t(c + 1), f"{tr} Trigger", perf(c), trigger(c))
        m.set(b(c + 1), f"{tr} Mute", perf(c), mute(c))
    m.set(k(1, 8), "FX P1", FX, param("fx_p1"))
    m.set(k(2, 8), "FX P2", FX, param("fx_p2"))
    m.set(k(3, 8), "Global S.rate", FX, param("voice_decimation_all"))
    m.set(f(8), "FX dry/wet", FX, param("fx_dry_wet"))
    m.set(t(8), "FX on/off", FX, fx_on_off())
    m.set(b(8), "Mute all", perf(6), mute_all())
    return m
