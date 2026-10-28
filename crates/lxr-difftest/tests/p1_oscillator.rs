//! P1 differential tests: Oscillator (all waveforms, FM, PWM, noise), frequency setters, cmsis QADD16/QSUB16.
mod p1_common;

use lxr_core::dsp::oscillator::*;
use lxr_core::dsp::wavetable::{recTable, sawTable, triTable};
use lxr_core::midi::midi_parser::midiParser_calcDetune;
use lxr_core::rng::xorshift32;
use lxr_core::Lxr;
use lxr_difftest::{fw_or_skip, Firmware, Sweep};
use p1_common::*;
use std::cell::Cell;
use std::rc::Rc;

fn w_osc(fw: &mut Firmware, a: u32, o: &OscInfo) {
    w_u16(fw, a, o.output as u16);
    w_u32(fw, a + 4, o.phaseInc);
    w_u32(fw, a + 8, o.phase);
    w_f32(fw, a + 12, o.freq);
    w_u8(fw, a + 16, o.waveform);
    w_u8(fw, a + 17, o.tableOffset);
    w_f32(fw, a + 20, o.pitchMod);
    w_f32(fw, a + 24, o.fmMod);
    w_f32(fw, a + 28, o.modNodeValue);
    w_u16(fw, a + 32, o.midiFreq);
    w_u8(fw, a + 34, o.baseNote);
    w_u32(fw, a + 36, o.startPhase);
}
fn r_osc(fw: &Firmware, a: u32) -> OscInfo {
    OscInfo {
        output: r_u16(fw, a) as i16,
        phaseInc: r_u32(fw, a + 4),
        phase: r_u32(fw, a + 8),
        freq: r_f32(fw, a + 12),
        waveform: r_u8(fw, a + 16),
        tableOffset: r_u8(fw, a + 17),
        pitchMod: r_f32(fw, a + 20),
        fmMod: r_f32(fw, a + 24),
        modNodeValue: r_f32(fw, a + 28),
        midiFreq: r_u16(fw, a + 32),
        baseNote: r_u8(fw, a + 34),
        startPhase: r_u32(fw, a + 36),
    }
}
#[track_caller]
fn eq_osc(ctx: &str, f: &OscInfo, r: &OscInfo) {
    assert_eq!(
        (f.output, f.phaseInc, f.phase, f.waveform, f.tableOffset, f.midiFreq, f.baseNote, f.startPhase),
        (r.output, r.phaseInc, r.phase, r.waveform, r.tableOffset, r.midiFreq, r.baseNote, r.startPhase),
        "{ctx}"
    );
    eq32(&format!("{ctx} freq"), f.freq, r.freq);
    eq32(&format!("{ctx} pitchMod"), f.pitchMod, r.pitchMod);
    eq32(&format!("{ctx} fmMod"), f.fmMod, r.fmMod);
    eq32(&format!("{ctx} modNodeValue"), f.modNodeValue, r.modNodeValue);
}

/// Replace the firmware's GetRngValue by the deterministic xorshift32 that lxr-core uses; returns the shared state.
fn hook_rng(fw: &mut Firmware) -> Rc<Cell<u32>> {
    let st = Rc::new(Cell::new(0x2545_F491u32));
    let s2 = st.clone();
    let addr = fw.addr("GetRngValue").unwrap();
    fw.intercept(addr, move |uc| {
        let mut x = s2.get();
        let v = xorshift32(&mut x);
        s2.set(x);
        uc.reg_write(unicorn_engine::RegisterARM::R0, v as u64).unwrap();
    })
    .unwrap();
    st
}

fn random_osc(sw: &mut Sweep, waveform: u8) -> OscInfo {
    OscInfo {
        output: sw.next_u32() as i16,
        phaseInc: match sw.next_u32() % 4 {
            0 => sw.next_u32(),
            1 => sw.next_u32() >> 8,
            2 => 0,
            _ => sw.next_u32() >> 4,
        },
        phase: sw.next_u32(),
        freq: sw.f32_in(10.0, 20000.0),
        waveform,
        tableOffset: (sw.next_u32() % 11) as u8,
        pitchMod: sw.f32_in(0.0, 4.0),
        fmMod: match sw.next_u32() % 5 {
            0 => 0.0,
            1 => f32_special(sw, -10.0, 10.0),
            2 => sw.f32_in(-1e6, 1e6),
            _ => sw.f32_in(0.0, 2.0),
        },
        modNodeValue: sw.f32_in(0.0, 2.0),
        midiFreq: sw.next_u32() as u16,
        baseNote: sw.u8(),
        startPhase: sw.next_u32(),
    }
}

