//! Shared helpers for the P3 (control path) differential tests: the Rust engine (`Lxr::new` and its glue)
//! runs side by side with the emulated firmware (`lxr_emu::EmuEngine`, same power-on sequence, FZ = 0), and
//! the whole engine state (every global of `lxr_core::cmem::GLOBALS`, byte for byte in C layout) and the
//! audio output are compared.
//!
//! Firmware builds: the reference ELF (`LXR_FW_ELF`) and, when it is available, the shipped v1.71 update
//! image (`LXR_FIRMWARE_IMG`, default `/home/user/lxr-ref/LXRV2_update_v1.71.img`). For the shipped image the
//! globals are found through `lxr_core::midi::shipped_layout` (from the recovered symbol map); RAM
//! pointers stored in globals are translated between the two layouts before comparing.
#![allow(dead_code, non_snake_case)]

use lxr_core::cmem::GLOBALS;
use lxr_core::prelude::*;
use lxr_emu::scenario::{decode_hex, Event, Scenario};
use lxr_emu::EmuEngine;
use std::path::PathBuf;

pub fn elf_path() -> Option<String> {
    std::env::var("LXR_FW_ELF").ok()
}

pub fn img_path() -> Option<String> {
    let p = std::env::var("LXR_FIRMWARE_IMG").unwrap_or_else(|_| "/home/user/lxr-ref/LXRV2_update_v1.71.img".into());
    std::path::Path::new(&p).exists().then_some(p)
}

/// The emulated builds to compare against: (name, constructor).
pub fn builds() -> Vec<(&'static str, Box<dyn Fn() -> EmuEngine + Send + Sync>)> {
    let mut v: Vec<(&'static str, Box<dyn Fn() -> EmuEngine + Send + Sync>)> = Vec::new();
    if let Some(p) = img_path() {
        v.push(("shipped v1.71", Box::new(move || EmuEngine::new_shipped(&p).unwrap())));
    }
    if let Some(p) = elf_path() {
        v.push(("reference ELF", Box::new(move || EmuEngine::new_elf(&p).unwrap())));
    }
    v
}

/// Globals whose value legitimately differs between `Lxr` (which follows the shipped v1.71 binary) and the
/// reference ELF build: none of them is read by code that the shipped binary runs.
/// - `lastWasNoteOn`: written by the C `seq_processFlam` noteOff branch and by `seq_sendMidiNoteOn`,
///   neither of which exists in the shipped binary.
/// - `seq_sendMidiNoteOn_msg`: the MIDI output message of `seq_sendMidiNoteOn` (MIDI out only), which the
///   shipped `voiceControl_noteOn` does not call.
pub const ELF_ONLY_GLOBALS: &[&str] = &["lastWasNoteOn", "seq_sendMidiNoteOn_msg"];

/// Single bytes whose value depends on the RAM layout of the build: MIDI CC 0 stores to
/// `midiParser_originalCcValues[65536]`, which is `seq_patternSet + 0xe4e8` in the shipped image (what `Lxr`
/// reproduces, `midi_parser::CC0_OOB_PATTERNSET_OFFSET`) and `seq_patternSet + 0xe4f0` in the reference ELF.
/// Both are `param1Nr` bytes of steps of pattern 18; ignored when comparing with the ELF.
pub const ELF_LAYOUT_BYTES: &[(&str, u32)] = &[("seq_patternSet", 0xe4e8), ("seq_patternSet", 0xe4f0)];

/// Runs `f` on a thread with a large stack (`Lxr` is several hundred KB).
pub fn on_big_stack<F: FnOnce() + Send + 'static>(f: F) {
    std::thread::Builder::new().stack_size(64 << 20).spawn(f).unwrap().join().unwrap();
}

