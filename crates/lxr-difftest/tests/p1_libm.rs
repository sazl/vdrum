//! Differential tests of `lxr_core::fpu::libm` against the libm functions of the firmware, bit for bit.
//!
//! Runs against every build that is available:
//! * the shipped v1.71 binary (`LXR_FIRMWARE_IMG` = `LXRV2_update_v1.71.img`, symbol map
//!   `lxr_emu::symbols::V171_SYMBOLS_JSON`: `exp` 0x08029648, `expf` 0x08029888, `logf` 0x08029970,
//!   `fmod` 0x08029600, `__ieee754_fmod` 0x08029ad0),
//! * the GCC 11.3 reference ELF (`LXR_FW_ELF`).
//!
//! Each function is tested with FPSCR = 0 (the engine's mode) and with FPSCR.FZ = 1 (Rust side under
//! `FtzGuard`). The inputs are fed through a small Thumb driver loop placed in scratch memory (one emulator
//! start per chunk of inputs), so millions of inputs per function are fast.
//!
//! `LXR_LIBM_EXHAUSTIVE=1` additionally runs `expf` and `logf` over all 2^32 inputs (FPSCR = 0).

use lxr_core::fpu::{libm, FtzGuard};
use lxr_difftest::Sweep;
use lxr_emu::firmware::FPSCR_FZ;
use lxr_emu::Firmware;

/// Thumb driver loops (assembled with arm-none-eabi-as 11.3):
/// ```text
/// drv_f32:   push {r3-r7, lr}; mov r4, r0; mov r5, r1; mov r6, r2; mov r7, r3
///         1: vldmia r5!, {s0};     blx r4; vstmia r6!, {s0}; subs r7, #1; bne 1b; pop {r3-r7, pc}
/// drv_f64:   ... 1: vldmia r5!, {d0};     blx r4; vstmia r6!, {d0}; ...
/// drv_f64x2: ... 1: vldmia r5!, {d0, d1}; blx r4; vstmia r6!, {d0}; ...
/// ```
/// r0 = function | 1, r1 = inputs, r2 = outputs, r3 = count (> 0).
const DRIVER: [u16; 39] = [
    // drv_f32 (offset 0x00)
    0xb5f8, 0x4604, 0x460d, 0x4616, 0x461f, 0xecb5, 0x0a01, 0x47a0, 0xeca6, 0x0a01, 0x3f01, 0xd1f8, 0xbdf8,
    // drv_f64 (offset 0x1a)
    0xb5f8, 0x4604, 0x460d, 0x4616, 0x461f, 0xecb5, 0x0b02, 0x47a0, 0xeca6, 0x0b02, 0x3f01, 0xd1f8, 0xbdf8,
    // drv_f64x2 (offset 0x34)
    0xb5f8, 0x4604, 0x460d, 0x4616, 0x461f, 0xecb5, 0x0b04, 0x47a0, 0xeca6, 0x0b02, 0x3f01, 0xd1f8, 0xbdf8,
];
const DRV_F32: u32 = 0x00;
const DRV_F64: u32 = 0x1a;
const DRV_F64X2: u32 = 0x34;
const BUF: usize = 1 << 20;

struct Lib {
    name: &'static str,
    fw: Firmware,
    drv: u32,
    inb: u32,
    outb: u32,
}

impl Lib {
    fn new(name: &'static str, mut fw: Firmware) -> Lib {
        let drv = fw.alloc(0x100).unwrap();
        let bytes: Vec<u8> = DRIVER.iter().flat_map(|h| h.to_le_bytes()).collect();
        fw.write(drv, &bytes).unwrap();
        let inb = fw.alloc(BUF).unwrap();
        let outb = fw.alloc(BUF).unwrap();
        fw.insn_limit = 0;
        Lib { name, fw, drv, inb, outb }
    }

    fn run(&mut self, func: &str, drv: u32, in_words: usize, out_words: usize, input: &[u32]) -> Vec<u32> {
        let f = self.fw.addr(func).unwrap() | 1;
        let per = BUF / 4 / in_words.max(out_words);
        let mut out = Vec::with_capacity(input.len() / in_words * out_words);
        for chunk in input.chunks(per * in_words) {
            let n = chunk.len() / in_words;
            let b: Vec<u8> = chunk.iter().flat_map(|w| w.to_le_bytes()).collect();
            self.fw.write(self.inb, &b).unwrap();
            self.fw.call_addr(self.drv + drv, &[f, self.inb, self.outb, n as u32], &[]).unwrap();
            let r = self.fw.read(self.outb, n * out_words * 4).unwrap();
            out.extend(r.chunks_exact(4).map(|c| u32::from_le_bytes(c.try_into().unwrap())));
        }
        out
    }

