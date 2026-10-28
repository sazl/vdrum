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
    // ((TABLESIZE*f)/REAL_FS)*1048576 folded to one multiplication
    (f * K_PHASE) as u32
}

/// C: `static __inline uint8_t fast_log2 (const uint32_t val)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:28`. Reference build: inlined into its callers only. Group P1.
pub fn fast_log2(val: u32) -> u8 {
    // 31 - __CLZ(val), truncated to uint8_t (val == 0 gives 255)
    31u32.wrapping_sub(val.leading_zeros()) as u8
}

/// C: `static __inline int freqToMidiNote(const float f)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:33`. Reference build: inlined into its callers only. Group P1.
/// Calls: `dsp::oscillator::fast_log2`.
pub fn freqToMidiNote(f: f32) -> i32 {
    12 * fast_log2((f * f32::from_bits(0x3b14f209)) as u32) as i32 + 70 // f * (1/440.f)
}

/// C: `static __inline uint8_t freqToTableIndex(const float f)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:38`. Reference build: inlined into its callers only. Group P1.
/// Calls: `dsp::oscillator::freqToMidiNote`.
pub fn freqToTableIndex(f: f32) -> u8 {
    (freqToMidiNote(f) / 12) as u8
}

/// C: `static __inline uint32_t freq2PhaseIncr1024(const float f)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:45`. Reference build: inlined into its callers only. Group P1.
pub fn freq2PhaseIncr1024(f: f32) -> u32 {
    (f * K_PHASE) as u32 // ((1024*f)/REAL_FS)*4194304 folded
}

/// C: `static __inline uint32_t freq2PhaseIncr32767(const float f)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:50`. Reference build: inlined into its callers only. Group P1.
pub fn freq2PhaseIncr32767(f: f32) -> u32 {
    (f * K_PHASE_SAMPLE) as u32 // ((1024*f)/REAL_FS)*131072 folded
}

/// C: `IN_ITCM_RAM void calcSineBlock(OscInfo* osc, int16_t* buf, const uint8_t size ,const float gain)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:55`. Reference build: out-of-line in ITCM. Group P1.
/// Tables: `sine_table`.
pub fn calcSineBlock(osc: &mut OscInfo, buf: &mut [i16], size: u8, gain: f32) {
    let inc = osc.phaseInc;
    for i in 0..size as usize {
        let phase = osc.phase;
        let out = interp(sine_table[(phase >> 20) as usize], sine_table[(phase >> 20) as usize + 1], phase & 0xFFFFF, 20);
        osc.phase = phase.wrapping_add(inc);
        buf[i] = gain_i16(out, gain);
    }
}

/// C: `int16_t calcSine(OscInfo* osc)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:85`. Reference build: not linked (removed by --gc-sections). Group P1.
/// Tables: `sine_table`.
pub fn calcSine(osc: &mut OscInfo) -> i16 {
    // not linked in the firmware (untestable): ported from C
    let phase = osc.phase;
    let out = interp(sine_table[(phase >> 20) as usize], sine_table[(phase >> 20) as usize + 1], phase & 0xFFFFF, 20);
    osc.phase = phase.wrapping_add(osc.phaseInc);
    osc.output = out;
    out
}

/// C: `IN_ITCM_RAM void calcFmSineBlock(OscInfo* osc, int16_t* modBuffer, int16_t* buf, uint8_t size,const float gain)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:109`. Reference build: out-of-line in ITCM. Group P1.
/// Tables: `sine_table`.
pub fn calcFmSineBlock(osc: &mut OscInfo, modBuffer: &[i16], buf: &mut [i16], size: u8, gain: f32) {
    let inc = osc.phaseInc;
    let fm = osc.fmMod;
    for i in 0..size as usize {
        let phase = osc.phase;
        let index = phase.wrapping_add(((modBuffer[i] as i32 as f32 * fm) as u32) << 17);
        let itg = (index >> 20) as usize;
        let out = interp(sine_table[itg], sine_table[itg + 1], index & 0xFFFFF, 20);
        osc.phase = phase.wrapping_add(inc);
        buf[i] = gain_i16(out, gain);
    }
}

