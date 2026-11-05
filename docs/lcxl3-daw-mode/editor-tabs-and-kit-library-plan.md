# Plan: editor tabs, separate hi-hat columns, kit and pattern library

**Status:** revision 2 (2026-11-04), with Sami's answers to the first draft's questions. Implemented and merged on 2026-11-04: PR #12 (host CCs), #13 (Part A), #14 and #15 (Part C), #16 and #17 (Part B). Hardware checks are still open (see *Not verifiable here*).

This plan covers three requested changes to the VDrum plugin editor:

1. The closed and open hi-hats get their own columns.
2. Global tabs: tab 1 is the current parameter view. Tabs 2 to 14 each show one Launch Control XL 3 mode on a drawing of the front panel, in sync with the hardware.
3. A library of kits (and patterns), saved with the plugin state and selectable from the editor and from the controller, with import and an LXR-02-compatible export.

Related documents: [design.md](design.md) (the LCXL3 DAW-mode design), [control-tables.md](control-tables.md), [../ARCHITECTURE.md](../ARCHITECTURE.md).

## Decisions (Sami, 2026-11-04)

| # | Question | Answer |
| --- | --- | --- |
| 1 | How do seven voice cards fit? | Wider window. |
| 2 | Does a hardware mode change leave the Params tab? | Yes, through a "Follow controller" checkbox that is on by default. |
| 3 | Mouse edits in the mode tabs? | Allowed. |
| 4 | Which edits show the "now editing" pill? | Hardware edits only. |
| 5 | Add the `zip` crate? | No. Zip archives are not imported; they are extracted first. The uploaded DLC kits are copyrighted and are not committed to the repository, not even as test data. |
| 6 | Init kits and `.PAT` patterns? | Init kits are skipped on import. When an import contains `.PAT` files, the editor asks whether to import them. Kits and patterns can also be imported separately. |
| 7 | What does *Save kit* overwrite? | The library entry only. An export writes all current kits, patterns and data in an LXR-02-compatible format. |

## Current state (checked in the code on main, 7c8ec8d)

- **Editor:** `plugin/src/editor.rs` (egui).
  - It draws six voice cards and a global column. The sixth card is shared by the two hi-hats, with a Closed/Open selector (`Data::hat_tab`).
  - Each card is 190 px plus a 5 px gap. The default window is 1392 × 790 logical px.
  - The test `default_size_fits_laptop_screens` requires the window to fit a 1470 × 956 display.
- **Controller:** `plugin/src/controller.rs` runs `vdrum_surface::Surface` on its own thread.
  - The surface keeps private state: the current mode, held buttons (`held`, `held_steps`), Shift, Accent, record arm, kit browsing, macro learning and the popup.
  - `Surface::frame()` gives LED colours, the three stationary screen lines and the popup (title, name, value). The surface has no popup timer; the device times its popups out (CC 113).
  - None of this reaches the editor today. The editor and the controller share only the parameter mirror, the pad hit counters, the mutes, the sequencer data and the settings text.
- **Kits:**
  - The editor loads and saves one `.snd` file at a time.
  - On the controller, Page up/down browses the `.snd` files in the folder of the current kit file.
  - The plugin state stores the current kit and its path.
- **Sequencer:** `vdrum_surface::SeqData` has 64 patterns of 7 tracks × 64 steps, the same shape as the firmware (`NUM_PATTERN` = 64, `NUM_TRACKS` = 7, `NUM_STEPS` = 64). The plugin sets `PAR_PATTERN_MODE` to 0, so a pattern change does not load a kit.

## LXR-02 file formats

The LXR-02 firmware source is the `third_party/lxr-firmware` submodule, `Core/Src/Preset/presetManager.c`. The formats below were read from that file and checked against the sizes of the files in the uploaded zip. The uploaded files are used only locally and are not committed.

- **Project folder:** 64 slots, `00` to `63`. Each slot has a kit `NN-NAME.SND`, a pattern `NN-NAME.PAT` and a song `NN-NAME.SNG`. The folder also holds `GLO.CFG` (global parameters) and `PRJ.NFO` (project name).
  - The firmware loads a slot by wildcard `NN-*.ext` and saves with the kit or pattern name (`preset_makeFileNameLoad`, `preset_makeFileNameSave`).
