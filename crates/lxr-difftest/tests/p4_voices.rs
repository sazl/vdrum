//! P4 differential tests: drum voices 1-3 (DrumVoice.c), Snare.c, HiHat.c, CymbalVoice.c.
//!
//! Firmware and Rust start from the same state (the Rust state is pushed into the emulator memory with the
//! DWARF layout), run the same function with the same arguments, and every P4 global (all four voice
//! structs, velocity modulators, mixer arrays), the output buffers and the RNG state are compared bit for bit
//! after every call. FPSCR = 0 (the engine runs in thread mode with FZ=0, docs/ENGINE_CONTRACT.md); no FtzGuard.
#![allow(non_snake_case)]

mod p4_common;

use lxr_core::dsp::decay::{DecayEg_setDecay, DecayEg_setSlope};
use lxr_core::dsp::distortion::setDistortionShape;
use lxr_core::dsp::resonant_filter::{SVF_directSetFilterValue, SVF_setDrive, SVF_setReso};
use lxr_core::dsp::slope_eg2::{slopeEg2_calcDecay, slopeEg2_setAttack, slopeEg2_setDecay, slopeEg2_setSlope};
use lxr_core::dsp::transient_generator::transient_setWaveform;
use lxr_core::prelude::*;
use lxr_difftest::{fw_or_skip, Firmware, Sweep};
use p4_common::*;
use std::cell::Cell;
use std::rc::Rc;

#[derive(Clone, Copy, Debug, PartialEq)]
enum Kind {
    Drum(u8),
    Snare,
    Hat,
    Cym,
}
const KINDS: [Kind; 6] = [Kind::Drum(0), Kind::Drum(1), Kind::Drum(2), Kind::Snare, Kind::Cym, Kind::Hat];

struct Pair {
    fw: Firmware,
    rng: Rc<Cell<u32>>,
    l: Box<Lxr>,
    buf: u32,
}

impl Pair {
    fn new(fw: Firmware) -> Self {
        let (mut fw, rng) = setup(fw);
        let buf = fw.alloc(512).unwrap();
        let mut l = Box::new(Lxr::new_zeroed());
        l.adc_faderValues = [FADER_FULL; 6];
        let mut p = Pair { fw, rng, l, buf };
        p.push();
        p
    }

    /// Rust state -> firmware memory.
    fn push(&mut self) {
        push_state(&mut self.fw, &self.l);
        self.rng.set(self.l.rng_state);
    }

    #[track_caller]
    fn check(&self, ctx: &str) {
        check_state(ctx, &self.fw, &self.l);
        assert_eq!(self.l.rng_state, self.rng.get(), "{ctx}: RNG state (number of GetRngValue calls)");
    }

    fn init(&mut self, k: Kind) {
        match k {
            Kind::Drum(_) => {
                self.fw.call("initDrumVoice", &[], &[]).unwrap();
                self.l.initDrumVoice();
            }
            Kind::Snare => {
                self.fw.call("Snare_init", &[], &[]).unwrap();
                self.l.Snare_init();
            }
            Kind::Hat => {
                self.fw.call("HiHat_init", &[], &[]).unwrap();
                self.l.HiHat_init();
            }
            Kind::Cym => {
                self.fw.call("Cymbal_init", &[], &[]).unwrap();
                self.l.Cymbal_init();
            }
        }
    }

    fn trigger(&mut self, k: Kind, vel: u8, note: u8, isOpen: u8) {
        match k {
            Kind::Drum(v) => {
                self.fw.call("Drum_trigger", &[v as u32, vel as u32, note as u32], &[]).unwrap();
                self.l.Drum_trigger(v, vel, note);
            }
            Kind::Snare => {
                self.fw.call("Snare_trigger", &[vel as u32, note as u32], &[]).unwrap();
                self.l.Snare_trigger(vel, note);
            }
            Kind::Hat => {
                self.fw.call("HiHat_trigger", &[vel as u32, isOpen as u32, note as u32], &[]).unwrap();
                self.l.HiHat_trigger(vel, isOpen, note);
            }
            Kind::Cym => {
                self.fw.call("Cymbal_trigger", &[vel as u32, note as u32], &[]).unwrap();
                self.l.Cymbal_trigger(vel, note);
            }
        }
    }

