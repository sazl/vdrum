//! Port of `Core/Inc/Soundengine/DSPAudio/Snare.h`, `Core/Src/Soundengine/DSPAudio/Snare.c`.
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
use crate::dsp::drum_voice::{triggered_pan, trigger_phase, RECIP_127};

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
        self.snareVoice.pan = pan;
    }

    /// C: `void Snare_init()`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/Snare.c:34`. Reference build: out-of-line in flash. Group P4.
    /// Globals: `snareVoice`.
    pub fn Snare_init(&mut self) {
        let v = &mut self.snareVoice;
        initOnePole(&mut v.FaderVolumeFilter);
        setOnePoleCoef(&mut v.FaderVolumeFilter, 0.1);

        SnapEg_init(&mut v.snapEg);
        self.Snare_setPan(0);
        let v = &mut self.snareVoice;
        v.vol = 0.8;

        v.noiseOsc.freq = 440.0;
        v.noiseOsc.waveform = 1;
        v.noiseOsc.fmMod = 0.0;
        v.noiseOsc.midiFreq = 70 << 8 | 64;
        v.noiseOsc.pitchMod = 1.0;
        v.noiseOsc.modNodeValue = 1.0;

        v.osc.freq = 440.0;
        v.osc.waveform = 1;
        v.osc.fmMod = 0.0;
        v.osc.midiFreq = 70 << 8 | 64;
        v.osc.modNodeValue = 1.0;

        setDistortionShape(&mut v.distortion, 0.0);

        v.volumeMod = 1;

        transient_init(&mut v.transGen);

        DecayEg_init(&mut v.oscPitchEg);
        v.egPitchModAmount = 0.5;

        slopeEg2_init(&mut v.oscVolEg);

        SVF_init(&mut v.filter);

        self.lfo_init(|s| &mut s.snareVoice.lfo);
    }

    /// C: `void Snare_updatePan()`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/Snare.c:76`. Reference build: inlined into its callers only. Group P4.
    /// Globals: `snareVoice`.
    pub fn Snare_updatePan(&mut self) {
        let (l, r) = triggered_pan(self.snareVoice.pan);
        self.snareVoice.triggeredPanL = l;
        self.snareVoice.triggeredPanR = r;
    }

    /// C: `void Snare_trigger(const uint8_t vel, const uint8_t note)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/Snare.c:82`. Reference build: out-of-line in flash. Group P4.
    /// Globals: `snareVoice`, `velocityModulators`.
    pub fn Snare_trigger(&mut self, vel: u8, note: u8) {
        self.lfo_retrigger(3);
        //update velocity modulation
        let velo = vel as i32 as f32 * RECIP_127;
        self.modNode_updateValue(|s| &mut s.velocityModulators[3], velo);

        let v = &mut self.snareVoice;
        let mut offset = 1.0f32;
        if v.transGen.waveform == 1 {
            offset -= v.transGen.volume;
        }
        v.osc.phase = trigger_phase(v.osc.waveform, offset);

        DecayEg_trigger(&mut v.oscPitchEg);
        slopeEg2_trigger(&mut v.oscVolEg);
        v.velo = velo;

        osc_setBaseNote(&mut v.osc, note);

        transient_trigger(&mut v.transGen);

        SnapEg_trigger(&mut v.snapEg);

        self.Snare_updatePan();
    }

    /// C: `IN_ITCM_RAM void Snare_calcAsync()`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/Snare.c:114`. Reference build: out-of-line in ITCM. Group P4.
    /// Globals: `snareVoice`.
    pub fn Snare_calcAsync(&mut self) {
        let v = &mut self.snareVoice;
        let egPitchVal = DecayEg_calc(&mut v.oscPitchEg);
        v.osc.pitchMod = vfma(1.0, v.egPitchModAmount, egPitchVal);

        //calc the osc  vol eg
        v.egValueOscVol = slopeEg2_calc(&mut v.oscVolEg) as f32;

        //calc snap EG if transient sample 0 is activated
        if v.transGen.waveform == 0 {
            let snapVal = SnapEg_calc(&mut v.snapEg, v.transGen.pitch);
            v.osc.pitchMod = vfma(v.osc.pitchMod, v.transGen.volume, snapVal);
        }

        osc_setFreq(&mut v.osc);
        osc_setFreq(&mut v.noiseOsc);
    }

    /// C: `void Snare_calcSyncBlock(int16_t* buf, const uint8_t size)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/Snare.c:148`. Reference build: out-of-line in flash. Group P4.
    /// Globals: `snareVoice` (USE_REVERB = 0: the `fxBus.fxType` check is compiled out).
    ///
    /// The sample loop is `buf[j] = (fader * tmp) * combinedVolume` with
    /// `tmp = trans + fmaf(buf, mix, osc)`: GCC reassociated the C product `combinedVolume * tmp * fader`.
    pub fn Snare_calcSyncBlock(&mut self, buf: &mut [i16], size: u8) {
        let sz = size as usize;
        let buf = &mut buf[..sz];
        let mut transStore = [0i16; 256];
        let mut oscStore = [0i16; 256];
        let transBuf = &mut transStore[..sz];
        let oscBuf = &mut oscStore[..sz];

        let mut noiseOsc = self.snareVoice.noiseOsc;
        self.calcNoiseBlock(&mut noiseOsc, buf, size, f32::from_bits(0x3f66_6666));
        self.snareVoice.noiseOsc = noiseOsc;

        let v = &mut self.snareVoice;
        SVF_calcBlockZDF(&mut v.filter, v.filterType, buf, size);

        //calc next osc sample
        let mut osc = self.snareVoice.osc;
        let (mix, pwm) = (self.snareVoice.mix, self.snareVoice.pwm);
        self.calcNextOscSampleBlock(&mut osc, oscBuf, size, 1.0 - mix, pwm);
        self.snareVoice.osc = osc;

        //calc transient sample
        transient_calcBlock(&mut self.snareVoice.transGen, transBuf, size);

        let v = &self.snareVoice;
        self.snareVoice.combinedVolume = if v.volumeMod != 0 {
            v.vol * v.velo * v.egValueOscVol
        } else {
            v.vol * v.egValueOscVol
        };

        for j in 0..sz {
            let cv = self.snareVoice.combinedVolume;
            let mix = self.snareVoice.mix;
            let tmp = vfma(oscBuf[j] as i32 as f32, buf[j] as i32 as f32, mix);
            let tmp = transBuf[j] as i32 as f32 + tmp;
            let fader = self.adc_getFaderVolume(3);
            let fv = calcOnePole(&mut self.snareVoice.FaderVolumeFilter, fader as f64);
            buf[j] = ((fv * tmp) * cv) as i32 as i16;
        }

        calcDistBlock(&self.snareVoice.distortion, buf, size);
    }
}
