// SPDX-License-Identifier: GPL-3.0-only
//! Host parameters, state shared between the audio thread and the GUI, and the persisted state.

use crossbeam_queue::ArrayQueue;
use lxr_params::kit::{Kit, KIT_FILE_LEN, KIT_PARAMS};
use lxr_params::{oh_params, Dtype, ParamInfo, ParamKind, Voice, GLOBAL_PARAMS, NUM_ALL_PARAMS, PARAMS};
use nih_plug::params::internals::ParamPtr;
use nih_plug::params::persist::PersistentField;
use nih_plug::prelude::*;
use nih_plug_egui::EguiState;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU16, AtomicU32, AtomicU8, Ordering};
use vdrum_engine::seq::SeqData;
use std::sync::{Arc, Mutex};

/// Parameter indices: the firmware's (`0..NUM_PARAMS`) and the open hi-hat's (`NUM_PARAMS..`).
pub const N: usize = NUM_ALL_PARAMS as usize;
/// MIDI learn slots: 16 channels × 128 CCs.
pub const LEARN_SLOTS: usize = 16 * 128;
/// Stable ID of the "sum aux into main" switch (not an engine parameter).
pub const SUM_AUX_ID: &str = "vdrum_sum_aux";
/// Stable ID of the "hardware clock" switch: run the engine at the hardware's 44140.625 Hz and
/// resample, instead of 44100 Hz (pass-through at a 44100 Hz host rate).
pub const HARDWARE_CLOCK_ID: &str = "vdrum_hardware_clock";
/// Stable ID prefix of the voice fader parameters (`vdrum_fader_1` .. `vdrum_fader_7`).
pub const FADER_ID_PREFIX: &str = "vdrum_fader_";
/// Number of voice faders: the firmware's six and the open hi-hat's.
pub const NUM_FADERS: usize = 7;
const FADER_NAMES: [&str; NUM_FADERS] =
    ["Drum 1 Fader", "Drum 2 Fader", "Drum 3 Fader", "Snare Fader", "Cymbal Fader", "Hi-hat Fader", "Open Hi-hat Fader"];

pub use vdrum_engine::defaults::{fader_adc, fader_gain, plugin_default, DEFAULT_TRACK_NOTES, FADER_MAX};

/// Display text of a fader step: its gain in dB.
pub fn fader_text(step: i32) -> String {
    let g = fader_gain(step);
    if g <= 0.0 {
        "-inf dB".to_string()
    } else {
        let db = 20.0 * g.log10();
        if db > -10.0 {
            format!("{db:.2} dB")
        } else {
            format!("{db:.1} dB")
        }
    }
}

/// MIDI learn targets after the engine parameters (`0..N`): the seven voice faders, then the two
/// plugin switches.
pub const LEARN_FADER: u16 = N as u16;
pub const LEARN_SUM_AUX: u16 = N as u16 + NUM_FADERS as u16;
pub const LEARN_HW_CLOCK: u16 = N as u16 + NUM_FADERS as u16 + 1;
/// Number of plugin (non-engine) learn targets.
pub const EXTRA_TARGETS: usize = NUM_FADERS + 2;

/// Stable ID of a learn target (for the saved learn map).
pub fn learn_id(target: u16) -> Option<String> {
    match target {
        t if t < LEARN_FADER => lxr_params::info(t).map(|i| i.id.to_string()),
        t if t < LEARN_SUM_AUX => Some(format!("{FADER_ID_PREFIX}{}", t - LEARN_FADER + 1)),
        LEARN_SUM_AUX => Some(SUM_AUX_ID.to_string()),
        LEARN_HW_CLOCK => Some(HARDWARE_CLOCK_ID.to_string()),
        _ => None,
    }
}

/// Inverse of [`learn_id`].
pub fn learn_target_by_id(id: &str) -> Option<u16> {
    if let Some(n) = id.strip_prefix(FADER_ID_PREFIX).and_then(|n| n.parse::<u16>().ok()) {
        return (1..=NUM_FADERS as u16).contains(&n).then(|| LEARN_FADER + n - 1);
    }
    match id {
        SUM_AUX_ID => Some(LEARN_SUM_AUX),
        HARDWARE_CLOCK_ID => Some(LEARN_HW_CLOCK),
        _ => lxr_params::by_id(id).map(|i| i.index),
    }
}

/// Display name of a learn target.
pub fn learn_name(target: u16) -> String {
    match target {
        t if t < LEARN_FADER => lxr_params::info(t).map(|i| i.name.to_string()).unwrap_or_default(),
        t if t < LEARN_SUM_AUX => FADER_NAMES[(t - LEARN_FADER) as usize].to_string(),
        LEARN_SUM_AUX => "Sum Aux Into Main".to_string(),
        LEARN_HW_CLOCK => "Hardware Clock".to_string(),
        _ => "?".to_string(),
    }
}

