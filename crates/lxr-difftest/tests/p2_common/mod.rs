//! Shared helpers for the P2 (FX) differential tests: firmware setup, callee intercepts and the DWARF
//! struct layouts of the FX types (`gdb-multiarch -batch -ex 'ptype /o FxBus' lxr_v1.7_gcc11.3.elf`).
#![allow(dead_code, non_snake_case)]

use lxr_core::dsp::fx::compressor::Compressor;
use lxr_core::dsp::fx::delay::Delay;
use lxr_core::dsp::fx::distortion::MasterDistortion;
use lxr_core::dsp::fx::ringmod::Ringmod;
use lxr_core::dsp::one_pole_lp::OnePoleFilter;
use lxr_core::dsp::oscillator::OscInfo;
use lxr_core::fx_bus::FxBus;
use lxr_emu::firmware::HookState;
use lxr_emu::Firmware;
use unicorn_engine::{RegisterARM, Unicorn};

/// Reference firmware with FPSCR = 0 (the engine runs in the main loop with FZ = 0, DN = 0, AHP = 0; see
/// docs/ENGINE_CONTRACT.md) and the libm callees replaced by the Rust `fpu::libm` functions.
///
/// libm: the GCC 11.3 reference ELF links a newer newlib than the shipped v1.71 binary, whose `expf`,
/// `logf`, `exp`, `fmod` are the reference (docs/ARCHITECTURE.md). Both sides therefore use
/// `lxr_core::fpu::libm` (ported from the shipped binary by P1), and these tests check the FX code
/// around the libm calls with identical libm results.
///
/// Oscillator: with `LXR_P2_STUB_OSC=1`, `osc_setFreq` and `calcNextOscSampleBlock` are replaced in the
/// firmware by the same deterministic stand-ins as `stub_osc_*` below (only meaningful together with
/// matching stand-ins on the Rust side, used to verify the FX code while P1's oscillator was unported).
/// Without it, the firmware runs its own oscillator code and the test checks the Rust P1 port as well.
pub fn setup() -> Option<Firmware> {
    let mut fw = lxr_difftest::reference_elf()?;
    fw.fpscr = 0;
    install_libm(&mut fw);
    if std::env::var("LXR_P2_STUB_OSC").map(|v| v == "1").unwrap_or(false) {
        install_osc_stub(&mut fw);
    }
    Some(fw)
}

fn s(uc: &Unicorn<HookState>, i: i32) -> u32 {
    uc.reg_read(RegisterARM::S0 as i32 + i).unwrap() as u32
}
fn set_s(uc: &mut Unicorn<HookState>, i: i32, v: u32) {
    uc.reg_write(RegisterARM::S0 as i32 + i, v as u64).unwrap();
}
fn d0(uc: &Unicorn<HookState>) -> f64 {
    f64::from_bits((s(uc, 0) as u64) | ((s(uc, 1) as u64) << 32))
}
fn d1(uc: &Unicorn<HookState>) -> f64 {
    f64::from_bits((s(uc, 2) as u64) | ((s(uc, 3) as u64) << 32))
}
fn set_d0(uc: &mut Unicorn<HookState>, v: f64) {
    let b = v.to_bits();
    set_s(uc, 0, b as u32);
    set_s(uc, 1, (b >> 32) as u32);
}

pub fn install_libm(fw: &mut Firmware) {
    use lxr_core::fpu::libm;
    let a = fw.addr("expf").unwrap();
    fw.intercept(a, |uc| {
        let x = f32::from_bits(s(uc, 0));
        set_s(uc, 0, libm::expf(x).to_bits());
    })
    .unwrap();
    let a = fw.addr("logf").unwrap();
    fw.intercept(a, |uc| {
        let x = f32::from_bits(s(uc, 0));
        set_s(uc, 0, libm::logf(x).to_bits());
    })
    .unwrap();
    let a = fw.addr("exp").unwrap();
    fw.intercept(a, |uc| {
        let x = d0(uc);
        set_d0(uc, libm::exp(x));
    })
    .unwrap();
    let a = fw.addr("fmod").unwrap();
    fw.intercept(a, |uc| {
        let (x, y) = (d0(uc), d1(uc));
        set_d0(uc, libm::fmod(x, y));
    })
    .unwrap();
}

