//! Port of `Core/Inc/menu/menu.h`, `Core/Src/menu/menu.c`.
//!
//! Porting group: P3. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

pub mod cc2_text;

use crate::prelude::*;
use crate::menu::cc2_text::{getModTargetGapIndex, paramToModTargetInit, modTargets};

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
    /// C: `void copyClear_clearCurrentPattern()` (`Core/Src/menu/copyClearTools.c:37`). Group P3 (engine
    /// init, `docs/ENGINE_CONTRACT.md` section 2).
    pub fn copyClear_clearCurrentPattern(&mut self) {
        let pattern = self.menu_getViewedPattern();
        self.led_clearSequencerLeds();
        self.seq_clearPattern(pattern);
    }

    /// C: `void menu_setShownPattern(uint8_t patternNr)`
    ///
    /// Source: `Core/Src/menu/menu.c:725`. Reference build: out-of-line in flash. Group P3.
    ///
    /// Hardware / UI function outside the engine, called from engine code. Decide its behaviour
    /// (no-op, fixed value, or an emulation of its effect on engine state) when porting the caller.
    pub fn menu_setShownPattern(&mut self, patternNr: u8) {
        self.menu_shownPattern = patternNr;
        let p = self.menu_shownPattern;
        self.frontPanel_sendData(ui::SEQ_CC, ui::SEQ_SET_SHOWN_PATTERN, p);
    }

    /// C: `uint8_t menu_getViewedPattern()`
    ///
    /// Source: `Core/Src/menu/menu.c:731`. Reference build: out-of-line in flash. Group P3.
    ///
    /// Hardware / UI function outside the engine, called from engine code. Decide its behaviour
    /// (no-op, fixed value, or an emulation of its effect on engine state) when porting the caller.
    pub fn menu_getViewedPattern(&mut self) -> u8 {
        self.menu_shownPattern
    }

    /// C: `void menu_repaintAll()`
    ///
    /// Source: `Core/Src/menu/menu.c:1026`. Reference build: out-of-line in flash. Group P3.
    ///
    /// Hardware / UI function outside the engine, called from engine code. Decide its behaviour
    /// (no-op, fixed value, or an emulation of its effect on engine state) when porting the caller.
    pub fn menu_repaintAll(&mut self) {
        // LCD only: no-op.
    }

    /// C: `void menu_repaint()`
    ///
    /// Source: `Core/Src/menu/menu.c:1811`. Reference build: out-of-line in flash. Group P3.
    ///
    /// Hardware / UI function outside the engine, called from engine code. Decide its behaviour
    /// (no-op, fixed value, or an emulation of its effect on engine state) when porting the caller.
    pub fn menu_repaint(&mut self) {
        // LCD only: no-op.
    }

    /// C: `void menu_switchSubPage(uint8_t subPageNr, uint8_t toggle)`
    ///
    /// Source: `Core/Src/menu/menu.c:3156`. Reference build: out-of-line in flash. Group P3.
    ///
    /// Hardware / UI function outside the engine, called from engine code. Decide its behaviour
    /// (no-op, fixed value, or an emulation of its effect on engine state) when porting the caller.
    pub fn menu_switchSubPage(&mut self, subPageNr: u8, toggle: u8) {
        // UI only (menuIndex, LCD, LEDs are not modelled): no-op.
        let _ = (subPageNr, toggle);
    }

    /// C: `uint8_t menu_getSubPage()`
    ///
    /// Source: `Core/Src/menu/menu.c:3212`. Reference build: out-of-line in flash. Group P3.
    ///
    /// Hardware / UI function outside the engine, called from engine code. Decide its behaviour
    /// (no-op, fixed value, or an emulation of its effect on engine state) when porting the caller.
    pub fn menu_getSubPage(&mut self) -> u8 {
        // `(menuIndex & MASK_PAGE) >> PAGE_SHIFT`; menuIndex is UI state that is not modelled and is 0
        // on the page that the engine queries it for (FX_MODE_PAGE is never entered without the UI).
        0
    }

    /// C: `uint8_t menu_getActiveVoice()`
    ///
    /// Source: `Core/Src/menu/menu.c:4282`. Reference build: out-of-line in flash. Group P3.
    ///
    /// Hardware / UI function outside the engine, called from engine code. Decide its behaviour
    /// (no-op, fixed value, or an emulation of its effect on engine state) when porting the caller.
    pub fn menu_getActiveVoice(&mut self) -> u8 {
        self.menu_activeVoice
    }
}

