// SPDX-License-Identifier: GPL-3.0-only
//! The surface state machine: decoded controller input in, plugin actions and display frames out.

use crate::frame::{dim, Frame, Popup};
use crate::layout::{Binding, Cell, Layout, Mode, Rgb, Section, Setting, TRACK_TITLES};
use crate::model::{options, scale, unscale, Action, Model, Switch, FADER_MAX};
use lxr_params::par;
use lcxl3::{Button, Control, Input};

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
        }
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
        self.layout = Layout::of(mode, false);
        for f in &mut self.faders {
            f.picked = false;
        }
        self.accum = [0; 24];
        self.held.retain(|h| matches!(h.control, Control::Button(Button::Shift)));
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
            _ => {}
        }
    }

    fn fader(&mut self, col: u8, cell: &Cell, raw: u8, model: &dyn Model, out: &mut Vec<Action>) {
        let st = self.faders[col as usize];
        let (min, max, cur) = match cell.binding {
            Binding::Param(idx) => match lxr_params::info(idx) {
                Some(info) => (info.min, info.max, model.param(idx)),
                None => return,
            },
            Binding::VoiceFader(t) => (0, FADER_MAX as u16, model.voice_fader(t).clamp(0, FADER_MAX) as u16),
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
            self.held.retain(|h| h.control != c);
        }
        match button {
            Button::Shift => self.shift = pressed,
            Button::Top(n) | Button::Bottom(n) if self.shift => {
                if pressed {
                    let slot = if matches!(button, Button::Top(_)) { n + 1 } else { n + 9 };
                    match Mode::from_number(slot) {
                        Some(m) => self.set_mode(m),
                        None => self.show_text("Mode", &format!("Slot {slot}"), "empty"),
                    }
                }
            }
            Button::TrackLeft | Button::TrackRight => {
                if pressed {
                    self.set_mode(self.mode.step(if button == Button::TrackLeft { -1 } else { 1 }));
                }
            }
            Button::Logo => {
                if pressed {
                    self.set_mode(self.prev_mode);
                }
            }
            Button::SoloArm => {
                if pressed {
                    self.set_mode(Mode::Sequencer);
                }
            }
            Button::MuteSelect => {
                if pressed {
                    self.set_mode(Mode::Perform);
                }
            }
            Button::PageUp | Button::PageDown => {
                if pressed {
                    self.browsing = true;
                    out.push(Action::KitBrowse(if button == Button::PageUp { -1 } else { 1 }));
                }
            }
            Button::Play => {
                if pressed {
                    if self.browsing {
                        self.browsing = false;
                        out.push(Action::KitLoadBrowsed);
                    } else if let Some(track) = self.mode.track() {
                        out.push(Action::Trigger { track, velocity: self.pad_velocity() });
                    }
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
            _ => {}
        }
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
            Binding::Pad(track) => out.push(Action::Trigger { track, velocity: self.pad_velocity() }),
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
            Binding::Undo => self.undo(false, out),
            Binding::Redo => self.undo(true, out),
            Binding::Panic => {
                out.push(Action::Panic);
                self.show(cell, "all voices off".to_string());
            }
            _ => {}
        }
    }

    fn release(&mut self, cell: &Cell, _model: &dyn Model, out: &mut Vec<Action>) {
        match cell.binding {
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
                if delta != 0 {
                    st.host_clock = delta < 0;
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
        self.show_text(&title, &cell.label.clone(), &value);
    }

    fn show_text(&mut self, title: &str, name: &str, value: &str) {
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
            leds.push((*c, self.led(*c, cell, now, model)));
        }
        // Navigation buttons: lit where they do something.
        let nav: Rgb = Section::Nav.colour();
        for b in [Button::TrackLeft, Button::TrackRight, Button::Logo, Button::SoloArm, Button::MuteSelect] {
            leds.push((Control::Button(b), nav));
        }
        let play = if self.mode.track().is_some() { Section::Pad(self.mode.track().unwrap_or(0)).colour() } else { (0, 0, 0) };
        leds.push((Control::Button(Button::Play), play));
        let stationary =
            [format!("{} {}", self.mode.number(), self.mode.name()), model.kit_name(), self.last_touched.clone()];
        Frame { leds, stationary, popup: self.popup.clone() }
    }

    fn led(&self, c: Control, cell: &Cell, now: u64, model: &dyn Model) -> Rgb {
        let col = cell.section.colour();
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
            Binding::Pad(t) => {
                let lit = self.hit_at[t as usize].is_some_and(|at| now.saturating_sub(at) < FLASH_MS) || self.is_held(c);
                if lit {
                    (255, 255, 255)
                } else {
                    dim(col)
                }
            }
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
            Binding::Undo => on_off(col, !self.history.is_empty()),
            Binding::Redo => on_off(col, !self.redo.is_empty()),
            Binding::Compare => on_off(col, self.comparing),
            Binding::PasteVoice => on_off(col, self.clipboard.as_ref().is_some_and(|(g, _)| Some(*g) == self.mode.track().map(copy_group))),
            _ => col,
        }
    }
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
