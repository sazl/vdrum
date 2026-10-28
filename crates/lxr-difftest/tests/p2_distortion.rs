//! P2: master distortion (tube, foldback, RAT) against the reference firmware: init, setters and long
//! processing runs with parameter sweeps for stereo, mono-left and mono-right outputs. FPSCR = 0.
mod p2_common;

use lxr_core::dsp::fx::distortion::*;
use lxr_difftest::Sweep;
use lxr_emu::Firmware;
use p2_common::*;

const MAX_BLOCK: usize = 256;

struct Ctx {
    fw: Firmware,
    dptr: u32,
    fxptr: u32,
    outptr: u32,
}

fn new_ctx() -> Option<Ctx> {
    let mut fw = setup()?;
    let dptr = fw.alloc(152).unwrap();
    let fxptr = fw.alloc(MAX_BLOCK * 4).unwrap();
    let outptr = fw.alloc(MAX_BLOCK * 4 + 8).unwrap();
    Some(Ctx { fw, dptr, fxptr, outptr })
}

impl Ctx {
    fn check(&self, d: &MasterDistortion, ctx: &str) {
        let fwb = canon(&self.fw.read(self.dptr, 152).unwrap(), dist_from, dist_bytes);
        assert_bytes("MasterDistortion", ctx, &fwb, &dist_bytes(d));
    }
    fn put(&mut self, d: &MasterDistortion) {
        self.fw.write(self.dptr, &dist_bytes(d)).unwrap();
    }
    fn setter(&mut self, d: &mut MasterDistortion, which: u32, v: f32) {
        let name = match which {
            0 => {
                Distortion_setDrive(d, v);
                "Distortion_setDrive"
            }
            1 => {
                Distortion_setParam1(d, v);
                "Distortion_setParam1"
            }
            2 => {
                Distortion_setParam2(d, v);
                "Distortion_setParam2"
            }
            _ => {
                Distortion_setParam3(d, v);
                "Distortion_setParam3"
            }
        };
        self.fw.call(name, &[self.dptr], &[v]).unwrap();
        self.check(d, &format!("{name}({v:e})"));
    }
    fn process(&mut self, d: &mut MasterDistortion, sw: &mut Sweep, size: u16, layout: u32, dryWet: f32, ctx: &str) {
        let n = size as usize * 2;
        let fx: Vec<i16> = (0..n).map(|_| sample(sw)).collect();
        let out0: Vec<i16> = (0..n + 2).map(|_| sample(sw) >> 2).collect();
        self.fw.write_i16s(self.fxptr, &fx).unwrap();
        self.fw.write_i16s(self.outptr, &out0).unwrap();
        let channels: u32 = if layout == 0 { 2 } else { 1 };
        let off = if layout == 2 { 1 } else { 0 };
        let o = self.outptr + 2 * off as u32;
        self.fw.call("Distortion_process", &[self.dptr, self.fxptr, o, o + 2, size as u32, channels], &[dryWet]).unwrap();
        let mut out = out0.clone();
        Distortion_process(d, &fx, &mut out[off..], size, channels as u8, dryWet);
        let fw_out = self.fw.read_i16s(self.outptr, n + 2).unwrap();
        assert_i16s("output", ctx, &fw_out, &out);
        self.check(d, ctx);
    }
}

/// `diode()` reads `table_1n914[257]`, the alignment padding after the table: it must be +0.0 in the image.
#[test]
fn table_1n914_padding_is_zero() {
    let Some(fw) = setup() else { return eprintln!("LXR_FW_ELF not set; skipping") };
    let t = fw.addr("table_1n914").unwrap();
    assert_eq!(t, 0x2000_0000);
    assert_eq!(fw.sym("table_1n914").unwrap().size, 257 * 4);
    assert_eq!(fw.read_u32(t + 257 * 4).unwrap(), 0);
    // and the Rust table equals the firmware's
    let b = fw.read(t, 257 * 4).unwrap();
    for i in 0..257 {
        assert_eq!(g32(&b, 4 * i), lxr_core::tables::distortion::table_1n914[i].to_bits(), "table_1n914[{i}]");
    }
}

#[test]
fn distortion_init_and_setters() {
    let Some(mut c) = new_ctx() else { return eprintln!("LXR_FW_ELF not set; skipping") };
    let mut sw = Sweep(0xd157_0001);
    for case in 0..1000 {
        let mut g = vec![0u8; 152];
        for b in g.iter_mut() {
            *b = sw.u8();
        }
        let mut d = dist_from(&g);
        c.put(&d);
        c.fw.call("Distortion_init", &[c.dptr], &[]).unwrap();
        Distortion_init(&mut d);
        c.check(&d, &format!("Distortion_init case {case}"));
        for _ in 0..20 {
            let which = sw.next_u32() % 4;
            let v = param(&mut sw);
            c.setter(&mut d, which, v);
        }
    }
}

fn run(seed: u64, dtype: u8, blocks: usize) {
    let Some(mut c) = new_ctx() else { return eprintln!("LXR_FW_ELF not set; skipping") };
    let mut sw = Sweep(seed);
    let mut d = MasterDistortion::ZERO;
    c.put(&d);
    c.fw.call("Distortion_init", &[c.dptr], &[]).unwrap();
    Distortion_init(&mut d);
    d.distortionType = dtype;
    c.put(&d);
    let mut samples = 0usize;
    for blk in 0..blocks {
        if blk % 4 == 0 {
            // parameter sweep: one setter per 4 blocks, sometimes all four
            let all = sw.next_u32() % 8 == 0;
            for which in 0..4 {
                if all || sw.next_u32() % 4 == which {
                    let v = param01(&mut sw);
                    c.setter(&mut d, which, v);
                }
            }
        }
        let size: u16 = match sw.next_u32() % 6 {
            0 => 1 + (sw.next_u32() % 255) as u16,
            1 => 1,
            _ => 32,
        };
        let layout = sw.next_u32() % 3;
        let dw = if sw.next_u32() % 4 == 0 { param(&mut sw) } else { 1.0 };
        c.process(&mut d, &mut sw, size, layout, dw, &format!("type {dtype} seed {seed:#x} block {blk} size {size} layout {layout}"));
        samples += size as usize;
    }
    eprintln!("distortion type {dtype}: {blocks} blocks, {samples} samples");
}

#[test]
fn distortion_tube_run() {
    run(0xd5_0001, DISTORTION_TUBE, 10000);
}
#[test]
fn distortion_fold_run() {
    run(0xd5_0002, DISTORTION_FOLD, 10000);
}
#[test]
fn distortion_rat_run() {
    run(0xd5_0003, DISTORTION_RAT, 10000);
}
#[test]
fn distortion_invalid_type_run() {
    run(0xd5_0004, 3, 500);
}
