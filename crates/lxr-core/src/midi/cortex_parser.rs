//! Port of `Core/Inc/Soundengine/MIDI/CortexParser.h`, `Core/Src/Soundengine/MIDI/CortexParser.c`.
//!
//! Porting group: P3. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;

// ---- functions that use globals (methods on `Lxr`)
impl Lxr {
    /// C: `void frontParser_parseUartData(unsigned char data)`
    ///
    /// Source: `Core/Src/Soundengine/MIDI/CortexParser.c:99`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `activeParameters`, `cortex_midiMsg`, `cymbalVoice`, `frontParser_activeStep`, `frontParser_activeTrack`, `frontParser_rxCnt`, `frontParser_shownPattern`, `frontParser_sysexActive`, `frontParser_sysexBuffer`, `frontParser_sysexSeqStepNr`, `frontParser_twoByteData`, `hatVoice`, `midi_MidiChannels`, `seq_patternSet`, `seq_resetBarOnPatternChange`, `seq_selectedStep`, `snareVoice`, `trigger_dividerClockOut1`, `trigger_prescalerClockInput`, `velocityModulators`, `voiceArray`.
    /// Calls: `Lxr::frontParser_handleMidiMessage`, `Lxr::frontParser_handleSysexData`, `Lxr::uart_clearFrontFifo`, `Lxr::uart_sendFrontpanelSysExByte`.
    pub fn frontParser_parseUartData(&mut self, data: u8) {
        todo!("port: Core/Src/Soundengine/MIDI/CortexParser.c:99")
    }

    /// C: `static void frontParser_handleSysexData(unsigned char data)`
    ///
    /// Source: `Core/Src/Soundengine/MIDI/CortexParser.c:148`. Reference build: inlined into its callers only. Group P3.
    /// Globals: `frontParser_rxCnt`, `frontParser_sysexActive`, `frontParser_sysexBuffer`, `frontParser_sysexSeqStepNr`, `frontParser_twoByteData`, `seq_patternSet`.
    /// Calls: `Lxr::seq_sendStepInfoToFront`.
    pub fn frontParser_handleSysexData(&mut self, data: u8) {
        todo!("port: Core/Src/Soundengine/MIDI/CortexParser.c:148")
    }

    /// C: `static void frontParser_handleMidiMessage()`
    ///
    /// Source: `Core/Src/Soundengine/MIDI/CortexParser.c:395`. Reference build: inlined into its callers only. Group P3.
    /// Globals: `cortex_midiMsg`, `cymbalVoice`, `frontParser_activeTrack`, `frontParser_shownPattern`, `hatVoice`, `seq_patternSet`, `seq_selectedStep`, `snareVoice`, `velocityModulators`, `voiceArray`.
    /// Calls: `Lxr::frontParser_handleSeqCC`, `Lxr::midiParser_ccHandler`, `Lxr::modNode_setDestination`, `Lxr::seq_armAutomationStep`, `Lxr::seq_isAutomationStepArmed`, `Lxr::seq_recordAutomation`, `Lxr::seq_setBpm`, `Lxr::seq_setExtSync`.
    pub fn frontParser_handleMidiMessage(&mut self) {
        todo!("port: Core/Src/Soundengine/MIDI/CortexParser.c:395")
    }

    /// C: `static void frontParser_handleSeqCC()`
    ///
    /// Source: `Core/Src/Soundengine/MIDI/CortexParser.c:704`. Reference build: inlined into its callers only. Group P3.
    /// Globals: `activeParameters`, `cortex_midiMsg`, `frontParser_activeStep`, `frontParser_activeTrack`, `frontParser_shownPattern`, `midi_MidiChannels`, `seq_patternSet`, `seq_resetBarOnPatternChange`, `seq_selectedStep`, `trigger_dividerClockOut1`, `trigger_prescalerClockInput`.
    /// Calls: `Lxr::euklid_getLength`, `Lxr::euklid_getRotation`, `Lxr::euklid_getSteps`, `Lxr::euklid_setLength`, `Lxr::euklid_setRotation`, `Lxr::euklid_setSteps`, `Lxr::led_updateSeqLeds`, `Lxr::menu_repaint`, `Lxr::midiParser_setFilter`, `Lxr::midiParser_setRouting`, `Lxr::seq_clearAutomation`, `Lxr::seq_clearTrack`, `Lxr::seq_copyPattern`, `Lxr::seq_copyTrack`, `Lxr::seq_resetPosition`, `Lxr::seq_setActiveAutomationTrack`, `Lxr::seq_setErasingMode`, `Lxr::seq_setMute`, `Lxr::seq_setNextPattern`, `Lxr::seq_setQuantisation`, `Lxr::seq_setRecordingMode`, `Lxr::seq_setRoll`, `Lxr::seq_setRollRate`, `Lxr::seq_setRunning`, `Lxr::seq_setShuffle`, `Lxr::seq_setTrackLength`, `Lxr::uart_sendFrontpanelByte`, `Lxr::voiceControl_noteOff`.
    pub fn frontParser_handleSeqCC(&mut self) {
        todo!("port: Core/Src/Soundengine/MIDI/CortexParser.c:704")
    }
}
