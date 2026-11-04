// SPDX-License-Identifier: GPL-3.0-only
//! Parameters of VDrum's separate open hi-hat voice (VDrum extension, not part of the firmware).
//!
//! The firmware has one hi-hat voice for tracks 6 and 7 (closed and open hi-hat), which share every
//! parameter except the decay. VDrum plays track 7 on a second hi-hat voice with its own parameters
//! (`lxr_core::vdrum::open_hat`). They are numbered after the firmware's parameters:
//! index [`OH_PARAM_BASE`]` + k`, `k` in `0..`[`OH_NUM_PARAMS`] (the `OH_*` order of `lxr-core`). Each one
//! corresponds to a firmware hi-hat parameter ([`OH_SPECS`]): same range (except where noted), same value
//! meaning, and the source of its value when a firmware kit is imported ([`oh_from_hat`]).
//!
//! Modulation destinations of the open hat's LFO and velocity modulator are indices into an extended
//! list: `0..229` are the firmware's [`MOD_TARGETS`], `229..=254` ([`OH_MOD_TARGETS`]) are parameters of
//! the open hat.

use crate::{par, Dtype, MidiMapping, ParamInfo, ParamKind, Section, Voice, MOD_TARGETS, NUM_PARAMS, PARAMS};
use std::sync::OnceLock;

/// Number of open hi-hat parameters.
pub const OH_NUM_PARAMS: usize = 37;
/// Parameter index of the first open hi-hat parameter.
pub const OH_PARAM_BASE: u16 = NUM_PARAMS;
/// Number of parameter indices including the open hi-hat's (`0..NUM_ALL_PARAMS`).
pub const NUM_ALL_PARAMS: u16 = NUM_PARAMS + OH_NUM_PARAMS as u16;

/// Index of the choke parameter ("closed hi-hat silences open hi-hat"), which has no firmware twin.
pub const OH_CHOKE: usize = 36;
/// Index of the open hat's decay.
pub const OH_VELOD: usize = 12;
pub const OH_VEL_DEST: usize = 23;
pub const OH_VOICE_LFO: usize = 25;
pub const OH_TARGET_LFO: usize = 26;
pub const OH_RETRIGGER_LFO: usize = 27;

/// `(host id, firmware twin)` of every open hi-hat parameter, in `lxr-core`'s `OH_*` order. The choke has
/// no twin (`None`).
pub const OH_SPECS: [(&str, Option<u16>); OH_NUM_PARAMS] = [
    ("wave1_oh", Some(par::WAVE1_HH)),
    ("coarse7", Some(par::COARSE6)),
    ("fine7", Some(par::FINE6)),
    ("wave2_oh", Some(par::WAVE2_HH)),
    ("wave3_oh", Some(par::WAVE3_HH)),
    ("mod_osc_f1_oh", Some(par::MOD_OSC_F1)),
    ("mod_osc_f2_oh", Some(par::MOD_OSC_F2)),
    ("mod_osc_gain1_oh", Some(par::MOD_OSC_GAIN1)),
    ("mod_osc_gain2_oh", Some(par::MOD_OSC_GAIN2)),
    ("filter_freq_7", Some(par::FILTER_FREQ_6)),
    ("reso_7", Some(par::RESO_6)),
    ("veloa7", Some(par::VELOA6)),
    ("velod7", Some(par::VELOD6_OPEN)),
    ("vol_slope7", Some(par::VOL_SLOPE6)),
    ("vol7", Some(par::VOL6)),
    ("pan7", Some(par::PAN6)),
    ("oh_distortion", Some(par::HAT_DISTORTION)),
    ("voice_decimation7", Some(par::VOICE_DECIMATION6)),
    ("freq_lfo7", Some(par::FREQ_LFO6)),
    ("amount_lfo7", Some(par::AMOUNT_LFO6)),
    ("filter_drive_7", Some(par::FILTER_DRIVE_6)),
    ("volume_mod_on_off7", Some(par::VOLUME_MOD_ON_OFF6)),
    ("velo_mod_amt_7", Some(par::VELO_MOD_AMT_6)),
    ("vel_dest_7", Some(par::VEL_DEST_6)),
    ("wave_lfo7", Some(par::WAVE_LFO6)),
    ("voice_lfo7", Some(par::VOICE_LFO6)),
    ("target_lfo7", Some(par::TARGET_LFO6)),
    ("retrigger_lfo7", Some(par::RETRIGGER_LFO6)),
    ("sync_lfo7", Some(par::SYNC_LFO6)),
    ("offset_lfo7", Some(par::OFFSET_LFO6)),
    ("filter_type_7", Some(par::FILTER_TYPE_6)),
    ("trans7_vol", Some(par::TRANS6_VOL)),
    ("trans7_wave", Some(par::TRANS6_WAVE)),
    ("trans7_freq", Some(par::TRANS6_FREQ)),
    ("audio_out7", Some(par::AUDIO_OUT6)),
    ("pwm_7", Some(par::PWM_6)),
    ("hat_choke", None),
];

