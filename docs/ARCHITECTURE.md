# VDrum architecture and porting rules

VDrum is a Rust port of the sound engine of the Erica Synths LXR-02 (LXR V2 desktop) firmware,
packaged as a standalone application and as CLAP, VST3 and AU plugins. It is not affiliated with
Erica Synths. The firmware source is GPL-3.0-only, so this project is GPL-3.0-only as well.

## Reference firmware

| Item | Value |
|---|---|
| Firmware source | https://github.com/erica-synths/lxr-desktop-app @ `a43cd4f` (version string 1.7), submodule `third_party/lxr-firmware` |
| Shipped firmware | `LXRV2_update_v1.71.img` (16 byte header `LXRV2IMG`, 273528 byte payload loaded at 0x08008000). Not committed; pass its path via `LXR_FIRMWARE_IMG`. |
| Reference build | Arm GNU Toolchain **11.3.rel1**, `tools/build_fw.sh <src> <out> <prefix>` (all files `-O3 -ffast-math -freciprocal-math`, no ITCM `-O2` override, no `-fno-thread-jumps`). |
| Equivalence of the two | 568 of 777 functions of the GCC 11.3 build are byte-identical to the shipped v1.71 binary after masking relocations (`tools/function_match.py`), including every DSP, voice, FX, mixer and MIDI-CC function. Differences: libm `expf`, `logf`, `exp`, `fmod` (different newlib), `voiceControl_noteOn`, `seq_processFlam`, `main`, parts of the HAL. |

The reference for "identical" is the shipped v1.71 binary executed in the emulator (`crates/lxr-emu`).
The GCC 11.3 source build (with symbols and DWARF) is used for function-level differential tests and in
CI, where the shipped image is not available.

## Crates

| Path | Purpose |
|---|---|
| `crates/lxr-core` | The engine: a faithful port of `Core/Src/Soundengine/**` plus the parts of the MIDI, preset and sequencer code that affect sound. `no_std`-friendly, no allocation in `tick`. |
| `crates/lxr-emu` | Unicorn (QEMU) based Cortex-M7 emulator that runs firmware code: function calls (`Firmware::call`) and the whole engine (`EmuEngine`) with the same interface as `lxr-core`. |
| `crates/lxr-difftest` | Differential tests: lxr-core vs lxr-emu, per function and per scenario, bitwise. |
| `plugin/` | nih-plug plugin (CLAP, VST3, standalone) and GUI; AU via clap-wrapper on macOS. |
| `xtask/` | `cargo xtask bundle vdrum --release` (nih-plug bundler). |

## Engine contract (both `lxr-core::Lxr` and `lxr-emu::EmuEngine` implement it)

The firmware renders audio in the TIM2 interrupt: `seq_tick(); soundengine_calcNextSampleBlock();`.
One call produces `OUTPUT_DMA_SIZE` = 64 stereo frames for DAC1 and 64 for DAC2 (int16, L/R interleaved).
MIDI bytes are parsed in the main loop between interrupts (`uart_processMidi()`).

* `new()` reproduces the power-on state without SD card (exact sequence documented in `docs/ENGINE_CONTRACT.md`,
  written together with the emulator), then loads the init kit.
* `midi_byte(b)` feeds one byte of DIN MIDI through the firmware parser.
* `tick() -> Block { dac1: [i16; 128], dac2: [i16; 128] }` renders one block.
* FPSCR: everything, including audio rendering, runs with **FZ = 0**. `USE_INTERRUPT_ONLY_DURING_SAVE_PROJ` = 1
  makes the firmware render in the main loop; `soundengine_init` only sets FZ in FPDSCR, which applies to
  interrupt handlers. Details and the exact call sequence: `docs/ENGINE_CONTRACT.md` (authoritative).
* The hardware RNG (`GetRngValue`) is replaced by a deterministic generator in both implementations:
  xorshift32 (`x ^= x << 13; x ^= x >> 17; x ^= x << 5;`), state seeded with `0x2545F491` by `new()`,
  returning the new state. Every call to `GetRngValue` in the firmware consumes exactly one value.
