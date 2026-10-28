//! Port of `Core/Inc/frontPanelParser_ui.h`, `Core/Src/frontPanelParser.c`.
//!
//! Porting group: P3. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//!
//! In this firmware the front panel (UI) and the sound engine run on one MCU: `uart_sendFrontpanelByte` calls
//! `frontPanel_parseData` directly, and `frontPanel_sendData` feeds `frontParser_parseUartData` (CortexParser.c).
//! `frontPanel_parseData` therefore changes engine state (`activeParameters`, kit loads on bank change) and is
//! ported here together with its helpers `frontPanel_ccHandler` and `frontParser_parseNrpn`. Their pure UI
//! side effects (LEDs, LCD repaint, `buttonHandler_*` state) are not modelled (see `crate::external`).
//!
//! The UI-side protocol constants (`SEQ_CC`, `SEQ_*`, `SET_P1_DEST`, ...) have the same values as the engine
//! side `FRONT_*` constants in `crate::midi::midi_messages`, which are used here.

use crate::prelude::*;

// ---- #define constants
/// `#define CC_2 0xd6` (Core/Inc/frontPanelParser_ui.h:48)
pub const CC_2: i32 = 214;
/// `#define CC_LFO_TARGET 0xd7` (Core/Inc/frontPanelParser_ui.h:49)
pub const CC_LFO_TARGET: i32 = 215;
/// `#define CC_VELO_TARGET 0xd8` (Core/Inc/frontPanelParser_ui.h:50)
pub const CC_VELO_TARGET: i32 = 216;
/// `#define NRPN_MUTE_1 1000` (Core/Src/frontPanelParser.c:44)
pub const NRPN_MUTE_1: i32 = 1000;
/// `#define NRPN_MUTE_7 1006` (Core/Src/frontPanelParser.c:45)
pub const NRPN_MUTE_7: i32 = 1006;
/// `#define LED_CC 0xd1` (Core/Inc/frontPanelParser_ui.h:43)
pub const LED_CC: i32 = 0xd1;
/// `#define SEQ_CC 0xd2` (Core/Inc/frontPanelParser_ui.h:44)
pub const SEQ_CC: i32 = 0xd2;
/// `#define SET_BPM 0xd5` (Core/Inc/frontPanelParser_ui.h:47)
pub const SET_BPM: i32 = 0xd5;
/// `#define SET_P1_DEST 0xda` (Core/Inc/frontPanelParser_ui.h:52)
pub const SET_P1_DEST: i32 = 0xda;
/// `#define SET_P2_DEST 0xdb` (Core/Inc/frontPanelParser_ui.h:53)
pub const SET_P2_DEST: i32 = 0xdb;
/// `#define SET_P1_VAL 0xdc` (Core/Inc/frontPanelParser_ui.h:54)
pub const SET_P1_VAL: i32 = 0xdc;
/// `#define SET_P2_VAL 0xdd` (Core/Inc/frontPanelParser_ui.h:55)
pub const SET_P2_VAL: i32 = 0xdd;
/// `#define VOICE_DECIMATION 0x03` (Core/Inc/frontPanelParser_ui.h)
pub const VOICE_DECIMATION: i32 = 0x03;

impl Lxr {
    /// `paramToModTarget[i]` with the firmware's out-of-range reads (indices 247..=255 read the globals
    /// that follow the array: `copyClear_*` (not modelled, 0), padding (0), `activeParameters[0..3]`).
    /// Non-C helper, group P3.
    pub fn paramToModTarget_read(&self, i: u8) -> u8 {
        if (i as usize) < self.paramToModTarget.len() {
            self.paramToModTarget[i as usize]
        } else {
            self.global_get_byte(crate::memmap::ADDR_paramToModTarget + i as u32).unwrap_or(0)
        }
    }

