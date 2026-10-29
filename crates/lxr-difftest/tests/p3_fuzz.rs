//! P3 robustness: random MIDI byte streams (every status, CC 0..127 with every value including bank select
//! above 63, NRPN, sysex, realtime, MTC, raw garbage) and random parameter changes go into `Lxr` with audio
//! rendered in between; the test asserts that nothing panics. Meant to be run in a debug build as well
//! (`cargo test -p lxr-difftest --test p3_fuzz`), where integer overflow checks are on.
//!
//! Needs no firmware. The bit-exact comparison of such streams with the emulator is in `p3_midi`
//! (`random_streams`, `all_ccs_on_all_channels`, `nrpn_sequences`), which leaves out bank select values
//! above 63: those load kits 64..127, which the firmware reads past `mainParameters` from memory the engine
//! does not model (`Lxr::preset_mainParametersByte`).

use lxr_core::prelude::*;
use lxr_difftest::Sweep;

fn on_big_stack<F: FnOnce() + Send + 'static>(f: F) {
    std::thread::Builder::new().stack_size(64 << 20).spawn(f).unwrap().join().unwrap();
}

fn message(rnd: &mut Sweep) -> Vec<u8> {
    let ch = (rnd.next_u32() % 16) as u8;
    let d = |r: &mut Sweep| r.u8() & 0x7f;
    match rnd.next_u32() % 20 {
        0..=4 => vec![0xB0 | ch, d(rnd), d(rnd)],
        5 => vec![0xB0 | ch, 0, d(rnd)], // bank select, any value
        6 => vec![0xB0 | ch, 99, d(rnd), 0xB0 | ch, 98, d(rnd), 0xB0 | ch, 6, d(rnd), 38, d(rnd)],
        7 => vec![0xB0 | ch, 101, d(rnd), 100, d(rnd), 6, d(rnd)],
        8..=9 => vec![0x90 | ch, d(rnd), d(rnd)],
        10 => vec![0x80 | ch, d(rnd), d(rnd)],
        11 => vec![0xC0 | ch, d(rnd)],
        12 => vec![0xA0 | ch, d(rnd), d(rnd), 0xD0 | ch, d(rnd), 0xE0 | ch, d(rnd), d(rnd)],
        13 => vec![d(rnd), d(rnd), d(rnd)],
        14 => vec![0xF8 + (rnd.next_u32() % 8) as u8],
        15 => vec![0xF1, d(rnd), 0xF2, d(rnd), d(rnd), 0xF3, d(rnd)],
        16 => {
            let mut v = vec![0xF0];
            v.extend((0..rnd.next_u32() % 12).map(|_| d(rnd)));
            v.push(0xF7);
            v
        }
        _ => (0..1 + rnd.next_u32() % 6).map(|_| rnd.u8()).collect(),
    }
}

#[test]
fn random_midi_streams_do_not_panic() {
    let seeds: u64 = std::env::var("LXR_P3_FUZZ_SEEDS").ok().map(|s| s.parse().unwrap()).unwrap_or(32);
    let len: u32 = if cfg!(debug_assertions) { 1500 } else { 6000 };
    on_big_stack(move || {
        for seed in 0..seeds {
            let mut rnd = Sweep(0xf022_0000 + seed);
            let mut l = Box::new(Lxr::new());
            if seed % 2 == 1 {
                // voices on channels 1..7, any note (as most scenarios)
                for v in 0..6u16 {
                    l.set_parameter(PAR_MIDI_CHAN_1 + v, v as u8 + 1);
                }
                l.set_parameter(PAR_MIDI_CHAN_7, 7);
            }
            if seed % 4 == 2 {
                l.set_parameter(PAR_BPM, 0); // external clock: F8/FA/FB/FC drive the sequencer
            }
            for i in 0..len {
                let m = message(&mut rnd);
                for &b in &m {
                    l.midi_byte(b);
                }
                if rnd.next_u32() % 3 == 0 {
                    l.tick();
                }
                if i % 500 == 250 {
                    // MTC start of the internal sequencer
                    l.midi(&[0xF1, 0x00, 0xF1, 0x70]);
                }
            }
        }
    });
}

/// Bank select with every value on every channel, directly after power-on and with a running sequencer.
#[test]
fn bank_select_all_values_do_not_panic() {
    on_big_stack(|| {
        let mut l = Box::new(Lxr::new());
        for ch in 0..16u8 {
            for v in 0..128u8 {
                l.midi(&[0xB0 | ch, 0, v]);
                // running status, then CC 32 (bank select LSB)
                l.midi(&[0, v ^ 0x7f, 32, v]);
            }
            l.tick();
        }
        l.midi(&[0xF1, 0x00, 0xF1, 0x70]);
        for v in 0..128u8 {
            l.midi(&[0xB0, 0, v]);
            for _ in 0..4 {
                l.tick();
            }
        }
    });
}

/// Every parameter through `set_parameter` with random values over the whole byte range (not only the
/// menu's legal range), random kits through `load_kit`, MIDI notes and rendering in between.
#[test]
fn random_parameters_and_kits_do_not_panic() {
    let seeds: u64 = std::env::var("LXR_P3_FUZZ_SEEDS").ok().map(|s| s.parse().unwrap()).unwrap_or(32);
    on_big_stack(move || {
        for seed in 0..seeds.div_ceil(8) {
            let mut rnd = Sweep(0x5e7_0000 + seed);
            let mut l = Box::new(Lxr::new());
            l.midi(&[0xF1, 0x00, 0xF1, 0x70]);
            for i in 0..4000u32 {
                let param = (rnd.next_u32() % NUM_PARAMS as u32) as u16;
                let value = rnd.u8();
                if std::env::var("LXR_FUZZ_TRACE").is_ok() {
                    eprintln!("set_parameter({param}, {value})");
                }
                l.set_parameter(param, value);
                if i % 7 == 0 {
                    l.midi(&[0x90 | (rnd.next_u32() % 8) as u8, rnd.u8() & 0x7f, rnd.u8() & 0x7f]);
                    l.tick();
                }
                if i % 500 == 0 {
                    let n = 8 + (rnd.next_u32() % (END_OF_SOUND_PARAMETERS as u32 + 1)) as usize;
                    let kit: Vec<u8> = (0..n).map(|_| rnd.u8()).collect();
                    l.load_kit(&kit);
                    l.tick();
                }
            }
        }
    });
}
