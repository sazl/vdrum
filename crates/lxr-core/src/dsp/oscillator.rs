//! Port of `Core/Inc/Soundengine/DSPAudio/Oscillator.h`, `Core/Src/Soundengine/DSPAudio/Oscillator.c`.
//!
//! Porting group: P1. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;

// ---- #define constants
/// `#define phaseShift 1000` (Core/Inc/Soundengine/DSPAudio/Oscillator.h:38)
pub const phaseShift: i32 = 1000;

// ---- enums
/// C: `enum OscWaveforms` (Core/Inc/Soundengine/DSPAudio/Oscillator.h:26), 1 bytes, underlying type `u8`.
/// Representation: integer type alias plus one `pub const` per enumerator.
pub type OscWaveforms = u8;
pub const SINE: OscWaveforms = 0;
pub const TRI: OscWaveforms = 1;
pub const SAW: OscWaveforms = 2;
pub const REC: OscWaveforms = 3;
pub const NOISE: OscWaveforms = 4;
pub const PWM: OscWaveforms = 5;
pub const NUM_OSC_WAVEFORMS: OscWaveforms = 6;

// ---- types
/// C: `struct OscStruct` (Core/Inc/Soundengine/DSPAudio/Oscillator.h:42), 40 bytes.
/// C typedef name `OscInfo`; the struct tag `OscStruct` is a type alias below.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OscInfo {
    /// `int16_t output;`
    pub output: i16,
    /// `uint32_t phaseInc;`
    pub phaseInc: u32,
    /// `uint32_t phase; // the current phase of the osc 8bit indexing 24 interpolation`
    pub phase: u32,
    /// `float freq; // frequency in [Hz]`
    pub freq: f32,
    /// `uint8_t waveform; // the selected waveform of the osc`
    pub waveform: u8,
    /// `uint8_t tableOffset; //overtone table`
    pub tableOffset: u8,
    /// `float pitchMod; // modulation value for frequency`
    pub pitchMod: f32,
    /// `float fmMod;`
    pub fmMod: f32,
    /// `float modNodeValue;`
    pub modNodeValue: f32,
    /// `uint16_t midiFreq; //upper 8 bit coarse, lower 8 bit fine -> the sound edit freq offset`
    pub midiFreq: u16,
    /// `uint8_t baseNote; // the last played midi note`
    pub baseNote: u8,
    /// `uint32_t startPhase; // the OSC is reset to this phase on retrigger`
    pub startPhase: u32,
}
/// C struct tag of `OscInfo`.
pub type OscStruct = OscInfo;
impl OscInfo {
    /// All-zero value (C static zero initialisation).
    pub const ZERO: Self = Self { output: 0, phaseInc: 0, phase: 0, freq: 0.0, waveform: 0, tableOffset: 0, pitchMod: 0.0, fmMod: 0.0, modNodeValue: 0.0, midiFreq: 0, baseNote: 0, startPhase: 0 };
}
impl Default for OscInfo {
    fn default() -> Self {
        Self::ZERO
    }
}

// ---- functions without access to globals
/// C: `static __inline uint32_t freq2PhaseIncr(const float f)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:22`. Reference build: inlined into its callers only. Group P1.
pub fn freq2PhaseIncr(f: f32) -> u32 {
    todo!("port: Core/Src/Soundengine/DSPAudio/Oscillator.c:22")
}

/// C: `static __inline uint8_t fast_log2 (const uint32_t val)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:28`. Reference build: inlined into its callers only. Group P1.
pub fn fast_log2(val: u32) -> u8 {
    todo!("port: Core/Src/Soundengine/DSPAudio/Oscillator.c:28")
}

/// C: `static __inline int freqToMidiNote(const float f)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:33`. Reference build: inlined into its callers only. Group P1.
/// Calls: `dsp::oscillator::fast_log2`.
pub fn freqToMidiNote(f: f32) -> i32 {
    todo!("port: Core/Src/Soundengine/DSPAudio/Oscillator.c:33")
}

/// C: `static __inline uint8_t freqToTableIndex(const float f)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:38`. Reference build: inlined into its callers only. Group P1.
/// Calls: `dsp::oscillator::freqToMidiNote`.
pub fn freqToTableIndex(f: f32) -> u8 {
    todo!("port: Core/Src/Soundengine/DSPAudio/Oscillator.c:38")
}

