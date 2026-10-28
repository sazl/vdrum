// SPDX-License-Identifier: GPL-3.0-only
//! The plugin's audio path with the real engine (`lxr-core`): at a 44100 Hz host rate the output is
//! exactly the engine's i16 samples / 32768, for any host buffer sizes, with the host's FTZ/DAZ
//! mode on (the IEEE guard), and the plugin's event scheduling reproduces the scenario corpus'
//! reference hashes (shipped firmware v1.71 in the emulator).

use crate::engine::{Engine, LxrEngine, TICK_FRAMES};
use crate::params::{self, Shared};
use crate::pipeline::{Ev, Pipeline};
use crate::{apply_event, fp};
use sha2::{Digest, Sha256};
use std::cell::Cell;
use std::collections::BTreeMap;
use std::path::PathBuf;

/// One scenario event (`docs/ENGINE_CONTRACT.md` section 8) without kit loads.
#[derive(Clone, Debug)]
enum ScEv {
    Midi(Vec<u8>),
    Param(u16, u16),
    Fader(u8, u16),
}

struct Scenario {
    name: String,
    ticks: usize,
    /// (tick, event) in file order.
    events: Vec<(usize, ScEv)>,
}

fn scenario_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../tests/scenarios")
}

/// Reads a scenario; `None` if it loads kits (the plugin loads kits at buffer starts, outside the
/// event schedule).
fn load(path: &std::path::Path) -> Option<Scenario> {
    let v: serde_json::Value = serde_json::from_slice(&std::fs::read(path).ok()?).ok()?;
    let mut events = Vec::new();
    for group in v["events"].as_array()? {
        let tick = group["tick"].as_u64()? as usize;
        for e in group["events"].as_array()? {
            let ev = if let Some(m) = e.get("midi") {
                ScEv::Midi(m.as_array()?.iter().map(|b| b.as_u64().unwrap() as u8).collect())
            } else if let Some(p) = e.get("set_parameter") {
                ScEv::Param(p["param"].as_u64()? as u16, p["value"].as_u64()? as u16)
            } else if let Some(f) = e.get("fader") {
                ScEv::Fader(f["index"].as_u64()? as u8, f["adc"].as_u64()? as u16)
            } else {
                return None;
            };
            events.push((tick, ev));
        }
    }
    Some(Scenario { name: v["name"].as_str()?.to_string(), ticks: v["ticks"].as_u64()? as usize, events })
}

fn to_evs(e: &ScEv) -> Vec<Ev> {
    match e {
        // One byte per event: scenarios contain running status and multi-message byte runs.
        ScEv::Midi(bytes) => bytes.iter().map(|b| Ev::Midi { bytes: [*b, 0, 0], len: 1 }).collect(),
        ScEv::Param(p, v) => vec![Ev::Param { idx: *p, val: *v, learned: false }],
        ScEv::Fader(i, adc) => vec![Ev::Fader { index: *i, adc: *adc }],
    }
}

