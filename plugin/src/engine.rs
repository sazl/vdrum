// SPDX-License-Identifier: GPL-3.0-only
//! The interface between the plugin and a sound engine, and a stub engine for testing the plugin
//! without the real port.
//!
//! The engine runs the firmware's audio interrupt: one [`Engine::tick`] renders 64 stereo frames
//! (`OUTPUT_DMA_SIZE`) at 44100 Hz for each of the two DACs, as signed 16 bit, L/R interleaved.

use lxr_params::kit::{Kit, KIT_FW_FILE_LEN, KIT_PARAMS};
use lxr_params::open_hat::{oh_index, oh_k};
use lxr_params::{par, GLOBAL_PARAMS, NUM_ALL_PARAMS, NUM_PARAMS, OH_NUM_PARAMS};
use vdrum_surface::seq::{SeqData, SeqStatus, StepData, NUM_PATTERNS, NUM_STEPS, NUM_TRACKS};

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

/// A sequencer operation (pattern data from the controller, or the host transport), applied on
/// the audio thread.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SeqEv {
    /// Replace one step (`data.prob` is the step's own probability; the engine plays it scaled by
    /// the densities).
    Step { pat: u8, track: u8, step: u8, data: StepData },
    /// On/off state of all steps of a track (bit `n` = step `n`).
    Bits { pat: u8, track: u8, bits: u64 },
    Length { pat: u8, track: u8, length: u8 },
    Shuffle { pat: u8, value: u8 },
    /// Playback density 0..=100 of a track, or of all tracks (`track` 7).
    Density { track: u8, value: u8 },
    /// Switch to a pattern at the end of the bar (at once when stopped).
    NextPattern(u8),
    /// Start or stop on the internal clock.
    Run(bool),
    /// Host transport: start; the argument is the position of the next 16th note.
    HostStart(u32),
    /// Host transport: the position jumped; the argument is the position of the next 16th note.
    HostAlign(u32),
    /// Host transport: one 16th note.
    HostPulse,
    HostStop,
    /// Tempo in bpm (rounded), for flams and shuffle.
    Tempo(u16),
    /// Back to the engine's own clock (`PAR_BPM`).
    InternalClock,
    /// Muted sequencer tracks (bit `t` = track `t`).
    Mutes(u8),
    /// Restart every LFO at its phase offset.
    LfoReset,
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
    /// Set a parameter (`ParamEnums` index, or an open hi-hat parameter from `NUM_PARAMS` on) as the
    /// front panel does (`activeParameters` semantics).
    fn set_parameter(&mut self, param: u16, value: u16);
    /// The current value of a parameter (`ParamEnums` index or open hi-hat parameter).
    fn get_parameter(&self, param: u16) -> u16;
    /// Load a kit from `.snd` file bytes (8 byte name + parameter bytes, optionally VDrum's extension
    /// block with the open hi-hat; see `lxr_params::kit`).
    fn load_kit(&mut self, kit: &[u8]);
    /// The current kit as VDrum `.snd` file bytes.
    fn save_kit(&self) -> Vec<u8>;
    /// Move voice fader `i` (0..7: Drum 1..3, Snare, Cymbal, Hi-hat, Open Hi-hat) to a 16 bit ADC
    /// reading (12 bits, left aligned), as the hardware's `adc_checkFaders` reads it. The default
    /// ignores it.
    fn set_fader_adc(&mut self, _i: usize, _adc: u16) {}
    /// Silence every voice (the controller's panic action). The default does nothing.
    fn all_sound_off(&mut self) {}
    /// Apply a sequencer operation. The default ignores it.
    fn seq(&mut self, _ev: SeqEv) {}
    /// Replace all sequencer data (outside the audio thread). The default ignores it.
    fn seq_load(&mut self, _data: &SeqData) {}
    /// The sequencer's playback state.
    fn seq_status(&self) -> SeqStatus {
        SeqStatus::default()
    }
    /// Whether the engine stopped after an internal error (it then renders silence and ignores
    /// input until the plugin replaces it in `initialize()`).
    fn faulted(&self) -> bool {
        false
    }
}

