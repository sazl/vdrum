//! Port of `Core/Inc/Soundengine/MIDI/CortexParser.h`, `Core/Src/Soundengine/MIDI/CortexParser.c`.
//!
//! Porting group: P3. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.

use crate::prelude::*;
use crate::midi::midi_parser::RECIP_127;

// ---- functions that use globals (methods on `Lxr`)
impl Lxr {
    /// C: `void frontParser_parseUartData(unsigned char data)`
    ///
    /// Source: `Core/Src/Soundengine/MIDI/CortexParser.c:99`. Reference build: out-of-line in flash. Group P3.
    pub fn frontParser_parseUartData(&mut self, data: u8) {
        if data & 0x80 != 0 {
            self.frontParser_rxCnt = 0;
            self.cortex_midiMsg.status = data;
            if data as i32 == SYSEX_START {
                self.frontParser_sysexActive = SYSEX_ACTIVE_MODE_NONE as u8;
                self.uart_clearFrontFifo();
                self.uart_sendFrontpanelSysExByte(SYSEX_START as u8);
            } else {
                self.frontParser_sysexActive = SYSEX_INACTIVE as u8;
            }
        } else if self.cortex_midiMsg.status as i32 == SYSEX_START {
            self.frontParser_handleSysexData(data);
        } else if self.frontParser_rxCnt == 0 {
            self.cortex_midiMsg.data1 = data;
            self.frontParser_rxCnt += 1;
        } else {
            self.cortex_midiMsg.data2 = data;
            self.frontParser_handleMidiMessage();
        }
    }

    /// C: `static void frontParser_handleSysexData(unsigned char data)`
    ///
    /// Source: `Core/Src/Soundengine/MIDI/CortexParser.c:144`. Reference build: inlined into its callers only. Group P3.
    pub fn frontParser_handleSysexData(&mut self, data: u8) {
        // The sysex modes SYSEX_REQUEST_STEP_DATA, SYSEX_RECEIVE_PAT_LEN_DATA and SYSEX_RECEIVE_STEP_DATA
        // carry sequencer steps and track lengths; VDrum has no sequencer, so their data is ignored.
        match self.frontParser_sysexActive as i32 {
            SYSEX_REQUEST_STEP_DATA | SYSEX_RECEIVE_PAT_LEN_DATA | SYSEX_RECEIVE_STEP_DATA => {}
            _ => {
                self.frontParser_sysexActive = data;
                self.frontParser_rxCnt = 0;
            }
        }
    }

