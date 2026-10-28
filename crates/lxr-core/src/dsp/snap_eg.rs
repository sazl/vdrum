//! Port of `Core/Inc/Soundengine/DSPAudio/snapEg.h`, `Core/Src/Soundengine/DSPAudio/snapEg.c`.
//!
//! Porting group: P1. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;

// ---- #define constants
/// `#define SNAP_MAX_VALUE 24.f` (Core/Src/Soundengine/DSPAudio/snapEg.c:16) = 24.0
pub const SNAP_MAX_VALUE: f32 = f32::from_bits(0x41c00000);
/// `#define SNAP_REDUCTION 0.2f` (Core/Src/Soundengine/DSPAudio/snapEg.c:17) = 0.20000000298023224
pub const SNAP_REDUCTION: f32 = f32::from_bits(0x3e4ccccd);

// ---- types
/// C: `struct Snap_EG_Struct` (Core/Inc/Soundengine/DSPAudio/snapEg.h:20), 4 bytes.
/// C typedef name `SnapEg`; the struct tag `Snap_EG_Struct` is a type alias below.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SnapEg {
    /// `float value;`
    pub value: f32,
}
/// C struct tag of `SnapEg`.
pub type Snap_EG_Struct = SnapEg;
impl SnapEg {
    /// All-zero value (C static zero initialisation).
    pub const ZERO: Self = Self { value: 0.0 };
}
impl Default for SnapEg {
    fn default() -> Self {
        Self::ZERO
    }
}

// ---- functions without access to globals
/// C: `void SnapEg_init(SnapEg* eg)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/snapEg.c:19`. Reference build: out-of-line in flash. Group P1.
pub fn SnapEg_init(eg: &mut SnapEg) {
    todo!("port: Core/Src/Soundengine/DSPAudio/snapEg.c:19")
}

/// C: `void SnapEg_trigger(SnapEg* eg)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/snapEg.c:24`. Reference build: out-of-line in flash. Group P1.
pub fn SnapEg_trigger(eg: &mut SnapEg) {
    todo!("port: Core/Src/Soundengine/DSPAudio/snapEg.c:24")
}

/// C: `float SnapEg_calc(SnapEg* eg, float time)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/snapEg.c:29`. Reference build: out-of-line in flash. Group P1.
pub fn SnapEg_calc(eg: &mut SnapEg, time: f32) -> f32 {
    todo!("port: Core/Src/Soundengine/DSPAudio/snapEg.c:29")
}
