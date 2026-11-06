# Surface spec: authoring the LCXL3 surface in Python

Status (2026-11-06): every step of the plan in `declarative-modes-brainstorm.md` (section 6.5) is
built. The spec covers the mode layouts, the overlays (sequencer view, held step), navigation (what
the side buttons and the 16 buttons do in each state), the side-button LEDs, the stationary screen
and the colours. New behaviour is Rust code (see "Adding a behaviour" and "Adding a navigation step
or a predicate"). The simulator runs the real surface code in the browser against a virtual LCXL3,
and can drive a real LCXL3 over Web MIDI.

## Files

| Path | What it is |
|---|---|
| `surface/surface.py` | The composition: which mode sits in which slot, the overlays, the rules. Start here. |
| `surface/modes/*.py` | One file per mode or group of modes; each function returns a `Layout`. |
| `surface/components/*.py` | Reusable pieces (`voice_common`, `lfo_row`, `metal`, `pads`, `steps`, `unassigned`). |
| `surface/rules/navigation.py` | Navigation rules: side buttons and the 16 buttons, by state. |
| `surface/rules/leds.py` | Side-button LED rules, and when the 16 buttons show the mode slots. |
| `surface/rules/screen.py` | The stationary screen templates. |
| `surface/rules/palette.py` | Section colours, track colours, named colours. |
| `surface/scenarios/*.py` | Scenarios: gestures with expected results, run by `cargo test` and in the simulator. |
| `tools/surface_spec/run.py` | The tool: `build`, `check`, `watch`, `serve`, `bundle`, `svg`. |
| `tools/surface_spec/surface_spec/` | The Python library: `dsl.py` (layouts), `behaviours.py`, `rules.py` (navigation, LEDs, screen, palette, Mermaid), `scenario.py`, `voice.py`, `build.py`, `docs.py` (generated docs), `hardware.py` and `device_svg.py` (device drawing). |
| `tools/surface_spec/surface_spec/param_ids.json` | Every parameter id, generated from `lxr-params`. |
| `tools/surface_sim/` | The simulator page (`index.html`, `sim.js`, `sim.css`) and its browser tests (`test/`). |
| `crates/vdrum-surface/spec/surface.json` | The built spec. Generated; commit it with the sources. |
| `crates/vdrum-surface/src/spec.rs` | The Rust loader, the behaviour catalogue and its tests. |
| `crates/vdrum-surface/src/rules.rs` | The rule types, their catalogues (buttons, predicates, steps, screen variables) and parser. |
| `crates/lcxl3/src/emulator.rs` | A virtual LCXL3: takes the host's DAW-mode messages, keeps what the device shows, turns gestures into the device's messages. |
| `crates/vdrum-sim/` | The simulator: surface + virtual device + a stand-in for the plugin, driven by JSON commands; builds natively and for WebAssembly. `scenarios.json` (generated) and its tests. |
| `docs/lcxl3-daw-mode/generated/` | Generated docs: `control-tables.md` with an image per mode, `navigation.md` with the state diagram. |

## Workflow

```sh
python3 tools/surface_spec/run.py serve     # simulator at http://localhost:8737; every save rebuilds
                                            # and the open page reloads the spec
python3 tools/surface_spec/run.py watch     # rebuild on every save, no page (for the plugin)
python3 tools/surface_spec/run.py build     # once
python3 tools/surface_spec/run.py check     # spec valid and every generated file up to date (before committing)
cargo test -p vdrum-surface -p vdrum-sim -p lcxl3      # loader, behaviour, protocol and scenario tests
node tools/surface_sim/test/browser.test.cjs           # browser tests (Playwright, Chromium)
```

`serve` needs the Rust target `wasm32-unknown-unknown` (`rustup target add wasm32-unknown-unknown`)
and builds the simulator with it on start. `build` writes `surface.json`, `scenarios.json` and
`docs/lcxl3-daw-mode/generated/`; `check` fails if any of them is out of date.

**With the plugin and the real device.** The plugin embeds `surface.json` at compile time. To try a
spec without rebuilding the plugin, start the host with `VDRUM_SURFACE_SPEC` set to the built file
and run `watch`:

```sh
VDRUM_SURFACE_SPEC=$PWD/crates/vdrum-surface/spec/surface.json <host>   # e.g. Bitwig, Reaper
python3 tools/surface_spec/run.py watch
```

The plugin checks the file every 500 ms and runs a new version at once (the popup shows "Surface
spec reloaded"); a file that does not load leaves the running spec in place and the popup shows the
start of the error (the whole error goes to stderr).

Errors name where they are, for example
`error: layout closed-hi-hat T1: param: bad argument 'wave1_hx' (expected param)` or
`error: navigation rule 2 (solo_arm): unknown predicate shfit`.

## The simulator

`run.py serve` (local, live reload) or the hosted page (`run.py bundle DIR` writes the static
files; load a spec file with "Load spec").

- **The device**: the drawing from `hardware.py` at the device's proportions. LEDs, the screen and
  fader positions are what the virtual LCXL3 holds after the surface's MIDI messages; the surface
  and the virtual device only talk MIDI bytes, as with the hardware.
- **Input**: encoders: scroll, or drag up and down (one detent per 6 px). Faders: drag or scroll.
  Buttons: held while the mouse button is down; right-click latches a button until the next
  click (for combinations such as Record + step). Space holds Shift.
- **Labels**: the current label of each control, in its section colour (toggle).
- **State**: mode, sequencer view, held buttons and steps, sequencer, kit, the editor's context
  layers; hovering a control shows its cell.
- **State diagram**: the 16 mode slots as on Shift + button N, the sequencer view and Shift, with
  the current state highlighted.
- **Navigation rules**: the rule table; the rule that handled the last button event is highlighted.
- **Log**: parameter changes (id, old and new value, as shown), other actions, the rule that
  fired, decoded input, MIDI in and out; filter by kind.
- **Scenarios**: runs one of `scenarios.json` step by step and reports the first failed check.
- **Web MIDI**: choose the LCXL3's DAW in and out ports and press Connect LCXL3: the page sends the
  same connect sequence as the plugin (DAW mode, relative encoders, display setup, settings), the
  device's input drives the simulator, and LEDs and screen go to the device. Disconnect returns it
  to standalone mode. Chrome and Edge; Firefox needs its site-permission add-on for Web MIDI; Safari
  has none. The page asks for SysEx access (LEDs and the screen are SysEx). Tested with a mocked
  MIDI port, not yet on a device.

What the simulator does not reproduce (unverified or out of scope): the device's own font and text
positions (the screen draws text with a generic face at fixed sizes, one bit per pixel; bitmaps are
drawn exactly); the Back button's LED (its logo was taken out of the drawing; its colour is in the
state); the encoders' physical rotation; whether the device combines fast turns into larger
relative steps (the virtual device sends one message per detent); the factory display timeout
before the host sets one; sound (the plugin stand-in keeps parameter values, triggers, mutes and a
simple sequencer clock, and its LFOs are 0.5 Hz sines).

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
  Mute/Select, the button right of Solo/Arm) are navigation: see "Navigation".
