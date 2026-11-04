// SPDX-License-Identifier: GPL-3.0-only
//! LED and screen contents, and the device messages that bring the device up to date.

use crate::layout::Rgb;
use lcxl3::display::arrangement;
use lcxl3::encode;
use lcxl3::{Button, Control, DisplayConfig, DisplayTarget};
use std::collections::HashMap;

/// A popup (the device's overlay display): title, name, value. `gen` changes whenever it is shown
/// again.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Popup {
    pub gen: u32,
    pub fields: [String; 3],
}

/// What the device should show.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    /// Colour of every LED the surface controls (controls without an entry are off).
    pub leds: Vec<(Control, Rgb)>,
    /// Stationary display: title, name, value.
    pub stationary: [String; 3],
    pub popup: Option<Popup>,
}

/// A quarter-brightness version of a colour (toggle off, idle pads).
pub fn dim(c: Rgb) -> Rgb {
    (c.0 / 4, c.1 / 4, c.2 / 4)
}

/// Maximum characters sent per text field. The guide does not state field lengths; longer texts
/// are cut.
pub const MAX_TEXT: usize = 20;

fn cut(s: &str) -> String {
    s.chars().take(MAX_TEXT).collect()
}

/// Every control with an LED: encoders and the buttons except Shift.
pub fn led_controls() -> Vec<Control> {
    let mut v = Vec::new();
    for row in 0..3 {
        for col in 0..8 {
            v.push(Control::Encoder { row, col });
        }
    }
    for c in 0..8 {
        v.push(Control::Button(Button::Top(c)));
        v.push(Control::Button(Button::Bottom(c)));
    }
    for b in [
        Button::SoloArm,
        Button::MuteSelect,
        Button::Logo,
        Button::PageUp,
        Button::PageDown,
        Button::TrackLeft,
        Button::TrackRight,
        Button::Play,
        Button::Record,
    ] {
        v.push(Control::Button(b));
    }
    v
}

/// Keeps track of what the device shows and produces the messages for a new [`Frame`].
#[derive(Default)]
pub struct DeviceState {
    leds: HashMap<Control, Rgb>,
    stationary: Option<[String; 3]>,
    popup_gen: Option<u32>,
}

impl DeviceState {
    /// Messages that put the device into the state the surface expects: DAW mode, relative
    /// encoders, the display arrangements, and no automatic popups of raw values. Resets the
    /// cached state so that the next [`DeviceState::update`] sends everything.
    pub fn connect(&mut self) -> Vec<Vec<u8>> {
        *self = DeviceState::default();
        let mut m = vec![encode::daw_mode(true).to_vec()];
        for row in 0..3 {
            m.extend(encode::relative_row(row, true).map(|x| x.to_vec()));
        }
        // The surface shows its own popups; the device's automatic ones would show raw CC values.
        let quiet = DisplayConfig::Arrange {
            arrangement: arrangement::TITLE_NAME_VALUE,
            auto_on_change: false,
            auto_on_touch: false,
        };
        for row in 0..3 {
            for col in 0..8 {
                m.extend(encode::configure_display(DisplayTarget::Control(Control::Encoder { row, col }), quiet));
            }
        }
        for col in 0..8 {
            m.extend(encode::configure_display(DisplayTarget::Control(Control::Fader(col)), quiet));
        }
        m.extend(encode::configure_display(DisplayTarget::Stationary, quiet));
        m.extend(encode::configure_display(DisplayTarget::Overlay, quiet));
        m
    }

    /// Forget what the device shows, so that the next [`DeviceState::update`] sends everything
    /// (after the device returned from one of its own modes).
    pub fn invalidate(&mut self) {
        *self = DeviceState::default();
    }

    /// Messages that return the device to standalone mode.
    pub fn disconnect(&mut self) -> Vec<Vec<u8>> {
        *self = DeviceState::default();
        let mut m = Vec::new();
        for row in 0..3 {
            m.extend(encode::relative_row(row, false).map(|x| x.to_vec()));
        }
        m.push(encode::daw_mode(false).to_vec());
        m
    }

    /// The messages that change what the device shows into `frame`.
    pub fn update(&mut self, frame: &Frame) -> Vec<Vec<u8>> {
        let mut m = Vec::new();
        let wanted: HashMap<Control, Rgb> = frame.leds.iter().copied().collect();
        for c in led_controls() {
            let rgb = wanted.get(&c).copied().unwrap_or((0, 0, 0));
            if self.leds.get(&c) != Some(&rgb) {
                if let Some(msg) = encode::led_rgb(c, rgb.0 >> 1, rgb.1 >> 1, rgb.2 >> 1) {
                    m.push(msg);
                }
                self.leds.insert(c, rgb);
            }
        }
        if self.stationary.as_ref() != Some(&frame.stationary) {
            for (i, s) in frame.stationary.iter().enumerate() {
                m.extend(encode::display_text(DisplayTarget::Stationary, i as u8, &cut(s)));
            }
            m.extend(encode::configure_display(DisplayTarget::Stationary, DisplayConfig::Trigger));
            self.stationary = Some(frame.stationary.clone());
        }
        if let Some(p) = &frame.popup {
            if self.popup_gen != Some(p.gen) {
                for (i, s) in p.fields.iter().enumerate() {
                    m.extend(encode::display_text(DisplayTarget::Overlay, i as u8, &cut(s)));
                }
                m.extend(encode::configure_display(DisplayTarget::Overlay, DisplayConfig::Trigger));
                self.popup_gen = Some(p.gen);
            }
        }
        m
    }
}