    /// C: `static void frontParser_handleMidiMessage()`
    ///
    /// Source: `Core/Src/Soundengine/MIDI/CortexParser.c:283`. Reference build: inlined into its callers only. Group P3.
    pub fn frontParser_handleMidiMessage(&mut self) {
        match self.cortex_midiMsg.status as i32 {
            FRONT_SEQ_CC => self.frontParser_handleSeqCC(),
            SAMPLE_CC => {}
            MIDI_CC => {
                self.cortex_midiMsg.data1 = self.cortex_midiMsg.data1.wrapping_add(1);
                self.cortex_midiMsg.data1 &= 0x7f;
                // no automation step can be armed (`seq_isAutomationStepArmed`) and nothing is
                // recorded (`seq_recordAutomation`): VDrum has no sequencer
                let m = self.cortex_midiMsg;
                self.midiParser_ccHandler(m, 1, 0, 0.0);
            }
            FRONT_CC_2 => {
                let m = self.cortex_midiMsg;
                self.midiParser_ccHandler(m, 1, 0, 0.0);
            }
            FRONT_CC_LFO_TARGET => {
                let upper = self.cortex_midiMsg.data1;
                let lower = self.cortex_midiMsg.data2;
                let value = (((upper & 0x01) << 7) | lower) as u16;
                let lfoNr = (upper & 0xfe) >> 1;
                match lfoNr {
                    0 | 1 | 2 => {
                        let i = lfoNr as usize;
                        self.modNode_setDestination(move |s| &mut s.voiceArray[i].lfo.modTarget, value)
                    }
                    3 => self.modNode_setDestination(|s| &mut s.snareVoice.lfo.modTarget, value),
                    4 => self.modNode_setDestination(|s| &mut s.cymbalVoice.lfo.modTarget, value),
                    5 => self.modNode_setDestination(|s| &mut s.hatVoice.lfo.modTarget, value),
                    _ => {}
                }
            }
            // FRONT_SET_P1/P2_DEST, FRONT_SET_P1/P2_VAL and FRONT_ARM_AUTOMATION_STEP edit sequencer
            // steps; VDrum has no sequencer.
            FRONT_CC_VELO_TARGET => {
                let upper = self.cortex_midiMsg.data1;
                let lower = self.cortex_midiMsg.data2;
                let value = (((upper & 0x01) << 7) | lower) as u16;
                let velModNr = ((upper & 0xfe) >> 1) as usize;
                self.modNode_setDestination(move |s| &mut s.velocityModulators[velModNr], value);
            }
            VOICE_CC => {}
            FRONT_SET_BPM => {
                let bpm = self.cortex_midiMsg.data1 as u16 | ((self.cortex_midiMsg.data2 as u16) << 7);
                // 0 and 1 select the sequencer's external and MIDI clock (`seq_setExtSync`), which
                // VDrum does not have; another value is the tempo synced LFOs follow.
                if bpm > 1 {
                    self.seq_setBpm(bpm);
                }
            }
            _ => {}
        }
    }

    /// C: `static void frontParser_handleSeqCC()`
    ///
    /// Source: `Core/Src/Soundengine/MIDI/CortexParser.c:548`. Reference build: inlined into its callers only. Group P3.
    pub fn frontParser_handleSeqCC(&mut self) {
        let d2 = self.cortex_midiMsg.data2;
        // The step, Euclid, track, pattern, automation, roll, run/stop, mute and quantisation commands
        // of the sequencer are removed: VDrum has no sequencer.
        match self.cortex_midiMsg.data1 as i32 {
            FRONT_SEQ_POSX | FRONT_SEQ_POSY | FRONT_SEQ_FLUX | FRONT_SEQ_SOM_FREQ => {}
            FRONT_SEQ_MIDI_CHAN => {
                let voice = d2 >> 4;
                let channel = d2 & 0x0f;
                if voice < 7 && self.midi_MidiChannels[voice as usize] != channel {
                    self.voiceControl_noteOff(voice);
                }
                self.midi_MidiChannels[voice as usize] = channel;
            }
            FRONT_SEQ_SET_SHOWN_PATTERN => self.frontParser_shownPattern = d2,
            FRONT_SEQ_SET_ACTIVE_TRACK => self.frontParser_activeTrack = d2,
            FRONT_SEQ_MIDI_ROUTING => self.midiParser_setRouting(d2),
            FRONT_SEQ_MIDI_TX_FILTER | FRONT_SEQ_MIDI_RX_FILTER => {
                let is_tx = (self.cortex_midiMsg.data1 as i32 == FRONT_SEQ_MIDI_TX_FILTER) as u8;
                self.midiParser_setFilter(is_tx, d2)
            }
            FRONT_SEQ_TRIGGER_IN_PPQ => {
                self.trigger_prescalerClockInput = if d2 == 1 { PRE_24_PPQ_IN } else { PRE_4_PPQ_IN };
            }
            FRONT_SEQ_TRIGGER_OUT1_PPQ => {
                self.trigger_dividerClockOut1 = if d2 <= PRE_48_PPQ_OUT { d2 } else { PRE_4_PPQ_OUT };
            }
            FRONT_SEQ_TRIGGER_OUT2_PPQ => {}
            FRONT_SEQ_TRIGGER_GATE_MODE => {}
            _ => {}
        }
    }
}
