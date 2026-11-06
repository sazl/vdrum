// SPDX-License-Identifier: GPL-3.0-only
//! A virtual Launch Control XL 3 in DAW mode: it takes the messages a host sends to the DAW In port
//! and keeps what the device would show (LEDs, screen), and turns physical gestures (turning an
//! encoder, moving a fader, pressing a button) into the messages the device sends on its DAW Out
//! port. The simulator runs the surface against this, so the simulator and the hardware go through
//! the same bytes.
//!
//! Behaviour follows the Programmer's Reference Guide v1.0. Where the guide says nothing, the choice
//! made here is marked UNVERIFIED: the temporary display timeout before the host sets one, the
//! default arrangement of a display the host never configured, what a palette colour index looks
//! like, and how a 7-bit RGB value maps to brightness (rendered here as value × 2).

use crate::control::{Button, Control, CH_ANALOGUE, CH_BUTTONS, CH_FEATURE, ENCODER_CC_BASE, FADER_CC_BASE, RELATIVE_CC_OFFSET, RELATIVE_PIVOT};
use crate::display::{arrangement, Bitmap, BITMAP_BYTES};
use crate::{cmd, Feature, SurfaceMode, SYSEX_HEADER};
use std::collections::HashMap;

/// Temporary display timeout used until the host sets feature CC 113, in tenths of a second.
/// UNVERIFIED: the guide does not give the factory value.
pub const DEFAULT_TIMEOUT_TENTHS: u8 = 10;
/// The guide's minimum temporary display time: 1 s.
pub const MIN_TIMEOUT_MS: u64 = 1000;

/// An LED's colour as the host set it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Led {
    /// RGB, each component 0..=127.
    Rgb(u8, u8, u8),
    /// A palette index 0..=127 (the palette is printed as a table in the guide, not reproduced).
    Palette(u8),
}

impl Led {
    /// 8-bit RGB for drawing. Palette indices other than 0 are drawn mid-grey (UNVERIFIED).
    pub fn rgb8(self) -> (u8, u8, u8) {
        match self {
            Led::Rgb(r, g, b) => (r.min(127) * 2, g.min(127) * 2, b.min(127) * 2),
            Led::Palette(0) => (0, 0, 0),
            Led::Palette(_) => (128, 128, 128),
        }
    }
}

/// One display target's configuration and text fields.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Display {
    /// Arrangement 1..=30, 0 before the host configured it.
    pub arrangement: u8,
    pub auto_on_change: bool,
    pub auto_on_touch: bool,
    pub fields: Vec<String>,
}

/// What the screen shows.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Screen {
    /// `stationary`, `overlay`, `control:<cc>`, `bitmap:stationary` or `bitmap:temporary`.
    pub source: String,
    pub arrangement: u8,
    /// The lines the arrangement shows, in order (title, name, value for arrangement 2).
    pub lines: Vec<String>,
    /// A bitmap, when one is shown instead of text.
    pub bitmap: Option<Bitmap>,
}

/// The virtual device.
#[derive(Clone, Debug)]
pub struct Device {
    pub daw_mode: bool,
    pub surface_mode: SurfaceMode,
    pub relative: [bool; 3],
    pub features: HashMap<u8, u8>,
    pub leds: HashMap<Control, Led>,
    pub stationary: Display,
    pub overlay: Display,
    pub controls: HashMap<u8, Display>,
    /// Position of each encoder in absolute mode, 0..=127.
    pub encoder_pos: [u8; 24],
    pub fader_pos: [u8; 8],
    /// The temporary display shown and when it ends (ms).
    temporary: Option<(String, u64)>,
    bitmap_stationary: Option<Bitmap>,
    bitmap_temporary: Option<(Bitmap, u64)>,
    /// Messages the host sent that the emulator does not understand.
    pub unknown: Vec<Vec<u8>>,
}

impl Default for Device {
    fn default() -> Self {
        Device {
            daw_mode: false,
            surface_mode: SurfaceMode::DawMixer,
            relative: [false; 3],
            features: HashMap::new(),
            leds: HashMap::new(),
            stationary: Display::default(),
            overlay: Display::default(),
            controls: HashMap::new(),
            encoder_pos: [64; 24],
            fader_pos: [0; 8],
            temporary: None,
            bitmap_stationary: None,
            bitmap_temporary: None,
            unknown: Vec::new(),
        }
    }
}