/// Globals (by name) whose bytes differ between `l` and the emulator: `name+offset: rust .. fw ..`.
pub fn state_diff(l: &Lxr, emu: &EmuEngine, ignore: &[&str]) -> Vec<String> {
    let mut out = Vec::new();
    let elf = emu.build == lxr_emu::Build::Elf;
    // `Lxr` uses the reference ELF addresses; the shipped image is addressed through its symbol map
    let addr_of = |a: u32, _name: &str| -> Option<u32> { if elf { Some(a) } else { lxr_core::midi::shipped_layout::to_shipped(a) } };
    // pointer translation: an `Lxr` (reference ELF layout) RAM address -> the same object in the emulated build
    let xlat = |p: u32| -> Option<u32> { if elf { Some(p) } else { lxr_core::midi::shipped_layout::to_shipped(p) } };
    for &(a, size, name) in GLOBALS {
        if ignore.contains(&name) {
            continue;
        }
        let Some(fa) = addr_of(a, name) else { continue };
        let mut fb = emu.fw.read(fa, size as usize).unwrap();
        let mut rb: Vec<u8> = (0..size).map(|k| l.global_get_byte(a + k).unwrap()).collect();
        if elf {
            for &(n, off) in ELF_LAYOUT_BYTES {
                if n == name {
                    rb[off as usize] = 0;
                    fb[off as usize] = 0;
                }
            }
        }
        if name == "parameterArray" {
            for k in 0..rb.len() {
                // padding bytes (not modelled: `CMem` reads 0; the firmware only writes them with
                // out-of-range stores and never reads them), and, for the reference ELF, the pointers of
                // entries 0..13 that out-of-range stores modify (`Lxr` reproduces the shipped layout there)
                if k % 8 >= 5 || (elf && k < 14 * 8 && k % 8 < 4) {
                    rb[k] = 0;
                    fb[k] = 0;
                }
            }
        }
        let mut k = 0usize;
        while k < size as usize {
            if rb[k] != fb[k] {
                let w = k & !3;
                if (a as usize + w) % 4 == 0 && w + 4 <= size as usize {
                    let rw = u32::from_le_bytes(rb[w..w + 4].try_into().unwrap());
                    let fw = u32::from_le_bytes(fb[w..w + 4].try_into().unwrap());
                    // a pointer field (a pointer outside the modelled objects holds the shipped address)
                    if xlat(rw) == Some(fw) || (xlat(rw).is_none() && rw == fw && rw != 0) {
                        k = w + 4;
                        continue;
                    }
                    // NaN payloads are the one documented host difference (lxr_core::fpu): any NaN equals any NaN
                    if f32::from_bits(rw).is_nan() && f32::from_bits(fw).is_nan() {
                        k = w + 4;
                        continue;
                    }
                }
                out.push(format!("{name}+{k}: rust {:#04x} fw {:#04x}", rb[k], fb[k]));
                break;
            }
            k += 1;
        }
    }
    out
}

/// Writes the `Lxr` bytes of the global `name` into the emulated firmware (for globals without pointers).
pub fn push_global(l: &Lxr, emu: &mut EmuEngine, name: &str) {
    let &(a, size, _) = GLOBALS.iter().find(|g| g.2 == name).unwrap_or_else(|| panic!("no global {name}"));
    let fa = if emu.build == lxr_emu::Build::Elf {
        a
    } else {
        lxr_core::midi::shipped_layout::to_shipped(a).unwrap_or_else(|| panic!("{name}: no shipped address"))
    };
    let b: Vec<u8> = (0..size).map(|k| l.global_get_byte(a + k).unwrap()).collect();
    emu.fw.write(fa, &b).unwrap();
}

/// Calls a firmware function in main-loop context (FPSCR = 0, as `EmuEngine` does for MIDI input).
pub fn call_fw(emu: &mut EmuEngine, addr: u32, args: &[u32]) -> u32 {
    emu.fw.fpscr = 0;
    emu.fw.call_addr(addr, args, &[]).unwrap().0
}

/// Number of globals that `state_diff` compares for this build.
pub fn compared_globals(emu: &EmuEngine) -> usize {
    if emu.build == lxr_emu::Build::Elf {
        GLOBALS.len()
    } else {
        lxr_core::midi::shipped_layout::SHIPPED_GLOBALS.len()
    }
}

pub fn apply(l: &mut Lxr, e: &Event) {
    match e {
        Event::Midi(bytes) => l.midi(bytes),
        Event::SetParameter { param, value } => l.set_parameter(*param, *value),
        Event::LoadKit { hex } => l.load_kit(&decode_hex(hex).unwrap()),
        Event::Fader { index, adc } => l.set_fader_adc(*index as usize, *adc),
    }
}

pub fn apply_emu(e: &mut EmuEngine, ev: &Event) {
    lxr_emu::scenario::apply_event(e, ev).unwrap();
}