/// C: `static __inline uint32_t freq2PhaseIncr1024(const float f)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:45`. Reference build: inlined into its callers only. Group P1.
pub fn freq2PhaseIncr1024(f: f32) -> u32 {
    todo!("port: Core/Src/Soundengine/DSPAudio/Oscillator.c:45")
}

/// C: `static __inline uint32_t freq2PhaseIncr32767(const float f)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:50`. Reference build: inlined into its callers only. Group P1.
pub fn freq2PhaseIncr32767(f: f32) -> u32 {
    todo!("port: Core/Src/Soundengine/DSPAudio/Oscillator.c:50")
}

/// C: `IN_ITCM_RAM void calcSineBlock(OscInfo* osc, int16_t* buf, const uint8_t size ,const float gain)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:55`. Reference build: out-of-line in ITCM. Group P1.
/// Tables: `sine_table`.
pub fn calcSineBlock(osc: &mut OscInfo, buf: &mut [i16], size: u8, gain: f32) {
    todo!("port: Core/Src/Soundengine/DSPAudio/Oscillator.c:55")
}

/// C: `int16_t calcSine(OscInfo* osc)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:85`. Reference build: not linked (removed by --gc-sections). Group P1.
/// Tables: `sine_table`.
pub fn calcSine(osc: &mut OscInfo) -> i16 {
    todo!("port: Core/Src/Soundengine/DSPAudio/Oscillator.c:85")
}

/// C: `IN_ITCM_RAM void calcFmSineBlock(OscInfo* osc, int16_t* modBuffer, int16_t* buf, uint8_t size,const float gain)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:109`. Reference build: out-of-line in ITCM. Group P1.
/// Tables: `sine_table`.
pub fn calcFmSineBlock(osc: &mut OscInfo, modBuffer: &mut [i16], buf: &mut [i16], size: u8, gain: f32) {
    todo!("port: Core/Src/Soundengine/DSPAudio/Oscillator.c:109")
}

/// C: `int16_t calcFmSine(OscInfo* osc, OscInfo* modOsc)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:135`. Reference build: not linked (removed by --gc-sections). Group P1.
/// Tables: `sine_table`.
pub fn calcFmSine(osc: &mut OscInfo, modOsc: &mut OscInfo) -> i16 {
    todo!("port: Core/Src/Soundengine/DSPAudio/Oscillator.c:135")
}

/// C: `IN_ITCM_RAM void calcFmBlock(OscInfo* osc, const int16_t table[][1024], int16_t* modBuffer, int16_t* buf, uint8_t size ,const float gain)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:158`. Reference build: out-of-line in ITCM. Group P1.
pub fn calcFmBlock(osc: &mut OscInfo, table: &mut [[i16; 1024]], modBuffer: &mut [i16], buf: &mut [i16], size: u8, gain: f32) {
    todo!("port: Core/Src/Soundengine/DSPAudio/Oscillator.c:158")
}

/// C: `void calcPwmFmBlock(OscInfo* osc, int16_t* modBuffer, int16_t* buf, uint8_t size ,const float gain, const float pwm)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:186`. Reference build: out-of-line in flash. Group P1.
/// Tables: `sawTable`.
/// Calls: `cmsis::__QSUB16`.
pub fn calcPwmFmBlock(osc: &mut OscInfo, modBuffer: &mut [i16], buf: &mut [i16], size: u8, gain: f32, pwm: f32) {
    todo!("port: Core/Src/Soundengine/DSPAudio/Oscillator.c:186")
}

/// C: `int16_t calcFm(OscInfo* osc, OscInfo* modOsc, const int16_t table[][1024])`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:225`. Reference build: not linked (removed by --gc-sections). Group P1.
pub fn calcFm(osc: &mut OscInfo, modOsc: &mut OscInfo, table: &mut [[i16; 1024]]) -> i16 {
    todo!("port: Core/Src/Soundengine/DSPAudio/Oscillator.c:225")
}

