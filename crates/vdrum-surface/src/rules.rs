// SPDX-License-Identifier: GPL-3.0-only
//! The rule parts of the surface spec: navigation (what the side buttons and the 16 buttons do in
//! each state), the side-button LEDs, the stationary screen and the colour palette.
//!
//! The rules are interpreted by [`crate::Surface`]; this module only holds and parses them. The
//! catalogues of guards ([`PREDICATES`]), navigation steps ([`NAV_STEPS`]), screen variables
//! ([`SCREEN_VARS`]) and button names ([`BUTTONS`]) are mirrored in
//! `tools/surface_spec/surface_spec/rules.py`.

use crate::layout::{Mode, Rgb};
use lcxl3::Button;
use serde::Deserialize;
use serde_json::Value;

/// The button a rule is for: one button, or a whole row of the 16 buttons (the action receives
/// the column).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NavButton {
    One(Button),
    Top,
    Bottom,
}

impl NavButton {
    pub fn matches(self, b: Button) -> bool {
        match self {
            NavButton::One(x) => x == b,
            NavButton::Top => matches!(b, Button::Top(_)),
            NavButton::Bottom => matches!(b, Button::Bottom(_)),
        }
    }
}

/// Spec names of the buttons (`top` and `bottom` stand for a whole row).
pub const BUTTONS: &[(&str, Button)] = &[
    ("shift", Button::Shift),
    ("solo_arm", Button::SoloArm),
    ("mute_select", Button::MuteSelect),
    ("logo", Button::Logo),
    ("page_up", Button::PageUp),
    ("page_down", Button::PageDown),
    ("track_left", Button::TrackLeft),
    ("track_right", Button::TrackRight),
    ("record", Button::Record),
    ("play", Button::Play),
];

pub fn button_from_str(s: &str) -> Option<Button> {
    BUTTONS.iter().find(|(n, _)| *n == s).map(|(_, b)| *b)
}

pub fn button_name(b: Button) -> String {
    match b {
        Button::Top(c) => format!("top:{c}"),
        Button::Bottom(c) => format!("bottom:{c}"),
        _ => BUTTONS.iter().find(|(_, x)| *x == b).map(|(n, _)| n.to_string()).unwrap_or_default(),
    }
}

/// A state predicate a rule can test.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pred {
    /// Shift is held.
    Shift,
    /// The sequencer view is on (Solo/Arm, or always in the sequencer mode).
    SeqView,
    /// The sequencer view was switched on with Solo/Arm (not implied by the mode).
    ViewToggle,
    /// The sequencer view stays on across mode changes.
    Sticky,
    /// The current mode is the given one.
    Mode(Mode),
    /// The current mode is a voice mode.
    VoiceMode,
    /// The button is held.
    Held(Button),
    /// The selected sequencer track is the first (D1) / the last (OH).
    SeqTrackFirst,
    SeqTrackLast,
    /// The bar shown on the step buttons is the first / the last.
    SeqBarFirst,
    SeqBarLast,
    /// The sequencer is running / silenced (host clock).
    SeqRunning,
    SeqSilent,
    /// The sequencer follows the host clock (Global mode setting).
    HostClock,
    /// A kit shown by Page up/down waits for Play.
    Browsing,
    /// A step button is held.
    StepHeld,
}

/// Predicate names. `mode:<key>` and `held:<button>` take an argument.
pub const PREDICATES: &[(&str, Pred)] = &[
    ("shift", Pred::Shift),
    ("seq_view", Pred::SeqView),
    ("view_toggle", Pred::ViewToggle),
    ("sticky", Pred::Sticky),
    ("voice_mode", Pred::VoiceMode),
    ("seq_track_first", Pred::SeqTrackFirst),
    ("seq_track_last", Pred::SeqTrackLast),
    ("seq_bar_first", Pred::SeqBarFirst),
    ("seq_bar_last", Pred::SeqBarLast),
    ("seq_running", Pred::SeqRunning),
    ("seq_silent", Pred::SeqSilent),
    ("host_clock", Pred::HostClock),
    ("browsing", Pred::Browsing),
    ("step_held", Pred::StepHeld),
];

/// A predicate, possibly negated (`!name`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Guard {
    pub pred: Pred,
    pub negate: bool,
}