/// C: `int16_t calcFmSine(OscInfo* osc, OscInfo* modOsc)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:135`. Reference build: not linked (removed by --gc-sections). Group P1.
/// Tables: `sine_table`.
pub fn calcFmSine(osc: &mut OscInfo, modOsc: &mut OscInfo) -> i16 {
    // not linked in the firmware (untestable): ported from C
    let phase = osc.phase;
    let index = phase.wrapping_add(((modOsc.output as i32 as f32 * osc.fmMod) as u32) << 17);
    let itg = (index >> 20) as usize;
    let out = interp(sine_table[itg], sine_table[itg + 1], index & 0xFFFFF, 20);
    osc.phase = phase.wrapping_add(osc.phaseInc);
    osc.output = out;
    out
}

/// C: `IN_ITCM_RAM void calcFmBlock(OscInfo* osc, const int16_t table[][1024], int16_t* modBuffer, int16_t* buf, uint8_t size ,const float gain)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:158`. Reference build: out-of-line in ITCM. Group P1.
pub fn calcFmBlock(osc: &mut OscInfo, table: &[[i16; 1024]], modBuffer: &[i16], buf: &mut [i16], size: u8, gain: f32) {
    let inc = osc.phaseInc;
    let fm = osc.fmMod;
    let base = osc.tableOffset as usize * 1024;
    for i in 0..size as usize {
        let phase = osc.phase;
        let index = phase.wrapping_add(((modBuffer[i] as i32 as f32 * fm) as u32) << 19);
        let flat = base + (index >> 22) as usize;
        let out = interp(wt_at(table, flat), wt_at(table, flat + 1), index & 0x3FFFFF, 22);
        osc.phase = phase.wrapping_add(inc);
        osc.output = out;
        buf[i] = gain_i16(out, gain);
    }
}

/// C: `void calcPwmFmBlock(OscInfo* osc, int16_t* modBuffer, int16_t* buf, uint8_t size ,const float gain, const float pwm)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:186`. Reference build: out-of-line in flash. Group P1.
/// Tables: `sawTable`.
/// Calls: `cmsis::__QSUB16`.
pub fn calcPwmFmBlock(osc: &mut OscInfo, modBuffer: &[i16], buf: &mut [i16], size: u8, gain: f32, pwm: f32) {
    let inc = osc.phaseInc;
    let fm = osc.fmMod;
    let pw = pwm * f32::from_bits(0x4f000000); // pwm * (UINT32_MAX/2) as float = 2^31
    let base = osc.tableOffset as usize * 1024;
    let table = &sawTable[..];
    for i in 0..size as usize {
        let phase = osc.phase;
        let index = phase.wrapping_add(((modBuffer[i] as i32 as f32 * fm) as u32) << 19);
        let index2 = (index as f32 + pw) as u32;
        let out = pwm_pair(table, base, index, index2);
        osc.phase = phase.wrapping_add(inc);
        osc.output = out;
        buf[i] = gain_i16(out, gain);
    }
}

/// C: `int16_t calcFm(OscInfo* osc, OscInfo* modOsc, const int16_t table[][1024])`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:225`. Reference build: not linked (removed by --gc-sections). Group P1.
pub fn calcFm(osc: &mut OscInfo, modOsc: &mut OscInfo, table: &[[i16; 1024]]) -> i16 {
    // not linked in the firmware (untestable): ported from C
    let phase = osc.phase;
    let index = phase.wrapping_add(((modOsc.output as i32 as f32 * osc.fmMod) as u32) << 19);
    let flat = osc.tableOffset as usize * 1024 + (index >> 22) as usize;
    let out = interp(wt_at(table, flat), wt_at(table, flat + 1), index & 0x3FFFFF, 22);
    osc.phase = phase.wrapping_add(osc.phaseInc);
    osc.output = out;
    out
}

