//! P2: FxBus against the reference firmware (global `fxBus` and `FX_BUFFER`): init, setParameter /
//! updateParameter sweeps for every effect type, and processing runs over all effect types, delay types and
//! audio routings (stereo, mono-left and mono-right DAC outputs), with effect and parameter changes between
//! blocks. FPSCR = 0. The ring modulator needs P1's oscillator (or `LXR_P2_STUB_OSC=1`, see p2_common).
mod p2_common;

use lxr_core::engine::Lxr;
use lxr_core::fx_bus::*;
use lxr_difftest::Sweep;
use lxr_emu::Firmware;
use p2_common::*;

const MAX_BLOCK: usize = 256;

struct Ctx {
    fw: Firmware,
    lxr: Box<Lxr>,
    bus: u32,
    fxbuf: u32,
    fxptr: u32,
    outptr: u32,
    out2ptr: u32,
}

fn new_ctx() -> Option<Ctx> {
    let mut fw = setup()?;
    let bus = fw.addr("fxBus").unwrap();
    assert_eq!(fw.sym("fxBus").unwrap().size, 352);
    let fxbuf = fw.addr("FX_BUFFER").unwrap();
    let fxptr = fw.alloc(MAX_BLOCK * 4).unwrap();
    let outptr = fw.alloc(MAX_BLOCK * 4 + 8).unwrap();
    let out2ptr = fw.alloc(MAX_BLOCK * 4 + 8).unwrap();
    Some(Ctx { fw, lxr: Box::new(Lxr::new_zeroed()), bus, fxbuf, fxptr, outptr, out2ptr })
}

impl Ctx {
    fn check(&self, ctx: &str) {
        let fwb = canon(&self.fw.read(self.bus, 352).unwrap(), fxbus_from, fxbus_bytes);
        assert_bytes("fxBus", ctx, &fwb, &fxbus_bytes(&self.lxr.fxBus));
    }
    fn put(&mut self) {
        let b = fxbus_bytes(&self.lxr.fxBus);
        self.fw.write(self.bus, &b).unwrap();
    }
    fn check_fx_buffer(&self, ctx: &str) {
        let fwb = self.fw.read(self.fxbuf, 65536).unwrap();
        let rb: Vec<u8> = self.lxr.FX_BUFFER.iter().flat_map(|x| x.to_le_bytes()).collect();
        assert_bytes("FX_BUFFER", ctx, &fwb, &rb);
    }
    fn init(&mut self) {
        self.fw.call("FxBus_init", &[], &[]).unwrap();
        self.lxr.FxBus_init();
        self.check("FxBus_init");
    }
    fn set_parameter(&mut self, idx: u8, v: f32) {
        self.fw.call("FxBus_setParameter", &[idx as u32], &[v]).unwrap();
        self.lxr.FxBus_setParameter(idx, v);
        self.check(&format!("FxBus_setParameter({idx}, {v:e}) type {}", self.lxr.fxBus.fxType));
    }
    fn update_parameter(&mut self) {
        self.fw.call("FxBus_updateParameter", &[], &[]).unwrap();
        self.lxr.FxBus_updateParameter();
        self.check(&format!("FxBus_updateParameter type {}", self.lxr.fxBus.fxType));
    }
    fn process(&mut self, sw: &mut Sweep, size: u32, ctx: &str) {
        let n = size as usize * 2;
        let mut fx: Vec<i16> = (0..n).map(|_| sample(sw)).collect();
        let o1: Vec<i16> = (0..n + 2).map(|_| sample(sw) >> 2).collect();
        let o2: Vec<i16> = (0..n + 2).map(|_| sample(sw) >> 2).collect();
        self.fw.write_i16s(self.fxptr, &fx).unwrap();
        self.fw.write_i16s(self.outptr, &o1).unwrap();
        self.fw.write_i16s(self.out2ptr, &o2).unwrap();
        self.fw.call("FxBus_process", &[self.fxptr, self.outptr, self.outptr + 2, self.out2ptr, self.out2ptr + 2, size], &[]).unwrap();
        let (mut r1, mut r2) = (o1.clone(), o2.clone());
        self.lxr.FxBus_process(&mut fx, &mut r1, &mut r2, size);
        assert_i16s("output", ctx, &self.fw.read_i16s(self.outptr, n + 2).unwrap(), &r1);
        assert_i16s("output2", ctx, &self.fw.read_i16s(self.out2ptr, n + 2).unwrap(), &r2);
        self.check(ctx);
    }
}

