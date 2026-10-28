//! Port of `Core/Inc/Preset/PresetManager.h`, `Core/Src/Preset/presetManager.c`.
//!
//! Porting group: P3. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

pub mod init_kit;
pub mod seq_step;

use crate::prelude::*;
use crate::front_panel_parser::{CC_2, CC_LFO_TARGET, CC_VELO_TARGET};
use crate::menu::cc2_text::{getNumModTargets, modTargets};

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
    let mut isEmpty: u8 = 1;
    if name[0] != b'E' && name[0] != 0 { isEmpty = 0; }
    if name[1] != b'm' && name[1] != 0 { isEmpty = 0; }
    if name[2] != b'p' && name[2] != 0 { isEmpty = 0; }
    if name[3] != b't' && name[3] != 0 { isEmpty = 0; }
    if name[4] != b'y' && name[4] != 0 { isEmpty = 0; }
    if name[5] != 0 { isEmpty = 0; }
    if name[6] != 0 { isEmpty = 0; }
    if name[7] != 0 { isEmpty = 0; }
    isEmpty
}

/// C: `static uint8_t interpolate(uint8_t a, uint8_t b, uint8_t x)`
///
/// Source: `Core/Src/Preset/presetManager.c:1253`. Reference build: inlined into its callers only. Group P3.
pub fn interpolate(a: u8, b: u8, x: u8) -> u8 {
    // int arithmetic, truncated to uint16_t
    let fixedPointValue = ((a as i32) * 256 + ((b as i32) - (a as i32)) * (x as i32)) as u16;
    let result = (fixedPointValue / 256) as u8;
    if (fixedPointValue & 0xff) < 0x7f { result } else { result.wrapping_add(1) }
}

// ---- functions that use globals (methods on `Lxr`)
impl Lxr {
    /// C: `void preset_loadDefaultGlobals()` (`Core/Src/Preset/presetManager.c:1029`). Group P3 (engine
    /// init, `docs/ENGINE_CONTRACT.md` section 2).
    pub fn preset_loadDefaultGlobals(&mut self) {
        self.activeParameters[PAR_BPM as usize] = 120;
        self.activeParameters[PAR_MIDI_CHAN_1 as usize] = 1;
        self.activeParameters[PAR_MIDI_CHAN_2 as usize] = 1;
        self.activeParameters[PAR_MIDI_CHAN_3 as usize] = 1;
        self.activeParameters[PAR_MIDI_CHAN_4 as usize] = 1;
        self.activeParameters[PAR_MIDI_CHAN_5 as usize] = 1;
        self.activeParameters[PAR_MIDI_CHAN_6 as usize] = 1;
        self.activeParameters[PAR_MIDI_CHAN_7 as usize] = 1;
        self.activeParameters[PAR_MIDI_NOTE1 as usize] = 48;
        self.activeParameters[PAR_MIDI_NOTE2 as usize] = 48;
        self.activeParameters[PAR_MIDI_NOTE3 as usize] = 48;
        self.activeParameters[PAR_MIDI_NOTE4 as usize] = 48;
        self.activeParameters[PAR_MIDI_NOTE5 as usize] = 48;
        self.activeParameters[PAR_MIDI_NOTE6 as usize] = 48;
        self.activeParameters[PAR_MIDI_NOTE7 as usize] = 48;
        self.activeParameters[PAR_FETCH as usize] = 0;
        self.activeParameters[PAR_FOLLOW as usize] = 0;
        self.activeParameters[PAR_QUANTISATION as usize] = 0;
        self.activeParameters[PAR_SCREENSAVER_ON_OFF as usize] = 1;
        self.activeParameters[PAR_MIDI_ROUTING as usize] = 0;
        self.activeParameters[PAR_MIDI_FILT_TX as usize] = 15;
        self.activeParameters[PAR_MIDI_FILT_RX as usize] = 15;
        self.activeParameters[PAR_PRESCALER_CLOCK_IN as usize] = 0;
        self.activeParameters[PAR_PRESCALER_CLOCK_OUT1 as usize] = 0;
        self.activeParameters[PAR_PRESCALER_CLOCK_OUT2 as usize] = 0;
        self.activeParameters[PAR_TRIG_GATE_MODE as usize] = 0;
        self.activeParameters[PAR_BAR_RESET_MODE as usize] = 1;
        self.activeParameters[PAR_MIDI_CHAN_GLOBAL as usize] = 0;
        self.activeParameters[PAR_PATTERN_MODE as usize] = 1;
    }

