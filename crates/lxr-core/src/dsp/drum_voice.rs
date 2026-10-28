//! Port of `Core/Inc/Soundengine/DSPAudio/DrumVoice.h`, `Core/Src/Soundengine/DSPAudio/DrumVoice.c`.
//!
//! Porting group: P4. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;

// ---- enums
// C: `enum {...}` (Core/Inc/Soundengine/DSPAudio/DrumVoice.h:32), 1 bytes, underlying type `u8`. (anonymous: enumerators only)
pub const IS_PLAYING: u8 = 1;

// ---- types
/// C: `struct VoiceStruct` (Core/Inc/Soundengine/DSPAudio/DrumVoice.h:36), 704 bytes.
/// C typedef name `DrumVoice`; the struct tag `VoiceStruct` is a type alias below.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DrumVoice {
    /// `OscInfo osc; // the main fm oscillator`
    pub osc: OscInfo,
    /// `OscInfo modOsc; // the mod osc`
    pub modOsc: OscInfo,
    /// `float fmModAmount;`
    pub fmModAmount: f32,
    /// `float vol; // volume of the voice`
    pub vol: f32,
    /// `float velo;`
    pub velo: f32,
    /// `uint8_t pan;`
    pub pan: u8,
    /// `float triggeredPanL; //only updated with the latest pan value when triggered`
    pub triggeredPanL: f32,
    /// `float triggeredPanR;`
    pub triggeredPanR: f32,
    /// `int16_t oscSample;`
    pub oscSample: i16,
    /// `DecayEg oscPitchEg;`
    pub oscPitchEg: DecayEg,
    /// `float egPitchModAmount;`
    pub egPitchModAmount: f32,
    /// `float offset; //the start offset for the waveform - used to generate clicks`
    pub offset: f32,
    /// `TransientGenerator transGen;`
    pub transGen: TransientGenerator,
    /// `Lfo lfo;`
    pub lfo: Lfo,
    /// `SlopeEg2 oscVolEg;`
    pub oscVolEg: SlopeEg2,
    /// `float egValueOscVol;`
    pub egValueOscVol: f32,
    /// `float volEgValueBlock[OUTPUT_DMA_SIZE];`
    pub volEgValueBlock: [f32; 64],
    /// `Distortion distortion;`
    pub distortion: Distortion,
    /// `ResonantFilter filter;`
    pub filter: ResonantFilter,
    /// `uint8_t filterType; // bit 0 = lp, bit 1 = hp, bit 3 = bp on/off`
    pub filterType: u8,
    /// `double ampFilterInput;`
    pub ampFilterInput: f64,
    /// `double lastGain;`
    pub lastGain: f64,
    /// `uint8_t mixOscs;`
    pub mixOscs: u8,
    /// `float decimationCnt;`
    pub decimationCnt: f32,
    /// `float decimationRate;`
    pub decimationRate: f32,
    /// `SnapEg snapEg;`
    pub snapEg: SnapEg,
    /// `uint8_t volumeMod; //modulate volume by midi velocity if 1`
    pub volumeMod: u8,
    /// `Dither dither;`
    pub dither: Dither,
    /// `float combinedVolume;`
    pub combinedVolume: f32,
    /// `OnePoleFilter FaderVolumeFilter;`
    pub FaderVolumeFilter: OnePoleFilter,
    /// `OnePoleFilter MixerPageFilter[2];`
    pub MixerPageFilter: [OnePoleFilter; 2],
    /// `float pwm;`
    pub pwm: f32,
}
/// C struct tag of `DrumVoice`.
pub type VoiceStruct = DrumVoice;
impl DrumVoice {
    /// All-zero value (C static zero initialisation).
    pub const ZERO: Self = Self { osc: OscInfo::ZERO, modOsc: OscInfo::ZERO, fmModAmount: 0.0, vol: 0.0, velo: 0.0, pan: 0, triggeredPanL: 0.0, triggeredPanR: 0.0, oscSample: 0, oscPitchEg: DecayEg::ZERO, egPitchModAmount: 0.0, offset: 0.0, transGen: TransientGenerator::ZERO, lfo: Lfo::ZERO, oscVolEg: SlopeEg2::ZERO, egValueOscVol: 0.0, volEgValueBlock: [0.0; 64], distortion: Distortion::ZERO, filter: ResonantFilter::ZERO, filterType: 0, ampFilterInput: 0.0, lastGain: 0.0, mixOscs: 0, decimationCnt: 0.0, decimationRate: 0.0, snapEg: SnapEg::ZERO, volumeMod: 0, dither: Dither::ZERO, combinedVolume: 0.0, FaderVolumeFilter: OnePoleFilter::ZERO, MixerPageFilter: [OnePoleFilter::ZERO; 2], pwm: 0.0 };
}
impl Default for DrumVoice {
    fn default() -> Self {
        Self::ZERO
    }
}

