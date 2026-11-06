// SPDX-License-Identifier: GPL-3.0-only
//! The types of a mode layout (cells, sections, behaviours) and the modes. The layouts themselves
//! come from the surface spec ([`crate::spec`]), authored in `surface/` (Python).

use lcxl3::{Button, Control};

/// The seven tracks, as named on the controller.
pub const TRACKS: [&str; 7] = ["D1", "D2", "D3", "SD", "CY", "CH", "OH"];
/// Long track titles.
pub const TRACK_TITLES: [&str; 7] = ["DRUM 1", "DRUM 2", "DRUM 3", "SNARE", "CYMBAL", "CL. HAT", "OP. HAT"];

/// An RGB colour, 8 bits per component.
pub type Rgb = (u8, u8, u8);

/// Track `t`'s colour (column-per-track modes, pads) in the built-in spec's palette.
pub fn track_colour(t: u8) -> Rgb {
    crate::spec::builtin().rules.palette.tracks[(t as usize).min(6)]
}

/// What a control belongs to; sets its LED colour and the popup title.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Section {
    Osc,
    Amp,
    Mod,
    Fm,
    Trans,
    Filter,
    Lfo,
    Velo,
    Mix,
    Fx,
    Moment,
    Seq,
    /// Sequencer, in the colour of the column's track.
    SeqTrack(u8),
    Step,
    Macro,
    Global,
    Device,
    /// Performance action in the colour of the column's track.
    Perf(u8),
    /// Trigger pad of a track.
    Pad(u8),
    Edit,
    Nav,
    Spare,
}

impl Section {
    pub fn label(self) -> &'static str {
        match self {
            Section::Osc => "Oscillator",
            Section::Amp => "Amp EG",
            Section::Mod => "Pitch mod EG",
            Section::Fm => "FM / noise / metal",
            Section::Trans => "Transient",
            Section::Filter => "Filter",
            Section::Lfo => "LFO",
            Section::Velo => "Velocity mod",
            Section::Mix => "Mix",
            Section::Fx => "FX bus",
            Section::Moment => "Momentary effect",
            Section::Seq | Section::SeqTrack(_) => "Sequencer",
            Section::Step => "Step",
            Section::Macro => "Macro / morph",
            Section::Global => "Global",
            Section::Device => "Controller",
            Section::Perf(_) => "Perform",
            Section::Pad(_) => "Pad",
            Section::Edit => "Edit",
            Section::Nav => "Navigation",
            Section::Spare => "Unassigned",
        }
    }

    pub fn colour(self) -> Rgb {
        crate::spec::builtin().colour(self)
    }
}

/// Controller and plugin settings of the Global mode's third knob row.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Setting {
    /// Device feature CC 111.
    LedBrightness,
    /// Device feature CC 112.
    ScreenBrightness,
    /// Device feature CC 113.
    PopupTime,
    /// Device feature CC 121.
    EncoderCurve,
    /// Velocity of the trigger pads.
    PadVelocity,
    /// Encoder detents per entry of a list.
    ListSpeed,
    /// Faders pick up the parameter (default) or jump to the fader position.
    FaderMode,
    /// Sequencer clock: host transport or internal tempo.
    Clock,
}

/// Fields of a held step (sequencer, step held).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum StepField {
    Velocity,
    Probability,
    Note,
    Flam,
    Shift,
    Lock1,
    Lock2,
    ClearLocks,
}

