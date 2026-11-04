# Plan: editor tabs, separate hi-hat columns, kit library

Status: draft for review (2026-11-04). Nothing in this plan is implemented yet.

This plan covers three requested changes to the VDrum plugin editor (the "VST view"):

1. The closed and open hi-hats get their own columns.
2. Global tabs: tab 1 is the current parameter view. Tabs 2 to 14 each show one Launch Control XL 3 mode on a picture of the front panel, kept in sync with the hardware.
3. A kit library: several kits loaded into the plugin, saved with the plugin state, and selectable from the editor and from the controller.

Related documents: [design.md](design.md) (the LCXL3 DAW-mode design), [control-tables.md](control-tables.md), [../ARCHITECTURE.md](../ARCHITECTURE.md).

## Current state (checked in the code on main, 7c8ec8d)

- **Editor:** `plugin/src/editor.rs` (egui).
  - It draws six voice cards and a global column.
  - The sixth card is shared by the two hi-hats, with a Closed/Open selector (`Data::hat_tab`).
  - Each card's rows are `ROW_W` = 180 logical px wide, so a card is 190 px plus a 5 px gap.
  - The default window is 1392 × 790 px. The test `default_size_fits_laptop_screens` requires it to fit a 1470 × 956 display.
- **Controller:** `plugin/src/controller.rs` runs `vdrum_surface::Surface` on its own thread.
  - The surface keeps the following state privately: the current mode, held buttons (`held`, `held_steps`), Shift, Accent, record arm, kit browsing, macro learning and the popup.
  - `Surface::frame()` returns LED colours, the three stationary screen lines and the popup (title, name, value).
  - None of this reaches the editor today. The editor and the controller share only the parameter mirror (`Shared::values`), the pad hit counters, mutes, the sequencer data and the settings text.
- **Kits:**
  - The editor loads and saves one `.snd` file at a time.
  - On the controller, Page up/down browses the `.snd` files in the folder of the current kit file, and Play loads the browsed kit (`Session::browsed`, `kit_files()`).
  - The plugin state stores the current kit and its path, not a list of kits.
- **Uploaded zip (`Hrtl_LXR-02_DLC2.zip`):**
  - It contains no `.syx` files. It holds two LXR-02 project folders, `DLCkits (kits only)` and `DLCprjct (w demo patterns)`.
  - Each folder has 64 `.SND` kit files, 64 `.PAT` pattern files, 64 `.SNG` song files, `GLO.CFG` and `PRJ.NFO`.
  - 16 of the kits are named: `00-LAMBO` to `15-MIST_`. The other 48 are `xx-INITK` (init kit).
  - Every `.SND` is 255 bytes: an 8-byte name plus 247 sound parameters. This is the firmware kit format that `Kit::from_file_bytes` already reads.
  - The kit files are identical in both folders (`00-LAMBO.SND` compared byte for byte).
  - The `.PAT` files hold the demo patterns. Importing them is not part of this request; see open question 6.

## Part A: separate hi-hat columns

- **Seven voice cards:** `update()` draws seven voice cards (Drum 1, Drum 2, Drum 3, Snare, Cymbal, Closed hi-hat, Open hi-hat) instead of six, and the Closed/Open selector is removed.
  - Each hi-hat card shows its own pad and trigger note: CH on the closed card, OH on the open card. Today the shared card shows both pads.
- **Choke switch:** the "closed hat chokes open hat" switch (`OH_CHOKE`) appears on both cards. It stays one parameter, as on the controller.
- **Fitting the extra card:** seven cards plus the global column need about 1581 px at the current widths, which does not fit the 1470 px laptop width the tests require. There are three options (open question 1):
  - **a. Narrower rows (recommended).** `ROW_W` goes from 180 to 150 px and `ROW_FONT` from 11 to 10. Seven cards are then about 1365 px, so the default size stays near 1392 px. Row labels that no longer fit are shortened, and the test `row_labels_are_spelled_out` is adapted to the new width.
  - **b.** Keep the widths. The default window becomes about 1590 px wide, and smaller screens use the existing zoom or scroll.
  - **c.** Two rows of cards, which needs a taller window.
- **Tests:** `every_host_parameter_is_on_screen` must still pass, and `default_size_fits_everything` is re-checked for whichever option is chosen.

## Part B: global tabs and the LCXL3 views

### B1. Tab bar

