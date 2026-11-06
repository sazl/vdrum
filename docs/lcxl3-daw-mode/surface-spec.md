# Surface spec: authoring the LCXL3 modes in Python

Status (2026-11-06): step 1 of the plan in `declarative-modes-brainstorm.md`. The mode layouts and
the two overlays (sequencer view, held step) come from the spec. Navigation, LED rules and screen
rules are still Rust code in `crates/vdrum-surface/src/surface/` and move into the spec in later
steps.

## Files

| Path | What it is |
|---|---|
| `surface/surface.py` | The composition: which mode sits in which slot, and the overlays. Start here. |
| `surface/modes/*.py` | One file per mode or group of modes; each function returns a `Layout`. |
| `surface/components/*.py` | Reusable pieces (`voice_common`, `lfo_row`, `metal`, `pads`, `steps`, `unassigned`). |
| `tools/surface_spec/run.py` | The tool: `build`, `check`, `watch`, `svg`. |
| `tools/surface_spec/surface_spec/` | The Python library: `dsl.py` (layouts, sections, controls), `behaviours.py` (catalogue), `voice.py` (`vid`), `build.py` (load, check, write), `hardware.py` and `device_svg.py` (device drawing). |
| `tools/surface_spec/surface_spec/param_ids.json` | Every parameter id, generated from `lxr-params`. |
| `crates/vdrum-surface/spec/surface.json` | The built spec. Generated; commit it with the sources. |
| `crates/vdrum-surface/src/spec.rs` | The Rust loader, the behaviour catalogue and the parity tests. |

## Workflow

```sh
python3 tools/surface_spec/run.py watch     # rebuilds surface.json on every save
python3 tools/surface_spec/run.py build     # once
python3 tools/surface_spec/run.py check     # spec valid and surface.json up to date (before committing)
cargo test -p vdrum-surface                 # the Rust loader accepts it; parity and behaviour tests
```

The plugin embeds `surface.json` at compile time (`include_str!`), so a change reaches the plugin
when it is rebuilt. Hot reload and the browser simulator are later steps.

Errors name the layout and the control, for example
`error: layout closed-hi-hat T1: param: bad argument 'wave1_hx' (expected param)`.

## The spec

A layout maps controls to cells. A cell has a label (shown on the screen), a section (LED colour
and popup title) and a behaviour with its argument.

```python
from surface_spec import *

def my_mode():
    m = Layout()
    m.set(k(1, 1), "Fine", OSC, param("fine1"))       # encoder row 1, column 1
    m.set(f(1), "D1 Level", perf(0), voice_fader(0))  # fader 1, in track 1's colour
    m.set(t(1), "D1 Trigger", perf(0), trigger(0))    # top button 1
    m.set(b(8), "Accent", pad_section(6), accent())   # bottom button 8
    return m
```

- Controls: `k(row, col)` encoders (rows 1 to 3), `f(col)` faders, `t(col)` top buttons, `b(col)`
  bottom buttons; columns 1 to 8. The side buttons (Shift, Page, Track, Record, Play, Solo/Arm,
  Mute/Select, the button right of Solo/Arm) are navigation and stay in Rust for now.
- Sections: `OSC AMP MOD FM TRANS FILTER LFO VELO MIX FX MOMENT SEQ STEP MACRO GLOBAL DEVICE EDIT
  NAV SPARE`, and per track `seq_track(t)`, `perf(t)`, `pad_section(t)` with t = 0..6.
- Tracks are 0..6: D1, D2, D3, SD, CY, CH, OH (`TRACKS`). `vid(t, kind)` gives a voice's parameter
  id, as `voice_param` in Rust (`vid(3, "cutoff")` is `filter_freq_4`).
- Order matters only for display order and for Copy voice (which copies the cells in order).
  Setting a control again replaces its cell in place.
- A mode should assign all 48 controls (24 encoders, 8 faders, 16 buttons); use
  `unassigned(m, control)` for a control without a function. `tests/surface.rs` checks this.

### Components and swapping

A component is a function that sets cells: `def lfo_row(m, v): ...`. Use it with
`m.use(lfo_row, 3)` or `layout((voice_common, 3), more)`. To try an alternative, write a new
function next to the old one and change one line in `surface/surface.py`:

```python
surface.mode("perform", 10, "Perform", "perform", perform())       # before
surface.mode("perform", 10, "Perform", "perform-b", perform_b())   # trying a variant
```

Every mode the plugin knows (`voice:0` to `voice:6`, `sequencer`, `mix_fx`, `perform`, `lfo_mod`,
`morph_macro`, `global`) needs exactly one slot. Adding a new kind of mode (a new key) needs Rust
until the navigation moves into the spec.

### JSON format (version 1)

See the comment at the top of `crates/vdrum-surface/src/spec.rs`. One line per cell:

```json
{"control": "K11", "label": "D1 Fills", "section": "seq_track:0", "do": "seq_fills", "arg": 0}
```

## Behaviours

`do` names what a control does. The catalogue is `BEHAVIOURS` in `crates/vdrum-surface/src/spec.rs`
and its copy in `tools/surface_spec/surface_spec/behaviours.py`; each entry maps to a
`Binding` variant in `crates/vdrum-surface/src/layout.rs`.

