// SPDX-License-Identifier: GPL-3.0-only
//! Ableton Move's controls as Schwung delivers them (cable 0), the LEDs `ui.js` drives, and the
//! colour palette. Sources: Schwung 1.9.2 `docs/API.md` ("Move Hardware MIDI Mapping") and
//! `src/shared/constants.mjs` (palette and LED groups).

/// First pad note (bottom-left); pads are notes 68..=99, bottom-left to top-right, 8 per row.
pub const PAD_BASE: u8 = 68;
/// First step button note; steps are notes 16..=31.
pub const STEP_BASE: u8 = 16;
/// The accent pad: the last pad of the bottom (voice) row.
pub const ACCENT_PAD: u8 = 7;

/// Pad 0..32 of a note (row = index / 8 from the bottom, column = index % 8).
pub fn pad_index(note: u8) -> Option<u8> {
    (PAD_BASE..PAD_BASE + 32).contains(&note).then(|| note - PAD_BASE)
}

/// Step button 0..16 of a note.
pub fn step_index(note: u8) -> Option<u8> {
    (STEP_BASE..STEP_BASE + 16).contains(&note).then(|| note - STEP_BASE)
}

// Control CCs.
pub const CC_WHEEL_CLICK: u8 = 3;
pub const CC_WHEEL: u8 = 14;
/// Track buttons: CC 43 is track 1 (top), CC 40 track 4.
pub const CC_TRACK1: u8 = 43;
pub const CC_TRACK4: u8 = 40;
pub const CC_SHIFT: u8 = 49;
pub const CC_MENU: u8 = 50;
pub const CC_BACK: u8 = 51;
pub const CC_CAPTURE: u8 = 52;
pub const CC_DOWN: u8 = 54;
pub const CC_UP: u8 = 55;
pub const CC_UNDO: u8 = 56;
pub const CC_LOOP: u8 = 58;
pub const CC_COPY: u8 = 60;
pub const CC_LEFT: u8 = 62;
pub const CC_RIGHT: u8 = 63;
/// Encoders 1..8: CC 71..=78, relative (1..=63 clockwise, 65..=127 counter-clockwise).
pub const CC_KNOB1: u8 = 71;
pub const CC_VOLUME: u8 = 79;
pub const CC_PLAY: u8 = 85;
pub const CC_REC: u8 = 86;
pub const CC_MUTE: u8 = 88;
pub const CC_SAMPLE: u8 = 118;
pub const CC_DELETE: u8 = 119;

/// A relative encoder value as a signed delta (as `decodeDelta` in Schwung's `input_filter.mjs`).
pub fn delta(v: u8) -> i32 {
    match v {
        1..=63 => v as i32,
        65..=127 => -(128 - v as i32),
        _ => 0,
    }
}

/// An LED `ui.js` sets.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Led {
    /// Note-addressed RGB LED (pads, step buttons): palette index.
    Note(u8),
    /// CC-addressed RGB LED (track buttons, Play, Rec, Sample): palette index.
    RgbCc(u8),
    /// CC-addressed white LED: brightness 0..=127.
    WhiteCc(u8),
}

/// White LED brightness levels (`WhiteLedDim`, `WhiteLedMedium`, `WhiteLedBright`).
pub const WHITE_OFF: u8 = 0;
pub const WHITE_DIM: u8 = 0x10;
pub const WHITE_MEDIUM: u8 = 0x40;
pub const WHITE_BRIGHT: u8 = 0x7c;

/// Index of the first step LED, the first track button and the first button in [`LEDS`].
pub const LED_STEPS: usize = 32;
pub const LED_TRACKS: usize = 48;
pub const LED_BUTTONS: usize = 52;

/// The buttons with LEDs, in [`LEDS`] order from [`LED_BUTTONS`].
pub const BUTTON_LEDS: [Led; 16] = [
    Led::RgbCc(CC_PLAY),
    Led::RgbCc(CC_REC),
    Led::WhiteCc(CC_SHIFT),
    Led::WhiteCc(CC_MENU),
    Led::WhiteCc(CC_BACK),
    Led::WhiteCc(CC_UNDO),
    Led::WhiteCc(CC_LOOP),
    Led::WhiteCc(CC_COPY),
    Led::WhiteCc(CC_MUTE),
    Led::WhiteCc(CC_DELETE),
    Led::WhiteCc(CC_LEFT),
    Led::WhiteCc(CC_RIGHT),
    Led::WhiteCc(CC_DOWN),
    Led::WhiteCc(CC_UP),
    Led::WhiteCc(CC_CAPTURE),
    Led::RgbCc(CC_SAMPLE),
];

/// Number of LEDs in a frame.
pub const NUM_LEDS: usize = LED_BUTTONS + BUTTON_LEDS.len();

/// The LED at position `i` of a frame: 32 pads, 16 steps, 4 track buttons (1 to 4), buttons.
pub fn led(i: usize) -> Led {
    match i {
        0..=31 => Led::Note(PAD_BASE + i as u8),
        32..=47 => Led::Note(STEP_BASE + (i - LED_STEPS) as u8),
        48..=51 => Led::RgbCc(CC_TRACK1 - (i - LED_TRACKS) as u8),
        _ => BUTTON_LEDS[(i - LED_BUTTONS).min(BUTTON_LEDS.len() - 1)],
    }
}