    /// C: `void frontParser_parseNrpn(uint8_t value)` (Core/Src/frontPanelParser.c:47). Group P3 (UI side
    /// file reached from `uart_sendFrontpanelByte`). `buttonHandler_muteVoice` (NRPN 1000..1006) only changes
    /// UI state and LEDs and is not modelled.
    pub fn frontParser_parseNrpn(&mut self, value: u8) {
        let paramNr: u16 = self.frontParser_nrpnNr.wrapping_add(128);
        if paramNr < NUM_PARAMS {
            self.activeParameters[paramNr as usize] = value;
        }
        if paramNr >= PAR_TARGET_LFO1 && paramNr <= PAR_TARGET_LFO6 {
            let mut lfoNr = (paramNr - PAR_TARGET_LFO1) as u8;
            if lfoNr > 5 {
                lfoNr = 5;
            }
            self.activeParameters[paramNr as usize] = self.paramToModTarget_read(value);
            let upper = ((value & 0x80) >> 7) | ((lfoNr & 0x3f) << 1);
            let lower = value & 0x7f;
            self.frontPanel_sendData(CC_LFO_TARGET as u8, upper, lower);
        } else if paramNr >= PAR_VEL_DEST_1 && paramNr <= PAR_VEL_DEST_6 {
            self.activeParameters[paramNr as usize] = self.paramToModTarget_read(value);
            let upper = (((value & 0x80) >> 7) as u16 | (((paramNr - PAR_VEL_DEST_1) & 0x3f) << 1)) as u8;
            let lower = value & 0x7f;
            self.frontPanel_sendData(CC_VELO_TARGET as u8, upper, lower);
        } else if self.frontParser_nrpnNr as i32 >= NRPN_MUTE_1 && self.frontParser_nrpnNr as i32 <= NRPN_MUTE_7 {
            // buttonHandler_muteVoice(voice, onOff): UI state only
        }
    }

    /// C: `void frontPanel_ccHandler()` (Core/Src/frontPanelParser.c:84). Group P3.
    pub fn frontPanel_ccHandler(&mut self) {
        let parNr = self.frontParser_midiMsg.data1.wrapping_sub(1);
        if parNr == 255 {
            let d2 = self.frontParser_midiMsg.data2;
            self.menu_selectedPresetNr[0] = d2;
            self.preset_loadDrumsetRam(d2, 0, 0);
            self.preset_touchTimer(PRESET_KIT_TIMER);
            self.menu_repaint();
            return;
        } else if parNr == NRPN_DATA_ENTRY_COARSE - 1 {
            let d2 = self.frontParser_midiMsg.data2;
            self.frontParser_parseNrpn(d2);
        } else if parNr == NRPN_FINE - 1 {
            self.frontParser_nrpnNr &= !0x7f;
            self.frontParser_nrpnNr |= self.frontParser_midiMsg.data2 as u16;
        } else if parNr == NRPN_COARSE - 1 {
            self.frontParser_nrpnNr &= 0x7f;
            self.frontParser_nrpnNr |= (self.frontParser_midiMsg.data2 as u16) << 7;
        }
        self.activeParameters[parNr as usize] = self.frontParser_midiMsg.data2;
        self.menu_repaint();
    }

