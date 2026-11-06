"""Mode 8: Sequencer, and its held-step overlay."""

from surface_spec import *

from components.common import steps

LOCK_NAMES = ["Pitch", "Decay", "Cutoff", "Reso", "Drive", "Pan", "S.rate", "FX send"]


def sequencer():
    m = Layout()
    for c in range(7):
        tr = TRACKS[c]
        m.set(k(1, c + 1), f"{tr} Fills", seq_track(c), seq_fills(c))
        m.set(k(2, c + 1), f"{tr} Rotate", seq_track(c), seq_rotate(c))
        m.set(k(3, c + 1), f"{tr} Length", seq_track(c), seq_length(c))
        m.set(f(c + 1), f"{tr} Density", seq_track(c), seq_density(c))
    m.set(k(1, 8), "Shuffle", SEQ, seq_shuffle())
    m.set(k(2, 8), "Next pattern", SEQ, seq_next_pattern())
    m.set(k(3, 8), "Entry velocity", SEQ, seq_entry_velocity())
    m.set(f(8), "Density all", SEQ, seq_density_all())
    steps(m)
    return m


def step_held():
    """Laid over the sequencer while a step button is held: the step's fields and locks."""
    m = Layout()
    row1 = [("Velocity", "velocity"), ("Probability", "probability"), ("Note", "note"), ("Flam / ratchet", "flam"),
            ("Micro-shift", "shift"), ("Lock 1 value", "lock1"), ("Lock 2 value", "lock2"), ("Clear locks", "clear_locks")]
    for c, (n, fld) in enumerate(row1):
        m.set(k(1, c + 1), n, STEP, step_field(fld))
    for c, n in enumerate(LOCK_NAMES):
        m.set(k(2, c + 1), f"Lock {n}", STEP, lock_target(c))
    return m
