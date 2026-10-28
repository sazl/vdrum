//! P1 differential tests: 1PoleLp, Decay, snapEg, SlopeEg2, transientGenerator, distortion.
mod p1_common;

use lxr_core::dsp::decay::*;
use lxr_core::dsp::distortion::*;
use lxr_core::dsp::one_pole_lp::*;
use lxr_core::dsp::slope_eg2::*;
use lxr_core::dsp::snap_eg::*;
use lxr_core::dsp::transient_generator::*;
use lxr_difftest::{fw_or_skip, Firmware, Sweep};
use p1_common::*;

// ---------------------------------------------------------------- 1PoleLp
fn w_onepole(fw: &mut Firmware, a: u32, f: &OnePoleFilter) {
    w_f64(fw, a, f.r);
    w_f64(fw, a + 8, f.b);
}
fn r_onepole(fw: &Firmware, a: u32) -> OnePoleFilter {
    OnePoleFilter { r: r_f64(fw, a), b: r_f64(fw, a + 8) }
}
fn eq_onepole(ctx: &str, f: &OnePoleFilter, r: &OnePoleFilter) {
    eq64(&format!("{ctx} r"), f.r, r.r);
    eq64(&format!("{ctx} b"), f.b, r.b);
}

#[test]
fn one_pole() {
    let mut fw = fw_or_skip!();
    fw.fpscr = 0;
    let a = fw.alloc(16).unwrap();
    // init / setCoef
    let mut rs = OnePoleFilter { r: 5.0, b: 7.0 };
    w_onepole(&mut fw, a, &rs);
    fw.call("initOnePole", &[a], &[]).unwrap();
    initOnePole(&mut rs);
    eq_onepole("init", &r_onepole(&fw, a), &rs);

    let mut sw = Sweep(0x1234_5678_9abc);
    for it in 0..20000 {
        if it % 500 == 0 {
            let r = match sw.next_u32() % 4 {
                0 => f64_special(&mut sw, 0.0, 1.0),
                1 => f64_in(&mut sw, 0.0, 0.01),
                _ => f64_in(&mut sw, -2.0, 2.0),
            };
            call_d(&mut fw, "setOnePoleCoef", &[a], &[r]);
            setOnePoleCoef(&mut rs, r);
            if it % 1500 == 0 {
                let b = f64_special(&mut sw, -40000.0, 40000.0);
                w_f64(&mut fw, a + 8, b);
                rs.b = b;
            }
        }
        let input = match sw.next_u32() % 3 {
            0 => f64_special(&mut sw, -1.0, 1.0),
            1 => f64_in(&mut sw, -40000.0, 40000.0),
            _ => (sw.next_u32() as i16) as f64,
        };
        let out_fw = call_d(&mut fw, "calcOnePole", &[a], &[input]);
        // calcOnePole returns float in s0
        let out_fw = f32::from_bits(out_fw.to_bits() as u32);
        let out_rs = {
            calcOnePole(&mut rs, input)
        };
        eq32(&format!("calcOnePole it {it}"), out_fw, out_rs);
        eq_onepole(&format!("calcOnePole it {it}"), &r_onepole(&fw, a), &rs);
    }
}

// ---------------------------------------------------------------- Decay
fn w_decay(fw: &mut Firmware, a: u32, e: &DecayEg) {
    w_f32(fw, a, e.decay);
    w_f32(fw, a + 4, e.slope);
    w_f32(fw, a + 8, e.value);
}
fn r_decay(fw: &Firmware, a: u32) -> DecayEg {
    DecayEg { decay: r_f32(fw, a), slope: r_f32(fw, a + 4), value: r_f32(fw, a + 8) }
}
fn eq_decay(ctx: &str, f: &DecayEg, r: &DecayEg) {
    eq32(&format!("{ctx} decay"), f.decay, r.decay);
    eq32(&format!("{ctx} slope"), f.slope, r.slope);
    eq32(&format!("{ctx} value"), f.value, r.value);
}

