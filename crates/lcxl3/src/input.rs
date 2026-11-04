// SPDX-License-Identifier: GPL-3.0-only
//! Messages from the device's DAW Out port.

use crate::control::{
    encoder_from_relative_cc, Button, Control, CH_ANALOGUE, CH_BUTTONS, CH_FEATURE, CH_TOUCH, RELATIVE_PIVOT,
};
use crate::{cmd, Feature, SurfaceMode, SYSEX_HEADER};

/// A decoded message from the device.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Input {
    /// An encoder in absolute mode or a fader moved to `value`.
    Analogue { control: Control, value: u8 },
    /// An encoder in relative mode moved by `delta` steps (positive = clockwise); value − 64.
    Relative { row: u8, col: u8, delta: i8 },
    /// A button was pressed (`true`, value > 0) or released (value 0). Includes Shift.
    Button { button: Button, pressed: bool },
    /// A touch on (`true`) or off event of an encoder or fader (channel 15, page 11).
    Touch { control: Control, on: bool },
    /// The surface changed mode (CC 30 on channel 7, page 10).
    SurfaceMode(SurfaceMode),
    /// A feature control reply on channel 7 (page 16), other than Shift and the surface mode.
    Feature { feature: Feature, value: u8 },
    /// The device is ready for the next bitmap (page 15).
    BitmapAck,
    /// Anything else.
    Other,
}

/// Decodes one complete MIDI message (no running status). SysEx messages may or may not include the
/// trailing `F7`.
pub fn decode(msg: &[u8]) -> Input {
    match msg {
        [0xF0, ..] => decode_sysex(msg),
        &[status, cc, value] if status & 0xF0 == 0xB0 && cc < 0x80 && value < 0x80 => decode_cc(status & 0x0F, cc, value),
        _ => Input::Other,
    }
}

fn decode_cc(ch: u8, cc: u8, value: u8) -> Input {
    match ch {
        CH_ANALOGUE => {
            if let Some(control) = Control::analogue_from_cc(cc) {
                Input::Analogue { control, value }
            } else if let Some((row, col)) = encoder_from_relative_cc(cc) {
                Input::Relative { row, col, delta: (value as i16 - RELATIVE_PIVOT as i16) as i8 }
            } else {
                Input::Other
            }
        }
        CH_BUTTONS => match Button::from_cc(cc) {
            Some(button) => Input::Button { button, pressed: value > 0 },
            None => Input::Other,
        },
        CH_TOUCH => match Control::analogue_from_cc(cc) {
            Some(control) => Input::Touch { control, on: value > 0 },
            None => Input::Other,
        },
        CH_FEATURE => match Feature::from_cc(cc) {
            Some(Feature::Shift) => Input::Button { button: Button::Shift, pressed: value > 0 },
            Some(Feature::SurfaceMode) => match SurfaceMode::from_value(value) {
                Some(m) => Input::SurfaceMode(m),
                None => Input::Other,
            },
            Some(feature) => Input::Feature { feature, value },
            None => Input::Other,
        },
        _ => Input::Other,
    }
}

fn decode_sysex(msg: &[u8]) -> Input {
    let body = msg.strip_suffix(&[0xF7]).unwrap_or(msg);
    match body.strip_prefix(&SYSEX_HEADER) {
        Some(&[cmd::BITMAP, 0x7F]) => Input::BitmapAck,
        _ => Input::Other,
    }
}