fn decode_text(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|&b| match b {
            0x20..=0x7E => b as char,
            0x1B => '□',
            0x1C => '■',
            0x1D => '♭',
            0x1E => '♥',
            _ => '?',
        })
        .collect()
}

fn cc(ch: u8, cc: u8, v: u8) -> Vec<u8> {
    vec![0xB0 | ch, cc, v]
}

impl Device {
    pub fn new() -> Device {
        Device::default()
    }

    /// Temporary display time in milliseconds (feature CC 113, tenths of a second, at least 1 s).
    pub fn timeout_ms(&self) -> u64 {
        let tenths = self.features.get(&113).copied().unwrap_or(DEFAULT_TIMEOUT_TENTHS);
        (tenths as u64 * 100).max(MIN_TIMEOUT_MS)
    }

    fn display_mut(&mut self, target: u8) -> Option<&mut Display> {
        match target {
            0x35 => Some(&mut self.stationary),
            0x36 => Some(&mut self.overlay),
            5..=36 => Some(self.controls.entry(target).or_default()),
            _ => None,
        }
    }

    /// Handle one complete message from the host (DAW In port) at time `now` (ms). Returns the
    /// device's replies (the bitmap acknowledgement).
    pub fn receive(&mut self, msg: &[u8], now: u64) -> Vec<Vec<u8>> {
        let mut replies = Vec::new();
        match msg {
            [0x9F, 0x0C, v] => self.daw_mode = *v > 0,
            [0x9F, 0x0B, _] => {}
            &[s, c, v] if s & 0xF0 == 0xB0 && c < 0x80 && v < 0x80 => {
                let ch = s & 0x0F;
                if ch == CH_FEATURE {
                    match Feature::from_cc(c) {
                        Some(Feature::RelativeRow(r)) => self.relative[r as usize] = v > 0,
                        Some(Feature::SurfaceMode) => {
                            if let Some(m) = SurfaceMode::from_value(v) {
                                self.surface_mode = m;
                            }
                        }
                        Some(_) => {
                            self.features.insert(c, v);
                        }
                        None => self.unknown.push(msg.to_vec()),
                    }
                } else if ch == CH_BUTTONS {
                    match led_control(c) {
                        Some(ctl) => {
                            self.leds.insert(ctl, Led::Palette(v));
                        }
                        None => self.unknown.push(msg.to_vec()),
                    }
                } else {
                    self.unknown.push(msg.to_vec());
                }
            }
            [0xF0, ..] => {
                let body = msg.strip_suffix(&[0xF7]).unwrap_or(msg);
                match body.strip_prefix(&SYSEX_HEADER) {
                    Some(&[cmd::DAW_MODE, v]) => self.daw_mode = v > 0,
                    Some(&[cmd::LED, cmd::LED_RGB, c, r, g, b]) => match led_control(c) {
                        Some(ctl) => {
                            self.leds.insert(ctl, Led::Rgb(r, g, b));
                        }
                        None => self.unknown.push(msg.to_vec()),
                    },
                    Some(&[cmd::DISPLAY_CONFIG, target, config]) => self.configure(target, config, now),
                    Some([cmd::DISPLAY_TEXT, target, field, text @ ..]) => {
                        let (target, field) = (*target, *field as usize);
                        match self.display_mut(target) {
                            Some(d) => {
                                if d.fields.len() <= field {
                                    d.fields.resize(field + 1, String::new());
                                }
                                d.fields[field] = decode_text(text);
                            }
                            None => self.unknown.push(msg.to_vec()),
                        }
                    }
                    Some([cmd::BITMAP, target, data @ ..]) if data.len() == BITMAP_BYTES => {
                        let mut a = [0u8; BITMAP_BYTES];
                        a.copy_from_slice(data);
                        let b = Bitmap::unpack(&a);
                        match *target {
                            0x20 => self.bitmap_stationary = Some(b),
                            0x21 => self.bitmap_temporary = Some((b, now + self.timeout_ms())),
                            _ => {}
                        }
                        let mut ack = SYSEX_HEADER.to_vec();
                        ack.extend([cmd::BITMAP, 0x7F, 0xF7]);
                        replies.push(ack);
                    }
                    _ => self.unknown.push(msg.to_vec()),
                }
            }
            _ => self.unknown.push(msg.to_vec()),
        }
        replies
    }