    fn f32s(&mut self, func: &str, xs: &[u32]) -> Vec<u32> {
        self.run(func, DRV_F32, 1, 1, xs)
    }

    fn f64s(&mut self, func: &str, xs: &[u64]) -> Vec<u64> {
        let w: Vec<u32> = xs.iter().flat_map(|x| [*x as u32, (*x >> 32) as u32]).collect();
        let r = self.run(func, DRV_F64, 2, 2, &w);
        r.chunks_exact(2).map(|c| c[0] as u64 | (c[1] as u64) << 32).collect()
    }

    fn f64x2s(&mut self, func: &str, xy: &[(u64, u64)]) -> Vec<u64> {
        let w: Vec<u32> =
            xy.iter().flat_map(|(x, y)| [*x as u32, (*x >> 32) as u32, *y as u32, (*y >> 32) as u32]).collect();
        let r = self.run(func, DRV_F64X2, 4, 2, &w);
        r.chunks_exact(2).map(|c| c[0] as u64 | (c[1] as u64) << 32).collect()
    }
}

fn libs() -> Vec<Lib> {
    let mut v = Vec::new();
    if let Ok(p) = std::env::var("LXR_FIRMWARE_IMG") {
        let img = std::fs::read(&p).unwrap_or_else(|e| panic!("read {p}: {e}"));
        let payload = lxr_emu::symbols::image_payload(&img).unwrap();
        let syms = lxr_emu::symbols::parse_symbol_map(lxr_emu::symbols::V171_SYMBOLS_JSON).unwrap();
        // The addresses found by disassembly (see fpu/libm.rs) must agree with the symbol map.
        for (n, a) in [("exp", 0x0802_9648), ("expf", 0x0802_9888), ("logf", 0x0802_9970), ("fmod", 0x0802_9600)] {
            assert_eq!(syms[n].addr, a, "{n}");
        }
        v.push(Lib::new("shipped v1.71", Firmware::from_bin(payload, syms).unwrap()));
    }
    if let Some(fw) = lxr_difftest::reference_elf() {
        v.push(Lib::new("GCC 11.3 ELF", fw));
    }
    if v.is_empty() {
        eprintln!("neither LXR_FIRMWARE_IMG nor LXR_FW_ELF set; skipping libm differential tests");
    }
    v
}

fn report<I: Copy, T: Copy + PartialEq + std::fmt::LowerHex>(
    what: &str,
    lib: &str,
    fz: bool,
    ins: &[I],
    fw: &[T],
    rs: &[T],
    fmt: impl Fn(I) -> String,
) {
    assert_eq!(fw.len(), rs.len());
    let mut bad = 0usize;
    for i in 0..fw.len() {
        if fw[i] != rs[i] {
            if bad < 10 {
                eprintln!(
                    "{what} [{lib}, FZ={}] input {}: firmware {:#x} rust {:#x}",
                    fz as u8,
                    fmt(ins[i]),
                    fw[i],
                    rs[i]
                );
            }
            bad += 1;
        }
    }
    eprintln!("{what} [{lib}, FZ={}]: {} inputs, {bad} mismatches", fz as u8, fw.len());
    assert_eq!(bad, 0, "{what} [{lib}, FZ={}]: {bad} mismatches", fz as u8);
}

fn rust_f32(f: fn(f32) -> f32, xs: &[u32], fz: bool) -> Vec<u32> {
    let _g = fz.then(FtzGuard::new);
    xs.iter().map(|&x| f(f32::from_bits(x)).to_bits()).collect()
}

fn rust_f64(f: fn(f64) -> f64, xs: &[u64], fz: bool) -> Vec<u64> {
    let _g = fz.then(FtzGuard::new);
    xs.iter().map(|&x| f(f64::from_bits(x)).to_bits()).collect()
}

