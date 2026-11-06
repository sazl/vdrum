"""Colours: one per section (LED colour of its controls), one per track, and named colours used by
the navigation LEDs and the mode-slot display. 8 bits per component; the device gets 7 bits."""

from surface_spec.rules import Palette

SECTIONS = {
    "osc": (244, 162, 89),
    "amp": (91, 186, 111),
    "mod": (242, 208, 75),
    "fm": (171, 130, 222),
    "trans": (232, 106, 166),
    "filter": (95, 165, 230),
    "lfo": (53, 201, 194),
    "velo": (214, 172, 128),
    "mix": (190, 198, 210),
    "fx": (238, 99, 82),
    "moment": (255, 196, 64),
    "seq": (140, 220, 120),
    "step": (250, 240, 150),
    "macro": (200, 150, 255),
    "global": (150, 160, 185),
    "device": (120, 130, 150),
    "edit": (180, 190, 205),
    "nav": (70, 76, 88),
    "spare": (0, 0, 0),
}

# D1, D2, D3, SD, CY, CH, OH: the column-per-track modes, the pads, the sequencer tracks.
TRACKS = [(255, 120, 90), (255, 170, 70), (240, 215, 80), (120, 210, 110), (80, 200, 220), (110, 150, 255), (170, 120, 255)]

NAMED = {
    "off": (0, 0, 0),
    "white": (255, 255, 255),
    # Shift held: the active mode's button (pulsing) and the other modes' buttons.
    "mode_active": (255, 80, 0),
    "mode_other": (40, 40, 40),
    # Record = Clear: lit in the sequencer view, bright while held.
    "record_ready": (64, 0, 0),
    "record_held": (255, 0, 0),
    # Play: silenced (host clock), running, ready to start (mode 8, internal clock).
    "play_silent": (255, 0, 0),
    "play_running": (0, 255, 0),
    "play_ready": (0, 64, 0),
}


def palette():
    return Palette(SECTIONS, TRACKS, NAMED)
