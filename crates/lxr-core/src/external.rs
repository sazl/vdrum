//! Port of `Core/Src/IO/adcPots.c`, `Core/Src/Soundengine/AudioCodecManager.c`, `Core/Src/Soundengine/TriggerOut.c`, `Core/Src/Soundengine/USB/usb_manager.c`, `Core/Src/buttonHandler.c`, `Core/Src/ledHandler.c`.
//!
//! Functions, constants and types of hardware / UI files that engine code uses. They are not part of the
//! sound engine; each stub must be given a deliberate behaviour (usually a no-op) by the group that ports
//! the caller (P3).
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;

// ---- #define constants
/// `#define HANDLE_SEQ_IN_CLOCK_INTERRUPT 0` (Core/Inc/Soundengine/TriggerOut.h:40)
pub const HANDLE_SEQ_IN_CLOCK_INTERRUPT: i32 = 0;
/// `#define switch_l1_Pin GPIO_PIN_6` (Core/Inc/main.h:104)
pub const switch_l1_Pin: u16 = 64;
/// `#define switch_r1_Pin GPIO_PIN_7` (Core/Inc/main.h:106)
pub const switch_r1_Pin: u16 = 128;
/// `#define switch_l2_Pin GPIO_PIN_4` (Core/Inc/main.h:108)
pub const switch_l2_Pin: u16 = 16;
/// `#define switch_r2_Pin GPIO_PIN_6` (Core/Inc/main.h:110)
pub const switch_r2_Pin: u16 = 64;

// ---- C macros that are not constants (function-like or not constant expressions); not translated:
// Core/Inc/main.h:105: #define switch_l1_GPIO_Port GPIOD    (not a constant expression; not translated)
// Core/Inc/main.h:107: #define switch_r1_GPIO_Port GPIOD    (not a constant expression; not translated)
// Core/Inc/main.h:109: #define switch_l2_GPIO_Port GPIOB    (not a constant expression; not translated)
// Core/Inc/main.h:111: #define switch_r2_GPIO_Port GPIOB    (not a constant expression; not translated)
// Core/Inc/main.h:123: #define INDTCMZ __attribute__ ((section(".dtcmz")))    (not a constant expression; not translated)

