//! Port of `Core/Inc/Soundengine/DSPAudio/FX/Delay.h`, `Core/Src/Soundengine/DSPAudio/FX/Delay.c`.
//!
//! Porting group: P2. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;

// ---- #define constants
/// `#define LOW_MIN SECONDS_TO_SAMPLES(0.0006)` (Core/Inc/Soundengine/DSPAudio/FX/Delay.h:35) = 26.483999999999998
pub const LOW_MIN: f64 = f64::from_bits(0x403a7be76c8b4395);
/// `#define LOW_MAX SECONDS_TO_SAMPLES(0.02)` (Core/Inc/Soundengine/DSPAudio/FX/Delay.h:36) = 882.8000000000001
pub const LOW_MAX: f64 = f64::from_bits(0x408b966666666667);
/// `#define HI_MIN SECONDS_TO_SAMPLES(0.02)` (Core/Inc/Soundengine/DSPAudio/FX/Delay.h:38) = 882.8000000000001
pub const HI_MIN: f64 = f64::from_bits(0x408b966666666667);
/// `#define HI_MAX FX_BUFFER_SIZE` (Core/Inc/Soundengine/DSPAudio/FX/Delay.h:39)
pub const HI_MAX: i32 = 32767;

// ---- C macros that are not constants (function-like or not constant expressions); not translated:
// Core/Inc/Soundengine/DSPAudio/FX/Delay.h:33: #define SECONDS_TO_SAMPLES(s) (s*REAL_FS)

// ---- enums
/// C: `enum DelayTypesEnum` (Core/Inc/Soundengine/DSPAudio/FX/Delay.h:41), 1 bytes, underlying type `u8`.
/// Representation: integer type alias plus one `pub const` per enumerator.
pub type DelayTypesEnum = u8;
pub const DELAY_NORMAL: DelayTypesEnum = 0;
pub const DELAY_PINGPONG: DelayTypesEnum = 1;

// ---- types
/// C: `struct DelayStruct` (Core/Inc/Soundengine/DSPAudio/FX/Delay.h:47), 96 bytes.
/// C typedef name `Delay`; the struct tag `DelayStruct` is a type alias below.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Delay {
    /// `uint32_t oldRead;`
    pub oldRead: u32,
    /// `uint32_t newRead;`
    pub newRead: u32,
    /// `uint32_t write;`
    pub write: u32,
    /// `float fracReadNew,fracReadOld;`
    pub fracReadNew: f32,
    /// `float fracReadNew,fracReadOld;`
    pub fracReadOld: f32,
    /// `float interpValue;`
    pub interpValue: f32,
    /// `float interpStep;`
    pub interpStep: f32,
    /// `int16_t tmpBuffer[2];`
    pub tmpBuffer: [i16; 2],
    /// `int16_t tmpBuffer2[2]; //used for stereo or pingpong delay`
    pub tmpBuffer2: [i16; 2],
    /// `float feedback;`
    pub feedback: f32,
    /// `float range;`
    pub range: f32,
    /// `float delayTime;`
    pub delayTime: f32,
    /// `float pendingDelayTime;`
    pub pendingDelayTime: f32,
    /// `uint8_t timePending;`
    pub timePending: u8,
    /// `OnePoleFilter tone;`
    pub tone: OnePoleFilter,
    /// `OnePoleFilter tone2;`
    pub tone2: OnePoleFilter,
    /// `uint8_t delayType;`
    pub delayType: u8,
}
/// C struct tag of `Delay`.
pub type DelayStruct = Delay;
impl Delay {
    /// All-zero value (C static zero initialisation).
    pub const ZERO: Self = Self { oldRead: 0, newRead: 0, write: 0, fracReadNew: 0.0, fracReadOld: 0.0, interpValue: 0.0, interpStep: 0.0, tmpBuffer: [0; 2], tmpBuffer2: [0; 2], feedback: 0.0, range: 0.0, delayTime: 0.0, pendingDelayTime: 0.0, timePending: 0, tone: OnePoleFilter::ZERO, tone2: OnePoleFilter::ZERO, delayType: 0 };
}
impl Default for Delay {
    fn default() -> Self {
        Self::ZERO
    }
}

// ---- functions without access to globals
/// C: `void Delay_init(Delay* d)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/Delay.c:18`. Reference build: out-of-line in flash. Group P2.
/// Calls: `dsp::one_pole_lp::initOnePole`.
pub fn Delay_init(d: &mut Delay) {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/Delay.c:18")
}

/// C: `float interpolate_s16(float a, float b, float x)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/Delay.c:40`. Reference build: inlined into its callers only. Group P2.
pub fn interpolate_s16(a: f32, b: f32, x: f32) -> f32 {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/Delay.c:40")
}

