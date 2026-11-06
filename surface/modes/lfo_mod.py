"""Mode 11: LFO + Mod."""

from surface_spec import *


def lfo_mod():
    m = Layout()
    for c in range(7):
        tr = TRACKS[c]
        m.set(k(1, c + 1), f"{tr} LFO rate", LFO, param(vid(c, "lfo_rate")))
        m.set(k(2, c + 1), f"{tr} LFO amt", LFO, param(vid(c, "lfo_amt")))
        m.set(k(3, c + 1), f"{tr} LFO dest", LFO, param(vid(c, "lfo_dest")))
        m.set(f(c + 1), f"{tr} Vel amt", VELO, param(vid(c, "velamt")))
        m.set(t(c + 1), f"{tr} LFO wave", LFO, param(vid(c, "lfo_wave")))
        m.set(b(c + 1), f"{tr} LFO sync", LFO, param(vid(c, "lfo_sync")))
    m.set(k(1, 8), "Rate scale", LFO, lfo_rate_scale())
    m.set(k(2, 8), "Vel scale", VELO, vel_scale())
    m.set(k(3, 8), "LFO offset all", LFO, lfo_offset_all())
    m.set(f(8), "Depth scale", LFO, lfo_depth_scale())
    m.set(t(8), "Sync to host", LFO, lfo_sync_host())
    m.set(b(8), "Reset phases", LFO, lfo_reset_phases())
    return m
