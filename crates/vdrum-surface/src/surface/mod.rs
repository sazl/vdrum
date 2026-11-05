// SPDX-License-Identifier: GPL-3.0-only
//! The surface state machine: decoded controller input in, plugin actions and display frames out.

use crate::frame::{dim, Frame, Popup};
use crate::layout::{
    Binding, Cell, Layout, Mode, Rgb, Section, Setting, StepField, LOCK_KINDS, TRACKS, TRACK_COLOURS,
    TRACK_TITLES,
};
use crate::model::{options, scale, unscale, Action, Model, Switch, FADER_MAX};
use crate::seq::{SeqEdit, StepData, FLAM_NAMES, MAX_FLAM, NO_LOCK, NO_SHIFT, NUM_PATTERNS, NUM_STEPS};
use lxr_params::par;
use lcxl3::{Button, Control, Input};

mod perform;
mod view;
pub use view::{ControlView, EditView, View};
pub use perform::{kit_params, voice_params, Macro, SLOTS};
use perform::{ActiveMoment, Morph, Scaler};

/// A macro assignment: (parameter, value at fader bottom, value at fader top).
pub type Assign = (u16, u16, u16);

/// Hold time for "hold 1 s" actions, in milliseconds.
pub const HOLD_MS: u64 = 1000;
/// How long a pad stays lit after its track was triggered, in milliseconds.
pub const FLASH_MS: u64 = 120;
/// Changes of one parameter closer together than this are one undo step, in milliseconds.
pub const UNDO_MERGE_MS: u64 = 1000;
/// Undo steps kept.
pub const UNDO_DEPTH: usize = 256;
/// `audio_outN` value of the FX bus.
pub const AUDIO_OUT_FX: u16 = 6;

/// One parameter change made from the controller (for undo and redo).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct Change {
    idx: u16,
    old: u16,
    new: u16,
    at: u64,
}

/// Voices whose parameters can be copied onto each other: drums 1 to 3, snare, cymbal, the two
/// hi-hats.
pub fn copy_group(track: u8) -> u8 {
    match track {
        0..=2 => 0,
        3 => 1,
        4 => 2,
        _ => 3,
    }
}

/// The FX type's own type parameter (distortion type, ring modulator waveform, compressor ratio,
/// delay type), if the type has one.
pub fn fx_sub_param(fx_type: u16) -> Option<u16> {
    let page = lxr_params::FX_PAGE.get(fx_type as usize)?;
    page.get(3).map(|(p, _)| *p).filter(|p| ![par::FX_P1, par::FX_P2, par::FX_P3, par::FX_P4].contains(p))
}

/// Settings the user changes in the Global mode (plugin side).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Settings {
    /// Pad velocity 1..=127.
    pub pad_velocity: u8,
    /// Encoder detents per list entry (lists of up to 16 entries), 1..=8.
    pub list_speed: u8,
    /// Faders pick up the parameter (true) or jump to the fader position.
    pub fader_pickup: bool,
    /// Device settings (feature CCs), as last sent.
    pub led_brightness: u8,
    pub screen_brightness: u8,
    pub popup_time: u8,
    pub encoder_curve: u8,
    /// The sequencer follows the host transport (true) or runs on its own tempo.
    pub host_clock: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            pad_velocity: 100,
            list_speed: 3,
            fader_pickup: true,
            led_brightness: 127,
            screen_brightness: 127,
            popup_time: 10,
            encoder_curve: 1,
            host_clock: true,
        }
    }
}

impl Settings {
    /// `key=value` pairs separated by `;`, for the plugin's saved state.
    pub fn to_text(&self) -> String {
        format!(
            "pad_velocity={};list_speed={};fader_pickup={};led_brightness={};screen_brightness={};popup_time={};encoder_curve={};host_clock={}",
            self.pad_velocity,
            self.list_speed,
            self.fader_pickup as u8,
            self.led_brightness,
            self.screen_brightness,
            self.popup_time,
            self.encoder_curve,
            self.host_clock as u8
        )
    }

