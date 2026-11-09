// SPDX-License-Identifier: GPL-3.0-only
//! Move's controls on VDrum's surface ([`vdrum_surface::Surface`], designed for the Launch Control
//! XL 3): Move's messages become the surface's inputs, and the surface's LEDs and screen become
//! Move's LED frame and display lines. Every mode and feature of the surface works the same way;
//! only the controls they sit on differ.
//!
//! | Move | Surface |
//! |---|---|
//! | Track buttons 1–3 | Knob bank: the 8 knobs are encoder row 1, 2 or 3 |
//! | Track button 4 | Knob bank: the 8 knobs move faders 1–8 (absolute, no pickup) |
//! | Pads, bottom row 1–7 | Play voices 1–7 (on the audio thread); pad 8 = accent (velocity 127) |
//! | Mute + bottom-row pad | Mute / unmute the track; Shift + Mute + pad: solo |
//! | Pads, row 2 and 3 | Bottom and top button rows; in the sequencer view: patterns 9–16 and 1–8 |
//! | Pads, top row 1–7 | Select the sequencer track (and the voice mode, in a voice mode) |
//! | Step buttons | Steps (sequencer view); otherwise mode 1–16; Shift + step: mode 1–16 |
//! | Shift | Shift |
//! | Play | Play |
//! | Menu | Global mode; again: back to the previous mode |
//! | Loop | Solo/Arm (sequencer view; Shift: sticky) |
//! | Delete | Record (clear: hold + step, or + Up/Down) |
//! | Left / Right | Page up / down (bars; Shift: kits) |
//! | Up / Down | Track right / left |
//! | Undo | Undo; Shift + Undo: redo |
//! | Copy | Copy voice; Shift + Copy: paste voice (hold 1 s) |
//! | Capture | Save the kit, patterns and session |
//! | Jog wheel | Browse kits; click (or Play) loads |
//! | Shift + jog wheel | Tempo |
//!
//! Schwung keeps Back for itself: Back puts the module in the background (it keeps playing),
//! Shift + Back quits it.

use crate::hw::{self, Led};
use crate::shared::{In, SRC_JS};
use lcxl3::{Button, Control, Input};
use lxr_params::par;
use vdrum_surface::layout::{Binding, Rgb, TRACK_COLOURS};
use vdrum_surface::model::{scale, unscale};
use vdrum_surface::{frame::dim, Action, Frame, Mode, Model, Surface, MODE_ACTIVE, MODE_OTHER};

/// How long a popup stays on the display, in milliseconds.
pub const POPUP_MS: u64 = 1500;
/// How long a voice pad stays bright after its track played, in milliseconds.
pub const FLASH_MS: u64 = 120;
/// Number of display lines.
pub const LINES: usize = 5;
/// Characters per display line (Schwung's 6 pixel font on 128 pixels).
pub const LINE_CHARS: usize = 21;

/// What the adapter asks of the worker besides the surface's actions.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Out {
    Act(Action),
    /// The accent pad was pressed or released (`ui.js` path; the audio thread handles its own).
    Accent(bool),
    /// Save the kit, the patterns and the session.
    Save,
}

/// A pad or step pressed on the surface: what its release must go to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Pressed {
    Button(Button),
    /// A pattern button (Mute/Select held by the adapter).
    Pattern(Button),
    Nothing,
}

pub struct Adapter {
    pub surface: Surface,
    /// Knob bank 0..4 (encoder rows 0..3, faders).
    bank: u8,
    mute_held: bool,
    /// Pattern buttons held (the adapter holds Mute/Select while any is).
    patterns_held: u8,
    /// What each pad (0..32) and step button (32..48) press went to.
    pressed: [Pressed; 48],
    /// Accent pad held (`ui.js` path).
    accent: bool,
    hit_seen: [u32; 7],
    hit_at: [Option<u64>; 7],
    /// Popup generation and the time it was first seen.
    popup: Option<(u32, u64)>,
}

impl Default for Adapter {
    fn default() -> Self {
        Adapter::new()
    }
}

fn button_of(i: u8) -> Button {
    if i < 8 {
        Button::Top(i)
    } else {
        Button::Bottom(i - 8)
    }
}

/// White LED brightness for a surface colour.
fn white(c: Rgb) -> u8 {
    match c.0.max(c.1).max(c.2) {
        0..=7 => hw::WHITE_OFF,
        8..=99 => hw::WHITE_DIM,
        100..=199 => hw::WHITE_MEDIUM,
        _ => hw::WHITE_BRIGHT,
    }
}

