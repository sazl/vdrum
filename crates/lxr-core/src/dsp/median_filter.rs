//! Port of `Core/Inc/Soundengine/DSPAudio/medianFilter.h`.
//!
//! Porting group: P1. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;

// ---- #define constants
/// `#define NIL 0` (Core/Inc/Soundengine/DSPAudio/medianFilter.h:20)
pub const NIL: i32 = 0;
/// `#define MEDIAN_FILTER_SIZE (7)` (Core/Inc/Soundengine/DSPAudio/medianFilter.h:21)
pub const MEDIAN_FILTER_SIZE: i32 = 7;

// ---- types
/// C: `struct pair` (Core/Inc/Soundengine/DSPAudio/medianFilter.h:25), 8 bytes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct pair {
    /// `struct pair *point; / * Pointers forming list linked in sorted order * /`
    ///
    /// C pointer: stored as the address in the reference firmware memory map (`CPtr`, 0 = NULL), see `memmap`.
    pub point: CPtr,
    /// `uint32_t value; / * Values to sort * /`
    pub value: u32,
}
impl pair {
    /// All-zero value (C static zero initialisation).
    pub const ZERO: Self = Self { point: 0, value: 0 };
}
impl Default for pair {
    fn default() -> Self {
        Self::ZERO
    }
}

// ---- functions without access to globals
/// C: `uint32_t medianFilter(uint32_t datum)`
///
/// Source: `Core/Inc/Soundengine/DSPAudio/medianFilter.h:23`. Reference build: not linked (removed by --gc-sections). Group P1.
pub fn medianFilter(datum: u32) -> u32 {
    todo!("port: Core/Inc/Soundengine/DSPAudio/medianFilter.h:23")
}
