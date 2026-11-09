// SPDX-License-Identifier: GPL-3.0-only
//! The audio thread's side: the engine, the commands from the worker, the pads of the voice row
//! (played here, on the audio thread, so that a pad sounds in the next block), and rendering.
//!
//! Runs on Schwung's SPI callback. Nothing here allocates, locks (other than `try_lock`), does file
//! I/O or logs.

use crate::hw;
use crate::shared::{Command, Shared, SRC_DIRECT};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Instant;
use vdrum_engine::defaults::fader_adc;
use vdrum_engine::engine::{track_trigger_note, Engine, LxrEngine, SeqEv, TICK_FRAMES};
use vdrum_engine::fp::IeeeGuard;
use vdrum_engine::seq::{NUM_PATTERNS, NUM_STEPS, NUM_TRACKS};

/// Frames per Schwung block.
pub const BLOCK_FRAMES: usize = 128;
/// Most commands applied per block (the rest wait for the next block).
const MAX_COMMANDS_PER_BLOCK: usize = 256;

pub struct Audio {
    pub engine: Box<LxrEngine>,
    shared: Arc<Shared>,
    /// Step of each track at the end of the last block (sequencer hits for the pad flashes).
    last_step: [i8; NUM_TRACKS],
    seq_reload_pending: bool,
}

impl Audio {
    /// The sequencer runs on the engine's own clock (`PAR_BPM`); Move's transport is not followed.
    pub fn new(engine: Box<LxrEngine>, shared: Arc<Shared>) -> Audio {
        let mut a = Audio { engine, shared, last_step: [-1; NUM_TRACKS], seq_reload_pending: false };
        {
            let _ieee = IeeeGuard::new();
            a.engine.seq(SeqEv::InternalClock);
        }
        a.sync_mirror();
        a
    }

    /// Copies every parameter from the engine to the mirror.
    pub fn sync_mirror(&self) {
        let _ieee = IeeeGuard::new();
        for (i, v) in self.shared.values.iter().enumerate() {
            v.store(self.engine.get_parameter(i as u16), Ordering::Relaxed);
        }
    }

    /// A note from Move's controls (pads, steps). The voice row plays here; everything also goes to
    /// the worker for the surface.
    pub fn control_note(&mut self, m: [u8; 3]) {
        let on = m[0] & 0xf0 == 0x90 && m[2] > 0;
        let off = m[0] & 0xf0 == 0x80 || (m[0] & 0xf0 == 0x90 && m[2] == 0);
        if let Some(pad) = hw::pad_index(m[1]) {
            if pad < 8 && !self.shared.mute_held.load(Ordering::Relaxed) {
                if pad == hw::ACCENT_PAD {
                    if on || off {
                        self.shared.accent.store(on, Ordering::Relaxed);
                    }
                } else if on {
                    let vel = if self.shared.accent.load(Ordering::Relaxed) { 127 } else { m[2] };
                    self.trigger(pad, vel);
                }
            }
        }
        self.shared.push_input([m[0], m[1], m[2], SRC_DIRECT]);
    }

    /// A channel message from the USB-A MIDI input: played by the engine as its own MIDI input
    /// (the tracks' notes and channels).
    pub fn external_midi(&mut self, m: &[u8]) {
        match *m {
            [s, a, b] if (0x80..0xf0).contains(&s) => self.apply(Command::Midi([s, a & 0x7f, b & 0x7f])),
            [s, a] if (0xc0..0xe0).contains(&s) => self.midi(&[s, a & 0x7f]),
            _ => {}
        }
    }

    /// Trigger track `t` with `velocity` through the engine's MIDI input (the track's note and
    /// channel), unless the track is muted.
    pub fn trigger(&mut self, t: u8, velocity: u8) {
        if t >= 7 || self.shared.mutes.load(Ordering::Relaxed) & (1 << t) != 0 {
            return;
        }
        let (ch, note) = track_trigger_note(t as usize, |p| self.engine.get_parameter(p));
        self.midi(&[0x90 | ch, note, velocity.clamp(1, 127)]);
        self.shared.hits[t as usize].fetch_add(1, Ordering::Relaxed);
    }

    fn midi(&mut self, bytes: &[u8]) {
        let _ieee = IeeeGuard::new();
        for b in bytes {
            self.engine.midi_byte(*b);
        }
    }

