// SPDX-License-Identifier: GPL-3.0-only
//! The module as Schwung drives it, without Move: the C entry points, the audio output against the
//! engine run directly, no allocation on the audio thread's paths, and random input.

use std::alloc::{GlobalAlloc, Layout, System};
use std::cell::Cell;
use std::ffi::{c_char, CString};
use std::time::Duration;
use vdrum_engine::engine::{Engine, SeqEv};
use vdrum_engine::fp::IeeeGuard;
use vdrum_move::abi::PluginApiV2;
use vdrum_move::audio::Audio;
use vdrum_move::shared::{Command, Shared};

/// Counts allocations made by the current thread while counting is on.
struct Counting;

thread_local! {
    static COUNT: Cell<Option<usize>> = const { Cell::new(None) };
}

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, l: Layout) -> *mut u8 {
        let _ = COUNT.try_with(|c| c.set(c.get().map(|n| n + 1)));
        unsafe { System.alloc(l) }
    }
    unsafe fn dealloc(&self, p: *mut u8, l: Layout) {
        let _ = COUNT.try_with(|c| c.set(c.get().map(|n| n + 1)));
        unsafe { System.dealloc(p, l) }
    }
    unsafe fn realloc(&self, p: *mut u8, l: Layout, n: usize) -> *mut u8 {
        let _ = COUNT.try_with(|c| c.set(c.get().map(|n| n + 1)));
        unsafe { System.realloc(p, l, n) }
    }
}

#[global_allocator]
static A: Counting = Counting;

fn allocations(f: impl FnOnce()) -> usize {
    COUNT.with(|c| c.set(Some(0)));
    f();
    COUNT.with(|c| c.replace(None)).unwrap_or(0)
}

