"""Components shared by several modes. A component is a function that sets cells on a layout."""

from surface_spec import *


def steps(m):
    """The 16 buttons as the sequencer's step buttons of the shown bar."""
    for c in range(8):
        m.set(t(c + 1), f"Step {c + 1}", SEQ, seq_step(c))
        m.set(b(c + 1), f"Step {c + 9}", SEQ, seq_step(c + 8))


def pads(m):
    """Bottom row: trigger pads of the seven tracks, and Accent."""
    for c in range(7):
        m.set(b(c + 1), f"{TRACKS[c]} pad", pad_section(c), pad(c))
    m.set(b(8), "Accent", pad_section(6), accent())


def unassigned(m, control):
    """Leave a control without a function (LED off)."""
    m.set(control, "", SPARE, spare())
