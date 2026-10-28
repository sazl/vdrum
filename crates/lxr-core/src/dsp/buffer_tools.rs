//! Port of `Core/Inc/Soundengine/DSPAudio/BufferTools.h`, `Core/Src/Soundengine/DSPAudio/BufferTools.c`.
//!
//! Porting group: P1. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF.
//!
//! The functions that are inlined into linked callers (`addBuffersSaturating`, `clearBuffer`, `addGain`,
//! `addGainInterpolated`) follow the machine code of those callers (`calcDrumVoiceSyncBlock`,
//! `HiHat_calcSyncBlock`, `Cymbal_calcSyncBlock`, `mixer_calcNextSampleBlock`). The others are not in the
//! reference ELF and are ported from the C semantics.
//!
//! A C float→`int16_t` store compiles to `VCVT.S32.F32` (saturating, NaN → 0) followed by a 16-bit store,
//! i.e. `(x as i32) as i16` in Rust.

use crate::cmsis::{__QADD16, __QSUB16};
use crate::prelude::*;

// ---- functions without access to globals
/// C: `static inline void bufferTool_addBuffers(int16_t* buf1, int16_t* buf2, const uint8_t size)`
///
/// Source: `Core/Inc/Soundengine/DSPAudio/BufferTools.h:23`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P1.
pub fn bufferTool_addBuffers(buf1: &mut [i16], buf2: &mut [i16], size: u8) {
    for i in 0..size as usize {
        buf1[i] = buf1[i].wrapping_add(buf2[i]);
    }
}

/// C: `IN_ITCM_RAM static inline void bufferTool_addBuffersSaturating(int16_t* buf1, int16_t* buf2, const uint8_t size)`
///
/// Source: `Core/Inc/Soundengine/DSPAudio/BufferTools.h:32`. Reference build: inlined into its callers only. Group P1.
/// Calls: `cmsis::__QADD16`.
pub fn bufferTool_addBuffersSaturating(buf1: &mut [i16], buf2: &mut [i16], size: u8) {
    // QADD16 on int32 words: `size/2` words, i.e. `(size/2)*2` samples (an odd last sample is untouched).
    for w in 0..(size / 2) as usize {
        let x = (buf1[2 * w] as u16 as u32) | ((buf1[2 * w + 1] as u16 as u32) << 16);
        let y = (buf2[2 * w] as u16 as u32) | ((buf2[2 * w + 1] as u16 as u32) << 16);
        let r = __QADD16(x, y);
        buf1[2 * w] = r as u16 as i16;
        buf1[2 * w + 1] = (r >> 16) as u16 as i16;
    }
}

/// C: `static inline void bufferTool_addBuffersSaturatingWithGain(int16_t* buf1, int16_t* buf2, const float gain, const uint8_t size)`
///
/// Source: `Core/Inc/Soundengine/DSPAudio/BufferTools.h:54`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P1.
/// Calls: `cmsis::__QADD16`.
pub fn bufferTool_addBuffersSaturatingWithGain(buf1: &mut [i16], buf2: &mut [i16], gain: f32, size: u8) {
    for i in 0..size as usize {
        // the int16 operands are sign-extended to uint32_t; the uint32_t result is converted to float
        let r = __QADD16(buf1[i] as i32 as u32, buf2[i] as i32 as u32);
        buf1[i] = ((r as f32 * gain) as i32) as i16;
    }
}

/// C: `static inline void bufferTool_subBuffersSaturating(int16_t* buf1, int16_t* buf2, const uint8_t size)`
///
/// Source: `Core/Inc/Soundengine/DSPAudio/BufferTools.h:63`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P1.
/// Calls: `cmsis::__QSUB16`.
pub fn bufferTool_subBuffersSaturating(buf1: &mut [i16], buf2: &mut [i16], size: u8) {
    for i in 0..size as usize {
        buf1[i] = __QSUB16(buf1[i] as i32 as u32, buf2[i] as i32 as u32) as u16 as i16;
    }
}

/// C: `static inline void bufferTool_copyWithGain(int16_t* buf1, int16_t* buf2, float gain, const uint8_t size)`
///
/// Source: `Core/Inc/Soundengine/DSPAudio/BufferTools.h:72`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P1.
pub fn bufferTool_copyWithGain(buf1: &mut [i16], buf2: &mut [i16], gain: f32, size: u8) {
    for i in 0..size as usize {
        buf1[i] = ((buf2[i] as f32 * gain) as i32) as i16;
    }
}

/// C: `IN_ITCM_RAM static inline void bufferTool_clearBuffer(int16_t* buf, const uint8_t size)`
///
/// Source: `Core/Inc/Soundengine/DSPAudio/BufferTools.h:81`. Reference build: inlined into its callers only. Group P1.
pub fn bufferTool_clearBuffer(buf: &mut [i16], size: u8) {
    // memset(buf, 0, size/2*sizeof(int32_t)): (size/2)*2 samples
    let n = (size / 2) as usize * 2;
    buf[..n].fill(0);
}

/// C: `static inline void bufferTool_addGain(int16_t* buf, const float gain, const uint8_t size)`
///
/// Source: `Core/Inc/Soundengine/DSPAudio/BufferTools.h:103`. Reference build: inlined into its callers only. Group P1.
pub fn bufferTool_addGain(buf: &mut [i16], gain: f32, size: u8) {
    // vcvt.f32.s32; vmul.f32; vcvt.s32.f32; strh
    for i in 0..size as usize {
        buf[i] = ((buf[i] as f32 * gain) as i32) as i16;
    }
}

