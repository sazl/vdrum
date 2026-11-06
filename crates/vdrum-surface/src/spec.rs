// SPDX-License-Identifier: GPL-3.0-only
//! The surface spec: the mode layouts and overlays as data, built by `tools/surface_spec` from the
//! Python sources in `surface/` into `crates/vdrum-surface/spec/surface.json`.
//!
//! JSON format (version 2):
//!
//! ```json
//! { "format": "vdrum-surface-spec", "version": 2,
//!   "modes": [ { "mode": "voice:0", "slot": 1, "name": "Drum 1", "layout": "drum-1" }, ... ],
//!   "layouts": { "drum-1": [ { "control": "T1", "label": "Osc wave", "section": "osc",
//!                              "do": "param", "arg": "osc_wave_drum1" }, ... ] },
//!   "overlays": { "steps": [ ... ], "step-held": [ ... ] },
//!   "navigation": [ { "button": "solo_arm", "when": ["shift"], "press": [{"do": "toggle_sticky"}],
//!                     "release": [] }, ... ],
//!   "leds": [ { "button": "solo_arm", "rules": [ { "when": ["sticky"], "colour": "mode_active" }, ... ] } ],
//!   "slot_leds": { "when": ["shift"] },
//!   "screen": [ { "when": ["mode:sequencer"], "lines": ["{mode_slot} {mode_name}", ...] }, ... ],
//!   "palette": { "sections": [["osc", [244, 162, 89]], ...], "tracks": [[255, 120, 90], ...],
//!                "named": [["mode_active", [255, 80, 0]], ...] } }
//! ```
//!
//! * `control`: `K<row><col>`, `F<col>`, `T<col>`, `B<col>` (1-based, as on the panel).
//! * `section`: a [`Section`] name in snake case; the track-coloured sections take the track:
//!   `seq_track:0`, `perf:3`, `pad:6`.
//! * `do` / `arg`: the behaviour ([`Binding`]) and its argument: a parameter id for `param`, a number
//!   for per-track or per-slot behaviours, a name for `step_field` and `setting`, nothing otherwise.
//!   [`BEHAVIOURS`] lists them; `docs/lcxl3-daw-mode/surface-spec.md` explains how to add one.
//! * Cells are applied in order; a cell for a control that is already set replaces it in place.
//! * `overlays.steps` is laid over any mode while the sequencer view is on; `overlays.step-held` is
//!   laid over the sequencer's layout while a step is held.
//! * `navigation`, `leds`, `slot_leds`, `screen`, `palette`: see [`crate::rules`].

use crate::layout::{Binding, Cell, Layout, Mode, Rgb, Section, Setting, StepField};
use crate::rules::{RawRules, Rules};
use lcxl3::{Button, Control};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::OnceLock;

/// The spec format version this loader reads.
pub const VERSION: u32 = 2;

/// The spec built into the plugin.
pub const BUILTIN_JSON: &str = include_str!("../spec/surface.json");

#[derive(Deserialize)]
struct RawSpec {
    format: String,
    version: u32,
    modes: Vec<RawMode>,
    layouts: HashMap<String, Vec<RawCell>>,
    overlays: HashMap<String, Vec<RawCell>>,
    #[serde(flatten)]
    rules: RawRules,
}

#[derive(Deserialize)]
struct RawMode {
    mode: String,
    slot: u8,
    name: String,
    layout: String,
}

#[derive(Deserialize)]
struct RawCell {
    control: String,
    label: String,
    section: String,
    #[serde(rename = "do")]
    behaviour: String,
    #[serde(default)]
    arg: serde_json::Value,
}

/// A mode of the spec.
#[derive(Clone, Debug)]
pub struct ModeSpec {
    pub mode: Mode,
    pub slot: u8,
    pub name: String,
    pub layout: Layout,
}

/// A loaded and checked spec.
#[derive(Clone, Debug)]
pub struct Spec {
    pub modes: Vec<ModeSpec>,
    /// The sequencer view's step buttons.
    pub steps: Vec<(Control, Cell)>,
    /// The sequencer's held-step cells.
    pub step_held: Vec<(Control, Cell)>,
    /// Navigation, side-button LEDs, the stationary screen and the palette.
    pub rules: Rules,
}

