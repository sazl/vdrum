//! Port of `Drivers/CMSIS/Include/cmsis_gcc.h`.
//!
//! Porting group: P1. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;

// ---- functions without access to globals
/// C: `__STATIC_FORCEINLINE uint32_t __get_APSR(void)`
///
/// Source: `Drivers/CMSIS/Include/cmsis_gcc.h:219`. Reference build: inlined into its callers only. Group P1.
pub fn __get_APSR() -> u32 {
    // Reads the flags of the preceding instruction; they are not observable from Rust. The only caller
    // (`calcNoise`, not linked in the firmware) computes the V flag itself. Not callable.
    unimplemented!("__get_APSR: flags are not observable in Rust; compute them at the call site")
}

/// C: `__STATIC_FORCEINLINE uint32_t __QADD16(uint32_t op1, uint32_t op2)`
///
/// Source: `Drivers/CMSIS/Include/cmsis_gcc.h:1644`. Reference build: inlined into its callers only. Group P1.
pub fn __QADD16(op1: u32, op2: u32) -> u32 {
    // QADD16: signed saturating addition of each halfword
    let lo = (op1 as i16).saturating_add(op2 as i16) as u16 as u32;
    let hi = ((op1 >> 16) as i16).saturating_add((op2 >> 16) as i16) as u16 as u32;
    (hi << 16) | lo
}

/// C: `__STATIC_FORCEINLINE uint32_t __QSUB16(uint32_t op1, uint32_t op2)`
///
/// Source: `Drivers/CMSIS/Include/cmsis_gcc.h:1692`. Reference build: inlined into its callers only. Group P1.
pub fn __QSUB16(op1: u32, op2: u32) -> u32 {
    // QSUB16: signed saturating subtraction of each halfword
    let lo = (op1 as i16).saturating_sub(op2 as i16) as u16 as u32;
    let hi = ((op1 >> 16) as i16).saturating_sub((op2 >> 16) as i16) as u16 as u32;
    (hi << 16) | lo
}