- Sections: `OSC AMP MOD FM TRANS FILTER LFO VELO MIX FX MOMENT SEQ STEP MACRO GLOBAL DEVICE EDIT
  NAV SPARE`, and per track `seq_track(t)`, `perf(t)`, `pad_section(t)` with t = 0..6. Their
  colours are in `surface/rules/palette.py`.
- Tracks are 0..6: D1, D2, D3, SD, CY, CH, OH (`TRACKS`). `vid(t, kind)` gives a voice's parameter
  id, as `voice_param` in Rust (`vid(3, "cutoff")` is `filter_freq_4`).
- Order matters only for display order and for Copy voice (which copies the cells in order).
  Setting a control again replaces its cell in place.
- A mode should assign all 48 controls (24 encoders, 8 faders, 16 buttons); use
  `unassigned(m, control)` for a control without a function. `builtin_spec_has_every_mode` checks
  this.

### Components and swapping

A component is a function that sets cells: `def lfo_row(m, v): ...`. Use it with
`m.use(lfo_row, 3)` or `layout((voice_common, 3), more)`. To try an alternative, write a new
function next to the old one and change one line in `surface/surface.py`:

```python
surface.mode("perform", 10, "Perform", "perform", perform())       # before
surface.mode("perform", 10, "Perform", "perform-b", perform_b())   # trying a variant
```

Every mode the plugin knows (`voice:0` to `voice:6`, `sequencer`, `mix_fx`, `perform`, `lfo_mod`,
`morph_macro`, `global`) needs exactly one slot (1 to 16). Moving a mode to another slot is a
spec change; the slot and name shown everywhere (screen, editor tabs) follow. Adding a new kind
of mode (a new key) needs Rust (`Mode` in `layout.rs`, `mode_from_str` in `spec.rs`).

## Navigation

`surface/rules/navigation.py` lists rules in order. For a button event, the first rule whose
button and guards match, and which has an entry for the event (press or release), handles it:

```python
nav.on("solo_arm", when=["shift"], press=[r.toggle_sticky()], release=[])
nav.on("top", when=["shift"], press=[r.mode_slot(1)])        # the release falls through
nav.on("top", when=["seq_view"], press=[r.seq_button(0)], release=[r.seq_button(0)])
```

- `release=[]` handles the release by doing nothing; leaving `release` out lets the release fall
  through to the next rule (above: Shift + top N selects a mode on press; its release goes on to
  the step buttons in the sequencer view, as before).
- Buttons: `shift solo_arm mute_select logo page_up page_down track_left track_right record play`,
  and `top` / `bottom` for a whole row (steps receive the column).
- Guards (all must hold; `!` negates): `shift seq_view view_toggle sticky voice_mode step_held
  seq_track_first seq_track_last seq_bar_first seq_bar_last seq_running seq_silent host_clock
  browsing`, `mode:<key>`, `held:<button>`.
