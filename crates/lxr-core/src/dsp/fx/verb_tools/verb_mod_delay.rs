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
//
// Not in the reference firmware (Reverb is disabled with `USE_REVERB 0`): ported from C, untestable.
// Buffer access as in `verb_delay` (`ram` = `Lxr::FX_BUFFER`, `ramBuffer` = firmware address).

use super::verb_delay::fx_index;
use crate::dsp::one_pole_lp::{calcOnePole, initOnePole, setOnePoleFreq};
use crate::fpu::{f16_bits_to_f32, f32_to_f16_bits};

const MASK: u32 = (MAX_DELAY_SIZE - 1) as u32;

/// C: `void VerbModDelay_init(VerbModDelay *d)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.c:19`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
/// Calls: `dsp::one_pole_lp::initOnePole`, `dsp::one_pole_lp::setOnePoleFreq`.
pub fn VerbModDelay_init(d: &mut VerbModDelay) {
    d.readPtr = 0;
    d.length = 256;
    d.writePtr = d.length;
    initOnePole(&mut d.lp);
    setOnePoleFreq(&mut d.lp, 0.0001308911269084528);
    d.roomSz = 1.0;
    d.frac = 0.0;
    d.depth = 0.0;
    d.modVal = 0.0;
}

/// C: `inline uint32_t VerbModDelay_getMaxSize(VerbModDelay *d)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.c:33`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
pub fn VerbModDelay_getMaxSize(d: &mut VerbModDelay) -> u32 {
    MAX_DELAY_SIZE as u32
}

/// C: `inline uint32_t VerbModDelay_getLength(VerbModDelay *d)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.c:38`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
pub fn VerbModDelay_getLength(d: &mut VerbModDelay) -> u32 {
    d.length
}

/// C: `void VerbModDelay_setLength(VerbModDelay *d, uint32_t l)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.c:43`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
pub fn VerbModDelay_setLength(d: &mut VerbModDelay, l: u32) {
    d.length = l;
}

/// C: `inline void VerbModDelay_setRoomSize(VerbModDelay *d, float sz)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.c:48`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
/// Calls: `dsp::fx::verb_tools::verb_mod_delay::VerbModDelay_calcLength`.
pub fn VerbModDelay_setRoomSize(d: &mut VerbModDelay, sz: f32) {
    d.roomSz = sz;
    VerbModDelay_calcLength(d);
}

/// C: `inline void VerbModDelay_setModDepth(VerbModDelay *d, float dep)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.c:54`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
pub fn VerbModDelay_setModDepth(d: &mut VerbModDelay, dep: f32) {
    d.depth = dep;
}

/// C: `inline void VerbModDelay_setModValue(VerbModDelay *d, float v)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.c:60`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
/// Calls: `dsp::fx::verb_tools::verb_mod_delay::VerbModDelay_calcLength`.
pub fn VerbModDelay_setModValue(d: &mut VerbModDelay, v: f32) {
    d.modVal = v;
    VerbModDelay_calcLength(d);
}

/// C: `inline float VerbModDelay_read(VerbModDelay *d)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.c:66`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
pub fn VerbModDelay_read(d: &mut VerbModDelay, ram: &[u16]) -> f32 {
    let r = d.readPtr;
    let result = ram[fx_index(d.ramBuffer, r as usize)];
    d.readPtr = r.wrapping_add(1) & MASK;
    f16_bits_to_f32(result)
}

/// C: `inline float VerbModDelay_readNext(VerbModDelay *d)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.c:74`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
pub fn VerbModDelay_readNext(d: &mut VerbModDelay, ram: &[u16]) -> f32 {
    let r = d.readPtr.wrapping_add(1) & MASK;
    f16_bits_to_f32(ram[fx_index(d.ramBuffer, r as usize)])
}

