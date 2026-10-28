//! Port of `Core/Inc/Soundengine/DSPAudio/SlopeEg2.h`, `Core/Src/Soundengine/DSPAudio/SlopeEg2.c`.
//!
//! Porting group: P1. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;

// ---- #define constants
/// `#define EG_STOPPED 0` (Core/Inc/Soundengine/DSPAudio/SlopeEg2.h:23)
pub const EG_STOPPED: i32 = 0;
/// `#define EG_A 1` (Core/Inc/Soundengine/DSPAudio/SlopeEg2.h:24)
pub const EG_A: i32 = 1;
/// `#define EG_D 2` (Core/Inc/Soundengine/DSPAudio/SlopeEg2.h:25)
pub const EG_D: i32 = 2;
/// `#define EG_REPEAT 3` (Core/Inc/Soundengine/DSPAudio/SlopeEg2.h:26)
pub const EG_REPEAT: i32 = 3;
/// `#define TIME_AMOUNT_DECAY 0.999` (Core/Src/Soundengine/DSPAudio/SlopeEg2.c:133) = 0.999
pub const TIME_AMOUNT_DECAY: f64 = f64::from_bits(0x3feff7ced916872b);
/// `#define TIME_AMOUNT_ATTACK 0.99` (Core/Src/Soundengine/DSPAudio/SlopeEg2.c:134) = 0.99
pub const TIME_AMOUNT_ATTACK: f64 = f64::from_bits(0x3fefae147ae147ae);
/// `#define TIME_K_ATTACK (2*TIME_AMOUNT_ATTACK/(1.0-TIME_AMOUNT_ATTACK))` (Core/Src/Soundengine/DSPAudio/SlopeEg2.c:135) = 197.99999999999983
pub const TIME_K_ATTACK: f64 = f64::from_bits(0x4068bffffffffffa);
/// `#define TIME_K_DECAY (2*TIME_AMOUNT_DECAY/(1.0-TIME_AMOUNT_DECAY))` (Core/Src/Soundengine/DSPAudio/SlopeEg2.c:136) = 1997.9999999999982
pub const TIME_K_DECAY: f64 = f64::from_bits(0x409f37fffffffff8);

// ---- types
/// C: `struct SLOPE_EG2_Struct` (Core/Inc/Soundengine/DSPAudio/SlopeEg2.h:28), 56 bytes.
/// C typedef name `SlopeEg2`; the struct tag `SLOPE_EG2_Struct` is a type alias below.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SlopeEg2 {
    /// `double attack;`
    pub attack: f64,
    /// `double decay;`
    pub decay: f64,
    /// `double slope;`
    pub slope: f64,
    /// `double invSlope;`
    pub invSlope: f64,
    /// `uint8_t repeat; / **<number of repetitions of the attack phase* /`
    pub repeat: u8,
    /// `uint8_t repeatCnt; / **<a counter for the already played repeats* /`
    pub repeatCnt: u8,
    /// `double value;`
    pub value: f64,
    /// `uint8_t state;`
    pub state: u8,
}
/// C struct tag of `SlopeEg2`.
pub type SLOPE_EG2_Struct = SlopeEg2;
impl SlopeEg2 {
    /// All-zero value (C static zero initialisation).
    pub const ZERO: Self = Self { attack: 0.0, decay: 0.0, slope: 0.0, invSlope: 0.0, repeat: 0, repeatCnt: 0, value: 0.0, state: 0 };
}
impl Default for SlopeEg2 {
    fn default() -> Self {
        Self::ZERO
    }
}

// ---- functions without access to globals
/// C: `void slopeEg2_init(SlopeEg2* eg)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/SlopeEg2.c:22`. Reference build: out-of-line in flash. Group P1.
/// Calls: `dsp::slope_eg2::slopeEg2_setSlope`.
pub fn slopeEg2_init(eg: &mut SlopeEg2) {
    todo!("port: Core/Src/Soundengine/DSPAudio/SlopeEg2.c:22")
}

/// C: `void slopeEg2_trigger(SlopeEg2* eg)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/SlopeEg2.c:33`. Reference build: out-of-line in flash. Group P1.
pub fn slopeEg2_trigger(eg: &mut SlopeEg2) {
    todo!("port: Core/Src/Soundengine/DSPAudio/SlopeEg2.c:33")
}

/// C: `IN_ITCM_RAM double slopeEg2_calcSlopeValue(double val, double slope)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/SlopeEg2.c:51`. Reference build: out-of-line in ITCM. Group P1.
pub fn slopeEg2_calcSlopeValue(val: f64, slope: f64) -> f64 {
    todo!("port: Core/Src/Soundengine/DSPAudio/SlopeEg2.c:51")
}

/// C: `IN_ITCM_RAM double slopeEg2_calc(SlopeEg2* eg)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/SlopeEg2.c:56`. Reference build: out-of-line in ITCM. Group P1.
/// Calls: `dsp::slope_eg2::slopeEg2_calcSlopeValue`.
pub fn slopeEg2_calc(eg: &mut SlopeEg2) -> f64 {
    todo!("port: Core/Src/Soundengine/DSPAudio/SlopeEg2.c:56")
}

/// C: `double slopeEg2_calcTime(uint8_t data2, double amount)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/SlopeEg2.c:137`. Reference build: inlined into its callers only. Group P1.
pub fn slopeEg2_calcTime(data2: u8, amount: f64) -> f64 {
    todo!("port: Core/Src/Soundengine/DSPAudio/SlopeEg2.c:137")
}

/// C: `IN_ITCM_RAM void slopeEg2_setAttack(SlopeEg2* eg, uint8_t data2, uint8_t isSync)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/SlopeEg2.c:150`. Reference build: out-of-line in ITCM. Group P1.
/// Calls: `dsp::slope_eg2::slopeEg2_calcTime`.
pub fn slopeEg2_setAttack(eg: &mut SlopeEg2, data2: u8, isSync: u8) {
    todo!("port: Core/Src/Soundengine/DSPAudio/SlopeEg2.c:150")
}

/// C: `IN_ITCM_RAM void slopeEg2_setDecay(SlopeEg2* eg, uint8_t data2, uint8_t isSync)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/SlopeEg2.c:157`. Reference build: out-of-line in ITCM. Group P1.
/// Calls: `dsp::slope_eg2::slopeEg2_calcTime`.
pub fn slopeEg2_setDecay(eg: &mut SlopeEg2, data2: u8, isSync: u8) {
    todo!("port: Core/Src/Soundengine/DSPAudio/SlopeEg2.c:157")
}

/// C: `double slopeEg2_calcDecay(uint8_t data2)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/SlopeEg2.c:164`. Reference build: out-of-line in flash. Group P1.
/// Calls: `dsp::slope_eg2::slopeEg2_calcTime`.
pub fn slopeEg2_calcDecay(data2: u8) -> f64 {
    todo!("port: Core/Src/Soundengine/DSPAudio/SlopeEg2.c:164")
}

/// C: `IN_ITCM_RAM void slopeEg2_setSlope(SlopeEg2* eg, uint8_t data2)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/SlopeEg2.c:169`. Reference build: out-of-line in ITCM. Group P1.
pub fn slopeEg2_setSlope(eg: &mut SlopeEg2, data2: u8) {
    todo!("port: Core/Src/Soundengine/DSPAudio/SlopeEg2.c:169")
}