- **Tabs:** a tab bar under the top bar has 14 tabs.
  - Tab 1 is "Params", the parameter view as it is after Part A.
  - Tabs 2 to 14 are the 13 assigned modes in slot order, named as on the controller: Drum 1, Drum 2, Drum 3, Snare, Cymbal, Closed hi-hat, Open hi-hat, Sequencer, Mix + FX, Perform, LFO + Mod, Morph + Macro, Global.
  - Modes 13 to 15 have no function, so they get no tab.
- **Saving:** the selected tab is saved with the editor state.

### B2. Sharing the controller's state with the editor

A new `vdrum_surface::View` struct is a snapshot of everything the editor needs to draw a mode tab. `Surface::view(now, model)` builds it, so the editor never runs surface logic itself. It contains:

- the current mode;
- for every control of the current mode, its label, its section colour, its LED colour (from `frame()`) and its current value as text. These come from the same code as the screen, so `lxr_params::format_value` is used;
- for faders, the parameter value and whether the fader has picked it up yet, with the direction to move;
- the active context layers: Shift held, Solo/Arm group layer, Mute/Select pattern layer, held sequencer steps (step numbers), Accent held, Record armed, kit browsing, macro learn (which slot), and a button hold in progress (for example "hold 1 s to save");
- the popup: title, parameter name, value, section colour, and a generation counter;
- the stationary screen lines.

The data flows through `Shared` in both directions:

- **Controller to editor:** the controller thread publishes the snapshot in `Shared::surface_view` (`Mutex<Arc<View>>`) at the frame rate it already uses (`FRAME_MS` = 20 ms), but only when the snapshot has changed. The editor reads the latest snapshot on each repaint.
- **Editor to controller:** a new `Shared::editor_requests` queue carries mode changes from the tab bar (`SetMode(Mode)`), so the hardware follows the editor. Control edits made in a mode tab use the normal parameter path.
- **No device connected:** the controller thread still runs the `Surface` (it is pure state), so the mode tabs work, show values and accept edits. A "Controller: not connected" chip is shown in the top bar. Today the thread only scans for the device, so this is a change in `controller::run`.

### B3. Front-panel drawing

- **Vector drawing:** each mode tab draws the LCXL3 front panel with the egui painter: the three rows of 8 encoders, the 8 faders, the 16 buttons, the left-hand buttons and the screen.
  - The rendered PNGs in `images/` are not reused. They are built from a photo that is not in the repository, they would add about 17 MB to the plugin binary, and they cannot show live values.
  - The proportions and the label positions follow `source/render.py`, so the look matches the design images: dark panel, the label above each control, and the section colour.
- **Controls:**
  - Encoders show a value arc in their LED colour.
  - Faders show the cap at the hardware position, and a marker at the parameter value while the fader has not picked it up.
  - Buttons show their LED colour and brightness, as on the device.
  - The screen area shows the stationary lines, or the popup while one is active.
- **Mouse editing:**
  - Encoders and faders can be dragged, and a double-click resets to the default.
  - Buttons can be clicked, and held where the binding is a hold action.
  - Each mouse action goes through the same `Surface` handlers as the hardware, through the request queue in `Shared` as a synthetic `lcxl3::Input`, so behaviour is identical.
  - Open question 3 asks whether mouse editing in the mode tabs is wanted at all.
- **Hovering** a control shows its parameter ID and full name, as in the control tables.

### B4. "Now editing" pill and context pills

- **Position:** a row at the top of the editor, on every tab, shows the following pills.
- **Edit pill:** the parameter or setting being changed, as a filled pill in the section colour, with the text "Section · Name · Value" (for example "FILTER · Cutoff · 84").
  - It stays for 1.5 s after the last change. This timer is new and runs in the editor; the surface itself has no popup timer, because the device times out its own popups (CC 113).
  - It is set by hardware edits, and also by edits made in the editor (open question 4).
- **Context pills:** each active layer from B2 is an outlined pill that stays while the layer is active. Examples: "SHIFT: select mode", "TRACK GROUP: D1 D2", "PATTERN: select", "STEP 5 HELD: locks", "ACCENT", "REC", "BROWSING KITS: 03 MALWA", "LEARNING MACRO 3".

### B5. Following the hardware

- **Mode follows the hardware:** when the hardware changes mode, the editor switches to that mode's tab.
- **Params tab exception:** while the Params tab is shown, a hardware mode change does not leave it unless "Follow controller" is on. This is a checkbox next to the tabs, on by default (open question 2).
- **Tab follows the editor:** clicking a mode tab sets the hardware mode. Clicking Params leaves the hardware mode unchanged.
- **Values:** all values come from the shared parameter mirror, so host automation, editor edits, kit loads and controller edits appear in every view.

