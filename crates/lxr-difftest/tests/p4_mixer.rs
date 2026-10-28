//! P4 differential tests: `mixer_calcNextSampleBlock` (all six voices, routing to DAC1/DAC2/FX bus,
//! decimation, triggered pan) over long block runs with triggers and parameter changes.
//! FPSCR = 0 (docs/ENGINE_CONTRACT.md), no FtzGuard. The FX bus runs with `FX_OFF` (its effects are P2's
//! tests); its routing is swept.
#![allow(non_snake_case)]

mod p4_common;

use lxr_core::memmap::{self, offset_of_c as O};
use lxr_core::prelude::*;
use lxr_difftest::{fw_or_skip, Sweep};
use p4_common::*;

fn set_fx_routing(fw: &mut lxr_difftest::Firmware, l: &mut Lxr, fxType: u8, routing: u8) {
    l.fxBus.fxType = fxType;
    l.fxBus.audioRouting = routing;
    fw.write_u8(memmap::ADDR_fxBus + O::FxBus_fxType, fxType).unwrap();
    fw.write_u8(memmap::ADDR_fxBus + O::FxBus_audioRouting, routing).unwrap();
}

fn mixer_run(seed: u64, blocks: usize) {
    let fw = fw_or_skip!();
    let (mut fw, rng) = setup(fw);
    let mut l = Box::new(Lxr::new_zeroed());
    l.adc_faderValues = [FADER_FULL; 6];
    push_state(&mut fw, &l);
    for f in ["initDrumVoice", "Snare_init", "HiHat_init", "Cymbal_init", "mixer_init"] {
        fw.call(f, &[], &[]).unwrap();
    }
    l.initDrumVoice();
    l.Snare_init();
    l.HiHat_init();
    l.Cymbal_init();
    l.mixer_init();
    rng.set(l.rng_state);
    set_fx_routing(&mut fw, &mut l, FX_OFF, 0);
    check_state("init", &fw, &l);

    let out = fw.alloc(256).unwrap();
    let out2 = fw.alloc(256).unwrap();
    let mut s = Sweep(seed);
    let mut nonzero = 0usize;
    for blk in 0..blocks {
        let mut changed = false;
        if blk % 50 == 0 {
            for v in 0..6 {
                l.mixer_audioRouting[v] = (s.next_u32() % 8) as u8; // 7 = invalid: no output
                l.mixer_decimation_rate[v] = if s.next_u32() % 2 == 0 { 1.0 } else { s.f32_in(0.0, 1.0) };
            }
            l.mixer_decimation_rate[6] = if s.next_u32() % 2 == 0 { 1.0 } else { s.f32_in(0.2, 1.0) };
            let r = (s.next_u32() % 7) as u8;
            set_fx_routing(&mut fw, &mut l, FX_OFF, r);
            changed = true;
        }
        if blk % 10 == 0 {
            for v in 0..3 {
                l.voiceArray[v].pan = (s.next_u32() % 128) as u8;
                l.voiceArray[v].vol = s.f32_in(0.0, 1.0);
            }
            l.snareVoice.pan = (s.next_u32() % 128) as u8;
            l.hatVoice.pan = (s.next_u32() % 128) as u8;
            l.cymbalVoice.pan = (s.next_u32() % 128) as u8;
            changed = true;
        }
        if changed {
            push_state(&mut fw, &l);
        }
        // triggers (main-loop context between interrupts)
        for k in 0..6u8 {
            if s.next_u32() % 12 == 0 {
                let vel = (s.next_u32() % 128) as u8;
                let note = (s.next_u32() % 128) as u8;
                match k {
                    0..=2 => {
                        fw.call("Drum_trigger", &[k as u32, vel as u32, note as u32], &[]).unwrap();
                        l.Drum_trigger(k, vel, note);
                    }
                    3 => {
                        fw.call("Snare_trigger", &[vel as u32, note as u32], &[]).unwrap();
                        l.Snare_trigger(vel, note);
                    }
                    4 => {
                        fw.call("Cymbal_trigger", &[vel as u32, note as u32], &[]).unwrap();
                        l.Cymbal_trigger(vel, note);
                    }
                    _ => {
                        let open = (s.next_u32() % 2) as u8;
                        fw.call("HiHat_trigger", &[vel as u32, open as u32, note as u32], &[]).unwrap();
                        l.HiHat_trigger(vel, open, note);
                    }
                }
            }
        }
        // garbage in the output buffers: the mixer clears them
        let g1: Vec<i16> = (0..128).map(|_| s.next_u32() as i16).collect();
        let g2: Vec<i16> = (0..128).map(|_| s.next_u32() as i16).collect();
        fw.write_i16s(out, &g1).unwrap();
        fw.write_i16s(out2, &g2).unwrap();
        let mut o1 = [0i16; 128];
        let mut o2 = [0i16; 128];
        o1.copy_from_slice(&g1);
        o2.copy_from_slice(&g2);
        fw.call("mixer_calcNextSampleBlock", &[out, out2], &[]).unwrap();
        l.mixer_calcNextSampleBlock(&mut o1, &mut o2);
        let f1 = fw.read_i16s(out, 128).unwrap();
        let f2 = fw.read_i16s(out2, 128).unwrap();
        assert_eq!(f1, o1.to_vec(), "seed {seed} block {blk}: output (dac2)");
        assert_eq!(f2, o2.to_vec(), "seed {seed} block {blk}: output2 (dac1)");
        check_state(&format!("seed {seed} block {blk}"), &fw, &l);
        assert_eq!(l.rng_state, rng.get(), "seed {seed} block {blk}: RNG state");
        nonzero += o1.iter().chain(o2.iter()).filter(|&&x| x != 0).count();
    }
    assert!(nonzero > 0, "mixer produced only silence");
    eprintln!("seed {seed}: {blocks} blocks, {nonzero} non-zero output samples");
}

#[test]
fn p4_mixer_blocks() {
    for seed in 1..=3 {
        mixer_run(seed, 2000);
    }
}