/// C: `IN_ITCM_RAM void calcWavetableOscBlock(OscInfo* osc, const int16_t table[][1024], int16_t* buf, const uint8_t size ,const float gain)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:450`. Reference build: out-of-line in ITCM. Group P1.
pub fn calcWavetableOscBlock(osc: &mut OscInfo, table: &[[i16; 1024]], buf: &mut [i16], size: u8, gain: f32) {
    let inc = osc.phaseInc;
    let base = osc.tableOffset as usize * 1024;
    for i in 0..size as usize {
        let phase = osc.phase;
        let flat = base + (phase >> 22) as usize;
        let out = interp(wt_at(table, flat), wt_at(table, flat + 1), phase & 0x3FFFFF, 22);
        osc.phase = phase.wrapping_add(inc);
        buf[i] = gain_i16(out, gain);
    }
}

/// C: `int16_t calcWavetableOsc(OscInfo* osc, const int16_t table[][1024])`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:475`. Reference build: not linked (removed by --gc-sections). Group P1.
pub fn calcWavetableOsc(osc: &mut OscInfo, table: &[[i16; 1024]]) -> i16 {
    // not linked in the firmware (untestable): ported from C
    let phase = osc.phase;
    let flat = osc.tableOffset as usize * 1024 + (phase >> 22) as usize;
    let out = interp(wt_at(table, flat), wt_at(table, flat + 1), phase & 0x3FFFFF, 22);
    osc.phase = phase.wrapping_add(osc.phaseInc);
    osc.output = out;
    out
}

/// C: `void calcUserSampleOscFmBlock(OscInfo* osc,int16_t* modBuffer, int16_t* buf, uint8_t size ,const float gain)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:555`. Reference build: not linked (removed by --gc-sections). Group P1.
pub fn calcUserSampleOscFmBlock(osc: &mut OscInfo, modBuffer: &[i16], buf: &mut [i16], size: u8, gain: f32) {
    // the C body is disabled (#if 0): empty function
}

/// C: `void calcUserSampleOscBlock(OscInfo* osc, int16_t* buf, const uint8_t size ,const float gain)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:592`. Reference build: not linked (removed by --gc-sections). Group P1.
pub fn calcUserSampleOscBlock(osc: &mut OscInfo, buf: &mut [i16], size: u8, gain: f32) {
    // the C body is disabled (#if 0): empty function
}

/// C: `IN_ITCM_RAM void calcPwmBlock(OscInfo* osc, int16_t* buf, uint8_t size ,const float gain, const float pwm)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:656`. Reference build: out-of-line in ITCM. Group P1.
/// Tables: `sawTable`.
/// Calls: `cmsis::__QSUB16`.
pub fn calcPwmBlock(osc: &mut OscInfo, buf: &mut [i16], size: u8, gain: f32, pwm: f32) {
    let inc = osc.phaseInc;
    let pw = pwm * f32::from_bits(0x4f000000); // pwm * (UINT32_MAX/2) as float = 2^31
    let base = osc.tableOffset as usize * 1024;
    let table = &sawTable[..];
    for i in 0..size as usize {
        let phase = osc.phase;
        let phase2 = (phase as f32 + pw) as u32;
        let out = pwm_pair(table, base, phase, phase2);
        osc.phase = phase.wrapping_add(inc);
        osc.output = out;
        buf[i] = gain_i16(out, gain);
    }
}

/// C: `void osc_calcSineFreq(OscInfo* osc)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:722`. Reference build: inlined into its callers only. Group P1.
/// Calls: `dsp::oscillator::freq2PhaseIncr`.
pub fn osc_calcSineFreq(osc: &mut OscInfo) {
    // ((freq*pitchMod)*K)*modNodeValue, in this order (osc_setFreq machine code)
    osc.phaseInc = ((osc.freq * osc.pitchMod) * K_PHASE * osc.modNodeValue) as u32;
}

