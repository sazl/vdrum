// SPDX-License-Identifier: GPL-3.0-only
//! Runs the engine in 64-frame ticks against a host with arbitrary buffer sizes and sample rate.
//!
//! Timing model: output is delayed by a FIFO that starts with `P` frames of silence (`P` = 64 at
//! 44100 Hz, ≈ 64 · host rate / 44100 otherwise). A tick is run when the FIFO is empty at host time
//! `h`; before it, every pending event with host time `< h` is applied. An event that falls inside
//! a tick's time span is therefore applied at the start of that tick, and with the reported latency
//! (`P`, plus the resampler delay when resampling) the host aligns it at most 63 engine frames
//! early, never late. At 44100 Hz the output and the event placement only depend on the absolute
//! host sample position, not on the host's buffer sizes.
//!
//! Nothing here allocates after [`Pipeline::new`].

use crate::engine::{Engine, TICK_FRAMES};
use rubato::{
    calculate_cutoff, Resampler, SincFixedIn, SincInterpolationParameters, SincInterpolationType,
    WindowFunction,
};

/// Output channels: DAC1 L, DAC1 R, DAC2 L, DAC2 R.
pub const CHANNELS: usize = 4;
const FIFO_CAP: usize = 8192;
const EVENT_CAP: usize = 4096;

/// An event scheduled onto the engine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ev {
    /// A MIDI message (`len` bytes of `bytes`).
    Midi { bytes: [u8; 3], len: u8 },
    /// A parameter change. `learned`: from a MIDI-learned CC (the host parameter must follow).
    Param { idx: u16, val: u16, learned: bool },
    /// A voice fader moved (`Engine::set_fader_adc`).
    Fader { index: u8, adc: u16 },
}

impl Ev {
    /// A MIDI message from its status byte (2 bytes for program change and channel pressure).
    pub fn midi(bytes: [u8; 3]) -> Ev {
        let len = match bytes[0] & 0xf0 {
            0xc0 | 0xd0 => 2,
            _ => 3,
        };
        Ev::Midi { bytes, len }
    }
}

/// Fixed-capacity FIFO of output frames.
struct Fifo {
    buf: Box<[[f32; CHANNELS]]>,
    head: usize,
    len: usize,
}

impl Fifo {
    fn new() -> Self {
        Fifo { buf: vec![[0.0; CHANNELS]; FIFO_CAP].into_boxed_slice(), head: 0, len: 0 }
    }
    fn clear(&mut self) {
        self.head = 0;
        self.len = 0;
    }
    fn push(&mut self, f: [f32; CHANNELS]) {
        if self.len < FIFO_CAP {
            let i = (self.head + self.len) % FIFO_CAP;
            self.buf[i] = f;
            self.len += 1;
        }
    }
    fn pop(&mut self) -> Option<[f32; CHANNELS]> {
        if self.len == 0 {
            return None;
        }
        let f = self.buf[self.head];
        self.head = (self.head + 1) % FIFO_CAP;
        self.len -= 1;
        Some(f)
    }
}

/// Fixed-capacity queue of (host time, event), in arrival order (hosts deliver events sorted).
struct EventQueue {
    buf: Box<[(u64, Ev)]>,
    head: usize,
    len: usize,
}

impl EventQueue {
    fn new() -> Self {
        EventQueue {
            buf: vec![(0, Ev::Param { idx: 0, val: 0, learned: false }); EVENT_CAP].into_boxed_slice(),
            head: 0,
            len: 0,
        }
    }
    fn is_full(&self) -> bool {
        self.len == EVENT_CAP
    }
    fn push(&mut self, t: u64, ev: Ev) {
        let i = (self.head + self.len) % EVENT_CAP;
        self.buf[i] = (t, ev);
        self.len += 1;
    }
    fn pop_before(&mut self, t: u64) -> Option<Ev> {
        if self.len == 0 || self.buf[self.head].0 >= t {
            return None;
        }
        let ev = self.buf[self.head].1;
        self.head = (self.head + 1) % EVENT_CAP;
        self.len -= 1;
        Some(ev)
    }
    fn pop_any(&mut self) -> Option<Ev> {
        self.pop_before(u64::MAX)
    }
}

