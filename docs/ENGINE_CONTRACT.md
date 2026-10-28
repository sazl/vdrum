# Engine contract (reference emulator `lxr-emu` ↔ Rust port `lxr-core`)

This document defines exactly what the reference emulator (`crates/lxr-emu`,
`EmuEngine`) does on every API call. It is precise enough that the Rust engine can
mirror it call for call and produce bit-identical output for every scenario in
`tests/scenarios/`. The implementation is `crates/lxr-emu/src/engine.rs`; keep the two
in sync.

The reference is the shipped firmware **v1.71** (`LXRV2_update_v1.71.img`), run
unmodified in Unicorn (Cortex-M7 model, Thumb mode). It uses the recovered symbol map
`crates/lxr-emu/data/v1.71_symbols.json`. The GCC 11.3 ELF of the v1.7 sources
(`lxr_v1.7_gcc11.3.elf`) can be run in the same way and gives the same output for the
corpus, apart from one scenario (see the last section). Function and variable names
below are firmware symbols, and source paths are relative to `lxr-desktop-app/Core/Src`.

## 1. Memory, CPU, FPU

- CPU: Unicorn `Arch::ARM`, `Mode::THUMB`, `ArmCpuModel::CORTEX_M7`.
  - `Mode::MCLASS` would force a Cortex-M33 model without double precision.
- The flash payload is mapped at its link address 0x08008000 (APP_BASE). ITCM is at 0, DTCM/SRAM is at 0x20000000.
- Peripheral space is plain MMIO.
  - Reads return 0, except for the values listed in §3.
  - Writes are ignored.
- **FPSCR per context.** The value is written before every call into firmware code:
  - Thread mode (main loop, init, MIDI, parameters, kits, **audio**): `FPSCR_MAIN = 0`. That is FZ=0, DN=0, AHP=0, round to nearest.
    - At reset FPDSCR is 0.
    - The first FP instruction in thread mode creates the FP context with FPSCR[26:22] = FPDSCR[26:22] = 0.
    - Neither binary contains a `vmsr fpscr` instruction.
  - Exception handlers: `FPSCR_IRQ = FZ` (bit 24).
    - `soundengine_init` executes `FPU->FPDSCR |= FZ` as its last statement, so every exception handler entered afterwards starts with FZ=1.
  - The audio path runs in thread mode.
    - `config.h` sets `USE_INTERRUPT_ONLY_DURING_SAVE_PROJ` = 1. `main()` only starts `HAL_TIM_Base_Start_IT(&htim14)`.
    - TIM2 (the interrupt that would call `soundengine_process`) only runs while a project is saved.
    - So `seq_tick` and `soundengine_calcNextSampleBlock` run with **FZ = 0**.
    - This contradicts the FZ=1 assumption in `docs/ARCHITECTURE.md`.
    - `AudioContext::Tim2Interrupt` / CLI `--fz` render audio with FZ=1 instead.
    - In the current corpus all 311 hashes are identical with FZ=0 and FZ=1.
  - Unknown: the FPSCR/FPDSCR state the bootloader leaves behind. The emulator assumes reset values.
- **VFMA/VFMS/VFNMA/VFNMS are fused** (single rounding).
  - VMLA/VMLS are not fused (two roundings).
  - The port must use `f32::mul_add` exactly where the firmware binary uses `vfma`. Use the disassembly, not the C source, because GCC contracts `a*b+c` into `vfma`.
  - Tests: `crates/lxr-emu/tests/fpu.rs`.

## 2. Startup (`EmuEngine::new_shipped` / `new_elf`)

1. **Intercepts** (host code replaces the function; the caller sees a normal return):
   - `GetRngValue` → xorshift32: `x ^= x<<13; x ^= x>>17; x ^= x<<5; return x`.
     - The state starts at `RNG_SEED = 0x2545F491`.
     - There is one generator per engine, and it advances once per call.
     - It is not called during init.
   - Return 0, no side effects (required): `codec_initCsCodec`, `HAL_I2S_Transmit_DMA`, `usb_tick`, `lcd_clear`, `lcd_home`, `lcd_string`, `lcd_setcursor`.
   - Return 0 (only if present): `lcd_turnOn`, `lcd_command`, `lcd_data`, `sendDisplayBuffer`, `HAL_Delay`, `DWT_Delay_us`.
