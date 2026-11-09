# VDrum as a Schwung module on Ableton Move: implementation plan

Status: brainstorm, revision 1 (2026-11-09). Nothing here is implemented except the feasibility
check in section 4. This plan is route M3 ("VDrum running on the Move") of
[mapping-brainstorm.md](mapping-brainstorm.md) (in PR #27). The control mapping itself is in that
document and is not repeated here.

## 1. Summary

- VDrum's engine fits Schwung's audio contract without resampling: both run at 44100 Hz with int16
  stereo output. One Move block (128 frames) is exactly two engine ticks (64 frames each).
- `lxr-core` builds for `aarch64-unknown-linux-gnu` unchanged. Under qemu it reproduces all 299
  kit-free reference scenarios of firmware v1.71 bit for bit (section 4). Speed on the Move's CPU
  is not measured yet.
- Schwung offers two places for a module with native DSP. Both are useful for VDrum:
  - **Chain sound generator** (`component_type: "sound_generator"`): VDrum is an instrument in one
    of Schwung's shadow slots, next to stock Move. Move's own drum track and sequencer play it;
    parameters appear in the Shadow UI.
  - **Overtake tool with DSP** (`component_type: "tool"`, `tool_config.overtake: true`, `dsp.so`):
    VDrum takes the whole surface (pads, steps, encoders, LEDs, display) and runs its own
    sequencer and modes, as with the LCXL3.
- Recommendation: one Rust `cdylib` that implements the Schwung C ABI, shipped first as a chain
  sound generator (smaller, tests the engine on the device), then as an overtake tool with the
  full VDrum surface. Phases and exit criteria are in section 8.

## 2. Sources

| Source | What was read |
|---|---|
| Schwung repository, commit `673e67a` (2026-11-05), release 1.9.2 in `module-catalog.json` | `README.md`, `docs/ARCHITECTURE.md` (first 200 lines), `docs/API.md` (display, host functions, overtake, audio), `docs/MODULES.md` (module structure, `module.json`, capabilities, tool config, Remote UI for overtake tools, threading, `split_voices`, building, parameter hierarchy, voices, overtake modules), `src/host/plugin_api_v1.h` (whole file), `src/schwung_shim.c` (overtake DSP render and MIDI delivery), `Dockerfile`, `BUILDING.md` (grep only), `src/modules/tools/seq-test/module.json`, `src/modules/overtake/rnbo-runner/module.json`. |
| Ableton Move manual | Not re-read for this plan; hardware facts come from mapping-brainstorm.md section 1. |
| VDrum `main` @ `b170272` | `docs/ARCHITECTURE.md`, `plugin/src/engine.rs`, `plugin/src/pipeline.rs`, `plugin/src/fp.rs`, `plugin/src/real_engine_tests.rs`, `crates/lxr-core` tests. |

Claims marked "unverified" were not checked against source or hardware.

## 3. Schwung facts that decide the design

### 3.1 Process and thread model

- Schwung is an `LD_PRELOAD` shim (`schwung-shim.so`) inside Ableton's `MoveOriginal` process. It
  runs slot DSP and overtake DSP **in that process**, on the SPI audio callback.
- UI code (`ui.js`) runs in a separate QuickJS process (`shadow_ui`). It talks to the DSP through
  `host_module_set_param` / `host_module_get_param` (string key, string value).
- `plugin_api_v1.h`: "THERE IS NO CONTROL THREAD." `create_instance`, `destroy_instance`,
  `set_param`, `get_param`, `on_midi` and `render_block` all run on the SPI callback (SCHED_FIFO 70,
  core 3, about 2370 µs of slack per 128-frame block, shared with Move and all other modules).
  Exception: when a module is loaded into a chain slot, `dlopen`, `create_instance` and the
  state-restoring `set_param` run on a SCHED_OTHER loader thread; the header still asks that
  create time stay free of the forbidden operations.
- Forbidden on the callback: file I/O, allocation and free, locks shared with non-RT threads,
  logging (including `fprintf(stderr)`), unbounded work. A violation causes a device-wide dropout.
- Worker threads inherit SCHED_FIFO 70 and must demote themselves to SCHED_OTHER and pin to cores
  0 to 2 as their first action.

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
/* Optional, dlsym'd: per-voice output for chain buses. Accumulates, buffers alias. */
void move_plugin_render_split(void *inst, int16_t *const *voice_out, int n_voices,
                              int16_t *main_out, int frames);
```

`host_api_v1_t` gives, among others: `sample_rate`, `frames_per_block`, `midi_send_internal`,
`midi_send_external`, `get_clock_status`, `get_bpm`, `get_beat_position` (may be NULL on older
hosts). The header warns that modules must not extend either struct; new capabilities are
`dlsym`'d exports.

### 3.3 The two module kinds that can carry VDrum

| | Chain sound generator | Overtake tool with DSP |
|---|---|---|
| `module.json` | `component_type: "sound_generator"`, `api_version: 2` | `component_type: "tool"`, `tool_config: {interactive, skip_file_browser, overtake: true}`, `dsp: "dsp.so"` (pattern of `seq-test`) |
| Install path | `modules/sound_generators/<id>/dsp.so` (the chain host ignores `module.json`'s `dsp`) | `modules/<tool path>/<id>/` (exact path for tools unverified) |
| MIDI input | The Move **track's** output: notes as the track plays them (a drum track starts at note 36), on the slot's channel. Not pad ids. | Cable-0 note on/off with note ≥ 10 (pads 68 to 99, steps 16 to 31) reach `on_midi` on the audio thread (`schwung_shim.c`). CCs (encoders, buttons, wheel) go to `ui.js` only. |
| Surface | Shadow UI draws pages from `get_param("ui_hierarchy")`; knobs 1 to 8 map to parameters. | The module owns display and all LEDs. Shift + Volume touch + wheel click always exits. |
| Audio | Through the slot's chain (MIDI FX, audio FX, buses, Master FX). | Mixed into Schwung's output after the slots (`shadow_deferred_dsp_buffer`), saturating int16 add. |
| Keeps running in background | Yes (it is a slot). | Only with `capabilities.suspend_keeps_js` (Back suspends, Shift + Back exits). |
| Coexists with Move | Fully. | Move keeps running underneath; `button_passthrough` can hand buttons such as Play back to Move. |

### 3.4 Surface limits in overtake mode

- The MIDI output buffer holds about 64 USB-MIDI packets per frame, shared with Move. The docs
  recommend queueing LED changes and sending about 8 per `tick()` (about 60 ticks per second).
- The display is 128 × 64, 1 bit. `ui.js` draws with `print`, `fill_rect`, `draw_line`,
  `draw_arc`, `draw_image` and so on; each JS→C binding call costs about 490 ns (API.md).

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

What this shows: the engine is portable to aarch64 and stays bit-exact there, because it uses
`crate::fpu` for fused operations and its own libm. What it does not show: the render time on the
Move's Cortex-A72, the worst case (the corpus mean includes quiet passages), and the Move process's
floating-point control state.

At the measured x86 mean, one Move block costs about 56 µs. The A72 figure has to be measured on
the device (phase 0). If it were 10 times slower, it would be about 560 µs of the 2370 µs that
everything on the callback shares.

The binary built here needs `GLIBC_2.34` (linked against Ubuntu's glibc 2.39). The Move's glibc
version is unknown (section 7.2).

## 5. Fit between VDrum and the Schwung contract

| Topic | VDrum today | On Schwung | Action |
|---|---|---|---|
| Sample rate | Engine runs at 44100 Hz (`ENGINE_RATE`); optional 44140.625 Hz with resampling | 44100 Hz fixed | Run at 44100 Hz, no resampler, no `Pipeline` FIFO latency. Drop the "real clock" option on the Move. |
| Block size | 64-frame ticks | 128-frame blocks | Two `tick()` calls per `render_block`. Events that arrive before a block are applied before its first tick, so event timing has a resolution of one block (128 frames, 2.9 ms), coarser than the hardware's 64-frame ticks. |
| Format | `i16`, L/R interleaved, DAC1 and DAC2 | `i16` stereo interleaved | Copy DAC1 directly. DAC2 (aux): sum into the stereo output (saturating) or publish it as a second `split_voices` entry (section 6.3). |
| Flush-to-zero | Must be off (`docs/ARCHITECTURE.md` porting rule 3); `plugin/src/fp.rs` already clears FPCR.FZ on aarch64 | Unknown state on MoveOriginal's audio thread | Wrap every engine call in `IeeeGuard` (move `fp.rs` to a shared crate). |
| Allocation | `Lxr::new()` allocates (`FX_BUFFER`, pattern set, song data, kit memory); `tick()` does not | Allocation only at create time (on the loader thread for chain slots) | Allocate once in `create_instance`. Measure `Lxr::new()` time and memory on the device. |
| File I/O | Kit library, pattern and kit files read in the plugin's controller thread | Forbidden on the callback | A demoted worker thread (SCHED_OTHER, cores 0 to 2) loads files and hands the result over through a lock-free single-producer queue; the audio side swaps a pointer. |
| Threads | nih-plug audio thread plus controller thread | No control thread | The surface state machine (`vdrum-surface`) must run either on the callback (if it is allocation-free) or on the worker. Today it uses `Vec` and `serde_json`, so it runs on the worker in phase 2 unless made allocation-free. |
| Panics | nih-plug process; a panic is contained by the host only partly | A panic inside MoveOriginal crashes Move, and a crash at slot restore can boot-loop the device (`plugin_api_v1.h`) | Build with `panic = "abort"` is not enough: wrap every `extern "C"` entry in `catch_unwind`, then put the instance into a silent error state reported by `get_error`. Add a test that no entry point can panic on any input. |
| Transport | Host transport via `SeqEv::HostStart/HostAlign/HostPulse/HostStop` | `get_clock_status`, `get_beat_position`, `get_bpm` | Derive 16th-note pulses from `get_beat_position()` per block, feed the existing `SeqEv::Host*` path. Fallback when `get_beat_position` is NULL: internal clock at `get_bpm()`. |
| Licence | GPL-3.0-only | Schwung source MIT; the module is a separate `.so` loaded into Ableton's proprietary process | Ship the module's source with each release, as for the plugin builds. Not legal advice. |

## 6. Architecture

### 6.1 Crates and files

```
crates/
  vdrum-engine/        NEW: Engine trait, LxrEngine, SeqEv, SeqAux, kit/pattern I/O, IeeeGuard
                       (moved out of plugin/src/{engine,fp,library,lxr_files}.rs; no nih-plug).
  vdrum-surface/       existing; gains a Move layout (or uses the surface library, if PR #27's plan
                       is built first).
  vdrum-schwung/       NEW: cdylib "dsp.so". The Schwung C ABI, the worker thread, the bridge
                       protocol to ui.js. Depends on vdrum-engine (+ vdrum-surface in phase 2).
schwung/
  vdrum/               module folder as installed on the Move
    module.json        (sound generator) — or a second folder for the overtake tool (6.4)
    ui.js              thin bridge (overtake tool only)
    ui_chain.js        optional
    help.json          optional
    dsp.so             built artefact, not committed
scripts/ or xtask      `cargo xtask schwung` → dist/vdrum-module.tar.gz
```

`plugin/` keeps using `vdrum-engine`, so the desktop plugin and the Move module share one engine
wrapper. The move of `engine.rs` out of `plugin/` is a refactor with no behaviour change; the
existing `real_engine_tests.rs` must still pass.

### 6.2 Data flow

```
             MoveOriginal process (shim, SPI callback, core 3)          shadow_ui process (QuickJS)
 pads/steps ─────────────► on_midi ─┐                                   ┌─ onMidiMessageInternal ◄─ encoders,
 (overtake: cable-0 notes ≥ 10;     │                                   │                           buttons, wheel
  chain: the track's notes)         ▼                                   │
                            ┌──────────────────┐   set_param("in",…)    │
                            │ vdrum-schwung    │ ◄──────────────────────┘
                            │  event ring      │
                            │  Lxr ×1          │   get_param("leds"/"disp") ──► ui.js: setLED queue,
                            │  render_block:   │ ─────────────────────────────►        display drawing
                            │   2 × tick()     │
                            └───────┬──────────┘
                                    │ SPSC queues (lock-free, preallocated)
                            ┌───────▼──────────┐
                            │ worker thread    │  file I/O (kits, patterns, state), surface model
                            │ SCHED_OTHER, 0-2 │  in phase 2 if it allocates
                            └──────────────────┘
```

### 6.3 Mode A: chain sound generator (phase 1)

- **Notes.** The Move drum track sends notes from 36 upwards on the slot's channel. VDrum's
  per-track MIDI note and channel settings (`tracks_for_note`) already map notes to voices; the
  module's defaults set voices 1 to 7 to notes 36 to 42 (open question 2) and the channel to the
  slot's receive channel.
- **Voices for the Shadow UI.** `get_param("ui_hierarchy")` publishes one level per voice with
  `note` and `role` (`kick`, `snare`, `hat`, …), so that the Shadow UI follows the pad that is
  played. Knobs 1 to 8 per page follow bank 1 to 4 of each voice in mapping-brainstorm.md section
  3.3 (one page per bank). Labels are about 5 characters (MODULES.md "the knob grid has ~5
  characters"); `lxr_params` names need short forms.
- **Parameter keys.** One key per LXR parameter, for example `p_<index>`, values as integers in the
  firmware range. `set_param` pushes the change into the event ring; `get_param` reads a cached
  `[u8; N]` copy (no formatting work on the callback beyond `itoa`).
- **State.** `get_param("state")` returns the kit (297-byte VDrum kit file, `lxr_params::kit`) and
  the pattern set, base64-encoded, under 64 KB. Whether the chain saves it with the Move set, and
  which key it uses, is to be checked in `docs/CHAIN.md` (not read).
- **Split outputs.** `split_voices` with two entries, `main` (DAC1) and `aux` (DAC2), lets the
  user put VDrum's aux output on its own chain bus. True per-voice outputs (7 entries) would need
  the engine's mixer to render each voice into its own buffer. That is a change to the firmware
  port's mixer path and is out of scope until asked for.
- **Internal sequencer.** In this mode Move's sequencer plays VDrum. VDrum's own sequencer stays
  off by default; it can be enabled and follow Move's transport (section 5, Transport).

### 6.4 Mode B: overtake tool (phase 2)

- **Input.** Pads and step buttons arrive in the DSP's `on_midi` on the audio thread, so pad-to-
  sound latency is one block at most. Encoders, buttons and the wheel arrive only in `ui.js`;
  `ui.js` forwards each raw message unchanged with `host_module_set_param("in", "<b0> <b1> <b2>")`.
  Pad pressure (polyphonic aftertouch, 0xA0) is not in the shim's note-only filter, so it also
  arrives through `ui.js` (to be verified on hardware).
- **Surface logic in Rust.** The Move mapping (mapping-brainstorm.md section 3) is implemented in
  Rust, either as a Move layout in `vdrum-surface` or, if the generic library from PR #27 is built
  first, as `devices/ableton-move` plus the VDrum profile. `ui.js` contains no mapping logic.
- **LEDs.** The surface produces an LED frame (32 pads, 16 steps, 4 track buttons, function
  buttons, 8 encoder LEDs). The DSP keeps the last sent frame and serves the difference through
  `get_param("leds")` as a compact string; `ui.js` queues it and sends at most about 8 per tick.
  Hit flashes therefore reach the pads with up to a few frames of delay; acceptable, but to be
  checked.
- **Display.** The surface produces a display list (text lines and value bars), not a bitmap,
  served by `get_param("disp")` only when it changed. `ui.js` draws it with `print`/`fill_rect`.
  A 1024-byte bitmap through `draw_image` is the alternative if the display list is too limiting.
- **Background.** `suspend_keeps_js: true` so that VDrum keeps playing when the user goes back to
  Move. `button_passthrough` is empty by default (VDrum owns Play).
- **Two modules or one.** Schwung's `component_type` is one value per `module.json`. Two module
  folders (`vdrum` as sound generator, `vdrum-perform` as overtake tool) can share one `dsp.so`
  source. Whether both can run at once with shared state (a tool controlling the instance in a
  slot) is unverified; the simplest first version is two independent instances.

### 6.5 Remote UI (optional, later)

Overtake tools can ship `web_ui.html`, served by Schwung Manager on port 7700, reading
`get_param("state")` and writing `set_param`. VDrum's editor tabs could be reproduced there.
Not planned in the phases below.

## 7. Build, packaging and test

### 7.1 Build

- Target `aarch64-unknown-linux-gnu`, crate type `cdylib`, output renamed to `dsp.so`.
- Exported symbols: `move_plugin_init_v2` and, for split outputs, `move_plugin_render_split`.
  No `move_plugin_init_v1`.
- C header bindings: hand-written `#[repr(C)]` structs for `host_api_v1_t` (including the 8-pointer
  reserved tail, `offset_of!(reserved) == 120` asserted at compile time) and `plugin_api_v2_t`.
- Linker: `aarch64-linux-gnu-gcc` (Debian/Ubuntu package `gcc-aarch64-linux-gnu`), or the Schwung
  `Dockerfile` (Debian bookworm, glibc 2.36), or `cargo-zigbuild` with an explicit glibc baseline
  such as `aarch64-unknown-linux-gnu.2.17`.
- Release profile: `lto`, `codegen-units = 1`, `-C target-cpu=cortex-a72` once the CPU is
  confirmed (it does not change results: `crate::fpu` controls fused operations).

### 7.2 Things to check on the device before relying on them

1. glibc version: `ldd --version` or `/lib/libc.so.6` over SSH. The build baseline must not exceed it.
2. CPU: `/proc/cpuinfo` (the manual says quad-core Cortex-A72; clock not stated).
3. Render time: a debug key `get_param("perf")` returning the mean and max µs per block, measured
   with `CLOCK_MONOTONIC` in `render_block` (Schwung's shim also measures overtake generator time,
   `spi_overtake_gen_max`).
4. FPCR on the audio thread before and after the guard.
5. Free memory for `Lxr` (size to be measured on x86 first with `size_of` plus heap totals).

### 7.3 Packaging and CI

- `cargo xtask schwung` builds `dsp.so` and writes `dist/vdrum-module.tar.gz`, which extracts to a
  folder named after the module id (MODULES.md "Publishing to the Module Catalog").
- GitHub Actions: an aarch64 cross-build job, and a job that runs the scenario corpus check of
  section 4 under qemu, so that aarch64 bit-exactness is tested on every change. The release
  workflow attaches the tarball to the GitHub release.
- Installing: copy the folder to `/data/UserData/schwung/modules/...` over SSH/SFTP and rescan, or
  install through Schwung Manager from a release URL. Catalog listing is a separate, later decision.

### 7.4 Tests off the device

- A host-side harness that loads `dsp.so` (x86 build) with `dlopen`, fakes `host_api_v1_t`, and
  drives `on_midi`/`set_param`/`render_block`. It checks: output equals `Lxr` direct output for the
  scenario corpus at 128-frame blocks; no allocation inside entry points (a counting global
  allocator that aborts when armed); no panic for random input (fuzz).
- The same harness built for aarch64 runs under qemu in CI.

## 8. Phases

| Phase | Work | Exit criterion |
|---|---|---|
| 0. Device facts | SSH to a Move with Schwung: glibc, CPU, memory. Minimal Rust `dsp.so` that renders a sine and reports `perf`. Then the same with `Lxr` rendering a busy pattern (all 7 voices, FX on). | Rust `cdylib` loads in a slot; max render time per block known. |
| 1. Engine extraction | Move `engine.rs`, `fp.rs`, kit/pattern file code out of `plugin/` into `crates/vdrum-engine`. | `plugin` tests unchanged and green. |
| 2. Chain sound generator | `crates/vdrum-schwung` with the v2 ABI, event ring, worker thread, `ui_hierarchy`, `state`, `split_voices` main/aux, transport following. Host harness and qemu CI job. | Move drum track plays VDrum; parameters editable in the Shadow UI; state survives reload; no dropouts at the measured load. |
| 3. Kit library on the device | Kit folder under `/data/UserData/...` (path open), browse and load through the worker; import of LXR `.kit`/`.lxr` files as the plugin does. | Kits load without audio dropouts. |
| 4. Overtake tool | Move surface layout in Rust (mapping-brainstorm.md section 3), `ui.js` bridge for CCs, LEDs and display; `suspend_keeps_js`. | All modes of mapping-brainstorm.md usable on the Move; LED and display update rates acceptable on hardware. |
| 5. Release | Release workflow, README section, optional catalog entry. | Tarball installs on a clean Schwung 1.9.x. |

Phases 0 and 1 are independent and can run in parallel. Phase 4 depends on the decisions in
mapping-brainstorm.md section 5 and on whether the surface library (PR #27) is built first.

## 9. Risks

| Risk | Effect | Mitigation |
|---|---|---|
| A72 too slow for the engine plus everything else on the callback | Device-wide dropouts | Phase 0 measures before anything else; the FX bus is the likely heaviest part and can be made optional. |
| Panic or fault in the module | Crashes Move; possible boot loop if restored into a slot | `catch_unwind` on every entry, fuzzing in CI, fault state that renders silence. |
| Hidden allocation or blocking (Rust `Vec` growth, `String` formatting, `std::sync::Mutex`) on the callback | Dropouts | Counting allocator test; preallocated buffers; `get_param` formats into the caller's buffer. |
| Schwung API changes (it is unofficial and says it is "not stable") | Module breaks after a Schwung update | Pin the tested Schwung version in docs; keep the ABI layer in one file. |
| Ableton firmware updates | Schwung may stop working until updated | Out of VDrum's control; documented as a limitation. |
| LED bandwidth (about 8 per tick) | Sluggish pad flashes in overtake mode | Diff-based frames, priority to playhead and hit flashes. |

## 10. Open questions

1. **Do you have a Move with Schwung installed, and which Schwung version?** Phase 0 needs SSH
   access to one, which I do not have from here.
2. **Which mode first:** chain sound generator (recommended) or straight to the overtake tool?
3. **Note mapping in chain mode:** voices 1 to 7 on notes 36 to 42 (contiguous, first two pad
   rows of a Move drum track), or GM-like notes (36 kick, 38 snare, 42 closed hat, 46 open hat …)?
4. **VDrum's own sequencer in chain mode:** off (Move sequences) or available and synced?
5. **Aux output (DAC2):** summed into the stereo output, or offered as a separate bus?
6. **Surface library order:** build the Move layout inside `vdrum-surface` now, or wait for the
   generic library of PR #27?
7. **Distribution:** private install only, or a later entry in Schwung's module catalog?
