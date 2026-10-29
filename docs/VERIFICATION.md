# Verification report

State of the `main` branch at the time of the first merge (2026-10-29). All results below were produced on
Linux x86_64 in the build container; nothing ran on macOS or Windows hardware.

## What "identical" means here

* **Reference:** the shipped firmware image `LXRV2_update_v1.71.img` (not in this repository), executed in a
  Cortex-M7 emulator (`crates/lxr-emu`, Unicorn/QEMU). The emulator runs the firmware's own startup code and
  the same functions the main loop and the audio path call; hardware access (codec, LCD, SD, USB, GPIO) is
  stubbed. The exact sequence is in `docs/ENGINE_CONTRACT.md`.
* **Compared quantity:** the 16-bit DAC samples (both stereo DACs) of every 64-frame block, byte for byte, plus
  in most tests the complete engine state (all globals) after every message and block.
* **Not verified:** equality with physical hardware. No hardware recordings were compared. Two hardware
  properties are replaced on purpose in both the emulator and the port:
  * the hardware true-random generator (noise oscillator, random LFO, dither) is replaced by a deterministic
    xorshift32 generator;
  * the six volume faders are set to the value the firmware computes for a fader at maximum.
* **Cross-check of the emulator itself:** the firmware built from the public v1.7 source with Arm GCC 11.3
  matches 568 of 777 functions of the shipped v1.71 binary byte for byte (relocations masked), including all
  DSP, voice, FX, mixer and MIDI-CC code; on the scenario corpus both builds produce identical audio except the
  documented `global_morph_no_target` scenario.

## Results

| Check | Result |
|---|---|
| Per-function differential tests (`crates/lxr-difftest`, p1/p2/p4: oscillators, filters, envelopes, LFO, modulation, FX, f16 conversion, voices, mixer), random sweeps + long runs vs. the GCC 11.3 reference build | 58 tests pass, bit-exact (NaN payload sign ignored in state comparisons; NaN never reaches the output) |
| libm (`expf`, `logf`, `exp`, `fmod`) vs. shipped binary | bit-exact; `expf` and `logf` checked on all 2^32 inputs |
| MIDI/control path (`p3_midi`): all CCs on all channels, NRPN, notes, clock, random streams, every parameter over its legal range, compared after every message | 8 tests pass |
| Shipped-only functions (`voiceControl_noteOn`, `seq_processFlam`) called in the shipped image | bit-exact |
| Whole engine, scenario corpus (311 scenarios, 101,414 blocks = 6.5 M frames) vs. the emulated shipped v1.71 image and vs. the GCC 11.3 build | 0 differences in audio or state, except `global_morph_no_target` (see below); 310 of 311 committed v1.71 hashes reproduced |
| Emulator self-check: corpus rendered from the shipped image vs. committed hashes (`.github/ci/check_scenarios.sh`) | 311 / 311 match |
| No-panic fuzzing (`p3_fuzz`): random MIDI, parameter values and kit contents, debug + release | pass (1024 seeds) |
| Plugin audio path at 44.1 kHz with the real engine, 5 host buffer patterns, jittered event positions, FTZ forced on | 299 non-kit scenarios reproduce the v1.71 hashes |
| `cargo test --workspace --release` with firmware paths set | 34 test binaries, 0 failures |
| clap-validator 0.3.2 on `VDrum.clap` | 18 passed, 0 failed, 3 skipped (preset discovery not implemented) |
| pluginval 1.0.4 on `VDrum.vst3`, strictness 10, editor tests (Xvfb) | SUCCESS |
| CPU (release, all voices retriggered, FX at 100 % wet) | mean 23.5 µs per 64-frame block = 1.6 % of one core |

## Known divergences (`docs/ENGINE_CONTRACT.md` §9)

All occur only for inputs where the firmware reads uninitialised or unmodelled memory, so the hardware output
is itself not defined by the firmware:

* oscillator waveform values above 5 (sent by MIDI CC) make the firmware render uninitialised stack memory;
* morph with no morph kit loaded (no SD card) interpolates towards ITCM code bytes (`global_morph_no_target`);
* bank select to kits 64–127 reads memory past the kit table; kit 68 differs in one byte that USB code writes;
* accesses through corrupted parameter pointers to unmodelled memory read 0 and drop writes;
* Euclid lengths above 17 overflow the firmware stack; the port caps the recursion.

Events (MIDI, parameter changes) are applied between 64-frame blocks, as the firmware's main loop does.

## Sample rate

The engine produces 64-frame blocks at a nominal 44.1 kHz (the hardware clock is 44140.625 Hz). At a host rate
of 44.1 kHz the plugin outputs the engine's samples unchanged (`sample / 32768`). At other host rates, or with
the "Hardware clock" option, the output is resampled and therefore no longer byte-identical.

## Platforms

| Platform | Status |
|---|---|
| Linux x86_64 | built and tested: standalone, CLAP, VST3 (tests and validators above) |
| Windows x86_64 | cross-compiled with `x86_64-pc-windows-gnu` (DLL + EXE build); not run |
| macOS (CLAP, VST3, AU via clap-wrapper, standalone app) | configured (`au/CMakeLists.txt`, CI workflow); not built |
| GitHub Actions workflow | present; not run (Actions not enabled for the repository) |
