//! Port of `Core/Inc/menu/menu.h`, `Core/Src/menu/menu.c`.
//!
//! Porting group: P3. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

pub mod cc2_text;

use crate::prelude::*;

// ---- #define constants
/// `#define NUM_PRESET_LOCATIONS 6` (Core/Inc/menu/menu.h:54)
pub const NUM_PRESET_LOCATIONS: i32 = 6;

// ---- C macros that are not constants (function-like or not constant expressions); not translated:
// Core/Inc/menu/menu.h:40: #define pgm_read_word(x) (*((uint16_t *)(x)))

// ---- enums
// C: `enum {...}` (Core/Inc/menu/menu.h:58), 1 bytes, underlying type `u8`. (anonymous: enumerators only)
pub const CURRENT_KIT: u8 = 0;
pub const CURRENT_MORPH: u8 = 1;
pub const CURRENT_SONG: u8 = 2;
pub const CURRENT_PROJECT: u8 = 3;
pub const NUM_CURRENT: u8 = 4;
/// C: `enum PageNames` (Core/Inc/menu/menu.h:73), 1 bytes, underlying type `u8`.
/// Representation: integer type alias plus one `pub const` per enumerator.
pub type PageNames = u8;
pub const VOICE1_PAGE: PageNames = 0;
pub const VOICE2_PAGE: PageNames = 1;
pub const VOICE3_PAGE: PageNames = 2;
pub const VOICE4_PAGE: PageNames = 3;
pub const VOICE5_PAGE: PageNames = 4;
pub const VOICE6_PAGE: PageNames = 5;
pub const VOICE7_PAGE: PageNames = 6;
pub const MENU_MIDI_PAGE: PageNames = 7;
pub const LOAD_PAGE: PageNames = 8;
pub const SAVE_PAGE: PageNames = 9;
pub const PERF_PAGE: PageNames = 10;
pub const SEQ_PAGE: PageNames = 11;
pub const EUKLID_PAGE: PageNames = 12;
pub const PATTERN_SETTINGS_PAGE: PageNames = 13;
pub const RECORDING_PAGE: PageNames = 14;
pub const FX_MODE_PAGE: PageNames = 15;
pub const PATTERN_LOAD_PAGE: PageNames = 16;
pub const PATTERN_SAVE_PAGE: PageNames = 17;
pub const PATTERN_SAVE_PAGE_OK: PageNames = 18;
pub const MENU_LAST_STEP_PAGE: PageNames = 19;
pub const KIT_SAVE_PAGE_OK: PageNames = 20;
pub const SONG_MODE_PAGE: PageNames = 21;
pub const NUM_PAGES: PageNames = 22;
/// C: `enum loadSaveEnum` (Core/Inc/menu/menu.h:516), 1 bytes, underlying type `u8`.
/// Representation: integer type alias plus one `pub const` per enumerator.
pub type loadSaveEnum = u8;
pub const SAVE_TYPE_KIT: loadSaveEnum = 0;
pub const SAVE_TYPE_PATTERN: loadSaveEnum = 1;
pub const SAVE_TYPE_SONG: loadSaveEnum = 2;
pub const SAVE_TYPE_MORPH: loadSaveEnum = 3;
pub const SAVE_TYPE_PROJECT: loadSaveEnum = 4;
pub const SAVE_TYPE_ALL: loadSaveEnum = 5;
pub const SAVE_TYPE_GLO: loadSaveEnum = 6;
pub const NUM_SAVE_TYPES: loadSaveEnum = 7;

// ---- types
/// C: `struct ModTargStruct` (Core/Inc/menu/menu.h:570), 4 bytes.
/// C typedef name `ModTarg`; the struct tag `ModTargStruct` is a type alias below.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ModTarg {
    /// `const uint8_t nameIdx; // index into NameStruct (valueNames)`
    pub nameIdx: u8,
    /// `const uint16_t param; // equivalent param number`
    pub param: u16,
}
/// C struct tag of `ModTarg`.
pub type ModTargStruct = ModTarg;
impl ModTarg {
    /// All-zero value (C static zero initialisation).
    pub const ZERO: Self = Self { nameIdx: 0, param: 0 };
}
impl Default for ModTarg {
    fn default() -> Self {
        Self::ZERO
    }
}