    /// C: `void preset_touchTimer(PresetTimer type)`
    ///
    /// Source: `Core/Src/Preset/presetManager.c:174`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `preset_kitDirty`, `preset_kitTimer`, `preset_patDirty`, `preset_patTimer`.
    pub fn preset_touchTimer(&mut self, r#type: PresetTimer) {
        if r#type == PRESET_KIT_TIMER {
            self.preset_kitTimer = self.HAL_GetTick();
            self.preset_kitDirty = 1;
        } else if r#type == PRESET_PAT_TIMER {
            self.preset_patTimer = self.HAL_GetTick();
            self.preset_patDirty = 1;
        }
    }

    /// C: `void preset_init(uint8_t presetNr)`
    ///
    /// Source: `Core/Src/Preset/presetManager.c:750`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `activeParameters`, `mainParameters`, `savefileNames`.
    /// Tables: `initKit`.
    /// Calls: `menu::cc2_text::getNumModTargets`, `preset::preset_isNameEmpty`, `Lxr::preset_sendDrumsetParameters`.
    pub fn preset_init(&mut self, presetNr: u8) {
        let p = presetNr as usize;
        // first the preset name
        self.savefileNames[p][KIT_NAMES as usize].copy_from_slice(&initKit[0..8]);
        if preset_isNameEmpty(&mut self.savefileNames[p][KIT_NAMES as usize]) != 0 {
            // sprintf(..., "Empty"): 5 characters and the terminating 0
            self.savefileNames[p][KIT_NAMES as usize][0..6].copy_from_slice(b"Empty\0");
        }

        // then the data
        for i in 8..END_OF_SOUND_PARAMETERS as usize {
            self.mainParameters[p][i - 8] = initKit[i];
        }

        // special case mod targets. normalize.
        let nmt = getNumModTargets();
        for i in 0..6usize {
            if self.mainParameters[p][PAR_VEL_DEST_1 as usize + i] >= nmt {
                self.mainParameters[p][PAR_VEL_DEST_1 as usize + i] = 0;
            }
            if self.mainParameters[p][PAR_TARGET_LFO1 as usize + i] >= nmt {
                self.mainParameters[p][PAR_TARGET_LFO1 as usize + i] = 0;
            }
        }

        // copy parameters to edit buffer
        let src = self.mainParameters[p];
        self.activeParameters[..END_OF_SOUND_PARAMETERS as usize].copy_from_slice(&src);
        self.preset_sendDrumsetParameters();
    }

    /// C: `void preset_loadDrumsetRam(uint8_t presetNr, uint8_t isMorph, uint8_t sendImmediate)`
    ///
    /// Source: `Core/Src/Preset/presetManager.c:834`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `activeParameters`, `mainParameters`, `morphParameters`.
    /// Calls: `Lxr::preset_sendDrumsetParameters`, `Lxr::preset_sendDrumsetParametersImmediate`, `Lxr::soundengine_process`.
    pub fn preset_loadDrumsetRam(&mut self, presetNr: u8, isMorph: u8, sendImmediate: u8) {
        self.soundengine_process();
        if isMorph != 0 {
            self.morphParameters = Some(presetNr as usize);
        } else {
            // copy parameters to edit buffer
            let src = self.mainParameters[presetNr as usize];
            self.activeParameters[..END_OF_SOUND_PARAMETERS as usize].copy_from_slice(&src);
        }
        self.soundengine_process();

        if sendImmediate != 0 {
            self.preset_sendDrumsetParametersImmediate();
        } else {
            self.preset_sendDrumsetParameters();
        }
    }

    /// C: `IN_ITCM_RAM void preset_sendDrumsetParametersImmediate()`
    ///
    /// Source: `Core/Src/Preset/presetManager.c:1263`. Reference build: out-of-line in ITCM. Group P3.
    /// Globals: `activeParameters`, `preset_sendDrumsetParametersImmediate_msg`.
    /// Tables: `modTargets`, `parameterOrder`.
    /// Calls: `Lxr::frontPanel_sendData`, `Lxr::midiParser_ccHandler`, `Lxr::soundengine_process`.
    pub fn preset_sendDrumsetParametersImmediate(&mut self) {
        self.preset_sendModTargets();

        // stuff from the morph function
        for i in 0..END_OF_SOUND_PARAMETERS as usize {
            let paramNumber = parameterOrder[i];
            let val = self.activeParameters[paramNumber as usize];
            let msg = &mut self.preset_sendDrumsetParametersImmediate_msg;
            if paramNumber < 128 {
                msg.status = MIDI_CC as u8;
                msg.data1 = paramNumber;
                msg.data2 = val;
                msg.data1 = msg.data1.wrapping_add(1);
                msg.data1 &= 0x7f;
            } else {
                msg.status = CC_2 as u8;
                msg.data1 = paramNumber.wrapping_sub(128); // stupid offset only in CC not CC2
                msg.data2 = val;
                msg.data1 &= 0x7f;
            }
            let m = *msg;
            self.midiParser_ccHandler(m, 1, 0, 0.0);
            self.soundengine_process();
        }
    }

    /// C: `void preset_sendDrumsetParameters()`
    ///
    /// Source: `Core/Src/Preset/presetManager.c:1350`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `activeParameters`, `morph_index`, `morph_state`.
    /// Tables: `modTargets`.
    /// Calls: `Lxr::frontPanel_sendData`, `Lxr::preset_morph`.
    pub fn preset_sendDrumsetParameters(&mut self) {
        self.preset_sendModTargets();

        // send parameters (possibly combined with morph parameters) to back
        let m = self.activeParameters[PAR_MORPH as usize];
        self.preset_morph(m);
    }

    /// C: `void preset_morph_tick()`
    ///
    /// Source: `Core/Src/Preset/presetManager.c:2128`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `activeParameters`, `morphParameters`, `morph_index`, `morph_state`, `preset_morph_tick_msg`.
    /// Calls: `preset::interpolate`, `Lxr::midiParser_ccHandler`.
    pub fn preset_morph_tick(&mut self) {
        if self.morph_state != 0 {
            let morph_index = self.morph_index;
            // Do not morph audio routing
            let morph: u8 = if morph_index as u16 >= PAR_AUDIO_OUT1 && morph_index as u16 <= PAR_AUDIO_OUT6 {
                0 // always use source kit (morph = 0) for audio routing
            } else {
                self.activeParameters[PAR_MORPH as usize]
            };

            let target = match self.morphParameters {
                Some(slot) => self.mainParameters[slot][morph_index as usize],
                // NULL pointer: the firmware reads ITCM code at 0x00000000 + index (see `SHIPPED_ITCM_HEAD`)
                None => SHIPPED_ITCM_HEAD[morph_index as usize],
            };
            let val = interpolate(self.activeParameters[morph_index as usize], target, morph);

            let msg = &mut self.preset_morph_tick_msg;
            if morph_index < 128 {
                msg.status = MIDI_CC as u8;
                msg.data1 = morph_index;
                msg.data2 = val;
                msg.data1 = msg.data1.wrapping_add(1);
                msg.data1 &= 0x7f;
            } else {
                msg.status = CC_2 as u8;
                msg.data1 = morph_index.wrapping_sub(128); // stupid offset only in CC not CC2
                msg.data2 = val;
                msg.data1 &= 0x7f;
            }
            let m = *msg;
            self.midiParser_ccHandler(m, 1, 0, 0.0);

            self.morph_index = self.morph_index.wrapping_add(1);
            if self.morph_index as u16 >= END_OF_SOUND_PARAMETERS {
                // finished morphing
                self.morph_state = 0;
            }
        }
    }

    /// C: `void preset_morph(uint8_t morph)`
    ///
    /// Source: `Core/Src/Preset/presetManager.c:2201`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `morph_index`, `morph_state`.
    pub fn preset_morph(&mut self, morph: u8) {
        let _ = morph; // the non-blocking (`#if 1`) variant ignores the value
        self.morph_state = 1; // start morph
        self.morph_index = 0; // reset index
    }
}

