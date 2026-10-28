//! Port of `Core/Inc/Soundengine/DSPAudio/mixer.h`, `Core/Src/Soundengine/DSPAudio/mixer.c`.
//!
//! Porting group: P4. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;
use crate::cmsis::__QADD16;
use crate::dsp::buffer_tools::bufferTool_clearBuffer;
use crate::dsp::one_pole_lp::calcOnePole;
use crate::dsp::resonant_filter::SVF_recalcFreq;

// ---- #define constants
/// `#define USE_SWITCH_ROUTING 1` (Core/Inc/Soundengine/DSPAudio/mixer.h:20)
pub const USE_SWITCH_ROUTING: i32 = 1;
/// `#define USE_DECIMATOR 1` (Core/Inc/Soundengine/DSPAudio/mixer.h:21)
pub const USE_DECIMATOR: i32 = 1;
/// `#define USE_TRIGGERED_PAN 1` (Core/Inc/Soundengine/DSPAudio/mixer.h:22)
pub const USE_TRIGGERED_PAN: i32 = 1;

// ---- enums
// C: `enum {...}` (Core/Inc/Soundengine/DSPAudio/mixer.h:31), 1 bytes, underlying type `u8`. (anonymous: enumerators only)
pub const MIXER_ROUTING_DAC1_STEREO: u8 = 0;
pub const MIXER_ROUTING_DAC2_STEREO: u8 = 1;
pub const MIXER_ROUTING_DAC1_L: u8 = 2;
pub const MIXER_ROUTING_DAC1_R: u8 = 3;
pub const MIXER_ROUTING_DAC2_L: u8 = 4;
pub const MIXER_ROUTING_DAC2_R: u8 = 5;
pub const MIXER_ROUTING_FX_BUS: u8 = 6;

// ---- helpers (not C functions)

/// `1/127.f` as GCC folds it (`-freciprocal-math`): literal pool word 0x3c010204.
const RECIP_127: f32 = f32::from_bits(0x3c01_0204);

/// One `int32_t` of an interleaved `int16_t` buffer (little endian): `*(int32_t*)&buf[i]`.
#[inline(always)]
fn ld32(buf: &[i16], i: usize) -> u32 {
    (buf[i] as u16 as u32) | ((buf[i + 1] as u16 as u32) << 16)
}

/// `*(int32_t*)&buf[i] = v`.
#[inline(always)]
fn st32(buf: &mut [i16], i: usize, v: u32) {
    buf[i] = v as u16 as i16;
    buf[i + 1] = (v >> 16) as u16 as i16;
}

/// `__PKHBT((int16_t)(d * panL), (int16_t)(d * panR), 16)`: `vcvt.s32.f32` of each product, `uxth` of the
/// first, second shifted into the top half.
#[inline(always)]
fn pack_pan(d: i16, panL: f32, panR: f32) -> u32 {
    let x = d as i32 as f32;
    let l = (panL * x) as i32;
    let r = (panR * x) as i32;
    (l as u32 & 0xffff) | ((r as u32) << 16)
}

/// `*out = __QADD16(*out, data)` on one `int16_t` (both operands sign-extended by `ldrsh`, low half stored).
#[inline(always)]
fn qadd_mono(out: i16, data: i16) -> i16 {
    __QADD16(out as i32 as u32, data as i32 as u32) as u16 as i16
}

// ---- functions that use globals (methods on `Lxr`)
impl Lxr {
    /// C: `void mixer_init()`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/mixer.c:43`. Reference build: out-of-line in flash. Group P4.
    /// Globals: `mixer_audioRouting`, `mixer_decimation_cnt`, `mixer_decimation_rate`, `mixer_voice_samples`.
    /// Calls: `Lxr::FxBus_init`.
    pub fn mixer_init(&mut self) {
        for i in 0..6 {
            self.mixer_decimation_rate[i] = 1.0;
            self.mixer_decimation_cnt[i] = 0.0;
            self.mixer_voice_samples[i] = 0;
            self.mixer_audioRouting[i] = 0;
        }
        self.mixer_decimation_rate[6] = 1.0;

        self.FxBus_init();
    }

