//! P2: ring modulator against the reference firmware: init, setters and long processing runs for stereo,
//! mono-left and mono-right outputs. FPSCR = 0.
//!
//! The oscillator (`osc_setFreq`, `calcNextOscSampleBlock`) is P1 code: without `LXR_P2_STUB_OSC` these tests
//! compare the firmware's oscillator with P1's port as well; with `LXR_P2_STUB_OSC=1` the firmware uses the
//! stand-ins of p2_common (the Rust side must then use the same stand-ins).
mod p2_common;

use lxr_core::dsp::fx::ringmod::*;
use lxr_core::engine::Lxr;
use lxr_difftest::Sweep;
use lxr_emu::Firmware;
use p2_common::*;

const MAX_BLOCK: usize = 256;

struct Ctx {
    fw: Firmware,
    lxr: Box<Lxr>,
    rng: std::rc::Rc<std::cell::Cell<u32>>,
    rptr: u32,
    fxptr: u32,
    outptr: u32,
}

fn new_ctx() -> Option<Ctx> {
    let mut fw = setup()?;
    let rng = hook_rng(&mut fw);
    let rptr = fw.alloc(44).unwrap();
    let fxptr = fw.alloc(MAX_BLOCK * 4).unwrap();
    let outptr = fw.alloc(MAX_BLOCK * 4 + 8).unwrap();
    Some(Ctx { fw, lxr: Box::new(Lxr::new_zeroed()), rng, rptr, fxptr, outptr })
}

impl Ctx {
    fn check(&self, r: &Ringmod, ctx: &str) {
        let fwb = canon(&self.fw.read(self.rptr, 44).unwrap(), ring_from, ring_bytes);
        assert_bytes("Ringmod", ctx, &fwb, &ring_bytes(r));
    }
    fn put(&mut self, r: &Ringmod) {
        self.fw.write(self.rptr, &ring_bytes(r)).unwrap();
    }
    fn init(&mut self, r: &mut Ringmod) {
        self.put(r);
        self.fw.call("Ringmod_init", &[self.rptr], &[]).unwrap();
        Ringmod_init(r);
        self.check(r, "Ringmod_init");
    }
    fn set_freq(&mut self, r: &mut Ringmod, f: f32) {
        self.fw.call("Ringmod_setFreq", &[self.rptr], &[f]).unwrap();
        Ringmod_setFreq(r, f);
        self.check(r, &format!("Ringmod_setFreq({f:e})"));
    }
    fn set_wave(&mut self, r: &mut Ringmod, w: u8) {
        self.fw.call("Ringmod_setWave", &[self.rptr, w as u32], &[]).unwrap();
        Ringmod_setWave(r, w);
        self.check(r, &format!("Ringmod_setWave({w})"));
    }
    fn process(&mut self, r: &mut Ringmod, sw: &mut Sweep, size: u16, layout: u32, dryWet: f32, ctx: &str) {
        let n = size as usize * 2;
        let fx: Vec<i16> = (0..n).map(|_| sample(sw)).collect();
        let out0: Vec<i16> = (0..n + 2).map(|_| sample(sw) >> 2).collect();
        self.fw.write_i16s(self.fxptr, &fx).unwrap();
        self.fw.write_i16s(self.outptr, &out0).unwrap();
        let channels: u32 = if layout == 0 { 2 } else { 1 };
        let off = if layout == 2 { 1 } else { 0 };
        let o = self.outptr + 2 * off as u32;
        self.fw.call("Ringmod_process", &[self.rptr, self.fxptr, o, o + 2, size as u32, channels], &[dryWet]).unwrap();
        let mut out = out0.clone();
        self.lxr.Ringmod_process(r, &fx, &mut out[off..], size, channels as u8, dryWet);
        let fw_out = self.fw.read_i16s(self.outptr, n + 2).unwrap();
        assert_i16s("output", ctx, &fw_out, &out);
        self.check(r, ctx);
        assert_eq!(self.rng.get(), self.lxr.rng_state, "rng state ({ctx})");
    }
}

#[test]
fn ringmod_init_and_setters() {
    let Some(mut c) = new_ctx() else { return eprintln!("LXR_FW_ELF not set; skipping") };
    let mut sw = Sweep(0x7177_0001);
    for _ in 0..500 {
        let mut g = vec![0u8; 44];
        for b in g.iter_mut() {
            *b = sw.u8();
        }
        let mut r = ring_from(&g);
        c.init(&mut r);
        for _ in 0..20 {
            if sw.next_u32() % 3 == 0 {
                let w = (sw.next_u32() % 6) as u8;
                c.set_wave(&mut r, w);
            } else {
                let f = param(&mut sw);
                c.set_freq(&mut r, f);
            }
        }
    }
}

fn run(seed: u64, blocks: usize) {
    let Some(mut c) = new_ctx() else { return eprintln!("LXR_FW_ELF not set; skipping") };
    let mut sw = Sweep(seed);
    let mut r = Ringmod::ZERO;
    c.init(&mut r);
    let mut samples = 0usize;
    for blk in 0..blocks {
        if sw.next_u32() % 6 == 0 {
            let f = param(&mut sw).abs();
            c.set_freq(&mut r, f);
        }
        if sw.next_u32() % 20 == 0 {
            let w = (sw.next_u32() % 5) as u8;
            c.set_wave(&mut r, w);
        }
        let size: u16 = match sw.next_u32() % 6 {
            0 => 1 + (sw.next_u32() % 255) as u16,
            1 => 1,
            _ => 32,
        };
        let layout = sw.next_u32() % 3;
        let dw = if sw.next_u32() % 4 == 0 { param(&mut sw) } else { 1.0 };
        c.process(&mut r, &mut sw, size, layout, dw, &format!("seed {seed:#x} block {blk} size {size} layout {layout}"));
        samples += size as usize;
    }
    eprintln!("ringmod run: {blocks} blocks, {samples} samples");
}

#[test]
fn ringmod_long_run() {
    run(0x7177_0002, 10000);
}