    fn configure(&mut self, target: u8, config: u8, now: u64) {
        let timeout = self.timeout_ms();
        match config {
            0x00 => {
                if target == 0x36 || target == 0x35 {
                    self.temporary = None;
                    self.bitmap_temporary = None;
                    if target == 0x35 {
                        self.bitmap_stationary = None;
                    }
                }
            }
            0x7F => {
                if target == 0x36 {
                    self.temporary = Some(("overlay".into(), now + timeout));
                } else if (5..=36).contains(&target) {
                    self.temporary = Some((format!("control:{target}"), now + timeout));
                } else if target == 0x35 {
                    // Triggering the normal display ends a bitmap (page 15).
                    self.bitmap_stationary = None;
                }
            }
            c => {
                if let Some(d) = self.display_mut(target) {
                    d.arrangement = c & 0x1F;
                    d.auto_on_change = c & 0x40 != 0;
                    d.auto_on_touch = c & 0x20 != 0;
                }
            }
        }
    }

    /// What the screen shows at time `now`.
    pub fn screen(&self, now: u64) -> Screen {
        if let Some((b, until)) = &self.bitmap_temporary {
            if now < *until {
                return Screen { source: "bitmap:temporary".into(), arrangement: 0, lines: vec![], bitmap: Some(b.clone()) };
            }
        }
        if let Some((src, until)) = &self.temporary {
            if now < *until {
                let d = if src == "overlay" {
                    Some(&self.overlay)
                } else {
                    src.strip_prefix("control:").and_then(|n| n.parse::<u8>().ok()).and_then(|n| self.controls.get(&n))
                };
                if let Some(d) = d {
                    return Screen { source: src.clone(), arrangement: d.arrangement, lines: lines(d), bitmap: None };
                }
            }
        }
        if let Some(b) = &self.bitmap_stationary {
            return Screen { source: "bitmap:stationary".into(), arrangement: 0, lines: vec![], bitmap: Some(b.clone()) };
        }
        Screen { source: "stationary".into(), arrangement: self.stationary.arrangement, lines: lines(&self.stationary), bitmap: None }
    }

    /// The colour of a control's LED (black when the host never set it).
    pub fn led(&self, c: Control) -> Led {
        self.leds.get(&c).copied().unwrap_or(Led::Rgb(0, 0, 0))
    }

    fn auto_display(&mut self, target: u8, value: u8, now: u64) {
        let timeout = self.timeout_ms();
        let d = self.controls.entry(target).or_insert_with(|| Display { arrangement: arrangement::NAME_NUMERIC, auto_on_change: true, auto_on_touch: true, fields: vec![] });
        if d.auto_on_change {
            if d.arrangement == 0 {
                d.arrangement = arrangement::NAME_NUMERIC;
            }
            if d.fields.is_empty() {
                d.fields = vec![format!("CC {target}")];
            }
            if d.arrangement == arrangement::NAME_NUMERIC {
                d.fields.truncate(1);
                d.fields.push(value.to_string());
            }
            self.temporary = Some((format!("control:{target}"), now + timeout));
        }
    }

    /// Turn encoder (`row` 0..=2, `col` 0..=7) by `detents` (positive = clockwise). Returns the
    /// messages the device sends, one per detent in relative mode (UNVERIFIED: the device may
    /// combine fast turns into larger deltas), one per change in absolute mode.
    pub fn turn(&mut self, row: u8, col: u8, detents: i32, now: u64) -> Vec<Vec<u8>> {
        if !self.daw_mode || row > 2 || col > 7 {
            return vec![];
        }
        let i = (row * 8 + col) as usize;
        let abs_cc = ENCODER_CC_BASE + row * 8 + col;
        let mut out = Vec::new();
        let step = detents.signum();
        for _ in 0..detents.unsigned_abs() {
            if self.relative[row as usize] {
                out.push(cc(CH_ANALOGUE, abs_cc + RELATIVE_CC_OFFSET, (RELATIVE_PIVOT as i32 + step) as u8));
            } else {
                let v = (self.encoder_pos[i] as i32 + step).clamp(0, 127) as u8;
                if v != self.encoder_pos[i] {
                    self.encoder_pos[i] = v;
                    out.push(cc(CH_ANALOGUE, abs_cc, v));
                }
            }
        }
        if !out.is_empty() {
            let shown = self.encoder_pos[i];
            self.auto_display(abs_cc, shown, now);
        }
        out
    }