// ---- enums
/// C: `enum PrescalerPpqIn` (Core/Inc/Soundengine/TriggerOut.h:66), 1 bytes, underlying type `u8`.
/// Representation: integer type alias plus one `pub const` per enumerator.
pub type PrescalerPpqIn = u8;
pub const PRE_4_PPQ_IN: PrescalerPpqIn = 1;
pub const PRE_24_PPQ_IN: PrescalerPpqIn = 2;
/// C: `enum PrescalerPpqOut` (Core/Inc/Soundengine/TriggerOut.h:74), 1 bytes, underlying type `u8`.
/// Representation: integer type alias plus one `pub const` per enumerator.
pub type PrescalerPpqOut = u8;
pub const PRE_2_PPQ_OUT: PrescalerPpqOut = 0;
pub const PRE_4_PPQ_OUT: PrescalerPpqOut = 1;
pub const PRE_8_PPQ_OUT: PrescalerPpqOut = 2;
pub const PRE_16_PPQ_OUT: PrescalerPpqOut = 3;
pub const PRE_24_PPQ_OUT: PrescalerPpqOut = 4;
pub const PRE_48_PPQ_OUT: PrescalerPpqOut = 5;
/// C: `enum LedNumbers` (Core/Inc/ledHandler.h:100), 1 bytes, underlying type `u8`.
/// Representation: integer type alias plus one `pub const` per enumerator.
pub type LedNumbers = u8;
pub const LED_PAGE_UP: LedNumbers = 0;
pub const LED_VOICE1: LedNumbers = 1;
pub const LED_VOICE2: LedNumbers = 2;
pub const LED_VOICE3: LedNumbers = 3;
pub const LED_VOICE4: LedNumbers = 4;
pub const LED_VOICE5: LedNumbers = 5;
pub const LED_VOICE6: LedNumbers = 6;
pub const LED_VOICE7: LedNumbers = 7;
pub const LED_PART_SELECT5: LedNumbers = 8;
pub const LED_PART_SELECT6: LedNumbers = 9;
pub const LED_PART_SELECT7: LedNumbers = 10;
pub const LED_PART_SELECT8: LedNumbers = 11;
pub const LED_STEP16: LedNumbers = 12;
pub const LED_STEP15: LedNumbers = 13;
pub const LED_STEP14: LedNumbers = 14;
pub const LED_STEP13: LedNumbers = 15;
pub const LED_PART_SELECT1: LedNumbers = 16;
pub const LED_PART_SELECT2: LedNumbers = 17;
pub const LED_PART_SELECT3: LedNumbers = 18;
pub const LED_PART_SELECT4: LedNumbers = 19;
pub const LED_STEP12: LedNumbers = 20;
pub const LED_STEP11: LedNumbers = 21;
pub const LED_STEP10: LedNumbers = 22;
pub const LED_STEP9: LedNumbers = 23;
pub const LED_MODE1: LedNumbers = 24;
pub const LED_MODE2: LedNumbers = 25;
pub const LED_MODE3: LedNumbers = 26;
pub const LED_MODE4: LedNumbers = 27;
pub const LED_STEP8: LedNumbers = 28;
pub const LED_STEP7: LedNumbers = 29;
pub const LED_STEP6: LedNumbers = 30;
pub const LED_STEP5: LedNumbers = 31;
pub const LED_SHIFT: LedNumbers = 32;
pub const LED_START_STOP: LedNumbers = 33;
pub const LED_REC: LedNumbers = 34;
pub const LED_COPY: LedNumbers = 35;
pub const LED_STEP4: LedNumbers = 36;
pub const LED_STEP3: LedNumbers = 37;
pub const LED_STEP2: LedNumbers = 38;
pub const LED_STEP1: LedNumbers = 39;
pub const LED_PAGE_DOWN: LedNumbers = 40;
/// C: `enum SelectModes` (Core/Inc/buttonHandler.h:58), 1 bytes, underlying type `u8`.
/// Representation: integer type alias plus one `pub const` per enumerator.
pub type SelectModes = u8;
pub const SELECT_MODE_VOICE: SelectModes = 0;
pub const SELECT_MODE_PERF: SelectModes = 1;
pub const SELECT_MODE_LOAD: SelectModes = 2;
pub const SELECT_MODE_SAVE: SelectModes = 3;
pub const SELECT_MODE_STEP: SelectModes = 4;
pub const SELECT_MODE_PAT_GEN: SelectModes = 5;
pub const SELECT_MODE_FX: SelectModes = 6;
pub const SELECT_MODE_MENU: SelectModes = 7;
pub const SELECT_MODE_PATTERN: SelectModes = 8;
pub const SELECT_MODE_SONG: SelectModes = 9;
pub const SELECT_MODE_LAST_STEP_TRACK: SelectModes = 10;
pub const SELECT_MODE_LAST_STEP_ALL: SelectModes = 11;

// ---- functions that use globals (methods on `Lxr`)
impl Lxr {
    /// C: `float adc_getFaderVolume(uint8_t idx)`
    ///
    /// Source: `Core/Src/IO/adcPots.c:282`. Reference build: out-of-line in flash. Group P3.
    ///
    /// Hardware / UI function outside the engine, called from engine code. Decide its behaviour
    /// (no-op, fixed value, or an emulation of its effect on engine state) when porting the caller.
    pub fn adc_getFaderVolume(&mut self, idx: u8) -> f32 {
        todo!("port: Core/Src/IO/adcPots.c:282")
    }

    /// C: `int CodecInit()`
    ///
    /// Source: `Core/Src/Soundengine/AudioCodecManager.c:24`. Reference build: out-of-line in flash. Group P3.
    ///
    /// Hardware / UI function outside the engine, called from engine code. Decide its behaviour
    /// (no-op, fixed value, or an emulation of its effect on engine state) when porting the caller.
    pub fn CodecInit(&mut self) -> i32 {
        todo!("port: Core/Src/Soundengine/AudioCodecManager.c:24")
    }