/// A message from the GUI to the audio thread.
#[derive(Clone, Copy)]
#[allow(clippy::large_enum_variant)] // inline on purpose: no allocation or deallocation on the audio thread
pub enum Command {
    /// A MIDI message (pads).
    Midi([u8; 3]),
    /// Load a kit (`.snd` file bytes).
    LoadKit([u8; KIT_FILE_LEN]),
    /// Load all of [`Shared::seq`] into the engine (after a pattern import).
    SeqReload,
}

/// The kit file the user works on: where it was loaded from or saved to, and its contents then
/// (for the editor's kit file handling).
#[derive(Clone, Default)]
pub struct KitFile {
    pub path: Option<std::path::PathBuf>,
    pub saved: Option<Vec<u8>>,
}

/// State restored from the host, applied in `initialize()`.
#[derive(Default)]
pub struct Restore {
    pub kit: Option<Kit>,
    pub globals: Vec<(u16, u16)>,
    /// Voice fader steps as saved (they may differ from the host parameters when a learned CC
    /// moved them while the editor was closed).
    pub faders: Option<[i32; NUM_FADERS]>,
}

/// Lock-free state shared by the audio thread and the GUI.
pub struct Shared {
    /// The engine's parameter values as last seen by the audio thread.
    pub values: [AtomicU16; N],
    /// Set when the engine changed a value on its own (MIDI, kit load); the GUI copies it to the
    /// host parameter and clears the flag.
    pub dirty: [AtomicBool; N],
    /// MIDI learn map: `channel * 128 + cc` → parameter index + 1 (0 = not learned).
    pub learn: [AtomicU16; LEARN_SLOTS],
    /// Parameter waiting for a CC to learn, or -1.
    pub learn_target: AtomicI32,
    /// Incremented whenever the learn map changes.
    pub learn_generation: AtomicU32,
    /// Output peak levels: DAC1 L/R, DAC2 L/R.
    pub meters: [AtomicF32; 4],
    /// Incremented when a track is triggered (for the pad lights).
    pub pad_hits: [AtomicU32; 7],
    /// Kit name (8 bytes as in the kit file).
    pub name: [AtomicU8; 8],
    pub commands: ArrayQueue<Command>,
    pub restore: Mutex<Restore>,
    /// Current host sample rate and latency (for display).
    pub host_rate: AtomicF32,
    pub latency: AtomicU32,
    pub has_aux: AtomicBool,
    /// Values of the plugin learn targets (faders: step, switches: 0/1) set by a learned CC, and
    /// the flags telling the GUI to copy them to the host parameters (indexed by
    /// `target - LEARN_FADER`).
    pub extra_values: [AtomicI32; EXTRA_TARGETS],
    pub extra_dirty: [AtomicBool; EXTRA_TARGETS],
    /// Current step of every voice fader (host parameter or learned CC).
    pub fader_values: [AtomicI32; NUM_FADERS],
    pub kit_file: Mutex<KitFile>,
    /// Current states of the *Sum aux* and *Hardware clock* switches (set by the audio thread).
    pub sum_aux_on: AtomicBool,
    pub hw_clock_on: AtomicBool,
    /// The sequencer's pattern data: pattern imports replace it (and reload the engine), the
    /// plugin state saves it, `initialize()` loads it into the engine.
    pub seq: Mutex<SeqData>,
    /// The sequencer follows the host transport (true) or runs on the engine's own clock.
    pub seq_host_clock: AtomicBool,
    /// The kit library (64 slots), saved with the plugin state.
    pub library: Mutex<crate::library::Library>,
    /// Incremented whenever the library changes (for the editor).
    pub library_generation: AtomicU32,
}

impl Shared {
    pub fn new() -> Self {
        let s = Shared {
            values: std::array::from_fn(|_| AtomicU16::new(0)),
            dirty: std::array::from_fn(|_| AtomicBool::new(false)),
            learn: std::array::from_fn(|_| AtomicU16::new(0)),
            learn_target: AtomicI32::new(-1),
            learn_generation: AtomicU32::new(0),
            meters: std::array::from_fn(|_| AtomicF32::new(0.0)),
            pad_hits: std::array::from_fn(|_| AtomicU32::new(0)),
            name: std::array::from_fn(|_| AtomicU8::new(0)),
            commands: ArrayQueue::new(1024),
            restore: Mutex::new(Restore::default()),
            host_rate: AtomicF32::new(0.0),
            latency: AtomicU32::new(0),
            extra_values: std::array::from_fn(|_| AtomicI32::new(0)),
            extra_dirty: std::array::from_fn(|_| AtomicBool::new(false)),
            has_aux: AtomicBool::new(true),
            fader_values: std::array::from_fn(|_| AtomicI32::new(FADER_MAX)),
            kit_file: Mutex::new(KitFile::default()),
            sum_aux_on: AtomicBool::new(false),
            hw_clock_on: AtomicBool::new(false),
            seq: Mutex::new(SeqData::default()),
            seq_host_clock: AtomicBool::new(true),
            library: Mutex::new(crate::library::Library::default()),
            library_generation: AtomicU32::new(0),
        };
        let init = Kit::init();
        s.set_name(&init.name);
        for p in PARAMS.iter().chain(GLOBAL_PARAMS.iter()).chain(oh_params().iter()) {
            s.values[p.index as usize].store(plugin_default(p), Ordering::Relaxed);
        }
        s
    }

