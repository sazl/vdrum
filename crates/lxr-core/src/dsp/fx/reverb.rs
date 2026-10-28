//! Port of `Core/Inc/Soundengine/DSPAudio/FX/Reverb.h`, `Core/Src/Soundengine/DSPAudio/FX/Reverb.c`.
//!
//! Porting group: P2. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;

// ---- #define constants
/// `#define MAX_DELAY_SIZE MAX_ALLPASS_SIZE` (Core/Inc/Soundengine/DSPAudio/FX/Reverb.h:32)
pub const MAX_DELAY_SIZE: i32 = 2048;
/// `#define EARLY_TAPS 4` (Core/Inc/Soundengine/DSPAudio/FX/Reverb.h:33)
pub const EARLY_TAPS: i32 = 4;
/// `#define USE_OUT_2 0` (Core/Inc/Soundengine/DSPAudio/FX/Reverb.h:35)
pub const USE_OUT_2: i32 = 0;
/// `#define USE_LOWER_DENSITY 1` (Core/Inc/Soundengine/DSPAudio/FX/Reverb.h:36)
pub const USE_LOWER_DENSITY: i32 = 1;
/// `#define HIGH_RES_SINE_COS 0` (Core/Inc/Soundengine/DSPAudio/FX/Reverb.h:37)
pub const HIGH_RES_SINE_COS: i32 = 0;
/// `#define SPIN_SEMI 0` (Core/Inc/Soundengine/DSPAudio/FX/Reverb.h:38)
pub const SPIN_SEMI: i32 = 0;

// ---- types
/// C: `struct ReverbStruct` (Core/Inc/Soundengine/DSPAudio/FX/Reverb.h:41), 520 bytes.
/// C typedef name `Reverb`; the struct tag `ReverbStruct` is a type alias below.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Reverb {
    /// `AllpassDelay apDelay[2];`
    pub apDelay: [AllpassDelay; 2],
    /// `AllpassModDelay apModDelay[2];`
    pub apModDelay: [AllpassModDelay; 2],
    /// `AllpassDelay inputAllpass[4];`
    pub inputAllpass: [AllpassDelay; 4],
    /// `VerbDelay delay[4];`
    pub delay: [VerbDelay; 4],
    /// `VerbDelay earlyReflectionDelay;`
    pub earlyReflectionDelay: VerbDelay,
    /// `VerbDelay preDelay;`
    pub preDelay: VerbDelay,
    /// `SineCosine lfo;`
    pub lfo: SineCosine,
    /// `float feedback;`
    pub feedback: f32,
    /// `float delayTime;`
    pub delayTime: f32,
    /// `float earlyVolume;`
    pub earlyVolume: f32,
    /// `OnePoleFilter lpFeedback;`
    pub lpFeedback: OnePoleFilter,
    /// `OnePoleFilter lpImpulse;`
    pub lpImpulse: OnePoleFilter,
    /// `volatile __fp16 *reverbBufferSram;`
    ///
    /// C pointer: stored as the address in the reference firmware memory map (`CPtr`, 0 = NULL), see `memmap`.
    pub reverbBufferSram: CPtr,
    /// `float lastOut;`
    pub lastOut: f32,
    /// `float lastOut2;`
    pub lastOut2: f32,
    /// `TiltEq eq;`
    pub eq: TiltEq,
    /// `TiltEq eqFb;`
    pub eqFb: TiltEq,
}
/// C struct tag of `Reverb`.
pub type ReverbStruct = Reverb;
impl Reverb {
    /// All-zero value (C static zero initialisation).
    pub const ZERO: Self = Self { apDelay: [AllpassDelay::ZERO; 2], apModDelay: [AllpassModDelay::ZERO; 2], inputAllpass: [AllpassDelay::ZERO; 4], delay: [VerbDelay::ZERO; 4], earlyReflectionDelay: VerbDelay::ZERO, preDelay: VerbDelay::ZERO, lfo: SineCosine::ZERO, feedback: 0.0, delayTime: 0.0, earlyVolume: 0.0, lpFeedback: OnePoleFilter::ZERO, lpImpulse: OnePoleFilter::ZERO, reverbBufferSram: 0, lastOut: 0.0, lastOut2: 0.0, eq: TiltEq::ZERO, eqFb: TiltEq::ZERO };
}
impl Default for Reverb {
    fn default() -> Self {
        Self::ZERO
    }
}

// ---- functions without access to globals
//
// Not in the reference firmware (`USE_REVERB 0` in FxBus.h, the file is not linked): ported from C, untestable.
// The buffer functions take `ram` = `Lxr::FX_BUFFER` (see `verb_tools::verb_delay`).

use super::verb_tools::allpass_delay::*;
use super::verb_tools::sin_cos::*;
use super::verb_tools::verb_delay::*;
use super::verb_tools::verb_mod_delay::*;
use super::{ld32, pkhbt, ssat16, st32, sx};
use crate::cmsis::__QADD16;
use crate::dsp::one_pole_lp::{calcOnePole, setOnePoleCoef};
use crate::dsp::tilt_eq::{TiltEq_init, TiltEq_process, TiltEq_setParameter};
use crate::memmap::ADDR_FX_BUFFER;

