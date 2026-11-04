// SPDX-License-Identifier: GPL-3.0-only
//! The kit library: 64 kit slots (`00` to `63`, as in an LXR-02 project) saved with the plugin
//! state, filled by importing `.snd` files and selectable from the editor and the controller.
//!
//! * Import skips init kits (the firmware's `INITK` slots) and kits already in the library.
//! * A file named `NN-NAME.SND` goes into slot `NN`, any other file into the first empty slot.
//! * Loading a slot goes through the normal kit load (`Command::LoadKit`).

use lxr_params::kit::Kit;
use std::path::{Path, PathBuf};

/// Number of kit slots, as in an LXR-02 project.
pub const SLOTS: usize = 64;

/// A filled slot: the kit and the file it came from.
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub kit: Kit,
    /// File name it was imported from, or empty.
    pub source: String,
}

/// The library.
#[derive(Clone, Debug, PartialEq)]
pub struct Library {
    pub slots: Vec<Option<Entry>>,
    /// Slot of the kit loaded last from the library.
    pub selected: Option<usize>,
    /// Pattern names (8 bytes, as in `.PAT`) and the kit slot each pattern names, for the 64
    /// sequencer patterns. Imported from and exported to LXR-02 pattern files.
    pub pattern_names: Vec<[u8; 8]>,
    pub pattern_kits: Vec<u8>,
}

impl Default for Library {
    fn default() -> Self {
        Library { slots: vec![None; SLOTS], selected: None, pattern_names: vec![[0; 8]; SLOTS], pattern_kits: vec![0; SLOTS] }
    }
}

/// Whether a kit has the init kit's voices (its name and the FX bus do not matter): the LXR-02's
/// empty `INITK` slots. Those keep FX values left over from other kits (seen in a commercial kit
/// pack: `fx_p2` to `fx_p4` and `fx_ratio` differ from the init kit), so the FX bus is ignored.
pub fn is_init_kit(kit: &Kit) -> bool {
    let init = Kit::init();
    let fx = |i: usize| lxr_params::info(i as u16).is_some_and(|p| p.voice == lxr_params::Voice::Fx);
    kit.oh == init.oh && (0..init.values.len()).all(|i| fx(i) || kit.values[i] == init.values[i])
}

/// Slot number from a file name of the form `NN-NAME.SND` (`00` to `63`).
pub fn slot_from_name(name: &str) -> Option<usize> {
    let (num, _) = name.split_once('-')?;
    if num.len() != 2 || !num.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    num.parse::<usize>().ok().filter(|&n| n < SLOTS)
}

/// The `.snd` files of a folder, sorted by name.
pub fn snd_files(folder: &Path) -> Vec<PathBuf> {
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

/// What an import will do.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ImportPlan {
    /// (slot, entry, name of the kit it replaces).
    pub puts: Vec<(usize, Entry, Option<String>)>,
    pub init_skipped: usize,
    pub duplicates: usize,
    /// No empty slot left.
    pub no_room: usize,
    /// (file, error).
    pub errors: Vec<(String, String)>,
}

impl ImportPlan {
    /// Whether the import replaces kits that are in the library.
    pub fn replaces(&self) -> Vec<String> {
        self.puts.iter().filter_map(|(s, _, r)| r.as_ref().map(|n| format!("{s:02} {n}"))).collect()
    }

    /// One line for the status bar.
    pub fn summary(&self) -> String {
        let mut s = format!("Imported {} kit{}", self.puts.len(), if self.puts.len() == 1 { "" } else { "s" });
        let mut extra = Vec::new();
        if self.init_skipped > 0 {
            extra.push(format!("{} init kits skipped", self.init_skipped));
        }
        if self.duplicates > 0 {
            extra.push(format!("{} already in the library", self.duplicates));
        }
        if self.no_room > 0 {
            extra.push(format!("{} did not fit (library full)", self.no_room));
        }
        if !self.errors.is_empty() {
            extra.push(format!("{} unreadable", self.errors.len()));
        }
        if !extra.is_empty() {
            s += &format!(" ({})", extra.join(", "));
        }
        s
    }
}

