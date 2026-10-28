//! Port of `Core/Inc/Soundengine/DSPAudio/FX/Compressor.h`, `Core/Src/Soundengine/DSPAudio/FX/Compressor.c`.
//!
//! Porting group: P2. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;

// ---- enums
// C: `enum {...}` (Core/Src/Soundengine/DSPAudio/FX/Compressor.c:21), 1 bytes, underlying type `u8`. (anonymous: enumerators only)
pub const RATIO_1: u8 = 0;
pub const RATIO_2: u8 = 1;
pub const RATIO_3: u8 = 2;
pub const RATIO_4: u8 = 3;
pub const RATIO_6: u8 = 4;
pub const RATIO_8: u8 = 5;

// ---- types
/// C: `struct CompressorStruct` (Core/Inc/Soundengine/DSPAudio/FX/Compressor.h:21), 32 bytes.
/// C typedef name `Compressor`; the struct tag `CompressorStruct` is a type alias below.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Compressor {
    /// `float tresh;`
    pub tresh: f32,
    /// `float attack;`
    pub attack: f32,
    /// `float release;`
    pub release: f32,
    /// `float ratio;`
    pub ratio: f32,
    /// `float markupGain;`
    pub markupGain: f32,
    /// `double env;`
    pub env: f64,
}
/// C struct tag of `Compressor`.
pub type CompressorStruct = Compressor;
impl Compressor {
    /// All-zero value (C static zero initialisation).
    pub const ZERO: Self = Self { tresh: 0.0, attack: 0.0, release: 0.0, ratio: 0.0, markupGain: 0.0, env: 0.0 };
}
impl Default for Compressor {
    fn default() -> Self {
        Self::ZERO
    }
}

// ---- functions without access to globals
/// C: `void Compressor_init(Compressor *c)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/Compressor.c:32`. Reference build: out-of-line in flash. Group P2.
pub fn Compressor_init(c: &mut Compressor) {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/Compressor.c:32")
}

/// C: `static double lin2dB( double lin )`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/Compressor.c:45`. Reference build: inlined into its callers only. Group P2.
pub fn lin2dB(lin: f64) -> f64 {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/Compressor.c:45")
}

/// C: `static double dB2lin( double dB )`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/Compressor.c:51`. Reference build: inlined into its callers only. Group P2.
pub fn dB2lin(dB: f64) -> f64 {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/Compressor.c:51")
}

/// C: `void Compressor_setAttack(Compressor *c, float a)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/Compressor.c:57`. Reference build: out-of-line in flash. Group P2.
pub fn Compressor_setAttack(c: &mut Compressor, a: f32) {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/Compressor.c:57")
}

/// C: `void Compressor_setTreshold(Compressor *c, float t)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/Compressor.c:62`. Reference build: out-of-line in flash. Group P2.
pub fn Compressor_setTreshold(c: &mut Compressor, t: f32) {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/Compressor.c:62")
}

/// C: `void Compressor_setGain(Compressor *c, float g)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/Compressor.c:67`. Reference build: out-of-line in flash. Group P2.
pub fn Compressor_setGain(c: &mut Compressor, g: f32) {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/Compressor.c:67")
}

/// C: `void Compressor_setRelease(Compressor *c, float r)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/Compressor.c:71`. Reference build: out-of-line in flash. Group P2.
pub fn Compressor_setRelease(c: &mut Compressor, r: f32) {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/Compressor.c:71")
}

/// C: `void Compressor_setRatio(Compressor *c, uint8_t r)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/Compressor.c:77`. Reference build: out-of-line in flash. Group P2.
pub fn Compressor_setRatio(c: &mut Compressor, r: u8) {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/Compressor.c:77")
}

/// C: `void Compressor_process(Compressor *c, int16_t *fxBuffer,int16_t* outL, int16_t* outR, uint16_t size, uint8_t channels, float dryWet)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/Compressor.c:104`. Reference build: out-of-line in flash. Group P2.
/// Calls: `cmsis::__QADD16`, `dsp::fx::compressor::dB2lin`, `dsp::fx::compressor::lin2dB`.
pub fn Compressor_process(c: &mut Compressor, fxBuffer: &mut [i16], outL: &mut [i16], outR: &mut [i16], size: u16, channels: u8, dryWet: f32) {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/Compressor.c:104")
}