#[test]
fn fxbus_init() {
    let Some(mut c) = new_ctx() else { return eprintln!("LXR_FW_ELF not set; skipping") };
    let mut sw = Sweep(0xfb_0001);
    for _ in 0..50 {
        let mut g = vec![0u8; 352];
        for b in g.iter_mut() {
            *b = sw.u8();
        }
        c.lxr.fxBus = fxbus_from(&g);
        c.put();
        // FxBus_prepareFxChange is empty (`bx lr`): no state change
        c.fw.call("FxBus_prepareFxChange", &[], &[]).unwrap();
        FxBus_prepareFxChange();
        c.check("FxBus_prepareFxChange");
        c.init();
    }
}

#[test]
fn fxbus_set_and_update_parameter() {
    let Some(mut c) = new_ctx() else { return eprintln!("LXR_FW_ELF not set; skipping") };
    let mut sw = Sweep(0xfb_0002);
    c.init();
    let mut n = 0;
    for round in 0..3000 {
        // every effect type (and out-of-range types, which update nothing)
        c.lxr.fxBus.fxType = (round % 7) as u8;
        if round % 10 == 0 {
            c.lxr.fxBus.delay.delayType ^= 1;
        }
        if round % 13 == 0 {
            // a cross-fade in progress: Delay_setDelayTime takes the pending path
            c.lxr.fxBus.delay.interpValue = 0.5;
        }
        c.put();
        for _ in 0..8 {
            let idx = (sw.next_u32() % 4) as u8;
            let v = param(&mut sw);
            c.set_parameter(idx, v);
            n += 1;
        }
        c.update_parameter();
        n += 1;
    }
    eprintln!("fxbus parameter calls: {n}");
}

fn run(seed: u64, blocks: usize, types: &[u8]) {
    let Some(mut c) = new_ctx() else { return eprintln!("LXR_FW_ELF not set; skipping") };
    let mut sw = Sweep(seed);
    // FX_BUFFER: audio-range halves
    for i in 0..32768 {
        c.lxr.FX_BUFFER[i] = lxr_core::fpu::f32_to_f16_bits(sw.f32_in(-0.5, 0.5));
    }
    let b: Vec<u8> = c.lxr.FX_BUFFER.iter().flat_map(|x| x.to_le_bytes()).collect();
    c.fw.write(c.fxbuf, &b).unwrap();
    c.init();
    let mut samples = 0usize;
    for blk in 0..blocks {
        if blk % 40 == 0 || sw.next_u32() % 30 == 0 {
            c.lxr.fxBus.fxType = types[(sw.next_u32() as usize) % types.len()];
            c.lxr.fxBus.audioRouting = (sw.next_u32() % 7) as u8;
            c.lxr.fxBus.dryWet = param(&mut sw);
            if sw.next_u32() % 3 == 0 {
                let d = &mut c.lxr.fxBus.delay;
                d.delayType ^= 1;
                if d.delayType == lxr_core::dsp::fx::delay::DELAY_PINGPONG {
                    // A normal -> ping-pong switch with pointers above the half size makes the firmware read
                    // (and write one sample) past FX_BUFFER into other globals, which the port does not model
                    // (documented as not bit-exact). Keep the pointers in the half buffer here.
                    d.write &= 0x3fff;
                    d.oldRead &= 0x3fff;
                    d.newRead &= 0x3fff;
                }
            }
            if sw.next_u32() % 3 == 0 {
                c.lxr.fxBus.distortion.distortionType = (sw.next_u32() % 3) as u8;
            }
            if sw.next_u32() % 3 == 0 {
                c.lxr.fxBus.compressor.ratio = [1.0, 0.5, 0.25, 0.125][(sw.next_u32() % 4) as usize];
            }
            c.put();
            c.update_parameter();
        }
        if sw.next_u32() % 5 == 0 {
            let idx = (sw.next_u32() % 4) as u8;
            let v = param01(&mut sw);
            c.set_parameter(idx, v);
        }
        let size: u32 = match sw.next_u32() % 6 {
            0 => 1 + sw.next_u32() % 255,
            1 => 1,
            _ => 32,
        };
        let f = &c.lxr.fxBus;
        let ctx = format!("seed {seed:#x} block {blk} type {} routing {} size {size}", f.fxType, f.audioRouting);
        c.process(&mut sw, size, &ctx);
        samples += size as usize;
        if blk % 100 == 99 {
            c.check_fx_buffer(&ctx);
        }
    }
    c.check_fx_buffer("end");
    eprintln!("fxbus run seed {seed:#x}: {blocks} blocks, {samples} samples");
}

#[test]
fn fxbus_process_without_ringmod() {
    run(0xfb_0003, 20000, &[FX_OFF, FX_DISTORTION, FX_COMPRESSOR, FX_DELAY, 5]);
}

#[test]
fn fxbus_process_all_types() {
    run(0xfb_0004, 10000, &[FX_OFF, FX_DISTORTION, FX_RING, FX_COMPRESSOR, FX_DELAY]);
}