    fn calc_async(&mut self, k: Kind) {
        match k {
            Kind::Drum(v) => {
                self.fw.call("calcDrumVoiceAsync", &[v as u32], &[]).unwrap();
                self.l.calcDrumVoiceAsync(v);
            }
            Kind::Snare => {
                self.fw.call("Snare_calcAsync", &[], &[]).unwrap();
                self.l.Snare_calcAsync();
            }
            Kind::Hat => {
                self.fw.call("HiHat_calcAsync", &[], &[]).unwrap();
                self.l.HiHat_calcAsync();
            }
            Kind::Cym => {
                self.fw.call("Cymbal_calcAsync", &[], &[]).unwrap();
                self.l.Cymbal_calcAsync();
            }
        }
    }

    /// Run the sync block function on a buffer pre-filled with `init` (the functions overwrite it; garbage in
    /// unused entries must not leak). Returns the Rust output, after asserting equality with the firmware.
    #[track_caller]
    fn calc_sync(&mut self, k: Kind, size: u8, init: &[i16; 256], ctx: &str) -> Vec<i16> {
        self.fw.write_i16s(self.buf, init).unwrap();
        let mut rb = *init;
        let b = self.buf;
        match k {
            Kind::Drum(v) => {
                self.fw.call("calcDrumVoiceSyncBlock", &[v as u32, b, size as u32], &[]).unwrap();
                self.l.calcDrumVoiceSyncBlock(v, &mut rb, size);
            }
            Kind::Snare => {
                self.fw.call("Snare_calcSyncBlock", &[b, size as u32], &[]).unwrap();
                self.l.Snare_calcSyncBlock(&mut rb, size);
            }
            Kind::Hat => {
                self.fw.call("HiHat_calcSyncBlock", &[b, size as u32], &[]).unwrap();
                self.l.HiHat_calcSyncBlock(&mut rb, size);
            }
            Kind::Cym => {
                self.fw.call("Cymbal_calcSyncBlock", &[b, size as u32], &[]).unwrap();
                self.l.Cymbal_calcSyncBlock(&mut rb, size);
            }
        }
        let fb = self.fw.read_i16s(self.buf, 256).unwrap();
        if fb[..] != rb[..] {
            let i = (0..256).find(|&i| fb[i] != rb[i]).unwrap();
            panic!("{ctx}: output buffer differs at sample {i}: rust {} fw {}", rb[i], fb[i]);
        }
        rb[..size as usize].to_vec()
    }
}

/// A float parameter value: mostly the 0..1 range of `value.flt` from the parameter array (n/127), plus
/// edge cases.
fn pf(s: &mut Sweep) -> f32 {
    match s.next_u32() % 16 {
        0 => 0.0,
        1 => 1.0,
        2 => s.f32_in(1.0, 4.0),
        3 => s.f32_in(-1.0, 0.0),
        _ => (s.next_u32() % 128) as f32 / 127.0,
    }
}