- **`.SND` kit:** 255 bytes, an 8-byte name plus the 247 sound parameters. `Kit::from_file_bytes` already reads this format.
- **`.PAT` pattern:** one pattern per file, 3663 bytes. The file holds:
  - an 8-byte name, then the tag `PatV03`;
  - for each of the 7 tracks: 64 steps of 7 bytes, then 64 step-timing bytes;
  - the main-step bits: 7 tracks × 4 bars × 2 bytes;
  - the shuffle (1 byte), the track lengths (7 bytes) and the pattern's kit number (1 byte).
  - The sizes add up to 3663 bytes, the size of every `.PAT` in the zip.
  - Reading code: `preset_loadPatternSD` and the data reader. Writing code: `preset_savePatternSD` and `preset_writePatternData`.
- **`.SNG`, `GLO.CFG`, `PRJ.NFO`:** written by `preset_saveSong…`, `preset_saveGlobals` and `preset_saveAll`. Their exact layout is read from the same file during implementation.
- **`.syx`:** the request mentions `.syx` files, but the uploaded zip contains none. The firmware's MIDI parser ignores SysEx data (`MidiParser.c`: "we don't interpret sysex data right now"), so there is no LXR-02 SysEx kit format to support. This plan treats kit files as `.SND`; Sami confirmed this (see *Open questions*).
- **Uploaded zip (`Hrtl_LXR-02_DLC2.zip`):** two project folders, `DLCkits (kits only)` and `DLCprjct (w demo patterns)`.
  - Each folder has 64 `.SND`, 64 `.PAT` and 64 `.SNG` files plus `GLO.CFG` and `PRJ.NFO`.
  - 16 kits are named, `00-LAMBO` to `15-MIST_`; the other 48 are `INITK`.
  - The kit files are identical in both folders.

## Part A: separate hi-hat columns

- **Seven cards:** the editor draws seven voice cards: Drum 1, Drum 2, Drum 3, Snare, Cymbal, Closed hi-hat and Open hi-hat. The Closed/Open selector is removed.
  - Each hi-hat card shows its own pad and trigger note.
  - The choke switch (`OH_CHOKE`, "closed hat chokes open hat") appears on both cards and stays one parameter.
- **Window size (decision 1):** row widths stay as they are, and the default window grows to about 1590 × 790 px.
  - A 1470 px laptop screen needs zoom (90 %) or horizontal scrolling.
  - The test `default_size_fits_laptop_screens` changes to "fits a 1728 × 1117 display (MacBook Pro 16") at 100 % and a 1470 × 956 display at 90 % zoom".
  - Saved states with the old default size (1392 × 790) migrate to the new one, the same way `LEGACY_SIZE` migrates today.
- **Tests:** `every_host_parameter_is_on_screen` must still pass, and the window-fit tests are updated.

## Part B: global tabs and the LCXL3 views

### B1. Tab bar

- **Tabs:** a tab bar under the top bar has 14 tabs.
  - Tab 1 is "Params", the parameter view after Part A.
  - Tabs 2 to 14 are the 13 assigned modes in slot order: Drum 1, Drum 2, Drum 3, Snare, Cymbal, Closed hi-hat, Open hi-hat, Sequencer, Mix + FX, Perform, LFO + Mod, Morph + Macro, Global.
  - Modes 13 to 15 have no function, so they get no tab.
- **Saving:** the selected tab is saved with the editor state.

### B2. Sharing the controller's state with the editor

A new `vdrum_surface::View` struct is a snapshot of everything a mode tab shows. `Surface::view(now, model)` builds it, so the editor never runs surface logic itself. It holds:

- the current mode;
- for each control of the mode: its label, section colour, LED colour, current value as text (`lxr_params::format_value`, the same text as the screen), and, for faders, the value, the hardware position and the pickup state;
- the active context layers: Shift, the Solo/Arm track-group layer, the Mute/Select pattern layer, held sequencer steps, Accent, Record armed, kit browsing (the browsed kit), macro learn (the slot) and a hold in progress (for example "hold 1 s to save");
- the last hardware edit: section, parameter name, value and section colour, with a counter that increases with every edit;
- the stationary screen lines.

The data moves through `Shared`:

- **Controller to editor:** the controller thread publishes the snapshot in `Shared::surface_view` every frame (`FRAME_MS` = 20 ms) in which it has changed.
- **Editor to controller:** a new queue `Shared::surface_input` carries tab clicks (set mode) and mouse edits (B3).
- **No device connected:** the controller thread runs the `Surface` even when no device is connected, so the tabs work, show values and accept mouse edits. A "Controller: not connected" chip in the top bar says so. Today the thread only scans for the device until one appears; that changes in `controller::run`.

