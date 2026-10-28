//! Port of `Core/Inc/Soundengine/DSPAudio/FX/VerbTools/AllpassDelay.h`, `Core/Src/Soundengine/DSPAudio/FX/VerbTools/AllpassDelay.c`.
//!
//! Porting group: P2. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;

// ---- #define constants
/// `#define INITIAL_ALLPASS_COFF 0.6` (Core/Inc/Soundengine/DSPAudio/FX/VerbTools/AllpassDelay.h:26) = 0.6
pub const INITIAL_ALLPASS_COFF: f64 = f64::from_bits(0x3fe3333333333333);

// ---- types
/// C: `struct AllpassDelayStruct` (Core/Inc/Soundengine/DSPAudio/FX/VerbTools/AllpassDelay.h:29), 28 bytes.
/// C typedef name `AllpassDelay`; the struct tag `AllpassDelayStruct` is a type alias below.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AllpassDelay {
    /// `float g;`
    pub g: f32,
    /// `float out,in;`
    pub out: f32,
    /// `float out,in;`
    pub r#in: f32,
    /// `VerbDelay delay;`
    pub delay: VerbDelay,
}
/// C struct tag of `AllpassDelay`.
pub type AllpassDelayStruct = AllpassDelay;
impl AllpassDelay {
    /// All-zero value (C static zero initialisation).
    pub const ZERO: Self = Self { g: 0.0, out: 0.0, r#in: 0.0, delay: VerbDelay::ZERO };
}
impl Default for AllpassDelay {
    fn default() -> Self {
        Self::ZERO
    }
}

/// C: `struct AllpassModDelayStruct` (Core/Inc/Soundengine/DSPAudio/FX/VerbTools/AllpassDelay.h:44), 64 bytes.
/// C typedef name `AllpassModDelay`; the struct tag `AllpassModDelayStruct` is a type alias below.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AllpassModDelay {
    /// `float g;`
    pub g: f32,
    /// `float out,in;`
    pub out: f32,
    /// `float out,in;`
    pub r#in: f32,
    /// `VerbModDelay delay;`
    pub delay: VerbModDelay,
}
/// C struct tag of `AllpassModDelay`.
pub type AllpassModDelayStruct = AllpassModDelay;
impl AllpassModDelay {
    /// All-zero value (C static zero initialisation).
    pub const ZERO: Self = Self { g: 0.0, out: 0.0, r#in: 0.0, delay: VerbModDelay::ZERO };
}
impl Default for AllpassModDelay {
    fn default() -> Self {
        Self::ZERO
    }
}

// ---- functions without access to globals
/// C: `void AllpassDelay_init(AllpassDelay *d)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/AllpassDelay.c:20`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
/// Calls: `dsp::fx::verb_tools::verb_delay::VerbDelay_init`.
pub fn AllpassDelay_init(d: &mut AllpassDelay) {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/VerbTools/AllpassDelay.c:20")
}

/// C: `float AllpassDelay_process(AllpassDelay *d, float input)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/AllpassDelay.c:26`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
/// Calls: `dsp::fx::verb_tools::verb_delay::VerbDelay_read`, `dsp::fx::verb_tools::verb_delay::VerbDelay_write`.
pub fn AllpassDelay_process(d: &mut AllpassDelay, input: f32) -> f32 {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/VerbTools/AllpassDelay.c:26")
}

/// C: `void AllpassDelay_setCoefficient(AllpassDelay *d, float coff)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/AllpassDelay.c:33`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
pub fn AllpassDelay_setCoefficient(d: &mut AllpassDelay, coff: f32) {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/VerbTools/AllpassDelay.c:33")
}

/// C: `void AllpassDelay_setDelay(AllpassDelay *d, float del)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/AllpassDelay.c:37`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
/// Calls: `dsp::fx::verb_tools::verb_delay::VerbDelay_setLength`.
pub fn AllpassDelay_setDelay(d: &mut AllpassDelay, del: f32) {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/VerbTools/AllpassDelay.c:37")
}

/// C: `void AllpassDelay_setBuffer(AllpassDelay *d, volatile __fp16 *buffer)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/AllpassDelay.c:41`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
/// Calls: `dsp::fx::verb_tools::verb_delay::VerbDelay_setBuffer`.
pub fn AllpassDelay_setBuffer(d: &mut AllpassDelay, buffer: &mut [u16]) {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/VerbTools/AllpassDelay.c:41")
}

/// C: `void AllpassDelay_setRoomSize(AllpassDelay *d, float sz)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/AllpassDelay.c:45`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
/// Calls: `dsp::fx::verb_tools::verb_delay::VerbDelay_setRoomSize`.
pub fn AllpassDelay_setRoomSize(d: &mut AllpassDelay, sz: f32) {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/VerbTools/AllpassDelay.c:45")
}

/// C: `void AllpassModDelay_init(AllpassModDelay *d)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/AllpassDelay.c:54`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
/// Calls: `dsp::fx::verb_tools::verb_mod_delay::VerbModDelay_init`.
pub fn AllpassModDelay_init(d: &mut AllpassModDelay) {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/VerbTools/AllpassDelay.c:54")
}

/// C: `float AllpassModDelay_process(AllpassModDelay *d, float input)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/AllpassDelay.c:60`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
/// Calls: `dsp::fx::verb_tools::verb_mod_delay::VerbModDelay_calcLength`, `dsp::fx::verb_tools::verb_mod_delay::VerbModDelay_read`, `dsp::fx::verb_tools::verb_mod_delay::VerbModDelay_write`.
pub fn AllpassModDelay_process(d: &mut AllpassModDelay, input: f32) -> f32 {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/VerbTools/AllpassDelay.c:60")
}

/// C: `void AllpassModDelay_setCoefficient(AllpassModDelay *d, float coff)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/AllpassDelay.c:68`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
pub fn AllpassModDelay_setCoefficient(d: &mut AllpassModDelay, coff: f32) {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/VerbTools/AllpassDelay.c:68")
}

/// C: `void AllpassModDelay_setDelay(AllpassModDelay *d, float del)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/AllpassDelay.c:72`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
/// Calls: `dsp::fx::verb_tools::verb_mod_delay::VerbModDelay_setLength`.
pub fn AllpassModDelay_setDelay(d: &mut AllpassModDelay, del: f32) {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/VerbTools/AllpassDelay.c:72")
}

/// C: `void AllpassModDelay_setBuffer(AllpassModDelay *d, volatile __fp16 *buffer)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/AllpassDelay.c:76`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
/// Calls: `dsp::fx::verb_tools::verb_mod_delay::VerbModDelay_setBuffer`.
pub fn AllpassModDelay_setBuffer(d: &mut AllpassModDelay, buffer: &mut [u16]) {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/VerbTools/AllpassDelay.c:76")
}

/// C: `void AllpassModDelay_setRoomSize(AllpassModDelay *d, float sz)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/AllpassDelay.c:80`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
/// Calls: `dsp::fx::verb_tools::verb_mod_delay::VerbModDelay_setRoomSize`.
pub fn AllpassModDelay_setRoomSize(d: &mut AllpassModDelay, sz: f32) {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/VerbTools/AllpassDelay.c:80")
}

/// C: `void AllpassModDelay_setModDepth(AllpassModDelay *d,float depth)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/AllpassDelay.c:84`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
/// Calls: `dsp::fx::verb_tools::verb_mod_delay::VerbModDelay_setModDepth`.
pub fn AllpassModDelay_setModDepth(d: &mut AllpassModDelay, depth: f32) {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/VerbTools/AllpassDelay.c:84")
}

/// C: `void AllpassModDelay_setModValue(AllpassModDelay *d,float v)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/AllpassDelay.c:89`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
/// Calls: `dsp::fx::verb_tools::verb_mod_delay::VerbModDelay_calcLength`, `dsp::fx::verb_tools::verb_mod_delay::VerbModDelay_setModValue`.
pub fn AllpassModDelay_setModValue(d: &mut AllpassModDelay, v: f32) {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/VerbTools/AllpassDelay.c:89")
}
