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
/// C: `void VerbDelay_init(VerbDelay* d)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbDelay.c:17`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
pub fn VerbDelay_init(d: &mut VerbDelay) {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbDelay.c:17")
}

/// C: `inline uint16_t VerbDelay_getMaxSize(VerbDelay* d)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbDelay.c:25`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
pub fn VerbDelay_getMaxSize(d: &mut VerbDelay) -> u16 {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbDelay.c:25")
}

/// C: `inline uint16_t VerbDelay_getLength(VerbDelay* d)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbDelay.c:30`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
pub fn VerbDelay_getLength(d: &mut VerbDelay) -> u16 {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbDelay.c:30")
}

/// C: `void VerbDelay_setLength(VerbDelay* d, uint16_t l)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbDelay.c:35`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
/// Calls: `dsp::fx::verb_tools::verb_delay::VerbDelay_calcLength`.
pub fn VerbDelay_setLength(d: &mut VerbDelay, l: u16) {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbDelay.c:35")
}

/// C: `inline void VerbDelay_setRoomSize(VerbDelay* d, float sz)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbDelay.c:45`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
/// Calls: `dsp::fx::verb_tools::verb_delay::VerbDelay_calcLength`.
pub fn VerbDelay_setRoomSize(d: &mut VerbDelay, sz: f32) {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbDelay.c:45")
}

/// C: `inline float VerbDelay_read(VerbDelay* d)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbDelay.c:51`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
pub fn VerbDelay_read(d: &mut VerbDelay) -> f32 {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbDelay.c:51")
}

/// C: `inline void VerbDelay_write(VerbDelay* d, float in)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbDelay.c:59`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
pub fn VerbDelay_write(d: &mut VerbDelay, r#in: f32) {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbDelay.c:59")
}

/// C: `inline float VerbDelay_ReadTap(VerbDelay* d, float percent)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbDelay.c:64`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
pub fn VerbDelay_ReadTap(d: &mut VerbDelay, percent: f32) -> f32 {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbDelay.c:64")
}

/// C: `float VerbDelay_ReadTapSample(VerbDelay* d, uint16_t idx)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbDelay.c:74`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
pub fn VerbDelay_ReadTapSample(d: &mut VerbDelay, idx: u16) -> f32 {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbDelay.c:74")
}

/// C: `inline float VerbDelay_process(VerbDelay* d, const float input)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbDelay.c:84`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
/// Calls: `dsp::fx::verb_tools::verb_delay::VerbDelay_read`, `dsp::fx::verb_tools::verb_delay::VerbDelay_write`.
pub fn VerbDelay_process(d: &mut VerbDelay, input: f32) -> f32 {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbDelay.c:84")
}

/// C: `inline void VerbDelay_setBuffer(VerbDelay* d, volatile __fp16 *buffer)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbDelay.c:92`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
pub fn VerbDelay_setBuffer(d: &mut VerbDelay, buffer: &mut [u16]) {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbDelay.c:92")
}

/// C: `inline void VerbDelay_calcLength(VerbDelay* d)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbDelay.c:96`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
pub fn VerbDelay_calcLength(d: &mut VerbDelay) {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/VerbTools/VerbDelay.c:96")
}
