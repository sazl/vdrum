//! Whole-engine harness: runs the LXR-02 firmware's own startup, sound-engine
//! initialisation, MIDI parser, parameter path and audio rendering in the emulator.
//!
//! The exact call sequence is specified in `docs/ENGINE_CONTRACT.md`; keep both in sync.

use crate::firmware::{Firmware, HookState, FLASH_BASE, FPSCR_FZ, ITCM_BASE, ITCM_SIZE};
use crate::symbols;
use anyhow::{anyhow, bail, Context, Result};
use sha2::{Digest, Sha256};
use std::cell::Cell;
use std::rc::Rc;
use unicorn_engine::{Prot, RegisterARM, Unicorn};

/// Stereo frames per block (`OUTPUT_DMA_SIZE`).
pub const OUTPUT_DMA_SIZE: usize = 64;
/// Samples (L/R interleaved) per DAC per block.
pub const BLOCK_SAMPLES: usize = OUTPUT_DMA_SIZE * 2;

/// Seed of the xorshift32 generator that replaces the hardware RNG.
pub const RNG_SEED: u32 = 0x2545_F491;

/// FPSCR of the main thread (thread mode). At reset FPDSCR = 0; the first FP
/// instruction in thread mode creates the FP context with FPSCR[26:22] = FPDSCR[26:22]
/// = 0, and no code writes FPSCR. Main-loop code therefore runs with FZ = DN = 0, RN.
pub const FPSCR_MAIN: u32 = 0;
/// FPSCR of exception handlers after `soundengine_init` set FPDSCR.FZ.
pub const FPSCR_IRQ: u32 = FPSCR_FZ;

/// Systick (TIM14, 0.25 ms) ticks after `n` blocks: the I2S clock is
/// PLLI2S = 1 MHz * 113 / 5 = 22.6 MHz, fs = 22.6 MHz / 512 = 44140.625 Hz,
/// one block = 64 / fs s = 16384 / 2825 systicks.
pub fn systick_after_blocks(n: u64) -> u32 {
    ((n * 16384) / 2825) as u32
}

// Peripheral base addresses (STM32F765) for the HAL handle `Instance` fields that
// MX_*_Init would have set.
const TIM2: u32 = 0x4000_0000;
const TIM14: u32 = 0x4000_2000;
const SPI2: u32 = 0x4000_3800;
const SPI3: u32 = 0x4000_3C00;
const USART3: u32 = 0x4000_4800;
/// GPIOB->IDR and GPIOD->IDR.
pub const GPIOB_IDR: u64 = 0x4002_0410;
pub const GPIOD_IDR: u64 = 0x4002_0C10;

// Constants from the firmware headers.
pub const END_OF_SOUND_PARAMETERS: u16 = 247;
pub const NUM_PARAMS: u16 = 302;
pub const PAR_VEL_DEST_1: u16 = 149;
pub const PAR_VOICE_LFO1: u16 = 161;
pub const PAR_TARGET_LFO1: u16 = 167;
pub const MIDI_CC: u8 = 0xB0;
pub const CC_2: u8 = 0xD6;
pub const CC_LFO_TARGET: u8 = 0xD7;
pub const CC_VELO_TARGET: u8 = 0xD8;
const DTYPE_TARGET_SELECTION_LFO: u16 = 7;
const DTYPE_TARGET_SELECTION_VELO: u16 = 8;
const DTYPE_VOICE_LFO: u16 = 9;
/// Kit file: 8 byte name + END_OF_SOUND_PARAMETERS parameter bytes.
pub const KIT_FILE_SIZE: usize = 8 + END_OF_SOUND_PARAMETERS as usize;
/// ADC value of a fader at the top (12 bit, left aligned in 16 bit: 4095 << 4).
pub const FADER_FULL: u16 = 0xFFF0;
/// adc_faderValues[i] for FADER_FULL: adc_checkFaders computes
/// v = (adc >> 4) / 4096.f = 4095/4096, stores v*v (f32) = 0.99951178 (bits 0x3F7FE001).
pub const FADER_FULL_VALUE: f32 = f32::from_bits(0x3F7F_E001);

