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

// ---- functions without access to globals
/// C: `void FxBus_prepareFxChange()`
///
/// Source: `Core/Src/Soundengine/FxBus.c:137`. Reference build: out-of-line in flash. Group P2.
pub fn FxBus_prepareFxChange() {
    todo!("port: Core/Src/Soundengine/FxBus.c:137")
}

// ---- functions that use globals (methods on `Lxr`)
impl Lxr {
    /// C: `void FxBus_init()`
    ///
    /// Source: `Core/Src/Soundengine/FxBus.c:21`. Reference build: out-of-line in flash. Group P2.
    /// Globals: `fxBus`.
    /// Calls: `dsp::fx::compressor::Compressor_init`, `dsp::fx::delay::Delay_init`, `dsp::fx::distortion::Distortion_init`, `dsp::fx::ringmod::Ringmod_init`.
    pub fn FxBus_init(&mut self) {
        todo!("port: Core/Src/Soundengine/FxBus.c:21")
    }

    /// C: `IN_ITCM_RAM void FxBus_process(int16_t *fxBuffer, int16_t* outL,int16_t* outR,int16_t* outL2, int16_t* outR2, uint32_t size)`
    ///
    /// Source: `Core/Src/Soundengine/FxBus.c:41`. Reference build: out-of-line in ITCM. Group P2.
    /// Globals: `fxBus`.
    /// Calls: `dsp::fx::compressor::Compressor_process`, `Lxr::Delay_process`, `Lxr::Delay_processPingPong`, `dsp::fx::distortion::Distortion_process`, `Lxr::Ringmod_process`, `cmsis::__QADD16`.
    pub fn FxBus_process(&mut self, fxBuffer: &mut [i16], outL: &mut [i16], outR: &mut [i16], outL2: &mut [i16], outR2: &mut [i16], size: u32) {
        todo!("port: Core/Src/Soundengine/FxBus.c:41")
    }

    /// C: `IN_ITCM_RAM void FxBus_updateParameter()`
    ///
    /// Source: `Core/Src/Soundengine/FxBus.c:143`. Reference build: out-of-line in ITCM. Group P2.
    /// Globals: `fxBus`.
    /// Calls: `dsp::fx::compressor::Compressor_setAttack`, `dsp::fx::compressor::Compressor_setGain`, `dsp::fx::compressor::Compressor_setRelease`, `dsp::fx::compressor::Compressor_setTreshold`, `dsp::fx::delay::Delay_setDelayTime`, `dsp::fx::delay::Delay_setFeedback`, `dsp::fx::delay::Delay_setFilter`, `dsp::fx::delay::Delay_setRange`, `dsp::fx::distortion::Distortion_setDrive`, `dsp::fx::distortion::Distortion_setParam1`, `dsp::fx::distortion::Distortion_setParam2`, `dsp::fx::distortion::Distortion_setParam3`, `dsp::fx::ringmod::Ringmod_setFreq`.
    pub fn FxBus_updateParameter(&mut self) {
        todo!("port: Core/Src/Soundengine/FxBus.c:143")
    }

    /// C: `IN_ITCM_RAM void FxBus_setParameter(uint8_t idx, float value)`
    ///
    /// Source: `Core/Src/Soundengine/FxBus.c:192`. Reference build: out-of-line in ITCM. Group P2.
    /// Globals: `fxBus`.
    /// Calls: `Lxr::FxBus_updateParameter`.
    pub fn FxBus_setParameter(&mut self, idx: u8, value: f32) {
        todo!("port: Core/Src/Soundengine/FxBus.c:192")
    }
}