impl Spec {
    /// Parse and check a spec. Every error names the layout and control it was found in.
    pub fn parse(json: &str) -> Result<Spec, String> {
        let raw: RawSpec = serde_json::from_str(json).map_err(|e| format!("spec: {e}"))?;
        if raw.format != "vdrum-surface-spec" || raw.version != VERSION {
            return Err(format!("spec: unsupported format {} version {}", raw.format, raw.version));
        }
        let cells = |name: &str, list: &[RawCell]| -> Result<Vec<(Control, Cell)>, String> {
            list.iter().map(|c| cell(c).map_err(|e| format!("{name} {}: {e}", c.control))).collect()
        };
        let overlay = |name: &str| -> Result<Vec<(Control, Cell)>, String> {
            cells(name, raw.overlays.get(name).ok_or_else(|| format!("spec: overlay {name} missing"))?)
        };
        let mut modes = Vec::new();
        for m in &raw.modes {
            let mode = mode_from_str(&m.mode).ok_or_else(|| format!("spec: unknown mode {}", m.mode))?;
            let list = raw.layouts.get(&m.layout).ok_or_else(|| format!("spec: mode {}: layout {} missing", m.mode, m.layout))?;
            let mut layout = Layout::default();
            for (c, cell) in cells(&m.layout, list)? {
                layout.put(c, cell);
            }
            modes.push(ModeSpec { mode, slot: m.slot, name: m.name.clone(), layout });
        }
        let rules = raw.rules.parse()?;
        Ok(Spec { modes, steps: overlay("steps")?, step_held: overlay("step-held")?, rules })
    }

    /// The mode in slot `slot` (Shift + button N: top row 1..=8, bottom row 9..=16).
    pub fn mode_at(&self, slot: u8) -> Option<Mode> {
        self.modes.iter().find(|m| m.slot == slot).map(|m| m.mode)
    }

    /// The slot and name of a mode; `(0, "")` for a mode the spec does not place.
    pub fn mode_info(&self, mode: Mode) -> (u8, &str) {
        self.modes.iter().find(|m| m.mode == mode).map(|m| (m.slot, m.name.as_str())).unwrap_or((0, ""))
    }

    /// A section's colour.
    pub fn colour(&self, s: Section) -> Rgb {
        match s {
            Section::SeqTrack(t) | Section::Perf(t) | Section::Pad(t) => self.rules.palette.tracks[(t as usize).min(6)],
            _ => self.rules.palette.section(&section_name(s)).unwrap_or((0, 0, 0)),
        }
    }

    /// A named colour of the palette (black if the spec has none of that name).
    pub fn named(&self, name: &str) -> Rgb {
        self.rules.palette.named(name).unwrap_or((0, 0, 0))
    }

    /// The layout of `mode`; `held` lays the held-step cells over the sequencer's layout.
    pub fn layout(&self, mode: Mode, held: bool) -> Layout {
        let mut l = self.modes.iter().find(|m| m.mode == mode).map(|m| m.layout.clone()).unwrap_or_default();
        if held && mode == Mode::Sequencer {
            for (c, cell) in &self.step_held {
                l.put(*c, cell.clone());
            }
        }
        l
    }

    /// `layout` with the sequencer view's step buttons laid over it.
    pub fn with_steps(&self, mut l: Layout) -> Layout {
        for (c, cell) in &self.steps {
            l.put(*c, cell.clone());
        }
        l
    }
}

/// The built-in spec. Panics if it does not load (the tests load it).
pub fn builtin() -> &'static Spec {
    builtin_arc()
}

/// The built-in spec, shared.
pub fn builtin_arc() -> &'static std::sync::Arc<Spec> {
    static SPEC: OnceLock<std::sync::Arc<Spec>> = OnceLock::new();
    SPEC.get_or_init(|| std::sync::Arc::new(Spec::parse(BUILTIN_JSON).unwrap_or_else(|e| panic!("built-in surface spec: {e}"))))
}

