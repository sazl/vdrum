// SPDX-License-Identifier: GPL-3.0-only
//! The Launch Control XL 3 controller thread.
//!
//! The thread opens the device's DAW port with `midir`, switches the device into DAW mode and runs
//! a [`vdrum_surface::Surface`]: decoded controller input becomes parameter changes, fader moves
//! and triggers, which reach the audio thread through the same lock-free command queue the editor
//! uses; LEDs and the screen follow the shared parameter mirror, so they also show host automation,
//! editor edits and kit loads. The thread never touches the audio thread's data directly.
//!
//! * The surface runs without a device too, so the editor's mode tabs work (and accept mouse edits)
//!   with no controller connected. It publishes a [`vdrum_surface::View`] in
//!   [`Shared::surface_view`] whenever what it shows changes, and takes the editor's mouse actions
//!   from [`Shared::surface_input`].
//! * One plugin instance per process owns the device; others leave it alone.
//! * `VDRUM_LCXL3=0` disables the controller. `VDRUM_LCXL3_PORT=<text>` selects the ports whose
//!   names contain `<text>` instead of the automatic choice.
//! * `VDRUM_LCXL3_LOG=<file>` appends every message received from the device to `<file>` (time in
//!   milliseconds, the bytes in hex and how the plugin decoded them), for checking what the device
//!   sends on real hardware.
//! * The device is looked for every [`SCAN_MS`] while it is not connected, and checked for at the
//!   same interval while it is.

use crate::engine::track_trigger_note;
use crate::params::{Command, EditorInput, Shared, N};
use lcxl3::Input;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use lxr_params::kit::Kit;
use std::path::{Path, PathBuf};
use crate::engine::SeqEv;
use vdrum_surface::{Action, DeviceState, Model, SeqData, SeqEdit, SeqStatus, Settings, Surface, Switch};

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
    /// Starts the controller thread, unless it is disabled (`VDRUM_LCXL3=0`). The thread uses the
    /// device only while no other instance in this process does.
    pub fn start(shared: Arc<Shared>) -> Option<Controller> {
        if std::env::var("VDRUM_LCXL3").is_ok_and(|v| v == "0") {
            return None;
        }
        let stop = Arc::new(AtomicBool::new(false));
        let s = stop.clone();
        let thread = std::thread::Builder::new()
            .name("vdrum-lcxl3".into())
            .spawn(move || {
                shared.surface_running.store(true, Ordering::Relaxed);
                run(&shared, &s);
                shared.surface_running.store(false, Ordering::Relaxed);
                shared.controller_connected.store(false, Ordering::Relaxed);
            })
            .ok()?;
        Some(Controller { stop, thread: Some(thread) })
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
    fn switch(&self, s: Switch) -> bool {
        match s {
            Switch::SumAux => self.0.sum_aux_on.load(Ordering::Relaxed),
            Switch::HwClock => self.0.hw_clock_on.load(Ordering::Relaxed),
        }
    }
    fn seq(&self) -> Box<dyn std::ops::Deref<Target = SeqData> + '_> {
        // A poisoned lock still holds valid data (edits are applied whole).
        Box::new(self.0.seq.lock().unwrap_or_else(|e| e.into_inner()))
    }
    fn lfo(&self, t: u8) -> f32 {
        self.0.lfo_values.get(t as usize).map_or(0.5, |v| f32::from_bits(v.load(Ordering::Relaxed)))
    }

    fn seq_status(&self) -> SeqStatus {
        self.0.seq_status.load()
    }
    fn mutes(&self) -> u8 {
        self.0.mutes.load(Ordering::Relaxed)
    }
    fn tempo(&self) -> f64 {
        self.0.tempo.load(Ordering::Relaxed) as f64
    }
}