struct Resampling {
    rs: SincFixedIn<f32>,
    input: Vec<Vec<f32>>,
    output: Vec<Vec<f32>>,
}

/// The engine plus the scheduling and resampling around it.
pub struct Pipeline<E: Engine> {
    pub engine: Box<E>,
    host_rate: f32,
    fifo: Fifo,
    events: EventQueue,
    resampling: Option<Resampling>,
    prefill: usize,
    latency: u32,
    alt: Option<(Option<Resampling>, usize, u32)>,
    /// Absolute host sample position since the last reset.
    host_pos: u64,
}

impl<E: Engine> Pipeline<E> {
    /// Output stage for one engine rate: `(resampler, prefill, latency)`. Allocates.
    fn stage(host_rate: f32, engine_rate: f64) -> (Option<Resampling>, usize, u32) {
        if (host_rate as f64 - engine_rate).abs() < 1e-3 {
            (None, TICK_FRAMES, TICK_FRAMES as u32)
        } else {
            let ratio = host_rate as f64 / engine_rate;
            let sinc_len = 128;
            let window = WindowFunction::BlackmanHarris2;
            let params = SincInterpolationParameters {
                sinc_len,
                f_cutoff: calculate_cutoff(sinc_len, window),
                oversampling_factor: 256,
                interpolation: SincInterpolationType::Cubic,
                window,
            };
            let rs = SincFixedIn::<f32>::new(ratio, 1.0, params, TICK_FRAMES, CHANNELS)
                .expect("valid resampler parameters");
            let out_max = rs.output_frames_max();
            let prefill = (TICK_FRAMES as f64 * ratio).ceil() as usize;
            let latency = (prefill + rs.output_delay()) as u32;
            let r = Resampling {
                rs,
                input: vec![vec![0.0; TICK_FRAMES]; CHANNELS],
                output: vec![vec![0.0; out_max]; CHANNELS],
            };
            (Some(r), prefill, latency)
        }
    }

    /// Set up for a host sample rate, with the engine running at `engine_rate` (44100, or the
    /// hardware's 44140.625). Passes samples through unmodified when the rates are equal. With
    /// `alt_engine_rate`, a second output stage is prepared that [`Pipeline::swap_stage`] switches
    /// to without allocating. Allocates.
    pub fn new(engine: Box<E>, host_rate: f32, engine_rate: f64) -> Self {
        Self::with_alternate(engine, host_rate, engine_rate, None)
    }

    pub fn with_alternate(engine: Box<E>, host_rate: f32, engine_rate: f64, alt_engine_rate: Option<f64>) -> Self {
        let (resampling, prefill, latency) = Self::stage(host_rate, engine_rate);
        let alt = alt_engine_rate.map(|r| Self::stage(host_rate, r));
        let mut p = Pipeline {
            engine,
            host_rate,
            fifo: Fifo::new(),
            events: EventQueue::new(),
            resampling,
            prefill,
            latency,
            alt,
            host_pos: 0,
        };
        p.reset();
        p
    }

    /// Switch to the alternate output stage (if one was prepared) and reset the output. Does not
    /// allocate. Returns whether it switched.
    pub fn swap_stage(&mut self) -> bool {
        let Some(alt) = self.alt.as_mut() else {
            return false;
        };
        std::mem::swap(&mut self.resampling, &mut alt.0);
        std::mem::swap(&mut self.prefill, &mut alt.1);
        std::mem::swap(&mut self.latency, &mut alt.2);
        self.reset_output();
        true
    }

    /// Latency in host samples.
    pub fn latency(&self) -> u32 {
        self.latency
    }

    pub fn host_rate(&self) -> f32 {
        self.host_rate
    }

    /// Whether the output is resampled.
    pub fn is_resampling(&self) -> bool {
        self.resampling.is_some()
    }