    /// C: `IN_ITCM_RAM void mixer_decimateBlock(const uint8_t voiceNr, int16_t* buffer)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/mixer.c:60`. Reference build: out-of-line in ITCM. Group P4.
    /// Globals: `mixer_decimation_cnt`, `mixer_decimation_rate`, `mixer_voice_samples`.
    pub fn mixer_decimateBlock(&mut self, voiceNr: u8, buffer: &mut [i16]) {
        let v = voiceNr as usize;
        let phaseInc = self.mixer_decimation_rate[v] * self.mixer_decimation_rate[6];
        for i in 0..OUTPUT_DMA_SIZE as usize {
            self.mixer_decimation_cnt[v] += phaseInc;
            // vcmpe + blt: NaN takes the "else" path, like `>=` in Rust
            if self.mixer_decimation_cnt[v] >= 1.0 {
                self.mixer_decimation_cnt[v] -= 1.0;
                self.mixer_voice_samples[v] = buffer[i];
            }
            buffer[i] = self.mixer_voice_samples[v];
        }
    }

    /// C: `uint8_t mixer_checkOutJackAvailable(uint8_t dest)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/mixer.c:79`. Reference build: out-of-line in flash. Group P4.
    /// Globals: `l1_Available`, `l2_Available`, `r1_Available`, `r2_Available`.
    pub fn mixer_checkOutJackAvailable(&mut self, dest: u8) -> u8 {
        let l1 = self.l1_Available != 0;
        let l2 = self.l2_Available != 0;
        let r1 = self.r1_Available != 0;
        let r2 = self.r2_Available != 0;
        match dest {
            MIXER_ROUTING_FX_BUS => return MIXER_ROUTING_FX_BUS,
            MIXER_ROUTING_DAC1_STEREO => {
                if r1 && l1 {
                    return MIXER_ROUTING_DAC1_STEREO;
                } else if r2 && l2 {
                    return MIXER_ROUTING_DAC2_STEREO;
                } else if r1 {
                    return MIXER_ROUTING_DAC1_R;
                } else if l1 {
                    return MIXER_ROUTING_DAC1_L;
                } else if r2 {
                    return MIXER_ROUTING_DAC2_R;
                } else if l2 {
                    return MIXER_ROUTING_DAC2_L;
                }
            }
            MIXER_ROUTING_DAC2_STEREO => {
                if r2 && l2 {
                    return MIXER_ROUTING_DAC2_STEREO;
                } else if r1 && l1 {
                    return MIXER_ROUTING_DAC1_STEREO;
                } else if r2 {
                    return MIXER_ROUTING_DAC2_R;
                } else if l2 {
                    return MIXER_ROUTING_DAC2_L;
                } else if r1 {
                    return MIXER_ROUTING_DAC1_R;
                } else if l1 {
                    return MIXER_ROUTING_DAC1_L;
                }
            }
            MIXER_ROUTING_DAC1_L => {
                if l1 {
                    return dest;
                } else if r1 {
                    return MIXER_ROUTING_DAC1_R;
                } else if l2 {
                    return MIXER_ROUTING_DAC2_L;
                } else if r2 {
                    return MIXER_ROUTING_DAC2_R;
                }
            }
            MIXER_ROUTING_DAC1_R => {
                if r1 {
                    return dest;
                } else if l1 {
                    return MIXER_ROUTING_DAC1_L;
                } else if l2 {
                    return MIXER_ROUTING_DAC2_L;
                } else if r2 {
                    return MIXER_ROUTING_DAC2_R;
                }
            }
            MIXER_ROUTING_DAC2_L => {
                if l2 {
                    return dest;
                } else if r2 {
                    return MIXER_ROUTING_DAC2_R;
                } else if l1 {
                    return MIXER_ROUTING_DAC1_L;
                } else if r1 {
                    return MIXER_ROUTING_DAC1_R;
                }
            }
            MIXER_ROUTING_DAC2_R => {
                if r2 {
                    return dest;
                } else if l2 {
                    return MIXER_ROUTING_DAC2_L;
                } else if r1 {
                    return MIXER_ROUTING_DAC1_R;
                } else if l1 {
                    return MIXER_ROUTING_DAC1_L;
                }
            }
            _ => {}
        }
        dest
    }

