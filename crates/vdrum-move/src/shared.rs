// SPDX-License-Identifier: GPL-3.0-only
//! State shared between the audio thread (Schwung's SPI callback) and the module's worker thread.
//!
//! * Worker → audio: [`Command`]s through a bounded lock-free queue.
//! * Hardware → worker: raw MIDI from the controls ([`Shared::input`]), pushed on the audio thread
//!   by `on_midi` and by `set_param("in", …)` (the controls `ui.js` forwards).
//! * Audio → worker: the parameter mirror, playback state, LFO values and hit counters, as atomics.
//! * Worker → `ui.js`: the LED and display frame ([`Shared::ui`]), read by `get_param("ui")` with
//!   `try_lock`, so the audio thread never waits.
//!
//! Nothing in the audio thread's path allocates: queue elements are fixed-size values and the
//! queues are allocated once.

use crossbeam_queue::ArrayQueue;
use lxr_params::kit::{Kit, KIT_FILE_LEN, KIT_PARAMS};
use lxr_params::NUM_ALL_PARAMS;
use std::sync::atomic::{AtomicBool, AtomicI32, AtomicI8, AtomicU16, AtomicU32, AtomicU64, AtomicU8, Ordering};
use std::sync::Mutex;
use vdrum_engine::engine::SeqEv;
use vdrum_engine::seq::{SeqData, SeqStatus, NUM_TRACKS};

/// Number of parameters in the mirror (engine parameters and open hi-hat parameters).
pub const N: usize = NUM_ALL_PARAMS as usize;

/// A request from the worker to the audio thread. A value, not boxed: the audio thread must not
/// free memory.
#[allow(clippy::large_enum_variant)]
#[derive(Clone, Copy)]
pub enum Command {
    /// Set an engine parameter.
    Param(u16, u16),
    /// Move voice fader `i` to step `0..=FADER_MAX`.
    Fader(u8, i32),
    /// A MIDI message for the engine (note-ons that trigger voices).
    Midi([u8; 3]),
    /// A sequencer operation.
    Seq(SeqEv),
    /// Load a kit (VDrum `.snd` bytes).
    LoadKit([u8; KIT_FILE_LEN]),
    /// Replace the engine's sequencer data with [`Shared::seq`].
    SeqReload,
    /// Silence every voice.
    Panic,
}

/// A raw message from Move's controls (cable 0): status, data 1, data 2, and where it came from
/// ([`SRC_DIRECT`] or [`SRC_JS`]).
pub type In = [u8; 4];
/// The message reached the DSP's `on_midi` (pads and steps; the voice row has already played).
pub const SRC_DIRECT: u8 = 0;
/// The message was forwarded by `ui.js` (`set_param("in", …)`).
pub const SRC_JS: u8 = 1;

/// The sequencer's playback state as atomics (written by the audio thread once per block).
#[derive(Default)]
pub struct AtomicSeqStatus {
    running: AtomicBool,
    pattern: AtomicU8,
    pending: AtomicU8,
    step: [AtomicI8; NUM_TRACKS],
    late: AtomicBool,
    silent: AtomicBool,
}

impl AtomicSeqStatus {
    pub fn store(&self, s: &SeqStatus) {
        self.running.store(s.running, Ordering::Relaxed);
        self.pattern.store(s.pattern, Ordering::Relaxed);
        self.pending.store(s.pending, Ordering::Relaxed);
        for (a, v) in self.step.iter().zip(s.step) {
            a.store(v, Ordering::Relaxed);
        }
        self.late.store(s.late, Ordering::Relaxed);
        self.silent.store(s.silent, Ordering::Relaxed);
    }

    pub fn load(&self) -> SeqStatus {
        SeqStatus {
            running: self.running.load(Ordering::Relaxed),
            pattern: self.pattern.load(Ordering::Relaxed),
            pending: self.pending.load(Ordering::Relaxed),
            step: std::array::from_fn(|i| self.step[i].load(Ordering::Relaxed)),
            late: self.late.load(Ordering::Relaxed),
            silent: self.silent.load(Ordering::Relaxed),
        }
    }
}

