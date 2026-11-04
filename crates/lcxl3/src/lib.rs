// SPDX-License-Identifier: GPL-3.0-only
//! Message codec for the DAW mode of the Novation Launch Control XL 3.
//!
//! Pure functions: [`encode`] builds the messages a host sends to the device's DAW In port, and
//! [`decode`] interprets complete messages read from its DAW Out port. There is no I/O and no state.
//!
//! Source: Novation, *Launch Control XL 3 Programmer's Reference Guide*, version 1.0. Page numbers in
//! the documentation refer to that guide. Where the guide is inconsistent, the choice made here is
//! stated at the item; those items have not been checked against a device.

pub mod control;
pub mod input;
pub mod display;
pub mod encode;

pub use control::{Button, Control};
pub use input::{decode, Input};
pub use display::{Bitmap, DisplayConfig, DisplayTarget};

/// Header of every SysEx message, in both directions (page 6): `F0 00 20 29 02 15`.
pub const SYSEX_HEADER: [u8; 6] = [0xF0, 0x00, 0x20, 0x29, 0x02, 0x15];
/// End of a SysEx message.
pub const SYSEX_END: u8 = 0xF7;

/// SysEx command bytes (the byte after the header).
pub mod cmd {
    /// RGB LED colour (page 12): `01 53 <control> <R> <G> <B>`.
    pub const LED: u8 = 0x01;
    /// Sub-command of [`LED`] for an RGB colour.
    pub const LED_RGB: u8 = 0x53;
    /// DAW mode on/off (page 8): `02 <7F|00>`.
    pub const DAW_MODE: u8 = 0x02;
    /// Configure a display (page 13): `04 <target> <config>`.
    pub const DISPLAY_CONFIG: u8 = 0x04;
    /// Set a text field (page 14): `06 <target> <field> <text>`.
    pub const DISPLAY_TEXT: u8 = 0x06;
    /// Bitmap (page 14): `09 <target> <1216 bytes>`.
    pub const BITMAP: u8 = 0x09;
}

/// Feature controls (page 17): Control Changes on channel 7, queried on channel 8.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Feature {
    /// CC 30: surface mode select / report, values per [`SurfaceMode`].
    SurfaceMode,
    /// CC 63: Shift.
    Shift,
    /// CC 69, 72, 73: relative output of encoder row 0, 1, 2.
    RelativeRow(u8),
    /// CC 70: DAW fader pickup.
    FaderPickup,
    /// CC 71: DAW touch events.
    TouchEvents,
    /// CC 100: global MIDI channel 0..=15 (non-volatile).
    GlobalChannel,
    /// CC 111: LED brightness 0..=127 (non-volatile).
    LedBrightness,
    /// CC 112: screen brightness 0..=127 (non-volatile).
    ScreenBrightness,
    /// CC 113: temporary display timeout 0..=99 in 1/10 s, minimum 1 s (non-volatile).
    DisplayTimeout,
    /// CC 120: Out2 MIDI thru (non-volatile).
    Out2Thru,
    /// CC 121: encoder curve, [`EncoderCurve`] (non-volatile).
    EncoderCurve,
}

impl Feature {
    pub const ALL: [Feature; 13] = [
        Feature::SurfaceMode,
        Feature::Shift,
        Feature::RelativeRow(0),
        Feature::RelativeRow(1),
        Feature::RelativeRow(2),
        Feature::FaderPickup,
        Feature::TouchEvents,
        Feature::GlobalChannel,
        Feature::LedBrightness,
        Feature::ScreenBrightness,
        Feature::DisplayTimeout,
        Feature::Out2Thru,
        Feature::EncoderCurve,
    ];

    /// The CC number; `None` for a relative row outside 0..=2.
    pub fn cc(self) -> Option<u8> {
        Some(match self {
            Feature::SurfaceMode => 30,
            Feature::Shift => 63,
            Feature::RelativeRow(0) => 69,
            Feature::RelativeRow(1) => 72,
            Feature::RelativeRow(2) => 73,
            Feature::RelativeRow(_) => return None,
            Feature::FaderPickup => 70,
            Feature::TouchEvents => 71,
            Feature::GlobalChannel => 100,
            Feature::LedBrightness => 111,
            Feature::ScreenBrightness => 112,
            Feature::DisplayTimeout => 113,
            Feature::Out2Thru => 120,
            Feature::EncoderCurve => 121,
        })
    }

    pub fn from_cc(cc: u8) -> Option<Feature> {
        Feature::ALL.into_iter().find(|f| f.cc() == Some(cc))
    }
}

/// Surface modes (pages 10 and 17).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SurfaceMode {
    /// Value 1.
    DawMixer,
    /// Value 2.
    DawControl,
    /// Custom mode 1..=16: values 6..=9 for 1..=4, 18..=29 for 5..=16.
    ///
    /// The list on page 10 gives 12h-15h as "modes 5 - 8" and 16h-1Dh as "Custom Modes 8 - 16",
    /// which counts mode 8 twice; the table on page 17 (12h-1Dh = Custom Modes 5-16) is used here.
    Custom(u8),
}

impl SurfaceMode {
    /// The CC value; `None` for a custom mode outside 1..=16.
    pub fn value(self) -> Option<u8> {
        match self {
            SurfaceMode::DawMixer => Some(1),
            SurfaceMode::DawControl => Some(2),
            SurfaceMode::Custom(n @ 1..=4) => Some(5 + n),
            SurfaceMode::Custom(n @ 5..=16) => Some(13 + n),
            SurfaceMode::Custom(_) => None,
        }
    }

    pub fn from_value(v: u8) -> Option<SurfaceMode> {
        match v {
            1 => Some(SurfaceMode::DawMixer),
            2 => Some(SurfaceMode::DawControl),
            6..=9 => Some(SurfaceMode::Custom(v - 5)),
            18..=29 => Some(SurfaceMode::Custom(v - 13)),
            _ => None,
        }
    }
}

/// Encoder curve values of [`Feature::EncoderCurve`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum EncoderCurve {
    Slow = 0,
    Medium = 1,
    Fast = 2,
}