/// One rendered block. Naming follows the mixer: `dac1` is what mixer.c routes to
/// MIXER_ROUTING_DAC1_* (the default routing of every voice; written through its
/// `output2` argument = `dma_buffer2`), `dac2` is MIXER_ROUTING_DAC2_* (`dma_buffer`).
/// Each holds 64 frames, L/R interleaved.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Block {
    pub dac1: [i16; BLOCK_SAMPLES],
    pub dac2: [i16; BLOCK_SAMPLES],
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Build {
    /// Shipped v1.71 payload + recovered symbol map.
    Shipped,
    /// ELF with symbols (GCC 11.3 reference build).
    Elf,
}

/// In which context `tick()` renders audio.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AudioContext {
    /// `soundengine_process()` in the main loop (what the v1.71 firmware does in normal
    /// operation: `USE_INTERRUPT_ONLY_DURING_SAVE_PROJ` = 1, TIM2 interrupt not started).
    MainLoop,
    /// TIM2 interrupt (only active while a project is saved): FZ = 1 from FPDSCR.
    Tim2Interrupt,
}

/// Resolved firmware addresses used by the engine.
#[derive(Clone, Debug)]
struct Addrs {
    reset_handler: u32,
    main: u32,
    memcpy: u32,
    menu_init: u32,
    soundengine_init: u32,
    copy_clear_current_pattern: u32,
    preset_init: u32,
    preset_load_default_globals: u32,
    init_midi_uart: u32,
    adc_check_faders: u32,
    preset_morph_tick: u32,
    midi_parse_uart_data: u32,
    seq_tick: u32,
    calc_next_sample_block: u32,
    i2s_tx_cplt_callback: u32,
    front_panel_send_data: u32,
    menu_parse_global_param: u32,
    get_mod_target_idx_from_gap_idx: u32,
    get_mod_target_gap_index: u32,
    get_num_mod_targets: u32,
    preset_load_drumset_ram: u32,
    // data
    active_parameters: u32,
    main_parameters: u32,
    savefile_names: u32,
    parameter_dtypes: u32,
    mod_targets: u32,
    menu_target_voice_gap_index: u32,
    morph_state: u32,
    dma_buffer: u32,
    dma_buffer2: u32,
    b_current_sample_valid: u32,
    dma_ptr: u32,
    systick_ticks: u32,
    adc_buf: u32,
    htim2: u32,
    htim14: u32,
    huart3: u32,
    hi2s2: u32,
    hi2s3: u32,
}

impl Addrs {
    fn resolve(fw: &Firmware) -> Result<Self> {
        let f = |n: &str| fw.addr(n).with_context(|| format!("symbol {n} missing"));
        Ok(Addrs {
            reset_handler: fw.read_u32(crate::firmware::APP_BASE as u32 + 4)? & !1,
            main: f("main")?,
            memcpy: f("memcpy")?,
            menu_init: f("menu_init")?,
            soundengine_init: f("soundengine_init")?,
            copy_clear_current_pattern: f("copyClear_clearCurrentPattern")?,
            preset_init: f("preset_init")?,
            preset_load_default_globals: f("preset_loadDefaultGlobals")?,
            init_midi_uart: f("initMidiUart")?,
            adc_check_faders: f("adc_checkFaders")?,
            preset_morph_tick: f("preset_morph_tick")?,
            midi_parse_uart_data: f("midiParser_parseUartData")?,
            seq_tick: f("seq_tick")?,
            calc_next_sample_block: f("soundengine_calcNextSampleBlock")?,
            i2s_tx_cplt_callback: f("HAL_I2S_TxCpltCallback")?,
            front_panel_send_data: f("frontPanel_sendData")?,
            menu_parse_global_param: f("menu_parseGlobalParam")?,
            get_mod_target_idx_from_gap_idx: f("getModTargetIdxFromGapIdx")?,
            get_mod_target_gap_index: f("getModTargetGapIndex")?,
            get_num_mod_targets: f("getNumModTargets")?,
            preset_load_drumset_ram: f("preset_loadDrumsetRam")?,
            active_parameters: f("activeParameters")?,
            main_parameters: f("mainParameters")?,
            savefile_names: f("savefileNames")?,
            parameter_dtypes: f("parameter_dtypes")?,
            mod_targets: f("modTargets")?,
            menu_target_voice_gap_index: f("menu_TargetVoiceGapIndex")?,
            morph_state: f("morph_state")?,
            dma_buffer: f("dma_buffer")?,
            dma_buffer2: f("dma_buffer2")?,
            b_current_sample_valid: f("bCurrentSampleValid")?,
            dma_ptr: f("dmaPtr")?,
            systick_ticks: f("systick_ticks")?,
            adc_buf: f("adc_buf")?,
            htim2: f("htim2")?,
            htim14: f("htim14")?,
            huart3: f("huart3")?,
            hi2s2: f("hi2s2")?,
            hi2s3: f("hi2s3")?,
        })
    }
}