/// Randomise the sound parameters of voice `k` in the Rust state (as `midiParser_ccHandler` would set them,
/// through the P1 setters where the parameter has one). The state is then pushed to the firmware.
fn randomize(l: &mut Lxr, k: Kind, s: &mut Sweep) {
    let u7 = |s: &mut Sweep| (s.next_u32() % 128) as u8;
    let wave = |s: &mut Sweep| (s.next_u32() % 6) as u8;
    let ftype = |s: &mut Sweep| (s.next_u32() % 9) as u8;
    let sync = |s: &mut Sweep| (s.next_u32() % 2) as u8;
    macro_rules! common {
        ($v:expr) => {{
            let v = $v;
            v.vol = pf(s);
            v.pan = u7(s);
            v.osc.waveform = wave(s);
            v.pwm = pf(s);
            v.volumeMod = (s.next_u32() % 2) as u8;
            transient_setWaveform(&mut v.transGen, (s.next_u32() % 13) as u8);
            if s.next_u32() % 3 == 0 {
                v.transGen.waveform = (s.next_u32() % 2) as u8;
            }
            v.transGen.volume = pf(s);
            v.transGen.pitch = s.f32_in(0.0, 3.0);
            slopeEg2_setAttack(&mut v.oscVolEg, if s.next_u32() % 4 == 0 { 0 } else { u7(s) }, sync(s));
            slopeEg2_setDecay(&mut v.oscVolEg, u7(s), sync(s));
            slopeEg2_setSlope(&mut v.oscVolEg, u7(s));
            v.oscVolEg.repeat = (s.next_u32() % 3) as u8;
            SVF_directSetFilterValue(&mut v.filter, pf(s).clamp(0.0, 1.0));
            SVF_setReso(&mut v.filter, pf(s).clamp(0.0, 1.0));
            SVF_setDrive(&mut v.filter, pf(s).clamp(0.0, 1.0));
            v.filterType = ftype(s);
            setDistortionShape(&mut v.distortion, pf(s).clamp(0.0, 1.0));
            v.lfo.retrigger = (s.next_u32() % 8) as u8;
            v.lfo.phaseOffset = s.next_u32();
        }};
    }
    match k {
        Kind::Drum(n) => {
            let v = &mut l.voiceArray[n as usize];
            common!(&mut *v);
            v.modOsc.waveform = wave(s);
            v.fmModAmount = pf(s);
            v.egPitchModAmount = s.f32_in(0.0, 8.0);
            DecayEg_setDecay(&mut v.oscPitchEg, u7(s));
            DecayEg_setSlope(&mut v.oscPitchEg, u7(s));
            v.mixOscs = (s.next_u32() % 2) as u8;
        }
        Kind::Snare => {
            let v = &mut l.snareVoice;
            common!(&mut *v);
            v.mix = pf(s);
            v.egPitchModAmount = s.f32_in(0.0, 8.0);
            DecayEg_setDecay(&mut v.oscPitchEg, u7(s));
            DecayEg_setSlope(&mut v.oscPitchEg, u7(s));
        }
        Kind::Hat => {
            let v = &mut l.hatVoice;
            common!(&mut *v);
            v.modOsc.waveform = wave(s);
            v.modOsc2.waveform = wave(s);
            v.fmModAmount1 = pf(s);
            v.fmModAmount2 = pf(s);
            v.decayClosed = slopeEg2_calcDecay(u7(s)) as f32;
            v.decayOpen = slopeEg2_calcDecay(u7(s)) as f32;
        }
        Kind::Cym => {
            let v = &mut l.cymbalVoice;
            common!(&mut *v);
            v.modOsc.waveform = wave(s);
            v.modOsc2.waveform = wave(s);
            v.fmModAmount1 = pf(s);
            v.fmModAmount2 = pf(s);
        }
    }
    if s.next_u32() % 4 == 0 {
        l.adc_faderValues[match k {
            Kind::Drum(n) => n as usize,
            Kind::Snare => 3,
            Kind::Cym => 4,
            Kind::Hat => 5,
        }] = pf(s).clamp(0.0, 1.0);
    }
}

#[test]
fn p4_layout_roundtrip() {
    // the generated marshalling covers every byte the firmware uses: firmware -> Rust -> firmware is identity
    // on the init state, and struct sizes match the DWARF
    let fw = fw_or_skip!();
    let mut p = Pair::new(fw);
    let raw = p.fw.read(lxr_core::memmap::ADDR_voiceArray, 704).unwrap();
    let v: DrumVoice = get(&p.fw, lxr_core::memmap::ADDR_voiceArray);
    put(&mut p.fw, lxr_core::memmap::ADDR_voiceArray, &v);
    assert_eq!(raw, p.fw.read(lxr_core::memmap::ADDR_voiceArray, 704).unwrap());
    assert_eq!(<DrumVoice as CMem>::SIZE, 704);
    assert_eq!(<SnareVoice as CMem>::SIZE, 368);
    assert_eq!(<HiHatVoice as CMem>::SIZE, 392);
    assert_eq!(<CymbalVoice as CMem>::SIZE, 384);
    // random bit patterns survive Rust -> firmware -> Rust
    let mut s = Sweep(0x1234_5678_9abc);
    let mut b = vec![0u8; 704];
    for x in b.iter_mut() {
        *x = s.u8();
    }
    let v = DrumVoice::rd(&b, 0);
    put(&mut p.fw, lxr_core::memmap::ADDR_voiceArray, &v);
    let w: DrumVoice = get(&p.fw, lxr_core::memmap::ADDR_voiceArray);
    assert_same("roundtrip", &v, &w);
}

