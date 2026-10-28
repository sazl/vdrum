// SPDX-License-Identifier: GPL-3.0-only
//! Parameter metadata of the LXR-02 sound engine as used by VDrum.
//!
//! Everything in [`generated`] is produced by `gen/gen_params.py` from the GPL-3.0 firmware source
//! (`Parameters.h`, `menu.h`, `MenuText.h`, `menu.c`, `menuPages.h`, `Cc2Text.c`, `initKit.c`,
//! `MidiParser.c`, `MidiMessages.h`, `presetManager.c`); see that script for the exact derivation.
//! [`format`] ports the menu's value display (`menu_repaintGeneric()` and helpers), [`kit`] the kit
//! file format of `presetManager.c`.
//!
//! Parameter values are the firmware's `activeParameters[]` bytes: the representation that is stored
//! in kit files and shown by the menu (for example modulation destinations are indices into
//! [`MOD_TARGETS`], not parameter numbers).

pub mod format;
pub mod generated;
pub mod kit;

pub use format::{format_value, format_value_short, parse_value};
pub use generated::*;

/// What kind of entry of `enum ParamEnums` a [`ParamInfo`] describes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParamKind {
    /// A sound parameter stored in kits and shown on a menu page.
    Sound,
    /// An index below `END_OF_SOUND_PARAMETERS` that is stored in kits but is not shown on any menu
    /// page and has no effect on the sound: `PAR_NONE`, the NRPN/bank select CC placeholders,
    /// `PAR_UNUSED_1..7`, `PAR_MORPH_KIT_DEST`.
    Unused,
    /// A global setting (not stored in kits) that changes how the engine reacts to MIDI/clock.
    Global,
}

/// The voice (menu page) a parameter belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Voice {
    Drum1,
    Drum2,
    Drum3,
    Snare,
    Cymbal,
    HiHat,
    /// The FX bus page.
    Fx,
    /// Global/performance settings.
    Global,
    None,
}

impl Voice {
    pub const VOICES: [Voice; 6] = [Voice::Drum1, Voice::Drum2, Voice::Drum3, Voice::Snare, Voice::Cymbal, Voice::HiHat];

    pub fn label(self) -> &'static str {
        match self {
            Voice::Drum1 => "Drum 1",
            Voice::Drum2 => "Drum 2",
            Voice::Drum3 => "Drum 3",
            Voice::Snare => "Snare",
            Voice::Cymbal => "Cymbal",
            Voice::HiHat => "Hi-hat",
            Voice::Fx => "FX",
            Voice::Global => "Global",
            Voice::None => "",
        }
    }

    /// 0-based voice number (0..=5) for the six sound voices.
    pub fn number(self) -> Option<u8> {
        Self::VOICES.iter().position(|v| *v == self).map(|i| i as u8)
    }
}

/// The section (menu sub page) a parameter belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Section {
    Oscillator,
    AmpEg,
    Modulation,
    Fm,
    Transient,
    Filter,
    Lfo,
    Mix,
    FxBus,
    FxDistortion,
    FxRing,
    FxCompressor,
    FxDelay,
    FxReverb,
    Performance,
    Midi,
    None,
}

impl Section {
    /// Sections of a voice page, in menu order.
    pub const VOICE_SECTIONS: [Section; 8] = [
        Section::Oscillator,
        Section::AmpEg,
        Section::Modulation,
        Section::Fm,
        Section::Transient,
        Section::Filter,
        Section::Lfo,
        Section::Mix,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Section::Oscillator => "Oscillator",
            Section::AmpEg => "Amp EG",
            Section::Modulation => "Modulation",
            Section::Fm => "FM",
            Section::Transient => "Transient",
            Section::Filter => "Filter",
            Section::Lfo => "LFO",
            Section::Mix => "Mix",
            Section::FxBus => "FX Bus",
            Section::FxDistortion => "Distortion",
            Section::FxRing => "Ring Mod",
            Section::FxCompressor => "Compressor",
            Section::FxDelay => "Delay",
            Section::FxReverb => "Reverb",
            Section::Performance => "Performance",
            Section::Midi => "MIDI",
            Section::None => "",
        }
    }
}

/// The menu data types of `enum Datatypes` (menu.h).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Dtype {
    U0to255,
    U0to127,
    Pm100,
    Menu,
    Pm63,
    U1to16,
    OnOff,
    TargetSelectionLfo,
    TargetSelectionVelo,
    VoiceLfo,
    AutomTarget,
    U0to1,
    NoteName,
    U0to15,
    U1to64,
    U0to100,
}