/// The `lxr-core` port: the LXR-02 sound engine (`docs/ENGINE_CONTRACT.md`), wrapped so that the
/// trait methods do not collide with `Lxr`'s own.
///
/// * The open hi-hat is a separate voice (`lxr_core::vdrum::open_hat`, hat split on). Its parameters
///   are indices `NUM_PARAMS..NUM_ALL_PARAMS` and fader 6.
/// * `set_parameter` is the menu path (`menu_encoderChangeParameter`), `get_parameter` reads
///   `activeParameters[]` (the edit buffer). Out-of-range indices are ignored / read as 0 instead
///   of panicking on the audio thread, and values are truncated to the firmware's `uint8_t`.
/// * `save_kit` writes what `preset_saveDrumsetSD` writes after the edit buffer was stored: the kit
///   name (`savefileNames[0][KIT_NAMES]`, set by `load_kit`) and `activeParameters[0..247]`.
/// * `load_kit` accepts 8..=255 bytes (anything else is ignored).
/// * Faders start at the top (the contract's power-on state) and move through `set_fader_adc`.
///
/// Last-resort safety net: a panic inside the engine (a bug: e.g. an index the C code would read
/// out of bounds) must not unwind into the host, which aborts it. It is caught, the engine is
/// marked faulted, renders silence and ignores all input from then on; `VDrum::initialize()`
/// replaces a faulted engine (outside the audio thread). Unwinding allocates, so this is not
/// real-time safe; it only runs on that bug path.
pub struct LxrEngine(pub lxr_core::Lxr, pub(crate) bool, pub(crate) Box<SeqAux>);

/// What the plugin needs besides the engine's pattern data to apply densities: every step's own
/// probability, the on/off states and the densities.
pub struct SeqAux {
    prob: [[[u8; NUM_STEPS]; NUM_TRACKS]; NUM_PATTERNS],
    bits: [[u64; NUM_TRACKS]; NUM_PATTERNS],
    density: [u8; NUM_TRACKS],
    density_all: u8,
}

impl SeqAux {
    fn new() -> Box<SeqAux> {
        Box::new(SeqAux {
            prob: [[[100; NUM_STEPS]; NUM_TRACKS]; NUM_PATTERNS],
            bits: [[0; NUM_TRACKS]; NUM_PATTERNS],
            density: [100; NUM_TRACKS],
            density_all: 100,
        })
    }

    fn effective(&self, track: usize, prob: u8) -> u8 {
        let d = self.density[track] as u32 * self.density_all as u32;
        ((prob.min(100) as u32 * d + 5000) / 10000) as u8
    }
}

fn vdrum_step(d: &StepData, prob: u8) -> lxr_core::vdrum::seq::VdrumStep {
    lxr_core::vdrum::seq::VdrumStep {
        on: d.on,
        velocity: d.velocity,
        prob,
        note: d.note,
        flam: d.flam,
        shift: d.shift,
        locks: d.locks,
    }
}

impl LxrEngine {
    /// Runs an engine call unless the engine is faulted; marks it faulted if the call panics.
    fn guarded<R>(&mut self, f: impl FnOnce(&mut lxr_core::Lxr) -> R) -> Option<R> {
        if self.1 {
            return None;
        }
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| f(&mut self.0))).ok();
        self.1 = r.is_none();
        r
    }
}

impl Engine for LxrEngine {
    fn new() -> Box<Self> {
        // `Lxr` itself is about 14 kB; its large buffers are boxed by `Lxr::new_zeroed`.
        let mut l = lxr_core::Lxr::new();
        l.set_hat_split(true);
        // `PAR_PATTERN_MODE` 1 (the firmware default) loads the pattern's kit slot from the
        // engine's kit memory on every pattern change, which would undo kit edits; the plugin keeps
        // the kit across pattern changes. Not a host parameter.
        l.activeParameters[lxr_core::parameters::PAR_PATTERN_MODE as usize] = 0;
        Box::new(LxrEngine(l, false, SeqAux::new()))
    }

    fn midi_byte(&mut self, b: u8) {
        self.guarded(|e| e.midi_byte(b));
    }

    fn tick(&mut self) -> Block {
        self.guarded(|e| {
            let b = e.tick();
            Block { dac1: b.dac1, dac2: b.dac2 }
        })
        .unwrap_or_default()
    }