/// C: `IN_ITCM_RAM void calcWavetableOscBlock(OscInfo* osc, const int16_t table[][1024], int16_t* buf, const uint8_t size ,const float gain)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:450`. Reference build: out-of-line in ITCM. Group P1.
pub fn calcWavetableOscBlock(osc: &mut OscInfo, table: &mut [[i16; 1024]], buf: &mut [i16], size: u8, gain: f32) {
    todo!("port: Core/Src/Soundengine/DSPAudio/Oscillator.c:450")
}

/// C: `int16_t calcWavetableOsc(OscInfo* osc, const int16_t table[][1024])`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:475`. Reference build: not linked (removed by --gc-sections). Group P1.
pub fn calcWavetableOsc(osc: &mut OscInfo, table: &mut [[i16; 1024]]) -> i16 {
    todo!("port: Core/Src/Soundengine/DSPAudio/Oscillator.c:475")
}

/// C: `void calcUserSampleOscFmBlock(OscInfo* osc,int16_t* modBuffer, int16_t* buf, uint8_t size ,const float gain)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:555`. Reference build: not linked (removed by --gc-sections). Group P1.
pub fn calcUserSampleOscFmBlock(osc: &mut OscInfo, modBuffer: &mut [i16], buf: &mut [i16], size: u8, gain: f32) {
    todo!("port: Core/Src/Soundengine/DSPAudio/Oscillator.c:555")
}

/// C: `void calcUserSampleOscBlock(OscInfo* osc, int16_t* buf, const uint8_t size ,const float gain)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:592`. Reference build: not linked (removed by --gc-sections). Group P1.
pub fn calcUserSampleOscBlock(osc: &mut OscInfo, buf: &mut [i16], size: u8, gain: f32) {
    todo!("port: Core/Src/Soundengine/DSPAudio/Oscillator.c:592")
}

/// C: `IN_ITCM_RAM void calcPwmBlock(OscInfo* osc, int16_t* buf, uint8_t size ,const float gain, const float pwm)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:656`. Reference build: out-of-line in ITCM. Group P1.
/// Tables: `sawTable`.
/// Calls: `cmsis::__QSUB16`.
pub fn calcPwmBlock(osc: &mut OscInfo, buf: &mut [i16], size: u8, gain: f32, pwm: f32) {
    todo!("port: Core/Src/Soundengine/DSPAudio/Oscillator.c:656")
}

/// C: `void osc_calcSineFreq(OscInfo* osc)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:722`. Reference build: inlined into its callers only. Group P1.
/// Calls: `dsp::oscillator::freq2PhaseIncr`.
pub fn osc_calcSineFreq(osc: &mut OscInfo) {
    todo!("port: Core/Src/Soundengine/DSPAudio/Oscillator.c:722")
}

/// C: `void osc_calcNoiseFreq(OscInfo* osc)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:727`. Reference build: not linked (removed by --gc-sections). Group P1.
/// Calls: `dsp::oscillator::osc_calcSineFreq`.
pub fn osc_calcNoiseFreq(osc: &mut OscInfo) {
    todo!("port: Core/Src/Soundengine/DSPAudio/Oscillator.c:727")
}

/// C: `void osc_calcWavetableFreq(OscInfo* osc)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:732`. Reference build: inlined into its callers only. Group P1.
/// Calls: `dsp::oscillator::freq2PhaseIncr1024`, `dsp::oscillator::freqToTableIndex`.
pub fn osc_calcWavetableFreq(osc: &mut OscInfo) {
    todo!("port: Core/Src/Soundengine/DSPAudio/Oscillator.c:732")
}

/// C: `void osc_calcSampleFreq(OscInfo* osc)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:741`. Reference build: not linked (removed by --gc-sections). Group P1.
/// Calls: `dsp::oscillator::freq2PhaseIncr32767`.
pub fn osc_calcSampleFreq(osc: &mut OscInfo) {
    todo!("port: Core/Src/Soundengine/DSPAudio/Oscillator.c:741")
}

/// C: `void osc_calcUserSampleFreq(OscInfo* osc)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:747`. Reference build: inlined into its callers only. Group P1.
/// Calls: `dsp::oscillator::freq2PhaseIncr32767`.
pub fn osc_calcUserSampleFreq(osc: &mut OscInfo) {
    todo!("port: Core/Src/Soundengine/DSPAudio/Oscillator.c:747")
}