/// Stand-in for `osc_setFreq` (see `setup`).
pub fn stub_osc_setFreq(osc: &mut OscInfo) {
    osc.phaseInc = (osc.freq * 1000.0) as u32;
}
/// Stand-in for `calcNextOscSampleBlock` (see `setup`).
pub fn stub_osc_block(osc: &mut OscInfo, buf: &mut [i16], size: u8) {
    for b in buf.iter_mut().take(size as usize) {
        osc.phase = osc.phase.wrapping_add(osc.phaseInc);
        *b = (osc.phase >> 16) as i16;
    }
}

fn install_osc_stub(fw: &mut Firmware) {
    let a = fw.addr("osc_setFreq").unwrap();
    fw.intercept(a, |uc| {
        let p = uc.reg_read(RegisterARM::R0).unwrap();
        let b = uc.mem_read_as_vec(p, 40).unwrap();
        let mut o = osc_from(&b);
        stub_osc_setFreq(&mut o);
        uc.mem_write(p, &osc_bytes(&o)).unwrap();
    })
    .unwrap();
    let a = fw.addr("calcNextOscSampleBlock").unwrap();
    fw.intercept(a, |uc| {
        let p = uc.reg_read(RegisterARM::R0).unwrap();
        let bufp = uc.reg_read(RegisterARM::R1).unwrap();
        let size = uc.reg_read(RegisterARM::R2).unwrap() as u8;
        let b = uc.mem_read_as_vec(p, 40).unwrap();
        let mut o = osc_from(&b);
        let mut buf = vec![0i16; size as usize];
        stub_osc_block(&mut o, &mut buf, size);
        uc.mem_write(p, &osc_bytes(&o)).unwrap();
        let bb: Vec<u8> = buf.iter().flat_map(|x| x.to_le_bytes()).collect();
        uc.mem_write(bufp, &bb).unwrap();
    })
    .unwrap();
}

// ---- little-endian field access
pub fn g32(b: &[u8], o: usize) -> u32 {
    u32::from_le_bytes(b[o..o + 4].try_into().unwrap())
}
pub fn gf(b: &[u8], o: usize) -> f32 {
    f32::from_bits(g32(b, o))
}
pub fn gd(b: &[u8], o: usize) -> f64 {
    f64::from_bits(u64::from_le_bytes(b[o..o + 8].try_into().unwrap()))
}
pub fn g16(b: &[u8], o: usize) -> u16 {
    u16::from_le_bytes([b[o], b[o + 1]])
}
pub fn p32(b: &mut [u8], o: usize, v: u32) {
    b[o..o + 4].copy_from_slice(&v.to_le_bytes());
}
pub fn pf(b: &mut [u8], o: usize, v: f32) {
    p32(b, o, v.to_bits());
}
pub fn pd(b: &mut [u8], o: usize, v: f64) {
    b[o..o + 8].copy_from_slice(&v.to_bits().to_le_bytes());
}
pub fn p16(b: &mut [u8], o: usize, v: u16) {
    b[o..o + 2].copy_from_slice(&v.to_le_bytes());
}

// ---- OnePoleFilter (16 bytes): r@0, b@8
pub fn onepole_from(b: &[u8]) -> OnePoleFilter {
    OnePoleFilter { r: gd(b, 0), b: gd(b, 8) }
}
pub fn onepole_put(b: &mut [u8], f: &OnePoleFilter) {
    pd(b, 0, f.r);
    pd(b, 8, f.b);
}