/// C: `inline void VerbModDelay_write(VerbModDelay *d,float in)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.c:83`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
pub fn VerbModDelay_write(d: &mut VerbModDelay, r#in: f32, ram: &mut [u16]) {
    let w = d.writePtr;
    ram[fx_index(d.ramBuffer, w as usize)] = f32_to_f16_bits(r#in);
    d.writePtr = w.wrapping_add(1) & MASK;
}

/// C: `float fnms(float a, float b, float c)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.c:90`. Reference build: inlined into its callers only. Group P2.
pub fn fnms(a: f32, b: f32, c: f32) -> f32 {
    c - a * b
}

/// C: `float lerp(float a, float b, float t)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.c:95`. Reference build: inlined into its callers only. Group P2.
/// Calls: `dsp::fx::verb_tools::verb_mod_delay::fnms`.
/// (C `fma` is the double precision function: the float arguments are promoted.)
pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
    crate::fpu::fma64(t as f64, b as f64, fnms(t, a, a) as f64) as f32
}

/// C: `float interpolated_read( float a, float b, float x)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.c:103`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
pub fn interpolated_read(a: f32, b: f32, x: f32) -> f32 {
    a * (1.0 - x) + b * x
}

/// C: `inline void VerbModDelay_calcLength(VerbModDelay *d)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.c:107`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
/// Calls: `dsp::one_pole_lp::calcOnePole`.
pub fn VerbModDelay_calcLength(d: &mut VerbModDelay) {
    let scaledSize = d.length as f32 * d.roomSz;
    let filteredPos = calcOnePole(&mut d.lp, (scaledSize + d.depth * d.modVal) as f64);
    let mut pos = d.writePtr as f32 - filteredPos;
    if pos < 0.0 {
        pos += MAX_DELAY_SIZE as f32;
    }
    let itg = pos as u32;
    d.frac = pos - itg as f32;
    d.readPtr = itg & MASK;
}

/// C: `inline float VerbModDelay_interpolatedRead(VerbModDelay *d)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.c:119`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
/// Calls: `dsp::fx::verb_tools::verb_mod_delay::VerbModDelay_read`, `dsp::fx::verb_tools::verb_mod_delay::VerbModDelay_readNext`, `dsp::fx::verb_tools::verb_mod_delay::lerp`.
pub fn VerbModDelay_interpolatedRead(d: &mut VerbModDelay, ram: &[u16]) -> f32 {
    let nextSample = VerbModDelay_readNext(d, ram);
    let a = VerbModDelay_read(d, ram);
    lerp(a, nextSample, d.frac)
}

/// C: `inline float VerbModDelay_interpolatedReadTap(VerbModDelay *d, float percent)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.c:127`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
/// Calls: `dsp::fx::verb_tools::verb_mod_delay::lerp`.
pub fn VerbModDelay_interpolatedReadTap(d: &mut VerbModDelay, percent: f32, ram: &[u16]) -> f32 {
    let offset = (percent / 100.0 * d.length as f32) as u32;
    let r1 = d.readPtr.wrapping_sub(offset) & MASK;
    let r2 = r1.wrapping_sub(offset).wrapping_add(1) & MASK;
    let a = f16_bits_to_f32(ram[fx_index(d.ramBuffer, r1 as usize)]);
    let b = f16_bits_to_f32(ram[fx_index(d.ramBuffer, r2 as usize)]);
    lerp(a, b, d.frac)
}

/// C: `inline float VerbModDelay_process(VerbModDelay *d, const float input)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.c:140`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
/// Calls: `dsp::fx::verb_tools::verb_mod_delay::VerbModDelay_read`, `dsp::fx::verb_tools::verb_mod_delay::VerbModDelay_readNext`, `dsp::fx::verb_tools::verb_mod_delay::VerbModDelay_write`, `dsp::fx::verb_tools::verb_mod_delay::lerp`.
pub fn VerbModDelay_process(d: &mut VerbModDelay, input: f32, ram: &mut [u16]) -> f32 {
    VerbModDelay_write(d, input, ram);
    let nextSample = VerbModDelay_readNext(d, ram);
    let a = VerbModDelay_read(d, ram);
    lerp(a, nextSample, d.frac)
}

/// C: `inline void VerbModDelay_setBuffer(VerbModDelay *d,volatile __fp16 *buffer)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbModDelay.c:156`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
pub fn VerbModDelay_setBuffer(d: &mut VerbModDelay, buffer: CPtr) {
    d.ramBuffer = buffer;
}