pub fn mode_from_str(s: &str) -> Option<Mode> {
    Some(match s {
        "sequencer" => Mode::Sequencer,
        "mix_fx" => Mode::MixFx,
        "perform" => Mode::Perform,
        "lfo_mod" => Mode::LfoMod,
        "morph_macro" => Mode::MorphMacro,
        "global" => Mode::Global,
        _ => {
            let t: u8 = s.strip_prefix("voice:")?.parse().ok()?;
            if t > 6 {
                return None;
            }
            Mode::Voice(t)
        }
    })
}

/// The spec name of a mode (inverse of the parser's mode names).
pub fn mode_name(m: Mode) -> String {
    match m {
        Mode::Voice(t) => format!("voice:{t}"),
        Mode::Sequencer => "sequencer".into(),
        Mode::MixFx => "mix_fx".into(),
        Mode::Perform => "perform".into(),
        Mode::LfoMod => "lfo_mod".into(),
        Mode::MorphMacro => "morph_macro".into(),
        Mode::Global => "global".into(),
    }
}

/// A control from its panel key (`K11`, `F8`, `T1`, `B16` is not valid: bottom buttons are `B1..B8`).
pub fn control_from_key(k: &str) -> Option<Control> {
    let (kind, rest) = k.split_at(1);
    let n: Vec<u8> = rest.chars().map(|c| c.to_digit(10).map(|d| d as u8)).collect::<Option<_>>()?;
    let col_ok = |c: u8| (1..=8).contains(&c);
    Some(match (kind, n.as_slice()) {
        ("K", [r, c]) if (1..=3).contains(r) && col_ok(*c) => Control::Encoder { row: r - 1, col: c - 1 },
        ("F", [c]) if col_ok(*c) => Control::Fader(c - 1),
        ("T", [c]) if col_ok(*c) => Control::Button(Button::Top(c - 1)),
        ("B", [c]) if col_ok(*c) => Control::Button(Button::Bottom(c - 1)),
        _ => return None,
    })
}

/// The panel key of a control in a layout.
pub fn control_key(c: Control) -> String {
    match c {
        Control::Encoder { row, col } => format!("K{}{}", row + 1, col + 1),
        Control::Fader(c) => format!("F{}", c + 1),
        Control::Button(Button::Top(c)) => format!("T{}", c + 1),
        Control::Button(Button::Bottom(c)) => format!("B{}", c + 1),
        Control::Button(b) => format!("{b:?}"),
    }
}

fn section_from_str(s: &str) -> Option<Section> {
    use Section::*;
    if let Some((name, t)) = s.split_once(':') {
        let t: u8 = t.parse().ok().filter(|t| *t < 7)?;
        return Some(match name {
            "seq_track" => SeqTrack(t),
            "perf" => Perf(t),
            "pad" => Pad(t),
            _ => return None,
        });
    }
    Some(match s {
        "osc" => Osc,
        "amp" => Amp,
        "mod" => Mod,
        "fm" => Fm,
        "trans" => Trans,
        "filter" => Filter,
        "lfo" => Lfo,
        "velo" => Velo,
        "mix" => Mix,
        "fx" => Fx,
        "moment" => Moment,
        "seq" => Seq,
        "step" => Step,
        "macro" => Macro,
        "global" => Global,
        "device" => Device,
        "edit" => Edit,
        "nav" => Nav,
        "spare" => Spare,
        _ => return None,
    })
}

/// The spec name of a section.
pub fn section_name(s: Section) -> String {
    use Section::*;
    match s {
        SeqTrack(t) => format!("seq_track:{t}"),
        Perf(t) => format!("perf:{t}"),
        Pad(t) => format!("pad:{t}"),
        _ => format!("{s:?}").to_lowercase(),
    }
}

/// The argument a behaviour takes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Arg {
    None,
    /// A parameter id (`lxr_params::by_id`).
    Param,
    /// A track 0..=6.
    Track,
    /// A number below the given bound.
    Index(u8),
    /// A [`StepField`] name.
    StepField,
    /// A [`Setting`] name.
    Setting,
}

