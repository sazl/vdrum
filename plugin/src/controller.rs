// SPDX-License-Identifier: GPL-3.0-only
//! The Launch Control XL 3 controller thread.
//!
//! The thread opens the device's DAW port with `midir`, switches the device into DAW mode and runs
//! a [`vdrum_surface::Surface`]: decoded controller input becomes parameter changes, fader moves
//! and triggers, which reach the audio thread through the same lock-free command queue the editor
//! uses; LEDs and the screen follow the shared parameter mirror, so they also show host automation,
//! editor edits and kit loads. The thread never touches the audio thread's data directly.
//!
//! * One plugin instance per process owns the device; others leave it alone.
//! * `VDRUM_LCXL3=0` disables the controller. `VDRUM_LCXL3_PORT=<text>` selects the ports whose
//!   names contain `<text>` instead of the automatic choice.
//! * The device is looked for every [`SCAN_MS`] while it is not connected, and checked for at the
//!   same interval while it is.

use crate::engine::track_trigger_note;
use crate::params::{Command, Shared, N};
use lcxl3::Input;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use vdrum_surface::{Action, DeviceState, Model, Settings, Surface};

/// Interval between device scans, in milliseconds.
pub const SCAN_MS: u64 = 2000;
/// Interval between LED and screen updates, in milliseconds.
pub const FRAME_MS: u64 = 20;
/// Sleep between polls of the input queue.
const POLL: Duration = Duration::from_millis(2);

/// Set while a plugin instance in this process drives the device.
static OWNER: AtomicBool = AtomicBool::new(false);

/// A running controller thread; stopping it (drop) returns the device to standalone mode.
pub struct Controller {
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl Controller {
    /// Starts the controller thread, unless it is disabled or another instance owns the device.
    pub fn start(shared: Arc<Shared>) -> Option<Controller> {
        if std::env::var("VDRUM_LCXL3").is_ok_and(|v| v == "0") {
            return None;
        }
        if OWNER.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire).is_err() {
            return None;
        }
        let stop = Arc::new(AtomicBool::new(false));
        let s = stop.clone();
        let thread = std::thread::Builder::new()
            .name("vdrum-lcxl3".into())
            .spawn(move || {
                run(&shared, &s);
                OWNER.store(false, Ordering::Release);
            })
            .ok();
        if thread.is_none() {
            OWNER.store(false, Ordering::Release);
            return None;
        }
        Some(Controller { stop, thread })
    }
}

impl Drop for Controller {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

/// The plugin state as the surface reads it.
pub struct SharedModel<'a>(pub &'a Shared);

impl Model for SharedModel<'_> {
    fn param(&self, idx: u16) -> u16 {
        if (idx as usize) < N {
            self.0.value(idx)
        } else {
            0
        }
    }
    fn voice_fader(&self, t: u8) -> i32 {
        self.0.fader_values.get(t as usize).map_or(0, |f| f.load(Ordering::Relaxed))
    }
    fn kit_name(&self) -> String {
        self.0.kit_name()
    }
    fn hits(&self, t: u8) -> u32 {
        self.0.pad_hits.get(t as usize).map_or(0, |h| h.load(Ordering::Relaxed))
    }
}

/// Carries out a surface action. Device messages are returned for the caller to send.
pub fn apply(shared: &Shared, action: Action) -> Option<Vec<u8>> {
    match action {
        Action::SetParam { idx, value } => {
            if (idx as usize) < N {
                // The mirror is updated at once so that the next encoder detent starts from the
                // new value; the flag makes the editor copy it to the host parameter and the saved
                // state use it.
                shared.values[idx as usize].store(value, Ordering::Relaxed);
                shared.dirty[idx as usize].store(true, Ordering::Relaxed);
                let _ = shared.commands.push(Command::Param(idx, value));
            }
            None
        }
        Action::SetVoiceFader { track, step } => {
            let i = track as usize;
            if i < shared.fader_values.len() {
                shared.fader_values[i].store(step, Ordering::Relaxed);
                shared.extra_values[i].store(step, Ordering::Relaxed);
                shared.extra_dirty[i].store(true, Ordering::Relaxed);
                let _ = shared.commands.push(Command::Fader(track, step));
            }
            None
        }
        Action::Trigger { track, velocity } => {
            let (ch, note) = track_trigger_note(track as usize, |p| shared.value(p));
            let _ = shared.commands.push(Command::Midi([0x90 | ch, note, velocity.clamp(1, 127)]));
            None
        }
        Action::Device(m) => Some(m),
    }
}