#[test]
fn p4_setPan_and_jack_routing() {
    let fw = fw_or_skip!();
    let mut p = Pair::new(fw);
    let mut s = Sweep(77);
    for i in 0..300 {
        let pan = s.u8();
        let v = (s.next_u32() % 3) as u8;
        match i % 4 {
            0 => {
                p.fw.call("setPan", &[v as u32, pan as u32], &[]).unwrap();
                p.l.setPan(v, pan);
            }
            1 => {
                p.fw.call("Snare_setPan", &[pan as u32], &[]).unwrap();
                p.l.Snare_setPan(pan);
            }
            2 => {
                p.fw.call("HiHat_setPan", &[pan as u32], &[]).unwrap();
                p.l.HiHat_setPan(pan);
            }
            _ => {
                p.fw.call("Cymbal_setPan", &[pan as u32], &[]).unwrap();
                p.l.Cymbal_setPan(pan);
            }
        }
    }
    p.check("setPan");
    // mixer_checkOutJackAvailable: every destination (incl. invalid ones) x every jack combination
    for jacks in 0..16u8 {
        p.l.l1_Available = jacks & 1;
        p.l.l2_Available = (jacks >> 1) & 1;
        p.l.r1_Available = (jacks >> 2) & 1;
        p.l.r2_Available = (jacks >> 3) & 1;
        p.push();
        for dest in 0..=255u8 {
            let (r0, _) = p.fw.call("mixer_checkOutJackAvailable", &[dest as u32], &[]).unwrap();
            assert_eq!(p.l.mixer_checkOutJackAvailable(dest) as u32, r0 & 0xff, "dest {dest} jacks {jacks:04b}");
        }
    }
}

#[test]
fn p4_mixer_decimateBlock() {
    let fw = fw_or_skip!();
    let mut p = Pair::new(fw);
    p.fw.call("mixer_init", &[], &[]).ok(); // FxBus_init is P2; only the arrays matter here
    let mut s = Sweep(99);
    let b = p.fw.alloc(128).unwrap();
    for it in 0..3000 {
        let v = (s.next_u32() % 6) as u8;
        p.l.mixer_decimation_rate[v as usize] = match s.next_u32() % 6 {
            0 => 1.0,
            1 => 0.0,
            2 => s.f32_in(0.0, 3.0),
            _ => s.f32_in(0.0, 1.0),
        };
        if it % 7 == 0 {
            p.l.mixer_decimation_rate[6] = s.f32_in(0.0, 1.5);
        }
        p.push();
        let data: Vec<i16> = (0..64).map(|_| s.next_u32() as i16).collect();
        p.fw.write_i16s(b, &data).unwrap();
        let mut rd = [0i16; 64];
        rd.copy_from_slice(&data);
        p.fw.call("mixer_decimateBlock", &[v as u32, b], &[]).unwrap();
        p.l.mixer_decimateBlock(v, &mut rd);
        assert_eq!(p.fw.read_i16s(b, 64).unwrap(), rd.to_vec(), "iteration {it}");
        p.check(&format!("decimate {it}"));
    }
}

