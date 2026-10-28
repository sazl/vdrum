//! Port of `Core/Inc/Soundengine/DSPAudio/transientGenerator.h`, `Core/Src/Soundengine/DSPAudio/transientGenerator.c`.
//!
//! Porting group: P1. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;

// ---- types
/// C: `struct TransientGeneratorStruct` (Core/Inc/Soundengine/DSPAudio/transientGenerator.h:26), 24 bytes.
/// C typedef name `TransientGenerator`; the struct tag `TransientGeneratorStruct` is a type alias below.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TransientGenerator {
    /// `int16_t output;`
    pub output: i16,
    /// `uint32_t phase; // the current phase of the osc`
    pub phase: u32,
    /// `float pitch; // we need no accurate tuning for a transient, so we use a pitch factor [0.25f:1.75f]`
    pub pitch: f32,
    /// `uint8_t waveform; // the selected transient table of the osc`
    pub waveform: u8,
    /// `float volume;`
    pub volume: f32,
    /// `uint16_t midiFreq; //upper 8 bit coarse, lower 8 bit fine`
    pub midiFreq: u16,
}
/// C struct tag of `TransientGenerator`.
pub type TransientGeneratorStruct = TransientGenerator;
impl TransientGenerator {
    /// All-zero value (C static zero initialisation).
    pub const ZERO: Self = Self { output: 0, phase: 0, pitch: 0.0, waveform: 0, volume: 0.0, midiFreq: 0 };
}
impl Default for TransientGenerator {
    fn default() -> Self {
        Self::ZERO
    }
}

// ---- functions without access to globals
/// C: `void transient_init(TransientGenerator* transient)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/transientGenerator.c:20`. Reference build: out-of-line in flash. Group P1.
pub fn transient_init(transient: &mut TransientGenerator) {
    transient.pitch = 1.0;
    transient.output = 0;
    transient.phase = 0;
    transient.waveform = 0;
    transient.volume = 1.0;
}

/// C: `IN_ITCM_RAM void transient_setWaveform(TransientGenerator* transient, const uint8_t waveform)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/transientGenerator.c:29`. Reference build: out-of-line in ITCM. Group P1.
pub fn transient_setWaveform(transient: &mut TransientGenerator, waveform: u8) {
    if (waveform as i32) < NUM_TRANSIENTS + 2 {
        transient.waveform = waveform;
    } else {
        transient.waveform = 0;
    }
}

/// C: `void transient_trigger(TransientGenerator* transient)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/transientGenerator.c:37`. Reference build: out-of-line in flash. Group P1.
pub fn transient_trigger(transient: &mut TransientGenerator) {
    transient.phase = 0;
}

/// C: `void transient_calcBlock(TransientGenerator* transient, int16_t* buf, const uint8_t size)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/transientGenerator.c:43`. Reference build: out-of-line in flash. Group P1.
/// Tables: `transientData`.
pub fn transient_calcBlock(transient: &mut TransientGenerator, buf: &mut [i16], size: u8) {
    if transient.waveform <= 1 || transient.phase >= 2311061504 {
        // memset(buf, 0, size/2*sizeof(int32_t)): an odd last sample is left untouched
        let n = (size as usize / 2) * 2;
        for x in buf[..n].iter_mut() {
            *x = 0;
        }
        return;
    }
    let table = &transientData[transient.waveform as usize - 2];
    for i in 0..size as usize {
        let phase = transient.phase;
        if phase < 2311061504 {
            let idx = (phase >> 20) as usize;
            let s = (table[idx] as i32) << 8;
            buf[i] = ((s as f32 * transient.volume) as i32) as i16;
            // phase += pitch*(1<<20): (float)phase + pitch*2^20 fused, vcvt.u32.f32
            transient.phase = vfma(phase as f32, transient.pitch, 1048576.0) as u32;
        } else {
            buf[i] = 0;
        }
    }
}

/// C: `void transient_calc(TransientGenerator* transient)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/transientGenerator.c:71`. Reference build: not linked (removed by --gc-sections). Group P1.
/// Tables: `transientData`, `transientVolumeTable`.
pub fn transient_calc(transient: &mut TransientGenerator) {
    // not linked in the firmware (untestable): ported from C
    let mut phase = transient.phase;
    if phase < 2311061504 {
        phase = (phase as f32 + transient.pitch * 1048576.0) as u32;
    }
    transient.phase = phase;
    let phase = (phase >> 20) as usize;
    if transient.waveform == 0 {
        transient.output = (transient.volume * 32512.0 * transientVolumeTable[phase >> 5]) as i32 as i16;
    } else {
        transient.output = (transient.volume
            * (((transientData[transient.waveform as usize - 1][phase] as i32) << 8) as f32)
            * transientVolumeTable[phase >> 5]) as i32 as i16;
    }
}
