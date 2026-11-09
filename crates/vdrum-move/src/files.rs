// SPDX-License-Identifier: GPL-3.0-only
//! Files in the module's folder on Move (`/data/UserData/schwung/modules/overtake/vdrum/`):
//!
//! * `kits/` — kits as VDrum `.snd` files (the desktop plugin's format, 297 bytes), browsed with the
//!   jog wheel. New kits are saved as `NN-NAME.SND`.
//! * `patterns/` — the sequencer's patterns as LXR-02 `PatV03` files (`NN-NAME.PAT`, `NN` = pattern
//!   number − 1, as the firmware and the desktop plugin's export name them).
//! * `state/` — the session: the edited kit (`kit.snd`), the sequencer data (`seq.txt`, lossless),
//!   the surface settings and snapshots/macros, and which kit file is current, faders and mutes
//!   (`session.txt`).
//!
//! On start the sequencer data comes from `state/seq.txt`; a `.PAT` file that is newer than it (copied
//! into `patterns/` from a computer) replaces its pattern. Saving writes every non-empty pattern to
//! `patterns/` first and `seq.txt` last. Files are written to a temporary name and renamed.

use lxr_params::kit::Kit;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use vdrum_engine::library::{slot_from_name, snd_files, Library};
use vdrum_engine::lxr_files::{file_name, name_bytes, pat_files, pat_from_bytes, pat_to_bytes, plan_patterns, read_files, PatFile};
use vdrum_engine::seq::{SeqData, NUM_PATTERNS};

/// The module's folders.
#[derive(Clone, Debug)]
pub struct Store {
    pub dir: PathBuf,
}

/// What `state/` holds.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Session {
    /// The kit as last edited.
    pub kit: Option<Kit>,
    /// File name (in `kits/`) the kit was loaded from or saved to.
    pub kit_file: Option<String>,
    pub seq: SeqData,
    /// [`vdrum_surface::Settings::to_text`].
    pub settings: Option<String>,
    /// [`vdrum_surface::Surface::state_text`].
    pub surface: Option<String>,
    pub faders: Option<[i32; 7]>,
    pub mutes: u8,
    /// Patterns replaced from `.PAT` files on load, as (pattern number − 1).
    pub imported: Vec<usize>,
}

/// Write `bytes` to `path` through a temporary file in the same folder.
fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, bytes)?;
    std::fs::rename(&tmp, path)
}

fn mtime(p: &Path) -> Option<SystemTime> {
    std::fs::metadata(p).and_then(|m| m.modified()).ok()
}

fn file_label(p: &Path) -> String {
    p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()
}

impl Store {
    pub fn new(dir: impl Into<PathBuf>) -> Store {
        Store { dir: dir.into() }
    }

    pub fn kits_dir(&self) -> PathBuf {
        self.dir.join("kits")
    }

    pub fn patterns_dir(&self) -> PathBuf {
        self.dir.join("patterns")
    }

    pub fn state_dir(&self) -> PathBuf {
        self.dir.join("state")
    }

    /// Create the folders that do not exist.
    pub fn ensure_dirs(&self) {
        for d in [self.kits_dir(), self.patterns_dir(), self.state_dir()] {
            let _ = std::fs::create_dir_all(d);
        }
    }

    /// The kit files, sorted by name.
    pub fn kit_files(&self) -> Vec<PathBuf> {
        snd_files(&self.kits_dir())
    }

    pub fn read_kit(path: &Path) -> Result<Kit, String> {
        let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
        Kit::from_file_bytes(&bytes).map_err(|e| e.to_string())
    }

    /// The path for saving `kit` as a new file: `NN-NAME.SND` with the first number no kit file
    /// uses.
    pub fn new_kit_path(&self, kit: &Kit) -> PathBuf {
        let used: Vec<usize> = self.kit_files().iter().filter_map(|p| slot_from_name(&file_label(p))).collect();
        let n = (0..100).find(|n| !used.contains(n)).unwrap_or(99);
        self.kits_dir().join(file_name(n, &kit.name, "SND"))
    }