    /// Clear the output delay line and the resampler state (not the engine state). Pending events
    /// are kept and applied at the next tick.
    pub fn reset(&mut self) {
        self.reset_output();
        // Pending events keep their order; rebase them to time 0.
        for i in 0..self.events.len {
            let j = (self.events.head + i) % EVENT_CAP;
            self.events.buf[j].0 = 0;
        }
        self.host_pos = 0;
    }

    fn reset_output(&mut self) {
        self.fifo.clear();
        for _ in 0..self.prefill {
            self.fifo.push([0.0; CHANNELS]);
        }
        if let Some(r) = &mut self.resampling {
            r.rs.reset();
        }
    }

    /// Schedule an event at sample `offset` of the next [`Pipeline::render`] call. If the queue is
    /// full the oldest pending event is applied right away.
    pub fn push_event(&mut self, offset: u32, ev: Ev, apply: &mut impl FnMut(&mut E, Ev)) {
        if self.events.is_full() {
            if let Some(old) = self.events.pop_any() {
                apply(&mut self.engine, old);
            }
        }
        self.events.push(self.host_pos + offset as u64, ev);
    }

    /// Render `frames` host frames. `apply` applies one event to the engine, `after_tick` runs
    /// after every tick, `out` receives every output frame (DAC1 L/R, DAC2 L/R) in order.
    pub fn render(
        &mut self,
        frames: usize,
        apply: &mut impl FnMut(&mut E, Ev),
        after_tick: &mut impl FnMut(&mut E, &crate::engine::Block),
        out: &mut impl FnMut(usize, [f32; CHANNELS]),
    ) {
        for i in 0..frames {
            let h = self.host_pos + i as u64;
            if self.fifo.len == 0 {
                while let Some(ev) = self.events.pop_before(h) {
                    apply(&mut self.engine, ev);
                }
                let block = {
                    let _ieee = crate::fp::IeeeGuard::new();
                    self.engine.tick()
                };
                after_tick(&mut self.engine, &block);
                self.push_block(&block);
            }
            let f = self.fifo.pop().unwrap_or([0.0; CHANNELS]);
            out(i, f);
        }
        self.host_pos += frames as u64;
    }

