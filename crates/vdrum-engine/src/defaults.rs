// SPDX-License-Identifier: GPL-3.0-only
//! VDrum's defaults that differ from the firmware's, the voice fader scale, and a new engine with
//! those defaults applied. Shared by the plugin and the Ableton Move module.

use crate::engine::{Engine, LxrEngine};
use crate::fp;
use lxr_params::{ParamInfo, GLOBAL_PARAMS};

/// Plugin default MIDI notes of the 7 tracks (Drum 1, Drum 2, Drum 3, Snare, Cymbal, closed hi-hat,
/// open hi-hat), all on MIDI channel 1: General MIDI drum notes (36 Bass Drum 1, 45 Low Tom,
/// 50 High Tom, 38 Acoustic Snare, 49 Crash Cymbal 1, 42 Closed Hi-Hat, 46 Open Hi-Hat), so that each
/// voice can be played on its own. The firmware's defaults (`preset_loadDefaultGlobals`, also what
/// `preset_createGlobals` writes to a new `glo.cfg`) put every track on note 48, so one note plays
/// all seven. The plugin sets these after `Lxr::new()` through the firmware's own menu path
/// (`set_parameter(PAR_MIDI_NOTEx)`, which only writes `activeParameters`); a track note only
/// selects which note triggers the voice (the voice then plays `SEQ_DEFAULT_NOTE` either way), so
/// the sound does not change.
pub const DEFAULT_TRACK_NOTES: [u8; 7] = [36, 45, 50, 38, 49, 42, 46];

/// The plugin's default value of a parameter: the firmware default, except for the track notes
/// ([`DEFAULT_TRACK_NOTES`]).
pub fn plugin_default(info: &ParamInfo) -> u16 {
    if (lxr_params::par::MIDI_NOTE1..=lxr_params::par::MIDI_NOTE7).contains(&info.index) {
        DEFAULT_TRACK_NOTES[(info.index - lxr_params::par::MIDI_NOTE1) as usize] as u16
    } else {
        info.default
    }
}

/// Highest step of a voice fader host parameter (0..=`FADER_MAX`).
pub const FADER_MAX: i32 = 99;

/// The 16 bit ADC reading (12 bits, left aligned) of fader step `step`: `36 + 41 * step` in 12 bits.
/// Consecutive steps are 41 counts apart, more than `adc_checkFaders`' hysteresis of 40, so every
/// step reaches the engine; step 0 reads below 1 % (silence), step 99 is the top (`0xFFF0`, the
/// power-on value).
pub fn fader_adc(step: i32) -> u16 {
    ((36 + 41 * step.clamp(0, FADER_MAX)) as u16) << 4
}

/// The gain `adc_checkFaders` derives from a fader step (`v * v`, `v = (adc >> 4) / 4096`, 0 below
/// 0.01).
pub fn fader_gain(step: i32) -> f32 {
    let v = (fader_adc(step) >> 4) as f32 / 4096.0;
    if v < 0.01 {
        0.0
    } else {
        v * v
    }
}

/// A new engine in its power-on state, with VDrum's MIDI defaults ([`DEFAULT_TRACK_NOTES`]) applied
/// through the firmware's parameter path. Allocates; not for the audio thread.
pub fn new_engine() -> Box<LxrEngine> {
    let _ieee = fp::IeeeGuard::new();
    let mut e = LxrEngine::new();
    apply_plugin_defaults(&mut *e);
    e
}

/// Set the parameters whose VDrum default differs from the firmware's power-on value.
pub fn apply_plugin_defaults<E: Engine>(e: &mut E) {
    let _ieee = fp::IeeeGuard::new();
    for p in GLOBAL_PARAMS.iter() {
        let d = plugin_default(p);
        if e.get_parameter(p.index) != d {
            e.set_parameter(p.index, d);
        }
    }
}
