// SPDX-License-Identifier: GPL-3.0-only

use lcxl3::{Button, Control, Input};
use lxr_params::{par, ParamKind, Voice};
use std::cell::RefCell;
use std::collections::{BTreeSet, HashMap};
use vdrum_surface::layout::{b, f, k, t, voice_param};
use vdrum_surface::*;

/// A plugin stand-in: parameter values, faders, hit counters.
#[derive(Default)]
struct Fake {
    params: RefCell<HashMap<u16, u16>>,
    faders: RefCell<[i32; 7]>,
    hits: RefCell<[u32; 7]>,
}

impl Fake {
    fn new() -> Fake {
        let f = Fake::default();
        for p in lxr_params::PARAMS.iter().chain(lxr_params::GLOBAL_PARAMS.iter()).chain(lxr_params::oh_params().iter()) {
            f.params.borrow_mut().insert(p.index, p.default);
        }
        *f.faders.borrow_mut() = [99; 7];
        f
    }
    fn apply(&self, actions: &[Action]) {
        for a in actions {
            match a {
                Action::SetParam { idx, value } => {
                    self.params.borrow_mut().insert(*idx, *value);
                }
                Action::SetVoiceFader { track, step } => self.faders.borrow_mut()[*track as usize] = *step,
                Action::Trigger { track, .. } => self.hits.borrow_mut()[*track as usize] += 1,
                _ => {}
            }
        }
    }
}

impl Model for Fake {
    fn param(&self, idx: u16) -> u16 {
        self.params.borrow().get(&idx).copied().unwrap_or(0)
    }
    fn voice_fader(&self, t: u8) -> i32 {
        self.faders.borrow()[t as usize]
    }
    fn kit_name(&self) -> String {
        "Init".into()
    }
    fn hits(&self, t: u8) -> u32 {
        self.hits.borrow()[t as usize]
    }
}

fn press(s: &mut Surface, m: &Fake, button: Button, now: u64) -> Vec<Action> {
    let a = s.handle(&Input::Button { button, pressed: true }, now, m);
    m.apply(&a);
    a
}
fn release(s: &mut Surface, m: &Fake, button: Button, now: u64) -> Vec<Action> {
    let a = s.handle(&Input::Button { button, pressed: false }, now, m);
    m.apply(&a);
    a
}
fn turn(s: &mut Surface, m: &Fake, row: u8, col: u8, delta: i8) -> Vec<Action> {
    let a = s.handle(&Input::Relative { row, col, delta }, 0, m);
    m.apply(&a);
    a
}
fn fader(s: &mut Surface, m: &Fake, col: u8, value: u8) -> Vec<Action> {
    let a = s.handle(&Input::Analogue { control: Control::Fader(col), value }, 0, m);
    m.apply(&a);
    a
}

fn params_of(l: &Layout) -> Vec<u16> {
    l.iter().filter_map(|(_, c)| if let Binding::Param(i) = c.binding { Some(i) } else { None }).collect()
}

#[test]
fn voice_modes_cover_every_parameter_of_their_voice() {
    let voices = [Voice::Drum1, Voice::Drum2, Voice::Drum3, Voice::Snare, Voice::Cymbal, Voice::HiHat, Voice::OpenHat];
    for (t0, voice) in voices.into_iter().enumerate() {
        let l = Layout::of(Mode::Voice(t0 as u8), false);
        let on_page: BTreeSet<u16> = params_of(&l).into_iter().collect();
        let mut want: BTreeSet<u16> = lxr_params::PARAMS
            .iter()
            .chain(lxr_params::oh_params().iter())
            .filter(|p| p.kind == ParamKind::Sound && p.voice == voice)
            .map(|p| p.index)
            .collect();
        // The open hat has its own decay now; the firmware's shared open decay is not used.
        want.remove(&par::VELOD6_OPEN);
        // The choke belongs to both hats.
        if t0 == 5 {
            want.insert(layout::p("hat_choke"));
        }
        assert_eq!(on_page, want, "mode {}: missing {:?}, extra {:?}", t0 + 1, want.difference(&on_page), on_page.difference(&want));
        // no parameter twice
        assert_eq!(params_of(&l).len(), on_page.len(), "mode {}", t0 + 1);
    }
}

#[test]
fn every_mode_builds_and_assigns_every_control() {
    for mode in Mode::ALL {
        for held in [false, true] {
            let l = Layout::of(mode, held);
            let n = l.iter().count();
            // 24 encoders, 8 faders, 16 buttons
            assert_eq!(n, 48, "{mode:?} held={held}");
        }
    }
    for n in 1..=16 {
        assert_eq!(Mode::from_number(n).map(|m| m.number()), Mode::from_number(n).map(|_| n));
    }
    assert!(Mode::from_number(13).is_none());
}