/// Runs a scenario through the plugin pipeline at 44100 Hz with the given host buffer sizes
/// (repeated), with FTZ/DAZ on as a plugin wrapper would set it. Events of scenario tick `n` are
/// placed at host sample `64 n + jitter` (0..63), which the pipeline applies before tick `n`.
/// Returns the output converted back to i16 (DAC1 then DAC2 per block, as the scenario stream).
fn run_plugin(sc: &Scenario, engine: Box<LxrEngine>, buffers: &[usize]) -> Vec<i16> {
    let shared = Shared::new();
    let midi_applied = Cell::new(false);
    let mut p = Pipeline::new(engine, 44100.0, 44100.0);
    assert_eq!(p.latency(), TICK_FRAMES as u32);
    let total = (sc.ticks + 1) * TICK_FRAMES; // + latency
    let mut timed: Vec<(u64, Ev)> = Vec::new();
    for (tick, e) in &sc.events {
        let jitter = ((*tick * 37) % 64) as u64;
        for ev in to_evs(e) {
            timed.push(((*tick * TICK_FRAMES) as u64 + jitter, ev));
        }
    }
    timed.sort_by_key(|(t, _)| *t); // stable: keeps file order within a tick
    let mut frames: Vec<[f32; 4]> = Vec::with_capacity(total);
    let mut pos = 0usize;
    let mut next_ev = 0;
    let mut k = 0;
    fp::force_flush_to_zero(true);
    while pos < total {
        let n = buffers[k % buffers.len()].min(total - pos);
        k += 1;
        let mut apply = |e: &mut LxrEngine, ev: Ev| apply_event(e, &shared, &midi_applied, ev);
        while next_ev < timed.len() && timed[next_ev].0 < (pos + n) as u64 {
            let (t, ev) = timed[next_ev];
            p.push_event((t - pos as u64) as u32, ev, &mut apply);
            next_ev += 1;
        }
        p.render(n, &mut apply, &mut |_, _| {}, &mut |_, f| frames.push(f));
        assert!(fp::flush_to_zero_enabled(), "the guard must restore the host's FTZ mode");
        pos += n;
    }
    fp::force_flush_to_zero(false);
    // Latency: the first 64 frames are silence.
    assert!(frames[..TICK_FRAMES].iter().all(|f| *f == [0.0; 4]));
    let mut out = Vec::with_capacity(sc.ticks * 256);
    for block in frames[TICK_FRAMES..].chunks(TICK_FRAMES) {
        for ch in [0, 2] {
            for f in block {
                for x in [f[ch], f[ch + 1]] {
                    let s = x * 32768.0;
                    assert!(s == s.trunc() && (-32768.0..=32767.0).contains(&s), "{x} is not an i16 / 32768");
                    out.push(s as i16);
                }
            }
        }
    }
    out
}

/// The engine directly, events applied before the tick as the reference emulator does.
fn run_direct(sc: &Scenario, mut e: lxr_core::Lxr) -> Vec<i16> {
    let mut out = Vec::with_capacity(sc.ticks * 256);
    for t in 0..sc.ticks {
        for (_, ev) in sc.events.iter().filter(|(tick, _)| *tick == t) {
            match ev {
                ScEv::Midi(b) => e.midi(b),
                ScEv::Param(p, v) => e.set_parameter(*p, *v as u8),
                ScEv::Fader(i, adc) => e.set_fader_adc(*i as usize, *adc),
            }
        }
        let b = e.tick();
        out.extend_from_slice(&b.dac1);
        out.extend_from_slice(&b.dac2);
    }
    out
}