/// Bit patterns within `±r` of each centre (wrapping), plus the special values.
fn around_f32(centres: &[u32], r: u32) -> Vec<u32> {
    let mut v = Vec::new();
    for &c in centres {
        for d in 0..=2 * r {
            v.push(c.wrapping_sub(r).wrapping_add(d));
        }
    }
    v
}

const SPECIAL_F32: [u32; 16] = [
    0x0000_0000, 0x8000_0000, 0x0000_0001, 0x8000_0001, 0x007f_ffff, 0x807f_ffff, 0x0080_0000, 0x8080_0000,
    0x7f80_0000, 0xff80_0000, 0x7fc0_0000, 0xffc0_0000, 0x7f80_0001, 0xff80_0001, 0x7fa0_1234, 0x7f7f_ffff,
];

/// Inputs for the f32 functions: a strided pass over all 2^32 patterns, random patterns, dense windows around
/// the given centres, all-subnormal samples, special values and engine-range values.
fn f32_inputs(centres: &[u32], engine: impl Fn(&mut Sweep) -> f32, seed: u64) -> Vec<u32> {
    let mut v: Vec<u32> = SPECIAL_F32.to_vec();
    // strided over the whole space (stride 251, ~17.1M patterns, all exponents, both signs)
    let mut b: u64 = 0;
    while b < 1u64 << 32 {
        v.push(b as u32);
        b += 251;
    }
    let mut s = Sweep(seed);
    for _ in 0..2_000_000 {
        v.push(s.next_u32() ^ (s.next_u32() << 16));
    }
    v.extend(around_f32(centres, 1 << 16));
    // subnormals (both signs), strided
    for m in (0..0x0080_0000u32).step_by(7) {
        v.push(m);
        v.push(m | 0x8000_0000);
    }
    for _ in 0..1_000_000 {
        v.push(engine(&mut s).to_bits());
    }
    v
}

#[test]
fn libm_expf() {
    // first x with expf(x) below 0x1p-126 region, overflow/underflow bounds, abstop boundary (|x| >= 88)
    let centres = [
        0x42b1_7217, 0xc2cf_f1b4, 0xc2ce_8ecf, 0x42b0_0000, 0xc2b0_0000, (-87.336_55f32).to_bits(), 0x0000_0000,
        0x8000_0000, 0x3f80_0000, 0xbf80_0000, 0x7f80_0000, 0xff80_0000, 0x3300_0000, 0xb300_0000,
    ];
    // engine: dB2lin(dB) = expf((float)(dB * DB_2_LOG)), dB in about [-200, 60]
    let xs = f32_inputs(&centres, |s| (s.f32_in(-200.0, 60.0) as f64 * 0.115_129_254_649_702_28) as f32, 1);
    for mut lib in libs() {
        for fz in [false, true] {
            lib.fw.fpscr = if fz { FPSCR_FZ } else { 0 };
            let fw = lib.f32s("expf", &xs);
            let rs = rust_f32(libm::expf, &xs, fz);
            report("expf", lib.name, fz, &xs, &fw, &rs, |x| format!("{x:#010x}"));
        }
    }
}

#[test]
fn libm_logf() {
    let centres = [
        0x3f80_0000, 0x3f33_0000, 0x0080_0000, 0x7f80_0000, 0x0000_0000, 0x8000_0000, 0x7f00_0000, 0x3f7f_ffff,
        0x4000_0000, 0x3f00_0000,
    ];
    // engine: lin2dB(lin) = logf((float)lin), lin = |signal| level, about [1e-12, 16]
    let xs = f32_inputs(&centres, |s| 10f32.powf(s.f32_in(-12.0, 1.2)), 2);
    for mut lib in libs() {
        for fz in [false, true] {
            lib.fw.fpscr = if fz { FPSCR_FZ } else { 0 };
            let fw = lib.f32s("logf", &xs);
            let rs = rust_f32(libm::logf, &xs, fz);
            report("logf", lib.name, fz, &xs, &fw, &rs, |x| format!("{x:#010x}"));
        }
    }
}

