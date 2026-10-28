//! Port of `Core/Inc/Soundengine/DSPAudio/BufferTools.h`, `Core/Src/Soundengine/DSPAudio/BufferTools.c`.
//!
//! Porting group: P1. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;

// ---- functions without access to globals
/// C: `static inline void bufferTool_addBuffers(int16_t* buf1, int16_t* buf2, const uint8_t size)`
///
/// Source: `Core/Inc/Soundengine/DSPAudio/BufferTools.h:23`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P1.
pub fn bufferTool_addBuffers(buf1: &mut [i16], buf2: &mut [i16], size: u8) {
    todo!("port: Core/Inc/Soundengine/DSPAudio/BufferTools.h:23")
}

/// C: `IN_ITCM_RAM static inline void bufferTool_addBuffersSaturating(int16_t* buf1, int16_t* buf2, const uint8_t size)`
///
/// Source: `Core/Inc/Soundengine/DSPAudio/BufferTools.h:32`. Reference build: inlined into its callers only. Group P1.
/// Calls: `cmsis::__QADD16`.
pub fn bufferTool_addBuffersSaturating(buf1: &mut [i16], buf2: &mut [i16], size: u8) {
    todo!("port: Core/Inc/Soundengine/DSPAudio/BufferTools.h:32")
}

/// C: `static inline void bufferTool_addBuffersSaturatingWithGain(int16_t* buf1, int16_t* buf2, const float gain, const uint8_t size)`
///
/// Source: `Core/Inc/Soundengine/DSPAudio/BufferTools.h:54`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P1.
/// Calls: `cmsis::__QADD16`.
pub fn bufferTool_addBuffersSaturatingWithGain(buf1: &mut [i16], buf2: &mut [i16], gain: f32, size: u8) {
    todo!("port: Core/Inc/Soundengine/DSPAudio/BufferTools.h:54")
}

/// C: `static inline void bufferTool_subBuffersSaturating(int16_t* buf1, int16_t* buf2, const uint8_t size)`
///
/// Source: `Core/Inc/Soundengine/DSPAudio/BufferTools.h:63`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P1.
/// Calls: `cmsis::__QSUB16`.
pub fn bufferTool_subBuffersSaturating(buf1: &mut [i16], buf2: &mut [i16], size: u8) {
    todo!("port: Core/Inc/Soundengine/DSPAudio/BufferTools.h:63")
}

/// C: `static inline void bufferTool_copyWithGain(int16_t* buf1, int16_t* buf2, float gain, const uint8_t size)`
///
/// Source: `Core/Inc/Soundengine/DSPAudio/BufferTools.h:72`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P1.
pub fn bufferTool_copyWithGain(buf1: &mut [i16], buf2: &mut [i16], gain: f32, size: u8) {
    todo!("port: Core/Inc/Soundengine/DSPAudio/BufferTools.h:72")
}

/// C: `IN_ITCM_RAM static inline void bufferTool_clearBuffer(int16_t* buf, const uint8_t size)`
///
/// Source: `Core/Inc/Soundengine/DSPAudio/BufferTools.h:81`. Reference build: inlined into its callers only. Group P1.
pub fn bufferTool_clearBuffer(buf: &mut [i16], size: u8) {
    todo!("port: Core/Inc/Soundengine/DSPAudio/BufferTools.h:81")
}

/// C: `static inline void bufferTool_addGain(int16_t* buf, const float gain, const uint8_t size)`
///
/// Source: `Core/Inc/Soundengine/DSPAudio/BufferTools.h:103`. Reference build: inlined into its callers only. Group P1.
pub fn bufferTool_addGain(buf: &mut [i16], gain: f32, size: u8) {
    todo!("port: Core/Inc/Soundengine/DSPAudio/BufferTools.h:103")
}