#[test]
fn decay_eg() {
    let mut fw = fw_or_skip!();
    let a = fw.alloc(12).unwrap();
    // setters for every u8 (main loop, FZ=0)
    fw.fpscr = 0;
    for d in 0..=255u8 {
        let mut rs = DecayEg { decay: 3.0, slope: 4.0, value: 5.0 };
        w_decay(&mut fw, a, &rs);
        fw.call("DecayEg_setDecay", &[a, d as u32], &[]).unwrap();
        fw.call("DecayEg_setSlope", &[a, d as u32], &[]).unwrap();
        DecayEg_setDecay(&mut rs, d);
        DecayEg_setSlope(&mut rs, d);
        eq_decay(&format!("set {d}"), &r_decay(&fw, a), &rs);
    }
    let mut rs = DecayEg::ZERO;
    w_decay(&mut fw, a, &DecayEg { decay: 9.0, slope: 9.0, value: 9.0 });
    rs.decay = 9.0;
    rs.slope = 9.0;
    rs.value = 9.0;
    fw.call("DecayEg_init", &[a], &[]).unwrap();
    DecayEg_init(&mut rs);
    eq_decay("init", &r_decay(&fw, a), &rs);

    fw.fpscr = 0;
    let mut sw = Sweep(0xdeca_1234_5555);
    for it in 0..30000 {
        if it % 700 == 0 {
            fw.call("DecayEg_trigger", &[a], &[]).unwrap();
            DecayEg_trigger(&mut rs);
            let d = u8_special(&mut sw);
            let s = u8_special(&mut sw);
            fw.fpscr = 0;
            fw.call("DecayEg_setDecay", &[a, d as u32], &[]).unwrap();
            fw.call("DecayEg_setSlope", &[a, s as u32], &[]).unwrap();
            fw.fpscr = 0;
            DecayEg_setDecay(&mut rs, d);
            DecayEg_setSlope(&mut rs, s);
        }
        if it % 3000 == 1 {
            // arbitrary state, incl. tiny decays that produce denormals
            let st = DecayEg {
                decay: f32_special(&mut sw, 0.0, 0.1),
                slope: f32_special(&mut sw, -0.99, 300.0),
                value: f32_special(&mut sw, -1.0, 1.0),
            };
            w_decay(&mut fw, a, &st);
            rs = st;
        }
        let (_, out_fw) = fw.call("DecayEg_calc", &[a], &[]).unwrap();
        let out_rs = {
            DecayEg_calc(&mut rs)
        };
        eq32(&format!("DecayEg_calc it {it}"), out_fw, out_rs);
        eq_decay(&format!("DecayEg_calc it {it}"), &r_decay(&fw, a), &rs);
    }
}

// ---------------------------------------------------------------- snapEg
#[test]
fn snap_eg() {
    let mut fw = fw_or_skip!();
    fw.fpscr = 0;
    let a = fw.alloc(4).unwrap();
    let mut rs = SnapEg { value: 3.0 };
    w_f32(&mut fw, a, 3.0);
    fw.call("SnapEg_init", &[a], &[]).unwrap();
    SnapEg_init(&mut rs);
    eq32("init", r_f32(&fw, a), rs.value);
    let mut sw = Sweep(0x5a5a_1111);
    let mut time = 0.01f32;
    for it in 0..30000 {
        if it % 200 == 0 {
            fw.call("SnapEg_trigger", &[a], &[]).unwrap();
            SnapEg_trigger(&mut rs);
            time = f32_special(&mut sw, 0.0, 1.0);
        }
        if it % 1000 == 7 {
            let v = f32_special(&mut sw, -1.0, 2.0);
            w_f32(&mut fw, a, v);
            rs.value = v;
        }
        let (_, out_fw) = fw.call("SnapEg_calc", &[a], &[time]).unwrap();
        let out_rs = {
            SnapEg_calc(&mut rs, time)
        };
        eq32(&format!("SnapEg_calc it {it}"), out_fw, out_rs);
        eq32(&format!("SnapEg value it {it}"), r_f32(&fw, a), rs.value);
    }
}