/// Picks the input/output port to use from the ports' names: names containing `filter` if given,
/// otherwise names that look like a Launch Control XL 3, preferring one with "DAW" in its name and
/// otherwise the device's second port (its DAW interface on Windows).
///
/// Unverified: the port names the device reports on each OS.
pub fn pick_port(names: &[String], filter: Option<&str>) -> Option<usize> {
    let lower: Vec<String> = names.iter().map(|n| n.to_lowercase()).collect();
    if let Some(f) = filter {
        let f = f.to_lowercase();
        return lower.iter().position(|n| n.contains(&f));
    }
    let device: Vec<usize> = (0..names.len())
        .filter(|&i| {
            let n = &lower[i];
            n.contains("lcxl3") || (n.contains("launch control xl") && n.contains('3'))
        })
        .collect();
    device.iter().copied().find(|&i| lower[i].contains("daw")).or_else(|| device.get(1).copied())
}

struct Connection {
    _input: midir::MidiInputConnection<()>,
    output: midir::MidiOutputConnection,
    rx: Receiver<Vec<u8>>,
    in_name: String,
}

impl Connection {
    fn open() -> Option<Connection> {
        let filter = std::env::var("VDRUM_LCXL3_PORT").ok();
        let input = midir::MidiInput::new("VDrum LCXL3 in").ok()?;
        let output = midir::MidiOutput::new("VDrum LCXL3 out").ok()?;
        let in_ports = input.ports();
        let in_names: Vec<String> = in_ports.iter().map(|p| input.port_name(p).unwrap_or_default()).collect();
        let out_ports = output.ports();
        let out_names: Vec<String> = out_ports.iter().map(|p| output.port_name(p).unwrap_or_default()).collect();
        let i = pick_port(&in_names, filter.as_deref())?;
        let o = pick_port(&out_names, filter.as_deref())?;
        let (tx, rx) = channel();
        let conn_in = input
            .connect(&in_ports[i], "vdrum-lcxl3-in", move |_, msg, _| {
                let _ = tx.send(msg.to_vec());
            }, ())
            .ok()?;
        let conn_out = output.connect(&out_ports[o], "vdrum-lcxl3-out").ok()?;
        Some(Connection { _input: conn_in, output: conn_out, rx, in_name: in_names[i].clone() })
    }

    fn send_all(&mut self, msgs: impl IntoIterator<Item = Vec<u8>>) -> bool {
        msgs.into_iter().all(|m| self.output.send(&m).is_ok())
    }

    /// Whether the input port is still listed.
    fn present(&self) -> bool {
        let Ok(input) = midir::MidiInput::new("VDrum LCXL3 probe") else { return true };
        input.ports().iter().any(|p| input.port_name(p).is_ok_and(|n| n == self.in_name))
    }
}

fn load_settings(shared: &Shared) -> Settings {
    shared.surface_settings.lock().map(|s| Settings::from_text(&s)).unwrap_or_default()
}

fn save_settings(shared: &Shared, s: &Settings) {
    if let Ok(mut cur) = shared.surface_settings.lock() {
        let t = s.to_text();
        if *cur != t {
            *cur = t;
        }
    }
}

/// The controller's logic without the MIDI ports: device messages in, device messages out.
pub struct Session {
    pub surface: Surface,
    device: DeviceState,
    next_frame: u64,
}

impl Session {
    pub fn new(shared: &Shared) -> Session {
        let mut surface = Surface::new();
        surface.settings = load_settings(shared);
        Session { surface, device: DeviceState::default(), next_frame: 0 }
    }

    /// The messages to send when the device was found.
    pub fn connect(&mut self) -> Vec<Vec<u8>> {
        self.next_frame = 0;
        self.device.connect().into_iter().chain(self.surface.device_settings()).collect()
    }

    /// The messages to send before the device is released.
    pub fn disconnect(&mut self) -> Vec<Vec<u8>> {
        self.device.disconnect()
    }

    /// Handles the messages received from the device and the passage of time (`t` in
    /// milliseconds); returns the messages to send to the device.
    pub fn step(&mut self, shared: &Shared, t: u64, received: impl IntoIterator<Item = Vec<u8>>) -> Vec<Vec<u8>> {
        let model = SharedModel(shared);
        let mut out = Vec::new();
        for msg in received {
            let input = lcxl3::decode(&msg);
            if let Input::SurfaceMode(lcxl3::SurfaceMode::DawMixer | lcxl3::SurfaceMode::DawControl) = input {
                // Back from one of the device's own modes: redraw everything.
                self.device.invalidate();
            }
            for a in self.surface.handle(&input, t, &model) {
                out.extend(apply(shared, a));
            }
        }
        for a in self.surface.tick(t, &model) {
            out.extend(apply(shared, a));
        }
        save_settings(shared, &self.surface.settings);
        if t >= self.next_frame {
            self.next_frame = t + FRAME_MS;
            out.extend(self.device.update(&self.surface.frame(t, &model)));
        }
        out
    }
}