/// The engine operations that bring the engine's copy of the sequencer data up to date after
/// `edit` was applied to `d`.
pub fn seq_events(d: &SeqData, edit: &SeqEdit) -> Vec<SeqEv> {
    match *edit {
        SeqEdit::Step { pat, track, step, .. } => vec![SeqEv::Step { pat, track, step, data: d.step(pat, track, step) }],
        SeqEdit::Euclid { pat, track, .. } => vec![SeqEv::Bits { pat, track, bits: d.track(pat, track).bits() }],
        SeqEdit::Length { pat, track, .. } => {
            let t = d.track(pat, track);
            let mut v = vec![SeqEv::Length { pat, track, length: t.length }];
            if t.fills > 0 {
                v.push(SeqEv::Bits { pat, track, bits: t.bits() });
            }
            v
        }
        SeqEdit::Shuffle { pat, .. } => {
            let value = d.patterns.get(pat as usize).map_or(0, |p| p.shuffle);
            vec![SeqEv::Shuffle { pat, value }]
        }
        SeqEdit::Density { track, .. } => {
            let value = d.density.get(track as usize).copied().unwrap_or(d.density_all);
            vec![SeqEv::Density { track, value }]
        }
        SeqEdit::NextPattern(p) => vec![SeqEv::NextPattern(p.min(63))],
        SeqEdit::Run(on) => vec![SeqEv::Run(on)],
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
        Action::Panic => {
            let _ = shared.commands.push(Command::Panic);
            None
        }
        Action::SetSwitch(sw, on) => {
            let _ = shared.commands.push(Command::Switch(sw == Switch::HwClock, on));
            None
        }
        Action::InitKit => {
            let k = Kit::init();
            shared.set_name(&k.name);
            let _ = shared.commands.push(Command::LoadKit(k.to_file_bytes()));
            None
        }
        Action::Seq(edit) => {
            let evs = {
                let mut d = shared.seq.lock().unwrap_or_else(|e| e.into_inner());
                d.apply(&edit);
                seq_events(&d, &edit)
            };
            for ev in evs {
                let _ = shared.commands.push(Command::Seq(ev));
            }
            None
        }
        Action::SetMutes(m) => {
            shared.mutes.store(m & 0x7f, Ordering::Relaxed);
            let _ = shared.commands.push(Command::Seq(SeqEv::Mutes(m & 0x7f)));
            None
        }
        Action::LfoReset => {
            let _ = shared.commands.push(Command::Seq(SeqEv::LfoReset));
            None
        }
        Action::SeqHostClock(host) => {
            shared.seq_host_clock.store(host, Ordering::Relaxed);
            None
        }
        // Handled by `Session` (file system and state).
        Action::KitBrowse(_) | Action::KitLoadBrowsed | Action::SaveKit | Action::Compare(_) => None,
    }
}

/// The kit files (`.snd`) in a folder, sorted by name.
pub fn kit_files(folder: &Path) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = std::fs::read_dir(folder)
        .map(|d| {
            d.filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| p.is_file() && p.extension().is_some_and(|x| x.eq_ignore_ascii_case("snd")))
                .collect()
        })
        .unwrap_or_default();
    v.sort();
    v
}

fn load_kit_file(shared: &Shared, path: &Path) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let kit = Kit::from_file_bytes(&bytes).map_err(|e| e.to_string())?;
    shared.set_name(&kit.name);
    shared.deselect_library();
    shared.set_kit_file(Some(path.to_path_buf()), kit.to_file_bytes().to_vec());
    let _ = shared.commands.push(Command::LoadKit(kit.to_file_bytes()));
    Ok(kit.name_str())
}

fn file_label(p: &Path) -> String {
    p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()
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
    /// The kit file shown by Page up/down (library empty).
    browsed: Option<PathBuf>,
    /// The library slot shown by Page up/down.
    browsed_slot: Option<usize>,
    /// The edited kit while *Compare* plays the saved one.
    compare_stash: Option<Kit>,
    /// The view last published for the editor.
    last_view: Option<vdrum_surface::View>,
}

