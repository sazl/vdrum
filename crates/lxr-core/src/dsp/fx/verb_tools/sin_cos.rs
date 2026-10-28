//! Port of `Core/Inc/Soundengine/DSPAudio/FX/VerbTools/sinCos.h`, `Core/Src/Soundengine/DSPAudio/FX/VerbTools/sinCos.c`.
//!
//! Porting group: P2. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;

// ---- types
/// C: `struct SinCosStruct` (Core/Inc/Soundengine/DSPAudio/FX/VerbTools/sinCos.h:24), 12 bytes.
/// C typedef name `SineCosine`; the struct tag `SinCosStruct` is a type alias below.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SineCosine {
    /// `float a;`
    pub a: f32,
    /// `float s[2];`
    pub s: [f32; 2],
}
/// C struct tag of `SineCosine`.
pub type SinCosStruct = SineCosine;
impl SineCosine {
    /// All-zero value (C static zero initialisation).
    pub const ZERO: Self = Self { a: 0.0, s: [0.0; 2] };
}
impl Default for SineCosine {
    fn default() -> Self {
        Self::ZERO
    }
}

// ---- functions without access to globals
/// C: `void SineCosine_init(SineCosine *sc)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/sinCos.c:21`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
/// Calls: `dsp::fx::verb_tools::sin_cos::SineCosine_setFreq`.
pub fn SineCosine_init(sc: &mut SineCosine) {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/VerbTools/sinCos.c:21")
}

/// C: `void SineCosine_calc(SineCosine *sc)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/sinCos.c:28`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
pub fn SineCosine_calc(sc: &mut SineCosine) {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/VerbTools/sinCos.c:28")
}

/// C: `float SineCosine_getSine(SineCosine *sc)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/sinCos.c:33`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
pub fn SineCosine_getSine(sc: &mut SineCosine) -> f32 {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/VerbTools/sinCos.c:33")
}

/// C: `float SineCosine_getCosine(SineCosine *sc)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/sinCos.c:37`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
pub fn SineCosine_getCosine(sc: &mut SineCosine) -> f32 {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/VerbTools/sinCos.c:37")
}

/// C: `void SineCosine_setFreq(SineCosine *sc, float f)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/VerbTools/sinCos.c:42`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
pub fn SineCosine_setFreq(sc: &mut SineCosine, f: f32) {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/VerbTools/sinCos.c:42")
}
