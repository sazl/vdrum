"""Mode 12: Morph + Macro."""

from surface_spec import *


def morph_macro():
    m = Layout()
    for c in range(8):
        m.set(f(c + 1), f"Macro {c + 1}", MACRO, macro(c))
        m.set(k(1, c + 1), f"Macro {c + 1} depth", MACRO, macro_depth(c))
        m.set(t(c + 1), f"Snapshot {c + 1}", MACRO, snapshot(c))
        m.set(b(c + 1), f"Learn {c + 1}", MACRO, learn(c))
    for c in range(7):
        tr = TRACKS[c]
        m.set(k(2, c + 1), f"{tr} Morph", MACRO, track_morph(c))
        m.set(k(3, c + 1), f"{tr} Target", MACRO, track_target(c))
    m.set(k(2, 8), "Morph all", MACRO, morph_all())
    m.set(k(3, 8), "Target all", MACRO, target_all())
    return m
