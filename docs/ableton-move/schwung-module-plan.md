# VDrum as a Schwung module on Ableton Move: implementation plan

Status: revision 3 (2026-11-09). Phases 1 to 3 are implemented in `crates/vdrum-move` and tested
off the device (x86_64, and aarch64 under qemu); nothing has run on a Move yet (phase 0 was
skipped by decision 10). Usage and install: [vdrum-move.md](vdrum-move.md). This is route M3 ("VDrum running on the Move") of
[mapping-brainstorm.md](mapping-brainstorm.md) (in PR #27). The control mapping itself is in that
document and is not repeated here.

## 0. Decisions (Sami, 2026-11-09)

| # | Question | Decision |
|---|---|---|
| 1 | Device | Sami has a Move with Schwung **1.9.2**. The Schwung source read for this plan is commit `673e67a`, which is the `v1.9.2` tag. |
| 2 | Module form | VDrum runs next to stock Move as a Schwung **takeover (overtake) module**: it owns the whole surface while on screen, and Move keeps running underneath. The chain sound generator form of revision 1 is dropped. |
| 3 | MIDI notes | Left to this plan. Decision: keep the desktop plugin's defaults, General MIDI drum notes on channel 1: 36, 45, 50, 38, 49, 42, 46 for tracks 1 to 7 (`DEFAULT_TRACK_NOTES` in `plugin/src/params.rs`). In takeover mode the pads arrive as pad ids, not notes, so this map applies only to external MIDI on USB-A. |
| 4 | Sequencer | VDrum owns the sequencer (its own clock, tempo, patterns), as on the LCXL3. |
| 5 | Aux output (DAC2) | Ignored for now: summed into the stereo output. |
| 6 | Code location | A separate crate, `crates/vdrum-move`. Revised by decision 8. |
| 7 | Distribution | Sami's own use only. No catalog entry. |
| 8 | Sharing (revision 3) | "As modular as possible while reusing as much shared code as possible": a shared crate. `crates/vdrum-engine` (extracted from `plugin/`) holds the engine wrapper, sequencer data, kit library, LXR file formats and defaults; the plugin, `vdrum-surface` and `vdrum-move` all use it. `vdrum-move` reuses VDrum's surface (`vdrum-surface`, all modes and features) through an adapter instead of a separate Move mapping. |
| 9 | Storage (revision 3) | Folders `kits/` and `patterns/` inside the module's folder on Schwung, in the desktop plugin's formats (VDrum `.snd`, LXR-02 `PatV03` `.PAT`). |
| 10 | Phase 0 (revision 3) | Skipped: implement the module directly. |

## 1. Summary

- VDrum's engine fits Schwung's audio contract without resampling: both run at 44100 Hz with int16
  stereo output, and one Move block (128 frames) is exactly two engine ticks (64 frames each).
- `lxr-core` builds for `aarch64-unknown-linux-gnu` unchanged and, under qemu, reproduces all 299
  kit-free reference scenarios of firmware v1.71 bit for bit (section 4). Render time on the
  Move's CPU is not measured yet.
- Form: a Schwung module with `component_type: "overtake"`, a Rust `dsp.so` (engine, sequencer,
  Move surface logic) and a thin `ui.js` (forwards controls the DSP cannot see, draws LEDs and the
  display). `suspend_keeps_js` keeps VDrum playing while the user is back in Move.
- Order: device measurements and a minimal Rust module first (phase 0), then engine and sequencer
  on the device with a minimal surface, then the full mapping (section 8).

## 2. Sources

| Source | What was read |
|---|---|
| Schwung `v1.9.2` (commit `673e67a`, 2026-11-05) | `README.md`, `docs/ARCHITECTURE.md` (first 200 lines), `docs/API.md` (display, host functions, overtake, audio), `docs/MODULES.md` (module structure, `module.json`, capabilities, tool config, Remote UI for overtake tools, threading, `split_voices`, building, voices, overtake modules), `src/host/plugin_api_v1.h` (whole file), `src/schwung_shim.c` (overtake DSP load, render, MIDI delivery, `overtake_midi_send_internal`), `src/shadow/shadow_ui.js` (`loadOvertakeModule` and the `host_module_*` shims), `Dockerfile`, `src/modules/tools/seq-test/module.json`, `src/modules/overtake/rnbo-runner/module.json`. |
| Ableton Move manual | Not re-read; hardware facts come from mapping-brainstorm.md section 1. |
| VDrum `main` @ `b170272` | `docs/ARCHITECTURE.md`, `plugin/src/{engine,pipeline,fp,params,real_engine_tests}.rs`, `crates/lxr-core` tests, `crates/vdrum-surface/src/seq.rs`. |

Claims marked "unverified" were not checked against source or hardware.

## 3. Schwung facts that decide the design

### 3.1 Process and thread model

- Schwung is an `LD_PRELOAD` shim (`schwung-shim.so`) inside Ableton's `MoveOriginal` process. An
  overtake module's `dsp.so` is loaded **into that process** and rendered on the SPI audio
  callback.
- `ui.js` runs in a separate QuickJS process (`shadow_ui`) at about 60 ticks per second. It talks to
  the DSP through `host_module_set_param` / `host_module_get_param` (string key, string value),
  which the host routes to the DSP with the prefix `overtake_dsp:`.
- `plugin_api_v1.h`: "THERE IS NO CONTROL THREAD." `set_param`, `get_param`, `on_midi` and
  `render_block` run on the SPI callback (SCHED_FIFO 70, core 3, about 2370 µs of slack per
  128-frame block, shared with Move and all other modules).
- For an overtake DSP, `dlopen` and `create_instance` run on the shim's worker thread
  (`overtake_dsp_load_body` in `schwung_shim.c`), not on the callback. Allocating the engine in
  `create_instance` is therefore possible; the header still asks that create time be short.
- Forbidden on the callback: file I/O, allocation and free, locks shared with non-RT threads,
  logging (including `fprintf(stderr)`), unbounded work. A violation causes a device-wide dropout.
- Threads created by the module must demote themselves to SCHED_OTHER and pin to cores 0 to 2.

### 3.2 The C ABI (`plugin_api_v2_t`)

```c
plugin_api_v2_t *move_plugin_init_v2(const host_api_v1_t *host);
struct plugin_api_v2 {
    uint32_t api_version;                                   /* 2 */
    void *(*create_instance)(const char *module_dir, const char *json_defaults);
    void  (*destroy_instance)(void *inst);
    void  (*on_midi)(void *inst, const uint8_t *msg, int len, int source);
    void  (*set_param)(void *inst, const char *key, const char *val);
    int   (*get_param)(void *inst, const char *key, char *buf, int buf_len);
    int   (*get_error)(void *inst, char *buf, int buf_len);
    void  (*render_block)(void *inst, int16_t *out_lr, int frames);  /* overwrites, 128 frames */
};
```

The overtake host fills `host_api_v1_t` with `sample_rate`, `frames_per_block`, `log`,
`midi_send_internal`, `midi_send_external`, `get_bpm`, `get_beat_position` and
`midi_inject_to_move` (`overtake_dsp_load_body`). Modules must not extend either struct; the
8-pointer reserved tail must start at offset 120.

### 3.3 What an overtake module with DSP can and cannot do

| Topic | Fact | Source |
|---|---|---|
| Declaring it | `component_type: "overtake"`, `ui: "ui.js"`, `dsp: "dsp.so"`, `api_version: 2`. `loadOvertakeModule` loads `basePath + "/" + dsp` before the JS. | `shadow_ui.js` |
| Pads and steps | Cable-0 note on/off with note ≥ 10 (pads 68 to 99, steps 16 to 31) reach the DSP's `on_midi` on the audio thread. | `schwung_shim.c` |
| Encoders, buttons, wheel, pad pressure | Not delivered to the DSP. They reach `ui.js` (`onMidiMessageInternal`). Polyphonic aftertouch is not in the DSP's note filter, so it reaches only `ui.js` (unverified on hardware). | `schwung_shim.c`, `API.md` |
| LEDs | Must be sent from `ui.js` (`setLED`, `setButtonLED`). The DSP's `midi_send_internal` goes to Schwung's chain slots (`shadow_chain_dispatch_midi_to_slots`), **not** to the hardware. The output buffer holds about 64 packets per frame, shared with Move; the docs recommend about 8 LED sends per tick. | `schwung_shim.c`, `API.md` |
| Display | Drawn by `ui.js`: `clear_screen`, `print`, `fill_rect`, `draw_line`, `draw_arc`, `draw_image`, `set_font`, … About 490 ns per JS→C call. | `API.md` |
| JS→DSP transport | `host_module_set_param` is fire-and-forget through one shared-memory slot; the host's own comment says it "can lose params when multiple rapid writes hit the single shared-memory slot". Alternatives: `host_module_set_param_blocking` (waits, 500 ms default timeout) and the bulk `host_module_set_params(blob)` / `host_module_get_params(blob)` (one round trip for many keys). | `shadow_ui.js` |
| Files | `ui.js` has `host_read_file`, `host_write_file`, `host_ensure_dir`, `host_file_exists`. | `API.md`, `shadow_ui.js` |
| Audio | The DSP's output is added (saturating int16) to Schwung's output after the slots. | `schwung_shim.c` |
| Background | `capabilities.suspend_keeps_js`: Back suspends the UI and the DSP keeps rendering; Shift + Back exits. `onResume()` is called on return; LEDs must then be repainted. | `MODULES.md` |
| Escape | Shift + Volume touch + wheel click always exits; the module must not use that combination. | `API.md` |
| Move's Play | Overtake modules receive all buttons. `button_passthrough: [85]` would hand Play back to Move; VDrum does not use it (it owns the sequencer). | `MODULES.md` |

## 4. Feasibility check done for this plan

A scratch program (not committed) ran the scenario corpus `tests/scenarios/` through
`lxr_core::Lxr` directly (events applied before their tick, as `run_direct` in
`plugin/src/real_engine_tests.rs`) and compared SHA-256 hashes with `reference_v1.71.json`.

| Build | Result | Mean time per 64-frame tick |
|---|---|---|
| x86_64, release (`lto = thin`), Intel Xeon @ 2.10 GHz in the cloud container | 299 match, 0 mismatch, 11 skipped (kit loads), 87,288 ticks | 28.1 µs |
| aarch64-unknown-linux-gnu, same settings, run under `qemu-aarch64` | 299 match, 0 mismatch, 11 skipped | not meaningful (emulated) |

`cargo test -p lxr-core --release --target aarch64-unknown-linux-gnu` under qemu: all tests pass
(12 unit, 9 `hat_split`, 3 `vdrum_seq`).

This shows that the engine is portable to aarch64 and stays bit-exact there (it uses `crate::fpu`
for fused operations and its own libm). It does not show the render time on the Move's
Cortex-A72, the worst case (the corpus mean includes quiet passages), or the floating-point
control state of Move's audio thread.

At the measured x86 mean, one Move block costs about 56 µs. If the A72 were 10 times slower, that
would be about 560 µs of the 2370 µs that everything on the callback shares. Phase 0 measures it.

The binary built here needs `GLIBC_2.34` (linked against Ubuntu's glibc 2.39). The Move's glibc
version is unknown (section 7.2).

## 5. Fit between VDrum and the Schwung contract

| Topic | VDrum today | On Schwung | Action |
|---|---|---|---|
| Sample rate | Engine at 44100 Hz (`ENGINE_RATE`); optional 44140.625 Hz with resampling | 44100 Hz fixed | 44100 Hz, no resampler, no `Pipeline` FIFO. |
| Block size | 64-frame ticks | 128-frame blocks | Two `tick()` calls per `render_block`. Events are applied before the first tick, so event timing has a resolution of one block (2.9 ms). |
| Format | `i16` L/R, DAC1 and DAC2 | `i16` stereo | DAC1 + DAC2, saturating sum (decision 5). |
| Flush-to-zero | Must be off; `plugin/src/fp.rs` clears FPCR.FZ on aarch64 | Unknown on Move's audio thread | Wrap every engine call in the same guard (copied into `vdrum-move`, see 6.1). |
| Allocation | `Lxr::new()` allocates; `tick()` does not | Allowed in `create_instance` (worker thread), forbidden afterwards | Allocate everything in `create_instance`: engine, sequencer data, event queue, LED and display buffers. |
| File I/O | Plugin controller thread reads kits and patterns | Forbidden in the DSP's callbacks | File I/O in `ui.js` (`host_read_file`, `host_write_file`) or a demoted Rust worker thread (section 6.5). |
| Sequencer | Firmware sequencer in `lxr-core`, plus `SeqAux` (step probabilities, densities) in `plugin/src/engine.rs`, plus `vdrum-surface::seq` data | VDrum owns it (decision 4) | Internal clock and tempo. Move's transport is not followed. |
| Panics | Plugin marks the engine faulted | A panic inside MoveOriginal crashes Move | `catch_unwind` around every `extern "C"` entry, `panic = "abort"` only as the last resort; after a caught panic the instance renders silence and reports it through `get_error`. A fuzz test drives all entry points. |
| Licence | GPL-3.0-only | Schwung source MIT; the module is a separate `.so` | Own use only (decision 7), so no distribution question for now. |

## 6. Architecture (as implemented)

### 6.1 Crates and files

```
crates/vdrum-engine/        shared by plugin/, vdrum-surface and vdrum-move
  engine.rs                 Engine trait, LxrEngine (lxr-core + SeqAux), SeqEv, trigger notes
  seq.rs                    pattern data and edits (SeqData, SeqEdit, StepData)
  library.rs, lxr_files.rs  kit library, .SND/.PAT/project files
  defaults.rs, fp.rs        VDrum's defaults, IEEE guard
crates/vdrum-move/          cdylib (dsp.so) + rlib (tests)
  src/abi.rs                host_api_v1_t, plugin_api_v2_t (offset of reserved checked: 120)
  src/lib.rs                Instance and the C entry points (catch_unwind on each)
  src/audio.rs              audio thread: commands, voice pads, 2 ticks per block, DAC sum
  src/shared.rs             queues and atomics between the audio thread and the worker
  src/worker.rs             worker thread: surface, files, frame for ui.js, autosave
  src/adapter.rs            Move's controls on vdrum_surface::Surface; LED frame, display lines
  src/model.rs              vdrum_surface::Model over the shared state; actions
  src/files.rs              kits/, patterns/, state/
  src/hw.rs                 Move's notes, CCs, LED addresses, palette
  module/                   module.json, ui.js, help.json
  tests/harness.rs          C entry points, output vs engine, allocation, random input
tools/package-move.sh       aarch64 build, dist/vdrum-module.tar.gz
```

### 6.2 Threads and data flow

```
 Move SPI callback (core 3, SCHED_FIFO)          worker thread (cores 0-2, SCHED_OTHER)        ui.js (shadow UI)
 on_midi: pads/steps ──► voice row plays at once   Adapter → Surface → actions                  CCs (+ notes) ──► set_param("in")
          └─────────────► input queue ───────────► (params, seq edits, kits, files)            get_param("ui") ◄── frame
 set_param("in") ───────► input queue              ui frame (rev, 68 LEDs, 5 lines) ──────────► LED diff, ≤ 8 per tick; display
 render_block: commands ◄── command queue ◄────────
              2 × tick, DAC1 + DAC2
```

- The audio thread never allocates, locks (only `try_lock`) or does file I/O; the harness checks
  allocation with a counting allocator.
- `create_instance` (on the shim's loader thread) allocates the engine and starts the worker. The
  worker sets itself to `SCHED_OTHER` on cores 0 to 2.
- Pads reach the DSP's `on_midi` directly (cable-0 notes ≥ 10). `ui.js` forwards notes too; the
  DSP ignores them once a direct note has arrived, so the worker plays the voice row from them only
  if the direct path does not work.

### 6.3 Mapping: the surface on Move

The adapter turns Move's controls into the Launch Control XL 3 surface's inputs, so every mode
(voices, sequencer, Mix/FX, Perform, LFO/Mod, Morph/Macro, Global) and feature (held-step locks,
pattern layer, momentary effects, macros, snapshots, undo, copy/paste, kit browse) is the same
code as on the desktop. The table is in [vdrum-move.md](vdrum-move.md) and in
`crates/vdrum-move/src/adapter.rs`. Choices made there:

- Track buttons select which knob row (or the faders) the 8 knobs drive. Faders are set from the
  knobs relative to their current value; fader pickup is off on Move.
- The sequencer runs on its own clock (host clock off); Move's transport is not followed.
- Back is Schwung's (background / quit); the surface's Logo (previous mode) is Menu pressed again.
- The pattern layer reaches patterns 1 to 16 (as on the LCXL3).

### 6.4 Output

- The worker publishes `rev\n<136 hex digits>\n<5 lines>` every 20 ms when it changed.
  `get_param("ui")` copies it with `try_lock` (answers nothing that block if the worker is
  writing).
- `ui.js` reads the frame every 6 ticks, sends at most 8 changed LEDs per tick through
  `setLED`/`setButtonLED`, re-checks every LED about once a second (a send dropped by a full MIDI
  buffer is retried), and repaints everything on `onResume`.

### 6.5 Files

In the module folder (`/data/UserData/schwung/modules/overtake/vdrum/`): `kits/` (VDrum `.snd`,
browsed with the wheel; Capture saves to the kit's file, or a new `NN-NAME.SND`), `patterns/`
(`NN-NAME.PAT`, written for every non-empty pattern on save) and `state/` (edited kit, lossless
sequencer data `seq.txt`, settings, snapshots/macros, faders, mutes). A `.PAT` newer than
`state/seq.txt` replaces its pattern on start. The session is saved 10 s after the last change, on
unload and on Capture.

### 6.6 Tests off the device

- `cargo test -p vdrum-move`: adapter (banks, faders in every mode, modes and steps, patterns,
  mute/solo, accent, wheel, frame), files (round trip, `.PAT` import, new kit names), worker
  (browse/load/save, frame, thread start and stop), C entry points (hex input, `perf`).
- `tests/harness.rs`: output equals `LxrEngine` run directly (400 blocks with pads and a parameter
  change); no allocation in `on_midi`, `set_param("in")`, `get_param`, `render_block` and the
  commands the worker sends; random input on every entry point.
- The same tests pass on aarch64 under qemu (CI job `move`). The release `dsp.so` was also loaded
  with `dlopen` from a C program compiled against Schwung's `plugin_api_v1.h` under qemu: create,
  pad, render, `get_param("ui")`, destroy (session saved).

## 7. Build, packaging and install

- `tools/package-move.sh` builds `aarch64-unknown-linux-gnu` with `aarch64-linux-gnu-gcc` and writes
  `dist/vdrum-module.tar.gz` (`vdrum/`: `module.json`, `ui.js`, `help.json`, `dsp.so`, empty `kits/`
  and `patterns/`). CI uploads it as the artifact `vdrum-move-module`.
- The library needs `GLIBC_2.34` at most, plus `libgcc_s`, `libm`, `libc`. Schwung's own build
  image is Debian bookworm (glibc 2.36) and its shim uses `pthread_*`, which links against
  `GLIBC_2.34` there, so Move very likely provides 2.34; not checked on a device.
- Install steps are in [vdrum-move.md](vdrum-move.md).

## 8. Phases

| Phase | State |
|---|---|
| 0. Probe | Skipped (decision 10). The `perf` key reports render time on the device. |
| 1. Engine and sequencer | Done, tested off the device. |
| 2. Kits and patterns | Done (decision 9), tested off the device. |
| 3. Full surface | Done through the shared surface, tested off the device. |
| 4. Polish | Open: pad pressure, external MIDI out, a Move-specific display, patterns 17 to 64 on the pads. |

## 9. Risks

| Risk | Effect | Mitigation |
|---|---|---|
| A72 too slow for the engine plus everything else on the callback | Device-wide dropouts | Phase 0 measures first; the FX bus is the likely heaviest part. |
| Panic or fault in the module | Crashes Move | `catch_unwind` on every entry, fuzzing, silent fault state. |
| Hidden allocation or blocking on the callback (`Vec` growth, `String` formatting, `Mutex`) | Dropouts | Counting allocator test; preallocated buffers; formatting into the host's buffer. |
| Lost or late control input through the JS bridge | Missed button presses, jumpy encoders | One batch per tick; encoder deltas summed; measure in phase 0. |
| LED bandwidth (about 8 per tick) | Sluggish pad flashes | Diff-based frames with priorities. |
| Schwung changes (unofficial, "not stable") or Ableton firmware updates | Module or Schwung stops working | Pin Schwung 1.9.2 in the docs; keep the ABI layer in one file. |
| Untested on hardware | Unknown render time, control delivery and LED rate on Move | `perf` key; first device run reports them. |
| Reinstalling the module replaces its folder | Kits and patterns in `kits/` and `patterns/` lost | Copy them off the Move before reinstalling (vdrum-move.md). |

## 10. Open questions

1. Render time, glibc version and control delivery on the device: to be read from the first run
   (`get_param("perf")`, Schwung's debug log).
2. Whether `suspend_keeps_js` is honoured for `component_type: "overtake"` (it is read in
   `loadOvertakeModule`; the modules that use it in 1.9.2 are tools).
3. Whether Schwung Manager replaces the module folder on reinstall (would delete `kits/` and
   `patterns/`); if so, the folders could move next to the module.
