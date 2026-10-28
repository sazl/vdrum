//! P1 differential tests: lfo, modulationNode / automationNode arithmetic and message construction.
mod p1_common;

use lxr_core::dsp::automation_node::*;
use lxr_core::dsp::lfo::*;
use lxr_core::dsp::modulation_node::*;
use lxr_core::rng::xorshift32;
use lxr_core::Lxr;
use lxr_difftest::{fw_or_skip, Firmware, Sweep};
use p1_common::*;
use std::cell::{Cell, RefCell};
use std::rc::Rc;
use unicorn_engine::RegisterARM;

/// lfo offsets inside the voice structs (from the disassembly of lfo_retrigger / lfo_recalcSync).
const LFO_IN_DRUMVOICE: u32 = 152;
const DRUMVOICE_SIZE: u32 = 704;
const LFO_IN_SNARE: u32 = 164;
const LFO_IN_CYMBAL: u32 = 188;
const LFO_IN_HAT: u32 = 308;

fn w_mod(fw: &mut Firmware, a: u32, m: &ModulationNode) {
    w_u16(fw, a, m.destination);
    w_u8(fw, a + 2, m.r#type);
    w_u32(fw, a + 4, m.originalValue.itg);
    w_f32(fw, a + 8, m.amount);
    w_f32(fw, a + 12, m.lastVal);
}
fn r_mod(fw: &Firmware, a: u32) -> ModulationNode {
    let mut m = ModulationNode::ZERO;
    m.destination = r_u16(fw, a);
    m.r#type = r_u8(fw, a + 2);
    m.originalValue.itg = r_u32(fw, a + 4);
    m.amount = r_f32(fw, a + 8);
    m.lastVal = r_f32(fw, a + 12);
    m
}
fn w_lfo(fw: &mut Firmware, a: u32, l: &Lfo) {
    w_u32(fw, a, l.phase);
    w_u32(fw, a + 4, l.phaseInc);
    w_u8(fw, a + 8, l.waveform);
    w_u8(fw, a + 9, l.retrigger);
    w_u32(fw, a + 12, l.phaseOffset);
    w_f32(fw, a + 16, l.rnd);
    w_u8(fw, a + 20, l.sync);
    w_f32(fw, a + 24, l.freq);
    w_mod(fw, a + 28, &l.modTarget);
    w_f32(fw, a + 44, l.modNodeValue);
}
fn r_lfo(fw: &Firmware, a: u32) -> Lfo {
    Lfo {
        phase: r_u32(fw, a),
        phaseInc: r_u32(fw, a + 4),
        waveform: r_u8(fw, a + 8),
        retrigger: r_u8(fw, a + 9),
        phaseOffset: r_u32(fw, a + 12),
        rnd: r_f32(fw, a + 16),
        sync: r_u8(fw, a + 20),
        freq: r_f32(fw, a + 24),
        modTarget: r_mod(fw, a + 28),
        modNodeValue: r_f32(fw, a + 44),
    }
}
#[track_caller]
fn eq_lfo(ctx: &str, f: &Lfo, r: &Lfo) {
    assert_eq!(
        (f.phase, f.phaseInc, f.waveform, f.retrigger, f.phaseOffset, f.sync),
        (r.phase, r.phaseInc, r.waveform, r.retrigger, r.phaseOffset, r.sync),
        "{ctx}"
    );
    eq32(&format!("{ctx} rnd"), f.rnd, r.rnd);
    eq32(&format!("{ctx} freq"), f.freq, r.freq);
    eq32(&format!("{ctx} modNodeValue"), f.modNodeValue, r.modNodeValue);
    let (fm, rm) = (&f.modTarget, &r.modTarget);
    assert_eq!((fm.destination, fm.r#type, fm.originalValue.itg), (rm.destination, rm.r#type, rm.originalValue.itg), "{ctx} modTarget");
    eq32(&format!("{ctx} amount"), fm.amount, rm.amount);
    eq32(&format!("{ctx} lastVal"), fm.lastVal, rm.lastVal);
}

fn hook_rng(fw: &mut Firmware) -> Rc<Cell<u32>> {
    let st = Rc::new(Cell::new(0x2545_F491u32));
    let s2 = st.clone();
    let addr = fw.addr("GetRngValue").unwrap();
    fw.intercept(addr, move |uc| {
        let mut x = s2.get();
        let v = xorshift32(&mut x);
        s2.set(x);
        uc.reg_write(RegisterARM::R0, v as u64).unwrap();
    })
    .unwrap();
    st
}

/// Recorded call of midiParser_ccHandler(MidiMsg msg, uint8_t updateOriginalValue, uint8_t isFloatModulator, float v):
/// (status, data1, data2, r2, r3, s0 bits).
type CcCall = (u8, u8, u8, u32, u32, u32);

fn hook_cc(fw: &mut Firmware) -> Rc<RefCell<Vec<CcCall>>> {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let c2 = calls.clone();
    let addr = fw.addr("midiParser_ccHandler").unwrap();
    fw.intercept(addr, move |uc| {
        let r0 = uc.reg_read(RegisterARM::R0).unwrap() as u32;
        let r2 = uc.reg_read(RegisterARM::R2).unwrap() as u32;
        let r3 = uc.reg_read(RegisterARM::R3).unwrap() as u32;
        let s0 = uc.reg_read(RegisterARM::S0).unwrap() as u32;
        c2.borrow_mut().push((r0 as u8, (r0 >> 16) as u8, (r0 >> 24) as u8, r2 & 0xff, r3 & 0xff, s0));
    })
    .unwrap();
    calls
}

fn random_lfo(sw: &mut Sweep) -> Lfo {
    let mut m = ModulationNode::ZERO;
    m.destination = (sw.next_u32() % 247) as u16;
    m.amount = sw.f32_in(-1.0, 1.0);
    m.lastVal = sw.f32_in(0.0, 1.0);
    m.originalValue.itg = sw.next_u32();
    Lfo {
        phase: sw.next_u32(),
        phaseInc: match sw.next_u32() % 3 {
            0 => sw.next_u32(),
            1 => sw.next_u32() >> 12,
            _ => 0,
        },
        waveform: (sw.next_u32() % 10) as u8,
        retrigger: (sw.next_u32() % 8) as u8,
        phaseOffset: sw.next_u32(),
        rnd: sw.f32_in(0.0, 1.0),
        sync: (sw.next_u32() % 14) as u8,
        freq: sw.f32_in(0.0, 200.0),
        modTarget: m,
        modNodeValue: match sw.next_u32() % 4 {
            0 => 1.0,
            1 => f32_special(sw, -2.0, 2.0),
            _ => sw.f32_in(0.0, 2.0),
        },
    }
}

#[test]
fn lfo_calc_and_dispatch() {
    let mut fw = fw_or_skip!();
    fw.fpscr = 0;
    let st = hook_rng(&mut fw);
    let mut lxr = Box::new(Lxr::new_zeroed());
    let a = fw.alloc(48).unwrap();
    let parr = fw.addr("parameterArray").unwrap();
    // parameterArray entries with ptr == NULL: modNode_updateValue only stores lastVal
    fw.write(parr, &vec![0u8; 247 * 8]).unwrap();
    let mut sw = Sweep(0x1f0_ca1c);
    let mut rs = random_lfo(&mut sw);
    for it in 0..40000 {
        if it % 50 == 0 {
            rs = random_lfo(&mut sw);
        }
        w_lfo(&mut fw, a, &rs);
        lxr.rng_state = st.get();
        if it % 2 == 0 {
            let (_, f) = fw.call("lfo_calc", &[a], &[]).unwrap();
            let r = lxr.lfo_calc(&mut rs);
            eq32(&format!("lfo_calc it {it} wf {}", rs.waveform), f, r);
        } else {
            // lfo_dispatchNextValue on an Lfo inside Lxr (place accessor)
            lxr.voiceArray[0].lfo = rs;
            assert_eq!(lxr.parameterArray[rs.modTarget.destination as usize].ptr, 0);
            fw.call("lfo_dispatchNextValue", &[a], &[]).unwrap();
            lxr.lfo_dispatchNextValue(|s| &mut s.voiceArray[0].lfo);
            rs = lxr.voiceArray[0].lfo;
        }
        eq_lfo(&format!("it {it}"), &r_lfo(&fw, a), &rs);
        assert_eq!(st.get(), lxr.rng_state, "rng it {it}");
    }
}

#[test]
fn lfo_phase_inc() {
    let mut fw = fw_or_skip!();
    fw.fpscr = 0;
    let mut lxr = Box::new(Lxr::new_zeroed());
    let a = fw.alloc(48).unwrap();
    let tempo = fw.addr("seq_tempo").unwrap();
    let mut sw = Sweep(0xb9_4c);
    for it in 0..30000 {
        let bpm: u16 = match sw.next_u32() % 4 {
            0 => 0,
            1 => sw.next_u32() as u16,
            _ => 20 + (sw.next_u32() % 400) as u16,
        };
        w_u16(&mut fw, tempo, bpm);
        lxr.seq_tempo = bpm;
        let sync = match sw.next_u32() % 3 {
            0 => sw.u8(),
            _ => (sw.next_u32() % 13) as u8,
        };
        let f = match sw.next_u32() % 3 {
            0 => f32_special(&mut sw, -1.0, 2.0),
            1 => (sw.u8() % 128) as f32 / 127.0,
            _ => sw.f32_in(0.0, 1.0),
        };
        let mut rs = random_lfo(&mut sw);
        w_lfo(&mut fw, a, &rs);
        let name = match it % 3 {
            0 => {
                let (r0, _) = fw.call("lfo_calcPhaseInc", &[sync as u32], &[f * 200.0]).unwrap();
                assert_eq!(r0, lxr.lfo_calcPhaseInc(f * 200.0, sync), "lfo_calcPhaseInc({}, {sync}) bpm {bpm}", f * 200.0);
                "lfo_calcPhaseInc"
            }
            1 => {
                fw.call("lfo_setFreq", &[a], &[f]).unwrap();
                lxr.lfo_setFreq(&mut rs, f);
                "lfo_setFreq"
            }
            _ => {
                fw.call("lfo_setSync", &[a, sync as u32], &[]).unwrap();
                lxr.lfo_setSync(&mut rs, sync);
                "lfo_setSync"
            }
        };
        eq_lfo(&format!("{name} it {it} bpm {bpm} sync {sync} f {f:e}"), &r_lfo(&fw, a), &rs);
    }
}

#[test]
fn lfo_globals_retrigger_recalc_sync() {
    let mut fw = fw_or_skip!();
    fw.fpscr = 0;
    let mut lxr = Box::new(Lxr::new_zeroed());
    let tempo = fw.addr("seq_tempo").unwrap();
    let addrs = [
        fw.addr("voiceArray").unwrap() + LFO_IN_DRUMVOICE,
        fw.addr("voiceArray").unwrap() + LFO_IN_DRUMVOICE + DRUMVOICE_SIZE,
        fw.addr("voiceArray").unwrap() + LFO_IN_DRUMVOICE + 2 * DRUMVOICE_SIZE,
        fw.addr("snareVoice").unwrap() + LFO_IN_SNARE,
        fw.addr("cymbalVoice").unwrap() + LFO_IN_CYMBAL,
        fw.addr("hatVoice").unwrap() + LFO_IN_HAT,
    ];
    fn get(l: &mut Lxr, i: usize) -> &mut Lfo {
        match i {
            0..=2 => &mut l.voiceArray[i].lfo,
            3 => &mut l.snareVoice.lfo,
            4 => &mut l.cymbalVoice.lfo,
            _ => &mut l.hatVoice.lfo,
        }
    }
    let mut sw = Sweep(0x2e7_9196);
    for it in 0..5000 {
        let bpm = 20 + (sw.next_u32() % 400) as u16;
        w_u16(&mut fw, tempo, bpm);
        lxr.seq_tempo = bpm;
        for i in 0..6 {
            let l = random_lfo(&mut sw);
            w_lfo(&mut fw, addrs[i], &l);
            *get(&mut lxr, i) = l;
        }
        if it % 2 == 0 {
            let v = (sw.next_u32() % 9) as u8;
            fw.call("lfo_retrigger", &[v as u32], &[]).unwrap();
            lxr.lfo_retrigger(v);
        } else {
            fw.call("lfo_recalcSync", &[], &[]).unwrap();
            lxr.lfo_recalcSync();
        }
        for i in 0..6 {
            eq_lfo(&format!("it {it} lfo {i}"), &r_lfo(&fw, addrs[i]), get(&mut lxr, i));
        }
    }
}

#[test]
fn mod_node_update_value_messages() {
    let mut fw = fw_or_skip!();
    fw.fpscr = 0;
    let calls = hook_cc(&mut fw);
    let a = fw.alloc(16).unwrap();
    let parr = fw.addr("parameterArray").unwrap();
    let act = fw.addr("activeParameters").unwrap();
    let vmods = fw.addr("velocityModulators").unwrap();
    let mut sw = Sweep(0x30d_e0de);
    for it in 0..20000 {
        // parameterArray: random NULL / non-NULL pointers; activeParameters random
        let ptrs: Vec<u32> = (0..247).map(|_| if sw.next_u32() % 4 == 0 { 0 } else { 0x2000_0000 }).collect();
        for (i, p) in ptrs.iter().enumerate() {
            w_u32(&mut fw, parr + 8 * i as u32, *p);
        }
        let actv: Vec<u8> = (0..302).map(|_| sw.u8()).collect();
        fw.write(act, &actv).unwrap();
        calls.borrow_mut().clear();
        let mk = |sw: &mut Sweep| {
            let mut m = ModulationNode::ZERO;
            m.destination = (sw.next_u32() % 247) as u16;
            m.amount = match sw.next_u32() % 4 {
                0 => f32_special(sw, -2.0, 2.0),
                _ => sw.f32_in(-1.0, 1.0),
            };
            m.lastVal = match sw.next_u32() % 4 {
                0 => f32_special(sw, -2.0, 2.0),
                1 => f32::NAN,
                _ => sw.f32_in(0.0, 1.0),
            };
            m
        };
        let mut expected: Vec<CcCall> = Vec::new();
        if it % 2 == 0 {
            let m = mk(&mut sw);
            let val = m.lastVal;
            let mut m0 = m;
            m0.lastVal = 0.5;
            w_mod(&mut fw, a, &m0);
            fw.call("modNode_updateValue", &[a], &[val]).unwrap();
            eq32("lastVal", r_f32(&fw, a + 12), val);
            if ptrs[m.destination as usize] != 0 {
                let msg = modNode_ccMsg(m.destination);
                let v = modNode_calcNewValue(m.amount, val, actv[m.destination as usize]);
                expected.push((msg.status, msg.data1, 0, 0, 1, v.to_bits()));
            }
        } else {
            for i in 0..6 {
                let m = mk(&mut sw);
                w_mod(&mut fw, vmods + 16 * i, &m);
                if ptrs[m.destination as usize] != 0 {
                    let msg = modNode_ccMsg(m.destination);
                    let v = modNode_calcNewValue(m.amount, m.lastVal, actv[m.destination as usize]);
                    expected.push((msg.status, msg.data1, 0, 0, 1, v.to_bits()));
                }
            }
            fw.call("modNode_reassignVeloMod", &[], &[]).unwrap();
        }
        let got: Vec<CcCall> = calls.borrow().iter().map(|c| (c.0, c.1, 0, c.3, c.4, c.5)).collect();
        assert_eq!(got, expected, "it {it}");
    }
}

#[test]
fn auto_node_messages() {
    let mut fw = fw_or_skip!();
    fw.fpscr = 0;
    let calls = hook_cc(&mut fw);
    let a = fw.alloc(4).unwrap();
    let orig = fw.addr("midiParser_originalCcValues").unwrap();
    let mut sw = Sweep(0xa070);
    let mut rs = AutomationNode::ZERO;
    w_u16(&mut fw, a, 7);
    rs.destination = 7;
    fw.call("autoNode_init", &[a], &[]).unwrap();
    autoNode_init(&mut rs);
    assert_eq!(r_u16(&fw, a), rs.destination);
    for it in 0..20000 {
        let ov: Vec<u8> = (0..255).map(|_| sw.u8()).collect();
        fw.write(orig, &ov).unwrap();
        let dest: u16 = match sw.next_u32() % 4 {
            0 => 255,
            _ => (sw.next_u32() % 255) as u16,
        };
        let cur: u16 = match sw.next_u32() % 4 {
            0 => 255,
            _ => (sw.next_u32() % 255) as u16,
        };
        w_u16(&mut fw, a, cur);
        calls.borrow_mut().clear();
        let mut expected: Vec<CcCall> = Vec::new();
        if it % 2 == 0 {
            fw.call("autoNode_setDestination", &[a, dest as u32], &[]).unwrap();
            assert_eq!(r_u16(&fw, a), dest);
            if cur != 255 {
                let m = autoNode_ccMsg(cur);
                expected.push((m.status, m.data1, ov[cur as usize], 0, 0, 0));
            }
        } else {
            let val = sw.u8();
            fw.call("autoNode_updateValue", &[a, val as u32], &[]).unwrap();
            if cur != 255 {
                let m = autoNode_ccMsg(cur);
                expected.push((m.status, m.data1, val, 0, 0, 0));
            }
        }
        assert_eq!(*calls.borrow(), expected, "it {it}");
    }
}
