// SPDX-License-Identifier: GPL-3.0-only
//! The module's worker thread: runs the surface on Move's controls ([`Adapter`]), does all file I/O
//! (kits, patterns, the session) and publishes the LED and display frame for `ui.js`.
//!
//! The thread is started by `create_instance` (on the shim's loader thread) and moves itself to
//! normal scheduling on cores 0–2, away from the audio callback's core 3. It never blocks the audio
//! thread: it talks to it through [`Shared`]'s queues and atomics, and the locks it takes are only
//! ever `try_lock`ed by the audio side.

use crate::adapter::{Adapter, Out, LINES};
use crate::files::{Session, Store};
use crate::hw;
use crate::model::{apply, load_kit, SharedModel};
use crate::shared::{Command, Shared};
use lxr_params::kit::Kit;
use std::fmt::Write as _;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};
use vdrum_surface::{Action, Settings};

/// Interval between frames for `ui.js`, in milliseconds.
pub const FRAME_MS: u64 = 20;
/// Sleep between polls of the input queue.
const POLL: Duration = Duration::from_millis(2);
/// The session is saved this long after the last change, in milliseconds.
pub const AUTOSAVE_MS: u64 = 10_000;

/// The worker's state (everything but the thread), driven by [`Worker::step`].
pub struct Worker {
    shared: Arc<Shared>,
    pub adapter: Adapter,
    pub store: Store,
    /// The current kit's file in `kits/`, and its contents when loaded or saved (for Compare).
    kit_file: Option<PathBuf>,
    kit_saved: Option<Kit>,
    browsed: Option<PathBuf>,
    compare_stash: Option<Kit>,
    next_frame: u64,
    rev: u32,
    last_frame: String,
    /// Time of the first change not saved yet.
    dirty_since: Option<u64>,
    last_kit: Option<Kit>,
    last_seq_text: String,
    last_settings: Settings,
}

impl Worker {
    /// Load the session from `store` into the shared state (and from there into the engine).
    pub fn new(shared: Arc<Shared>, store: Store) -> Worker {
        store.ensure_dirs();
        let session = store.load();
        let mut adapter = Adapter::new();
        if let Some(t) = &session.settings {
            adapter.surface.settings = Settings::from_text(t);
            Adapter::fix_settings(&mut adapter.surface);
        }
        if let Some(t) = &session.surface {
            adapter.surface.load_state(t);
        }
        let kit_file = session.kit_file.as_ref().map(|n| store.kits_dir().join(n)).filter(|p| p.is_file());
        let kit_saved = kit_file.as_ref().and_then(|p| Store::read_kit(p).ok());
        // the edited kit, else the first kit file, else the engine's init kit
        let kit = session.kit.clone().or_else(|| kit_saved.clone());
        let (kit, kit_file, kit_saved) = match kit {
            Some(k) => (Some(k), kit_file, kit_saved),
            None => match store.kit_files().first() {
                Some(p) => match Store::read_kit(p) {
                    Ok(k) => (Some(k.clone()), Some(p.clone()), Some(k)),
                    Err(_) => (None, None, None),
                },
                None => (None, None, None),
            },
        };
        if let Some(k) = &kit {
            load_kit(&shared, k);
        }
        if let Some(f) = session.faders {
            for (t, step) in f.into_iter().enumerate() {
                apply(&shared, Action::SetVoiceFader { track: t as u8, step });
            }
        }
        apply(&shared, Action::SetMutes(session.mutes));
        if let Ok(mut d) = shared.seq.lock() {
            *d = session.seq.clone();
        }
        shared.send(Command::SeqReload);
        if !session.imported.is_empty() {
            let n = session.imported.len();
            adapter.surface.notify("Patterns", &format!("{n} loaded from"), "patterns/");
        }
        let last_settings = adapter.surface.settings;
        Worker {
            shared,
            adapter,
            store,
            kit_file,
            kit_saved,
            browsed: None,
            compare_stash: None,
            next_frame: 0,
            rev: 0,
            last_frame: String::new(),
            dirty_since: None,
            last_kit: kit,
            last_seq_text: session.seq.to_text(),
            last_settings,
        }
    }

    fn notify(&mut self, title: &str, name: &str, value: &str) {
        self.adapter.surface.notify(title, name, value);
    }

    fn file_label(p: &std::path::Path) -> String {
        p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()
    }

    /// Carry out the adapter's output.
    fn perform(&mut self, out: Out) {
        match out {
            Out::Accent(on) => self.shared.accent.store(on, Ordering::Relaxed),
            Out::Save => match self.save_all() {
                Ok(()) => {
                    let name = self.kit_file.as_deref().map(Self::file_label).unwrap_or_default();
                    self.notify("Saved", &name, "kit + patterns");
                }
                Err(e) => self.notify("Not saved", "", &e),
            },
            Out::Act(a) => {
                if let Some(a) = apply(&self.shared, a) {
                    self.files(a);
                }
            }
        }
    }