fn led_of(frame: &Frame, c: Control) -> Rgb {
    frame.leds.iter().find(|(k, _)| *k == c).map_or((0, 0, 0), |(_, v)| *v)
}

fn fit(s: &str) -> String {
    s.chars().filter(|c| !c.is_control()).take(LINE_CHARS).collect()
}

impl Adapter {
    /// A surface for Move: faders jump (Move's knobs set them relative to the current value) and
    /// the sequencer runs on its own clock.
    pub fn new() -> Adapter {
        let mut surface = Surface::new();
        Adapter::fix_settings(&mut surface);
        Adapter {
            surface,
            bank: 0,
            mute_held: false,
            patterns_held: 0,
            pressed: [Pressed::Nothing; 48],
            accent: false,
            hit_seen: [0; 7],
            hit_at: [None; 7],
            popup: None,
        }
    }

    /// Settings that Move's controls need, whatever was saved.
    pub fn fix_settings(surface: &mut Surface) {
        surface.settings.fader_pickup = false;
        surface.settings.host_clock = false;
    }

    pub fn bank(&self) -> u8 {
        self.bank
    }

    pub fn mute_held(&self) -> bool {
        self.mute_held
    }

    fn input(&mut self, input: Input, now: u64, model: &dyn Model, out: &mut Vec<Out>) {
        out.extend(self.surface.handle(&input, now, model).into_iter().map(Out::Act));
    }

    fn press(&mut self, b: Button, pressed: bool, now: u64, model: &dyn Model, out: &mut Vec<Out>) {
        self.input(Input::Button { button: b, pressed }, now, model, out);
    }

    /// Press and release a button.
    fn click(&mut self, b: Button, now: u64, model: &dyn Model, out: &mut Vec<Out>) {
        self.press(b, true, now, model, out);
        self.press(b, false, now, model, out);
    }

    /// The control that has `binding` in the current mode.
    fn find(&self, binding: Binding) -> Option<Button> {
        self.surface.layout().iter().find_map(|(c, cell)| match c {
            Control::Button(b) if cell.binding == binding => Some(*b),
            _ => None,
        })
    }

    /// The fader position (0..=127) that moves fader `col`'s value by `delta` units (at least one
    /// value, whatever the range), or `None` if the fader is unassigned or at the end.
    fn fader_position(&self, col: u8, delta: i32, now: u64, model: &dyn Model) -> Option<u8> {
        let c = Control::Fader(col);
        let cell = self.surface.layout().get(c)?;
        let (min, max) = match cell.binding {
            Binding::Param(idx) => lxr_params::info(idx).map(|i| (i.min, i.max))?,
            Binding::VoiceFader(_) => (0, vdrum_surface::model::FADER_MAX as u16),
            Binding::Macro(_) => (0, 127),
            Binding::LfoDepthScale => (0, 200),
            Binding::SeqDensity(_) | Binding::SeqDensityAll => (0, 100),
            _ => return None,
        };
        let frac = self.surface.view(now, model).control(c).and_then(|v| v.frac)?;
        let cur = min as i32 + (frac * (max - min) as f32).round() as i32;
        let target = (cur + delta).clamp(min as i32, max as i32) as u16;
        if target as i32 == cur {
            return None;
        }
        // the position for the target value, moved on until it gives a different value than now
        let mut raw = unscale(min, max, target) as i32;
        let dir = delta.signum();
        while (0..=127).contains(&raw) && scale(min, max, raw as u8) as i32 == cur {
            raw += dir;
        }
        (0..=127).contains(&raw).then_some(raw as u8)
    }

    /// Handle one message from Move's controls at `now` (milliseconds).
    pub fn handle(&mut self, m: In, now: u64, model: &dyn Model) -> Vec<Out> {
        let mut out = Vec::new();
        let kind = m[0] & 0xf0;
        match kind {
            0x80 | 0x90 => {
                let on = kind == 0x90 && m[2] > 0;
                if let Some(pad) = hw::pad_index(m[1]) {
                    self.pad(pad, on, m[2], m[3] == SRC_JS, now, model, &mut out);
                } else if let Some(step) = hw::step_index(m[1]) {
                    self.step(step, on, now, model, &mut out);
                }
            }
            0xb0 => self.cc(m[1], m[2], now, model, &mut out),
            _ => {}
        }
        out
    }

