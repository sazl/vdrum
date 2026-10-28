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
/// C: `void Reverb_process(Reverb *r, int16_t *fxBuffer,int16_t* outL, int16_t* outR, uint16_t size, uint8_t channels, float dryWet)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/Reverb.c:207`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
/// Calls: `dsp::fx::verb_tools::allpass_delay::AllpassDelay_process`, `dsp::fx::verb_tools::allpass_delay::AllpassModDelay_process`, `dsp::fx::verb_tools::sin_cos::SineCosine_calc`, `dsp::fx::verb_tools::sin_cos::SineCosine_getCosine`, `dsp::fx::verb_tools::sin_cos::SineCosine_getSine`, `dsp::tilt_eq::TiltEq_process`, `dsp::fx::verb_tools::verb_delay::VerbDelay_ReadTap`, `dsp::fx::verb_tools::verb_delay::VerbDelay_ReadTapSample`, `dsp::fx::verb_tools::verb_delay::VerbDelay_process`, `dsp::fx::verb_tools::verb_mod_delay::VerbModDelay_setModValue`, `cmsis::__QADD16`, `dsp::one_pole_lp::calcOnePole`.
pub fn Reverb_process(r: &mut Reverb, fxBuffer: &mut [i16], outL: &mut [i16], outR: &mut [i16], size: u16, channels: u8, dryWet: f32) {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/Reverb.c:207")
}

/// C: `IN_ITCM_RAM void Reverb_setDelayTime(Reverb *r, float t)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/Reverb.c:452`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
/// Calls: `dsp::fx::verb_tools::verb_delay::VerbDelay_setRoomSize`.
pub fn Reverb_setDelayTime(r: &mut Reverb, t: f32) {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/Reverb.c:452")
}

/// C: `IN_ITCM_RAM void Reverb_setFeedback(Reverb *r, float fb)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/Reverb.c:480`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
pub fn Reverb_setFeedback(r: &mut Reverb, fb: f32) {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/Reverb.c:480")
}

/// C: `IN_ITCM_RAM void Reverb_setComposition(Reverb *r, float c)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/Reverb.c:485`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
/// Calls: `dsp::tilt_eq::TiltEq_setParameter`, `dsp::one_pole_lp::setOnePoleCoef`.
pub fn Reverb_setComposition(r: &mut Reverb, c: f32) {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/Reverb.c:485")
}

/// C: `IN_ITCM_RAM void Reverb_setVariation(Reverb *r, float v)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/Reverb.c:494`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
pub fn Reverb_setVariation(r: &mut Reverb, v: f32) {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/Reverb.c:494")
}

// ---- functions that use globals (methods on `Lxr`)
impl Lxr {
    /// C: `void Reverb_init(Reverb *r)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/FX/Reverb.c:29`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P2.
    /// Globals: `FX_BUFFER`.
    /// Calls: `dsp::fx::verb_tools::allpass_delay::AllpassDelay_init`, `dsp::fx::verb_tools::allpass_delay::AllpassDelay_setBuffer`, `dsp::fx::verb_tools::allpass_delay::AllpassDelay_setCoefficient`, `dsp::fx::verb_tools::allpass_delay::AllpassDelay_setDelay`, `dsp::fx::verb_tools::allpass_delay::AllpassModDelay_init`, `dsp::fx::verb_tools::allpass_delay::AllpassModDelay_setBuffer`, `dsp::fx::verb_tools::allpass_delay::AllpassModDelay_setDelay`, `dsp::fx::verb_tools::sin_cos::SineCosine_init`, `dsp::fx::verb_tools::sin_cos::SineCosine_setFreq`, `dsp::tilt_eq::TiltEq_init`, `dsp::fx::verb_tools::verb_delay::VerbDelay_init`, `dsp::fx::verb_tools::verb_delay::VerbDelay_setBuffer`, `dsp::fx::verb_tools::verb_delay::VerbDelay_setLength`, `dsp::one_pole_lp::setOnePoleCoef`.
    pub fn Reverb_init(&mut self, r: &mut Reverb) {
        todo!("port: Core/Src/Soundengine/DSPAudio/FX/Reverb.c:29")
    }
}