/// All 2^32 inputs of expf and logf (FPSCR = 0); only with `LXR_LIBM_EXHAUSTIVE=1` (both functions) or
/// `LXR_LIBM_EXHAUSTIVE=<expf|logf>[:<k>/<n>]` (one function, shard k of n of the 2^32 patterns, to run
/// several processes in parallel).
#[test]
fn libm_f32_exhaustive() {
    let Some(spec) = std::env::var("LXR_LIBM_EXHAUSTIVE").ok() else {
        eprintln!("LXR_LIBM_EXHAUSTIVE not set; skipping");
        return;
    };
    let (which, shard) = spec.split_once(':').unwrap_or((spec.as_str(), "0/1"));
    let (k, n) = shard.split_once('/').expect("shard k/n");
    let (k, n): (u64, u64) = (k.parse().unwrap(), n.parse().unwrap());
    let step = 1u64 << 24;
    assert!(n > 0 && k < n && (256 % n) == 0, "n must divide 256");
    let (lo, hi) = (k * (1u64 << 32) / n, (k + 1) * (1u64 << 32) / n);
    for mut lib in libs() {
        lib.fw.fpscr = 0;
        for (name, f) in [("expf", libm::expf as fn(f32) -> f32), ("logf", libm::logf)] {
            if which != "1" && which != name {
                continue;
            }
            let mut bad = 0u64;
            let mut base = lo;
            while base < hi {
                let xs: Vec<u32> = (base..base + step).map(|b| b as u32).collect();
                let fw = lib.f32s(name, &xs);
                let rs = rust_f32(f, &xs, false);
                for i in 0..xs.len() {
                    if fw[i] != rs[i] {
                        if bad < 10 {
                            eprintln!("{name} [{}] {:#010x}: fw {:#x} rust {:#x}", lib.name, xs[i], fw[i], rs[i]);
                        }
                        bad += 1;
                    }
                }
                base += step;
            }
            eprintln!("{name} [{}, FZ=0]: inputs {lo:#x}..{hi:#x} (exhaustive), {bad} mismatches", lib.name);
            assert_eq!(bad, 0);
        }
    }
}

const SPECIAL_F64: [u64; 18] = [
    0x0000_0000_0000_0000,
    0x8000_0000_0000_0000,
    0x0000_0000_0000_0001,
    0x8000_0000_0000_0001,
    0x000f_ffff_ffff_ffff,
    0x800f_ffff_ffff_ffff,
    0x0010_0000_0000_0000,
    0x8010_0000_0000_0000,
    0x7ff0_0000_0000_0000,
    0xfff0_0000_0000_0000,
    0x7ff8_0000_0000_0000,
    0xfff8_0000_0000_0000,
    0x7ff0_0000_0000_0001,
    0xfff4_0000_dead_beef,
    0x7ffc_1234_5678_9abc,
    0x7fef_ffff_ffff_ffff,
    0xffef_ffff_ffff_ffff,
    0x3ff0_0000_0000_0000,
];

fn rand_u64(s: &mut Sweep) -> u64 {
    (s.next_u32() as u64) << 32 | s.next_u32() as u64
}

fn rand_f64(s: &mut Sweep, lo: f64, hi: f64) -> f64 {
    lo + (hi - lo) * ((rand_u64(s) >> 11) as f64 / (1u64 << 53) as f64)
}

