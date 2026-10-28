//! Port of `Core/Inc/Soundengine/MIDI/MidiVoiceControl.h`, `Core/Src/Soundengine/MIDI/MidiVoiceControl.c`.
//!
//! Porting group: P3. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;

// ---- functions that use globals (methods on `Lxr`)
impl Lxr {
    /// C: `void voiceControl_noteOn(uint8_t voice, uint8_t note, uint8_t vel)`
    ///
    /// Source: `Core/Src/Soundengine/MIDI/MidiVoiceControl.c:30`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `active_voices`, `midi_MidiChannels`.
    /// Calls: `Lxr::Cymbal_trigger`, `Lxr::Drum_trigger`, `Lxr::HiHat_trigger`, `Lxr::Snare_trigger`, `Lxr::seq_sendMidiNoteOn`, `Lxr::uart_sendFrontpanelByte`.
    pub fn voiceControl_noteOn(&mut self, voice: u8, note: u8, vel: u8) {
        todo!("port: Core/Src/Soundengine/MIDI/MidiVoiceControl.c:30")
    }

    /// C: `void voiceControl_noteOff(uint8_t voice)`
    ///
    /// Source: `Core/Src/Soundengine/MIDI/MidiVoiceControl.c:66`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `active_voices`, `midi_MidiChannels`.
    /// Calls: `Lxr::seq_midiNoteOff`.
    pub fn voiceControl_noteOff(&mut self, voice: u8) {
        todo!("port: Core/Src/Soundengine/MIDI/MidiVoiceControl.c:66")
    }

    /// C: `uint8_t voiceControl_isVoicePlaying(uint8_t voice)`
    ///
    /// Source: `Core/Src/Soundengine/MIDI/MidiVoiceControl.c:88`. Reference build: not linked (removed by --gc-sections). Group P3.
    /// Globals: `active_voices`.
    pub fn voiceControl_isVoicePlaying(&mut self, voice: u8) -> u8 {
        todo!("port: Core/Src/Soundengine/MIDI/MidiVoiceControl.c:88")
    }
}