/// `static const float earlyTaps[]`
const earlyTaps: [f32; 4] = [12.0, 24.0, 38.0, 50.0];

/// C: `void Reverb_process(Reverb *r, int16_t *fxBuffer,int16_t* outL, int16_t* outR, uint16_t size, uint8_t channels, float dryWet)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/Reverb.c:207`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
/// `outL` is the interleaved output starting at the C `outL` pointer (`outR` and `channels` are unused in C).
pub fn Reverb_process(r: &mut Reverb, fxBuffer: &[i16], outL: &mut [i16], size: u16, channels: u8, dryWet: f32, ram: &mut [u16]) {
    SineCosine_calc(&mut r.lfo);
    let s = SineCosine_getSine(&mut r.lfo);
    VerbModDelay_setModValue(&mut r.apModDelay[1].delay, s);
    let c = SineCosine_getCosine(&mut r.lfo);
    VerbModDelay_setModValue(&mut r.apModDelay[0].delay, c);

    let mut i: u8 = 0;
    let mut k: usize = 0;
    while (i as u16) < size {
        let idx1 = i.wrapping_mul(2) as usize;
        let idx2 = i.wrapping_mul(2).wrapping_add(1) as usize;
        let mix = __QADD16(sx(fxBuffer[idx1] / 2), sx(fxBuffer[idx2] / 2)) as i16;
        let mut monoInput = mix as f32 / 32768.0;
        monoInput = VerbDelay_process(&mut r.preDelay, monoInput, ram);

        let mut early: f32 = 0.0;
        VerbDelay_process(&mut r.earlyReflectionDelay, monoInput, ram);
        early += VerbDelay_ReadTap(&mut r.earlyReflectionDelay, earlyTaps[0], ram);
        early += VerbDelay_ReadTap(&mut r.earlyReflectionDelay, earlyTaps[1], ram) * 0.5;
        early += VerbDelay_ReadTap(&mut r.earlyReflectionDelay, earlyTaps[2], ram) * 0.125;
        early += VerbDelay_ReadTap(&mut r.earlyReflectionDelay, earlyTaps[3], ram) * 0.25;

        monoInput = (0.5 * TiltEq_process(&mut r.eq, monoInput) as f64) as f32;
        let t = AllpassDelay_process(&mut r.inputAllpass[1], monoInput, ram);
        let r#in = AllpassDelay_process(&mut r.inputAllpass[0], t, ram);
        let t = AllpassDelay_process(&mut r.inputAllpass[3], monoInput, ram);
        let in2 = AllpassDelay_process(&mut r.inputAllpass[2], t, ram);

        let mut tmp = AllpassModDelay_process(&mut r.apModDelay[0], r#in + r.lastOut2, ram);
        tmp = VerbDelay_process(&mut r.delay[0], tmp, ram);
        tmp *= r.feedback;
        tmp = AllpassDelay_process(&mut r.apDelay[1], tmp, ram);
        tmp = VerbDelay_process(&mut r.delay[1], tmp, ram);
        r.lastOut = calcOnePole(&mut r.lpFeedback, (tmp * r.feedback) as f64);
        tmp = AllpassModDelay_process(&mut r.apModDelay[1], in2 + r.lastOut, ram);
        tmp = VerbDelay_process(&mut r.delay[2], tmp, ram);
        tmp *= r.feedback;
        tmp = AllpassDelay_process(&mut r.apDelay[1], tmp, ram);
        tmp = VerbDelay_process(&mut r.delay[3], tmp, ram);
        r.lastOut2 = tmp * r.feedback;

        let mut out2: f32 = (VerbDelay_ReadTapSample(&mut r.delay[2], 266, ram) as f64 * 0.6) as f32;
        out2 = (out2 as f64 - VerbDelay_ReadTapSample(&mut r.apDelay[1].delay, 1913, ram) as f64 * 0.6) as f32;
        out2 = (out2 as f64 + VerbDelay_ReadTapSample(&mut r.delay[3], 996, ram) as f64 * 0.6) as f32;
        out2 = (out2 as f64 - VerbDelay_ReadTapSample(&mut r.apDelay[0].delay, 187, ram) as f64 * 0.6) as f32;
        out2 = (out2 as f64 - VerbDelay_ReadTapSample(&mut r.delay[1], 1066, ram) as f64 * 0.6) as f32;
        let out = TiltEq_process(&mut r.eqFb, fma(early, r.earlyVolume, out2));

        let wet = ssat16((out * 32768.0) as i32);
        let f1 = fxBuffer[idx1] as i32;
        let f2 = fxBuffer[idx2] as i32;
        let x = pkhbt(f1 + ssat16((((wet - f1) as f32) * dryWet) as i32), f2 + ssat16((((wet - f2) as f32) * dryWet) as i32));
        let w = __QADD16(ld32(outL, k), x);
        st32(outL, k, w);
        k += 2;
        i = i.wrapping_add(1);
    }
}

/// C: `IN_ITCM_RAM void Reverb_setDelayTime(Reverb *r, float t)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/Reverb.c:452`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
pub fn Reverb_setDelayTime(r: &mut Reverb, t: f32) {
    let sz = 0.02166f32 + t * 0.97784342688330871492f32;
    VerbDelay_setRoomSize(&mut r.preDelay, sz);
}

/// C: `IN_ITCM_RAM void Reverb_setFeedback(Reverb *r, float fb)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/Reverb.c:480`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
pub fn Reverb_setFeedback(r: &mut Reverb, fb: f32) {
    r.feedback = fb;
}

/// C: `IN_ITCM_RAM void Reverb_setComposition(Reverb *r, float c)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/Reverb.c:485`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
pub fn Reverb_setComposition(r: &mut Reverb, c: f32) {
    TiltEq_setParameter(&mut r.eq, -1.0 + 2.0 * c);
    TiltEq_setParameter(&mut r.eqFb, -1.0 + 2.0 * c);
    setOnePoleCoef(&mut r.lpFeedback, c as f64);
}

/// C: `IN_ITCM_RAM void Reverb_setVariation(Reverb *r, float v)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/Reverb.c:494`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
pub fn Reverb_setVariation(r: &mut Reverb, v: f32) {
    r.earlyVolume = v;
}

// ---- functions that use globals (methods on `Lxr`)
impl Lxr {
    /// C: `void Reverb_init(Reverb *r)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/FX/Reverb.c:29`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
    /// Globals: `FX_BUFFER` (only its address).
    pub fn Reverb_init(&mut self, r: &mut Reverb) {
        TiltEq_init(&mut r.eq);
        TiltEq_init(&mut r.eqFb);
        SineCosine_init(&mut r.lfo);
        r.reverbBufferSram = ADDR_FX_BUFFER;
        r.earlyVolume = 1.0;
        setOnePoleCoef(&mut r.lpFeedback, 0.5);
        setOnePoleCoef(&mut r.lpImpulse, 0.5);
        r.lastOut = 0.0;
        r.lastOut2 = 0.0;
        SineCosine_setFreq(&mut r.lfo, 1.31);
        // &r->reverbBufferSram[n]: element n of the __fp16 buffer
        let base = r.reverbBufferSram;
        let at = |n: i32| base.wrapping_add(2 * n as u32);
        for i in 0..2 {
            AllpassDelay_init(&mut r.apDelay[i]);
            AllpassModDelay_init(&mut r.apModDelay[i]);
            AllpassDelay_setBuffer(&mut r.apDelay[i], at(i as i32 * 2 * MAX_ALLPASS_SIZE));
            AllpassModDelay_setBuffer(&mut r.apModDelay[i], at((i as i32 * 2 + 1) * MAX_ALLPASS_SIZE));
        }
        for i in 0..4 {
            AllpassDelay_init(&mut r.inputAllpass[i]);
            AllpassDelay_setBuffer(&mut r.inputAllpass[i], at(4 * MAX_ALLPASS_SIZE + i as i32 * MAX_ALLPASS_SIZE));
        }
        for i in 0..4 {
            VerbDelay_init(&mut r.delay[i]);
            VerbDelay_setBuffer(&mut r.delay[i], at(8 * MAX_ALLPASS_SIZE + i as i32 * MAX_DELAY_SIZE));
        }
        VerbDelay_init(&mut r.earlyReflectionDelay);
        VerbDelay_setBuffer(&mut r.earlyReflectionDelay, at(12 * MAX_ALLPASS_SIZE));
        VerbDelay_init(&mut r.preDelay);
        VerbDelay_setBuffer(&mut r.preDelay, at(13 * MAX_ALLPASS_SIZE));

        VerbDelay_setLength(&mut r.earlyReflectionDelay, 2000);
        VerbDelay_setLength(&mut r.preDelay, 2031);

        AllpassDelay_setDelay(&mut r.inputAllpass[0], 142.0);
        AllpassDelay_setDelay(&mut r.inputAllpass[1], 107.0);
        AllpassDelay_setDelay(&mut r.inputAllpass[2], 379.0);
        AllpassDelay_setDelay(&mut r.inputAllpass[3], 277.0);

        VerbDelay_setLength(&mut r.delay[0], 2047);
        VerbDelay_setLength(&mut r.delay[1], 1913);
        VerbDelay_setLength(&mut r.delay[2], 1720);
        VerbDelay_setLength(&mut r.delay[3], 1163);
        AllpassDelay_setDelay(&mut r.apDelay[0], 1800.0);
        AllpassDelay_setDelay(&mut r.apDelay[1], 2039.0);
        AllpassModDelay_setDelay(&mut r.apModDelay[0], 672.0);
        AllpassModDelay_setDelay(&mut r.apModDelay[1], 908.0);

        AllpassDelay_setCoefficient(&mut r.inputAllpass[0], 0.75);
        AllpassDelay_setCoefficient(&mut r.inputAllpass[1], 0.75);
        AllpassDelay_setCoefficient(&mut r.inputAllpass[2], 0.625);
        AllpassDelay_setCoefficient(&mut r.inputAllpass[3], 0.625);
    }
}
