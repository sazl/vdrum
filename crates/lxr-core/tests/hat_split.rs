//! VDrum extension: the open hi-hat as a separate voice (`lxr_core::vdrum::open_hat`).

use lxr_core::parameters::*;
use lxr_core::vdrum::*;
use lxr_core::Lxr;

fn on_big_stack(f: impl FnOnce() + Send + 'static) {
    std::thread::Builder::new().stack_size(64 << 20).spawn(f).unwrap().join().unwrap();
}

fn render(l: &mut Lxr, ticks: usize, out: &mut Vec<i16>) {
    for _ in 0..ticks {
        let b = l.tick();
        out.extend_from_slice(&b.dac1);
        out.extend_from_slice(&b.dac2);
    }
}

fn energy(x: &[i16]) -> f64 {
    x.iter().map(|&s| (s as f64) * (s as f64)).sum()
}

/// Track 6 = closed hat (voice 5), track 7 = open hat (voice 6).
const CH: u8 = 5;
const OH: u8 = 6;

#[test]
fn split_off_is_the_firmware_path() {
    on_big_stack(|| {
        let mut a = Box::new(Lxr::new());
        assert!(!a.vdrum.hat_split);
        // the open hat's state exists but the firmware voice plays both tracks
        a.voiceControl_noteOn(OH, 60, 100);
        assert_eq!(a.hatVoice.isOpen, 1);
        assert_eq!(a.vdrum.ohVoice.oscVolEg.state, 0);
    });
}

#[test]
fn imported_open_hat_sounds_like_the_firmware_open_hat() {
    on_big_stack(|| {
        // With the parameters imported from the shared hi-hat, open hits alone render the same output
        // with and without the split (the init kit's hi-hat uses no noise oscillator, so both paths draw
        // the same random numbers).
        let mut off = Box::new(Lxr::new());
        let mut on = Box::new(Lxr::new());
        assert_ne!(on.activeParameters[PAR_WAVE3_HH as usize], 4, "init kit hat must not use noise");
        on.set_hat_split(true);
        let (mut a, mut b) = (Vec::new(), Vec::new());
        for (vel, ticks) in [(100u8, 40usize), (30, 25), (127, 60)] {
            off.voiceControl_noteOn(OH, 60, vel);
            on.voiceControl_noteOn(OH, 60, vel);
            render(&mut off, ticks, &mut a);
            render(&mut on, ticks, &mut b);
        }
        assert!(energy(&a) > 0.0);
        if let Some(i) = a.iter().zip(&b).position(|(x, y)| x != y) {
            panic!("first difference at sample {i} (block {}): {} vs {}", i / 256, a[i], b[i]);
        }
    });
}

#[test]
fn closed_hit_does_not_cut_the_open_hat_without_choke() {
    on_big_stack(|| {
        let mut l = Box::new(Lxr::new());
        l.set_hat_split(true);
        l.oh_set_parameter(OH_CHOKE, 0);
        l.oh_set_parameter(OH_VELOD, 127);
        l.voiceControl_noteOn(OH, 60, 127);
        let mut x = Vec::new();
        render(&mut l, 5, &mut x);
        l.voiceControl_noteOn(CH, 60, 127);
        assert!(l.vdrum.ohVoice.oscVolEg.value > 0.0, "open hat still sounding");
        assert_eq!(l.hatVoice.isOpen, 0);

        // with choke the closed hit silences it
        l.oh_set_parameter(OH_CHOKE, 1);
        l.voiceControl_noteOn(OH, 60, 127);
        render(&mut l, 5, &mut x);
        l.voiceControl_noteOn(CH, 60, 127);
        assert_eq!(l.vdrum.ohVoice.oscVolEg.value, 0.0);
    });
}

#[test]
fn open_and_closed_parameters_are_independent() {
    on_big_stack(|| {
        let mut l = Box::new(Lxr::new());
        l.set_hat_split(true);
        let ch_coarse = l.hatVoice.osc.midiFreq;
        l.oh_set_parameter(OH_COARSE, 100);
        assert_eq!(l.vdrum.ohVoice.osc.midiFreq >> 8, 100);
        assert_eq!(l.hatVoice.osc.midiFreq, ch_coarse);
        l.set_parameter(PAR_COARSE6, 10);
        assert_eq!(l.hatVoice.osc.midiFreq >> 8, 10);
        assert_eq!(l.vdrum.ohVoice.osc.midiFreq >> 8, 100);
        // PAR_VELOD6_OPEN no longer reaches the open hat
        let d = l.vdrum.ohVoice.decayOpen;
        l.set_parameter(PAR_VELOD6_OPEN, 1);
        assert_eq!(l.vdrum.ohVoice.decayOpen, d);
    });
}