    /// C: `void frontPanel_parseData(uint8_t data)` (Core/Src/frontPanelParser.c:117). Group P3.
    /// LED and LCD updates and `buttonHandler_setRunStopState` / `menu_setNumSamples` (UI state only) are
    /// not modelled.
    pub fn frontPanel_parseData(&mut self, data: u8) {
        if data & 0x80 != 0 {
            self.frontPanelParser_rxCnt = 0;
            self.frontParser_midiMsg.status = data;
            return;
        }
        if self.frontParser_midiMsg.status as i32 == SYSEX_START {
            if self.frontPanel_sysexMode as i32 == SYSEX_REQUEST_STEP_DATA {
                if self.frontPanelParser_rxCnt < 7 {
                    self.frontPanelParser_sysexBuffer[self.frontPanelParser_rxCnt as usize] = data;
                    self.frontPanelParser_rxCnt += 1;
                } else {
                    for i in 0..7u32 {
                        let add = (((data as u32) & (1 << i)) << (7 - i)) as u8;
                        self.frontPanelParser_sysexBuffer[i as usize] |= add;
                    }
                    let b = self.frontPanelParser_sysexBuffer;
                    self.frontParser_stepData.volume = b[0];
                    self.frontParser_stepData.prob = b[1];
                    self.frontParser_stepData.note = b[2];
                    self.frontParser_stepData.param1Nr = b[3];
                    self.frontParser_stepData.param1Val = b[4];
                    self.frontParser_stepData.param2Nr = b[5];
                    self.frontParser_stepData.param2Val = b[6];
                    self.frontParser_newSeqDataAvailable = 1;
                    self.frontPanelParser_rxCnt = 0;
                }
            }
        } else if self.frontPanelParser_rxCnt == 0 {
            self.frontParser_midiMsg.data1 = data;
            self.frontPanelParser_rxCnt += 1;
        } else {
            self.frontParser_midiMsg.data2 = data;
            self.frontPanelParser_rxCnt = 0;
            let m = self.frontParser_midiMsg;
            if (m.status & 0xF0) as i32 == MIDI_CC {
                self.frontPanel_ccHandler();
            } else if m.status as i32 == SET_P1_DEST || m.status as i32 == SET_P2_DEST {
                let mut dst = ((m.data1 as u32) << 7 | m.data2 as u32) as u8;
                if dst == 0xFF {
                    dst = 0;
                }
                let v = self.paramToModTarget_read(dst);
                if m.status as i32 == SET_P1_DEST {
                    self.activeParameters[PAR_P1_DEST as usize] = v;
                } else {
                    self.activeParameters[PAR_P2_DEST as usize] = v;
                }
                self.menu_repaintAll();
            } else if m.status as i32 == SET_P1_VAL {
                self.activeParameters[PAR_P1_VAL as usize] = ((m.data1 as u32) << 7 | m.data2 as u32) as u8;
                self.menu_repaintAll();
            } else if m.status as i32 == SET_P2_VAL {
                self.activeParameters[PAR_P2_VAL as usize] = ((m.data1 as u32) << 7 | m.data2 as u32) as u8;
                self.menu_repaintAll();
            } else if m.status as i32 == SEQ_CC {
                match m.data1 as i32 {
                    FRONT_SEQ_TRACK_LENGTH => {
                        self.activeParameters[PAR_TRACK_LENGTH as usize] = m.data2;
                        self.menu_repaint();
                    }
                    FRONT_SEQ_VOLUME => {
                        self.activeParameters[PAR_STEP_VOLUME as usize] = m.data2;
                        self.menu_repaintAll();
                    }
                    FRONT_SEQ_PROB => {
                        self.activeParameters[PAR_STEP_PROB as usize] = m.data2;
                        self.menu_repaintAll();
                    }
                    FRONT_SEQ_NOTE => {
                        self.activeParameters[PAR_STEP_NOTE as usize] = m.data2;
                        self.menu_repaintAll();
                    }
                    FRONT_SEQ_FLAM => {
                        self.activeParameters[PAR_STEP_FLAM as usize] = m.data2;
                        self.menu_repaintAll();
                    }
                    FRONT_SEQ_SHIFT => {
                        self.activeParameters[PAR_STEP_SHIFT as usize] = m.data2;
                        self.menu_repaintAll();
                    }
                    FRONT_SEQ_CHANGE_PAT => {
                        self.menu_setShownPattern(m.data2);
                        self.led_clearSequencerLeds();
                        self.led_updateSeqLeds();
                        // re-read: menu_setShownPattern may have re-entered this parser (it does not write
                        // frontParser_midiMsg, but stay faithful to the volatile load)
                        self.menu_playedPattern = self.frontParser_midiMsg.data2;
                        if self.buttonHandler_getMode() == SELECT_MODE_PATTERN {
                            self.led_clearSequencerLeds();
                            self.led_clearAllBlinkLeds();
                            self.led_initPatternSelectLeds();
                        }
                    }
                    FRONT_SEQ_RUN_STOP => {
                        // buttonHandler_setRunStopState: UI state only
                    }
                    _ => {}
                }
            }
            // SAMPLE_CC (menu_setNumSamples), LED_CC and NOTE_ON (LED pulses): UI only
        }
    }

    /// C: `void frontPanel_sendData(uint8_t status, uint8_t data1, uint8_t data2)`
    ///
    /// Source: `Core/Src/frontPanelParser.c:617`. Reference build: out-of-line in flash. Group P3.
    /// Calls: `Lxr::frontParser_parseUartData`.
    pub fn frontPanel_sendData(&mut self, status: u8, data1: u8, data2: u8) {
        self.frontParser_parseUartData(status);
        self.frontParser_parseUartData(data1);
        self.frontParser_parseUartData(data2);
    }
}
