//! Port of `Core/Inc/Preset/SeqStep.h`.
//!
//! Porting group: P3. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;

// ---- enums
/// C: `enum StepPositionEnum` (Core/Inc/Preset/SeqStep.h:44), 1 bytes, underlying type `u8`.
/// Representation: integer type alias plus one `pub const` per enumerator.
pub type StepPositionEnum = u8;
/// C: `typedef enum StepPositionEnum StepPosition;`
pub type StepPosition = StepPositionEnum;
pub const STEP_PREV: StepPositionEnum = 0;
pub const STEP_CURRENT: StepPositionEnum = 1;
pub const STEP_NEXT: StepPositionEnum = 2;

// ---- types
/// C: `typedef struct StepTiming { uint8_t flam:4; uint8_t shift:4; } StepTiming;` (Core/Inc/Preset/SeqStep.h:37), 1 byte.
///
/// Representation: the bit-field storage byte `bits`; GCC (little endian ARM) allocates `flam` in bits 0..3 and
/// `shift` in bits 4..7. Use the accessors.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct StepTiming {
    pub bits: u8,
}
impl StepTiming {
    pub const ZERO: Self = Self { bits: 0 };
    /// `uint8_t flam:4;`
    #[inline(always)]
    pub fn flam(&self) -> u8 { self.bits & 0x0f }
    #[inline(always)]
    pub fn set_flam(&mut self, v: u8) { self.bits = (self.bits & 0xf0) | (v & 0x0f) }
    /// `uint8_t shift:4;`
    #[inline(always)]
    pub fn shift(&self) -> u8 { self.bits >> 4 }
    #[inline(always)]
    pub fn set_shift(&mut self, v: u8) { self.bits = (self.bits & 0x0f) | (v << 4) }
}

/// C: `struct StepStruct` (Core/Inc/Preset/SeqStep.h:22), 7 bytes.
/// C typedef name `Step`; the struct tag `StepStruct` is a type alias below.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Step {
    /// `uint8_t volume; // 0-127 volume -> 0x7f => lower 7 bit, upper bit => active <- active c an probably be removed`
    pub volume: u8,
    /// `uint8_t prob; //step probability 7 bit [0:100]`
    pub prob: u8,
    /// `uint8_t note; // 7 bit`
    pub note: u8,
    /// `uint8_t param1Nr;`
    pub param1Nr: u8,
    /// `uint8_t param1Val; //probably 7`
    pub param1Val: u8,
    /// `uint8_t param2Nr;`
    pub param2Nr: u8,
    /// `uint8_t param2Val; //probably 7`
    pub param2Val: u8,
}
/// C struct tag of `Step`.
pub type StepStruct = Step;
impl Step {
    /// All-zero value (C static zero initialisation).
    pub const ZERO: Self = Self { volume: 0, prob: 0, note: 0, param1Nr: 0, param1Val: 0, param2Nr: 0, param2Val: 0 };
}
impl Default for Step {
    fn default() -> Self {
        Self::ZERO
    }
}