#[test]
fn open_hat_routing_volume_and_fader() {
    on_big_stack(|| {
        let mut l = Box::new(Lxr::new());
        l.set_hat_split(true);
        // route the open hat to DAC2 (stereo): DAC1 stays silent
        l.oh_set_parameter(OH_AUDIO_OUT, lxr_core::prelude::MIXER_ROUTING_DAC2_STEREO);
        l.voiceControl_noteOn(OH, 60, 127);
        let b = l.tick();
        let b2 = l.tick();
        assert!(b.dac1.iter().chain(b2.dac1.iter()).all(|&s| s == 0));
        assert!(b2.dac2.iter().any(|&s| s != 0));
        // fader 7 down: silence
        let mut l = Box::new(Lxr::new());
        l.set_hat_split(true);
        l.set_fader_adc(6, 0);
        l.voiceControl_noteOn(OH, 60, 127);
        let mut x = Vec::new();
        render(&mut l, 20, &mut x);
        assert_eq!(energy(&x), 0.0);
    });
}

#[test]
fn open_hat_lfo_modulates_its_own_parameter() {
    on_big_stack(|| {
        let mut l = Box::new(Lxr::new());
        l.set_hat_split(true);
        let t = OH_TARGET_BASE + OH_MOD_TARGETS.iter().position(|(k, _)| *k as usize == OH_FILTER_FREQ).unwrap() as u8;
        l.oh_set_parameter(OH_TARGET_LFO, t);
        assert_eq!(l.vdrum.ohVoice.lfo.modTarget.destination, OH_DEST_BASE + OH_FILTER_FREQ as u16);
        l.oh_set_parameter(OH_AMOUNT_LFO, 127);
        l.oh_set_parameter(OH_FREQ_LFO, 100);
        let ch_filter = l.hatVoice.filter;
        let mut seen = std::collections::BTreeSet::new();
        for _ in 0..50 {
            l.tick();
            seen.insert(l.vdrum.ohVoice.filter.f.to_bits());
        }
        assert!(seen.len() > 5, "filter frequency moves");
        assert_eq!(l.hatVoice.filter.f.to_bits(), ch_filter.f.to_bits(), "closed hat untouched");
        // switching the LFO voice to the closed hat keeps the parameter
        l.oh_set_parameter(OH_VOICE_LFO, 6);
        assert_eq!(l.oh_get_parameter(OH_TARGET_LFO), 211);
        assert_eq!(l.vdrum.ohVoice.lfo.modTarget.destination, PAR_FILTER_FREQ_6);
        l.oh_set_parameter(OH_VOICE_LFO, 7);
        assert_eq!(l.oh_get_parameter(OH_TARGET_LFO), t);
    });
}

#[test]
fn import_maps_hat_destinations() {
    on_big_stack(|| {
        let mut l = Box::new(Lxr::new());
        l.set_parameter(PAR_VELOD6_OPEN, 77);
        l.set_parameter(PAR_TARGET_LFO6, 211); // hat filter
        l.oh_import_from_hat();
        assert_eq!(l.oh_get_parameter(OH_VELOD), 77);
        assert_eq!(hat_target_for_oh_target(l.oh_get_parameter(OH_TARGET_LFO)), Some(211));
        assert_eq!(l.oh_get_parameter(OH_VOICE_LFO), 7);
        assert_eq!(l.oh_get_parameter(OH_CHOKE), 1);
    });
}

#[test]
fn mod_target_table_fits_in_a_byte() {
    assert_eq!(OH_MAX_TARGET, 254);
    for (i, (_, ch)) in OH_MOD_TARGETS.iter().enumerate() {
        assert!((191..=224).contains(ch));
        assert_eq!(oh_target_for_hat_target(*ch), Some(OH_TARGET_BASE + i as u8));
    }
}


#[test]
fn all_sound_off_silences_every_voice() {
    on_big_stack(|| {
        let mut l = Box::new(Lxr::new());
        l.set_hat_split(true);
        for v in 0..7u8 {
            l.voiceControl_noteOn(v, 60, 127);
        }
        let mut x = Vec::new();
        render(&mut l, 3, &mut x);
        assert!(energy(&x) > 0.0);
        l.vdrum_all_sound_off();
        // the filters and the FX bus (off in the init kit) ring out within a block
        let mut y = Vec::new();
        render(&mut l, 1, &mut y);
        let mut z = Vec::new();
        render(&mut l, 20, &mut z);
        assert_eq!(energy(&z), 0.0);
    });
}
