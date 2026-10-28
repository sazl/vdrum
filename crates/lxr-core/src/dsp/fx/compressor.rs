//! Port of `Core/Inc/Soundengine/DSPAudio/FX/Compressor.h`, `Core/Src/Soundengine/DSPAudio/FX/Compressor.c`.
//!
//! Porting group: P2. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;

// ---- enums
// C: `enum {...}` (Core/Src/Soundengine/DSPAudio/FX/Compressor.c:21), 1 bytes, underlying type `u8`. (anonymous: enumerators only)
pub const RATIO_1: u8 = 0;
pub const RATIO_2: u8 = 1;
pub const RATIO_3: u8 = 2;
pub const RATIO_4: u8 = 3;
pub const RATIO_6: u8 = 4;
pub const RATIO_8: u8 = 5;

// ---- types
/// C: `struct CompressorStruct` (Core/Inc/Soundengine/DSPAudio/FX/Compressor.h:21), 32 bytes.
/// C typedef name `Compressor`; the struct tag `CompressorStruct` is a type alias below.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Compressor {
    /// `float tresh;`
    pub tresh: f32,
    /// `float attack;`
    pub attack: f32,
    /// `float release;`
    pub release: f32,
    /// `float ratio;`
    pub ratio: f32,
    /// `float markupGain;`
    pub markupGain: f32,
    /// `double env;`
    pub env: f64,
}
/// C struct tag of `Compressor`.
pub type CompressorStruct = Compressor;
impl Compressor {
    /// All-zero value (C static zero initialisation).
    pub const ZERO: Self = Self { tresh: 0.0, attack: 0.0, release: 0.0, ratio: 0.0, markupGain: 0.0, env: 0.0 };
}
impl Default for Compressor {
    fn default() -> Self {
        Self::ZERO
    }
}


use super::{ld32, pkhbt, ssat16, st32, sx, vmaxnm64};
use crate::cmsis::__QADD16;
use crate::fpu::vfma64;

/// `static const double DC_OFFSET = 1.0E-25;`
const DC_OFFSET: f64 = f64::from_bits(0x3abe_f2d0_f5da_7dd9);
/// `LOG_2_DB = 20 / ln(10)`
const LOG_2_DB: f64 = f64::from_bits(0x4021_5f2c_ed38_4f29);
/// `DB_2_LOG = ln(10) / 20`
const DB_2_LOG: f64 = f64::from_bits(0x3fbd_791c_5f88_8822);
/// `REAL_FS` as float (44140.0).
const FS_F: f32 = f32::from_bits(0x472c_6c00);
/// `1/32768.f`
const INV_32768: f32 = f32::from_bits(0x3800_0000);

// ---- functions without access to globals
/// C: `void Compressor_init(Compressor *c)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/Compressor.c:32`. Reference build: out-of-line in flash. Group P2.
pub fn Compressor_init(c: &mut Compressor) {
    c.attack = f32::from_bits(0x3dcc_cccd); // 0.1
    c.release = 300.0;
    c.env = 0.0;
    c.tresh = 3.0;
    c.markupGain = 1.0;
}

/// C: `static double lin2dB( double lin )`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/Compressor.c:45`. Reference build: inlined into its callers only. Group P2.
/// (C form; `Compressor_process` fuses the multiplication with the threshold subtraction.)
pub fn lin2dB(lin: f64) -> f64 {
    libm::logf(lin as f32) as f64 * LOG_2_DB
}

/// C: `static double dB2lin( double dB )`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/Compressor.c:51`. Reference build: inlined into its callers only. Group P2.
/// (C form; `Compressor_process` folds `DB_2_LOG` into the ratio factor.)
pub fn dB2lin(dB: f64) -> f64 {
    libm::expf((dB * DB_2_LOG) as f32) as f64
}

/// C: `void Compressor_setAttack(Compressor *c, float a)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/Compressor.c:57`. Reference build: out-of-line in flash. Group P2.
pub fn Compressor_setAttack(c: &mut Compressor, a: f32) {
    let t = vfma(f32::from_bits(0x3cf5_c28f), a, 100.0) * FS_F;
    c.attack = libm::exp(-1000.0 / t as f64) as f32;
}

/// C: `void Compressor_setTreshold(Compressor *c, float t)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/Compressor.c:62`. Reference build: out-of-line in flash. Group P2.
/// `-96 + t*96` compiled as `(t - 1) * 96`.
pub fn Compressor_setTreshold(c: &mut Compressor, t: f32) {
    c.tresh = (t - 1.0) * 96.0;
}

/// C: `void Compressor_setGain(Compressor *c, float g)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/Compressor.c:67`. Reference build: out-of-line in flash. Group P2.
pub fn Compressor_setGain(c: &mut Compressor, g: f32) {
    c.markupGain = vfma(1.0, g, 2.0);
}

