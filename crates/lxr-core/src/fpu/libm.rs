//! The newlib libm functions that the engine calls, as bit-exact ports of the implementations **in the shipped
//! v1.71 binary** (its newlib differs from the one of GCC 11.3; see docs/ARCHITECTURE.md). Never use the host
//! libm or `f32::exp` etc.
//!
//! Called from linked engine code of the reference ELF: `expf`, `logf`, `exp`, `fmod` (`fmod` reaches
//! `__ieee754_fmod`). `sinf` is only used by `sinCos.c`, which is not linked (Reverb is not part of the
//! firmware); it is listed for completeness. `sqrtf`/`fabsf`/`fabs`/`fmaf` compile to `vsqrt`/`vabs`/`vfma`
//! instructions (use `f32::sqrt`, `f32::abs`, `crate::fpu::fma`).

/// newlib `float expf(float)`.
pub fn expf(x: f32) -> f32 {
    todo!("port: expf of the shipped v1.71 binary")
}

/// newlib `float logf(float)`.
pub fn logf(x: f32) -> f32 {
    todo!("port: logf of the shipped v1.71 binary")
}

/// newlib `double exp(double)`.
pub fn exp(x: f64) -> f64 {
    todo!("port: exp of the shipped v1.71 binary")
}

/// newlib `double fmod(double, double)` (wrapper + `__ieee754_fmod`).
pub fn fmod(x: f64, y: f64) -> f64 {
    todo!("port: fmod of the shipped v1.71 binary")
}

/// newlib `float sinf(float)`. Only used by `sinCos.c`, which is not linked in the firmware.
pub fn sinf(x: f32) -> f32 {
    todo!("port: sinf (not linked in the firmware; only needed if VerbTools/sinCos is ported)")
}
