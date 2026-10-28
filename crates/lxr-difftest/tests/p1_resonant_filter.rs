//! P1 differential tests: ResonantFilter (SVF).
mod p1_common;

use lxr_core::dsp::resonant_filter::*;
use lxr_difftest::{fw_or_skip, Firmware, Sweep};
use p1_common::*;

fn w_svf(fw: &mut Firmware, a: u32, s: &ResonantFilter) {
    for (i, v) in [s.f, s.g, s.q, s.s1, s.s2, s.a, s.b, s.drive].iter().enumerate() {
        w_f32(fw, a + 4 * i as u32, *v);
    }
}
fn r_svf(fw: &Firmware, a: u32) -> ResonantFilter {
    let v: Vec<f32> = (0..8).map(|i| r_f32(fw, a + 4 * i)).collect();
    ResonantFilter { f: v[0], g: v[1], q: v[2], s1: v[3], s2: v[4], a: v[5], b: v[6], drive: v[7] }
}
fn eq_svf(ctx: &str, f: &ResonantFilter, r: &ResonantFilter) {
    let fv = [f.f, f.g, f.q, f.s1, f.s2, f.a, f.b, f.drive];
    let rv = [r.f, r.g, r.q, r.s1, r.s2, r.a, r.b, r.drive];
    let names = ["f", "g", "q", "s1", "s2", "a", "b", "drive"];
    for i in 0..8 {
        eq32(&format!("{ctx} {}", names[i]), fv[i], rv[i]);
    }
}

#[test]
fn svf_scalar_functions() {
    let mut fw = fw_or_skip!();
    fw.fpscr = 0;
    let mut sw = Sweep(0x5f1_7e2);
    for it in 0..20000 {
        let x = match it % 3 {
            0 => f32_special(&mut sw, -2.0, 2.0),
            1 => sw.f32_in(-100.0, 100.0),
            _ => f32::from_bits(sw.next_u32()),
        };
        for name in ["fastTan", "tanhXdX", "softClipTwo", "removeMaliciousSample"] {
            let (_, f) = fw.call(name, &[], &[x]).unwrap();
            let r = match name {
                "fastTan" => fastTan(x),
                "tanhXdX" => tanhXdX(x),
                "softClipTwo" => softClipTwo(x),
                _ => removeMaliciousSample(x),
            };
            eq32(&format!("{name}({x:e})"), f, r);
        }
    }
}

#[test]
fn svf_setters() {
    let mut fw = fw_or_skip!();
    fw.fpscr = 0;
    let a = fw.alloc(32).unwrap();
    let mut rs = ResonantFilter { f: 1.0, g: 2.0, q: 3.0, s1: 4.0, s2: 5.0, a: 6.0, b: 7.0, drive: 8.0 };
    w_svf(&mut fw, a, &rs);
    fw.call("SVF_init", &[a], &[]).unwrap();
    SVF_init(&mut rs);
    eq_svf("init", &r_svf(&fw, a), &rs);
    // Note: SVF_init's g (0x3ebce2f3, folded by GCC) differs by 1 ulp from what
    // SVF_directSetFilterValue(0.25) computes at run time (0x3ebce2f4); the port uses the literal.
    let mut chk = rs;
    SVF_directSetFilterValue(&mut chk, 0.25);
    assert_eq!(chk.f.to_bits(), rs.f.to_bits());
    assert_eq!((chk.g.to_bits(), rs.g.to_bits()), (0x3ebce2f4, 0x3ebce2f3));
    rs = ResonantFilter { f: 1.0, g: 2.0, q: 3.0, s1: 4.0, s2: 5.0, a: 6.0, b: 7.0, drive: 8.0 };
    w_svf(&mut fw, a, &rs);
    fw.call("SVF_reset", &[a], &[]).unwrap();
    SVF_reset(&mut rs);
    eq_svf("reset", &r_svf(&fw, a), &rs);
    let mut sw = Sweep(0xfeed_5eed);
    for it in 0..20000 {
        let v = match it % 4 {
            0 => (sw.u8() % 128) as f32 / 127.0,
            1 => f32_special(&mut sw, -1.0, 2.0),
            2 => sw.f32_in(0.0, 1.0),
            _ => f32::from_bits(sw.next_u32()),
        };
        let name = ["SVF_setReso", "SVF_setDrive", "SVF_directSetFilterValue", "SVF_recalcFreq"][it % 4];
        if name == "SVF_recalcFreq" {
            rs.f = v;
            w_f32(&mut fw, a, v);
        }
        fw.call(name, &[a], &[v]).unwrap();
        match it % 4 {
            0 => SVF_setReso(&mut rs, v),
            1 => SVF_setDrive(&mut rs, v),
            2 => SVF_directSetFilterValue(&mut rs, v),
            _ => SVF_recalcFreq(&mut rs),
        }
        eq_svf(&format!("{name}({v:e}) it {it}"), &r_svf(&fw, a), &rs);
    }
}