    /// C: `void trigger_clockTick(uint8_t onOff, uint8_t cnt)`
    ///
    /// Source: `Core/Src/Soundengine/TriggerOut.c:484`. Reference build: out-of-line in flash. Group P3.
    ///
    /// Hardware / UI function outside the engine, called from engine code. Decide its behaviour
    /// (no-op, fixed value, or an emulation of its effect on engine state) when porting the caller.
    pub fn trigger_clockTick(&mut self, onOff: u8, cnt: u8) {
        todo!("port: Core/Src/Soundengine/TriggerOut.c:484")
    }

    /// C: `void trigger_reset(uint8_t value)`
    ///
    /// Source: `Core/Src/Soundengine/TriggerOut.c:558`. Reference build: out-of-line in flash. Group P3.
    ///
    /// Hardware / UI function outside the engine, called from engine code. Decide its behaviour
    /// (no-op, fixed value, or an emulation of its effect on engine state) when porting the caller.
    pub fn trigger_reset(&mut self, value: u8) {
        todo!("port: Core/Src/Soundengine/TriggerOut.c:558")
    }

    /// C: `void usb_tick()`
    ///
    /// Source: `Core/Src/Soundengine/USB/usb_manager.c:61`. Reference build: out-of-line in flash. Group P3.
    ///
    /// Hardware / UI function outside the engine, called from engine code. Decide its behaviour
    /// (no-op, fixed value, or an emulation of its effect on engine state) when porting the caller.
    pub fn usb_tick(&mut self) {
        todo!("port: Core/Src/Soundengine/USB/usb_manager.c:61")
    }

    /// C: `void usb_sendMidi(MidiMsg msg)`
    ///
    /// Source: `Core/Src/Soundengine/USB/usb_manager.c:136`. Reference build: out-of-line in flash. Group P3.
    ///
    /// Hardware / UI function outside the engine, called from engine code. Decide its behaviour
    /// (no-op, fixed value, or an emulation of its effect on engine state) when porting the caller.
    pub fn usb_sendMidi(&mut self, msg: MidiMsg) {
        todo!("port: Core/Src/Soundengine/USB/usb_manager.c:136")
    }

    /// C: `uint8_t buttonHandler_getMode()`
    ///
    /// Source: `Core/Src/buttonHandler.c:2212`. Reference build: out-of-line in flash. Group P3.
    ///
    /// Hardware / UI function outside the engine, called from engine code. Decide its behaviour
    /// (no-op, fixed value, or an emulation of its effect on engine state) when porting the caller.
    pub fn buttonHandler_getMode(&mut self) -> u8 {
        todo!("port: Core/Src/buttonHandler.c:2212")
    }

    /// C: `void led_initPatternSelectLeds()`
    ///
    /// Source: `Core/Src/ledHandler.c:77`. Reference build: out-of-line in flash. Group P3.
    ///
    /// Hardware / UI function outside the engine, called from engine code. Decide its behaviour
    /// (no-op, fixed value, or an emulation of its effect on engine state) when porting the caller.
    pub fn led_initPatternSelectLeds(&mut self) {
        todo!("port: Core/Src/ledHandler.c:77")
    }

    /// C: `void led_setValue(uint8_t val, uint8_t ledNr)`
    ///
    /// Source: `Core/Src/ledHandler.c:230`. Reference build: out-of-line in flash. Group P3.
    ///
    /// Hardware / UI function outside the engine, called from engine code. Decide its behaviour
    /// (no-op, fixed value, or an emulation of its effect on engine state) when porting the caller.
    pub fn led_setValue(&mut self, val: u8, ledNr: u8) {
        todo!("port: Core/Src/ledHandler.c:230")
    }

    /// C: `uint8_t led_idx2StepLed(uint8_t idx)`
    ///
    /// Source: `Core/Src/ledHandler.c:321`. Reference build: out-of-line in flash. Group P3.
    ///
    /// Hardware / UI function outside the engine, called from engine code. Decide its behaviour
    /// (no-op, fixed value, or an emulation of its effect on engine state) when porting the caller.
    pub fn led_idx2StepLed(&mut self, idx: u8) -> u8 {
        todo!("port: Core/Src/ledHandler.c:321")
    }