2. **Jack-detect inputs** (read by `mixer_calcNextSampleBlock`). All four jacks are plugged in:
   - GPIOB->IDR (0x40020410) = `(1<<4)|(1<<6)` (l2, r2)
   - GPIOD->IDR (0x40020C10) = `(1<<6)|(1<<7)` (l1, r1)
3. **Reset_Handler** runs from vector word 1, with SP = vector word 0 and FPSCR = 0, until PC reaches `main`.
   - It copies .data/.ccm, zeroes .bss, and runs `SystemInit` and `__libc_init_array`.
4. **`main()` up to and including its first call**: `memcpy(_itcm_text_start=0, _itcm_data, len)` (the ITCM code copy).
   - Execution stops when `memcpy` returns into `main`. ITCM is then made read-only.
   - Nothing else of `main()` is executed: no `MX_*_Init`, LCD, SD, or USB.
5. **The FP context** has been created with FPSCR = 0 (a dummy FP instruction at `Firmware` creation).
6. **HAL handle instances**, normally written by `MX_*_Init`:
   - `htim2.Instance = TIM2 (0x40000000)`
   - `htim14.Instance = TIM14 (0x40002000)`
   - `huart3.Instance = USART3 (0x40004800)`
   - `hi2s2.Instance = SPI2 (0x40003800)`
   - `hi2s3.Instance = SPI3 (0x40003C00)`
7. **Init calls** (thread mode, FPSCR 0). This is the no-SD-card path of `main()`:
   1. `menu_init()`
   2. `soundengine_init()`
      - This includes `mixer_init`, `CodecInit` (with `codec_initCsCodec` stubbed), `parameterArray_init`, `euklid_init`, `modNode_init` ×6, `seq_init`, `seq_setBpm(120)`, the zeroing of `midi_MidiChannels`, and the voice inits.
      - **It renders one block** (`soundengine_calcNextSampleBlock`, with `bCurrentSampleValid` = 0 → buffer half 0). That block is not part of the output stream, but it advances all DSP state.
      - Finally it sets FPDSCR.FZ.
   3. `copyClear_clearCurrentPattern()`
   4. `preset_init(0)`
   5. `preset_loadDefaultGlobals()`
   6. `initMidiUart()`
8. **Faders at the top.** For i = 0..5:
   - Set `adc_buf[8+i] = 0xFFF0`, then call `adc_checkFaders()` 6 times. It handles one fader per call, round robin.
   - `adc_checkFaders` computes `v = (adc>>4)/4096.f`, flushes `v < 0.01` to 0, and stores **`adc_faderValues[i] = v*v`**.
     - For 0xFFF0 this is `FADER_FULL_VALUE` = 0.99951178f (bits **0x3F7FE001**). `adc_faderValues` is a .bss global (0.0 until then, i.e. silence).
     - The port must start with `adc_faderValues = [f32::from_bits(0x3F7FE001); 6]`.
   - Every voice's `FaderVolumeFilter` (`calcOnePole`) state starts at 0 and ramps towards that value once audio runs. The ramp is part of the reference output.
   - `adc_checkPots` is not called. All pots read 0, which is a no-op because of the hysteresis.
9. **Morph drain**: while `morph_state != 0`, call `preset_morph_tick()`.
   - `preset_init` → `preset_sendDrumsetParameters` → `preset_morph` start a non-blocking transfer. `preset_morph_tick` sends one parameter per call.

The resulting no-SD defaults:
- All `midi_MidiChannels` = 0 (every voice, and the global channel, on MIDI channel 1).
- `PAR_MIDI_NOTE1..7` = 48. Only note 48 triggers a voice, and it plays at the voice's own pitch.
- `midiParser_txRxFilter` = 0xFF, `midiParser_routing` = 0.
- Globals are only in `activeParameters`. `menu_sendAllGlobals` is not called without an SD card.
- `morphParameters` = NULL (see §8).

## 3. Per-block rendering (`tick() -> Block`)