impl Session {
    pub fn new(shared: &Shared) -> Session {
        let mut surface = Surface::new();
        surface.settings = load_settings(shared);
        if let Ok(s) = shared.surface_state.lock() {
            surface.load_state(&s);
        }
        Session { surface, device: DeviceState::default(), next_frame: 0, browsed: None, browsed_slot: None, compare_stash: None, last_view: None }
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

    /// Carries out a surface action; returns device messages to send.
    pub fn perform(&mut self, shared: &Shared, action: Action) -> Option<Vec<u8>> {
        let kit_path = || shared.kit_file.lock().ok().and_then(|f| f.path.clone());
        match action {
            Action::KitBrowse(dir) => {
                // the library's filled slots when there are any, otherwise the kit files in the
                // folder of the current kit file
                let lib = shared.library.lock().ok().filter(|l| !l.is_empty()).map(|l| {
                    l.step(self.browsed_slot.or(l.selected), dir as i32).map(|s| (s, l.slots[s].as_ref().map(|e| e.kit.name_str()).unwrap_or_default()))
                });
                if let Some(found) = lib {
                    if let Some((s, name)) = found {
                        self.surface.notify("Kit browse", &format!("Kit {s:02} · {}", name.trim_end()), "Play loads");
                        self.browsed_slot = Some(s);
                    }
                    return None;
                }
                let Some(cur) = self.browsed.clone().or_else(kit_path) else {
                    self.surface.notify("Kit browse", "no kit folder", "load a kit in the editor");
                    return None;
                };
                let folder = cur.parent().map(Path::to_path_buf).unwrap_or_default();
                let files = kit_files(&folder);
                if files.is_empty() {
                    self.surface.notify("Kit browse", "no kits", &folder.display().to_string());
                    return None;
                }
                let n = files.len() as i32;
                let i = match files.iter().position(|f| *f == cur) {
                    Some(i) => (i as i32 + dir as i32).rem_euclid(n),
                    None => 0,
                } as usize;
                self.surface.notify("Kit browse", &file_label(&files[i]), "Play loads");
                self.browsed = Some(files[i].clone());
                None
            }
            Action::KitLoadBrowsed => {
                if let Some(s) = self.browsed_slot.take() {
                    match shared.load_library_slot(s) {
                        Some(label) => self.surface.notify("Kit loaded", "Library", &label),
                        None => self.surface.notify("Kit not loaded", "Library", &format!("{s:02} is empty")),
                    }
                } else if let Some(p) = self.browsed.take() {
                    match load_kit_file(shared, &p) {
                        Ok(name) => self.surface.notify("Kit loaded", &file_label(&p), &name),
                        Err(e) => self.surface.notify("Kit not loaded", &file_label(&p), &e),
                    }
                }
                None
            }
            Action::SaveKit => {
                // a kit from the library is saved into its slot only
                if let Some(s) = shared.library_selected() {
                    if let Some(label) = shared.store_in_library(s) {
                        self.surface.notify("Kit saved", "Library", &label);
                    }
                    return None;
                }
                let Some(p) = kit_path() else {
                    self.surface.notify("Save kit", "no kit file", "save once in the editor");
                    return None;
                };
                let bytes = shared.kit().to_file_bytes();
                match std::fs::write(&p, bytes) {
                    Ok(()) => {
                        shared.set_kit_file(Some(p.clone()), bytes.to_vec());
                        self.surface.notify("Kit saved", &file_label(&p), "");
                    }
                    Err(e) => self.surface.notify("Kit not saved", &file_label(&p), &e.to_string()),
                }
                None
            }
            Action::Compare(true) => {
                let saved = shared.kit_file.lock().ok().and_then(|f| f.saved.clone());
                match saved.and_then(|b| Kit::from_file_bytes(&b).ok()) {
                    Some(k) => {
                        self.compare_stash = Some(shared.kit());
                        shared.set_name(&k.name);
                        let _ = shared.commands.push(Command::LoadKit(k.to_file_bytes()));
                    }
                    None => self.surface.notify("Compare", "no saved kit", "load or save a kit first"),
                }
                None
            }
            Action::Compare(false) => {
                if let Some(k) = self.compare_stash.take() {
                    shared.set_name(&k.name);
                    let _ = shared.commands.push(Command::LoadKit(k.to_file_bytes()));
                }
                None
            }
            a => apply(shared, a),
        }
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
                out.extend(self.perform(shared, a));
            }
        }
        // mouse actions from the editor's mode tabs
        while let Some(i) = shared.surface_input.pop() {
            match i {
                EditorInput::Input(input) => {
                    for a in self.surface.handle_editor(&input, t, &model) {
                        out.extend(self.perform(shared, a));
                    }
                }
                EditorInput::SetMode(m) => self.surface.set_mode(m),
            }
        }
        for a in self.surface.tick(t, &model) {
            out.extend(self.perform(shared, a));
        }
        save_settings(shared, &self.surface.settings);
        if self.surface.take_state_changed() {
            if let Ok(mut cur) = shared.surface_state.lock() {
                *cur = self.surface.state_text();
            }
        }
        if t >= self.next_frame {
            self.next_frame = t + FRAME_MS;
            out.extend(self.device.update(&self.surface.frame(t, &model)));
            self.publish_view(shared, t);
        }
        out
    }
}

