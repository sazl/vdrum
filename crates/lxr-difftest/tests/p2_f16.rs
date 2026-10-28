//! P2: `__fp16` conversions (`fpu::f16_bits_to_f32` / `fpu::f32_to_f16_bits`) against the emulated
//! `vcvtb.f32.f16` / `vcvtb.f16.f32` instructions (the instructions the FX code uses for `FX_BUFFER`),
//! with FPSCR = 0 as in the firmware's main loop (FZ = 0, DN = 0, AHP = 0, round to nearest).
mod p2_common;

use lxr_core::fpu::{f16_bits_to_f32, f32_to_f16_bits};
use lxr_difftest::Sweep;

/// Thumb-2 snippets placed in scratch memory:
/// to_half:   vcvtb.f16.f32 s0, s0 ; vmov r0, s0 ; bx lr
/// to_single: vmov s0, r0 ; vcvtb.f32.f16 s0, s0 ; bx lr
const CODE: [u16; 10] = [
    0xeeb3, 0x0a40, 0xee10, 0x0a10, 0x4770, // to_half
    0xee00, 0x0a10, 0xeeb2, 0x0a40, 0x4770, // to_single
];

fn load(fw: &mut lxr_emu::Firmware) -> (u32, u32) {
    let a = fw.alloc(64).unwrap();
    let b: Vec<u8> = CODE.iter().flat_map(|x| x.to_le_bytes()).collect();
    fw.write(a, &b).unwrap();
    (a, a + 10)
}

#[test]
fn f16_to_f32_exhaustive() {
    let Some(mut fw) = p2_common::setup() else { return eprintln!("LXR_FW_ELF not set; skipping") };
    let (_, to_single) = load(&mut fw);
    for h in 0..=0xffffu32 {
        let (_, s0) = fw.call_addr(to_single, &[h], &[]).unwrap();
        let r = f16_bits_to_f32(h as u16);
        assert_eq!(s0.to_bits(), r.to_bits(), "half {h:#06x}");
    }
}

#[test]
fn f32_to_f16_sweep() {
    let Some(mut fw) = p2_common::setup() else { return eprintln!("LXR_FW_ELF not set; skipping") };
    let (to_half, _) = load(&mut fw);
    let mut inputs: Vec<u32> = Vec::new();
    // every half value, its neighbours in f32 and the exact midpoints between adjacent halves
    for h in 0..=0xffffu32 {
        let f = f16_bits_to_f32(h as u16).to_bits();
        inputs.extend([f, f.wrapping_add(1), f.wrapping_sub(1)]);
        if h & 0x7fff < 0x7c00 {
            let g = f16_bits_to_f32((h + 1) as u16);
            if g.is_finite() {
                let mid = ((f32::from_bits(f) as f64 + g as f64) / 2.0) as f32;
                inputs.extend([mid.to_bits(), mid.to_bits() + 1, mid.to_bits() - 1]);
            }
        }
    }
    // f32 specials: denormals, NaN payloads, overflow boundary, tiny values
    inputs.extend([
        0x0000_0001, 0x8000_0001, 0x007f_ffff, 0x807f_ffff, 0x0080_0000, 0x7f80_0001, 0xff80_0001, 0x7fc0_0000,
        0x7fbf_ffff, 0x7f80_2000, 0x7fff_ffff, 0x477f_efff, 0x477f_f000, 0x477f_f001, 0x4780_0000, 0x7f7f_ffff,
        0x3300_0000, 0x3300_0001, 0x32ff_ffff, 0x3380_0000, 0x33c0_0000, 0x387f_e000, 0x387f_f000,
    ]);
    let mut sw = Sweep(0x1234_5678_9abc_def1);
    for _ in 0..200_000 {
        inputs.push(sw.next_u32() ^ (sw.next_u32() << 16));
    }
    for (n, &x) in inputs.iter().enumerate() {
        let (r0, _) = fw.call_addr(to_half, &[], &[f32::from_bits(x)]).unwrap();
        let r = f32_to_f16_bits(f32::from_bits(x));
        assert_eq!(r0 & 0xffff, r as u32, "input {x:#010x} (#{n})");
    }
    eprintln!("f32_to_f16: {} inputs", inputs.len());
}