```
blocks += 1
systick_ticks = floor(blocks * 16384 / 2825)                  // write u32
[FPSCR_IRQ]  HAL_I2S_TxCpltCallback(&hi2s3)                   // dmaPtr++, HAL_I2S_Transmit_DMA (stub),
                                                              // bCurrentSampleValid = 1 - (dmaPtr & 1)
half = bCurrentSampleValid                                    // 0 or 1
[audio FPSCR = 0]  seq_tick()
[audio FPSCR = 0]  soundengine_calcNextSampleBlock()          // mixer_calcNextSampleBlock(&dma_buffer[half*128],
                                                              //   &dma_buffer2[half*128]); bCurrentSampleValid = SAMPLE_VALID
dac1 = dma_buffer2[half*128 .. +128]   (int16, L/R interleaved, 64 frames)
dac2 = dma_buffer [half*128 .. +128]
```

Notes:
- **Systick.** `systick_ticks` is the 0.25 ms TIM14 counter. The I2S clock is PLLI2S = 1 MHz·113/5 = 22.6 MHz, so fs = 22.6 MHz/512 = **44140.625 Hz**. One 64-frame block is 16384/2825 systicks. After init `systick_ticks` = 0.
- **DAC naming follows the mixer.**
  - `mixer_calcNextSampleBlock(output, output2)` writes `MIXER_ROUTING_DAC1_*` (the default for every voice) to `output2`, which is `dma_buffer2`, played by SPI2/I2S2.
  - `Block.dac1` = `dma_buffer2`, `Block.dac2` = `dma_buffer`.
  - With the default routing, `dac2` is all zero.
- On the hardware the main loop is `seq_tick(); soundengine_calcNextSampleBlock(); uart_processMidi(); usb_tick(); …; preset_morph_tick()`.
  - The emulator runs `seq_tick` exactly once per rendered block.
  - It applies all MIDI/parameter events between blocks, which is equivalent to `uart_processMidi` after the previous block.
  - It drains morph transfers synchronously (see below).

## 4. MIDI input (`midi_byte(b)`, `midi(bytes)`)

For each byte:
1. Call `midiParser_parseUartData(b)`. This is what `uart_processMidi` does per byte from the DIN UART.
2. Drain morph (`preset_morph_tick` while `morph_state != 0`).

## 5. Parameter change (`set_parameter(param, value)`)

This mirrors `menu_encoderChangeParameter` after a new value has been chosen. `value` is not range-limited; the corpus only uses legal ranges (`tests/scenarios/params.json`).

1. `activeParameters[param] = value`
2. Let `dtype = parameter_dtypes[param] & 0xFF`. The entries are u16 with the menu id in the high byte. `modTargets[i]` is `{u8 nameIdx; u16 param}`, 4 bytes, so `.param` is at +2.
   - `DTYPE_TARGET_SELECTION_VELO` (8), voice = param − `PAR_VEL_DEST_1` (149):
     - `v = modTargets[value].param`
     - send `frontPanel_sendData(CC_VELO_TARGET=0xD8, ((v&0x80)>>7) | (voice<<1), v&0x7F)`
   - `DTYPE_VOICE_LFO` (9), lfo = param − `PAR_VOICE_LFO1` (161):
     - `t = getModTargetIdxFromGapIdx(value−1, menu_TargetVoiceGapIndex)`
     - `activeParameters[PAR_TARGET_LFO1 + lfo] = t`
     - `v = modTargets[t].param`
     - send `frontPanel_sendData(CC_LFO_TARGET=0xD7, ((v&0x80)>>7) | (lfo<<1), v&0x7F)`
   - `DTYPE_TARGET_SELECTION_LFO` (7), lfo = param − `PAR_TARGET_LFO1` (167):
     - `menu_TargetVoiceGapIndex = getModTargetGapIndex(value)`
     - `v = modTargets[value].param`
     - send `frontPanel_sendData(0xD7, ((v&0x80)>>7) | (lfo<<1), v&0x7F)`
3. Then:
   - `param < 128` → `frontPanel_sendData(MIDI_CC=0xB0, param, value)`
   - `param < END_OF_SOUND_PARAMETERS (247)` → `frontPanel_sendData(CC_2=0xD6, param−128, value)`
   - otherwise → `menu_parseGlobalParam(param, value)`
4. Drain morph.

## 6. Kit load (`load_kit(bytes)`)