// ---------------------------------------------------------------- SlopeEg2
fn w_slope(fw: &mut Firmware, a: u32, e: &SlopeEg2) {
    w_f64(fw, a, e.attack);
    w_f64(fw, a + 8, e.decay);
    w_f64(fw, a + 16, e.slope);
    w_f64(fw, a + 24, e.invSlope);
    w_u8(fw, a + 32, e.repeat);
    w_u8(fw, a + 33, e.repeatCnt);
    w_f64(fw, a + 40, e.value);
    w_u8(fw, a + 48, e.state);
}
fn r_slope(fw: &Firmware, a: u32) -> SlopeEg2 {
    SlopeEg2 {
        attack: r_f64(fw, a),
        decay: r_f64(fw, a + 8),
        slope: r_f64(fw, a + 16),
        invSlope: r_f64(fw, a + 24),
        repeat: r_u8(fw, a + 32),
        repeatCnt: r_u8(fw, a + 33),
        value: r_f64(fw, a + 40),
        state: r_u8(fw, a + 48),
    }
}
fn eq_slope(ctx: &str, f: &SlopeEg2, r: &SlopeEg2) {
    eq64(&format!("{ctx} attack"), f.attack, r.attack);
    eq64(&format!("{ctx} decay"), f.decay, r.decay);
    eq64(&format!("{ctx} slope"), f.slope, r.slope);
    eq64(&format!("{ctx} invSlope"), f.invSlope, r.invSlope);
    eq64(&format!("{ctx} value"), f.value, r.value);
    assert_eq!((f.repeat, f.repeatCnt, f.state), (r.repeat, r.repeatCnt, r.state), "{ctx} ints");
}

#[test]
fn slope_eg2_setters() {
    let mut fw = fw_or_skip!();
    fw.fpscr = 0;
    let a = fw.alloc(56).unwrap();
    let base = SlopeEg2 { attack: 1.5, decay: 2.5, slope: 3.5, invSlope: 4.5, repeat: 5, repeatCnt: 6, value: 7.5, state: 2 };
    let mut rs = base;
    w_slope(&mut fw, a, &rs);
    fw.call("slopeEg2_init", &[a], &[]).unwrap();
    slopeEg2_init(&mut rs);
    eq_slope("init", &r_slope(&fw, a), &rs);
    for d in 0..=255u8 {
        for sync in [0u8, 1, 200] {
            let mut rs = base;
            w_slope(&mut fw, a, &rs);
            fw.call("slopeEg2_setAttack", &[a, d as u32, sync as u32], &[]).unwrap();
            fw.call("slopeEg2_setDecay", &[a, (255 - d) as u32, sync as u32], &[]).unwrap();
            fw.call("slopeEg2_setSlope", &[a, d as u32], &[]).unwrap();
            slopeEg2_setAttack(&mut rs, d, sync);
            slopeEg2_setDecay(&mut rs, 255 - d, sync);
            slopeEg2_setSlope(&mut rs, d);
            eq_slope(&format!("set {d} {sync}"), &r_slope(&fw, a), &rs);
        }
        let f = call_d(&mut fw, "slopeEg2_calcDecay", &[d as u32], &[]);
        eq64(&format!("calcDecay {d}"), f, slopeEg2_calcDecay(d));
    }
    // calcSlopeValue is out of line in ITCM as well
    let mut sw = Sweep(77);
    for _ in 0..5000 {
        let v = f64_special(&mut sw, -1.5, 1.5);
        let s = f64_special(&mut sw, -1.0, 4000.0);
        let f = call_d(&mut fw, "slopeEg2_calcSlopeValue", &[], &[v, s]);
        eq64(&format!("calcSlopeValue {v} {s}"), f, slopeEg2_calcSlopeValue(v, s));
    }
}