    /// C: `void led_setActive_step(uint8_t stepNr)`
    ///
    /// Source: `Core/Src/ledHandler.c:380`. Reference build: out-of-line in flash. Group P3.
    ///
    /// Hardware / UI function outside the engine, called from engine code. Decide its behaviour
    /// (no-op, fixed value, or an emulation of its effect on engine state) when porting the caller.
    pub fn led_setActive_step(&mut self, stepNr: u8) {
        todo!("port: Core/Src/ledHandler.c:380")
    }

    /// C: `void led_clearActive_step()`
    ///
    /// Source: `Core/Src/ledHandler.c:398`. Reference build: out-of-line in flash. Group P3.
    ///
    /// Hardware / UI function outside the engine, called from engine code. Decide its behaviour
    /// (no-op, fixed value, or an emulation of its effect on engine state) when porting the caller.
    pub fn led_clearActive_step(&mut self) {
        todo!("port: Core/Src/ledHandler.c:398")
    }

    /// C: `void led_updateSeqLeds()`
    ///
    /// Source: `Core/Src/ledHandler.c:403`. Reference build: out-of-line in flash. Group P3.
    ///
    /// Hardware / UI function outside the engine, called from engine code. Decide its behaviour
    /// (no-op, fixed value, or an emulation of its effect on engine state) when porting the caller.
    pub fn led_updateSeqLeds(&mut self) {
        todo!("port: Core/Src/ledHandler.c:403")
    }

    /// C: `void led_clearSequencerLeds()`
    ///
    /// Source: `Core/Src/ledHandler.c:450`. Reference build: out-of-line in flash. Group P3.
    ///
    /// Hardware / UI function outside the engine, called from engine code. Decide its behaviour
    /// (no-op, fixed value, or an emulation of its effect on engine state) when porting the caller.
    pub fn led_clearSequencerLeds(&mut self) {
        todo!("port: Core/Src/ledHandler.c:450")
    }

    /// C: `void led_clearAllBlinkLeds()`
    ///
    /// Source: `Core/Src/ledHandler.c:563`. Reference build: out-of-line in flash. Group P3.
    ///
    /// Hardware / UI function outside the engine, called from engine code. Decide its behaviour
    /// (no-op, fixed value, or an emulation of its effect on engine state) when porting the caller.
    pub fn led_clearAllBlinkLeds(&mut self) {
        todo!("port: Core/Src/ledHandler.c:563")
    }

    /// C: `void led_setBlinkLed(const uint8_t ledNr, const uint8_t onOff)`
    ///
    /// Source: `Core/Src/ledHandler.c:574`. Reference build: out-of-line in flash. Group P3.
    ///
    /// Hardware / UI function outside the engine, called from engine code. Decide its behaviour
    /// (no-op, fixed value, or an emulation of its effect on engine state) when porting the caller.
    pub fn led_setBlinkLed(&mut self, ledNr: u8, onOff: u8) {
        todo!("port: Core/Src/ledHandler.c:574")
    }

    /// C: `void led_resetBlinkTimer()`
    ///
    /// Source: `Core/Src/ledHandler.c:614`. Reference build: out-of-line in flash. Group P3.
    ///
    /// Hardware / UI function outside the engine, called from engine code. Decide its behaviour
    /// (no-op, fixed value, or an emulation of its effect on engine state) when porting the caller.
    pub fn led_resetBlinkTimer(&mut self) {
        todo!("port: Core/Src/ledHandler.c:614")
    }

    /// C: `void led_blinkHandler()`
    ///
    /// Source: `Core/Src/ledHandler.c:619`. Reference build: out-of-line in flash. Group P3.
    ///
    /// Hardware / UI function outside the engine, called from engine code. Decide its behaviour
    /// (no-op, fixed value, or an emulation of its effect on engine state) when porting the caller.
    pub fn led_blinkHandler(&mut self) {
        todo!("port: Core/Src/ledHandler.c:619")
    }
}