/// What a control does.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Binding {
    /// An engine parameter (index as in `lxr_params`). Encoders step it, faders set it absolutely
    /// (with pickup), buttons toggle (two values) or cycle through its values.
    Param(u16),
    /// The plugin's voice fader of a track (0..7).
    VoiceFader(u8),
    /// Trigger pad of a track.
    Pad(u8),
    /// While held, pads play at velocity 127.
    Accent,
    CopyVoice,
    /// Hold 1 s: paste the copied voice.
    PasteVoice,
    /// Send a track to the FX bus and back.
    ToFx(u8),
    /// FX type off and back.
    FxOnOff,
    /// The FX type's own type parameter (distortion type, ring waveform, ratio, delay type).
    FxSubType,
    /// Momentary effect 0..8 (active while held).
    Moment(u8),
    MomentDepth,
    MomentRate,
    SeqFills(u8),
    SeqRotate(u8),
    SeqLength(u8),
    SeqDensity(u8),
    SeqShuffle,
    SeqNextPattern,
    SeqEntryVelocity,
    SeqDensityAll,
    /// Step 0..16 of the shown bar.
    SeqStep(u8),
    StepField(StepField),
    /// Parameter lock target 0..8 (pitch, decay, cutoff, reso, drive, pan, sample rate, FX send).
    LockTarget(u8),
    Trigger(u8),
    Mute(u8),
    MuteAll,
    LfoRateScale,
    VelScale,
    LfoOffsetAll,
    LfoDepthScale,
    LfoSyncHost,
    LfoResetPhases,
    Macro(u8),
    MacroDepth(u8),
    Snapshot(u8),
    Learn(u8),
    TrackMorph(u8),
    TrackTarget(u8),
    MorphAll,
    TargetAll,
    SumAux,
    HwClock,
    InitKit,
    SaveKit,
    Compare,
    Undo,
    Redo,
    Panic,
    Setting(Setting),
    Spare,
}

/// One control's assignment in a mode.
#[derive(Clone, Debug, PartialEq)]
pub struct Cell {
    pub label: String,
    pub section: Section,
    pub binding: Binding,
}

/// A surface mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Mode {
    /// Voice modes 1..=7: track 0..7.
    Voice(u8),
    Sequencer,
    MixFx,
    Perform,
    LfoMod,
    MorphMacro,
    Global,
}

impl Mode {
    /// The modes that exist, in slot order.
    pub const ALL: [Mode; 13] = [
        Mode::Voice(0),
        Mode::Voice(1),
        Mode::Voice(2),
        Mode::Voice(3),
        Mode::Voice(4),
        Mode::Voice(5),
        Mode::Voice(6),
        Mode::Sequencer,
        Mode::MixFx,
        Mode::Perform,
        Mode::LfoMod,
        Mode::MorphMacro,
        Mode::Global,
    ];

    /// Slot number 1..=16 (Shift + button N) in the built-in spec.
    pub fn number(self) -> u8 {
        crate::spec::builtin().mode_info(self).0
    }

    /// The mode in slot `n` of the built-in spec; `None` for an empty slot.
    pub fn from_number(n: u8) -> Option<Mode> {
        crate::spec::builtin().mode_at(n)
    }

    /// The mode's name in the built-in spec.
    pub fn name(self) -> &'static str {
        crate::spec::builtin().mode_info(self).1
    }

    /// The next (`dir` > 0) or previous existing mode, wrapping.
    pub fn step(self, dir: i8) -> Mode {
        let i = Mode::ALL.iter().position(|m| *m == self).unwrap_or(0) as i32;
        let n = Mode::ALL.len() as i32;
        Mode::ALL[(i + dir.signum() as i32).rem_euclid(n) as usize]
    }

    /// The track a voice mode edits.
    pub fn track(self) -> Option<u8> {
        match self {
            Mode::Voice(t) => Some(t),
            _ => None,
        }
    }
}

/// Parameter index of a parameter id. Panics on an unknown id (the layouts are checked by tests).
pub fn p(id: &str) -> u16 {
    lxr_params::by_id(id).unwrap_or_else(|| panic!("unknown parameter id {id}")).index
}

