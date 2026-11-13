// SPDX-License-Identifier: GPL-3.0-only
//! LXR-02 pattern files (`.PAT`, `PatV03`) and project folders, as written by the firmware's
//! `presetManager.c`:
//!
//! * `.PAT` (`preset_savePatternSD`, `preset_writePatternData`), 3663 bytes: the 8-byte name, the
//!   tag `PatV03`, then per track the 64 steps (`Step`, 7 bytes: volume with the active bit 0x80,
//!   probability, note, two parameter locks) and the 64 timing bytes (`StepTiming`: flam in bits
//!   0..3, shift in bits 4..7); then the main steps (7 tracks × 4 bars × `uint16_t`, little
//!   endian), the shuffle, the 7 track lengths and the kit number.
//! * `.SNG` (`preset_saveSongSD`): the 8-byte name and 64 pattern numbers (`SONG_END` = 64 ends).
//! * `GLO.CFG` (`preset_writeGlobalData`): the global parameters `PAR_BEGINNING_OF_GLOBALS` to
//!   `NUM_PARAMS - 1`, one byte each.
//! * `PRJ.NFO`: the 8-byte project name.
//! * File names (`preset_makeFileNameSave`): `NN-` and the first 5 characters of the name, with
//!   characters FAT does not allow replaced by `_`. They are written in upper case, as an LXR-02's
//!   FAT card shows them.

use crate::library::Library;
use lxr_params::kit::{Kit, KIT_FW_FILE_LEN};
use lxr_params::open_hat::OH_VELOD;
use lxr_params::par;
use std::path::{Path, PathBuf};
use crate::seq::{PatternData, SeqData, StepData, NUM_PATTERNS, NUM_STEPS, NUM_TRACKS};

pub const PAT_TAG: &[u8; 6] = b"PatV03";
const STEP_LEN: usize = 7;
const TRACK_LEN: usize = NUM_STEPS * STEP_LEN + NUM_STEPS;
const NUM_BARS: usize = 4;
/// Length of a `.PAT` file.
pub const PAT_LEN: usize = 8 + 6 + NUM_TRACKS * TRACK_LEN + NUM_TRACKS * NUM_BARS * 2 + 1 + NUM_TRACKS + 1;
/// `MAX_SONG_LENGTH`, and `SONG_END` (= `MAX_PATTERN`).
const SONG_LEN: usize = 64;
const SONG_END: u8 = 64;
/// `PAR_BEGINNING_OF_GLOBALS` .. `NUM_PARAMS`.
pub const GLOBALS: std::ops::Range<usize> = 272..lxr_params::NUM_PARAMS as usize;
const STEP_ACTIVE: u8 = 0x80;

/// A pattern file's contents.
#[derive(Clone, Debug, PartialEq)]
pub struct PatFile {
    pub name: [u8; 8],
    pub pattern: PatternData,
    /// The kit slot the LXR-02 loads with the pattern (kept for export; VDrum does not switch
    /// kits on pattern changes).
    pub kit_nr: u8,
}

/// Reads a `PatV03` pattern file.
pub fn pat_from_bytes(b: &[u8]) -> Result<PatFile, String> {
    if b.len() < 14 || &b[8..14] != PAT_TAG {
        return Err("not a PatV03 pattern file".into());
    }
    if b.len() != PAT_LEN {
        return Err(format!("{} bytes, expected {PAT_LEN}", b.len()));
    }
    let mut name = [0u8; 8];
    name.copy_from_slice(&b[..8]);
    let body = &b[14..];
    let main = &body[NUM_TRACKS * TRACK_LEN..];
    let tail = &main[NUM_TRACKS * NUM_BARS * 2..];
    let mut pattern = PatternData { shuffle: tail[0].min(127), ..Default::default() };
    for (t, track) in pattern.tracks.iter_mut().enumerate() {
        let tb = &body[t * TRACK_LEN..(t + 1) * TRACK_LEN];
        let mut bits = 0u64;
        for bar in 0..NUM_BARS {
            let o = (t * NUM_BARS + bar) * 2;
            bits |= (u16::from_le_bytes([main[o], main[o + 1]]) as u64) << (16 * bar);
        }
        for (s, step) in track.steps.iter_mut().enumerate() {
            let st = &tb[s * STEP_LEN..(s + 1) * STEP_LEN];
            let timing = tb[NUM_STEPS * STEP_LEN + s];
            *step = StepData {
                // the sequencer plays the main steps
                on: bits >> s & 1 != 0,
                velocity: (st[0] & 0x7f).max(1),
                prob: st[1].min(100),
                note: st[2] & 0x7f,
                flam: (timing & 0x0f).min(crate::seq::MAX_FLAM),
                shift: (timing >> 4).min(14),
                locks: [(st[3], st[4] & 0x7f), (st[5], st[6] & 0x7f)],
            };
        }
        track.length = tail[1 + t].clamp(1, NUM_STEPS as u8);
        track.fills = 0;
        track.rotate = 0;
    }
    Ok(PatFile { name, pattern, kit_nr: tail[1 + NUM_TRACKS] })
}

