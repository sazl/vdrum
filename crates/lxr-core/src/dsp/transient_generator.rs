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
    todo!("port: Core/Src/Soundengine/DSPAudio/transientGenerator.c:20")
}

/// C: `IN_ITCM_RAM void transient_setWaveform(TransientGenerator* transient, const uint8_t waveform)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/transientGenerator.c:29`. Reference build: out-of-line in ITCM. Group P1.
pub fn transient_setWaveform(transient: &mut TransientGenerator, waveform: u8) {
    todo!("port: Core/Src/Soundengine/DSPAudio/transientGenerator.c:29")
}

/// C: `void transient_trigger(TransientGenerator* transient)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/transientGenerator.c:37`. Reference build: out-of-line in flash. Group P1.
pub fn transient_trigger(transient: &mut TransientGenerator) {
    todo!("port: Core/Src/Soundengine/DSPAudio/transientGenerator.c:37")
}

/// C: `void transient_calcBlock(TransientGenerator* transient, int16_t* buf, const uint8_t size)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/transientGenerator.c:43`. Reference build: out-of-line in flash. Group P1.
/// Tables: `transientData`.
pub fn transient_calcBlock(transient: &mut TransientGenerator, buf: &mut [i16], size: u8) {
    todo!("port: Core/Src/Soundengine/DSPAudio/transientGenerator.c:43")
}

/// C: `void transient_calc(TransientGenerator* transient)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/transientGenerator.c:71`. Reference build: not linked (removed by --gc-sections). Group P1.
/// Tables: `transientData`, `transientVolumeTable`.
pub fn transient_calc(transient: &mut TransientGenerator) {
    todo!("port: Core/Src/Soundengine/DSPAudio/transientGenerator.c:71")
}