/// Parameter id of `kind` for track `t` (0..7), as `vid` in `tools/surface_spec/surface_spec/voice.py`.
pub fn voice_param(t: u8, kind: &str) -> u16 {
    let v = t + 1;
    let id = match kind {
        "drive" => ["drive1", "drive2", "drive3", "snare_distortion", "cymbal_distortion", "hat_distortion", "oh_distortion"]
            [t as usize]
            .to_string(),
        "decay" => match v {
            6 => "velod6_closed".to_string(),
            _ => format!("velod{v}"),
        },
        "wave" => ["osc_wave_drum1", "osc_wave_drum2", "osc_wave_drum3", "osc_wave_snare", "wave1_cym", "wave1_hh", "wave1_oh"]
            [t as usize]
            .to_string(),
        "coarse" => format!("coarse{v}"),
        "fine" => format!("fine{v}"),
        "pwm" => format!("pwm_{v}"),
        "attack" => format!("veloa{v}"),
        "slope" => format!("vol_slope{v}"),
        "trans_wave" => format!("trans{v}_wave"),
        "trans_freq" => format!("trans{v}_freq"),
        "trans_vol" => format!("trans{v}_vol"),
        "ftype" => format!("filter_type_{v}"),
        "reso" => format!("reso_{v}"),
        "fdrive" => format!("filter_drive_{v}"),
        "cutoff" => format!("filter_freq_{v}"),
        "velvol" => format!("volume_mod_on_off{v}"),
        "veldest" => format!("vel_dest_{v}"),
        "velamt" => format!("velo_mod_amt_{v}"),
        "srate" => format!("voice_decimation{v}"),
        "out" => format!("audio_out{v}"),
        "pan" => format!("pan{v}"),
        "vol" => format!("vol{v}"),
        "lfo_rate" => format!("freq_lfo{v}"),
        "lfo_amt" => format!("amount_lfo{v}"),
        "lfo_ofs" => format!("offset_lfo{v}"),
        "lfo_wave" => format!("wave_lfo{v}"),
        "lfo_sync" => format!("sync_lfo{v}"),
        "lfo_rtg" => format!("retrigger_lfo{v}"),
        "lfo_voice" => format!("voice_lfo{v}"),
        "lfo_dest" => format!("target_lfo{v}"),
        _ => panic!("unknown voice parameter kind {kind}"),
    };
    p(&id)
}

/// Knob in row `r` (1..=3), column `c` (1..=8), as `K<r><c>` in the spec.
pub fn k(r: u8, c: u8) -> Control {
    Control::Encoder { row: r - 1, col: c - 1 }
}
pub fn f(c: u8) -> Control {
    Control::Fader(c - 1)
}
pub fn t(c: u8) -> Control {
    Control::Button(Button::Top(c - 1))
}
pub fn b(c: u8) -> Control {
    Control::Button(Button::Bottom(c - 1))
}

/// A mode's assignments.
#[derive(Clone, Debug, Default)]
pub struct Layout {
    cells: Vec<(Control, Cell)>,
}

impl Layout {
    pub fn get(&self, c: Control) -> Option<&Cell> {
        self.cells.iter().find(|(k, _)| *k == c).map(|(_, v)| v)
    }

    pub fn iter(&self) -> impl Iterator<Item = &(Control, Cell)> {
        self.cells.iter()
    }

    /// Set the cell of control `c`, replacing an existing cell in place.
    pub(crate) fn put(&mut self, c: Control, cell: Cell) {
        match self.cells.iter_mut().find(|(k, _)| *k == c) {
            Some(slot) => slot.1 = cell,
            None => self.cells.push((c, cell)),
        }
    }

    /// This layout with the sequencer view's step buttons laid over it (the 16 buttons become the
    /// step buttons in any mode), from the built-in spec.
    pub fn with_steps(self) -> Layout {
        crate::spec::builtin().with_steps(self)
    }

    /// The layout of `mode` from the built-in spec; `held` lays the held-step cells over the
    /// sequencer's layout.
    pub fn of(mode: Mode, held: bool) -> Layout {
        crate::spec::builtin().layout(mode, held)
    }
}

/// Names of the eight parameter-lock targets (sequencer, step held).
pub const LOCK_NAMES: [&str; 8] = ["Pitch", "Decay", "Cutoff", "Reso", "Drive", "Pan", "S.rate", "FX send"];
/// The voice parameter kind of each lock target.
pub const LOCK_KINDS: [&str; 8] = ["coarse", "decay", "cutoff", "reso", "drive", "pan", "srate", "out"];

/// Names of the eight momentary effects (Mix + FX bottom row).
pub const MOMENT_NAMES: [&str; 8] =
    ["Stutter 1/16", "Stutter 1/32", "Filter sweep", "Reso up", "Swell", "Long decay", "Lo-fi", "FX throw"];
