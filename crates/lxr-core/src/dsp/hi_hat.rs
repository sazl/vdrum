//! Port of `Core/Inc/Soundengine/DSPAudio/HiHat.h`, `Core/Src/Soundengine/DSPAudio/HiHat.c`.
//!
//! Porting group: P4. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;

// ---- types
/// C: `struct HiHatStruct` (Core/Inc/Soundengine/DSPAudio/HiHat.h:33), 392 bytes.
/// C typedef name `HiHatVoice`; the struct tag `HiHatStruct` is a type alias below.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HiHatVoice {
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
    /// `TransientGenerator transGen;`
    pub transGen: TransientGenerator,
    /// `float decayClosed;`
    pub decayClosed: f32,
    /// `float decayOpen;`
    pub decayOpen: f32,
    /// `Distortion distortion;`
    pub distortion: Distortion,
    /// `SlopeEg2 oscVolEg;`
    pub oscVolEg: SlopeEg2,
    /// `float egValueOscVol;`
    pub egValueOscVol: f32,
    /// `Lfo lfo;`
    pub lfo: Lfo,
    /// `SnapEg snapEg;`
    pub snapEg: SnapEg,
    /// `uint8_t volumeMod; //modulate volume by midi velocity if 1`
    pub volumeMod: u8,
    /// `uint8_t isOpen; //is 1 if urrent playing voice is open hihat, 0 for closed`
    pub isOpen: u8,
    /// `float combinedVolume;`
    pub combinedVolume: f32,
    /// `OnePoleFilter FaderVolumeFilter;`
    pub FaderVolumeFilter: OnePoleFilter,
    /// `float pwm;`
    pub pwm: f32,
}
/// C struct tag of `HiHatVoice`.
pub type HiHatStruct = HiHatVoice;
impl HiHatVoice {
    /// All-zero value (C static zero initialisation).
    pub const ZERO: Self = Self { osc: OscInfo::ZERO, modOsc: OscInfo::ZERO, modOsc2: OscInfo::ZERO, fmModAmount1: 0.0, fmModAmount2: 0.0, vol: 0.0, pan: 0, triggeredPanL: 0.0, triggeredPanR: 0.0, velo: 0.0, noiseSample: 0, filter: ResonantFilter::ZERO, filterType: 0, transGen: TransientGenerator::ZERO, decayClosed: 0.0, decayOpen: 0.0, distortion: Distortion::ZERO, oscVolEg: SlopeEg2::ZERO, egValueOscVol: 0.0, lfo: Lfo::ZERO, snapEg: SnapEg::ZERO, volumeMod: 0, isOpen: 0, combinedVolume: 0.0, FaderVolumeFilter: OnePoleFilter::ZERO, pwm: 0.0 };
}
impl Default for HiHatVoice {
    fn default() -> Self {
        Self::ZERO
    }
}

// ---- functions that use globals (methods on `Lxr`)
impl Lxr {
    /// C: `void HiHat_setPan(const uint8_t pan)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/HiHat.c:27`. Reference build: out-of-line in flash. Group P4.
    /// Globals: `hatVoice`.
    pub fn HiHat_setPan(&mut self, pan: u8) {
        todo!("port: Core/Src/Soundengine/DSPAudio/HiHat.c:27")
    }

    /// C: `void HiHat_init()`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/HiHat.c:35`. Reference build: out-of-line in flash. Group P4.
    /// Globals: `hatVoice`.
    /// Calls: `Lxr::HiHat_setPan`, `dsp::resonant_filter::SVF_init`, `dsp::snap_eg::SnapEg_init`, `dsp::one_pole_lp::initOnePole`, `Lxr::lfo_init`, `dsp::distortion::setDistortionShape`, `dsp::one_pole_lp::setOnePoleCoef`, `dsp::slope_eg2::slopeEg2_init`, `dsp::transient_generator::transient_init`.
    pub fn HiHat_init(&mut self) {
        todo!("port: Core/Src/Soundengine/DSPAudio/HiHat.c:35")
    }

    /// C: `void HiHat_updatePan()`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/HiHat.c:84`. Reference build: inlined into its callers only. Group P4.
    /// Globals: `hatVoice`.
    pub fn HiHat_updatePan(&mut self) {
        todo!("port: Core/Src/Soundengine/DSPAudio/HiHat.c:84")
    }

    /// C: `void HiHat_trigger( uint8_t vel, uint8_t isOpen, const uint8_t note)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/HiHat.c:90`. Reference build: out-of-line in flash. Group P4.
    /// Globals: `hatVoice`, `velocityModulators`.
    /// Calls: `Lxr::HiHat_updatePan`, `dsp::snap_eg::SnapEg_trigger`, `Lxr::lfo_retrigger`, `Lxr::modNode_updateValue`, `dsp::oscillator::osc_setBaseNote`, `dsp::slope_eg2::slopeEg2_trigger`, `dsp::transient_generator::transient_trigger`.
    pub fn HiHat_trigger(&mut self, vel: u8, isOpen: u8, note: u8) {
        todo!("port: Core/Src/Soundengine/DSPAudio/HiHat.c:90")
    }

    /// C: `IN_ITCM_RAM void HiHat_calcAsync( )`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/HiHat.c:128`. Reference build: out-of-line in ITCM. Group P4.
    /// Globals: `hatVoice`.
    /// Calls: `dsp::snap_eg::SnapEg_calc`, `dsp::oscillator::osc_setFreq`, `dsp::slope_eg2::slopeEg2_calc`.
    pub fn HiHat_calcAsync(&mut self) {
        todo!("port: Core/Src/Soundengine/DSPAudio/HiHat.c:128")
    }

    /// C: `void HiHat_calcSyncBlock(int16_t* buf, const uint8_t size)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/HiHat.c:165`. Reference build: out-of-line in flash. Group P4.
    /// Globals: `hatVoice`.
    /// Calls: `dsp::resonant_filter::SVF_calcBlockZDF`, `cmsis::__QADD16`, `Lxr::adc_getFaderVolume`, `dsp::buffer_tools::bufferTool_addBuffersSaturating`, `dsp::distortion::calcDistBlock`, `Lxr::calcNextOscSampleBlock`, `Lxr::calcNextOscSampleFmBlock`, `dsp::one_pole_lp::calcOnePole`, `dsp::transient_generator::transient_calcBlock`.
    pub fn HiHat_calcSyncBlock(&mut self, buf: &mut [i16], size: u8) {
        todo!("port: Core/Src/Soundengine/DSPAudio/HiHat.c:165")
    }
}