fn hash(samples: &[i16]) -> String {
    let mut h = Sha256::new();
    for s in samples {
        h.update(s.to_le_bytes());
    }
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

fn reference() -> BTreeMap<String, String> {
    serde_json::from_slice(&std::fs::read(scenario_dir().join("reference_v1.71.json")).unwrap()).unwrap()
}

const BUFFER_PATTERNS: [&[usize]; 5] = [&[512], &[1, 7, 64, 13, 500, 3, 1, 947], &[4096], &[100, 28], &[63, 65, 2048]];

#[test]
fn plugin_path_reproduces_reference_hashes() {
    let refs = reference();
    let mut dir: Vec<_> = std::fs::read_dir(scenario_dir()).unwrap().map(|e| e.unwrap().path()).collect();
    dir.sort();
    // All scenarios by default; VDRUM_SCENARIO_STRIDE=n checks every n-th (debug builds are slow).
    let stride: usize = std::env::var("VDRUM_SCENARIO_STRIDE").ok().and_then(|s| s.parse().ok()).unwrap_or(1);
    let (mut checked, mut skipped_kit) = (0, 0);
    for (i, path) in dir.iter().filter(|p| p.extension().is_some_and(|e| e == "json")).enumerate() {
        if path.file_name().unwrap() == "reference_v1.71.json" || path.file_name().unwrap() == "params.json" || i % stride != 0 {
            continue;
        }
        let Some(sc) = load(path) else {
            skipped_kit += 1;
            continue;
        };
        if sc.name == "global_morph_no_target" {
            continue; // reads ITCM code bytes (docs/ENGINE_CONTRACT.md section 9)
        }
        let Some(want) = refs.get(&sc.name) else { continue };
        let got = hash(&run_plugin(&sc, LxrEngine::new(), BUFFER_PATTERNS[i % BUFFER_PATTERNS.len()]));
        assert_eq!(&got, want, "{}: plugin output differs from the reference", sc.name);
        checked += 1;
    }
    eprintln!("{checked} scenarios match the reference; {skipped_kit} skipped (kit loads)");
    assert!(checked > 0);
}

/// A sequence with all voices and the FX bus, with the plugin's MIDI defaults: identical to the
/// engine output for every buffer pattern.
#[test]
fn plugin_defaults_output_equals_engine_output_at_any_buffer_size() {
    use lxr_params::par;
    let mut events = vec![
        (0, ScEv::Param(par::FX_TYPE, 2)),
        (0, ScEv::Param(par::FX_DRY_WET, 90)),
        (0, ScEv::Param(par::AUDIO_OUT1, 6)),
        (0, ScEv::Param(par::AUDIO_OUT1 + 3, 6)),
        (0, ScEv::Param(par::AUDIO_OUT1 + 5, 1)),
        (0, ScEv::Fader(2, params::fader_adc(60))),
    ];
    for (t, note) in params::DEFAULT_TRACK_NOTES.iter().enumerate() {
        events.push((1 + 9 * t, ScEv::Midi(vec![0x90, *note, 100])));
        events.push((70 + 3 * t, ScEv::Midi(vec![0x90, *note, 60 + t as u8])));
    }
    events.push((40, ScEv::Midi(vec![0xb0, 9, 90]))); // CC 9: Drum 1 coarse
    events.push((41, ScEv::Midi(vec![0xb0, 99, 0, 98, 10, 6, 77]))); // NRPN
    events.push((90, ScEv::Param(par::COARSE1, 10)));
    events.push((100, ScEv::Fader(2, params::fader_adc(0))));
    events.sort_by_key(|(t, _)| *t);
    let sc = Scenario { name: "defaults".into(), ticks: 160, events };

    let mut direct = lxr_core::Lxr::new();
    for (i, n) in params::DEFAULT_TRACK_NOTES.iter().enumerate() {
        direct.set_parameter(par::MIDI_NOTE1 + i as u16, *n);
    }
    let want = run_direct(&sc, direct);
    // Every voice was triggered on its own note.
    assert!(want.iter().any(|s| *s != 0));
    for pattern in BUFFER_PATTERNS {
        let got = run_plugin(&sc, crate::new_engine(), pattern);
        assert_eq!(got.len(), want.len());
        assert!(got == want, "buffer pattern {pattern:?}: output differs from the engine");
    }
}

#[test]
fn default_notes_trigger_one_voice_each() {
    use nih_plug::prelude::Param;
    let e = crate::new_engine();
    for (t, note) in params::DEFAULT_TRACK_NOTES.iter().enumerate() {
        let mask = crate::engine::tracks_for_note(0, *note, |p| e.get_parameter(p));
        assert_eq!(mask, 1 << t, "note {note}");
        assert_eq!(crate::engine::track_trigger_note(t, |p| e.get_parameter(p)), (0, *note));
    }
    // The firmware's note 48 no longer triggers anything.
    assert_eq!(crate::engine::tracks_for_note(0, 48, |p| e.get_parameter(p)), 0);
    // Host parameter defaults agree with the engine.
    let p = params::VDrumParams::new(std::sync::Arc::new(Shared::new()));
    for (slot, info) in p.infos.iter().enumerate() {
        assert_eq!(p.ints[slot].default_plain_value() as u16, e.get_parameter(info.index), "{}", info.id);
    }
}

#[test]
fn save_kit_roundtrips_through_load_kit() {
    let mut e = LxrEngine::new();
    e.set_parameter(lxr_params::par::COARSE1, 77);
    let k = e.save_kit();
    assert_eq!(k.len(), 255);
    let mut e2 = LxrEngine::new();
    e2.load_kit(&k);
    assert_eq!(e2.get_parameter(lxr_params::par::COARSE1), 77);
    assert_eq!(e2.save_kit(), k);
    // Out-of-range calls are ignored instead of panicking.
    e2.set_parameter(9999, 1);
    assert_eq!(e2.get_parameter(9999), 0);
    e2.load_kit(&[0; 3]);
}
