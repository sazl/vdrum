//! Port of `Core/Inc/Soundengine/DSPAudio/DrumVoice.h`, `Core/Src/Soundengine/DSPAudio/DrumVoice.c`.
//!
//! Porting group: P4. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;
use crate::dsp::buffer_tools::{bufferTool_addBuffersSaturating, bufferTool_addGain, bufferTool_addGainInterpolated};
use crate::dsp::decay::{DecayEg_calc, DecayEg_init, DecayEg_trigger};
use crate::dsp::distortion::{calcDistBlock, calcDistBlockFiltered, setDistortionShape};
use crate::dsp::one_pole_lp::{calcOnePole, initOnePole, setOnePoleCoef};
use crate::dsp::oscillator::{osc_setBaseNote, osc_setFreq};
use crate::dsp::resonant_filter::{SVF_calcBlockZDF, SVF_init, SVF_reset};
use crate::dsp::slope_eg2::{slopeEg2_calc, slopeEg2_init, slopeEg2_trigger};
use crate::dsp::snap_eg::{SnapEg_calc, SnapEg_init, SnapEg_trigger};
use crate::dsp::transient_generator::{transient_calcBlock, transient_init, transient_trigger};

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

// ---- helpers shared by the voice modules (not C functions)

/// `1/127.f` as GCC folds it (`-freciprocal-math`): literal pool word 0x3c010204.
pub(crate) const RECIP_127: f32 = f32::from_bits(0x3c01_0204);

/// `(0x3ff<<20)*offset` (`vmul` by 0x4e7fc000, `vcvt.u32.f32`): SINE start phase of snare, hihat, cymbal.
pub(crate) const PHASE_SINE_F: f32 = f32::from_bits(0x4e7f_c000);
/// `(0xff<<22)*offset` (`vmul` by 0x4e7f0000, `vcvt.u32.f32`): TRI/SAW/REC start phase.
pub(crate) const PHASE_TRI_F: f32 = f32::from_bits(0x4e7f_0000);

/// The inlined `X_updatePan()`: `triggeredPanL = sqrtf((127-pan)/127.f)`, `triggeredPanR = sqrtf(pan/127.f)`
/// (int conversion of `127-pan` and `pan`, multiplication by the reciprocal, `vsqrt`). Returns (L, R).
#[inline(always)]
pub(crate) fn triggered_pan(pan: u8) -> (f32, f32) {
    let r = (pan as i32 as f32 * RECIP_127).sqrt();
    let l = ((127 - pan as i32) as f32 * RECIP_127).sqrt();
    (l, r)
}

/// Start phase of the main oscillator on trigger in Snare/HiHat/Cymbal_trigger:
/// SINE `(0x3ff<<20)*offset`, TRI..REC `(0xff<<22)*offset`, else 0.
#[inline(always)]
pub(crate) fn trigger_phase(waveform: u8, offset: f32) -> u32 {
    if waveform == SINE {
        (offset * PHASE_SINE_F) as u32
    } else if waveform > SINE && waveform <= REC {
        (offset * PHASE_TRI_F) as u32
    } else {
        0
    }
}

/// The `__QADD16` loop over `int32_t` views of two `int16_t` buffers (`size/2` iterations), as inlined in
/// `HiHat_calcSyncBlock` / `Cymbal_calcSyncBlock` (`x[i] = __QADD16(x[i], y[i])`).
#[inline(always)]
pub(crate) fn qadd16_pairs(x: &mut [i16], y: &[i16], size: u8) {
    for i in 0..(size / 2) as usize {
        let a = (x[2 * i] as u16 as u32) | ((x[2 * i + 1] as u16 as u32) << 16);
        let b = (y[2 * i] as u16 as u32) | ((y[2 * i + 1] as u16 as u32) << 16);
        let r = crate::cmsis::__QADD16(a, b);
        x[2 * i] = r as u16 as i16;
        x[2 * i + 1] = (r >> 16) as u16 as i16;
    }
}

