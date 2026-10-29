//! P3 control path against the emulated firmware: MIDI streams (every CC on every channel, NRPN
//! sequences, notes and other channel messages, random byte streams, MIDI clock with external sync, MTC
//! start of the internal sequencer) and parameter changes, with audio rendered in between. After every
//! message (or every few, see each test) the whole engine state is compared (`p3_common::state_diff`,
//! all globals byte for byte), and every rendered block is compared sample for sample.
//!
//! Runs against the shipped v1.71 image (`LXR_FIRMWARE_IMG`, default path below) and the reference ELF
//! (`LXR_FW_ELF`); skipped without `LXR_FW_ELF`.
#![allow(non_snake_case)]

mod p3_common;

use lxr_core::prelude::*;
use lxr_difftest::Sweep;
use lxr_emu::scenario::Event;
use p3_common::*;

/// Voice v on MIDI channel v (1-based), any note (`PAR_MIDI_NOTEv = 0`), as the scenario prologue.
fn prologue(p: &mut Pair) {
    let chans = [PAR_MIDI_CHAN_1, PAR_MIDI_CHAN_2, PAR_MIDI_CHAN_3, PAR_MIDI_CHAN_4, PAR_MIDI_CHAN_5, PAR_MIDI_CHAN_6, PAR_MIDI_CHAN_7];
    for (v, &c) in chans.iter().enumerate() {
        p.event(&Event::SetParameter { param: c, value: v as u8 + 1 });
        p.event(&Event::SetParameter { param: PAR_MIDI_NOTE1 + v as u16, value: 0 });
    }
    p.check("prologue");
}

/// Runs `f` once per available firmware build, on a big stack.
fn for_each_build(f: fn(&mut Pair, &str)) {
    let _fw = lxr_difftest::fw_or_skip!();
    on_big_stack(move || {
        for (build, make) in builds() {
            let mut p = Pair::new(make());
            p.check(&format!("{build}: power-on"));
            f(&mut p, build);
            p.check(&format!("{build}: end"));
            eprintln!("{build}: ok, {} blocks", p.blocks);
        }
    });
}

#[test]
fn all_ccs_on_all_channels() {
    for_each_build(|p, build| {
        prologue(p);
        let mut rnd = Sweep(0xcc00_0001);
        for ch in 0..16u8 {
            for cc in 0..128u8 {
                // CC 0 (bank select) loads kit `value`; kits 64..127 read past `mainParameters` into memory
                // the engine does not model (`preset_mainParametersByte`), so only 0..63 is compared
                let vals = if cc == 0 { [0u8, 63, rnd.u8() & 0x3f] } else { [0u8, 127, rnd.u8() & 0x7f] };
                for v in vals {
                    p.midi(&[0xB0 | ch, cc, v]);
                    p.check(&format!("{build}: CC {cc} = {v} on channel {ch}"));
                }
                p.tick(&format!("{build}: after CC {cc} on channel {ch}"));
            }
            // a note on the channel so that the CC changes are heard
            p.midi(&[0x90 | ch, 60, 100]);
            for _ in 0..8 {
                p.tick(&format!("{build}: note on channel {ch}"));
            }
            p.check(&format!("{build}: channel {ch}"));
        }
    });
}

#[test]
fn nrpn_sequences() {
    for_each_build(|p, build| {
        prologue(p);
        let mut rnd = Sweep(0x4e52_504e);
        for ch in [0u8, 1, 3, 6, 7, 15] {
            for nrpn in 0..400u16 {
                let (msb, lsb) = ((nrpn >> 7) as u8, (nrpn & 0x7f) as u8);
                let v = rnd.u8() & 0x7f;
                // full sequence, then running status for the data entry
                p.midi(&[0xB0 | ch, 99, msb, 0xB0 | ch, 98, lsb, 0xB0 | ch, 6, v]);
                p.check(&format!("{build}: NRPN {nrpn} = {v} on channel {ch}"));
                if nrpn % 3 == 0 {
                    p.midi(&[38, rnd.u8() & 0x7f, 6, rnd.u8() & 0x7f]);
                    p.check(&format!("{build}: NRPN {nrpn} running status on channel {ch}"));
                }
                if nrpn % 16 == 0 {
                    p.midi(&[0x90 | ch, 48, 100]);
                    p.tick(&format!("{build}: NRPN {nrpn} channel {ch}"));
                }
            }
        }
    });
}

