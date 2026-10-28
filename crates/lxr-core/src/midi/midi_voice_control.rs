//! Port of `Core/Inc/Soundengine/MIDI/MidiVoiceControl.h`, `Core/Src/Soundengine/MIDI/MidiVoiceControl.c`.
//!
//! Porting group: P3. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.

use crate::prelude::*;

/// `1 << voice` as the Cortex-M7 `lsl.w` computes it (a shift by 32 or more gives 0), truncated to u8.
#[inline(always)]
fn voice_bit(voice: u8) -> u8 {
    if voice < 32 { (1u32 << voice) as u8 } else { 0 }
}

// ---- functions that use globals (methods on `Lxr`)
impl Lxr {
    /// C: `void voiceControl_noteOn(uint8_t voice, uint8_t note, uint8_t vel)`
    ///
    /// Source: `Core/Src/Soundengine/MIDI/MidiVoiceControl.c:30`. Group P3.
    ///
    /// Ported from the SHIPPED v1.71 binary (0x08016f48), which differs from the C source: the shipped
    /// function does not contain the `if (active_voices & (1<<voice)) seq_sendMidiNoteOn(...)` block,
    /// so no MIDI note-on is sent (and `midi_MidiChannels` is not read). Everything else (setting the
    /// `active_voices` bit, the per-voice trigger, the three front panel bytes) is as in the C source.
    pub fn voiceControl_noteOn(&mut self, voice: u8, note: u8, vel: u8) {
        self.active_voices |= voice_bit(voice);

        if voice < 3 {
            self.Drum_trigger(voice, vel, note);
        } else if voice < 4 {
            self.Snare_trigger(vel, note);
        } else if voice < 5 {
            self.Cymbal_trigger(vel, note);
        } else {
            self.HiHat_trigger(vel, voice.wrapping_sub(5), note);
        }

        // Send to front panel so it can pulse the LED
        self.uart_sendFrontpanelByte(NOTE_ON as u8);
        self.uart_sendFrontpanelByte(voice);
        self.uart_sendFrontpanelByte(0);
    }

    /// C: `void voiceControl_noteOff(uint8_t voice)`
    ///
    /// Source: `Core/Src/Soundengine/MIDI/MidiVoiceControl.c:66`. Group P3.
    pub fn voiceControl_noteOff(&mut self, voice: u8) {
        if voice == 0xff {
            self.active_voices = 0;
            self.seq_midiNoteOff(0xff);
            return;
        }

        // only set voice inactive and send MIDI off when voice is currently playing
        if self.active_voices & voice_bit(voice) != 0 {
            self.active_voices &= !voice_bit(voice);
            let midiChan = self.midi_MidiChannels[voice as usize];
            self.seq_midiNoteOff(midiChan);
        }
    }

    /// C: `uint8_t voiceControl_isVoicePlaying(uint8_t voice)`
    ///
    /// Source: `Core/Src/Soundengine/MIDI/MidiVoiceControl.c:88`. Group P3.
    pub fn voiceControl_isVoicePlaying(&mut self, voice: u8) -> u8 {
        self.active_voices & voice_bit(voice)
    }
}