/// Functions replaced by host stubs (hardware access only). They return 0 (HAL_OK)
/// unless noted. A missing symbol is an error for the ones marked required.
const STUBS: &[(&str, bool)] = &[
    ("codec_initCsCodec", true), // I2S/DMA/GPIO setup of the codecs (CodecInit keeps dma_buffer[0] = 32756)
    ("HAL_I2S_Transmit_DMA", true), // restarting the DMA in the transfer-complete callback
    ("usb_tick", true),          // USB MIDI flush (CDC_Transmit_FS)
    ("lcd_clear", true),
    ("lcd_home", true),
    ("lcd_string", true),
    ("lcd_setcursor", true),
    ("lcd_turnOn", false),
    ("lcd_command", false),
    ("lcd_data", false),
    ("sendDisplayBuffer", false),
    ("HAL_Delay", false),
    ("DWT_Delay_us", false),
];

pub struct EmuEngine {
    pub fw: Firmware,
    pub build: Build,
    a: Addrs,
    rng: Rc<Cell<u32>>,
    rng_calls: Rc<Cell<u64>>,
    blocks: u64,
    /// FPSCR for main-loop code (init, MIDI, parameters, kit loading).
    pub main_fpscr: u32,
    /// FPSCR for `tick()` (default: main loop, see `AudioContext`).
    pub audio_fpscr: u32,
}

impl EmuEngine {
    /// Shipped SD-card update image (`LXRV2_update_v1.71.img`) + embedded symbol map.
    pub fn new_shipped(img_path: impl AsRef<std::path::Path>) -> Result<Self> {
        let img = std::fs::read(img_path.as_ref())
            .with_context(|| format!("read {:?}", img_path.as_ref()))?;
        let payload = symbols::image_payload(&img)?;
        let sha = hex(&Sha256::digest(payload));
        if payload.len() != symbols::V171_PAYLOAD_SIZE || sha != symbols::V171_PAYLOAD_SHA256 {
            bail!("image payload is not v1.71 (sha256 {sha}); the symbol map does not apply");
        }
        let syms = symbols::parse_symbol_map(symbols::V171_SYMBOLS_JSON)?;
        let fw = Firmware::from_bin(payload, syms)?;
        Self::boot(fw, Build::Shipped)
    }

    /// Firmware ELF with symbols (e.g. the GCC 11.3 reference build).
    pub fn new_elf(elf_path: impl AsRef<std::path::Path>) -> Result<Self> {
        let data = std::fs::read(elf_path.as_ref())
            .with_context(|| format!("read {:?}", elf_path.as_ref()))?;
        let fw = Firmware::from_elf(&data)?;
        Self::boot(fw, Build::Elf)
    }

    /// Select the FPSCR used by `tick()`.
    pub fn set_audio_context(&mut self, ctx: AudioContext) {
        self.audio_fpscr = match ctx {
            AudioContext::MainLoop => self.main_fpscr,
            AudioContext::Tim2Interrupt => FPSCR_IRQ,
        };
    }

    /// Number of `GetRngValue` calls so far.
    pub fn rng_calls(&self) -> u64 {
        self.rng_calls.get()
    }

    /// Blocks rendered so far.
    pub fn blocks(&self) -> u64 {
        self.blocks
    }