/// How the firmware maps external MIDI to a parameter (`midiParser_parseMidiMessage()`,
/// `midiParser_ccHandler()`, `frontPanel_ccHandler()`, `frontParser_parseNrpn()`).
///
/// * Parameters `1..=126` are controlled by CC number `index + 1` (7 bit). The CC is accepted on the
///   channel `midi_MidiChannels[cc_voice]` or on the global channel when `cc_voice` is 1..=6 (note:
///   `midi_MidiChannels[n]` is the channel of track `n + 1`, so the drum 1 CCs follow the drum 2
///   channel; this is what the firmware does), and on any channel that is set for a track or as
///   global channel when `cc_voice` is 0 or 7.
/// * Parameters `128..END_OF_SOUND_PARAMETERS` are controlled by NRPN number `index - 128`
///   (CC 99 = NRPN MSB, CC 98 = NRPN LSB, CC 6 = data entry; the value is 7 bit, there is no
///   data entry LSB). The NRPN CCs are accepted on any channel set for a track or as global channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MidiMapping {
    pub cc: Option<u8>,
    /// `midiParser_cc2voiceNr(cc)` (see [`CC_TO_VOICE`]).
    pub cc_voice: u8,
    pub nrpn: Option<u16>,
    /// `midiParser_ccHandler()` has a case for this CC / CC2 number that does something.
    pub engine_handles: bool,
    /// The front panel NRPN handler translates the received value (a parameter number) to a
    /// [`MOD_TARGETS`] index and sends it to the engine (`PAR_TARGET_LFOx`, `PAR_VEL_DEST_x`).
    pub front_translates_nrpn: bool,
}

impl MidiMapping {
    /// The MIDI value resolution in bits (all firmware CCs and NRPNs carry a 7 bit value).
    pub const fn bits(&self) -> u8 {
        7
    }

    /// Whether external MIDI changes the sound engine at all.
    pub const fn effective(&self) -> bool {
        self.engine_handles || self.front_translates_nrpn
    }
}

/// Metadata of one parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParamInfo {
    /// Value of the `enum ParamEnums` entry.
    pub index: u16,
    /// Name of the `enum ParamEnums` entry (`PAR_COARSE1`).
    pub enum_name: &'static str,
    /// Stable identifier for hosts: `enum_name` without `PAR_`, lower case.
    pub id: &'static str,
    pub kind: ParamKind,
    pub voice: Voice,
    pub section: Section,
    /// The `TEXT_*` (`enum NamesEnum`) the menu shows this parameter with.
    pub text: Option<u8>,
    /// 3 character menu label (`shortNames`), verbatim.
    pub short_name: &'static str,
    /// Menu category text (`catNames`), verbatim.
    pub category: &'static str,
    /// Menu long name (`longNames`), verbatim.
    pub long_name: &'static str,
    /// Readable unique name: voice + spelled-out category/section + long name.
    pub name: &'static str,
    /// Description text. Empty: the firmware source has none.
    pub description: &'static str,
    pub dtype: Dtype,
    pub menu: Option<MenuList>,
    pub min: u16,
    pub max: u16,
    /// Value in the init kit (`preset_init()` + `initKit[]`) or `preset_loadDefaultGlobals()`.
    pub default: u16,
    pub midi: MidiMapping,
    /// For per-track global settings: track 0..=6 (6 = open hi-hat).
    pub track: Option<u8>,
}

impl ParamInfo {
    /// Number of discrete values.
    pub fn steps(&self) -> u16 {
        self.max - self.min + 1
    }

    /// Whether the parameter is a modulation destination selector (index into [`MOD_TARGETS`]).
    pub fn is_mod_target(&self) -> bool {
        matches!(self.dtype, Dtype::TargetSelectionLfo | Dtype::TargetSelectionVelo | Dtype::AutomTarget)
    }

    /// Firmware edit-mode text for `value`.
    pub fn format(&self, value: u8) -> String {
        format_value(self, value)
    }

    /// The `(min, max)` [`MOD_TARGETS`] index range the menu lets the encoder select for a
    /// modulation destination, given the destination voice (0-based). Index 0 ("off") is always
    /// allowed as well.
    pub fn mod_target_range_for_voice(voice: u8) -> (u8, u8) {
        MOD_TARGET_VOICE_OFFSETS[voice.min(5) as usize]
    }
}

/// Information on a parameter: kit parameters by index, then the global settings.
pub fn info(index: u16) -> Option<&'static ParamInfo> {
    if (index as usize) < PARAMS.len() {
        return Some(&PARAMS[index as usize]);
    }
    GLOBAL_PARAMS.iter().find(|p| p.index == index)
}