/// C: `float interpolate(float a, float b, float x)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/Delay.c:272`. Reference build: inlined into its callers only. Group P2.
pub fn interpolate(a: f32, b: f32, x: f32) -> f32 {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/Delay.c:272")
}

/// C: `float calcDelayTime(Delay* d, float timeCv)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/Delay.c:276`. Reference build: inlined into its callers only. Group P2.
/// Calls: `dsp::fx::delay::interpolate`.
pub fn calcDelayTime(d: &mut Delay, timeCv: f32) -> f32 {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/Delay.c:276")
}

/// C: `void Delay_setDelayTime(Delay* d, float t)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/Delay.c:293`. Reference build: out-of-line in flash. Group P2.
/// Calls: `dsp::fx::delay::calcDelayTime`.
pub fn Delay_setDelayTime(d: &mut Delay, t: f32) {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/Delay.c:293")
}

/// C: `void Delay_setFeedback(Delay* d, float fb)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/Delay.c:349`. Reference build: out-of-line in flash. Group P2.
pub fn Delay_setFeedback(d: &mut Delay, fb: f32) {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/Delay.c:349")
}

/// C: `void Delay_setRange(Delay* d, float r)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/Delay.c:353`. Reference build: out-of-line in flash. Group P2.
/// Calls: `dsp::fx::delay::Delay_setDelayTime`.
pub fn Delay_setRange(d: &mut Delay, r: f32) {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/Delay.c:353")
}

/// C: `void Delay_setFilter(Delay* d, float f)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/Delay.c:359`. Reference build: out-of-line in flash. Group P2.
/// Calls: `dsp::one_pole_lp::setOnePoleCoef`.
pub fn Delay_setFilter(d: &mut Delay, f: f32) {
    todo!("port: Core/Src/Soundengine/DSPAudio/FX/Delay.c:359")
}

// ---- functions that use globals (methods on `Lxr`)
impl Lxr {
    /// C: `void Delay_calcBufferPingPong(Delay* d, uint8_t bufferIdx)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/FX/Delay.c:45`. Reference build: inlined into its callers only. Group P2.
    /// Globals: `FX_BUFFER`.
    /// Calls: `dsp::fx::delay::interpolate_s16`.
    pub fn Delay_calcBufferPingPong(&mut self, d: &mut Delay, bufferIdx: u8) {
        todo!("port: Core/Src/Soundengine/DSPAudio/FX/Delay.c:45")
    }

    /// C: `void Delay_calcBuffer(Delay* d, uint8_t bufferIdx)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/FX/Delay.c:71`. Reference build: inlined into its callers only. Group P2.
    /// Globals: `FX_BUFFER`.
    /// Calls: `dsp::fx::delay::interpolate_s16`.
    pub fn Delay_calcBuffer(&mut self, d: &mut Delay, bufferIdx: u8) {
        todo!("port: Core/Src/Soundengine/DSPAudio/FX/Delay.c:71")
    }

    /// C: `void Delay_processPingPong(Delay *d, int16_t *fxBuffer,int16_t* outL, int16_t* outR, uint16_t size, uint8_t channels, float dryWet)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/FX/Delay.c:93`. Reference build: out-of-line in flash. Group P2.
    /// Globals: `FX_BUFFER`.
    /// Calls: `Lxr::Delay_calcBufferPingPong`, `dsp::fx::delay::Delay_setDelayTime`, `cmsis::__QADD16`, `dsp::one_pole_lp::calcOnePole`.
    pub fn Delay_processPingPong(&mut self, d: &mut Delay, fxBuffer: &mut [i16], outL: &mut [i16], outR: &mut [i16], size: u16, channels: u8, dryWet: f32) {
        todo!("port: Core/Src/Soundengine/DSPAudio/FX/Delay.c:93")
    }

    /// C: `void Delay_process(Delay *d, int16_t *fxBuffer,int16_t* outL, int16_t* outR, uint16_t size, uint8_t channels, float dryWet)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/FX/Delay.c:187`. Reference build: out-of-line in flash. Group P2.
    /// Globals: `FX_BUFFER`.
    /// Calls: `Lxr::Delay_calcBuffer`, `dsp::fx::delay::Delay_setDelayTime`, `cmsis::__QADD16`, `dsp::one_pole_lp::calcOnePole`.
    pub fn Delay_process(&mut self, d: &mut Delay, fxBuffer: &mut [i16], outL: &mut [i16], outR: &mut [i16], size: u16, channels: u8, dryWet: f32) {
        todo!("port: Core/Src/Soundengine/DSPAudio/FX/Delay.c:187")
    }
}