#[test]
fn shift_and_buttons_select_modes() {
    let m = Fake::new();
    let mut s = Surface::new();
    assert_eq!(s.mode(), Mode::Voice(0));
    press(&mut s, &m, Button::Shift, 0);
    press(&mut s, &m, Button::Bottom(0), 0); // button 9
    assert_eq!(s.mode(), Mode::MixFx);
    press(&mut s, &m, Button::Bottom(7), 0); // button 16
    assert_eq!(s.mode(), Mode::Global);
    press(&mut s, &m, Button::Bottom(4), 0); // 13: empty
    assert_eq!(s.mode(), Mode::Global);
    press(&mut s, &m, Button::Top(6), 0); // 7
    assert_eq!(s.mode(), Mode::Voice(6));
    release(&mut s, &m, Button::Shift, 0);
    // without Shift the top row is the voice's buttons, not mode select
    press(&mut s, &m, Button::Top(0), 0);
    assert_eq!(s.mode(), Mode::Voice(6));
    // track buttons step through the modes, skipping empty slots
    press(&mut s, &m, Button::TrackRight, 0);
    assert_eq!(s.mode(), Mode::Sequencer);
    s.set_mode(Mode::MorphMacro);
    press(&mut s, &m, Button::TrackRight, 0);
    assert_eq!(s.mode(), Mode::Global);
    press(&mut s, &m, Button::TrackRight, 0);
    assert_eq!(s.mode(), Mode::Voice(0));
    press(&mut s, &m, Button::TrackLeft, 0);
    assert_eq!(s.mode(), Mode::Global);
    // the logo button returns to the previous mode
    press(&mut s, &m, Button::Logo, 0);
    assert_eq!(s.mode(), Mode::Voice(0));
    press(&mut s, &m, Button::SoloArm, 0);
    assert_eq!(s.mode(), Mode::Sequencer);
    press(&mut s, &m, Button::MuteSelect, 0);
    assert_eq!(s.mode(), Mode::Perform);
}

#[test]
fn encoders_step_values_and_lists_need_several_detents() {
    let m = Fake::new();
    let mut s = Surface::new();
    let fine = voice_param(0, "fine");
    let before = m.param(fine);
    assert_eq!(turn(&mut s, &m, 0, 0, 3), vec![Action::SetParam { idx: fine, value: before + 3 }]);
    // clamps at the top
    turn(&mut s, &m, 0, 0, 127);
    assert_eq!(m.param(fine), 127);
    assert!(turn(&mut s, &m, 0, 0, 5).is_empty());
    // LFO wave (8 entries): 3 detents per entry by default
    let wave = voice_param(0, "lfo_wave");
    let w0 = m.param(wave);
    assert!(turn(&mut s, &m, 2, 3, 1).is_empty());
    assert!(turn(&mut s, &m, 2, 3, 1).is_empty());
    assert_eq!(turn(&mut s, &m, 2, 3, 1), vec![Action::SetParam { idx: wave, value: w0 + 1 }]);
}

#[test]
fn lfo_destination_offers_the_destination_voices_targets() {
    let m = Fake::new();
    let mut s = Surface::new();
    let dest = voice_param(0, "lfo_dest");
    m.params.borrow_mut().insert(voice_param(0, "lfo_voice"), 2); // Drum 2
    m.params.borrow_mut().insert(dest, 0);
    turn(&mut s, &m, 2, 7, 1);
    let (lo, _) = lxr_params::ParamInfo::mod_target_range_for_voice(1);
    assert_eq!(m.param(dest), lo as u16);
    turn(&mut s, &m, 2, 7, -5);
    assert_eq!(m.param(dest), 0);
}

#[test]
fn faders_pick_up_before_they_change_a_parameter() {
    let m = Fake::new();
    let mut s = Surface::new();
    let coarse = voice_param(0, "coarse");
    m.params.borrow_mut().insert(coarse, 64);
    // far below the value: nothing happens, the popup says which way to move
    assert!(fader(&mut s, &m, 0, 10).is_empty());
    let fr = s.frame(0, &m);
    assert!(fr.popup.unwrap().fields[2].contains("move up"));
    assert!(fader(&mut s, &m, 0, 40).is_empty());
    // crossing the value picks it up
    assert_eq!(fader(&mut s, &m, 0, 70), vec![Action::SetParam { idx: coarse, value: 70 }]);
    assert_eq!(fader(&mut s, &m, 0, 20), vec![Action::SetParam { idx: coarse, value: 20 }]);
    // a mode change drops the pickup
    s.set_mode(Mode::Voice(1));
    s.set_mode(Mode::Voice(0));
    m.params.borrow_mut().insert(coarse, 100);
    assert!(fader(&mut s, &m, 0, 30).is_empty());
    // jump mode applies at once
    s.settings.fader_pickup = false;
    assert_eq!(fader(&mut s, &m, 0, 31), vec![Action::SetParam { idx: coarse, value: 31 }]);
}

#[test]
fn voice_faders_in_perform_mode() {
    let m = Fake::new();
    let mut s = Surface::new();
    s.set_mode(Mode::Perform);
    // fader at 99 = raw 127
    assert_eq!(fader(&mut s, &m, 2, 127), vec![]);
    assert_eq!(fader(&mut s, &m, 2, 0), vec![Action::SetVoiceFader { track: 2, step: 0 }]);
    assert_eq!(m.voice_fader(2), 0);
}