// ---- functions that use globals (methods on `Lxr`)
impl Lxr {
    /// C: `IN_ITCM_RAM void setPan(const uint8_t voiceNr, const uint8_t pan)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/DrumVoice.c:31`. Reference build: out-of-line in ITCM. Group P4.
    /// Globals: `voiceArray`.
    pub fn setPan(&mut self, voiceNr: u8, pan: u8) {
        self.voiceArray[voiceNr as usize].pan = pan;
    }

    /// C: `void drum_setPhase(const uint8_t phase, const uint8_t voiceNr)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/DrumVoice.c:36`. Reference build: not linked (removed by
    /// --gc-sections). Group P4. Ported from the C (`/127.f` as the reciprocal multiplication, the product with
    /// `0xffffffff` (= 4294967296.f) in single precision, `vcvt.u32.f32`); untestable.
    pub fn drum_setPhase(&mut self, phase: u8, voiceNr: u8) {
        let startPhase = ((phase as i32 as f32 * RECIP_127) * 4294967296.0f32) as u32;
        self.voiceArray[voiceNr as usize].osc.startPhase = startPhase;
        self.voiceArray[voiceNr as usize].modOsc.startPhase = startPhase;
    }

    /// C: `void initDrumVoice()`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/DrumVoice.c:43`. Reference build: out-of-line in flash. Group P4.
    /// Globals: `voiceArray`.
    /// Calls: `dsp::decay::DecayEg_init`, `dsp::resonant_filter::SVF_init`, `dsp::snap_eg::SnapEg_init`, `dsp::one_pole_lp::initOnePole`, `Lxr::lfo_init`, `dsp::distortion::setDistortionShape`, `dsp::one_pole_lp::setOnePoleCoef`, `Lxr::setPan`, `dsp::slope_eg2::slopeEg2_init`, `dsp::transient_generator::transient_init`.
    ///
    /// The static `ampSmoothValue = 0.1f` store is dead (USE_AMP_FILTER is not defined) and removed by GCC.
    pub fn initDrumVoice(&mut self) {
        for i in 0..NUM_DRUM_VOICES as usize {
            let v = &mut self.voiceArray[i];
            initOnePole(&mut v.FaderVolumeFilter);
            setOnePoleCoef(&mut v.FaderVolumeFilter, 0.01);

            // SMOOTH_MIXER_VOL_PAN_DRIVE
            for j in 0..2 {
                initOnePole(&mut v.MixerPageFilter[j]);
                setOnePoleCoef(&mut v.MixerPageFilter[j], 0.01);
            }
            initOnePole(&mut v.distortion.DriveParamFilter);
            setOnePoleCoef(&mut v.distortion.DriveParamFilter, 0.01);

            SnapEg_init(&mut v.snapEg);
            self.setPan(i as u8, 0);
            let v = &mut self.voiceArray[i];
            v.vol = 0.8;
            v.fmModAmount = 0.5;
            transient_init(&mut v.transGen);
            // ENABLE_DRUM_SVF
            SVF_init(&mut v.filter);
            v.filterType = 0x01;
            self.lfo_init(move |s| &mut s.voiceArray[i].lfo);

            let v = &mut self.voiceArray[i];
            v.modOsc.freq = 440.0;
            v.modOsc.waveform = 1;
            v.modOsc.fmMod = 0.0;
            v.modOsc.midiFreq = 70 << 8 | 64;
            v.modOsc.pitchMod = 1.0;
            v.modOsc.modNodeValue = 1.0;

            v.volumeMod = 1;

            v.osc.freq = 440.0;
            v.osc.modNodeValue = 1.0;
            v.osc.waveform = TRI + i as u8;
            v.osc.fmMod = 0.0;
            v.osc.midiFreq = 70 << 8 | 64;

            DecayEg_init(&mut v.oscPitchEg);
            v.egPitchModAmount = 0.5;

            slopeEg2_init(&mut v.oscVolEg);
            setDistortionShape(&mut v.distortion, 0.0);

            // ENABLE_MIX_OSC
            v.mixOscs = 1;
            v.decimationCnt = 0.0;
            v.decimationRate = 1.0;
        }
    }

    /// C: `void Drum_updatePan(const uint8_t voiceNr)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/DrumVoice.c:115`. Reference build: inlined into its callers only. Group P4.
    /// Globals: `voiceArray`.
    pub fn Drum_updatePan(&mut self, voiceNr: u8) {
        let v = &mut self.voiceArray[voiceNr as usize];
        let (l, r) = triggered_pan(v.pan);
        v.triggeredPanL = l;
        v.triggeredPanR = r;
    }