#[test]
fn notes_and_channel_messages() {
    for_each_build(|p, build| {
        prologue(p);
        let mut rnd = Sweep(0x4e07_e500);
        for ch in 0..16u8 {
            for note in (0..128u8).step_by(5) {
                let vel = rnd.u8() & 0x7f;
                p.midi(&[0x90 | ch, note, vel]);
                p.check(&format!("{build}: note on {note} vel {vel} channel {ch}"));
                p.tick(&format!("{build}: note {note} channel {ch}"));
                // pitch bend, poly and channel aftertouch
                p.midi(&[0xE0 | ch, rnd.u8() & 0x7f, rnd.u8() & 0x7f, 0xA0 | ch, note, rnd.u8() & 0x7f, 0xD0 | ch, rnd.u8() & 0x7f]);
                p.check(&format!("{build}: bend/aftertouch channel {ch}"));
                if note % 2 == 0 {
                    p.midi(&[0x80 | ch, note, 64]);
                } else {
                    p.midi(&[0x90 | ch, note, 0]);
                }
                p.check(&format!("{build}: note off {note} channel {ch}"));
                p.tick(&format!("{build}: note off {note} channel {ch}"));
            }
            p.midi(&[0xC0 | ch, rnd.u8() & 0x7f]);
            p.check(&format!("{build}: program change channel {ch}"));
        }
    });
}

/// One random "message": structured channel messages, running status data, system common / realtime,
/// sysex, or raw bytes.
fn random_message(rnd: &mut Sweep) -> Vec<u8> {
    let ch = (rnd.next_u32() % 16) as u8;
    let d = |r: &mut Sweep| r.u8() & 0x7f;
    match rnd.next_u32() % 16 {
        0..=3 => vec![0x90 | ch, d(rnd), d(rnd)],
        4 => vec![0x80 | ch, d(rnd), d(rnd)],
        5..=7 => {
            let cc = d(rnd);
            // bank select above 63: see all_ccs_on_all_channels
            let v = if cc == 0 { d(rnd) & 0x3f } else { d(rnd) };
            vec![0xB0 | ch, cc, v]
        }
        8 => vec![0xC0 | ch, d(rnd)],
        9 => vec![0xE0 | ch, d(rnd), d(rnd)],
        10 => vec![d(rnd) | 1, d(rnd)],                   // running status data (never CC 0)
        11 => vec![0xF8 + (rnd.next_u32() % 8) as u8],     // realtime (F8..FF)
        12 => vec![0xF1, d(rnd)],                          // MTC quarter frame
        13 => {
            let n = (rnd.next_u32() % 10) as usize;
            let mut v = vec![0xF0];
            v.extend((0..n).map(|_| d(rnd)));
            v.push(0xF7);
            v
        }
        // raw bytes (never a 0 data byte, which could complete a bank select above 63)
        _ => (0..1 + rnd.next_u32() % 4).map(|_| rnd.u8() | 1).collect(),
    }
}

/// Waveform CCs store the raw data byte (0..127) in `OscInfo::waveform` without clamping, in the firmware as
/// in `Lxr`, and the state stays equal. An oscillator with an out-of-range waveform writes nothing to its
/// output buffer, and the voice then renders the uninitialised stack contents of the firmware's VLA (for
/// example `int16_t mod2[size]` in `HiHat_calcSyncBlock`), which `Lxr` cannot reproduce (the same divergence
/// as the `global_morph_no_target` scenario, in voice code outside P3). The random streams reach this
/// (seed 5: `WAVE3_HH` = 64), so after every message any out-of-range oscillator waveform is reduced modulo
/// `NUM_OSC_WAVEFORMS` in `Lxr` and the voice is copied into the emulated firmware.
fn clamp_osc_waveforms(p: &mut Pair) {
    use lxr_core::dsp::oscillator::NUM_OSC_WAVEFORMS as N;
    fn fix(w: &mut u8) -> bool {
        let bad = *w >= N;
        if bad {
            *w %= N;
        }
        bad
    }
    let l = &mut p.l;
    let mut touched: Vec<&str> = Vec::new();
    let mut drums = false;
    for v in l.voiceArray.iter_mut() {
        drums |= fix(&mut v.osc.waveform) | fix(&mut v.modOsc.waveform);
    }
    if drums {
        touched.push("voiceArray");
    }
    if fix(&mut l.snareVoice.osc.waveform) {
        touched.push("snareVoice");
    }
    let c = &mut l.cymbalVoice;
    if fix(&mut c.osc.waveform) | fix(&mut c.modOsc.waveform) | fix(&mut c.modOsc2.waveform) {
        touched.push("cymbalVoice");
    }
    let h = &mut l.hatVoice;
    if fix(&mut h.osc.waveform) | fix(&mut h.modOsc.waveform) | fix(&mut h.modOsc2.waveform) {
        touched.push("hatVoice");
    }
    for name in touched {
        push_global(&p.l, &mut p.emu, name);
    }
}