    pub fn write_kit(path: &Path, kit: &Kit) -> std::io::Result<()> {
        write_atomic(path, &kit.to_file_bytes())
    }

    /// Read the session. Missing or unreadable files give defaults.
    pub fn load(&self) -> Session {
        let st = self.state_dir();
        let mut s = Session { kit: std::fs::read(st.join("kit.snd")).ok().and_then(|b| Kit::from_file_bytes(&b).ok()), ..Default::default() };
        let seq_path = st.join("seq.txt");
        if let Some(d) = std::fs::read_to_string(&seq_path).ok().and_then(|t| SeqData::from_text(&t)) {
            s.seq = d;
        }
        s.settings = std::fs::read_to_string(st.join("settings.txt")).ok();
        s.surface = std::fs::read_to_string(st.join("surface.json")).ok();
        if let Ok(text) = std::fs::read_to_string(st.join("session.txt")) {
            for line in text.lines() {
                let Some((k, v)) = line.split_once('=') else { continue };
                match k.trim() {
                    "kit_file" if !v.trim().is_empty() => s.kit_file = Some(v.trim().to_string()),
                    "mutes" => s.mutes = v.trim().parse::<u8>().unwrap_or(0) & 0x7f,
                    "faders" => {
                        let f: Vec<i32> = v.split(',').filter_map(|x| x.trim().parse().ok()).collect();
                        if let Ok(a) = <[i32; 7]>::try_from(f) {
                            s.faders = Some(a.map(|x| x.clamp(0, vdrum_engine::defaults::FADER_MAX)));
                        }
                    }
                    _ => {}
                }
            }
        }
        // pattern files newer than the saved sequencer data
        let since = mtime(&seq_path);
        let newer: Vec<PathBuf> = pat_files(&self.patterns_dir())
            .into_iter()
            .filter(|p| match (since, mtime(p)) {
                (Some(a), Some(b)) => b > a,
                _ => true,
            })
            .collect();
        if !newer.is_empty() {
            let (files, _) = read_files(&newer);
            let plan = plan_patterns(&s.seq, &files);
            let mut lib = Library::default();
            plan.apply(&mut s.seq, &mut lib);
            s.imported = plan.puts.iter().map(|(n, _)| *n).collect();
        }
        s
    }