impl Session {
    /// Publishes the surface's view for the editor when it changed.
    fn publish_view(&mut self, shared: &Shared, t: u64) {
        let view = self.surface.view(t, &SharedModel(shared));
        if self.last_view.as_ref() != Some(&view) {
            if let Ok(mut v) = shared.surface_view.lock() {
                *v = Some(Arc::new(view.clone()));
            }
            shared.surface_view_gen.fetch_add(1, Ordering::Relaxed);
            self.last_view = Some(view);
        }
    }
}

/// Appends the received messages to the file named by `VDRUM_LCXL3_LOG`, if set.
struct InputLog(Option<std::fs::File>);

impl InputLog {
    fn from_env() -> InputLog {
        let file = std::env::var_os("VDRUM_LCXL3_LOG")
            .filter(|p| !p.is_empty())
            .and_then(|p| std::fs::OpenOptions::new().create(true).append(true).open(p).ok());
        InputLog(file)
    }

    fn note(&mut self, t: u64, text: &str) {
        if let Some(f) = self.0.as_mut() {
            use std::io::Write;
            let _ = writeln!(f, "{t:>9} {text}");
        }
    }

    fn received(&mut self, t: u64, msgs: &[Vec<u8>]) {
        if self.0.is_none() {
            return;
        }
        for m in msgs {
            let hex: Vec<String> = m.iter().map(|b| format!("{b:02X}")).collect();
            let text = format!("IN  {}  {:?}", hex.join(" "), lcxl3::decode(m));
            self.note(t, &text);
        }
    }
}