/// The first 256 bytes of the shipped v1.71 ITCM code (payload offset 0x3f3a0, runtime address 0x00000000).
///
/// `preset_morph_tick` reads `morphParameters[i]` through a NULL `morphParameters` when no morph kit was
/// ever loaded (always the case without an SD card), i.e. it interpolates towards these code bytes. They
/// are specific to the shipped binary (the GCC 11.3 reference ELF has different ITCM code), see
/// docs/ENGINE_CONTRACT.md section 9.
pub static SHIPPED_ITCM_HEAD: [u8; 256] = [
    0x2d, 0xe9, 0xf8, 0x43, 0x31, 0x4d, 0x00, 0x24, 0x31, 0x4e, 0x01, 0x27, 0x2d, 0xed, 0x02, 0x8b,
    0x15, 0xf8, 0x01, 0x3f, 0xd8, 0x20, 0x06, 0xeb, 0x83, 0x03, 0x9a, 0x78, 0x44, 0xea, 0xd2, 0x11,
    0x02, 0xf0, 0x7f, 0x02, 0x03, 0xf0, 0xa0, 0xfb, 0x2b, 0x7b, 0xd7, 0x20, 0x01, 0x3b, 0x05, 0x2b,
    0xab, 0x7c, 0x06, 0xeb, 0x83, 0x03, 0x94, 0xbf, 0xd7, 0x20, 0x2f, 0x73, 0x9a, 0x78, 0x44, 0xea,
    0xd2, 0x11, 0x02, 0x34, 0x02, 0xf0, 0x7f, 0x02, 0xe4, 0xb2, 0x03, 0xf0, 0x8d, 0xfb, 0x0c, 0x2c,
    0xde, 0xd1, 0x20, 0x4d, 0x4f, 0xf0, 0xd6, 0x08, 0x1f, 0x4e, 0x20, 0x4c, 0x05, 0xf1, 0xf7, 0x07,
    0x9f, 0xed, 0x1f, 0x8a, 0x0c, 0xe0, 0x84, 0xf8, 0x00, 0xe0, 0x84, 0xf8, 0x03, 0xc0, 0xa0, 0x70,
    0x94, 0xe8, 0x03, 0x00, 0x03, 0xf0, 0x6c, 0xfb, 0x03, 0xf0, 0xae, 0xfb, 0xbd, 0x42, 0x21, 0xd0,
    0x15, 0xf8, 0x01, 0x1f, 0xb0, 0xee, 0x48, 0x0a, 0x4f, 0xf0, 0xb0, 0x0e, 0x00, 0x23, 0x48, 0x1c,
    0xa1, 0xf1, 0x80, 0x09, 0x16, 0xf8, 0x01, 0xc0, 0x09, 0x06, 0x4f, 0xf0, 0x01, 0x02, 0x00, 0xf0,
    0x7f, 0x00, 0xe0, 0xd5, 0x84, 0xf8, 0x03, 0xc0, 0x00, 0x23, 0x01, 0x22, 0x84, 0xf8, 0x00, 0x80,
    0x84, 0xf8, 0x02, 0x90, 0x94, 0xe8, 0x03, 0x00, 0x03, 0xf0, 0x4a, 0xfb, 0x03, 0xf0, 0x8c, 0xfb,
    0xbd, 0x42, 0xdd, 0xd1, 0xbd, 0xec, 0x02, 0x8b, 0xbd, 0xe8, 0xf8, 0x83, 0xbc, 0xd2, 0x05, 0x20,
    0x44, 0xac, 0x03, 0x08, 0xff, 0xff, 0x01, 0x20, 0x28, 0xd2, 0x05, 0x20, 0x1c, 0x0d, 0x02, 0x20,
    0x00, 0x00, 0x00, 0x00, 0xf0, 0xb5, 0x47, 0x4c, 0x05, 0x9f, 0x94, 0xf8, 0x01, 0xc0, 0x06, 0x9d,
    0xbc, 0xf1, 0x02, 0x0f, 0x0c, 0xf1, 0xff, 0x3c, 0x34, 0xbf, 0x02, 0x26, 0x01, 0x26, 0xbc, 0xf1,
];

