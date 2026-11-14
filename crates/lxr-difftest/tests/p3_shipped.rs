//! P3: the function ported from the SHIPPED v1.71 binary instead of the C source, called directly in
//! the shipped image (`Firmware::from_bin` through `EmuEngine::new_shipped`, then `call_addr`) and compared
//! with the Rust port on the whole engine state.
//!
//! - `voiceControl_noteOn` (shipped 0x08016f48): the shipped function has no `seq_sendMidiNoteOn` call.
//!
//! The second one, `seq_processFlam`, was removed with the sequencer port on 2026-11-14.
//!
//! Needs `LXR_FW_ELF` (gate) and the shipped image (`LXR_FIRMWARE_IMG`, default
//! `/home/user/lxr-ref/LXRV2_update_v1.71.img`); skipped otherwise.
#![allow(non_snake_case)]

mod p3_common;

use lxr_difftest::Sweep;
use lxr_emu::EmuEngine;
use p3_common::*;

const SHIPPED_NOTE_ON: u32 = 0x0801_6f48;

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
