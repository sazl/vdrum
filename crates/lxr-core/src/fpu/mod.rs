//! Floating point helpers that reproduce the Cortex-M7 FPU (FPv5-D16) bit for bit.
//!
//! The firmware is built with `-O3 -ffast-math`, so GCC emits fused multiply-add instructions. Every fused
//! operation of the machine code must be written with one of the helpers below (one rounding), never as
//! `a * b + c` (two roundings). `f32::mul_add` / `f64::mul_add` may only be used through this module
//! (docs/ARCHITECTURE.md, rule 4).
//!
//! ARM definitions (ARMv7-M Architecture Reference Manual, A7.7 "VFMA, VFMS", "VFNMA, VFNMS"; the
//! pseudocode is `FPMulAdd(addend, op1, op2)` with operands negated *before* the fused operation, so the
//! negations are exact):
//!
//! | instruction            | result (one rounding)       | helper                 |
//! |------------------------|-----------------------------|------------------------|
//! | `VFMA.F32 Sd, Sn, Sm`  | `Sd + Sn*Sm`                | `vfma(sd, sn, sm)`     |
//! | `VFMS.F32 Sd, Sn, Sm`  | `Sd + (-Sn)*Sm = Sd - Sn*Sm` | `vfms(sd, sn, sm)`     |
//! | `VFNMA.F32 Sd, Sn, Sm` | `(-Sd) + (-Sn)*Sm = -Sd - Sn*Sm` | `vfnma(sd, sn, sm)` |
//! | `VFNMS.F32 Sd, Sn, Sm` | `(-Sd) + Sn*Sm = -Sd + Sn*Sm` | `vfnms(sd, sn, sm)`  |
//!
//! The `.F64` forms are `vfma64` etc. `fma(a, b, c)` is the C `fmaf(a, b, c)` = `a*b + c`.
//!
//! Known limits of the host emulation (check in differential tests when they matter):
//! * NaN payloads: the Cortex-M7 runs with FPSCR.DN as set by the startup code; hosts propagate payloads.
//!   The engine is not expected to produce NaNs.
//! * Flush-to-zero: ARM FZ flushes denormal inputs and results to signed zero; x86 FTZ+DAZ does the same
//!   but detects tininess after rounding, ARM before rounding. Results that round up to the smallest normal
//!   number can differ. AArch64 FPCR.FZ matches ARMv7 FZ.
//! * `mul_add` on x86 without the FMA target feature calls the C library's `fmaf`/`fma` (correctly rounded on
//!   glibc and macOS). Values computed at compile time by rustc are not flushed to zero.

pub mod libm;

/// `fmaf(a, b, c)`: `a * b + c` with a single rounding.
#[inline(always)]
pub fn fma(a: f32, b: f32, c: f32) -> f32 {
    a.mul_add(b, c)
}

/// `fma(a, b, c)` in double precision: `a * b + c` with a single rounding.
#[inline(always)]
pub fn fma64(a: f64, b: f64, c: f64) -> f64 {
    a.mul_add(b, c)
}

/// `VFMA.F32 Sd, Sn, Sm`: `acc + n * m`, one rounding.
#[inline(always)]
pub fn vfma(acc: f32, n: f32, m: f32) -> f32 {
    n.mul_add(m, acc)
}

/// `VFMS.F32 Sd, Sn, Sm`: `acc - n * m` (= `acc + (-n) * m`), one rounding.
#[inline(always)]
pub fn vfms(acc: f32, n: f32, m: f32) -> f32 {
    (-n).mul_add(m, acc)
}

/// `VFNMA.F32 Sd, Sn, Sm`: `-acc - n * m` (= `(-acc) + (-n) * m`), one rounding.
#[inline(always)]
pub fn vfnma(acc: f32, n: f32, m: f32) -> f32 {
    (-n).mul_add(m, -acc)
}

/// `VFNMS.F32 Sd, Sn, Sm`: `-acc + n * m` (= `(-acc) + n * m`), one rounding.
#[inline(always)]
pub fn vfnms(acc: f32, n: f32, m: f32) -> f32 {
    n.mul_add(m, -acc)
}

/// `VFMA.F64 Dd, Dn, Dm`: `acc + n * m`, one rounding.
#[inline(always)]
pub fn vfma64(acc: f64, n: f64, m: f64) -> f64 {
    n.mul_add(m, acc)
}

/// `VFMS.F64 Dd, Dn, Dm`: `acc - n * m`, one rounding.
#[inline(always)]
pub fn vfms64(acc: f64, n: f64, m: f64) -> f64 {
    (-n).mul_add(m, acc)
}

/// `VFNMA.F64 Dd, Dn, Dm`: `-acc - n * m`, one rounding.
#[inline(always)]
pub fn vfnma64(acc: f64, n: f64, m: f64) -> f64 {
    (-n).mul_add(m, -acc)
}

/// `VFNMS.F64 Dd, Dn, Dm`: `-acc + n * m`, one rounding.
#[inline(always)]
pub fn vfnms64(acc: f64, n: f64, m: f64) -> f64 {
    n.mul_add(m, -acc)
}

/// `VCVTB.F32.F16`: IEEE half precision (`__fp16`, `-mfp16-format=ieee`) bits to `f32`.
/// Used for `FX_BUFFER` (`__fp16`). Group P2 (check FZ behaviour for half precision denormals).
pub fn f16_bits_to_f32(h: u16) -> f32 {
    todo!("port: VCVTB.F32.F16 semantics")
}

/// `VCVTB.F16.F32`: `f32` to IEEE half precision bits with the FPSCR rounding mode (round to nearest even).
/// Group P2 (check FZ behaviour and the overflow / NaN cases against the emulator).
pub fn f32_to_f16_bits(v: f32) -> u16 {
    todo!("port: VCVTB.F16.F32 semantics")
}