    pub fn value(&self, idx: u16) -> u16 {
        self.values[idx as usize].load(Ordering::Relaxed)
    }

    /// The engine's current kit, from the parameter mirror.
    pub fn kit(&self) -> Kit {
        let mut k = Kit::from_values(self.name_bytes(), [0; KIT_PARAMS]);
        for (i, v) in k.values.iter_mut().enumerate() {
            *v = self.value(i as u16).min(255) as u8;
        }
        for (k_, v) in k.oh.iter_mut().enumerate() {
            *v = self.value(lxr_params::open_hat::oh_index(k_)).min(255) as u8;
        }
        k
    }

    /// Remember the kit file the user loaded or saved.
    pub fn set_kit_file(&self, path: Option<std::path::PathBuf>, bytes: Vec<u8>) {
        if let Ok(mut f) = self.kit_file.lock() {
            if path.is_some() {
                f.path = path;
            }
            f.saved = Some(bytes);
        }
    }

    /// Loads the kit of a library slot (and selects the slot); returns the slot's label, or
    /// `None` for an empty slot.
    pub fn load_library_slot(&self, slot: usize) -> Option<String> {
        let (kit, label) = {
            let mut l = self.library.lock().ok()?;
            let kit = l.slots.get(slot)?.as_ref()?.kit.clone();
            l.selected = Some(slot);
            (kit, l.label(slot))
        };
        self.library_generation.fetch_add(1, Ordering::Relaxed);
        self.set_name(&kit.name);
        // the slot's kit is the kit as last loaded
        self.set_kit_file(None, kit.to_file_bytes().to_vec());
        let _ = self.commands.push(Command::LoadKit(kit.to_file_bytes()));
        Some(label)
    }

    /// Writes the current kit into a library slot (and selects it); returns the slot's label.
    pub fn store_in_library(&self, slot: usize) -> Option<String> {
        let kit = self.kit();
        let label = {
            let mut l = self.library.lock().ok()?;
            let source = l.slots.get(slot)?.as_ref().map(|e| e.source.clone()).unwrap_or_default();
            l.slots[slot] = Some(crate::library::Entry { kit: kit.clone(), source });
            l.selected = Some(slot);
            l.label(slot)
        };
        self.library_generation.fetch_add(1, Ordering::Relaxed);
        self.set_kit_file(None, kit.to_file_bytes().to_vec());
        Some(label)
    }

    /// The selected library slot, if the current kit came from the library.
    pub fn library_selected(&self) -> Option<usize> {
        self.library.lock().ok().and_then(|l| l.selected)
    }

