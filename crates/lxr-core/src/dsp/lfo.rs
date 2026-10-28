//! Port of `Core/Inc/Soundengine/DSPAudio/lfo.h`, `Core/Src/Soundengine/DSPAudio/lfo.c`.
//!
//! Porting group: P1. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;

// ---- #define constants
/// `#define LFO_MAX_F 200` (Core/Inc/Soundengine/DSPAudio/lfo.h:41)
pub const LFO_MAX_F: i32 = 200;
/// `#define LFO_SR (REAL_FS/(float)OUTPUT_DMA_SIZE)` (Core/Inc/Soundengine/DSPAudio/lfo.h:42) = 689.6875
pub const LFO_SR: f32 = f32::from_bits(0x442c6c00);

// ---- enums
// C: `enum {...}` (Core/Inc/Soundengine/DSPAudio/lfo.h:28), 1 bytes, underlying type `u8`. (anonymous: enumerators only)
pub const LFO_SINE: u8 = 0;
pub const LFO_TRI: u8 = 1;
pub const LFO_SAW_UP: u8 = 2;
pub const LFO_SAW_DOWN: u8 = 3;
pub const LFO_REC: u8 = 4;
pub const LFO_NOISE: u8 = 5;
pub const LFO_EXP_UP: u8 = 6;
pub const LFO_EXP_DOWN: u8 = 7;
pub const NUM_LFO_WAVEFORMS: u8 = 8;

// ---- types
/// C: `struct LfoStruct` (Core/Inc/Soundengine/DSPAudio/lfo.h:44), 48 bytes.
/// C typedef name `Lfo`; the struct tag `LfoStruct` is a type alias below.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Lfo {
    /// `uint32_t phase; // the current phase of the LFO (upper bits cnt 0-255)`
    pub phase: u32,
    /// `uint32_t phaseInc; // the phase increment controls the LFO frequency`
    pub phaseInc: u32,
    /// `uint8_t waveform; // selects the waveform`
    pub waveform: u8,
    /// `uint8_t retrigger; // defines the voice nr that retriggers the LFO (0=no retrigger)`
    pub retrigger: u8,
    /// `uint32_t phaseOffset;// the phase value to which the LFO is retriggered`
    pub phaseOffset: u32,
    /// `float rnd;`
    pub rnd: f32,
    /// `uint8_t sync;`
    pub sync: u8,
    /// `float freq;`
    pub freq: f32,
    /// `ModulationNode modTarget;`
    pub modTarget: ModulationNode,
    /// `float modNodeValue;`
    pub modNodeValue: f32,
}
/// C struct tag of `Lfo`.
pub type LfoStruct = Lfo;
impl Lfo {
    /// All-zero value (C static zero initialisation).
    pub const ZERO: Self = Self { phase: 0, phaseInc: 0, waveform: 0, retrigger: 0, phaseOffset: 0, rnd: 0.0, sync: 0, freq: 0.0, modTarget: ModulationNode::ZERO, modNodeValue: 0.0 };
}
impl Default for Lfo {
    fn default() -> Self {
        Self::ZERO
    }
}

// ---- functions that use globals (methods on `Lxr`)
impl Lxr {
    /// C: `void lfo_init(Lfo *lfo)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/lfo.c:25`. Reference build: out-of-line in flash. Group P1.
    /// Calls: `Lxr::modNode_init`.
    ///
    /// `lfo` is a place accessor (see `crate::place`): the C pointer may point into `Lxr` and this
    /// function can also reach the pointee through globals, so a plain `&mut` would not be faithful.
    pub fn lfo_init(&mut self, lfo: impl Fn(&mut Lxr) -> &mut Lfo + Copy) {
            {
            let l = lfo(self);
            l.phase = 0;
            l.phaseInc = 1;
            l.phaseOffset = 0;
            l.retrigger = 0;
            l.waveform = SINE;
            l.sync = 0;
            l.freq = 1.0;
            l.modNodeValue = 1.0;
        }
        self.modNode_init(move |s| &mut lfo(s).modTarget);
    }

    /// C: `IN_ITCM_RAM float lfo_calc(Lfo *lfo)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/lfo.c:39`. Reference build: out-of-line in ITCM. Group P1.
    /// Tables: `sine_table`.
    /// Calls: `Lxr::GetRngValue`.
    pub fn lfo_calc(&mut self, lfo: &mut Lfo) -> f32 {
            let oldPhase = lfo.phase;
        let incInt = (lfo.phaseInc as f32 * lfo.modNodeValue) as u32;
        let (phase, overflow) = oldPhase.overflowing_add(incInt);
        lfo.phase = phase;
        const INV_2_32: f32 = f32::from_bits(0x2f800000); // 1/(float)0xffffffff = 2^-32
        match lfo.waveform {
            LFO_SINE => {
                let s = sine_table[(phase >> 20) as usize] as i32 as f32;
                vfma(1.0, s, f32::from_bits(0x38000100)) * 0.5
            }
            LFO_TRI => {
                let t = vfma(-1.0, phase as f32, f32::from_bits(0x30000000)); // phase*2^-31 - 1
                1.0 - t.abs()
            }
            LFO_SAW_UP => phase as f32 * INV_2_32,
            LFO_REC => {
                if (phase as i32) < 0 {
                    1.0
                } else {
                    0.0
                }
            }
            LFO_NOISE => {
                if overflow {
                    lfo.rnd = self.GetRngValue() as f32 * INV_2_32;
                }
                lfo.rnd
            }
            LFO_SAW_DOWN => (!phase) as f32 * INV_2_32,
            LFO_EXP_UP => {
                let x = phase as f32 * INV_2_32;
                x * x * x
            }
            LFO_EXP_DOWN => {
                let x = (!phase) as f32 * INV_2_32;
                x * x * x
            }
            _ => 0.0,
        }
    }