/// C: `void osc_calcNoiseFreq(OscInfo* osc)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:727`. Reference build: not linked (removed by --gc-sections). Group P1.
/// Calls: `dsp::oscillator::osc_calcSineFreq`.
pub fn osc_calcNoiseFreq(osc: &mut OscInfo) {
    // not linked in the firmware (untestable): ported from C
    osc_calcSineFreq(osc);
}

/// C: `void osc_calcWavetableFreq(OscInfo* osc)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:732`. Reference build: inlined into its callers only. Group P1.
/// Calls: `dsp::oscillator::freq2PhaseIncr1024`, `dsp::oscillator::freqToTableIndex`.
pub fn osc_calcWavetableFreq(osc: &mut OscInfo) {
    let currentFreq = osc.freq * osc.pitchMod * osc.modNodeValue;
    osc.phaseInc = freq2PhaseIncr1024(currentFreq);
    let overtoneIndex = freqToTableIndex(currentFreq);
    osc.tableOffset = if overtoneIndex > 10 { 10 } else { overtoneIndex };
}

/// C: `void osc_calcSampleFreq(OscInfo* osc)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:741`. Reference build: not linked (removed by --gc-sections). Group P1.
/// Calls: `dsp::oscillator::freq2PhaseIncr32767`.
pub fn osc_calcSampleFreq(osc: &mut OscInfo) {
    // not linked in the firmware (untestable): ported from C
    let currentFreq = osc.freq * osc.pitchMod * osc.modNodeValue;
    osc.phaseInc = freq2PhaseIncr32767(currentFreq);
}

/// C: `void osc_calcUserSampleFreq(OscInfo* osc)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:747`. Reference build: inlined into its callers only. Group P1.
/// Calls: `dsp::oscillator::freq2PhaseIncr32767`.
pub fn osc_calcUserSampleFreq(osc: &mut OscInfo) {
    // as inlined in osc_setFreq: ((freq*pitchMod)*K)*modNodeValue
    osc.phaseInc = ((osc.freq * osc.pitchMod) * K_PHASE_SAMPLE * osc.modNodeValue) as u32;
}

/// C: `void osc_setFreq(OscInfo* osc)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:753`. Reference build: out-of-line in flash. Group P1.
/// Calls: `dsp::oscillator::osc_calcNoiseFreq`, `dsp::oscillator::osc_calcSineFreq`, `dsp::oscillator::osc_calcUserSampleFreq`, `dsp::oscillator::osc_calcWavetableFreq`.
pub fn osc_setFreq(osc: &mut OscInfo) {
    match osc.waveform {
        SINE | NOISE => osc_calcSineFreq(osc),
        TRI | SAW | REC | PWM => osc_calcWavetableFreq(osc),
        _ => osc_calcUserSampleFreq(osc),
    }
}

/// C: `void osc_setBaseNote(OscInfo* osc, uint8_t baseNote)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:790`. Reference build: out-of-line in flash. Group P1.
/// Tables: `MidiNoteFrequencies`.
/// Calls: `midi::midi_parser::midiParser_calcDetune`.
pub fn osc_setBaseNote(osc: &mut OscInfo, baseNote: u8) {
    let cent = crate::midi::midi_parser::midiParser_calcDetune((osc.midiFreq & 0xff) as u8);
    let note = ((osc.midiFreq >> 8) as i32 + (baseNote as i32 - SEQ_DEFAULT_NOTE as i32)).clamp(0, 127);
    osc.baseNote = baseNote;
    osc.freq = MidiNoteFrequencies[note as usize] * cent;
}

/// C: `IN_ITCM_RAM void osc_recalcFreq(OscInfo* osc)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:805`. Reference build: out-of-line in ITCM. Group P1.
/// Tables: `MidiNoteFrequencies`.
/// Calls: `midi::midi_parser::midiParser_calcDetune`.
pub fn osc_recalcFreq(osc: &mut OscInfo) {
    let cent = crate::midi::midi_parser::midiParser_calcDetune((osc.midiFreq & 0xff) as u8);
    let note = ((osc.midiFreq >> 8) as i32 + (osc.baseNote as i32 - SEQ_DEFAULT_NOTE as i32)).clamp(0, 127);
    osc.freq = MidiNoteFrequencies[note as usize] * cent;
}

