//! P2: Delay (normal and ping-pong) against the reference firmware: init, setters (parameter sweeps,
//! pending delay time changes during cross-fades) and long processing runs with feedback, through the
//! `__fp16` FX_BUFFER (random finite contents incl. denormal and maximal halves), for stereo, mono-left
//! and mono-right outputs. FPSCR = 0 (main loop audio).
mod p2_common;

use lxr_core::dsp::fx::delay::*;
use lxr_core::engine::Lxr;
use lxr_difftest::Sweep;
use lxr_emu::Firmware;
use p2_common::*;

const MAX_BLOCK: usize = 256;

struct Ctx {
    fw: Firmware,
    lxr: Box<Lxr>,
    dptr: u32,
    fxptr: u32,
    outptr: u32,
    fxbuf: u32,
}

fn new_ctx() -> Option<Ctx> {
    let mut fw = setup()?;
    let dptr = fw.alloc(96).unwrap();
    let fxptr = fw.alloc(MAX_BLOCK * 4).unwrap();
    let outptr = fw.alloc(MAX_BLOCK * 4 + 8).unwrap();
    let fxbuf = fw.addr("FX_BUFFER").unwrap();
    let lxr = Box::new(Lxr::new_zeroed());
    Some(Ctx { fw, lxr, dptr, fxptr, outptr, fxbuf })
}

impl Ctx {
    fn fw_delay(&self) -> Vec<u8> {
        canon(&self.fw.read(self.dptr, 96).unwrap(), delay_from, delay_bytes)
    }
    fn check(&self, d: &Delay, ctx: &str) {
        assert_bytes("Delay", ctx, &self.fw_delay(), &delay_bytes(d));
    }
    fn check_fx_buffer(&self, ctx: &str) {
        let fwb = self.fw.read(self.fxbuf, 65536).unwrap();
        let rb: Vec<u8> = self.lxr.FX_BUFFER.iter().flat_map(|x| x.to_le_bytes()).collect();
        if fwb != rb {
            let i = fwb.iter().zip(&rb).position(|(a, b)| a != b).unwrap() / 2;
            panic!("FX_BUFFER differs at {i}: fw {:#06x} rust {:#06x} ({ctx})", u16::from_le_bytes([fwb[2 * i], fwb[2 * i + 1]]), self.lxr.FX_BUFFER[i]);
        }
    }
    fn fill_fx_buffer(&mut self, sw: &mut Sweep, mode: u32) {
        for i in 0..32768 {
            let h: u16 = match mode {
                0 => 0,
                1 => {
                    // any finite bits incl. denormals and the largest halves (the delay only writes finite
                    // halves, so NaN / infinity never occur in FX_BUFFER; see the NaN note in PORTING_STATUS)
                    let h = sw.next_u32() as u16;
                    if h & 0x7c00 == 0x7c00 { h & 0xfbff } else { h }
                }
                _ => {
                    // mostly audio-range halves, some denormals
                    if sw.next_u32() % 16 == 0 {
                        (sw.next_u32() as u16) & 0x83ff
                    } else {
                        lxr_core::fpu::f32_to_f16_bits(sw.f32_in(-1.0, 1.0))
                    }
                }
            };
            self.lxr.FX_BUFFER[i] = h;
        }
        let b: Vec<u8> = self.lxr.FX_BUFFER.iter().flat_map(|x| x.to_le_bytes()).collect();
        self.fw.write(self.fxbuf, &b).unwrap();
    }
    fn put(&mut self, d: &Delay) {
        self.fw.write(self.dptr, &delay_bytes(d)).unwrap();
    }
    fn setter(&mut self, d: &mut Delay, which: u32, v: f32) {
        let name = match which {
            0 => {
                Delay_setDelayTime(d, v);
                "Delay_setDelayTime"
            }
            1 => {
                Delay_setFeedback(d, v);
                "Delay_setFeedback"
            }
            2 => {
                Delay_setRange(d, v);
                "Delay_setRange"
            }
            _ => {
                Delay_setFilter(d, v);
                "Delay_setFilter"
            }
        };
        self.fw.call(name, &[self.dptr], &[v]).unwrap();
        self.check(d, &format!("{name}({v:e} / {:#010x})", v.to_bits()));
    }
    /// One process call; `layout`: 0 stereo, 1 mono into the left samples, 2 mono into the right samples.
    fn process(&mut self, d: &mut Delay, sw: &mut Sweep, size: u16, layout: u32, dryWet: f32, input_scale: u32, ctx: &str) {
        let n = size as usize * 2;
        let fx: Vec<i16> = (0..n).map(|_| sample(sw) >> input_scale).collect();
        let out0: Vec<i16> = (0..n + 2).map(|_| sample(sw) >> 2).collect();
        self.fw.write_i16s(self.fxptr, &fx).unwrap();
        self.fw.write_i16s(self.outptr, &out0).unwrap();
        let channels: u8 = if layout == 0 { 2 } else { 1 };
        let off = if layout == 2 { 1 } else { 0 };
        let name = if d.delayType == DELAY_NORMAL { "Delay_process" } else { "Delay_processPingPong" };
        let o = self.outptr + 2 * off as u32;
        self.fw.call(name, &[self.dptr, self.fxptr, o, o + 2, size as u32, channels as u32], &[dryWet]).unwrap();
        let mut out = out0.clone();
        if d.delayType == DELAY_NORMAL {
            self.lxr.Delay_process(d, &fx, &mut out[off..], size, channels, dryWet);
        } else {
            self.lxr.Delay_processPingPong(d, &fx, &mut out[off..], size, channels, dryWet);
        }
        let fw_out = self.fw.read_i16s(self.outptr, n + 2).unwrap();
        assert_i16s("output", ctx, &fw_out, &out);
        self.check(d, ctx);
    }
}