impl Lxr {
    /// The mod target block shared by `preset_sendDrumsetParameters` and
    /// `preset_sendDrumsetParametersImmediate` (identical source in both functions).
    fn preset_sendModTargets(&mut self) {
        for i in 0..6u8 {
            // **VELO load drumkit. translate to param value before sending
            let value = modTargets[self.activeParameters[PAR_VEL_DEST_1 as usize + i as usize] as usize].param as u8;
            let upper = ((value & 0x80) >> 7) | ((i & 0x3f) << 1);
            let lower = value & 0x7f;
            self.frontPanel_sendData(CC_VELO_TARGET as u8, upper, lower);

            // ensure target voice # is valid
            let vl = PAR_VOICE_LFO1 as usize + i as usize;
            if self.activeParameters[vl] < 1 || self.activeParameters[vl] > 6 {
                self.activeParameters[vl] = 1;
            }

            // **LFO par_target_lfo will be an index into modTargets, but we need a parameter number to send
            let value = modTargets[self.activeParameters[PAR_TARGET_LFO1 as usize + i as usize] as usize].param as u8;
            let upper = ((value & 0x80) >> 7) | ((i & 0x3f) << 1);
            let lower = value & 0x7f;
            self.frontPanel_sendData(CC_LFO_TARGET as u8, upper, lower);
        }
    }

    /// C: `uint32_t HAL_GetTick()`. `uwTick` is advanced by the SysTick interrupt, which the reference
    /// emulator never runs (it stays 0); only the preset save timers read it.
    pub fn HAL_GetTick(&mut self) -> u32 {
        0
    }
}
