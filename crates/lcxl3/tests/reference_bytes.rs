// SPDX-License-Identifier: GPL-3.0-only
//! Byte strings printed in the Launch Control XL 3 Programmer's Reference Guide v1.0, and the CC map
//! of its page 9 surface diagram.

use lcxl3::display::{self, arrangement, Bitmap};
use lcxl3::encode::*;
use lcxl3::*;

const H: [u8; 6] = [0xF0, 0x00, 0x20, 0x29, 0x02, 0x15];

fn sx(body: &[u8]) -> Vec<u8> {
    let mut v = H.to_vec();
    v.extend_from_slice(body);
    v.push(0xF7);
    v
}

#[test]
fn header_page_6() {
    assert_eq!(SYSEX_HEADER, [240, 0, 32, 41, 2, 21]);
}

#[test]
fn daw_mode_page_8() {
    assert_eq!(daw_mode(true), [0x9F, 0x0C, 0x7F]);
    assert_eq!(daw_mode(false), [0x9F, 0x0C, 0x00]);
    assert_eq!(daw_mode_sysex(true), vec![0xF0, 0x00, 0x20, 0x29, 0x02, 0x15, 0x02, 0x7F, 0xF7]);
    assert_eq!(daw_mode_sysex(false), vec![240, 0, 32, 41, 2, 21, 2, 0, 247]);
}

#[test]
fn feature_controls_page_16() {
    assert_eq!(feature_controls(true), [0x9F, 0x0B, 0x7F]);
    assert_eq!(feature_controls(false), [159, 11, 0]);
}

#[test]
fn surface_cc_map_page_9() {
    // encoders: rows 13-20, 21-28, 29-36; faders 5-12 (channel 16)
    for col in 0..8u8 {
        assert_eq!(Control::Encoder { row: 0, col }.cc(), Some(13 + col));
        assert_eq!(Control::Encoder { row: 1, col }.cc(), Some(21 + col));
        assert_eq!(Control::Encoder { row: 2, col }.cc(), Some(29 + col));
        assert_eq!(Control::Fader(col).cc(), Some(5 + col));
        assert_eq!(Button::Top(col).cc(), Some(37 + col));
        assert_eq!(Button::Bottom(col).cc(), Some(45 + col));
    }
    let named = [
        (Button::SoloArm, 65),
        (Button::MuteSelect, 66),
        (Button::Logo, 104),
        (Button::PageUp, 106),
        (Button::PageDown, 107),
        (Button::TrackLeft, 103),
        (Button::TrackRight, 102),
        (Button::Record, 118),
        (Button::Play, 116),
        (Button::Shift, 63),
    ];
    for (b, cc) in named {
        assert_eq!(b.cc(), Some(cc), "{b:?}");
        if b != Button::Shift {
            assert_eq!(Button::from_cc(cc), Some(b));
            assert_eq!(b.channel(), 0);
        }
    }
    assert_eq!(Button::Shift.channel(), 6);
    assert_eq!(Button::from_cc(63), None, "Shift is not on channel 1");
    assert_eq!(Control::Encoder { row: 3, col: 0 }.cc(), None);
    assert_eq!(Control::Fader(8).cc(), None);
}

#[test]
fn decode_surface_messages() {
    // encoders and faders on channel 16
    assert_eq!(decode(&[0xBF, 13, 64]), Input::Analogue { control: Control::Encoder { row: 0, col: 0 }, value: 64 });
    assert_eq!(decode(&[0xBF, 36, 127]), Input::Analogue { control: Control::Encoder { row: 2, col: 7 }, value: 127 });
    assert_eq!(decode(&[0xBF, 5, 0]), Input::Analogue { control: Control::Fader(0), value: 0 });
    assert_eq!(decode(&[0xBF, 12, 1]), Input::Analogue { control: Control::Fader(7), value: 1 });
    // buttons on channel 1
    assert_eq!(decode(&[0xB0, 37, 127]), Input::Button { button: Button::Top(0), pressed: true });
    assert_eq!(decode(&[0xB0, 52, 0]), Input::Button { button: Button::Bottom(7), pressed: false });
    assert_eq!(decode(&[0xB0, 116, 127]), Input::Button { button: Button::Play, pressed: true });
    // Shift on channel 7
    assert_eq!(decode(&[0xB6, 63, 127]), Input::Button { button: Button::Shift, pressed: true });
    // the same CCs on other channels are not surface controls
    assert_eq!(decode(&[0xB1, 37, 127]), Input::Other);
    assert_eq!(decode(&[0xB0, 13, 1]), Input::Other);
    assert_eq!(decode(&[0x9F, 0x0C, 0x7F]), Input::Other);
    assert_eq!(decode(&[0xBF, 5]), Input::Other);
}