/// Every behaviour a cell can have: its spec name and its argument. Keep in step with
/// [`binding_from_spec`], [`binding_to_spec`] and `tools/surface_spec/surface_spec/behaviours.py`.
pub const BEHAVIOURS: &[(&str, Arg)] = &[
    ("param", Arg::Param),
    ("voice_fader", Arg::Track),
    ("pad", Arg::Track),
    ("accent", Arg::None),
    ("copy_voice", Arg::None),
    ("paste_voice", Arg::None),
    ("to_fx", Arg::Track),
    ("fx_on_off", Arg::None),
    ("fx_sub_type", Arg::None),
    ("moment", Arg::Index(8)),
    ("moment_depth", Arg::None),
    ("moment_rate", Arg::None),
    ("seq_fills", Arg::Track),
    ("seq_rotate", Arg::Track),
    ("seq_length", Arg::Track),
    ("seq_density", Arg::Track),
    ("seq_shuffle", Arg::None),
    ("seq_next_pattern", Arg::None),
    ("seq_entry_velocity", Arg::None),
    ("seq_density_all", Arg::None),
    ("seq_step", Arg::Index(16)),
    ("step_field", Arg::StepField),
    ("lock_target", Arg::Index(8)),
    ("trigger", Arg::Track),
    ("mute", Arg::Track),
    ("mute_all", Arg::None),
    ("lfo_rate_scale", Arg::None),
    ("vel_scale", Arg::None),
    ("lfo_offset_all", Arg::None),
    ("lfo_depth_scale", Arg::None),
    ("lfo_sync_host", Arg::None),
    ("lfo_reset_phases", Arg::None),
    ("macro", Arg::Index(8)),
    ("macro_depth", Arg::Index(8)),
    ("snapshot", Arg::Index(8)),
    ("learn", Arg::Index(8)),
    ("track_morph", Arg::Track),
    ("track_target", Arg::Track),
    ("morph_all", Arg::None),
    ("target_all", Arg::None),
    ("sum_aux", Arg::None),
    ("hw_clock", Arg::None),
    ("init_kit", Arg::None),
    ("save_kit", Arg::None),
    ("compare", Arg::None),
    ("undo", Arg::None),
    ("redo", Arg::None),
    ("panic", Arg::None),
    ("setting", Arg::Setting),
    ("spare", Arg::None),
];

const STEP_FIELDS: [(&str, StepField); 8] = [
    ("velocity", StepField::Velocity),
    ("probability", StepField::Probability),
    ("note", StepField::Note),
    ("flam", StepField::Flam),
    ("shift", StepField::Shift),
    ("lock1", StepField::Lock1),
    ("lock2", StepField::Lock2),
    ("clear_locks", StepField::ClearLocks),
];

const SETTINGS: [(&str, Setting); 8] = [
    ("led_brightness", Setting::LedBrightness),
    ("screen_brightness", Setting::ScreenBrightness),
    ("popup_time", Setting::PopupTime),
    ("encoder_curve", Setting::EncoderCurve),
    ("pad_velocity", Setting::PadVelocity),
    ("list_speed", Setting::ListSpeed),
    ("fader_mode", Setting::FaderMode),
    ("clock", Setting::Clock),
];

fn cell(c: &RawCell) -> Result<(Control, Cell), String> {
    let control = control_from_key(&c.control).ok_or("unknown control")?;
    let section = section_from_str(&c.section).ok_or_else(|| format!("unknown section {}", c.section))?;
    let binding = binding_from_spec(&c.behaviour, &c.arg)?;
    Ok((control, Cell { label: c.label.clone(), section, binding }))
}