fn guard(s: &str) -> Result<Guard, String> {
    let (negate, s) = match s.strip_prefix('!') {
        Some(rest) => (true, rest),
        None => (false, s),
    };
    let pred = if let Some(m) = s.strip_prefix("mode:") {
        Pred::Mode(crate::spec::mode_from_str(m).ok_or_else(|| format!("unknown mode {m}"))?)
    } else if let Some(b) = s.strip_prefix("held:") {
        Pred::Held(button_from_str(b).ok_or_else(|| format!("unknown button {b}"))?)
    } else {
        PREDICATES.iter().find(|(n, _)| *n == s).map(|(_, p)| *p).ok_or_else(|| format!("unknown predicate {s}"))?
    };
    Ok(Guard { pred, negate })
}

pub fn guard_name(g: &Guard) -> String {
    let base = match g.pred {
        Pred::Mode(m) => format!("mode:{}", crate::spec::mode_name(m)),
        Pred::Held(b) => format!("held:{}", button_name(b)),
        p => PREDICATES.iter().find(|(_, x)| *x == p).map(|(n, _)| n.to_string()).unwrap_or_default(),
    };
    if g.negate {
        format!("!{base}")
    } else {
        base
    }
}

/// One step of a navigation rule's press or release.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NavStep {
    /// Shift follows the button (held on press, released on release).
    Shift,
    /// Toggle the sticky sequencer view (sticky on also switches the view on); shows the result.
    ToggleSticky,
    /// Select the mode in slot `base + column` (Shift + button N); an empty slot says so.
    ModeSlot(u8),
    /// Step button `base + column` of the sequencer view (press and release).
    SeqButton(u8),
    /// Clear the selected sequencer track's pattern.
    ClearSeqTrack,
    /// Select the previous (-1) or next (+1) sequencer track.
    SeqTrackStep(i8),
    /// Return to the previous mode.
    PreviousMode,
    /// Toggle the sequencer view; shows the result and the selected track.
    ToggleSeqView,
    /// Show a popup: title, name, value.
    Message([String; 3]),
    /// Start a layer: the release's `mode_unless_layer_used` acts only if no layer control was used.
    LayerBegin,
    /// Switch to a mode.
    Mode(Mode),
    /// Switch to a mode unless the layer begun on press was used.
    ModeUnlessLayerUsed(Mode),
    /// Show the previous (-1) or next (+1) kit; Play loads it.
    KitBrowse(i8),
    /// Show the previous (-1) or next (+1) bar on the step buttons.
    SeqBarStep(i8),
    /// Play: load a browsed kit, start/stop/silence the sequencer, or trigger the mode's voice.
    Play,
    /// The control's cell in the current layout (press / release behaviour).
    Cell,
}

/// Navigation step names and their argument: `None`, `"int"`, `"dir"` (-1 or 1), `"mode"`,
/// `"text3"`.
pub const NAV_STEPS: &[(&str, &str)] = &[
    ("shift", "none"),
    ("toggle_sticky", "none"),
    ("mode_slot", "int"),
    ("seq_button", "int"),
    ("clear_seq_track", "none"),
    ("seq_track_step", "dir"),
    ("previous_mode", "none"),
    ("toggle_seq_view", "none"),
    ("message", "text3"),
    ("layer_begin", "none"),
    ("mode", "mode"),
    ("mode_unless_layer_used", "mode"),
    ("kit_browse", "dir"),
    ("seq_bar_step", "dir"),
    ("play", "none"),
    ("cell", "none"),
];

fn nav_step(name: &str, arg: &Value) -> Result<NavStep, String> {
    let kind = NAV_STEPS.iter().find(|(n, _)| *n == name).map(|(_, k)| *k).ok_or_else(|| format!("unknown step {name}"))?;
    let int = || arg.as_u64().filter(|v| *v <= 16).map(|v| v as u8).ok_or_else(|| format!("{name}: argument must be 0..=16, got {arg}"));
    let dir = || match arg.as_i64() {
        Some(-1) => Ok(-1i8),
        Some(1) => Ok(1),
        _ => Err(format!("{name}: argument must be -1 or 1, got {arg}")),
    };
    let mode = || {
        arg.as_str().and_then(crate::spec::mode_from_str).ok_or_else(|| format!("{name}: argument must be a mode key, got {arg}"))
    };
    if kind == "none" && !arg.is_null() {
        return Err(format!("{name} takes no argument, got {arg}"));
    }
    Ok(match name {
        "shift" => NavStep::Shift,
        "toggle_sticky" => NavStep::ToggleSticky,
        "mode_slot" => NavStep::ModeSlot(int()?),
        "seq_button" => NavStep::SeqButton(int()?),
        "clear_seq_track" => NavStep::ClearSeqTrack,
        "seq_track_step" => NavStep::SeqTrackStep(dir()?),
        "previous_mode" => NavStep::PreviousMode,
        "toggle_seq_view" => NavStep::ToggleSeqView,
        "message" => {
            let v: Vec<String> = arg
                .as_array()
                .filter(|a| a.len() == 3)
                .and_then(|a| a.iter().map(|x| x.as_str().map(String::from)).collect())
                .ok_or_else(|| format!("message: argument must be [title, name, value], got {arg}"))?;
            NavStep::Message([v[0].clone(), v[1].clone(), v[2].clone()])
        }
        "layer_begin" => NavStep::LayerBegin,
        "mode" => NavStep::Mode(mode()?),
        "mode_unless_layer_used" => NavStep::ModeUnlessLayerUsed(mode()?),
        "kit_browse" => NavStep::KitBrowse(dir()?),
        "seq_bar_step" => NavStep::SeqBarStep(dir()?),
        "play" => NavStep::Play,
        "cell" => NavStep::Cell,
        _ => return Err(format!("step {name} is listed in NAV_STEPS but not handled")),
    })
}