// ---- Delay (96 bytes)
pub fn delay_bytes(d: &Delay) -> Vec<u8> {
    let mut b = vec![0u8; 96];
    p32(&mut b, 0, d.oldRead);
    p32(&mut b, 4, d.newRead);
    p32(&mut b, 8, d.write);
    pf(&mut b, 12, d.fracReadNew);
    pf(&mut b, 16, d.fracReadOld);
    pf(&mut b, 20, d.interpValue);
    pf(&mut b, 24, d.interpStep);
    p16(&mut b, 28, d.tmpBuffer[0] as u16);
    p16(&mut b, 30, d.tmpBuffer[1] as u16);
    p16(&mut b, 32, d.tmpBuffer2[0] as u16);
    p16(&mut b, 34, d.tmpBuffer2[1] as u16);
    pf(&mut b, 36, d.feedback);
    pf(&mut b, 40, d.range);
    pf(&mut b, 44, d.delayTime);
    pf(&mut b, 48, d.pendingDelayTime);
    b[52] = d.timePending;
    onepole_put(&mut b[56..72], &d.tone);
    onepole_put(&mut b[72..88], &d.tone2);
    b[88] = d.delayType;
    b
}
pub fn delay_from(b: &[u8]) -> Delay {
    Delay {
        oldRead: g32(b, 0),
        newRead: g32(b, 4),
        write: g32(b, 8),
        fracReadNew: gf(b, 12),
        fracReadOld: gf(b, 16),
        interpValue: gf(b, 20),
        interpStep: gf(b, 24),
        tmpBuffer: [g16(b, 28) as i16, g16(b, 30) as i16],
        tmpBuffer2: [g16(b, 32) as i16, g16(b, 34) as i16],
        feedback: gf(b, 36),
        range: gf(b, 40),
        delayTime: gf(b, 44),
        pendingDelayTime: gf(b, 48),
        timePending: b[52],
        tone: onepole_from(&b[56..72]),
        tone2: onepole_from(&b[72..88]),
        delayType: b[88],
    }
}

// ---- MasterDistortion (152 bytes)
pub fn dist_bytes(d: &MasterDistortion) -> Vec<u8> {
    let mut b = vec![0u8; 152];
    b[0] = d.distortionType;
    pf(&mut b, 4, d.drive);
    pf(&mut b, 8, d.p1);
    pf(&mut b, 12, d.p2);
    pf(&mut b, 16, d.p3);
    let arrs = [d.hp1_state, d.hp2_state, d.hp3_state, d.lp1_state, d.lp2_state, d.lp3_state, d.slewState, d.lastOut];
    for (j, a) in arrs.iter().enumerate() {
        pd(&mut b, 24 + 16 * j, a[0]);
        pd(&mut b, 32 + 16 * j, a[1]);
    }
    b
}
pub fn dist_from(b: &[u8]) -> MasterDistortion {
    let a = |j: usize| [gd(b, 24 + 16 * j), gd(b, 32 + 16 * j)];
    MasterDistortion {
        distortionType: b[0],
        drive: gf(b, 4),
        p1: gf(b, 8),
        p2: gf(b, 12),
        p3: gf(b, 16),
        hp1_state: a(0),
        hp2_state: a(1),
        hp3_state: a(2),
        lp1_state: a(3),
        lp2_state: a(4),
        lp3_state: a(5),
        slewState: a(6),
        lastOut: a(7),
    }
}

// ---- Compressor (32 bytes)
pub fn comp_bytes(c: &Compressor) -> Vec<u8> {
    let mut b = vec![0u8; 32];
    pf(&mut b, 0, c.tresh);
    pf(&mut b, 4, c.attack);
    pf(&mut b, 8, c.release);
    pf(&mut b, 12, c.ratio);
    pf(&mut b, 16, c.markupGain);
    pd(&mut b, 24, c.env);
    b
}
pub fn comp_from(b: &[u8]) -> Compressor {
    Compressor { tresh: gf(b, 0), attack: gf(b, 4), release: gf(b, 8), ratio: gf(b, 12), markupGain: gf(b, 16), env: gd(b, 24) }
}

// ---- OscInfo (40 bytes)
pub fn osc_bytes(o: &OscInfo) -> Vec<u8> {
    let mut b = vec![0u8; 40];
    p16(&mut b, 0, o.output as u16);
    p32(&mut b, 4, o.phaseInc);
    p32(&mut b, 8, o.phase);
    pf(&mut b, 12, o.freq);
    b[16] = o.waveform;
    b[17] = o.tableOffset;
    pf(&mut b, 20, o.pitchMod);
    pf(&mut b, 24, o.fmMod);
    pf(&mut b, 28, o.modNodeValue);
    p16(&mut b, 32, o.midiFreq);
    b[34] = o.baseNote;
    p32(&mut b, 36, o.startPhase);
    b
}
pub fn osc_from(b: &[u8]) -> OscInfo {
    OscInfo {
        output: g16(b, 0) as i16,
        phaseInc: g32(b, 4),
        phase: g32(b, 8),
        freq: gf(b, 12),
        waveform: b[16],
        tableOffset: b[17],
        pitchMod: gf(b, 20),
        fmMod: gf(b, 24),
        modNodeValue: gf(b, 28),
        midiFreq: g16(b, 32),
        baseNote: b[34],
        startPhase: g32(b, 36),
    }
}

