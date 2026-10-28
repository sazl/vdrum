// SPDX-License-Identifier: GPL-3.0-only
//! Kit (`.snd`) files as written and read by `presetManager.c`:
//!
//! * `preset_saveDrumsetSD()`: 8 bytes kit name (`savefileNames[..][KIT_NAMES]`, not necessarily NUL
//!   terminated), then `END_OF_SOUND_PARAMETERS` bytes: `activeParameters[0..END_OF_SOUND_PARAMETERS]`
//!   in parameter index order (`preset_writeDrumsetData()`).
//! * `preset_loadDrumsetSD()` / `preset_readDrumsetData()`: reads the 8 name bytes (fails if fewer),
//!   then up to `END_OF_SOUND_PARAMETERS` bytes, sets the missing ones to 0 and resets modulation
//!   destinations (`PAR_VEL_DEST_x`, `PAR_TARGET_LFOx`) that are not valid [`MOD_TARGETS`] indices
//!   to 0. A name that `preset_isNameEmpty()` considers empty becomes `"Empty"`.
//! * File names: `preset_makeFileNameSave()`: two digit slot number, `-`, up to five characters of
//!   the name (characters outside `[A-Za-z0-9 !#$%&()-@^_`{}~]` and blanks become `_`), `.snd`.

use crate::{par, END_OF_SOUND_PARAMETERS, INIT_KIT, MOD_TARGETS};

/// Length of the kit name.
pub const KIT_NAME_LEN: usize = 8;
/// Number of parameter bytes in a kit.
pub const KIT_PARAMS: usize = END_OF_SOUND_PARAMETERS as usize;
/// Length of a complete kit file.
pub const KIT_FILE_LEN: usize = KIT_NAME_LEN + KIT_PARAMS;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KitError {
    /// Fewer than 8 bytes (no complete name): `preset_loadDrumsetSD()` fails with `FR_DISK_ERR`.
    TooShort,
}

impl std::fmt::Display for KitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KitError::TooShort => write!(f, "kit file is shorter than the 8 byte name"),
        }
    }
}

impl std::error::Error for KitError {}

/// A kit: name and the values of all sound parameters (`activeParameters[0..END_OF_SOUND_PARAMETERS]`).
#[derive(Clone, PartialEq, Eq)]
pub struct Kit {
    pub name: [u8; KIT_NAME_LEN],
    pub values: [u8; KIT_PARAMS],
}

impl std::fmt::Debug for Kit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Kit").field("name", &self.name_str()).finish_non_exhaustive()
    }
}

impl Default for Kit {
    fn default() -> Self {
        Self::init()
    }
}

/// `preset_isNameEmpty()`.
pub fn is_name_empty(name: &[u8; KIT_NAME_LEN]) -> bool {
    let e = b"Empty";
    for i in 0..5 {
        if name[i] != e[i] && name[i] != 0 {
            return false;
        }
    }
    name[5] == 0 && name[6] == 0 && name[7] == 0
}

fn empty_name() -> [u8; KIT_NAME_LEN] {
    *b"Empty\0\0\0"
}

impl Kit {
    /// The init kit as loaded by `preset_init()`: name from `initKit[0..8]`, then
    /// `para[i - 8] = initKit[i]` for `i` in `8..END_OF_SOUND_PARAMETERS` (parameters from
    /// `END_OF_SOUND_PARAMETERS - 8` on keep their previous value; here: 0), mod targets normalised.
    pub fn init() -> Kit {
        let mut name = [0u8; KIT_NAME_LEN];
        name.copy_from_slice(&INIT_KIT[..KIT_NAME_LEN]);
        if is_name_empty(&name) {
            name = empty_name();
        }
        let mut values = [0u8; KIT_PARAMS];
        values[..KIT_PARAMS - 8].copy_from_slice(&INIT_KIT[8..KIT_PARAMS]);
        let mut k = Kit { name, values };
        k.normalize_mod_targets();
        k
    }

    /// Parse a kit file (`preset_loadDrumsetSD()` + `preset_readDrumsetData()`).
    pub fn from_file_bytes(data: &[u8]) -> Result<Kit, KitError> {
        if data.len() < KIT_NAME_LEN {
            return Err(KitError::TooShort);
        }
        let mut name = [0u8; KIT_NAME_LEN];
        name.copy_from_slice(&data[..KIT_NAME_LEN]);
        if is_name_empty(&name) {
            name = empty_name();
        }
        let mut values = [0u8; KIT_PARAMS];
        let n = (data.len() - KIT_NAME_LEN).min(KIT_PARAMS);
        values[..n].copy_from_slice(&data[KIT_NAME_LEN..KIT_NAME_LEN + n]);
        let mut k = Kit { name, values };
        k.normalize_mod_targets();
        Ok(k)
    }