#[test]
fn buttons_cycle_and_toggle_parameters() {
    let m = Fake::new();
    let mut s = Surface::new();
    let ftype = voice_param(0, "ftype");
    let info = lxr_params::info(ftype).unwrap();
    m.params.borrow_mut().insert(ftype, info.max);
    press(&mut s, &m, Button::Top(5), 0);
    assert_eq!(m.param(ftype), info.min, "cycle wraps");
    let velvol = voice_param(0, "velvol");
    let v0 = m.param(velvol);
    press(&mut s, &m, Button::Top(6), 0);
    assert_ne!(m.param(velvol), v0);
    press(&mut s, &m, Button::Top(6), 0);
    assert_eq!(m.param(velvol), v0);
}

#[test]
fn pads_trigger_and_accent_plays_full_velocity() {
    let m = Fake::new();
    let mut s = Surface::new();
    assert_eq!(press(&mut s, &m, Button::Bottom(3), 0), vec![Action::Trigger { track: 3, velocity: 100 }]);
    press(&mut s, &m, Button::Bottom(7), 0);
    assert_eq!(press(&mut s, &m, Button::Bottom(6), 0), vec![Action::Trigger { track: 6, velocity: 127 }]);
    release(&mut s, &m, Button::Bottom(7), 0);
    assert_eq!(press(&mut s, &m, Button::Bottom(6), 0), vec![Action::Trigger { track: 6, velocity: 100 }]);
    // Play auditions the current voice
    assert_eq!(press(&mut s, &m, Button::Play, 0), vec![Action::Trigger { track: 0, velocity: 100 }]);
}

#[test]
fn pads_flash_when_their_track_plays() {
    let m = Fake::new();
    let mut s = Surface::new();
    let led = |s: &Surface, now| s.frame(now, &m).leds.iter().find(|(c, _)| *c == b(2)).unwrap().1;
    s.tick(0, &m);
    let idle = led(&s, 0);
    m.hits.borrow_mut()[1] += 1; // e.g. host MIDI
    s.tick(1000, &m);
    assert_eq!(led(&s, 1010), (255, 255, 255));
    assert_eq!(led(&s, 1000 + 500), idle);
}

#[test]
fn global_settings_send_feature_controls() {
    let m = Fake::new();
    let mut s = Surface::new();
    s.set_mode(Mode::Global);
    let a = turn(&mut s, &m, 2, 0, -27);
    assert_eq!(a, vec![Action::Device(vec![0xB6, 0x6F, 100])]);
    let a = turn(&mut s, &m, 2, 3, 1);
    assert_eq!(a, vec![Action::Device(vec![0xB6, 0x79, 2])]);
    turn(&mut s, &m, 2, 4, -50);
    assert_eq!(s.settings.pad_velocity, 50);
}

#[test]
fn device_state_sends_only_changes() {
    let m = Fake::new();
    let mut s = Surface::new();
    let mut d = DeviceState::default();
    let init = d.connect();
    assert_eq!(init[0], vec![0x9F, 0x0C, 0x7F]);
    assert!(init.contains(&vec![0xB6, 0x45, 0x7F]));
    let first = d.update(&s.frame(0, &m));
    // every LED once, the stationary display, no popup yet
    assert!(first.len() >= vdrum_surface::frame::led_controls().len());
    assert!(d.update(&s.frame(0, &m)).is_empty());
    turn(&mut s, &m, 0, 0, 1);
    let msgs = d.update(&s.frame(0, &m));
    // popup (3 fields + trigger) and the stationary's last-touched line (3 fields + trigger)
    assert_eq!(msgs.len(), 8, "{msgs:x?}");
    assert!(msgs.iter().all(|m| m[..6] == lcxl3::SYSEX_HEADER));
    assert!(d.update(&s.frame(0, &m)).is_empty());
    // leaving DAW mode
    assert_eq!(d.disconnect().last(), Some(&vec![0x9F, 0x0C, 0x00]));
}

#[test]
fn spare_controls_are_dark_and_toggles_dim_when_off() {
    let m = Fake::new();
    let s = Surface::new();
    let fr = s.frame(0, &m);
    let led = |c| fr.leds.iter().find(|(x, _)| *x == c).map(|x| x.1);
    assert_eq!(led(k(2, 5)), Some((0, 0, 0)));
    let velvol = voice_param(0, "velvol");
    let on = m.param(velvol) != lxr_params::info(velvol).unwrap().min;
    let col = Section::Velo.colour();
    assert_eq!(led(t(7)), Some(if on { col } else { vdrum_surface::frame::dim(col) }));
    assert!(led(f(1)).is_none(), "faders have no LED");
}

#[test]
fn settings_round_trip_as_text() {
    let s = Settings { pad_velocity: 64, list_speed: 5, fader_pickup: false, host_clock: false, ..Settings::default() };
    assert_eq!(Settings::from_text(&s.to_text()), s);
    assert_eq!(Settings::from_text("pad_velocity=0;bogus=1;list_speed=x"), Settings { pad_velocity: 1, ..Settings::default() });
}