// ---- UI-side front panel protocol constants (`Core/Inc/frontPanelParser_ui.h`) used by the menu code below
mod ui {
    pub const SEQ_CC: u8 = 0xd2;
    pub const VOICE_CC: u8 = 0xd4;
    pub const SET_BPM: u8 = 0xd5;
    pub const SET_P1_DEST: u8 = 0xda;
    pub const SET_P2_DEST: u8 = 0xdb;
    pub const SET_P1_VAL: u8 = 0xdc;
    pub const SET_P2_VAL: u8 = 0xdd;
    pub const VOICE_DECIMATION: u8 = 0x03;
    pub const SEQ_REQUEST_STEP_PARAMS: u8 = 0x0f;
    pub const SEQ_ROLL_RATE: u8 = 0x11;
    pub const SEQ_VOLUME: u8 = 0x12;
    pub const SEQ_NOTE: u8 = 0x13;
    pub const SEQ_PROB: u8 = 0x14;
    pub const SEQ_SET_ACTIVE_TRACK: u8 = 0x15;
    pub const SEQ_FLAM: u8 = 0x16;
    pub const SEQ_SET_SHOWN_PATTERN: u8 = 0x1a;
    pub const SEQ_SHIFT: u8 = 0x1c;
    pub const SEQ_SET_AUTOM_TRACK: u8 = 0x23;
    pub const SEQ_SELECT_ACTIVE_STEP: u8 = 0x24;
    pub const SEQ_SHUFFLE: u8 = 0x25;
    pub const SEQ_TRACK_LENGTH: u8 = 0x26;
    pub const SEQ_POSY: u8 = 0x2a;
    pub const SEQ_FLUX: u8 = 0x2b;
    pub const SEQ_SOM_FREQ: u8 = 0x2c;
    pub const SEQ_MIDI_CHAN: u8 = 0x2d;
    pub const SEQ_MIDI_ROUTING: u8 = 0x2f;
    pub const SEQ_MIDI_FILT_TX: u8 = 0x30;
    pub const SEQ_MIDI_FILT_RX: u8 = 0x31;
    pub const SEQ_BAR_RESET_MODE: u8 = 0x32;
    pub const SEQ_TRIGGER_IN_PPQ: u8 = 0x36;
    pub const SEQ_TRIGGER_OUT1_PPQ: u8 = 0x37;
    pub const SEQ_TRIGGER_OUT2_PPQ: u8 = 0x38;
    pub const SEQ_TRIGGER_GATE_MODE: u8 = 0x39;
}
pub use ui::SEQ_SET_ACTIVE_TRACK as UI_SEQ_SET_ACTIVE_TRACK;

// ---- parameter data types (`Core/Src/menu/menu.c`, `parameter_dtypes`)
/// `DTYPE_TARGET_SELECTION_LFO` (low byte of a `parameter_dtypes` entry).
pub const DTYPE_TARGET_SELECTION_LFO: u8 = 7;
/// `DTYPE_TARGET_SELECTION_VELO`.
pub const DTYPE_TARGET_SELECTION_VELO: u8 = 8;
/// `DTYPE_VOICE_LFO`.
pub const DTYPE_VOICE_LFO: u8 = 9;

