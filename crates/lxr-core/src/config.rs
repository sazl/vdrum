//! Port of `Core/Inc/Soundengine/config.h`.
//!
//! Porting group: P3. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;

// ---- #define constants
/// `#define CALC_SAMPLES_IN_INTERRUPT 1` (Core/Inc/Soundengine/config.h:29)
pub const CALC_SAMPLES_IN_INTERRUPT: i32 = 1;
/// `#define USE_INTERRUPT_ONLY_DURING_SAVE_PROJ 1` (Core/Inc/Soundengine/config.h:30)
pub const USE_INTERRUPT_ONLY_DURING_SAVE_PROJ: i32 = 1;
/// `#define USE_BIG_SAMPLE_BUFFER 1` (Core/Inc/Soundengine/config.h:31)
pub const USE_BIG_SAMPLE_BUFFER: i32 = 1;
/// `#define SMOOTH_MIXER_VOL_PAN_DRIVE 1` (Core/Inc/Soundengine/config.h:33)
pub const SMOOTH_MIXER_VOL_PAN_DRIVE: i32 = 1;
/// `#define UART_DEBUG_ECHO_MODE 0` (Core/Inc/Soundengine/config.h:35)
pub const UART_DEBUG_ECHO_MODE: i32 = 0;
/// `#define OUTPUT_DMA_SIZE 64` (Core/Inc/Soundengine/config.h:38)
pub const OUTPUT_DMA_SIZE: i32 = 64;
/// `#define DMA_MASK ((OUTPUT_DMA_SIZE*2)-1)` (Core/Inc/Soundengine/config.h:42)
pub const DMA_MASK: i32 = 127;
/// `#define UNIT_GAIN_DRIVE 0` (Core/Inc/Soundengine/config.h:43)
pub const UNIT_GAIN_DRIVE: i32 = 0;
/// `#define SET_PARAM_ARRAY_IN_PARSER 1` (Core/Inc/Soundengine/config.h:44)
pub const SET_PARAM_ARRAY_IN_PARSER: i32 = 1;
/// `#define USE_FILTER_DRIVE 0` (Core/Inc/Soundengine/config.h:45)
pub const USE_FILTER_DRIVE: i32 = 0;
/// `#define CALC_TONE_CONTROL 0` (Core/Inc/Soundengine/config.h:46)
pub const CALC_TONE_CONTROL: i32 = 0;
/// `#define AMP_EG_SYNC 0` (Core/Inc/Soundengine/config.h:51)
pub const AMP_EG_SYNC: i32 = 0;
/// `#define ENABLE_MIX_OSC 1` (Core/Inc/Soundengine/config.h:54)
pub const ENABLE_MIX_OSC: i32 = 1;
/// `#define ENABLE_DRUM_SVF 1` (Core/Inc/Soundengine/config.h:55)
pub const ENABLE_DRUM_SVF: i32 = 1;
/// `#define PITCH_AMOUNT_FACTOR 32` (Core/Inc/Soundengine/config.h:58)
pub const PITCH_AMOUNT_FACTOR: i32 = 32;
/// `#define USE_DAC2 1` (Core/Inc/Soundengine/config.h:61)
pub const USE_DAC2: i32 = 1;
/// `#define INTERPOLATE_OSC 1` (Core/Inc/Soundengine/config.h:66)
pub const INTERPOLATE_OSC: i32 = 1;
/// `#define INTERPOLATE_FM_OSC 1` (Core/Inc/Soundengine/config.h:67)
pub const INTERPOLATE_FM_OSC: i32 = 1;
/// `#define USE_BOOTLOADER 1` (Core/Inc/Soundengine/config.h:69)
pub const USE_BOOTLOADER: i32 = 1;
/// `#define SYNTH_FS I2S_AUDIOFREQ_44K` (Core/Inc/Soundengine/config.h:72)
pub const SYNTH_FS: u32 = 44100;
/// `#define REAL_FS (float)44140.f` (Core/Inc/Soundengine/config.h:74) = 44140.0
pub const REAL_FS: f32 = f32::from_bits(0x472c6c00);
/// `#define REAL_FS2 (REAL_FS/2.f)` (Core/Inc/Soundengine/config.h:75) = 22070.0
pub const REAL_FS2: f32 = f32::from_bits(0x46ac6c00);
/// `#define NUM_DRUM_VOICES 3` (Core/Inc/Soundengine/config.h:78)
pub const NUM_DRUM_VOICES: i32 = 3;

// ---- C macros that are not constants (function-like or not constant expressions); not translated:
// Core/Inc/Soundengine/config.h:57: #define EG_SPEED 1;    (not a constant expression; not translated)
// Core/Inc/Soundengine/config.h:92: #define IN_ITCM_RAM __attribute__((section(".itcm_text")))    (not a constant expression; not translated)