    #[allow(clippy::too_many_arguments)]
    fn pad(&mut self, pad: u8, on: bool, velocity: u8, from_js: bool, now: u64, model: &dyn Model, out: &mut Vec<Out>) {
        let (row, col) = (pad / 8, pad % 8);
        match row {
            0 => {
                if self.mute_held && on && col < 7 {
                    let bit = 1u8 << col;
                    let mutes = model.mutes();
                    let new = if self.surface.shift() {
                        // solo: only this track plays; soloing it again unmutes all
                        let solo = 0x7f & !bit;
                        if mutes == solo {
                            0
                        } else {
                            solo
                        }
                    } else {
                        mutes ^ bit
                    };
                    out.push(Out::Act(Action::SetMutes(new)));
                    let state = if new & bit != 0 { "muted" } else { "playing" };
                    let name = vdrum_surface::layout::TRACK_TITLES[col as usize];
                    self.surface.notify(if self.surface.shift() { "Solo" } else { "Mute" }, name, state);
                } else if from_js && !self.mute_held {
                    // fallback when pads do not reach the DSP directly
                    if col == hw::ACCENT_PAD {
                        self.accent = on;
                        out.push(Out::Accent(on));
                    } else if on {
                        let velocity = if self.accent { 127 } else { velocity.max(1) };
                        out.push(Out::Act(Action::Trigger { track: col, velocity }));
                    }
                }
            }
            1 | 2 => {
                let b = if row == 2 { Button::Top(col) } else { Button::Bottom(col) };
                let i = pad as usize;
                if on {
                    if self.surface.seq_view() && !self.surface.shift() {
                        if self.patterns_held == 0 {
                            self.press(Button::MuteSelect, true, now, model, out);
                        }
                        self.patterns_held += 1;
                        self.press(b, true, now, model, out);
                        self.pressed[i] = Pressed::Pattern(b);
                    } else {
                        self.press(b, true, now, model, out);
                        self.pressed[i] = Pressed::Button(b);
                    }
                } else {
                    match std::mem::replace(&mut self.pressed[i], Pressed::Nothing) {
                        Pressed::Pattern(b) => {
                            self.press(b, false, now, model, out);
                            self.patterns_held = self.patterns_held.saturating_sub(1);
                            if self.patterns_held == 0 {
                                self.press(Button::MuteSelect, false, now, model, out);
                            }
                        }
                        Pressed::Button(b) => self.press(b, false, now, model, out),
                        Pressed::Nothing => {}
                    }
                }
            }
            _ => {
                if on && col < 7 {
                    self.surface.select_track(col);
                    if matches!(self.surface.mode(), Mode::Voice(_)) {
                        self.surface.set_mode(Mode::Voice(col));
                    }
                }
            }
        }
    }

    fn step(&mut self, step: u8, on: bool, now: u64, model: &dyn Model, out: &mut Vec<Out>) {
        let i = 32 + step as usize;
        let b = button_of(step);
        if on {
            if self.surface.seq_view() || self.surface.shift() {
                self.press(b, true, now, model, out);
                self.pressed[i] = Pressed::Button(b);
            } else {
                match Mode::from_number(step + 1) {
                    Some(m) => self.surface.set_mode(m),
                    None => self.surface.notify("Mode", &format!("Slot {}", step + 1), "empty"),
                }
            }
        } else if let Pressed::Button(b) = std::mem::replace(&mut self.pressed[i], Pressed::Nothing) {
            self.press(b, false, now, model, out);
        }
    }