    /// C: `void Drum_trigger(const uint8_t voiceNr, const uint8_t vol, const uint8_t note)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/DrumVoice.c:121`. Reference build: out-of-line in flash. Group P4.
    /// Globals: `velocityModulators`, `voiceArray`.
    /// Calls: `dsp::decay::DecayEg_trigger`, `Lxr::Drum_updatePan`, `dsp::resonant_filter::SVF_reset`, `dsp::snap_eg::SnapEg_trigger`, `Lxr::lfo_retrigger`, `Lxr::modNode_updateValue`, `dsp::oscillator::osc_setBaseNote`, `dsp::slope_eg2::slopeEg2_trigger`, `dsp::transient_generator::transient_trigger`.
    pub fn Drum_trigger(&mut self, voiceNr: u8, vol: u8, note: u8) {
        let n = voiceNr as usize;
        self.lfo_retrigger(voiceNr);

        //update velocity modulation
        let velo = vol as i32 as f32 * RECIP_127;
        self.modNode_updateValue(move |s| &mut s.velocityModulators[n], velo);

        let v = &mut self.voiceArray[n];
        let mut offset = 1.0f32;
        if v.transGen.waveform == 1 {
            offset -= v.transGen.volume;
        }
        v.osc.phase = if v.osc.waveform == SINE {
            // 1024 + ((0x3ff<<20) - 1024)*offset, fused: vfma(1024, offset, 1072692224)
            vfma(1024.0, offset, f32::from_bits(0x4e7f_bff0)) as u32
        } else if v.osc.waveform > SINE && v.osc.waveform <= REC {
            (offset * PHASE_TRI_F) as u32
        } else {
            0
        };

        //reset mod osc phase
        v.modOsc.phase = if v.modOsc.waveform == SINE {
            0x3ff0_0000
        } else if v.modOsc.waveform > SINE && v.modOsc.waveform <= REC {
            0x3fc0_0000
        } else {
            0
        };

        osc_setBaseNote(&mut v.osc, note);
        osc_setBaseNote(&mut v.modOsc, note);

        DecayEg_trigger(&mut v.oscPitchEg);
        slopeEg2_trigger(&mut v.oscVolEg);
        v.velo = velo;

        transient_trigger(&mut v.transGen);

        SnapEg_trigger(&mut v.snapEg);

        //reset filter coeffs to prevent wrong transient
        SVF_reset(&mut v.filter);

        //update pan value
        self.Drum_updatePan(voiceNr);
    }

    /// C: `IN_ITCM_RAM void calcDrumVoiceAsync(const uint8_t voiceNr)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/DrumVoice.c:183`. Reference build: out-of-line in ITCM. Group P4.
    /// Globals: `voiceArray`.
    /// Calls: `dsp::decay::DecayEg_calc`, `dsp::snap_eg::SnapEg_calc`, `dsp::oscillator::osc_setFreq`, `dsp::slope_eg2::slopeEg2_calc`.
    pub fn calcDrumVoiceAsync(&mut self, voiceNr: u8) {
        let v = &mut self.voiceArray[voiceNr as usize];

        let egPitchVal = DecayEg_calc(&mut v.oscPitchEg);
        v.osc.pitchMod = vfma(1.0, v.egPitchModAmount, egPitchVal);

        //calc snap EG if transient sample 0 is activated
        if v.transGen.waveform == 0 {
            let snapVal = SnapEg_calc(&mut v.snapEg, v.transGen.pitch);
            v.osc.pitchMod = vfma(v.osc.pitchMod, v.transGen.volume, snapVal);
        }

        // fm amount with pitch eg
        v.osc.fmMod = v.fmModAmount * egPitchVal;

        //check if in attack phase (`state & 0xfd == 1`: EG_A or EG_REPEAT)
        if v.oscVolEg.attack == 1.0 && (v.oscVolEg.state as i32 == EG_A || v.oscVolEg.state as i32 == EG_REPEAT) {
            v.ampFilterInput = slopeEg2_calc(&mut v.oscVolEg);
            v.lastGain = v.ampFilterInput;
        } else {
            v.lastGain = v.ampFilterInput;
            v.ampFilterInput = slopeEg2_calc(&mut v.oscVolEg);
        }

        //update osc phaseInc
        osc_setFreq(&mut v.osc);
        osc_setFreq(&mut v.modOsc);
    }

