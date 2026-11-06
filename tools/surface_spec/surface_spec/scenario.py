"""Scenarios: sequences of gestures with expectations, run by `cargo test -p vdrum-sim` and
replayable in the simulator page.

    s = Scenario("Solo/Arm toggles the sequencer view")
    s.press("SOLO").expect({"surface.seq_view": True, "leds.SOLO": [254, 254, 254]})

Keys are panel keys: K11..K38, F1..F8, T1..T8, B1..B8, SOLO, MUTE, LOGO, PAGE_UP, PAGE_DOWN,
TRACK_LEFT, TRACK_RIGHT, REC, PLAY, SHIFT. Expectations are paths into the simulator's state
(`Sim::state` in crates/vdrum-sim/src/lib.rs): `surface.mode`, `screen.lines.0`, `leds.K11`,
`seq.running`, `kit`, `cells.K11.value`, ... A path ending in `~` checks that the value's text
(lines joined with " | ") contains the expected text; for an LED, "lit" and "off" check that it is
on or off at any colour. The checks are `vdrum_sim::check` (Rust), used by the tests and the page.
"""


class Scenario:
    def __init__(self, name):
        self.name = name
        self.steps = []

    def _step(self, **cmd):
        self.steps.append(cmd)
        return self

    def press(self, key):
        """Press and release a button."""
        return self._step(cmd="press", key=key)

    def hold(self, key):
        """Press a button and keep it down."""
        return self._step(cmd="button", key=key, pressed=True)

    def release(self, key):
        return self._step(cmd="button", key=key, pressed=False)

    def with_held(self, key, *steps):
        """Hold `key` while the steps run: s.with_held("SHIFT", lambda s: s.press("T8"))."""
        self.hold(key)
        for f in steps:
            f(self)
        return self.release(key)

    def turn(self, key, detents):
        """Turn encoder K<row><col> by detents (positive = clockwise)."""
        if not (len(key) == 3 and key[0] == "K"):
            raise ValueError(f"turn: {key} is not an encoder")
        return self._step(cmd="turn", row=int(key[1]) - 1, col=int(key[2]) - 1, detents=detents)

    def fader(self, key, value):
        """Move fader F<col> to value 0..127."""
        return self._step(cmd="fader", col=int(key[1:]) - 1, value=value)

    def wait(self, ms):
        """Let time pass (hold actions, popups timing out, the sequencer clock)."""
        return self._step(cmd="advance", ms=ms)

    def expect(self, checks=None, **kw):
        c = dict(checks or {})
        c.update({k.replace("__", "."): v for k, v in kw.items()})
        self.steps.append({"expect": c})
        return self

    def to_json(self):
        return {"name": self.name, "steps": self.steps}


KEYS = ({f"K{r}{c}" for r in (1, 2, 3) for c in range(1, 9)} | {f"F{c}" for c in range(1, 9)}
        | {f"T{c}" for c in range(1, 9)} | {f"B{c}" for c in range(1, 9)}
        | {"SOLO", "MUTE", "LOGO", "PAGE_UP", "PAGE_DOWN", "TRACK_LEFT", "TRACK_RIGHT", "REC", "PLAY", "SHIFT"})


def check_scenarios(scs):
    errors = []
    names = set()
    for sc in scs:
        if sc["name"] in names:
            errors.append(f"scenario {sc['name']}: name used twice")
        names.add(sc["name"])
        for i, st in enumerate(sc["steps"]):
            if "key" in st and st["key"] not in KEYS:
                errors.append(f"scenario {sc['name']} step {i + 1}: unknown key {st['key']}")
            if st.get("cmd") in ("press", "button") and st["key"][0] in "KF":
                errors.append(f"scenario {sc['name']} step {i + 1}: {st['key']} is not a button")
            if "expect" in st and not st["expect"]:
                errors.append(f"scenario {sc['name']} step {i + 1}: empty expectation")
    return errors
