//! Port of `Core/Inc/Soundengine/DSPAudio/FX/VerbTools/VerbDelay.h`, `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbDelay.c`.
//!
//! Porting group: P2. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;

// ---- types
/// C: `struct VerbDelayStruct` (Core/Inc/Soundengine/DSPAudio/FX/VerbTools/VerbDelay.h:29), 16 bytes.
/// C typedef name `VerbDelay`; the struct tag `VerbDelayStruct` is a type alias below.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VerbDelay {
    /// `volatile __fp16 *ramBuffer;`
    ///
    /// C pointer: stored as the address in the reference firmware memory map (`CPtr`, 0 = NULL), see `memmap`.
    pub ramBuffer: CPtr,
    /// `uint16_t length;`
    pub length: u16,
    /// `uint16_t readPtr;`
    pub readPtr: u16,
    /// `uint16_t writePtr;`
    pub writePtr: u16,
    /// `float roomSz;`
    pub roomSz: f32,
}
/// C struct tag of `VerbDelay`.
pub type VerbDelayStruct = VerbDelay;
impl VerbDelay {
    /// All-zero value (C static zero initialisation).
    pub const ZERO: Self = Self { ramBuffer: 0, length: 0, readPtr: 0, writePtr: 0, roomSz: 0.0 };
}
impl Default for VerbDelay {
    fn default() -> Self {
        Self::ZERO
    }
}

// ---- functions without access to globals
//
// Not in the reference firmware (Reverb is disabled with `USE_REVERB 0`): ported from C, untestable.
// The C `volatile __fp16 *ramBuffer` is a firmware address inside `FX_BUFFER` (`CPtr`); the functions that
// access the buffer take `ram` = `Lxr::FX_BUFFER` (raw `__fp16` bits) and index it relative to
// `memmap::ADDR_FX_BUFFER`.

use crate::fpu::{f16_bits_to_f32, f32_to_f16_bits};
use crate::memmap::ADDR_FX_BUFFER;

const MASK: u16 = (MAX_ALLPASS_SIZE - 1) as u16;

/// Index into `FX_BUFFER` of element `i` of the buffer that starts at firmware address `base`.
#[inline(always)]
pub(crate) fn fx_index(base: CPtr, i: usize) -> usize {
    (base.wrapping_sub(ADDR_FX_BUFFER) / 2) as usize + i
}

/// C: `void VerbDelay_init(VerbDelay* d)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbDelay.c:17`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
pub fn VerbDelay_init(d: &mut VerbDelay) {
    d.readPtr = 0;
    d.length = 256;
    d.writePtr = d.length;
    d.roomSz = 1.0;
}

/// C: `inline uint16_t VerbDelay_getMaxSize(VerbDelay* d)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbDelay.c:25`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
pub fn VerbDelay_getMaxSize(d: &mut VerbDelay) -> u16 {
    MAX_ALLPASS_SIZE as u16
}

/// C: `inline uint16_t VerbDelay_getLength(VerbDelay* d)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbDelay.c:30`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
pub fn VerbDelay_getLength(d: &mut VerbDelay) -> u16 {
    d.length
}

/// C: `void VerbDelay_setLength(VerbDelay* d, uint16_t l)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbDelay.c:35`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
/// Calls: `dsp::fx::verb_tools::verb_delay::VerbDelay_calcLength`.
pub fn VerbDelay_setLength(d: &mut VerbDelay, l: u16) {
    d.length = l;
    VerbDelay_calcLength(d);
}

/// C: `inline void VerbDelay_setRoomSize(VerbDelay* d, float sz)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbDelay.c:45`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
/// Calls: `dsp::fx::verb_tools::verb_delay::VerbDelay_calcLength`.
pub fn VerbDelay_setRoomSize(d: &mut VerbDelay, sz: f32) {
    d.roomSz = sz;
    VerbDelay_calcLength(d);
}

/// C: `inline float VerbDelay_read(VerbDelay* d)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbDelay.c:51`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
pub fn VerbDelay_read(d: &mut VerbDelay, ram: &[u16]) -> f32 {
    let result = ram[fx_index(d.ramBuffer, d.readPtr as usize)];
    d.readPtr = (d.readPtr.wrapping_add(1)) & MASK;
    f16_bits_to_f32(result)
}

/// C: `inline void VerbDelay_write(VerbDelay* d, float in)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbDelay.c:59`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
pub fn VerbDelay_write(d: &mut VerbDelay, r#in: f32, ram: &mut [u16]) {
    ram[fx_index(d.ramBuffer, d.writePtr as usize)] = f32_to_f16_bits(r#in);
    d.writePtr = (d.writePtr.wrapping_add(1)) & MASK;
}

/// C: `inline float VerbDelay_ReadTap(VerbDelay* d, float percent)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbDelay.c:64`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
pub fn VerbDelay_ReadTap(d: &mut VerbDelay, percent: f32, ram: &[u16]) -> f32 {
    let offset = (percent / 100.0 * d.length as f32) as u32 as u16;
    let r1 = ((d.readPtr as i32 - offset as i32) as u16) & MASK;
    f16_bits_to_f32(ram[fx_index(d.ramBuffer, r1 as usize)])
}

/// C: `float VerbDelay_ReadTapSample(VerbDelay* d, uint16_t idx)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbDelay.c:74`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
pub fn VerbDelay_ReadTapSample(d: &mut VerbDelay, idx: u16, ram: &[u16]) -> f32 {
    let r1 = ((d.readPtr as i32 - idx as i32) as u16) & MASK;
    f16_bits_to_f32(ram[fx_index(d.ramBuffer, r1 as usize)])
}

/// C: `inline float VerbDelay_process(VerbDelay* d, const float input)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbDelay.c:84`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
/// Calls: `dsp::fx::verb_tools::verb_delay::VerbDelay_read`, `dsp::fx::verb_tools::verb_delay::VerbDelay_write`.
pub fn VerbDelay_process(d: &mut VerbDelay, input: f32, ram: &mut [u16]) -> f32 {
    VerbDelay_write(d, input, ram);
    VerbDelay_read(d, ram)
}

/// C: `inline void VerbDelay_setBuffer(VerbDelay* d, volatile __fp16 *buffer)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbDelay.c:92`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
/// `buffer` is the firmware address of the first element (inside `FX_BUFFER`).
pub fn VerbDelay_setBuffer(d: &mut VerbDelay, buffer: CPtr) {
    d.ramBuffer = buffer;
}

/// C: `inline void VerbDelay_calcLength(VerbDelay* d)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbDelay.c:96`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
pub fn VerbDelay_calcLength(d: &mut VerbDelay) {
    let scaledSize = d.length as f32 * d.roomSz;
    let mut pos = d.writePtr as f32 - scaledSize;
    if pos < 0.0 {
        pos += MAX_ALLPASS_SIZE as f32;
    }
    let itg = pos as u32 as u16;
    d.readPtr = itg & MASK;
}