/// Move's 128-colour palette (index → RGB), from the table in Schwung's `constants.mjs`.
/// Duplicates of black and white are left out so that the nearest match picks the main entry.
const PALETTE: &[(u8, u32)] = &[
    (0, 0x000000),
    (1, 0xFF4032),
    (2, 0x800400),
    (3, 0xC93C00),
    (4, 0xAC1F00),
    (5, 0x8C5018),
    (6, 0x491804),
    (7, 0xFADC3B),
    (8, 0xFFC516),
    (9, 0xB6FF0E),
    (10, 0x79FF18),
    (11, 0x34C216),
    (12, 0x4F8A04),
    (13, 0x62FF55),
    (14, 0x297D53),
    (15, 0x269E72),
    (16, 0x31ADFF),
    (17, 0x3663FC),
    (18, 0x1A34FF),
    (19, 0x1C0CE6),
    (20, 0x153999),
    (21, 0x3937FF),
    (22, 0x5722FF),
    (23, 0x972BFF),
    (24, 0x852178),
    (25, 0xFF1032),
    (26, 0xFF2BD4),
    (27, 0xA63421),
    (28, 0x995628),
    (29, 0x876700),
    (30, 0x90821F),
    (31, 0x4A8700),
    (32, 0x007F12),
    (33, 0x1853B2),
    (34, 0x624BAD),
    (35, 0x733A67),
    (36, 0xF8BCAF),
    (37, 0xFF9B76),
    (38, 0xFFBF5F),
    (39, 0xD9AF71),
    (40, 0xFFF480),
    (41, 0xBFBA69),
    (42, 0xBCCC88),
    (43, 0xAEFF99),
    (44, 0x7CDD9F),
    (45, 0x89B47D),
    (46, 0x80F3FF),
    (47, 0x7ACEFC),
    (48, 0x68A1D3),
    (49, 0x858FC2),
    (50, 0xBBAAF2),
    (51, 0xCDBBE4),
    (52, 0xEF8BB0),
    (53, 0x859D8C),
    (54, 0x6B756E),
    (55, 0x84909B),
    (56, 0x6A7075),
    (57, 0x88859D),
    (58, 0x6C6A75),
    (59, 0x9D859C),
    (60, 0x746A74),
    (61, 0x9C9D85),
    (62, 0x74756A),
    (63, 0x9D8484),
    (64, 0x756A6A),
    (65, 0x661914),
    (66, 0x210806),
    (67, 0x460300),
    (68, 0x280000),
    (69, 0x5D1700),
    (70, 0x200D00),
    (71, 0x470C00),
    (72, 0x1C0800),
    (73, 0x3B2B14),
    (74, 0x1C130A),
    (75, 0x250E05),
    (76, 0x0D0602),
    (77, 0x645817),
    (78, 0x201C07),
    (79, 0x664E08),
    (80, 0x211902),
    (81, 0x486605),
    (82, 0x172101),
    (83, 0x306609),
    (84, 0x0F2103),
    (85, 0x144D08),
    (86, 0x061902),
    (87, 0x1F3701),
    (88, 0x0A1100),
    (89, 0x276622),
    (90, 0x0C210B),
    (91, 0x143E29),
    (92, 0x081910),
    (93, 0x004D36),
    (94, 0x00180E),
    (95, 0x134566),
    (96, 0x061621),
    (97, 0x152764),
    (98, 0x070C20),
    (99, 0x0A1466),
    (100, 0x030621),
    (101, 0x0B045C),
    (102, 0x03011D),
    (103, 0x0A1C4C),
    (104, 0x040B1E),
    (105, 0x161666),
    (106, 0x070721),
    (107, 0x220D66),
    (108, 0x0B0421),
    (109, 0x3C1166),
    (110, 0x130521),
    (111, 0x350D30),
    (112, 0x11040F),
    (113, 0x660614),
    (114, 0x210206),
    (115, 0x661154),
    (116, 0x21051B),
    (118, 0x595959),
    (119, 0x1A1A1A),
    (120, 0xFFFFFF),
    (122, 0xCCCCCC),
    (123, 0x404040),
    (124, 0x141414),
    (125, 0x0000FF),
    (126, 0x00FF00),
    (127, 0xFF0000),
];

/// The palette index closest to an RGB colour (squared distance, with black kept for very dark
/// colours so that "off" stays off).
pub fn palette_index(rgb: (u8, u8, u8)) -> u8 {
    if rgb.0 < 8 && rgb.1 < 8 && rgb.2 < 8 {
        return 0;
    }
    let mut best = (u32::MAX, 0u8);
    for &(i, c) in PALETTE.iter().skip(1) {
        let (r, g, b) = ((c >> 16) as i32, ((c >> 8) & 0xff) as i32, (c & 0xff) as i32);
        let d = (r - rgb.0 as i32).pow(2) + (g - rgb.1 as i32).pow(2) + (b - rgb.2 as i32).pow(2);
        if (d as u32) < best.0 {
            best = (d as u32, i);
        }
    }
    best.1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn addressing() {
        assert_eq!(pad_index(68), Some(0));
        assert_eq!(pad_index(99), Some(31));
        assert_eq!(pad_index(100), None);
        assert_eq!(step_index(16), Some(0));
        assert_eq!(step_index(31), Some(15));
        assert_eq!(led(0), Led::Note(68));
        assert_eq!(led(47), Led::Note(31));
        assert_eq!(led(48), Led::RgbCc(43));
        assert_eq!(led(51), Led::RgbCc(40));
        assert_eq!(led(52), Led::RgbCc(85));
        assert_eq!(led(NUM_LEDS - 1), Led::RgbCc(118));
        assert_eq!(delta(1), 1);
        assert_eq!(delta(127), -1);
        assert_eq!(delta(64), 0);
    }

    #[test]
    fn palette() {
        assert_eq!(palette_index((0, 0, 0)), 0);
        assert_eq!(palette_index((255, 255, 255)), 120);
        assert_eq!(palette_index((255, 0, 0)), 127);
        assert_eq!(palette_index((0, 255, 0)), 126);
        assert_eq!(palette_index((0, 0, 255)), 125);
    }
}