// ---- functions that use globals (methods on `Lxr`)
impl Lxr {
    /// C: `IN_ITCM_RAM void setPan(const uint8_t voiceNr, const uint8_t pan)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/DrumVoice.c:31`. Reference build: out-of-line in ITCM. Group P4.
    /// Globals: `voiceArray`.
    pub fn setPan(&mut self, voiceNr: u8, pan: u8) {
        todo!("port: Core/Src/Soundengine/DSPAudio/DrumVoice.c:31")
    }

    /// C: `void drum_setPhase(const uint8_t phase, const uint8_t voiceNr)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/DrumVoice.c:36`. Reference build: not linked (removed by --gc-sections). Group P4.
    /// Globals: `voiceArray`.
    pub fn drum_setPhase(&mut self, phase: u8, voiceNr: u8) {
        todo!("port: Core/Src/Soundengine/DSPAudio/DrumVoice.c:36")
    }

    /// C: `void initDrumVoice()`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/DrumVoice.c:43`. Reference build: out-of-line in flash. Group P4.
    /// Globals: `voiceArray`.
    /// Calls: `dsp::decay::DecayEg_init`, `dsp::resonant_filter::SVF_init`, `dsp::snap_eg::SnapEg_init`, `dsp::one_pole_lp::initOnePole`, `Lxr::lfo_init`, `dsp::distortion::setDistortionShape`, `dsp::one_pole_lp::setOnePoleCoef`, `Lxr::setPan`, `dsp::slope_eg2::slopeEg2_init`, `dsp::transient_generator::transient_init`.
    pub fn initDrumVoice(&mut self) {
        todo!("port: Core/Src/Soundengine/DSPAudio/DrumVoice.c:43")
    }

    /// C: `void Drum_updatePan(const uint8_t voiceNr)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/DrumVoice.c:115`. Reference build: inlined into its callers only. Group P4.
    /// Globals: `voiceArray`.
    pub fn Drum_updatePan(&mut self, voiceNr: u8) {
        todo!("port: Core/Src/Soundengine/DSPAudio/DrumVoice.c:115")
    }

    /// C: `void Drum_trigger(const uint8_t voiceNr, const uint8_t vol, const uint8_t note)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/DrumVoice.c:121`. Reference build: out-of-line in flash. Group P4.
    /// Globals: `velocityModulators`, `voiceArray`.
    /// Calls: `dsp::decay::DecayEg_trigger`, `Lxr::Drum_updatePan`, `dsp::resonant_filter::SVF_reset`, `dsp::snap_eg::SnapEg_trigger`, `Lxr::lfo_retrigger`, `Lxr::modNode_updateValue`, `dsp::oscillator::osc_setBaseNote`, `dsp::slope_eg2::slopeEg2_trigger`, `dsp::transient_generator::transient_trigger`.
    pub fn Drum_trigger(&mut self, voiceNr: u8, vol: u8, note: u8) {
        todo!("port: Core/Src/Soundengine/DSPAudio/DrumVoice.c:121")
    }

    /// C: `IN_ITCM_RAM void calcDrumVoiceAsync(const uint8_t voiceNr)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/DrumVoice.c:183`. Reference build: out-of-line in ITCM. Group P4.
    /// Globals: `voiceArray`.
    /// Calls: `dsp::decay::DecayEg_calc`, `dsp::snap_eg::SnapEg_calc`, `dsp::oscillator::osc_setFreq`, `dsp::slope_eg2::slopeEg2_calc`.
    pub fn calcDrumVoiceAsync(&mut self, voiceNr: u8) {
        todo!("port: Core/Src/Soundengine/DSPAudio/DrumVoice.c:183")
    }

    /// C: `IN_ITCM_RAM void calcDrumVoiceSyncBlock(const uint8_t voiceNr, int16_t* buf, const uint8_t size)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/DrumVoice.c:239`. Reference build: out-of-line in ITCM. Group P4.
    /// Globals: `voiceArray`.
    /// Calls: `dsp::resonant_filter::SVF_calcBlockZDF`, `Lxr::adc_getFaderVolume`, `dsp::buffer_tools::bufferTool_addBuffersSaturating`, `dsp::buffer_tools::bufferTool_addGain`, `dsp::buffer_tools::bufferTool_addGainInterpolated`, `dsp::distortion::calcDistBlockFiltered`, `Lxr::calcNextOscSampleBlock`, `Lxr::calcNextOscSampleFmBlock`, `dsp::one_pole_lp::calcOnePole`, `dsp::transient_generator::transient_calcBlock`.
    pub fn calcDrumVoiceSyncBlock(&mut self, voiceNr: u8, buf: &mut [i16], size: u8) {
        todo!("port: Core/Src/Soundengine/DSPAudio/DrumVoice.c:239")
    }
}
