# VDrum on Ableton Move: mapping brainstorm

Status: brainstorm, revision 1 (2026-11-09). Nothing here is implemented. The Move is the second
device for the generic surface library ([../surface-library/architecture-plan.md](../surface-library/architecture-plan.md)).

## Sources

- **Ableton Move manual:** https://www.ableton.com/en/move/manual/ (one page). Sections 1 to 17.1
  were read verbatim. Sections 17.2 to 20 (rest of Session Mode, Control Live Mode, USB operation
  modes, control reference) were read only as a summary and are marked "unverified" where used.
- **Schwung** (https://github.com/charlesvestal/schwung): the README, `docs/API.md` and
  `docs/ARCHITECTURE.md` were read. `docs/MODULES.md` (225k characters) was read in part; claims
  from its unread part are marked "unverified".
- **Sami's front-panel photo** (2026-11-09).
- **VDrum's current LCXL3 modes:** `docs/lcxl3-daw-mode/design.md` and `control-tables.md` on
  `main`.

## 1. The hardware, as far as it matters here

| Control | Count | Facts | Source |
|---|---|---|---|
| Pads | 32 (4 × 8) | Velocity-sensitive, independently pressure-sensitive (polyphonic aftertouch), backlit in many colours. Schwung: notes 68 to 99, bottom-left to top-right. | Manual 1.2, 9.4; Schwung API.md |
| Step buttons | 16 | Backlit. Beat marks under 1, 5, 9, 13. Notes 16 to 31. | Manual; API.md |
| Encoders | 8 above the pads + 1 Volume (top right) | Touch-sensitive ("high-resolution"). Schwung sends relative CC 71 to 78 and absolute CC 102 to 109, Volume CC 79 relative, and touch as notes 0 to 8. The manual does not say whether they have detents. | Manual 1.2; API.md |
| Encoder LEDs | 1 per encoder (the photo shows a white dot under each) | The manual uses them for automation state: red, white or off. Whether brightness or colour can be set per value is not documented. | Manual 14; photo |
| Wheel | 1 (left) | Touch-sensitive, clickable, with detents. Schwung: CC 14 (turn), CC 3 (click), note 9 (touch). | Manual 1.2; API.md |
| Track buttons | 4 (left of the pads) | Each has a coloured light strip (cyan, lavender, red, yellow in the photo). Schwung: CC 43 to 40 (tracks 1 to 4). | Photo; API.md |
| Function buttons | Play, Record, Back, Menu (≡), Capture, Sample (●), Loop, Mute, Delete, Copy, Undo, Shift (…), Left, Right, Plus, Minus | Schwung CCs: Shift 49, Menu 50, Back 51, Capture 52, Minus 54, Plus 55, Undo 56, Copy 60, Left 62, Right 63, Play 85, Record 86, Mute 88, Sample 118 (RGB LED), Delete 119. Loop's CC is not in the part of API.md that was read. | API.md; photo |
| Display | 1 | 1.3-inch white OLED, 128 × 64, monochrome. | Manual 1.2; API.md |
| Audio, platform | — | 44.1 kHz. Schwung blocks are 128 frames. Quad-core ARM Cortex-A72, 2 GB RAM. Battery up to about four hours. | Manual; API.md |

**Mismatch with the LCXL3.** The LCXL3 has 24 knobs, 8 faders and 16 RGB buttons. The Move has
8 encoders, 32 pads, 16 steps and 4 track buttons. VDrum has 7 tracks, so the Move's 4 track
buttons cannot be per-track.

## 2. How VDrum would reach the Move: three routes

The mapping in section 3 works for all three routes. Which route is chosen decides what is
possible, so it is the first question (section 5).

| Route | How | Works? | Notes |
|---|---|---|---|
| **M1. Move as a controller over USB-C** for the VDrum plugin on a computer | The plugin talks MIDI to the Move, as it does to the LCXL3 today. | **Not documented.** Move's computer-control mode is "Control Live", for Live 12.1+ or 11.3.35+ only. The manual documents no generic MIDI-controller mode, and no note or CC map over USB-C. | Would need Ableton to open this, or reverse engineering of Control Live, which is fragile. |
| **M2. Schwung module as a bridge** | A Schwung module on the Move reads pads, encoders and buttons, and sends them over Wi-Fi or the USB-C network to the plugin on the computer. LEDs and display data come back the same way. | Feasible: Schwung modules are native code "with full system access". | Custom transport on both sides. Adds network latency, which is not measured. |
| **M3. VDrum running on the Move** (standalone) | The LXR engine (`lxr-core`) and the sequencer are compiled for aarch64 as a Schwung `sound_generator` (C ABI `dsp.so`: `create_instance`, `on_midi`, `render_block`, …). The surface runtime runs on the device. | Plausible. Schwung documents native DSP plugins at 44.1 kHz with 128-frame blocks. The engine first ran on an LXR-02, a microcontroller, so a Cortex-A72 should have room; not measured. | Turns the Move into a battery-powered VDrum. The Schwung UI side is JavaScript (QuickJS), so the Rust runtime would need a bridge to it. Unverified (MODULES.md summary): ordinary modules cannot claim Shift, Menu, Back, the wheel, knobs 1 to 8, Volume, the track buttons or Mute; only "overtake" modules take the UI. |

Two cautions on Schwung:
- It is unofficial and not endorsed by Ableton. It modifies system files and needs SSH.
- Its README says it "is not stable, or generally usable as a daily driver."

Schwung is MIT-licensed. Its default shim links eSpeak NG and is then GPL-3.0-or-later, which is
compatible with VDrum's GPL-3.0-only.

## 3. Proposed mapping

### 3.1 Principle: rows become banks, columns become encoders

Every LCXL3 mode is an 8-column grid: 3 knob rows and a fader row, plus a top and a bottom button
row. On the Move:

| LCXL3 | Move |
|---|---|
| Columns 1 to 8 | Encoders 1 to 8 |
| Knob row 1, knob row 2, knob row 3, fader row | 4 parameter banks, chosen with **track buttons 1 to 4** |
| Top button row | **Pad row 3** |
| Bottom button row (where a mode has its own) | **Pad row 2** |
| — | **Pad row 1** (bottom): the 7 voices + Accent, in every mode |
| — | **Pad row 4** (top): per-mode extras (section 3.4) |

This keeps the existing layouts nearly 1:1. In the generic library it means the same VDrum
profile can drive both devices through a device-specific adapter, rather than through a second
hand-written layout. The cost is one bank switch for parameters that are one knob away on the
LCXL3.

Each track button's light strip shows its bank's section colour. Touching an encoder shows the
parameter and value on the display, which is Move's own convention.

### 3.2 Controls that are the same in every mode

| Move control | VDrum function | LCXL3 equivalent | Move convention it follows |
|---|---|---|---|
| Shift + Step 1 to 16 | Go to mode 1 to 16. While Shift is held, the step LEDs show the mode slots. | Shift + button N | Shift + Step N opens Move's own menus. |
| Pad row 1, pads 1 to 7 | Play the voice (velocity-sensitive) and select it. In modes 1 to 7, selecting a voice switches to that voice's mode, so these pads replace Shift + Step 1 to 7. | Bottom-row trigger pads | Pressing a drum pad plays and selects it. |
| Pad row 1, pad 8 | Accent (hold: pads play at velocity 127) | Button 8 = Accent | — |
| Track buttons 1 to 4 | Parameter bank 1 to 4 | (knob rows and faders) | Shift + wheel switches banks on Move. |
| Back | Previous mode (toggles the last two) | Novation logo button | Back goes up a level. |
| Menu (≡) | Mode 16, Global | Shift + button 16 | Opens settings. |
| Wheel turn / click / touch | Browse kits / load the shown kit / show the kit name. Shift + wheel: tempo. | Shift + Page up/down, then Play | Wheel browses; a click loads. |
| Play | As on the LCXL3: start/stop (internal clock) or silence/unsilence (host clock) | Play | Play starts and stops. |
| Record | **Open question.** Live recording was removed on 2026-11-05. Options: unused; or bring back live recording of pads into the pattern. | (was Record = Clear) | Record records. |
| Delete (hold) | Clear: + a step clears the step, + a voice pad clears that track's pattern, + an encoder clears that lock while a step is held | Hold Record + step / Track | Delete + pad / step deletes notes. |
| Mute (hold) + voice pad | Mute or unmute that track. Shift + Mute + pad: solo, meaning mute all the others. Mute alone mutes the selected track. | Mode 10, bottom row | Same on Move. |
| Copy (hold) + voice pad, then another voice pad | Copy one voice's parameters to another. Copy + step, then step: copy a step. | Copy voice / Paste voice buttons in modes 1 to 7 | Copy + step / pad. |
| Undo / Shift + Undo | Undo / Redo | Mode 16, buttons 6 and 7 | Undo. |
| Left / Right | Bar 1 to 4 in the sequencer view | Page up/down | Arrows move between bars. |
| Plus / Minus | Next pattern ±1, queued to bar end. With a step held: the step's note ±1, long press ±12. | Hold Mute/Select, pattern layer | With a step held, they transpose. |
| Loop (hold) | Step buttons show bars 1 to 4 of the selected track, and the track length. Hold Loop + wheel: track length. | Mode 8, knob row 3 | Loop + wheel sets the loop length. |
| Volume encoder | Move's own output volume. Hold a voice pad: that voice's level. Hold a step: that step's velocity. | Faders | Volume + held pad = pad gain; + held step = velocity. |
| Capture | **Open.** Candidates: store the current kit as a snapshot, or capture recent pad hits into the pattern (Move's own meaning). | — | Capture keeps what was just played. |
| Sample (●) | **Open.** VDrum has no sampling. Candidate: Compare (hold = hear the saved kit). | Mode 16, Compare | — |
| Step buttons outside the sequencer view | The selected track's steps. The LCXL3 needs a sequencer-view toggle (Solo/Arm) to show steps; on the Move the step buttons are free in every mode, so the steps are always visible. | Solo/Arm view | Steps are always the sequencer. |

### 3.3 Mode by mode

Encoder banks follow the LCXL3 rows. Abbreviations: D1 to OH are the seven voices.

| Mode | Bank 1 (track button 1) | Bank 2 | Bank 3 | Bank 4 | Pad row 3 | Pad row 2 |
|---|---|---|---|---|---|---|
| 1 to 7 Voice (the selected voice) | Fine, Attack, Pitch EG decay, FM / Noise / Metal freq, Trans freq, Resonance, Vel dest, Pan | Pulse width, Amp slope, Pitch EG slope, FM wave / …, (spare or Repeat), Filter drive, Vel amount, Overdrive | LFO: rate, amount, offset, wave, sync, retrigger, dest voice, dest param | Coarse, Decay, Pitch EG amount, FM amount / mix / metal gain, Trans volume, Cutoff, Sample rate, Volume | Osc wave, (Copy and Paste move to the Copy button: 2 pads free), FM/Mix or Choke, Trans wave, Filter type, Vel > Vol, Audio out | Spare (row 1 already plays the voices) |
| 8 Sequencer | Fills (Euclid pulses) D1 to OH, Shuffle | Rotate D1 to OH, Next pattern | Length D1 to OH, Entry velocity | Density D1 to OH, Density all | Patterns 1 to 8 (queued) | Patterns 9 to 16 |
| 8, step held | Velocity, Probability, Note, Flam, Micro-shift, Lock 1, Lock 2, Clear locks | — | — | — | — | — |
| 9 Mix + FX | FX P1 to P4, FX type, FX sub-type, FX output, Global S.rate | Pans, Effect depth | Drives, Effect rate | Volumes, FX dry/wet | Voice > FX sends, FX on/off | Momentary effects: Stutter 1/16, Stutter 1/32, Filter sweep, Reso up, Swell, Long decay, Lo-fi, FX throw |
| 10 Perform | Pitches, FX P1 | Decays, FX P2 | Cutoffs, Global S.rate | Levels, FX dry/wet | Mutes D1 to OH, Mute all | Momentary effects (as mode 9) |
| 11 LFO + Mod | LFO rates, Rate scale | LFO amounts, Vel scale | LFO dests, Offset all | Vel amounts, Depth scale | LFO waves, Sync to host | LFO syncs, Reset phases |
| 12 Morph + Macro | Macro depths 1 to 8 | Morph D1 to OH, Morph all | Targets D1 to OH, Target all | Macros 1 to 8 | Snapshots 1 to 8 (hold 1 s: store) | Learn 1 to 8 |
| 16 Global | MIDI notes, Tempo | MIDI channels, Global ch. | Settings: pad velocity, list speed, clock (the LCXL3 settings drop out) | HW faders, Global S.rate | Sum aux, HW clock, Init kit, Save kit, Compare, Undo, Redo, Panic | — |

Notes on specific modes:

- **Step-held bank (mode 8).** The step-held bank is shown on the encoders while a step is held,
  in any mode. Move's convention is that turning any encoder while holding a step records
  automation for that step. In voice modes, VDrum can do the same: turn a voice parameter while
  holding a step and it becomes one of the step's two parameter locks. This replaces the fixed
  "Lock Pitch / Decay / Cutoff …" list of the LCXL3 with any parameter, limited to two locks per
  step by the firmware's step format (`seq.rs`, `locks: [(u8, u8); 2]`).
- **Perform (mode 10).** On the LCXL3 the trigger buttons sit on the top row. On the Move, pad
  row 1 already triggers, so row 3 takes the mutes and row 2 the momentary effects. Momentary
  effects on velocity-sensitive pads could use pad pressure (polyphonic aftertouch) as the effect
  depth.
- **Mix + FX vs Perform.** These now overlap (both have momentary effects). They could be merged
  into one mode on the Move.

### 3.4 Pad row 4 and pad pressure (proposals)

- **Row 4 in voice modes:** 8 velocity layers of the selected voice (velocity 16, 32, …, 127),
  like the Move's "16 levels" idea. Alternative: 8 pitches of the selected voice.
- **Row 4 in the sequencer:** bars 1 to 4 and track-length shortcuts, or a second voice-pad row
  for a 2-bar overview.
- **Pad pressure:** on voice pads, pressure → a chosen parameter of that voice (per-voice
  "aftertouch destination", for example filter cutoff or decay), set in mode 16.

### 3.5 LEDs and display

- **Pads:**
  - Voice pads: the track colour, flashing on each hit from any source; red when muted.
  - Toggles: bright when on, dim when off.
  - Momentary effects: dim, bright while held.
- **Step buttons:**
  - Set steps in the track colour; empty steps dim; playhead white.
  - Move uses green for the playhead. Which colour VDrum uses is open.
- **Encoder LEDs:** at least on/off per encoder, for example lit when the parameter is not at its
  default. Value brightness, as on the LCXL3, depends on undocumented LED capabilities.
- **Track buttons:** the bank colour; white for the active bank.
- **Display (128 × 64, 1-bit):**
  - Stationary: the mode, the voice or bank name and the kit name.
  - Mode 8: pattern, bar and track.
  - On encoder touch or turn: section, parameter and value (formatted by `lxr_params`, as on the
    LCXL3).
  - Because this is a bitmap display, the library's display layer renders it. This is the
    "bitmap" display kind in the plan.

## 4. What this means for the library plan

- A `devices/ableton-move` crate replaces the synthetic `generic-cc` as the second device. It
  contains the Schwung MIDI map as the protocol, the panel geometry for the SVG, and an emulator.
- The "rows become banks" adapter is a library feature: a grid profile (columns × rows) rendered
  on a device with fewer controls through banks. VDrum's profile stays one definition.
- Route M3 would make the surface runtime run on the device, inside a Schwung module. That
  supports keeping the runtime pure Rust with a C ABI and no threads, as the plan proposes.

## 5. Questions

1. **Route.** M1 (USB-C controller, undocumented), M2 (Schwung bridge to the plugin on a
   computer), or M3 (VDrum running on the Move)? M3 is the most capable and M2 the closest to
   today's setup. I recommend checking M3's feasibility first, by building `lxr-core` for aarch64
   and measuring render time per 128-frame block.
2. **Record button:** unused, or bring back live recording on the Move?
3. **Capture and Sample buttons:** which functions?
4. **Merge Mix + FX and Perform on the Move?**
5. **Pad row 4 and pad pressure:** which of the proposals in 3.4?
6. **Do you have a Move with Schwung installed?** That would allow checking the encoder LEDs, the
   Loop CC and the overtake limits listed as unverified above.
