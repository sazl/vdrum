// SPDX-License-Identifier: GPL-3.0-only
//! Screen: display targets, configuration byte, text encoding and bitmaps (pages 13 to 15).

use crate::control::Control;

/// A display target of the Configure Display and Set Text messages.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DisplayTarget {
    /// Temporary display of a fader or encoder (target = its absolute CC, 5..=36).
    Control(Control),
    /// Permanent/stationary display, 35h.
    Stationary,
    /// Overlay/temporary display, 36h.
    Overlay,
    /// Any other target byte.
    ///
    /// The guide names further targets without listing them with the others: 22h-28h ("targets
    /// only setting names", page 14) and, for bitmaps, 20h (stationary) and 21h (global temporary)
    /// (page 15), which differ from 35h and 36h above.
    Raw(u8),
}

impl DisplayTarget {
    /// The target byte; `None` for a button or an out-of-range control, or a raw value above 7Fh.
    pub fn byte(self) -> Option<u8> {
        match self {
            DisplayTarget::Control(c @ (Control::Encoder { .. } | Control::Fader(_))) => c.cc(),
            DisplayTarget::Control(Control::Button(_)) => None,
            DisplayTarget::Stationary => Some(0x35),
            DisplayTarget::Overlay => Some(0x36),
            DisplayTarget::Raw(b) if b < 0x80 => Some(b),
            DisplayTarget::Raw(_) => None,
        }
    }
}

/// Display arrangements (page 14). Field use: 1 = name, value; 2 = title, name, value; 3 = title
/// and 8 names; 4 = name with the numeric value (the default).
pub mod arrangement {
    pub const NAME_VALUE: u8 = 1;
    pub const TITLE_NAME_VALUE: u8 = 2;
    pub const TITLE_EIGHT_NAMES: u8 = 3;
    pub const NAME_NUMERIC: u8 = 4;
}

/// The `<config>` byte of Configure Display.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum DisplayConfig {
    /// 00h: cancel the display.
    Cancel,
    /// 7Fh: bring up the display with its current contents.
    Trigger,
    /// An arrangement 1..=30 with the two automatic-display flags.
    Arrange {
        arrangement: u8,
        /// Bit 6: the device shows the temporary display when the control changes (default set).
        auto_on_change: bool,
        /// Bit 5: the device shows the temporary display on touch, Shift + turn (default set).
        auto_on_touch: bool,
    },
}

impl DisplayConfig {
    /// An arrangement with both automatic-display flags set, as the device default.
    pub fn arrangement(arrangement: u8) -> DisplayConfig {
        DisplayConfig::Arrange { arrangement, auto_on_change: true, auto_on_touch: true }
    }

    /// The config byte; `None` for an arrangement outside 1..=30.
    pub fn byte(self) -> Option<u8> {
        match self {
            DisplayConfig::Cancel => Some(0x00),
            DisplayConfig::Trigger => Some(0x7F),
            DisplayConfig::Arrange { arrangement, auto_on_change, auto_on_touch } => {
                if !(1..=30).contains(&arrangement) {
                    return None;
                }
                Some(arrangement | (auto_on_change as u8) << 6 | (auto_on_touch as u8) << 5)
            }
        }
    }
}

/// Encodes text for Set Text: ASCII 20h..=7Eh as is, and four characters at reassigned control
/// codes: '□' (empty box) 1Bh, '■' (filled box) 1Ch, '♭' (flat) 1Dh, '♥' (heart) 1Eh. Any other
/// character becomes '?'. The guide states no length limit.
pub fn encode_text(text: &str) -> impl Iterator<Item = u8> + '_ {
    text.chars().map(|c| match c {
        ' '..='~' => c as u8,
        '□' => 0x1B,
        '■' => 0x1C,
        '♭' => 0x1D,
        '♥' => 0x1E,
        _ => b'?',
    })
}

/// Display width in pixels.
pub const WIDTH: usize = 128;
/// Display height in pixels.
pub const HEIGHT: usize = 64;
/// SysEx bytes per pixel row: 7 pixels per byte, 19 bytes (the last byte has 5 unused bits).
pub const ROW_BYTES: usize = 19;
/// Bytes of bitmap data in a Bitmap message: 19 × 64.
pub const BITMAP_BYTES: usize = ROW_BYTES * HEIGHT;

/// A 128 × 64 monochrome image, one bit per pixel, `true` = lit.
#[derive(Clone, PartialEq, Eq)]
pub struct Bitmap {
    rows: [[bool; WIDTH]; HEIGHT],
}

impl Default for Bitmap {
    fn default() -> Self {
        Bitmap { rows: [[false; WIDTH]; HEIGHT] }
    }
}

impl core::fmt::Debug for Bitmap {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let lit = self.rows.iter().flatten().filter(|p| **p).count();
        write!(f, "Bitmap({lit} lit)")
    }
}

impl Bitmap {
    pub fn get(&self, x: usize, y: usize) -> bool {
        x < WIDTH && y < HEIGHT && self.rows[y][x]
    }

    /// Sets a pixel; coordinates outside the display are ignored.
    pub fn set(&mut self, x: usize, y: usize, on: bool) {
        if x < WIDTH && y < HEIGHT {
            self.rows[y][x] = on;
        }
    }

    /// The 1216 data bytes: per row, 19 bytes of 7 pixels, the highest bit (bit 6) leftmost.
    pub fn pack(&self) -> [u8; BITMAP_BYTES] {
        let mut out = [0u8; BITMAP_BYTES];
        for (y, row) in self.rows.iter().enumerate() {
            for (x, &on) in row.iter().enumerate() {
                if on {
                    out[y * ROW_BYTES + x / 7] |= 0x40 >> (x % 7);
                }
            }
        }
        out
    }

    /// Inverse of [`Bitmap::pack`]; the unused bits of each row's last byte are ignored.
    pub fn unpack(data: &[u8; BITMAP_BYTES]) -> Bitmap {
        let mut b = Bitmap::default();
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                b.rows[y][x] = data[y * ROW_BYTES + x / 7] & (0x40 >> (x % 7)) != 0;
            }
        }
        b
    }
}