/// A [`Binding`] from its spec name and argument.
pub fn binding_from_spec(name: &str, arg: &serde_json::Value) -> Result<Binding, String> {
    use Binding as B;
    let kind = BEHAVIOURS.iter().find(|(n, _)| *n == name).map(|(_, a)| *a).ok_or_else(|| format!("unknown behaviour {name}"))?;
    let n = || -> Result<u8, String> {
        let bound = match kind {
            Arg::Track => 7,
            Arg::Index(b) => b,
            _ => unreachable!(),
        };
        arg.as_u64().filter(|v| *v < bound as u64).map(|v| v as u8).ok_or_else(|| format!("{name}: argument must be 0..{bound}, got {arg}"))
    };
    let s = || arg.as_str().ok_or_else(|| format!("{name}: argument must be a string, got {arg}"));
    match kind {
        Arg::None if !arg.is_null() => return Err(format!("{name} takes no argument, got {arg}")),
        _ => {}
    }
    Ok(match name {
        "param" => B::Param(lxr_params::by_id(s()?).ok_or_else(|| format!("unknown parameter id {arg}"))?.index),
        "voice_fader" => B::VoiceFader(n()?),
        "pad" => B::Pad(n()?),
        "accent" => B::Accent,
        "copy_voice" => B::CopyVoice,
        "paste_voice" => B::PasteVoice,
        "to_fx" => B::ToFx(n()?),
        "fx_on_off" => B::FxOnOff,
        "fx_sub_type" => B::FxSubType,
        "moment" => B::Moment(n()?),
        "moment_depth" => B::MomentDepth,
        "moment_rate" => B::MomentRate,
        "seq_fills" => B::SeqFills(n()?),
        "seq_rotate" => B::SeqRotate(n()?),
        "seq_length" => B::SeqLength(n()?),
        "seq_density" => B::SeqDensity(n()?),
        "seq_shuffle" => B::SeqShuffle,
        "seq_next_pattern" => B::SeqNextPattern,
        "seq_entry_velocity" => B::SeqEntryVelocity,
        "seq_density_all" => B::SeqDensityAll,
        "seq_step" => B::SeqStep(n()?),
        "step_field" => {
            let v = s()?;
            B::StepField(STEP_FIELDS.iter().find(|(k, _)| *k == v).ok_or_else(|| format!("unknown step field {v}"))?.1)
        }
        "lock_target" => B::LockTarget(n()?),
        "trigger" => B::Trigger(n()?),
        "mute" => B::Mute(n()?),
        "mute_all" => B::MuteAll,
        "lfo_rate_scale" => B::LfoRateScale,
        "vel_scale" => B::VelScale,
        "lfo_offset_all" => B::LfoOffsetAll,
        "lfo_depth_scale" => B::LfoDepthScale,
        "lfo_sync_host" => B::LfoSyncHost,
        "lfo_reset_phases" => B::LfoResetPhases,
        "macro" => B::Macro(n()?),
        "macro_depth" => B::MacroDepth(n()?),
        "snapshot" => B::Snapshot(n()?),
        "learn" => B::Learn(n()?),
        "track_morph" => B::TrackMorph(n()?),
        "track_target" => B::TrackTarget(n()?),
        "morph_all" => B::MorphAll,
        "target_all" => B::TargetAll,
        "sum_aux" => B::SumAux,
        "hw_clock" => B::HwClock,
        "init_kit" => B::InitKit,
        "save_kit" => B::SaveKit,
        "compare" => B::Compare,
        "undo" => B::Undo,
        "redo" => B::Redo,
        "panic" => B::Panic,
        "setting" => {
            let v = s()?;
            B::Setting(SETTINGS.iter().find(|(k, _)| *k == v).ok_or_else(|| format!("unknown setting {v}"))?.1)
        }
        "spare" => B::Spare,
        _ => return Err(format!("behaviour {name} is listed in BEHAVIOURS but not handled")),
    })
}

