# Plan: controller quality-of-life changes (2026-11-05)

**Status:** implemented on branch `claude/project-thread-8ss548` (all six items). Not yet tested on
hardware; see the hardware test checklist in the project files.

Request (Sami, 2026-11-05): six changes to the Launch Control XL 3 surface. The code is
`crates/vdrum-surface` (surface state machine), `plugin/src/controller.rs` (device thread) and
`plugin/src/engine.rs` / `lib.rs` (engine and transport).

## 1. Active mode button pulses orange

Chosen (Sami, question A): **only while Shift is held.** While Shift is held the 16 buttons show the
mode slots: the active mode's button pulses orange (`MODE_ACTIVE`, scaled by
`vdrum_surface::pulse`, the raised cosine of the selected-track knob pulse), the other modes are dim
white (`MODE_OTHER`) and the empty slots 13 to 15 are off. Without Shift the buttons show the mode's
own LEDs.

## 2. Shift + Page up/down: kits

Chosen (question B): **browse, and load with Play** (as before). Shift + Page up/down steps through
the library's filled slots (with an empty library: the `.snd` files of the current kit's folder);
Play loads the kit shown, any other control cancels. Page up/down without Shift move between bars in
the sequencer view and do nothing elsewhere.

## 3. LFO mode: top row follows each LFO

In mode 11 the top-row buttons 1 to 7 (*LFO wave*, a cycle button) show the LFO of track 1 to 7: the
brightness follows the LFO's output, so rate, waveform, phase, sync and amount are all reflected.

- Source: the engine's own LFO output, not a separate simulation, so the phase is exact. The audio
  thread publishes the seven LFO values (one atomic per voice, once per block) and the controller
  reads them at its 20 ms frame rate. LFOs faster than about 25 Hz alias at that rate.
- Brightness = the LFO section colour scaled by `(1 - amount) * 0.6 + amount * (floor + (1 -
  floor) * lfo)` (`vdrum_surface::lfo_brightness`; LFO output 0..=1, amount = *LFO amt* as a
  fraction, floor 8 %): a zero amount gives a steady 60 %, a full amount swings from the floor to
  full with the LFO.
- The buttons keep their function (cycle the waveform). Engine side:
  `Lxr::vdrum_lfo_values` (`crates/lxr-core/src/vdrum/lfo_view.rs`).

## 4. Record button: remove live recording, new function

Live recording (Record arm; pads and Perform triggers write steps at the playhead) is removed.

Chosen (Sami, decision card): **Clear.**

- Hold Record + a step button (sequencer view): clears that step on the selected track (off,
  default velocity and timing, no locks). The step is not toggled.
- Hold Record + Track < or >: clears the selected track's pattern in the playing pattern (Euclid
  pulses off, every step back to default). Length and density stay; the selection does not move.
- LED: dim red in the sequencer view, bright while held; off elsewhere.

Other options offered were Save kit (hold 1 s), Tap tempo and Copy / paste.

## 5. Play stops a running sequencer

- Internal clock: Play stops a running sequencer in every mode; it starts it in mode 8.
- Host clock: the plugin cannot stop the host's transport (nih-plug has no transport control).
  Chosen (question C): **Play silences the sequencer** until pressed again. The engine keeps
  following the host clock but triggers no steps (`VdrumState::seq_silent`, checked in the
  sequencer's step-trigger loop; `SeqEdit::Silence`, `SeqStatus::silent`). Play's LED is red while
  silenced and the mode 8 screen shows "silent". A change of the clock setting ends it.
- Otherwise Play loads a browsed kit, or auditions the current track in a voice mode.

## 6. Sequencer view, decoupled from the mode

- **Solo/Arm (press):** toggles the sequencer view: the 16 buttons become the step buttons of the
  selected track in any mode. Knobs and faders keep the current mode's functions. LED white while
  on.
- **While the view is active:** Track < / > select the track; Page up/down move between bars;
  hold Mute/Select = pattern layer (a tap still goes to Perform); Record = Clear.
- **Hold a step + move a control (other modes):** stores a parameter lock of that control's
  parameter on the held step of the selected track; the kit value does not change. An encoder moves
  the lock from the kit value; a fader sets it to the fader's position (the fader then needs pickup
  again). Limits from the engine: two lock slots per step (a third replaces the oldest), and
  parameters with numbers above 254 (the open hi-hat's) cannot be locked; controls without a
  parameter cannot lock. In mode 8 the step-held layout (velocity, probability and so on) stays as
  it is.
- **Shift + Solo/Arm:** toggles "sticky": the view stays on when the mode changes (Shift + button
  N). Without sticky, a mode change turns the view off. LED orange while sticky.
- **Mode 8** keeps its knobs (Euclid fills, rotate, length, density) and always has the view.
- Chosen (question D): **the track-group layer (hold Solo/Arm) is removed.** Steps and locks are
  edited on the selected track only.

## Questions (answered 2026-11-05)

- A. Active mode pulse: only while Shift is held.
- B. Shift + Page: browse and load with Play.
- C. Host clock: Play silences the sequencer until pressed again.
- D. Track-group layer: dropped entirely.
- E. Record: Clear.