fn file_name(p: &Path) -> String {
    p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default()
}

impl Library {
    pub fn is_empty(&self) -> bool {
        self.slots.iter().all(Option::is_none)
    }

    pub fn len(&self) -> usize {
        self.slots.iter().filter(|s| s.is_some()).count()
    }

    /// Plans the import of kit files given as (file name, bytes).
    pub fn plan_import(&self, files: &[(String, Vec<u8>)]) -> ImportPlan {
        let mut plan = ImportPlan::default();
        // slots taken by this import, and their kits
        let mut taken: Vec<Option<Kit>> = self.slots.iter().map(|s| s.as_ref().map(|e| e.kit.clone())).collect();
        let mut placed = [false; SLOTS];
        let same_sound = |a: &Kit, b: &Kit| a.values == b.values && a.oh == b.oh;
        // numbered files first, so that unnumbered files fill the gaps
        let mut order: Vec<&(String, Vec<u8>)> = files.iter().collect();
        order.sort_by_key(|(n, _)| slot_from_name(n).is_none());
        for (name, bytes) in order {
            let kit = match Kit::from_file_bytes(bytes) {
                Ok(k) => k,
                Err(e) => {
                    plan.errors.push((name.clone(), e.to_string()));
                    continue;
                }
            };
            if is_init_kit(&kit) {
                plan.init_skipped += 1;
                continue;
            }
            if taken.iter().flatten().any(|k| same_sound(k, &kit) && k.name == kit.name) {
                plan.duplicates += 1;
                continue;
            }
            let slot = slot_from_name(name).filter(|&s| !placed[s]).or_else(|| (0..SLOTS).find(|&s| taken[s].is_none()));
            let Some(slot) = slot else {
                plan.no_room += 1;
                continue;
            };
            let replaced = self.slots[slot].as_ref().filter(|_| !placed[slot]).map(|e| e.kit.name_str());
            taken[slot] = Some(kit.clone());
            placed[slot] = true;
            plan.puts.push((slot, Entry { kit, source: name.clone() }, replaced));
        }
        plan.puts.sort_by_key(|(s, _, _)| *s);
        plan
    }

    /// Reads kit files from disk and plans their import.
    pub fn plan_import_paths(&self, paths: &[PathBuf]) -> ImportPlan {
        let mut files = Vec::new();
        let mut errors = Vec::new();
        for p in paths {
            match std::fs::read(p) {
                Ok(b) => files.push((file_name(p), b)),
                Err(e) => errors.push((file_name(p), e.to_string())),
            }
        }
        let mut plan = self.plan_import(&files);
        plan.errors.extend(errors);
        plan
    }

    pub fn apply(&mut self, plan: &ImportPlan) {
        for (slot, entry, _) in &plan.puts {
            self.slots[*slot] = Some(entry.clone());
        }
    }

    /// The next filled slot after `from` in direction `dir` (±1), wrapping; the first or last
    /// filled slot when `from` is `None`.
    pub fn step(&self, from: Option<usize>, dir: i32) -> Option<usize> {
        let filled: Vec<usize> = (0..SLOTS).filter(|&s| self.slots[s].is_some()).collect();
        if filled.is_empty() {
            return None;
        }
        let n = filled.len() as i32;
        let i = match from.and_then(|f| filled.iter().position(|&s| s == f)) {
            Some(i) => (i as i32 + dir).rem_euclid(n),
            None => match from {
                // an empty slot: the nearest filled one in that direction
                Some(f) if dir > 0 => filled.iter().position(|&s| s > f).unwrap_or(0) as i32,
                Some(f) => filled.iter().rposition(|&s| s < f).unwrap_or(filled.len() - 1) as i32,
                None if dir > 0 => 0,
                None => n - 1,
            },
        };
        Some(filled[i as usize])
    }

    /// Moves a slot's content up (`dir` = -1) or down (+1) by swapping with the neighbour.
    pub fn move_slot(&mut self, slot: usize, dir: i32) {
        let to = slot as i32 + dir;
        if (0..SLOTS as i32).contains(&to) {
            self.slots.swap(slot, to as usize);
            if self.selected == Some(slot) {
                self.selected = Some(to as usize);
            } else if self.selected == Some(to as usize) {
                self.selected = Some(slot);
            }
        }
    }

