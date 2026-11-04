// SPDX-License-Identifier: GPL-3.0-only
//! What the surface reads from the plugin, and what it asks the plugin to do.

use lxr_params::{Dtype, ParamInfo};

/// Highest step of a voice fader (the plugin's `FADER_MAX`).
pub const FADER_MAX: i32 = 99;

/// Read access to the plugin's state. Every call must be cheap and must not block.
pub trait Model {
    /// The current value of a parameter (engine units, `info.min..=info.max`).
    fn param(&self, idx: u16) -> u16;
    /// The plugin's voice fader of track `t` (0..7), 0..=[`FADER_MAX`].
    fn voice_fader(&self, t: u8) -> i32;
    /// The kit name.
    fn kit_name(&self) -> String;
    /// A counter that increases every time track `t` is triggered, from any source.
    fn hits(&self, t: u8) -> u32;
    /// The state of one of the plugin's switches.
    fn switch(&self, _s: Switch) -> bool {
        false
    }
}

/// The plugin's output switches (host parameters).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Switch {
    /// Sum the aux output (DAC 2) into the main output.
    SumAux,
    /// Run the engine at the hardware's sample rate.
    HwClock,
}

/// A request from the surface to the plugin.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Action {
    /// Set an engine parameter.
    SetParam { idx: u16, value: u16 },
    /// Move a voice fader (0..=[`FADER_MAX`]).
    SetVoiceFader { track: u8, step: i32 },
    /// Trigger a track.
    Trigger { track: u8, velocity: u8 },
    /// A message for the device itself (feature controls).
    Device(Vec<u8>),
    /// Show the next (+1) or previous (-1) kit of the current kit's folder; the plugin reports the
    /// name with [`crate::Surface::notify`].
    KitBrowse(i8),
    /// Load the kit shown by [`Action::KitBrowse`].
    KitLoadBrowsed,
    /// Load the init kit.
    InitKit,
    /// Save the kit to its file.
    SaveKit,
    /// Start (true) or end comparing with the kit as last loaded or saved.
    Compare(bool),
    /// Silence every voice.
    Panic,
    /// Set one of the plugin's switches.
    SetSwitch(Switch, bool),
}

/// The values the controller can choose for a parameter: `min..=max`, except for modulation
/// destinations, which offer "off" and the targets of their destination voice (as the editor and
/// the firmware's menu do).
pub fn options(info: &ParamInfo, model: &dyn Model) -> Vec<u16> {
    use lxr_params::open_hat::{oh_index, oh_k, OH_MAX_TARGET, OH_TARGET_BASE, OH_VOICE_LFO};
    use lxr_params::par;
    if !info.is_mod_target() {
        return (info.min..=info.max).collect();
    }
    let oh = oh_k(info.index).is_some();
    let voice = if oh && info.dtype == Dtype::TargetSelectionLfo {
        model.param(oh_index(OH_VOICE_LFO)).clamp(1, 7) as u8 - 1
    } else if oh {
        6
    } else if info.dtype == Dtype::TargetSelectionLfo {
        let lfo = info.index - par::TARGET_LFO1;
        model.param(par::VOICE_LFO1 + lfo).clamp(1, 6) as u8 - 1
    } else {
        info.voice.number().unwrap_or(0)
    };
    let (lo, hi) =
        if voice == 6 { (OH_TARGET_BASE, OH_MAX_TARGET) } else { ParamInfo::mod_target_range_for_voice(voice) };
    let mut v: Vec<u16> = std::iter::once(0).chain(lo as u16..=hi as u16).collect();
    let cur = model.param(info.index);
    if !v.contains(&cur) {
        v.push(cur);
        v.sort_unstable();
    }
    v
}

/// The value of an absolute 7-bit control position for a parameter range (as the plugin's
/// `scale_cc`).
pub fn scale(min: u16, max: u16, raw: u8) -> u16 {
    let span = (max - min) as u32;
    min + ((raw.min(127) as u32 * span + 63) / 127) as u16
}

/// The 7-bit control position closest to `value` (inverse of [`scale`]).
pub fn unscale(min: u16, max: u16, value: u16) -> u8 {
    let span = (max - min) as u32;
    if span == 0 {
        return 0;
    }
    let v = value.clamp(min, max) as u32 - min as u32;
    ((v * 127 + span / 2) / span) as u8
}