    /// Write the patterns, then the rest of the session. Returns the first error.
    pub fn save(&self, s: &Session) -> Result<(), String> {
        self.ensure_dirs();
        let mut first_err: Option<String> = None;
        let mut note = |r: std::io::Result<()>, what: &str| {
            if let Err(e) = r {
                first_err.get_or_insert_with(|| format!("{what}: {e}"));
            }
        };
        note(self.save_patterns(&s.seq), "patterns");
        let st = self.state_dir();
        if let Some(k) = &s.kit {
            note(write_atomic(&st.join("kit.snd"), &k.to_file_bytes()), "kit.snd");
        }
        if let Some(t) = &s.settings {
            note(write_atomic(&st.join("settings.txt"), t.as_bytes()), "settings.txt");
        }
        if let Some(t) = &s.surface {
            note(write_atomic(&st.join("surface.json"), t.as_bytes()), "surface.json");
        }
        let faders = s.faders.map(|f| f.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",")).unwrap_or_default();
        let session = format!("kit_file={}\nmutes={}\nfaders={}\n", s.kit_file.as_deref().unwrap_or(""), s.mutes, faders);
        note(write_atomic(&st.join("session.txt"), session.as_bytes()), "session.txt");
        note(write_atomic(&st.join("seq.txt"), s.seq.to_text().as_bytes()), "seq.txt");
        first_err.map_or(Ok(()), Err)
    }

    /// Write every non-empty pattern whose file differs: into the file that already holds its
    /// number, or a new `NN-Pnn.PAT`. Files of patterns that are now empty are left alone (they are
    /// older than `seq.txt`, so they are not loaded again).
    pub fn save_patterns(&self, seq: &SeqData) -> std::io::Result<()> {
        let dir = self.patterns_dir();
        let existing = pat_files(&dir);
        for n in 0..NUM_PATTERNS {
            let pattern = &seq.patterns[n];
            if pattern.is_empty() {
                continue;
            }
            let path = existing.iter().find(|p| slot_from_name(&file_label(p)) == Some(n)).cloned();
            let (path, name, kit_nr) = match path {
                Some(p) => {
                    let old = std::fs::read(&p).ok().and_then(|b| pat_from_bytes(&b).ok());
                    if old.as_ref().is_some_and(|o| &o.pattern == pattern) {
                        continue;
                    }
                    let (name, kit_nr) = old.map_or((name_bytes(&format!("P{:02}", n + 1)), 0), |o| (o.name, o.kit_nr));
                    (p, name, kit_nr)
                }
                None => {
                    let name = name_bytes(&format!("P{:02}", n + 1));
                    (dir.join(file_name(n, &name, "PAT")), name, 0)
                }
            };
            let bytes = pat_to_bytes(&PatFile { name, pattern: pattern.clone(), kit_nr });
            // a pattern the file format cannot hold exactly is still written (seq.txt keeps it all)
            write_atomic(&path, &bytes)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use vdrum_engine::seq::{SeqEdit, StepData};

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("vdrum-move-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        d
    }

    #[test]
    fn session_round_trip_and_pattern_import() {
        let dir = tmp("store");
        let store = Store::new(&dir);
        store.ensure_dirs();
        let mut s = Session::default();
        s.seq.apply(&SeqEdit::Step { pat: 2, track: 1, step: 5, data: StepData { on: true, velocity: 90, ..Default::default() } });
        let mut kit = Kit::init();
        kit.set_name("GROOVE");
        s.kit = Some(kit.clone());
        s.kit_file = Some("00-GROOV.SND".into());
        s.settings = Some("pad_velocity=90".into());
        s.surface = Some("{}".into());
        s.faders = Some([99, 50, 0, 99, 99, 99, 1]);
        s.mutes = 5;
        store.save(&s).unwrap();
        assert!(dir.join("patterns/02-P03.PAT").is_file());
        let back = store.load();
        assert_eq!(back.seq, s.seq);
        assert_eq!(back.kit, s.kit);
        assert_eq!(back.kit_file, s.kit_file);
        assert_eq!(back.faders, s.faders);
        assert_eq!(back.mutes, 5);
        assert!(back.imported.is_empty());

        // a pattern file copied in later replaces its pattern
        std::thread::sleep(std::time::Duration::from_millis(20));
        let mut other = SeqData::default();
        other.apply(&SeqEdit::Step { pat: 2, track: 3, step: 0, data: StepData { on: true, velocity: 127, ..Default::default() } });
        let p = PatFile { name: name_bytes("NEW"), pattern: other.patterns[2].clone(), kit_nr: 0 };
        std::fs::write(dir.join("patterns/02-NEW.PAT"), pat_to_bytes(&p)).unwrap();
        // make sure it is newer even on file systems with coarse times
        let later = SystemTime::now() + std::time::Duration::from_secs(2);
        let f = std::fs::File::options().write(true).open(dir.join("patterns/02-NEW.PAT")).unwrap();
        f.set_modified(later).unwrap();
        let back = store.load();
        assert_eq!(back.imported, vec![2]);
        assert!(back.seq.step(2, 3, 0).on);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn new_kit_paths_take_free_numbers() {
        let dir = tmp("kits");
        let store = Store::new(&dir);
        store.ensure_dirs();
        let mut k = Kit::init();
        k.set_name("ABC");
        let p = store.new_kit_path(&k);
        assert_eq!(file_label(&p), "00-ABC.SND");
        Store::write_kit(&p, &k).unwrap();
        assert_eq!(file_label(&store.new_kit_path(&k)), "01-ABC.SND");
        assert_eq!(Store::read_kit(&p).unwrap().name_str().trim_end_matches('\0'), k.name_str().trim_end_matches('\0'));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