/// Writes a `PatV03` pattern file.
pub fn pat_to_bytes(p: &PatFile) -> Vec<u8> {
    let mut out = Vec::with_capacity(PAT_LEN);
    out.extend_from_slice(&p.name);
    out.extend_from_slice(PAT_TAG);
    for track in &p.pattern.tracks {
        for s in &track.steps {
            out.extend_from_slice(&[
                (s.velocity & 0x7f) | if s.on { STEP_ACTIVE } else { 0 },
                s.prob,
                s.note,
                s.locks[0].0,
                s.locks[0].1,
                s.locks[1].0,
                s.locks[1].1,
            ]);
        }
        out.extend(track.steps.iter().map(|s| (s.flam & 0x0f) | (s.shift << 4)));
    }
    for track in &p.pattern.tracks {
        let bits = track.bits();
        for bar in 0..NUM_BARS {
            out.extend_from_slice(&((bits >> (16 * bar)) as u16).to_le_bytes());
        }
    }
    out.push(p.pattern.shuffle);
    out.extend(p.pattern.tracks.iter().map(|t| t.length));
    out.push(p.kit_nr);
    debug_assert_eq!(out.len(), PAT_LEN);
    out
}

/// `preset_makeFileNameSave` (in upper case).
pub fn file_name(num: usize, name: &[u8; 8], ext: &str) -> String {
    let mut s = format!("{:02}-", num.min(99));
    for &c in name.iter().take(5) {
        if c == 0 {
            break;
        }
        let ok = c.is_ascii_alphanumeric() || b"!#$%&()-@^_`{}~".contains(&c);
        s.push(if ok { c as char } else { '_' });
    }
    s.push('.');
    s.push_str(ext);
    s.to_ascii_uppercase()
}

/// Name bytes from text (8 bytes, NUL padded, non-ASCII becomes `_`).
pub fn name_bytes(name: &str) -> [u8; 8] {
    let mut k = Kit::init();
    k.set_name(name);
    k.name
}

pub fn name_text(name: &[u8; 8]) -> String {
    let end = name.iter().position(|b| *b == 0).unwrap_or(8);
    name[..end].iter().map(|b| *b as char).collect()
}

/// The firmware kit file for an LXR-02: the open hi-hat's decay goes to the hi-hat's open decay
/// (`PAR_VELOD6_OPEN`); its other parameters have no place in the firmware kit.
pub fn snd_bytes(kit: &Kit) -> [u8; KIT_FW_FILE_LEN] {
    let mut k = kit.clone();
    k.values[par::VELOD6_OPEN as usize] = k.oh[OH_VELOD];
    k.to_firmware_bytes()
}

/// The firmware's empty song (`preset_clearSongs` / a song never edited).
pub fn empty_song() -> [u8; 8 + SONG_LEN] {
    let mut b = [SONG_END; 8 + SONG_LEN];
    b[..8].copy_from_slice(b"Empty\0\0\0");
    b
}

/// What the LXR-02 cannot hold; shown before an export.
pub const EXPORT_LOSSES: &[&str] = &[
    "Open hi-hat: only its decay is kept (as the hi-hat's open decay); its other 36 parameters are dropped.",
    "Sequencer: track densities and Euclid settings are not saved; the steps they produced are.",
    "Songs are written empty (VDrum has no song mode).",
];