/// A navigation rule. The first rule whose button and guards match, and which has an entry for the
/// event (press or release), handles it; `Some(vec![])` handles it by doing nothing, `None` lets
/// the event fall through to the next rule.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NavRule {
    pub button: NavButton,
    pub when: Vec<Guard>,
    pub press: Option<Vec<NavStep>>,
    pub release: Option<Vec<NavStep>>,
    /// A short description, shown in the simulator and the diagram.
    pub note: String,
}

/// An LED colour of a rule: a palette colour, or a colour that depends on the state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LedColour {
    Rgb(Rgb),
    /// The pad colour of the current voice mode's track; off in other modes.
    ModePad,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LedRule {
    pub when: Vec<Guard>,
    pub colour: LedColour,
}

/// A stationary screen template: the first whose guards match is shown.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScreenRule {
    pub when: Vec<Guard>,
    pub lines: [String; 3],
}

/// Variables a screen line can use as `{name}`.
pub const SCREEN_VARS: &[&str] = &[
    "mode_slot",
    "mode_name",
    "kit",
    "last_touched",
    "pattern",
    "bar",
    "next_pattern",
    "silent",
    "seq_track_number",
    "seq_track_name",
];

/// The palette: section colours, the seven track colours and named colours.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Palette {
    pub sections: Vec<(String, Rgb)>,
    pub tracks: [Rgb; 7],
    pub named: Vec<(String, Rgb)>,
}

impl Palette {
    pub fn named(&self, name: &str) -> Option<Rgb> {
        self.named.iter().find(|(n, _)| n == name).map(|(_, c)| *c)
    }
    pub fn section(&self, name: &str) -> Option<Rgb> {
        self.sections.iter().find(|(n, _)| n == name).map(|(_, c)| *c)
    }
}

/// Every rule part of a spec.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rules {
    pub navigation: Vec<NavRule>,
    /// Side-button LEDs, in the order they are sent.
    pub leds: Vec<(Button, Vec<LedRule>)>,
    /// While these guards hold, the 16 buttons show the mode slots instead of their cells.
    pub slot_leds: Vec<Guard>,
    pub screen: Vec<ScreenRule>,
    pub palette: Palette,
}

#[derive(Deserialize)]
pub(crate) struct RawRules {
    navigation: Vec<RawNav>,
    leds: Vec<RawLed>,
    slot_leds: RawWhen,
    screen: Vec<RawScreen>,
    palette: RawPalette,
}

#[derive(Deserialize)]
struct RawWhen {
    #[serde(default)]
    when: Vec<String>,
}

#[derive(Deserialize)]
struct RawNav {
    button: String,
    #[serde(default)]
    when: Vec<String>,
    #[serde(default)]
    press: Option<Vec<RawStep>>,
    #[serde(default)]
    release: Option<Vec<RawStep>>,
    #[serde(default)]
    note: String,
}

#[derive(Deserialize)]
struct RawStep {
    #[serde(rename = "do")]
    step: String,
    #[serde(default)]
    arg: Value,
}

#[derive(Deserialize)]
struct RawLed {
    button: String,
    rules: Vec<RawLedRule>,
}

#[derive(Deserialize)]
struct RawLedRule {
    #[serde(default)]
    when: Vec<String>,
    colour: String,
}

#[derive(Deserialize)]
struct RawScreen {
    #[serde(default)]
    when: Vec<String>,
    lines: [String; 3],
}