    /// Kit browsing, loading, saving and Compare.
    fn files(&mut self, a: Action) {
        match a {
            Action::KitBrowse(dir) => {
                let files = self.store.kit_files();
                if files.is_empty() {
                    self.notify("Kit browse", "no kits in", "kits/");
                    return;
                }
                let n = files.len() as i32;
                let cur = self.browsed.clone().or_else(|| self.kit_file.clone());
                let i = match cur.and_then(|c| files.iter().position(|f| *f == c)) {
                    Some(i) => (i as i32 + dir as i32).rem_euclid(n),
                    None => 0,
                } as usize;
                self.notify("Kit browse", &Self::file_label(&files[i]), "click loads");
                self.browsed = Some(files[i].clone());
            }
            Action::KitLoadBrowsed => {
                let Some(p) = self.browsed.take() else { return };
                match Store::read_kit(&p) {
                    Ok(k) => {
                        load_kit(&self.shared, &k);
                        self.notify("Kit loaded", &Self::file_label(&p), &k.name_str());
                        self.kit_file = Some(p);
                        self.kit_saved = Some(k);
                        self.compare_stash = None;
                    }
                    Err(e) => self.notify("Kit not loaded", &Self::file_label(&p), &e),
                }
            }
            Action::SaveKit => {
                if let Err(e) = self.save_kit() {
                    self.notify("Kit not saved", "", &e);
                } else {
                    let name = self.kit_file.as_deref().map(Self::file_label).unwrap_or_default();
                    self.notify("Kit saved", &name, "");
                }
            }
            Action::Compare(true) => match self.kit_saved.clone() {
                Some(k) => {
                    self.compare_stash = Some(self.shared.kit());
                    load_kit(&self.shared, &k);
                }
                None => self.notify("Compare", "no saved kit", "save with Capture"),
            },
            Action::Compare(false) => {
                if let Some(k) = self.compare_stash.take() {
                    load_kit(&self.shared, &k);
                }
            }
            _ => {}
        }
    }

    /// Write the kit to its file (a new `NN-NAME.SND` if it has none).
    fn save_kit(&mut self) -> Result<(), String> {
        if self.compare_stash.is_some() {
            return Err("comparing".into());
        }
        let kit = self.shared.kit();
        let path = self.kit_file.clone().unwrap_or_else(|| self.store.new_kit_path(&kit));
        self.store.ensure_dirs();
        Store::write_kit(&path, &kit).map_err(|e| e.to_string())?;
        self.kit_file = Some(path);
        self.kit_saved = Some(kit);
        Ok(())
    }

    /// The session as it stands.
    fn session(&self) -> Session {
        Session {
            // while comparing, the edited kit is the one put aside
            kit: Some(self.compare_stash.clone().unwrap_or_else(|| self.shared.kit())),
            kit_file: self.kit_file.as_deref().map(Self::file_label),
            seq: self.shared.seq.lock().map(|d| d.clone()).unwrap_or_else(|e| e.into_inner().clone()),
            settings: Some(self.adapter.surface.settings.to_text()),
            surface: Some(self.adapter.surface.state_text()),
            faders: Some(std::array::from_fn(|t| self.shared.faders[t].load(Ordering::Relaxed))),
            mutes: self.shared.mutes.load(Ordering::Relaxed),
            imported: Vec::new(),
        }
    }

    /// Save the kit file, the patterns and the session (Capture).
    pub fn save_all(&mut self) -> Result<(), String> {
        let kit = self.save_kit();
        kit.and(self.save_session())
    }

    /// Save the patterns and the session; the kit file is only written by Capture, the edited kit
    /// is kept in `state/kit.snd`.
    pub fn save_session(&mut self) -> Result<(), String> {
        let s = self.session();
        self.dirty_since = None;
        self.store.save(&s)
    }

    /// Save the session (not the kit file) when it changed and has been left alone for a while, so
    /// that a power cut loses little.
    fn autosave(&mut self, now: u64) {
        let kit = self.shared.kit();
        let seq_text = self.shared.seq.lock().map(|d| d.to_text()).unwrap_or_default();
        let settings = self.adapter.surface.settings;
        let state_changed = self.adapter.surface.take_state_changed();
        if self.last_kit.as_ref() != Some(&kit) || seq_text != self.last_seq_text || settings != self.last_settings || state_changed {
            self.last_kit = Some(kit);
            self.last_seq_text = seq_text;
            self.last_settings = settings;
            self.dirty_since.get_or_insert(now);
        }
        if self.dirty_since.is_some_and(|t| now.saturating_sub(t) >= AUTOSAVE_MS) {
            let _ = self.save_session();
        }
    }

    /// Handle the queued input and the passage of time (`now` in milliseconds); publish a frame
    /// every [`FRAME_MS`].
    pub fn step(&mut self, now: u64) {
        let mut outs = Vec::new();
        while let Some(m) = self.shared.input.pop() {
            outs.extend(self.adapter.handle(m, now, &SharedModel(&self.shared)));
            self.shared.mute_held.store(self.adapter.mute_held(), Ordering::Relaxed);
            for o in outs.drain(..) {
                self.perform(o);
            }
        }
        for o in self.adapter.tick(now, &SharedModel(&self.shared)) {
            self.perform(o);
        }
        if self.shared.save_requested.swap(false, Ordering::Relaxed) {
            let _ = self.save_session();
        }
        if now >= self.next_frame {
            self.next_frame = now + FRAME_MS;
            self.autosave(now);
            self.publish(now);
        }
    }