    /// C: `IN_ITCM_RAM static inline void mixer_addDataToOutputFilteredPan(uint8_t dest, const float pan, int16_t* data,int16_t* outL,int16_t* outR,int16_t* outL2, int16_t* outR2, int16_t *fxBuffer, OnePoleFilter* filter)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/mixer.c:310`. Reference build: not in the reference ELF (its
    /// only callers are `#if 0`). Group P4. Ported from the C with the conventions of the inlined
    /// `mixer_addDataToOutput` (division by 127 as multiplication by the reciprocal); untestable.
    ///
    /// Rust signature: as for [`Lxr::mixer_addDataToOutput`], the aliasing C pointers `outL/outR` and
    /// `outL2/outR2` are the interleaved buffers `output` (`outL = &output[0]`, `outR = &output[1]`) and `output2`.
    pub fn mixer_addDataToOutputFilteredPan(&mut self, dest: u8, pan: f32, data: &[i16], output: &mut [i16], output2: &mut [i16], fxBuffer: &mut [i16], filter: &mut OnePoleFilter) {
        let dest = self.mixer_checkOutJackAvailable(dest);
        let n = OUTPUT_DMA_SIZE as usize;
        let mut pans = |filter: &mut OnePoleFilter| {
            let mixPan = calcOnePole(filter, pan as f64);
            let panL = ((127.0 - mixPan) * RECIP_127).sqrt();
            let panR = (mixPan * RECIP_127).sqrt();
            (panL, panR)
        };
        match dest {
            MIXER_ROUTING_FX_BUS => {
                for i in 0..n {
                    let (panL, panR) = pans(filter);
                    let y = ld32(fxBuffer, 2 * i);
                    let x = pack_pan(data[i], panL, panR);
                    st32(fxBuffer, 2 * i, __QADD16(x, y));
                }
            }
            MIXER_ROUTING_DAC1_STEREO => {
                for i in 0..n {
                    let (panL, panR) = pans(filter);
                    let y = ld32(output2, 2 * i);
                    let x = pack_pan(data[i], panL, panR);
                    st32(output2, 2 * i, __QADD16(x, y));
                }
            }
            MIXER_ROUTING_DAC2_STEREO => {
                for i in 0..n {
                    let (panL, panR) = pans(filter);
                    let y = ld32(output, 2 * i);
                    let x = pack_pan(data[i], panL, panR);
                    st32(output, 2 * i, __QADD16(x, y));
                }
            }
            MIXER_ROUTING_DAC1_L => {
                for i in 0..n {
                    output2[2 * i] = qadd_mono(output2[2 * i], data[i]);
                }
            }
            MIXER_ROUTING_DAC1_R => {
                for i in 0..n {
                    output2[2 * i + 1] = qadd_mono(output2[2 * i + 1], data[i]);
                }
            }
            MIXER_ROUTING_DAC2_L => {
                for i in 0..n {
                    output[2 * i] = qadd_mono(output[2 * i], data[i]);
                }
            }
            MIXER_ROUTING_DAC2_R => {
                for i in 0..n {
                    output[2 * i + 1] = qadd_mono(output[2 * i + 1], data[i]);
                }
            }
            _ => {}
        }
    }