#[test]
fn surface_mode_page_10_and_17() {
    assert_eq!(surface_mode(SurfaceMode::DawMixer), Some([0xB6, 0x1E, 0x01]));
    assert_eq!(surface_mode(SurfaceMode::DawControl), Some([182, 30, 2]));
    assert_eq!(surface_mode(SurfaceMode::Custom(1)), Some([0xB6, 0x1E, 0x06]));
    assert_eq!(surface_mode(SurfaceMode::Custom(4)), Some([0xB6, 0x1E, 0x09]));
    assert_eq!(surface_mode(SurfaceMode::Custom(5)), Some([0xB6, 0x1E, 0x12]));
    assert_eq!(surface_mode(SurfaceMode::Custom(16)), Some([0xB6, 0x1E, 0x1D]));
    assert_eq!(surface_mode(SurfaceMode::Custom(0)), None);
    assert_eq!(surface_mode(SurfaceMode::Custom(17)), None);
    assert_eq!(decode(&[0xB6, 0x1E, 0x01]), Input::SurfaceMode(SurfaceMode::DawMixer));
    assert_eq!(decode(&[0xB6, 0x1E, 0x02]), Input::SurfaceMode(SurfaceMode::DawControl));
    for v in 0..128u8 {
        if let Some(m) = SurfaceMode::from_value(v) {
            assert_eq!(m.value(), Some(v));
        }
    }
    assert_eq!(SurfaceMode::from_value(0x0A), None);
    assert_eq!(SurfaceMode::from_value(0x1E), None);
}

#[test]
fn relative_encoders_page_10() {
    assert_eq!(relative_row(0, true), Some([0xB6, 0x45, 0x7F]));
    assert_eq!(relative_row(1, true), Some([182, 72, 127]));
    assert_eq!(relative_row(2, false), Some([0xB6, 0x49, 0x00]));
    assert_eq!(relative_row(3, true), None);
    // row 1: 4Dh-54h, row 2: 55h-5Ch, row 3: 5Dh-64h; pivot 40h; 41h = 1 step cw, 3Fh = 1 step ccw
    assert_eq!(decode(&[0xBF, 0x4D, 0x41]), Input::Relative { row: 0, col: 0, delta: 1 });
    assert_eq!(decode(&[0xBF, 0x54, 0x3F]), Input::Relative { row: 0, col: 7, delta: -1 });
    assert_eq!(decode(&[0xBF, 0x55, 0x40]), Input::Relative { row: 1, col: 0, delta: 0 });
    assert_eq!(decode(&[0xBF, 0x5C, 0x7F]), Input::Relative { row: 1, col: 7, delta: 63 });
    assert_eq!(decode(&[0xBF, 0x5D, 0x00]), Input::Relative { row: 2, col: 0, delta: -64 });
    assert_eq!(decode(&[0xBF, 0x64, 0x42]), Input::Relative { row: 2, col: 7, delta: 2 });
    assert_eq!(decode(&[0xBF, 0x65, 0x41]), Input::Other);
}

