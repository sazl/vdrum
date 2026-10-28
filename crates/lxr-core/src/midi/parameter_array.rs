//! Port of `Core/Inc/Soundengine/MIDI/ParameterArray.h`, `Core/Src/Soundengine/MIDI/ParameterArray.c`.
//!
//! Porting group: P3. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;

// ---- #define constants
/// `#define TYPE_UINT8 0` (Core/Inc/Soundengine/MIDI/ParameterArray.h:18)
pub const TYPE_UINT8: i32 = 0;
/// `#define TYPE_FLT 1` (Core/Inc/Soundengine/MIDI/ParameterArray.h:19)
pub const TYPE_FLT: i32 = 1;
/// `#define TYPE_SPECIAL_F 2` (Core/Inc/Soundengine/MIDI/ParameterArray.h:20)
pub const TYPE_SPECIAL_F: i32 = 2;
/// `#define TYPE_UINT32 3` (Core/Inc/Soundengine/MIDI/ParameterArray.h:21)
pub const TYPE_UINT32: i32 = 3;
/// `#define TYPE_SPECIAL_P 4` (Core/Inc/Soundengine/MIDI/ParameterArray.h:22)
pub const TYPE_SPECIAL_P: i32 = 4;
/// `#define TYPE_SPECIAL_FILTER_F 5` (Core/Inc/Soundengine/MIDI/ParameterArray.h:23)
pub const TYPE_SPECIAL_FILTER_F: i32 = 5;

// ---- types
/// C: `typedef union { float flt; uint32_t itg; } ptrValue;` (Core/Inc/Soundengine/MIDI/ParameterArray.h:367), 4 bytes.
///
/// Representation: the union's storage as the raw 32 bit pattern (`itg`); `flt` is accessed through
/// `flt()`/`set_flt()` (`f32::from_bits`/`to_bits`), which is exactly what the C union does.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct ptrValue {
    /// `uint32_t itg;`
    pub itg: u32,
}
impl ptrValue {
    pub const ZERO: Self = Self { itg: 0 };
    /// `float flt;` (read)
    #[inline(always)]
    pub fn flt(&self) -> f32 { f32::from_bits(self.itg) }
    /// `float flt;` (write)
    #[inline(always)]
    pub fn set_flt(&mut self, v: f32) { self.itg = v.to_bits() }
}

/// C: `struct ParameterStruct` (Core/Inc/Soundengine/MIDI/ParameterArray.h:373), 8 bytes.
/// C typedef name `Parameter`; the struct tag `ParameterStruct` is a type alias below.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Parameter {
    /// `void* ptr;`
    ///
    /// C pointer: stored as the address in the reference firmware memory map (`CPtr`, 0 = NULL), see `memmap`.
    pub ptr: CPtr,
    /// `uint8_t type;`
    pub r#type: u8,
}
/// C struct tag of `Parameter`.
pub type ParameterStruct = Parameter;
impl Parameter {
    /// All-zero value (C static zero initialisation).
    pub const ZERO: Self = Self { ptr: 0, r#type: 0 };
}
impl Default for Parameter {
    fn default() -> Self {
        Self::ZERO
    }
}

// ---- functions that use globals (methods on `Lxr`)
impl Lxr {
    /// C: `IN_ITCM_RAM void paramArray_setParameter(uint16_t idx, ptrValue newValue)`
    ///
    /// Source: `Core/Src/Soundengine/MIDI/ParameterArray.c:32`. Reference build: out-of-line in ITCM. Group P3.
    /// Globals: `activeParameters`, `parameterArray`.
    /// Calls: `Lxr::midiParser_ccHandler`.
    pub fn paramArray_setParameter(&mut self, idx: u16, newValue: ptrValue) {
        todo!("port: Core/Src/Soundengine/MIDI/ParameterArray.c:32")
    }

    /// C: `void parameterArray_init()`
    ///
    /// Source: `Core/Src/Soundengine/MIDI/ParameterArray.c:108`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `cymbalVoice`, `fxBus`, `hatVoice`, `mixer_audioRouting`, `mixer_decimation_rate`, `parameterArray`, `snareVoice`, `velocityModulators`, `voiceArray`.
    pub fn parameterArray_init(&mut self) {
        todo!("port: Core/Src/Soundengine/MIDI/ParameterArray.c:108")
    }
}