fn rand_buf(sw: &mut Sweep) -> Vec<i16> {
    let mode = sw.next_u32() % 4;
    (0..256)
        .map(|_| match mode {
            0 => sw.next_u32() as i16,
            1 => [i16::MIN, i16::MAX, 0, -1][(sw.next_u32() % 4) as usize],
            _ => (sw.next_u32() as i16) >> (sw.next_u32() % 12),
        })
        .collect()
}

fn rand_gain(sw: &mut Sweep) -> f32 {
    match sw.next_u32() % 6 {
        0 => 1.0,
        1 => 0.5,
        2 => f32_special(sw, -3.0, 3.0),
        3 => sw.f32_in(-1e6, 1e6),
        _ => sw.f32_in(0.0, 1.0),
    }
}

fn rand_size(sw: &mut Sweep) -> u8 {
    match sw.next_u32() % 4 {
        0 => sw.u8(),
        1 => 0,
        _ => 32,
    }
}

#[test]
fn osc_dispatch_blocks() {
    let mut fw = fw_or_skip!();
    fw.fpscr = 0;
    let st = hook_rng(&mut fw);
    let mut lxr = Box::new(Lxr::new_zeroed());
    let a = fw.alloc(40).unwrap();
    let modb = fw.alloc(512).unwrap();
    let buf = fw.alloc(512).unwrap();
    let mut sw = Sweep(0x05c1_11a7);
    let mut rs = OscInfo::ZERO;
    for it in 0..30000 {
        if it % 20 == 0 {
            let wf = match sw.next_u32() % 10 {
                n @ 0..=5 => n as u8,
                6 => 4,
                7 => 5,
                _ => sw.u8(),
            };
            rs = random_osc(&mut sw, wf);
            if sw.next_u32() % 2 == 0 {
                // realistic frequency state through osc_setFreq
                osc_setFreq(&mut rs);
                // tableOffset stays as computed for wavetables
            }
            w_osc(&mut fw, a, &rs);
            lxr.rng_state = st.get();
        }
        let size = rand_size(&mut sw);
        let gain = rand_gain(&mut sw);
        let pwm = match sw.next_u32() % 4 {
            0 => f32_special(&mut sw, -1.0, 2.0),
            _ => sw.f32_in(0.0, 1.0),
        };
        let m = rand_buf(&mut sw);
        let init = rand_buf(&mut sw);
        fw.write_i16s(modb, &m).unwrap();
        fw.write_i16s(buf, &init).unwrap();
        let mut rbuf = init.clone();
        let fm = it % 2 == 1;
        let name = if fm { "calcNextOscSampleFmBlock" } else { "calcNextOscSampleBlock" };
        if fm {
            fw.call(name, &[a, modb, buf, size as u32], &[gain, pwm]).unwrap();
            lxr.calcNextOscSampleFmBlock(&mut rs, &m, &mut rbuf, size, gain, pwm);
        } else {
            fw.call(name, &[a, buf, size as u32], &[gain, pwm]).unwrap();
            lxr.calcNextOscSampleBlock(&mut rs, &mut rbuf, size, gain, pwm);
        }
        let ctx = format!("{name} it {it} wf {} size {size} gain {gain:e} pwm {pwm:e}", rs.waveform);
        assert_eq!(fw.read_i16s(buf, 256).unwrap(), rbuf, "{ctx}");
        eq_osc(&ctx, &r_osc(&fw, a), &rs);
        assert_eq!(st.get(), lxr.rng_state, "{ctx} rng");
    }
}

