//! Shared helpers for the P1 (DSP primitives) differential tests.
#![allow(dead_code)]

use lxr_difftest::{Firmware, Sweep};

pub fn w_f32(fw: &mut Firmware, a: u32, v: f32) {
    fw.write_f32(a, v).unwrap();
}
pub fn w_f64(fw: &mut Firmware, a: u32, v: f64) {
    fw.write(a, &v.to_le_bytes()).unwrap();
}
pub fn w_u8(fw: &mut Firmware, a: u32, v: u8) {
    fw.write_u8(a, v).unwrap();
}
pub fn w_u16(fw: &mut Firmware, a: u32, v: u16) {
    fw.write_u16(a, v).unwrap();
}
pub fn w_u32(fw: &mut Firmware, a: u32, v: u32) {
    fw.write_u32(a, v).unwrap();
}
pub fn r_f32(fw: &Firmware, a: u32) -> f32 {
    fw.read_f32(a).unwrap()
}
pub fn r_f64(fw: &Firmware, a: u32) -> f64 {
    fw.read_f64(a).unwrap()
}
pub fn r_u8(fw: &Firmware, a: u32) -> u8 {
    fw.read_u8(a).unwrap()
}
pub fn r_u16(fw: &Firmware, a: u32) -> u16 {
    fw.read_u16(a).unwrap()
}
pub fn r_u32(fw: &Firmware, a: u32) -> u32 {
    fw.read_u32(a).unwrap()
}

/// Bit-exact float comparison with a readable message.
#[track_caller]
pub fn eq32(ctx: &str, fw: f32, rs: f32) {
    // NaN payload/sign: the Cortex-M7 (DN=0) generates the default NaN 0x7fc00000 for invalid operations,
    // x86 SSE generates 0xffc00000. The engine never inspects NaN bits (vcvt to integer gives 0 on both),
    // so any NaN matches any NaN.
    if fw.is_nan() && rs.is_nan() {
        return;
    }
    assert_eq!(fw.to_bits(), rs.to_bits(), "{ctx}: firmware {fw:e} ({:#010x}) vs rust {rs:e} ({:#010x})", fw.to_bits(), rs.to_bits());
}
#[track_caller]
pub fn eq64(ctx: &str, fw: f64, rs: f64) {
    if fw.is_nan() && rs.is_nan() {
        return;
    }
    assert_eq!(fw.to_bits(), rs.to_bits(), "{ctx}: firmware {fw:e} ({:#018x}) vs rust {rs:e} ({:#018x})", fw.to_bits(), rs.to_bits());
}

/// Return value of a function returning `double` (d0 = s0:s1).
pub fn ret_f64(fw: &Firmware) -> f64 {
    let lo = fw.uc.reg_read(unicorn_engine::RegisterARM::S0).unwrap() as u64;
    let hi = fw.uc.reg_read(unicorn_engine::RegisterARM::S1).unwrap() as u64;
    f64::from_bits((hi << 32) | lo)
}

/// Call with double arguments in d0..d7 (AAPCS-VFP), plus integer args.
pub fn call_d(fw: &mut Firmware, name: &str, ints: &[u32], doubles: &[f64]) -> f64 {
    let mut fl = Vec::new();
    for d in doubles {
        let b = d.to_bits();
        fl.push(f32::from_bits(b as u32));
        fl.push(f32::from_bits((b >> 32) as u32));
    }
    fw.call(name, ints, &fl).unwrap();
    ret_f64(fw)
}

/// Interesting f32 values: random in range, plus a few extremes and denormal-ish values.
pub fn f32_special(sw: &mut Sweep, lo: f32, hi: f32) -> f32 {
    match sw.next_u32() % 16 {
        0 => 0.0,
        1 => -0.0,
        2 => f32::from_bits(sw.next_u32() & 0x807f_ffff), // denormal
        3 => f32::from_bits((sw.next_u32() & 0x80ff_ffff) | 0x0080_0000), // tiny normal
        4 => lo,
        5 => hi,
        _ => sw.f32_in(lo, hi),
    }
}

pub fn f64_in(sw: &mut Sweep, lo: f64, hi: f64) -> f64 {
    let a = sw.next_u32() as u64;
    let b = sw.next_u32() as u64;
    let u = ((a << 32) | b) as f64 / u64::MAX as f64;
    lo + (hi - lo) * u
}

pub fn f64_special(sw: &mut Sweep, lo: f64, hi: f64) -> f64 {
    match sw.next_u32() % 16 {
        0 => 0.0,
        1 => -0.0,
        2 => f64::from_bits(((sw.next_u32() as u64) << 20) & 0x800f_ffff_ffff_ffff), // denormal
        3 => f32::from_bits(sw.next_u32() & 0x807f_ffff) as f64,                        // f32 denormal range
        4 => lo,
        5 => hi,
        _ => f64_in(sw, lo, hi),
    }
}

/// Full u8 range with extra weight on the ends.
pub fn u8_special(sw: &mut Sweep) -> u8 {
    match sw.next_u32() % 8 {
        0 => 0,
        1 => 127,
        2 => 255,
        3 => 128,
        _ => sw.u8(),
    }
}