* Jack detection inputs (`l1_Available`, `r1_Available`, `l2_Available`, `r2_Available`) are 1.

## Porting rules for `lxr-core`

1. **Structure mirrors the C code.** One Rust module per C file (`DSPAudio/ResonantFilter.c` →
   `dsp/resonant_filter.rs`). C struct typedefs keep their names (`ResonantFilter`, `OscInfo`, `DrumVoice`),
   field names stay as in C, functions keep their C names (`SVF_calcBlockZDF`, `osc_setFreq`), with
   `#![allow(non_snake_case)]`. C globals become fields of `Lxr` with the C name. This keeps every Rust item
   greppable against the C source and the disassembly.
2. **Port the machine code, not only the C.** The firmware is compiled with `-O3 -ffast-math
   -freciprocal-math -mfpu=fpv5-d16`. GCC therefore fuses multiply-adds (`vfma/vfms/vfnma/vfnms`),
   replaces division by constants with multiplication by the reciprocal, reassociates sums and keeps
   values in single precision where the C promotes. For every function, read the GCC 11.3 disassembly
   (`arm-none-eabi-objdump -d --disassemble=<fn> lxr_v1.7_gcc11.3.elf`) and reproduce the exact sequence
   of floating-point operations:
   * fused ops → `crate::fpu::fma(a, b, c)` (= a*b+c, one rounding), `fms`, `fnma`, `fnms` helpers;
   * `vdiv` → `/`, multiplication by a reciprocal constant → `*` with that exact constant (copy the bits
     from the literal pool: `f32::from_bits(0x...)`);
   * `vsqrt` → `f32::sqrt`; `vcvt.s32.f32` → `as i32` (truncating, saturating); `vcvt.u32.f32` → `as u32`;
     `vcvt.f32.s32` → `as f32`; `vcvtr`/`vrint*` with the rounding mode used; `vmaxnm/vminnm` → `f32::max/min`
     (check NaN/zero-sign cases if they can occur);
   * double precision where the machine code uses `.f64` operations;
   * calls to libm (`expf`, `logf`, `exp`, `fmod`, ...) → `crate::fpu::libm` functions that are bit-exact
     ports of the implementation **in the shipped v1.71 binary** (its newlib differs from GCC 11.3's).
   * integer conversions exactly as the machine code (`ssat`, `usat`, `uxtb`, `sxth`, wrap-around).
3. **No flush-to-zero.** All engine code runs with FZ = 0 (IEEE subnormals), see the engine contract.
   Rust code must therefore NOT run under `FtzGuard`, and the plugin must make sure the host's FTZ/DAZ
   setting is cleared while calling the engine (hosts and wrappers often enable FTZ).
4. **Determinism across platforms.** No platform libm, no `f32::powf/exp/ln/sin` etc. from std (they call
   the system libm); use `crate::fpu::libm`. `f32::mul_add` is allowed only through `crate::fpu`.
5. **No allocation or locking in `tick()`.** Large buffers (delay, reverb, sample RAM) are allocated in `new()`.
6. **Tables** (wavetables, samples, transient tables, LUTs) are generated from the reference ELF's
   `.rodata` bytes by `tools/gen_tables.py`, never retyped.
7. **Every ported function has a differential test** in `crates/lxr-difftest` that drives the firmware
   function in the emulator and the Rust function with the same inputs (random sweeps plus edge cases)
   and compares the results bit for bit (`to_bits()` for floats, exact for integers and buffers).
   Struct layouts of the firmware are available with
   `gdb-multiarch -batch -ex 'ptype /o <Type>' lxr_v1.7_gcc11.3.elf`.

## Environment variables

| Variable | Meaning |
|---|---|
| `LXR_FW_ELF` | Path to the GCC 11.3 reference ELF (`tools/build_fw.sh` output). Differential tests skip without it. |
| `LXR_FIRMWARE_IMG` | Path to the shipped `LXRV2_update_v1.71.img`. Shipped-binary verification skips without it. |