| Argument kind | Behaviours |
|---|---|
| parameter id | `param` (encoders step it, faders set it with pickup, buttons toggle or cycle) |
| track 0..6 | `voice_fader pad to_fx seq_fills seq_rotate seq_length seq_density trigger mute track_morph track_target` |
| index | `moment` 0..7, `seq_step` 0..15, `lock_target` 0..7, `macro macro_depth snapshot learn` 0..7 |
| name | `step_field` (`velocity probability note flam shift lock1 lock2 clear_locks`), `setting` (`led_brightness screen_brightness popup_time encoder_curve pad_velocity list_speed fader_mode clock`) |
| none | `accent copy_voice paste_voice fx_on_off fx_sub_type moment_depth moment_rate seq_shuffle seq_next_pattern seq_entry_velocity seq_density_all mute_all lfo_rate_scale vel_scale lfo_offset_all lfo_depth_scale lfo_sync_host lfo_reset_phases morph_all target_all sum_aux hw_clock init_kit save_kit compare undo redo panic spare` |

### Adding a behaviour

A behaviour is Rust code; the spec only places it. Example: a button `trigger_all` that plays
every track at once (at the pad velocity, or 127 while Accent is held).

1. **Name it.** Pick a snake-case spec name (`trigger_all`) and its argument kind (none, track, an
   index with a bound, or a name list).

2. **Add the `Binding` variant** in `crates/vdrum-surface/src/layout.rs`, with a doc comment that
   says what it does on each control kind it supports:
   ```rust
   /// Button: trigger every track at the pad velocity.
   TriggerAll,
   ```

3. **Register it in the catalogue** in `crates/vdrum-surface/src/spec.rs`, in three places:
   - `BEHAVIOURS`: `("trigger_all", Arg::None),`
   - `binding_from_spec`: `"trigger_all" => B::TriggerAll,`
   - `binding_to_spec`: `B::TriggerAll => ("trigger_all", V::Null),`
   The compiler reports a missing arm in `binding_to_spec`; `behaviours_round_trip` fails if
   `binding_from_spec` does not undo it.

4. **Implement it** in `crates/vdrum-surface/src/surface/`. Where each kind of input arrives:

   | Input | Function (`surface/mod.rs` unless noted) |
   |---|---|
   | Encoder turn | `encoder()`; sequencer-only behaviours in `seq_encoder()` |
   | Fader move | `fader()` (pickup is handled there; absolute 0..127 in) |
   | Button press / release | `press()` / `release()` |
   | Button held for `HOLD_MS` (1 s) | `hold_fired()`, called from `tick()` |
   | Time passing (animations, momentary effects) | `tick()`, `moment_tick()` in `perform.rs` |
   | LED colour | `led()` (default: the section colour; buttons toggle full or dim) |
   | Value shown and its range (editor arcs, fader markers) | `range_of()` in `view.rs` |
   | Label shown | `label()` |
   | Context shown in the editor ("HOLD 1 s: ...") | `layers()` in `view.rs` |

   State the behaviour keeps goes in `Surface` (and in `state_text`/`load_state` in `perform.rs`
   if it must survive a reload). Requests to the plugin are `Action`s (`model.rs`); add a variant if
   none fits, and handle it in `plugin/src/controller.rs`. Read plugin state through `Model` only
   (cheap, non-blocking).

   ```rust
   // press()
   Binding::TriggerAll => {
       let velocity = self.pad_velocity();
       out.extend((0..7).map(|track| Action::Trigger { track, velocity }));
       self.show(cell, format!("all tracks, velocity {velocity}"));
   }
   ```
   A behaviour that needs memory (a timer, a previous value) adds a field to `Surface` and sets it
   in `Surface::new()`.

5. **Add it to the Python catalogue** in `tools/surface_spec/surface_spec/behaviours.py`:
   `"trigger_all": None,`. The function `trigger_all()` then exists for the spec.

6. **Place it** in a mode: `m.set(t(8), "All", perf(6), trigger_all())`, then
   `python3 tools/surface_spec/run.py build`.

7. **Test it** in `crates/vdrum-surface/tests/surface.rs` with the `Fake` model: press, advance the
   clock, assert on the returned `Action`s, `s.frame(...)` (LEDs, screen) or `s.view(...)`. The
   existing tests show the helpers (`press`, `release`, `turn`, `fader`).

8. **Check and document**: `python3 tools/surface_spec/run.py check`, `cargo test -p vdrum-surface`;
   add the behaviour to the table above and, if it changes what a mode does, to `design.md` and
   `control-tables.md`.

### Adding or changing a parameter id

`param_ids.json` is generated from `lxr-params`. After adding parameters there, run
`cargo run -q -p vdrum-surface --example param_ids > tools/surface_spec/surface_spec/param_ids.json`;
the test `param_id_list_is_current` fails until it is regenerated.

## Parity

`builtin_spec_matches_the_rust_tables` (in `spec.rs`) compares every mode, with and without a held
step and with and without the sequencer view, against the Rust tables the spec replaced (kept in
`layout.rs` as `legacy`, test-only), cell by cell and in order. It will be removed once the spec is
the only source and the old tables are deleted; until then, a deliberate layout change has to be
made in both places or the test has to be retired with it.
