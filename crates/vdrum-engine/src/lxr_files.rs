// SPDX-License-Identifier: GPL-3.0-only
//! LXR-02 project folders, as written by the firmware's `presetManager.c`. VDrum has no sequencer,
//! so an export writes empty patterns and an import reads only the kits.
//!
//! * `.PAT` (`preset_savePatternSD`, `preset_writePatternData`), 3663 bytes: the 8-byte name, the
//!   tag `PatV03`, then per track the 64 steps (`Step`, 7 bytes: volume with the active bit 0x80,
//!   probability, note, two parameter locks) and the 64 timing bytes (`StepTiming`: flam in bits
//!   0..3, shift in bits 4..7); then the main steps (7 tracks × 4 bars × `uint16_t`, little
//!   endian), the shuffle, the 7 track lengths and the kit number. Written empty
//!   ([`empty_pattern`]).
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

pub const PAT_TAG: &[u8; 6] = b"PatV03";
/// Patterns, kits and songs in a project folder.
pub const NUM_PATTERNS: usize = 64;
const NUM_TRACKS: usize = 7;
const NUM_STEPS: usize = 64;
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

/// The firmware's cleared pattern (`seq_clearPattern`, `seq_resetNote`) as a `PatV03` file: no
/// step on, velocity 127, probability 100, note 60 (`SEQ_DEFAULT_NOTE`), no parameter locks
/// (`NO_AUTOMATION` = 255), no flam, no shift (7), shuffle 0, track length 16, kit 0.
pub fn empty_pattern() -> Vec<u8> {
    let mut out = Vec::with_capacity(PAT_LEN);
    out.extend_from_slice(&[0; 8]);
    out.extend_from_slice(PAT_TAG);
    for _ in 0..NUM_TRACKS {
        for _ in 0..NUM_STEPS {
            out.extend_from_slice(&[127, 100, 60, 255, 0, 255, 0]);
        }
        out.extend(std::iter::repeat_n(7 << 4, NUM_STEPS));
    }
    out.extend(std::iter::repeat_n(0, NUM_TRACKS * NUM_BARS * 2));
    out.push(0);
    out.extend(std::iter::repeat_n(16, NUM_TRACKS));
    out.push(0);
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
    "Patterns and songs are written empty (VDrum has no sequencer).",
];

/// The files of an LXR-02 project folder: (file name, bytes).
pub fn project_files(lib: &Library, globals: &[u8], project_name: &[u8; 8]) -> Vec<(String, Vec<u8>)> {
    let mut files = Vec::new();
    files.push(("PRJ.NFO".to_string(), project_name.to_vec()));
    files.push(("GLO.CFG".to_string(), globals.to_vec()));
    let init = Kit::init();
    for n in 0..NUM_PATTERNS {
        let kit = lib.slots[n].as_ref().map(|e| &e.kit).unwrap_or(&init);
        files.push((file_name(n, &kit.name, "SND"), snd_bytes(kit).to_vec()));
        files.push((file_name(n, &[0; 8], "PAT"), empty_pattern()));
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

    #[test]
    fn empty_pattern_is_the_firmware_layout() {
        let b = empty_pattern();
        assert_eq!(b.len(), PAT_LEN);
        assert_eq!(PAT_LEN, 3663);
        assert_eq!(&b[8..14], b"PatV03");
        // first step of track 1, its timing byte, the track lengths
        assert_eq!(&b[14..21], &[127, 100, 60, 255, 0, 255, 0]);
        assert_eq!(b[14 + NUM_STEPS * 7], 0x70);
        assert_eq!(&b[PAT_LEN - 8..PAT_LEN - 1], &[16; 7]);
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
    fn project_export_and_import() {
        let mut lib = Library::default();
        let mut k = Kit::init();
        k.set_name("LAMBO");
        k.values[par::COARSE1 as usize] = 77;
        k.oh[OH_VELOD] = 99;
        lib.slots[0] = Some(crate::library::Entry { kit: k.clone(), source: String::new() });
        let globals: Vec<u8> = (0..GLOBALS.len() as u8).collect();
        let files = project_files(&lib, &globals, &name_bytes("MYPROJ"));
        assert_eq!(files.len(), 2 + 3 * 64);
        let get = |n: &str| &files.iter().find(|(f, _)| f == n).unwrap_or_else(|| panic!("{n}")).1;
        assert_eq!(get("PRJ.NFO"), b"MYPROJ\0\0");
        assert_eq!(get("GLO.CFG").len(), 30);
        let snd = get("00-LAMBO.SND");
        assert_eq!(snd.len(), 255);
        assert_eq!(snd[8 + par::VELOD6_OPEN as usize], 99, "open hat decay");
        assert_eq!(snd[8 + par::COARSE1 as usize], 77);
        assert_eq!(get("63-INITK.SND").len(), 255);
        assert_eq!(get("00-.PAT"), &empty_pattern());
        assert_eq!(get("63-.PAT"), &empty_pattern());
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
        let _ = std::fs::remove_dir_all(&dir);
    }
}
