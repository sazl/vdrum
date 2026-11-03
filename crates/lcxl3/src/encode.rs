// SPDX-License-Identifier: GPL-3.0-only
//! Messages from the host to the device's DAW In port.

use crate::control::{Button, Control, CH_BUTTONS, CH_FEATURE, CH_QUERY};
use crate::display::{Bitmap, DisplayConfig, DisplayTarget};
use crate::{cmd, Feature, SurfaceMode, SYSEX_END, SYSEX_HEADER};

const NOTE_ON_CH16: u8 = 0x9F;
const CC: u8 = 0xB0;

fn on_off(on: bool) -> u8 {
    if on {
        0x7F
    } else {
        0x00
    }
}

fn sysex(command: u8, data: impl IntoIterator<Item = u8>) -> Vec<u8> {
    let mut m = SYSEX_HEADER.to_vec();
    m.push(command);
    m.extend(data);
    m.push(SYSEX_END);
    m
}

/// Enter (`true`) or leave DAW mode with a note message (page 8): `9F 0C 7F` / `9F 0C 00`.
pub fn daw_mode(on: bool) -> [u8; 3] {
    [NOTE_ON_CH16, 0x0C, on_off(on)]
}

/// Enter or leave DAW mode with SysEx (page 8): `F0 00 20 29 02 15 02 7F F7` / `... 02 00 F7`.
pub fn daw_mode_sysex(on: bool) -> Vec<u8> {
    sysex(cmd::DAW_MODE, [on_off(on)])
}

/// Enable all feature controls, or revert to the DAW set (page 16): `9F 0B 7F` / `9F 0B 00`.
pub fn feature_controls(on: bool) -> [u8; 3] {
    [NOTE_ON_CH16, 0x0B, on_off(on)]
}

/// Set a feature control (page 16): CC on channel 7. Values above 7Fh are masked to 7 bits.
pub fn set_feature(f: Feature, value: u8) -> Option<[u8; 3]> {
    Some([CC | CH_FEATURE, f.cc()?, value & 0x7F])
}

/// Query a feature control (page 16): the same CC on channel 8; the reply comes on channel 7.
///
/// The guide does not specify the value of a query; 0 is sent.
pub fn query_feature(f: Feature) -> Option<[u8; 3]> {
    Some([CC | CH_QUERY, f.cc()?, 0])
}

/// Switch an encoder row (0..=2) to relative (`true`) or absolute output (page 10): `B6 45|48|49 7F|00`.
pub fn relative_row(row: u8, on: bool) -> Option<[u8; 3]> {
    set_feature(Feature::RelativeRow(row), on_off(on))
}

/// Select a surface mode (page 10): `B6 1E <mode>`.
pub fn surface_mode(mode: SurfaceMode) -> Option<[u8; 3]> {
    set_feature(Feature::SurfaceMode, mode.value()?)
}

/// Enable or disable touch events (page 10): `B6 47 7F|00`.
pub fn touch_events(on: bool) -> [u8; 3] {
    [CC | CH_FEATURE, 71, on_off(on)]
}

/// Set a control's LED to a palette colour (page 11): `B0 <control> <colour>`. `None` for Shift
/// (tied to a feature control) and for out-of-range positions. The guide colours a control's LED
/// "if the control has any"; this function does not know which controls have one.
pub fn led_palette(c: Control, colour: u8) -> Option<[u8; 3]> {
    if c == Control::Button(Button::Shift) {
        return None;
    }
    Some([CC | CH_BUTTONS, c.cc()?, colour & 0x7F])
}

/// Set a button or encoder LED to an RGB colour, each component 0..=127 (page 12):
/// `F0 00 20 29 02 15 01 53 <control> <R> <G> <B> F7`. Encoders use their absolute CC in either
/// encoder mode. `None` for faders, Shift and out-of-range positions.
pub fn led_rgb(c: Control, r: u8, g: u8, b: u8) -> Option<Vec<u8>> {
    match c {
        Control::Fader(_) | Control::Button(Button::Shift) => None,
        _ => Some(sysex(cmd::LED, [cmd::LED_RGB, c.cc()?, r & 0x7F, g & 0x7F, b & 0x7F])),
    }
}

/// Configure a display (page 13): `F0 00 20 29 02 15 04 <target> <config> F7`.
pub fn configure_display(t: DisplayTarget, config: DisplayConfig) -> Option<Vec<u8>> {
    Some(sysex(cmd::DISPLAY_CONFIG, [t.byte()?, config.byte()?]))
}

/// Fill a text field of a configured display (page 14):
/// `F0 00 20 29 02 15 06 <target> <field> <text> F7`, text per [`crate::display::encode_text`].
pub fn display_text(t: DisplayTarget, field: u8, text: &str) -> Option<Vec<u8>> {
    if field > 0x7F {
        return None;
    }
    let target = t.byte()?;
    Some(sysex(cmd::DISPLAY_TEXT, [target, field].into_iter().chain(crate::display::encode_text(text))))
}

/// Send a bitmap (page 14): `F0 00 20 29 02 15 09 <target> <1216 bytes> F7`.
///
/// The guide prints the message as ending in `7Fh`, which is not a SysEx terminator; this function
/// ends it with `F7`. The guide names targets 20h (stationary) and 21h (global temporary) for
/// bitmaps; pass them as [`DisplayTarget::Raw`].
pub fn bitmap(t: DisplayTarget, image: &Bitmap) -> Option<Vec<u8>> {
    let target = t.byte()?;
    Some(sysex(cmd::BITMAP, core::iter::once(target).chain(image.pack())))
}

/// Stationary display target for bitmaps (page 15).
pub const BITMAP_STATIONARY: DisplayTarget = DisplayTarget::Raw(0x20);
/// Global temporary display target for bitmaps (page 15).
pub const BITMAP_TEMPORARY: DisplayTarget = DisplayTarget::Raw(0x21);
