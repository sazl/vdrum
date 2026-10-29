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
        match self.frontParser_sysexActive as i32 {
            SYSEX_REQUEST_STEP_DATA => {
                self.frontParser_rxCnt = self.frontParser_rxCnt.wrapping_add(1);
                if self.frontParser_rxCnt == 1 {
                    self.frontParser_twoByteData = (data as u16) << 7;
                } else {
                    self.frontParser_twoByteData |= (data & 0x7f) as u16;
                    self.frontParser_rxCnt = 0;
                    let n = self.frontParser_twoByteData;
                    self.seq_sendStepInfoToFront(n);
                }
            }
            SYSEX_RECEIVE_PAT_LEN_DATA => {
                let currentPattern = (self.frontParser_sysexSeqStepNr / 7) as u8;
                let currentTrack = (self.frontParser_sysexSeqStepNr as u32).wrapping_sub(currentPattern as u32 * 7) as u8;
                self.seq_patternSet.length[currentPattern as usize][currentTrack as usize] = data;
                self.frontParser_sysexSeqStepNr = self.frontParser_sysexSeqStepNr.wrapping_add(1);
                self.frontParser_rxCnt = 0;
            }
            SYSEX_RECEIVE_STEP_DATA => {
                if self.frontParser_rxCnt < 7 {
                    self.frontParser_sysexBuffer[self.frontParser_rxCnt as usize] = data;
                    self.frontParser_rxCnt += 1;
                } else {
                    for i in 0..7u32 {
                        self.frontParser_sysexBuffer[i as usize] |= (((data as u32) & (1 << i)) << (7 - i)) as u8;
                    }
                    let absPat = (self.frontParser_sysexSeqStepNr / 128) as u8;
                    let currentTrack = absPat / 8;
                    let currentPattern = absPat.wrapping_sub(currentTrack.wrapping_mul(8));
                    let currentStep = (self.frontParser_sysexSeqStepNr as u32).wrapping_sub(absPat as u32 * 128) as u8;
                    let b = self.frontParser_sysexBuffer;
                    // C indexes unchecked: steps 64..127 land in the following track / pattern
                    let st = Step {
                        volume: b[0],
                        prob: b[1],
                        note: b[2],
                        param1Nr: b[3],
                        param1Val: b[4],
                        param2Nr: b[5],
                        param2Val: b[6],
                    };
                    self.seq_step_write(currentPattern, currentTrack, currentStep as i32, st);
                    self.frontParser_rxCnt = 0;
                    self.frontParser_sysexSeqStepNr = self.frontParser_sysexSeqStepNr.wrapping_add(1);
                }
            }
            _ => {
                self.frontParser_sysexActive = data;
                self.frontParser_sysexSeqStepNr = 0;
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
                if self.seq_isAutomationStepArmed() == 0 {
                    let m = self.cortex_midiMsg;
                    self.midiParser_ccHandler(m, 1, 0, 0.0);
                }
                let (t, d1, d2) = (self.frontParser_activeTrack, self.cortex_midiMsg.data1, self.cortex_midiMsg.data2);
                self.seq_recordAutomation(t, d1, d2);
            }
            FRONT_CC_2 => {
                if self.seq_isAutomationStepArmed() == 0 {
                    let m = self.cortex_midiMsg;
                    self.midiParser_ccHandler(m, 1, 0, 0.0);
                }
                let (t, d1, d2) = (self.frontParser_activeTrack, self.cortex_midiMsg.data1, self.cortex_midiMsg.data2);
                self.seq_recordAutomation(t, d1.wrapping_add(128), d2);
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
            FRONT_SET_P1_DEST | FRONT_SET_P2_DEST => {
                let hi = self.cortex_midiMsg.data1;
                let lo = self.cortex_midiMsg.data2;
                let mut val = (hi << 7) | lo;
                if val != 0 && val < 128 {
                    val += 1;
                }
                // C indexes unchecked (seq_selectedStep is 0..127): flat helpers
                let (pat, track, step) =
                    (self.frontParser_shownPattern, self.frontParser_activeTrack, self.seq_selectedStep as i32);
                let mut st = self.seq_step_read(pat, track, step);
                if self.cortex_midiMsg.status as i32 == FRONT_SET_P1_DEST {
                    st.param1Nr = val;
                } else {
                    st.param2Nr = val;
                }
                self.seq_step_write(pat, track, step, st);
            }
            FRONT_SET_P1_VAL | FRONT_SET_P2_VAL => {
                let stepNr = self.cortex_midiMsg.data1;
                let value = self.cortex_midiMsg.data2;
                let (pat, track) = (self.frontParser_shownPattern, self.frontParser_activeTrack);
                let mut st = self.seq_step_read(pat, track, stepNr as i32);
                if self.cortex_midiMsg.status as i32 == FRONT_SET_P1_VAL {
                    st.param1Val = value;
                } else {
                    st.param2Val = value;
                }
                self.seq_step_write(pat, track, stepNr as i32, st);
            }
            FRONT_ARM_AUTOMATION_STEP => {
                let stepNr = self.cortex_midiMsg.data1;
                let onOff = self.cortex_midiMsg.data2 & 0x40;
                let trackNr = self.cortex_midiMsg.data2 & 0x3f;
                self.seq_armAutomationStep(stepNr, trackNr, onOff);
            }
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
                if bpm == 0 {
                    self.seq_setExtSync(1);
                } else if bpm == 1 {
                    self.seq_setExtSync(2);
                } else {
                    self.seq_setBpm(bpm);
                    self.seq_setExtSync(0);
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
        let sp = self.frontParser_shownPattern as usize;
        let at = self.frontParser_activeTrack as usize;
        let ast = self.frontParser_activeStep as usize;
        match self.cortex_midiMsg.data1 as i32 {
            FRONT_SEQ_REC_ON_OFF => self.seq_setRecordingMode(d2),
            FRONT_SEQ_ERASE_ON_OFF => self.seq_setErasingMode(d2),
            // [sp][at][ast] with C (unchecked, flat) indexing: the active step can be 0..127
            FRONT_SEQ_NOTE => self.edit_step(sp, at, ast, |st| st.note = d2),
            FRONT_SEQ_VOLUME => self.edit_step(sp, at, ast, |st| {
                st.volume &= !0x7f;
                st.volume |= d2 & 0x7f;
            }),
            FRONT_SEQ_FLAM => self.edit_timing(sp, at, ast, |tm| tm.set_flam(d2)),
            FRONT_SEQ_SHIFT => self.edit_timing(sp, at, ast, |tm| tm.set_shift(d2)),
            FRONT_SEQ_PROB => self.edit_step(sp, at, ast, |st| st.prob = d2),
            FRONT_SEQ_EUKLID_LENGTH => {
                let length = (d2 >> 3) + 1;
                self.euklid_setLength(at as u8, length);
            }
            FRONT_SEQ_EUKLID_STEPS => {
                let steps = (d2 >> 3) + 1;
                let pattern = d2 & 0x7;
                self.euklid_setSteps(at as u8, steps, pattern);
                self.led_updateSeqLeds();
            }
            FRONT_SEQ_EUKLID_ROTATION => {
                let rotation = d2 >> 3;
                let pattern = d2 & 0x7;
                self.euklid_setRotation(at as u8, rotation, pattern);
                self.led_updateSeqLeds();
            }
            FRONT_SEQ_CLEAR_TRACK => self.seq_clearTrack(d2, sp as u8),
            FRONT_SEQ_POSX | FRONT_SEQ_POSY | FRONT_SEQ_FLUX | FRONT_SEQ_SOM_FREQ => {}
            FRONT_SEQ_MIDI_CHAN => {
                let voice = d2 >> 4;
                let channel = d2 & 0x0f;
                if voice < 7 && self.midi_MidiChannels[voice as usize] != channel {
                    self.voiceControl_noteOff(voice);
                }
                self.midi_MidiChannels[voice as usize] = channel;
            }
            FRONT_SEQ_CLEAR_AUTOM => {
                let voice = d2 >> 4;
                let automTrack = d2 & 0x0f;
                self.seq_clearAutomation(voice, sp as u8, automTrack);
            }
            FRONT_SEQ_COPY_TRACK => self.seq_copyTrack(d2 >> 4, d2 & 0xf, sp as u8),
            FRONT_SEQ_COPY_PATTERN => self.seq_copyPattern(d2 >> 4, d2 & 0xf),
            FRONT_SEQ_TRACK_LENGTH => self.seq_setTrackLength(at as u8, d2),
            FRONT_SEQ_SHUFFLE => self.seq_setShuffle((d2 as i32 as f32) * RECIP_127),
            FRONT_SEQ_SELECT_ACTIVE_STEP => self.seq_selectedStep = d2,
            FRONT_SEQ_SET_AUTOM_TRACK => self.seq_setActiveAutomationTrack(d2),
            FRONT_SEQ_SET_QUANT => self.seq_setQuantisation(d2),
            FRONT_SEQ_REQUEST_EUKLID_PARAMS => {
                self.activeParameters[PAR_EUKLID_LENGTH as usize] = self.euklid_getLength(d2);
                self.activeParameters[PAR_EUKLID_STEPS as usize] = self.euklid_getSteps(d2);
                self.activeParameters[PAR_EUKLID_ROTATION as usize] = self.euklid_getRotation(d2);
                self.menu_repaint();
            }
            FRONT_SEQ_SET_SHOWN_PATTERN => self.frontParser_shownPattern = d2,
            FRONT_SEQ_SET_ACTIVE_TRACK => self.frontParser_activeTrack = d2,
            FRONT_SEQ_REQUEST_STEP_PARAMS => {
                // the replies go to frontPanel_parseData (uart_sendFrontpanelByte), which updates activeParameters
                // the requested step (PAR_ACTIVE_STEP, 0..127) is not range checked: flat indexing
                let st = self.req_step(d2);
                let seq = FRONT_SEQ_CC as u8;
                self.uart_sendFrontpanelByte(seq);
                self.uart_sendFrontpanelByte(FRONT_SEQ_VOLUME as u8);
                self.uart_sendFrontpanelByte(st.volume & STEP_VOLUME_MASK as u8);
                self.uart_sendFrontpanelByte(seq);
                self.uart_sendFrontpanelByte(FRONT_SEQ_NOTE as u8);
                let st = self.req_step(d2);
                self.uart_sendFrontpanelByte(st.note);
                self.uart_sendFrontpanelByte(seq);
                self.uart_sendFrontpanelByte(FRONT_SEQ_PROB as u8);
                let st = self.req_step(d2);
                self.uart_sendFrontpanelByte(st.prob);
                self.uart_sendFrontpanelByte(seq);
                self.uart_sendFrontpanelByte(FRONT_SEQ_FLAM as u8);
                let tm2 = self.req_timing(d2);
                self.uart_sendFrontpanelByte(tm2.flam());
                self.uart_sendFrontpanelByte(seq);
                self.uart_sendFrontpanelByte(FRONT_SEQ_SHIFT as u8);
                let tm3 = self.req_timing(d2);
                self.uart_sendFrontpanelByte(tm3.shift());
                let mut dest = self.req_step(d2).param1Nr;
                if dest < 128 && dest != 0 {
                    dest -= 1;
                }
                self.uart_sendFrontpanelByte(FRONT_SET_P1_DEST as u8);
                self.uart_sendFrontpanelByte(dest >> 7);
                self.uart_sendFrontpanelByte(dest & 0x7f);
                let val = self.req_step(d2).param1Val;
                self.uart_sendFrontpanelByte(FRONT_SET_P1_VAL as u8);
                self.uart_sendFrontpanelByte(val >> 7);
                self.uart_sendFrontpanelByte(val & 0x7f);
                let mut dest = self.req_step(d2).param2Nr;
                if dest < 128 && dest != 0 {
                    dest -= 1;
                }
                self.uart_sendFrontpanelByte(FRONT_SET_P2_DEST as u8);
                self.uart_sendFrontpanelByte(dest >> 7);
                self.uart_sendFrontpanelByte(dest & 0x7f);
                let val = self.req_step(d2).param2Val;
                self.uart_sendFrontpanelByte(FRONT_SET_P2_VAL as u8);
                self.uart_sendFrontpanelByte(val >> 7);
                self.uart_sendFrontpanelByte(val & 0x7f);
                self.frontParser_activeStep = self.cortex_midiMsg.data2;
            }
            FRONT_SEQ_ROLL_RATE => self.seq_setRollRate(d2),
            FRONT_SEQ_ROLL_ON_OFF => self.seq_setRoll(d2 & 0xf, d2 >> 4),
            FRONT_SEQ_CHANGE_PAT => self.seq_setNextPattern(d2),
            FRONT_SEQ_RUN_STOP => {
                if d2 == 0 {
                    self.seq_resetPosition();
                }
                let d2 = self.cortex_midiMsg.data2;
                self.seq_setRunning(d2);
            }
            FRONT_SEQ_MUTE_TRACK => self.seq_setMute(d2, 1),
            FRONT_SEQ_UNMUTE_TRACK => self.seq_setMute(d2, 0),
            FRONT_SEQ_MIDI_ROUTING => self.midiParser_setRouting(d2),
            FRONT_SEQ_MIDI_TX_FILTER | FRONT_SEQ_MIDI_RX_FILTER => {
                let is_tx = (self.cortex_midiMsg.data1 as i32 == FRONT_SEQ_MIDI_TX_FILTER) as u8;
                self.midiParser_setFilter(is_tx, d2)
            }
            FRONT_SEQ_BAR_RESET_MODE => self.seq_resetBarOnPatternChange = d2,
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

    /// `seq_patternSet.seq_subStepPattern[frontParser_shownPattern][frontParser_activeTrack][step]`, re-read
    /// (the firmware re-reads the globals after each `uart_sendFrontpanelByte`).
    fn req_step(&self, step: u8) -> Step {
        self.seq_step_read(self.frontParser_shownPattern, self.frontParser_activeTrack, step as i32)
    }
    fn req_timing(&self, step: u8) -> StepTiming {
        self.seq_stepTiming_read(self.frontParser_shownPattern, self.frontParser_activeTrack, step as i32)
    }
    /// Read-modify-write of `seq_subStepPattern[sp][at][st]` with C (flat) indexing.
    fn edit_step(&mut self, sp: usize, at: usize, st: usize, f: impl FnOnce(&mut Step)) {
        let mut s = self.seq_step_read(sp as u8, at as u8, st as i32);
        f(&mut s);
        self.seq_step_write(sp as u8, at as u8, st as i32, s);
    }
    /// Read-modify-write of `seq_stepTiming[sp][at][st]` with C (flat) indexing.
    fn edit_timing(&mut self, sp: usize, at: usize, st: usize, f: impl FnOnce(&mut StepTiming)) {
        let mut t = self.seq_stepTiming_read(sp as u8, at as u8, st as i32);
        f(&mut t);
        self.seq_stepTiming_write(sp as u8, at as u8, st as i32, t);
    }
}