    fn boot(mut fw: Firmware, build: Build) -> Result<Self> {
        let a = Addrs::resolve(&fw)?;
        let rng = Rc::new(Cell::new(RNG_SEED));
        let rng_calls = Rc::new(Cell::new(0u64));
        // --- intercepts ---
        {
            let rng = rng.clone();
            let calls = rng_calls.clone();
            fw.intercept(
                fw.addr("GetRngValue")?,
                move |uc: &mut Unicorn<HookState>| {
                    let mut x = rng.get();
                    x ^= x << 13;
                    x ^= x >> 17;
                    x ^= x << 5;
                    rng.set(x);
                    calls.set(calls.get() + 1);
                    uc.reg_write(RegisterARM::R0, x as u64).unwrap();
                },
            )?;
        }
        for (name, required) in STUBS {
            match fw.addr(name) {
                Ok(addr) => fw.intercept(addr, |uc: &mut Unicorn<HookState>| {
                    uc.reg_write(RegisterARM::R0, 0).unwrap();
                })?,
                Err(e) if *required => return Err(e),
                Err(_) => {}
            }
        }
        // Jack detection switches (main.h): l1 = PD6, r1 = PD7, l2 = PB4, r2 = PB6, read
        // from the GPIO input data registers by mixer_calcNextSampleBlock. All plugged in.
        {
            let st = fw.uc.get_data_mut();
            st.mmio_values.insert(GPIOB_IDR, (1 << 4) | (1 << 6));
            st.mmio_values.insert(GPIOD_IDR, (1 << 6) | (1 << 7));
        }
        let mut eng = EmuEngine {
            fw,
            build,
            a,
            rng,
            rng_calls,
            blocks: 0,
            main_fpscr: FPSCR_MAIN,
            audio_fpscr: FPSCR_MAIN,
        };
        eng.startup()?;
        eng.power_on_init()?;
        Ok(eng)
    }

    /// Reset_Handler (copy .data/.dtcm, zero .bss/.dtcmz, SystemInit, __libc_init_array)
    /// up to the call of main(), then main() up to and including its ITCM memcpy.
    fn startup(&mut self) -> Result<()> {
        let a = self.a.clone();
        let limit = 50_000_000;
        let uc = &mut self.fw.uc;
        let sp0 = u32::from_le_bytes(
            uc.mem_read_as_vec(crate::firmware::APP_BASE, 4)
                .unwrap()
                .try_into()
                .unwrap(),
        );
        uc.reg_write(RegisterARM::SP, sp0 as u64)
            .map_err(|e| anyhow!("{e:?}"))?;
        uc.reg_write(RegisterARM::FPSCR, FPSCR_MAIN as u64)
            .map_err(|e| anyhow!("{e:?}"))?;
        uc.emu_start((a.reset_handler | 1) as u64, a.main as u64, 0, limit)
            .map_err(|e| {
                anyhow!(
                    "Reset_Handler: {e:?} at pc={:#x}",
                    uc.reg_read(RegisterARM::PC).unwrap_or(0)
                )
            })?;
        let pc = uc.reg_read(RegisterARM::PC).unwrap() as u32;
        if pc & !1 != a.main {
            bail!("Reset_Handler did not reach main (pc={pc:#x})");
        }
        // main(): runs until memcpy is entered (the ITCM copy is its first call) ...
        uc.emu_start((a.main | 1) as u64, a.memcpy as u64, 0, limit)
            .map_err(|e| anyhow!("main prologue: {e:?}"))?;
        let pc = uc.reg_read(RegisterARM::PC).unwrap() as u32;
        if pc & !1 != a.memcpy {
            bail!("main did not call memcpy first (pc={pc:#x})");
        }
        let dst = uc.reg_read(RegisterARM::R0).unwrap() as u32;
        let src = uc.reg_read(RegisterARM::R1).unwrap() as u32;
        let len = uc.reg_read(RegisterARM::R2).unwrap() as u32;
        if dst != ITCM_BASE as u32
            || !(FLASH_BASE as u32..FLASH_BASE as u32 + 0x10_0000).contains(&src)
            || len as u64 > ITCM_SIZE
        {
            bail!("unexpected first memcpy in main: dst={dst:#x} src={src:#x} len={len:#x}");
        }
        let ret = uc.reg_read(RegisterARM::LR).unwrap() as u32;
        // ... and through memcpy back into main.
        uc.emu_start((a.memcpy | 1) as u64, (ret & !1) as u64, 0, limit)
            .map_err(|e| anyhow!("ITCM memcpy: {e:?}"))?;
        // ITCM code is never written afterwards: make it read-only to catch stray writes
        // (e.g. through uninitialised HAL handles).
        uc.mem_protect(ITCM_BASE, ITCM_SIZE, Prot::READ | Prot::EXEC)
            .map_err(|e| anyhow!("protect ITCM: {e:?}"))?;
        Ok(())
    }