/// The spec name and argument of a [`Binding`] (inverse of [`binding_from_spec`]).
pub fn binding_to_spec(b: Binding) -> (&'static str, serde_json::Value) {
    use serde_json::Value as V;
    use Binding as B;
    let n = |v: u8| V::from(v);
    match b {
        B::Param(i) => ("param", V::from(lxr_params::info(i).map(|p| p.id).unwrap_or("?"))),
        B::VoiceFader(t) => ("voice_fader", n(t)),
        B::Pad(t) => ("pad", n(t)),
        B::Accent => ("accent", V::Null),
        B::CopyVoice => ("copy_voice", V::Null),
        B::PasteVoice => ("paste_voice", V::Null),
        B::ToFx(t) => ("to_fx", n(t)),
        B::FxOnOff => ("fx_on_off", V::Null),
        B::FxSubType => ("fx_sub_type", V::Null),
        B::Moment(k) => ("moment", n(k)),
        B::MomentDepth => ("moment_depth", V::Null),
        B::MomentRate => ("moment_rate", V::Null),
        B::SeqFills(t) => ("seq_fills", n(t)),
        B::SeqRotate(t) => ("seq_rotate", n(t)),
        B::SeqLength(t) => ("seq_length", n(t)),
        B::SeqDensity(t) => ("seq_density", n(t)),
        B::SeqShuffle => ("seq_shuffle", V::Null),
        B::SeqNextPattern => ("seq_next_pattern", V::Null),
        B::SeqEntryVelocity => ("seq_entry_velocity", V::Null),
        B::SeqDensityAll => ("seq_density_all", V::Null),
        B::SeqStep(k) => ("seq_step", n(k)),
        B::StepField(f) => ("step_field", V::from(STEP_FIELDS.iter().find(|(_, v)| *v == f).map(|(k, _)| *k).unwrap_or("?"))),
        B::LockTarget(k) => ("lock_target", n(k)),
        B::Trigger(t) => ("trigger", n(t)),
        B::Mute(t) => ("mute", n(t)),
        B::MuteAll => ("mute_all", V::Null),
        B::LfoRateScale => ("lfo_rate_scale", V::Null),
        B::VelScale => ("vel_scale", V::Null),
        B::LfoOffsetAll => ("lfo_offset_all", V::Null),
        B::LfoDepthScale => ("lfo_depth_scale", V::Null),
        B::LfoSyncHost => ("lfo_sync_host", V::Null),
        B::LfoResetPhases => ("lfo_reset_phases", V::Null),
        B::Macro(k) => ("macro", n(k)),
        B::MacroDepth(k) => ("macro_depth", n(k)),
        B::Snapshot(k) => ("snapshot", n(k)),
        B::Learn(k) => ("learn", n(k)),
        B::TrackMorph(t) => ("track_morph", n(t)),
        B::TrackTarget(t) => ("track_target", n(t)),
        B::MorphAll => ("morph_all", V::Null),
        B::TargetAll => ("target_all", V::Null),
        B::SumAux => ("sum_aux", V::Null),
        B::HwClock => ("hw_clock", V::Null),
        B::InitKit => ("init_kit", V::Null),
        B::SaveKit => ("save_kit", V::Null),
        B::Compare => ("compare", V::Null),
        B::Undo => ("undo", V::Null),
        B::Redo => ("redo", V::Null),
        B::Panic => ("panic", V::Null),
        B::Setting(s) => ("setting", V::from(SETTINGS.iter().find(|(_, v)| *v == s).map(|(k, _)| *k).unwrap_or("?"))),
        B::Spare => ("spare", V::Null),
    }
}

/// Every parameter id a `param` cell can name, sorted, as a JSON list (one id per line).
pub fn param_ids_json() -> String {
    let mut ids: Vec<&str> = lxr_params::PARAMS
        .iter()
        .chain(lxr_params::GLOBAL_PARAMS.iter())
        .chain(lxr_params::oh_params().iter())
        .map(|p| p.id)
        .collect();
    ids.sort_unstable();
    ids.dedup();
    let body: Vec<String> = ids.iter().map(|i| format!("  \"{i}\"")).collect();
    format!("[\n{}\n]", body.join(",\n"))
}

