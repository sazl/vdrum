//! Port of `Core/Inc/Soundengine/MIDI/valueShaper.h`.
//!
//! Porting group: P3. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.

use crate::prelude::*;

/// `k = 2*shape/(1.0001f-shape)` in single precision (GCC folds it at compile time with the same roundings).
#[inline(always)]
fn shaper_k(shape: f32) -> f32 {
    2.0 * shape / (1.0001 - shape)
}

// ---- functions without access to globals
/// C: `static inline float valueShaperI2F(uint8_t data2, float shape)`
///
/// Source: `Core/Inc/Soundengine/MIDI/valueShaper.h:22`. Reference build: not in the reference ELF (file not
/// linked or function optimised away). Group P3. Ported with the instruction pattern of the inlined
/// `valueShaperF2F` copies (unverifiable: no compiled copy exists).
pub fn valueShaperI2F(data2: u8, shape: f32) -> f32 {
    let val = (data2 as i32 as f32) * crate::midi::midi_parser::RECIP_127;
    valueShaperF2F(val, shape)
}

/// C: `static inline float valueShaperF2F(float val, float shape)`
///
/// Source: `Core/Inc/Soundengine/MIDI/valueShaper.h:30`. Reference build: inlined into its callers only. Group P3.
/// Machine code (inlined copies in `midiParser_ccHandler`): `(val*(1+k)) / vfma(1.0, |val|, k)` with the
/// constants folded (shape -0.9: 1+k = 0x3d57c880, k = 0xbf728378; shape -0.7: 0x3e34c168, 0xbf52cfa6).
pub fn valueShaperF2F(val: f32, shape: f32) -> f32 {
    let k = shaper_k(shape);
    let k1 = 1.0 + k;
    (val * k1) / vfma(1.0, val.abs(), k)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn folded_constants_match_firmware() {
        let k = shaper_k(-0.9);
        assert_eq!((1.0 + k).to_bits(), 0x3d57c880);
        assert_eq!(k.to_bits(), 0xbf728378);
        let k = shaper_k(-0.7);
        assert_eq!((1.0 + k).to_bits(), 0x3e34c168);
        assert_eq!(k.to_bits(), 0xbf52cfa6);
    }
}
