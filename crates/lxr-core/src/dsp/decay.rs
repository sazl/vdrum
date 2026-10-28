//! Port of `Core/Inc/Soundengine/DSPAudio/Decay.h`, `Core/Src/Soundengine/DSPAudio/Decay.c`.
//!
//! Porting group: P1. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;

// ---- #define constants
/// `#define TIME_K (2*0.99f/(1.f-0.99f))` (Core/Src/Soundengine/DSPAudio/Decay.c:32) = 198.0001983642578
pub const TIME_K: f32 = f32::from_bits(0x4346000d);

// ---- types
/// C: `struct Decay_EG_Struct` (Core/Inc/Soundengine/DSPAudio/Decay.h:21), 12 bytes.
/// C typedef name `DecayEg`; the struct tag `Decay_EG_Struct` is a type alias below.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DecayEg {
    /// `float decay;`
    pub decay: f32,
    /// `float slope;`
    pub slope: f32,
    /// `float value;`
    pub value: f32,
}
/// C struct tag of `DecayEg`.
pub type Decay_EG_Struct = DecayEg;
impl DecayEg {
    /// All-zero value (C static zero initialisation).
    pub const ZERO: Self = Self { decay: 0.0, slope: 0.0, value: 0.0 };
}
impl Default for DecayEg {
    fn default() -> Self {
        Self::ZERO
    }
}

// ---- functions without access to globals
/// C: `float DecayEg_calcSlopeValue(float val, float slope)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Decay.c:21`. Reference build: inlined into its callers only. Group P1.
pub fn DecayEg_calcSlopeValue(val: f32, slope: f32) -> f32 {
    todo!("port: Core/Src/Soundengine/DSPAudio/Decay.c:21")
}

/// C: `void DecayEg_init(DecayEg* eg)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Decay.c:26`. Reference build: out-of-line in flash. Group P1.
pub fn DecayEg_init(eg: &mut DecayEg) {
    todo!("port: Core/Src/Soundengine/DSPAudio/Decay.c:26")
}

/// C: `float DecayEg_calcTime(uint8_t data2)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Decay.c:33`. Reference build: inlined into its callers only. Group P1.
pub fn DecayEg_calcTime(data2: u8) -> f32 {
    todo!("port: Core/Src/Soundengine/DSPAudio/Decay.c:33")
}

/// C: `void DecayEg_trigger(DecayEg* eg)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Decay.c:47`. Reference build: out-of-line in flash. Group P1.
pub fn DecayEg_trigger(eg: &mut DecayEg) {
    todo!("port: Core/Src/Soundengine/DSPAudio/Decay.c:47")
}

/// C: `IN_ITCM_RAM float DecayEg_calc(DecayEg* eg)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Decay.c:52`. Reference build: out-of-line in ITCM. Group P1.
/// Calls: `dsp::decay::DecayEg_calcSlopeValue`.
pub fn DecayEg_calc(eg: &mut DecayEg) -> f32 {
    todo!("port: Core/Src/Soundengine/DSPAudio/Decay.c:52")
}

/// C: `void DecayEg_setDecay(DecayEg* eg, uint8_t data2)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Decay.c:69`. Reference build: out-of-line in flash. Group P1.
/// Calls: `dsp::decay::DecayEg_calcTime`.
pub fn DecayEg_setDecay(eg: &mut DecayEg, data2: u8) {
    todo!("port: Core/Src/Soundengine/DSPAudio/Decay.c:69")
}

/// C: `IN_ITCM_RAM void DecayEg_setSlope(DecayEg* eg, uint8_t data2)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Decay.c:74`. Reference build: out-of-line in ITCM. Group P1.
pub fn DecayEg_setSlope(eg: &mut DecayEg, data2: u8) {
    todo!("port: Core/Src/Soundengine/DSPAudio/Decay.c:74")
}
