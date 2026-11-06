"""Navigation, side-button LEDs, the stationary screen and the palette.

Mirrors the catalogues in crates/vdrum-surface/src/rules.rs (BUTTONS, PREDICATES, NAV_STEPS,
SCREEN_VARS). See docs/lcxl3-daw-mode/surface-spec.md, "Navigation and state".

A navigation rule says what a button does in a state:

    nav.on("solo_arm", when=["shift"], press=[toggle_sticky()], release=[])

The first rule whose button and guards match, and which has an entry for the event, handles it.
`release=[]` handles the release by doing nothing; leaving `release` out (None) lets the release
fall through to the next rule. Guards are predicate names, `mode:<key>`, `held:<button>`, each
negated with a leading `!`.
"""

from .dsl import MODE_KEYS, PLAIN_SECTIONS

BUTTONS = ["shift", "solo_arm", "mute_select", "logo", "page_up", "page_down", "track_left",
           "track_right", "record", "play"]
ROWS = ["top", "bottom"]
PREDICATES = ["shift", "seq_view", "view_toggle", "sticky", "voice_mode", "seq_track_first",
              "seq_track_last", "seq_bar_first", "seq_bar_last", "seq_running", "seq_silent",
              "host_clock", "browsing", "step_held"]
# step name -> argument kind: None, "int" (0..16), "dir" (-1 or 1), "mode", "text3"
NAV_STEPS = {
    "shift": None,
    "toggle_sticky": None,
    "mode_slot": "int",
    "seq_button": "int",
    "clear_seq_track": None,
    "seq_track_step": "dir",
    "previous_mode": None,
    "toggle_seq_view": None,
    "message": "text3",
    "layer_begin": None,
    "mode": "mode",
    "mode_unless_layer_used": "mode",
    "kit_browse": "dir",
    "seq_bar_step": "dir",
    "play": None,
    "cell": None,
}
SCREEN_VARS = ["mode_slot", "mode_name", "kit", "last_touched", "pattern", "bar", "next_pattern",
               "silent", "seq_track_number", "seq_track_name"]

# Which state a step changes, for the diagram: region -> (step names).
CHANGES = {
    "mode": ("mode_slot", "previous_mode", "mode", "mode_unless_layer_used"),
    "seq_view": ("toggle_seq_view", "toggle_sticky"),
    "shift": ("shift",),
}


class Step(dict):
    def __init__(self, name, arg=None):
        if name not in NAV_STEPS:
            raise ValueError(f"unknown navigation step {name}")
        super().__init__({"do": name} if arg is None else {"do": name, "arg": arg})


def _maker(name):
    kind = NAV_STEPS[name]
    if kind is None:
        f = lambda: Step(name)  # noqa: E731
    elif kind == "text3":
        f = lambda title, name_, value: Step(name, [title, name_, value])  # noqa: E731
    else:
        f = lambda arg: Step(name, arg)  # noqa: E731
    f.__name__ = name
    return f


for _n in NAV_STEPS:
    globals()[_n] = _maker(_n)
# `mode` and `shift` read badly as bare names in a rule; offer both spellings.
go_mode = globals()["mode"]
shift_follows = globals()["shift"]


class Navigation:
    def __init__(self):
        self.rules = []

    def on(self, button, when=(), press=None, release=None, note=""):
        r = {"button": button, "when": list(when)}
        if press is not None:
            r["press"] = list(press)
        if release is not None:
            r["release"] = list(release)
        if note:
            r["note"] = note
        self.rules.append(r)
        return self


class Leds:
    """Side-button LEDs: for each button, rules `(guards, colour)`; the first match wins, no match
    is off. A colour is a palette name, a section name, or `mode_pad` (the current voice mode's pad
    colour)."""

    def __init__(self):
        self.buttons = []
        self.slot_leds = []

    def button(self, name, *rules):
        self.buttons.append({"button": name, "rules": [{"when": list(w), "colour": c} for w, c in rules]})
        return self

    def mode_slots_when(self, *guards):
        """While these guards hold, the 16 buttons show the mode slots."""
        self.slot_leds = list(guards)
        return self


class Screen:
    """Stationary screen templates (title, name, value); the first whose guards match is shown.
    The last must have no guards."""

    def __init__(self):
        self.rules = []

    def show(self, lines, when=()):
        if len(lines) != 3:
            raise ValueError("a screen rule has three lines (title, name, value)")
        self.rules.append({"when": list(when), "lines": list(lines)})
        return self


class Palette:
    def __init__(self, sections, tracks, named):
        self.sections = sections
        self.tracks = tracks
        self.named = named

    def to_json(self):
        return {
            "sections": [[k, list(v)] for k, v in self.sections.items()],
            "tracks": [list(c) for c in self.tracks],
            "named": [[k, list(v)] for k, v in self.named.items()],
        }


def check_guard(g):
    """None if the guard is valid, else the reason."""
    g = g[1:] if g.startswith("!") else g
    if g.startswith("mode:"):
        return None if g[5:] in MODE_KEYS else f"unknown mode {g[5:]}"
    if g.startswith("held:"):
        return None if g[5:] in BUTTONS else f"unknown button {g[5:]}"
    return None if g in PREDICATES else f"unknown predicate {g}"


