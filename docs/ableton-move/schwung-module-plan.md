# VDrum as a Schwung module on Ableton Move: implementation plan

Status: plan, revision 2 (2026-11-09). Nothing is implemented except the feasibility check in
section 4. This is route M3 ("VDrum running on the Move") of
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
| 6 | Code location | A separate crate, `crates/vdrum-move`, independent of `vdrum-surface` and of the surface library of PR #27. |
| 7 | Distribution | Sami's own use only. No catalog entry. |

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

## 6. Architecture

### 6.1 Crate and files

```
crates/vdrum-move/              NEW, independent of vdrum-surface and plugin/
  Cargo.toml                    crate-type = ["cdylib", "rlib"]; deps: lxr-core, lxr-params
  src/lib.rs                    exports move_plugin_init_v2
  src/abi.rs                    #[repr(C)] host_api_v1_t, plugin_api_v2_t; offset assert (120)
  src/instance.rs               create/destroy, catch_unwind wrappers, fault state
  src/engine.rs                 Lxr + IEEE guard + two ticks per block + DAC sum
  src/seq.rs                    sequencer edits and step data the surface needs (see 6.6)
  src/input.rs                  event queue: pads/steps from on_midi, controls from ui.js
  src/surface/                  Move modes and mapping (mapping-brainstorm.md section 3)
    mod.rs  modes.rs  leds.rs  display.rs
  src/bridge.rs                 set_param/get_param keys, encoding of LED and display frames
  tests/                        host harness (6.7)
schwung/vdrum/                  module folder as installed on the Move
  module.json
  ui.js                         thin bridge, no mapping logic
  help.json                     optional
xtask: `cargo xtask move`       builds dsp.so for aarch64, writes dist/vdrum-module.tar.gz
```

`module.json`:

```json
{
  "id": "vdrum",
  "name": "VDrum",
  "abbrev": "VDRUM",
  "version": "0.1.0",
  "component_type": "overtake",
  "ui": "ui.js",
  "dsp": "dsp.so",
  "api_version": 2,
  "capabilities": { "audio_out": true, "midi_in": true, "aftertouch": true, "suspend_keeps_js": true }
}
```

### 6.2 Data flow

```
             MoveOriginal process (shim, SPI callback, core 3)        shadow_ui process (QuickJS, ~60 Hz)
 pads, steps ───────────► on_midi ─┐                               ┌─ onMidiMessageInternal ◄─ encoders, buttons,
 (cable-0 notes ≥ 10)              │                               │                           wheel, pressure
                                   ▼                               │
                       ┌───────────────────────┐  set_params("in") │  one batch per tick
                       │ vdrum-move dsp.so     │ ◄─────────────────┘
                       │  input queue          │
                       │  surface state        │  get_params("leds","disp")
                       │  sequencer + Lxr      │ ─────────────────────────► ui.js: LED queue (≈8/tick),
                       │  render_block:        │                            display drawing
                       │   2 × tick(), sum     │
                       └───────────────────────┘
```

### 6.3 Input

- **Pads and steps** go straight into the DSP's input queue in `on_midi`, on the audio thread, so a
  pad hit sounds in the next block at the latest.
- **Everything else** is collected by `ui.js` during a tick and sent once per tick as one batch
  (`host_module_set_params`, or `host_module_set_param("in", "<hex bytes>")` if the bulk call is not
  available). Encoder deltas of the same encoder are summed in JS before sending. This avoids the
  lossy single-slot behaviour of repeated `host_module_set_param` calls (section 3.3).
- Shift + Volume touch + wheel click is reserved by Schwung and is not mapped.

### 6.4 Output: LEDs and display

- The surface computes a full LED frame each block it changes (32 pads, 16 steps, 4 track
  buttons, function buttons, encoder LEDs). The DSP keeps the frame `ui.js` last confirmed and
  serves only the differences from `get_param("leds")`, in priority order (playhead and hit flashes
  first). `ui.js` sends at most about 8 per tick. On `onResume()` the DSP resets the confirmed frame
  so that everything is repainted.
- The display is served as a display list (text lines, value bars, a small inverted header) from
  `get_param("disp")`, with a revision number so that `ui.js` redraws only on change. A 1-bit
  bitmap through `draw_image` is the fallback if the list proves too limiting.
- Both `get_param` answers are formatted into the host's buffer without allocation.

### 6.5 Files: kits and patterns

Two options; to be decided in phase 2:

- **In `ui.js`**: JS reads a kit file with `host_read_file` and passes it to the DSP with a
  blocking `set_param("kit", <hex or base64>)`; saving goes the other way. Simple, but kit parsing
  (`lxr_params::kit`) then happens on the audio thread inside `set_param` (bounded: 297 bytes).
- **In a Rust worker thread** (SCHED_OTHER, cores 0 to 2) inside `dsp.so`, with lock-free queues.
  Needed if loading becomes heavier (folder scans, LXR `.snd`/`.all` imports).

Storage folder on the Move: open (for example `/data/UserData/UserLibrary/VDrum/`; not checked).

### 6.6 What "independent" means for the sequencer and engine wrapper

The sequencer logic VDrum needs beyond the firmware's (`SeqAux`: per-step probability, densities;
`SeqEv` application) lives in `plugin/src/engine.rs`, and the step and pattern types live in
`vdrum-surface::seq`. To keep `vdrum-move` independent (decision 6), it gets its own copy of the
parts it needs, written against `lxr-core` directly. The cost is duplicated code (roughly the
300 lines of `LxrEngine`/`SeqAux` plus the data types) that can drift from the plugin. The
alternative is to extract a shared `vdrum-engine` crate from `plugin/`, which touches the plugin.
Open question 1.

