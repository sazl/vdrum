//! Port of `Core/Inc/Soundengine/FIFO.h`, `Core/Src/Soundengine/FIFO.c`.
//!
//! Porting group: P3. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;

// ---- #define constants
/// `#define BUFFER_SIZE 256` (Core/Inc/Soundengine/FIFO.h:18)
pub const BUFFER_SIZE: i32 = 256;
/// `#define BUFFER_MASK (BUFFER_SIZE-1)` (Core/Inc/Soundengine/FIFO.h:19)
pub const BUFFER_MASK: i32 = 255;
/// `#define BUFFER_SIZE_BIG 64` (Core/Inc/Soundengine/FIFO.h:39)
pub const BUFFER_SIZE_BIG: i32 = 64;
/// `#define BUFFER_MASK_BIG (BUFFER_SIZE_BIG-1)` (Core/Inc/Soundengine/FIFO.h:40)
pub const BUFFER_MASK_BIG: i32 = 63;

// ---- types
/// C: `struct FifoStruct` (Core/Inc/Soundengine/FIFO.h:21), 258 bytes.
/// C typedef name `Fifo`; the struct tag `FifoStruct` is a type alias below.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Fifo {
    /// `volatile uint8_t data[BUFFER_SIZE];`
    pub data: [u8; 256],
    /// `volatile uint8_t read; // zeigt auf das Feld mit dem �ltesten Inhalt`
    pub read: u8,
    /// `volatile uint8_t write; // zeigt immer auf leeres Feld`
    pub write: u8,
}
/// C struct tag of `Fifo`.
pub type FifoStruct = Fifo;
impl Fifo {
    /// All-zero value (C static zero initialisation).
    pub const ZERO: Self = Self { data: [0; 256], read: 0, write: 0 };
}
impl Default for Fifo {
    fn default() -> Self {
        Self::ZERO
    }
}

/// C: `struct FifoStructBig` (Core/Inc/Soundengine/FIFO.h:42), 66 bytes.
/// C typedef name `FifoBig`; the struct tag `FifoStructBig` is a type alias below.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FifoBig {
    /// `volatile uint8_t data[BUFFER_SIZE_BIG];`
    pub data: [u8; 64],
    /// `volatile uint8_t read; // zeigt auf das Feld mit dem �ltesten Inhalt`
    pub read: u8,
    /// `volatile uint8_t write; // zeigt immer auf leeres Feld`
    pub write: u8,
}
/// C struct tag of `FifoBig`.
pub type FifoStructBig = FifoBig;
impl FifoBig {
    /// All-zero value (C static zero initialisation).
    pub const ZERO: Self = Self { data: [0; 64], read: 0, write: 0 };
}
impl Default for FifoBig {
    fn default() -> Self {
        Self::ZERO
    }
}

// ---- functions without access to globals
/// C: `uint8_t fifo_bufferOut(Fifo* fifo, uint8_t *pByte)`
///
/// Source: `Core/Src/Soundengine/FIFO.c:36`. Reference build: out-of-line in flash. Group P3.
pub fn fifo_bufferOut(fifo: &mut Fifo, pByte: &mut u8) -> u8 {
    todo!("port: Core/Src/Soundengine/FIFO.c:36")
}
