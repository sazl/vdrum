//! P3: the two functions ported from the SHIPPED v1.71 binary instead of the C source, called directly in
//! the shipped image (`Firmware::from_bin` through `EmuEngine::new_shipped`, then `call_addr`) and compared
//! with the Rust port on the whole engine state.
//!
//! - `voiceControl_noteOn` (shipped 0x08016f48): the shipped function has no `seq_sendMidiNoteOn` call.
//! - `seq_processFlam` (shipped `seq_processFlam.part.0`, 0x080187f8, the body after the `seq_running` test):
//!   no `lastWasNoteOn` handling and no noteOff branch.
//!
//! Needs `LXR_FW_ELF` (gate) and the shipped image (`LXR_FIRMWARE_IMG`, default
//! `/home/user/lxr-ref/LXRV2_update_v1.71.img`); skipped otherwise.
#![allow(non_snake_case)]

mod p3_common;

use lxr_core::memmap::ADDR_seq_patternSet;
use lxr_difftest::Sweep;
use lxr_emu::EmuEngine;
use p3_common::*;

const SHIPPED_NOTE_ON: u32 = 0x0801_6f48;
const SHIPPED_PROCESS_FLAM_PART0: u32 = 0x0801_87f8;

fn shipped_pair() -> Option<Pair> {
    let img = img_path()?;
    Some(Pair::new(EmuEngine::new_shipped(img).unwrap()))
}

#[test]
fn voiceControl_noteOn_vs_shipped() {
    let _fw = lxr_difftest::fw_or_skip!();
    if img_path().is_none() {
        eprintln!("shipped image not available; skipping");
        return;
    }
    on_big_stack(|| {
        let mut p = shipped_pair().unwrap();
        let mut rnd = Sweep(0x5eed_0001);
        let mut n = 0;
        // systematic: every voice, edge velocities and notes
        for voice in 0..7u8 {
            for &(note, vel) in &[(0u8, 0u8), (48, 1), (60, 64), (127, 127), (36, 100)] {
                p.l.voiceControl_noteOn(voice, note, vel);
                call_fw(&mut p.emu, SHIPPED_NOTE_ON, &[voice as u32, note as u32, vel as u32]);
                p.check(&format!("noteOn({voice}, {note}, {vel})"));
                p.tick(&format!("block after noteOn({voice}, {note}, {vel})"));
                n += 1;
            }
        }
        // random, with audio rendered in between
        for i in 0..400 {
            let voice = (rnd.next_u32() % 7) as u8;
            let note = rnd.u8() & 0x7f;
            let vel = rnd.u8() & 0x7f;
            p.l.voiceControl_noteOn(voice, note, vel);
            call_fw(&mut p.emu, SHIPPED_NOTE_ON, &[voice as u32, note as u32, vel as u32]);
            p.check(&format!("random #{i}: noteOn({voice}, {note}, {vel})"));
            for _ in 0..(rnd.next_u32() % 4) {
                p.tick(&format!("random #{i}"));
            }
            n += 1;
        }
        p.check("end");
        eprintln!("{n} noteOn calls, {} blocks, state identical", p.blocks);
    });
}

/// Random sequencer state for `seq_processFlam`: step data and timing (flam, shift) of the active pattern,
/// track lengths (including 0), step indices (including -1), mutes, the choke flags.
fn randomize_seq(p: &mut Pair, rnd: &mut Sweep) {
    let l = &mut p.l;
    l.seq_activePattern = (rnd.next_u32() % 64) as u8;
    let ap = l.seq_activePattern as u32;
    // seq_subStepPattern[ap] (7 tracks x 64 steps x 7 bytes) and seq_stepTiming[ap] (7 x 64 x 1 byte)
    let sub = ADDR_seq_patternSet + ap * 7 * 64 * 7;
    for k in 0..7 * 64 * 7 {
        let b = rnd.u8();
        l.global_set_byte(sub + k, b);
    }
    let timing = ADDR_seq_patternSet + lxr_core::memmap::offset_of_c::PatternSet_seq_stepTiming + ap * 7 * 64;
    for k in 0..7 * 64 {
        // flam in the low bits, shift above; mostly flams
        let b = if rnd.next_u32() % 3 == 0 { rnd.u8() } else { (rnd.u8() & 0x0f) | (7 << 4) };
        l.global_set_byte(timing + k, b);
    }
    for t in 0..7 {
        let len = match rnd.next_u32() % 8 {
            0 => 0,
            1 => 64,
            _ => 1 + (rnd.next_u32() % 64) as u8,
        };
        l.seq_patternSet.length[ap as usize][t] = len;
        l.seq_stepIndex[t] = match rnd.next_u32() % 10 {
            0 => -1,
            _ => (rnd.next_u32() % 64) as i8,
        };
    }
    l.seq_mutedTracks = (rnd.next_u32() & 0x7f) as u8 & if rnd.next_u32() % 2 == 0 { 0 } else { 0x7f };
    l.chokeCurrent = (rnd.next_u32() & 1) as u8;
    l.chokeNext = (rnd.next_u32() & 1) as u8;
    l.seq_running = 1;
    for name in ["seq_patternSet", "seq_activePattern", "seq_stepIndex", "seq_mutedTracks", "chokeCurrent", "chokeNext", "seq_running"] {
        push_global(&p.l, &mut p.emu, name);
    }
    p.check("after randomizing the sequencer state");
}

#[test]
fn seq_processFlam_vs_shipped() {
    let _fw = lxr_difftest::fw_or_skip!();
    if img_path().is_none() {
        eprintln!("shipped image not available; skipping");
        return;
    }
    on_big_stack(|| {
        let mut p = shipped_pair().unwrap();
        let mut rnd = Sweep(0x5eed_0002);
        let mut calls = 0;
        for round in 0..60 {
            randomize_seq(&mut p, &mut rnd);
            // the 24 flam sub-steps of one step, as seq_tick calls them, plus a few out-of-range indices
            let mut idxs: Vec<u8> = (0..24).collect();
            if round % 4 == 0 {
                idxs.extend([24u8, 31, 32, 200, 255]);
            }
            for idx in idxs {
                p.l.seq_processFlam(idx);
                // seq_processFlam(idx) = `if (seq_running) seq_processFlam.part.0(idx)`
                call_fw(&mut p.emu, SHIPPED_PROCESS_FLAM_PART0, &[idx as u32]);
                p.check(&format!("round {round}: seq_processFlam({idx})"));
                calls += 1;
            }
            p.tick(&format!("round {round}"));
        }
        // not running: no effect
        p.l.seq_running = 0;
        push_global(&p.l, &mut p.emu, "seq_running");
        let before = state_diff(&p.l, &p.emu, &[]);
        p.l.seq_processFlam(0);
        assert_eq!(state_diff(&p.l, &p.emu, &[]), before, "seq_processFlam with seq_running = 0 changed the state");
        eprintln!("{calls} seq_processFlam calls, state identical");
    });
}