    fn apply(&mut self, c: Command) {
        let _ieee = IeeeGuard::new();
        match c {
            Command::Param(idx, val) => {
                if self.engine.get_parameter(idx) != val {
                    self.engine.set_parameter(idx, val);
                }
                if let Some(v) = self.shared.values.get(idx as usize) {
                    v.store(self.engine.get_parameter(idx), Ordering::Relaxed);
                }
            }
            Command::Fader(i, step) => {
                if let Some(f) = self.shared.faders.get(i as usize) {
                    f.store(step, Ordering::Relaxed);
                    self.engine.set_fader_adc(i as usize, fader_adc(step));
                }
            }
            Command::Midi(m) => {
                // a note-on from the surface (Play in a voice mode, Perform triggers)
                if m[0] & 0xf0 == 0x90 && m[2] > 0 {
                    let tracks = vdrum_engine::engine::tracks_for_note(m[0] & 0x0f, m[1], |p| self.engine.get_parameter(p));
                    let mutes = self.shared.mutes.load(Ordering::Relaxed);
                    if tracks != 0 && tracks & !mutes == 0 {
                        return;
                    }
                    for t in 0..7 {
                        if tracks & (1 << t) != 0 {
                            self.shared.hits[t].fetch_add(1, Ordering::Relaxed);
                        }
                    }
                }
                self.midi(&m);
            }
            Command::Seq(ev) => self.engine.seq(ev),
            Command::LoadKit(bytes) => {
                self.engine.load_kit(&bytes);
                self.sync_mirror();
            }
            Command::SeqReload => self.seq_reload_pending = true,
            Command::Panic => self.engine.all_sound_off(),
        }
    }

    /// Render one Schwung block (normally 128 stereo frames, interleaved) after applying the
    /// pending commands. DAC 2 (aux) is summed into the output. The block is rendered in engine
    /// ticks of 64 frames; frames past the last whole tick are silent.
    pub fn render(&mut self, out: &mut [i16]) {
        let t0 = Instant::now();
        for _ in 0..MAX_COMMANDS_PER_BLOCK {
            match self.shared.commands.pop() {
                Some(c) => self.apply(c),
                None => break,
            }
        }
        if self.seq_reload_pending {
            if let Ok(d) = self.shared.seq.try_lock() {
                let _ieee = IeeeGuard::new();
                self.engine.seq_load(&d);
                self.seq_reload_pending = false;
            }
        }
        {
            let _ieee = IeeeGuard::new();
            let mut chunks = out.chunks_exact_mut(TICK_FRAMES * 2);
            for chunk in &mut chunks {
                let b = self.engine.tick();
                for (i, o) in chunk.iter_mut().enumerate() {
                    *o = b.dac1[i].saturating_add(b.dac2[i]);
                }
            }
            chunks.into_remainder().fill(0);
        }
        self.after_block();
        let us = t0.elapsed().as_micros().min(u32::MAX as u128) as u32;
        let s = &self.shared;
        s.perf_last_us.store(us, Ordering::Relaxed);
        s.perf_max_us.fetch_max(us, Ordering::Relaxed);
        s.perf_sum_us.fetch_add(us as u64, Ordering::Relaxed);
        s.perf_blocks.fetch_add(1, Ordering::Relaxed);
    }

    /// Publish the playback state and count sequencer hits.
    fn after_block(&mut self) {
        let st = self.engine.seq_status();
        self.shared.seq_status.store(&st);
        for (o, v) in self.shared.lfo.iter().zip(self.engine.lfo_values()) {
            o.store(v.to_bits(), Ordering::Relaxed);
        }
        let mutes = self.shared.mutes.load(Ordering::Relaxed);
        let pat = (st.pattern as usize).min(NUM_PATTERNS - 1);
        for t in 0..NUM_TRACKS {
            let s = st.step[t];
            let new_step = st.running && !st.silent && s != self.last_step[t] && s >= 0 && (s as usize) < NUM_STEPS;
            if new_step && mutes & (1 << t) == 0 && self.engine.2.bits(pat, t) & (1u64 << s) != 0 {
                self.shared.hits[t].fetch_add(1, Ordering::Relaxed);
            }
            self.last_step[t] = s;
        }
    }
}