    /// C: `IN_ITCM_RAM static inline void mixer_addDataToOutput(uint8_t dest, const float panL, const float panR, int16_t* data,int16_t* outL,int16_t* outR,int16_t* outL2, int16_t* outR2, int16_t *fxBuffer)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/mixer.c:438`. Reference build: inlined into its callers only. Group P4.
    /// Calls: `cmsis::__QADD16`, `Lxr::mixer_checkOutJackAvailable`.
    ///
    /// Rust signature: the only caller passes `&output[0], &output[1], &output2[0], &output2[1]` for
    /// `outL, outR, outL2, outR2`, which alias; the two interleaved buffers are passed whole instead
    /// (C `outL[k]` = `output[k]`, `outR[k]` = `output[1 + k]`, `outL2[k]` = `output2[k]`, `outR2[k]` = `output2[1 + k]`).
    /// `fxBuffer` is the interleaved stereo FX bus buffer (`OUTPUT_DMA_SIZE*2` samples).
    pub fn mixer_addDataToOutput(&mut self, dest: u8, panL: f32, panR: f32, data: &[i16], output: &mut [i16], output2: &mut [i16], fxBuffer: &mut [i16]) {
        let dest = self.mixer_checkOutJackAvailable(dest);
        let n = OUTPUT_DMA_SIZE as usize;
        match dest {
            MIXER_ROUTING_FX_BUS => {
                for i in 0..n {
                    let y = ld32(fxBuffer, 2 * i);
                    let x = pack_pan(data[i], panL, panR);
                    st32(fxBuffer, 2 * i, __QADD16(x, y));
                }
            }
            MIXER_ROUTING_DAC1_STEREO => {
                for i in 0..n {
                    let y = ld32(output2, 2 * i);
                    let x = pack_pan(data[i], panL, panR);
                    st32(output2, 2 * i, __QADD16(x, y));
                }
            }
            MIXER_ROUTING_DAC2_STEREO => {
                for i in 0..n {
                    let y = ld32(output, 2 * i);
                    let x = pack_pan(data[i], panL, panR);
                    st32(output, 2 * i, __QADD16(x, y));
                }
            }
            MIXER_ROUTING_DAC1_L => {
                for i in 0..n {
                    output2[2 * i] = qadd_mono(output2[2 * i], data[i]);
                }
            }
            MIXER_ROUTING_DAC1_R => {
                for i in 0..n {
                    output2[2 * i + 1] = qadd_mono(output2[2 * i + 1], data[i]);
                }
            }
            MIXER_ROUTING_DAC2_L => {
                for i in 0..n {
                    output[2 * i] = qadd_mono(output[2 * i], data[i]);
                }
            }
            MIXER_ROUTING_DAC2_R => {
                for i in 0..n {
                    output[2 * i + 1] = qadd_mono(output[2 * i + 1], data[i]);
                }
            }
            _ => {}
        }
    }