#[test]
fn slope_eg2_run() {
    let mut fw = fw_or_skip!();
    fw.fpscr = 0;
    let a = fw.alloc(56).unwrap();
    let mut rs = SlopeEg2::ZERO;
    w_slope(&mut fw, a, &rs);
    fw.call("slopeEg2_init", &[a], &[]).unwrap();
    slopeEg2_init(&mut rs);
    let mut sw = Sweep(0x51_0e_e6_02);
    for it in 0..60000 {
        if it % 900 == 0 {
            let (at, de, sl) = (u8_special(&mut sw), u8_special(&mut sw), u8_special(&mut sw));
            let sync = (sw.next_u32() % 3 == 0) as u32;
            let rep = if sw.next_u32() % 2 == 0 { 0 } else { sw.u8() % 8 };
            fw.call("slopeEg2_setAttack", &[a, at as u32, sync], &[]).unwrap();
            fw.call("slopeEg2_setDecay", &[a, de as u32, sync], &[]).unwrap();
            fw.call("slopeEg2_setSlope", &[a, sl as u32], &[]).unwrap();
            w_u8(&mut fw, a + 32, rep);
            fw.call("slopeEg2_trigger", &[a], &[]).unwrap();
            slopeEg2_setAttack(&mut rs, at, sync as u8);
            slopeEg2_setDecay(&mut rs, de, sync as u8);
            slopeEg2_setSlope(&mut rs, sl);
            rs.repeat = rep;
            slopeEg2_trigger(&mut rs);
        }
        if it % 4000 == 13 {
            // arbitrary states incl. denormal-producing values and unknown state numbers
            let st = SlopeEg2 {
                attack: f64_special(&mut sw, 0.0, 1.0),
                decay: f64_special(&mut sw, 0.0, 1e-3),
                slope: f64_special(&mut sw, -0.9995, 4000.0),
                invSlope: f64_special(&mut sw, -0.9995, 4000.0),
                repeat: sw.u8() % 4,
                repeatCnt: sw.u8() % 4,
                value: f64_special(&mut sw, -0.5, 1.5),
                state: (sw.next_u32() % 5) as u8,
            };
            w_slope(&mut fw, a, &st);
            rs = st;
        }
        let f = call_d(&mut fw, "slopeEg2_calc", &[a], &[]);
        let r = {
            slopeEg2_calc(&mut rs)
        };
        eq64(&format!("slopeEg2_calc it {it}"), f, r);
        eq_slope(&format!("slopeEg2_calc it {it}"), &r_slope(&fw, a), &rs);
    }
}

// ---------------------------------------------------------------- transientGenerator
fn w_tr(fw: &mut Firmware, a: u32, t: &TransientGenerator) {
    w_u16(fw, a, t.output as u16);
    w_u32(fw, a + 4, t.phase);
    w_f32(fw, a + 8, t.pitch);
    w_u8(fw, a + 12, t.waveform);
    w_f32(fw, a + 16, t.volume);
    w_u16(fw, a + 20, t.midiFreq);
}
fn r_tr(fw: &Firmware, a: u32) -> TransientGenerator {
    TransientGenerator {
        output: r_u16(fw, a) as i16,
        phase: r_u32(fw, a + 4),
        pitch: r_f32(fw, a + 8),
        waveform: r_u8(fw, a + 12),
        volume: r_f32(fw, a + 16),
        midiFreq: r_u16(fw, a + 20),
    }
}
fn eq_tr(ctx: &str, f: &TransientGenerator, r: &TransientGenerator) {
    assert_eq!((f.output, f.phase, f.waveform, f.midiFreq), (r.output, r.phase, r.waveform, r.midiFreq), "{ctx}");
    eq32(&format!("{ctx} pitch"), f.pitch, r.pitch);
    eq32(&format!("{ctx} volume"), f.volume, r.volume);
}

#[test]
fn transient() {
    let mut fw = fw_or_skip!();
    fw.fpscr = 0;
    let a = fw.alloc(24).unwrap();
    let buf = fw.alloc(512).unwrap();
    let mut rs = TransientGenerator { output: 5, phase: 6, pitch: 7.0, waveform: 8, volume: 9.0, midiFreq: 10 };
    w_tr(&mut fw, a, &rs);
    fw.call("transient_init", &[a], &[]).unwrap();
    transient_init(&mut rs);
    eq_tr("init", &r_tr(&fw, a), &rs);
    for w in 0..=255u8 {
        fw.call("transient_setWaveform", &[a, w as u32], &[]).unwrap();
        transient_setWaveform(&mut rs, w);
        eq_tr(&format!("setWaveform {w}"), &r_tr(&fw, a), &rs);
    }
    let mut sw = Sweep(0x7a_a5_17);
    for it in 0..6000 {
        if it % 40 == 0 {
            fw.call("transient_trigger", &[a], &[]).unwrap();
            transient_trigger(&mut rs);
            let w = (sw.next_u32() % 14) as u8;
            fw.call("transient_setWaveform", &[a, w as u32], &[]).unwrap();
            transient_setWaveform(&mut rs, w);
            rs.pitch = match sw.next_u32() % 6 {
                0 => sw.f32_in(0.0, 40.0),
                1 => f32_special(&mut sw, -2.0, 2.0),
                _ => sw.f32_in(0.25, 4.0),
            };
            rs.volume = f32_special(&mut sw, -2.0, 2.0);
            if sw.next_u32() % 8 == 0 {
                rs.phase = sw.next_u32();
            }
            w_tr(&mut fw, a, &rs);
        }
        let size = match sw.next_u32() % 4 {
            0 => sw.u8(),
            1 => 0,
            _ => 32,
        };
        let init: Vec<i16> = (0..256).map(|_| sw.next_u32() as i16).collect();
        fw.write_i16s(buf, &init).unwrap();
        let mut rbuf = init.clone();
        fw.call("transient_calcBlock", &[a, buf, size as u32], &[]).unwrap();
        {
            transient_calcBlock(&mut rs, &mut rbuf, size);
        }
        assert_eq!(fw.read_i16s(buf, 256).unwrap(), rbuf, "transient_calcBlock it {it} size {size}");
        eq_tr(&format!("transient_calcBlock it {it}"), &r_tr(&fw, a), &rs);
    }
}