- Steps: `shift` (Shift follows the button), `toggle_sticky`, `mode_slot(base)`,
  `seq_button(base)`, `clear_seq_track`, `seq_track_step(±1)`, `previous_mode`,
  `toggle_seq_view`, `message(title, name, value)`, `layer_begin`, `mode(key)`,
  `mode_unless_layer_used(key)`, `kit_browse(±1)`, `seq_bar_step(±1)`, `play`, `cell` (the
  control's cell in the current layout). The doc comments of `NavStep` and `Pred` in
  `crates/vdrum-surface/src/rules.rs` say exactly what each does.
- `note` is shown in the simulator's rule table and log.

The rules, the state diagram (Mermaid) and the LED and screen rules are written out in
[generated/navigation.md](generated/navigation.md).

**LEDs** (`surface/rules/leds.py`): per side button, rules `(guards, colour)`; the first match
wins, no match is off. Colours are names from the palette, section names, or `mode_pad` (the
current voice mode's pad colour). `l.mode_slots_when("shift")` says when the 16 buttons show the
mode slots (active slot pulsing in `mode_active`, others `mode_other`, empty slots off).

**Screen** (`surface/rules/screen.py`): the stationary display's three lines (title, name, value)
as templates; the first rule whose guards match is shown and the last has none. Variables:
`{mode_slot} {mode_name} {kit} {last_touched} {pattern} {bar} {next_pattern} {silent}
{seq_track_number} {seq_track_name}`. Popups are written by the behaviours (Rust).

### Adding a navigation step or a predicate

1. Add the variant to `NavStep` (or `Pred`) in `crates/vdrum-surface/src/rules.rs`, with a doc
   comment, and its name to `NAV_STEPS` (with its argument kind) or `PREDICATES`; parse it in
   `nav_step` (`guard` handles `PREDICATES` by itself).
2. Implement it: `Surface::nav_step` (or `Surface::guard`) in `crates/vdrum-surface/src/surface/mod.rs`.
3. Add the name to `NAV_STEPS` / `PREDICATES` in `tools/surface_spec/surface_spec/rules.py`; the
   function `r.<name>()` then exists. If it changes the mode or the sequencer view, add it to the
   diagram in `mermaid()` there.
4. Use it in `surface/rules/navigation.py`, add a scenario, run `check` and the tests.

A new screen variable: add it to `SCREEN_VARS` in both files and give it a value in
`Surface::frame` (the `var` closure).

## Scenarios

`surface/scenarios/*.py`, each with a function `scenarios()` returning a list:

```python
s = Scenario("Solo/Arm toggles the sequencer view")
s.press("SOLO").expect({"surface.seq_view": True, "leds.SOLO": [254, 254, 254]})
s.with_held("SHIFT", lambda s: s.press("T8"))
s.turn("K11", 2).fader("F1", 100).wait(1000)
s.expect({"screen.lines": ["8 Sequencer", "P01 bar 1/4", "Track 1 D1"]})
```

Gestures: `press hold release with_held turn fader wait`. Keys are panel keys (`K11`..`K38`,
`F1`..`F8`, `T1`..`T8`, `B1`..`B8`, `SOLO MUTE LOGO PAGE_UP PAGE_DOWN TRACK_LEFT TRACK_RIGHT REC
PLAY SHIFT`). Expectations are paths into the simulator's state (`Sim::state` in
`crates/vdrum-sim/src/lib.rs`): `surface.mode`, `screen.source`, `screen.lines.0`, `leds.T1`,
`cells.K11.value`, `seq.running`, `kit`, ... LED values are what the device shows: the surface's
8-bit colours go out as 7 bits, so 255 reads 254. A path ending in `~` checks that the text
contains the value; `"lit"` / `"off"` check an LED at any colour. Each scenario starts from a fresh
simulator in mode 1. `cargo test -p vdrum-sim` runs them all and lists every failed check.

## JSON format (version 2)

See the comment at the top of `crates/vdrum-surface/src/spec.rs`. One line per cell or rule:

```json
{"control": "K11", "label": "D1 Fills", "section": "seq_track:0", "do": "seq_fills", "arg": 0}
{"button": "solo_arm", "when": ["shift"], "press": [{"do": "toggle_sticky"}], "release": []}
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
   add the behaviour to the table above. `build` regenerates the control tables and mode images in
   `generated/`.

### Adding or changing a parameter id

`param_ids.json` is generated from `lxr-params`. After adding parameters there, run
`cargo run -q -p vdrum-surface --example param_ids > tools/surface_spec/surface_spec/param_ids.json`;
the test `param_id_list_is_current` fails until it is regenerated.

## Parity

Step 1 reproduced the former Rust tables exactly: a test compared every mode, with and without a
held step and with and without the sequencer view, cell by cell and in order, and the plugin's
tests passed. On Sami's go-ahead (2026-11-06) the Rust tables and that test were removed.

When navigation, LEDs, the screen and the palette moved into the spec, the new code was checked
against the previous commit with a differential test outside the repository: 3,000 random
sequences of 120 inputs each (buttons including Shift combinations and held buttons, encoder turns,
fader moves, time steps up to 1.1 s), run on both, compared the actions, frames (every LED, the
screen, the popup), the mode and the editor view after every input. All 360,000 were identical,
and every mode was reached with and without the sequencer view. A deliberate one-colour change in
the spec was caught by the same test.