#[test]
fn libm_exp() {
    let mut xs: Vec<u64> = SPECIAL_F64.to_vec();
    let mut s = Sweep(3);
    // random bit patterns (all exponents, both signs)
    for _ in 0..1_000_000 {
        xs.push(rand_u64(&mut s));
    }
    // random patterns with the exponent in the interesting range 0x3c0..0x40a
    for _ in 0..1_000_000 {
        let e = 0x3c0 + (s.next_u32() % 0x4b) as u64;
        xs.push((rand_u64(&mut s) & 0x800f_ffff_ffff_ffff) | e << 52);
    }
    // uniform over the whole finite-result range and beyond
    for _ in 0..1_000_000 {
        xs.push(rand_f64(&mut s, -760.0, 720.0).to_bits());
    }
    // dense around the thresholds: 0x1p-54, 512, 1024, the overflow bound, the subnormal / underflow bounds
    let centres: [f64; 10] = [
        f64::from_bits(0x3c90_0000_0000_0000),
        512.0,
        1024.0,
        709.782_712_893_384,
        -708.396_418_532_264_1,
        -745.133_219_101_941_1,
        -744.440_071_921_381_3,
        -737.0,
        0.0,
        -512.0,
    ];
    for c in centres {
        for sign in [1.0f64, -1.0] {
            let b = (c * sign).to_bits();
            for d in 0..20_000u64 {
                xs.push(b.wrapping_sub(10_000).wrapping_add(d));
            }
            for _ in 0..20_000 {
                xs.push((c * sign + rand_f64(&mut s, -1.0, 1.0)).to_bits());
            }
        }
    }
    // engine: Compressor_setAttack / setRelease: exp(-1000.0 / (t * REAL_FS)), t in [0.03, 100.03] / [25, 200]
    for _ in 0..200_000 {
        let t = (0.03f32 + s.f32_in(0.0, 1.0) * 100.0) as f64;
        xs.push((-1000.0 / (t * 44100.0)).to_bits());
        xs.push(rand_f64(&mut s, -1.0, 0.0).to_bits());
    }
    for mut lib in libs() {
        for fz in [false, true] {
            lib.fw.fpscr = if fz { FPSCR_FZ } else { 0 };
            let fw = lib.f64s("exp", &xs);
            let rs = rust_f64(libm::exp, &xs, fz);
            report("exp", lib.name, fz, &xs, &fw, &rs, |x| format!("{x:#018x} ({:e})", f64::from_bits(x)));
        }
    }
}

#[test]
fn libm_fmod() {
    let mut xy: Vec<(u64, u64)> = Vec::new();
    let mut s = Sweep(4);
    // all pairs of special values
    let mut special: Vec<u64> = SPECIAL_F64.to_vec();
    for v in [1.5f64, 3.0, -3.0, 0.1, 1e300, 1e-300, 2.5e-310, 7.0, 0.75] {
        special.push(v.to_bits());
        special.push((-v).to_bits());
    }
    for &a in &special {
        for &b in &special {
            xy.push((a, b));
        }
    }
    // random bit patterns
    for _ in 0..400_000 {
        xy.push((rand_u64(&mut s), rand_u64(&mut s)));
    }
    // random with exponents close to each other (long and short shift-subtract loops, all output paths)
    for _ in 0..400_000 {
        let ex = s.next_u32() as u64 % 2047;
        let ey = (ex as i64 + (s.next_u32() % 120) as i64 - 60).clamp(0, 2046) as u64;
        let x = (rand_u64(&mut s) & 0x800f_ffff_ffff_ffff) | ex << 52;
        let y = (rand_u64(&mut s) & 0x800f_ffff_ffff_ffff) | ey << 52;
        xy.push((x, y));
    }
    // subnormal / tiny operands
    for _ in 0..200_000 {
        let x = rand_u64(&mut s) & 0x801f_ffff_ffff_ffff;
        let y = rand_u64(&mut s) & 0x801f_ffff_ffff_ffff;
        xy.push((x, y));
        xy.push((x, y >> (s.next_u32() % 50)));
    }
    // exact multiples and |x| == |y|
    for _ in 0..100_000 {
        let y = rand_f64(&mut s, -10.0, 10.0);
        let k = (s.next_u32() % 1000) as f64;
        xy.push(((y * k).to_bits(), y.to_bits()));
        xy.push(((-y).to_bits(), y.to_bits()));
    }
    // engine: Distortion_process: fmod(in - threshold, threshold * 4) on float values promoted to double
    for _ in 0..1_000_000 {
        let th = s.f32_in(0.0, 1.0);
        let input = s.f32_in(-8.0, 8.0);
        let x = (input as f64) - (th as f64);
        xy.push((x.to_bits(), (th as f64 * 4.0).to_bits()));
        xy.push((((input - th) as f64).to_bits(), ((th * 4.0) as f64).to_bits()));
    }
    for mut lib in libs() {
        for fz in [false, true] {
            lib.fw.fpscr = if fz { FPSCR_FZ } else { 0 };
            let fw = lib.f64x2s("fmod", &xy);
            let rs: Vec<u64> = {
                let _g = fz.then(FtzGuard::new);
                xy.iter().map(|&(x, y)| libm::fmod(f64::from_bits(x), f64::from_bits(y)).to_bits()).collect()
            };
            report("fmod", lib.name, fz, &xy, &fw, &rs, |(x, y)| format!("x = {x:#018x}, y = {y:#018x}"));
        }
    }
}
