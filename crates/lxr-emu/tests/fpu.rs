//! Checks of the emulated FPU that the bit-exactness of the reference depends on:
//! flush-to-zero follows FPSCR.FZ, VFMA is fused, double precision is available
//! (Cortex-M7 model, not the M33 Unicorn uses with MCLASS mode).
//! These tests do not need firmware files.
use lxr_emu::firmware::{Firmware, FPSCR_FZ};
use std::collections::HashMap;

fn fw_with(code: &[u16]) -> (Firmware, u32) {
    let mut fw = Firmware::from_bin(&[0u8; 16], HashMap::new()).unwrap();
    let a = fw.alloc(code.len() * 2).unwrap();
    let bytes: Vec<u8> = code.iter().flat_map(|h| h.to_le_bytes()).collect();
    fw.write(a, &bytes).unwrap();
    (fw, a)
}

const VMUL_S0_S0_S1: [u16; 3] = [0xEE20, 0x0A20, 0x4770];
const VFMA_S0_S1_S2: [u16; 3] = [0xEEA0, 0x0A81, 0x4770];
const VMUL_VADD: [u16; 5] = [0xEE20, 0x0A20, 0xEE30, 0x0A01, 0x4770];
const VMRS_R0_FPSCR: [u16; 3] = [0xEEF1, 0x0A10, 0x4770];
const VMUL_F64: [u16; 3] = [0xEE20, 0x0B01, 0x4770];

#[test]
fn flush_to_zero_follows_fpscr_fz() {
    let (mut fw, a) = fw_with(&VMUL_S0_S0_S1);
    // 1e-30 * 1e-10 = 1e-40: a subnormal result
    let x = 1e-30f32;
    let y = 1e-10f32;
    let expect = x * y;
    assert!(expect != 0.0 && !expect.is_normal());

    fw.fpscr = 0;
    let (_, r) = fw.call_addr(a, &[], &[x, y]).unwrap();
    assert_eq!(
        r.to_bits(),
        expect.to_bits(),
        "FZ=0 must keep subnormal results"
    );

    fw.fpscr = FPSCR_FZ;
    let (_, r) = fw.call_addr(a, &[], &[x, y]).unwrap();
    assert_eq!(r.to_bits(), 0, "FZ=1 must flush subnormal results to +0");

    // subnormal input: FZ=1 treats it as zero, FZ=0 uses it
    let sub = f32::from_bits(0x0000_1234);
    fw.fpscr = 0;
    let (_, r) = fw.call_addr(a, &[], &[sub, 2.0]).unwrap();
    assert_eq!(r.to_bits(), (sub * 2.0).to_bits());
    fw.fpscr = FPSCR_FZ;
    let (_, r) = fw.call_addr(a, &[], &[sub, 2.0]).unwrap();
    assert_eq!(r.to_bits(), 0);
    let (_, r) = fw.call_addr(a, &[], &[-sub, 2.0]).unwrap();
    assert_eq!(
        r.to_bits(),
        0x8000_0000,
        "flushed negative subnormal keeps its sign"
    );
}

#[test]
fn fpscr_is_set_per_call() {
    let (mut fw, a) = fw_with(&VMRS_R0_FPSCR);
    fw.fpscr = FPSCR_FZ;
    assert_eq!(fw.call_addr(a, &[], &[]).unwrap().0 & 0x07C0_0000, FPSCR_FZ);
    fw.fpscr = 0;
    assert_eq!(fw.call_addr(a, &[], &[]).unwrap().0 & 0x07C0_0000, 0);
}

#[test]
fn vfma_is_fused() {
    // s0 + s1*s2 with s1 = s2 = 1 + 2^-12: the exact product 1 + 2^-11 + 2^-24 is not
    // representable; fused: s0 = -(1 + 2^-11) gives exactly 2^-24.
    let s1 = f32::from_bits(0x3F80_0800); // 1 + 2^-12
    let s0 = -f32::from_bits(0x3F80_1000); // -(1 + 2^-11)
    let fused = s1.mul_add(s1, s0);
    assert_eq!(fused, 2f32.powi(-24));
    let unfused = s1 * s1 + s0;
    assert_ne!(fused.to_bits(), unfused.to_bits());

    let (mut fw, a) = fw_with(&VFMA_S0_S1_S2);
    fw.fpscr = FPSCR_FZ;
    let (_, r) = fw.call_addr(a, &[], &[s0, s1, s1]).unwrap();
    assert_eq!(r.to_bits(), fused.to_bits(), "VFMA must round once");

    // separate VMUL + VADD rounds twice
    let (mut fw, a) = fw_with(&VMUL_VADD);
    let (_, r) = fw.call_addr(a, &[], &[s1, s1, s0]).unwrap();
    assert_eq!(r.to_bits(), unfused.to_bits());
}

#[test]
fn double_precision_available() {
    let (mut fw, a) = fw_with(&VMUL_F64);
    // d0 = s0:s1, d1 = s2:s3
    let x = 1.0f64 / 3.0;
    let y = 3.0f64;
    let xb = x.to_bits();
    let yb = y.to_bits();
    let f = |b: u64| f32::from_bits(b as u32);
    let g = |b: u64| f32::from_bits((b >> 32) as u32);
    fw.call_addr(a, &[], &[f(xb), g(xb), f(yb), g(yb)]).unwrap();
    let lo = fw.uc.reg_read(unicorn_engine::RegisterARM::S0).unwrap();
    let hi = fw.uc.reg_read(unicorn_engine::RegisterARM::S1).unwrap();
    assert_eq!(((hi << 32) | lo), (x * y).to_bits());
}
