// SPDX-License-Identifier: GPL-3.0-only
//! The interface between the plugin and a sound engine, and a stub engine for testing the plugin
//! without the real port.
//!
//! The engine runs the firmware's audio interrupt: one [`Engine::tick`] renders 64 stereo frames
//! (`OUTPUT_DMA_SIZE`) at 44100 Hz for each of the two DACs, as signed 16 bit, L/R interleaved.

use lxr_params::kit::{Kit, KIT_PARAMS};
use lxr_params::{par, GLOBAL_PARAMS, NUM_PARAMS};

/// Frames per engine tick.
pub const TICK_FRAMES: usize = 64;
/// The engine's nominal sample rate (pass-through at this host rate).
pub const ENGINE_RATE: u32 = 44100;
/// The hardware's real sample clock (the engine's coefficients use `REAL_FS` = 44140).
pub const HARDWARE_RATE: f64 = 44140.625;

/// One tick of output: 64 stereo frames per DAC, L/R interleaved.
#[derive(Clone, Copy)]
pub struct Block {
    /// DAC 1 (main output).
    pub dac1: [i16; 128],
    /// DAC 2 (aux output).
    pub dac2: [i16; 128],
}

impl Default for Block {
    fn default() -> Self {
        Block { dac1: [0; 128], dac2: [0; 128] }
    }
}

/// A sound engine. `lxr-core` (the port) and `lxr-emu` (the firmware in an emulator) implement this
/// contract; see `docs/ARCHITECTURE.md`.
pub trait Engine: Send {
    /// Power-on state with the init kit loaded. May allocate.
    fn new() -> Box<Self>;
    /// Feed one byte of DIN MIDI to the firmware's MIDI parser.
    fn midi_byte(&mut self, b: u8);
    /// Render one tick (64 frames per DAC). Must not allocate or lock.
    fn tick(&mut self) -> Block;
    /// Set a parameter (`ParamEnums` index) as the front panel does (`activeParameters` semantics).
    fn set_parameter(&mut self, param: u16, value: u16);
    /// The current value of a parameter (`ParamEnums` index).
    fn get_parameter(&self, param: u16) -> u16;
    /// Load a kit from `.snd` file bytes (8 byte name + parameter bytes).
    fn load_kit(&mut self, kit: &[u8]);
    /// The current kit as `.snd` file bytes.
    fn save_kit(&self) -> Vec<u8>;
}

/// A stand-in engine that reacts like the firmware at the interface level (parameter storage, kit
/// files, CC/NRPN parameter mapping, note triggers by channel/note) and renders a simple test sound:
/// a decaying sine per triggered track on DAC 1, and the same at half level on DAC 2. It is not the
/// LXR-02 sound engine.
pub struct StubEngine {
    params: [u16; NUM_PARAMS as usize],
    name: [u8; 8],
    // MIDI parser state
    status: u8,
    data: [u8; 2],
    have: usize,
    nrpn_msb: u8,
    nrpn_lsb: u8,
    // test sound: one voice per track (7 tracks)
    phase: [f32; 7],
    freq: [f32; 7],
    amp: [f32; 7],
}

/// MIDI channel and note parameters of the 7 tracks.
const TRACK_CHANNEL: [u16; 7] = [
    par::MIDI_CHAN_1,
    par::MIDI_CHAN_2,
    par::MIDI_CHAN_3,
    par::MIDI_CHAN_4,
    par::MIDI_CHAN_5,
    par::MIDI_CHAN_6,
    par::MIDI_CHAN_7,
];
const TRACK_NOTE: [u16; 7] = [
    par::MIDI_NOTE1,
    par::MIDI_NOTE2,
    par::MIDI_NOTE3,
    par::MIDI_NOTE4,
    par::MIDI_NOTE5,
    par::MIDI_NOTE6,
    par::MIDI_NOTE7,
];
const TRACK_COARSE: [u16; 7] = [
    par::COARSE1,
    par::COARSE2,
    par::COARSE3,
    par::COARSE4,
    par::COARSE5,
    par::COARSE6,
    par::COARSE6,
];

/// The tracks (bit 0..6) a note-on triggers given the track MIDI channel (1..=16) and note
/// (0 = any) settings, following `midiParser_parseMidiMessage()` (`MIDI_NOTE_ON`).
pub fn tracks_for_note(channel: u8, note: u8, get: impl Fn(u16) -> u16) -> u8 {
    let mut mask = 0u8;
    for t in 0..7 {
        let ch = get(TRACK_CHANNEL[t]);
        if ch == 0 || (ch - 1) as u8 != channel {
            continue;
        }
        let n = get(TRACK_NOTE[t]);
        if n == 0 || n as u8 == note {
            mask |= 1 << t;
        }
    }
    mask
}

/// MIDI channel (0-based) and note a pad of track `t` (0..=6) sends to trigger it.
pub fn track_trigger_note(t: usize, get: impl Fn(u16) -> u16) -> (u8, u8) {
    let ch = get(TRACK_CHANNEL[t]).clamp(1, 16) as u8 - 1;
    let n = get(TRACK_NOTE[t]) as u8;
    // A note setting of 0 means "any note"; the firmware then plays SEQ_DEFAULT_NOTE (60).
    (ch, if n == 0 { 60 } else { n.min(127) })
}