// ---- Ringmod (44 bytes): freq@0, osc@4
pub fn ring_bytes(r: &Ringmod) -> Vec<u8> {
    let mut b = vec![0u8; 44];
    pf(&mut b, 0, r.freq);
    b[4..44].copy_from_slice(&osc_bytes(&r.osc));
    b
}
pub fn ring_from(b: &[u8]) -> Ringmod {
    Ringmod { freq: gf(b, 0), osc: osc_from(&b[4..44]) }
}

// ---- FxBus (352 bytes)
pub fn fxbus_bytes(f: &FxBus) -> Vec<u8> {
    let mut b = vec![0u8; 352];
    b[0] = f.fxType;
    b[1] = f.audioRouting;
    pf(&mut b, 4, f.dryWet);
    for i in 0..4 {
        pf(&mut b, 8 + 4 * i, f.parameter[i]);
    }
    b[24..176].copy_from_slice(&dist_bytes(&f.distortion));
    b[176..220].copy_from_slice(&ring_bytes(&f.ringmod));
    b[224..256].copy_from_slice(&comp_bytes(&f.compressor));
    b[256..352].copy_from_slice(&delay_bytes(&f.delay));
    b
}
pub fn fxbus_from(b: &[u8]) -> FxBus {
    FxBus {
        fxType: b[0],
        audioRouting: b[1],
        dryWet: gf(b, 4),
        parameter: [gf(b, 8), gf(b, 12), gf(b, 16), gf(b, 20)],
        distortion: dist_from(&b[24..176]),
        ringmod: ring_from(&b[176..220]),
        compressor: comp_from(&b[224..256]),
        delay: delay_from(&b[256..352]),
    }
}

/// Canonical bytes of a struct read back from firmware memory (holes zeroed) for bitwise comparison.
pub fn canon<T>(raw: &[u8], from: fn(&[u8]) -> T, to: fn(&T) -> Vec<u8>) -> Vec<u8> {
    to(&from(raw))
}

/// Random audio sample with a bias to full scale and small values.
pub fn sample(sw: &mut lxr_difftest::Sweep) -> i16 {
    match sw.next_u32() % 8 {
        0 => [i16::MIN, i16::MAX, -1, 0, 1, -32767][(sw.next_u32() % 6) as usize],
        1 => (sw.next_u32() as i16) >> 8,
        _ => sw.next_u32() as i16,
    }
}

/// Random float parameter: mostly in [0,1], sometimes edge values or out of range.
pub fn param(sw: &mut lxr_difftest::Sweep) -> f32 {
    match sw.next_u32() % 10 {
        0 => [0.0, 1.0, -0.0, 0.5, 1e-30, 0.999_999_94][(sw.next_u32() % 6) as usize],
        1 => sw.f32_in(-2.0, 4.0),
        _ => sw.f32_in(0.0, 1.0),
    }
}

/// Random float parameter in the range the firmware passes (0..=1, from 7-bit MIDI / menu values), with
/// edge values. Used where out-of-range values make a recursive filter diverge to infinity and then to a
/// generated NaN, whose sign is host specific (ARM default NaN 0x7fc00000, x86 0xffc00000): states the
/// firmware cannot reach.
pub fn param01(sw: &mut lxr_difftest::Sweep) -> f32 {
    match sw.next_u32() % 10 {
        0 => [0.0, 1.0, -0.0, 0.5, 1e-30, 0.999_999_94][(sw.next_u32() % 6) as usize],
        _ => sw.f32_in(0.0, 1.0),
    }
}

/// Element-wise comparison with a useful message.
pub fn assert_i16s(what: &str, ctx: &str, a: &[i16], b: &[i16]) {
    if a != b {
        let i = a.iter().zip(b).position(|(x, y)| x != y).unwrap();
        panic!("{what} differs at {i}: fw {} rust {} ({ctx})", a[i], b[i]);
    }
}
pub fn assert_bytes(what: &str, ctx: &str, fw: &[u8], rust: &[u8]) {
    if fw != rust {
        let i = fw.iter().zip(rust).position(|(x, y)| x != y).unwrap();
        panic!("{what} differs at byte {i}: fw {:02x?} rust {:02x?} ({ctx})", &fw[i & !7..(i & !7) + 8], &rust[i & !7..(i & !7) + 8]);
    }
}
