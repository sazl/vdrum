// SPDX-License-Identifier: GPL-3.0-only
//! VDrum: CLAP/VST3/standalone plugin around a port of the LXR-02 sound engine.
//!
//! See `plugin/README.md` for the timing model, MIDI handling and limitations.

pub mod editor;
pub mod engine;
pub mod fp;
pub mod params;
pub mod pipeline;
#[cfg(test)]
mod real_engine_tests;

use engine::{tracks_for_note, Block, Engine};
use lxr_params::{GLOBAL_PARAMS, PARAMS};
use nih_plug::midi::MidiResult;
use nih_plug::prelude::*;
use params::{scale_cc, Command, Shared, VDrumParams, N};
use pipeline::{Ev, Pipeline};
use std::cell::Cell;
use std::num::NonZeroU32;
use std::sync::atomic::Ordering;
use std::sync::Arc;

/// The engine the plugin runs: the `lxr-core` port of the LXR-02 sound engine.
pub type ActiveEngine = engine::LxrEngine;

/// A new engine in its power-on state, with the plugin's MIDI defaults
/// ([`params::DEFAULT_TRACK_NOTES`]) applied through the firmware's parameter path. Allocates; not
/// for the audio thread.
pub fn new_engine() -> Box<ActiveEngine> {
    let _ieee = fp::IeeeGuard::new();
    let mut e = ActiveEngine::new();
    apply_plugin_defaults(&mut *e);
    e
}

/// Set the parameters whose plugin default differs from the firmware's power-on value.
pub fn apply_plugin_defaults<E: Engine>(e: &mut E) {
    let _ieee = fp::IeeeGuard::new();
    for p in GLOBAL_PARAMS.iter() {
        let d = params::plugin_default(p);
        if e.get_parameter(p.index) != d {
            e.set_parameter(p.index, d);
        }
    }
}

pub struct VDrum {
    params: Arc<VDrumParams>,
    shared: Arc<Shared>,
    engine: Option<Box<ActiveEngine>>,
    pipeline: Option<Pipeline<ActiveEngine>>,
    /// Last seen host value of every host parameter slot.
    last_host: Vec<i32>,
    /// Last seen value of the fader parameters.
    last_fader: [i32; 6],
    has_aux: bool,
    /// Whether the pipeline currently runs the engine at the hardware clock rate.
    hardware_clock: bool,
    meter_decay_per_sample: f32,
    meter_state: [f32; 4],
}

impl Default for VDrum {
    fn default() -> Self {
        let shared = Arc::new(Shared::new());
        let params = Arc::new(VDrumParams::new(shared.clone()));
        let engine = new_engine();
        let s = VDrum {
            last_host: params.ints.iter().map(|p| p.value()).collect(),
            last_fader: std::array::from_fn(|i| params.faders[i].value()),
            params,
            shared,
            engine: Some(engine),
            pipeline: None,
            has_aux: true,
            hardware_clock: false,
            meter_decay_per_sample: 0.9999,
            meter_state: [0.0; 4],
        };
        if let Some(e) = s.engine.as_deref() {
            sync_mirror(e, &s.shared, false);
        }
        s
    }
}

/// Copy the engine's parameter values to the shared mirror. With `mark_dirty`, every changed value
/// is flagged so that the GUI copies it to the host parameter.
fn sync_mirror<E: Engine>(engine: &E, shared: &Shared, mark_dirty: bool) {
    let _ieee = fp::IeeeGuard::new();
    for p in PARAMS.iter().chain(GLOBAL_PARAMS.iter()) {
        let i = p.index as usize;
        let v = engine.get_parameter(p.index);
        if shared.values[i].swap(v, Ordering::Relaxed) != v && mark_dirty {
            shared.dirty[i].store(true, Ordering::Relaxed);
        }
    }
}

const AUX_NAMES: &[&str] = &["Aux (DAC 2)"];