/// C: `static inline void bufferTool_addGainInterpolated(int16_t* buf, const float gain, const float lastGain, const uint8_t size)`
///
/// Source: `Core/Inc/Soundengine/DSPAudio/BufferTools.h:121`. Reference build: inlined into its callers only. Group P1.
pub fn bufferTool_addGainInterpolated(buf: &mut [i16], gain: f32, lastGain: f32, size: u8) {
    // As inlined in calcDrumVoiceSyncBlock (-ffast-math):
    //   diff = gain - lastGain (f32); rec = 1.f/((float)size - 1.f)
    //   frac = (float)i * rec; cur = vfma.f64(lastGain, frac, diff); buf = vcvt.s32.f64(buf * cur)
    let diff = (gain - lastGain) as f64;
    let lg = lastGain as f64;
    let rec = 1.0f32 / (size as i32 as f32 - 1.0);
    for i in 0..size as usize {
        let frac = (i as i32 as f32) * rec;
        let cur = fpu::vfma64(lg, frac as f64, diff);
        buf[i] = ((buf[i] as f64 * cur) as i32) as i16;
    }
}

/// C: `static inline void bufferTool_addGainInterpolatedDouble(int16_t* buf, const double gain, const double lastGain, const uint8_t size)`
///
/// Source: `Core/Inc/Soundengine/DSPAudio/BufferTools.h:173`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P1.
pub fn bufferTool_addGainInterpolatedDouble(buf: &mut [i16], gain: f64, lastGain: f64, size: u8) {
    for i in 0..size as usize {
        let frac = i as f64 / (size as f64 - 1.0);
        let current_gain = lastGain + frac * (gain - lastGain);
        buf[i] = ((buf[i] as f64 * current_gain) as i32) as i16;
    }
}

/// C: `static inline void bufferTool_mulInt(int16_t* buf, const int16_t gain, const uint8_t size)`
///
/// Source: `Core/Inc/Soundengine/DSPAudio/BufferTools.h:203`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P1.
pub fn bufferTool_mulInt(buf: &mut [i16], gain: i16, size: u8) {
    for i in 0..size as usize {
        buf[i] = (buf[i] as i32 * gain as i32) as i16;
    }
}

/// C: `static inline void bufferTool_multiplyWithFloatBuffer(int16_t* buf, float* fltBuf, const uint8_t size)`
///
/// Source: `Core/Inc/Soundengine/DSPAudio/BufferTools.h:212`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P1.
pub fn bufferTool_multiplyWithFloatBuffer(buf: &mut [i16], fltBuf: &mut [f32], size: u8) {
    for i in 0..size as usize {
        buf[i] = ((buf[i] as f32 * fltBuf[i]) as i32) as i16;
    }
}

/// C: `static inline void bufferTool_moveBuffer(int16_t* dst, int16_t* src, const uint8_t size)`
///
/// Source: `Core/Inc/Soundengine/DSPAudio/BufferTools.h:231`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P1.
pub fn bufferTool_moveBuffer(dst: &mut [i16], src: &mut [i16], size: u8) {
    let n = size as usize;
    dst[..n].copy_from_slice(&src[..n]);
}

// ---- functions that use globals (methods on `Lxr`)
impl Lxr {
    /// C: `static inline void bufferTool_addGainDithered(Dither* dither, int16_t* buf, const float gain, const uint8_t size)`
    ///
    /// Source: `Core/Inc/Soundengine/DSPAudio/BufferTools.h:112`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P1.
    /// Calls: `Lxr::dither_process`.
    pub fn bufferTool_addGainDithered(&mut self, dither: &mut Dither, buf: &mut [i16], gain: f32, size: u8) {
        for i in 0..size as usize {
            buf[i] = self.dither_process(dither, (buf[i] as f32 / 32768.0) * gain);
        }
    }

    /// C: `static inline void bufferTool_addGainInterpolatedDoubleDithered(Dither* dither, int16_t* buf, const double gain, const double lastGain, const uint8_t size)`
    ///
    /// Source: `Core/Inc/Soundengine/DSPAudio/BufferTools.h:186`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P1.
    /// Calls: `Lxr::dither_process`.
    pub fn bufferTool_addGainInterpolatedDoubleDithered(&mut self, dither: &mut Dither, buf: &mut [i16], gain: f64, lastGain: f64, size: u8) {
        for i in 0..size as usize {
            let mut r#in = (buf[i] as f32 / 32768.0) as f64;
            if r#in > 1.0 {
                r#in = 1.0;
            } else if r#in < -1.0 {
                r#in = -1.0;
            }
            let frac = i as f64 / (size as f64 - 1.0);
            let current_gain = lastGain + frac * (gain - lastGain);
            buf[i] = self.dither_process(dither, (r#in * current_gain) as f32);
        }
    }

    /// C: `static inline void bufferTool_multiplyWithFloatBufferDithered(Dither* dither, int16_t* buf, float* fltBuf, const uint8_t size)`
    ///
    /// Source: `Core/Inc/Soundengine/DSPAudio/BufferTools.h:221`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P1.
    /// Calls: `Lxr::dither_process`.
    pub fn bufferTool_multiplyWithFloatBufferDithered(&mut self, dither: &mut Dither, buf: &mut [i16], fltBuf: &mut [f32], size: u8) {
        for i in 0..size as usize {
            buf[i] = self.dither_process(dither, (buf[i] as f32 / 32768.0) * fltBuf[i]);
        }
    }
}