    fn push_block(&mut self, block: &crate::engine::Block) {
        const SCALE: f32 = 1.0 / 32768.0;
        match &mut self.resampling {
            None => {
                for i in 0..TICK_FRAMES {
                    self.fifo.push([
                        block.dac1[2 * i] as f32 * SCALE,
                        block.dac1[2 * i + 1] as f32 * SCALE,
                        block.dac2[2 * i] as f32 * SCALE,
                        block.dac2[2 * i + 1] as f32 * SCALE,
                    ]);
                }
            }
            Some(r) => {
                for i in 0..TICK_FRAMES {
                    r.input[0][i] = block.dac1[2 * i] as f32 * SCALE;
                    r.input[1][i] = block.dac1[2 * i + 1] as f32 * SCALE;
                    r.input[2][i] = block.dac2[2 * i] as f32 * SCALE;
                    r.input[3][i] = block.dac2[2 * i + 1] as f32 * SCALE;
                }
                if let Ok((_, n)) = r.rs.process_into_buffer(&r.input, &mut r.output, None) {
                    for i in 0..n {
                        self.fifo.push([r.output[0][i], r.output[1][i], r.output[2][i], r.output[3][i]]);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::{Block, Engine};

    /// Engine that outputs a ramp (frame counter) and records at which tick events arrived.
    struct Probe {
        frame: u32,
        ticks: u32,
        seen: Vec<(u32, u16)>,
    }

    impl Engine for Probe {
        fn new() -> Box<Self> {
            Box::new(Probe { frame: 0, ticks: 0, seen: Vec::new() })
        }
        fn midi_byte(&mut self, _b: u8) {}
        fn tick(&mut self) -> Block {
            let mut b = Block::default();
            for i in 0..64 {
                let v = (self.frame % 30000) as i16;
                b.dac1[2 * i] = v;
                b.dac1[2 * i + 1] = -v;
                b.dac2[2 * i] = v / 2;
                b.dac2[2 * i + 1] = 7;
                self.frame += 1;
            }
            self.ticks += 1;
            b
        }
        fn set_parameter(&mut self, param: u16, _value: u16) {
            self.seen.push((self.ticks, param));
        }
        fn get_parameter(&self, _param: u16) -> u16 {
            0
        }
        fn load_kit(&mut self, _kit: &[u8]) {}
        fn save_kit(&self) -> Vec<u8> {
            Vec::new()
        }
    }

    fn run(buffer_sizes: &[usize], events: &[(u64, u16)]) -> (Vec<[f32; 4]>, Vec<(u32, u16)>) {
        let mut p = Pipeline::new(Probe::new(), 44100.0, 44100.0);
        let mut out = Vec::new();
        let mut apply = |e: &mut Probe, ev: Ev| {
            if let Ev::Param { idx, val, .. } = ev {
                e.set_parameter(idx, val)
            }
        };
        let mut pos = 0u64;
        for &n in buffer_sizes {
            for &(t, idx) in events {
                if t >= pos && t < pos + n as u64 {
                    p.push_event((t - pos) as u32, Ev::Param { idx, val: 0, learned: false }, &mut apply);
                }
            }
            p.render(n, &mut apply, &mut |_, _| {}, &mut |_, f| out.push(f));
            pos += n as u64;
        }
        (out, p.engine.seen.clone())
    }

    #[test]
    fn passthrough_at_44100_is_exact_and_buffer_size_independent() {
        let events = [(0u64, 1u16), (63, 2), (64, 3), (100, 4), (1000, 5)];
        let (a, ea) = run(&[512, 512, 512], &events);
        let (b, eb) = run(&[1, 7, 64, 13, 500, 3, 1, 947], &events);
        assert_eq!(a.len(), 1536);
        assert_eq!(a.len(), b.len());
        assert_eq!(a, b);
        assert_eq!(ea, eb);
        // 64 frames of latency, then the samples divided by 32768, unmodified.
        assert!(a[..64].iter().all(|f| *f == [0.0; 4]));
        for (i, f) in a[64..].iter().enumerate() {
            let v = (i as u32 % 30000) as i16;
            assert_eq!(*f, [v as f32 / 32768.0, -(v as f32) / 32768.0, (v / 2) as f32 / 32768.0, 7.0 / 32768.0]);
        }
        // Events in [64k, 64k+64) are applied before tick k.
        assert_eq!(ea, vec![(0, 1), (0, 2), (1, 3), (1, 4), (15, 5)]);
    }

    #[test]
    fn resampled_output_has_expected_length_and_latency() {
        let mut p = Pipeline::new(Probe::new(), 48000.0, 44100.0);
        assert!(p.is_resampling());
        assert!(p.latency() > 70 && p.latency() < 200, "{}", p.latency());
        let mut n = 0;
        p.render(48000, &mut |_, _| {}, &mut |_, _| {}, &mut |_, _| n += 1);
        assert_eq!(n, 48000);
        // About one second of engine time was rendered.
        let ticks = p.engine.ticks;
        assert!((688..=692).contains(&ticks), "{ticks}");
    }

    #[test]
    fn hardware_clock_mode_resamples_at_44100() {
        let mut p = Pipeline::new(Probe::new(), 44100.0, crate::engine::HARDWARE_RATE);
        assert!(p.is_resampling());
        let mut n = 0;
        p.render(44100, &mut |_, _| {}, &mut |_, _| {}, &mut |_, _| n += 1);
        assert_eq!(n, 44100);
        // 44140.625 / 64 = 689.7 ticks per second
        let ticks = p.engine.ticks;
        assert!((688..=692).contains(&ticks), "{ticks}");
    }

    #[test]
    fn stage_swap_switches_between_passthrough_and_resampling() {
        let mut p = Pipeline::with_alternate(Probe::new(), 44100.0, 44100.0, Some(crate::engine::HARDWARE_RATE));
        assert!(!p.is_resampling());
        assert_eq!(p.latency(), 64);
        assert!(p.swap_stage());
        assert!(p.is_resampling());
        assert!(p.swap_stage());
        assert!(!p.is_resampling());
        assert_eq!(p.latency(), 64);
    }
}
