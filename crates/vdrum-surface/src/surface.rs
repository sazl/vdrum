// SPDX-License-Identifier: GPL-3.0-only
//! The surface state machine: decoded controller input in, plugin actions and display frames out.

use crate::frame::{dim, Frame, Popup};
use crate::layout::{Binding, Cell, Layout, Mode, Rgb, Section, Setting, TRACK_TITLES};
use crate::model::{options, scale, unscale, Action, Model, FADER_MAX};
use lcxl3::{Button, Control, Input};

/// Hold time for "hold 1 s" actions, in milliseconds.
pub const HOLD_MS: u64 = 1000;
/// How long a pad stays lit after its track was triggered, in milliseconds.
pub const FLASH_MS: u64 = 120;

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
        }
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
                if new != cur {
                    out.push(Action::SetParam { idx, value: new });
                }
                self.show_param(cell, idx, new);
            }
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
                if value != cur {
                    out.push(Action::SetParam { idx, value });
                }
                self.show_param(cell, idx, value);
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
            Button::Play => {
                if pressed {
                    if let Some(track) = self.mode.track() {
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
                out.push(Action::SetParam { idx, value: new });
                self.show_param(cell, idx, new);
            }
            Binding::Pad(track) => out.push(Action::Trigger { track, velocity: self.pad_velocity() }),
            Binding::Accent => self.accent = true,
            _ => {}
        }
    }

    fn release(&mut self, cell: &Cell, _model: &dyn Model, _out: &mut [Action]) {
        if cell.binding == Binding::Accent {
            self.accent = false;
        }
    }

    fn hold_fired(&mut self, _cell: &Cell, _model: &dyn Model, _out: &mut Vec<Action>) {}

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

    fn show_param(&mut self, cell: &Cell, idx: u16, value: u16) {
        let text = match lxr_params::info(idx) {
            Some(info) => lxr_params::format_value(info, value.min(255) as u8).trim().to_string(),
            None => value.to_string(),
        };
        self.show(cell, text);
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
            _ => col,
        }
    }
}