    /// Inverse of [`Settings::to_text`]; unknown keys and invalid values are ignored.
    pub fn from_text(text: &str) -> Settings {
        let mut s = Settings::default();
        for pair in text.split(';') {
            let Some((k, v)) = pair.split_once('=') else { continue };
            let Ok(v) = v.trim().parse::<u8>() else { continue };
            match k.trim() {
                "pad_velocity" => s.pad_velocity = v.clamp(1, 127),
                "list_speed" => s.list_speed = v.clamp(1, 8),
                "fader_pickup" => s.fader_pickup = v != 0,
                "led_brightness" => s.led_brightness = v.min(127),
                "screen_brightness" => s.screen_brightness = v.min(127),
                "popup_time" => s.popup_time = v.min(99),
                "encoder_curve" => s.encoder_curve = v.min(2),
                "host_clock" => s.host_clock = v != 0,
                _ => {}
            }
        }
        s
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct FaderState {
    /// Last position (0..=127), if the fader has moved since the plugin started.
    raw: Option<u8>,
    /// The fader has reached the parameter's value since the last mode change.
    picked: bool,
}

/// A control currently held down, with the time it was pressed.
#[derive(Clone, Copy, Debug)]
struct Held {
    control: Control,
    since: u64,
    /// The hold action has fired.
    fired: bool,
}

/// The value a parameter lock gets: moved by an encoder, or set by a fader.
#[derive(Clone, Copy, Debug)]
enum LockValue {
    Delta(i32),
    Set(u8),
}

/// A sequencer step button held down (sequencer view).
#[derive(Clone, Copy, Debug)]
struct HeldStep {
    /// Step 0..64 of the pattern.
    step: u8,
    /// The press set the step on the selected track.
    set_on_press: bool,
    /// A knob changed the step while it was held.
    edited: bool,
}

/// The controller's state.
pub struct Surface {
    mode: Mode,
    prev_mode: Mode,
    layout: Layout,
    shift: bool,
    accent: bool,
    /// Detent accumulators for list parameters, per encoder (row * 8 + column).
    accum: [i16; 24],
    faders: [FaderState; 8],
    held: Vec<Held>,
    popup: Option<Popup>,
    popup_gen: u32,
    last_touched: String,
    pub settings: Settings,
    /// Last seen hit counter and time of the last hit, per track.
    hit_seen: [u32; 7],
    hit_at: [Option<u64>; 7],
    /// Time of the input being handled.
    now: u64,
    /// Copied voice: its copy group and the values by control.
    clipboard: Option<(u8, Vec<(Control, u16)>)>,
    /// Audio output of each track before it was sent to the FX bus.
    fx_prev_out: [u16; 7],
    /// FX type before the FX was switched off.
    fx_prev_type: u16,
    /// Page up/down showed a kit that Play would load.
    browsing: bool,
    comparing: bool,
    history: Vec<Change>,
    redo: Vec<Change>,
    /// Sequencer: bar shown on the step buttons (0..4).
    seq_bar: u8,
    /// Sequencer view (Solo/Arm): the 16 buttons are the step buttons in any mode. Always on in
    /// mode 8.
    view: bool,
    /// Shift + Solo/Arm: the sequencer view stays on when the mode changes.
    sticky: bool,
    /// Sequencer: the selected track (Track left/right). The step buttons show it and its knob
    /// LEDs pulse in mode 8.
    seq_track: u8,
    held_steps: Vec<HeldStep>,
    /// Velocity of steps set with the step buttons.
    entry_velocity: u8,
    /// A layer button (Solo/Arm, Mute/Select in mode 8) was used while held.
    layer_used: bool,
    /// The button being released had fired its hold action.
    release_fired: bool,
    moments: Vec<ActiveMoment>,
    /// Momentary effect depth and rate, 0..=127.
    moment_depth: u8,
    moment_rate: u8,
    /// LFO rate, LFO depth, velocity amount scales.
    scalers: [Scaler; 3],
    /// LFO sync settings before *Sync to host*.
    lfo_sync_saved: Option<Vec<(u16, u16)>>,
    snapshots: [Option<Vec<(u16, u16)>>; SLOTS],
    morph: [Morph; 7],
    morph_all: u8,
    target_all: u8,
    macros: [Macro; SLOTS],
    macro_pos: [u8; SLOTS],
    /// Macro being learned and the changes recorded so far.
    learning: Option<(u8, Vec<Assign>)>,
    state_changed: bool,
    /// Input from the plugin editor is being handled (it is not a hardware edit).
    editor_input: bool,
    edit_gen: u32,
    last_edit: Option<EditView>,
}

impl Default for Surface {
    fn default() -> Self {
        Surface::new()
    }
}

impl Surface {
    pub fn new() -> Surface {
        let mode = Mode::Voice(0);
        Surface {
            mode,
            prev_mode: mode,
            layout: Layout::of(mode, false),
            shift: false,
            accent: false,
            accum: [0; 24],
            faders: [FaderState::default(); 8],
            held: Vec::new(),
            popup: None,
            popup_gen: 0,
            last_touched: String::new(),
            settings: Settings::default(),
            hit_seen: [0; 7],
            hit_at: [None; 7],
            now: 0,
            clipboard: None,
            fx_prev_out: [0; 7],
            fx_prev_type: 1,
            browsing: false,
            comparing: false,
            history: Vec::new(),
            redo: Vec::new(),
            seq_bar: 0,
            view: false,
            sticky: false,
            seq_track: 0,
            held_steps: Vec::new(),
            entry_velocity: 100,
            layer_used: false,
            release_fired: false,
            moments: Vec::new(),
            moment_depth: 96,
            moment_rate: 64,
            scalers: std::array::from_fn(|_| Scaler { pct: 100, base: Vec::new() }),
            lfo_sync_saved: None,
            snapshots: Default::default(),
            morph: Default::default(),
            morph_all: 0,
            target_all: 0,
            macros: std::array::from_fn(|_| Macro { assign: Vec::new(), depth: 100 }),
            macro_pos: [0; SLOTS],
            learning: None,
            state_changed: false,
            editor_input: false,
            edit_gen: 0,
            last_edit: None,
        }
    }

    /// The sequencer view is active: the 16 buttons are the step buttons (always in mode 8).
    pub fn seq_view(&self) -> bool {
        self.view || self.mode == Mode::Sequencer
    }

    /// The sequencer view stays on across mode changes (Shift + Solo/Arm).
    pub fn seq_view_sticky(&self) -> bool {
        self.sticky
    }

    /// Rebuild the layout for the mode, the held steps (mode 8) and the sequencer view.
    fn relayout(&mut self) {
        let l = Layout::of(self.mode, !self.held_steps.is_empty());
        self.layout = if self.view && self.mode != Mode::Sequencer { l.with_steps() } else { l };
    }

    /// The sequencer's selected track (0..7).
    pub fn seq_track(&self) -> u8 {
        self.seq_track
    }


    /// Clear track `t`'s pattern in the playing pattern: Euclid pulses off and every step back to
    /// its default (off, no locks). Length and density stay.
    fn clear_track(&mut self, t: u8, model: &dyn Model, out: &mut Vec<Action>) {
        let pat = model.seq_status().pattern;
        out.push(Action::Seq(SeqEdit::Euclid { pat, track: t, fills: 0, rotate: 0 }));
        let edits: Vec<SeqEdit> = {
            let d = model.seq();
            (0..NUM_STEPS as u8)
                .filter(|&step| d.step(pat, t, step) != StepData::default())
                .map(|step| SeqEdit::Step { pat, track: t, step, data: StepData::default() })
                .collect()
        };
        out.extend(edits.into_iter().map(Action::Seq));
        self.show_coloured("Clear", "Track", &format!("{} {}", t + 1, TRACKS[t as usize]), TRACK_COLOURS[t as usize]);
    }

    /// Select sequencer track `t`.
    fn select_seq_track(&mut self, t: u8) {
        self.seq_track = t.min(6);
        self.show_coloured(
            "Sequencer",
            "Track",
            &format!("{} {}", self.seq_track + 1, TRACKS[self.seq_track as usize]),
            TRACK_COLOURS[self.seq_track as usize],
        );
    }

    /// Show a message in the popup (for results the plugin reports, such as kit names).
    pub fn notify(&mut self, title: &str, name: &str, value: &str) {
        self.show_text(title, name, value);
    }

    /// Whether a kit shown by Page up/down is waiting for Play.
    pub fn browsing(&self) -> bool {
        self.browsing
    }

    /// Set a parameter from the controller, keeping the undo history.
    fn set(&mut self, idx: u16, value: u16, model: &dyn Model, out: &mut Vec<Action>) {
        let old = model.param(idx);
        if old == value {
            return;
        }
        out.push(Action::SetParam { idx, value });
        self.learn_record(idx, old, value);
        self.redo.clear();
        match self.history.last_mut() {
            Some(c) if c.idx == idx && self.now.saturating_sub(c.at) < UNDO_MERGE_MS => {
                c.new = value;
                c.at = self.now;
            }
            _ => {
                self.history.push(Change { idx, old, new: value, at: self.now });
                if self.history.len() > UNDO_DEPTH {
                    self.history.remove(0);
                }
            }
        }
    }

    fn undo(&mut self, redo: bool, out: &mut Vec<Action>) {
        let (from, to) = if redo { (&mut self.redo, &mut self.history) } else { (&mut self.history, &mut self.redo) };
        let Some(mut c) = from.pop() else {
            self.show_text("Edit", if redo { "Redo" } else { "Undo" }, "nothing to do");
            return;
        };
        let value = if redo { c.new } else { c.old };
        out.push(Action::SetParam { idx: c.idx, value });
        c.at = 0;
        to.push(c);
        let name = lxr_params::info(c.idx).map(|i| i.name).unwrap_or("?");
        let text = lxr_params::info(c.idx).map(|i| lxr_params::format_value(i, value.min(255) as u8)).unwrap_or_default();
        self.show_text(if redo { "Redo" } else { "Undo" }, name, text.trim());
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn layout(&self) -> &Layout {
        &self.layout
    }

    /// Switch to `mode`. Faders must pick up their new parameters again.
    pub fn set_mode(&mut self, mode: Mode) {
        if mode != self.mode {
            self.prev_mode = self.mode;
            self.mode = mode;
        }
        if !self.sticky {
            self.view = false;
        }
        self.held_steps.clear();
        self.relayout();
        for f in &mut self.faders {
            f.picked = false;
        }
        self.accum = [0; 24];
        self.held.retain(|h| matches!(h.control, Control::Button(Button::Shift)));
        self.held_steps.clear();
    }

    /// The device messages that set the device's own settings to `self.settings`.
    pub fn device_settings(&self) -> Vec<Vec<u8>> {
        use lcxl3::encode::set_feature;
        use lcxl3::Feature;
        let s = &self.settings;
        [
            (Feature::LedBrightness, s.led_brightness),
            (Feature::ScreenBrightness, s.screen_brightness),
            (Feature::DisplayTimeout, s.popup_time),
            (Feature::EncoderCurve, s.encoder_curve),
        ]
        .into_iter()
        .filter_map(|(f, v)| set_feature(f, v).map(|m| m.to_vec()))
        .collect()
    }

    /// Handle one decoded message from the device at time `now` (milliseconds, any origin).
    pub fn handle(&mut self, input: &Input, now: u64, model: &dyn Model) -> Vec<Action> {
        let mut out = Vec::new();
        self.now = now;
        let browse_key = matches!(
            input,
            Input::Button { button: Button::PageUp | Button::PageDown | Button::Play, .. }
                | Input::Button { pressed: false, .. }
        );
        if !browse_key && !matches!(input, Input::Other | Input::Touch { .. } | Input::Feature { .. }) {
            self.browsing = false;
        }
        match *input {
            Input::Relative { row, col, delta } => {
                let c = Control::Encoder { row, col };
                if let Some(cell) = self.layout.get(c).cloned() {
                    self.encoder(c, &cell, delta as i32, model, &mut out);
                }
            }
            Input::Analogue { control: c @ Control::Fader(col), value } => {
                if let Some(cell) = self.layout.get(c).cloned() {
                    self.fader(col, &cell, value, model, &mut out);
                } else {
                    self.faders[col as usize].raw = Some(value);
                }
            }
            Input::Button { button, pressed } => self.button(button, pressed, now, model, &mut out),
            // Encoders are switched to relative mode; absolute encoder messages are ignored.
            _ => {}
        }
        out
    }

    /// Advance time: fires hold actions. Call regularly (every few milliseconds).
    pub fn tick(&mut self, now: u64, model: &dyn Model) -> Vec<Action> {
        let mut out = Vec::new();
        self.now = now;
        self.moment_tick(model, &mut out);
        for t in 0..7u8 {
            let h = model.hits(t);
            if h != self.hit_seen[t as usize] {
                self.hit_seen[t as usize] = h;
                self.hit_at[t as usize] = Some(now);
            }
        }
        let due: Vec<Control> = self
            .held
            .iter()
            .filter(|h| !h.fired && now.saturating_sub(h.since) >= HOLD_MS)
            .map(|h| h.control)
            .collect();
        for c in due {
            if let Some(h) = self.held.iter_mut().find(|h| h.control == c) {
                h.fired = true;
            }
            if let Some(cell) = self.layout.get(c).cloned() {
                self.hold_fired(&cell, model, &mut out);
            }
        }
        out
    }

    fn is_held(&self, c: Control) -> bool {
        self.held.iter().any(|h| h.control == c)
    }

    fn encoder(&mut self, c: Control, cell: &Cell, delta: i32, model: &dyn Model, out: &mut Vec<Action>) {
        let Control::Encoder { row, col } = c else { return };
        // Sequencer view outside mode 8, step held: the control's parameter is locked on the step.
        if self.mode != Mode::Sequencer && !self.held_steps.is_empty() {
            match cell.binding {
                Binding::Param(idx) => self.lock_param(cell, idx, LockValue::Delta(delta), model, out),
                _ => self.show(cell, "cannot be locked".to_string()),
            }
            return;
        }
        match cell.binding {
            Binding::Param(idx) => {
                let Some(info) = lxr_params::info(idx) else { return };
                let opts = options(info, model);
                let cur = model.param(idx);
                let pos = opts.iter().position(|v| *v == cur).unwrap_or(0) as i32;
                let steps = if opts.len() <= 16 {
                    let a = &mut self.accum[(row * 8 + col) as usize];
                    *a += delta as i16;
                    let n = self.settings.list_speed.max(1) as i16;
                    let s = *a / n;
                    *a -= s * n;
                    s as i32
                } else {
                    delta
                };
                if steps == 0 {
                    return;
                }
                let new = opts[(pos + steps).clamp(0, opts.len() as i32 - 1) as usize];
                self.set(idx, new, model, out);
                self.show_param(cell, idx, new, model);
            }
            Binding::FxSubType => match fx_sub_param(model.param(par::FX_TYPE)) {
                Some(idx) => {
                    let label = lxr_params::info(idx).map(|i| i.name).unwrap_or("FX sub-type");
                    let sub = Cell { label: label.trim_start_matches("FX ").to_string(), ..cell.clone() };
                    self.encoder(c, &Cell { binding: Binding::Param(idx), ..sub }, delta, model, out);
                }
                None => self.show(cell, "none for this FX".to_string()),
            },
            Binding::Setting(s) => self.setting(cell, s, delta, out),
            Binding::SeqFills(_)
            | Binding::SeqRotate(_)
            | Binding::SeqLength(_)
            | Binding::SeqShuffle
            | Binding::SeqNextPattern
            | Binding::SeqEntryVelocity => self.seq_encoder(cell, delta, model, out),
            Binding::StepField(f) => self.step_field(cell, f, delta, model, out),
            Binding::MomentDepth | Binding::MomentRate => {
                let v = if cell.binding == Binding::MomentDepth { &mut self.moment_depth } else { &mut self.moment_rate };
                *v = (*v as i32 + delta).clamp(0, 127) as u8;
                let text = v.to_string();
                self.show(cell, text);
            }
            Binding::LfoRateScale => {
                let pct = self.scalers[0].pct + delta * 5;
                self.scale(cell, 0, pct, model, out);
            }
            Binding::VelScale => {
                let pct = self.scalers[2].pct + delta * 5;
                self.scale(cell, 2, pct, model, out);
            }
            Binding::LfoOffsetAll => self.lfo_offset_all(cell, delta, model, out),
            Binding::MacroDepth(n) => self.macro_depth(cell, n, delta),
            Binding::TrackMorph(t) => {
                let a = (self.morph[t as usize].amount as i32 + delta).clamp(0, 127) as u8;
                let text = self.morph_set(t, Some(a), None, model, out);
                self.show(cell, text);
            }
            Binding::TrackTarget(t) => {
                let g = (self.morph[t as usize].target as i32 + delta).clamp(0, SLOTS as i32 - 1) as u8;
                let text = self.morph_set(t, None, Some(g), model, out);
                self.show(cell, text);
            }
            Binding::MorphAll | Binding::TargetAll => {
                let all = cell.binding == Binding::MorphAll;
                let v = if all { &mut self.morph_all } else { &mut self.target_all };
                let hi = if all { 127 } else { SLOTS as i32 - 1 };
                *v = (*v as i32 + delta).clamp(0, hi) as u8;
                let v = *v;
                let mut text = String::new();
                for t in 0..7u8 {
                    text = if all {
                        self.morph_set(t, Some(v), None, model, out)
                    } else {
                        self.morph_set(t, None, Some(v), model, out)
                    };
                }
                self.show(cell, text);
            }
            Binding::LockTarget(k) => self.lock_target(cell, k, delta, model, out),
            _ => {}
        }
    }

    fn seq_encoder(&mut self, cell: &Cell, delta: i32, model: &dyn Model, out: &mut Vec<Action>) {
        let st = model.seq_status();
        let pat = st.pattern;
        let (len, fills, rot, shuffle) = {
            let d = model.seq();
            let t = match cell.binding {
                Binding::SeqFills(t) | Binding::SeqRotate(t) | Binding::SeqLength(t) => t,
                _ => 0,
            };
            let tr = d.track(pat, t);
            (tr.length as i32, tr.fills as i32, tr.rotate as i32, d.patterns[pat as usize].shuffle as i32)
        };
        let text = match cell.binding {
            Binding::SeqFills(track) => {
                let f = (fills + delta).clamp(0, len);
                if f != fills {
                    out.push(Action::Seq(SeqEdit::Euclid { pat, track, fills: f as u8, rotate: rot as u8 }));
                }
                if f == 0 {
                    "off".to_string()
                } else {
                    format!("{f} of {len}")
                }
            }
            Binding::SeqRotate(track) => {
                if fills == 0 {
                    "turn Fills first".to_string()
                } else {
                    let r = (rot + delta).rem_euclid(len);
                    if r != rot {
                        out.push(Action::Seq(SeqEdit::Euclid { pat, track, fills: fills as u8, rotate: r as u8 }));
                    }
                    r.to_string()
                }
            }
            Binding::SeqLength(track) => {
                let l = (len + delta).clamp(1, NUM_STEPS as i32);
                if l != len {
                    out.push(Action::Seq(SeqEdit::Length { pat, track, length: l as u8 }));
                }
                format!("{l} steps")
            }
            Binding::SeqShuffle => {
                let v = (shuffle + delta).clamp(0, 127);
                if v != shuffle {
                    out.push(Action::Seq(SeqEdit::Shuffle { pat, value: v as u8 }));
                }
                v.to_string()
            }
            Binding::SeqNextPattern => {
                let p = (st.pending as i32 + delta).clamp(0, NUM_PATTERNS as i32 - 1) as u8;
                if p != st.pending {
                    out.push(Action::Seq(SeqEdit::NextPattern(p)));
                }
                if st.running {
                    format!("P{:02} at bar end", p + 1)
                } else {
                    format!("P{:02}", p + 1)
                }
            }
            Binding::SeqEntryVelocity => {
                self.entry_velocity = (self.entry_velocity as i32 + delta).clamp(1, 127) as u8;
                self.entry_velocity.to_string()
            }
            _ => return,
        };
        self.show(cell, text);
    }

    /// Change a field of the held steps of the selected track.
    fn step_field(&mut self, cell: &Cell, field: StepField, delta: i32, model: &dyn Model, out: &mut Vec<Action>) {
        let pat = model.seq_status().pattern;
        let tracks = [self.seq_track];
        let mut text = None;
        {
            let d = model.seq();
            for h in &self.held_steps {
                for &track in &tracks {
                    let old = d.step(pat, track, h.step);
                    let mut s = old;
                    let bump = |v: u8, lo: i32, hi: i32| (v as i32 + delta).clamp(lo, hi) as u8;
                    let t = match field {
                        StepField::Velocity => {
                            s.velocity = bump(s.velocity, 1, 127);
                            s.velocity.to_string()
                        }
                        StepField::Probability => {
                            s.prob = bump(s.prob, 0, 100);
                            format!("{} %", s.prob)
                        }
                        StepField::Note => {
                            s.note = bump(s.note, 0, 127);
                            s.note.to_string()
                        }
                        StepField::Flam => {
                            s.flam = bump(s.flam, 0, MAX_FLAM as i32);
                            FLAM_NAMES[s.flam as usize].to_string()
                        }
                        StepField::Shift => {
                            s.shift = bump(s.shift, 0, 14);
                            format!("{:+}", s.shift as i32 - NO_SHIFT as i32)
                        }
                        StepField::Lock1 | StepField::Lock2 => {
                            let slot = &mut s.locks[(field == StepField::Lock2) as usize];
                            if slot.0 == NO_LOCK {
                                "no lock".to_string()
                            } else {
                                slot.1 = bump(slot.1, 0, 127);
                                format!("{} {}", lock_name(slot.0), slot.1)
                            }
                        }
                        StepField::ClearLocks => {
                            s.locks = [(NO_LOCK, 0); 2];
                            "cleared".to_string()
                        }
                    };
                    text.get_or_insert(t);
                    if s != old {
                        out.push(Action::Seq(SeqEdit::Step { pat, track, step: h.step, data: s }));
                    }
                }
            }
        }
        for h in &mut self.held_steps {
            h.edited = true;
        }
        self.show(cell, text.unwrap_or_else(|| "hold a step".to_string()));
    }

    /// Mode 8, step held: lock knob `k`'s parameter kind of the selected track on the held steps.
    fn lock_target(&mut self, cell: &Cell, k: u8, delta: i32, model: &dyn Model, out: &mut Vec<Action>) {
        let idx = crate::layout::voice_param(self.seq_track, LOCK_KINDS[k as usize % 8]);
        self.lock_param(cell, idx, LockValue::Delta(delta), model, out);
    }

    /// Store a lock of parameter `idx` on the held steps of the selected track. The engine's step
    /// locks hold parameter numbers 0..254 only, so the open hi-hat's parameters cannot be locked.
    fn lock_param(&mut self, cell: &Cell, idx: u16, value: LockValue, model: &dyn Model, out: &mut Vec<Action>) {
        let pat = model.seq_status().pattern;
        // (track, parameter, current kit value as a CC value)
        let targets: Vec<(u8, u8, u8)> = lxr_params::info(idx)
            .filter(|_| idx < NO_LOCK as u16)
            .map(|info| (self.seq_track, idx as u8, unscale(info.min, info.max, model.param(idx))))
            .into_iter()
            .collect();
        let mut text = None;
        let mut replaced = None;
        {
            let d = model.seq();
            for h in &self.held_steps {
                for &(track, idx, base) in &targets {
                    let old = d.step(pat, track, h.step);
                    let mut s = old;
                    let slot = match s.locks.iter().position(|l| l.0 == idx) {
                        Some(i) => i,
                        None => match s.locks.iter().position(|l| l.0 == NO_LOCK) {
                            Some(i) => {
                                s.locks[i] = (idx, base);
                                i
                            }
                            None => {
                                // a third lock replaces the oldest (slot 0)
                                replaced.get_or_insert(s.locks[0].0);
                                s.locks = [s.locks[1], (idx, base)];
                                1
                            }
                        },
                    };
                    let v = match value {
                        LockValue::Delta(delta) => (s.locks[slot].1 as i32 + delta).clamp(0, 127) as u8,
                        LockValue::Set(v) => v.min(127),
                    };
                    s.locks[slot].1 = v;
                    text.get_or_insert(format!("{} {v}", lock_name(idx)));
                    if s != old {
                        out.push(Action::Seq(SeqEdit::Step { pat, track, step: h.step, data: s }));
                    }
                }
            }
        }
        for h in &mut self.held_steps {
            h.edited = true;
        }
        let text = match (text, replaced) {
            (None, _) if targets.is_empty() => "not for the open hi-hat".to_string(),
            (None, _) => "hold a step".to_string(),
            (Some(t), Some(r)) => format!("{t} (replaced {})", lock_name(r)),
            (Some(t), None) => t,
        };
        self.show(cell, text);
    }

    fn fader(&mut self, col: u8, cell: &Cell, raw: u8, model: &dyn Model, out: &mut Vec<Action>) {
        if self.mode != Mode::Sequencer && !self.held_steps.is_empty() {
            self.faders[col as usize] = FaderState { raw: Some(raw), picked: false };
            match cell.binding {
                Binding::Param(idx) => self.lock_param(cell, idx, LockValue::Set(raw), model, out),
                _ => self.show(cell, "cannot be locked".to_string()),
            }
            return;
        }
        let st = self.faders[col as usize];
        let (min, max, cur) = match cell.binding {
            Binding::Param(idx) => match lxr_params::info(idx) {
                Some(info) => (info.min, info.max, model.param(idx)),
                None => return,
            },
            Binding::VoiceFader(t) => (0, FADER_MAX as u16, model.voice_fader(t).clamp(0, FADER_MAX) as u16),
            Binding::SeqDensity(t) => (0, 100, model.seq().density[t as usize % 7] as u16),
            Binding::Macro(n) => (0, 127, self.macro_pos[n as usize % SLOTS] as u16),
            Binding::LfoDepthScale => (0, 200, self.scalers[1].pct as u16),
            Binding::SeqDensityAll => (0, 100, model.seq().density_all as u16),
            _ => {
                self.faders[col as usize].raw = Some(raw);
                return;
            }
        };
        let cur_raw = unscale(min, max, cur) as i16;
        let picked = st.picked || !self.settings.fader_pickup || {
            let r = raw as i16;
            let crossed = st.raw.is_some_and(|p| (p as i16 - cur_raw) * (r - cur_raw) <= 0);
            crossed || (r - cur_raw).abs() <= 1
        };
        self.faders[col as usize] = FaderState { raw: Some(raw), picked };
        let value = scale(min, max, raw);
        if !picked {
            let dir = if cur_raw > raw as i16 { "up" } else { "down" };
            self.show(cell, format!("{} (move {dir})", self.value_text(cell, cur, model)));
            return;
        }
        match cell.binding {
            Binding::Param(idx) => {
                self.set(idx, value, model, out);
                self.show_param(cell, idx, value, model);
            }
            Binding::VoiceFader(track) => {
                if value != cur {
                    out.push(Action::SetVoiceFader { track, step: value as i32 });
                }
                self.show(cell, value.to_string());
            }
            Binding::Macro(n) => {
                self.macro_pos[n as usize % SLOTS] = value as u8;
                let text = self.macro_move(n, value as u8, model, out);
                self.show(cell, text);
            }
            Binding::LfoDepthScale => self.scale(cell, 1, value as i32, model, out),
            Binding::SeqDensity(_) | Binding::SeqDensityAll => {
                let track = if let Binding::SeqDensity(t) = cell.binding { t } else { 7 };
                if value != cur {
                    out.push(Action::Seq(SeqEdit::Density { track, value: value as u8 }));
                }
                self.show(cell, format!("{value} %"));
            }
            _ => {}
        }
    }

    fn button(&mut self, button: Button, pressed: bool, now: u64, model: &dyn Model, out: &mut Vec<Action>) {
        let c = Control::Button(button);
        if pressed {
            if !self.is_held(c) {
                self.held.push(Held { control: c, since: now, fired: false });
            }
        } else {
            self.release_fired = self.held.iter().any(|h| h.control == c && h.fired);
            self.held.retain(|h| h.control != c);
        }
        match button {
            Button::Shift => self.shift = pressed,
            // Shift + Solo/Arm: the sequencer view stays on across mode changes (sticky), or not
            Button::SoloArm if self.shift => {
                if pressed {
                    self.sticky = !self.sticky;
                    if self.sticky {
                        self.view = true;
                    }
                    self.relayout();
                    self.show_text("Sequencer view", "Sticky", if self.sticky { "on" } else { "off" });
                }
            }
            Button::Top(n) | Button::Bottom(n) if self.shift && pressed => {
                let slot = if matches!(button, Button::Top(_)) { n + 1 } else { n + 9 };
                match Mode::from_number(slot) {
                    Some(m) => self.set_mode(m),
                    None => self.show_text("Mode", &format!("Slot {slot}"), "empty"),
                }
            }
            Button::Top(n) | Button::Bottom(n) if self.seq_view() => {
                let idx = if matches!(button, Button::Top(_)) { n } else { n + 8 };
                self.seq_button(idx, pressed, model, out);
            }
            // Track left/right select the sequencer track in the sequencer view and do nothing
            // elsewhere: modes are selected with Shift + button N.
            Button::TrackLeft | Button::TrackRight if self.seq_view() => {
                if pressed && self.is_held(Control::Button(Button::Record)) {
                    self.clear_track(self.seq_track, model, out);
                } else if pressed {
                    let t = self.seq_track as i32 + if button == Button::TrackLeft { -1 } else { 1 };
                    self.select_seq_track(t.clamp(0, 6) as u8);
                }
            }
            Button::TrackLeft | Button::TrackRight => {}
            Button::Logo => {
                if pressed {
                    self.set_mode(self.prev_mode);
                }
            }
            // Solo/Arm toggles the sequencer view (always on in mode 8).
            Button::SoloArm => {
                if pressed {
                    if self.mode == Mode::Sequencer {
                        self.show_text("Sequencer view", "always on", "in mode 8");
                    } else {
                        self.view = !self.view;
                        self.held_steps.clear();
                        self.relayout();
                        let track = format!("{} {}", self.seq_track + 1, TRACKS[self.seq_track as usize]);
                        self.show_text("Sequencer view", if self.view { "on" } else { "off" }, &track);
                    }
                }
            }
            // In the sequencer view Mute/Select is the pattern layer while held; a short press that
            // did not use the layer still goes to Perform.
            Button::MuteSelect if self.seq_view() => {
                if pressed {
                    self.layer_used = false;
                    self.show_text("Sequencer", "Patterns", "patterns 1 to 16");
                } else if !self.layer_used {
                    self.set_mode(Mode::Perform);
                }
            }
            Button::MuteSelect => {
                if pressed {
                    self.set_mode(Mode::Perform);
                }
            }
            // Shift + Page up/down: browse the kits; Play loads the kit shown.
            Button::PageUp | Button::PageDown if self.shift => {
                if pressed {
                    self.browsing = true;
                    out.push(Action::KitBrowse(if button == Button::PageUp { -1 } else { 1 }));
                }
            }
            Button::PageUp | Button::PageDown if self.seq_view() => {
                if pressed {
                    let bar = self.seq_bar as i32 + if button == Button::PageUp { -1 } else { 1 };
                    self.seq_bar = bar.clamp(0, 3) as u8;
                    self.show_text("Sequencer", "Bar", &format!("{} of 4", self.seq_bar + 1));
                }
            }
            Button::PageUp | Button::PageDown => {}
            // Record = Clear (2026-11-05): hold it and press a step (sequencer view) to clear that
            // step, or Track left/right to clear the selected track's pattern.
            Button::Record => {
                if pressed {
                    self.show_text("Clear", "hold +", "step / Track < >");
                }
            }
            Button::Play => {
                if pressed {
                    self.play(model, out);
                }
            }
            Button::Top(_) | Button::Bottom(_) => {
                if let Some(cell) = self.layout.get(c).cloned() {
                    if pressed {
                        self.press(&cell, model, out);
                    } else {
                        self.release(&cell, model, out);
                    }
                }
            }
        }
    }

    /// Play: loads a browsed kit; otherwise stops a running sequencer (internal clock) or silences
    /// it and lets it play again (host clock, which the plugin cannot stop); otherwise starts the
    /// sequencer in mode 8 (internal clock) or plays the voice of a voice mode.
    fn play(&mut self, model: &dyn Model, out: &mut Vec<Action>) {
        if self.browsing {
            self.browsing = false;
            out.push(Action::KitLoadBrowsed);
            return;
        }
        let st = model.seq_status();
        if self.settings.host_clock && (st.running || st.silent) {
            let silent = !st.silent;
            out.push(Action::Seq(SeqEdit::Silence(silent)));
            self.show_text("Sequencer", "Host clock", if silent { "silenced" } else { "playing" });
        } else if !self.settings.host_clock && st.running {
            out.push(Action::Seq(SeqEdit::Run(false)));
            self.show_text("Sequencer", "Internal clock", "stop");
        } else if self.mode == Mode::Sequencer {
            if self.settings.host_clock {
                self.show_text("Sequencer", "Clock: host", "set Internal in Global");
            } else {
                out.push(Action::Seq(SeqEdit::Run(true)));
                self.show_text("Sequencer", "Internal clock", "start");
            }
        } else if let Some(track) = self.mode.track() {
            out.push(Action::Trigger { track, velocity: self.pad_velocity() });
        }
    }

    /// A step button (or, in the pattern layer, a pattern button) in the sequencer view.
    fn seq_button(&mut self, idx: u8, pressed: bool, model: &dyn Model, out: &mut Vec<Action>) {
        let st = model.seq_status();
        if self.is_held(Control::Button(Button::Record)) {
            if pressed {
                let step = self.seq_bar * 16 + idx;
                let track = self.seq_track;
                out.push(Action::Seq(SeqEdit::Step { pat: st.pattern, track, step, data: StepData::default() }));
                self.show_text("Clear", &format!("Step {}", step + 1), TRACKS[track as usize]);
            }
            return;
        }
        if self.is_held(Control::Button(Button::MuteSelect)) {
            if pressed {
                self.layer_used = true;
                out.push(Action::Seq(SeqEdit::NextPattern(idx)));
                let when = if st.running { "at bar end" } else { "now" };
                self.show_text("Sequencer", &format!("Pattern {}", idx + 1), when);
            }
            return;
        }
        let pat = st.pattern;
        let step = self.seq_bar * 16 + idx;
        if pressed {
            if self.held_steps.iter().any(|h| h.step == step) {
                return;
            }
            let edits: Vec<SeqEdit> = {
                let d = model.seq();
                std::iter::once(self.seq_track)
                    .filter_map(|track| {
                        let s = d.step(pat, track, step);
                        (!s.on).then_some(SeqEdit::Step {
                            pat,
                            track,
                            step,
                            data: StepData { on: true, velocity: self.entry_velocity, ..s },
                        })
                    })
                    .collect()
            };
            let set = !edits.is_empty();
            out.extend(edits.into_iter().map(Action::Seq));
            self.held_steps.push(HeldStep { step, set_on_press: set, edited: false });
            self.show_text("Sequencer", &format!("Step {}", step + 1), if set { "on" } else { "release: off" });
        } else if let Some(pos) = self.held_steps.iter().position(|h| h.step == step) {
            let h = self.held_steps.remove(pos);
            if !h.set_on_press && !h.edited {
                let edits: Vec<SeqEdit> = {
                    let d = model.seq();
                    std::iter::once(self.seq_track)
                        .filter_map(|track| {
                            let s = d.step(pat, track, step);
                            s.on.then_some(SeqEdit::Step { pat, track, step, data: StepData { on: false, ..s } })
                        })
                        .collect()
                };
                out.extend(edits.into_iter().map(Action::Seq));
                self.show_text("Sequencer", &format!("Step {}", step + 1), "off");
            }
        }
        self.relayout();
    }

    fn pad_velocity(&self) -> u8 {
        if self.accent {
            127
        } else {
            self.settings.pad_velocity.clamp(1, 127)
        }
    }

    fn press(&mut self, cell: &Cell, model: &dyn Model, out: &mut Vec<Action>) {
        match cell.binding {
            Binding::Param(idx) => {
                let Some(info) = lxr_params::info(idx) else { return };
                let opts = options(info, model);
                let cur = model.param(idx);
                let pos = opts.iter().position(|v| *v == cur).unwrap_or(0);
                let new = opts[(pos + 1) % opts.len()];
                self.set(idx, new, model, out);
                self.show_param(cell, idx, new, model);
            }
            Binding::Pad(track) | Binding::Trigger(track) => {
                let velocity = self.pad_velocity();
                out.push(Action::Trigger { track, velocity });
            }
            Binding::Accent => self.accent = true,
            Binding::CopyVoice => {
                if let Some(t) = self.mode.track() {
                    let values = self
                        .layout
                        .iter()
                        .filter_map(|(c, cell)| match cell.binding {
                            Binding::Param(i) if i != crate::layout::p("hat_choke") => Some((*c, model.param(i))),
                            _ => None,
                        })
                        .collect();
                    self.clipboard = Some((copy_group(t), values));
                    self.show(cell, "copied; hold Paste 1 s".to_string());
                }
            }
            Binding::PasteVoice => self.show(cell, "hold 1 s".to_string()),
            Binding::ToFx(t) => {
                let idx = crate::layout::voice_param(t, "out");
                let cur = model.param(idx);
                let new = if cur == AUDIO_OUT_FX {
                    self.fx_prev_out[t as usize]
                } else {
                    self.fx_prev_out[t as usize] = cur;
                    AUDIO_OUT_FX
                };
                self.set(idx, new, model, out);
                self.show_param(cell, idx, new, model);
            }
            Binding::FxOnOff => {
                let cur = model.param(par::FX_TYPE);
                let new = if cur == 0 {
                    self.fx_prev_type.max(1)
                } else {
                    self.fx_prev_type = cur;
                    0
                };
                self.set(par::FX_TYPE, new, model, out);
                self.show_param(cell, par::FX_TYPE, new, model);
            }
            Binding::SumAux | Binding::HwClock => {
                let sw = if cell.binding == Binding::SumAux { Switch::SumAux } else { Switch::HwClock };
                let on = !model.switch(sw);
                out.push(Action::SetSwitch(sw, on));
                self.show(cell, if on { "on" } else { "off" }.to_string());
            }
            Binding::InitKit | Binding::SaveKit => self.show(cell, "hold 1 s".to_string()),
            Binding::Compare => {
                self.comparing = true;
                out.push(Action::Compare(true));
                self.show(cell, "saved kit (release: back)".to_string());
            }
            Binding::Mute(t) => self.toggle_mute(cell, Some(t), model, out),
            Binding::MuteAll => self.toggle_mute(cell, None, model, out),
            Binding::Moment(k) => self.moment_press(cell, k, model, out),
            Binding::LfoSyncHost => self.lfo_sync_host(cell, model, out),
            Binding::LfoResetPhases => {
                out.push(Action::LfoReset);
                self.show(cell, "done".to_string());
            }
            Binding::Snapshot(_) => self.show(cell, "release: recall, hold 1 s: store".to_string()),
            Binding::Learn(n) => self.learn_press(cell, n),
            Binding::Undo => self.undo(false, out),
            Binding::Redo => self.undo(true, out),
            Binding::Panic => {
                out.push(Action::Panic);
                self.show(cell, "all voices off".to_string());
            }
            _ => {}
        }
    }

    fn release(&mut self, cell: &Cell, model: &dyn Model, out: &mut Vec<Action>) {
        match cell.binding {
            Binding::Moment(k) => self.moment_release(cell, k, model, out),
            Binding::Snapshot(n) if !self.release_fired => self.snapshot_recall(cell, n, model, out),
            Binding::Accent => self.accent = false,
            Binding::Compare if self.comparing => {
                self.comparing = false;
                out.push(Action::Compare(false));
                self.show(cell, "back to the edited kit".to_string());
            }
            _ => {}
        }
    }

    fn hold_fired(&mut self, cell: &Cell, model: &dyn Model, out: &mut Vec<Action>) {
        match cell.binding {
            Binding::PasteVoice => {
                let Some(t) = self.mode.track() else { return };
                match self.clipboard.clone() {
                    Some((g, values)) if g == copy_group(t) => {
                        for (c, v) in values {
                            if let Some(Binding::Param(idx)) = self.layout.get(c).map(|x| x.binding) {
                                self.set(idx, v, model, out);
                            }
                        }
                        self.show(cell, "pasted".to_string());
                    }
                    Some(_) => self.show(cell, "different voice type".to_string()),
                    None => self.show(cell, "nothing copied".to_string()),
                }
            }
            Binding::Snapshot(n) => self.snapshot_store(cell, n, model),
            Binding::InitKit => {
                out.push(Action::InitKit);
                self.show(cell, "init kit loaded".to_string());
            }
            Binding::SaveKit => {
                out.push(Action::SaveKit);
                self.show(cell, "saving".to_string());
            }
            _ => {}
        }
    }

    fn setting(&mut self, cell: &Cell, s: Setting, delta: i32, out: &mut Vec<Action>) {
        use lcxl3::encode::set_feature;
        use lcxl3::Feature;
        let st = &mut self.settings;
        let bump = |v: &mut u8, lo: i32, hi: i32| {
            *v = (*v as i32 + delta).clamp(lo, hi) as u8;
            *v
        };
        let text = match s {
            Setting::LedBrightness => {
                let v = bump(&mut st.led_brightness, 0, 127);
                out.extend(set_feature(Feature::LedBrightness, v).map(|m| Action::Device(m.to_vec())));
                v.to_string()
            }
            Setting::ScreenBrightness => {
                let v = bump(&mut st.screen_brightness, 0, 127);
                out.extend(set_feature(Feature::ScreenBrightness, v).map(|m| Action::Device(m.to_vec())));
                v.to_string()
            }
            Setting::PopupTime => {
                let v = bump(&mut st.popup_time, 0, 99);
                out.extend(set_feature(Feature::DisplayTimeout, v).map(|m| Action::Device(m.to_vec())));
                format!("{:.1} s", (v.max(10)) as f32 / 10.0)
            }
            Setting::EncoderCurve => {
                let v = bump(&mut st.encoder_curve, 0, 2);
                out.extend(set_feature(Feature::EncoderCurve, v).map(|m| Action::Device(m.to_vec())));
                ["Slow", "Medium", "Fast"][v as usize].to_string()
            }
            Setting::PadVelocity => bump(&mut st.pad_velocity, 1, 127).to_string(),
            Setting::ListSpeed => bump(&mut st.list_speed, 1, 8).to_string(),
            Setting::FaderMode => {
                if delta != 0 {
                    st.fader_pickup = delta < 0;
                }
                if st.fader_pickup { "Pickup" } else { "Jump" }.to_string()
            }
            Setting::Clock => {
                if delta != 0 && st.host_clock != (delta < 0) {
                    st.host_clock = delta < 0;
                    out.push(Action::SeqHostClock(st.host_clock));
                }
                if st.host_clock { "Host" } else { "Internal" }.to_string()
            }
        };
        self.show(cell, text);
    }

    fn value_text(&self, cell: &Cell, value: u16, _model: &dyn Model) -> String {
        match cell.binding {
            Binding::Param(idx) => match lxr_params::info(idx) {
                Some(info) => lxr_params::format_value(info, value.min(255) as u8).trim().to_string(),
                None => value.to_string(),
            },
            _ => value.to_string(),
        }
    }

    fn show_param(&mut self, cell: &Cell, idx: u16, value: u16, model: &dyn Model) {
        let text = match lxr_params::info(idx) {
            Some(info) => lxr_params::format_value(info, value.min(255) as u8).trim().to_string(),
            None => value.to_string(),
        };
        let label = fx_label(idx, model).unwrap_or_else(|| cell.label.clone());
        self.show(&Cell { label, ..cell.clone() }, text);
    }

    fn title(&self, cell: &Cell) -> String {
        match self.mode {
            Mode::Voice(t) => format!("{} {}", TRACK_TITLES[t as usize], cell.section.label()),
            _ => cell.section.label().to_string(),
        }
    }

    fn show(&mut self, cell: &Cell, value: String) {
        let title = self.title(cell);
        self.last_touched = format!("{} {}", cell.label, value);
        self.show_coloured(&title, &cell.label.clone(), &value, cell.section.colour());
    }

    fn show_text(&mut self, title: &str, name: &str, value: &str) {
        self.show_coloured(title, name, value, Section::Edit.colour());
    }

    fn show_coloured(&mut self, title: &str, name: &str, value: &str, colour: Rgb) {
        self.record_edit(title.to_string(), name, value, colour);
        self.popup_gen = self.popup_gen.wrapping_add(1);
        self.popup = Some(Popup { gen: self.popup_gen, fields: [title.to_string(), name.to_string(), value.to_string()] });
    }

    /// The label of a control in the current mode (FX parameters are named after the FX type).
    pub fn label(&self, c: Control, model: &dyn Model) -> Option<String> {
        let cell = self.layout.get(c)?;
        Some(match cell.binding {
            Binding::Param(idx) => fx_label(idx, model).unwrap_or_else(|| cell.label.clone()),
            _ => cell.label.clone(),
        })
    }

    /// The LEDs and screen contents for the current state at time `now`.
    pub fn frame(&self, now: u64, model: &dyn Model) -> Frame {
        let mut leds = Vec::new();
        for (c, cell) in self.layout.iter() {
            if matches!(c, Control::Fader(_)) {
                continue;
            }
            let led = match c {
                Control::Button(b @ (Button::Top(_) | Button::Bottom(_))) if self.shift => self.mode_led(*b, now),
                _ => self.led(*c, cell, now, model),
            };
            leds.push((*c, led));
        }
        // Navigation buttons: lit where they do something.
        let nav: Rgb = Section::Nav.colour();
        for b in [Button::Logo, Button::MuteSelect] {
            leds.push((Control::Button(b), nav));
        }
        let view = self.seq_view();
        // Solo/Arm: sequencer view off (nav), on (white), sticky (orange)
        let solo = match (self.sticky, view) {
            (true, _) => MODE_ACTIVE,
            (false, true) => (255, 255, 255),
            (false, false) => nav,
        };
        leds.push((Control::Button(Button::SoloArm), solo));
        // Track left/right only act in the sequencer view (sequencer track); lit where they can move.
        leds.push((Control::Button(Button::TrackLeft), if view && self.seq_track > 0 { nav } else { (0, 0, 0) }));
        leds.push((Control::Button(Button::TrackRight), if view && self.seq_track < 6 { nav } else { (0, 0, 0) }));
        // Record = Clear: lit in the sequencer view, bright while held
        let rec = match (view, self.is_held(Control::Button(Button::Record))) {
            (false, _) => (0, 0, 0),
            (true, false) => (64, 0, 0),
            (true, true) => (255, 0, 0),
        };
        leds.push((Control::Button(Button::Record), rec));
        // Page up/down: kits with Shift, bars in the sequencer view
        let (up, down) = if self.shift {
            (nav, nav)
        } else if view {
            (if self.seq_bar > 0 { nav } else { (0, 0, 0) }, if self.seq_bar < 3 { nav } else { (0, 0, 0) })
        } else {
            ((0, 0, 0), (0, 0, 0))
        };
        leds.push((Control::Button(Button::PageUp), up));
        leds.push((Control::Button(Button::PageDown), down));
        // Play: red while silenced (host clock), green while running; otherwise start (mode 8,
        // internal clock) or the voice of a voice mode
        let st = model.seq_status();
        let play = if st.silent {
            (255, 0, 0)
        } else if st.running {
            (0, 255, 0)
        } else if self.mode == Mode::Sequencer {
            if self.settings.host_clock {
                (0, 0, 0)
            } else {
                (0, 64, 0)
            }
        } else if let Some(t) = self.mode.track() {
            Section::Pad(t).colour()
        } else {
            (0, 0, 0)
        };
        leds.push((Control::Button(Button::Play), play));
        if self.mode == Mode::Sequencer {
            let next = if st.pending != st.pattern { format!(" > P{:02}", st.pending + 1) } else { String::new() };
            let silent = if st.silent { " silent" } else { "" };
            let stationary = [
                format!("{} {}", self.mode.number(), self.mode.name()),
                format!("P{:02} bar {}/4{next}{silent}", st.pattern + 1, self.seq_bar + 1),
                format!("Track {} {}", self.seq_track + 1, TRACKS[self.seq_track as usize]),
            ];
            return Frame { leds, stationary, popup: self.popup.clone() };
        }
        let stationary =
            [format!("{} {}", self.mode.number(), self.mode.name()), model.kit_name(), self.last_touched.clone()];
        Frame { leds, stationary, popup: self.popup.clone() }
    }

    /// LED of step / pattern-layer button `idx` (0..16) in the sequencer view.
    fn seq_led(&self, idx: u8, model: &dyn Model) -> Rgb {
        let st = model.seq_status();
        if self.is_held(Control::Button(Button::MuteSelect)) {
            let empty = model.seq().patterns.get(idx as usize).is_none_or(|p| p.is_empty());
            return if idx == st.pattern {
                (0, 255, 0)
            } else if idx == st.pending {
                (255, 200, 0)
            } else if empty {
                (0, 0, 0)
            } else {
                (60, 60, 60)
            };
        }
        let step = self.seq_bar * 16 + idx;
        let first = self.seq_track;
        if st.running && st.step[first as usize] == step as i8 {
            return (255, 255, 255);
        }
        let d = model.seq();
        if d.step(st.pattern, first, step).on {
            let c = TRACK_COLOURS[first as usize];
            return if self.held_steps.iter().any(|h| h.step == step) { c } else { (c.0 / 2, c.1 / 2, c.2 / 2) };
        }
        if step < d.track(st.pattern, first).length {
            dim(dim(TRACK_COLOURS[first as usize]))
        } else {
            (0, 0, 0)
        }
    }

    /// While Shift is held the 16 buttons show the mode slots: the active mode pulses orange, the
    /// other modes are dim white and the empty slots are off.
    fn mode_led(&self, b: Button, now: u64) -> Rgb {
        let slot = match b {
            Button::Top(n) => n + 1,
            Button::Bottom(n) => n + 9,
            _ => return (0, 0, 0),
        };
        match Mode::from_number(slot) {
            Some(m) if m == self.mode => scale_rgb(MODE_ACTIVE, pulse(now)),
            Some(_) => MODE_OTHER,
            None => (0, 0, 0),
        }
    }

    fn led(&self, c: Control, cell: &Cell, now: u64, model: &dyn Model) -> Rgb {
        let col = cell.section.colour();
        // LFO mode: top-row buttons 1 to 7 follow their track's LFO output, scaled by its amount
        if self.mode == Mode::LfoMod && !matches!(cell.binding, Binding::SeqStep(_)) {
            if let Control::Button(Button::Top(t)) = c {
                if t < 7 {
                    return scale_rgb(col, lfo_brightness(model.lfo(t), param_frac(model, crate::layout::voice_param(t, "lfo_amt"))));
                }
            }
        }
        match cell.binding {
            Binding::Spare => (0, 0, 0),
            Binding::Param(idx) if matches!(c, Control::Button(_)) => match lxr_params::info(idx) {
                Some(info) if info.steps() == 2 => {
                    if model.param(idx) != info.min {
                        col
                    } else {
                        dim(col)
                    }
                }
                _ => col,
            },
            Binding::Pad(t) | Binding::Trigger(t) => {
                let lit = self.hit_at[t as usize].is_some_and(|at| now.saturating_sub(at) < FLASH_MS) || self.is_held(c);
                if lit {
                    (255, 255, 255)
                } else if model.mutes() >> t & 1 != 0 {
                    (96, 0, 0)
                } else {
                    dim(col)
                }
            }
            Binding::Mute(t) => {
                if model.mutes() >> t & 1 != 0 {
                    (255, 0, 0)
                } else {
                    dim(col)
                }
            }
            Binding::MuteAll => {
                if model.mutes() & 0x7f != 0 {
                    (255, 0, 0)
                } else {
                    dim(col)
                }
            }
            Binding::Moment(k) => on_off(col, self.moments.iter().any(|m| m.k == k)),
            Binding::LfoSyncHost => on_off(col, self.lfo_sync_saved.is_some()),
            Binding::Snapshot(n) => on_off(col, self.snapshots[n as usize % SLOTS].is_some()),
            Binding::Learn(n) => match &self.learning {
                Some((l, _)) if *l == n => (255, 255, 255),
                _ => on_off(col, !self.macros[n as usize % SLOTS].assign.is_empty()),
            },
            Binding::Accent => {
                if self.accent {
                    (255, 255, 255)
                } else {
                    dim(col)
                }
            }
            Binding::ToFx(t) => on_off(col, model.param(crate::layout::voice_param(t, "out")) == AUDIO_OUT_FX),
            Binding::FxOnOff => on_off(col, model.param(par::FX_TYPE) != 0),
            Binding::SumAux => on_off(col, model.switch(Switch::SumAux)),
            Binding::HwClock => on_off(col, model.switch(Switch::HwClock)),
            Binding::SeqStep(_) => match c {
                Control::Button(Button::Top(n)) => self.seq_led(n, model),
                Control::Button(Button::Bottom(n)) => self.seq_led(n + 8, model),
                _ => col,
            },
            Binding::Undo => on_off(col, !self.history.is_empty()),
            Binding::Redo => on_off(col, !self.redo.is_empty()),
            Binding::Compare => on_off(col, self.comparing),
            Binding::PasteVoice => on_off(col, self.clipboard.as_ref().is_some_and(|(g, _)| Some(*g) == self.mode.track().map(copy_group))),
            // encoders with a value: brightness follows the value, as in the device's own modes
            _ if matches!(c, Control::Encoder { .. }) => {
                let lit = match view::range_of(self, cell, model) {
                    Some((min, max, cur)) if max > min => value_brightness(col, cur.saturating_sub(min) as f32 / (max - min) as f32),
                    _ => col,
                };
                // mode 8: the selected track's knobs pulse, at their value brightness
                if self.mode == Mode::Sequencer && cell.section == Section::SeqTrack(self.seq_track) {
                    scale_rgb(lit, pulse(now))
                } else {
                    lit
                }
            }
            _ => col,
        }
    }
}

/// Lowest brightness of an encoder LED (value at its minimum), so the section colour stays visible.
pub const ENCODER_LED_FLOOR: f32 = 0.1;

/// `col` scaled by an encoder's value position `frac` (0..=1), from [`ENCODER_LED_FLOOR`] to full.
pub fn value_brightness(col: Rgb, frac: f32) -> Rgb {
    let k = ENCODER_LED_FLOOR + (1.0 - ENCODER_LED_FLOOR) * frac.clamp(0.0, 1.0);
    let f = |v: u8| (v as f32 * k).round() as u8;
    (f(col.0), f(col.1), f(col.2))
}

/// Brightness of an LFO-mode button for LFO output `lfo` (0..=1) at amount `amt` (0..=1): a steady
/// level at amount 0, swinging from [`PULSE_FLOOR`] to full at full amount.
pub fn lfo_brightness(lfo: f32, amt: f32) -> f32 {
    let amt = amt.clamp(0.0, 1.0);
    let swing = PULSE_FLOOR + (1.0 - PULSE_FLOOR) * lfo.clamp(0.0, 1.0);
    (1.0 - amt) * LFO_STEADY + amt * swing
}

/// Level of an LFO-mode button whose LFO amount is 0.
pub const LFO_STEADY: f32 = 0.6;

/// A parameter's value as a fraction of its range.
fn param_frac(model: &dyn Model, idx: u16) -> f32 {
    match lxr_params::info(idx) {
        Some(i) if i.max > i.min => model.param(idx).saturating_sub(i.min) as f32 / (i.max - i.min) as f32,
        _ => 0.0,
    }
}

/// Period of the selected track's knob LED pulse in mode 8, in milliseconds.
pub const PULSE_MS: u64 = 1200;
/// Lowest level of the pulse (fraction of the value brightness).
pub const PULSE_FLOOR: f32 = 0.08;

/// Pulse level at time `now`: a raised cosine from [`PULSE_FLOOR`] to 1 and back over [`PULSE_MS`].
/// Shift held: colour of the active mode's button (pulsing) and of the other modes' buttons.
pub const MODE_ACTIVE: Rgb = (255, 80, 0);
pub const MODE_OTHER: Rgb = (40, 40, 40);

pub fn pulse(now: u64) -> f32 {
    let phase = (now % PULSE_MS) as f32 / PULSE_MS as f32;
    let up = 0.5 - 0.5 * (phase * std::f32::consts::TAU).cos();
    PULSE_FLOOR + (1.0 - PULSE_FLOOR) * up
}

fn scale_rgb(col: Rgb, k: f32) -> Rgb {
    let f = |v: u8| (v as f32 * k).round() as u8;
    (f(col.0), f(col.1), f(col.2))
}

/// Name of a lockable parameter (short).
fn lock_name(idx: u8) -> String {
    lxr_params::info(idx as u16).map(|i| i.name.to_string()).unwrap_or_else(|| idx.to_string())
}

fn on_off(col: Rgb, on: bool) -> Rgb {
    if on {
        col
    } else {
        dim(col)
    }
}

/// The name of FX parameter 1..4 for the current FX type (they mean different things per type).
fn fx_label(idx: u16, model: &dyn Model) -> Option<String> {
    if ![par::FX_P1, par::FX_P2, par::FX_P3, par::FX_P4].contains(&idx) {
        return None;
    }
    let text = lxr_params::fx_param_text(model.param(par::FX_TYPE).min(255) as u8, idx)?;
    let (_, _, long) = lxr_params::text_names(text);
    Some(format!("FX {}", long.trim()))
}