    /// C: `IN_ITCM_RAM void mixer_calcNextSampleBlock(int16_t* output,int16_t* output2)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/mixer.c:548`. Reference build: out-of-line in ITCM. Group P4.
    /// Globals: `cymbalVoice`, `hatVoice`, `l1_Available`, `l2_Available`, `mixer_audioRouting`, `mixer_decimation_rate`, `r1_Available`, `r2_Available`, `snareVoice`, `voiceArray`.
    /// Calls: `Lxr::Cymbal_calcAsync`, `Lxr::Cymbal_calcSyncBlock`, `Lxr::FxBus_process`, `Lxr::HiHat_calcAsync`, `Lxr::HiHat_calcSyncBlock`, `dsp::resonant_filter::SVF_recalcFreq`, `Lxr::Snare_calcAsync`, `Lxr::Snare_calcSyncBlock`, `dsp::buffer_tools::bufferTool_clearBuffer`, `Lxr::calcDrumVoiceAsync`, `Lxr::calcDrumVoiceSyncBlock`, `Lxr::lfo_dispatchNextValue`, `Lxr::mixer_addDataToOutput`, `Lxr::mixer_decimateBlock`, `Lxr::modNode_reassignVeloMod`, `Lxr::modNode_resetTargets`.
    ///
    /// `output`, `output2`: the interleaved L/R DMA halves of DAC1/DAC2 (`OUTPUT_DMA_SIZE*2` samples each).
    ///
    /// Jack detection: the firmware reads the four switch pins (`GPIOB->IDR` bits 6/4, `GPIOD->IDR` bits 6/7).
    /// The engine contract (docs/ARCHITECTURE.md) fixes all four inputs at 1, so the flags are set to 1 here.
    pub fn mixer_calcNextSampleBlock(&mut self, output: &mut [i16], output2: &mut [i16]) {
        self.r2_Available = 1;
        self.l1_Available = 1;
        self.l2_Available = 1;
        self.r1_Available = 1;

        self.modNode_resetTargets();
        //re assign velocity modulation
        self.modNode_reassignVeloMod();

        //calc and dispatch LFO
        self.lfo_dispatchNextValue(|s| &mut s.voiceArray[0].lfo);
        self.lfo_dispatchNextValue(|s| &mut s.voiceArray[1].lfo);
        self.lfo_dispatchNextValue(|s| &mut s.voiceArray[2].lfo);
        self.lfo_dispatchNextValue(|s| &mut s.snareVoice.lfo);
        self.lfo_dispatchNextValue(|s| &mut s.cymbalVoice.lfo);
        self.lfo_dispatchNextValue(|s| &mut s.hatVoice.lfo);

        //update filter frequencies
        SVF_recalcFreq(&mut self.voiceArray[0].filter);
        SVF_recalcFreq(&mut self.voiceArray[1].filter);
        SVF_recalcFreq(&mut self.voiceArray[2].filter);
        SVF_recalcFreq(&mut self.snareVoice.filter);
        SVF_recalcFreq(&mut self.cymbalVoice.filter);
        SVF_recalcFreq(&mut self.hatVoice.filter);

        //--- Calc async -----
        self.calcDrumVoiceAsync(0);
        self.calcDrumVoiceAsync(1);
        self.calcDrumVoiceAsync(2);

        self.Snare_calcAsync();
        self.Cymbal_calcAsync();
        self.HiHat_calcAsync();

        let mut sampleData = [0i16; OUTPUT_DMA_SIZE as usize];
        let mut fxBusData = [0i16; OUTPUT_DMA_SIZE as usize * 2];

        //Clear is necessary, otherwise we keep trash data on non routed outputs
        bufferTool_clearBuffer(output, (OUTPUT_DMA_SIZE * 2) as u8);
        bufferTool_clearBuffer(output2, (OUTPUT_DMA_SIZE * 2) as u8);
        bufferTool_clearBuffer(&mut fxBusData, (OUTPUT_DMA_SIZE * 2) as u8);

        // drum voices 1-3 (USE_TRIGGERED_PAN)
        for v in 0..3u8 {
            self.calcDrumVoiceSyncBlock(v, &mut sampleData, OUTPUT_DMA_SIZE as u8);
            self.mixer_decimateBlock(v, &mut sampleData);
            let (panL, panR) = (self.voiceArray[v as usize].triggeredPanL, self.voiceArray[v as usize].triggeredPanR);
            self.mixer_addDataToOutput(self.mixer_audioRouting[v as usize], panL, panR, &sampleData, output, output2, &mut fxBusData);
        }

        //calc snare
        self.Snare_calcSyncBlock(&mut sampleData, OUTPUT_DMA_SIZE as u8);
        self.mixer_decimateBlock(3, &mut sampleData);
        let (panL, panR) = (self.snareVoice.triggeredPanL, self.snareVoice.triggeredPanR);
        self.mixer_addDataToOutput(self.mixer_audioRouting[3], panL, panR, &sampleData, output, output2, &mut fxBusData);

        //calc cymbal
        self.Cymbal_calcSyncBlock(&mut sampleData, OUTPUT_DMA_SIZE as u8);
        self.mixer_decimateBlock(4, &mut sampleData);
        let (panL, panR) = (self.cymbalVoice.triggeredPanL, self.cymbalVoice.triggeredPanR);
        self.mixer_addDataToOutput(self.mixer_audioRouting[4], panL, panR, &sampleData, output, output2, &mut fxBusData);

        //calc HiHat
        self.HiHat_calcSyncBlock(&mut sampleData, OUTPUT_DMA_SIZE as u8);
        self.mixer_decimateBlock(5, &mut sampleData);
        let (panL, panR) = (self.hatVoice.triggeredPanL, self.hatVoice.triggeredPanR);
        self.mixer_addDataToOutput(self.mixer_audioRouting[5], panL, panR, &sampleData, output, output2, &mut fxBusData);

        self.FxBus_process(&mut fxBusData, output, output2, OUTPUT_DMA_SIZE as u32);
    }
}
