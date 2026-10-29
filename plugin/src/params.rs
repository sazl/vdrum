// SPDX-License-Identifier: GPL-3.0-only
//! Host parameters, state shared between the audio thread and the GUI, and the persisted state.

use crossbeam_queue::ArrayQueue;
use lxr_params::kit::{Kit, KIT_FILE_LEN, KIT_PARAMS};
use lxr_params::{Dtype, ParamInfo, ParamKind, Voice, GLOBAL_PARAMS, NUM_PARAMS, PARAMS};
use nih_plug::params::internals::ParamPtr;
use nih_plug::params::persist::PersistentField;
use nih_plug::prelude::*;
use nih_plug_egui::EguiState;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicU16, AtomicU32, AtomicU8, Ordering};
use std::sync::{Arc, Mutex};

pub const N: usize = NUM_PARAMS as usize;
/// MIDI learn slots: 16 channels × 128 CCs.
pub const LEARN_SLOTS: usize = 16 * 128;
/// Stable ID of the "sum aux into main" switch (not an engine parameter).
pub const SUM_AUX_ID: &str = "vdrum_sum_aux";
/// Stable ID of the "hardware clock" switch: run the engine at the hardware's 44140.625 Hz and
/// resample, instead of 44100 Hz (pass-through at a 44100 Hz host rate).
pub const HARDWARE_CLOCK_ID: &str = "vdrum_hardware_clock";
/// Stable ID prefix of the voice fader parameters (`vdrum_fader_1` .. `vdrum_fader_6`).
pub const FADER_ID_PREFIX: &str = "vdrum_fader_";
const FADER_NAMES: [&str; 6] =
    ["Drum 1 Fader", "Drum 2 Fader", "Drum 3 Fader", "Snare Fader", "Cymbal Fader", "Hi-hat Fader"];

/// Plugin default MIDI notes of the 7 tracks (Drum 1, Drum 2, Drum 3, Snare, Cymbal, closed hi-hat,
/// open hi-hat), all on MIDI channel 1: General MIDI drum notes (36 Bass Drum 1, 45 Low Tom,
/// 50 High Tom, 38 Acoustic Snare, 49 Crash Cymbal 1, 42 Closed Hi-Hat, 46 Open Hi-Hat), so that each
/// voice can be played on its own. The firmware's defaults (`preset_loadDefaultGlobals`, also what
/// `preset_createGlobals` writes to a new `glo.cfg`) put every track on note 48, so one note plays
/// all seven. The plugin sets these after `Lxr::new()` through the firmware's own menu path
/// (`set_parameter(PAR_MIDI_NOTEx)`, which only writes `activeParameters`); a track note only
/// selects which note triggers the voice (the voice then plays `SEQ_DEFAULT_NOTE` either way), so
/// the sound does not change.
pub const DEFAULT_TRACK_NOTES: [u8; 7] = [36, 45, 50, 38, 49, 42, 46];

/// The plugin's default value of a parameter: the firmware default, except for the track notes
/// ([`DEFAULT_TRACK_NOTES`]).
pub fn plugin_default(info: &ParamInfo) -> u16 {
    if (lxr_params::par::MIDI_NOTE1..=lxr_params::par::MIDI_NOTE7).contains(&info.index) {
        DEFAULT_TRACK_NOTES[(info.index - lxr_params::par::MIDI_NOTE1) as usize] as u16
    } else {
        info.default
    }
}

/// Highest step of a voice fader host parameter (0..=`FADER_MAX`).
pub const FADER_MAX: i32 = 99;

/// The 16 bit ADC reading (12 bits, left aligned) of fader step `step`: `36 + 41 * step` in 12 bits.
/// Consecutive steps are 41 counts apart, more than `adc_checkFaders`' hysteresis of 40, so every
/// step reaches the engine; step 0 reads below 1 % (silence), step 99 is the top (`0xFFF0`, the
/// power-on value).
pub fn fader_adc(step: i32) -> u16 {
    ((36 + 41 * step.clamp(0, FADER_MAX)) as u16) << 4
}