    fn cc(&mut self, cc: u8, v: u8, now: u64, model: &dyn Model, out: &mut Vec<Out>) {
        let pressed = v > 0;
        let map = |cc: u8| match cc {
            hw::CC_SHIFT => Some(Button::Shift),
            hw::CC_PLAY => Some(Button::Play),
            hw::CC_LOOP => Some(Button::SoloArm),
            hw::CC_DELETE => Some(Button::Record),
            hw::CC_LEFT => Some(Button::PageUp),
            hw::CC_RIGHT => Some(Button::PageDown),
            hw::CC_UP => Some(Button::TrackRight),
            hw::CC_DOWN => Some(Button::TrackLeft),
            _ => None,
        };
        if let Some(b) = map(cc) {
            self.press(b, pressed, now, model, out);
            return;
        }
        match cc {
            hw::CC_KNOB1..=78 => {
                let col = cc - hw::CC_KNOB1;
                let delta = hw::delta(v);
                if delta == 0 {
                    return;
                }
                if self.bank < 3 {
                    let delta = delta.clamp(-63, 63) as i8;
                    self.input(Input::Relative { row: self.bank, col, delta }, now, model, out);
                } else if let Some(pos) = self.fader_position(col, delta, now, model) {
                    self.input(Input::Analogue { control: Control::Fader(col), value: pos }, now, model, out);
                }
            }
            hw::CC_TRACK4..=hw::CC_TRACK1 => {
                if pressed {
                    self.bank = hw::CC_TRACK1 - cc;
                    let name = match self.bank {
                        3 => "Faders".to_string(),
                        b => format!("Knob row {}", b + 1),
                    };
                    self.surface.notify("Knobs", &name, "");
                }
            }
            hw::CC_MUTE => self.mute_held = pressed,
            hw::CC_MENU => {
                if pressed {
                    if self.surface.mode() == Mode::Global {
                        self.click(Button::Logo, now, model, out);
                    } else {
                        self.surface.set_mode(Mode::Global);
                    }
                }
            }
            hw::CC_UNDO => {
                if pressed {
                    let redo = self.surface.shift();
                    out.extend(self.surface.undo_redo(redo, now).into_iter().map(Out::Act));
                }
            }
            hw::CC_COPY => {
                let binding = if self.surface.shift() { Binding::PasteVoice } else { Binding::CopyVoice };
                match self.find(binding) {
                    // Shift is held for paste: the surface would take Shift + button as a mode
                    // change, so Shift is let go for the press and taken again after.
                    Some(b) => {
                        let shift = self.surface.shift();
                        if shift {
                            self.press(Button::Shift, false, now, model, out);
                        }
                        self.press(b, pressed, now, model, out);
                        if shift {
                            self.press(Button::Shift, true, now, model, out);
                        }
                    }
                    None if pressed => self.surface.notify("Copy voice", "in a voice mode", ""),
                    None => {}
                }
            }
            hw::CC_CAPTURE => {
                if pressed {
                    out.push(Out::Save);
                }
            }
            hw::CC_WHEEL => {
                let delta = hw::delta(v);
                if delta == 0 {
                    return;
                }
                if self.surface.shift() {
                    let info = lxr_params::info(par::BPM);
                    let (lo, hi) = info.map_or((20, 250), |i| (i.min.max(20), i.max));
                    let bpm = (model.param(par::BPM) as i32 + delta).clamp(lo as i32, hi as i32) as u16;
                    out.push(Out::Act(Action::SetParam { idx: par::BPM, value: bpm }));
                    self.surface.notify("Tempo", "BPM", &bpm.to_string());
                } else {
                    let page = if delta < 0 { Button::PageUp } else { Button::PageDown };
                    self.press(Button::Shift, true, now, model, out);
                    for _ in 0..delta.unsigned_abs().min(4) {
                        self.click(page, now, model, out);
                    }
                    self.press(Button::Shift, false, now, model, out);
                }
            }
            hw::CC_WHEEL_CLICK if pressed && self.surface.browsing() => self.click(Button::Play, now, model, out),
            _ => {}
        }
    }

    /// Advance time (hold actions, momentary effects, pad flashes).
    pub fn tick(&mut self, now: u64, model: &dyn Model) -> Vec<Out> {
        for t in 0..7u8 {
            let h = model.hits(t);
            if h != self.hit_seen[t as usize] {
                self.hit_seen[t as usize] = h;
                self.hit_at[t as usize] = Some(now);
            }
        }
        self.surface.tick(now, model).into_iter().map(Out::Act).collect()
    }