    /// The kit file bytes (`preset_saveDrumsetSD()` without morphing).
    pub fn to_file_bytes(&self) -> [u8; KIT_FILE_LEN] {
        let mut out = [0u8; KIT_FILE_LEN];
        out[..KIT_NAME_LEN].copy_from_slice(&self.name);
        out[KIT_NAME_LEN..].copy_from_slice(&self.values);
        out
    }

    /// Reset invalid modulation destinations to 0 (`preset_readDrumsetData()`).
    pub fn normalize_mod_targets(&mut self) {
        let nmt = MOD_TARGETS.len();
        for i in 0..6u16 {
            for base in [par::VEL_DEST_1, par::TARGET_LFO1] {
                let p = (base + i) as usize;
                if self.values[p] as usize >= nmt {
                    self.values[p] = 0;
                }
            }
        }
    }

    /// Set invalid LFO destination voices to 1, as `preset_sendDrumsetParameters()` does when a kit
    /// is sent to the engine.
    pub fn normalize_lfo_voices(&mut self) {
        for i in 0..6u16 {
            let p = (par::VOICE_LFO1 + i) as usize;
            if !(1..=6).contains(&self.values[p]) {
                self.values[p] = 1;
            }
        }
    }

    /// The name as text (up to the first NUL, other bytes as Latin-1).
    pub fn name_str(&self) -> String {
        let end = self.name.iter().position(|b| *b == 0).unwrap_or(KIT_NAME_LEN);
        self.name[..end].iter().map(|b| *b as char).collect()
    }

    /// Set the name (truncated to 8 bytes, NUL padded; characters outside Latin-1 become `_`).
    pub fn set_name(&mut self, name: &str) {
        self.name = [0; KIT_NAME_LEN];
        for (i, c) in name.chars().take(KIT_NAME_LEN).enumerate() {
            self.name[i] = if (c as u32) < 256 { c as u8 } else { b'_' };
        }
    }

    /// `preset_makeFileNameSave(buf, slot, FEXT_SOUND, name)`.
    pub fn file_name(&self, slot: u8) -> String {
        let mut s = String::new();
        // numtostrpu_2Bytes
        if slot > 99 {
            s.push_str(">>");
        } else {
            s.push_str(&format!("{slot:02}"));
        }
        s.push('-');
        let cleared = clear_illegal_characters(&self.name);
        for &c in cleared.iter().take(5) {
            if c == 0 {
                break;
            }
            s.push(if c == b' ' { '_' } else { c as char });
        }
        s.push_str(".snd");
        s
    }
}

/// `clearIllegalCharacters()`.
fn clear_illegal_characters(input: &[u8; KIT_NAME_LEN]) -> [u8; KIT_NAME_LEN] {
    let mut out = [0u8; KIT_NAME_LEN];
    for i in 0..KIT_NAME_LEN {
        let c = input[i];
        out[i] = if c.is_ascii_alphanumeric() || b"!#$%&()-@^_`{}~".contains(&c) || c == 0 {
            c
        } else {
            b'_' // includes ' '
        };
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn init_kit_matches_param_defaults() {
        let k = Kit::init();
        assert_eq!(k.name_str(), "Initkit");
        for p in crate::PARAMS.iter() {
            assert_eq!(k.values[p.index as usize] as u16, p.default, "{}", p.enum_name);
        }
    }

    #[test]
    fn roundtrip() {
        let mut k = Kit::init();
        k.set_name("My Kit 1");
        k.values[par::VOL1 as usize] = 99;
        let b = k.to_file_bytes();
        assert_eq!(b.len(), 8 + 247);
        let k2 = Kit::from_file_bytes(&b).unwrap();
        assert_eq!(k, k2);
        assert_eq!(k.file_name(3), "03-My_Ki.snd");
    }

    #[test]
    fn short_files_and_empty_names() {
        assert_eq!(Kit::from_file_bytes(b"abc"), Err(KitError::TooShort));
        let k = Kit::from_file_bytes(&[0u8; 10]).unwrap();
        assert_eq!(k.name_str(), "Empty");
        let mut data = vec![b'x'; 8];
        data.extend(std::iter::repeat_n(255u8, KIT_PARAMS));
        let k = Kit::from_file_bytes(&data).unwrap();
        assert_eq!(k.values[par::VEL_DEST_3 as usize], 0);
        assert_eq!(k.values[par::TARGET_LFO6 as usize], 0);
        assert_eq!(k.values[par::VOL1 as usize], 255);
    }
}
