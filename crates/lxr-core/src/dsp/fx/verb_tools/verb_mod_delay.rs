//! Port of `Core/Inc/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.h`, `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.c`.
//!
//! Porting group: P2. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;

// ---- types
/// C: `struct VerbModDelayStruct` (Core/Inc/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.h:26), 48 bytes.
/// C typedef name `VerbModDelay`; the struct tag `VerbModDelayStruct` is a type alias below.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VerbModDelay {
    /// `volatile __fp16 *ramBuffer;`
    ///
    /// C pointer: stored as the address in the reference firmware memory map (`CPtr`, 0 = NULL), see `memmap`.
    pub ramBuffer: CPtr,
    /// `uint32_t length;`
    pub length: u32,
    /// `uint32_t readPtr;`
    pub readPtr: u32,
    /// `uint32_t writePtr;`
    pub writePtr: u32,
    /// `OnePoleFilter lp;`
    pub lp: OnePoleFilter,
    /// `float frac;`
    pub frac: f32,
    /// `float depth;`
    pub depth: f32,
    /// `float modVal;`
    pub modVal: f32,
    /// `float roomSz;`
    pub roomSz: f32,
}
/// C struct tag of `VerbModDelay`.
pub type VerbModDelayStruct = VerbModDelay;
impl VerbModDelay {
    /// All-zero value (C static zero initialisation).
    pub const ZERO: Self = Self { ramBuffer: 0, length: 0, readPtr: 0, writePtr: 0, lp: OnePoleFilter::ZERO, frac: 0.0, depth: 0.0, modVal: 0.0, roomSz: 0.0 };
}
impl Default for VerbModDelay {
    fn default() -> Self {
        Self::ZERO
    }
}

// ---- functions without access to globals
/// C: `void VerbModDelay_init(VerbModDelay *d)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.c:19`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
/// Calls: `dsp::one_pole_lp::initOnePole`, `dsp::one_pole_lp::setOnePoleFreq`.
pub fn VerbModDelay_init(d: &mut VerbModDelay) {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.c:19")
}

/// C: `inline uint32_t VerbModDelay_getMaxSize(VerbModDelay *d)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.c:33`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
pub fn VerbModDelay_getMaxSize(d: &mut VerbModDelay) -> u32 {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.c:33")
}

/// C: `inline uint32_t VerbModDelay_getLength(VerbModDelay *d)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.c:38`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
pub fn VerbModDelay_getLength(d: &mut VerbModDelay) -> u32 {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.c:38")
}

/// C: `void VerbModDelay_setLength(VerbModDelay *d, uint32_t l)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.c:43`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
pub fn VerbModDelay_setLength(d: &mut VerbModDelay, l: u32) {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.c:43")
}

/// C: `inline void VerbModDelay_setRoomSize(VerbModDelay *d, float sz)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.c:48`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
/// Calls: `dsp::fx::verb_tools::verb_mod_delay::VerbModDelay_calcLength`.
pub fn VerbModDelay_setRoomSize(d: &mut VerbModDelay, sz: f32) {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.c:48")
}

/// C: `inline void VerbModDelay_setModDepth(VerbModDelay *d, float dep)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.c:54`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
pub fn VerbModDelay_setModDepth(d: &mut VerbModDelay, dep: f32) {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.c:54")
}

/// C: `inline void VerbModDelay_setModValue(VerbModDelay *d, float v)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.c:60`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
/// Calls: `dsp::fx::verb_tools::verb_mod_delay::VerbModDelay_calcLength`.
pub fn VerbModDelay_setModValue(d: &mut VerbModDelay, v: f32) {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.c:60")
}

/// C: `inline float VerbModDelay_read(VerbModDelay *d)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.c:66`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
pub fn VerbModDelay_read(d: &mut VerbModDelay) -> f32 {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.c:66")
}

/// C: `inline float VerbModDelay_readNext(VerbModDelay *d)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.c:74`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
pub fn VerbModDelay_readNext(d: &mut VerbModDelay) -> f32 {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.c:74")
}

/// C: `inline void VerbModDelay_write(VerbModDelay *d,float in)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.c:83`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
pub fn VerbModDelay_write(d: &mut VerbModDelay, r#in: f32) {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.c:83")
}

/// C: `float fnms(float a, float b, float c)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.c:90`. Reference build: inlined into its callers only. Group P2.
pub fn fnms(a: f32, b: f32, c: f32) -> f32 {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.c:90")
}

/// C: `float lerp(float a, float b, float t)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.c:95`. Reference build: inlined into its callers only. Group P2.
/// Calls: `dsp::fx::verb_tools::verb_mod_delay::fnms`.
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.c:95")
}

/// C: `float interpolated_read( float a, float b, float x)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.c:103`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
pub fn interpolated_read(a: f32, b: f32, x: f32) -> f32 {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.c:103")
}

/// C: `inline void VerbModDelay_calcLength(VerbModDelay *d)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.c:107`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
/// Calls: `dsp::one_pole_lp::calcOnePole`.
pub fn VerbModDelay_calcLength(d: &mut VerbModDelay) {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.c:107")
}

/// C: `inline float VerbModDelay_interpolatedRead(VerbModDelay *d)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.c:119`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
/// Calls: `dsp::fx::verb_tools::verb_mod_delay::VerbModDelay_read`, `dsp::fx::verb_tools::verb_mod_delay::VerbModDelay_readNext`, `dsp::fx::verb_tools::verb_mod_delay::lerp`.
pub fn VerbModDelay_interpolatedRead(d: &mut VerbModDelay) -> f32 {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.c:119")
}

/// C: `inline float VerbModDelay_interpolatedReadTap(VerbModDelay *d, float percent)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.c:127`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
/// Calls: `dsp::fx::verb_tools::verb_mod_delay::lerp`.
pub fn VerbModDelay_interpolatedReadTap(d: &mut VerbModDelay, percent: f32) -> f32 {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.c:127")
}

/// C: `inline float VerbModDelay_process(VerbModDelay *d, const float input)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.c:140`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
/// Calls: `dsp::fx::verb_tools::verb_mod_delay::VerbModDelay_read`, `dsp::fx::verb_tools::verb_mod_delay::VerbModDelay_readNext`, `dsp::fx::verb_tools::verb_mod_delay::VerbModDelay_write`, `dsp::fx::verb_tools::verb_mod_delay::lerp`.
pub fn VerbModDelay_process(d: &mut VerbModDelay, input: f32) -> f32 {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.c:140")
}

/// C: `inline void VerbModDelay_setBuffer(VerbModDelay *d,volatile __fp16 *buffer)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.c:156`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
pub fn VerbModDelay_setBuffer(d: &mut VerbModDelay, buffer: &mut [u16]) {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.c:156")
}
