// SPDX-License-Identifier: GPL-3.0-only
//! VDrum on Ableton Move: a Schwung (1.9.2) takeover module. This crate builds `dsp.so`, which runs
//! the VDrum engine and sequencer ([`vdrum_engine`]) on Move's audio callback and VDrum's surface
//! ([`vdrum_surface`], every mode of the Launch Control XL 3 design) on Move's pads, buttons and
//! knobs. `ui.js` (in `schwung/vdrum/`) forwards Move's controls and draws the LEDs and the display.
//!
//! * [`audio`]: the audio thread (engine, commands, the voice pads).
//! * [`worker`]: the worker thread (surface, files, frames for `ui.js`).
//! * [`adapter`]: Move's controls on the surface.
//! * [`files`]: `kits/`, `patterns/` and `state/` in the module folder.
//! * [`abi`]: Schwung's C interface; the entry points are below.
//!
//! `set_param` keys: `in` (Move control messages from `ui.js`, as hex, 3 bytes each), `save`
//! (save the session). `get_param` keys: `ui` (the frame: revision, LEDs, display lines), `perf`
//! (render time), `dropped` (messages lost to full queues).

pub mod abi;
pub mod adapter;
pub mod audio;
pub mod files;
pub mod hw;
pub mod model;
pub mod shared;
pub mod worker;

use abi::{HostApiV1, PluginApiV2, MIDI_SOURCE_EXTERNAL, MOVE_PLUGIN_API_VERSION_2};
use audio::Audio;
use shared::{Shared, SRC_JS};
use std::cell::UnsafeCell;
use std::ffi::{c_char, c_int, c_void, CStr};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use worker::WorkerThread;

/// One module instance: the audio side and the worker thread.
pub struct Instance {
    audio: UnsafeCell<Audio>,
    shared: Arc<Shared>,
    /// Stopping the worker (drop) saves the session.
    _worker: Option<WorkerThread>,
}

impl Instance {
    /// A new instance for the module folder `dir`. Allocates and starts the worker thread.
    pub fn new(dir: &std::path::Path) -> Instance {
        let shared = Arc::new(Shared::new());
        let audio = Audio::new(vdrum_engine::defaults::new_engine(), shared.clone());
        let worker = WorkerThread::start(shared.clone(), files::Store::new(dir)).ok();
        Instance { audio: UnsafeCell::new(audio), shared, _worker: worker }
    }

    /// The audio side. Schwung calls every entry point but create and destroy from the audio
    /// callback, one at a time.
    #[allow(clippy::mut_from_ref)]
    fn audio(&self) -> &mut Audio {
        // SAFETY: only the audio callback calls the entry points that use this, never two at once.
        unsafe { &mut *self.audio.get() }
    }

    pub fn shared(&self) -> &Shared {
        &self.shared
    }

    /// A MIDI message from the host: Move's pads and steps (`source` internal) or USB-A MIDI.
    pub fn on_midi(&self, msg: &[u8], source: c_int) {
        if source == MIDI_SOURCE_EXTERNAL {
            self.audio().external_midi(msg);
            return;
        }
        if let &[s, n, v] = msg {
            if matches!(s & 0xf0, 0x80 | 0x90) {
                self.shared.direct_notes.store(true, Ordering::Relaxed);
                self.audio().control_note([s, n, v]);
            }
        }
    }

    pub fn set_param(&self, key: &[u8], val: &[u8]) {
        match key {
            b"in" => self.input_hex(val),
            b"save" => self.shared.save_requested.store(true, Ordering::Relaxed),
            _ => {}
        }
    }