    fn call_main(&mut self, addr: u32, ints: &[u32]) -> Result<u32> {
        self.fw.fpscr = self.main_fpscr;
        Ok(self.fw.call_addr(addr, ints, &[])?.0)
    }

    /// The part of main() after the ITCM copy that matters for the sound engine, for
    /// the boot without SD card (see docs/ENGINE_CONTRACT.md).
    fn power_on_init(&mut self) -> Result<()> {
        let a = self.a.clone();
        // HAL handle instances (what MX_TIM2_Init, MX_TIM14_Init, MX_USART3_UART_Init,
        // MX_I2S2_Init, MX_I2S3_Init store); peripheral register accesses go to MMIO.
        for (h, inst) in [
            (a.htim2, TIM2),
            (a.htim14, TIM14),
            (a.huart3, USART3),
            (a.hi2s2, SPI2),
            (a.hi2s3, SPI3),
        ] {
            self.fw.write_u32(h, inst)?;
        }
        self.call_main(a.menu_init, &[])?;
        self.call_main(a.soundengine_init, &[])?;
        // no SD card: main() takes the else branch
        self.call_main(a.copy_clear_current_pattern, &[])?;
        self.call_main(a.preset_init, &[0])?;
        self.call_main(a.preset_load_default_globals, &[])?;
        self.call_main(a.init_midi_uart, &[])?;
        // Main loop, first iteration(s): faders at the top.
        for i in 0..6 {
            self.set_fader_adc(i, FADER_FULL)?;
        }
        // preset_init -> preset_sendDrumsetParameters -> preset_morph started a
        // non-blocking parameter transfer; complete it.
        self.drain_morph()?;
        Ok(())
    }

    /// Set fader `i` (0..6) to an ADC reading and run adc_checkFaders until it has
    /// visited that fader (it handles one fader per call, round robin).
    pub fn set_fader_adc(&mut self, i: usize, value: u16) -> Result<()> {
        if i >= 6 {
            bail!("fader index {i}");
        }
        let a = self.a.clone();
        self.fw.write_u16(a.adc_buf + 2 * (8 + i as u32), value)?;
        for _ in 0..6 {
            self.call_main(a.adc_check_faders, &[])?;
        }
        Ok(())
    }

    /// `adc_faderValues[0..6]` (what `adc_getFaderVolume` returns). After boot every
    /// entry is `FADER_FULL_VALUE` (faders at the top).
    pub fn fader_values(&self) -> Result<[f32; 6]> {
        let base = self.fw.addr("adc_faderValues")?;
        let mut v = [0f32; 6];
        for (i, x) in v.iter_mut().enumerate() {
            *x = self.fw.read_f32(base + 4 * i as u32)?;
        }
        Ok(v)
    }

    /// Write `adc_faderValues[i]` directly, bypassing the ADC path (adc_checkFaders,
    /// its hysteresis and the x^2 law). A later `set_fader_adc` may overwrite it.
    pub fn set_fader_value(&mut self, i: usize, value: f32) -> Result<()> {
        if i >= 6 {
            bail!("fader index {i}");
        }
        let base = self.fw.addr("adc_faderValues")?;
        self.fw.write(base + 4 * i as u32, &value.to_le_bytes())
    }