/// Look up a parameter by its `enum ParamEnums` name (aliases such as `PAR_MOD_WHEEL` resolve to the
/// canonical entry of the same value).
pub fn by_enum_name(name: &str) -> Option<&'static ParamInfo> {
    PARAM_ENUM.iter().find(|(n, _)| *n == name).and_then(|(_, v)| info(*v))
}

/// Look up a parameter by its stable host identifier.
pub fn by_id(id: &str) -> Option<&'static ParamInfo> {
    PARAMS.iter().chain(GLOBAL_PARAMS.iter()).find(|p| p.id == id)
}

/// The parameters that change the sound (kind [`ParamKind::Sound`]), in index order.
pub fn sound_params() -> impl Iterator<Item = &'static ParamInfo> {
    PARAMS.iter().filter(|p| p.kind == ParamKind::Sound)
}

/// The voice (1-based, as `voiceFromModTargValue()`) a [`MOD_TARGETS`] index belongs to, 0 for "off"
/// and the global entry.
pub fn mod_target_voice(idx: u8) -> u8 {
    for (v, (start, end)) in MOD_TARGET_VOICE_OFFSETS.iter().enumerate() {
        if idx >= *start && idx <= *end {
            return v as u8 + 1;
        }
    }
    0
}

/// The parameter number a [`MOD_TARGETS`] index stands for (what the front panel sends to the engine).
pub fn mod_target_param(idx: u8) -> Option<u16> {
    MOD_TARGETS.get(idx as usize).map(|t| t.1)
}

/// `paramToModTarget[]` (Cc2Text.c, `paramToModTargetInit()`): the last [`MOD_TARGETS`] index that
/// refers to `param` (0 if none).
pub fn param_to_mod_target(param: u16) -> u8 {
    let mut r = 0u8;
    for (i, t) in MOD_TARGETS.iter().enumerate() {
        if t.1 == param {
            r = i as u8;
        }
    }
    r
}

/// The text (TEXT_* index) the FX page shows for `param` when the FX type is `fx_type`
/// (`PAR_FX_P1..4` mean different things for every FX).
pub fn fx_param_text(fx_type: u8, param: u16) -> Option<u8> {
    FX_PAGE.get(fx_type as usize)?.iter().find(|(p, _)| *p == param).map(|(_, t)| *t)
}

/// (short, category, long) texts of a TEXT_* index.
pub fn text_names(text: u8) -> (&'static str, &'static str, &'static str) {
    let (s, c, l) = VALUE_NAMES[text as usize];
    (SHORT_NAMES[s as usize], CAT_NAMES[c as usize], LONG_NAMES[l as usize])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn table_is_indexed_by_enum_value() {
        assert_eq!(PARAMS.len(), END_OF_SOUND_PARAMETERS as usize);
        for (i, p) in PARAMS.iter().enumerate() {
            assert_eq!(p.index as usize, i);
            assert!(p.min <= p.default && p.default <= p.max, "{}", p.enum_name);
        }
        for p in GLOBAL_PARAMS.iter() {
            assert!(p.index >= END_OF_SOUND_PARAMETERS && p.index < NUM_PARAMS);
        }
    }

    #[test]
    fn ids_and_names_unique() {
        let all: Vec<_> = PARAMS.iter().chain(GLOBAL_PARAMS.iter()).collect();
        for (i, a) in all.iter().enumerate() {
            for b in &all[i + 1..] {
                assert_ne!(a.id, b.id);
                assert_ne!(a.name, b.name);
            }
        }
    }

    #[test]
    fn known_entries() {
        let p = by_enum_name("PAR_COARSE1").unwrap();
        assert_eq!(p.index, par::COARSE1);
        assert_eq!(p.midi.cc, Some(par::COARSE1 as u8 + 1));
        assert_eq!((p.short_name, p.category, p.long_name), ("coa", "Oscilltr", "Coarse"));
        assert_eq!(by_enum_name("PAR_MOD_WHEEL").unwrap().index, par::OSC_WAVE_DRUM1);
        let f = by_enum_name("PAR_FILTER_DRIVE_1").unwrap();
        assert_eq!(f.midi.nrpn, Some(0));
        assert_eq!(by_id("fx_dry_wet").unwrap().max, 100);
        assert_eq!(mod_target_param(2), Some(par::COARSE1));
        assert_eq!(mod_target_voice(2), 1);
        assert_eq!(mod_target_voice(228), 6);
        assert_eq!(param_to_mod_target(par::FX_P1), 225);
    }
}
