// SPDX-License-Identifier: GPL-3.0-only
//! Surface controls and their DAW-mode Control Change numbers (programmer's reference v1.0, page 9).

/// MIDI channel (0-based) of the encoders and faders: channel 16.
pub const CH_ANALOGUE: u8 = 15;
/// MIDI channel (0-based) of the buttons other than Shift: channel 1.
pub const CH_BUTTONS: u8 = 0;
/// MIDI channel (0-based) of the feature controls, their replies, Shift and the surface mode report:
/// channel 7.
pub const CH_FEATURE: u8 = 6;
/// MIDI channel (0-based) for feature control queries: channel 8.
pub const CH_QUERY: u8 = 7;
/// MIDI channel (0-based) of touch events: channel 15.
pub const CH_TOUCH: u8 = 14;

/// First fader CC (fader 1). Faders are CC 5..=12.
pub const FADER_CC_BASE: u8 = 5;
/// First encoder CC in absolute mode (row 1, column 1). Encoders are CC 13..=36, row by row.
pub const ENCODER_CC_BASE: u8 = 13;
/// Offset added to an encoder's CC in relative mode (CC 77..=100).
pub const RELATIVE_CC_OFFSET: u8 = 64;
/// Value of a relative encoder message that encodes no movement.
pub const RELATIVE_PIVOT: u8 = 64;
/// First CC of the top button row. The top row is CC 37..=44, the bottom row CC 45..=52.
pub const TOP_ROW_CC_BASE: u8 = 37;
pub const BOTTOM_ROW_CC_BASE: u8 = 45;

/// A button. Every button except Shift sends on channel 1; Shift sends CC 63 on channel 7. The Mode
/// button sends nothing in DAW mode (n/a on page 9).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Button {
    /// Top row, column 0..=7 (CC 37..=44).
    Top(u8),
    /// Bottom row, column 0..=7 (CC 45..=52).
    Bottom(u8),
    /// CC 65.
    SoloArm,
    /// CC 66.
    MuteSelect,
    /// The button with the Novation logo, CC 104.
    Logo,
    /// CC 106.
    PageUp,
    /// CC 107.
    PageDown,
    /// Track left, CC 103.
    TrackLeft,
    /// Track right, CC 102.
    TrackRight,
    /// CC 116.
    Play,
    /// CC 118.
    Record,
    /// CC 63 on channel 7 (a feature control).
    Shift,
}

impl Button {
    pub const SHIFT_CC: u8 = 63;

    /// The button's CC number, `None` for a row column outside 0..=7.
    pub fn cc(self) -> Option<u8> {
        Some(match self {
            Button::Top(c) if c < 8 => TOP_ROW_CC_BASE + c,
            Button::Bottom(c) if c < 8 => BOTTOM_ROW_CC_BASE + c,
            Button::Top(_) | Button::Bottom(_) => return None,
            Button::SoloArm => 65,
            Button::MuteSelect => 66,
            Button::Logo => 104,
            Button::PageUp => 106,
            Button::PageDown => 107,
            Button::TrackLeft => 103,
            Button::TrackRight => 102,
            Button::Play => 116,
            Button::Record => 118,
            Button::Shift => Self::SHIFT_CC,
        })
    }

    /// The MIDI channel (0-based) the button sends on.
    pub fn channel(self) -> u8 {
        if self == Button::Shift {
            CH_FEATURE
        } else {
            CH_BUTTONS
        }
    }

    /// The channel 1 button with CC `cc` (Shift is not on channel 1).
    pub fn from_cc(cc: u8) -> Option<Button> {
        Some(match cc {
            37..=44 => Button::Top(cc - TOP_ROW_CC_BASE),
            45..=52 => Button::Bottom(cc - BOTTOM_ROW_CC_BASE),
            65 => Button::SoloArm,
            66 => Button::MuteSelect,
            104 => Button::Logo,
            106 => Button::PageUp,
            107 => Button::PageDown,
            103 => Button::TrackLeft,
            102 => Button::TrackRight,
            116 => Button::Play,
            118 => Button::Record,
            _ => return None,
        })
    }
}

/// A control of the surface.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Control {
    /// Encoder in row 0..=2 (top to bottom), column 0..=7.
    Encoder { row: u8, col: u8 },
    /// Fader in column 0..=7.
    Fader(u8),
    Button(Button),
}

impl Control {
    /// The CC number the control reports with in absolute mode, which is also the index used to set
    /// its LED and its temporary display. `None` for positions outside the surface.
    pub fn cc(self) -> Option<u8> {
        match self {
            Control::Encoder { row, col } if row < 3 && col < 8 => Some(ENCODER_CC_BASE + 8 * row + col),
            Control::Fader(c) if c < 8 => Some(FADER_CC_BASE + c),
            Control::Encoder { .. } | Control::Fader(_) => None,
            Control::Button(b) => b.cc(),
        }
    }

    /// The encoder or fader with absolute CC `cc` (5..=36).
    pub fn analogue_from_cc(cc: u8) -> Option<Control> {
        match cc {
            5..=12 => Some(Control::Fader(cc - FADER_CC_BASE)),
            13..=36 => {
                let i = cc - ENCODER_CC_BASE;
                Some(Control::Encoder { row: i / 8, col: i % 8 })
            }
            _ => None,
        }
    }
}

/// An encoder row (0..=2) and column (0..=7) for a relative-mode CC (77..=100).
pub fn encoder_from_relative_cc(cc: u8) -> Option<(u8, u8)> {
    match cc {
        77..=100 => {
            let i = cc - ENCODER_CC_BASE - RELATIVE_CC_OFFSET;
            Some((i / 8, i % 8))
        }
        _ => None,
    }
}