`bytes` is a kit file: an 8-byte name followed by up to 247 parameter bytes (8..=255 bytes in total). This mirrors `preset_loadDrumsetSD` into slot 0, followed by the LOAD menu's `preset_loadDrumsetRam(0,0,0)`.

1. Name:
   - If all 8 name bytes are ' ' or 0, the name is `"Empty\0\0\0"`.
   - Write it to `savefileNames[0][0..8]`.
2. Parameters:
   - `params[0..247]` = the file bytes after the name. Missing bytes are 0.
   - For i in 0..6, set `params[PAR_VEL_DEST_1+i]` and `params[PAR_TARGET_LFO1+i]` to 0 if they are ≥ `getNumModTargets()` (229).
   - Write the result to `mainParameters[0][0..247]`.
3. Call `preset_loadDrumsetRam(0, 0, 0)`.
4. Drain morph.

## 7. Faders (`set_fader_adc(i, adc)`, `set_fader_value(i, f)`, `fader_values()`)

- `set_fader_adc(i, adc)`: set `adc_buf[8+i] = adc` (12 bits, left-aligned in 16), then call `adc_checkFaders()` 6 times.
  - The hysteresis is 40 counts on `adc>>4` against the last accepted reading. A smaller move leaves the value unchanged.
- `set_fader_value(i, f)` writes `adc_faderValues[i]` directly, bypassing the ADC path. The scenario format does not use it.
- `fader_values()` reads the six values. They are what `adc_getFaderVolume` returns, and they feed each voice's `FaderVolumeFilter`.

## 8. Scenarios and output format

Scenario JSON (`crates/lxr-emu/src/scenario.rs`):

```json
{ "name": "x", "description": "...", "ticks": 300,
  "events": [ { "tick": 0, "events": [
      {"midi": [144, 48, 100]},
      {"set_parameter": {"param": 50, "value": 20}},
      {"load_kit": {"hex": "4b6974..."}},
      {"fader": {"index": 0, "adc": 32768}} ] } ] }
```

- Events with `tick = n` are applied in file order before the (n+1)-th `tick()`.
- Raw output: for each block, DAC1 (128 × i16 LE), then DAC2 (128 × i16 LE).
- The scenario hash is the SHA-256 of the whole stream.
- Reference hashes for the shipped v1.71 are in `tests/scenarios/reference_v1.71.json`.

Corpus (`tools/gen_scenarios.py`, deterministic):
- 311 scenarios, 101594 blocks in total.
- Most scenarios (all but `default_boot_*` and a few `global_*`/`midi*` ones) start with a prologue: `PAR_MIDI_CHAN_v = v` (voice v on MIDI channel v) and `PAR_MIDI_NOTEv = 0` (any note), so that notes can be played at different pitches.

CLI:

```
lxr-emu render (--img <img> | --elf <elf>) <scenario.json> --out <file.raw> [--fz]
lxr-emu hashes (--img|--elf) <files|dirs> [--json out.json] [--jobs N] [--fz]
lxr-emu check  (--img|--elf) tests/scenarios/reference_v1.71.json tests/scenarios [--jobs N]
```

## 9. Behaviour outside the contract (excluded from the corpus)

- **Out-of-range oscillator waveforms.** MIDI CC 2,3,4,5,7,8,21–27 (waveform parameters) with values above 5 select "sample playback".
  - Without samples, this leaves the oscillator buffer unwritten (stale stack).
  - The output depends on stack contents, so it differs between the shipped binary and the ELF, and it is not deterministic on the hardware.
  - The generator clamps these CC values to ≤ 5.
- **Morph without a loaded morph kit.** `morphParameters` is NULL without an SD card, so `PAR_MORPH` ≠ 0 interpolates towards bytes read from address 0 (ITCM code).
  - `global_morph_no_target` is in the corpus and is the only scenario whose ELF output differs from the shipped one, because the ITCM code differs.
  - The port should not try to match it except as "reads the code bytes of the shipped ITCM".
- Not modelled: pots, buttons/front panel input, LCD/menu navigation, the SD card, USB MIDI, the sequencer clock beyond `seq_tick` once per block, project save (TIM2 interrupt), and DIN MIDI output (`midiParser_txRxFilter` routing is executed but the UART TX is not observed).
