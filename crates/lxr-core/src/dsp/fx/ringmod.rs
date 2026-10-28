//! Port of `Core/Inc/Soundengine/DSPAudio/FX/Ringmod.h`, `Core/Src/Soundengine/DSPAudio/FX/Ringmod.c`.
//!
//! Porting group: P2. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;

// ---- types
/// C: `struct RingmodStruct` (Core/Inc/Soundengine/DSPAudio/FX/Ringmod.h:23), 44 bytes.
/// C typedef name `Ringmod`; the struct tag `RingmodStruct` is a type alias below.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ringmod {
    /// `float freq; //drive`
    pub freq: f32,
    /// `OscInfo osc;`
    pub osc: OscInfo,
}
/// C struct tag of `Ringmod`.
pub type RingmodStruct = Ringmod;
impl Ringmod {
    /// All-zero value (C static zero initialisation).
    pub const ZERO: Self = Self { freq: 0.0, osc: OscInfo::ZERO };
}
impl Default for Ringmod {
    fn default() -> Self {
        Self::ZERO
    }
}

// ---- functions without access to globals
/// C: `void Ringmod_init(Ringmod *rm)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/Ringmod.c:18`. Reference build: out-of-line in flash. Group P2.
/// Calls: `dsp::oscillator::osc_setFreq`.
pub fn Ringmod_init(rm: &mut Ringmod) {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/Ringmod.c:18")
}

/// C: `void Ringmod_setFreq(Ringmod *rm, float f)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/Ringmod.c:79`. Reference build: out-of-line in flash. Group P2.
/// Calls: `dsp::oscillator::osc_setFreq`.
pub fn Ringmod_setFreq(rm: &mut Ringmod, f: f32) {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/Ringmod.c:79")
}

/// C: `void Ringmod_setWave(Ringmod *rm, uint8_t w)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/Ringmod.c:84`. Reference build: out-of-line in flash. Group P2.
pub fn Ringmod_setWave(rm: &mut Ringmod, w: u8) {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/Ringmod.c:84")
}

/// C: `void Ringmod_resetPhase(Ringmod* rm)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/Ringmod.c:89`. Reference build: not linked (removed by --gc-sections). Group P2.
pub fn Ringmod_resetPhase(rm: &mut Ringmod) {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/Ringmod.c:89")
}

// ---- functions that use globals (methods on `Lxr`)
impl Lxr {
    /// C: `void Ringmod_process(Ringmod *rm, int16_t *fxBuffer, int16_t* outL, int16_t* outR, uint16_t size, uint8_t channels, float dryWet)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/FX/Ringmod.c:31`. Reference build: out-of-line in flash. Group P2.
    /// Calls: `cmsis::__QADD16`, `Lxr::calcNextOscSampleBlock`.
    pub fn Ringmod_process(&mut self, rm: &mut Ringmod, fxBuffer: &mut [i16], outL: &mut [i16], outR: &mut [i16], size: u16, channels: u8, dryWet: f32) {
        todo!("port: Core/Src/Soundengine/DSPAudio/FX/Ringmod.c:31")
    }
}