// ---- functions that use globals (methods on `Lxr`)
impl Lxr {
    /// C: `void menu_setShownPattern(uint8_t patternNr)`
    ///
    /// Source: `Core/Src/menu/menu.c:725`. Reference build: out-of-line in flash. Group P3.
    ///
    /// Hardware / UI function outside the engine, called from engine code. Decide its behaviour
    /// (no-op, fixed value, or an emulation of its effect on engine state) when porting the caller.
    pub fn menu_setShownPattern(&mut self, patternNr: u8) {
        todo!("port: Core/Src/menu/menu.c:725")
    }

    /// C: `uint8_t menu_getViewedPattern()`
    ///
    /// Source: `Core/Src/menu/menu.c:731`. Reference build: out-of-line in flash. Group P3.
    ///
    /// Hardware / UI function outside the engine, called from engine code. Decide its behaviour
    /// (no-op, fixed value, or an emulation of its effect on engine state) when porting the caller.
    pub fn menu_getViewedPattern(&mut self) -> u8 {
        todo!("port: Core/Src/menu/menu.c:731")
    }

    /// C: `void menu_repaintAll()`
    ///
    /// Source: `Core/Src/menu/menu.c:1026`. Reference build: out-of-line in flash. Group P3.
    ///
    /// Hardware / UI function outside the engine, called from engine code. Decide its behaviour
    /// (no-op, fixed value, or an emulation of its effect on engine state) when porting the caller.
    pub fn menu_repaintAll(&mut self) {
        todo!("port: Core/Src/menu/menu.c:1026")
    }

    /// C: `void menu_repaint()`
    ///
    /// Source: `Core/Src/menu/menu.c:1811`. Reference build: out-of-line in flash. Group P3.
    ///
    /// Hardware / UI function outside the engine, called from engine code. Decide its behaviour
    /// (no-op, fixed value, or an emulation of its effect on engine state) when porting the caller.
    pub fn menu_repaint(&mut self) {
        todo!("port: Core/Src/menu/menu.c:1811")
    }

    /// C: `void menu_switchSubPage(uint8_t subPageNr, uint8_t toggle)`
    ///
    /// Source: `Core/Src/menu/menu.c:3156`. Reference build: out-of-line in flash. Group P3.
    ///
    /// Hardware / UI function outside the engine, called from engine code. Decide its behaviour
    /// (no-op, fixed value, or an emulation of its effect on engine state) when porting the caller.
    pub fn menu_switchSubPage(&mut self, subPageNr: u8, toggle: u8) {
        todo!("port: Core/Src/menu/menu.c:3156")
    }

    /// C: `uint8_t menu_getSubPage()`
    ///
    /// Source: `Core/Src/menu/menu.c:3212`. Reference build: out-of-line in flash. Group P3.
    ///
    /// Hardware / UI function outside the engine, called from engine code. Decide its behaviour
    /// (no-op, fixed value, or an emulation of its effect on engine state) when porting the caller.
    pub fn menu_getSubPage(&mut self) -> u8 {
        todo!("port: Core/Src/menu/menu.c:3212")
    }

    /// C: `uint8_t menu_getActiveVoice()`
    ///
    /// Source: `Core/Src/menu/menu.c:4282`. Reference build: out-of-line in flash. Group P3.
    ///
    /// Hardware / UI function outside the engine, called from engine code. Decide its behaviour
    /// (no-op, fixed value, or an emulation of its effect on engine state) when porting the caller.
    pub fn menu_getActiveVoice(&mut self) -> u8 {
        todo!("port: Core/Src/menu/menu.c:4282")
    }
}