    /// The current kit no longer comes from the library (a kit file or the init kit was loaded).
    pub fn deselect_library(&self) {
        if let Ok(mut l) = self.library.lock() {
            if l.selected.take().is_some() {
                self.library_generation.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    /// The kit name as text (trailing spaces and NULs removed).
    pub fn kit_name(&self) -> String {
        let b = self.name_bytes();
        String::from_utf8_lossy(&b).trim_end_matches(['\0', ' ']).to_string()
    }

    pub fn name_bytes(&self) -> [u8; 8] {
        std::array::from_fn(|i| self.name[i].load(Ordering::Relaxed))
    }

    pub fn set_name(&self, name: &[u8; 8]) {
        for (a, b) in self.name.iter().zip(name) {
            a.store(*b, Ordering::Relaxed);
        }
    }

    /// The learned parameter for a CC.
    pub fn learned(&self, channel: u8, cc: u8) -> Option<u16> {
        let v = self.learn[(channel as usize & 15) * 128 + (cc as usize & 127)].load(Ordering::Relaxed);
        (v != 0).then(|| v - 1)
    }

    /// The CCs learned for a parameter, as (channel, cc).
    pub fn learned_for(&self, idx: u16) -> impl Iterator<Item = (u8, u8)> + '_ {
        (0..LEARN_SLOTS).filter_map(move |s| {
            (self.learn[s].load(Ordering::Relaxed) == idx + 1).then_some(((s / 128) as u8, (s % 128) as u8))
        })
    }

    pub fn forget(&self, idx: u16) {
        for s in self.learn.iter() {
            if s.load(Ordering::Relaxed) == idx + 1 {
                s.store(0, Ordering::Relaxed);
            }
        }
        self.learn_generation.fetch_add(1, Ordering::Relaxed);
    }

    pub fn forget_all(&self) {
        for s in self.learn.iter() {
            s.store(0, Ordering::Relaxed);
        }
        self.learn_generation.fetch_add(1, Ordering::Relaxed);
    }
}

impl Default for Shared {
    fn default() -> Self {
        Self::new()
    }
}

/// Scale a 7 bit CC value onto a parameter's range.
pub fn scale_cc(info: &ParamInfo, v: u8) -> u16 {
    let span = (info.max - info.min) as u32;
    info.min + ((v.min(127) as u32 * span + 63) / 127) as u16
}

/// The parameters exposed to the host: every sound parameter, the global settings and the open
/// hi-hat's parameters (after the others, so that the host slots of the firmware parameters do not
/// move). Not exposed (but kept in the kit): the unused kit slots, and parameters with a single
/// possible value (`PAR_FX_REVERB_TYPE`: the firmware lists one reverb type), which a host parameter
/// cannot represent (zero-width range).
pub fn host_param_infos() -> Vec<&'static ParamInfo> {
    PARAMS
        .iter()
        .filter(|p| p.kind == ParamKind::Sound && p.min < p.max)
        .chain(GLOBAL_PARAMS.iter())
        .chain(oh_params().iter())
        .collect()
}

/// Host-facing text of a value: the firmware's edit-mode text, except that modulation targets are
/// spelled out ("Drum 1 Oscillator Coarse").
pub fn value_text(info: &ParamInfo, v: u16) -> String {
    if info.is_mod_target() {
        if let Some(s) = lxr_params::format::mod_target_readable(v as u8) {
            return s;
        }
    }
    let s = lxr_params::format_value(info, v.min(255) as u8);
    if s.is_empty() {
        v.to_string()
    } else {
        s
    }
}

fn group_of(info: &ParamInfo) -> String {
    match info.kind {
        ParamKind::Global => format!("Global/{}", info.section.label()),
        _ => match info.section.label() {
            "" => info.voice.label().to_string(),
            s => format!("{}/{}", info.voice.label(), s),
        },
    }
}

#[derive(Serialize, Deserialize)]
struct LearnEntry {
    channel: u8,
    cc: u8,
    param: String,
}

pub struct VDrumParams {
    pub editor_state: Arc<EguiState>,
    /// GUI zoom factor (persisted).
    pub ui_scale: AtomicF32,
    pub infos: Vec<&'static ParamInfo>,
    pub ints: Vec<IntParam>,
    /// Host slot of each parameter index, or -1.
    pub slot_of: [i16; N],
    pub sum_aux: BoolParam,
    pub hardware_clock: BoolParam,
    /// The seven voice volume faders (Drum 1..3, Snare, Cymbal, Hi-hat, Open Hi-hat), steps
    /// 0..=[`FADER_MAX`].
    pub faders: [IntParam; NUM_FADERS],
    pub shared: Arc<Shared>,
    groups: Vec<String>,
}

impl VDrumParams {
    pub fn new(shared: Arc<Shared>) -> Self {
        let infos = host_param_infos();
        let mut slot_of = [-1i16; N];
        let ints = infos
            .iter()
            .enumerate()
            .map(|(slot, info)| {
                slot_of[info.index as usize] = slot as i16;
                let i: &'static ParamInfo = info;
                IntParam::new(
                    info.name,
                    plugin_default(info) as i32,
                    IntRange::Linear { min: info.min as i32, max: info.max as i32 },
                )
                .with_value_to_string(Arc::new(move |v| value_text(i, v as u16)))
                .with_string_to_value(Arc::new(move |s| {
                    lxr_params::parse_value(i, s).map(|v| v as i32).filter(|v| *v >= i.min as i32 && *v <= i.max as i32)
                }))
            })
            .collect();
        let groups = infos.iter().map(|i| group_of(i)).collect();
        VDrumParams {
            editor_state: EguiState::from_size(crate::editor::DEFAULT_SIZE.0, crate::editor::DEFAULT_SIZE.1),
            ui_scale: AtomicF32::new(1.0),
            infos,
            ints,
            slot_of,
            sum_aux: BoolParam::new("Sum Aux Into Main", false),
            hardware_clock: BoolParam::new("Hardware Clock (44140.625 Hz)", false).non_automatable(),
            faders: std::array::from_fn(|i| {
                IntParam::new(FADER_NAMES[i], FADER_MAX, IntRange::Linear { min: 0, max: FADER_MAX })
                    .with_value_to_string(Arc::new(fader_text))
                    .with_string_to_value(Arc::new(|s| {
                        let s = s.trim().trim_end_matches("dB").trim();
                        if s.eq_ignore_ascii_case("-inf") {
                            return Some(0);
                        }
                        let db: f32 = s.parse().ok()?;
                        (0..=FADER_MAX).min_by(|a, b| {
                            let d = |x: i32| (20.0 * fader_gain(x).max(1e-9).log10() - db).abs();
                            d(*a).total_cmp(&d(*b))
                        })
                    }))
            }),
            shared,
            groups,
        }
    }