    fn set_parameter(&mut self, param: u16, value: u16) {
        if param < NUM_PARAMS {
            self.guarded(|e| e.set_parameter(param, value.min(255) as u8));
        } else if let Some(k) = oh_k(param) {
            self.guarded(|e| e.oh_set_parameter(k, value.min(255) as u8));
        }
    }

    fn get_parameter(&self, param: u16) -> u16 {
        if let Some(k) = oh_k(param) {
            return self.0.oh_get_parameter(k) as u16;
        }
        self.0.activeParameters.get(param as usize).map_or(0, |v| *v as u16)
    }

    fn load_kit(&mut self, kit: &[u8]) {
        if kit.len() < 8 {
            return;
        }
        if let Ok(k) = Kit::from_file_bytes(kit) {
            // the firmware part as the firmware reads it (the bytes present, up to 255)
            let fw = &kit[..kit.len().min(KIT_FW_FILE_LEN)];
            self.guarded(|e| {
                e.load_kit(fw);
                e.oh_set_parameters(&k.oh);
            });
        }
    }

    fn save_kit(&self) -> Vec<u8> {
        let mut values = [0u8; KIT_PARAMS];
        values.copy_from_slice(&self.0.activeParameters[..KIT_PARAMS]);
        let mut k = Kit::from_values(self.0.savefileNames[0][0], values);
        for (i, v) in k.oh.iter_mut().enumerate() {
            *v = self.0.oh_get_parameter(i);
        }
        k.to_file_bytes().to_vec()
    }

    fn set_fader_adc(&mut self, i: usize, adc: u16) {
        if i < 7 {
            self.guarded(|e| e.set_fader_adc(i, adc));
        }
    }

    fn all_sound_off(&mut self) {
        self.guarded(|e| e.vdrum_all_sound_off());
    }

