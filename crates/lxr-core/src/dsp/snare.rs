//! Port of `Core/Inc/Soundengine/DSPAudio/Snare.h`, `Core/Src/Soundengine/DSPAudio/Snare.c`.
//!
//! Porting group: P4. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;

// ---- #define constants
/// `#define USE_PEAK 0` (Core/Inc/Soundengine/DSPAudio/Snare.h:31)
pub const USE_PEAK: i32 = 0;

// ---- types
/// C: `struct SnareStruct` (Core/Inc/Soundengine/DSPAudio/Snare.h:33), 368 bytes.
/// C typedef name `SnareVoice`; the struct tag `SnareStruct` is a type alias below.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SnareVoice {
    /// `OscInfo osc; // the tonal oscillator`
    pub osc: OscInfo,
    /// `OscInfo noiseOsc; // the noise osc`
    pub noiseOsc: OscInfo,
    /// `uint8_t filterType; // bit 0 = lp, bit 1 = hp, bit 3 = bp on/off`
    pub filterType: u8,
    /// `float vol; // volume of the voice`
    pub vol: f32,
    /// `uint8_t pan;`
    pub pan: u8,
    /// `float triggeredPanL; //only updated with the latest pan value when triggered`
    pub triggeredPanL: f32,
    /// `float triggeredPanR;`
    pub triggeredPanR: f32,
    /// `float mix; // defines the mix between osc and noise [0 = 100% osc: 1 = 100% noise]`
    pub mix: f32,
    /// `float velo;`
    pub velo: f32,
    /// `int16_t oscSample;`
    pub oscSample: i16,
    /// `int32_t noiseSample;`
    pub noiseSample: i32,
    /// `ResonantFilter filter;`
    pub filter: ResonantFilter,
    /// `DecayEg oscPitchEg;`
    pub oscPitchEg: DecayEg,
    /// `float egPitchModAmount;`
    pub egPitchModAmount: f32,
    /// `Lfo lfo;`
    pub lfo: Lfo,
    /// `TransientGenerator transGen;`
    pub transGen: TransientGenerator,
    /// `SlopeEg2 oscVolEg;`
    pub oscVolEg: SlopeEg2,
    /// `float egValueOscVol;`
    pub egValueOscVol: f32,
    /// `Distortion distortion;`
    pub distortion: Distortion,
    /// `SnapEg snapEg;`
    pub snapEg: SnapEg,
    /// `uint8_t volumeMod; //modulate volume by midi velocity if 1`
    pub volumeMod: u8,
    /// `float combinedVolume;`
    pub combinedVolume: f32,
    /// `OnePoleFilter FaderVolumeFilter;`
    pub FaderVolumeFilter: OnePoleFilter,
    /// `float pwm;`
    pub pwm: f32,
}
/// C struct tag of `SnareVoice`.
pub type SnareStruct = SnareVoice;
impl SnareVoice {
    /// All-zero value (C static zero initialisation).
    pub const ZERO: Self = Self { osc: OscInfo::ZERO, noiseOsc: OscInfo::ZERO, filterType: 0, vol: 0.0, pan: 0, triggeredPanL: 0.0, triggeredPanR: 0.0, mix: 0.0, velo: 0.0, oscSample: 0, noiseSample: 0, filter: ResonantFilter::ZERO, oscPitchEg: DecayEg::ZERO, egPitchModAmount: 0.0, lfo: Lfo::ZERO, transGen: TransientGenerator::ZERO, oscVolEg: SlopeEg2::ZERO, egValueOscVol: 0.0, distortion: Distortion::ZERO, snapEg: SnapEg::ZERO, volumeMod: 0, combinedVolume: 0.0, FaderVolumeFilter: OnePoleFilter::ZERO, pwm: 0.0 };
}
impl Default for SnareVoice {
    fn default() -> Self {
        Self::ZERO
    }
}

// ---- functions that use globals (methods on `Lxr`)
impl Lxr {
    /// C: `void Snare_setPan(const uint8_t pan)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/Snare.c:27`. Reference build: out-of-line in flash. Group P4.
    /// Globals: `snareVoice`.
    pub fn Snare_setPan(&mut self, pan: u8) {
        todo!("port: Core/Src/Soundengine/DSPAudio/Snare.c:27")
    }

    /// C: `void Snare_init()`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/Snare.c:34`. Reference build: out-of-line in flash. Group P4.
    /// Globals: `snareVoice`.
    /// Calls: `dsp::decay::DecayEg_init`, `dsp::resonant_filter::SVF_init`, `dsp::snap_eg::SnapEg_init`, `Lxr::Snare_setPan`, `dsp::one_pole_lp::initOnePole`, `Lxr::lfo_init`, `dsp::distortion::setDistortionShape`, `dsp::one_pole_lp::setOnePoleCoef`, `dsp::slope_eg2::slopeEg2_init`, `dsp::transient_generator::transient_init`.
    pub fn Snare_init(&mut self) {
        todo!("port: Core/Src/Soundengine/DSPAudio/Snare.c:34")
    }

    /// C: `void Snare_updatePan()`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/Snare.c:76`. Reference build: inlined into its callers only. Group P4.
    /// Globals: `snareVoice`.
    pub fn Snare_updatePan(&mut self) {
        todo!("port: Core/Src/Soundengine/DSPAudio/Snare.c:76")
    }

    /// C: `void Snare_trigger(const uint8_t vel, const uint8_t note)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/Snare.c:82`. Reference build: out-of-line in flash. Group P4.
    /// Globals: `snareVoice`, `velocityModulators`.
    /// Calls: `dsp::decay::DecayEg_trigger`, `dsp::snap_eg::SnapEg_trigger`, `Lxr::Snare_updatePan`, `Lxr::lfo_retrigger`, `Lxr::modNode_updateValue`, `dsp::oscillator::osc_setBaseNote`, `dsp::slope_eg2::slopeEg2_trigger`, `dsp::transient_generator::transient_trigger`.
    pub fn Snare_trigger(&mut self, vel: u8, note: u8) {
        todo!("port: Core/Src/Soundengine/DSPAudio/Snare.c:82")
    }

    /// C: `IN_ITCM_RAM void Snare_calcAsync()`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/Snare.c:114`. Reference build: out-of-line in ITCM. Group P4.
    /// Globals: `snareVoice`.
    /// Calls: `dsp::decay::DecayEg_calc`, `dsp::snap_eg::SnapEg_calc`, `dsp::oscillator::osc_setFreq`, `dsp::slope_eg2::slopeEg2_calc`.
    pub fn Snare_calcAsync(&mut self) {
        todo!("port: Core/Src/Soundengine/DSPAudio/Snare.c:114")
    }

    /// C: `void Snare_calcSyncBlock(int16_t* buf, const uint8_t size)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/Snare.c:148`. Reference build: out-of-line in flash. Group P4.
    /// Globals: `snareVoice`.
    /// Calls: `dsp::resonant_filter::SVF_calcBlockZDF`, `Lxr::adc_getFaderVolume`, `dsp::distortion::calcDistBlock`, `Lxr::calcNextOscSampleBlock`, `Lxr::calcNoiseBlock`, `dsp::one_pole_lp::calcOnePole`, `dsp::transient_generator::transient_calcBlock`.
    pub fn Snare_calcSyncBlock(&mut self, buf: &mut [i16], size: u8) {
        todo!("port: Core/Src/Soundengine/DSPAudio/Snare.c:148")
    }
}
