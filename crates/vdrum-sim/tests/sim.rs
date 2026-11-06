// SPDX-License-Identifier: GPL-3.0-only
//! The simulator end to end: gestures on the virtual device, the MIDI between device and surface,
//! what the device shows, a real device attached through MIDI in/out, and live spec reload.

use lcxl3::{Button, Control};
use serde_json::{json, Value};
use vdrum_sim::Sim;
use vdrum_surface::spec::BUILTIN_JSON;
use vdrum_surface::Model;

fn sim() -> Sim {
    Sim::default()
}

fn hold(s: &mut Sim, b: Button, f: impl FnOnce(&mut Sim)) {
    s.button(b, true);
    f(s);
    s.button(b, false);
}

fn press(s: &mut Sim, b: Button) {
    s.button(b, true);
    s.button(b, false);
}

fn screen(s: &Sim) -> Vec<String> {
    s.device.screen(s.now()).lines
}

#[test]
fn connect_puts_the_device_into_daw_mode_with_relative_encoders() {
    let s = sim();
    assert!(s.device.daw_mode);
    assert_eq!(s.device.relative, [true; 3]);
    // device settings: LED and screen brightness, popup time 1 s, encoder curve
    assert_eq!(s.device.features.get(&111), Some(&127));
    assert_eq!(s.device.features.get(&113), Some(&10));
    assert_eq!(s.device.features.get(&121), Some(&1));
    // the surface switched the device's own popups off on every control
    assert!(s.device.controls.values().all(|d| !d.auto_on_change && !d.auto_on_touch));
    assert_eq!(s.device.controls.len(), 32);
    assert!(s.device.unknown.is_empty(), "{:?}", s.device.unknown);
    assert_eq!(screen(&s), ["1 Drum 1", "Init", ""]);
}

#[test]
fn leds_on_the_device_are_the_frame_colours() {
    let s = sim();
    let f = s.surface.frame(s.now(), &s.model);
    for (c, rgb) in f.leds {
        // 8-bit colours go to the device as 7 bits; the emulator draws them back at 8 bits.
        assert_eq!(s.device.led(c).rgb8(), (rgb.0 >> 1 << 1, rgb.1 >> 1 << 1, rgb.2 >> 1 << 1), "{c:?}");
    }
}

#[test]
fn encoder_turn_goes_through_midi_changes_the_parameter_and_shows_a_popup() {
    let mut s = sim();
    let idx = lxr_params::by_id("fine1").unwrap().index;
    let before = s.model.param(idx);
    s.turn(0, 0, 3);
    assert_eq!(s.model.param(idx), before + 3);
    // the device sent one relative message per detent on channel 16, CC 13 + 64
    let ins: Vec<&str> = s.log().iter().filter(|e| e.kind == "midi_in").map(|e| e.text.as_str()).collect();
    assert_eq!(ins, ["BF 4D 41", "BF 4D 41", "BF 4D 41"]);
    assert!(s.log().iter().any(|e| e.kind == "param" && e.text.starts_with("fine1 ")));
    assert_eq!(s.device.screen(s.now()).source, "overlay");
    assert_eq!(screen(&s)[1], "Fine");
    // the popup goes after the device's timeout (CC 113 = 10: 1 s), and the stationary display,
    // which now shows the last touched control, returns
    let t = s.now();
    s.tick(t + 999);
    assert_eq!(s.device.screen(s.now()).source, "overlay");
    s.tick(t + 1000);
    assert_eq!(s.device.screen(s.now()).source, "stationary");
    assert_eq!(screen(&s)[0], "1 Drum 1");
}

#[test]
fn shift_and_a_button_select_a_mode_and_the_rule_is_logged() {
    let mut s = sim();
    hold(&mut s, Button::Shift, |s| press(s, Button::Top(7)));
    assert_eq!(s.surface.mode(), vdrum_surface::Mode::Sequencer);
    assert_eq!(screen(&s), ["8 Sequencer", "P01 bar 1/4", "Track 1 D1"]);
    let rules: Vec<&str> = s.log().iter().filter(|e| e.kind == "rule").map(|e| e.text.as_str()).collect();
    assert_eq!(rules[1], "navigation rule 3: Shift + top N: mode slot N");
    // the Shift message is CC 63 on channel 7
    assert!(s.log().iter().any(|e| e.kind == "midi_in" && e.text == "B6 3F 7F"));
}

