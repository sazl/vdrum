// SPDX-License-Identifier: GPL-3.0-only
//! The firmware runs all engine code with flush-to-zero off (IEEE subnormals), but plugin wrappers
//! (nih-plug included) enable FTZ/DAZ during `process()`. [`IeeeGuard`] clears them for the duration
//! of an engine call and restores the previous state when dropped.
//!
//! * x86/x86_64: MXCSR bit 15 (FTZ) and bit 6 (DAZ).
//! * aarch64: FPCR bit 24 (FZ).
//! * Other targets: no-op.

/// Clears FTZ/DAZ while alive; restores the previous floating point control state on drop.
pub struct IeeeGuard {
    #[allow(dead_code)]
    saved: u64,
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
mod imp {
    const FTZ: u32 = 1 << 15;
    const DAZ: u32 = 1 << 6;

    #[inline(always)]
    fn get() -> u32 {
        let mut v: u32 = 0;
        // SAFETY: stmxcsr stores the 32 bit MXCSR register to the given address.
        unsafe { std::arch::asm!("stmxcsr [{}]", in(reg) &mut v, options(nostack, preserves_flags)) };
        v
    }

    #[inline(always)]
    fn set(v: u32) {
        // SAFETY: ldmxcsr loads MXCSR from the given address; only mode bits are changed.
        unsafe { std::arch::asm!("ldmxcsr [{}]", in(reg) &v, options(nostack, preserves_flags)) };
    }

    #[inline(always)]
    pub fn enter() -> u64 {
        let v = get();
        if v & (FTZ | DAZ) != 0 {
            set(v & !(FTZ | DAZ));
        }
        v as u64
    }

    #[inline(always)]
    pub fn leave(saved: u64) {
        if saved as u32 & (FTZ | DAZ) != 0 {
            set(saved as u32);
        }
    }

    pub fn flush_to_zero_enabled() -> bool {
        get() & FTZ != 0
    }

    #[cfg(test)]
    pub fn force(on: bool) {
        let v = get();
        set(if on { v | FTZ | DAZ } else { v & !(FTZ | DAZ) });
    }
}

#[cfg(target_arch = "aarch64")]
mod imp {
    const FZ: u64 = 1 << 24;

    #[inline(always)]
    fn get() -> u64 {
        let v: u64;
        // SAFETY: reading FPCR has no side effects.
        unsafe { std::arch::asm!("mrs {}, fpcr", out(reg) v, options(nomem, nostack, preserves_flags)) };
        v
    }

    #[inline(always)]
    fn set(v: u64) {
        // SAFETY: writing FPCR only changes floating point modes.
        unsafe { std::arch::asm!("msr fpcr, {}", in(reg) v, options(nomem, nostack, preserves_flags)) };
    }

    #[inline(always)]
    pub fn enter() -> u64 {
        let v = get();
        if v & FZ != 0 {
            set(v & !FZ);
        }
        v
    }

    #[inline(always)]
    pub fn leave(saved: u64) {
        if saved & FZ != 0 {
            set(saved);
        }
    }

    pub fn flush_to_zero_enabled() -> bool {
        get() & FZ != 0
    }

    #[cfg(test)]
    pub fn force(on: bool) {
        let v = get();
        set(if on { v | FZ } else { v & !FZ });
    }
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "x86", target_arch = "aarch64")))]
mod imp {
    pub fn enter() -> u64 {
        0
    }
    pub fn leave(_: u64) {}
    pub fn flush_to_zero_enabled() -> bool {
        false
    }
    #[cfg(test)]
    pub fn force(_: bool) {}
}

pub use imp::flush_to_zero_enabled;

impl IeeeGuard {
    #[inline(always)]
    #[must_use]
    pub fn new() -> Self {
        IeeeGuard { saved: imp::enter() }
    }
}

impl Default for IeeeGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for IeeeGuard {
    #[inline(always)]
    fn drop(&mut self) {
        imp::leave(self.saved);
    }
}

#[cfg(all(test, any(target_arch = "x86_64", target_arch = "aarch64")))]
mod tests {
    use super::*;

    #[test]
    fn guard_clears_and_restores_ftz() {
        imp::force(true);
        assert!(flush_to_zero_enabled());
        {
            let _g = IeeeGuard::new();
            assert!(!flush_to_zero_enabled());
            // A subnormal survives arithmetic inside the guard.
            let tiny = std::hint::black_box(f32::MIN_POSITIVE);
            assert!(std::hint::black_box(tiny / 4.0) > 0.0);
        }
        assert!(flush_to_zero_enabled());
        imp::force(false);
        {
            let _g = IeeeGuard::new();
            assert!(!flush_to_zero_enabled());
        }
        assert!(!flush_to_zero_enabled());
    }
}