#[derive(Deserialize)]
struct RawPalette {
    sections: Vec<(String, [u8; 3])>,
    tracks: [[u8; 3]; 7],
    named: Vec<(String, [u8; 3])>,
}

fn guards(list: &[String]) -> Result<Vec<Guard>, String> {
    list.iter().map(|s| guard(s)).collect()
}

const SECTION_NAMES: &[&str] = &[
    "osc", "amp", "mod", "fm", "trans", "filter", "lfo", "velo", "mix", "fx", "moment", "seq", "step", "macro", "global",
    "device", "edit", "nav", "spare",
];

impl RawRules {
    pub(crate) fn parse(self) -> Result<Rules, String> {
        let rgb = |c: [u8; 3]| (c[0], c[1], c[2]);
        let palette = Palette {
            sections: self.palette.sections.into_iter().map(|(n, c)| (n, rgb(c))).collect(),
            tracks: self.palette.tracks.map(rgb),
            named: self.palette.named.into_iter().map(|(n, c)| (n, rgb(c))).collect(),
        };
        for s in SECTION_NAMES {
            if palette.section(s).is_none() {
                return Err(format!("palette: section {s} has no colour"));
            }
        }
        let mut navigation = Vec::new();
        for (i, r) in self.navigation.iter().enumerate() {
            let at = |e: String| format!("navigation rule {} ({}): {e}", i + 1, r.button);
            let button = match r.button.as_str() {
                "top" => NavButton::Top,
                "bottom" => NavButton::Bottom,
                b => NavButton::One(button_from_str(b).ok_or_else(|| at("unknown button".into()))?),
            };
            let steps = |l: &Option<Vec<RawStep>>| -> Result<Option<Vec<NavStep>>, String> {
                l.as_ref().map(|l| l.iter().map(|s| nav_step(&s.step, &s.arg)).collect()).transpose()
            };
            navigation.push(NavRule {
                button,
                when: guards(&r.when).map_err(at)?,
                press: steps(&r.press).map_err(at)?,
                release: steps(&r.release).map_err(at)?,
                note: r.note.clone(),
            });
        }
        let mut leds = Vec::new();
        for l in &self.leds {
            let at = |e: String| format!("leds {}: {e}", l.button);
            let b = button_from_str(&l.button).filter(|b| *b != Button::Shift).ok_or_else(|| at("unknown button".into()))?;
            let mut rules = Vec::new();
            for r in &l.rules {
                let colour = match r.colour.as_str() {
                    "mode_pad" => LedColour::ModePad,
                    c => LedColour::Rgb(
                        palette.named(c).or_else(|| palette.section(c)).ok_or_else(|| at(format!("unknown colour {c}")))?,
                    ),
                };
                rules.push(LedRule { when: guards(&r.when).map_err(at)?, colour });
            }
            leds.push((b, rules));
        }
        let mut screen = Vec::new();
        for (i, s) in self.screen.iter().enumerate() {
            let at = |e: String| format!("screen rule {}: {e}", i + 1);
            for line in &s.lines {
                for var in template_vars(line) {
                    if !SCREEN_VARS.contains(&var) {
                        return Err(at(format!("unknown variable {{{var}}}")));
                    }
                }
            }
            screen.push(ScreenRule { when: guards(&s.when).map_err(at)?, lines: s.lines.clone() });
        }
        if screen.last().is_none_or(|s| !s.when.is_empty()) {
            return Err("screen: the last rule must have no guards (the default)".into());
        }
        Ok(Rules { navigation, leds, slot_leds: guards(&self.slot_leds.when)?, screen, palette })
    }
}

/// The `{name}` variables of a template.
pub fn template_vars(s: &str) -> Vec<&str> {
    let mut v = Vec::new();
    let mut rest = s;
    while let Some(i) = rest.find('{') {
        let Some(j) = rest[i..].find('}') else { break };
        v.push(&rest[i + 1..i + j]);
        rest = &rest[i + j + 1..];
    }
    v
}

/// Fill a template's variables.
pub fn fill(s: &str, var: impl Fn(&str) -> String) -> String {
    let mut out = String::new();
    let mut rest = s;
    while let Some(i) = rest.find('{') {
        let Some(j) = rest[i..].find('}') else { break };
        out.push_str(&rest[..i]);
        out.push_str(&var(&rest[i + 1..i + j]));
        rest = &rest[i + j + 1..];
    }
    out.push_str(rest);
    out
}