### B3. Front-panel drawing and mouse edits

- **Drawing:** each mode tab draws the LCXL3 front panel with the egui painter: three rows of 8 encoders, 8 faders, 16 buttons, the left-hand buttons and the screen.
  - The PNGs in `docs/lcxl3-daw-mode/images/` are not used. They are built from a photo that is not in the repository, they would add about 17 MB to the plugin, and they cannot show live values.
  - The proportions and label positions follow `source/render.py`: a dark panel, the label above each control, and the section colour.
- **Controls:**
  - Encoders show a value arc in their LED colour.
  - Faders show the cap at the hardware position, and a marker at the parameter value while the fader has not picked it up.
  - Buttons show their LED colour.
  - The screen shows the stationary lines.
- **Mouse edits (decision 3):** mouse actions are sent to the surface as synthetic `lcxl3::Input` events through `Shared::surface_input`. They run through the same handlers as the hardware, so the behaviour is identical.
  - Dragging an encoder sends relative steps.
  - Dragging a fader sends absolute values, which pick up the parameter immediately.
  - Clicking a button sends press and release. Holding the mouse button holds the control, which covers step-held edits and 1 s holds.
  - Double-clicking a control resets its parameter to the default through the normal parameter path.
- **Hovering** a control shows its parameter ID and full name.

### B4. "Now editing" pill and context pills

- **Position:** a row at the top of the editor, on every tab.
- **Edit pill (decision 4):** the last hardware edit, as a pill filled with the section colour, reading "Section · Name · Value" (for example "FILTER · Cutoff · 84").
  - It stays for 1.5 s after the last change. The timer is new and runs in the editor.
  - Only edits from the hardware set it. Mouse edits made in the mode tabs go through the surface but are marked as editor input and do not set it.
- **Context pills:** each active layer from B2 shows as an outlined pill for as long as it is active. Examples: "SHIFT: select mode", "TRACK GROUP: D1 D2", "PATTERN: select", "STEP 5 HELD: locks", "ACCENT", "REC", "BROWSING: 03 MALWA", "LEARNING MACRO 3".

### B5. Following the hardware

- **Hardware to editor (decision 2):** when the hardware changes mode, the editor switches to that mode's tab. With "Follow controller" on (the default), this also happens while the Params tab is shown. The checkbox sits next to the tabs and is saved with the editor state.
- **Editor to hardware:** clicking a mode tab sets the hardware mode. Clicking Params leaves the hardware mode unchanged.
- **Values:** all values come from the shared parameter mirror, so host automation, editor edits, kit loads and controller edits appear in every view.

## Part C: kit and pattern library

### C1. Data

- **Kit slots:** the library has 64 kit slots, numbered `00` to `63`, the same as an LXR-02 project. Each slot is empty or holds a kit and its source file name.
  - A kit is stored as VDrum's kit file bytes: the 255-byte firmware part plus the open hi-hat extension, 297 bytes in all.
- **Patterns:** the sequencer's 64 patterns gain a name each (8 characters, as in `.PAT`) and the kit number from the `.PAT` file. The kit number is stored for export; the plugin does not load kits on pattern change (`PAR_PATTERN_MODE` = 0).
- **State:**
  - The library, the selected slot and the pattern names are saved with the plugin state in a new persisted field, `kit-library`. 64 kits take about 19 KB.
  - A state without the field loads with an empty library. The current kit and the patterns are unchanged.

### C2. Import (no zip support, decision 5)

The top bar gets a "Library" menu with these entries:

- **Import kits…** opens a multi-file dialog for `.snd` files.
- **Import patterns…** opens a multi-file dialog for `.pat` files.
- **Import LXR-02 project folder…** reads a folder laid out like the uploaded project folders.
  - Its kits go into the slots named by their file numbers (`NN-`).
  - When the folder contains `.PAT` files, a dialog asks "Also import N patterns? This replaces patterns NN to NN in the sequencer." with Import and Skip buttons (decision 6).
- **Where files go:**
  - A file with a number prefix (`NN-`) goes into that slot or pattern.
  - A file without one goes into the first empty kit slot, or the first empty pattern.
  - If a target is not empty, a confirmation lists what will be replaced.