/// C: `static inline void bufferTool_addGainInterpolated(int16_t* buf, const float gain, const float lastGain, const uint8_t size)`
///
/// Source: `Core/Inc/Soundengine/DSPAudio/BufferTools.h:121`. Reference build: inlined into its callers only. Group P1.
pub fn bufferTool_addGainInterpolated(buf: &mut [i16], gain: f32, lastGain: f32, size: u8) {
    todo!("port: Core/Inc/Soundengine/DSPAudio/BufferTools.h:121")
}

/// C: `static inline void bufferTool_addGainInterpolatedDouble(int16_t* buf, const double gain, const double lastGain, const uint8_t size)`
///
/// Source: `Core/Inc/Soundengine/DSPAudio/BufferTools.h:173`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P1.
pub fn bufferTool_addGainInterpolatedDouble(buf: &mut [i16], gain: f64, lastGain: f64, size: u8) {
    todo!("port: Core/Inc/Soundengine/DSPAudio/BufferTools.h:173")
}

/// C: `static inline void bufferTool_mulInt(int16_t* buf, const int16_t gain, const uint8_t size)`
///
/// Source: `Core/Inc/Soundengine/DSPAudio/BufferTools.h:203`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P1.
pub fn bufferTool_mulInt(buf: &mut [i16], gain: i16, size: u8) {
    todo!("port: Core/Inc/Soundengine/DSPAudio/BufferTools.h:203")
}

/// C: `static inline void bufferTool_multiplyWithFloatBuffer(int16_t* buf, float* fltBuf, const uint8_t size)`
///
/// Source: `Core/Inc/Soundengine/DSPAudio/BufferTools.h:212`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P1.
pub fn bufferTool_multiplyWithFloatBuffer(buf: &mut [i16], fltBuf: &mut [f32], size: u8) {
    todo!("port: Core/Inc/Soundengine/DSPAudio/BufferTools.h:212")
}

/// C: `static inline void bufferTool_moveBuffer(int16_t* dst, int16_t* src, const uint8_t size)`
///
/// Source: `Core/Inc/Soundengine/DSPAudio/BufferTools.h:231`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P1.
pub fn bufferTool_moveBuffer(dst: &mut [i16], src: &mut [i16], size: u8) {
    todo!("port: Core/Inc/Soundengine/DSPAudio/BufferTools.h:231")
}

// ---- functions that use globals (methods on `Lxr`)
impl Lxr {
    /// C: `static inline void bufferTool_addGainDithered(Dither* dither, int16_t* buf, const float gain, const uint8_t size)`
    ///
    /// Source: `Core/Inc/Soundengine/DSPAudio/BufferTools.h:112`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P1.
    /// Calls: `Lxr::dither_process`.
    pub fn bufferTool_addGainDithered(&mut self, dither: &mut Dither, buf: &mut [i16], gain: f32, size: u8) {
        todo!("port: Core/Inc/Soundengine/DSPAudio/BufferTools.h:112")
    }

    /// C: `static inline void bufferTool_addGainInterpolatedDoubleDithered(Dither* dither, int16_t* buf, const double gain, const double lastGain, const uint8_t size)`
    ///
    /// Source: `Core/Inc/Soundengine/DSPAudio/BufferTools.h:186`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P1.
    /// Calls: `Lxr::dither_process`.
    pub fn bufferTool_addGainInterpolatedDoubleDithered(&mut self, dither: &mut Dither, buf: &mut [i16], gain: f64, lastGain: f64, size: u8) {
        todo!("port: Core/Inc/Soundengine/DSPAudio/BufferTools.h:186")
    }

    /// C: `static inline void bufferTool_multiplyWithFloatBufferDithered(Dither* dither, int16_t* buf, float* fltBuf, const uint8_t size)`
    ///
    /// Source: `Core/Inc/Soundengine/DSPAudio/BufferTools.h:221`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P1.
    /// Calls: `Lxr::dither_process`.
    pub fn bufferTool_multiplyWithFloatBufferDithered(&mut self, dither: &mut Dither, buf: &mut [i16], fltBuf: &mut [f32], size: u8) {
        todo!("port: Core/Inc/Soundengine/DSPAudio/BufferTools.h:221")
    }
}