#[test]
fn touch_events_page_10_and_11() {
    assert_eq!(touch_events(true), [0xB6, 0x47, 0x7F]);
    assert_eq!(touch_events(false), [182, 71, 0]);
    // "the leftmost Fader would send BEh 05h 7Fh for Touch On, and BEh 05h 00h for Touch Off"
    assert_eq!(decode(&[0xBE, 0x05, 0x7F]), Input::Touch { control: Control::Fader(0), on: true });
    assert_eq!(decode(&[0xBE, 0x05, 0x00]), Input::Touch { control: Control::Fader(0), on: false });
    assert_eq!(decode(&[0xBE, 0x24, 0x7F]), Input::Touch { control: Control::Encoder { row: 2, col: 7 }, on: true });
}

#[test]
fn led_colour_page_11_and_12() {
    assert_eq!(led_palette(Control::Button(Button::Top(0)), 5), Some([0xB0, 37, 5]));
    assert_eq!(led_palette(Control::Encoder { row: 1, col: 2 }, 0x7F), Some([176, 23, 127]));
    assert_eq!(led_palette(Control::Button(Button::Shift), 5), None);
    assert_eq!(
        led_rgb(Control::Button(Button::Bottom(0)), 0x7F, 0x00, 0x20),
        Some(sx(&[0x01, 0x53, 45, 0x7F, 0x00, 0x20]))
    );
    // encoders by absolute index in either mode
    assert_eq!(led_rgb(Control::Encoder { row: 0, col: 0 }, 1, 2, 3), Some(sx(&[1, 83, 13, 1, 2, 3])));
    // components are 7-bit
    assert_eq!(led_rgb(Control::Button(Button::Play), 0xFF, 0x80, 0x00), Some(sx(&[1, 83, 116, 0x7F, 0x00, 0x00])));
    assert_eq!(led_rgb(Control::Fader(0), 1, 2, 3), None);
    assert_eq!(led_rgb(Control::Button(Button::Shift), 1, 2, 3), None);
}

#[test]
fn configure_display_page_13() {
    let enc = DisplayTarget::Control(Control::Encoder { row: 0, col: 0 });
    // default arrangement 4 with both auto flags (bits 6 and 5)
    assert_eq!(configure_display(enc, DisplayConfig::arrangement(arrangement::NAME_NUMERIC)), Some(sx(&[0x04, 0x0D, 0x64])));
    assert_eq!(configure_display(DisplayTarget::Stationary, DisplayConfig::Cancel), Some(sx(&[4, 0x35, 0x00])));
    assert_eq!(configure_display(DisplayTarget::Overlay, DisplayConfig::Trigger), Some(sx(&[4, 0x36, 0x7F])));
    let plain = DisplayConfig::Arrange { arrangement: arrangement::TITLE_NAME_VALUE, auto_on_change: false, auto_on_touch: false };
    assert_eq!(configure_display(DisplayTarget::Control(Control::Fader(7)), plain), Some(sx(&[4, 0x0C, 0x02])));
    let change_only = DisplayConfig::Arrange { arrangement: 1, auto_on_change: true, auto_on_touch: false };
    assert_eq!(change_only.byte(), Some(0x41));
    // arrangement 0 and 31 are the special cancel/trigger values, not arrangements
    assert_eq!(DisplayConfig::arrangement(0).byte(), None);
    assert_eq!(DisplayConfig::arrangement(31).byte(), None);
    assert_eq!(DisplayTarget::Control(Control::Button(Button::Play)).byte(), None);
}

#[test]
fn display_text_page_14() {
    let t = DisplayTarget::Stationary;
    assert_eq!(display_text(t, 0, "Kick"), Some(sx(&[0x06, 0x35, 0x00, b'K', b'i', b'c', b'k'])));
    // reassigned control codes
    assert_eq!(display_text(t, 1, "□■♭♥"), Some(sx(&[6, 0x35, 1, 0x1B, 0x1C, 0x1D, 0x1E])));
    // range 20h-7Eh passes, everything else becomes '?'
    assert_eq!(display::encode_text(" ~\u{7f}é\n").collect::<Vec<_>>(), vec![0x20, 0x7E, b'?', b'?', b'?']);
    assert_eq!(display_text(t, 0x80, "x"), None);
}

