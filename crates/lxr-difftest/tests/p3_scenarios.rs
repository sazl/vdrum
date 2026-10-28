//! P3 scenario corpus: every scenario in `tests/scenarios` is rendered by the Rust engine (`Lxr::new`, the
//! scenario events through `midi` / `set_parameter` / `load_kit` / `set_fader_adc`, `Lxr::tick`), and
//!
//! - its SHA-256 is compared with the committed shipped-v1.71 reference hashes
//!   (`tests/scenarios/reference_v1.71.json`);
//! - it runs side by side with the emulated firmware (`EmuEngine`: the shipped v1.71 image when available,
//!   and the reference ELF from `LXR_FW_ELF`): every block of output is compared, and the whole engine
//!   state (all globals) after every event, every `LXR_P3_STATE_EVERY` blocks (default 32) and at the end.
//!
//! One scenario cannot be bit-exact: `global_morph_no_target` (see `NOT_EXACT`).
//! Everything runs with FZ = 0 (`docs/ENGINE_CONTRACT.md`).
#![allow(non_snake_case)]

mod p3_common;

use lxr_emu::scenario::Scenario;
use p3_common::*;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

/// Scenarios whose output cannot be reproduced exactly, with the reason.
///
/// `global_morph_no_target`: morphing towards the ITCM code bytes (NULL `morphParameters`) sets oscillator
/// waveforms out of range (e.g. 66, 108, 117, 118). `calcNextOscSampleBlock` then takes the `default:` case,
/// which leaves the caller's buffer untouched, and the callers (`Cymbal_calcSyncBlock`, the drum voices)
/// pass uninitialised stack arrays (`int16_t mod[size]`). The firmware output then depends on stale stack
/// contents of earlier calls; the port (P4) uses zeroed buffers. The engine state is identical up to the
/// first block that renders such an oscillator (checked by `morph_no_target_state_until_render`). This
/// is also the one scenario whose reference-ELF output differs from the shipped one.
const NOT_EXACT: &[&str] = &["global_morph_no_target"];

fn reference_hashes() -> BTreeMap<String, String> {
    let s = std::fs::read_to_string(scenario_dir().join("reference_v1.71.json")).unwrap();
    serde_json::from_str(&s).unwrap()
}

fn hash_lxr(sc: &Scenario) -> String {
    let mut l = Box::new(lxr_core::prelude::Lxr::new());
    let sched = sc.schedule();
    let mut out = Vec::with_capacity(sc.ticks as usize * 512);
    let mut i = 0;
    for t in 0..sc.ticks {
        while i < sched.len() && sched[i].0 == t {
            apply(&mut l, sched[i].1);
            i += 1;
        }
        let b = l.tick();
        for s in b.dac1.iter().chain(b.dac2.iter()) {
            out.extend_from_slice(&s.to_le_bytes());
        }
    }
    lxr_emu::engine::hex(&Sha256::digest(&out))
}

/// All scenarios against the committed shipped-v1.71 hashes (no firmware needed).
#[test]
fn scenarios_match_reference_hashes() {
    on_big_stack(|| {
        let refs = reference_hashes();
        let scs = scenarios();
        let mut bad = Vec::new();
        let mut checked = 0;
        for sc in &scs {
            let want = refs.get(&sc.name).unwrap_or_else(|| panic!("no reference hash for {}", sc.name));
            let got = hash_lxr(sc);
            if NOT_EXACT.contains(&sc.name.as_str()) {
                eprintln!("{}: not exact by design (got {got}, reference {want})", sc.name);
                continue;
            }
            checked += 1;
            if &got != want {
                eprintln!("MISMATCH {}: {} (reference {})", sc.name, got, want);
                bad.push(sc.name.clone());
            }
        }
        eprintln!("{checked} scenarios checked against reference_v1.71.json, {} mismatches", bad.len());
        assert!(bad.is_empty(), "scenarios differing from reference_v1.71.json: {bad:?}");
    });
}