#[test]
fn osc_direct_functions() {
    let mut fw = fw_or_skip!();
    fw.fpscr = 0;
    let st = hook_rng(&mut fw);
    let mut lxr = Box::new(Lxr::new_zeroed());
    let a = fw.alloc(40).unwrap();
    let modb = fw.alloc(512).unwrap();
    let buf = fw.alloc(512).unwrap();
    let tables = [
        (fw.addr("sawTable").unwrap(), &sawTable),
        (fw.addr("triTable").unwrap(), &triTable),
        (fw.addr("recTable").unwrap(), &recTable),
    ];
    let mut sw = Sweep(0xd1_2ec7);
    for it in 0..30000 {
        let wf = (sw.next_u32() % 6) as u8;
        let mut rs = random_osc(&mut sw, wf);
        if it % 7 == 0 {
            rs.tableOffset = 10; // exercises the read past the last row
        }
        w_osc(&mut fw, a, &rs);
        lxr.rng_state = st.get();
        let size = rand_size(&mut sw);
        let gain = rand_gain(&mut sw);
        let pwm = sw.f32_in(-0.2, 1.2);
        let m = rand_buf(&mut sw);
        let init = rand_buf(&mut sw);
        fw.write_i16s(modb, &m).unwrap();
        fw.write_i16s(buf, &init).unwrap();
        let mut rbuf = init.clone();
        let (taddr, table) = tables[(sw.next_u32() % 3) as usize];
        let name = match it % 7 {
            0 => {
                fw.call("calcSineBlock", &[a, buf, size as u32], &[gain]).unwrap();
                calcSineBlock(&mut rs, &mut rbuf, size, gain);
                "calcSineBlock"
            }
            1 => {
                fw.call("calcFmSineBlock", &[a, modb, buf, size as u32], &[gain]).unwrap();
                calcFmSineBlock(&mut rs, &m, &mut rbuf, size, gain);
                "calcFmSineBlock"
            }
            2 => {
                fw.call("calcFmBlock", &[a, taddr, modb, buf, size as u32], &[gain]).unwrap();
                calcFmBlock(&mut rs, table, &m, &mut rbuf, size, gain);
                "calcFmBlock"
            }
            3 => {
                fw.call("calcPwmFmBlock", &[a, modb, buf, size as u32], &[gain, pwm]).unwrap();
                calcPwmFmBlock(&mut rs, &m, &mut rbuf, size, gain, pwm);
                "calcPwmFmBlock"
            }
            4 => {
                fw.call("calcWavetableOscBlock", &[a, taddr, buf, size as u32], &[gain]).unwrap();
                calcWavetableOscBlock(&mut rs, table, &mut rbuf, size, gain);
                "calcWavetableOscBlock"
            }
            5 => {
                fw.call("calcPwmBlock", &[a, buf, size as u32], &[gain, pwm]).unwrap();
                calcPwmBlock(&mut rs, &mut rbuf, size, gain, pwm);
                "calcPwmBlock"
            }
            _ => {
                fw.call("calcNoiseBlock", &[a, buf, size as u32], &[gain]).unwrap();
                lxr.calcNoiseBlock(&mut rs, &mut rbuf, size, gain);
                "calcNoiseBlock"
            }
        };
        let ctx = format!("{name} it {it} size {size} gain {gain:e} pwm {pwm:e} off {}", rs.tableOffset);
        assert_eq!(fw.read_i16s(buf, 256).unwrap(), rbuf, "{ctx}");
        eq_osc(&ctx, &r_osc(&fw, a), &rs);
        assert_eq!(st.get(), lxr.rng_state, "{ctx} rng");
    }
}

#[test]
fn osc_frequency() {
    let mut fw = fw_or_skip!();
    fw.fpscr = 0;
    let a = fw.alloc(40).unwrap();
    let mut sw = Sweep(0xf2e9);
    for d in 0..=255u8 {
        let (_, f) = fw.call("midiParser_calcDetune", &[d as u32], &[]).unwrap();
        eq32(&format!("calcDetune {d}"), f, midiParser_calcDetune(d));
    }
    for it in 0..40000 {
        let wf = sw.u8() % 9;
        let mut rs = random_osc(&mut sw, wf);
        match sw.next_u32() % 5 {
            0 => rs.freq = f32_special(&mut sw, 0.0, 30000.0),
            1 => rs.freq = sw.f32_in(0.0, 440.0),
            2 => {
                rs.freq = sw.f32_in(0.0, 1e9);
                rs.pitchMod = sw.f32_in(0.0, 100.0);
            }
            3 => rs.modNodeValue = f32_special(&mut sw, -2.0, 2.0),
            _ => {}
        }
        w_osc(&mut fw, a, &rs);
        let name = match it % 3 {
            0 => {
                fw.call("osc_setFreq", &[a], &[]).unwrap();
                osc_setFreq(&mut rs);
                "osc_setFreq"
            }
            1 => {
                let n = sw.u8();
                fw.call("osc_setBaseNote", &[a, n as u32], &[]).unwrap();
                osc_setBaseNote(&mut rs, n);
                "osc_setBaseNote"
            }
            _ => {
                fw.call("osc_recalcFreq", &[a], &[]).unwrap();
                osc_recalcFreq(&mut rs);
                "osc_recalcFreq"
            }
        };
        eq_osc(&format!("{name} it {it}"), &r_osc(&fw, a), &rs);
    }
}

#[test]
fn cmsis_saturating_halfwords() {
    // QADD16/QSUB16 are inlined; check them against the hardware semantics on edge values.
    use lxr_core::cmsis::{__QADD16, __QSUB16};
    let vals = [0i16, 1, -1, 32767, -32768, 16384, -16384, 12345, -23456];
    for &a in &vals {
        for &b in &vals {
            for &c in &vals {
                let x = ((a as u16 as u32) << 16) | c as u16 as u32;
                let y = ((b as u16 as u32) << 16) | a as u16 as u32;
                let add = __QADD16(x, y);
                let sub = __QSUB16(x, y);
                assert_eq!((add >> 16) as i16, a.saturating_add(b));
                assert_eq!(add as i16, c.saturating_add(a));
                assert_eq!((sub >> 16) as i16, a.saturating_sub(b));
                assert_eq!(sub as i16, c.saturating_sub(a));
            }
        }
    }
}