/// First modulation target index of the open hi-hat.
pub const OH_TARGET_BASE: u8 = 229;

/// The open hi-hat modulation targets, index `OH_TARGET_BASE + i`: (open hi-hat parameter `k`, the
/// firmware hi-hat [`MOD_TARGETS`] index it corresponds to). Same table as `lxr-core`.
pub const OH_MOD_TARGETS: [(u8, u8); 26] = [
    (1, 191),
    (2, 192),
    (0, 193),
    (35, 194),
    (11, 195),
    (12, 197),
    (13, 198),
    (5, 202),
    (6, 203),
    (7, 204),
    (8, 205),
    (3, 206),
    (4, 207),
    (32, 208),
    (31, 209),
    (33, 210),
    (9, 211),
    (10, 212),
    (30, 213),
    (20, 214),
    (18, 215),
    (19, 217),
    (14, 221),
    (15, 222),
    (17, 223),
    (16, 224),
];

/// Highest open hi-hat modulation target index.
pub const OH_MAX_TARGET: u8 = OH_TARGET_BASE + OH_MOD_TARGETS.len() as u8 - 1;

/// The parameter index of open hi-hat parameter `k`.
pub const fn oh_index(k: usize) -> u16 {
    OH_PARAM_BASE + k as u16
}

/// The open hi-hat parameter `k` of a parameter index, if it is one.
pub fn oh_k(index: u16) -> Option<usize> {
    let k = index.checked_sub(OH_PARAM_BASE)? as usize;
    (k < OH_NUM_PARAMS).then_some(k)
}

/// The open hi-hat target for a firmware hi-hat target (`191..=224`; the closed decay maps to the decay).
pub fn oh_target_for_hat_target(t: u8) -> Option<u8> {
    let t = if t == 196 { 197 } else { t };
    OH_MOD_TARGETS.iter().position(|(_, ch)| *ch == t).map(|i| OH_TARGET_BASE + i as u8)
}

/// The parameter index (an open hi-hat parameter) an open hi-hat target stands for.
pub fn oh_target_param(t: u8) -> Option<u16> {
    let i = t.checked_sub(OH_TARGET_BASE)? as usize;
    OH_MOD_TARGETS.get(i).map(|(k, _)| oh_index(*k as usize))
}

/// The firmware hi-hat target an open hi-hat target corresponds to.
pub fn hat_target_for_oh_target(t: u8) -> Option<u8> {
    let i = t.checked_sub(OH_TARGET_BASE)? as usize;
    OH_MOD_TARGETS.get(i).map(|(_, ch)| *ch)
}

/// `(min, max)` of open hi-hat parameter `k`: the twin's range, except the destinations (which include
/// the open hat's targets), the LFO voice (1..=7) and the LFO retrigger (0..=7, 7 = open hi-hat).
pub fn oh_param_range(k: usize) -> (u16, u16) {
    match k {
        OH_CHOKE => (0, 1),
        OH_VEL_DEST | OH_TARGET_LFO => (0, OH_MAX_TARGET as u16),
        OH_VOICE_LFO => (1, 7),
        OH_RETRIGGER_LFO => (0, 7),
        _ => {
            let t = &PARAMS[OH_SPECS[k].1.unwrap() as usize];
            (t.min, t.max)
        }
    }
}

/// The open hi-hat values imported from a firmware kit (one shared hi-hat): every parameter from its
/// twin (the decay from the open decay), destinations on the hi-hat moved to the same parameter of the
/// open hat (and the LFO voice to 7), choke on. The same as `Lxr::oh_import_from_hat`.
pub fn oh_from_hat(values: &[u8]) -> [u8; OH_NUM_PARAMS] {
    let mut v = [0u8; OH_NUM_PARAMS];
    for (k, (_, twin)) in OH_SPECS.iter().enumerate() {
        v[k] = match twin {
            Some(p) => values.get(*p as usize).copied().unwrap_or(0),
            None => 1,
        };
    }
    for k in [OH_TARGET_LFO, OH_VEL_DEST] {
        if let Some(t) = oh_target_for_hat_target(v[k]) {
            v[k] = t;
        }
    }
    if v[OH_TARGET_LFO] >= OH_TARGET_BASE {
        v[OH_VOICE_LFO] = 7;
    }
    v
}