    /// C: `void lfo_dispatchNextValue(Lfo* lfo)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/lfo.c:108`. Reference build: out-of-line in flash. Group P1.
    /// Tables: `sine_table`.
    /// Calls: `Lxr::lfo_calc`, `Lxr::modNode_updateValue`.
    ///
    /// `lfo` is a place accessor (see `crate::place`): the C pointer may point into `Lxr` and this
    /// function can also reach the pointee through globals, so a plain `&mut` would not be faithful.
    pub fn lfo_dispatchNextValue(&mut self, lfo: impl Fn(&mut Lxr) -> &mut Lfo + Copy) {
            let mut l = *lfo(self);
        let val = self.lfo_calc(&mut l);
        *lfo(self) = l;
        self.modNode_updateValue(move |s| &mut lfo(s).modTarget, val);
    }

    /// C: `IN_ITCM_RAM uint32_t lfo_calcPhaseInc(float freq, uint8_t sync)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/lfo.c:114`. Reference build: out-of-line in ITCM. Group P1.
    /// Calls: `Lxr::seq_getBpm`.
    pub fn lfo_calcPhaseInc(&mut self, freq: f32, sync: u8) -> u32 {
            // freq/(LFO_SR) * 0xffffffff folded to freq * K
        const K: f32 = f32::from_bits(0x4abe0ba5);
        if sync == 0 {
            return (freq * K) as u32;
        }
        let bpm = self.seq_getBpm() as u32;
        let tempoAsFrequency = if bpm != 0 { bpm as f32 * f32::from_bits(0x3b888889) } else { 0.0 }; // bpm/60/4
        let s = sync.wrapping_sub(1);
        if s > 10 {
            return (freq * K) as u32;
        }
        ((LFO_SYNC_SCALERS[s as usize] * K) * tempoAsFrequency) as u32
    }

    /// C: `IN_ITCM_RAM void lfo_setFreq(Lfo *lfo, float f)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/lfo.c:185`. Reference build: out-of-line in ITCM. Group P1.
    /// Calls: `Lxr::lfo_calcPhaseInc`.
    pub fn lfo_setFreq(&mut self, lfo: &mut Lfo, f: f32) {
            let f = vfma(f32::from_bits(0x3c000000), f, f32::from_bits(0x3f7e0000)); // 0.0078125f + f*(1-0.0078125f)
        lfo.freq = ((f * f) * 200.0) * f;
        lfo.phaseInc = self.lfo_calcPhaseInc(lfo.freq, lfo.sync);
    }

    /// C: `IN_ITCM_RAM void lfo_setSync(Lfo* lfo, uint8_t sync)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/lfo.c:195`. Reference build: out-of-line in ITCM. Group P1.
    /// Calls: `Lxr::lfo_calcPhaseInc`.
    pub fn lfo_setSync(&mut self, lfo: &mut Lfo, sync: u8) {
            lfo.sync = sync;
        lfo.phaseInc = self.lfo_calcPhaseInc(lfo.freq, lfo.sync);
    }

    /// C: `void lfo_recalcSync()`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/lfo.c:201`. Reference build: out-of-line in flash. Group P1.
    /// Globals: `cymbalVoice`, `hatVoice`, `snareVoice`, `voiceArray`.
    /// Calls: `Lxr::lfo_calcPhaseInc`.
    pub fn lfo_recalcSync(&mut self) {
            for v in 0..3 {
            let (f, s) = (self.voiceArray[v].lfo.freq, self.voiceArray[v].lfo.sync);
            self.voiceArray[v].lfo.phaseInc = self.lfo_calcPhaseInc(f, s);
        }
        let (f, s) = (self.snareVoice.lfo.freq, self.snareVoice.lfo.sync);
        self.snareVoice.lfo.phaseInc = self.lfo_calcPhaseInc(f, s);
        let (f, s) = (self.cymbalVoice.lfo.freq, self.cymbalVoice.lfo.sync);
        self.cymbalVoice.lfo.phaseInc = self.lfo_calcPhaseInc(f, s);
        let (f, s) = (self.hatVoice.lfo.freq, self.hatVoice.lfo.sync);
        self.hatVoice.lfo.phaseInc = self.lfo_calcPhaseInc(f, s);
    }

    /// C: `void lfo_retrigger(uint8_t voice)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/lfo.c:222`. Reference build: out-of-line in flash. Group P1.
    /// Globals: `cymbalVoice`, `hatVoice`, `snareVoice`, `voiceArray`.
    pub fn lfo_retrigger(&mut self, voice: u8) {
            let v = voice as u32 + 1;
        for i in 0..3 {
            let l = &mut self.voiceArray[i].lfo;
            if l.retrigger as u32 == v {
                l.phase = l.phaseOffset;
            }
        }
        for l in [&mut self.snareVoice.lfo, &mut self.cymbalVoice.lfo, &mut self.hatVoice.lfo] {
            if l.retrigger as u32 == v {
                l.phase = l.phaseOffset;
            }
        }
    }
}

/// The `scaler` values of the `switch(sync)` in `lfo_calcPhaseInc` (`CSWTCH.5` in the reference ELF), index
/// `sync - 1` for sync 1..=11.
pub const LFO_SYNC_SCALERS: [f32; 11] = [0.25, 0.5, 1.0, 2.0, 3.0, 4.0, 6.0, 8.0, 12.0, 16.0, 32.0];