#[test]
fn random_streams() {
    for_each_build(|p, build| {
        prologue(p);
        for seed in 1..=6u64 {
            let mut rnd = Sweep(0x7a4d_0000 + seed);
            for i in 0..1500 {
                let m = random_message(&mut rnd);
                p.midi(&m);
                p.check(&format!("{build}: seed {seed} message {i}: {m:02x?}"));
                clamp_osc_waveforms(p);
                for _ in 0..(rnd.next_u32() % 3) {
                    p.tick(&format!("{build}: seed {seed} message {i}"));
                    p.check(&format!("{build}: seed {seed} message {i}, after block {}", p.blocks));
                }
            }
            p.check(&format!("{build}: seed {seed} end"));
        }
    });
}

/// Euclidean patterns on every track (through the menu parameters), plus step parameters.
fn setup_patterns(p: &mut Pair, rnd: &mut Sweep) {
    for track in 0..7u8 {
        // the menu's active voice selects the track the EUKLID/step parameters apply to
        p.midi(&[0xB0 | (track + 1), 0, 0]); // harmless traffic in between
        // (the UI's menu_setActiveVoice also repaints; only its effect on the engine is set here)
        p.l.menu_activeVoice = track;
        push_global(&p.l, &mut p.emu, "menu_activeVoice");
        p.check(&format!("active voice {track}"));
        p.event(&Event::SetParameter { param: PAR_EUKLID_LENGTH, value: 8 + (rnd.next_u32() % 9) as u8 });
        p.event(&Event::SetParameter { param: PAR_EUKLID_STEPS, value: 1 + (rnd.next_u32() % 7) as u8 });
        p.event(&Event::SetParameter { param: PAR_EUKLID_ROTATION, value: (rnd.next_u32() % 4) as u8 });
        for _ in 0..4 {
            p.event(&Event::SetParameter { param: PAR_ACTIVE_STEP, value: (rnd.next_u32() % 128) as u8 });
            p.event(&Event::SetParameter { param: PAR_STEP_FLAM, value: (rnd.next_u32() % 8) as u8 });
            p.event(&Event::SetParameter { param: PAR_STEP_NOTE, value: rnd.u8() & 0x7f });
            p.event(&Event::SetParameter { param: PAR_STEP_PROB, value: 50 + (rnd.next_u32() % 51) as u8 });
        }
        p.check(&format!("patterns, track {track}"));
    }
}

#[test]
fn midi_clock_external_sync() {
    for_each_build(|p, build| {
        prologue(p);
        let mut rnd = Sweep(0xc10c_0001);
        setup_patterns(p, &mut rnd);
        p.event(&Event::SetParameter { param: PAR_SHUFFLE, value: 40 });
        // PAR_BPM = 0: external MIDI clock
        p.event(&Event::SetParameter { param: PAR_BPM, value: 0 });
        p.check(&format!("{build}: ext sync"));
        p.midi(&[0xFA]);
        for i in 0..2400u32 {
            // ~120 bpm: 24 clocks per quarter, one clock every ~3.7 blocks; jitter it
            if rnd.next_u32() % 4 == 0 {
                p.midi(&[0xF8]);
            }
            if i == 1200 {
                p.midi(&[0xFC]);
            }
            if i == 1300 {
                p.midi(&[0xFB]);
            }
            if i % 300 == 150 {
                p.event(&Event::SetParameter { param: PAR_SHUFFLE, value: (rnd.next_u32() % 128) as u8 });
            }
            p.tick(&format!("{build}: clock block {i}"));
            if i % 8 == 0 {
                p.check(&format!("{build}: clock block {i}"));
            }
        }
    });
}