#[test]
fn bitmap_page_14_and_15() {
    assert_eq!(display::BITMAP_BYTES, 1216);
    let mut img = Bitmap::default();
    img.set(0, 0, true); // leftmost pixel: bit 6 of byte 0
    img.set(6, 0, true); // bit 0 of byte 0
    img.set(7, 0, true); // bit 6 of byte 1
    img.set(127, 63, true); // pixel 127 = byte 18 (pixels 126..132), bit 6 - 1
    img.set(128, 0, true); // outside: ignored
    let data = img.pack();
    assert_eq!(&data[..2], &[0x41, 0x40]);
    assert_eq!(data[63 * 19 + 18], 0x20);
    assert_eq!(data.iter().filter(|b| **b != 0).count(), 3);
    assert!(data.iter().all(|b| *b < 0x80));
    assert_eq!(Bitmap::unpack(&data), img);

    let m = bitmap(BITMAP_STATIONARY, &img).unwrap();
    assert_eq!(m.len(), 6 + 1 + 1 + 1216 + 1);
    assert_eq!(&m[..8], &[0xF0, 0x00, 0x20, 0x29, 0x02, 0x15, 0x09, 0x20]);
    assert_eq!(&m[8..8 + 1216], &data[..]);
    assert_eq!(m.last(), Some(&0xF7));
    assert_eq!(bitmap(BITMAP_TEMPORARY, &img).unwrap()[7], 0x21);

    // the reply: F0 00 20 29 02 15 09 7F
    assert_eq!(decode(&[0xF0, 0x00, 0x20, 0x29, 0x02, 0x15, 0x09, 0x7F]), Input::BitmapAck);
    assert_eq!(decode(&[240, 0, 32, 41, 2, 21, 9, 127, 247]), Input::BitmapAck);
    assert_eq!(decode(&[0xF0, 0x00, 0x20, 0x29, 0x02, 0x14, 0x09, 0x7F, 0xF7]), Input::Other);
}

#[test]
fn feature_table_page_17() {
    let table = [
        (Feature::SurfaceMode, 0x1E),
        (Feature::Shift, 0x3F),
        (Feature::RelativeRow(0), 0x45),
        (Feature::RelativeRow(1), 0x48),
        (Feature::RelativeRow(2), 0x49),
        (Feature::FaderPickup, 0x46),
        (Feature::TouchEvents, 0x47),
        (Feature::LedBrightness, 0x6F),
        (Feature::ScreenBrightness, 0x70),
        (Feature::DisplayTimeout, 0x71),
        (Feature::Out2Thru, 0x78),
        (Feature::GlobalChannel, 0x64),
        (Feature::EncoderCurve, 0x79),
    ];
    assert_eq!(table.len(), Feature::ALL.len());
    for (f, cc) in table {
        assert_eq!(f.cc(), Some(cc), "{f:?}");
        assert_eq!(Feature::from_cc(cc), Some(f));
        assert_eq!(set_feature(f, 0x40), Some([0xB6, cc, 0x40]));
        assert_eq!(query_feature(f), Some([0xB7, cc, 0x00]));
    }
    assert_eq!(set_feature(Feature::EncoderCurve, EncoderCurve::Fast as u8), Some([0xB6, 0x79, 0x02]));
    assert_eq!(decode(&[0xB6, 0x6F, 0x7F]), Input::Feature { feature: Feature::LedBrightness, value: 127 });
    assert_eq!(decode(&[0xB6, 0x46, 0x00]), Input::Feature { feature: Feature::FaderPickup, value: 0 });
    assert_eq!(decode(&[0xB6, 0x01, 0x00]), Input::Other);
}

#[test]
fn every_three_byte_message_decodes_without_panicking() {
    for s in 0x80..=0xFFu8 {
        for a in 0..=0xFFu8 {
            for b in [0u8, 1, 63, 64, 65, 127, 128, 255] {
                let _ = decode(&[s, a, b]);
            }
        }
    }
    let _ = decode(&[]);
    let _ = decode(&[0xF0]);
    let _ = decode(&[0xF7]);
}