/// `const uint16_t parameter_dtypes[NUM_PARAMS]` (menu.c, flash 0x0803cfe4 in the reference ELF; byte-identical
/// in the shipped v1.71 at 0x0803cbc0). Low byte: data type, high byte: menu id.
pub static parameter_dtypes: [u16; 302] = [
    0x0001, 0x0203, 0x0203, 0x0203, 0x0203, 0x0001, 0x0203, 0x0203, 0x0004, 0x0004, 0x0004, 0x0004,
    0x0004, 0x0004, 0x0004, 0x0004, 0x0004, 0x0004, 0x0004, 0x0004, 0x0203, 0x0203, 0x0203, 0x0203,
    0x0203, 0x0203, 0x0203, 0x0001, 0x0001, 0x0004, 0x0004, 0x0001, 0x0001, 0x0004, 0x0004, 0x0001,
    0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001,
    0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001,
    0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001,
    0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0004,
    0x0001, 0x0004, 0x0001, 0x0004, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0004, 0x0004,
    0x0004, 0x0001, 0x0001, 0x0004, 0x0004, 0x0004, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001,
    0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001,
    0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001,
    0x0001, 0x0001, 0x0f03, 0x0f03, 0x0f03, 0x0006, 0x0006, 0x0006, 0x0006, 0x0006, 0x0006, 0x0001,
    0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0008, 0x0008, 0x0008, 0x0008, 0x0008, 0x0008, 0x0603,
    0x0603, 0x0603, 0x0603, 0x0603, 0x0603, 0x0009, 0x0009, 0x0009, 0x0009, 0x0009, 0x0009, 0x0007,
    0x0007, 0x0007, 0x0007, 0x0007, 0x0007, 0x0703, 0x0703, 0x0703, 0x0703, 0x0703, 0x0703, 0x0503,
    0x0503, 0x0503, 0x0503, 0x0503, 0x0503, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0103,
    0x0103, 0x0103, 0x0103, 0x0103, 0x0103, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0a03,
    0x0a03, 0x0a03, 0x0a03, 0x0a03, 0x0a03, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0303,
    0x0303, 0x0303, 0x0303, 0x0303, 0x0303, 0x000c, 0x000c, 0x000c, 0x000c, 0x000c, 0x000c, 0x000c,
    0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x0001, 0x1203, 0x1a03, 0x000f, 0x1303, 0x0001, 0x0001,
    0x0001, 0x0001, 0x1403, 0x0203, 0x1603, 0x1503, 0x0001, 0x0403, 0x0000, 0x0001, 0x0001, 0x000f,
    0x000c, 0x0005, 0x0005, 0x000d, 0x000b, 0x000a, 0x000a, 0x0001, 0x0001, 0x0001, 0x0001, 0x0803,
    0x000e, 0x0001, 0x0001, 0x0001, 0x0001, 0x0005, 0x1803, 0x1903, 0x0000, 0x0005, 0x0005, 0x0005,
    0x0005, 0x0005, 0x0005, 0x0006, 0x0006, 0x0903, 0x0006, 0x0b03, 0x0005, 0x0c03, 0x0d03, 0x0d03,
    0x0e03, 0x1003, 0x1003, 0x0006, 0x0006, 0x0005, 0x000c, 0x000c, 0x000c, 0x000c, 0x000c, 0x000c,
    0x000c, 0x1703,
];

impl Lxr {
    /// C: `void menu_init()` (`Core/Src/menu/menu.c:737`), reduced to its effects on engine state.
    ///
    /// LCD, LED and display calls are UI only. `saveScreenPots`, `editModeActive`, `menu_muteModeActive`
    /// and the menu's active parameter are UI state that is not modelled. `menu_switchPage(0)` reduces
    /// to its default branch: `menu_activePage = 0; menu_setActiveVoice(0)`.
    pub fn menu_init(&mut self) {
        paramToModTargetInit(&mut self.paramToModTarget);

        self.menu_selectedPresetNr = [0; 6];
        self.menu_loadedPresetNr = [0; 4];
        for row in self.mainParameters.iter_mut() {
            for b in row.iter_mut() {
                *b = 0;
            }
        }

        self.activeParameters[PAR_EUKLID_LENGTH as usize] = 16;
        self.activeParameters[PAR_EUKLID_STEPS as usize] = 16;
        self.activeParameters[PAR_EUKLID_ROTATION as usize] = 0;
        self.activeParameters[PAR_ROLL as usize] = 5;
        self.activeParameters[PAR_BPM as usize] = 120;

        // menu_switchPage(0): led_clearSequencerLeds (UI), default branch
        self.menu_activePage = 0;
        self.menu_setActiveVoice(0);
        // led_updateSeqLeds, led_setActiveVoiceLeds, menu_resetActiveParameter, menu_repaintAll: UI only

        self.frontPanel_sendData(ui::SEQ_CC, ui::SEQ_SET_ACTIVE_TRACK, 0);
    }

