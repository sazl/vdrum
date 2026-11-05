//! VDrum extensions of the engine. Not part of the firmware port.
//!
//! Everything in this module is new behaviour that the LXR-02 firmware does not have. It is switched off
//! after [`Lxr::new`]: with the extensions off, the engine runs exactly the ported firmware code (the
//! scenario reference hashes and the differential tests cover that path). The hooks in the ported code are
//! guarded by [`VdrumState::hat_split`] or by modulation destinations that the firmware cannot produce
//! (`>= open_hat::OH_DEST_BASE`), so they do not change the firmware path.
//!
//! * [`open_hat`]: the open hi-hat as a separate (seventh) voice with its own parameters.
//! * [`Lxr::vdrum_all_sound_off`]: silences every voice (the controller's panic action).
//! * [`seq`]: pattern writes and host transport for the plugin's sequencer.

pub mod open_hat;
pub mod lfo_view;
pub mod seq;

use crate::prelude::*;
pub use open_hat::*;

/// State of the VDrum extensions (a field of [`Lxr`]; not a firmware global, not in the `cmem` map).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct VdrumState {
    /// Track 7 (open hi-hat) plays [`VdrumState::ohVoice`] instead of the shared firmware hi-hat voice.
    pub hat_split: bool,
    /// The open hi-hat voice (same DSP as the firmware's `hatVoice`).
    pub ohVoice: HiHatVoice,
    /// Velocity modulator of the open hi-hat (the firmware has six, `velocityModulators[0..6]`).
    pub ohVelocityModulator: ModulationNode,
    /// `mixer_audioRouting` entry of the open hi-hat.
    pub ohAudioRouting: u8,
    /// `mixer_decimation_rate` entry of the open hi-hat (multiplied by the global rate, entry 6).
    pub ohDecimationRate: f32,
    /// `mixer_decimation_cnt` entry of the open hi-hat.
    pub ohDecimationCnt: f32,
    /// `mixer_voice_samples` entry of the open hi-hat.
    pub ohVoiceSample: i16,
    /// `adc_faderValues` entry of the open hi-hat fader (gain).
    pub ohFaderValue: f32,
    /// Last accepted fader ADC reading (hysteresis, as `fader_adc_old`).
    pub ohFaderAdcOld: u16,
    /// Parameter values of the open hi-hat (`activeParameters` semantics), indexed by `OH_*`.
    pub ohParams: [u8; OH_NUM_PARAMS],
    /// The sequencer keeps running but triggers no steps (Play under the host clock).
    pub seq_silent: bool,
}

impl VdrumState {
    pub const ZERO: Self = Self {
        hat_split: false,
        ohVoice: HiHatVoice::ZERO,
        ohVelocityModulator: ModulationNode::ZERO,
        ohAudioRouting: 0,
        ohDecimationRate: 0.0,
        ohDecimationCnt: 0.0,
        ohVoiceSample: 0,
        ohFaderValue: 0.0,
        ohFaderAdcOld: 0,
        ohParams: [0; OH_NUM_PARAMS],
        seq_silent: false,
    };
}

impl Default for VdrumState {
    fn default() -> Self {
        Self::ZERO
    }
}

impl Lxr {
    /// Silences every voice at once: the amplitude envelopes of the seven voices (and the transient
    /// generators) drop to zero, as the open hi-hat choke does. FX bus tails and the sequencer are not
    /// affected. Only called by VDrum's controller; the firmware has no equivalent.
    pub fn vdrum_all_sound_off(&mut self) {
        fn off(eg: &mut SlopeEg2, snap: &mut SnapEg) {
            eg.value = 0.0;
            eg.state = EG_D as u8;
            eg.repeatCnt = 0;
            snap.value = 0.0;
        }
        for v in self.voiceArray.iter_mut() {
            off(&mut v.oscVolEg, &mut v.snapEg);
        }
        off(&mut self.snareVoice.oscVolEg, &mut self.snareVoice.snapEg);
        off(&mut self.cymbalVoice.oscVolEg, &mut self.cymbalVoice.snapEg);
        off(&mut self.hatVoice.oscVolEg, &mut self.hatVoice.snapEg);
        off(&mut self.vdrum.ohVoice.oscVolEg, &mut self.vdrum.ohVoice.snapEg);
    }
}