    /// Move fader `col` to `value` (0..=127). Returns the device's message, if the value changed.
    pub fn move_fader(&mut self, col: u8, value: u8, now: u64) -> Vec<Vec<u8>> {
        if col > 7 {
            return vec![];
        }
        let value = value.min(127);
        if self.fader_pos[col as usize] == value {
            return vec![];
        }
        self.fader_pos[col as usize] = value;
        if !self.daw_mode {
            return vec![];
        }
        self.auto_display(FADER_CC_BASE + col, value, now);
        vec![cc(CH_ANALOGUE, FADER_CC_BASE + col, value)]
    }

    /// Press (`true`) or release a button. Shift sends CC 63 on channel 7; the Mode button sends
    /// nothing in DAW mode (page 9: n/a).
    pub fn button(&mut self, b: Button, pressed: bool) -> Vec<Vec<u8>> {
        if !self.daw_mode {
            return vec![];
        }
        let v = if pressed { 127 } else { 0 };
        if b == Button::Shift {
            return vec![cc(CH_FEATURE, Button::SHIFT_CC, v)];
        }
        match b.cc() {
            Some(n) => vec![cc(CH_BUTTONS, n, v)],
            None => vec![],
        }
    }
}

/// The lines an arrangement shows: 1 name, value; 2 title, name, value; 3 title and 8 names;
/// 4 name and numeric value.
fn lines(d: &Display) -> Vec<String> {
    let n = match d.arrangement {
        1 | 4 => 2,
        2 => 3,
        3 => 9,
        _ => d.fields.len(),
    };
    (0..n).map(|i| d.fields.get(i).cloned().unwrap_or_default()).collect()
}