    /// C: `void menu_setActiveVoice(uint8_t voiceNr)` (`Core/Src/menu/menu.c:4287`).
    pub fn menu_setActiveVoice(&mut self, voiceNr: u8) {
        let t = self.activeParameters[(PAR_TARGET_LFO1 as usize) + voiceNr as usize];
        self.menu_TargetVoiceGapIndex = getModTargetGapIndex(t);
        self.menu_activeVoice = voiceNr;
    }

    /// C: `void menu_parseGlobalParam(uint16_t paramNr, uint8_t value)` (`Core/Src/menu/menu.c:3418`).
    /// Byte-identical in the shipped v1.71 binary.
    pub fn menu_parseGlobalParam(&mut self, paramNr: u16, value: u8) {
        use ui::*;
        let mut paramNr = paramNr;
        match paramNr {
            PAR_MIDI_CHAN_7 | PAR_MIDI_CHAN_1 | PAR_MIDI_CHAN_2 | PAR_MIDI_CHAN_3 | PAR_MIDI_CHAN_4
            | PAR_MIDI_CHAN_5 | PAR_MIDI_CHAN_6 | PAR_MIDI_CHAN_GLOBAL => {
                if paramNr == PAR_MIDI_CHAN_7 {
                    paramNr = paramNr.wrapping_sub(5); // they are not after one another in the param list
                }
                let channel = value.wrapping_sub(1);
                let voice: u8 = if paramNr == PAR_MIDI_CHAN_GLOBAL {
                    7
                } else {
                    paramNr.wrapping_sub(PAR_MIDI_CHAN_1) as u8
                };
                self.frontPanel_sendData(SEQ_CC, SEQ_MIDI_CHAN, (voice << 4) | channel);
            }
            UNUSED_PAR_GLO_SAT => {}
            PAR_POS_Y => {
                let v = self.menu_getActiveVoice();
                self.frontPanel_sendData(SEQ_CC, SEQ_SET_ACTIVE_TRACK, v);
                self.frontPanel_sendData(SEQ_CC, SEQ_POSY, value);
            }
            PAR_FLUX => {
                let v = self.menu_getActiveVoice();
                self.frontPanel_sendData(SEQ_CC, SEQ_SET_ACTIVE_TRACK, v);
                self.frontPanel_sendData(SEQ_CC, SEQ_FLUX, value);
            }
            PAR_SOM_FREQ => {
                let v = self.menu_getActiveVoice();
                self.frontPanel_sendData(SEQ_CC, SEQ_SET_ACTIVE_TRACK, v);
                self.frontPanel_sendData(SEQ_CC, SEQ_SOM_FREQ, value);
            }
            PAR_TRACK_LENGTH => {
                let v = self.menu_getActiveVoice();
                self.frontPanel_sendData(SEQ_CC, SEQ_SET_ACTIVE_TRACK, v);
                self.frontPanel_sendData(SEQ_CC, SEQ_TRACK_LENGTH, value);
            }
            PAR_SHUFFLE => self.frontPanel_sendData(SEQ_CC, SEQ_SHUFFLE, value),
            PAR_AUTOM_TRACK => self.frontPanel_sendData(SEQ_CC, SEQ_SET_AUTOM_TRACK, value),
            PAR_P1_DEST | PAR_P2_DEST => {
                let tmp = modTargets[value as usize].param as u8;
                let st = self.activeParameters[PAR_ACTIVE_STEP as usize];
                self.frontPanel_sendData(SEQ_CC, SEQ_SELECT_ACTIVE_STEP, st);
                let cmd = if paramNr == PAR_P1_DEST { SET_P1_DEST } else { SET_P2_DEST };
                self.frontPanel_sendData(cmd, tmp >> 7, tmp & 0x7f);
            }
            PAR_P1_VAL => {
                let st = self.activeParameters[PAR_ACTIVE_STEP as usize];
                self.frontPanel_sendData(SET_P1_VAL, st, value);
            }
            PAR_P2_VAL => {
                let st = self.activeParameters[PAR_ACTIVE_STEP as usize];
                self.frontPanel_sendData(SET_P2_VAL, st, value);
            }
            PAR_SCREENSAVER_ON_OFF => {}
            PAR_BPM => self.frontPanel_sendData(SET_BPM, value & 0x7f, (value >> 7) & 0x7f),
            PAR_MORPH => self.preset_morph(value),
            PAR_FETCH => {}
            PAR_VOICE_DECIMATION1 | PAR_VOICE_DECIMATION2 | PAR_VOICE_DECIMATION3 | PAR_VOICE_DECIMATION4
            | PAR_VOICE_DECIMATION5 | PAR_VOICE_DECIMATION6 | PAR_VOICE_DECIMATION_ALL => {
                self.frontPanel_sendData(SEQ_CC, SEQ_SET_ACTIVE_TRACK, paramNr.wrapping_sub(PAR_VOICE_DECIMATION1) as u8);
                self.frontPanel_sendData(VOICE_CC, VOICE_DECIMATION, value);
            }
            PAR_ROLL => self.frontPanel_sendData(SEQ_CC, SEQ_ROLL_RATE, value),
            PAR_EUKLID_LENGTH => {
                let length = value.wrapping_sub(1);
                let pattern = self.menu_shownPattern;
                let v = self.menu_getActiveVoice();
                self.frontPanel_sendData(SEQ_CC, SEQ_SET_ACTIVE_TRACK, v);
                let v = self.menu_getActiveVoice();
                self.euklid_setLength(v, length.wrapping_add(1));
                let steps = self.activeParameters[PAR_EUKLID_STEPS as usize];
                let v = self.menu_getActiveVoice();
                self.euklid_setSteps(v, steps, pattern);
                self.led_updateSeqLeds();
            }
            PAR_EUKLID_STEPS => {
                let pattern = self.menu_shownPattern;
                let v = self.menu_getActiveVoice();
                self.frontPanel_sendData(SEQ_CC, SEQ_SET_ACTIVE_TRACK, v);
                let v = self.menu_getActiveVoice();
                self.euklid_setSteps(v, value, pattern);
                self.led_updateSeqLeds();
            }
            PAR_EUKLID_ROTATION => {
                let pattern = self.menu_shownPattern;
                let v = self.menu_getActiveVoice();
                self.frontPanel_sendData(SEQ_CC, SEQ_SET_ACTIVE_TRACK, v);
                let v = self.menu_getActiveVoice();
                self.euklid_setRotation(v, value, pattern);
                self.led_updateSeqLeds();
            }
            PAR_ACTIVE_STEP => self.frontPanel_sendData(SEQ_CC, SEQ_REQUEST_STEP_PARAMS, value),
            PAR_STEP_PROB => self.frontPanel_sendData(SEQ_CC, SEQ_PROB, value),
            PAR_STEP_NOTE => self.frontPanel_sendData(SEQ_CC, SEQ_NOTE, value),
            PAR_STEP_FLAM => self.frontPanel_sendData(SEQ_CC, SEQ_FLAM, value),
            PAR_STEP_SHIFT => self.frontPanel_sendData(SEQ_CC, SEQ_SHIFT, value),
            PAR_STEP_VOLUME => self.frontPanel_sendData(SEQ_CC, SEQ_VOLUME, value),
            PAR_MIDI_ROUTING => self.frontPanel_sendData(SEQ_CC, SEQ_MIDI_ROUTING, value),
            PAR_MIDI_FILT_TX => self.frontPanel_sendData(SEQ_CC, SEQ_MIDI_FILT_TX, value),
            PAR_MIDI_FILT_RX => self.frontPanel_sendData(SEQ_CC, SEQ_MIDI_FILT_RX, value),
            PAR_PRESCALER_CLOCK_IN => self.frontPanel_sendData(SEQ_CC, SEQ_TRIGGER_IN_PPQ, value),
            PAR_PRESCALER_CLOCK_OUT1 => self.frontPanel_sendData(SEQ_CC, SEQ_TRIGGER_OUT1_PPQ, value),
            PAR_PRESCALER_CLOCK_OUT2 => self.frontPanel_sendData(SEQ_CC, SEQ_TRIGGER_OUT2_PPQ, value),
            PAR_TRIG_GATE_MODE => self.frontPanel_sendData(SEQ_CC, SEQ_TRIGGER_GATE_MODE, value),
            PAR_BAR_RESET_MODE => self.frontPanel_sendData(SEQ_CC, SEQ_BAR_RESET_MODE, value),
            _ => {}
        }
    }
}