fn run(shared: &Shared, stop: &AtomicBool) {
    let start = Instant::now();
    let now = || start.elapsed().as_millis() as u64;
    let mut session = Session::new(shared);
    let mut conn: Option<Connection> = None;
    let mut next_scan = 0u64;
    while !stop.load(Ordering::Relaxed) {
        let t = now();
        if t >= next_scan {
            next_scan = t + SCAN_MS;
            if conn.as_ref().is_some_and(|c| !c.present()) {
                conn = None;
            }
            if conn.is_none() {
                if let Some(mut c) = Connection::open() {
                    if c.send_all(session.connect()) {
                        conn = Some(c);
                    }
                }
            }
        }
        let Some(c) = conn.as_mut() else {
            std::thread::sleep(POLL * 10);
            continue;
        };
        let received: Vec<Vec<u8>> = c.rx.try_iter().collect();
        let out = session.step(shared, t, received);
        if !out.is_empty() && !c.send_all(out) {
            conn = None;
            continue;
        }
        std::thread::sleep(POLL);
    }
    if let Some(mut c) = conn {
        c.send_all(session.disconnect());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lxr_params::par;

    fn names(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn port_choice() {
        assert_eq!(pick_port(&names(&["Midi Through", "LCXL3 1 MIDI In", "LCXL3 1 DAW In"]), None), Some(2));
        assert_eq!(pick_port(&names(&["LCXL3 1 MIDI", "LCXL3 1 DAW"]), None), Some(1));
        // Windows-style names without "DAW": the second port of the device
        assert_eq!(pick_port(&names(&["Launch Control XL 3", "MIDIIN2 (Launch Control XL 3)"]), None), Some(1));
        assert_eq!(pick_port(&names(&["Launch Control XL 3"]), None), None);
        assert_eq!(pick_port(&names(&["Launch Control XL"]), None), None, "the first-generation device");
        assert_eq!(pick_port(&names(&["Midi Through", "My Port"]), Some("my port")), Some(1));
    }

    #[test]
    fn actions_reach_the_command_queue_and_the_mirror() {
        let shared = Shared::new();
        assert_eq!(apply(&shared, Action::SetParam { idx: par::COARSE1, value: 77 }), None);
        assert_eq!(shared.value(par::COARSE1), 77);
        assert!(shared.dirty[par::COARSE1 as usize].load(Ordering::Relaxed));
        assert!(matches!(shared.commands.pop(), Some(Command::Param(p, 77)) if p == par::COARSE1));

        apply(&shared, Action::SetVoiceFader { track: 6, step: 40 });
        assert_eq!(SharedModel(&shared).voice_fader(6), 40);
        assert!(shared.extra_dirty[6].load(Ordering::Relaxed));
        assert!(matches!(shared.commands.pop(), Some(Command::Fader(6, 40))));

        apply(&shared, Action::Trigger { track: 0, velocity: 90 });
        let (ch, note) = track_trigger_note(0, |p| shared.value(p));
        assert!(matches!(shared.commands.pop(), Some(Command::Midi(m)) if m == [0x90 | ch, note, 90]));

        assert_eq!(apply(&shared, Action::Device(vec![1, 2, 3])), Some(vec![1, 2, 3]));
        // out of range: ignored
        apply(&shared, Action::SetParam { idx: 9999, value: 1 });
        assert!(shared.commands.pop().is_none());
    }

    #[test]
    fn session_drives_the_device_and_the_plugin() {
        let shared = Shared::new();
        let mut s = Session::new(&shared);
        let init = s.connect();
        assert_eq!(init[0], vec![0x9F, 0x0C, 0x7F], "DAW mode first");
        // first frame: LEDs and the stationary display
        let first = s.step(&shared, 0, []);
        assert!(first.len() > 40);
        assert!(s.step(&shared, 1, []).is_empty(), "nothing new before the next frame");
        // encoder 1 of row 1 in mode 1 (Drum 1 fine), two steps clockwise
        let fine = lxr_params::by_id("fine1").unwrap().index;
        let v0 = shared.value(fine);
        let out = s.step(&shared, 30, [vec![0xBF, 77, 66]]);
        assert_eq!(shared.value(fine), v0 + 2);
        assert!(matches!(shared.commands.pop(), Some(Command::Param(p, v)) if p == fine && v == v0 + 2));
        assert!(!out.is_empty(), "popup");
        // Shift + bottom button 1 = mode 9; the frame redraws the LEDs
        s.step(&shared, 40, [vec![0xB6, 63, 127], vec![0xB0, 45, 127], vec![0xB0, 45, 0], vec![0xB6, 63, 0]]);
        assert_eq!(s.surface.mode(), vdrum_surface::Mode::MixFx);
        // a settings change is saved with the plugin state
        s.surface.settings.pad_velocity = 33;
        s.step(&shared, 100, []);
        assert!(shared.surface_settings.lock().unwrap().contains("pad_velocity=33"));
        assert_eq!(s.disconnect().last(), Some(&vec![0x9F, 0x0C, 0x00]));
    }
}
