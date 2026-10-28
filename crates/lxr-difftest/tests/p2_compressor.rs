//! P2: compressor against the reference firmware: init, setters (parameter and ratio sweeps) and long
//! processing runs (envelope follower attack/release, quiet stretches) for stereo, mono-left and mono-right
//! outputs. FPSCR = 0. libm calls use `fpu::libm` on both sides (see p2_common).
mod p2_common;

use lxr_core::dsp::fx::compressor::*;
use lxr_difftest::Sweep;
use lxr_emu::Firmware;
use p2_common::*;

const MAX_BLOCK: usize = 256;

struct Ctx {
    fw: Firmware,
    cptr: u32,
    fxptr: u32,
    outptr: u32,
}

fn new_ctx() -> Option<Ctx> {
    let mut fw = setup()?;
    let cptr = fw.alloc(32).unwrap();
    let fxptr = fw.alloc(MAX_BLOCK * 4).unwrap();
    let outptr = fw.alloc(MAX_BLOCK * 4 + 8).unwrap();
    Some(Ctx { fw, cptr, fxptr, outptr })
}

impl Ctx {
    fn check(&self, c: &Compressor, ctx: &str) {
        let fwb = canon(&self.fw.read(self.cptr, 32).unwrap(), comp_from, comp_bytes);
        assert_bytes("Compressor", ctx, &fwb, &comp_bytes(c));
    }
    fn put(&mut self, c: &Compressor) {
        self.fw.write(self.cptr, &comp_bytes(c)).unwrap();
    }
    fn setter(&mut self, c: &mut Compressor, which: u32, v: f32, r: u8) {
        let name = match which {
            0 => {
                Compressor_setAttack(c, v);
                "Compressor_setAttack"
            }
            1 => {
                Compressor_setRelease(c, v);
                "Compressor_setRelease"
            }
            2 => {
                Compressor_setTreshold(c, v);
                "Compressor_setTreshold"
            }
            3 => {
                Compressor_setGain(c, v);
                "Compressor_setGain"
            }
            _ => {
                Compressor_setRatio(c, r);
                "Compressor_setRatio"
            }
        };
        self.fw.call(name, &[self.cptr, r as u32], &[v]).unwrap();
        self.check(c, &format!("{name}({v:e}, {r})"));
    }
    fn process(&mut self, c: &mut Compressor, sw: &mut Sweep, size: u16, layout: u32, dryWet: f32, scale: u32, ctx: &str) {
        let n = size as usize * 2;
        let fx: Vec<i16> = (0..n).map(|_| sample(sw) >> scale).collect();
        let out0: Vec<i16> = (0..n + 2).map(|_| sample(sw) >> 2).collect();
        self.fw.write_i16s(self.fxptr, &fx).unwrap();
        self.fw.write_i16s(self.outptr, &out0).unwrap();
        let channels: u32 = if layout == 0 { 2 } else { 1 };
        let off = if layout == 2 { 1 } else { 0 };
        let o = self.outptr + 2 * off as u32;
        self.fw.call("Compressor_process", &[self.cptr, self.fxptr, o, o + 2, size as u32, channels], &[dryWet]).unwrap();
        let mut out = out0.clone();
        Compressor_process(c, &fx, &mut out[off..], size, channels as u8, dryWet);
        let fw_out = self.fw.read_i16s(self.outptr, n + 2).unwrap();
        assert_i16s("output", ctx, &fw_out, &out);
        self.check(c, ctx);
    }
}

#[test]
fn compressor_init_and_setters() {
    let Some(mut c) = new_ctx() else { return eprintln!("LXR_FW_ELF not set; skipping") };
    let mut sw = Sweep(0xc0_0001);
    let mut n = 0;
    for case in 0..1000 {
        let mut g = vec![0u8; 32];
        for b in g.iter_mut() {
            *b = sw.u8();
        }
        let mut cp = comp_from(&g);
        c.put(&cp);
        c.fw.call("Compressor_init", &[c.cptr], &[]).unwrap();
        Compressor_init(&mut cp);
        c.check(&cp, &format!("Compressor_init case {case}"));
        for _ in 0..30 {
            let which = sw.next_u32() % 5;
            let v = param(&mut sw);
            let r = if sw.next_u32() % 4 == 0 { sw.u8() } else { (sw.next_u32() % 7) as u8 };
            c.setter(&mut cp, which, v, r);
            n += 1;
        }
    }
    // dense sweeps of the libm-based setters
    let mut cp = Compressor::ZERO;
    c.put(&cp);
    for i in 0..=1000 {
        let v = i as f32 / 1000.0;
        c.setter(&mut cp, 0, v, 0);
        c.setter(&mut cp, 1, v, 0);
        n += 2;
    }
    eprintln!("compressor setters: {n} calls");
}

fn run(seed: u64, blocks: usize) {
    let Some(mut c) = new_ctx() else { return eprintln!("LXR_FW_ELF not set; skipping") };
    let mut sw = Sweep(seed);
    let mut cp = Compressor::ZERO;
    c.put(&cp);
    c.fw.call("Compressor_init", &[c.cptr], &[]).unwrap();
    Compressor_init(&mut cp);
    c.setter(&mut cp, 4, 0.0, 3);
    c.setter(&mut cp, 2, 0.7, 0);
    let mut samples = 0usize;
    for blk in 0..blocks {
        if sw.next_u32() % 6 == 0 {
            let which = sw.next_u32() % 5;
            let v = param(&mut sw);
            let r = (sw.next_u32() % 7) as u8;
            c.setter(&mut cp, which, v, r);
        }
        let size: u16 = match sw.next_u32() % 6 {
            0 => 1 + (sw.next_u32() % 255) as u16,
            1 => 1,
            _ => 32,
        };
        let layout = sw.next_u32() % 3;
        let dw = if sw.next_u32() % 4 == 0 { param(&mut sw) } else { 1.0 };
        // loud / quiet / silent stretches (attack, release, envelope decaying towards denormals)
        let scale = match (blk / 50) % 4 {
            0 | 1 => 0,
            2 => 10,
            _ => 15,
        };
        c.process(&mut cp, &mut sw, size, layout, dw, scale, &format!("seed {seed:#x} block {blk} size {size} layout {layout}"));
        samples += size as usize;
    }
    eprintln!("compressor run: {blocks} blocks, {samples} samples");
}

#[test]
fn compressor_long_run_a() {
    run(0xc0_0002, 10000);
}
#[test]
fn compressor_long_run_b() {
    run(0xc0_0003, 10000);
}
