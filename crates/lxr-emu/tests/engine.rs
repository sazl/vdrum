//! Whole-engine tests. They need the firmware files, which are not in the repository:
//!   LXR_FIRMWARE_IMG = path of LXRV2_update_v1.71.img (shipped SD-card update image)
//!   LXR_FW_ELF       = path of a firmware ELF with symbols (GCC 11.3 reference build)
//! Without them the tests print a message and pass.
use lxr_emu::scenario::{self, Scenario};
use lxr_emu::{symbols, AudioContext, EmuEngine};
use std::collections::BTreeMap;
use std::path::PathBuf;

fn env_path(var: &str) -> Option<PathBuf> {
    match std::env::var_os(var) {
        Some(p) if !p.is_empty() => Some(PathBuf::from(p)),
        _ => {
            eprintln!("SKIPPED: set {var} to run this test");
            None
        }
    }
}

fn shipped() -> Option<EmuEngine> {
    env_path("LXR_FIRMWARE_IMG").map(|p| EmuEngine::new_shipped(p).expect("boot shipped image"))
}

fn elf() -> Option<EmuEngine> {
    env_path("LXR_FW_ELF").map(|p| EmuEngine::new_elf(p).expect("boot ELF"))
}

fn scenarios_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/scenarios")
}

fn reference() -> BTreeMap<String, String> {
    let s = std::fs::read_to_string(scenarios_dir().join("reference_v1.71.json")).unwrap();
    serde_json::from_str(&s).unwrap()
}

/// Short scenarios covering note triggering, parameters, LFO targets, FX and kits.
const SAMPLE: &[&str] = &[
    "default_boot_note48",
    "default_boot_silence",
    "param_004_osc_wave_snare",
    "fx_type1_route2",
    "lfo1_all_targets",
    "velmod3_all_targets",
    "kit_random_0",
    "midi_running_status_nrpn",
];

#[test]
fn symbol_map_parses() {
    let m = symbols::parse_symbol_map(symbols::V171_SYMBOLS_JSON).unwrap();
    for n in [
        "main",
        "Reset_Handler",
        "soundengine_calcNextSampleBlock",
        "activeParameters",
        "dma_buffer2",
    ] {
        assert!(m.contains_key(n), "{n} missing from symbol map");
    }
}

fn check_boot_state(e: &EmuEngine) {
    // no SD card: default globals, MIDI channel 1, note 48 for every voice
    let mc = e.addr("midi_MidiChannels").unwrap();
    assert_eq!(e.fw.read(mc, 7).unwrap(), vec![0; 7]);
    for p in 294..=300 {
        assert_eq!(e.active_parameter(p).unwrap(), 48, "PAR_MIDI_NOTE {p}");
    }
    assert_eq!(e.fw.read_u8(e.addr("morph_state").unwrap()).unwrap(), 0);
    // faders at the top: adc_faderValues = (4095/4096)^2 computed by adc_checkFaders
    assert_eq!(
        e.fader_values().unwrap().map(f32::to_bits),
        [lxr_emu::engine::FADER_FULL_VALUE.to_bits(); 6]
    );
    assert_eq!(e.rng_calls(), 0, "GetRngValue is not called during init");
}

#[test]
fn boots_to_expected_state() {
    if let Some(e) = shipped() {
        check_boot_state(&e);
    }
    if let Some(e) = elf() {
        check_boot_state(&e);
    }
}

fn render_named(e: &mut EmuEngine, name: &str) -> String {
    let sc = Scenario::load(scenarios_dir().join(format!("{name}.json"))).unwrap();
    scenario::render(e, &sc).unwrap().1
}

#[test]
fn note_produces_audio_on_dac1() {
    let Some(mut e) = shipped() else { return };
    let mut peak = 0i32;
    for t in 0..200 {
        if t == 1 {
            e.midi(&[0x90, 48, 127]).unwrap();
        }
        let b = e.tick().unwrap();
        peak = peak.max(b.dac1.iter().map(|&x| (x as i32).abs()).max().unwrap());
        assert!(
            b.dac2.iter().all(|&x| x == 0),
            "nothing is routed to DAC2 by default"
        );
    }
    assert!(peak > 1000, "peak {peak}");
}

#[test]
fn shipped_matches_reference_hashes() {
    let Some(_) = env_path("LXR_FIRMWARE_IMG") else {
        return;
    };
    let want = reference();
    for name in SAMPLE {
        let mut e = shipped().unwrap();
        assert_eq!(&render_named(&mut e, name), &want[*name], "{name}");
    }
}

#[test]
fn rendering_is_deterministic() {
    let Some(mut a) = shipped() else { return };
    let mut b = shipped().unwrap();
    assert_eq!(
        render_named(&mut a, "fx_type1_route2"),
        render_named(&mut b, "fx_type1_route2")
    );
}

#[test]
fn elf_matches_shipped_reference() {
    let Some(_) = env_path("LXR_FW_ELF") else {
        return;
    };
    let want = reference();
    for name in SAMPLE {
        let mut e = elf().unwrap();
        assert_eq!(&render_named(&mut e, name), &want[*name], "{name}");
    }
}

#[test]
fn audio_context_switch() {
    let Some(mut e) = shipped() else { return };
    assert_eq!(e.audio_fpscr, lxr_emu::engine::FPSCR_MAIN);
    e.set_audio_context(AudioContext::Tim2Interrupt);
    assert_eq!(e.audio_fpscr, lxr_emu::engine::FPSCR_IRQ);
}