    /// Run preset_morph_tick until the pending kit/morph transfer is complete
    /// (on the hardware this happens within a few main-loop iterations).
    fn drain_morph(&mut self) -> Result<()> {
        let a = self.a.clone();
        let mut n = 0;
        while self.fw.read_u8(a.morph_state)? != 0 {
            self.call_main(a.preset_morph_tick, &[])?;
            n += 1;
            if n > 10_000 {
                bail!("morph does not terminate");
            }
        }
        Ok(())
    }

    /// One byte of DIN MIDI input (what uart_processMidi does per received byte).
    pub fn midi_byte(&mut self, b: u8) -> Result<()> {
        let a = self.a.clone();
        self.call_main(a.midi_parse_uart_data, &[b as u32])?;
        self.drain_morph()
    }

    pub fn midi(&mut self, bytes: &[u8]) -> Result<()> {
        for &b in bytes {
            self.midi_byte(b)?;
        }
        Ok(())
    }

    fn front_panel_send(&mut self, status: u8, d1: u8, d2: u8) -> Result<()> {
        let a = self.a.clone();
        self.call_main(
            a.front_panel_send_data,
            &[status as u32, d1 as u32, d2 as u32],
        )?;
        Ok(())
    }

    /// Change a parameter as the menu does when the encoder edits it
    /// (menu_encoderChangeParameter after the value has been chosen): the value is
    /// stored in activeParameters and sent through frontPanel_sendData (sound
    /// parameters) or menu_parseGlobalParam (others). `value` is not range-limited.
    pub fn set_parameter(&mut self, param: u16, value: u8) -> Result<()> {
        if param >= NUM_PARAMS {
            bail!("parameter {param} out of range");
        }
        let a = self.a.clone();
        let p = param as u32;
        self.fw.write_u8(a.active_parameters + p, value)?;
        let dtype = self.fw.read_u16(a.parameter_dtypes + 2 * p)? & 0xFF;
        let mod_param = |eng: &mut Self, idx: u8| -> Result<u8> {
            // modTargets[idx].param (struct {uint8_t nameIdx; uint16_t param;}, 4 bytes)
            Ok(eng.fw.read_u16(a.mod_targets + 4 * idx as u32 + 2)? as u8)
        };
        match dtype {
            DTYPE_TARGET_SELECTION_VELO => {
                let voice = (param - PAR_VEL_DEST_1) as u8;
                let v = mod_param(self, value)?;
                let upper = ((v & 0x80) >> 7) | ((voice & 0x3f) << 1);
                self.front_panel_send(CC_VELO_TARGET, upper, v & 0x7f)?;
            }
            DTYPE_VOICE_LFO => {
                let lfo = (param - PAR_VOICE_LFO1) as u8;
                let gap = self.fw.read_u8(a.menu_target_voice_gap_index)?;
                let new_targ = self.call_main(
                    a.get_mod_target_idx_from_gap_idx,
                    &[value.wrapping_sub(1) as u32, gap as u32],
                )? as u8;
                self.fw.write_u8(
                    a.active_parameters + (PAR_TARGET_LFO1 as u32 + lfo as u32),
                    new_targ,
                )?;
                let v = mod_param(self, new_targ)?;
                let upper = ((v & 0x80) >> 7) | ((lfo & 0x3f) << 1);
                self.front_panel_send(CC_LFO_TARGET, upper, v & 0x7f)?;
            }
            DTYPE_TARGET_SELECTION_LFO => {
                let lfo = (param - PAR_TARGET_LFO1) as u8;
                let gap = self.call_main(a.get_mod_target_gap_index, &[value as u32])? as u8;
                self.fw.write_u8(a.menu_target_voice_gap_index, gap)?;
                let v = mod_param(self, value)?;
                let upper = ((v & 0x80) >> 7) | ((lfo & 0x3f) << 1);
                self.front_panel_send(CC_LFO_TARGET, upper, v & 0x7f)?;
            }
            _ => {}
        }
        if param < 128 {
            self.front_panel_send(MIDI_CC, param as u8, value)?;
        } else if param < END_OF_SOUND_PARAMETERS {
            self.front_panel_send(CC_2, (param - 128) as u8, value)?;
        } else {
            self.call_main(a.menu_parse_global_param, &[p, value as u32])?;
        }
        self.drain_morph()
    }