    /// Label of a slot: `03 MALWA`, or `03 (empty)`.
    pub fn label(&self, slot: usize) -> String {
        match &self.slots[slot] {
            Some(e) => format!("{slot:02} {}", e.kit.name_str()),
            None => format!("{slot:02} (empty)"),
        }
    }

    /// Text form saved with the plugin state: the selected slot, then one line per filled slot
    /// (`slot hex-kit-bytes source`), then one line per named pattern or pattern with a kit number
    /// (`pattern number kit hex-name`).
    pub fn to_text(&self) -> String {
        let mut s = match self.selected {
            Some(n) => format!("selected {n}\n"),
            None => "selected -\n".to_string(),
        };
        for (i, e) in self.slots.iter().enumerate() {
            if let Some(e) = e {
                let hex: String = e.kit.to_file_bytes().iter().map(|b| format!("{b:02x}")).collect();
                s += &format!("{i} {hex} {}\n", e.source.replace('\n', " "));
            }
        }
        for n in 0..SLOTS {
            if self.pattern_names[n] != [0; 8] || self.pattern_kits[n] != 0 {
                let hex: String = self.pattern_names[n].iter().map(|b| format!("{b:02x}")).collect();
                s += &format!("pattern {n} {} {hex}\n", self.pattern_kits[n]);
            }
        }
        s
    }

