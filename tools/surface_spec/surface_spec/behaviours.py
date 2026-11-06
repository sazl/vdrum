"""The behaviour catalogue: every `do` a cell can have, and its argument.

This mirrors `BEHAVIOURS` in crates/vdrum-surface/src/spec.rs. A behaviour is implemented in Rust
(crates/vdrum-surface/src/surface/); the spec only refers to it by name. To add one, follow
docs/lcxl3-daw-mode/surface-spec.md, "Adding a behaviour".

Argument kinds:
  None        no argument
  "param"     a parameter id (checked against param_ids.json)
  "track"     a track 0..6 (D1, D2, D3, SD, CY, CH, OH)
  int n       an index 0..n-1
  "step_field" one of STEP_FIELDS
  "setting"   one of SETTINGS
"""

STEP_FIELDS = ["velocity", "probability", "note", "flam", "shift", "lock1", "lock2", "clear_locks"]
SETTINGS = ["led_brightness", "screen_brightness", "popup_time", "encoder_curve", "pad_velocity",
            "list_speed", "fader_mode", "clock"]

BEHAVIOURS = {
    "param": "param",
    "voice_fader": "track",
    "pad": "track",
    "accent": None,
    "copy_voice": None,
    "paste_voice": None,
    "to_fx": "track",
    "fx_on_off": None,
    "fx_sub_type": None,
    "moment": 8,
    "moment_depth": None,
    "moment_rate": None,
    "seq_fills": "track",
    "seq_rotate": "track",
    "seq_length": "track",
    "seq_density": "track",
    "seq_shuffle": None,
    "seq_next_pattern": None,
    "seq_entry_velocity": None,
    "seq_density_all": None,
    "seq_step": 16,
    "step_field": "step_field",
    "lock_target": 8,
    "trigger": "track",
    "mute": "track",
    "mute_all": None,
    "lfo_rate_scale": None,
    "vel_scale": None,
    "lfo_offset_all": None,
    "lfo_depth_scale": None,
    "lfo_sync_host": None,
    "lfo_reset_phases": None,
    "macro": 8,
    "macro_depth": 8,
    "snapshot": 8,
    "learn": 8,
    "track_morph": "track",
    "track_target": "track",
    "morph_all": None,
    "target_all": None,
    "sum_aux": None,
    "hw_clock": None,
    "init_kit": None,
    "save_kit": None,
    "compare": None,
    "undo": None,
    "redo": None,
    "panic": None,
    "setting": "setting",
    "spare": None,
}


class Do:
    """A behaviour with its argument: `Do("seq_fills", 3)`."""

    __slots__ = ("name", "arg")

    def __init__(self, name, arg=None):
        self.name, self.arg = name, arg

    def __repr__(self):
        return f"Do({self.name!r}, {self.arg!r})"

    def __eq__(self, other):
        return isinstance(other, Do) and (self.name, self.arg) == (other.name, other.arg)


def _make(name):
    kind = BEHAVIOURS[name]
    if kind is None:
        def f():
            return Do(name)
    else:
        def f(arg):
            return Do(name, arg)
    f.__name__ = name
    f.__doc__ = f"Behaviour `{name}` (argument: {kind})."
    return f


# One function per behaviour: param("fine1"), seq_fills(3), accent(), ...
for _n in BEHAVIOURS:
    globals()[_n] = _make(_n)
# `macro` and others are fine as names; `compare`/`panic` shadow nothing in the spec modules.

__all__ = ["Do", "BEHAVIOURS", "STEP_FIELDS", "SETTINGS"] + list(BEHAVIOURS)