/// C: `void osc_setFreq(OscInfo* osc)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:753`. Reference build: out-of-line in flash. Group P1.
/// Calls: `dsp::oscillator::osc_calcNoiseFreq`, `dsp::oscillator::osc_calcSineFreq`, `dsp::oscillator::osc_calcUserSampleFreq`, `dsp::oscillator::osc_calcWavetableFreq`.
pub fn osc_setFreq(osc: &mut OscInfo) {
    todo!("port: Core/Src/Soundengine/DSPAudio/Oscillator.c:753")
}

/// C: `void osc_setBaseNote(OscInfo* osc, uint8_t baseNote)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:790`. Reference build: out-of-line in flash. Group P1.
/// Tables: `MidiNoteFrequencies`.
/// Calls: `midi::midi_parser::midiParser_calcDetune`.
pub fn osc_setBaseNote(osc: &mut OscInfo, baseNote: u8) {
    todo!("port: Core/Src/Soundengine/DSPAudio/Oscillator.c:790")
}

/// C: `IN_ITCM_RAM void osc_recalcFreq(OscInfo* osc)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:805`. Reference build: out-of-line in ITCM. Group P1.
/// Tables: `MidiNoteFrequencies`.
/// Calls: `midi::midi_parser::midiParser_calcDetune`.
pub fn osc_recalcFreq(osc: &mut OscInfo) {
    todo!("port: Core/Src/Soundengine/DSPAudio/Oscillator.c:805")
}

// ---- functions that use globals (methods on `Lxr`)
impl Lxr {
    /// C: `IN_ITCM_RAM void calcNextOscSampleFmBlock(OscInfo* osc, int16_t* modBuffer, int16_t* buf, uint8_t size ,const float gain, const float pwm)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:250`. Reference build: out-of-line in ITCM. Group P1.
    /// Tables: `recTable`, `sawTable`, `sine_table`, `triTable`.
    /// Calls: `dsp::oscillator::calcFmBlock`, `dsp::oscillator::calcFmSineBlock`, `Lxr::calcNoiseBlock`, `dsp::oscillator::calcPwmFmBlock`, `dsp::oscillator::calcUserSampleOscFmBlock`.
    pub fn calcNextOscSampleFmBlock(&mut self, osc: &mut OscInfo, modBuffer: &mut [i16], buf: &mut [i16], size: u8, gain: f32, pwm: f32) {
        todo!("port: Core/Src/Soundengine/DSPAudio/Oscillator.c:250")
    }

    /// C: `IN_ITCM_RAM void calcNoiseBlock(OscInfo* osc, int16_t* buf, const uint8_t size ,const float gain)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:325`. Reference build: out-of-line in ITCM. Group P1.
    /// Calls: `Lxr::GetRngValue`.
    pub fn calcNoiseBlock(&mut self, osc: &mut OscInfo, buf: &mut [i16], size: u8, gain: f32) {
        todo!("port: Core/Src/Soundengine/DSPAudio/Oscillator.c:325")
    }

    /// C: `int16_t calcNoise(OscInfo* osc)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:343`. Reference build: not linked (removed by --gc-sections). Group P1.
    /// Calls: `Lxr::GetRngValue`, `cmsis::__get_APSR`.
    pub fn calcNoise(&mut self, osc: &mut OscInfo) -> i16 {
        todo!("port: Core/Src/Soundengine/DSPAudio/Oscillator.c:343")
    }

    /// C: `IN_ITCM_RAM void calcNextOscSampleBlock(OscInfo* osc, int16_t* buf, const uint8_t size, const float gain, const float pwm)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:375`. Reference build: out-of-line in ITCM. Group P1.
    /// Tables: `recTable`, `sawTable`, `sine_table`, `triTable`.
    /// Calls: `Lxr::calcNoiseBlock`, `dsp::oscillator::calcPwmBlock`, `dsp::oscillator::calcSineBlock`, `dsp::oscillator::calcWavetableOscBlock`.
    pub fn calcNextOscSampleBlock(&mut self, osc: &mut OscInfo, buf: &mut [i16], size: u8, gain: f32, pwm: f32) {
        todo!("port: Core/Src/Soundengine/DSPAudio/Oscillator.c:375")
    }
}
