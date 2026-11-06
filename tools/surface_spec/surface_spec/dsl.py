"""Building blocks of a surface spec: layouts, sections, controls, the surface itself."""

from .behaviours import Do

# Sections (LED colour and popup title). Track-coloured sections take the track: perf(3).
OSC, AMP, MOD, FM, TRANS, FILTER, LFO, VELO = "osc", "amp", "mod", "fm", "trans", "filter", "lfo", "velo"
MIX, FX, MOMENT, SEQ, STEP, MACRO, GLOBAL = "mix", "fx", "moment", "seq", "step", "macro", "global"
DEVICE, EDIT, NAV, SPARE = "device", "edit", "nav", "spare"
PLAIN_SECTIONS = {OSC, AMP, MOD, FM, TRANS, FILTER, LFO, VELO, MIX, FX, MOMENT, SEQ, STEP, MACRO, GLOBAL,
                  DEVICE, EDIT, NAV, SPARE}


def seq_track(t):
    return f"seq_track:{t}"


def perf(t):
    return f"perf:{t}"


def pad_section(t):
    return f"pad:{t}"


# Controls, as printed on the panel (1-based).
def k(row, col):
    return f"K{row}{col}"


def f(col):
    return f"F{col}"


def t(col):
    return f"T{col}"


def b(col):
    return f"B{col}"


ENCODERS = [k(r, c) for r in (1, 2, 3) for c in range(1, 9)]
FADERS = [f(c) for c in range(1, 9)]
BUTTONS = [t(c) for c in range(1, 9)] + [b(c) for c in range(1, 9)]
CONTROLS = set(ENCODERS + FADERS + BUTTONS)

TRACKS = ["D1", "D2", "D3", "SD", "CY", "CH", "OH"]


class Cell:
    __slots__ = ("label", "section", "do")

    def __init__(self, label, section, do):
        self.label, self.section, self.do = label, section, do

    def to_json(self, control):
        d = {"control": control, "label": self.label, "section": self.section, "do": self.do.name}
        if self.do.arg is not None:
            d["arg"] = self.do.arg
        return d


class Layout:
    """Control -> cell, in the order cells were first set. Setting a control again replaces its
    cell in place (the same rule the Rust loader applies)."""

    def __init__(self):
        self.cells = {}

    def set(self, control, label, section, do):
        if control not in CONTROLS:
            raise ValueError(f"unknown control {control}")
        if not isinstance(do, Do):
            raise TypeError(f"{control}: behaviour must be a Do, got {do!r}")
        self.cells[control] = Cell(label, section, do)
        return self

    def use(self, component, *args):
        """Apply a component: a function that sets cells on a layout."""
        component(self, *args)
        return self

    def to_json(self):
        return [c.to_json(k_) for k_, c in self.cells.items()]


def layout(*components):
    """A layout made of components applied in order; each item is a function or (function, args...)."""
    m = Layout()
    for c in components:
        if isinstance(c, tuple):
            m.use(c[0], *c[1:])
        else:
            m.use(c)
    return m


# Mode keys the plugin knows (Rust `Mode`). Slots 1..16 are selected with Shift + button N.
MODE_KEYS = [f"voice:{i}" for i in range(7)] + ["sequencer", "mix_fx", "perform", "lfo_mod", "morph_macro", "global"]


class Surface:
    """The whole spec: modes in slots, their layouts, and the overlays."""

    def __init__(self):
        from .rules import Leds, Navigation, Screen
        self.modes = []
        self.layouts = {}
        self.overlays = {}
        self.navigation = Navigation()
        self.leds = Leds()
        self.screen = Screen()
        self.palette = None

    def mode(self, key, slot, name, layout_id, lay):
        if key not in MODE_KEYS:
            raise ValueError(f"unknown mode {key}; known: {MODE_KEYS}")
        if layout_id in self.layouts and self.layouts[layout_id] is not lay:
            raise ValueError(f"layout id {layout_id} used twice")
        self.modes.append({"mode": key, "slot": slot, "name": name, "layout": layout_id})
        self.layouts[layout_id] = lay
        return self

    def overlay(self, name, lay):
        self.overlays[name] = lay
        return self

    def to_json(self):
        return {
            "format": "vdrum-surface-spec",
            "version": 2,
            "modes": self.modes,
            "layouts": {k_: v.to_json() for k_, v in self.layouts.items()},
            "overlays": {k_: v.to_json() for k_, v in self.overlays.items()},
            "navigation": self.navigation.rules,
            "leds": self.leds.buttons,
            "slot_leds": {"when": self.leds.slot_leds},
            "screen": self.screen.rules,
            "palette": self.palette.to_json() if self.palette else {"sections": [], "tracks": [], "named": []},
        }
