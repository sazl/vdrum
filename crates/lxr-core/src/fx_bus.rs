//! Port of `Core/Inc/Soundengine/FxBus.h`, `Core/Src/Soundengine/FxBus.c`.
//!
//! Porting group: P2. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;

// ---- #define constants
/// `#define USE_REVERB 0` (Core/Inc/Soundengine/FxBus.h:26)
pub const USE_REVERB: i32 = 0;

// ---- enums
/// C: `enum FxTypeEnum` (Core/Inc/Soundengine/FxBus.h:28), 1 bytes, underlying type `u8`.
/// Representation: integer type alias plus one `pub const` per enumerator.
pub type FxTypeEnum = u8;
pub const FX_OFF: FxTypeEnum = 0;
pub const FX_DISTORTION: FxTypeEnum = 1;
pub const FX_RING: FxTypeEnum = 2;
pub const FX_COMPRESSOR: FxTypeEnum = 3;
pub const FX_DELAY: FxTypeEnum = 4;

// ---- types
/// C: `struct FxStruct` (Core/Inc/Soundengine/FxBus.h:40), 352 bytes.
/// C typedef name `FxBus`; the struct tag `FxStruct` is a type alias below.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FxBus {
    /// `uint8_t fxType;`
    pub fxType: u8,
    /// `uint8_t audioRouting;`
    pub audioRouting: u8,
    /// `float dryWet;`
    pub dryWet: f32,
    /// `float parameter[4];`
    pub parameter: [f32; 4],
    /// `MasterDistortion distortion;`
    pub distortion: MasterDistortion,
    /// `Ringmod ringmod;`
    pub ringmod: Ringmod,
    /// `Compressor compressor;`
    pub compressor: Compressor,
    /// `Delay delay;`
    pub delay: Delay,
}
/// C struct tag of `FxBus`.
pub type FxStruct = FxBus;
impl FxBus {
    /// All-zero value (C static zero initialisation).
    pub const ZERO: Self = Self { fxType: 0, audioRouting: 0, dryWet: 0.0, parameter: [0.0; 4], distortion: MasterDistortion::ZERO, ringmod: Ringmod::ZERO, compressor: Compressor::ZERO, delay: Delay::ZERO };
}
impl Default for FxBus {
    fn default() -> Self {
        Self::ZERO
    }
}


use crate::cmsis::__QADD16;
use crate::dsp::fx::compressor::{Compressor_init, Compressor_process, Compressor_setAttack, Compressor_setGain, Compressor_setRelease, Compressor_setTreshold};
use crate::dsp::fx::delay::{Delay_init, Delay_setDelayTime, Delay_setFeedback, Delay_setFilter, Delay_setRange};
use crate::dsp::fx::distortion::{Distortion_init, Distortion_process, Distortion_setDrive, Distortion_setParam1, Distortion_setParam2, Distortion_setParam3};
use crate::dsp::fx::ringmod::{Ringmod_init, Ringmod_setFreq};
use crate::dsp::fx::{ld32, st32, sx};

// ---- functions without access to globals
/// C: `void FxBus_prepareFxChange()`
///
/// Source: `Core/Src/Soundengine/FxBus.c:137`. Reference build: out-of-line in flash. Group P2.
/// The body is commented out in C (`bx lr`).
pub fn FxBus_prepareFxChange() {}

// ---- functions that use globals (methods on `Lxr`)
impl Lxr {
    /// C: `void FxBus_init()`
    ///
    /// Source: `Core/Src/Soundengine/FxBus.c:21`. Reference build: out-of-line in flash. Group P2.
    /// Globals: `fxBus`.
    /// Calls: `dsp::fx::compressor::Compressor_init`, `dsp::fx::delay::Delay_init`, `dsp::fx::distortion::Distortion_init`, `dsp::fx::ringmod::Ringmod_init`.
    pub fn FxBus_init(&mut self) {
        let fx = &mut self.fxBus;
        fx.dryWet = 0.0;
        fx.audioRouting = 0;
        fx.fxType = FX_OFF;
        fx.delay.delayType = DELAY_NORMAL;
        Distortion_init(&mut fx.distortion);
        Ringmod_init(&mut fx.ringmod);
        Compressor_init(&mut fx.compressor);
        Delay_init(&mut fx.delay);
    }