/// The gain `adc_checkFaders` derives from a fader step (`v * v`, `v = (adc >> 4) / 4096`, 0 below
/// 0.01).
pub fn fader_gain(step: i32) -> f32 {
    let v = (fader_adc(step) >> 4) as f32 / 4096.0;
    if v < 0.01 {
        0.0
    } else {
        v * v
    }
}

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

/// MIDI learn targets after the engine parameters (`0..N`): the six voice faders, then the two
/// plugin switches.
pub const LEARN_FADER: u16 = N as u16;
pub const LEARN_SUM_AUX: u16 = N as u16 + 6;
pub const LEARN_HW_CLOCK: u16 = N as u16 + 7;
/// Number of plugin (non-engine) learn targets.
pub const EXTRA_TARGETS: usize = 8;

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
        return (1..=6).contains(&n).then(|| LEARN_FADER + n - 1);
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
}

/// State restored from the host, applied in `initialize()`.
#[derive(Default)]
pub struct Restore {
    pub kit: Option<Kit>,
    pub globals: Vec<(u16, u16)>,
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
            commands: ArrayQueue::new(256),
            restore: Mutex::new(Restore::default()),
            host_rate: AtomicF32::new(0.0),
            latency: AtomicU32::new(0),
            extra_values: std::array::from_fn(|_| AtomicI32::new(0)),
            extra_dirty: std::array::from_fn(|_| AtomicBool::new(false)),
            has_aux: AtomicBool::new(true),
        };
        let init = Kit::init();
        s.set_name(&init.name);
        for p in PARAMS.iter().chain(GLOBAL_PARAMS.iter()) {
            s.values[p.index as usize].store(plugin_default(p), Ordering::Relaxed);
        }
        s
    }

    pub fn value(&self, idx: u16) -> u16 {
        self.values[idx as usize].load(Ordering::Relaxed)
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

/// The parameters exposed to the host: every sound parameter and the global settings. Not exposed
/// (but kept in the kit): the unused kit slots, and parameters with a single possible value
/// (`PAR_FX_REVERB_TYPE`: the firmware lists one reverb type), which a host parameter cannot
/// represent (zero-width range).
pub fn host_param_infos() -> Vec<&'static ParamInfo> {
    PARAMS
        .iter()
        .filter(|p| p.kind == ParamKind::Sound && p.min < p.max)
        .chain(GLOBAL_PARAMS.iter())
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
    /// The six voice volume faders (Drum 1..3, Snare, Cymbal, Hi-hat), steps 0..=[`FADER_MAX`].
    pub faders: [IntParam; 6],
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
        let mut k = Kit { name: self.shared.name_bytes(), values: [0; KIT_PARAMS] };
        for (i, v) in k.values.iter_mut().enumerate() {
            *v = self.current(i as u16).min(255) as u8;
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
            v.push((format!("{FADER_ID_PREFIX}{}", i + 1), f.as_ptr(), format!("{}/Mix", Voice::VOICES[i].label())));
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
        assert_eq!(map.len(), 233 + 16 + 2 + 6);
        // Ranges must not be empty (hosts reject them).
        assert!(p.infos.iter().all(|i| i.min < i.max));
        let mut ids: Vec<_> = map.iter().map(|m| m.0.clone()).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), map.len());
        assert!(map.iter().any(|m| m.0 == "coarse1" && m.2 == "Drum 1/Oscillator"));
        assert!(map.iter().any(|m| m.0 == "midi_chan_global" && m.2 == "Global/MIDI"));
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
        // Current layout: both kept.
        let cur = [("editor-state", state((1531, 869))), ("ui-scale", "1.1".into()), ("editor-layout", "2".into())];
        assert_eq!(load(&cur), ((1531, 869), 1.1));
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
}
