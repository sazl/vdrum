//! Port of `Core/Inc/Soundengine/DSPAudio/FX/FxRamBuffer.h`, `Core/Src/Soundengine/DSPAudio/FX/FxRamBuffer.c`.
//!
//! Porting group: P2. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;

// ---- #define constants
/// `#define FX_BUFFER_SIZE 0x7FFF` (Core/Inc/Soundengine/DSPAudio/FX/FxRamBuffer.h:22)
pub const FX_BUFFER_SIZE: i32 = 32767;
/// `#define MAX_ALLPASS_SIZE 0x800` (Core/Inc/Soundengine/DSPAudio/FX/FxRamBuffer.h:29)
pub const MAX_ALLPASS_SIZE: i32 = 2048;