/// Clamps open hi-hat values to their ranges (invalid destinations become 0, as the firmware does with
/// invalid kit destinations).
pub fn oh_normalize(v: &mut [u8; OH_NUM_PARAMS]) {
    for k in 0..OH_NUM_PARAMS {
        let (lo, hi) = oh_param_range(k);
        if k == OH_VEL_DEST || k == OH_TARGET_LFO {
            if v[k] as u16 > hi {
                v[k] = 0;
            }
        } else {
            v[k] = (v[k] as u16).clamp(lo, hi) as u8;
        }
    }
}

fn leak(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

/// The [`ParamInfo`] of every open hi-hat parameter (index order). Names: "Open Hi-hat ..." (the twin's
/// name with "Hi-hat" replaced); no MIDI CC/NRPN mapping (the firmware has none for them).
pub fn oh_params() -> &'static [ParamInfo] {
    static P: OnceLock<Vec<ParamInfo>> = OnceLock::new();
    P.get_or_init(|| {
        OH_SPECS
            .iter()
            .enumerate()
            .map(|(k, (id, twin))| {
                let (min, max) = oh_param_range(k);
                let no_midi = MidiMapping { cc: None, cc_voice: 0, nrpn: None, engine_handles: false, front_translates_nrpn: false };
                let base = match twin {
                    Some(p) => {
                        let t = &PARAMS[*p as usize];
                        let name = match k {
                            OH_VELOD => "Open Hi-hat Amp EG Decay".to_string(),
                            _ => t.name.replacen("Hi-hat", "Open Hi-hat", 1),
                        };
                        let long = if k == OH_VELOD { "Decay" } else { t.long_name };
                        ParamInfo { name: leak(name), long_name: long, ..*t }
                    }
                    None => ParamInfo {
                        index: 0,
                        enum_name: "",
                        id: "",
                        kind: ParamKind::Sound,
                        voice: Voice::OpenHat,
                        section: Section::AmpEg,
                        text: None,
                        short_name: "chk",
                        category: "Amp EG",
                        long_name: "Closed Hat Choke",
                        name: "Open Hi-hat Amp EG Closed Hat Choke",
                        description: "A closed hi-hat hit silences the open hi-hat.",
                        dtype: Dtype::OnOff,
                        menu: None,
                        min: 0,
                        max: 1,
                        default: 1,
                        midi: no_midi,
                        track: None,
                    },
                };
                let default = match k {
                    OH_CHOKE => 1,
                    _ => base.default.clamp(min, max),
                };
                ParamInfo {
                    index: oh_index(k),
                    enum_name: leak(format!("VDRUM_OH_{}", id.to_uppercase())),
                    id,
                    voice: Voice::OpenHat,
                    min,
                    max,
                    default,
                    midi: no_midi,
                    ..base
                }
            })
            .collect()
    })
}

/// Number of firmware modulation targets.
pub const FW_NUM_MOD_TARGETS: usize = MOD_TARGETS.len();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table() {
        assert_eq!(FW_NUM_MOD_TARGETS, OH_TARGET_BASE as usize);
        assert_eq!(OH_MAX_TARGET, 254);
        let p = oh_params();
        assert_eq!(p.len(), OH_NUM_PARAMS);
        for (k, info) in p.iter().enumerate() {
            assert_eq!(info.index, oh_index(k));
            assert!(info.min < info.max, "{}", info.id);
            assert!(info.min <= info.default && info.default <= info.max, "{}", info.id);
            assert_eq!(oh_k(info.index), Some(k));
        }
        assert_eq!(p[1].name, "Open Hi-hat Oscillator Coarse");
        assert_eq!(p[OH_VELOD].default, PARAMS[par::VELOD6_OPEN as usize].default);
        for (i, (k, ch)) in OH_MOD_TARGETS.iter().enumerate() {
            // the open hat target stands for the twin of the hi-hat target's parameter
            let hat_param = MOD_TARGETS[*ch as usize].1;
            assert_eq!(OH_SPECS[*k as usize].1, Some(hat_param), "target {i}");
            assert_eq!(oh_target_param(OH_TARGET_BASE + i as u8), Some(oh_index(*k as usize)));
        }
    }

    #[test]
    fn import_from_init_kit() {
        let k = crate::kit::Kit::init();
        let v = oh_from_hat(&k.values);
        assert_eq!(v[OH_VELOD], k.values[par::VELOD6_OPEN as usize]);
        assert_eq!(v[1], k.values[par::COARSE6 as usize]);
        assert_eq!(v[OH_CHOKE], 1);
    }
}
