//! Deterministic replacement of the hardware RNG (docs/ARCHITECTURE.md, "Engine contract").
//!
//! The firmware's `GetRngValue()` (`Core/Src/Soundengine/DSPAudio/random.c:27`) returns
//! `HAL_RNG_GetRandomNumber(&hrng)`. Both `lxr-core` and `lxr-emu` replace it with xorshift32
//! (`x ^= x << 13; x ^= x >> 17; x ^= x << 5;`), state seeded with `RNG_SEED` at power-on, returning the
//! new state. Every call consumes exactly one value.

use crate::engine::Lxr;

/// Initial state of the generator.
pub const RNG_SEED: u32 = 0x2545_F491;

/// One xorshift32 step (shifts 13, 17, 5): updates `state` and returns the new state.
#[inline(always)]
pub fn xorshift32(state: &mut u32) -> u32 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 17;
    x ^= x << 5;
    *state = x;
    x
}

impl Lxr {
    /// C: `uint32_t GetRngValue()`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/random.c:27` (hardware RNG, replaced; see module docs).
    #[inline]
    pub fn GetRngValue(&mut self) -> u32 {
        xorshift32(&mut self.rng_state)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xorshift_sequence() {
        let mut s = RNG_SEED;
        let a = xorshift32(&mut s);
        let mut x = RNG_SEED;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        assert_eq!(a, x);
        assert_eq!(s, a);
        // the generator never reaches 0 from a non-zero state
        for _ in 0..10_000 {
            assert_ne!(xorshift32(&mut s), 0);
        }
    }

    #[test]
    fn lxr_rng_is_seeded_and_consumes_one_value_per_call() {
        let mut lxr = Lxr::new_zeroed();
        assert_eq!(lxr.rng_state, RNG_SEED);
        let mut s = RNG_SEED;
        for _ in 0..4 {
            assert_eq!(lxr.GetRngValue(), xorshift32(&mut s));
        }
    }
}