/// A layout's cells as spec JSON values (for tests and tools).
pub fn cells_to_json(cells: &[(Control, Cell)]) -> serde_json::Value {
    serde_json::Value::Array(
        cells
            .iter()
            .map(|(c, cell)| {
                let (d, arg) = binding_to_spec(cell.binding);
                let mut o = serde_json::Map::new();
                o.insert("control".into(), control_key(*c).into());
                o.insert("label".into(), cell.label.clone().into());
                o.insert("section".into(), section_name(cell.section).into());
                o.insert("do".into(), d.into());
                if !arg.is_null() {
                    o.insert("arg".into(), arg);
                }
                serde_json::Value::Object(o)
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every mode the plugin knows has one slot (1 to 16, no two alike) and a name, and assigns all
    /// 48 controls.
    #[test]
    fn builtin_spec_has_every_mode() {
        let spec = builtin();
        assert_eq!(spec.modes.len(), Mode::ALL.len());
        let mut slots = std::collections::BTreeSet::new();
        for m in Mode::ALL {
            let ms = spec.modes.iter().find(|s| s.mode == m).unwrap_or_else(|| panic!("{m:?} missing"));
            assert!((1..=16).contains(&ms.slot) && slots.insert(ms.slot) && !ms.name.is_empty(), "{m:?}");
            assert_eq!(ms.layout.iter().count(), 48, "{m:?}");
        }
        assert_eq!(spec.steps.len(), 16);
    }

    /// `tools/surface_spec/surface_spec/param_ids.json` (used by the Python checks) is current.
    #[test]
    fn param_id_list_is_current() {
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tools/surface_spec/surface_spec/param_ids.json");
        let file = std::fs::read_to_string(path).expect("param_ids.json");
        assert_eq!(file.trim_end(), param_ids_json(), "run: cargo run -q -p vdrum-surface --example param_ids > tools/surface_spec/surface_spec/param_ids.json");
    }

    #[test]
    fn errors_name_the_layout_and_control() {
        let bad = BUILTIN_JSON.replacen("\"arg\": \"wave1_hh\"", "\"arg\": \"no_such_param\"", 1);
        let e = Spec::parse(&bad).unwrap_err();
        assert!(e.contains("closed-hi-hat T1") && e.contains("no_such_param"), "{e}");
        let bad = BUILTIN_JSON.replacen("\"do\": \"seq_fills\", \"arg\": 0", "\"do\": \"seq_fills\", \"arg\": 9", 1);
        assert!(Spec::parse(&bad).unwrap_err().contains("sequencer K11"));
    }

    #[test]
    fn rule_errors_name_the_rule() {
        let bad = BUILTIN_JSON.replacen("\"when\": [\"shift\"]", "\"when\": [\"shfit\"]", 1);
        let e = Spec::parse(&bad).unwrap_err();
        assert!(e.contains("navigation rule 2 (solo_arm)") && e.contains("shfit"), "{e}");
        let bad = BUILTIN_JSON.replacen("{mode_slot}", "{mode_slott}", 1);
        assert!(Spec::parse(&bad).unwrap_err().contains("unknown variable {mode_slott}"));
        let bad = BUILTIN_JSON.replacen("\"colour\": \"mode_active\"", "\"colour\": \"orange\"", 1);
        assert!(Spec::parse(&bad).unwrap_err().contains("leds solo_arm: unknown colour orange"));
        let bad = BUILTIN_JSON.replacen("\"version\": 2", "\"version\": 1", 1);
        assert!(Spec::parse(&bad).unwrap_err().contains("version 1"));
    }

    #[test]
    fn modes_slots_and_colours_come_from_the_spec() {
        let s = builtin();
        assert_eq!(s.mode_at(8), Some(Mode::Sequencer));
        assert_eq!(s.mode_at(13), None);
        assert_eq!(s.mode_info(Mode::Global), (16, "Global"));
        assert_eq!(s.colour(Section::Pad(3)), s.rules.palette.tracks[3]);
        assert_eq!(Section::Osc.colour(), s.colour(Section::Osc));
    }

    #[test]
    fn behaviours_round_trip() {
        for m in Mode::ALL {
            for (_, cell) in Layout::of(m, true).with_steps().iter() {
                let (name, arg) = binding_to_spec(cell.binding);
                assert_eq!(binding_from_spec(name, &arg), Ok(cell.binding));
            }
        }
    }
}
