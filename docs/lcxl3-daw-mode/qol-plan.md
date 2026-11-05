# Plan: controller quality-of-life changes (2026-11-05)

**Status:** draft, waiting for answers to the questions at the end. Nothing is implemented.

Request (Sami, 2026-11-05): six changes to the Launch Control XL 3 surface. The code is
`crates/vdrum-surface` (surface state machine), `plugin/src/controller.rs` (device thread) and
`plugin/src/engine.rs` / `lib.rs` (engine and transport).

## 1. Active mode button pulses orange

The button N that selects the current mode (Shift + button N) pulses orange, using the same raised
cosine as the selected-track knob pulse (`vdrum_surface::pulse`).

- Open question A: always, or only while Shift is held (see Questions).

## 2. Shift + Page up/down: kits

Page up/down currently browse the kit library's filled slots and Play loads the shown kit. With the
sequencer view (item 6) Page up/down move between bars, so kit selection moves to Shift + Page.

- Shift + Page up/down steps through the library's filled slots (with an empty library: the `.snd`
  files of the current kit's folder, as now).
- Open question B: load on each press, or browse and load with Play (as now).

## 3. LFO mode: top row follows each LFO

In mode 11 the top-row buttons 1 to 7 (currently *LFO wave*, a cycle button) show the LFO of track
1 to 7: the brightness follows the LFO's output, so rate, waveform, phase, sync and amount are all
reflected.

- Source: the engine's own LFO output, not a separate simulation, so the phase is exact. The audio
  thread publishes the seven LFO values (one atomic per voice, once per 64-frame block) and the
  controller reads them at its 20 ms frame rate. LFOs faster than about 25 Hz alias at that rate.
- Brightness = the LFO section colour scaled by `(1 - amount) * 0.6 + amount * (floor + (1 -
  floor) * lfo)` (`vdrum_surface::lfo_brightness`; LFO output 0..=1, amount = *LFO amt* as a
  fraction, floor 8 %): a zero amount gives a steady 60 %, a full amount swings from the floor to
  full with the LFO.
- The buttons keep their function (cycle the waveform). Implemented; the engine side is
  `Lxr::vdrum_lfo_values` (`crates/lxr-core/src/vdrum/lfo_view.rs`).

## 4. Record button: remove live recording, new function

Live recording (Record arm; pads and Perform triggers write steps at the playhead) is removed:
`record`, `record_hit`, the REC marker on the screen, and the docs. Implemented; Record does
nothing (LED off) until its new function is chosen.

Suggested new functions, one to choose (decision card in the thread):

| Option | What it does |
| --- | --- |
| Save kit (hold 1 s) | Stores the current kit to its library slot. Today this is only in the Global mode list and the editor. The hold avoids accidental overwrites. |
| Tap tempo | Taps set `PAR_BPM` (internal clock). No effect with the host clock, which is the default. |
| Copy / paste | Hold Record + step N copies; hold Record + another step pastes (step data and locks). |
| Clear | Hold Record + step N clears that step on the edit group; hold Record + Track < / > clears the selected track's pattern. |

## 5. Play stops a running sequencer

- Internal clock: Play starts and stops (already the case in mode 8; it extends to every mode).
- Host clock (default): the plugin cannot stop the host's transport (nih-plug has no transport
  control). Open question C.
- Outside the sequencer, Play currently auditions the current track and loads a browsed kit.

## 6. Sequencer view, decoupled from the mode

- **Solo/Arm (press):** toggles the sequencer view: the 16 buttons become the step buttons of the
  selected track / edit group in any mode. Knobs and faders keep the current mode's functions.
- **While the view is active:** Track < / > select the track; Page up/down move between bars;
  hold Mute/Select = pattern layer (as in mode 8).
- **Hold a step + move a control (any mode):** stores a parameter lock of that control's parameter
  on the held step, for each track of the edit group. Limits from the engine: two lock slots per
  step (a third replaces the oldest), and parameters with numbers above 255 (the open hi-hat's) cannot
  be locked; faders lock only if they control a lockable parameter. In mode 8 the step-held layout
  (velocity, probability and so on) stays as it is.
- **Shift + Solo/Arm:** toggles "sticky": the view stays on when the mode changes (Shift + button
  N). Without sticky, a mode change turns the view off.
- **Mode 8** stays: its knobs (Euclid fills, rotate, length, density) and the view on by default.
- The track-group layer currently on *hold* Solo/Arm: open question D.

## Questions

- A. Active mode pulse: always on button N (it then overrides that button's own LED), or only while
  Shift is held (the buttons show the modes; button N pulses orange)?
- B. Shift + Page: load each kit at once, or browse and load with Play?
- C. Host clock: Play cannot stop the host. Should Play then mute the sequencer (the host keeps
  playing, steps are not triggered) until pressed again, or do nothing?
- D. Solo/Arm press toggles the view. Keep *hold* Solo/Arm for the track-group layer (multi-track
  editing), or drop the group layer (Track < / > only)?
- E. Record: which function (decision card)?
</content>
</invoke>