#[test]
fn sequencer_runs_on_the_internal_clock() {
    let mut s = sim();
    s.surface.settings.host_clock = false;
    hold(&mut s, Button::Shift, |s| press(s, Button::Top(7)));
    press(&mut s, Button::Top(0)); // step 1 on
    press(&mut s, Button::Play);
    assert!(s.model.seq_status().running);
    let t = s.now();
    s.tick(t + 10);
    // step 1 is playing: its button is white, and the track was hit
    assert_eq!(s.device.led(Control::Button(Button::Top(0))).rgb8(), (254, 254, 254));
    assert_eq!(s.model.hits(0), 1);
    s.tick(t + 2010); // 16 sixteenths at 120 bpm: one more pass
    assert_eq!(s.model.hits(0), 2);
    assert_eq!(s.device.led(Control::Button(Button::Play)).rgb8(), (0, 254, 0));
}

#[test]
fn a_real_device_gets_the_same_messages_and_drives_the_surface() {
    let mut s = sim();
    s.command(&json!({"cmd": "hardware", "on": true}));
    let out = s.take_midi_out();
    // connect: DAW mode on first, then relative rows, display setup, settings, a full frame
    assert_eq!(out[0], lcxl3::encode::daw_mode(true).to_vec());
    let leds = out.iter().filter(|m| m.len() > 8 && m[6] == 0x01 && m[7] == 0x53).count();
    assert_eq!(leds, vdrum_surface::frame::led_controls().len());
    // the real device's message for Solo/Arm (CC 65 on channel 1)
    s.midi_in(&[0xB0, 65, 127]);
    s.midi_in(&[0xB0, 65, 0]);
    assert!(s.surface.seq_view());
    let out = s.take_midi_out();
    // the step buttons and Solo/Arm changed colour, and the popup was sent and triggered
    assert!(out.iter().any(|m| m[..8] == [0xF0, 0x00, 0x20, 0x29, 0x02, 0x15, 0x01, 0x53] && m[8] == 65));
    assert!(out.iter().any(|m| m[..8] == [0xF0, 0x00, 0x20, 0x29, 0x02, 0x15, 0x04, 0x36] && m[8] == 0x7F));
    // the virtual device mirrors the real one
    assert_eq!(s.device.screen(s.now()).lines[0], "Sequencer view");
}

#[test]
fn bitmap_messages_are_acknowledged_back_to_the_surface() {
    let mut s = sim();
    let mut b = lcxl3::Bitmap::default();
    b.set(0, 0, true);
    let msg = lcxl3::encode::bitmap(lcxl3::DisplayTarget::Stationary, &b).unwrap();
    let replies = s.device.receive(&msg, s.now());
    assert_eq!(replies, vec![vec![0xF0, 0x00, 0x20, 0x29, 0x02, 0x15, 0x09, 0x7F, 0xF7]]);
    assert_eq!(lcxl3::decode(&replies[0]), lcxl3::Input::BitmapAck);
}

#[test]
fn a_new_spec_is_used_at_once_and_a_bad_one_is_refused() {
    let mut s = sim();
    // Move Perform from slot 10 to slot 13 (Shift + bottom 5).
    let moved = BUILTIN_JSON.replacen("\"slot\": 10,", "\"slot\": 13,", 1);
    assert_ne!(moved, BUILTIN_JSON);
    s.load_spec(&moved).unwrap();
    hold(&mut s, Button::Shift, |s| press(s, Button::Bottom(4)));
    assert_eq!(s.surface.mode(), vdrum_surface::Mode::Perform);
    assert_eq!(screen(&s)[0], "13 Perform");
    let e = s.load_spec("{").unwrap_err();
    assert!(e.starts_with("spec:"), "{e}");
    assert_eq!(s.surface.spec().mode_info(vdrum_surface::Mode::Perform).0, 13, "the running spec stays");
    assert!(s.log().iter().any(|l| l.kind == "error"));
}

#[test]
fn json_commands_through_the_wasm_entry_point() {
    let r: Value = serde_json::from_str(&vdrum_sim::wasm::run(br#"{"cmd": "reset"}"#)).unwrap();
    assert_eq!(r["surface"]["name"], "Drum 1");
    let r: Value = serde_json::from_str(&vdrum_sim::wasm::run(br#"{"cmd": "button", "key": "SOLO", "pressed": true}"#)).unwrap();
    assert_eq!(r["surface"]["seq_view"], true);
    assert_eq!(r["leds"]["SOLO"], json!([254, 254, 254]));
    assert_eq!(r["screen"]["lines"][0], "Sequencer view");
    let r: Value = serde_json::from_str(&vdrum_sim::wasm::run(br#"{"cmd": "nope"}"#)).unwrap();
    assert!(r["error"].as_str().unwrap().contains("unknown command"));
    let r: Value = serde_json::from_str(&vdrum_sim::wasm::run(b"not json")).unwrap();
    assert!(r["error"].as_str().unwrap().starts_with("command:"));
    let log: Value = serde_json::from_str(&vdrum_sim::wasm::run(br#"{"cmd": "log", "from": 0}"#)).unwrap();
    assert!(log.as_array().unwrap().iter().any(|e| e["kind"] == "rule"));
}