    /// Control messages from `ui.js`: 6 hex digits per message. Notes are dropped once they reach
    /// the DSP directly; otherwise the voice row is played by the worker.
    fn input_hex(&self, val: &[u8]) {
        let hex = |c: u8| match c {
            b'0'..=b'9' => Some(c - b'0'),
            b'a'..=b'f' => Some(c - b'a' + 10),
            b'A'..=b'F' => Some(c - b'A' + 10),
            _ => None,
        };
        let direct = self.shared.direct_notes.load(Ordering::Relaxed);
        for chunk in val.chunks_exact(6) {
            let mut m = [0u8; 3];
            let mut ok = true;
            for (i, b) in m.iter_mut().enumerate() {
                match (hex(chunk[2 * i]), hex(chunk[2 * i + 1])) {
                    (Some(h), Some(l)) => *b = h << 4 | l,
                    _ => ok = false,
                }
            }
            if !ok || m[0] < 0x80 {
                continue;
            }
            if direct && matches!(m[0] & 0xf0, 0x80 | 0x90) {
                continue;
            }
            self.shared.push_input([m[0], m[1] & 0x7f, m[2] & 0x7f, SRC_JS]);
        }
    }

    /// Write the answer for `key` into `buf`; returns its length, or -1. Does not allocate.
    pub fn get_param(&self, key: &[u8], buf: &mut [u8]) -> c_int {
        let mut w = BufWriter { buf, len: 0 };
        use std::fmt::Write;
        let s = &self.shared;
        let ok = match key {
            b"ui" => match s.ui.try_lock() {
                Ok(ui) => w.write_str(&ui).is_ok(),
                // the worker is writing a frame: no answer this time
                Err(_) => return 0,
            },
            b"perf" => {
                let blocks = s.perf_blocks.load(Ordering::Relaxed).max(1);
                let mean = s.perf_sum_us.load(Ordering::Relaxed) / blocks;
                write!(
                    w,
                    "last={} mean={} max={} blocks={}",
                    s.perf_last_us.load(Ordering::Relaxed),
                    mean,
                    s.perf_max_us.load(Ordering::Relaxed),
                    s.perf_blocks.load(Ordering::Relaxed)
                )
                .is_ok()
            }
            b"dropped" => write!(w, "{}", s.dropped.load(Ordering::Relaxed)).is_ok(),
            _ => return -1,
        };
        if !ok {
            return -1;
        }
        let n = w.len;
        if n < w.buf.len() {
            w.buf[n] = 0;
        }
        n as c_int
    }

    pub fn render(&self, out: &mut [i16]) {
        self.audio().render(out);
    }
}

/// `fmt::Write` into a byte buffer, keeping one byte for the terminating NUL; fails when full.
struct BufWriter<'a> {
    buf: &'a mut [u8],
    len: usize,
}

impl std::fmt::Write for BufWriter<'_> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        let b = s.as_bytes();
        if self.len + b.len() + 1 > self.buf.len() {
            return Err(std::fmt::Error);
        }
        self.buf[self.len..self.len + b.len()].copy_from_slice(b);
        self.len += b.len();
        Ok(())
    }
}

// ---- C entry points ----

/// # Safety
/// `p` is null or a NUL-terminated string.
unsafe fn cstr<'a>(p: *const c_char) -> &'a [u8] {
    if p.is_null() {
        &[]
    } else {
        // SAFETY: the caller passes a NUL-terminated string.
        unsafe { CStr::from_ptr(p) }.to_bytes()
    }
}

unsafe extern "C" fn create_instance(module_dir: *const c_char, _defaults: *const c_char) -> *mut c_void {
    let r = catch_unwind(|| {
        // SAFETY: the host passes a NUL-terminated path.
        let dir = String::from_utf8_lossy(unsafe { cstr(module_dir) }).to_string();
        Box::into_raw(Box::new(Instance::new(std::path::Path::new(&dir)))) as *mut c_void
    });
    r.unwrap_or(std::ptr::null_mut())
}

unsafe extern "C" fn destroy_instance(inst: *mut c_void) {
    if inst.is_null() {
        return;
    }
    // SAFETY: `inst` came from `create_instance` and is destroyed once.
    let b = unsafe { Box::from_raw(inst as *mut Instance) };
    let _ = catch_unwind(AssertUnwindSafe(|| drop(b)));
}

unsafe extern "C" fn on_midi(inst: *mut c_void, msg: *const u8, len: c_int, source: c_int) {
    if inst.is_null() || msg.is_null() || len <= 0 {
        return;
    }
    // SAFETY: `inst` is a live instance; `msg` holds `len` bytes.
    let (i, m) = unsafe { (&*(inst as *const Instance), std::slice::from_raw_parts(msg, len as usize)) };
    let _ = catch_unwind(AssertUnwindSafe(|| i.on_midi(m, source)));
}

