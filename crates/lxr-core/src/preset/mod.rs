//! Port of `Core/Inc/Preset/PresetManager.h`, `Core/Src/Preset/presetManager.c`.
//!
//! Porting group: P3. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

pub mod init_kit;
pub mod seq_step;

use crate::prelude::*;

// ---- #define constants
/// `#define SKIP_NEXT_KIT_AUTOLOAD 2` (Core/Inc/Preset/PresetManager.h:44)
pub const SKIP_NEXT_KIT_AUTOLOAD: i32 = 2;
/// `#define FILE_VERSION 2` (Core/Src/Preset/presetManager.c:51)
pub const FILE_VERSION: i32 = 2;
/// `#define STEPS_PER_PATTERN NUM_STEPS` (Core/Src/Preset/presetManager.c:56)
pub const STEPS_PER_PATTERN: i32 = 64;
/// `#define FEXT_SOUND 0` (Core/Src/Preset/presetManager.c:59)
pub const FEXT_SOUND: i32 = 0;
/// `#define FEXT_PAT 1` (Core/Src/Preset/presetManager.c:60)
pub const FEXT_PAT: i32 = 1;
/// `#define FEXT_ALL 2` (Core/Src/Preset/presetManager.c:61)
pub const FEXT_ALL: i32 = 2;
/// `#define FEXT_PROJ 3` (Core/Src/Preset/presetManager.c:62)
pub const FEXT_PROJ: i32 = 3;
/// `#define FEXT_SONG 4` (Core/Src/Preset/presetManager.c:63)
pub const FEXT_SONG: i32 = 4;
/// `#define PRESET_SAVE_TIME 2000` (Core/Src/Preset/presetManager.c:79)
pub const PRESET_SAVE_TIME: i32 = 2000;

// ---- enums
// C: `enum {...}` (Core/Inc/Preset/PresetManager.h:28), 1 bytes, underlying type `u8`. (anonymous: enumerators only)
pub const KIT_NAMES: u8 = 0;
pub const PATTERN_NAMES: u8 = 1;
pub const SONG_NAMES: u8 = 2;
pub const PROJECT_NAMES: u8 = 3;
pub const NUM_SAVEFILE_NAMES: u8 = 4;
pub const UNUSED_NAMES: u8 = 5;
/// C: `enum {...}` (Core/Inc/Preset/PresetManager.h:50), 1 bytes, underlying type `u8`.
/// Representation: integer type alias plus one `pub const` per enumerator.
pub type PresetTimer = u8;
pub const PRESET_KIT_TIMER: PresetTimer = 0;
pub const PRESET_PAT_TIMER: PresetTimer = 1;
pub const PRESET_STOP_PAT_TIMER: PresetTimer = 2;

// ---- functions without access to globals
/// C: `uint8_t preset_isNameEmpty(char* name)`
///
/// Source: `Core/Src/Preset/presetManager.c:732`. Reference build: out-of-line in flash. Group P3.
pub fn preset_isNameEmpty(name: &mut [u8]) -> u8 {
    todo!("port: Core/Src/Preset/presetManager.c:732")
}

/// C: `static uint8_t interpolate(uint8_t a, uint8_t b, uint8_t x)`
///
/// Source: `Core/Src/Preset/presetManager.c:1253`. Reference build: inlined into its callers only. Group P3.
pub fn interpolate(a: u8, b: u8, x: u8) -> u8 {
    todo!("port: Core/Src/Preset/presetManager.c:1253")
}

// ---- functions that use globals (methods on `Lxr`)
impl Lxr {
    /// C: `void preset_touchTimer(PresetTimer type)`
    ///
    /// Source: `Core/Src/Preset/presetManager.c:174`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `preset_kitDirty`, `preset_kitTimer`, `preset_patDirty`, `preset_patTimer`.
    pub fn preset_touchTimer(&mut self, r#type: PresetTimer) {
        todo!("port: Core/Src/Preset/presetManager.c:174")
    }

    /// C: `void preset_init(uint8_t presetNr)`
    ///
    /// Source: `Core/Src/Preset/presetManager.c:750`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `activeParameters`, `mainParameters`, `savefileNames`.
    /// Tables: `initKit`.
    /// Calls: `menu::cc2_text::getNumModTargets`, `preset::preset_isNameEmpty`, `Lxr::preset_sendDrumsetParameters`.
    pub fn preset_init(&mut self, presetNr: u8) {
        todo!("port: Core/Src/Preset/presetManager.c:750")
    }

    /// C: `void preset_loadDrumsetRam(uint8_t presetNr, uint8_t isMorph, uint8_t sendImmediate)`
    ///
    /// Source: `Core/Src/Preset/presetManager.c:834`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `activeParameters`, `mainParameters`, `morphParameters`.
    /// Calls: `Lxr::preset_sendDrumsetParameters`, `Lxr::preset_sendDrumsetParametersImmediate`, `Lxr::soundengine_process`.
    pub fn preset_loadDrumsetRam(&mut self, presetNr: u8, isMorph: u8, sendImmediate: u8) {
        todo!("port: Core/Src/Preset/presetManager.c:834")
    }

    /// C: `IN_ITCM_RAM void preset_sendDrumsetParametersImmediate()`
    ///
    /// Source: `Core/Src/Preset/presetManager.c:1263`. Reference build: out-of-line in ITCM. Group P3.
    /// Globals: `activeParameters`, `preset_sendDrumsetParametersImmediate_msg`.
    /// Tables: `modTargets`, `parameterOrder`.
    /// Calls: `Lxr::frontPanel_sendData`, `Lxr::midiParser_ccHandler`, `Lxr::soundengine_process`.
    pub fn preset_sendDrumsetParametersImmediate(&mut self) {
        todo!("port: Core/Src/Preset/presetManager.c:1263")
    }

    /// C: `void preset_sendDrumsetParameters()`
    ///
    /// Source: `Core/Src/Preset/presetManager.c:1350`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `activeParameters`, `morph_index`, `morph_state`.
    /// Tables: `modTargets`.
    /// Calls: `Lxr::frontPanel_sendData`, `Lxr::preset_morph`.
    pub fn preset_sendDrumsetParameters(&mut self) {
        todo!("port: Core/Src/Preset/presetManager.c:1350")
    }

    /// C: `void preset_morph_tick()`
    ///
    /// Source: `Core/Src/Preset/presetManager.c:2128`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `activeParameters`, `morphParameters`, `morph_index`, `morph_state`, `preset_morph_tick_msg`.
    /// Calls: `preset::interpolate`, `Lxr::midiParser_ccHandler`.
    pub fn preset_morph_tick(&mut self) {
        todo!("port: Core/Src/Preset/presetManager.c:2128")
    }

    /// C: `void preset_morph(uint8_t morph)`
    ///
    /// Source: `Core/Src/Preset/presetManager.c:2201`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `morph_index`, `morph_state`.
    pub fn preset_morph(&mut self, morph: u8) {
        todo!("port: Core/Src/Preset/presetManager.c:2201")
    }
}