    pub fn param(&self, idx: u16) -> Option<&IntParam> {
        let s = self.slot_of.get(idx as usize).copied().unwrap_or(-1);
        (s >= 0).then(|| &self.ints[s as usize])
    }

    /// The value to persist for a parameter: the engine's value while it has not been copied to the
    /// host parameter yet, the host parameter otherwise.
    pub fn current(&self, idx: u16) -> u16 {
        let sh = &self.shared;
        match self.param(idx) {
            Some(p) if !sh.dirty[idx as usize].load(Ordering::Relaxed) => p.value() as u16,
            _ => sh.value(idx),
        }
    }

    /// The current kit.
    pub fn kit(&self) -> Kit {
        let mut k = Kit::from_values(self.shared.name_bytes(), [0; KIT_PARAMS]);
        for (i, v) in k.values.iter_mut().enumerate() {
            *v = self.current(i as u16).min(255) as u8;
        }
        for (k_, v) in k.oh.iter_mut().enumerate() {
            *v = self.current(lxr_params::open_hat::oh_index(k_)).min(255) as u8;
        }
        k
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn unhex(s: &str) -> Option<Vec<u8>> {
    if !s.len().is_multiple_of(2) {
        return None;
    }
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(s.get(i..i + 2)?, 16).ok()).collect()
}

unsafe impl Params for VDrumParams {
    fn param_map(&self) -> Vec<(String, ParamPtr, String)> {
        let mut v: Vec<(String, ParamPtr, String)> = self
            .infos
            .iter()
            .zip(&self.ints)
            .zip(&self.groups)
            .map(|((info, p), g)| (info.id.to_string(), p.as_ptr(), g.clone()))
            .collect();
        v.push((SUM_AUX_ID.to_string(), self.sum_aux.as_ptr(), "Output".to_string()));
        v.push((HARDWARE_CLOCK_ID.to_string(), self.hardware_clock.as_ptr(), "Output".to_string()));
        for (i, f) in self.faders.iter().enumerate() {
            let voice = Voice::VOICES.get(i).copied().unwrap_or(Voice::OpenHat);
            v.push((format!("{FADER_ID_PREFIX}{}", i + 1), f.as_ptr(), format!("{}/Mix", voice.label())));
        }
        v
    }

    fn serialize_fields(&self) -> BTreeMap<String, String> {
        let mut m = BTreeMap::new();
        if let Ok(s) = self.editor_state.map(serde_json::to_string) {
            m.insert("editor-state".into(), s);
        }
        m.insert("ui-scale".into(), self.ui_scale.load(Ordering::Relaxed).to_string());
        m.insert("editor-layout".into(), crate::editor::LAYOUT_VERSION.to_string());
        let kit = self.kit();
        if let Ok(s) = serde_json::to_string(&hex(&kit.to_file_bytes())) {
            m.insert("kit".into(), s);
        }
        let globals: BTreeMap<&str, u16> = GLOBAL_PARAMS.iter().map(|g| (g.id, self.current(g.index))).collect();
        if let Ok(s) = serde_json::to_string(&globals) {
            m.insert("globals".into(), s);
        }
        let mut learn = Vec::new();
        for slot in 0..LEARN_SLOTS {
            let v = self.shared.learn[slot].load(Ordering::Relaxed);
            if v != 0 {
                if let Some(id) = learn_id(v - 1) {
                    learn.push(LearnEntry { channel: (slot / 128) as u8, cc: (slot % 128) as u8, param: id });
                }
            }
        }
        if let Ok(s) = serde_json::to_string(&learn) {
            m.insert("midi_learn".into(), s);
        }
        // The host parameter, unless a learned CC moved the fader and the host parameter has not
        // caught up yet (the editor copies it while open). Independent of whether `process` ran.
        let faders: Vec<i32> = (0..NUM_FADERS)
            .map(|i| {
                if self.shared.extra_dirty[i].load(Ordering::Relaxed) {
                    self.shared.fader_values[i].load(Ordering::Relaxed)
                } else {
                    self.faders[i].value()
                }
            })
            .collect();
        if let Ok(s) = serde_json::to_string(&faders) {
            m.insert("faders".into(), s);
        }
        if let Ok(l) = self.shared.library.lock() {
            if *l != crate::library::Library::default() {
                m.insert("kit-library".into(), l.to_text());
            }
        }
        if let Ok(d) = self.shared.seq.lock() {
            if *d != SeqData::default() {
                m.insert("sequencer".into(), d.to_text());
            }
        }
        if let Some(p) = self.shared.kit_file.lock().ok().and_then(|f| f.path.clone()) {
            if let Ok(s) = serde_json::to_string(&p) {
                m.insert("kit-path".into(), s);
            }
        }
        m
    }