/// The files of an LXR-02 project folder: (file name, bytes).
pub fn project_files(lib: &Library, seq: &SeqData, globals: &[u8], project_name: &[u8; 8]) -> Vec<(String, Vec<u8>)> {
    let mut files = Vec::new();
    files.push(("PRJ.NFO".to_string(), project_name.to_vec()));
    files.push(("GLO.CFG".to_string(), globals.to_vec()));
    let init = Kit::init();
    for n in 0..NUM_PATTERNS {
        let kit = lib.slots[n].as_ref().map(|e| &e.kit).unwrap_or(&init);
        files.push((file_name(n, &kit.name, "SND"), snd_bytes(kit).to_vec()));
        let p = PatFile { name: lib.pattern_names[n], pattern: seq.patterns[n].clone(), kit_nr: lib.pattern_kits[n] };
        files.push((file_name(n, &p.name, "PAT"), pat_to_bytes(&p)));
        let song = empty_song();
        let mut song_name = [0u8; 8];
        song_name.copy_from_slice(&song[..8]);
        files.push((file_name(n, &song_name, "SNG"), song.to_vec()));
    }
    files
}

/// Whether a file name is one an LXR-02 project folder holds (`NN-*.SND/PAT/SNG`, `GLO.CFG`,
/// `PRJ.NFO`).
pub fn is_project_file(name: &str) -> bool {
    let u = name.to_ascii_uppercase();
    if u == "GLO.CFG" || u == "PRJ.NFO" {
        return true;
    }
    let numbered = u.len() > 3 && u.as_bytes()[..2].iter().all(u8::is_ascii_digit) && u.as_bytes()[2] == b'-';
    numbered && (u.ends_with(".SND") || u.ends_with(".PAT") || u.ends_with(".SNG"))
}

/// The LXR-02 project files already in a folder (an export replaces them).
pub fn existing_project_files(dir: &Path) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = std::fs::read_dir(dir)
        .map(|d| {
            d.filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| p.is_file() && p.file_name().is_some_and(|n| is_project_file(&n.to_string_lossy())))
                .collect()
        })
        .unwrap_or_default();
    v.sort();
    v
}

/// Writes a project folder: removes the LXR-02 project files there (old names would otherwise
/// stay next to the new ones, and the firmware loads files by their number), then writes the new
/// ones. Returns the number of files written.
pub fn write_project(dir: &Path, files: &[(String, Vec<u8>)]) -> std::io::Result<usize> {
    std::fs::create_dir_all(dir)?;
    for p in existing_project_files(dir) {
        std::fs::remove_file(p)?;
    }
    for (name, bytes) in files {
        std::fs::write(dir.join(name), bytes)?;
    }
    Ok(files.len())
}

/// The `.pat` files of a folder, sorted by name.
pub fn pat_files(folder: &Path) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = std::fs::read_dir(folder)
        .map(|d| {
            d.filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| p.is_file() && p.extension().is_some_and(|x| x.eq_ignore_ascii_case("pat")))
                .collect()
        })
        .unwrap_or_default();
    v.sort();
    v
}

/// What a pattern import will do.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PatternPlan {
    /// (pattern number, file contents).
    pub puts: Vec<(usize, PatFile)>,
    /// Patterns with no step on.
    pub empty_skipped: usize,
    pub no_room: usize,
    pub errors: Vec<(String, String)>,
}

impl PatternPlan {
    /// The pattern numbers it writes that have steps now.
    pub fn replaces(&self, seq: &SeqData) -> Vec<usize> {
        self.puts.iter().map(|(n, _)| *n).filter(|&n| !seq.patterns[n].is_empty()).collect()
    }

    /// "patterns 01 to 16", or "pattern 03".
    pub fn range_text(numbers: &[usize]) -> String {
        match (numbers.first(), numbers.last()) {
            (Some(a), Some(b)) if a != b => format!("patterns {:02} to {:02}", a + 1, b + 1),
            (Some(a), _) => format!("pattern {:02}", a + 1),
            _ => "no patterns".into(),
        }
    }

    pub fn summary(&self) -> String {
        let n = self.puts.len();
        let mut s = format!("{n} pattern{}", if n == 1 { "" } else { "s" });
        let mut extra = Vec::new();
        if self.empty_skipped > 0 {
            extra.push(format!("{} empty skipped", self.empty_skipped));
        }
        if self.no_room > 0 {
            extra.push(format!("{} did not fit", self.no_room));
        }
        if !self.errors.is_empty() {
            extra.push(format!("{} unreadable", self.errors.len()));
        }
        if !extra.is_empty() {
            s += &format!(" ({})", extra.join(", "));
        }
        s
    }