    /// The colour of every LED, in [`hw::led`] order: palette indices for RGB LEDs, brightness for
    /// white ones. `accent` is the accent pad's state.
    pub fn leds(&mut self, now: u64, model: &dyn Model, accent: bool) -> [u8; hw::NUM_LEDS] {
        let frame = self.surface.frame(now, model);
        let mut rgb = [(0u8, 0u8, 0u8); hw::NUM_LEDS];
        let st = model.seq_status();
        let mutes = model.mutes();
        let view = self.surface.seq_view();
        let shift = self.surface.shift();
        // voice pads
        for t in 0..7 {
            let col = TRACK_COLOURS[t];
            let muted = mutes & (1 << t) != 0;
            let flash = self.hit_at[t].is_some_and(|at| now.saturating_sub(at) < FLASH_MS);
            rgb[t] = if self.mute_held {
                if muted {
                    (255, 0, 0)
                } else {
                    col
                }
            } else if muted {
                (40, 0, 0)
            } else if flash {
                col
            } else {
                dim(col)
            };
        }
        rgb[hw::ACCENT_PAD as usize] = if accent || self.accent { (255, 255, 255) } else { (40, 40, 40) };
        // button rows / patterns
        for col in 0..8u8 {
            for (row, b) in [(1usize, Button::Bottom(col)), (2, Button::Top(col))] {
                let i = row * 8 + col as usize;
                rgb[i] = if view && !shift {
                    let p = if row == 2 { col } else { col + 8 };
                    let filled = model.seq().patterns.get(p as usize).is_some_and(|d| !d.is_empty());
                    if p == st.pattern {
                        (255, 255, 255)
                    } else if p == st.pending {
                        MODE_ACTIVE
                    } else if filled {
                        (60, 140, 60)
                    } else {
                        (0, 0, 0)
                    }
                } else {
                    led_of(&frame, Control::Button(b))
                };
            }
        }
        // track select
        let sel = self.surface.seq_track() as usize;
        for t in 0..7 {
            rgb[24 + t] = if t == sel { TRACK_COLOURS[t] } else { dim(dim(TRACK_COLOURS[t])) };
        }
        // step buttons: steps, or modes
        for s in 0..16u8 {
            let b = button_of(s);
            rgb[hw::LED_STEPS + s as usize] = if view || shift {
                led_of(&frame, Control::Button(b))
            } else {
                match Mode::from_number(s + 1) {
                    Some(m) if m == self.surface.mode() => MODE_ACTIVE,
                    Some(_) => MODE_OTHER,
                    None => (0, 0, 0),
                }
            };
        }
        // knob banks
        for b in 0..4 {
            rgb[hw::LED_TRACKS + b] = if b == self.bank as usize { (255, 255, 255) } else { (30, 30, 30) };
        }
        let mut out = [0u8; hw::NUM_LEDS];
        for (i, o) in out.iter_mut().enumerate() {
            *o = match hw::led(i) {
                Led::Note(_) => hw::palette_index(rgb[i]),
                Led::RgbCc(cc) | Led::WhiteCc(cc) => self.button_led(cc, &frame, rgb[i], shift),
            };
        }
        out
    }

    fn button_led(&self, cc: u8, frame: &Frame, own: Rgb, shift: bool) -> u8 {
        let f = |b: Button| led_of(frame, Control::Button(b));
        let dimmed = hw::WHITE_DIM;
        match cc {
            hw::CC_TRACK4..=hw::CC_TRACK1 => hw::palette_index(own),
            hw::CC_PLAY => hw::palette_index(f(Button::Play)),
            hw::CC_REC | hw::CC_SAMPLE => 0,
            hw::CC_SHIFT => if shift { hw::WHITE_BRIGHT } else { dimmed },
            hw::CC_MUTE => if self.mute_held { hw::WHITE_BRIGHT } else { dimmed },
            hw::CC_LOOP => white(f(Button::SoloArm)),
            hw::CC_DELETE => white(f(Button::Record)),
            hw::CC_LEFT => white(f(Button::PageUp)),
            hw::CC_RIGHT => white(f(Button::PageDown)),
            hw::CC_UP => white(f(Button::TrackRight)),
            hw::CC_DOWN => white(f(Button::TrackLeft)),
            hw::CC_COPY => {
                if self.find(Binding::CopyVoice).is_some() {
                    dimmed
                } else {
                    0
                }
            }
            _ => dimmed,
        }
    }