#[test]
fn p4_init_all_voices() {
    let fw = fw_or_skip!();
    let mut p = Pair::new(fw);
    // init from a garbage state: every field the init does not write must stay as it was
    let mut s = Sweep(5);
    for round in 0..4 {
        let mut b = vec![0u8; 704];
        for x in b.iter_mut() {
            *x = s.u8();
        }
        for v in 0..3 {
            p.l.voiceArray[v] = DrumVoice::rd(&b, 0);
        }
        p.l.snareVoice = SnareVoice::rd(&b, 0);
        p.l.hatVoice = HiHatVoice::rd(&b, 0);
        p.l.cymbalVoice = CymbalVoice::rd(&b, 0);
        if round == 0 {
            p.l.voiceArray = [DrumVoice::ZERO; 3];
        }
        p.push();
        for k in [Kind::Drum(0), Kind::Snare, Kind::Hat, Kind::Cym] {
            p.init(k);
            p.check(&format!("init {k:?} round {round}"));
        }
    }
}

#[test]
fn p4_trigger_sweep() {
    let fw = fw_or_skip!();
    let mut p = Pair::new(fw);
    for k in [Kind::Drum(0), Kind::Snare, Kind::Hat, Kind::Cym] {
        p.init(k);
    }
    p.check("init");
    let mut s = Sweep(0xC0FFEE);
    for it in 0..4000 {
        let k = KINDS[(s.next_u32() % 6) as usize];
        if it % 3 == 0 {
            randomize(&mut p.l, k, &mut s);
            p.push();
        }
        let vel = match s.next_u32() % 8 {
            0 => 0,
            1 => 127,
            2 => 255,
            _ => s.u8() & 0x7f,
        };
        let note = s.u8() & 0x7f;
        let open = (s.next_u32() % 3) as u8;
        p.trigger(k, vel, note, open);
        p.check(&format!("trigger {it} {k:?} vel {vel} note {note}"));
    }
}

/// Long runs: trigger, then async + sync blocks per voice, with parameter changes and retriggers in between.
fn block_run(seed: u64, kinds: &[Kind], blocks: usize, size_mode: u8) {
    let fw = fw_or_skip!();
    let mut p = Pair::new(fw);
    for k in [Kind::Drum(0), Kind::Snare, Kind::Hat, Kind::Cym] {
        p.init(k);
    }
    let mut s = Sweep(seed);
    let mut garbage = [0i16; 256];
    let mut nonzero = 0usize;
    for blk in 0..blocks {
        for &k in kinds {
            if s.next_u32() % 40 == 0 {
                randomize(&mut p.l, k, &mut s);
                p.push();
            }
            if s.next_u32() % 25 == 0 {
                let vel = s.u8() & 0x7f;
                let note = s.u8() & 0x7f;
                p.trigger(k, vel, note, (s.next_u32() % 2) as u8);
            }
            p.calc_async(k);
            for x in garbage.iter_mut() {
                *x = s.next_u32() as i16;
            }
            let size = match size_mode {
                0 => 64,
                _ => match s.next_u32() % 8 {
                    0 => 0,
                    1 => 1,
                    2 => 3,
                    3 => s.u8(),
                    _ => 64,
                },
            };
            let out = p.calc_sync(k, size, &garbage, &format!("seed {seed} block {blk} {k:?} size {size}"));
            nonzero += out.iter().filter(|&&x| x != 0).count();
            p.check(&format!("seed {seed} block {blk} {k:?}"));
        }
    }
    assert!(nonzero > 0, "the run produced only silence; the test is not exercising the voices");
    eprintln!("seed {seed}: {blocks} blocks x {} voices, {nonzero} non-zero samples", kinds.len());
}

#[test]
fn p4_drum_voices_blocks() {
    for seed in 1..=3 {
        block_run(seed, &[Kind::Drum(0), Kind::Drum(1), Kind::Drum(2)], 1500, 0);
    }
}

#[test]
fn p4_snare_blocks() {
    for seed in 11..=13 {
        block_run(seed, &[Kind::Snare], 3000, 0);
    }
}

#[test]
fn p4_hihat_blocks() {
    for seed in 21..=23 {
        block_run(seed, &[Kind::Hat], 3000, 0);
    }
}

#[test]
fn p4_cymbal_blocks() {
    for seed in 31..=33 {
        block_run(seed, &[Kind::Cym], 3000, 0);
    }
}

#[test]
fn p4_all_voices_odd_block_sizes() {
    block_run(41, &KINDS, 1500, 1);
}