/// The control whose LED a CC index addresses: encoders by their absolute CC, buttons by theirs.
fn led_control(c: u8) -> Option<Control> {
    if let Some(ctl @ Control::Encoder { .. }) = Control::analogue_from_cc(c) {
        return Some(ctl);
    }
    Button::from_cc(c).map(Control::Button)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::display::{DisplayConfig, DisplayTarget};
    use crate::{decode, encode, Input};

    fn daw() -> Device {
        let mut d = Device::new();
        d.receive(&encode::daw_mode(true), 0);
        d
    }

    #[test]
    fn daw_mode_and_relative_rows() {
        let mut d = Device::new();
        assert!(d.turn(0, 0, 1, 0).is_empty(), "nothing on the DAW port outside DAW mode");
        d.receive(&encode::daw_mode_sysex(true), 0);
        assert!(d.daw_mode);
        // absolute: CC 13 with the new position
        assert_eq!(d.turn(0, 0, 1, 0), vec![vec![0xBF, 13, 65]]);
        d.receive(&encode::relative_row(0, true).unwrap(), 0);
        assert_eq!(d.turn(0, 0, -2, 0), vec![vec![0xBF, 77, 63], vec![0xBF, 77, 63]]);
        assert_eq!(decode(&d.turn(0, 7, 1, 0)[0]), Input::Relative { row: 0, col: 7, delta: 1 });
        // row 2 still absolute
        assert_eq!(decode(&d.turn(2, 0, 1, 0)[0]), Input::Analogue { control: Control::Encoder { row: 2, col: 0 }, value: 65 });
    }

    #[test]
    fn faders_and_buttons_send_what_decode_reads() {
        let mut d = daw();
        assert_eq!(decode(&d.move_fader(3, 100, 0)[0]), Input::Analogue { control: Control::Fader(3), value: 100 });
        assert!(d.move_fader(3, 100, 0).is_empty(), "no message without a change");
        for b in [Button::Top(0), Button::Bottom(7), Button::SoloArm, Button::MuteSelect, Button::Logo, Button::PageUp, Button::PageDown, Button::TrackLeft, Button::TrackRight, Button::Play, Button::Record, Button::Shift] {
            assert_eq!(decode(&d.button(b, true)[0]), Input::Button { button: b, pressed: true }, "{b:?}");
            assert_eq!(decode(&d.button(b, false)[0]), Input::Button { button: b, pressed: false }, "{b:?}");
        }
    }

    #[test]
    fn leds_rgb_and_palette() {
        let mut d = daw();
        let k = Control::Encoder { row: 1, col: 2 };
        d.receive(&encode::led_rgb(k, 127, 10, 0).unwrap(), 0);
        assert_eq!(d.led(k), Led::Rgb(127, 10, 0));
        assert_eq!(d.led(k).rgb8(), (254, 20, 0));
        let b = Control::Button(Button::Play);
        d.receive(&encode::led_palette(b, 5).unwrap(), 0);
        assert_eq!(d.led(b), Led::Palette(5));
        assert!(d.unknown.is_empty());
    }

    #[test]
    fn stationary_and_overlay_displays() {
        let mut d = daw();
        let cfg = DisplayConfig::Arrange { arrangement: arrangement::TITLE_NAME_VALUE, auto_on_change: false, auto_on_touch: false };
        d.receive(&encode::configure_display(DisplayTarget::Stationary, cfg).unwrap(), 0);
        d.receive(&encode::configure_display(DisplayTarget::Overlay, cfg).unwrap(), 0);
        for (i, s) in ["1 Drum 1", "Init", "Fine 64"].iter().enumerate() {
            d.receive(&encode::display_text(DisplayTarget::Stationary, i as u8, s).unwrap(), 0);
        }
        d.receive(&encode::configure_display(DisplayTarget::Stationary, DisplayConfig::Trigger).unwrap(), 0);
        assert_eq!(d.screen(0).lines, vec!["1 Drum 1", "Init", "Fine 64"]);
        for (i, s) in ["Filter", "Cutoff", "90 \u{266d}"].iter().enumerate() {
            d.receive(&encode::display_text(DisplayTarget::Overlay, i as u8, s).unwrap(), 100);
        }
        d.receive(&encode::configure_display(DisplayTarget::Overlay, DisplayConfig::Trigger).unwrap(), 100);
        let s = d.screen(150);
        assert_eq!((s.source.as_str(), s.lines.clone()), ("overlay", vec!["Filter".to_string(), "Cutoff".into(), "90 ♭".into()]));
        // default timeout 1 s, then back to the stationary display
        assert_eq!(d.screen(100 + 999).source, "overlay");
        assert_eq!(d.screen(100 + 1000).source, "stationary");
        // timeout 2.5 s
        d.receive(&encode::set_feature(Feature::DisplayTimeout, 25).unwrap(), 0);
        d.receive(&encode::configure_display(DisplayTarget::Overlay, DisplayConfig::Trigger).unwrap(), 5000);
        assert_eq!(d.screen(7499).source, "overlay");
        assert_eq!(d.screen(7500).source, "stationary");
        assert!(d.unknown.is_empty(), "{:?}", d.unknown);
    }

    #[test]
    fn automatic_control_display_unless_disabled() {
        let mut d = daw();
        d.move_fader(0, 90, 0);
        let s = d.screen(10);
        assert_eq!((s.source.as_str(), s.lines.clone()), ("control:5", vec!["CC 5".to_string(), "90".into()]));
        let quiet = DisplayConfig::Arrange { arrangement: arrangement::TITLE_NAME_VALUE, auto_on_change: false, auto_on_touch: false };
        d.receive(&encode::configure_display(DisplayTarget::Control(Control::Fader(1)), quiet).unwrap(), 2000);
        d.move_fader(1, 10, 2000);
        assert_eq!(d.screen(2010).source, "stationary");
    }

    #[test]
    fn bitmap_is_acknowledged_and_shown() {
        let mut d = daw();
        let mut b = Bitmap::default();
        b.set(0, 0, true);
        b.set(127, 63, true);
        let ack = d.receive(&encode::bitmap(encode::BITMAP_STATIONARY, &b).unwrap(), 0);
        assert_eq!(decode(&ack[0]), Input::BitmapAck);
        let s = d.screen(0);
        assert_eq!(s.source, "bitmap:stationary");
        assert!(s.bitmap.as_ref().unwrap().get(127, 63));
        d.receive(&encode::configure_display(DisplayTarget::Stationary, DisplayConfig::Trigger).unwrap(), 0);
        assert_eq!(d.screen(0).source, "stationary");
    }
}