/// `Lxr` and an emulated firmware driven with the same inputs.
pub struct Pair {
    pub l: Box<Lxr>,
    pub emu: EmuEngine,
    pub ignore: &'static [&'static str],
    pub blocks: u64,
}

impl Pair {
    pub fn new(emu: EmuEngine) -> Self {
        let ignore: &'static [&'static str] = if emu.build == lxr_emu::Build::Elf { ELF_ONLY_GLOBALS } else { &[] };
        Pair { l: Box::new(Lxr::new()), emu, ignore, blocks: 0 }
    }

    /// Reference ELF only: true once out-of-range stores have changed bytes of the `parameterArray[0..14]`
    /// pointers so that `Lxr` (which reproduces the shipped layout) and the ELF point to different fields.
    /// From then on the two programs write through those pointers to different places by design, so only
    /// the audio output is compared with the ELF (the shipped image is compared in full).
    pub fn elf_layout_diverged(&self) -> bool {
        if self.emu.build != lxr_emu::Build::Elf {
            return false;
        }
        (0..14).any(|k| {
            let fw = self.emu.fw.read_u32(lxr_core::memmap::ADDR_parameterArray + 8 * k as u32).unwrap();
            fw != self.l.parameterArray[k].ptr
        })
    }

    /// `state_diff` with this pair's ignore list (empty once `elf_layout_diverged`).
    pub fn diff(&self) -> Vec<String> {
        if self.elf_layout_diverged() {
            return Vec::new();
        }
        state_diff(&self.l, &self.emu, self.ignore)
    }

    #[track_caller]
    pub fn check(&self, ctx: &str) {
        let d = self.diff();
        assert!(d.is_empty(), "{ctx}: state differs: {d:?}");
        assert_eq!(self.l.rng_state, self.emu.rng_state(), "{ctx}: RNG state");
    }

    pub fn event(&mut self, e: &Event) {
        apply(&mut self.l, e);
        apply_emu(&mut self.emu, e);
    }

    pub fn midi(&mut self, bytes: &[u8]) {
        self.event(&Event::Midi(bytes.to_vec()));
    }

    /// One block on both sides; panics if the output differs.
    #[track_caller]
    pub fn tick(&mut self, ctx: &str) {
        let a = self.l.tick();
        let b = self.emu.tick().unwrap();
        self.blocks += 1;
        if a.dac1 != b.dac1 || a.dac2 != b.dac2 {
            let k = a.dac1.iter().chain(a.dac2.iter()).zip(b.dac1.iter().chain(b.dac2.iter())).position(|(x, y)| x != y).unwrap();
            let l = &self.l;
            let wf = [
                l.voiceArray[0].osc.waveform, l.voiceArray[0].modOsc.waveform,
                l.voiceArray[1].osc.waveform, l.voiceArray[1].modOsc.waveform,
                l.voiceArray[2].osc.waveform, l.voiceArray[2].modOsc.waveform,
                l.snareVoice.osc.waveform, l.cymbalVoice.osc.waveform, l.cymbalVoice.modOsc.waveform,
                l.cymbalVoice.modOsc2.waveform, l.hatVoice.osc.waveform, l.hatVoice.modOsc.waveform,
            ];
            panic!("{ctx}: block {} differs at sample {k} (oscillator waveforms {wf:?})", self.blocks);
        }
    }
}

pub fn scenario_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/scenarios")
}

/// All scenarios of the corpus (optionally filtered by `LXR_SCENARIO_FILTER`, a substring of the name).
pub fn scenarios() -> Vec<Scenario> {
    let mut files: Vec<PathBuf> = std::fs::read_dir(scenario_dir())
        .expect("tests/scenarios")
        .map(|e| e.unwrap().path())
        .filter(|p| {
            let n = p.file_name().unwrap().to_str().unwrap();
            n.ends_with(".json") && n != "params.json" && !n.starts_with("reference")
        })
        .collect();
    files.sort();
    let filter = std::env::var("LXR_SCENARIO_FILTER").ok();
    files
        .iter()
        .map(|p| Scenario::load(p).unwrap())
        .filter(|s| filter.as_deref().is_none_or(|f| s.name.contains(f)))
        .collect()
}