/// Runs `sc` on `Lxr` and the emulator; compares output every block and the state after every event and
/// every `state_every` blocks. Returns the first difference.
/// `Ok(true)`: the state comparison with the reference ELF stopped at a pointer-layout divergence
/// (`Pair::elf_layout_diverged`); the output was compared to the end.
fn run_pair(sc: &Scenario, mut p: Pair, state_every: u64) -> Result<bool, String> {
    let sched = sc.schedule();
    let mut i = 0;
    for t in 0..sc.ticks {
        while i < sched.len() && sched[i].0 == t {
            p.event(sched[i].1);
            let d = p.diff();
            if !d.is_empty() {
                return Err(format!("tick {t}, after event {:?}: state differs: {d:?}", sched[i].1));
            }
            i += 1;
        }
        let a = p.l.tick();
        let b = p.emu.tick().unwrap();
        if a.dac1 != b.dac1 || a.dac2 != b.dac2 {
            return Err(format!("block {t}: output differs"));
        }
        if state_every != 0 && (t + 1) % state_every == 0 {
            let d = p.diff();
            if !d.is_empty() {
                return Err(format!("after block {t}: state differs: {d:?}"));
            }
        }
    }
    let d = p.diff();
    if !d.is_empty() {
        return Err(format!("at the end: state differs: {d:?}"));
    }
    if p.l.rng_state != p.emu.rng_state() {
        return Err("RNG state differs".into());
    }
    Ok(p.elf_layout_diverged())
}

/// All scenarios side by side with each available firmware build (output and full engine state).
#[test]
fn scenarios_match_emulator() {
    let _fw = lxr_difftest::fw_or_skip!();
    let state_every: u64 = std::env::var("LXR_P3_STATE_EVERY").ok().map(|s| s.parse().unwrap()).unwrap_or(32);
    on_big_stack(move || {
        let scs = scenarios();
        for (build, make) in builds() {
            let mut bad = Vec::new();
            let mut output_only = Vec::new();
            let mut blocks = 0;
            let probe = make();
            eprintln!("{build}: comparing {} of {} globals", compared_globals(&probe), lxr_core::cmem::GLOBALS.len());
            drop(probe);
            for sc in &scs {
                if NOT_EXACT.contains(&sc.name.as_str()) {
                    continue;
                }
                let p = Pair::new(make());
                // power-on state
                let d = p.diff();
                assert!(d.is_empty(), "{build}: state after power-on differs: {d:?}");
                match run_pair(sc, p, state_every) {
                    Err(e) => {
                        eprintln!("{build} {}: {e}", sc.name);
                        bad.push(sc.name.clone());
                    }
                    Ok(true) => output_only.push(sc.name.clone()),
                    Ok(false) => {}
                }
                blocks += sc.ticks;
            }
            eprintln!("{build}: {} scenarios, {blocks} blocks, {} differ", scs.len(), bad.len());
            if !output_only.is_empty() {
                eprintln!("{build}: state compared until the parameterArray pointer layouts diverged in {output_only:?}");
            }
            assert!(bad.is_empty(), "{build}: scenarios differing from the emulator: {bad:?}");
        }
    });
}

/// `global_morph_no_target`: the state matches the shipped firmware through the morph itself (the kit
/// transfer towards the ITCM code bytes) and up to the first block that renders an out-of-range waveform.
#[test]
fn morph_no_target_state_until_render() {
    let _fw = lxr_difftest::fw_or_skip!();
    let Some(img) = img_path() else {
        eprintln!("shipped image not available; skipping");
        return;
    };
    on_big_stack(move || {
        let sc = Scenario::load(scenario_dir().join("global_morph_no_target.json")).unwrap();
        let mut p = Pair::new(lxr_emu::EmuEngine::new_shipped(&img).unwrap());
        let sched = sc.schedule();
        let mut i = 0;
        let morph_tick = sched.iter().find(|(_, e)| matches!(e, lxr_emu::scenario::Event::SetParameter { param: 248, value: v } if *v != 0)).unwrap().0;
        for t in 0..=morph_tick {
            while i < sched.len() && sched[i].0 == t {
                p.event(sched[i].1);
                p.check(&format!("tick {t} after {:?}", sched[i].1));
                i += 1;
            }
            if t < morph_tick {
                p.tick(&format!("block {t}"));
                p.check(&format!("after block {t}"));
            }
        }
        // the morph has changed the waveforms to values the oscillators do not handle
        let l = &p.l;
        let wf = [l.cymbalVoice.modOsc.waveform, l.voiceArray[0].modOsc.waveform];
        assert!(wf.iter().any(|&w| w > 5), "expected out-of-range waveforms after the morph, got {wf:?}");
    });
}