/// RAII guard that enables flush-to-zero for the duration of a `tick()` (the firmware sets FPSCR.FZ in
/// FPDSCR, so interrupt code runs with FZ = 1; main-loop code runs with FZ = 0).
///
/// * x86 / x86_64: sets MXCSR.FTZ (bit 15) and MXCSR.DAZ (bit 6).
/// * aarch64: sets FPCR.FZ (bit 24).
/// * other targets: no-op.
///
/// The previous control register value is restored on drop.
pub struct FtzGuard {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
    prev: u64,
}

#[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
mod arch {
    pub const FTZ_BITS: u64 = (1 << 15) | (1 << 6);

    #[inline]
    pub fn read() -> u64 {
        let mut csr: u32 = 0;
        // SAFETY: stores MXCSR into a local; no other effects.
        unsafe {
            core::arch::asm!("stmxcsr [{}]", in(reg) &mut csr as *mut u32, options(nostack, preserves_flags));
        }
        csr as u64
    }

    #[inline]
    pub fn write(v: u64) {
        let csr = v as u32;
        // SAFETY: loads MXCSR from a local; only changes the floating point control state.
        unsafe {
            core::arch::asm!("ldmxcsr [{}]", in(reg) &csr as *const u32, options(nostack, preserves_flags));
        }
    }
}

#[cfg(target_arch = "aarch64")]
mod arch {
    pub const FTZ_BITS: u64 = 1 << 24;

    #[inline]
    pub fn read() -> u64 {
        let v: u64;
        // SAFETY: reads FPCR.
        unsafe {
            core::arch::asm!("mrs {}, fpcr", out(reg) v, options(nomem, nostack, preserves_flags));
        }
        v
    }

    #[inline]
    pub fn write(v: u64) {
        // SAFETY: writes FPCR; only changes the floating point control state.
        unsafe {
            core::arch::asm!("msr fpcr, {}", in(reg) v, options(nomem, nostack, preserves_flags));
        }
    }
}

impl FtzGuard {
    /// Enables flush-to-zero (and denormals-are-zero on x86) until the guard is dropped.
    #[inline]
    pub fn new() -> Self {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
        {
            let prev = arch::read();
            arch::write(prev | arch::FTZ_BITS);
            FtzGuard { prev }
        }
        #[cfg(not(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64")))]
        {
            FtzGuard {}
        }
    }
}

impl Default for FtzGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for FtzGuard {
    #[inline]
    fn drop(&mut self) {
        #[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
        arch::write(self.prev);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // a = 1 + 2^-12, a*a = 1 + 2^-11 + 2^-24: the 2^-24 term is lost when the product is rounded first.
    const A: f32 = 1.0 + 1.0 / 4096.0;

    #[test]
    fn fused_single_rounding() {
        let p = A * A; // rounded product
        assert_eq!(fma(A, A, -p), f32::from_bits(0x3380_0000));
        assert_eq!(vfma(-p, A, A), f32::from_bits(0x3380_0000));
        assert_eq!(A * A - p, 0.0);
    }

    #[test]
    fn sign_conventions() {
        let (acc, n, m) = (10.0f32, 3.0f32, 2.0f32);
        assert_eq!(vfma(acc, n, m), 16.0);
        assert_eq!(vfms(acc, n, m), 4.0);
        assert_eq!(vfnma(acc, n, m), -16.0);
        assert_eq!(vfnms(acc, n, m), -4.0);
        assert_eq!(fma(n, m, acc), 16.0);
        let (acc, n, m) = (10.0f64, 3.0f64, 2.0f64);
        assert_eq!(vfma64(acc, n, m), 16.0);
        assert_eq!(vfms64(acc, n, m), 4.0);
        assert_eq!(vfnma64(acc, n, m), -16.0);
        assert_eq!(vfnms64(acc, n, m), -4.0);
    }

    #[test]
    fn fused_residuals() {
        let p = A * A;
        // vfms: p - A*A exactly
        assert_eq!(vfms(p, A, A), -(f32::from_bits(0x3380_0000)));
        // vfnma: -p - A*(-A) = A*A - p
        assert_eq!(vfnma(p, A, -A), f32::from_bits(0x3380_0000));
        // vfnms: -p + A*A
        assert_eq!(vfnms(p, A, A), f32::from_bits(0x3380_0000));
        let b = 1.0 + f64::from_bits(0x3E40_0000_0000_0000);
        let q = b * b;
        assert_eq!(vfnms64(q, b, b), f64::from_bits(0x3C90_0000_0000_0000));
        assert_eq!(fma64(b, b, -q), f64::from_bits(0x3C90_0000_0000_0000));
    }

    #[test]
    fn zero_signs() {
        // (+0) - (+0)*(+1) = +0 ; -(+0) - 0*1 = -0
        assert_eq!(vfms(0.0, 0.0, 1.0).to_bits(), 0.0f32.to_bits());
        assert_eq!(vfnma(0.0, 0.0, 1.0).to_bits(), (-0.0f32).to_bits());
        assert_eq!(vfnms(0.0, 0.0, 1.0).to_bits(), 0.0f32.to_bits());
    }

    #[test]
    fn ftz_guard() {
        let tiny = f32::from_bits(0x0000_0010); // denormal
        let x = std::hint::black_box(tiny);
        assert_ne!(std::hint::black_box(x * 2.0), 0.0);
        {
            let _g = FtzGuard::new();
            #[cfg(any(target_arch = "x86", target_arch = "x86_64", target_arch = "aarch64"))]
            assert_eq!(std::hint::black_box(std::hint::black_box(x) * 2.0), 0.0);
        }
        assert_ne!(std::hint::black_box(std::hint::black_box(x) * 2.0), 0.0);
    }
}