def check_rules(spec):
    errors = []
    pal = spec.get("palette") or {}
    sections = {k for k, _ in pal.get("sections", [])}
    named = {k for k, _ in pal.get("named", [])}
    for s in PLAIN_SECTIONS:
        if s not in sections:
            errors.append(f"palette: section {s} has no colour")
    if len(pal.get("tracks", [])) != 7:
        errors.append("palette: tracks needs 7 colours")
    for k, v in pal.get("sections", []) + pal.get("named", []):
        if len(v) != 3 or not all(isinstance(x, int) and 0 <= x <= 255 for x in v):
            errors.append(f"palette {k}: colour must be [r, g, b] in 0..255")

    def guards(where, gs):
        for g in gs:
            e = check_guard(g)
            if e:
                errors.append(f"{where}: {e}")

    for i, r in enumerate(spec.get("navigation", [])):
        where = f"navigation rule {i + 1} ({r['button']})"
        if r["button"] not in BUTTONS + ROWS:
            errors.append(f"{where}: unknown button")
        guards(where, r.get("when", []))
        if "press" not in r and "release" not in r:
            errors.append(f"{where}: neither press nor release")
        for ev in ("press", "release"):
            for st in r.get(ev) or []:
                kind = NAV_STEPS.get(st["do"], "?")
                arg = st.get("arg")
                ok = {
                    None: arg is None,
                    "int": isinstance(arg, int) and 0 <= arg <= 16,
                    "dir": arg in (-1, 1),
                    "mode": arg in MODE_KEYS,
                    "text3": isinstance(arg, list) and len(arg) == 3 and all(isinstance(x, str) for x in arg),
                }.get(kind, False)
                if not ok:
                    errors.append(f"{where}: {ev}: {st['do']}: bad step or argument {arg!r}")
    for b in spec.get("leds", []):
        where = f"leds {b['button']}"
        if b["button"] not in BUTTONS or b["button"] == "shift":
            errors.append(f"{where}: unknown button (Shift has no LED)")
        for r in b["rules"]:
            guards(where, r["when"])
            c = r["colour"]
            if c != "mode_pad" and c not in named and c not in sections:
                errors.append(f"{where}: unknown colour {c}")
    guards("slot_leds", spec.get("slot_leds", {}).get("when", []))
    screen = spec.get("screen", [])
    for i, r in enumerate(screen):
        where = f"screen rule {i + 1}"
        guards(where, r["when"])
        import re
        for line in r["lines"]:
            for v in re.findall(r"\{([^}]*)\}", line):
                if v not in SCREEN_VARS:
                    errors.append(f"{where}: unknown variable {{{v}}}")
    if not screen or screen[-1]["when"]:
        errors.append("screen: the last rule must have no guards (the default)")
    return errors


def mermaid(spec):
    """A Mermaid stateDiagram-v2 of the navigation: the mode slots, the sequencer view and Shift,
    with the transitions the rules make."""
    names = {m["mode"]: (m["slot"], m["name"]) for m in spec["modes"]}

    def sid(key):
        return "m_" + key.replace(":", "_")

    def label(r, ev):
        g = " ".join(r.get("when", []))
        b = r["button"]
        return f"{b} {ev}" + (f" [{g}]" if g else "")

    out = ["stateDiagram-v2", "  state Mode {"]
    for key, (slot, name) in sorted(names.items(), key=lambda x: x[1][0]):
        out.append(f'    {sid(key)}: {slot} {name}')
    out.append("  }")
    out += ["  state SeqView {", "    sv_off: off", "    sv_on: on", "    sv_sticky: sticky", "  }"]
    out += ["  state ShiftKey {", "    sh_up: up", "    sh_down: held", "  }"]
    edges = []
    for r in spec.get("navigation", []):
        for ev in ("press", "release"):
            for st in r.get(ev) or []:
                d, arg = st["do"], st.get("arg")
                if d == "mode_slot":
                    edges.append(f"  Mode --> Mode: {label(r, ev)} (slot {arg} + column)")
                elif d in ("mode", "mode_unless_layer_used"):
                    edges.append(f"  Mode --> {sid(arg)}: {label(r, ev)}")
                elif d == "previous_mode":
                    edges.append(f"  Mode --> Mode: {label(r, ev)} (previous)")
                elif d == "toggle_seq_view":
                    edges.append(f"  sv_off --> sv_on: {label(r, ev)}")
                    edges.append(f"  sv_on --> sv_off: {label(r, ev)}")
                elif d == "toggle_sticky":
                    edges.append(f"  sv_off --> sv_sticky: {label(r, ev)}")
                    edges.append(f"  sv_on --> sv_sticky: {label(r, ev)}")
                    edges.append(f"  sv_sticky --> sv_on: {label(r, ev)}")
                elif d == "shift":
                    edges.append(f"  sh_up --> sh_down: {r['button']} press" if ev == "press" else f"  sh_down --> sh_up: {r['button']} release")
    seen = set()
    for e in edges:
        if e not in seen:
            seen.add(e)
            out.append(e)
    return "\n".join(out) + "\n"