fn run(shared: &Shared, stop: &AtomicBool) {
    let start = Instant::now();
    let now = || start.elapsed().as_millis() as u64;
    let mut log = InputLog::from_env();
    let mut session = Session::new(shared);
    let mut conn: Option<Connection> = None;
    let mut next_scan = 0u64;
    while !stop.load(Ordering::Relaxed) {
        let t = now();
        if t >= next_scan {
            next_scan = t + SCAN_MS;
            if conn.as_ref().is_some_and(|c| !c.present()) {
                conn = None;
                OWNER.store(false, Ordering::Release);
            }
            // one instance per process drives the device
            if conn.is_none() && OWNER.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire).is_ok() {
                let opened = Connection::open().and_then(|mut c| c.send_all(session.connect()).then_some(c));
                if opened.is_none() {
                    OWNER.store(false, Ordering::Release);
                }
                if let Some(c) = &opened {
                    log.note(t, &format!("connected: in {:?}", c.in_name));
                }
                conn = opened;
            }
            shared.controller_connected.store(conn.is_some(), Ordering::Relaxed);
        }
        match conn.as_mut() {
            Some(c) => {
                let received: Vec<Vec<u8>> = c.rx.try_iter().collect();
                log.received(t, &received);
                let out = session.step(shared, t, received);
                if !out.is_empty() && !c.send_all(out) {
                    conn = None;
                    OWNER.store(false, Ordering::Release);
                    shared.controller_connected.store(false, Ordering::Relaxed);
                    continue;
                }
                std::thread::sleep(POLL);
            }
            None => {
                // no device: the surface still runs for the editor's mode tabs
                let _ = session.step(shared, t, std::iter::empty());
                std::thread::sleep(POLL * 5);
            }
        }
    }
    if let Some(mut c) = conn {
        c.send_all(session.disconnect());
        OWNER.store(false, Ordering::Release);
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

        // sequencer edits change the shared copy and send the engine the resulting data
        apply(&shared, Action::Seq(SeqEdit::Euclid { pat: 1, track: 2, fills: 4, rotate: 0 }));
        assert_eq!(SharedModel(&shared).seq().track(1, 2).bits(), 0x1111);
        assert!(matches!(shared.commands.pop(), Some(Command::Seq(SeqEv::Bits { pat: 1, track: 2, bits: 0x1111 }))));
        apply(&shared, Action::Seq(SeqEdit::Length { pat: 1, track: 2, length: 8 }));
        assert!(matches!(shared.commands.pop(), Some(Command::Seq(SeqEv::Length { length: 8, .. }))));
        assert!(matches!(shared.commands.pop(), Some(Command::Seq(SeqEv::Bits { bits: 0x55, .. }))));
        apply(&shared, Action::Seq(SeqEdit::Density { track: 7, value: 120 }));
        assert!(matches!(shared.commands.pop(), Some(Command::Seq(SeqEv::Density { track: 7, value: 100 }))));
        apply(&shared, Action::SetMutes(0xff));
        assert_eq!(SharedModel(&shared).mutes(), 0x7f);
        assert!(matches!(shared.commands.pop(), Some(Command::Seq(SeqEv::Mutes(0x7f)))));
        apply(&shared, Action::SeqHostClock(false));
        assert!(!shared.seq_host_clock.load(Ordering::Relaxed));
        // out of range: ignored
        apply(&shared, Action::SetParam { idx: 9999, value: 1 });
        assert!(shared.commands.pop().is_none());
    }

    #[test]
    fn kit_browsing_saving_and_compare() {
        let dir = std::env::temp_dir().join(format!("vdrum-kits-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let mut a = Kit::init();
        a.set_name("KIT A");
        a.values[par::COARSE1 as usize] = 10;
        let mut b = Kit::init();
        b.set_name("KIT B");
        b.values[par::COARSE1 as usize] = 20;
        std::fs::write(dir.join("01-A.snd"), a.to_file_bytes()).unwrap();
        std::fs::write(dir.join("02-B.SND"), b.to_file_bytes()).unwrap();
        std::fs::write(dir.join("notes.txt"), b"x").unwrap();
        assert_eq!(kit_files(&dir).len(), 2);

        let shared = Shared::new();
        let mut s = Session::new(&shared);
        // no kit file yet
        s.perform(&shared, Action::KitBrowse(1));
        assert!(s.browsed.is_none());
        shared.set_kit_file(Some(dir.join("01-A.snd")), a.to_file_bytes().to_vec());
        s.perform(&shared, Action::KitBrowse(1));
        assert_eq!(s.browsed.as_deref(), Some(dir.join("02-B.SND").as_path()));
        s.perform(&shared, Action::KitBrowse(1)); // wraps
        assert_eq!(s.browsed.as_deref(), Some(dir.join("01-A.snd").as_path()));
        s.perform(&shared, Action::KitBrowse(-1));
        s.perform(&shared, Action::KitLoadBrowsed);
        assert_eq!(shared.kit_name(), "KIT B");
        assert!(matches!(shared.commands.pop(), Some(Command::LoadKit(k)) if k[..] == b.to_file_bytes()[..]));
        assert_eq!(shared.kit_file.lock().unwrap().path.as_deref(), Some(dir.join("02-B.SND").as_path()));

        // save writes the mirror's kit to the current file
        shared.values[par::COARSE1 as usize].store(33, Ordering::Relaxed);
        s.perform(&shared, Action::SaveKit);
        let saved = Kit::from_file_bytes(&std::fs::read(dir.join("02-B.SND")).unwrap()).unwrap();
        assert_eq!(saved.values[par::COARSE1 as usize], 33);

        // compare plays the saved kit and returns to the edited one
        shared.values[par::COARSE1 as usize].store(44, Ordering::Relaxed);
        s.perform(&shared, Action::Compare(true));
        assert!(matches!(shared.commands.pop(), Some(Command::LoadKit(k)) if Kit::from_file_bytes(&k).unwrap().values[par::COARSE1 as usize] == 33));
        s.perform(&shared, Action::Compare(false));
        assert!(matches!(shared.commands.pop(), Some(Command::LoadKit(k)) if Kit::from_file_bytes(&k).unwrap().values[par::COARSE1 as usize] == 44));

        s.perform(&shared, Action::Panic);
        assert!(matches!(shared.commands.pop(), Some(Command::Panic)));
        s.perform(&shared, Action::SetSwitch(Switch::HwClock, true));
        assert!(matches!(shared.commands.pop(), Some(Command::Switch(true, true))));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn library_browsing_loading_and_saving() {
        let shared = Shared::new();
        {
            let mut l = shared.library.lock().unwrap();
            for (slot, name, coarse) in [(2usize, "TWO", 12u8), (5, "FIVE", 15)] {
                let mut k = Kit::init();
                k.set_name(name);
                k.values[par::COARSE1 as usize] = coarse;
                l.slots[slot] = Some(crate::library::Entry { kit: k, source: String::new() });
            }
        }
        let mut s = Session::new(&shared);
        // Page down walks the filled slots and wraps
        s.perform(&shared, Action::KitBrowse(1));
        assert_eq!(s.browsed_slot, Some(2));
        s.perform(&shared, Action::KitBrowse(1));
        assert_eq!(s.browsed_slot, Some(5));
        s.perform(&shared, Action::KitBrowse(1));
        assert_eq!(s.browsed_slot, Some(2));
        assert!(shared.commands.pop().is_none(), "browsing loads nothing");
        // Play loads the browsed slot and selects it
        s.perform(&shared, Action::KitLoadBrowsed);
        assert_eq!(shared.kit_name(), "TWO");
        assert_eq!(shared.library_selected(), Some(2));
        assert!(matches!(shared.commands.pop(), Some(Command::LoadKit(k)) if Kit::from_file_bytes(&k).unwrap().values[par::COARSE1 as usize] == 12));
        // browsing continues from the selected slot
        s.perform(&shared, Action::KitBrowse(1));
        assert_eq!(s.browsed_slot, Some(5));
        // Save kit writes the slot, not a file
        shared.values[par::COARSE1 as usize].store(40, Ordering::Relaxed);
        s.perform(&shared, Action::SaveKit);
        let l = shared.library.lock().unwrap();
        assert_eq!(l.slots[2].as_ref().unwrap().kit.values[par::COARSE1 as usize], 40);
        assert_eq!(l.slots[5].as_ref().unwrap().kit.values[par::COARSE1 as usize], 15);
        drop(l);
        assert!(shared.kit_file.lock().unwrap().path.is_none());
        // Compare plays the slot's saved kit
        shared.values[par::COARSE1 as usize].store(41, Ordering::Relaxed);
        s.perform(&shared, Action::Compare(true));
        assert!(matches!(shared.commands.pop(), Some(Command::LoadKit(k)) if Kit::from_file_bytes(&k).unwrap().values[par::COARSE1 as usize] == 40));
        s.perform(&shared, Action::Compare(false));
        let _ = shared.commands.pop();
        // loading a kit file deselects the library
        shared.deselect_library();
        assert_eq!(shared.library_selected(), None);
    }

    #[test]
    fn editor_input_and_published_view() {
        let shared = Shared::new();
        let mut s = Session::new(&shared);
        // no device: stepping still runs the surface and publishes the view
        s.step(&shared, 0, []);
        let g0 = shared.surface_view_gen.load(Ordering::Relaxed);
        let v = shared.surface_view.lock().unwrap().clone().unwrap();
        assert_eq!(v.mode, vdrum_surface::Mode::Voice(0));
        // an encoder turned with the mouse
        let fine = lxr_params::by_id("fine1").unwrap().index;
        let v0 = shared.value(fine);
        let _ = shared.surface_input.push(EditorInput::Input(Input::Relative { row: 0, col: 0, delta: 2 }));
        s.step(&shared, 30, []);
        assert_eq!(shared.value(fine), v0 + 2);
        assert!(matches!(shared.commands.pop(), Some(Command::Param(p, _)) if p == fine));
        let v = shared.surface_view.lock().unwrap().clone().unwrap();
        assert!(v.edit.is_none(), "editor edits do not show the edit pill");
        assert!(shared.surface_view_gen.load(Ordering::Relaxed) > g0);
        // a tab click sets the mode
        let _ = shared.surface_input.push(EditorInput::SetMode(vdrum_surface::Mode::Sequencer));
        s.step(&shared, 60, []);
        assert_eq!(shared.surface_view.lock().unwrap().as_ref().unwrap().mode, vdrum_surface::Mode::Sequencer);
        // a hardware edit shows it
        s.step(&shared, 90, [vec![0xBF, 77 + 1, 65]]);
        let v = shared.surface_view.lock().unwrap().clone().unwrap();
        assert!(v.edit.is_some());
        // in mode 8 the selected track's knob LEDs pulse, so the view keeps changing
        let g = shared.surface_view_gen.load(Ordering::Relaxed);
        s.step(&shared, 400, []);
        assert!(shared.surface_view_gen.load(Ordering::Relaxed) > g);
        // elsewhere, with nothing changed: no new snapshot
        let _ = shared.surface_input.push(EditorInput::SetMode(vdrum_surface::Mode::Voice(0)));
        s.step(&shared, 430, []);
        let g = shared.surface_view_gen.load(Ordering::Relaxed);
        s.step(&shared, 500, []);
        s.step(&shared, 530, []);
        assert_eq!(shared.surface_view_gen.load(Ordering::Relaxed), g);
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