#[allow(clippy::needless_range_loop)]
impl StubEngine {
    fn handle_message(&mut self, status: u8, d0: u8, d1: u8) {
        let ch = status & 0x0f;
        match status & 0xf0 {
            0x90 if d1 > 0 => {
                let params = &self.params;
                let mask = tracks_for_note(ch, d0, |p| params[p as usize]);
                for t in 0..7 {
                    if mask & (1 << t) != 0 {
                        let coarse = self.params[TRACK_COARSE[t] as usize] as f32;
                        self.freq[t] = 40.0 * 2f32.powf(coarse / 24.0);
                        self.amp[t] = d1 as f32 / 127.0;
                        self.phase[t] = 0.0;
                    }
                }
            }
            0xb0 => match d0 {
                99 => self.nrpn_msb = d1,
                98 => self.nrpn_lsb = d1,
                6 => {
                    let p = 128 + (((self.nrpn_msb as u16) << 7) | self.nrpn_lsb as u16);
                    if p < KIT_PARAMS as u16 {
                        let info = &lxr_params::PARAMS[p as usize];
                        let v = if info.midi.front_translates_nrpn {
                            lxr_params::param_to_mod_target(d1 as u16) as u16
                        } else {
                            d1 as u16
                        };
                        self.set_parameter(p, v);
                    }
                }
                cc @ 1..=127 => {
                    let p = cc as u16 - 1;
                    if let Some(info) = lxr_params::info(p) {
                        if info.midi.cc == Some(cc) && info.midi.engine_handles {
                            self.set_parameter(p, d1 as u16);
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        }
    }
}

#[allow(clippy::needless_range_loop)]
impl Engine for StubEngine {
    fn new() -> Box<Self> {
        let mut e = Box::new(StubEngine {
            params: [0; NUM_PARAMS as usize],
            name: [0; 8],
            status: 0,
            data: [0; 2],
            have: 0,
            nrpn_msb: 0,
            nrpn_lsb: 0,
            phase: [0.0; 7],
            freq: [0.0; 7],
            amp: [0.0; 7],
        });
        for g in GLOBAL_PARAMS.iter() {
            e.params[g.index as usize] = g.default;
        }
        e.load_kit(&Kit::init().to_file_bytes());
        e
    }

    fn midi_byte(&mut self, b: u8) {
        if b >= 0xf8 {
            return; // realtime
        }
        if b & 0x80 != 0 {
            self.status = if b < 0xf0 { b } else { 0 };
            self.have = 0;
            return;
        }
        if self.status == 0 {
            return;
        }
        self.data[self.have] = b;
        self.have += 1;
        let needed = match self.status & 0xf0 {
            0xc0 | 0xd0 => 1,
            _ => 2,
        };
        if self.have == needed {
            self.have = 0; // running status
            let (s, d0, d1) = (self.status, self.data[0], self.data[1]);
            self.handle_message(s, d0, d1);
        }
    }

    fn tick(&mut self) -> Block {
        let mut block = Block::default();
        let k = 2.0 * std::f32::consts::PI / ENGINE_RATE as f32;
        for i in 0..TICK_FRAMES {
            let mut s = 0.0f32;
            for t in 0..7 {
                if self.amp[t] > 1e-4 {
                    s += self.amp[t] * (self.phase[t]).sin();
                    self.phase[t] = (self.phase[t] + k * self.freq[t]) % (2.0 * std::f32::consts::PI);
                    self.amp[t] *= 0.9997;
                }
            }
            let v = (s * 0.3 * 32767.0).clamp(-32768.0, 32767.0) as i16;
            block.dac1[2 * i] = v;
            block.dac1[2 * i + 1] = v;
            block.dac2[2 * i] = v / 2;
            block.dac2[2 * i + 1] = v / 2;
        }
        block
    }

    fn set_parameter(&mut self, param: u16, value: u16) {
        if let Some(p) = self.params.get_mut(param as usize) {
            *p = value;
        }
    }

    fn get_parameter(&self, param: u16) -> u16 {
        self.params.get(param as usize).copied().unwrap_or(0)
    }

    fn load_kit(&mut self, kit: &[u8]) {
        if let Ok(mut k) = Kit::from_file_bytes(kit) {
            k.normalize_lfo_voices();
            self.name = k.name;
            for (i, v) in k.values.iter().enumerate() {
                self.params[i] = *v as u16;
            }
        }
    }

    fn save_kit(&self) -> Vec<u8> {
        let mut k = Kit { name: self.name, values: [0; KIT_PARAMS] };
        for (i, v) in k.values.iter_mut().enumerate() {
            *v = self.params[i] as u8;
        }
        k.to_file_bytes().to_vec()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_maps_cc_and_nrpn_like_the_firmware_table() {
        let mut e = StubEngine::new();
        // CC 9 -> PAR_COARSE1 (index 8)
        for b in [0xb0, 9, 100] {
            e.midi_byte(b);
        }
        assert_eq!(e.get_parameter(par::COARSE1), 100);
        // NRPN 0 -> index 128
        for b in [0xb0, 99, 0, 98, 0, 6, 77] {
            e.midi_byte(b);
        }
        assert_eq!(e.get_parameter(128), 77);
        let k = e.save_kit();
        assert_eq!(k.len(), 255);
        let mut e2 = StubEngine::new();
        e2.load_kit(&k);
        assert_eq!(e2.get_parameter(par::COARSE1), 100);
    }

    #[test]
    fn default_note_triggers_all_tracks() {
        let e = StubEngine::new();
        // Defaults: all tracks on channel 1, note 48.
        assert_eq!(tracks_for_note(0, 48, |p| e.get_parameter(p)), 0x7f);
        assert_eq!(tracks_for_note(0, 49, |p| e.get_parameter(p)), 0);
        assert_eq!(track_trigger_note(0, |p| e.get_parameter(p)), (0, 48));
    }
}