- **Skipped on import (decision 6):**
  - init kits, meaning kits whose parameters equal the init kit, as the 48 `INITK` slots in the zip are;
  - empty patterns, meaning patterns with no step set;
  - kits that are identical to one already in the library.
- **Report:** the status line reports the result, for example "Imported 16 kits (48 init kits skipped), 16 patterns".

### C3. Library panel and selecting a kit

- **Library panel:** a pop-over from the "Library" menu lists the 64 slots.
  - Click a slot to load its kit.
  - Other actions: rename a slot, clear it, move it up or down, store the current kit in an empty slot, and export one kit as `.snd` or one pattern as `.pat`.
- **Editor:** previous and next buttons next to the kit name in the top bar step through the non-empty slots and load the kit.
- **Controller:**
  - Page up/down browses the non-empty slots instead of the folder of the current kit file. The screen shows "Kit 03 · MALWA", and Play loads it, as now.
  - When the library is empty, the old folder browsing stays.
- **Loading** uses the existing `Command::LoadKit` path, so the engine, the editor and the controller all follow.
- **Save kit (decision 7):** *Save kit* (the editor button, and Global mode "hold 1 s") writes the current kit into the selected library slot only.
  - "Save…" to a `.snd` file stays as a separate editor action.
  - *Compare* (Global mode) compares with the library slot.

### C4. LXR-02-compatible export (decision 7)

"Export LXR-02 project…" writes a folder an LXR-02 can load from its SD card:

- **Kits:** `NN-NAME.SND` for all 64 slots, in the 255-byte firmware format. Empty slots are written as the init kit named `INITK`, as in the uploaded projects.
- **Patterns:** `NN-NAME.PAT` for all 64 patterns, in the `PatV03` format.
- **Songs:** `NN-NAME.SNG`. VDrum has no song mode, so these are written as the firmware's empty song (`SONG_END`), the way the firmware clears songs.
- **`GLO.CFG`:** the global parameters from the current state (`PAR_BEGINNING_OF_GLOBALS` to `NUM_PARAMS`).
- **`PRJ.NFO`:** the project name, which defaults to the current kit name.
- **Single files:** "Export kit…" and "Export pattern…" write one file each.

An export loses data the LXR-02 cannot hold. The export dialog lists these losses:

- **The open hi-hat voice.** Its decay is written to the firmware's open-hat decay `velod6_open`, and its other 36 parameters are dropped, because the LXR-02 has one hi-hat voice. The same mapping is used in reverse on import (`oh_from_hat`).
- **Sequencer extras:** densities and the Euclid settings are not part of `.PAT`. The steps they produce are exported as they are.
- **Controller state:** macros, snapshots and the LCXL3 settings.

## Testing

- **Test data:** the copyrighted DLC files are not committed. Tests build synthetic `.SND`, `.PAT` and project folders in the same formats.
- **Round trips:** byte-for-byte export of an imported project, for kits and patterns.
- **Real files:** a local check, not committed, that importing the uploaded zip's project folder and exporting it again gives byte-identical `.SND` and `.PAT` files. The result is reported in the PR.
- **Other tests:** state round trip, Page up/down over the library, the init-kit and empty-pattern skipping, and the slot assignment from file names.

## Delivery

One pull request per part. Each updates the docs: an addendum to `design.md`, `docs/ARCHITECTURE.md` and the plugin `README.md`.

| PR | Content |
| --- | --- |
| 1 | Part A: seven voice cards and the wider window |
| 2 | Part C: library, imports, panel, controller browsing, state |
| 3 | Part C: LXR-02 `.PAT` and project-folder import and export |
| 4 | Part B, data: `View` snapshot, context layers, editor input to the surface, surface running without a device |
| 5 | Part B, editor: tab bar, front-panel drawing, mouse edits, pills, follow mode |

## Not verifiable here

- The look and readability of the new views need a real screen. Only the layout tests and headless egui runs can be done here.
- Whether an LXR-02 accepts the exported project folder needs the device.
- Hardware-to-editor sync timing needs the LCXL3.
- These items will be added to `/mnt/project-files/lcxl3-daw-mode/hardware-test-checklist.md`.

## Open questions

None open.

1. **`.syx` files (resolved 2026-11-04):** the zip has no `.syx` files, and the LXR-02 firmware has no SysEx kit format. Sami confirmed that "`.syx`" meant the `.SND` kit files. Kits import on their own with *Import kits…* (`.SND`), patterns on their own with *Import patterns…* (`.PAT`), and both together from a project folder.
