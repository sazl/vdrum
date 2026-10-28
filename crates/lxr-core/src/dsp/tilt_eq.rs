//! Port of `Core/Inc/Soundengine/DSPAudio/TiltEq.h`, `Core/Src/Soundengine/DSPAudio/TiltEq.c`.
//!
//! Porting group: P1. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;

// ---- #define constants
/// `#define SR3 (3*REAL_FS)` (Core/Inc/Soundengine/DSPAudio/TiltEq.h:22) = 132420.0
pub const SR3: f32 = f32::from_bits(0x48015100);

// ---- types
/// C: `struct TiltEqStruct` (Core/Inc/Soundengine/DSPAudio/TiltEq.h:25), 28 bytes.
/// C typedef name `TiltEq`; the struct tag `TiltEqStruct` is a type alias below.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TiltEq {
    /// `float amp;`
    pub amp: f32,
    /// `float gfactor; //gfactor is the proportional gain`
    pub gfactor: f32,
    /// `float a0,b1;`
    pub a0: f32,
    /// `float a0,b1;`
    pub b1: f32,
    /// `float lp_out;`
    pub lp_out: f32,
    /// `float lgain,hgain;`
    pub lgain: f32,
    /// `float lgain,hgain;`
    pub hgain: f32,
}
/// C struct tag of `TiltEq`.
pub type TiltEqStruct = TiltEq;
impl TiltEq {
    /// All-zero value (C static zero initialisation).
    pub const ZERO: Self = Self { amp: 0.0, gfactor: 0.0, a0: 0.0, b1: 0.0, lp_out: 0.0, lgain: 0.0, hgain: 0.0 };
}
impl Default for TiltEq {
    fn default() -> Self {
        Self::ZERO
    }
}

// ---- functions without access to globals
/// C: `fasterpow2 (float p)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/TiltEq.c:20`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P1.
pub fn fasterpow2(p: f32) -> f32 {
    todo!("port: Core/Src/Soundengine/DSPAudio/TiltEq.c:20")
}

/// C: `fasterexp (float p)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/TiltEq.c:28`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P1.
/// Calls: `dsp::tilt_eq::fasterpow2`.
pub fn fasterexp(p: f32) -> f32 {
    todo!("port: Core/Src/Soundengine/DSPAudio/TiltEq.c:28")
}

/// C: `void TiltEq_init(TiltEq* eq)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/TiltEq.c:35`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P1.
/// Calls: `dsp::tilt_eq::TiltEq_setFreq`.
pub fn TiltEq_init(eq: &mut TiltEq) {
    todo!("port: Core/Src/Soundengine/DSPAudio/TiltEq.c:35")
}

/// C: `float TiltEq_process(TiltEq* eq, float in)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/TiltEq.c:40`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P1.
pub fn TiltEq_process(eq: &mut TiltEq, r#in: f32) -> f32 {
    todo!("port: Core/Src/Soundengine/DSPAudio/TiltEq.c:40")
}

/// C: `void TiltEq_setFreq(TiltEq* eq, float fc)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/TiltEq.c:51`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P1.
pub fn TiltEq_setFreq(eq: &mut TiltEq, fc: f32) {
    todo!("port: Core/Src/Soundengine/DSPAudio/TiltEq.c:51")
}

/// C: `void TiltEq_setParameter(TiltEq* eq, float p)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/TiltEq.c:60`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P1.
/// Calls: `dsp::tilt_eq::TiltEq_setFreq`.
pub fn TiltEq_setParameter(eq: &mut TiltEq, p: f32) {
    todo!("port: Core/Src/Soundengine/DSPAudio/TiltEq.c:60")
}