fn run_blocks(name: &str, seed: u64) {
    let mut fw = fw_or_skip!();
    fw.fpscr = 0;
    let a = fw.alloc(32).unwrap();
    let buf = fw.alloc(512).unwrap();
    let mut rs = ResonantFilter::ZERO;
    w_svf(&mut fw, a, &rs);
    fw.call("SVF_init", &[a], &[]).unwrap();
    SVF_init(&mut rs);
    let mut sw = Sweep(seed);
    let mut ty = 1u8;
    // signal generator state: mixture of noise, square and silence
    let mut phase = 0u32;
    for it in 0..12000 {
        if it % 25 == 0 {
            // parameter change via the setters (as the firmware does)
            let fval = match sw.next_u32() % 4 {
                0 => (sw.u8() % 128) as f32 / 127.0,
                1 => sw.f32_in(0.9, 1.0),
                _ => sw.f32_in(0.0, 1.0),
            };
            let reso = match sw.next_u32() % 3 {
                0 => (sw.u8() % 128) as f32 / 127.0,
                _ => sw.f32_in(0.0, 1.0),
            };
            let drive = sw.f32_in(0.0, 1.0);
            fw.call("SVF_directSetFilterValue", &[a], &[fval]).unwrap();
            fw.call("SVF_setReso", &[a], &[reso]).unwrap();
            fw.call("SVF_setDrive", &[a], &[drive]).unwrap();
            SVF_directSetFilterValue(&mut rs, fval);
            SVF_setReso(&mut rs, reso);
            SVF_setDrive(&mut rs, drive);
            ty = match sw.next_u32() % 12 {
                0 => 0,
                1 => 8,
                2 => 9 + sw.u8() % 100,
                n => [1, 2, 3, 4, 5, 6, 7, 1, 7][n as usize - 3],
            };
            if sw.next_u32() % 10 == 0 {
                fw.call("SVF_reset", &[a], &[]).unwrap();
                SVF_reset(&mut rs);
            }
        }
        if it % 600 == 17 {
            // raw states: extreme values, NaN, inf, denormals
            let pick = |sw: &mut Sweep| match sw.next_u32() % 6 {
                0 => f32::NAN,
                1 => f32::INFINITY,
                2 => f32::from_bits(sw.next_u32() & 0x807f_ffff),
                3 => sw.f32_in(-1e6, 1e6),
                _ => sw.f32_in(-2.0, 2.0),
            };
            rs.s1 = pick(&mut sw);
            rs.s2 = pick(&mut sw);
            rs.a = pick(&mut sw);
            rs.b = pick(&mut sw);
            if sw.next_u32() % 3 == 0 {
                rs.f = pick(&mut sw);
                rs.g = pick(&mut sw);
                rs.q = pick(&mut sw);
                rs.drive = sw.f32_in(0.0, 50.0);
            }
            w_svf(&mut fw, a, &rs);
        }
        let size = match sw.next_u32() % 5 {
            0 => sw.u8(),
            1 => 0,
            _ => 32,
        };
        let mode = sw.next_u32() % 4;
        let init: Vec<i16> = (0..256)
            .map(|_| {
                phase = phase.wrapping_add(0x0300_0000);
                match mode {
                    0 => sw.next_u32() as i16,
                    1 => if phase & 0x8000_0000 != 0 { 32767 } else { -32768 },
                    2 => 0,
                    _ => ((phase >> 16) as i16) / 4,
                }
            })
            .collect();
        fw.write_i16s(buf, &init).unwrap();
        let mut rbuf = init.clone();
        fw.call(name, &[a, ty as u32, buf, size as u32], &[]).unwrap();
        if name == "SVF_calcBlockZDF" {
            SVF_calcBlockZDF(&mut rs, ty, &mut rbuf, size);
        } else {
            SVF_calcBlockCheapZDF(&mut rs, ty, &mut rbuf, size);
        }
        assert_eq!(fw.read_i16s(buf, 256).unwrap(), rbuf, "{name} it {it} type {ty} size {size}");
        eq_svf(&format!("{name} it {it} type {ty}"), &r_svf(&fw, a), &rs);
    }
}

#[test]
fn svf_calc_block_zdf() {
    run_blocks("SVF_calcBlockZDF", 0x2df_0001);
}

#[test]
fn svf_calc_block_cheap_zdf() {
    run_blocks("SVF_calcBlockCheapZDF", 0xc4ea_9002);
}
