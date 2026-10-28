//! Port of `Core/Inc/Soundengine/DSPAudio/CymbalVoice.h`, `Core/Src/Soundengine/DSPAudio/CymbalVoice.c`.
//!
//! Porting group: P4. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;

// ---- types
/// C: `struct CymbalStruct` (Core/Inc/Soundengine/DSPAudio/CymbalVoice.h:31), 384 bytes.
/// C typedef name `CymbalVoice`; the struct tag `CymbalStruct` is a type alias below.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CymbalVoice {
    /// `OscInfo osc; // the tonal oscillator`
    pub osc: OscInfo,
    /// `OscInfo modOsc; // the mod osc1`
    pub modOsc: OscInfo,
    /// `OscInfo modOsc2; // the mod osc2`
    pub modOsc2: OscInfo,
    /// `float fmModAmount1;`
    pub fmModAmount1: f32,
    /// `float fmModAmount2;`
    pub fmModAmount2: f32,
    /// `float vol; // volume of the voice`
    pub vol: f32,
    /// `uint8_t pan;`
    pub pan: u8,
    /// `float triggeredPanL; //only updated with the latest pan value when triggered`
    pub triggeredPanL: f32,
    /// `float triggeredPanR;`
    pub triggeredPanR: f32,
    /// `float velo;`
    pub velo: f32,
    /// `int32_t noiseSample;`
    pub noiseSample: i32,
    /// `ResonantFilter filter;`
    pub filter: ResonantFilter,
    /// `uint8_t filterType;`
    pub filterType: u8,
    /// `Lfo lfo;`
    pub lfo: Lfo,
    /// `TransientGenerator transGen;`
    pub transGen: TransientGenerator,
    /// `Distortion distortion;`
    pub distortion: Distortion,
    /// `SlopeEg2 oscVolEg;`
    pub oscVolEg: SlopeEg2,
    /// `float egValueOscVol;`
    pub egValueOscVol: f32,
    /// `uint8_t volumeMod; //modulate volume by midi velocity if 1`
    pub volumeMod: u8,
    /// `SnapEg snapEg;`
    pub snapEg: SnapEg,
    /// `float combinedVolume;`
    pub combinedVolume: f32,
    /// `OnePoleFilter FaderVolumeFilter;`
    pub FaderVolumeFilter: OnePoleFilter,
    /// `float pwm;`
    pub pwm: f32,
}
/// C struct tag of `CymbalVoice`.
pub type CymbalStruct = CymbalVoice;
impl CymbalVoice {
    /// All-zero value (C static zero initialisation).
    pub const ZERO: Self = Self { osc: OscInfo::ZERO, modOsc: OscInfo::ZERO, modOsc2: OscInfo::ZERO, fmModAmount1: 0.0, fmModAmount2: 0.0, vol: 0.0, pan: 0, triggeredPanL: 0.0, triggeredPanR: 0.0, velo: 0.0, noiseSample: 0, filter: ResonantFilter::ZERO, filterType: 0, lfo: Lfo::ZERO, transGen: TransientGenerator::ZERO, distortion: Distortion::ZERO, oscVolEg: SlopeEg2::ZERO, egValueOscVol: 0.0, volumeMod: 0, snapEg: SnapEg::ZERO, combinedVolume: 0.0, FaderVolumeFilter: OnePoleFilter::ZERO, pwm: 0.0 };
}
impl Default for CymbalVoice {
    fn default() -> Self {
        Self::ZERO
    }
}

// ---- functions that use globals (methods on `Lxr`)
impl Lxr {
    /// C: `void Cymbal_setPan(const uint8_t pan)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/CymbalVoice.c:26`. Reference build: out-of-line in flash. Group P4.
    /// Globals: `cymbalVoice`.
    pub fn Cymbal_setPan(&mut self, pan: u8) {
        todo!("port: Core/Src/Soundengine/DSPAudio/CymbalVoice.c:26")
    }

    /// C: `void Cymbal_init()`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/CymbalVoice.c:34`. Reference build: out-of-line in flash. Group P4.
    /// Globals: `cymbalVoice`.
    /// Calls: `Lxr::Cymbal_setPan`, `dsp::resonant_filter::SVF_init`, `dsp::snap_eg::SnapEg_init`, `dsp::one_pole_lp::initOnePole`, `Lxr::lfo_init`, `dsp::distortion::setDistortionShape`, `dsp::one_pole_lp::setOnePoleCoef`, `dsp::slope_eg2::slopeEg2_init`, `dsp::transient_generator::transient_init`.
    pub fn Cymbal_init(&mut self) {
        todo!("port: Core/Src/Soundengine/DSPAudio/CymbalVoice.c:34")
    }

    /// C: `void Cymbal_updatePan()`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/CymbalVoice.c:85`. Reference build: inlined into its callers only. Group P4.
    /// Globals: `cymbalVoice`.
    pub fn Cymbal_updatePan(&mut self) {
        todo!("port: Core/Src/Soundengine/DSPAudio/CymbalVoice.c:85")
    }

    /// C: `void Cymbal_trigger( const uint8_t vel, const uint8_t note)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/CymbalVoice.c:91`. Reference build: out-of-line in flash. Group P4.
    /// Globals: `cymbalVoice`, `velocityModulators`.
    /// Calls: `Lxr::Cymbal_updatePan`, `dsp::snap_eg::SnapEg_trigger`, `Lxr::lfo_retrigger`, `Lxr::modNode_updateValue`, `dsp::oscillator::osc_setBaseNote`, `dsp::slope_eg2::slopeEg2_trigger`, `dsp::transient_generator::transient_trigger`.
    pub fn Cymbal_trigger(&mut self, vel: u8, note: u8) {
        todo!("port: Core/Src/Soundengine/DSPAudio/CymbalVoice.c:91")
    }

    /// C: `IN_ITCM_RAM void Cymbal_calcAsync()`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/CymbalVoice.c:126`. Reference build: out-of-line in ITCM. Group P4.
    /// Globals: `cymbalVoice`.
    /// Calls: `dsp::snap_eg::SnapEg_calc`, `dsp::oscillator::osc_setFreq`, `dsp::slope_eg2::slopeEg2_calc`.
    pub fn Cymbal_calcAsync(&mut self) {
        todo!("port: Core/Src/Soundengine/DSPAudio/CymbalVoice.c:126")
    }

    /// C: `void Cymbal_calcSyncBlock(int16_t* buf, const uint8_t size)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/CymbalVoice.c:157`. Reference build: out-of-line in flash. Group P4.
    /// Globals: `cymbalVoice`.
    /// Calls: `dsp::resonant_filter::SVF_calcBlockZDF`, `cmsis::__QADD16`, `Lxr::adc_getFaderVolume`, `dsp::buffer_tools::bufferTool_addBuffersSaturating`, `dsp::distortion::calcDistBlock`, `Lxr::calcNextOscSampleBlock`, `Lxr::calcNextOscSampleFmBlock`, `dsp::one_pole_lp::calcOnePole`, `dsp::transient_generator::transient_calcBlock`.
    pub fn Cymbal_calcSyncBlock(&mut self, buf: &mut [i16], size: u8) {
        todo!("port: Core/Src/Soundengine/DSPAudio/CymbalVoice.c:157")
    }
}