/// Shipped build only: `Lxr::seq_processFlam` follows the shipped binary, which has no
/// `lastWasNoteOn` note-off branch; the reference ELF (built from the C source) sends those note-offs, so
/// `active_voices` / `midi_notes_on` differ from it as soon as a sequenced note is followed by an empty step.
#[test]
fn internal_sequencer_started_by_mtc() {
    for_each_build(|p, build| {
        if p.emu.build == lxr_emu::Build::Elf {
            return;
        }
        prologue(p);
        let mut rnd = Sweep(0x5e9_0002);
        setup_patterns(p, &mut rnd);
        p.event(&Event::SetParameter { param: PAR_BPM, value: 150 });
        // MTC quarter frames: frame 0 resets the ignore flag, 0x70 (piece 7, even) starts the sequencer
        p.midi(&[0xF1, 0x00, 0xF1, 0x70]);
        p.check(&format!("{build}: MTC start"));
        let params = [PAR_SHUFFLE, PAR_ROLL, PAR_TRACK_LENGTH, PAR_PATTERN_NEXT, PAR_STEP_FLAM, PAR_ACTIVE_STEP, PAR_STEP_SHIFT, PAR_BPM];
        for i in 0..3000u32 {
            if i % 97 == 50 {
                let par = params[(rnd.next_u32() as usize) % params.len()];
                let value = match par {
                    PAR_BPM => 60 + (rnd.next_u32() % 180) as u8,
                    PAR_TRACK_LENGTH => 1 + (rnd.next_u32() % 64) as u8,
                    PAR_PATTERN_NEXT => (rnd.next_u32() % 8) as u8,
                    PAR_STEP_FLAM | PAR_ROLL => (rnd.next_u32() % 8) as u8,
                    PAR_STEP_SHIFT => (rnd.next_u32() % 15) as u8,
                    _ => (rnd.next_u32() % 128) as u8,
                };
                p.event(&Event::SetParameter { param: par, value });
                p.check(&format!("{build}: block {i}: param {par} = {value}"));
            }
            if i % 31 == 0 {
                p.midi(&[0x90 | (1 + (rnd.next_u32() % 7) as u8), rnd.u8() & 0x7f, rnd.u8() & 0x7f]);
            }
            // quarter frame piece 0 keeps MTC alive (midiParser_checkMtc stops the sequencer after 400 ms)
            if i % 32 == 16 && i < 2600 {
                p.midi(&[0xF1, 0x00]);
            }
            p.tick(&format!("{build}: sequencer block {i}"));
            if i == 2599 {
                assert!(p.l.seq_running != 0, "the sequencer should be running while MTC arrives");
            }
            if i % 8 == 0 {
                p.check(&format!("{build}: sequencer block {i}"));
            }
        }
        assert!(p.l.seq_running == 0, "the MTC timeout should have stopped the sequencer");
    });
}

/// Every parameter through `set_parameter` (menu path) over its legal range (`tests/scenarios/params.json`).
#[test]
fn set_parameter_all() {
    for_each_build(|p, build| {
        let s = std::fs::read_to_string(scenario_dir().join("params.json")).unwrap();
        let v: serde_json::Value = serde_json::from_str(&s).unwrap();
        let mut rnd = Sweep(0x9a7a_0001);
        for par in v["params"].as_array().unwrap() {
            let idx = par["index"].as_u64().unwrap() as u16;
            let (lo, hi) = (par["min"].as_u64().unwrap() as u32, par["max"].as_u64().unwrap() as u32);
            // PAR_MORPH towards the ITCM bytes renders out-of-range waveforms (see p3_scenarios NOT_EXACT)
            if idx == PAR_MORPH {
                continue;
            }
            let mut vals = vec![lo, hi, lo + rnd.next_u32() % (hi - lo + 1)];
            vals.dedup();
            for val in vals {
                p.event(&Event::SetParameter { param: idx, value: val as u8 });
                p.check(&format!("{build}: set_parameter({idx}, {val})"));
            }
            if idx % 4 == 0 {
                p.midi(&[0x90 | (1 + (idx % 7) as u8), 60, 100]);
                p.tick(&format!("{build}: after parameter {idx}"));
                p.tick(&format!("{build}: after parameter {idx}"));
            }
        }
    });
}

/// Bank select (CC 0) above 63 loads kits 64..127 from past the end of `mainParameters`
/// (`Lxr::preset_mainParametersByte`). In the shipped image that memory holds UI, SD card, USB and newlib
/// globals and then heap/stack; `Lxr` reads the modelled globals there and 0 elsewhere. After power-on the
/// emulated firmware has zeros there too, so every kit number matches except 68, whose byte 216 is
/// `usb_MidiInBuff[0]`, written (0x0f) by the firmware's USB code (`usb_tick`), which is not modelled.
/// On the device these kits depend on USB / SD / stack contents: a documented divergence
/// (docs/ENGINE_CONTRACT.md).
#[test]
fn bank_select_above_63() {
    let _fw = lxr_difftest::fw_or_skip!();
    let Some(img) = img_path() else {
        eprintln!("shipped image not available; skipping");
        return;
    };
    on_big_stack(move || {
        for v in 64..128u8 {
            let mut p = Pair::new(lxr_emu::EmuEngine::new_shipped(&img).unwrap());
            p.midi(&[0xB0, 0, v]);
            let d = p.diff();
            if v == 68 {
                assert_eq!(
                    d,
                    ["mixer_audioRouting+1: rust 0x00 fw 0x0f", "midiParser_originalCcValues+216: rust 0x00 fw 0x0f", "activeParameters+216: rust 0x00 fw 0x0f"],
                    "kit 68: only usb_MidiInBuff[0] (parameter 216) should differ"
                );
                continue;
            }
            assert!(d.is_empty(), "bank select {v}: state differs: {d:?}");
            for i in 0..4 {
                p.tick(&format!("bank select {v}, block {i}"));
            }
            p.check(&format!("bank select {v}"));
        }
    });
}