/// C: `void Compressor_setRelease(Compressor *c, float r)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/Compressor.c:71`. Reference build: out-of-line in flash. Group P2.
pub fn Compressor_setRelease(c: &mut Compressor, r: f32) {
    let t = vfma(25.0, r, 175.0) * FS_F;
    c.release = libm::exp(-1000.0 / t as f64) as f32;
}

/// C: `void Compressor_setRatio(Compressor *c, uint8_t r)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/Compressor.c:77`. Reference build: out-of-line in flash. Group P2.
pub fn Compressor_setRatio(c: &mut Compressor, r: u8) {
    c.ratio = match r {
        RATIO_2 => 0.5,
        RATIO_3 => f32::from_bits(0x3eaa_aaab),
        RATIO_4 => 0.25,
        RATIO_6 => f32::from_bits(0x3e2a_aaab),
        RATIO_8 => 0.125,
        _ => 1.0,
    };
}

/// Envelope follower step shared by both channel modes (as compiled):
/// `overdB = max(logf(lin) * LOG_2_DB - tresh, 0)` (fused, `vmaxnm`), `env = overdB + coeff * (env - overdB)`
/// (fused), `gr = expf(env * ((ratio - 1) * DB_2_LOG))`.
#[inline(always)]
fn env_step(c: &mut Compressor, lin: f32, ntresh: f64, ratio_k: f64) -> f64 {
    let over = vmaxnm64(vfma64(ntresh, libm::logf(lin) as f64, LOG_2_DB), 0.0);
    let coeff = if over > c.env { c.attack } else { c.release } as f64;
    c.env = vfma64(over, c.env - over, coeff);
    libm::expf((c.env * ratio_k) as f32) as f64
}

/// C: `void Compressor_process(Compressor *c, int16_t *fxBuffer,int16_t* outL, int16_t* outR, uint16_t size, uint8_t channels, float dryWet)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/Compressor.c:104`. Reference build: out-of-line in flash. Group P2.
/// Calls: `cmsis::__QADD16`, `dsp::fx::compressor::dB2lin`, `dsp::fx::compressor::lin2dB`.
///
/// `outL` is the interleaved output starting at the C `outL` pointer (`outR` is unused in C). The C local
/// arrays `left[size]`/`right[size]` are write-only scratch and not reproduced.
pub fn Compressor_process(c: &mut Compressor, fxBuffer: &[i16], outL: &mut [i16], size: u16, channels: u8, dryWet: f32) {
    if size == 0 {
        return;
    }
    let ratio_k = (c.ratio as f64 - 1.0) * DB_2_LOG;
    let ntresh = -(c.tresh as f64);
    let markup = c.markupGain as f64;
    let mut i: u8 = 0;
    let mut k: usize = 0;
    if channels == 2 {
        loop {
            // the inputs are loaded with the unmasked index 2*i, the dry/wet part with uint8_t idx1/idx2
            let fa = fxBuffer[2 * i as usize];
            let fb = fxBuffer[2 * i as usize + 1];
            let sum = (fa as f32 * INV_32768) as f64 + (fb as f32 * INV_32768) as f64;
            let r#in = (sum * 0.5) as f32;
            let lin = (r#in.abs() as f64 + DC_OFFSET) as f32;
            let gr = env_step(c, lin, ntresh, ratio_k);
            let fx1 = fxBuffer[i.wrapping_mul(2) as usize];
            let fx2 = fxBuffer[i.wrapping_mul(2).wrapping_add(1) as usize];
            let left = ssat16(((fa as f64 * markup) * gr) as i32);
            let right = ssat16(((fb as f64 * markup) * gr) as i32);
            let lo = fx1 as i32 + ssat16((((left - fx1 as i32) as f32) * dryWet) as i32);
            let hi = fx2 as i32 + ssat16((((right - fx2 as i32) as f32) * dryWet) as i32);
            let w = __QADD16(ld32(outL, k), pkhbt(lo, hi));
            st32(outL, k, w);
            k += 2;
            i = i.wrapping_add(1);
            if i as u16 >= size {
                break;
            }
        }
    } else {
        loop {
            let fx = fxBuffer[i.wrapping_mul(2) as usize];
            let lin = ((fx as f32 * INV_32768).abs() as f64 + DC_OFFSET) as f32;
            let gr = env_step(c, lin, ntresh, ratio_k);
            // left[i] = fx * gr * markupGain (no saturation in the mono path: int16_t truncation)
            let left = (((fx as f64 * markup) * gr) as i32) as i16;
            let d = (fx as i32 + ssat16((((left as i32 - fx as i32) as f32) * dryWet) as i32)) as i16;
            outL[k] = __QADD16(sx(outL[k]), sx(d)) as i16;
            k += 2;
            i = i.wrapping_add(1);
            if i as u16 >= size {
                break;
            }
        }
    }
}