// ---- functions that use globals (methods on `Lxr`)
impl Lxr {
    /// C: `IN_ITCM_RAM void calcNextOscSampleFmBlock(OscInfo* osc, int16_t* modBuffer, int16_t* buf, uint8_t size ,const float gain, const float pwm)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:250`. Reference build: out-of-line in ITCM. Group P1.
    /// Tables: `recTable`, `sawTable`, `sine_table`, `triTable`.
    /// Calls: `dsp::oscillator::calcFmBlock`, `dsp::oscillator::calcFmSineBlock`, `Lxr::calcNoiseBlock`, `dsp::oscillator::calcPwmFmBlock`, `dsp::oscillator::calcUserSampleOscFmBlock`.
    pub fn calcNextOscSampleFmBlock(&mut self, osc: &mut OscInfo, modBuffer: &[i16], buf: &mut [i16], size: u8, gain: f32, pwm: f32) {
            match osc.waveform {
            SINE => calcFmSineBlock(osc, modBuffer, buf, size, gain),
            SAW => calcFmBlock(osc, &sawTable, modBuffer, buf, size, gain),
            TRI => calcFmBlock(osc, &triTable, modBuffer, buf, size, gain),
            REC => calcFmBlock(osc, &recTable, modBuffer, buf, size, gain),
            NOISE => self.calcNoiseBlock(osc, buf, size, gain),
            PWM => calcPwmFmBlock(osc, modBuffer, buf, size, gain, pwm),
            _ => calcUserSampleOscFmBlock(osc, modBuffer, buf, size, gain),
        }
    }

    /// C: `IN_ITCM_RAM void calcNoiseBlock(OscInfo* osc, int16_t* buf, const uint8_t size ,const float gain)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:325`. Reference build: out-of-line in ITCM. Group P1.
    /// Calls: `Lxr::GetRngValue`.
    pub fn calcNoiseBlock(&mut self, osc: &mut OscInfo, buf: &mut [i16], size: u8, gain: f32) {
            let size = size as usize;
        for i in 0..size {
            let lastPhase = osc.phase;
            osc.phase = lastPhase.wrapping_add(osc.phaseInc);
            if lastPhase > osc.phase {
                // phase wrapped: new random value (int16_t)
                osc.output = self.GetRngValue() as i16;
            }
            buf[i] = gain_i16(osc.output, gain);
        }
    }

    /// C: `int16_t calcNoise(OscInfo* osc)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:343`. Reference build: not linked (removed by --gc-sections). Group P1.
    /// Calls: `Lxr::GetRngValue`, `cmsis::__get_APSR`.
    pub fn calcNoise(&mut self, osc: &mut OscInfo) -> i16 {
            // not linked in the firmware (untestable): ported from C. __get_APSR() reads the V flag of the
        // preceding `phase += phaseInc` (signed overflow of the 32 bit addition).
        let (sum, v) = (osc.phase as i32).overflowing_add(osc.phaseInc as i32);
        osc.phase = sum as u32;
        if v || osc.phase == osc.phaseInc {
            let rnd = self.GetRngValue() as u16;
            if rnd > 0x00ff {
                if rnd > 0x000f {
                    osc.output = 32767;
                } else {
                    osc.output = -32768;
                }
            }
        } else {
            osc.output = 0;
        }
        osc.output
    }