    fn deserialize_fields(&self, serialized: &BTreeMap<String, String>) {
        if let Some(s) = serialized.get("editor-state") {
            if let Ok(state) = serde_json::from_str::<EguiState>(s) {
                self.editor_state.set(state);
            }
        }
        // States saved before the compact layout have no layout version. Their zoom had no
        // effect and their default window size was too large for laptop screens, so both are
        // reset; a size the user chose by dragging the corner is kept.
        let legacy = !serialized.contains_key("editor-layout");
        if let Some(Ok(z)) = serialized.get("ui-scale").map(|s| s.parse::<f32>()) {
            if z.is_finite() && !legacy {
                self.ui_scale.store(z.clamp(crate::editor::MIN_ZOOM, crate::editor::MAX_ZOOM), Ordering::Relaxed);
            }
        }
        if legacy {
            self.ui_scale.store(1.0, Ordering::Relaxed);
            if self.editor_state.size() == crate::editor::LEGACY_SIZE {
                self.editor_state.set_size(crate::editor::DEFAULT_SIZE);
            }
        }
        // Layout 2 had one card for both hi-hats; its default size is too narrow for seven cards.
        let version = serialized.get("editor-layout").and_then(|s| s.trim_matches('"').parse::<u32>().ok());
        if version == Some(2) && self.editor_state.size() == crate::editor::SHARED_HAT_SIZE {
            self.editor_state.set_size(crate::editor::DEFAULT_SIZE);
        }
        // Layout 4 had a tab bar and a pill row; its default size is taller than needed.
        if version == Some(4) && self.editor_state.size() == crate::editor::TABS_SIZE {
            self.editor_state.set_size(crate::editor::DEFAULT_SIZE);
        }
        let mut restore = Restore::default();
        if let Some(s) = serialized.get("kit") {
            if let Some(bytes) = serde_json::from_str::<String>(s).ok().and_then(|h| unhex(&h)) {
                restore.kit = Kit::from_file_bytes(&bytes).ok();
            }
        }
        if let Some(s) = serialized.get("globals") {
            if let Ok(g) = serde_json::from_str::<BTreeMap<String, u16>>(s) {
                for (id, v) in g {
                    if let Some(info) = GLOBAL_PARAMS.iter().find(|p| p.id == id) {
                        restore.globals.push((info.index, v.clamp(info.min, info.max)));
                    }
                }
            }
        }
        if let Some(s) = serialized.get("midi_learn") {
            if let Ok(entries) = serde_json::from_str::<Vec<LearnEntry>>(s) {
                for slot in self.shared.learn.iter() {
                    slot.store(0, Ordering::Relaxed);
                }
                for e in entries {
                    if let Some(target) = learn_target_by_id(&e.param) {
                        let slot = (e.channel as usize & 15) * 128 + (e.cc as usize & 127);
                        self.shared.learn[slot].store(target + 1, Ordering::Relaxed);
                    }
                }
                self.shared.learn_generation.fetch_add(1, Ordering::Relaxed);
            }
        }
        if let Some(s) = serialized.get("faders") {
            if let Ok(v) = serde_json::from_str::<Vec<i32>>(s) {
                if v.len() == NUM_FADERS {
                    let v: [i32; NUM_FADERS] = std::array::from_fn(|i| v[i].clamp(0, FADER_MAX));
                    // Saving again before the next `initialize` must give the same state.
                    for (i, x) in v.into_iter().enumerate() {
                        self.shared.fader_values[i].store(x, Ordering::Relaxed);
                        let differs = x != self.faders[i].value();
                        if differs {
                            self.shared.extra_values[i].store(x, Ordering::Relaxed);
                        }
                        self.shared.extra_dirty[i].store(differs, Ordering::Relaxed);
                    }
                    restore.faders = Some(v);
                }
            }
        }
        if let Some(p) = serialized.get("kit-path").and_then(|s| serde_json::from_str::<std::path::PathBuf>(s).ok()) {
            if let Ok(mut f) = self.shared.kit_file.lock() {
                f.path = Some(p);
            }
        }
        if let Ok(mut l) = self.shared.library.lock() {
            *l = serialized.get("kit-library").map(|s| crate::library::Library::from_text(s)).unwrap_or_default();
            self.shared.library_generation.fetch_add(1, Ordering::Relaxed);
        }
        // A state without sequencer data has empty patterns; `initialize()` loads them into the
        // engine.
        let seq = serialized.get("sequencer").and_then(|s| SeqData::from_text(s)).unwrap_or_default();
        if let Ok(mut d) = self.shared.seq.lock() {
            *d = seq;
        }
        if let Some(k) = &restore.kit {
            self.shared.set_name(&k.name);
        }
        if let Ok(mut r) = self.shared.restore.lock() {
            *r = restore;
        }
    }
}

/// Whether the GUI shows a parameter as a list (menu) rather than a value bar.
pub fn is_list(info: &ParamInfo) -> bool {
    matches!(info.dtype, Dtype::Menu | Dtype::VoiceLfo | Dtype::OnOff) || info.is_mod_target()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_params_have_unique_stable_ids() {
        let p = VDrumParams::new(Arc::new(Shared::new()));
        let map = p.param_map();
        assert_eq!(map.len(), 233 + 16 + 37 + 2 + 7);
        // Ranges must not be empty (hosts reject them).
        assert!(p.infos.iter().all(|i| i.min < i.max));
        let mut ids: Vec<_> = map.iter().map(|m| m.0.clone()).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), map.len());
        assert!(map.iter().any(|m| m.0 == "coarse1" && m.2 == "Drum 1/Oscillator"));
        assert!(map.iter().any(|m| m.0 == "midi_chan_global" && m.2 == "Global/MIDI"));
        assert!(map.iter().any(|m| m.0 == "coarse7" && m.2 == "Open Hi-hat/Oscillator"));
        assert!(map.iter().any(|m| m.0 == "vdrum_fader_7" && m.2 == "Open Hi-hat/Mix"));
        // the firmware parameters keep their host slots (and order)
        let first: Vec<_> = map.iter().take(233 + 16).map(|m| m.0.clone()).collect();
        assert_eq!(first[0], "osc_wave_drum1");
        assert!(!first.iter().any(|id| lxr_params::by_id(id).is_some_and(|i| i.voice == Voice::OpenHat)));
    }