## Part C: kit library

### C1. Data

- **Library:** `Shared::kit_library` is a `Mutex<KitLibrary>`, a list of up to 128 entries. Each entry holds the kit file bytes (the 255-byte firmware part plus VDrum's open hi-hat extension) and its source label (the file name).
- **State:** the library and the index of the selected kit are saved with the plugin state as a new persisted field `kit-library`, encoded like the existing kit field. Each entry is 297 bytes (255 + 5 + 37 open hi-hat values), so 128 kits take about 38 KB of state.
- **Old states:** a state without the field loads with an empty library and the current kit unchanged.

### C2. Import

- **Editor buttons:** "Import kits…" opens a multi-file dialog that accepts `.snd` files and `.zip` archives.
  - Zip archives are read in memory with the `zip` crate, which would be a new dependency (open question 5). Every `.snd` inside is imported, and `__MACOSX` entries are skipped.
  - "Import folder…" imports every `.snd` in a folder.
- **Duplicates:** kits identical to one already in the library are skipped.
- **Init kits:** kits whose parameters equal the init kit (the 48 `INITK` slots in the DLC zip) are skipped by default (open question 6).
- **Result:** the status line reports the outcome, for example "Imported 16 kits, skipped 48 init kits and 16 duplicates" for the uploaded zip.
- **Library panel:** a panel in the global column (or a "Kits" pop-over from the top bar) lists the kits with name and source.
  - Click selects a kit and loads it.
  - The panel can also rename, delete and move kits up or down, save the current kit as a new entry, overwrite the selected entry with the current kit, and export an entry as `.snd`.

### C3. Selecting a kit

- **Editor:** click in the library panel, or use the previous/next buttons in the top bar.
- **Controller:** Page up/down browses the library instead of the folder of the current kit file. The screen shows "Kit 3/16 · MALWA", and Play loads the kit, as now.
  - When the library is empty, the current folder browsing stays.
- **Loading:** loading a library kit uses the existing `Command::LoadKit` path, so the engine, the editor and the controller all follow it.
- **Unsaved edits:** edits to a loaded kit are not written back to the library automatically. "Compare" (Global mode) compares with the library entry. "Save kit" (Global mode, hold 1 s) overwrites the selected library entry; the `.snd` file is written as well only if the kit was loaded from a file (open question 7).

## Delivery

One pull request per part, each with its tests and with the docs updated:

| PR | Content | Tests |
| --- | --- | --- |
| 1 | Part A: seven voice cards | existing editor tests adapted; window-fit tests |
| 2 | Part C: kit library, import (`.snd`, folder, `.zip`), editor panel, controller browsing, state | import of the uploaded zip's layout (16 kits, 48 init kits skipped); state round trip; Page up/down over the library |
| 3 | Part B, data: `View` snapshot, context layers, editor → controller requests, surface running without a device | surface unit tests for `view()` in each mode and context layer |
| 4 | Part B, editor: tab bar, front-panel drawing, pills, follow mode | snapshot of the layout geometry; every control of every mode has a label on its tab |

Docs to update in the same PRs: `docs/lcxl3-daw-mode/design.md` (an addendum on the editor tabs and the kit library), `docs/ARCHITECTURE.md` (the new shared state), and the plugin `README.md`.

## Not verifiable here

- Rendering, layout and readability of the new views need a look on a real screen. Only the layout tests and headless egui runs can be done here.
- Hardware-to-editor sync timing (20 ms frames) needs the device. These items join the phase 8 hardware checklist.

## Open questions

1. **Seven cards:** narrower rows (recommended), a wider default window, or two rows of cards?
2. **Following the hardware:** should a hardware mode change also leave the Params tab? The proposal is a "Follow controller" checkbox, on by default.
3. **Mouse editing:** should the mode tabs accept mouse edits (drag knobs and faders, click buttons), or only display?
4. **Edit pill:** should edits made with the mouse in the editor also show it, or only hardware edits?
5. **Zip import:** may the `zip` crate be added as a dependency? Without it, the zip has to be extracted first and the `.snd` files or the folder imported.
6. **Init kits and patterns:** should init kits be skipped on import? And should the `.PAT` demo patterns from the zip also be imported into the sequencer? That would be separate work.
7. **Save kit:** should it overwrite the library entry only, or also the `.snd` file it came from?
