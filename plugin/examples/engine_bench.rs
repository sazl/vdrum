// SPDX-License-Identifier: GPL-3.0-only
//! CPU cost of the engine: `cargo run --release -p vdrum --example engine_bench [seconds]`.
//!
//! All seven tracks are retriggered every 8 ticks (about 11.6 ms) with the FX bus active (each FX
//! type in turn, wet), and the time per `tick()` (64 frames) is compared with the real-time budget
//! at 44100 Hz (1.451 ms).

use lxr_params::par;
use std::time::Instant;
use vdrum::engine::{Engine, TICK_FRAMES};
use vdrum::params::DEFAULT_TRACK_NOTES;

fn main() {
    let seconds: f64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(20.0);
    let t0 = Instant::now();
    let mut e = vdrum::new_engine();
    eprintln!("new_engine: {:.1} ms", t0.elapsed().as_secs_f64() * 1e3);
    let _ieee = vdrum::fp::IeeeGuard::new();
    let ticks_per_second = 44100.0 / TICK_FRAMES as f64;
    let budget = 1.0 / ticks_per_second;
    println!("fx_type  ticks  mean_us  max_us  realtime_fraction(mean)  (budget {:.0} us per tick)", budget * 1e6);
    let mut all = Vec::new();
    for fx in 0..5u16 {
        e.set_parameter(par::FX_TYPE, fx);
        e.set_parameter(par::FX_DRY_WET, 100);
        // All voices to the FX bus ("FX" output), FX parameters mid-range.
        for v in 0..6 {
            e.set_parameter(par::AUDIO_OUT1 + v, 6);
        }
        for p in par::FX_P1..=par::FX_P4 {
            e.set_parameter(p, 64);
        }
        let n = (seconds / 5.0 * ticks_per_second) as usize;
        let mut times = Vec::with_capacity(n);
        let mut sink = 0i64;
        for i in 0..n {
            if i % 8 == 0 {
                for note in DEFAULT_TRACK_NOTES {
                    for b in [0x90, note, 100 + (i % 27) as u8] {
                        e.midi_byte(b);
                    }
                }
            }
            let t = Instant::now();
            let b = e.tick();
            times.push(t.elapsed().as_secs_f64());
            sink += b.dac1[5] as i64 + b.dac2[7] as i64;
        }
        let mean = times.iter().sum::<f64>() / n as f64;
        let max = times.iter().cloned().fold(0.0, f64::max);
        println!("{fx:7}  {n:5}  {:7.1}  {:6.1}  {:.3}   (checksum {sink})", mean * 1e6, max * 1e6, mean / budget);
        all.extend(times);
    }
    all.sort_by(f64::total_cmp);
    let mean = all.iter().sum::<f64>() / all.len() as f64;
    let p99 = all[all.len() * 99 / 100];
    println!(
        "overall: mean {:.1} us/tick = {:.1}% of one core in real time at 44.1 kHz; p99 {:.1} us; max {:.1} us",
        mean * 1e6,
        100.0 * mean / budget,
        p99 * 1e6,
        all.last().unwrap() * 1e6
    );
}