    #[test]
    fn state_roundtrip() {
        let sh = Arc::new(Shared::new());
        let p = VDrumParams::new(sh.clone());
        sh.learn[3 * 128 + 20].store(lxr_params::par::COARSE1 + 1, Ordering::Relaxed);
        sh.learn[21].store(LEARN_FADER + 2 + 1, Ordering::Relaxed);
        sh.learn[22].store(LEARN_SUM_AUX + 1, Ordering::Relaxed);
        sh.learn[23].store(LEARN_HW_CLOCK + 1, Ordering::Relaxed);
        sh.values[lxr_params::par::VOL1 as usize].store(5, Ordering::Relaxed);
        sh.dirty[lxr_params::par::VOL1 as usize].store(true, Ordering::Relaxed);
        let f = p.serialize_fields();
        let sh2 = Arc::new(Shared::new());
        let p2 = VDrumParams::new(sh2.clone());
        p2.deserialize_fields(&f);
        assert_eq!(sh2.learned(3, 20), Some(lxr_params::par::COARSE1));
        assert_eq!(sh2.learned(0, 21), Some(LEARN_FADER + 2));
        assert_eq!(sh2.learned(0, 22), Some(LEARN_SUM_AUX));
        assert_eq!(sh2.learned(0, 23), Some(LEARN_HW_CLOCK));
        let r = sh2.restore.lock().unwrap();
        let kit = r.kit.as_ref().unwrap();
        assert_eq!(kit.values[lxr_params::par::VOL1 as usize], 5);
        assert_eq!(kit.name_str(), "Initkit");
        assert_eq!(r.globals.len(), 16);
    }

    #[test]
    fn editor_state_migration() {
        use crate::editor::{DEFAULT_SIZE, LEGACY_SIZE};
        let load = |pairs: &[(&str, String)]| {
            let p = VDrumParams::new(Arc::new(Shared::new()));
            p.deserialize_fields(&pairs.iter().map(|(k, v)| (k.to_string(), v.clone())).collect());
            (p.editor_state.size(), p.ui_scale.load(Ordering::Relaxed))
        };
        let state = |(w, h): (u32, u32)| format!("{{\"size\":[{w},{h}]}}");
        // First layout, default size: new default size, zoom reset (it had no effect).
        assert_eq!(load(&[("editor-state", state(LEGACY_SIZE)), ("ui-scale", "1.3".into())]), (DEFAULT_SIZE, 1.0));
        // First layout, resized by hand: size kept.
        assert_eq!(load(&[("editor-state", state((1500, 700))), ("ui-scale", "1.3".into())]), ((1500, 700), 1.0));
        // Layout 2 (one card for both hi-hats): its default size grows to the new default, a size
        // chosen by hand and the zoom are kept.
        use crate::editor::SHARED_HAT_SIZE;
        assert_eq!(load(&[("editor-state", state(SHARED_HAT_SIZE)), ("ui-scale", "1.1".into()), ("editor-layout", "2".into())]), (DEFAULT_SIZE, 1.1));
        assert_eq!(load(&[("editor-state", state((1531, 869))), ("ui-scale", "1.1".into()), ("editor-layout", "2".into())]), ((1531, 869), 1.1));
        // Layout 4 (tab bar and pill row): its default size shrinks to the new default, a size
        // chosen by hand is kept.
        use crate::editor::TABS_SIZE;
        assert_eq!(load(&[("editor-state", state(TABS_SIZE)), ("ui-scale", "1.1".into()), ("editor-layout", "4".into())]), (DEFAULT_SIZE, 1.1));
        assert_eq!(load(&[("editor-state", state((1600, 900))), ("ui-scale", "1.1".into()), ("editor-layout", "4".into())]), ((1600, 900), 1.1));
        // Current layout: both kept.
        let cur = [("editor-state", state(SHARED_HAT_SIZE)), ("ui-scale", "1.1".into()), ("editor-layout", "5".into())];
        assert_eq!(load(&cur), (SHARED_HAT_SIZE, 1.1));
        // And written back.
        let p = VDrumParams::new(Arc::new(Shared::new()));
        p.ui_scale.store(1.1, Ordering::Relaxed);
        let f = p.serialize_fields();
        let p2 = VDrumParams::new(Arc::new(Shared::new()));
        p2.deserialize_fields(&f);
        assert_eq!((p2.editor_state.size(), p2.ui_scale.load(Ordering::Relaxed)), (DEFAULT_SIZE, 1.1));
    }