    /// The frame for `ui.js`: `rev`, the LEDs as two hex digits each in [`hw::led`] order, and
    /// the display lines, separated by newlines. `rev` changes whenever the rest does.
    fn publish(&mut self, now: u64) {
        let accent = self.shared.accent.load(Ordering::Relaxed);
        let leds = self.adapter.leds(now, &SharedModel(&self.shared), accent);
        let lines = self.adapter.lines(now, &SharedModel(&self.shared));
        let mut body = String::with_capacity(hw::NUM_LEDS * 2 + LINES * 24);
        for v in leds {
            let _ = write!(body, "{v:02x}");
        }
        for l in &lines {
            body.push('\n');
            body.push_str(l);
        }
        if body == self.last_frame {
            return;
        }
        self.rev = self.rev.wrapping_add(1);
        if let Ok(mut ui) = self.shared.ui.lock() {
            ui.clear();
            let _ = write!(ui, "{}\n{}", self.rev, body);
        }
        self.last_frame = body;
    }
}

/// Move the calling thread off the audio core and out of real-time scheduling (the loader thread
/// that starts it may be real-time).
fn demote() {
    #[cfg(target_os = "linux")]
    // SAFETY: plain system calls on the calling thread with valid arguments.
    unsafe {
        let p = libc::sched_param { sched_priority: 0 };
        libc::sched_setscheduler(0, libc::SCHED_OTHER, &p);
        let mut set: libc::cpu_set_t = std::mem::zeroed();
        for c in 0..3 {
            libc::CPU_SET(c, &mut set);
        }
        libc::sched_setaffinity(0, std::mem::size_of::<libc::cpu_set_t>(), &set);
    }
}

/// The running worker thread; dropping it stops the thread, which saves first.
pub struct WorkerThread {
    stop: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}

impl WorkerThread {
    pub fn start(shared: Arc<Shared>, store: Store) -> std::io::Result<WorkerThread> {
        let stop = Arc::new(AtomicBool::new(false));
        let s = stop.clone();
        let thread = std::thread::Builder::new().name("vdrum-move".into()).spawn(move || {
            demote();
            let t0 = Instant::now();
            let mut w = Worker::new(shared, store);
            while !s.load(Ordering::Relaxed) {
                w.step(t0.elapsed().as_millis() as u64);
                std::thread::sleep(POLL);
            }
            let _ = w.save_session();
        })?;
        Ok(WorkerThread { stop, thread: Some(thread) })
    }
}

impl Drop for WorkerThread {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::SRC_JS;

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("vdrum-move-w-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d
    }

    #[test]
    fn browse_load_save_and_frame() {
        let dir = tmp("browse");
        let store = Store::new(&dir);
        store.ensure_dirs();
        let mut a = Kit::init();
        a.set_name("ALPHA");
        let mut b = Kit::init();
        b.set_name("BETA");
        b.values[0] = b.values[0].wrapping_add(1);
        Store::write_kit(&store.kits_dir().join("00-ALPHA.SND"), &a).unwrap();
        Store::write_kit(&store.kits_dir().join("01-BETA.SND"), &b).unwrap();
        let shared = Arc::new(Shared::new());
        let mut w = Worker::new(shared.clone(), store.clone());
        // the first kit file is loaded when there is no session
        assert_eq!(&shared.kit_name_bytes()[..5], b"ALPHA");
        // wheel right, click: BETA
        shared.push_input([0xb0, hw::CC_WHEEL, 1, SRC_JS]);
        shared.push_input([0xb0, hw::CC_WHEEL_CLICK, 127, SRC_JS]);
        w.step(0);
        assert_eq!(&shared.kit_name_bytes()[..4], b"BETA");
        assert!(std::iter::from_fn(|| shared.commands.pop()).any(|c| matches!(c, Command::LoadKit(_))));
        // a frame was published
        let ui = shared.ui.lock().unwrap().clone();
        let mut parts = ui.split('\n');
        assert_eq!(parts.next(), Some("1"));
        assert_eq!(parts.next().map(str::len), Some(hw::NUM_LEDS * 2));
        assert_eq!(parts.count(), LINES);
        // Capture saves the session
        shared.push_input([0xb0, hw::CC_CAPTURE, 127, SRC_JS]);
        w.step(30);
        assert!(dir.join("state/seq.txt").is_file());
        assert!(dir.join("state/session.txt").is_file());
        let s = store.load();
        assert_eq!(s.kit_file.as_deref(), Some("01-BETA.SND"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn thread_starts_and_saves_on_stop() {
        let dir = tmp("thread");
        let shared = Arc::new(Shared::new());
        let t = WorkerThread::start(shared.clone(), Store::new(&dir)).unwrap();
        std::thread::sleep(Duration::from_millis(50));
        drop(t);
        assert!(dir.join("state/seq.txt").is_file());
        assert!(!shared.ui.lock().unwrap().is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