    /// The display: mode and knob bank, three lines of the surface's screen (or its popup), and
    /// the transport.
    pub fn lines(&mut self, now: u64, model: &dyn Model) -> [String; LINES] {
        let frame = self.surface.frame(now, model);
        let mode = self.surface.mode();
        let bank = match self.bank {
            3 => "FDR".to_string(),
            b => format!("K{}", b + 1),
        };
        let title = format!("{} {}", mode.number(), mode.name());
        let head = format!("{:<w$}{bank}", fit(&title), w = LINE_CHARS - bank.len());
        let fields = match &frame.popup {
            Some(p) => {
                let at = match self.popup {
                    Some((gen, at)) if gen == p.gen => at,
                    _ => {
                        self.popup = Some((p.gen, now));
                        now
                    }
                };
                if now.saturating_sub(at) < POPUP_MS {
                    p.fields.clone()
                } else {
                    frame.stationary.clone()
                }
            }
            None => frame.stationary.clone(),
        };
        let st = model.seq_status();
        let run = if st.running { "PLAY" } else { "STOP" };
        let next = if st.pending != st.pattern { format!(">{:02}", st.pending + 1) } else { String::new() };
        let mutes = model.mutes();
        let m: String = (0..7).map(|t| if mutes & (1 << t) != 0 { '-' } else { '*' }).collect();
        let status = format!("{:>3} P{:02}{next} {run} {m}", model.tempo().round() as i32, st.pattern + 1);
        [head, fit(&fields[0]), fit(&fields[1]), fit(&fields[2]), fit(&status)]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{apply, SharedModel};
    use crate::shared::{Command, Shared};

    fn cc(cc: u8, v: u8) -> In {
        [0xb0, cc, v, SRC_JS]
    }

    fn note(n: u8, on: bool) -> In {
        [if on { 0x90 } else { 0x80 }, n, if on { 100 } else { 0 }, 0]
    }

    /// Feed messages and carry out the actions on the shared state (no audio thread).
    fn run(a: &mut Adapter, s: &Shared, msgs: &[In], now: u64) -> Vec<Out> {
        let mut all = Vec::new();
        for m in msgs {
            let out = a.handle(*m, now, &SharedModel(s));
            for o in &out {
                if let Out::Act(act) = o {
                    apply(s, act.clone());
                }
            }
            all.extend(out);
        }
        all
    }

    fn params(s: &Shared) -> Vec<(u16, u16)> {
        let mut v = Vec::new();
        while let Some(c) = s.commands.pop() {
            if let Command::Param(i, x) = c {
                v.push((i, x));
            }
        }
        v
    }

    #[test]
    fn knobs_follow_the_selected_bank() {
        let s = Shared::new();
        let mut a = Adapter::new();
        // bank 1 (top encoder row) of mode 1: a knob moves a parameter
        run(&mut a, &s, &[cc(hw::CC_KNOB1, 1)], 0);
        let p1 = params(&s);
        assert_eq!(p1.len(), 1, "{p1:?}");
        // bank 2: another parameter
        run(&mut a, &s, &[cc(hw::CC_TRACK1 - 1, 127), cc(hw::CC_KNOB1, 1)], 10);
        assert_eq!(a.bank(), 1);
        let p2 = params(&s);
        assert_eq!(p2.len(), 1);
        assert_ne!(p1[0].0, p2[0].0);
    }

    #[test]
    fn fader_bank_moves_faders_absolutely() {
        let s = Shared::new();
        let mut a = Adapter::new();
        run(&mut a, &s, &[cc(hw::CC_TRACK4, 127)], 0);
        assert_eq!(a.bank(), 3);
        // every bound fader of every mode moves by one value per detent, both ways
        for m in Mode::ALL {
            a.surface.set_mode(m);
            for col in 0..8u8 {
                let Some(cell) = a.surface.layout().get(Control::Fader(col)).cloned() else { continue };
                let read = |a: &Adapter, s: &Shared| {
                    a.surface.view(0, &SharedModel(s)).control(Control::Fader(col)).and_then(|v| v.value.clone())
                };
                let before = read(&a, &s);
                if before.is_none() {
                    continue;
                }
                run(&mut a, &s, &[cc(hw::CC_KNOB1 + col, 1)], 10);
                let up = read(&a, &s);
                run(&mut a, &s, &[cc(hw::CC_KNOB1 + col, 127)], 20);
                let down = read(&a, &s);
                assert!(before != up || before != down, "{m:?} fader {col} {}: {before:?} {up:?} {down:?}", cell.label);
            }
        }
    }

    #[test]
    fn steps_select_modes_and_edit_steps_in_the_sequencer() {
        let s = Shared::new();
        let mut a = Adapter::new();
        run(&mut a, &s, &[note(hw::STEP_BASE + 7, true), note(hw::STEP_BASE + 7, false)], 0);
        assert_eq!(a.surface.mode(), Mode::Sequencer);
        run(&mut a, &s, &[note(hw::STEP_BASE + 2, true), note(hw::STEP_BASE + 2, false)], 10);
        assert!(s.seq.lock().unwrap().step(0, 0, 2).on);
        // a pattern pad: pattern 3 is queued (stopped: at once)
        let out = run(&mut a, &s, &[note(hw::PAD_BASE + 16 + 2, true), note(hw::PAD_BASE + 16 + 2, false)], 20);
        assert!(out.iter().any(|o| matches!(o, Out::Act(Action::Seq(vdrum_surface::SeqEdit::NextPattern(2))))), "{out:?}");
        // Mute/Select was let go after the pattern layer: still in mode 8
        assert_eq!(a.surface.mode(), Mode::Sequencer);
        // Shift + step: mode 1
        run(&mut a, &s, &[cc(hw::CC_SHIFT, 127), note(hw::STEP_BASE, true), note(hw::STEP_BASE, false), cc(hw::CC_SHIFT, 0)], 30);
        assert_eq!(a.surface.mode(), Mode::Voice(0));
    }

    #[test]
    fn mute_and_solo_with_the_voice_pads() {
        let s = Shared::new();
        let mut a = Adapter::new();
        run(&mut a, &s, &[cc(hw::CC_MUTE, 127), note(hw::PAD_BASE + 2, true), note(hw::PAD_BASE + 2, false)], 0);
        assert_eq!(SharedModel(&s).mutes(), 0b100);
        run(&mut a, &s, &[cc(hw::CC_SHIFT, 127), note(hw::PAD_BASE + 1, true), cc(hw::CC_SHIFT, 0), cc(hw::CC_MUTE, 0)], 10);
        assert_eq!(SharedModel(&s).mutes(), 0x7f & !0b10);
        assert!(!a.mute_held());
    }

    #[test]
    fn js_pads_trigger_with_accent() {
        let s = Shared::new();
        let mut a = Adapter::new();
        let js = |n: u8, on: bool| [if on { 0x90 } else { 0x80 }, n, if on { 80 } else { 0 }, SRC_JS];
        let out = run(&mut a, &s, &[js(hw::PAD_BASE + 3, true)], 0);
        assert_eq!(out, vec![Out::Act(Action::Trigger { track: 3, velocity: 80 })]);
        let out = run(&mut a, &s, &[js(hw::PAD_BASE + 7, true), js(hw::PAD_BASE + 3, true)], 0);
        assert_eq!(out, vec![Out::Accent(true), Out::Act(Action::Trigger { track: 3, velocity: 127 })]);
        // direct pads are played by the audio thread, not here
        assert!(run(&mut a, &s, &[note(hw::PAD_BASE + 3, true)], 0).is_empty());
    }

    #[test]
    fn wheel_browses_and_shift_wheel_sets_tempo() {
        let s = Shared::new();
        let mut a = Adapter::new();
        let out = run(&mut a, &s, &[cc(hw::CC_WHEEL, 1)], 0);
        assert!(out.contains(&Out::Act(Action::KitBrowse(1))), "{out:?}");
        assert!(a.surface.browsing());
        assert!(!a.surface.shift());
        let out = run(&mut a, &s, &[cc(hw::CC_WHEEL_CLICK, 127)], 5);
        assert!(out.contains(&Out::Act(Action::KitLoadBrowsed)), "{out:?}");
        s.values[par::BPM as usize].store(120, std::sync::atomic::Ordering::Relaxed);
        let out = run(&mut a, &s, &[cc(hw::CC_SHIFT, 127), cc(hw::CC_WHEEL, 2), cc(hw::CC_SHIFT, 0)], 10);
        assert!(out.contains(&Out::Act(Action::SetParam { idx: par::BPM, value: 122 })), "{out:?}");
    }

    #[test]
    fn frame_and_lines() {
        let s = Shared::new();
        let mut a = Adapter::new();
        let leds = a.leds(0, &SharedModel(&s), false);
        // mode 1 is lit on the step buttons, the first knob bank on the track buttons
        assert_ne!(leds[hw::LED_STEPS], 0);
        assert_eq!(leds[hw::LED_TRACKS], hw::palette_index((255, 255, 255)));
        let lines = a.lines(0, &SharedModel(&s));
        assert!(lines[0].starts_with("1 Drum 1"), "{lines:?}");
        assert!(lines.iter().all(|l| l.chars().count() <= LINE_CHARS));
    }
}