impl Plugin for VDrum {
    const NAME: &'static str = "VDrum";
    const VENDOR: &'static str = "VDrum";
    const URL: &'static str = "https://github.com/sazl/vdrum";
    const EMAIL: &'static str = "";
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[
        AudioIOLayout {
            main_input_channels: None,
            main_output_channels: NonZeroU32::new(2),
            aux_input_ports: &[],
            aux_output_ports: &[new_nonzero_u32(2)],
            names: PortNames {
                layout: Some("Main + Aux"),
                main_input: None,
                main_output: Some("Main (DAC 1)"),
                aux_inputs: &[],
                aux_outputs: AUX_NAMES,
            },
        },
        AudioIOLayout {
            main_input_channels: None,
            main_output_channels: NonZeroU32::new(2),
            aux_input_ports: &[],
            aux_output_ports: &[],
            names: PortNames {
                layout: Some("Main only (aux summed)"),
                main_input: None,
                main_output: Some("Main (DAC 1 + DAC 2)"),
                aux_inputs: &[],
                aux_outputs: &[],
            },
        },
    ];

    const MIDI_INPUT: MidiConfig = MidiConfig::MidiCCs;
    const SAMPLE_ACCURATE_AUTOMATION: bool = false;

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&mut self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        editor::create(self.params.clone(), self.shared.clone())
    }

    fn initialize(
        &mut self,
        audio_io_layout: &AudioIOLayout,
        buffer_config: &BufferConfig,
        context: &mut impl InitContext<Self>,
    ) -> bool {
        let mut engine = match (self.pipeline.take(), self.engine.take()) {
            (Some(p), _) => p.engine,
            (None, Some(e)) => e,
            (None, None) => new_engine(),
        };
        if engine.faulted() {
            // The engine hit an internal error (see `LxrEngine`); start over. The host
            // parameters below restore the kit and the settings.
            nih_log!("VDrum: the engine stopped after an internal error and was restarted");
            engine = new_engine();
        }

        let ieee = fp::IeeeGuard::new();
        // State restored by the host: the kit and the global settings are authoritative.
        let restore = self.shared.restore.lock().map(|mut r| std::mem::take(&mut *r)).unwrap_or_default();
        let restored = restore.kit.is_some();
        if let Some(kit) = restore.kit {
            engine.load_kit(&kit.to_file_bytes());
            self.shared.set_name(&kit.name);
        }
        for (idx, v) in restore.globals {
            engine.set_parameter(idx, v);
        }
        if !restored {
            // Apply host values that differ from the engine, except values the engine changed
            // itself that have not reached the host parameter yet.
            for (slot, info) in self.params.infos.iter().enumerate() {
                let v = self.params.ints[slot].value() as u16;
                if !self.shared.dirty[info.index as usize].load(Ordering::Relaxed) && engine.get_parameter(info.index) != v {
                    engine.set_parameter(info.index, v);
                }
            }
        }
        for (i, f) in self.params.faders.iter().enumerate() {
            engine.set_fader_adc(i, params::fader_adc(f.value()));
        }
        drop(ieee);
        sync_mirror(&*engine, &self.shared, false);
        // Host parameters that differ from the engine are updated by the GUI.
        for (slot, info) in self.params.infos.iter().enumerate() {
            if self.params.ints[slot].value() as u16 != self.shared.value(info.index) {
                self.shared.dirty[info.index as usize].store(true, Ordering::Relaxed);
            }
        }
        self.last_host = self.params.ints.iter().map(|p| p.value()).collect();
        self.last_fader = std::array::from_fn(|i| self.params.faders[i].value());

        self.hardware_clock = self.params.hardware_clock.value();
        let (rate, alt) = if self.hardware_clock {
            (engine::HARDWARE_RATE, engine::ENGINE_RATE as f64)
        } else {
            (engine::ENGINE_RATE as f64, engine::HARDWARE_RATE)
        };
        let pipeline = Pipeline::with_alternate(engine, buffer_config.sample_rate, rate, Some(alt));
        context.set_latency_samples(pipeline.latency());
        self.shared.latency.store(pipeline.latency(), Ordering::Relaxed);
        self.shared.host_rate.store(buffer_config.sample_rate, Ordering::Relaxed);
        self.pipeline = Some(pipeline);

        self.has_aux = !audio_io_layout.aux_output_ports.is_empty();
        self.shared.has_aux.store(self.has_aux, Ordering::Relaxed);
        // Peak meters fall by 20 dB in about 300 ms.
        self.meter_decay_per_sample = 0.1f32.powf(1.0 / (0.3 * buffer_config.sample_rate));
        true
    }

    fn reset(&mut self) {
        if let Some(p) = &mut self.pipeline {
            p.reset();
        }
        self.meter_state = [0.0; 4];
    }

    fn process(
        &mut self,
        buffer: &mut Buffer,
        aux: &mut AuxiliaryBuffers,
        context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        let Some(pipeline) = self.pipeline.as_mut() else {
            return ProcessStatus::Normal;
        };
        let shared = &*self.shared;
        let params = &*self.params;
        let midi_applied = Cell::new(false);
        let mut apply = |e: &mut ActiveEngine, ev: Ev| apply_event(e, shared, &midi_applied, ev);

        // GUI commands
        while let Some(cmd) = shared.commands.pop() {
            match cmd {
                Command::Midi(bytes) => {
                    pad_hits(shared, bytes);
                    pipeline.push_event(0, Ev::midi(bytes), &mut apply);
                }
                Command::LoadKit(bytes) => {
                    {
                        let _ieee = fp::IeeeGuard::new();
                        pipeline.engine.load_kit(&bytes);
                    }
                    sync_mirror(&*pipeline.engine, shared, true);
                }
            }
        }

        // Engine clock: 44100 Hz or the hardware's 44140.625 Hz (both prepared in initialize()).
        let hw = params.hardware_clock.value();
        if hw != self.hardware_clock && pipeline.swap_stage() {
            self.hardware_clock = hw;
            context.set_latency_samples(pipeline.latency());
            shared.latency.store(pipeline.latency(), Ordering::Relaxed);
        }

        // Host parameter changes (polled once per buffer)
        for (slot, p) in params.ints.iter().enumerate() {
            let v = p.value();
            if v != self.last_host[slot] {
                self.last_host[slot] = v;
                let idx = params.infos[slot].index;
                pipeline.push_event(0, Ev::Param { idx, val: v as u16, learned: false }, &mut apply);
            }
        }

        for (i, f) in params.faders.iter().enumerate() {
            let v = f.value();
            if v != self.last_fader[i] {
                self.last_fader[i] = v;
                pipeline.push_event(0, Ev::Fader { index: i as u8, adc: params::fader_adc(v) }, &mut apply);
            }
        }

        // MIDI from the host
        while let Some(event) = context.next_event() {
            let timing = event.timing();
            let Some(MidiResult::Basic(bytes)) = event.as_midi() else {
                continue;
            };
            if bytes[0] & 0xf0 == 0xb0 {
                let (ch, cc) = (bytes[0] & 0x0f, bytes[1] & 0x7f);
                let target = shared.learn_target.load(Ordering::Relaxed);
                if target >= 0
                    && shared
                        .learn_target
                        .compare_exchange(target, -1, Ordering::Relaxed, Ordering::Relaxed)
                        .is_ok()
                {
                    shared.learn[ch as usize * 128 + cc as usize].store(target as u16 + 1, Ordering::Relaxed);
                    shared.learn_generation.fetch_add(1, Ordering::Relaxed);
                    continue;
                }
                if let Some(idx) = shared.learned(ch, cc) {
                    if let Some(info) = lxr_params::info(idx) {
                        let val = scale_cc(info, bytes[2]);
                        pipeline.push_event(timing, Ev::Param { idx, val, learned: true }, &mut apply);
                        continue;
                    }
                }
            }
            pad_hits(shared, bytes);
            pipeline.push_event(timing, Ev::midi(bytes), &mut apply);
        }

        // Render
        let frames = buffer.samples();
        let sum_aux = params.sum_aux.value() || !self.has_aux;
        let main = buffer.as_slice();
        let mut aux_out = aux.outputs.first_mut().map(|b| b.as_slice());
        let mut peak = [0.0f32; 4];
        let mut after_tick = |e: &mut ActiveEngine, _b: &Block| {
            if midi_applied.replace(false) {
                sync_mirror(&*e, shared, true);
            }
        };
        pipeline.render(
            frames,
            &mut apply,
            &mut after_tick,
            &mut |i, f: [f32; 4]| {
                for (p, x) in peak.iter_mut().zip(f) {
                    *p = p.max(x.abs());
                }
                let (mut l, mut r) = (f[0], f[1]);
                if sum_aux {
                    l += f[2];
                    r += f[3];
                }
                if main.len() >= 2 {
                    main[0][i] = l;
                    main[1][i] = r;
                }
                if let Some(a) = aux_out.as_mut() {
                    if a.len() >= 2 {
                        a[0][i] = if sum_aux { 0.0 } else { f[2] };
                        a[1][i] = if sum_aux { 0.0 } else { f[3] };
                    }
                }
            },
        );

        let decay = self.meter_decay_per_sample.powi(frames as i32);
        for ((m, p), out) in self.meter_state.iter_mut().zip(peak).zip(&shared.meters) {
            *m = p.max(*m * decay);
            out.store(*m, Ordering::Relaxed);
        }
        ProcessStatus::KeepAlive
    }
}

