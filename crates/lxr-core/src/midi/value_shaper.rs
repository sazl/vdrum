//! Port of `Core/Inc/Soundengine/MIDI/valueShaper.h`.
//!
//! Porting group: P3. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;

// ---- functions without access to globals
/// C: `static inline float valueShaperI2F(uint8_t data2, float shape)`
///
/// Source: `Core/Inc/Soundengine/MIDI/valueShaper.h:22`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P3.
pub fn valueShaperI2F(data2: u8, shape: f32) -> f32 {
    todo!("port: Core/Inc/Soundengine/MIDI/valueShaper.h:22")
}

/// C: `static inline float valueShaperF2F(float val, float shape)`
///
/// Source: `Core/Inc/Soundengine/MIDI/valueShaper.h:30`. Reference build: inlined into its callers only. Group P3.
pub fn valueShaperF2F(val: f32, shape: f32) -> f32 {
    todo!("port: Core/Inc/Soundengine/MIDI/valueShaper.h:30")
}