    #[test]
    fn fader_steps_pass_the_hysteresis() {
        assert_eq!(fader_adc(FADER_MAX), lxr_core::engine::FADER_FULL_ADC);
        assert_eq!(fader_gain(FADER_MAX).to_bits(), 0x3F7FE001);
        assert_eq!(fader_gain(0), 0.0);
        assert!(fader_gain(1) > 0.0);
        for s in 1..=FADER_MAX {
            assert!((fader_adc(s) >> 4) - (fader_adc(s - 1) >> 4) > 40);
        }
        // Every step reaches the engine, also after jumps.
        let mut e = lxr_core::Lxr::new();
        for s in [98, 0, 1, 50, 49, 99] {
            e.set_fader_adc(0, fader_adc(s));
            assert_eq!(e.adc_faderValues[0].to_bits(), fader_gain(s).to_bits(), "step {s}");
        }
    }

    #[test]
    fn cc_scaling() {
        let i = lxr_params::info(lxr_params::par::COARSE1).unwrap();
        assert_eq!(scale_cc(i, 0), 0);
        assert_eq!(scale_cc(i, 127), 127);
        let w = lxr_params::info(lxr_params::par::OSC_WAVE_DRUM1).unwrap();
        assert_eq!(scale_cc(w, 127), w.max);
        assert_eq!(scale_cc(w, 0), w.min);
    }

    #[test]
    fn learned_state_is_saved_with_the_editor_closed() {
        let sh = Arc::new(Shared::new());
        let p = VDrumParams::new(sh.clone());
        // what a learned CC does: mirror + dirty flag, fader value
        sh.values[lxr_params::par::COARSE1 as usize].store(99, Ordering::Relaxed);
        sh.dirty[lxr_params::par::COARSE1 as usize].store(true, Ordering::Relaxed);
        sh.fader_values[3].store(12, Ordering::Relaxed);
        sh.extra_values[3].store(12, Ordering::Relaxed);
        sh.extra_dirty[3].store(true, Ordering::Relaxed);
        let f = p.serialize_fields();
        let sh2 = Arc::new(Shared::new());
        let p2 = VDrumParams::new(sh2.clone());
        p2.deserialize_fields(&f);
        let r = sh2.restore.lock().unwrap();
        assert_eq!(r.kit.as_ref().unwrap().values[lxr_params::par::COARSE1 as usize], 99);
        assert_eq!(r.faders.unwrap()[3], 12);
        // saving again before `initialize` gives the same fader field
        drop(r);
        let f2 = p2.serialize_fields();
        assert_eq!(f2["faders"], f["faders"]);
    }

    #[test]
    fn sequencer_data_is_saved() {
        use vdrum_engine::seq::SeqEdit;
        let sh = Arc::new(Shared::new());
        let p = VDrumParams::new(sh.clone());
        assert!(!p.serialize_fields().contains_key("sequencer"), "empty patterns are not saved");
        sh.seq.lock().unwrap().apply(&SeqEdit::Euclid { pat: 2, track: 1, fills: 5, rotate: 1 });
        let f = p.serialize_fields();
        let sh2 = Arc::new(Shared::new());
        let p2 = VDrumParams::new(sh2.clone());
        sh2.seq.lock().unwrap().apply(&SeqEdit::Euclid { pat: 0, track: 0, fills: 1, rotate: 0 });
        p2.deserialize_fields(&f);
        assert_eq!(*sh2.seq.lock().unwrap(), *sh.seq.lock().unwrap());
        // the kit library
        assert!(!f.contains_key("kit-library"));
        {
            let mut l = sh.library.lock().unwrap();
            let mut k = lxr_params::kit::Kit::init();
            k.set_name("LIB");
            k.values[lxr_params::par::COARSE1 as usize] = 99;
            l.slots[3] = Some(crate::library::Entry { kit: k, source: "03-LIB.SND".into() });
            l.selected = Some(3);
        }
        p2.deserialize_fields(&p.serialize_fields());
        assert_eq!(*sh2.library.lock().unwrap(), *sh.library.lock().unwrap());
        // a state without sequencer data clears the patterns
        let mut f2 = f.clone();
        f2.remove("sequencer");
        p2.deserialize_fields(&f2);
        assert_eq!(*sh2.seq.lock().unwrap(), SeqData::default());
    }
}