    /// C: `IN_ITCM_RAM void calcNextOscSampleBlock(OscInfo* osc, int16_t* buf, const uint8_t size, const float gain, const float pwm)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/Oscillator.c:375`. Reference build: out-of-line in ITCM. Group P1.
    /// Tables: `recTable`, `sawTable`, `sine_table`, `triTable`.
    /// Calls: `Lxr::calcNoiseBlock`, `dsp::oscillator::calcPwmBlock`, `dsp::oscillator::calcSineBlock`, `dsp::oscillator::calcWavetableOscBlock`.
    pub fn calcNextOscSampleBlock(&mut self, osc: &mut OscInfo, buf: &mut [i16], size: u8, gain: f32, pwm: f32) {
            match osc.waveform {
            SINE => calcSineBlock(osc, buf, size, gain),
            SAW => calcWavetableOscBlock(osc, &sawTable, buf, size, gain),
            TRI => calcWavetableOscBlock(osc, &triTable, buf, size, gain),
            REC => calcWavetableOscBlock(osc, &recTable, buf, size, gain),
            NOISE => self.calcNoiseBlock(osc, buf, size, gain),
            PWM => calcPwmBlock(osc, buf, size, gain, pwm),
            _ => {} // sample playback disabled in the C source
        }
    }
}

// ---- helpers reproducing the inlined machine code (not C functions)

/// `((TABLESIZE*f)/REAL_FS)*1048576` = `((1024*f)/REAL_FS)*4194304`, folded by GCC to one constant.
const K_PHASE: f32 = f32::from_bits(0x47be0ba5);
/// `((1024*f)/REAL_FS)*131072`, folded by GCC to one constant.
const K_PHASE_SAMPLE: f32 = f32::from_bits(0x453e0ba5);

/// `a + (int16_t)(((int64_t)(b - a) * frac) >> shift)`, as compiled (umull/mla, low 32 bits of the shift,
/// then 16 bit truncation of the sum).
#[inline(always)]
fn interp(a: i16, b: i16, frac: u32, shift: u32) -> i16 {
    let d = b as i32 - a as i32;
    let p = ((d as i64 * frac as i64) >> shift) as i32;
    (a as i32).wrapping_add(p) as i16
}

/// `buf[i] = oscOut * gain`: vcvt.f32.s32, vmul, vcvt.s32.f32 (saturating), store of the low 16 bits.
#[inline(always)]
fn gain_i16(v: i16, gain: f32) -> i16 {
    ((v as i32 as f32 * gain) as i32) as i16
}

/// Element `flat` of a wavetable `table[11][1024]` viewed as one flat array, as the firmware reads it
/// (`table[tableOffset][itg+1]` with `itg == 1023` reads the next row; past the last row it reads the data
/// that follows the table in `.rodata`: recTable is followed by sawTable, sawTable by triTable and triTable by
/// `CSWTCH.54`, whose first halfword is 0x0201 in both the reference ELF and the shipped v1.71 binary).
#[inline(always)]
fn wt_at(table: &[[i16; 1024]], flat: usize) -> i16 {
    let n = table.len() * 1024;
    if flat < n {
        return table[flat >> 10][flat & 1023];
    }
    let rest = flat - n;
    if core::ptr::eq(table.as_ptr(), recTable.as_ptr()) {
        wt_at(&sawTable, rest)
    } else if core::ptr::eq(table.as_ptr(), sawTable.as_ptr()) {
        wt_at(&triTable, rest)
    } else if core::ptr::eq(table.as_ptr(), triTable.as_ptr()) && rest == 0 {
        0x0201
    } else {
        panic!("wavetable read out of range (flat index {flat}); tableOffset > 10 is not reachable in the firmware")
    }
}

/// The PWM oscillator pair (calcPwmBlock / calcPwmFmBlock): two interpolated saw readouts at `i1` and `i2`,
/// combined with `__QSUB16` (signed saturating 16 bit subtraction of the low halfwords).
#[inline(always)]
fn pwm_pair(table: &[[i16; 1024]], base: usize, i1: u32, i2: u32) -> i16 {
    let f1 = base + (i1 >> 22) as usize;
    let o1 = interp(wt_at(table, f1), wt_at(table, f1 + 1), i1 & 0x3FFFFF, 22);
    let f2 = base + (i2 >> 22) as usize;
    let o2 = interp(wt_at(table, f2), wt_at(table, f2 + 1), i2 & 0x3FFFFF, 22);
    o1.saturating_sub(o2)
}
