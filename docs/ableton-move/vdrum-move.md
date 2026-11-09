# VDrum on Ableton Move

VDrum runs on Move as a Schwung (1.9.2) takeover module: the engine, the sequencer and every mode of
VDrum's controller surface, on Move's pads, buttons and knobs. Design and status:
[schwung-module-plan.md](schwung-module-plan.md). Code: `crates/vdrum-move`.

Status: tested off the device only (x86_64, and aarch64 under qemu). Not yet run on a Move.

## Build and install

1. Build: `./tools/package-move.sh` (needs `rustup target add aarch64-unknown-linux-gnu` and
   `aarch64-linux-gnu-gcc`; on Debian/Ubuntu the package `gcc-aarch64-linux-gnu`). Output:
   `dist/vdrum-module.tar.gz`. CI also builds it (artifact `vdrum-move-module` of the `move` job).
2. Copy to the Move and unpack into the overtake modules folder:
   ```
   scp dist/vdrum-module.tar.gz root@move.local:/tmp/
   ssh root@move.local 'tar -xzf /tmp/vdrum-module.tar.gz -C /data/UserData/schwung/modules/overtake/'
   ```
   The user and host name depend on how SSH was set up on the Move; they are not checked here.
   Unpacking over an existing install keeps `kits/`, `patterns/` and `state/`. Schwung Manager may
   replace the whole folder on install: copy `kits/` and `patterns/` off first.
3. Rescan modules or restart Schwung, then open VDrum from the overtake menu.

Back puts VDrum in the background (it keeps playing while Move is used); Shift + Back quits it.
Shift + Volume touch + wheel click is Schwung's emergency exit.

## Controls

VDrum's modes are those of the Launch Control XL 3 surface (`crates/vdrum-surface`); Move's
controls stand in for the LCXL3's:

| Move | Does |
|---|---|
| Track buttons 1–3 | The 8 knobs control knob row 1, 2 or 3 of the mode |
| Track button 4 | The 8 knobs move the mode's faders 1–8 |
| Pads, row 1 (bottom), 1–7 | Play voices 1–7 (Drum 1–3, Snare, Cymbal, Closed hat, Open hat) |
| Pads, row 1, pad 8 | Accent: held, pads play at full velocity |
| Mute + row 1 pad | Mute or unmute the track |
| Shift + Mute + row 1 pad | Solo the track (again: unmute all) |
| Pads, rows 2 and 3 | The mode's bottom and top button rows (toggles, triggers, macros…) |
| Pads, rows 3 and 2, sequencer view | Patterns 1–8 and 9–16 (queued to the end of the bar while playing) |
| Pads, row 4 (top), 1–7 | Select the sequencer track; in a voice mode also that voice's mode |
| Step buttons | Modes 1–16 (lit: modes that exist; orange: current) |
| Step buttons, sequencer view | Steps of the selected track in the shown bar; hold a step + knobs: locks |
| Shift + step | Mode 1–16 (also in the sequencer view) |
| Play | Stop the running sequencer; when stopped: start it (mode 8) or play the voice (voice modes) |
| Loop | Sequencer view on/off in any mode; Shift + Loop: keep it on across modes |
| Left / Right | Previous / next bar (sequencer view); with Shift: browse kits |
| Up / Down | Next / previous sequencer track |
| Delete | Sequencer view: hold + step clears the step; hold + Up/Down clears the track's pattern |
| Menu | Global mode; Menu again: back to the previous mode |
| Undo | Undo the last parameter change; Shift + Undo: redo |
| Copy | Copy the voice (voice modes); Shift + Copy (hold 1 s): paste |
| Capture | Save the kit file, the patterns and the session |
| Jog wheel | Browse the kits in `kits/`; click (or Play) loads the one shown |
| Shift + jog wheel | Tempo |

Display: line 1 is the mode and the knob bank (`K1`–`K3`, `FDR`); lines 2–4 the surface's screen
(or the value of the control just moved); line 5 tempo, pattern, play state and mutes
(`*` playing, `-` muted).

USB-A MIDI in plays the voices on the plugin's default notes, channel 1: 36, 45, 50, 38, 49, 42, 46.

## Files

In `/data/UserData/schwung/modules/overtake/vdrum/`:

| Folder | Contents |
|---|---|
| `kits/` | Kits as VDrum `.snd` files, the desktop plugin's format. Copy kits here to browse them; Capture saves the current kit to its file, or to a new `NN-NAME.SND`. |
| `patterns/` | Patterns as LXR-02 `PatV03` files, `NN-NAME.PAT` with `NN` = pattern number − 1 (as the desktop plugin exports them). Written for every non-empty pattern on save. A `.PAT` copied in that is newer than `state/seq.txt` replaces its pattern on the next start. |
| `state/` | The session: the edited kit, all sequencer data (`seq.txt`, lossless: `.PAT` cannot hold track densities and Euclid settings), surface settings, snapshots and macros, faders, mutes. |

The session is saved 10 s after the last change, when the module is unloaded and on Capture.

## Diagnostics

`get_param` keys of the DSP (for example from a test `ui.js`): `perf` (render time per block in
µs: last, mean, max), `dropped` (control messages or commands lost to full queues), `ui` (the
frame `ui.js` draws).
