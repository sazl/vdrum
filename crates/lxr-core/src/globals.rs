//! Port of `Core/Inc/Soundengine/globals.h`.
//!
//! Porting group: P3. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;

// ---- #define constants
/// `#define SAMPLE_VALID 0xff` (Core/Inc/Soundengine/globals.h:21)
pub const SAMPLE_VALID: i32 = 255;
/// `#define FILTER_SHAPER -0.9f` (Core/Inc/Soundengine/globals.h:22) = -0.8999999761581421
pub const FILTER_SHAPER: f32 = f32::from_bits(0xbf666666);
