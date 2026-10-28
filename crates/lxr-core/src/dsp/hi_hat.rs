//! Port of `Core/Inc/Soundengine/DSPAudio/HiHat.h`, `Core/Src/Soundengine/DSPAudio/HiHat.c`.
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
use crate::dsp::drum_voice::{qadd16_pairs, triggered_pan, trigger_phase, RECIP_127};

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
        self.hatVoice.pan = pan;
    }

    /// C: `void HiHat_init()`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/HiHat.c:35`. Reference build: out-of-line in flash. Group P4.
    /// Globals: `hatVoice`.
    pub fn HiHat_init(&mut self) {
        let v = &mut self.hatVoice;
        initOnePole(&mut v.FaderVolumeFilter);
        setOnePoleCoef(&mut v.FaderVolumeFilter, 0.1);

        SnapEg_init(&mut v.snapEg);
        self.HiHat_setPan(0);
        let v = &mut self.hatVoice;
        v.vol = 0.8;

        transient_init(&mut v.transGen);

        v.fmModAmount1 = 0.5;
        v.fmModAmount2 = 0.5;

        setDistortionShape(&mut v.distortion, 0.0);

        v.modOsc.freq = 440.0;
        v.modOsc.waveform = SINE;
        v.modOsc.fmMod = 0.0;
        v.modOsc.midiFreq = 70 << 8 | 64;
        v.modOsc.pitchMod = 1.0;
        v.modOsc.modNodeValue = 1.0;

        v.modOsc2.freq = 440.0;
        v.modOsc2.waveform = NOISE;
        v.modOsc2.fmMod = 0.0;
        v.modOsc2.midiFreq = 70 << 8 | 64;
        v.modOsc2.pitchMod = 1.0;
        v.modOsc2.modNodeValue = 1.0;

        v.osc.freq = 440.0;
        v.osc.waveform = 1;
        v.osc.fmMod = 1.0;
        v.osc.midiFreq = 70 << 8 | 64;
        v.osc.pitchMod = 1.0;
        v.osc.modNodeValue = 1.0;

        v.volumeMod = 1;

        slopeEg2_init(&mut v.oscVolEg);

        SVF_init(&mut v.filter);

        self.lfo_init(|s| &mut s.hatVoice.lfo);
    }

    /// C: `void HiHat_updatePan()`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/HiHat.c:84`. Reference build: inlined into its callers only. Group P4.
    /// Globals: `hatVoice`.
    pub fn HiHat_updatePan(&mut self) {
        let (l, r) = triggered_pan(self.hatVoice.pan);
        self.hatVoice.triggeredPanL = l;
        self.hatVoice.triggeredPanR = r;
    }

    /// C: `void HiHat_trigger( uint8_t vel, uint8_t isOpen, const uint8_t note)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/HiHat.c:90`. Reference build: out-of-line in flash. Group P4.
    /// Globals: `hatVoice`, `velocityModulators`.
    pub fn HiHat_trigger(&mut self, vel: u8, isOpen: u8, note: u8) {
        self.lfo_retrigger(5);

        //update velocity modulation
        let velo = vel as i32 as f32 * RECIP_127;
        self.modNode_updateValue(|s| &mut s.velocityModulators[5], velo);

        let v = &mut self.hatVoice;
        let mut offset = 1.0f32;
        if v.transGen.waveform == 1 {
            offset -= v.transGen.volume;
        }
        v.osc.phase = trigger_phase(v.osc.waveform, offset);

        v.modOsc.phase = 0;
        v.modOsc2.phase = 0;

        osc_setBaseNote(&mut v.osc, note);
        osc_setBaseNote(&mut v.modOsc, note);
        osc_setBaseNote(&mut v.modOsc2, note);

        v.isOpen = isOpen;
        v.oscVolEg.decay = (if isOpen != 0 { v.decayOpen } else { v.decayClosed }) as f64;

        slopeEg2_trigger(&mut v.oscVolEg);
        v.velo = velo;
        transient_trigger(&mut v.transGen);

        SnapEg_trigger(&mut v.snapEg);

        self.HiHat_updatePan();
    }

    /// C: `IN_ITCM_RAM void HiHat_calcAsync( )`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/HiHat.c:128`. Reference build: out-of-line in ITCM. Group P4.
    /// Globals: `hatVoice`.
    pub fn HiHat_calcAsync(&mut self) {
        let v = &mut self.hatVoice;
        //calc the osc  vol eg
        v.egValueOscVol = slopeEg2_calc(&mut v.oscVolEg) as f32;

        //calc snap EG if transient sample 0 is activated
        if v.transGen.waveform == 0 {
            let snapVal = SnapEg_calc(&mut v.snapEg, v.transGen.pitch);
            v.osc.pitchMod = vfma(1.0, v.transGen.volume, snapVal);
        }

        osc_setFreq(&mut v.osc);
        osc_setFreq(&mut v.modOsc);
        osc_setFreq(&mut v.modOsc2);
    }

    /// C: `void HiHat_calcSyncBlock(int16_t* buf, const uint8_t size)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/HiHat.c:165`. Reference build: out-of-line in flash. Group P4.
    /// Globals: `hatVoice` (USE_REVERB = 0: the `fxBus.fxType` check is compiled out).
    ///
    /// The volume loop is `buf[j] = (buf[j] * fader) * combinedVolume` (GCC reassociated the C
    /// `buf[j] *= combinedVolume * fader`).
    pub fn HiHat_calcSyncBlock(&mut self, buf: &mut [i16], size: u8) {
        let sz = size as usize;
        let buf = &mut buf[..sz];
        let mut mod1Store = [0i16; 256];
        let mut mod2Store = [0i16; 256];
        let mod1 = &mut mod1Store[..sz];
        let mod2 = &mut mod2Store[..sz];

        //calc next mod osc samples, scaled with mod amount
        let mut modOsc = self.hatVoice.modOsc;
        let (a, pwm) = (self.hatVoice.fmModAmount1, self.hatVoice.pwm);
        self.calcNextOscSampleBlock(&mut modOsc, mod1, size, a, pwm);
        self.hatVoice.modOsc = modOsc;
        let mut modOsc2 = self.hatVoice.modOsc2;
        let (a, pwm) = (self.hatVoice.fmModAmount2, self.hatVoice.pwm);
        self.calcNextOscSampleBlock(&mut modOsc2, mod2, size, a, pwm);
        self.hatVoice.modOsc2 = modOsc2;

        //combine both mod oscs to 1 modulation signal
        bufferTool_addBuffersSaturating(mod1, mod2, size);

        let mut osc = self.hatVoice.osc;
        let pwm = self.hatVoice.pwm;
        self.calcNextOscSampleFmBlock(&mut osc, mod1, buf, size, 0.5, pwm);
        self.hatVoice.osc = osc;

        let v = &mut self.hatVoice;
        SVF_calcBlockZDF(&mut v.filter, v.filterType, buf, size);
        //calc transient sample
        transient_calcBlock(&mut v.transGen, mod1, size);

        v.combinedVolume = if v.volumeMod != 0 { v.vol * v.velo * v.egValueOscVol } else { v.vol * v.egValueOscVol };

        //add filter to buffer
        qadd16_pairs(buf, mod1, size);

        for j in 0..sz {
            let cv = self.hatVoice.combinedVolume;
            let fader = self.adc_getFaderVolume(5);
            let fv = calcOnePole(&mut self.hatVoice.FaderVolumeFilter, fader as f64);
            buf[j] = ((buf[j] as i32 as f32 * fv) * cv) as i32 as i16;
        }

        calcDistBlock(&self.hatVoice.distortion, buf, size);
    }
}