    /// C: `IN_ITCM_RAM void calcDrumVoiceSyncBlock(const uint8_t voiceNr, int16_t* buf, const uint8_t size)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/DrumVoice.c:239`. Reference build: out-of-line in ITCM. Group P4.
    /// Globals: `voiceArray`.
    /// Calls: `dsp::resonant_filter::SVF_calcBlockZDF`, `Lxr::adc_getFaderVolume`, `dsp::buffer_tools::bufferTool_addBuffersSaturating`, `dsp::buffer_tools::bufferTool_addGain`, `dsp::buffer_tools::bufferTool_addGainInterpolated`, `dsp::distortion::calcDistBlockFiltered`, `Lxr::calcNextOscSampleBlock`, `Lxr::calcNextOscSampleFmBlock`, `dsp::one_pole_lp::calcOnePole`, `dsp::transient_generator::transient_calcBlock`.
    pub fn calcDrumVoiceSyncBlock(&mut self, voiceNr: u8, buf: &mut [i16], size: u8) {
        let n = voiceNr as usize;
        let sz = size as usize;
        let buf = &mut buf[..sz];
        let mut modBufStore = [0i16; 256];
        let modBuf = &mut modBufStore[..sz];

        //calc next mod osc sampleBlock - int16_t processing
        let mut modOsc = self.voiceArray[n].modOsc;
        let (fmModAmount, pwm) = (self.voiceArray[n].fmModAmount, self.voiceArray[n].pwm);
        self.calcNextOscSampleBlock(&mut modOsc, modBuf, size, fmModAmount, pwm);
        self.voiceArray[n].modOsc = modOsc;

        let mut osc = self.voiceArray[n].osc;
        if self.voiceArray[n].mixOscs != 0 {
            //calc main osc buffer - int16_t processing
            let (fmModAmount, pwm) = (self.voiceArray[n].fmModAmount, self.voiceArray[n].pwm);
            self.calcNextOscSampleBlock(&mut osc, buf, size, 1.0 - fmModAmount, pwm);
            self.voiceArray[n].osc = osc;
            //add mod buffer to main osc buffer - int16_t processing
            bufferTool_addBuffersSaturating(buf, modBuf, size);
        } else {
            let pwm = self.voiceArray[n].pwm;
            self.calcNextOscSampleFmBlock(&mut osc, modBuf, buf, size, 1.0, pwm);
            self.voiceArray[n].osc = osc;
        }

        let v = &mut self.voiceArray[n];
        //calc transient sample - int16_t processing
        transient_calcBlock(&mut v.transGen, modBuf, size);

        //Mix with transient buffer - int16_t processing
        bufferTool_addBuffersSaturating(buf, modBuf, size);

        //calc filter block - float processing
        SVF_calcBlockZDF(&mut v.filter, v.filterType, buf, size);

        //attentuate main OSCs by amp EG
        bufferTool_addGainInterpolated(buf, v.ampFilterInput as f32, v.lastGain as f32, size);

        //MIDI velocity - float processing
        if v.volumeMod != 0 {
            bufferTool_addGain(buf, v.velo, size);
        }
        //distortion - float processing (SMOOTH_MIXER_VOL_PAN_DRIVE)
        calcDistBlockFiltered(&mut v.distortion, buf, size);

        for i in 0..sz {
            let vol = self.voiceArray[n].vol;
            let mixVol = calcOnePole(&mut self.voiceArray[n].MixerPageFilter[0], vol as f64);
            let fader = self.adc_getFaderVolume(voiceNr);
            let fv = calcOnePole(&mut self.voiceArray[n].FaderVolumeFilter, fader as f64);
            let cv = fv * mixVol;
            self.voiceArray[n].combinedVolume = cv;
            buf[i] = (buf[i] as i32 as f32 * cv) as i32 as i16;
        }
    }
}