    /// Reads [`Library::to_text`]; lines it cannot read are skipped.
    pub fn from_text(text: &str) -> Library {
        let mut lib = Library::default();
        for line in text.lines() {
            let mut it = line.splitn(3, ' ');
            let (Some(a), Some(b)) = (it.next(), it.next()) else { continue };
            if a == "selected" {
                lib.selected = b.parse::<usize>().ok().filter(|&n| n < SLOTS);
                continue;
            }
            if a == "pattern" {
                let mut rest = it.next().unwrap_or("").split(' ');
                let (Some(n), Some(kit), Some(hex)) = (b.parse::<usize>().ok().filter(|&n| n < SLOTS), rest.next().and_then(|k| k.parse::<u8>().ok()), rest.next()) else { continue };
                if hex.len() == 16 && hex.is_ascii() {
                    for i in 0..8 {
                        lib.pattern_names[n][i] = u8::from_str_radix(&hex[2 * i..2 * i + 2], 16).unwrap_or(0);
                    }
                }
                lib.pattern_kits[n] = kit;
                continue;
            }
            let Some(slot) = a.parse::<usize>().ok().filter(|&n| n < SLOTS) else { continue };
            if b.len() % 2 != 0 {
                continue;
            }
            let bytes: Option<Vec<u8>> = (0..b.len()).step_by(2).map(|i| u8::from_str_radix(b.get(i..i + 2)?, 16).ok()).collect();
            if let Some(kit) = bytes.and_then(|b| Kit::from_file_bytes(&b).ok()) {
                lib.slots[slot] = Some(Entry { kit, source: it.next().unwrap_or("").to_string() });
            }
        }
        if lib.selected.is_some_and(|s| lib.slots[s].is_none()) {
            lib.selected = None;
        }
        lib
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kit(name: &str, coarse: u8) -> Kit {
        let mut k = Kit::init();
        k.set_name(name);
        k.values[lxr_params::par::COARSE1 as usize] = coarse;
        k
    }

    fn file(name: &str, k: &Kit) -> (String, Vec<u8>) {
        (name.to_string(), k.to_firmware_bytes().to_vec())
    }

    #[test]
    fn slot_numbers_from_file_names() {
        assert_eq!(slot_from_name("00-LAMBO.SND"), Some(0));
        assert_eq!(slot_from_name("63-INITK.SND"), Some(63));
        assert_eq!(slot_from_name("64-X.SND"), None);
        assert_eq!(slot_from_name("3-X.SND"), None);
        assert_eq!(slot_from_name("kick.snd"), None);
    }

    #[test]
    fn import_skips_init_kits_and_duplicates_and_uses_slot_numbers() {
        let mut lib = Library::default();
        // A project folder: two kits and init kits in the other slots (named INITK, as on the
        // LXR-02).
        let mut files = vec![file("00-AAA.SND", &kit("AAA", 10)), file("05-BBB.SND", &kit("BBB", 20))];
        let mut init = Kit::init();
        for s in [1, 2, 3, 4, 6] {
            // the LXR-02's init slots keep FX values from other kits
            init.values[lxr_params::par::FX_P2 as usize] = 10 * s as u8;
            files.push(file(&format!("{s:02}-INITK.SND"), &init));
        }
        let plan = lib.plan_import(&files);
        assert_eq!(plan.puts.iter().map(|(s, _, _)| *s).collect::<Vec<_>>(), vec![0, 5]);
        assert_eq!(plan.init_skipped, 5);
        assert!(plan.replaces().is_empty());
        assert_eq!(plan.summary(), "Imported 2 kits (5 init kits skipped)");
        lib.apply(&plan);
        assert_eq!(lib.len(), 2);
        assert_eq!(lib.label(5), "05 BBB");
        // The same files again: duplicates.
        let plan = lib.plan_import(&files[..2]);
        assert_eq!((plan.puts.len(), plan.duplicates), (0, 2));
        // An unnumbered file goes into the first empty slot; a numbered one replaces.
        let plan = lib.plan_import(&[file("kick.snd", &kit("KICK", 30)), file("05-CCC.SND", &kit("CCC", 40))]);
        assert_eq!(plan.puts.iter().map(|(s, _, _)| *s).collect::<Vec<_>>(), vec![1, 5]);
        assert_eq!(plan.replaces(), vec!["05 BBB".to_string()]);
        // Unreadable input.
        let plan = lib.plan_import(&[("x.snd".into(), vec![1, 2])]);
        assert_eq!(plan.errors.len(), 1);
    }

    #[test]
    fn full_library() {
        let mut lib = Library::default();
        let files: Vec<_> = (0..SLOTS + 2).map(|i| file(&format!("k{i}.snd"), &kit("K", 100 + i as u8))).collect();
        let plan = lib.plan_import(&files);
        assert_eq!((plan.puts.len(), plan.no_room), (SLOTS, 2));
        lib.apply(&plan);
        assert_eq!(lib.len(), SLOTS);
    }

    #[test]
    fn stepping_and_moving() {
        let mut lib = Library::default();
        for s in [2, 5, 9] {
            lib.slots[s] = Some(Entry { kit: kit("K", s as u8), source: String::new() });
        }
        assert_eq!(lib.step(None, 1), Some(2));
        assert_eq!(lib.step(None, -1), Some(9));
        assert_eq!(lib.step(Some(2), 1), Some(5));
        assert_eq!(lib.step(Some(9), 1), Some(2));
        assert_eq!(lib.step(Some(2), -1), Some(9));
        assert_eq!(lib.step(Some(6), 1), Some(9));
        assert_eq!(lib.step(Some(6), -1), Some(5));
        assert_eq!(Library::default().step(None, 1), None);
        lib.selected = Some(5);
        lib.move_slot(5, -1);
        assert!(lib.slots[4].is_some() && lib.slots[5].is_none());
        assert_eq!(lib.selected, Some(4));
        lib.move_slot(0, -1);
    }

    #[test]
    fn text_round_trip() {
        let mut lib = Library::default();
        let mut k = kit("OPEN", 50);
        k.oh[lxr_params::open_hat::OH_VELOD] = 77;
        lib.slots[7] = Some(Entry { kit: k, source: "07-OPEN.SND".into() });
        lib.slots[63] = Some(Entry { kit: kit("LAST", 1), source: String::new() });
        lib.selected = Some(7);
        lib.pattern_names[3] = *b"BEAT 1\0\0";
        lib.pattern_kits[3] = 9;
        lib.pattern_kits[60] = 2;
        let back = Library::from_text(&lib.to_text());
        assert_eq!(back, lib);
        assert_eq!(Library::from_text("garbage\n99 00\nselected 3\n"), Library::default());
    }
}