    fn seq(&mut self, ev: SeqEv) {
        let (p, t) = match ev {
            SeqEv::Step { pat, track, .. } | SeqEv::Bits { pat, track, .. } | SeqEv::Length { pat, track, .. } => {
                (pat as usize, track as usize)
            }
            _ => (0, 0),
        };
        if p >= NUM_PATTERNS || t >= NUM_TRACKS {
            return;
        }
        match ev {
            SeqEv::Step { pat, track, step, data } => {
                if step as usize >= NUM_STEPS {
                    return;
                }
                let aux = &mut self.2;
                aux.prob[p][t][step as usize] = data.prob;
                let bit = 1u64 << step;
                aux.bits[p][t] = if data.on { aux.bits[p][t] | bit } else { aux.bits[p][t] & !bit };
                let s = vdrum_step(&data, aux.effective(t, data.prob));
                self.guarded(|e| e.vdrum_seq_set_step(pat, track, step, s, true));
            }
            SeqEv::Bits { pat, track, bits } => {
                self.2.bits[p][t] = bits;
                let probs: [u8; NUM_STEPS] = std::array::from_fn(|i| self.2.effective(t, self.2.prob[p][t][i]));
                self.guarded(|e| {
                    for (i, pr) in probs.iter().enumerate() {
                        e.vdrum_seq_set_prob(pat, track, i as u8, *pr, false);
                    }
                    e.vdrum_seq_set_bits(pat, track, bits);
                });
            }
            SeqEv::Length { pat, track, length } => {
                self.guarded(|e| e.vdrum_seq_set_length(pat, track, length, true));
            }
            SeqEv::Shuffle { pat, value } => {
                if (pat as usize) < NUM_PATTERNS {
                    self.guarded(|e| e.vdrum_seq_set_shuffle(pat, value, true));
                }
            }
            SeqEv::Density { track, value } => {
                let v = value.min(100);
                match self.2.density.get_mut(track as usize) {
                    Some(d) => *d = v,
                    None => self.2.density_all = v,
                }
                let tracks = if (track as usize) < NUM_TRACKS { track..track + 1 } else { 0..NUM_TRACKS as u8 };
                let aux = &*self.2;
                let Self(e, faulted, _) = self;
                if *faulted {
                    return;
                }
                // only the steps that are on (a step turned on later gets its probability then)
                let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    for pat in 0..NUM_PATTERNS {
                        for t in tracks.clone() {
                            let mut b = aux.bits[pat][t as usize];
                            while b != 0 {
                                let s = b.trailing_zeros() as usize;
                                b &= b - 1;
                                let pr = aux.effective(t as usize, aux.prob[pat][t as usize][s]);
                                e.vdrum_seq_set_prob(pat as u8, t, s as u8, pr, false);
                            }
                        }
                    }
                    e.vdrum_seq_loaded();
                }));
                *faulted = r.is_err();
            }
            SeqEv::NextPattern(n) => {
                self.guarded(|e| e.seq_setNextPattern(n));
            }
            SeqEv::Run(on) => {
                self.guarded(|e| e.vdrum_seq_run(on));
            }
            SeqEv::HostStart(n) => {
                self.guarded(|e| e.vdrum_seq_host_start(n));
            }
            SeqEv::HostAlign(n) => {
                self.guarded(|e| e.vdrum_seq_align(n));
            }
            SeqEv::HostPulse => {
                self.guarded(|e| e.vdrum_seq_host_pulse());
            }
            SeqEv::HostStop => {
                self.guarded(|e| e.vdrum_seq_stop());
            }
            SeqEv::Tempo(bpm) => {
                self.guarded(|e| {
                    if e.seq_tempo != bpm {
                        e.seq_setBpm(bpm);
                    }
                });
            }
            SeqEv::InternalClock => {
                self.guarded(|e| e.vdrum_seq_internal_clock());
            }
            SeqEv::Mutes(m) => {
                self.guarded(|e| e.vdrum_seq_set_mutes(m));
            }
            SeqEv::LfoReset => {
                self.guarded(|e| e.vdrum_lfo_reset_phases());
            }
        }
    }

    fn seq_load(&mut self, data: &SeqData) {
        let aux = &mut self.2;
        aux.density = data.density;
        aux.density_all = data.density_all;
        for (p, pd) in data.patterns.iter().enumerate().take(NUM_PATTERNS) {
            for (t, td) in pd.tracks.iter().enumerate() {
                aux.bits[p][t] = td.bits();
                for (s, sd) in td.steps.iter().enumerate() {
                    aux.prob[p][t][s] = sd.prob;
                }
            }
        }
        let aux = &*self.2;
        let Self(e, faulted, _) = self;
        if *faulted {
            return;
        }
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            for (p, pd) in data.patterns.iter().enumerate().take(NUM_PATTERNS) {
                for (t, td) in pd.tracks.iter().enumerate() {
                    for (s, sd) in td.steps.iter().enumerate() {
                        let st = vdrum_step(sd, aux.effective(t, sd.prob));
                        e.vdrum_seq_set_step(p as u8, t as u8, s as u8, st, false);
                    }
                    e.vdrum_seq_set_length(p as u8, t as u8, td.length, false);
                }
                e.vdrum_seq_set_shuffle(p as u8, pd.shuffle, false);
            }
            e.vdrum_seq_loaded();
        }));
        *faulted = r.is_err();
    }

    fn seq_status(&self) -> SeqStatus {
        let e = &self.0;
        SeqStatus {
            running: e.seq_running != 0,
            pattern: e.seq_activePattern,
            pending: e.seq_pendingPattern,
            step: e.seq_stepIndex,
            late: (12..24).contains(&e.seq_prescaleCounter),
        }
    }

    fn faulted(&self) -> bool {
        self.1
    }
}

/// A stand-in engine that reacts like the firmware at the interface level (parameter storage, kit
/// files, CC/NRPN parameter mapping, note triggers by channel/note) and renders a simple test sound:
/// a decaying sine per triggered track on DAC 1, and the same at half level on DAC 2. It is not the
/// LXR-02 sound engine.
pub struct StubEngine {
    params: [u16; NUM_ALL_PARAMS as usize],
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
            params: [0; NUM_ALL_PARAMS as usize],
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
            for (i, v) in k.oh.iter().enumerate() {
                self.params[oh_index(i) as usize] = *v as u16;
            }
        }
    }

    fn save_kit(&self) -> Vec<u8> {
        let mut k = Kit::from_values(self.name, [0; KIT_PARAMS]);
        for (i, v) in k.values.iter_mut().enumerate() {
            *v = self.params[i] as u8;
        }
        for i in 0..OH_NUM_PARAMS {
            k.oh[i] = self.params[oh_index(i) as usize] as u8;
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
        assert_eq!(k.len(), lxr_params::kit::KIT_FILE_LEN);
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
