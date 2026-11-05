//! Read-only view of the voices' LFOs for the controller's LEDs. Not part of the firmware port.

use crate::dsp::lfo::*;
use crate::prelude::*;

/// The value `lfo_calc` returns for the LFO's current phase (0..=1), without advancing it or
/// drawing a new random value (the noise waveform shows its current value).
pub fn lfo_peek(lfo: &Lfo) -> f32 {
    const INV_2_32: f32 = f32::from_bits(0x2f800000);
    let phase = lfo.phase;
    match lfo.waveform {
        LFO_SINE => {
            let s = sine_table[(phase >> 20) as usize] as i32 as f32;
            vfma(1.0, s, f32::from_bits(0x38000100)) * 0.5
        }
        LFO_TRI => {
            let t = vfma(-1.0, phase as f32, f32::from_bits(0x30000000));
            1.0 - t.abs()
        }
        LFO_SAW_UP => phase as f32 * INV_2_32,
        LFO_REC => {
            if (phase as i32) < 0 {
                1.0
            } else {
                0.0
            }
        }
        LFO_NOISE => lfo.rnd,
        LFO_SAW_DOWN => (!phase) as f32 * INV_2_32,
        LFO_EXP_UP => {
            let x = phase as f32 * INV_2_32;
            x * x * x
        }
        LFO_EXP_DOWN => {
            let x = (!phase) as f32 * INV_2_32;
            x * x * x
        }
        _ => 0.0,
    }
}

impl Lxr {
    /// The current LFO output (0..=1) of each track's voice: Drum 1 to 3, snare, cymbal, closed
    /// hi-hat, open hi-hat.
    pub fn vdrum_lfo_values(&self) -> [f32; 7] {
        [
            lfo_peek(&self.voiceArray[0].lfo),
            lfo_peek(&self.voiceArray[1].lfo),
            lfo_peek(&self.voiceArray[2].lfo),
            lfo_peek(&self.snareVoice.lfo),
            lfo_peek(&self.cymbalVoice.lfo),
            lfo_peek(&self.hatVoice.lfo),
            lfo_peek(&self.vdrum.ohVoice.lfo),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn peek_matches_calc_without_advancing() {
        let mut lxr = Lxr::new();
        for wave in [LFO_SINE, LFO_TRI, LFO_SAW_UP, LFO_SAW_DOWN, LFO_REC, LFO_EXP_UP, LFO_EXP_DOWN] {
            let mut l = Lfo::ZERO;
            l.waveform = wave;
            l.phaseInc = 0x0123_4567;
            l.modNodeValue = 1.0;
            for _ in 0..50 {
                let v = lxr.lfo_calc(&mut l);
                assert_eq!(lfo_peek(&l).to_bits(), v.to_bits(), "wave {wave}");
            }
        }
    }
}