unsafe extern "C" fn set_param(inst: *mut c_void, key: *const c_char, val: *const c_char) {
    if inst.is_null() {
        return;
    }
    // SAFETY: `inst` is a live instance; key and value are NUL-terminated strings.
    let (i, k, v) = unsafe { (&*(inst as *const Instance), cstr(key), cstr(val)) };
    let _ = catch_unwind(AssertUnwindSafe(|| i.set_param(k, v)));
}

unsafe extern "C" fn get_param(inst: *mut c_void, key: *const c_char, buf: *mut c_char, len: c_int) -> c_int {
    if inst.is_null() || buf.is_null() || len <= 0 {
        return -1;
    }
    // SAFETY: `inst` is a live instance; `buf` holds `len` bytes.
    let (i, k, b) = unsafe { (&*(inst as *const Instance), cstr(key), std::slice::from_raw_parts_mut(buf as *mut u8, len as usize)) };
    catch_unwind(AssertUnwindSafe(|| i.get_param(k, b))).unwrap_or(-1)
}

unsafe extern "C" fn get_error(_inst: *mut c_void, buf: *mut c_char, len: c_int) -> c_int {
    if !buf.is_null() && len > 0 {
        // SAFETY: `buf` holds at least one byte.
        unsafe { *buf = 0 };
    }
    0
}

unsafe extern "C" fn render_block(inst: *mut c_void, out: *mut i16, frames: c_int) {
    if out.is_null() || frames <= 0 {
        return;
    }
    // SAFETY: `out` holds `frames` stereo frames.
    let o = unsafe { std::slice::from_raw_parts_mut(out, frames as usize * 2) };
    if inst.is_null() {
        o.fill(0);
        return;
    }
    // SAFETY: `inst` is a live instance.
    let i = unsafe { &*(inst as *const Instance) };
    if catch_unwind(AssertUnwindSafe(|| i.render(o))).is_err() {
        o.fill(0);
    }
}

static API: PluginApiV2 = PluginApiV2 {
    api_version: MOVE_PLUGIN_API_VERSION_2,
    create_instance,
    destroy_instance,
    on_midi,
    set_param,
    get_param,
    get_error,
    render_block,
};

/// The entry point Schwung looks up (`MOVE_PLUGIN_INIT_V2_SYMBOL`).
///
/// # Safety
/// Called by the host with a valid `host_api_v1_t` (unused: the module needs none of its services).
#[no_mangle]
pub unsafe extern "C" fn move_plugin_init_v2(_host: *const HostApiV1) -> *const PluginApiV2 {
    &API
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_input_and_params() {
        let dir = std::env::temp_dir().join(format!("vdrum-move-lib-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let i = Instance::new(&dir);
        // stop the worker so that the queue can be read here
        let Instance { audio, shared, _worker } = i;
        drop(_worker);
        let i = Instance { audio, shared, _worker: None };
        i.set_param(b"in", b"b03101b03100zz9044647f");
        let got: Vec<_> = std::iter::from_fn(|| i.shared.input.pop()).collect();
        assert_eq!(got, vec![[0xb0, 0x31, 0x01, SRC_JS], [0xb0, 0x31, 0x00, SRC_JS]]);
        // after a direct note, forwarded notes are dropped
        i.on_midi(&[0x90, 70, 100], 0);
        let _ = std::iter::from_fn(|| i.shared.input.pop()).count();
        i.set_param(b"in", b"904464b03101");
        let got: Vec<_> = std::iter::from_fn(|| i.shared.input.pop()).collect();
        assert_eq!(got, vec![[0xb0, 0x31, 0x01, SRC_JS]]);
        let mut buf = [0u8; 64];
        let n = i.get_param(b"perf", &mut buf);
        assert!(n > 0 && buf[n as usize] == 0);
        assert_eq!(i.get_param(b"nope", &mut buf), -1);
        let mut small = [0u8; 4];
        assert_eq!(i.get_param(b"perf", &mut small), -1);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