    /// C: `IN_ITCM_RAM void FxBus_process(int16_t *fxBuffer, int16_t* outL,int16_t* outR,int16_t* outL2, int16_t* outR2, uint32_t size)`
    ///
    /// Source: `Core/Src/Soundengine/FxBus.c:41`. Reference build: out-of-line in ITCM. Group P2.
    /// Globals: `fxBus`.
    /// Calls: `dsp::fx::compressor::Compressor_process`, `Lxr::Delay_process`, `Lxr::Delay_processPingPong`, `dsp::fx::distortion::Distortion_process`, `Lxr::Ringmod_process`, `cmsis::__QADD16`.
    ///
    /// The mixer passes `&output[pos], &output[pos+1], &output2[pos], &output2[pos+1]`: `output` / `output2`
    /// are the interleaved DAC buffers starting at `pos` (C `outL[k] = output[k]`, `outR[k] = output[1+k]`,
    /// `outL2[k] = output2[k]`, `outR2[k] = output2[1+k]`). `fxBuffer` is only read.
    pub fn FxBus_process(&mut self, fxBuffer: &mut [i16], output: &mut [i16], output2: &mut [i16], size: u32) {
        let routing = self.fxBus.audioRouting;
        let channels: u8 = if routing < MIXER_ROUTING_DAC1_L { 2 } else { 1 };
        // `out` (the only output pointer the effects write through; `out2` is never used by them)
        let out: &mut [i16] = match routing {
            MIXER_ROUTING_DAC2_STEREO => output,
            MIXER_ROUTING_DAC1_L => output2,
            MIXER_ROUTING_DAC1_R => &mut output2[1..],
            MIXER_ROUTING_DAC2_L => output,
            MIXER_ROUTING_DAC2_R => &mut output[1..],
            _ => output2, // MIXER_ROUTING_DAC1_STEREO and default
        };
        let fxBuffer: &[i16] = fxBuffer;
        let size16 = size as u16;
        let dryWet = self.fxBus.dryWet;
        match self.fxBus.fxType {
            FX_DISTORTION => {
                Distortion_process(&mut self.fxBus.distortion, fxBuffer, out, size16, channels, dryWet);
            }
            FX_RING => {
                let mut rm = self.fxBus.ringmod;
                self.Ringmod_process(&mut rm, fxBuffer, out, size16, channels, dryWet);
                self.fxBus.ringmod = rm;
            }
            FX_COMPRESSOR => {
                Compressor_process(&mut self.fxBus.compressor, fxBuffer, out, size16, channels, dryWet);
            }
            FX_DELAY => {
                let mut d = self.fxBus.delay;
                if d.delayType == DELAY_NORMAL {
                    self.Delay_process(&mut d, fxBuffer, out, size16, channels, dryWet);
                } else {
                    self.Delay_processPingPong(&mut d, fxBuffer, out, size16, channels, dryWet);
                }
                self.fxBus.delay = d;
            }
            _ => {
                // FX_OFF and default; uint8_t loop counter compared with the uint32_t size
                let mut i: u8 = 0;
                let mut k: usize = 0;
                if channels == 2 {
                    while (i as u32) < size {
                        let w = __QADD16(ld32(out, k), ld32(fxBuffer, k));
                        st32(out, k, w);
                        k += 2;
                        i = i.wrapping_add(1);
                    }
                } else {
                    while (i as u32) < size {
                        // ldrsh out; ldr in; qadd16; strh
                        out[k] = __QADD16(sx(out[k]), ld32(fxBuffer, k)) as i16;
                        k += 2;
                        i = i.wrapping_add(1);
                    }
                }
            }
        }
    }

    /// C: `IN_ITCM_RAM void FxBus_updateParameter()`
    ///
    /// Source: `Core/Src/Soundengine/FxBus.c:143`. Reference build: out-of-line in ITCM. Group P2.
    /// Globals: `fxBus`.
    /// Calls: `dsp::fx::compressor::Compressor_setAttack`, `dsp::fx::compressor::Compressor_setGain`, `dsp::fx::compressor::Compressor_setRelease`, `dsp::fx::compressor::Compressor_setTreshold`, `dsp::fx::delay::Delay_setDelayTime`, `dsp::fx::delay::Delay_setFeedback`, `dsp::fx::delay::Delay_setFilter`, `dsp::fx::delay::Delay_setRange`, `dsp::fx::distortion::Distortion_setDrive`, `dsp::fx::distortion::Distortion_setParam1`, `dsp::fx::distortion::Distortion_setParam2`, `dsp::fx::distortion::Distortion_setParam3`, `dsp::fx::ringmod::Ringmod_setFreq`.
    pub fn FxBus_updateParameter(&mut self) {
        let fx = &mut self.fxBus;
        let p = fx.parameter;
        match fx.fxType {
            FX_DISTORTION => {
                Distortion_setDrive(&mut fx.distortion, p[0]);
                Distortion_setParam1(&mut fx.distortion, p[1]);
                Distortion_setParam2(&mut fx.distortion, p[2]);
                Distortion_setParam3(&mut fx.distortion, p[3]);
            }
            FX_RING => {
                Ringmod_setFreq(&mut fx.ringmod, p[0]);
            }
            FX_COMPRESSOR => {
                Compressor_setAttack(&mut fx.compressor, p[0]);
                Compressor_setRelease(&mut fx.compressor, p[1]);
                Compressor_setTreshold(&mut fx.compressor, p[2]);
                Compressor_setGain(&mut fx.compressor, p[3]);
            }
            FX_DELAY => {
                Delay_setRange(&mut fx.delay, p[1]);
                Delay_setDelayTime(&mut fx.delay, p[0]);
                Delay_setFilter(&mut fx.delay, p[2]);
                Delay_setFeedback(&mut fx.delay, p[3]);
            }
            _ => {}
        }
    }

    /// C: `IN_ITCM_RAM void FxBus_setParameter(uint8_t idx, float value)`
    ///
    /// Source: `Core/Src/Soundengine/FxBus.c:192`. Reference build: out-of-line in ITCM. Group P2.
    /// Globals: `fxBus`.
    /// Calls: `Lxr::FxBus_updateParameter`.
    /// (`idx` > 3 would write past `parameter[]` in C; callers only pass 0..=3, here it panics.)
    pub fn FxBus_setParameter(&mut self, idx: u8, value: f32) {
        self.fxBus.parameter[idx as usize] = value;
        self.FxBus_updateParameter();
    }
}
