//! The parts of `Core/{Inc,Src}/Soundengine/Sequencer/sequencer` that the sound engine uses without a
//! sequencer:
//!
//! * the tempo that clock-synced LFOs follow (`seq_tempo`, `seq_setBpm`, `seq_getBpm`), read by
//!   `lfo_calcPhaseInc`;
//! * the track mutes (`seq_mutedTracks`, `seq_setMute`, `seq_isTrackMuted`): `midiParser_noteOn` drops
//!   MIDI notes of a muted voice, and the `CC2_MUTE_1..7` parameters (MIDI NRPN, or a velocity / LFO
//!   modulation destination) set them;
//! * `SEQ_DEFAULT_NOTE`.
//!
//! VDrum has no sequencer: the rest of the firmware's sequencer (`sequencer.c`, `clockSync.c`,
//! `EuklidGenerator.c`, `SeqStep.h`, and `automationNode.c` of its automation tracks) was removed on
//! 2026-11-14. Porting group: P3.

use crate::prelude::*;
use crate::midi::midi_voice_control::voice_bit;

/// `#define SEQ_DEFAULT_NOTE 60` (Core/Inc/Soundengine/Sequencer/sequencer.h): the note at which a voice
/// plays its tuning (oscillator pitch tracking and MIDI note triggers).
pub const SEQ_DEFAULT_NOTE: i32 = 60;

impl Lxr {
    /// C: `void seq_setBpm(uint16_t bpm)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:437`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_tempo`.
    /// Calls: `Lxr::lfo_recalcSync`.
    pub fn seq_setBpm(&mut self, bpm: u16) {
        self.seq_tempo = bpm;
        self.lfo_recalcSync();
    }

    /// C: `uint16_t seq_getBpm()`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:444`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_tempo`.
    pub fn seq_getBpm(&mut self) -> u16 {
        self.seq_tempo
    }

    /// C: `IN_ITCM_RAM void seq_setMute(uint8_t trackNr, uint8_t isMuted)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:2047`. Reference build: out-of-line in ITCM. Group P3.
    /// Globals: `midi_MidiChannels`, `seq_mutedTracks`.
    /// Calls: `Lxr::voiceControl_noteOff`.
    pub fn seq_setMute(&mut self, trackNr: u8, isMuted: u8) {
        if trackNr == 7 {
            // unmute all
            self.seq_mutedTracks = 0;
        } else if isMuted != 0 {
            // mute track
            self.seq_mutedTracks |= voice_bit(trackNr);
            // turn off the midi note that may be playing on that track (C passes the channel as the voice)
            let ch = self.midi_MidiChannels[trackNr as usize];
            self.voiceControl_noteOff(ch);
        } else {
            // unmute track
            self.seq_mutedTracks &= !voice_bit(trackNr);
        }
    }

    /// C: `uint8_t seq_isTrackMuted(uint8_t trackNr)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:2067`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_mutedTracks`.
    pub fn seq_isTrackMuted(&mut self, trackNr: u8) -> u8 {
        if self.seq_mutedTracks & voice_bit(trackNr) != 0 { 1 } else { 0 }
    }
}