#[test]
fn delay_init_and_setters() {
    let Some(mut c) = new_ctx() else { return eprintln!("LXR_FW_ELF not set; skipping") };
    let mut sw = Sweep(0xde1a_0001);
    let mut n = 0;
    for case in 0..2000 {
        let mut d = Delay::ZERO;
        // start from garbage so init has to write every field it owns
        let mut g = delay_bytes(&d);
        for b in g.iter_mut() {
            *b = sw.u8();
        }
        g[88] &= 1;
        d = delay_from(&g);
        // fields Delay_init leaves alone: finite values (the firmware never holds NaN / infinity here)
        d.range = sw.f32_in(-1.0, 2.0);
        d.delayTime = sw.f32_in(-1.0, 2.0);
        c.put(&d);
        c.fw.call("Delay_init", &[c.dptr], &[]).unwrap();
        Delay_init(&mut d);
        c.check(&d, &format!("Delay_init case {case}"));
        d.delayType = (case & 1) as u8;
        d.write = sw.next_u32() & 0x7fff;
        c.put(&d);
        for _ in 0..40 {
            let which = sw.next_u32() % 4;
            let v = param(&mut sw);
            if sw.next_u32() % 3 == 0 {
                // let a cross-fade run partly so setDelayTime takes the pending path
                d.interpValue = [0.5, 0.0, -0.0, 1e-38, -1.0, 1e-45][(sw.next_u32() % 6) as usize];
                c.put(&d);
            }
            c.setter(&mut d, which, v);
            n += 1;
        }
    }
    eprintln!("delay setters: {n} calls");
}

fn run(seed: u64, ping_pong: bool, blocks: usize, fb_hi: bool, fill: u32) {
    let Some(mut c) = new_ctx() else { return eprintln!("LXR_FW_ELF not set; skipping") };
    let mut sw = Sweep(seed);
    c.fill_fx_buffer(&mut sw, fill);
    let mut d = Delay::ZERO;
    c.put(&d);
    c.fw.call("Delay_init", &[c.dptr], &[]).unwrap();
    Delay_init(&mut d);
    d.delayType = ping_pong as u8;
    c.put(&d);
    c.setter(&mut d, 2, 0.3);
    c.setter(&mut d, 0, 0.5);
    c.setter(&mut d, 3, 0.7);
    c.setter(&mut d, 1, if fb_hi { 0.98 } else { 0.6 });
    let mut samples = 0usize;
    for blk in 0..blocks {
        if sw.next_u32() % 8 == 0 {
            let which = sw.next_u32() % 4;
            let mut v = param01(&mut sw);
            if which == 1 && fb_hi {
                v = 0.9 + 0.1 * sw.f32_in(0.0, 1.0);
            }
            c.setter(&mut d, which, v);
        }
        let size: u16 = match sw.next_u32() % 6 {
            0 => 1 + (sw.next_u32() % 255) as u16,
            1 => 1,
            _ => 32,
        };
        let layout = sw.next_u32() % 3;
        let dw = if sw.next_u32() % 4 == 0 { param(&mut sw) } else { 0.5 };
        // quiet input in some stretches so the feedback path decays into half precision denormals
        let scale = if (blk / 64) % 3 == 2 { 15 } else { 0 };
        c.process(&mut d, &mut sw, size, layout, dw, scale, &format!("seed {seed:#x} block {blk} size {size} layout {layout}"));
        samples += size as usize;
        if blk % 50 == 49 {
            c.check_fx_buffer(&format!("seed {seed:#x} after block {blk}"));
        }
    }
    c.check_fx_buffer(&format!("seed {seed:#x} end"));
    eprintln!("delay run seed {seed:#x} pingpong {ping_pong}: {blocks} blocks, {samples} samples");
}

#[test]
fn delay_normal_long_run() {
    run(0xd1, false, 20000, false, 2);
}
#[test]
fn delay_normal_high_feedback_quiet() {
    run(0xd2, false, 20000, true, 0);
}
#[test]
fn delay_normal_random_bits() {
    run(0xd3, false, 8000, false, 1);
}
#[test]
fn delay_pingpong_long_run() {
    run(0xd4, true, 20000, false, 2);
}
#[test]
fn delay_pingpong_high_feedback_quiet() {
    run(0xd5, true, 20000, true, 0);
}
#[test]
fn delay_pingpong_random_bits() {
    run(0xd6, true, 8000, false, 1);
}
