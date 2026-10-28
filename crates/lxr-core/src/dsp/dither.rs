//! Port of `Core/Inc/Soundengine/DSPAudio/dither.h`, `Core/Src/Soundengine/DSPAudio/dither.c`.
//!
//! Porting group: P1. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;

// ---- #define constants
/// `#define BITS 16` (Core/Inc/Soundengine/DSPAudio/dither.h:41)
pub const BITS: i32 = 16;
/// `#define RAND_MAX 32767` (Core/Inc/Soundengine/DSPAudio/dither.h:43)
pub const RAND_MAX: i32 = 32767;
/// `#define DITHER_S (0.5f)` (Core/Inc/Soundengine/DSPAudio/dither.h:45) = 0.5
pub const DITHER_S: f32 = f32::from_bits(0x3f000000);
/// `#define DITHER_W 32768.f` (Core/Inc/Soundengine/DSPAudio/dither.h:46) = 32768.0
pub const DITHER_W: f32 = f32::from_bits(0x47000000);
/// `#define DITHER_WI (1.0f/DITHER_W)` (Core/Inc/Soundengine/DSPAudio/dither.h:47) = 3.0517578125e-05
pub const DITHER_WI: f32 = f32::from_bits(0x38000000);
/// `#define DITHER_D (DITHER_WI / RAND_MAX)` (Core/Inc/Soundengine/DSPAudio/dither.h:48) = 1.4210854715202004e-14
pub const DITHER_D: f32 = f32::from_bits(0x28800000);
/// `#define DITHER_O (DITHER_WI * 0.5f)` (Core/Inc/Soundengine/DSPAudio/dither.h:49) = 1.52587890625e-05
pub const DITHER_O: f32 = f32::from_bits(0x37800000);

// ---- types
/// C: `struct Dither_Struct` (Core/Inc/Soundengine/DSPAudio/dither.h:51), 24 bytes.
/// C typedef name `Dither`; the struct tag `Dither_Struct` is a type alias below.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Dither {
    /// `int16_t r1, r2; //rectangular-PDF random numbers`
    pub r1: i16,
    /// `int16_t r1, r2; //rectangular-PDF random numbers`
    pub r2: i16,
    /// `float s1, s2; //error feedback buffers`
    pub s1: f32,
    /// `float s1, s2; //error feedback buffers`
    pub s2: f32,
    /// `float in, tmp;`
    pub r#in: f32,
    /// `float in, tmp;`
    pub tmp: f32,
    /// `int16_t out;`
    pub out: i16,
}
/// C struct tag of `Dither`.
pub type Dither_Struct = Dither;
impl Dither {
    /// All-zero value (C static zero initialisation).
    pub const ZERO: Self = Self { r1: 0, r2: 0, s1: 0.0, s2: 0.0, r#in: 0.0, tmp: 0.0, out: 0 };
}
impl Default for Dither {
    fn default() -> Self {
        Self::ZERO
    }
}

// ---- functions that use globals (methods on `Lxr`)
impl Lxr {
    /// C: `int16_t dither_process(Dither* dither, float in)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/dither.c:26`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P1.
    /// Calls: `Lxr::GetRngValue`.
    pub fn dither_process(&mut self, dither: &mut Dither, r#in: f32) -> i16 {
        // Not in the firmware build: ported from the C semantics (untestable against the reference ELF).
        let mut r#in = r#in;
        dither.r2 = dither.r1;
        dither.r1 = self.GetRngValue() as i16;

        r#in += DITHER_S * (dither.s1 + dither.s1 - dither.s2);
        dither.tmp = r#in + DITHER_O + DITHER_D * (dither.r1 as i32 - dither.r2 as i32) as f32;

        dither.out = ((DITHER_W * dither.tmp) as i32) as i16;
        if dither.tmp < 0.0 {
            dither.out = dither.out.wrapping_sub(1);
        }

        dither.s2 = dither.s1;
        dither.s1 = r#in - DITHER_WI * dither.out as f32;

        dither.out
    }
}
