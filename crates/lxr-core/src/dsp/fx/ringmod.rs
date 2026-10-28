//! Port of `Core/Inc/Soundengine/DSPAudio/FX/Ringmod.h`, `Core/Src/Soundengine/DSPAudio/FX/Ringmod.c`.
//!
//! Porting group: P2. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;

// ---- types
/// C: `struct RingmodStruct` (Core/Inc/Soundengine/DSPAudio/FX/Ringmod.h:23), 44 bytes.
/// C typedef name `Ringmod`; the struct tag `RingmodStruct` is a type alias below.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ringmod {
    /// `float freq; //drive`
    pub freq: f32,
    /// `OscInfo osc;`
    pub osc: OscInfo,
}
/// C struct tag of `Ringmod`.
pub type RingmodStruct = Ringmod;
impl Ringmod {
    /// All-zero value (C static zero initialisation).
    pub const ZERO: Self = Self { freq: 0.0, osc: OscInfo::ZERO };
}
impl Default for Ringmod {
    fn default() -> Self {
        Self::ZERO
    }
}


use super::{ld32, pkhbt, ssat16, st32, sx};
use crate::cmsis::__QADD16;
use crate::dsp::oscillator::osc_setFreq;

// ---- functions without access to globals
/// C: `void Ringmod_init(Ringmod *rm)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/Ringmod.c:18`. Reference build: out-of-line in flash. Group P2.
/// Calls: `dsp::oscillator::osc_setFreq`.
pub fn Ringmod_init(rm: &mut Ringmod) {
    rm.freq = 0.5;
    rm.osc.freq = 100.0;
    rm.osc.modNodeValue = 1.0;
    rm.osc.waveform = REC;
    rm.osc.fmMod = 0.0;
    rm.osc.midiFreq = 70 << 8 | 64;
    rm.osc.pitchMod = 1.0;
    osc_setFreq(&mut rm.osc);
}

/// C: `void Ringmod_setFreq(Ringmod *rm, float f)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/Ringmod.c:79`. Reference build: out-of-line in flash. Group P2.
/// Calls: `dsp::oscillator::osc_setFreq`.
pub fn Ringmod_setFreq(rm: &mut Ringmod, f: f32) {
    rm.osc.freq = f * 880.0;
    osc_setFreq(&mut rm.osc);
}

/// C: `void Ringmod_setWave(Ringmod *rm, uint8_t w)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/Ringmod.c:84`. Reference build: out-of-line in flash. Group P2.
pub fn Ringmod_setWave(rm: &mut Ringmod, w: u8) {
    rm.osc.waveform = w;
}

/// C: `void Ringmod_resetPhase(Ringmod* rm)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/FX/Ringmod.c:89`. Reference build: not linked (removed by --gc-sections). Group P2.
/// Ported from C (not in the firmware build, untestable).
pub fn Ringmod_resetPhase(rm: &mut Ringmod) {
    if rm.osc.waveform == SINE {
        rm.osc.phase = 1024u32.wrapping_add((0x3ffu32 << 20).wrapping_sub(1024));
    } else {
        rm.osc.phase = 0xffu32 << 22;
    }
}

// ---- functions that use globals (methods on `Lxr`)
impl Lxr {
    /// C: `void Ringmod_process(Ringmod *rm, int16_t *fxBuffer, int16_t* outL, int16_t* outR, uint16_t size, uint8_t channels, float dryWet)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/FX/Ringmod.c:31`. Reference build: out-of-line in flash. Group P2.
    /// Calls: `Lxr::calcNextOscSampleBlock`, `cmsis::__QADD16`.
    ///
    /// `outL` is the interleaved output starting at the C `outL` pointer (`outR` is unused in C).
    pub fn Ringmod_process(&mut self, rm: &mut Ringmod, fxBuffer: &[i16], outL: &mut [i16], size: u16, channels: u8, dryWet: f32) {
        // int16_t buf[size] (VLA); the oscillator gets size as uint8_t
        let mut buf = [0i16; 256];
        let n = (size as u8) as usize;
        self.calcNextOscSampleBlock(&mut rm.osc, &mut buf[..n], size as u8, 1.0, f32::from_bits(0x3e99_999a));
        if size == 0 {
            return;
        }
        let mut i: u8 = 0;
        let mut k: usize = 0;
        if channels == 2 {
            loop {
                let idx1 = i.wrapping_mul(2) as usize;
                let idx2 = i.wrapping_mul(2).wrapping_add(1) as usize;
                let fx1 = fxBuffer[idx1] as i32;
                let fx2 = fxBuffer[idx2] as i32;
                let b = buf[i as usize] as i32;
                let wet = (fx1 * b) >> 16;
                let wet2 = (fx2 * b) >> 16;
                let lo = fx1 + ssat16((((wet - fx1) as f32) * dryWet) as i32);
                let hi = fx2 + ssat16((((wet2 - fx2) as f32) * dryWet) as i32);
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
                let idx1 = i.wrapping_mul(2) as usize;
                let fx1 = fxBuffer[idx1];
                // smulbb of the (unsigned loaded) buffer sample and the input: signed 16x16
                let val1 = fx1 as i32 * buf[i as usize] as i32;
                let o = sx(outL[k]);
                let wet = __QADD16(o, (val1 >> 16) as u32) as i16;
                let d = (fx1 as i32 + ssat16((((wet as i32 - fx1 as i32) as f32) * dryWet) as i32)) as i16;
                outL[k] = __QADD16(o, sx(d)) as i16;
                k += 2;
                i = i.wrapping_add(1);
                if i as u16 >= size {
                    break;
                }
            }
        }
    }
}