fn tmp(name: &str) -> std::path::PathBuf {
    let d = std::env::temp_dir().join(format!("vdrum-move-h-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    d
}

fn api() -> &'static PluginApiV2 {
    // SAFETY: the module ignores the host API.
    unsafe { &*vdrum_move::move_plugin_init_v2(std::ptr::null()) }
}

/// The module's output equals the engine's own (DAC 1 + DAC 2, saturating) for the same input:
/// pads played through `on_midi`, parameters through the command queue.
#[test]
fn output_matches_the_engine() {
    let shared = std::sync::Arc::new(Shared::new());
    let mut audio = Audio::new(vdrum_engine::defaults::new_engine(), shared.clone());
    let mut reference = vdrum_engine::defaults::new_engine();
    {
        let _g = IeeeGuard::new();
        reference.seq(SeqEv::InternalClock);
    }
    let mut out = [0i16; 256];
    let mut sound = false;
    for block in 0..400 {
        if block % 37 == 0 {
            let pad = (block / 37 % 7) as u8;
            audio.control_note([0x90, vdrum_move::hw::PAD_BASE + pad, 100]);
            let (ch, note) = vdrum_engine::engine::track_trigger_note(pad as usize, |p| reference.get_parameter(p));
            let _g = IeeeGuard::new();
            for b in [0x90 | ch, note, 100] {
                reference.midi_byte(b);
            }
        }
        if block == 100 {
            shared.send(Command::Param(lxr_params::par::OSC_WAVE_DRUM1, 2));
            let _g = IeeeGuard::new();
            reference.set_parameter(lxr_params::par::OSC_WAVE_DRUM1, 2);
        }
        audio.render(&mut out);
        sound |= out.iter().any(|s| *s != 0);
        let _g = IeeeGuard::new();
        for half in 0..2 {
            let b = reference.tick();
            for i in 0..128 {
                assert_eq!(out[half * 128 + i], b.dac1[i].saturating_add(b.dac2[i]), "block {block} frame {}", half * 64 + i / 2);
            }
        }
    }
    assert!(sound, "the pads made no sound");
}

/// The audio thread's entry points do not allocate (after the first block).
#[test]
fn audio_paths_do_not_allocate() {
    let dir = tmp("alloc");
    let api = api();
    let path = CString::new(dir.to_string_lossy().as_bytes()).unwrap();
    // SAFETY: the entry points are used as Schwung uses them.
    unsafe {
        let inst = (api.create_instance)(path.as_ptr(), std::ptr::null());
        assert!(!inst.is_null());
        let mut out = [0i16; 256];
        let mut buf = vec![0u8; 131072];
        (api.render_block)(inst, out.as_mut_ptr(), 128);
        std::thread::sleep(Duration::from_millis(60));
        let key = |s: &str| CString::new(s).unwrap();
        let (k_in, k_ui, k_perf) = (key("in"), key("ui"), key("perf"));
        let v_in = key("b04701b03101b03100b05501");
        let n = allocations(|| {
            for i in 0..50u8 {
                (api.on_midi)(inst, [0x90, 68 + i % 8, 100].as_ptr(), 3, 0);
                (api.on_midi)(inst, [0x80, 68 + i % 8, 0].as_ptr(), 3, 0);
                (api.on_midi)(inst, [0x99, 36, 100].as_ptr(), 3, 2);
                (api.set_param)(inst, k_in.as_ptr(), v_in.as_ptr());
                (api.get_param)(inst, k_ui.as_ptr(), buf.as_mut_ptr() as *mut c_char, buf.len() as i32);
                (api.get_param)(inst, k_perf.as_ptr(), buf.as_mut_ptr() as *mut c_char, buf.len() as i32);
                (api.render_block)(inst, out.as_mut_ptr(), 128);
            }
        });
        assert_eq!(n, 0, "allocations on the audio thread");
        // the frame for ui.js
        let len = (api.get_param)(inst, k_ui.as_ptr(), buf.as_mut_ptr() as *mut c_char, buf.len() as i32);
        assert!(len > 0);
        let text = std::str::from_utf8(&buf[..len as usize]).unwrap();
        assert_eq!(text.lines().nth(1).map(str::len), Some(vdrum_move::hw::NUM_LEDS * 2));
        (api.destroy_instance)(inst);
    }
    assert!(dir.join("state/seq.txt").is_file(), "the session is saved on destroy");
    let _ = std::fs::remove_dir_all(&dir);
}

/// Commands the worker sends during play do not allocate on the audio thread.
#[test]
fn commands_do_not_allocate() {
    let shared = std::sync::Arc::new(Shared::new());
    let mut audio = Audio::new(vdrum_engine::defaults::new_engine(), shared.clone());
    let mut out = [0i16; 256];
    audio.render(&mut out);
    let step = vdrum_engine::seq::StepData { on: true, velocity: 100, ..Default::default() };
    let n = allocations(|| {
        for i in 0..100u16 {
            shared.send(Command::Param(i % 200, i % 100));
            shared.send(Command::Fader((i % 7) as u8, (i % 100) as i32));
            shared.send(Command::Midi([0x90, 36, 100]));
            shared.send(Command::Seq(SeqEv::Step { pat: 0, track: (i % 7) as u8, step: (i % 64) as u8, data: step }));
            shared.send(Command::Seq(SeqEv::Run(i % 50 == 0)));
            audio.render(&mut out);
        }
    });
    assert_eq!(n, 0);
}

/// Random bytes into every entry point and the worker: nothing panics or hangs.
#[test]
fn random_input() {
    let dir = tmp("fuzz");
    let api = api();
    let path = CString::new(dir.to_string_lossy().as_bytes()).unwrap();
    let mut x: u64 = 0x9e3779b97f4a7c15;
    let mut rnd = move || {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        x
    };
    // SAFETY: the entry points are used as Schwung uses them.
    unsafe {
        let inst = (api.create_instance)(path.as_ptr(), std::ptr::null());
        let mut out = [0i16; 256];
        let mut buf = [0u8; 4096];
        let ui = CString::new("ui").unwrap();
        let k_in = CString::new("in").unwrap();
        for round in 0..3000 {
            let r = rnd();
            let msg = [0x80 | (r as u8 & 0x7f), (r >> 8) as u8 & 0x7f, (r >> 16) as u8 & 0x7f];
            (api.on_midi)(inst, msg.as_ptr(), 3, (r >> 24) as i32 & 2);
            // controls as ui.js forwards them: CCs and notes of Move's range
            let cc = [3u8, 14, 40, 41, 42, 43, 49, 50, 51, 52, 54, 55, 56, 58, 60, 62, 63, 71, 74, 78, 85, 86, 88, 119];
            let pick = cc[(r >> 32) as usize % cc.len()];
            let val = if r >> 40 & 1 == 0 { (r >> 48) as u8 & 0x7f } else { 0 };
            let note = 16 + ((r >> 33) as u8 % 84);
            let s = format!("b0{pick:02x}{val:02x}{:02x}{note:02x}{val:02x}", if val > 0 { 0x90 } else { 0x80 });
            let v = CString::new(s).unwrap();
            (api.set_param)(inst, k_in.as_ptr(), v.as_ptr());
            (api.render_block)(inst, out.as_mut_ptr(), 128);
            (api.get_param)(inst, ui.as_ptr(), buf.as_mut_ptr() as *mut c_char, buf.len() as i32);
            if round % 100 == 0 {
                std::thread::sleep(Duration::from_millis(3));
            }
        }
        (api.destroy_instance)(inst);
    }
    let _ = std::fs::remove_dir_all(&dir);
}