/// Apply one scheduled event to the engine and keep the mirror up to date.
fn apply_event<E: Engine>(e: &mut E, shared: &Shared, midi_applied: &Cell<bool>, ev: Ev) {
    let _ieee = fp::IeeeGuard::new();
    match ev {
        Ev::Midi { bytes, len } => {
            // TEMPORARY WORKAROUND (remove once lxr-core handles it): CC 0 (bank change) loads
            // kit slot `value` (`frontPanel_ccHandler` -> `preset_loadDrumsetRam`). The firmware
            // has 64 slots and reads past `mainParameters` for values 64..=127; lxr-core (as of
            // c1c8c37) panics there: `Lxr::midi(&[0xB0, 0, 93])`, index out of bounds at
            // crates/lxr-core/src/preset/mod.rs:174. Such messages are dropped.
            if len == 3 && bytes[0] & 0xf0 == 0xb0 && bytes[1] == 0 && bytes[2] >= engine::KIT_SLOTS {
                return;
            }
            for b in &bytes[..len as usize] {
                e.midi_byte(*b);
            }
            midi_applied.set(true);
        }
        Ev::Param { idx, val, learned } => {
            if e.get_parameter(idx) != val {
                e.set_parameter(idx, val);
            }
            let v = e.get_parameter(idx);
            if (idx as usize) < N {
                let old = shared.values[idx as usize].swap(v, Ordering::Relaxed);
                if learned && old != v {
                    shared.dirty[idx as usize].store(true, Ordering::Relaxed);
                }
            }
        }
        Ev::Fader { index, adc } => e.set_fader_adc(index as usize, adc),
    }
}

/// Light the pads of the tracks a note-on triggers (by the current channel/note settings).
fn pad_hits(shared: &Shared, bytes: [u8; 3]) {
    if bytes[0] & 0xf0 == 0x90 && bytes[2] > 0 {
        let mask = tracks_for_note(bytes[0] & 0x0f, bytes[1], |p| shared.value(p));
        for t in 0..7 {
            if mask & (1 << t) != 0 {
                shared.pad_hits[t].fetch_add(1, Ordering::Relaxed);
            }
        }
    }
}

impl ClapPlugin for VDrum {
    const CLAP_ID: &'static str = "io.github.sazl.vdrum";
    const CLAP_DESCRIPTION: Option<&'static str> =
        Some("Drum synthesizer built on a port of the LXR-02 sound engine");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] =
        &[ClapFeature::Instrument, ClapFeature::DrumMachine, ClapFeature::Synthesizer, ClapFeature::Stereo];
}

impl Vst3Plugin for VDrum {
    const VST3_CLASS_ID: [u8; 16] = *b"VDrumSazlPlugin1";
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Instrument, Vst3SubCategory::Drum, Vst3SubCategory::Stereo];
}

nih_export_clap!(VDrum);
nih_export_vst3!(VDrum);