/// What the audio thread and the worker share.
pub struct Shared {
    pub commands: ArrayQueue<Command>,
    pub input: ArrayQueue<In>,
    /// Parameter mirror (engine units), updated by the audio thread after every change it applies.
    pub values: Vec<AtomicU16>,
    /// Voice fader steps.
    pub faders: [AtomicI32; 7],
    /// Hits per track (pads, sequencer steps, triggers), for the pad flashes.
    pub hits: [AtomicU32; 7],
    /// Muted tracks (bit `t`).
    pub mutes: AtomicU8,
    /// Accent pad held: pads play at velocity 127.
    pub accent: AtomicBool,
    /// Mute held: the voice pads toggle mutes instead of playing.
    pub mute_held: AtomicBool,
    pub seq_status: AtomicSeqStatus,
    pub lfo: [AtomicU32; 7],
    /// The sequencer data the worker edits (the engine plays a copy). The audio thread only uses
    /// `try_lock` (for [`Command::SeqReload`]).
    pub seq: Mutex<SeqData>,
    pub kit_name: Mutex<[u8; 8]>,
    /// The frame for `ui.js` (`get_param("ui")`).
    pub ui: Mutex<String>,
    /// Render time of the last block and the maximum, in microseconds.
    pub perf_last_us: AtomicU32,
    pub perf_max_us: AtomicU32,
    pub perf_sum_us: AtomicU64,
    pub perf_blocks: AtomicU64,
    /// Pads and steps arrived through `on_midi` (the direct path); the copies `ui.js` forwards are
    /// then ignored.
    pub direct_notes: AtomicBool,
    /// `ui.js` asked the worker to save (module exit or suspend).
    pub save_requested: AtomicBool,
    /// Messages dropped because a queue was full.
    pub dropped: AtomicU32,
}

impl Shared {
    pub fn new() -> Shared {
        Shared {
            commands: ArrayQueue::new(1024),
            input: ArrayQueue::new(1024),
            values: (0..N).map(|_| AtomicU16::new(0)).collect(),
            faders: std::array::from_fn(|_| AtomicI32::new(vdrum_engine::defaults::FADER_MAX)),
            hits: std::array::from_fn(|_| AtomicU32::new(0)),
            mutes: AtomicU8::new(0),
            accent: AtomicBool::new(false),
            mute_held: AtomicBool::new(false),
            seq_status: AtomicSeqStatus::default(),
            lfo: std::array::from_fn(|_| AtomicU32::new(0.5f32.to_bits())),
            seq: Mutex::new(SeqData::default()),
            kit_name: Mutex::new(*b"INIT\0\0\0\0"),
            ui: Mutex::new(String::with_capacity(1024)),
            perf_last_us: AtomicU32::new(0),
            perf_max_us: AtomicU32::new(0),
            perf_sum_us: AtomicU64::new(0),
            perf_blocks: AtomicU64::new(0),
            direct_notes: AtomicBool::new(false),
            save_requested: AtomicBool::new(false),
            dropped: AtomicU32::new(0),
        }
    }

    pub fn value(&self, idx: u16) -> u16 {
        self.values.get(idx as usize).map_or(0, |v| v.load(Ordering::Relaxed))
    }

    /// Queue a command for the audio thread; counts it as dropped if the queue is full.
    pub fn send(&self, c: Command) {
        if self.commands.push(c).is_err() {
            self.dropped.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Queue a control message for the worker; counts it as dropped if the queue is full.
    pub fn push_input(&self, m: In) {
        if self.input.push(m).is_err() {
            self.dropped.fetch_add(1, Ordering::Relaxed);
        }
    }

    pub fn kit_name_bytes(&self) -> [u8; 8] {
        self.kit_name.lock().map(|n| *n).unwrap_or([0; 8])
    }

    pub fn set_kit_name(&self, name: [u8; 8]) {
        if let Ok(mut n) = self.kit_name.lock() {
            *n = name;
        }
    }

    /// The current kit, from the mirror.
    pub fn kit(&self) -> Kit {
        let mut k = Kit::from_values(self.kit_name_bytes(), [0; KIT_PARAMS]);
        for (i, v) in k.values.iter_mut().enumerate() {
            *v = self.value(i as u16).min(255) as u8;
        }
        for (k_, v) in k.oh.iter_mut().enumerate() {
            *v = self.value(lxr_params::open_hat::oh_index(k_)).min(255) as u8;
        }
        k
    }
}

impl Default for Shared {
    fn default() -> Self {
        Shared::new()
    }
}