    /// Writes the patterns into the sequencer data and their names and kit numbers into the
    /// library.
    pub fn apply(&self, seq: &mut SeqData, lib: &mut Library) {
        for (n, p) in &self.puts {
            seq.patterns[*n] = p.pattern.clone();
            lib.pattern_names[*n] = p.name;
            lib.pattern_kits[*n] = p.kit_nr;
        }
    }
}

/// Plans a pattern import: `NN-*.PAT` goes to pattern `NN + 1` (the firmware numbers from 00),
/// any other file to the first empty pattern; empty patterns are skipped.
pub fn plan_patterns(seq: &SeqData, files: &[(String, Vec<u8>)]) -> PatternPlan {
    let mut plan = PatternPlan::default();
    let mut taken: Vec<bool> = seq.patterns.iter().map(|p| !p.is_empty()).collect();
    let mut placed = [false; NUM_PATTERNS];
    let mut order: Vec<&(String, Vec<u8>)> = files.iter().collect();
    order.sort_by_key(|(n, _)| crate::library::slot_from_name(n).is_none());
    for (name, bytes) in order {
        let p = match pat_from_bytes(bytes) {
            Ok(p) => p,
            Err(e) => {
                plan.errors.push((name.clone(), e));
                continue;
            }
        };
        if p.pattern.is_empty() {
            plan.empty_skipped += 1;
            continue;
        }
        let n = crate::library::slot_from_name(name).filter(|&n| !placed[n]).or_else(|| (0..NUM_PATTERNS).find(|&n| !taken[n] && !placed[n]));
        let Some(n) = n else {
            plan.no_room += 1;
            continue;
        };
        taken[n] = true;
        placed[n] = true;
        plan.puts.push((n, p));
    }
    plan.puts.sort_by_key(|(n, _)| *n);
    plan
}

/// Reads files as (file name, bytes); unreadable ones are returned as errors.
/// Files as (file name, bytes), and unreadable files as (file name, error).
pub type Files = (Vec<(String, Vec<u8>)>, Vec<(String, String)>);