### 6.7 Tests off the device

- A host-side harness (`crates/vdrum-move/tests/`) calls the ABI functions of the x86 build
  directly (or via `dlopen` of the cdylib) with a fake `host_api_v1_t`, and checks:
  - output equals `Lxr` direct output for the scenario corpus at 128-frame blocks, summed DACs;
  - no allocation inside entry points after `create_instance` (counting global allocator that
    aborts when armed);
  - no panic and no fault for random input on every entry point (fuzz);
  - surface behaviour: input sequences produce the expected LED frames and display lists.
- The same harness for aarch64 runs under qemu in CI.
- A desktop simulator of `ui.js` is not planned; `ui.js` is kept small enough to test on the device.

## 7. Build, packaging and install

### 7.1 Build

- Target `aarch64-unknown-linux-gnu`, `cdylib`, output renamed to `dsp.so`. Only
  `move_plugin_init_v2` is exported.
- Linker: `aarch64-linux-gnu-gcc`, the Schwung `Dockerfile` (Debian bookworm, glibc 2.36), or
  `cargo-zigbuild` with an explicit glibc baseline such as `aarch64-unknown-linux-gnu.2.17`.
- Release profile: `lto`, `codegen-units = 1`.

### 7.2 Checks on the device (phase 0)

1. glibc version (`ldd --version` over SSH). The build baseline must not exceed it.
2. CPU and clock (`/proc/cpuinfo`).
3. Render time: `get_param("perf")` returns mean and max µs per `render_block`
   (`CLOCK_MONOTONIC`); shown on the display.
4. FPCR on the audio thread before and after the guard.
5. Which controls reach `ui.js` and the DSP (encoders, pad pressure, wheel, Loop's CC).
6. LED behaviour: encoder LEDs, track-button colours, how many LED sends per tick survive.

### 7.3 Install

Own use (decision 7): `cargo xtask move` produces `dist/vdrum-module.tar.gz`; it is copied over
SSH/SFTP to the Move's modules folder (`/data/UserData/schwung/modules/`; exact subfolder for
overtake modules unverified) and found with a module rescan or a Schwung restart. A GitHub Actions
job cross-builds `dsp.so` and runs the aarch64 harness under qemu; no release workflow for now.

## 8. Phases

| Phase | Work | Exit criterion |
|---|---|---|
| 0. Probe | `vdrum-move` skeleton: ABI, `catch_unwind`, `perf` key; `ui.js` that shows `perf` and logs which controls arrive. First a sine, then `Lxr` playing a busy pattern (all 7 voices, FX on). Sami installs and reports. | Module loads as an overtake module on Schwung 1.9.2; max render time known; control delivery known. |
| 1. Engine and sequencer | Engine wrapper, sequencer with internal clock, pads play voices, steps edit the selected track, Play starts and stops. Minimal display (mode, tempo, pattern). Host harness and qemu CI job. | VDrum plays patterns on the Move without dropouts; harness green. |
| 2. Kits and patterns | Load and save kits and patterns (section 6.5), kit browsing on the wheel. Init kit and the plugin's default track notes. | Kits and patterns survive a module exit and reload. |
| 3. Full surface | All modes of mapping-brainstorm.md section 3: encoder banks on the track buttons, Shift + step modes, held-step locks, Mute, Copy, Delete, Undo, LED and display rules (its section 3.5). Background play with `suspend_keeps_js`. | Every mode usable on the Move; LED update rate acceptable. |
| 4. Polish | Pad pressure destinations, momentary effects, external MIDI in and out on USB-A, help.json. | As agreed after phase 3. |

Phase 3 depends on the open questions in mapping-brainstorm.md section 5 (Record, Capture, Sample
buttons; merging Mix + FX with Perform; pad row 4 and pressure).

## 9. Risks

| Risk | Effect | Mitigation |
|---|---|---|
| A72 too slow for the engine plus everything else on the callback | Device-wide dropouts | Phase 0 measures first; the FX bus is the likely heaviest part. |
| Panic or fault in the module | Crashes Move | `catch_unwind` on every entry, fuzzing, silent fault state. |
| Hidden allocation or blocking on the callback (`Vec` growth, `String` formatting, `Mutex`) | Dropouts | Counting allocator test; preallocated buffers; formatting into the host's buffer. |
| Lost or late control input through the JS bridge | Missed button presses, jumpy encoders | One batch per tick; encoder deltas summed; measure in phase 0. |
| LED bandwidth (about 8 per tick) | Sluggish pad flashes | Diff-based frames with priorities. |
| Schwung changes (unofficial, "not stable") or Ableton firmware updates | Module or Schwung stops working | Pin Schwung 1.9.2 in the docs; keep the ABI layer in one file. |
| Duplicated engine wrapper drifts from the plugin (6.6) | Different sequencer behaviour on desktop and Move | Port the plugin's sequencer tests into `vdrum-move`, or extract a shared crate (open question 1). |

## 10. Open questions

1. **Engine wrapper:** copy the needed parts of the plugin's engine wrapper and sequencer types
   into `vdrum-move` (fully independent, some duplication), or extract a shared `vdrum-engine`
   crate from `plugin/` (no duplication, touches the plugin)? I recommend the copy for now,
   matching decision 6, and extracting later if the two drift.
2. **Kit and pattern storage:** a folder on the Move (which path), and should it share the
   desktop plugin's kit file formats so that kits can be copied between the two?
3. **Phase 0:** can you install a test module over SSH and report the numbers it shows? I cannot
   reach the Move from here.
