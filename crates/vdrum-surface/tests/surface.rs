// SPDX-License-Identifier: GPL-3.0-only

use lcxl3::{Button, Control, Input};
use lxr_params::{par, ParamKind, Voice};
use std::cell::RefCell;
use std::collections::{BTreeSet, HashMap};
use vdrum_surface::layout::{b, f, k, t, voice_param, TRACK_COLOURS};
use vdrum_surface::*;
use vdrum_surface::layout;

/// A plugin stand-in: parameter values, faders, hit counters.
#[derive(Default)]
struct Fake {
    params: RefCell<HashMap<u16, u16>>,
    faders: RefCell<[i32; 7]>,
    hits: RefCell<[u32; 7]>,
    switches: RefCell<[bool; 2]>,
    seq: RefCell<SeqData>,
    status: RefCell<SeqStatus>,
    mutes: RefCell<u8>,
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
                Action::SetSwitch(sw, on) => self.switches.borrow_mut()[(*sw == Switch::HwClock) as usize] = *on,
                Action::Seq(e) => {
                    self.seq.borrow_mut().apply(e);
                    let mut st = self.status.borrow_mut();
                    match *e {
                        SeqEdit::NextPattern(p) => {
                            st.pending = p;
                            if !st.running {
                                st.pattern = p;
                            }
                        }
                        SeqEdit::Run(r) => st.running = r,
                        _ => {}
                    }
                }
                Action::SetMutes(m) => *self.mutes.borrow_mut() = *m,
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
    fn switch(&self, sw: Switch) -> bool {
        self.switches.borrow()[(sw == Switch::HwClock) as usize]
    }
    fn seq(&self) -> Box<dyn std::ops::Deref<Target = SeqData> + '_> {
        Box::new(self.seq.borrow())
    }
    fn seq_status(&self) -> SeqStatus {
        *self.status.borrow()
    }
    fn mutes(&self) -> u8 {
        *self.mutes.borrow()
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
    // the track buttons do not change mode (only Shift + button does)
    press(&mut s, &m, Button::TrackRight, 0);
    press(&mut s, &m, Button::TrackLeft, 0);
    assert_eq!(s.mode(), Mode::Voice(6));
    s.set_mode(Mode::Voice(0));
    s.set_mode(Mode::Global);
    // the logo button returns to the previous mode
    press(&mut s, &m, Button::Logo, 0);
    assert_eq!(s.mode(), Mode::Voice(0));
    press(&mut s, &m, Button::SoloArm, 0);
    assert_eq!(s.mode(), Mode::Sequencer);
    // in mode 8 Mute/Select is a layer while held; a tap goes to Perform on release
    press(&mut s, &m, Button::MuteSelect, 0);
    assert_eq!(s.mode(), Mode::Sequencer);
    release(&mut s, &m, Button::MuteSelect, 0);
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
    // popup (3 fields + trigger), the stationary's last-touched line (3 fields + trigger) and the
    // encoder's LED, whose brightness follows the value
    assert_eq!(msgs.len(), 9, "{msgs:x?}");
    assert_eq!(msgs.iter().filter(|m| m[6..8] == [0x01, 0x53]).count(), 1, "{msgs:x?}");
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

fn tick(s: &mut Surface, m: &Fake, now: u64) -> Vec<Action> {
    let a = s.tick(now, m);
    m.apply(&a);
    a
}

#[test]
fn copy_and_paste_a_voice_between_voices_of_the_same_kind() {
    let m = Fake::new();
    let mut s = Surface::new();
    let (c1, c2) = (voice_param(0, "coarse"), voice_param(1, "coarse"));
    let (d1, d2) = (voice_param(0, "decay"), voice_param(1, "decay"));
    m.params.borrow_mut().insert(c1, 11);
    m.params.borrow_mut().insert(d1, 22);
    press(&mut s, &m, Button::Top(1), 0); // copy Drum 1
    release(&mut s, &m, Button::Top(1), 10);
    s.set_mode(Mode::Voice(1));
    press(&mut s, &m, Button::Top(2), 100); // paste: needs 1 s
    assert!(tick(&mut s, &m, 500).is_empty());
    let a = tick(&mut s, &m, 1100);
    assert!(a.contains(&Action::SetParam { idx: c2, value: 11 }));
    assert_eq!((m.param(c2), m.param(d2)), (11, 22));
    assert!(tick(&mut s, &m, 3000).is_empty(), "fires once per hold");
    release(&mut s, &m, Button::Top(2), 3000);
    // not onto the snare
    s.set_mode(Mode::Voice(3));
    press(&mut s, &m, Button::Top(2), 4000);
    assert!(tick(&mut s, &m, 5100).is_empty());
    assert_eq!(s.frame(5100, &m).popup.unwrap().fields[2], "different voice type");
}

#[test]
fn hats_copy_onto_each_other_but_not_the_shared_choke() {
    let m = Fake::new();
    let mut s = Surface::new();
    s.set_mode(Mode::Voice(5));
    let choke = layout::p("hat_choke");
    m.params.borrow_mut().insert(voice_param(5, "fine"), 3);
    press(&mut s, &m, Button::Top(1), 0);
    m.params.borrow_mut().insert(choke, 0);
    s.set_mode(Mode::Voice(6));
    press(&mut s, &m, Button::Top(2), 0);
    let a = tick(&mut s, &m, 1000);
    assert_eq!(m.param(voice_param(6, "fine")), 3);
    assert!(!a.iter().any(|x| matches!(x, Action::SetParam { idx, .. } if *idx == choke)));
}

#[test]
fn mix_fx_sends_tracks_to_the_fx_and_back() {
    let m = Fake::new();
    let mut s = Surface::new();
    s.set_mode(Mode::MixFx);
    let out3 = voice_param(2, "out");
    m.params.borrow_mut().insert(out3, 2);
    press(&mut s, &m, Button::Top(2), 0);
    assert_eq!(m.param(out3), 6);
    press(&mut s, &m, Button::Top(2), 0);
    assert_eq!(m.param(out3), 2, "previous output restored");
    // FX on/off restores the type
    m.params.borrow_mut().insert(par::FX_TYPE, 3);
    press(&mut s, &m, Button::Top(7), 0);
    assert_eq!(m.param(par::FX_TYPE), 0);
    press(&mut s, &m, Button::Top(7), 0);
    assert_eq!(m.param(par::FX_TYPE), 3);
}

#[test]
fn fx_sub_type_and_parameter_names_follow_the_fx_type() {
    let m = Fake::new();
    let mut s = Surface::new();
    s.set_mode(Mode::MixFx);
    m.params.borrow_mut().insert(par::FX_TYPE, 3); // compressor: ratio
    for _ in 0..3 {
        turn(&mut s, &m, 0, 5, 1);
    }
    assert_eq!(m.param(par::FX_RATIO), 1);
    m.params.borrow_mut().insert(par::FX_TYPE, 4); // delay: delay type
    for _ in 0..3 {
        turn(&mut s, &m, 0, 5, 1);
    }
    assert_eq!(m.param(par::FX_DELAY_TYPE), 1);
    m.params.borrow_mut().insert(par::FX_TYPE, 0);
    assert!(turn(&mut s, &m, 0, 5, 1).is_empty());
    // P1 is named after the FX type
    m.params.borrow_mut().insert(par::FX_TYPE, 3);
    let n3 = s.label(k(1, 1), &m).unwrap();
    m.params.borrow_mut().insert(par::FX_TYPE, 4);
    let n4 = s.label(k(1, 1), &m).unwrap();
    assert_ne!(n3, n4);
    assert!(n3.starts_with("FX ") && n3 != "FX P1", "{n3}");
}

#[test]
fn undo_and_redo_controller_changes() {
    let m = Fake::new();
    let mut s = Surface::new();
    let fine = voice_param(0, "fine");
    let v0 = m.param(fine);
    // two turns within a second: one step
    s.handle(&Input::Relative { row: 0, col: 0, delta: 1 }, 0, &m).iter().for_each(|a| m.apply(std::slice::from_ref(a)));
    s.handle(&Input::Relative { row: 0, col: 0, delta: 1 }, 500, &m).iter().for_each(|a| m.apply(std::slice::from_ref(a)));
    s.handle(&Input::Relative { row: 0, col: 0, delta: 1 }, 3000, &m).iter().for_each(|a| m.apply(std::slice::from_ref(a)));
    assert_eq!(m.param(fine), v0 + 3);
    s.set_mode(Mode::Global);
    press(&mut s, &m, Button::Top(5), 4000); // undo
    assert_eq!(m.param(fine), v0 + 2);
    press(&mut s, &m, Button::Top(5), 4100);
    assert_eq!(m.param(fine), v0);
    press(&mut s, &m, Button::Top(5), 4200);
    assert_eq!(m.param(fine), v0, "nothing left");
    press(&mut s, &m, Button::Top(6), 4300); // redo
    assert_eq!(m.param(fine), v0 + 2);
    // a new change clears redo
    s.set_mode(Mode::Voice(0));
    turn(&mut s, &m, 0, 1, 1);
    s.set_mode(Mode::Global);
    press(&mut s, &m, Button::Top(6), 9000);
    assert_eq!(m.param(fine), v0 + 2);
}

#[test]
fn page_buttons_browse_kits_and_play_loads() {
    let m = Fake::new();
    let mut s = Surface::new();
    assert_eq!(press(&mut s, &m, Button::PageDown, 0), vec![Action::KitBrowse(1)]);
    release(&mut s, &m, Button::PageDown, 0);
    assert_eq!(press(&mut s, &m, Button::PageUp, 0), vec![Action::KitBrowse(-1)]);
    assert!(s.browsing());
    assert_eq!(press(&mut s, &m, Button::Play, 0), vec![Action::KitLoadBrowsed]);
    // any other control cancels
    press(&mut s, &m, Button::PageDown, 0);
    turn(&mut s, &m, 0, 0, 1);
    assert!(!s.browsing());
    assert_eq!(press(&mut s, &m, Button::Play, 0), vec![Action::Trigger { track: 0, velocity: 100 }]);
}

#[test]
fn global_kit_actions_compare_panic_and_switches() {
    let m = Fake::new();
    let mut s = Surface::new();
    s.set_mode(Mode::Global);
    press(&mut s, &m, Button::Top(2), 0); // init: hold
    assert!(tick(&mut s, &m, 900).is_empty());
    assert_eq!(tick(&mut s, &m, 1000), vec![Action::InitKit]);
    release(&mut s, &m, Button::Top(2), 1000);
    press(&mut s, &m, Button::Top(3), 2000); // save: hold
    release(&mut s, &m, Button::Top(3), 2500);
    assert!(tick(&mut s, &m, 3500).is_empty(), "released early");
    press(&mut s, &m, Button::Top(3), 4000);
    assert_eq!(tick(&mut s, &m, 5000), vec![Action::SaveKit]);
    assert_eq!(press(&mut s, &m, Button::Top(4), 0), vec![Action::Compare(true)]);
    assert_eq!(release(&mut s, &m, Button::Top(4), 0), vec![Action::Compare(false)]);
    assert_eq!(press(&mut s, &m, Button::Top(7), 0), vec![Action::Panic]);
    assert_eq!(press(&mut s, &m, Button::Top(0), 0), vec![Action::SetSwitch(Switch::SumAux, true)]);
    assert_eq!(press(&mut s, &m, Button::Top(0), 0), vec![Action::SetSwitch(Switch::SumAux, false)]);
    assert_eq!(press(&mut s, &m, Button::Top(1), 0), vec![Action::SetSwitch(Switch::HwClock, true)]);
    let fr = s.frame(0, &m);
    let led = |c| fr.leds.iter().find(|(x, _)| *x == c).unwrap().1;
    assert_eq!(led(t(2)), Section::Global.colour(), "HW clock on");
    assert_eq!(led(t(1)), vdrum_surface::frame::dim(Section::Global.colour()), "sum aux off");
}


fn seq_edits(a: &[Action]) -> Vec<SeqEdit> {
    a.iter().filter_map(|x| if let Action::Seq(e) = x { Some(*e) } else { None }).collect()
}

#[test]
fn step_buttons_set_on_press_and_clear_on_release() {
    let m = Fake::new();
    let mut s = Surface::new();
    s.set_mode(Mode::Sequencer);
    // press sets the step for the edit group (Drum 1), release keeps it
    press(&mut s, &m, Button::Top(4), 0);
    release(&mut s, &m, Button::Top(4), 50);
    let st = m.seq.borrow().step(0, 0, 4);
    assert!(st.on);
    assert_eq!(st.velocity, 100);
    // a second press and release clears it
    press(&mut s, &m, Button::Top(4), 100);
    assert!(m.seq.borrow().step(0, 0, 4).on);
    release(&mut s, &m, Button::Top(4), 150);
    assert!(!m.seq.borrow().step(0, 0, 4).on);
    // bottom row = steps 9..16, Page down = next bar
    press(&mut s, &m, Button::PageDown, 0);
    press(&mut s, &m, Button::Bottom(0), 0);
    release(&mut s, &m, Button::Bottom(0), 0);
    assert!(m.seq.borrow().step(0, 0, 24).on);
    // the step LED is lit in the track colour
    let fr = s.frame(0, &m);
    let led = fr.leds.iter().find(|(c, _)| *c == Control::Button(Button::Bottom(0))).unwrap().1;
    assert_ne!(led, (0, 0, 0));
    // the playhead is white
    m.status.borrow_mut().running = true;
    m.status.borrow_mut().step = [17; 7];
    let fr = s.frame(0, &m);
    let led = fr.leds.iter().find(|(c, _)| *c == Control::Button(Button::Top(1))).unwrap().1;
    assert_eq!(led, (255, 255, 255));
    assert!(fr.stationary[1].contains("bar 2/4"), "{:?}", fr.stationary);
}

#[test]
fn edit_group_sets_steps_on_several_tracks() {
    let m = Fake::new();
    let mut s = Surface::new();
    press(&mut s, &m, Button::SoloArm, 0); // go to mode 8
    release(&mut s, &m, Button::SoloArm, 0);
    assert_eq!(s.mode(), Mode::Sequencer);
    // hold Solo/Arm: top row = tracks
    press(&mut s, &m, Button::SoloArm, 0);
    press(&mut s, &m, Button::Top(6), 0);
    release(&mut s, &m, Button::Top(6), 0);
    release(&mut s, &m, Button::SoloArm, 0);
    assert_eq!(s.edit_group(), 0b100_0001);
    assert_eq!(s.mode(), Mode::Sequencer);
    // where the group disagrees a press sets the step on all
    let on = StepData { on: true, ..StepData::default() };
    m.apply(&[Action::Seq(SeqEdit::Step { pat: 0, track: 0, step: 0, data: on })]);
    let a = press(&mut s, &m, Button::Top(0), 0);
    assert_eq!(seq_edits(&a).len(), 1);
    release(&mut s, &m, Button::Top(0), 0);
    assert!(m.seq.borrow().step(0, 6, 0).on);
    // now both are on: press and release clears both
    press(&mut s, &m, Button::Top(0), 0);
    let a = release(&mut s, &m, Button::Top(0), 0);
    assert_eq!(seq_edits(&a).len(), 2);
    assert!(!m.seq.borrow().step(0, 0, 0).on && !m.seq.borrow().step(0, 6, 0).on);
    // the group never becomes empty; button 8 = all, then back to Drum 1
    press(&mut s, &m, Button::SoloArm, 0);
    press(&mut s, &m, Button::Top(0), 0);
    press(&mut s, &m, Button::Top(6), 0);
    assert_eq!(s.edit_group(), 0b100_0000);
    press(&mut s, &m, Button::Top(7), 0);
    assert_eq!(s.edit_group(), 0x7f);
    press(&mut s, &m, Button::Top(7), 0);
    assert_eq!(s.edit_group(), 1);
}

#[test]
fn track_buttons_select_the_sequencer_track() {
    let m = Fake::new();
    let mut s = Surface::new();
    s.set_mode(Mode::Sequencer);
    assert_eq!(s.seq_track(), 0);
    // a step on Drum 1 and one on Drum 2
    let on = StepData { on: true, ..StepData::default() };
    m.apply(&[Action::Seq(SeqEdit::Step { pat: 0, track: 0, step: 0, data: on })]);
    m.apply(&[Action::Seq(SeqEdit::Step { pat: 0, track: 1, step: 1, data: on })]);
    let led = |s: &Surface, b: Button| s.frame(0, &m).leds.iter().find(|(c, _)| *c == Control::Button(b)).unwrap().1;
    let half = |c: (u8, u8, u8)| (c.0 / 2, c.1 / 2, c.2 / 2);
    assert_eq!(led(&s, Button::Top(0)), half(TRACK_COLOURS[0]));
    assert_ne!(led(&s, Button::Top(1)), half(TRACK_COLOURS[1]));
    assert_eq!(led(&s, Button::TrackLeft), (0, 0, 0), "nothing left of track 1");
    // Track right: track 2 is selected and is the edit group; the step buttons show its steps
    press(&mut s, &m, Button::TrackRight, 0);
    release(&mut s, &m, Button::TrackRight, 0);
    assert_eq!(s.mode(), Mode::Sequencer);
    assert_eq!(s.seq_track(), 1);
    assert_eq!(s.edit_group(), 0b10);
    assert_eq!(led(&s, Button::Top(1)), half(TRACK_COLOURS[1]));
    assert_ne!(led(&s, Button::Top(0)), half(TRACK_COLOURS[0]));
    let fr = s.frame(0, &m);
    assert_eq!(fr.stationary[2], "Track 2 D2", "{:?}", fr.stationary);
    assert!(fr.popup.is_some_and(|p| p.fields[2] == "2 D2"));
    // a press sets the step on the selected track
    press(&mut s, &m, Button::Top(5), 0);
    release(&mut s, &m, Button::Top(5), 0);
    assert!(m.seq.borrow().step(0, 1, 5).on && !m.seq.borrow().step(0, 0, 5).on);
    // it stops at track 7
    for _ in 0..10 {
        press(&mut s, &m, Button::TrackRight, 0);
    }
    assert_eq!(s.seq_track(), 6);
    assert_eq!(led(&s, Button::TrackRight), (0, 0, 0));
    press(&mut s, &m, Button::TrackLeft, 0);
    assert_eq!(s.seq_track(), 5);
    // Solo/Arm still adds tracks; the display shows how many more
    press(&mut s, &m, Button::SoloArm, 0);
    press(&mut s, &m, Button::Top(0), 0);
    release(&mut s, &m, Button::SoloArm, 0);
    assert_eq!(s.edit_group(), 0b10_0001);
    assert_eq!(s.frame(0, &m).stationary[2], "Track 6 CH +1");
    // removing the selected track from the group selects another track of the group
    press(&mut s, &m, Button::SoloArm, 0);
    press(&mut s, &m, Button::Top(5), 0);
    release(&mut s, &m, Button::SoloArm, 0);
    assert_eq!(s.edit_group(), 1);
    assert_eq!(s.seq_track(), 0);
}

#[test]
fn selected_track_knobs_pulse_at_their_value_brightness() {
    let m = Fake::new();
    let mut s = Surface::new();
    s.set_mode(Mode::Sequencer);
    press(&mut s, &m, Button::TrackRight, 0); // track 2
    let enc = |row: u8, col: u8, now: u64| {
        s.frame(now, &m).leds.iter().find(|(c, _)| *c == Control::Encoder { row, col }).unwrap().1
    };
    // the selected column (2) changes over a pulse period; another column does not
    let lum = |c: (u8, u8, u8)| c.0 as u32 + c.1 as u32 + c.2 as u32;
    let low = enc(0, 1, 0);
    let high = enc(0, 1, PULSE_MS / 2);
    assert!(lum(high) > lum(low) * 4, "{low:?} {high:?}");
    assert_eq!(enc(0, 0, 0), enc(0, 0, PULSE_MS / 2));
    // at the peak the LED is the value brightness (as in other columns)
    assert_eq!(high, enc(0, 1, PULSE_MS + PULSE_MS / 2));
    assert!((pulse(PULSE_MS / 2) - 1.0).abs() < 1e-6 && (pulse(0) - PULSE_FLOOR).abs() < 1e-6);
    // outside mode 8 nothing pulses
    s.set_mode(Mode::Perform);
    let a = s.frame(0, &m).leds;
    let b = s.frame(PULSE_MS / 2, &m).leds;
    assert_eq!(a, b);
}

#[test]
fn held_steps_change_fields_and_locks() {
    let m = Fake::new();
    let mut s = Surface::new();
    s.set_mode(Mode::Sequencer);
    press(&mut s, &m, Button::Top(2), 0);
    press(&mut s, &m, Button::Top(3), 0);
    assert_eq!(s.layout().get(k(1, 1)).unwrap().binding, Binding::StepField(layout::StepField::Velocity));
    turn(&mut s, &m, 0, 0, -10); // velocity
    turn(&mut s, &m, 0, 3, 2); // flam x3
    // lock: cutoff (row 2, column 3), starting from the kit's value
    let cutoff = voice_param(0, "cutoff");
    let base = vdrum_surface::model::unscale(0, lxr_params::info(cutoff).unwrap().max, m.param(cutoff));
    turn(&mut s, &m, 1, 2, -5);
    release(&mut s, &m, Button::Top(2), 0);
    release(&mut s, &m, Button::Top(3), 0);
    for step in [2, 3] {
        let st = m.seq.borrow().step(0, 0, step);
        assert!(st.on, "edited steps stay on after release");
        assert_eq!(st.velocity, 90);
        assert_eq!(st.flam, 2);
        assert_eq!(st.locks[0], (cutoff as u8, base - 5));
    }
    assert_eq!(s.layout().get(k(1, 1)).unwrap().binding, Binding::SeqFills(0));
    // a third lock replaces the oldest
    press(&mut s, &m, Button::Top(2), 0);
    turn(&mut s, &m, 1, 0, 1); // pitch
    turn(&mut s, &m, 1, 1, 1); // decay
    release(&mut s, &m, Button::Top(2), 0);
    let st = m.seq.borrow().step(0, 0, 2);
    assert_eq!(st.locks[0].0 as u16, voice_param(0, "coarse"));
    assert_eq!(st.locks[1].0 as u16, voice_param(0, "decay"));
    // the open hi-hat's parameters cannot be locked
    press(&mut s, &m, Button::SoloArm, 0);
    press(&mut s, &m, Button::Top(6), 0);
    press(&mut s, &m, Button::Top(0), 0);
    release(&mut s, &m, Button::SoloArm, 0);
    press(&mut s, &m, Button::Top(5), 0);
    let a = turn(&mut s, &m, 1, 2, 1);
    assert!(seq_edits(&a).is_empty());
}

#[test]
fn generator_knobs_and_density_faders() {
    let m = Fake::new();
    let mut s = Surface::new();
    s.set_mode(Mode::Sequencer);
    turn(&mut s, &m, 0, 1, 4); // Drum 2 fills
    assert_eq!(m.seq.borrow().track(0, 1).bits(), 0x1111);
    turn(&mut s, &m, 1, 1, 2); // rotate
    assert_eq!(m.seq.borrow().track(0, 1).bits(), 0x4444);
    turn(&mut s, &m, 2, 1, -8); // length 8
    assert_eq!(m.seq.borrow().track(0, 1).length, 8);
    assert_eq!(m.seq.borrow().track(0, 1).fills, 4);
    // rotate without fills does nothing
    assert!(seq_edits(&turn(&mut s, &m, 1, 0, 1)).is_empty());
    // shuffle and next pattern (column 8)
    turn(&mut s, &m, 0, 7, 20);
    assert_eq!(m.seq.borrow().patterns[0].shuffle, 20);
    turn(&mut s, &m, 1, 7, 2);
    assert_eq!(m.status.borrow().pattern, 2);
    // density faders pick up at the current value (100 % = top)
    fader(&mut s, &m, 2, 0);
    assert_eq!(m.seq.borrow().density[2], 100);
    fader(&mut s, &m, 2, 127);
    fader(&mut s, &m, 2, 64);
    assert_eq!(m.seq.borrow().density[2], 50);
    fader(&mut s, &m, 7, 127);
    fader(&mut s, &m, 7, 0);
    assert_eq!(m.seq.borrow().density_all, 0);
}

#[test]
fn pattern_layer_queues_patterns() {
    let m = Fake::new();
    let mut s = Surface::new();
    s.set_mode(Mode::Sequencer);
    m.status.borrow_mut().running = true;
    press(&mut s, &m, Button::MuteSelect, 0);
    let a = press(&mut s, &m, Button::Bottom(2), 0);
    assert_eq!(seq_edits(&a), vec![SeqEdit::NextPattern(10)]);
    release(&mut s, &m, Button::Bottom(2), 0);
    release(&mut s, &m, Button::MuteSelect, 0);
    assert_eq!(s.mode(), Mode::Sequencer);
    // no step was set
    assert!(m.seq.borrow().patterns[0].is_empty());
}

#[test]
fn record_writes_pad_hits_at_the_playhead() {
    let m = Fake::new();
    let mut s = Surface::new();
    press(&mut s, &m, Button::Record, 0);
    assert!(s.record_armed());
    // stopped: nothing is recorded
    let a = press(&mut s, &m, Button::Bottom(3), 0);
    assert!(seq_edits(&a).is_empty());
    {
        let mut st = m.status.borrow_mut();
        st.running = true;
        st.step = [5; 7];
        st.late = true;
    }
    press(&mut s, &m, Button::Bottom(3), 0); // snare pad
    let st = m.seq.borrow().step(0, 3, 6);
    assert!(st.on);
    assert_eq!(st.velocity, 100);
    // the next step wraps at the track length
    m.status.borrow_mut().step = [15; 7];
    press(&mut s, &m, Button::Bottom(3), 0);
    assert!(m.seq.borrow().step(0, 3, 0).on);
}

#[test]
fn play_runs_the_internal_clock_only() {
    let m = Fake::new();
    let mut s = Surface::new();
    s.set_mode(Mode::Sequencer);
    assert!(seq_edits(&press(&mut s, &m, Button::Play, 0)).is_empty());
    // Global: clock setting (knob row 3, column 8) to Internal
    s.set_mode(Mode::Global);
    let a = turn(&mut s, &m, 2, 7, 1);
    assert!(a.contains(&Action::SeqHostClock(false)), "{a:?}");
    s.set_mode(Mode::Sequencer);
    let a = press(&mut s, &m, Button::Play, 0);
    assert_eq!(seq_edits(&a), vec![SeqEdit::Run(true)]);
    let a = press(&mut s, &m, Button::Play, 10);
    assert_eq!(seq_edits(&a), vec![SeqEdit::Run(false)]);
}


fn led_of(s: &Surface, m: &Fake, c: Control) -> (u8, u8, u8) {
    s.frame(0, m).leds.iter().find(|(x, _)| *x == c).map(|x| x.1).unwrap()
}

#[test]
fn perform_mutes_and_triggers() {
    let m = Fake::new();
    let mut s = Surface::new();
    s.set_mode(Mode::Perform);
    press(&mut s, &m, Button::Bottom(2), 0);
    assert_eq!(m.mutes(), 0b100);
    assert_eq!(led_of(&s, &m, Control::Button(Button::Bottom(2))), (255, 0, 0));
    press(&mut s, &m, Button::Bottom(7), 0); // mute all
    assert_eq!(m.mutes(), 0x7f);
    press(&mut s, &m, Button::Bottom(7), 0);
    assert_eq!(m.mutes(), 0);
    let a = press(&mut s, &m, Button::Top(4), 0);
    assert!(a.iter().any(|x| matches!(x, Action::Trigger { track: 4, .. })));
}

#[test]
fn momentary_effects_restore_exactly() {
    let m = Fake::new();
    let mut s = Surface::new();
    s.set_mode(Mode::MixFx);
    let reso: Vec<u16> = (0..7).map(|t| voice_param(t, "reso")).filter(|i| lxr_params::info(*i).is_some()).collect();
    m.apply(&[Action::SetParam { idx: reso[0], value: 33 }]);
    let before: Vec<u16> = reso.iter().map(|i| m.param(*i)).collect();
    press(&mut s, &m, Button::Bottom(3), 0); // Reso up
    assert!(m.param(reso[0]) > 33);
    release(&mut s, &m, Button::Bottom(3), 10);
    assert_eq!(reso.iter().map(|i| m.param(*i)).collect::<Vec<_>>(), before);

    // filter sweep moves over time
    let cut = voice_param(0, "cutoff");
    let c0 = m.param(cut);
    press(&mut s, &m, Button::Bottom(2), 1000);
    tick(&mut s, &m, 1100);
    let mid = m.param(cut);
    tick(&mut s, &m, 5000);
    let end = m.param(cut);
    assert!(end < mid && mid <= c0, "{c0} {mid} {end}");
    release(&mut s, &m, Button::Bottom(2), 5001);
    assert_eq!(m.param(cut), c0);

    // stutter repeats the tracks hit recently at 1/16 of the tempo (120 bpm: 125 ms)
    m.hits.borrow_mut()[3] += 1;
    tick(&mut s, &m, 6000);
    let a = press(&mut s, &m, Button::Bottom(0), 6100);
    assert_eq!(a.iter().filter(|x| matches!(x, Action::Trigger { track: 3, .. })).count(), 1);
    let a = tick(&mut s, &m, 6100 + 125 * 3);
    assert_eq!(a.iter().filter(|x| matches!(x, Action::Trigger { track: 3, .. })).count(), 3);
    release(&mut s, &m, Button::Bottom(0), 6500);
    assert!(tick(&mut s, &m, 7000).iter().all(|x| !matches!(x, Action::Trigger { .. })));
}

#[test]
fn lfo_scales_are_non_destructive() {
    let m = Fake::new();
    let mut s = Surface::new();
    s.set_mode(Mode::LfoMod);
    let r0 = voice_param(0, "lfo_rate");
    m.apply(&[Action::SetParam { idx: r0, value: 40 }]);
    turn(&mut s, &m, 0, 7, 20); // rate scale 200 %
    assert_eq!(m.param(r0), 80);
    turn(&mut s, &m, 0, 7, -20);
    assert_eq!(m.param(r0), 40);
    // LFO sync to host sets free LFOs to 1/1 and back
    let sy = voice_param(1, "lfo_sync");
    press(&mut s, &m, Button::Top(7), 0);
    assert_eq!(m.param(sy), 3);
    press(&mut s, &m, Button::Top(7), 0);
    assert_eq!(m.param(sy), 0);
    let a = press(&mut s, &m, Button::Bottom(7), 0);
    assert!(a.contains(&Action::LfoReset));
}

#[test]
fn snapshots_morph_and_macros() {
    let m = Fake::new();
    let mut s = Surface::new();
    s.set_mode(Mode::MorphMacro);
    let coarse = voice_param(0, "coarse");
    m.apply(&[Action::SetParam { idx: coarse, value: 100 }]);
    // hold 1 s stores snapshot 1
    press(&mut s, &m, Button::Top(0), 0);
    tick(&mut s, &m, 1000);
    release(&mut s, &m, Button::Top(0), 1100);
    assert!(s.take_state_changed());
    m.apply(&[Action::SetParam { idx: coarse, value: 20 }]);
    // morph Drum 1 toward snapshot 1 and back
    turn(&mut s, &m, 1, 0, 127);
    assert_eq!(m.param(coarse), 100);
    turn(&mut s, &m, 1, 0, -64);
    assert_eq!(m.param(coarse), 59); // 20 + 80 * 63 / 127
    turn(&mut s, &m, 1, 0, -127);
    assert_eq!(m.param(coarse), 20);
    // a short press recalls
    press(&mut s, &m, Button::Top(0), 2000);
    release(&mut s, &m, Button::Top(0), 2100);
    assert_eq!(m.param(coarse), 100);

    // learn macro 1 from a knob turned in a voice mode
    press(&mut s, &m, Button::Bottom(0), 3000);
    s.set_mode(Mode::Voice(0));
    let fine = voice_param(0, "fine");
    let f0 = m.param(fine);
    turn(&mut s, &m, 0, 0, 10);
    s.set_mode(Mode::MorphMacro);
    press(&mut s, &m, Button::Bottom(0), 4000);
    fader(&mut s, &m, 0, 0);
    assert_eq!(m.param(fine), f0);
    fader(&mut s, &m, 0, 127);
    assert_eq!(m.param(fine), f0 + 10);
    // the state survives a reload
    let text = s.state_text();
    let mut s2 = Surface::new();
    s2.load_state(&text);
    assert_eq!(s2.state_text(), text);
}

#[test]
fn view_shows_controls_layers_and_hardware_edits_only() {
    let m = Fake::new();
    let mut s = Surface::new();
    let v = s.view(0, &m);
    assert_eq!(v.mode, Mode::Voice(0));
    assert_eq!(v.controls.len(), 48);
    assert!(v.edit.is_none());
    // encoder 1 of row 1: Drum 1 fine tune, with its value and parameter ID
    let fine = layout::p("fine1");
    let c = v.control(k(1, 1)).unwrap();
    assert_eq!(c.param.map(|p| p.0), Some("fine1"));
    assert!(c.value.is_some() && c.frac.is_some());
    assert_ne!(c.led, (0, 0, 0));
    // a hardware edit sets the edit pill; an editor edit changes the value but not the pill
    turn(&mut s, &m, 0, 0, 2);
    let e = s.view(0, &m).edit.unwrap();
    assert_eq!(e.name, s.view(0, &m).control(k(1, 1)).unwrap().label);
    let before = m.param(fine);
    let a = s.handle_editor(&Input::Relative { row: 0, col: 0, delta: 3 }, 10, &m);
    m.apply(&a);
    assert_eq!(m.param(fine), before + 3);
    assert_eq!(s.view(10, &m).edit.unwrap().gen, e.gen, "editor edits are not hardware edits");
    // faders: position and pickup
    let fv = s.view(10, &m).controls.iter().find(|c| c.control == f(1)).unwrap().fader;
    assert_eq!(fv, Some((None, false)));
    // context layers
    press(&mut s, &m, Button::Shift, 20);
    assert!(s.view(20, &m).layers.iter().any(|l| l.starts_with("SHIFT")));
    press(&mut s, &m, Button::Bottom(7), 20); // mode 16: Global
    release(&mut s, &m, Button::Shift, 20);
    assert_eq!(s.view(20, &m).mode, Mode::Global);
    assert!(s.view(20, &m).layers.is_empty());
    // a hold in progress
    let save = s.layout().iter().find(|(_, c)| c.binding == Binding::SaveKit).map(|(c, _)| *c).unwrap();
    let Control::Button(save) = save else { panic!() };
    press(&mut s, &m, save, 30);
    assert!(s.view(30, &m).layers.iter().any(|l| l.starts_with("HOLD 1 s")), "{:?}", s.view(30, &m).layers);
    release(&mut s, &m, save, 40);
    // sequencer layers
    press(&mut s, &m, Button::Shift, 50);
    press(&mut s, &m, Button::Top(7), 50); // mode 8
    release(&mut s, &m, Button::Shift, 50);
    press(&mut s, &m, Button::SoloArm, 60);
    assert!(s.view(60, &m).layers.iter().any(|l| l.starts_with("TRACK GROUP: D1")));
    release(&mut s, &m, Button::SoloArm, 60);
    press(&mut s, &m, Button::Top(4), 70);
    assert!(s.view(70, &m).layers.iter().any(|l| l == "STEP 5 HELD: locks"));
    release(&mut s, &m, Button::Top(4), 80);
}

#[test]
fn encoder_led_brightness_follows_the_value() {
    let m = Fake::new();
    let mut s = Surface::new();
    let enc = Control::Encoder { row: 0, col: 0 };
    let led = |s: &Surface| s.frame(0, &m).leds.iter().find(|(c, _)| *c == enc).map(|x| x.1).unwrap();
    let sum = |c: (u8, u8, u8)| c.0 as u32 + c.1 as u32 + c.2 as u32;
    turn(&mut s, &m, 0, 0, -63);
    turn(&mut s, &m, 0, 0, -63);
    turn(&mut s, &m, 0, 0, -63);
    let low = led(&s);
    turn(&mut s, &m, 0, 0, 63);
    turn(&mut s, &m, 0, 0, 63);
    turn(&mut s, &m, 0, 0, 63);
    let high = led(&s);
    assert!(sum(low) > 0, "the minimum stays visible: {low:?}");
    assert!(sum(high) > sum(low) * 4, "{low:?} {high:?}");
    assert_eq!(vdrum_surface::value_brightness((200, 100, 0), 1.0), (200, 100, 0));
    assert_eq!(vdrum_surface::value_brightness((200, 100, 0), 0.0), (20, 10, 0));
}