    /// Load a kit (contents of a kit file: 8 byte name + parameter bytes) into kit slot 0
    /// the way preset_loadDrumsetSD reads it (missing bytes = 0, invalid mod targets
    /// = 0), then apply it as the LOAD menu does: preset_loadDrumsetRam(0, 0, 0).
    pub fn load_kit(&mut self, kit: &[u8]) -> Result<()> {
        if kit.len() < 8 || kit.len() > KIT_FILE_SIZE {
            bail!("kit must be 8..={KIT_FILE_SIZE} bytes, got {}", kit.len());
        }
        let a = self.a.clone();
        let mut name = [0u8; 8];
        name.copy_from_slice(&kit[..8]);
        if name.iter().all(|&c| c == b' ' || c == 0) {
            name = *b"Empty\0\0\0";
        }
        self.fw.write(a.savefile_names, &name)?; // savefileNames[0][KIT_NAMES]
        let mut params = vec![0u8; END_OF_SOUND_PARAMETERS as usize];
        params[..kit.len() - 8].copy_from_slice(&kit[8..]);
        let nmt = self.call_main(a.get_num_mod_targets, &[])? as u8;
        for i in 0..6 {
            for base in [PAR_VEL_DEST_1, PAR_TARGET_LFO1] {
                let k = (base + i) as usize;
                if params[k] >= nmt {
                    params[k] = 0;
                }
            }
        }
        self.fw.write(a.main_parameters, &params)?; // mainParameters[0]
        self.call_main(a.preset_load_drumset_ram, &[0, 0, 0])?;
        self.drain_morph()
    }

    /// Render one block: the SPI3 (DAC 2) DMA transfer-complete interrupt selects the
    /// buffer half, then the main loop's soundengine_process() runs
    /// `seq_tick(); soundengine_calcNextSampleBlock();`.
    pub fn tick(&mut self) -> Result<Block> {
        let a = self.a.clone();
        self.blocks += 1;
        // TIM14 interrupts (systick_ticks++) that happened during the last block.
        let st = systick_after_blocks(self.blocks);
        self.fw.write_u32(a.systick_ticks, st)?;
        // DMA interrupt: HAL_I2S_TxCpltCallback(&hi2s3) -> bCurrentSampleValid = 1-(dmaPtr&1)
        self.fw.fpscr = FPSCR_IRQ;
        self.fw.call_addr(a.i2s_tx_cplt_callback, &[a.hi2s3], &[])?;
        let half = self.fw.read_u8(a.b_current_sample_valid)? as u32;
        if half > 1 {
            bail!("bCurrentSampleValid = {half} after DMA callback");
        }
        self.fw.fpscr = self.audio_fpscr;
        self.fw.call_addr(a.seq_tick, &[], &[])?;
        self.fw.call_addr(a.calc_next_sample_block, &[], &[])?;
        let off = half * BLOCK_SAMPLES as u32 * 2;
        let mut blk = Block {
            dac1: [0; BLOCK_SAMPLES],
            dac2: [0; BLOCK_SAMPLES],
        };
        blk.dac1
            .copy_from_slice(&self.fw.read_i16s(a.dma_buffer2 + off, BLOCK_SAMPLES)?);
        blk.dac2
            .copy_from_slice(&self.fw.read_i16s(a.dma_buffer + off, BLOCK_SAMPLES)?);
        Ok(blk)
    }

    /// Address of a firmware symbol (for raw memory inspection).
    pub fn addr(&self, name: &str) -> Result<u32> {
        self.fw.addr(name)
    }

    pub fn active_parameter(&self, param: u16) -> Result<u8> {
        self.fw.read_u8(self.a.active_parameters + param as u32)
    }

    /// Current xorshift32 state.
    pub fn rng_state(&self) -> u32 {
        self.rng.get()
    }

    /// dmaPtr (DMA half counter of DAC 2).
    pub fn dma_ptr(&self) -> Result<u8> {
        self.fw.read_u8(self.a.dma_ptr)
    }
}

pub fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}