pub fn read_files(paths: &[PathBuf]) -> Files {
    let mut files = Vec::new();
    let mut errors = Vec::new();
    for p in paths {
        let name = p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        match std::fs::read(p) {
            Ok(b) => files.push((name, b)),
            Err(e) => errors.push((name, e.to_string())),
        }
    }
    (files, errors)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pattern(seed: u8) -> PatternData {
        let mut p = PatternData { shuffle: seed % 128, ..Default::default() };
        for (t, track) in p.tracks.iter_mut().enumerate() {
            track.length = 1 + (seed as usize + t * 7) as u8 % 64;
            for (s, st) in track.steps.iter_mut().enumerate() {
                let x = (s * 31 + t * 17 + seed as usize) as u8;
                *st = StepData {
                    on: x.is_multiple_of(3),
                    velocity: 1 + x % 127,
                    prob: x % 101,
                    note: x % 128,
                    flam: x % 6,
                    shift: x % 15,
                    locks: if x.is_multiple_of(5) { [(x, x % 128), (255, 0)] } else { [(255, 0), (255, 0)] },
                };
            }
        }
        p
    }

    #[test]
    fn pattern_round_trip() {
        let p = PatFile { name: name_bytes("GROOVE"), pattern: pattern(9), kit_nr: 12 };
        let b = pat_to_bytes(&p);
        assert_eq!(b.len(), PAT_LEN);
        assert_eq!(PAT_LEN, 3663);
        assert_eq!(&b[8..14], b"PatV03");
        let back = pat_from_bytes(&b).unwrap();
        assert_eq!(back, p);
        assert_eq!(pat_to_bytes(&back), b, "byte identical");
        // the main steps decide what plays; the active bit follows them
        assert!(pat_from_bytes(&b[..100]).is_err());
        let mut v2 = b.clone();
        v2[13] = b'2';
        assert!(pat_from_bytes(&v2).is_err());
    }

    #[test]
    fn file_names_follow_the_firmware() {
        assert_eq!(file_name(3, &name_bytes("Malwa x"), "SND"), "03-MALWA.SND");
        assert_eq!(file_name(15, &name_bytes("MIST "), "snd"), "15-MIST_.SND");
        assert_eq!(file_name(0, &[0; 8], "PAT"), "00-.PAT");
        assert_eq!(file_name(1, &name_bytes("a.b"), "PAT"), "01-A_B.PAT");
        assert_eq!(file_name(7, &Kit::init().name, "SND"), "07-INITK.SND");
        assert!(is_project_file("07-INITK.SND") && is_project_file("glo.cfg") && is_project_file("00-.pat"));
        assert!(!is_project_file("notes.txt") && !is_project_file("A-B.SND"));
    }

    #[test]
    fn pattern_import_plan() {
        let mut seq = SeqData::default();
        seq.patterns[2] = pattern(1);
        let f = |n: &str, seed: u8| (n.to_string(), pat_to_bytes(&PatFile { name: name_bytes(n), pattern: pattern(seed), kit_nr: seed }));
        let empty = ("05-.PAT".to_string(), pat_to_bytes(&PatFile { name: [0; 8], pattern: PatternData::default(), kit_nr: 0 }));
        let files = vec![f("other.pat", 3), f("02-X.PAT", 4), empty, ("bad.pat".into(), vec![1, 2, 3])];
        let plan = plan_patterns(&seq, &files);
        assert_eq!(plan.puts.iter().map(|(n, _)| *n).collect::<Vec<_>>(), vec![0, 2]);
        assert_eq!(plan.replaces(&seq), vec![2]);
        assert_eq!((plan.empty_skipped, plan.errors.len()), (1, 1));
        assert_eq!(plan.summary(), "2 patterns (1 empty skipped, 1 unreadable)");
        let mut lib = Library::default();
        plan.apply(&mut seq, &mut lib);
        assert_eq!(seq.patterns[2], pattern(4));
        assert_eq!(lib.pattern_kits[2], 4);
        assert_eq!(name_text(&lib.pattern_names[2]), "02-X.PAT");
        assert_eq!(PatternPlan::range_text(&[0, 1, 15]), "patterns 01 to 16");
    }

    #[test]
    fn project_export_and_import() {
        let mut lib = Library::default();
        let mut k = Kit::init();
        k.set_name("LAMBO");
        k.values[par::COARSE1 as usize] = 77;
        k.oh[OH_VELOD] = 99;
        lib.slots[0] = Some(crate::library::Entry { kit: k.clone(), source: String::new() });
        let mut seq = SeqData::default();
        seq.patterns[1] = pattern(5);
        lib.pattern_names[1] = name_bytes("BEAT");
        lib.pattern_kits[1] = 0;
        let globals: Vec<u8> = (0..GLOBALS.len() as u8).collect();
        let files = project_files(&lib, &seq, &globals, &name_bytes("MYPROJ"));
        assert_eq!(files.len(), 2 + 3 * 64);
        let get = |n: &str| &files.iter().find(|(f, _)| f == n).unwrap_or_else(|| panic!("{n}")).1;
        assert_eq!(get("PRJ.NFO"), b"MYPROJ\0\0");
        assert_eq!(get("GLO.CFG").len(), 30);
        let snd = get("00-LAMBO.SND");
        assert_eq!(snd.len(), 255);
        assert_eq!(snd[8 + par::VELOD6_OPEN as usize], 99, "open hat decay");
        assert_eq!(snd[8 + par::COARSE1 as usize], 77);
        assert_eq!(get("63-INITK.SND").len(), 255);
        assert_eq!(get("01-BEAT.PAT").len(), PAT_LEN);
        assert_eq!(get("00-.PAT").len(), PAT_LEN);
        assert_eq!(get("05-EMPTY.SNG").len(), 72);

        let dir = std::env::temp_dir().join(format!("vdrum-prj-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("05-OLD.SND"), [0u8; 255]).unwrap();
        std::fs::write(dir.join("keep.txt"), b"x").unwrap();
        assert_eq!(write_project(&dir, &files).unwrap(), files.len());
        assert!(!dir.join("05-OLD.SND").exists(), "old project files are replaced");
        assert!(dir.join("keep.txt").exists(), "other files stay");
        // and back
        let (kits, _) = read_files(&crate::library::snd_files(&dir));
        let kplan = Library::default().plan_import(&kits);
        assert_eq!(kplan.puts.len(), 1);
        assert_eq!(kplan.init_skipped, 63);
        let (pats, _) = read_files(&pat_files(&dir));
        let pplan = plan_patterns(&SeqData::default(), &pats);
        assert_eq!(pplan.puts.len(), 1);
        assert_eq!(pplan.puts[0].0, 1);
        assert_eq!(pplan.puts[0].1.pattern, seq.patterns[1]);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
