//! Port of `Core/Src/Soundengine/DSPAudio/FX/` (master effects of the FX bus).

pub mod compressor;
pub mod delay;
pub mod distortion;
pub mod fx_ram_buffer;
pub mod reverb;
pub mod ringmod;
pub mod verb_tools;

// ---- helpers shared by the FX modules (CMSIS macros and instructions as used by the machine code)

/// `__SSAT(x, 16)`: signed saturation to 16 bits (`ssat rX, #16, rX`).
#[inline(always)]
pub(crate) fn ssat16(x: i32) -> i32 {
    x.clamp(-32768, 32767)
}

/// `__PKHBT(lo, hi, 16)` = `(lo & 0xFFFF) | (hi << 16)` (`uxth` + `orr ..., lsl #16`).
#[inline(always)]
pub(crate) fn pkhbt(lo: i32, hi: i32) -> u32 {
    (lo as u32 & 0xFFFF) | ((hi as u32) << 16)
}

/// Sign-extended 32 bit register value of an `int16_t` (`ldrsh`).
#[inline(always)]
pub(crate) fn sx(v: i16) -> u32 {
    v as i32 as u32
}

/// 32 bit load of the interleaved stereo pair at `buf[k]`, `buf[k+1]` (`ldr` through an `int32_t*`).
#[inline(always)]
pub(crate) fn ld32(buf: &[i16], k: usize) -> u32 {
    (buf[k] as u16 as u32) | ((buf[k + 1] as u16 as u32) << 16)
}

/// 32 bit store of an interleaved stereo pair (`str` through an `int32_t*`).
#[inline(always)]
pub(crate) fn st32(buf: &mut [i16], k: usize, v: u32) {
    buf[k] = v as i16;
    buf[k + 1] = (v >> 16) as i16;
}

/// `VMAXNM.F64`: IEEE 754-2008 maxNum (a NaN operand loses against a number, `+0 > -0`).
#[inline(always)]
pub(crate) fn vmaxnm64(a: f64, b: f64) -> f64 {
    if a.is_nan() {
        b
    } else if b.is_nan() {
        a
    } else if a == 0.0 && b == 0.0 {
        if a.is_sign_positive() {
            a
        } else {
            b
        }
    } else if a > b {
        a
    } else {
        b
    }
}

/// `VMAXNM.F32`.
#[inline(always)]
pub(crate) fn vmaxnm32(a: f32, b: f32) -> f32 {
    if a.is_nan() {
        b
    } else if b.is_nan() {
        a
    } else if a == 0.0 && b == 0.0 {
        if a.is_sign_positive() {
            a
        } else {
            b
        }
    } else if a > b {
        a
    } else {
        b
    }
}

/// `VMINNM.F32`.
#[inline(always)]
pub(crate) fn vminnm32(a: f32, b: f32) -> f32 {
    if a.is_nan() {
        b
    } else if b.is_nan() {
        a
    } else if a == 0.0 && b == 0.0 {
        if a.is_sign_negative() {
            a
        } else {
            b
        }
    } else if a < b {
        a
    } else {
        b
    }
}