// ---------------------------------------------------------------- distortion
fn w_dist(fw: &mut Firmware, a: u32, d: &Distortion) {
    w_f32(fw, a, d.shape);
    w_f32(fw, a + 4, d.inv_shape);
    w_onepole(fw, a + 8, &d.DriveParamFilter);
}
fn r_dist(fw: &Firmware, a: u32) -> Distortion {
    Distortion { shape: r_f32(fw, a), inv_shape: r_f32(fw, a + 4), DriveParamFilter: r_onepole(fw, a + 8) }
}
fn eq_dist(ctx: &str, f: &Distortion, r: &Distortion) {
    eq32(&format!("{ctx} shape"), f.shape, r.shape);
    eq32(&format!("{ctx} inv_shape"), f.inv_shape, r.inv_shape);
    eq_onepole(ctx, &f.DriveParamFilter, &r.DriveParamFilter);
}

#[test]
fn distortion() {
    let mut fw = fw_or_skip!();
    fw.fpscr = 0;
    let a = fw.alloc(24).unwrap();
    let buf = fw.alloc(512).unwrap();
    let mut rs = Distortion::ZERO;
    w_dist(&mut fw, a, &rs);
    let mut sw = Sweep(0xd157);
    for it in 0..4000 {
        if it % 10 == 0 {
            let shape = match sw.next_u32() % 4 {
                0 => (sw.u8() % 128) as f32 / 127.0,
                1 => f32_special(&mut sw, -1.0, 1.0),
                _ => sw.f32_in(0.0, 1.0),
            };
            fw.call("setDistortionShape", &[a], &[shape]).unwrap();
            {
                setDistortionShape(&mut rs, shape);
            }
            eq_dist(&format!("setShape {shape}"), &r_dist(&fw, a), &rs);
            if it % 50 == 0 {
                rs.DriveParamFilter = OnePoleFilter { r: f64_in(&mut sw, 0.0, 1.0), b: f64_special(&mut sw, -1.0, 300.0) };
                w_dist(&mut fw, a, &rs);
            }
        }
        let size = match sw.next_u32() % 4 {
            0 => sw.u8(),
            _ => 32,
        };
        let init: Vec<i16> = (0..256)
            .map(|_| match sw.next_u32() % 8 {
                0 => i16::MIN,
                1 => i16::MAX,
                _ => sw.next_u32() as i16,
            })
            .collect();
        let name = if it % 2 == 0 { "calcDistBlock" } else { "calcDistBlockFiltered" };
        fw.write_i16s(buf, &init).unwrap();
        let mut rbuf = init.clone();
        fw.call(name, &[a, buf, size as u32], &[]).unwrap();
        {
            if it % 2 == 0 {
                calcDistBlock(&rs, &mut rbuf, size);
            } else {
                calcDistBlockFiltered(&mut rs, &mut rbuf, size);
            }
        }
        assert_eq!(fw.read_i16s(buf, 256).unwrap(), rbuf, "{name} it {it} size {size}");
        eq_dist(&format!("{name} it {it}"), &r_dist(&fw, a), &rs);
    }
}
