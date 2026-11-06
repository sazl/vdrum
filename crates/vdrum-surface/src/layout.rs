// SPDX-License-Identifier: GPL-3.0-only
//! What every control does in every mode: the design's mapping tables (revision 2,
//! `lcxl3-daw-mode/source/layout.py`) as data.

use lcxl3::{Button, Control};
#[cfg(test)]
use lxr_params::par;

/// The seven tracks, as named on the controller.
pub const TRACKS: [&str; 7] = ["D1", "D2", "D3", "SD", "CY", "CH", "OH"];
/// Long track titles.
pub const TRACK_TITLES: [&str; 7] = ["DRUM 1", "DRUM 2", "DRUM 3", "SNARE", "CYMBAL", "CL. HAT", "OP. HAT"];

/// An RGB colour, 8 bits per component.
pub type Rgb = (u8, u8, u8);

/// Track colours (column-per-track modes, pads).
pub const TRACK_COLOURS: [Rgb; 7] =
    [(255, 120, 90), (255, 170, 70), (240, 215, 80), (120, 210, 110), (80, 200, 220), (110, 150, 255), (170, 120, 255)];
pub const FX_COLOUR: Rgb = (238, 99, 82);

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
        match self {
            Section::Osc => (244, 162, 89),
            Section::Amp => (91, 186, 111),
            Section::Mod => (242, 208, 75),
            Section::Fm => (171, 130, 222),
            Section::Trans => (232, 106, 166),
            Section::Filter => (95, 165, 230),
            Section::Lfo => (53, 201, 194),
            Section::Velo => (214, 172, 128),
            Section::Mix => (190, 198, 210),
            Section::Fx => FX_COLOUR,
            Section::Moment => (255, 196, 64),
            Section::Seq => (140, 220, 120),
            Section::Step => (250, 240, 150),
            Section::Macro => (200, 150, 255),
            Section::Global => (150, 160, 185),
            Section::Device => (120, 130, 150),
            Section::SeqTrack(t) | Section::Perf(t) | Section::Pad(t) => TRACK_COLOURS[(t as usize).min(6)],
            Section::Edit => (180, 190, 205),
            Section::Nav => (70, 76, 88),
            Section::Spare => (0, 0, 0),
        }
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

    /// Slot number 1..=16 (Shift + button N).
    pub fn number(self) -> u8 {
        match self {
            Mode::Voice(t) => t + 1,
            Mode::Sequencer => 8,
            Mode::MixFx => 9,
            Mode::Perform => 10,
            Mode::LfoMod => 11,
            Mode::MorphMacro => 12,
            Mode::Global => 16,
        }
    }

    /// The mode in slot `n`; `None` for an empty slot (13..=15) or a number outside 1..=16.
    pub fn from_number(n: u8) -> Option<Mode> {
        Mode::ALL.into_iter().find(|m| m.number() == n)
    }

    pub fn name(self) -> &'static str {
        match self {
            Mode::Voice(0) => "Drum 1",
            Mode::Voice(1) => "Drum 2",
            Mode::Voice(2) => "Drum 3",
            Mode::Voice(3) => "Snare",
            Mode::Voice(4) => "Cymbal",
            Mode::Voice(5) => "Closed hi-hat",
            Mode::Voice(_) => "Open hi-hat",
            Mode::Sequencer => "Sequencer",
            Mode::MixFx => "Mix + FX",
            Mode::Perform => "Perform",
            Mode::LfoMod => "LFO + Mod",
            Mode::MorphMacro => "Morph + Macro",
            Mode::Global => "Global",
        }
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

/// Parameter id of `kind` for track `t` (0..7), as `vid` in `layout.py`.
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

/// Knob in row `r` (1..=3), column `c` (1..=8), as `K<r><c>` in `layout.py`.
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

/// The mapping tables as they were written in Rust before the spec (revision 2 of `layout.py`),
/// kept as the oracle for the spec's parity test.
#[cfg(test)]
pub(crate) mod legacy {
    use super::*;

    impl Layout {
        fn set(&mut self, c: Control, label: impl Into<String>, section: Section, binding: Binding) {
            self.put(c, Cell { label: label.into(), section, binding });
        }

        fn param(&mut self, c: Control, label: &str, section: Section, idx: u16) {
            self.set(c, label, section, Binding::Param(idx));
        }

        fn spare(&mut self, c: Control) {
            self.set(c, "", Section::Spare, Binding::Spare);
        }
    }

    pub fn of(mode: Mode, held: bool) -> Layout {
        match mode {
            Mode::Voice(t @ 0..=2) => drum(t),
            Mode::Voice(3) => snare(),
            Mode::Voice(4) => cymbal(),
            Mode::Voice(t) => hat(t.min(6)),
            Mode::Sequencer => sequencer(held),
            Mode::MixFx => mix_fx(),
            Mode::Perform => perform(),
            Mode::LfoMod => lfo_mod(),
            Mode::MorphMacro => morph_macro(),
            Mode::Global => global(),
        }
    }

    pub fn with_steps(mut l: Layout) -> Layout {
        steps(&mut l);
        l
    }

    fn steps(m: &mut Layout) {
        for c in 0..8u8 {
            m.set(t(c + 1), format!("Step {}", c + 1), Section::Seq, Binding::SeqStep(c));
            m.set(b(c + 1), format!("Step {}", c + 9), Section::Seq, Binding::SeqStep(c + 8));
        }
    }

    fn pads(m: &mut Layout) {
        for c in 0..7u8 {
            m.set(b(c + 1), format!("{} pad", TRACKS[c as usize]), Section::Pad(c), Binding::Pad(c));
        }
        m.set(b(8), "Accent", Section::Pad(6), Binding::Accent);
    }

    fn lfo_row(m: &mut Layout, v: u8) {
        let keys = ["lfo_rate", "lfo_amt", "lfo_ofs", "lfo_wave", "lfo_sync", "lfo_rtg", "lfo_voice", "lfo_dest"];
        let names = ["Rate", "Amount", "Offset", "Wave", "Sync", "Retrigger", "Dest voice", "Dest param"];
        for (c, (n, key)) in names.iter().zip(keys).enumerate() {
            m.param(k(3, c as u8 + 1), &format!("LFO {n}"), Section::Lfo, voice_param(v, key));
        }
    }

    fn voice_common(m: &mut Layout, v: u8) {
        use Section::*;
        m.param(t(1), "Osc wave", Osc, voice_param(v, "wave"));
        m.param(k(1, 1), "Fine", Osc, voice_param(v, "fine"));
        m.param(k(2, 1), "Pulse width", Osc, voice_param(v, "pwm"));
        m.param(f(1), "Coarse", Osc, voice_param(v, "coarse"));
        m.set(t(2), "Copy voice", Edit, Binding::CopyVoice);
        m.param(k(1, 2), "Attack", Amp, voice_param(v, "attack"));
        m.param(k(2, 2), "Amp slope", Amp, voice_param(v, "slope"));
        m.param(f(2), "Decay", Amp, voice_param(v, "decay"));
        m.set(t(3), "Paste voice", Edit, Binding::PasteVoice);
        m.param(t(5), "Trans wave", Trans, voice_param(v, "trans_wave"));
        m.param(k(1, 5), "Trans freq", Trans, voice_param(v, "trans_freq"));
        m.param(f(5), "Trans volume", Trans, voice_param(v, "trans_vol"));
        m.param(t(6), "Filter type", Filter, voice_param(v, "ftype"));
        m.param(k(1, 6), "Resonance", Filter, voice_param(v, "reso"));
        m.param(k(2, 6), "Filter drive", Filter, voice_param(v, "fdrive"));
        m.param(f(6), "Cutoff", Filter, voice_param(v, "cutoff"));
        m.param(t(7), "Vel > Vol", Velo, voice_param(v, "velvol"));
        m.param(k(1, 7), "Vel dest", Velo, voice_param(v, "veldest"));
        m.param(k(2, 7), "Vel amount", Velo, voice_param(v, "velamt"));
        m.param(f(7), "Sample rate", Mix, voice_param(v, "srate"));
        m.param(t(8), "Audio out", Mix, voice_param(v, "out"));
        m.param(k(1, 8), "Pan", Mix, voice_param(v, "pan"));
        m.param(k(2, 8), "Overdrive", Mix, voice_param(v, "drive"));
        m.param(f(8), "Volume", Mix, voice_param(v, "vol"));
        lfo_row(m, v);
        pads(m);
    }

    fn drum(t0: u8) -> Layout {
        use Section::*;
        let v = t0 + 1;
        let mut m = Layout::default();
        voice_common(&mut m, t0);
        m.param(k(1, 3), "Pitch EG decay", Mod, p(&format!("mod_eg{v}")));
        m.param(k(2, 3), "Pitch EG slope", Mod, p(&format!("pitch_slope{v}")));
        m.param(f(3), "Pitch EG amt", Mod, p(&format!("modamnt{v}")));
        m.param(t(4), "FM / Mix", Fm, p(&format!("mix_mod_{v}")));
        m.param(k(1, 4), "FM freq", Fm, p(&format!("fm_freq{v}")));
        m.param(k(2, 4), "FM wave", Fm, p(&format!("mod_wave_drum{v}")));
        m.param(f(4), "FM amount", Fm, p(&format!("fmamnt{v}")));
        m.spare(k(2, 5));
        m
    }

    fn snare() -> Layout {
        use Section::*;
        let mut m = Layout::default();
        voice_common(&mut m, 3);
        m.param(k(1, 3), "Pitch EG decay", Mod, p("mod_eg4"));
        m.param(k(2, 3), "Pitch EG slope", Mod, p("pitch_slope4"));
        m.param(f(3), "Pitch EG amt", Mod, p("modamnt4"));
        m.spare(t(4));
        m.param(k(1, 4), "Noise freq", Fm, p("noise_freq1"));
        m.spare(k(2, 4));
        m.param(f(4), "Noise/Osc mix", Fm, p("mix1"));
        m.param(k(2, 5), "Amp repeat", Amp, p("repeat4"));
        m
    }

    fn metal(m: &mut Layout, ids: [&str; 6]) {
        use Section::*;
        let [f1, f2, g1, g2, w1, w2] = ids;
        m.param(k(1, 3), "Metal freq 1", Fm, p(f1));
        m.param(k(2, 3), "Metal wave 1", Fm, p(w1));
        m.param(f(3), "Metal gain 1", Fm, p(g1));
        m.param(k(1, 4), "Metal freq 2", Fm, p(f2));
        m.param(k(2, 4), "Metal wave 2", Fm, p(w2));
        m.param(f(4), "Metal gain 2", Fm, p(g2));
    }

    fn cymbal() -> Layout {
        let mut m = Layout::default();
        voice_common(&mut m, 4);
        metal(&mut m, ["mod_osc_f1_cym", "mod_osc_f2_cym", "mod_osc_gain1_cym", "mod_osc_gain2_cym", "wave2_cym", "wave3_cym"]);
        m.spare(t(4));
        m.param(k(2, 5), "Amp repeat", Section::Amp, p("repeat5"));
        m
    }

    fn hat(t0: u8) -> Layout {
        let mut m = Layout::default();
        voice_common(&mut m, t0);
        if t0 == 5 {
            metal(&mut m, ["mod_osc_f1", "mod_osc_f2", "mod_osc_gain1", "mod_osc_gain2", "wave2_hh", "wave3_hh"]);
            m.param(t(4), "Choke OH", Section::Amp, p("hat_choke"));
        } else {
            metal(&mut m, ["mod_osc_f1_oh", "mod_osc_f2_oh", "mod_osc_gain1_oh", "mod_osc_gain2_oh", "wave2_oh", "wave3_oh"]);
            m.param(t(4), "Choke by CH", Section::Amp, p("hat_choke"));
        }
        m.spare(k(2, 5));
        m
    }

    fn sequencer(held: bool) -> Layout {
        use Section::*;
        let mut m = Layout::default();
        for c in 0..7u8 {
            let tr = TRACKS[c as usize];
            m.set(k(1, c + 1), format!("{tr} Fills"), SeqTrack(c), Binding::SeqFills(c));
            m.set(k(2, c + 1), format!("{tr} Rotate"), SeqTrack(c), Binding::SeqRotate(c));
            m.set(k(3, c + 1), format!("{tr} Length"), SeqTrack(c), Binding::SeqLength(c));
            m.set(f(c + 1), format!("{tr} Density"), SeqTrack(c), Binding::SeqDensity(c));
        }
        m.set(k(1, 8), "Shuffle", Seq, Binding::SeqShuffle);
        m.set(k(2, 8), "Next pattern", Seq, Binding::SeqNextPattern);
        m.set(k(3, 8), "Entry velocity", Seq, Binding::SeqEntryVelocity);
        m.set(f(8), "Density all", Seq, Binding::SeqDensityAll);
        steps(&mut m);
        if held {
            let row1 = [
                ("Velocity", StepField::Velocity),
                ("Probability", StepField::Probability),
                ("Note", StepField::Note),
                ("Flam / ratchet", StepField::Flam),
                ("Micro-shift", StepField::Shift),
                ("Lock 1 value", StepField::Lock1),
                ("Lock 2 value", StepField::Lock2),
                ("Clear locks", StepField::ClearLocks),
            ];
            for (c, (n, fld)) in row1.into_iter().enumerate() {
                m.set(k(1, c as u8 + 1), n, Step, Binding::StepField(fld));
            }
            for (c, n) in LOCK_NAMES.iter().enumerate() {
                m.set(k(2, c as u8 + 1), format!("Lock {n}"), Step, Binding::LockTarget(c as u8));
            }
        }
        m
    }

    fn mix_fx() -> Layout {
        use Section::*;
        let mut m = Layout::default();
        m.param(k(1, 1), "FX P1", Fx, par::FX_P1);
        m.param(k(1, 2), "FX P2", Fx, par::FX_P2);
        m.param(k(1, 3), "FX P3", Fx, par::FX_P3);
        m.param(k(1, 4), "FX P4", Fx, par::FX_P4);
        m.param(k(1, 5), "FX type", Fx, par::FX_TYPE);
        m.set(k(1, 6), "FX sub-type", Fx, Binding::FxSubType);
        m.param(k(1, 7), "FX output", Fx, par::FX_ROUTING);
        m.param(k(1, 8), "Global S.rate", Global, par::VOICE_DECIMATION_ALL);
        for c in 0..7u8 {
            let tr = TRACKS[c as usize];
            m.param(k(2, c + 1), &format!("{tr} Pan"), Mix, voice_param(c, "pan"));
            m.param(k(3, c + 1), &format!("{tr} Drive"), Mix, voice_param(c, "drive"));
            m.param(f(c + 1), &format!("{tr} Volume"), Mix, voice_param(c, "vol"));
            m.set(t(c + 1), format!("{tr} > FX"), Fx, Binding::ToFx(c));
        }
        m.set(k(2, 8), "Effect depth", Moment, Binding::MomentDepth);
        m.set(k(3, 8), "Effect rate", Moment, Binding::MomentRate);
        m.param(f(8), "FX dry/wet", Fx, par::FX_DRY_WET);
        m.set(t(8), "FX on/off", Fx, Binding::FxOnOff);
        for (c, n) in MOMENT_NAMES.iter().enumerate() {
            m.set(b(c as u8 + 1), *n, Moment, Binding::Moment(c as u8));
        }
        m
    }

    fn perform() -> Layout {
        use Section::*;
        let mut m = Layout::default();
        for c in 0..7u8 {
            let tr = TRACKS[c as usize];
            m.param(k(1, c + 1), &format!("{tr} Pitch"), Perf(c), voice_param(c, "coarse"));
            m.param(k(2, c + 1), &format!("{tr} Decay"), Perf(c), voice_param(c, "decay"));
            m.param(k(3, c + 1), &format!("{tr} Cutoff"), Perf(c), voice_param(c, "cutoff"));
            m.set(f(c + 1), format!("{tr} Level"), Perf(c), Binding::VoiceFader(c));
            m.set(t(c + 1), format!("{tr} Trigger"), Perf(c), Binding::Trigger(c));
            m.set(b(c + 1), format!("{tr} Mute"), Perf(c), Binding::Mute(c));
        }
        m.param(k(1, 8), "FX P1", Fx, par::FX_P1);
        m.param(k(2, 8), "FX P2", Fx, par::FX_P2);
        m.param(k(3, 8), "Global S.rate", Fx, par::VOICE_DECIMATION_ALL);
        m.param(f(8), "FX dry/wet", Fx, par::FX_DRY_WET);
        m.set(t(8), "FX on/off", Fx, Binding::FxOnOff);
        m.set(b(8), "Mute all", Perf(6), Binding::MuteAll);
        m
    }

    fn lfo_mod() -> Layout {
        use Section::*;
        let mut m = Layout::default();
        for c in 0..7u8 {
            let tr = TRACKS[c as usize];
            m.param(k(1, c + 1), &format!("{tr} LFO rate"), Lfo, voice_param(c, "lfo_rate"));
            m.param(k(2, c + 1), &format!("{tr} LFO amt"), Lfo, voice_param(c, "lfo_amt"));
            m.param(k(3, c + 1), &format!("{tr} LFO dest"), Lfo, voice_param(c, "lfo_dest"));
            m.param(f(c + 1), &format!("{tr} Vel amt"), Velo, voice_param(c, "velamt"));
            m.param(t(c + 1), &format!("{tr} LFO wave"), Lfo, voice_param(c, "lfo_wave"));
            m.param(b(c + 1), &format!("{tr} LFO sync"), Lfo, voice_param(c, "lfo_sync"));
        }
        m.set(k(1, 8), "Rate scale", Lfo, Binding::LfoRateScale);
        m.set(k(2, 8), "Vel scale", Velo, Binding::VelScale);
        m.set(k(3, 8), "LFO offset all", Lfo, Binding::LfoOffsetAll);
        m.set(f(8), "Depth scale", Lfo, Binding::LfoDepthScale);
        m.set(t(8), "Sync to host", Lfo, Binding::LfoSyncHost);
        m.set(b(8), "Reset phases", Lfo, Binding::LfoResetPhases);
        m
    }

    fn morph_macro() -> Layout {
        use Section::*;
        let mut m = Layout::default();
        for c in 0..8u8 {
            m.set(f(c + 1), format!("Macro {}", c + 1), Macro, Binding::Macro(c));
            m.set(k(1, c + 1), format!("Macro {} depth", c + 1), Macro, Binding::MacroDepth(c));
            m.set(t(c + 1), format!("Snapshot {}", c + 1), Macro, Binding::Snapshot(c));
            m.set(b(c + 1), format!("Learn {}", c + 1), Macro, Binding::Learn(c));
        }
        for c in 0..7u8 {
            let tr = TRACKS[c as usize];
            m.set(k(2, c + 1), format!("{tr} Morph"), Macro, Binding::TrackMorph(c));
            m.set(k(3, c + 1), format!("{tr} Target"), Macro, Binding::TrackTarget(c));
        }
        m.set(k(2, 8), "Morph all", Macro, Binding::MorphAll);
        m.set(k(3, 8), "Target all", Macro, Binding::TargetAll);
        m
    }

    fn global() -> Layout {
        use Section::*;
        let mut m = Layout::default();
        for c in 0..7u8 {
            let tr = TRACKS[c as usize];
            m.param(k(1, c + 1), &format!("{tr} note"), Global, par::MIDI_NOTE1 + c as u16);
            m.param(k(2, c + 1), &format!("{tr} channel"), Global, p(&format!("midi_chan_{}", c + 1)));
            m.set(f(c + 1), format!("{tr} HW fader"), Mix, Binding::VoiceFader(c));
        }
        m.param(k(1, 8), "Tempo", Global, par::BPM);
        m.param(k(2, 8), "Global ch.", Global, par::MIDI_CHAN_GLOBAL);
        let dev = [
            ("LED bright", Setting::LedBrightness),
            ("Screen bright", Setting::ScreenBrightness),
            ("Popup time", Setting::PopupTime),
            ("Enc. curve", Setting::EncoderCurve),
            ("Pad velocity", Setting::PadVelocity),
            ("List speed", Setting::ListSpeed),
            ("Fader mode", Setting::FaderMode),
            ("Clock", Setting::Clock),
        ];
        for (c, (n, s)) in dev.into_iter().enumerate() {
            m.set(k(3, c as u8 + 1), n, Device, Binding::Setting(s));
        }
        m.param(f(8), "Global S.rate", Global, par::VOICE_DECIMATION_ALL);
        let tops = [
            ("Sum aux", Binding::SumAux),
            ("HW clock", Binding::HwClock),
            ("Init kit", Binding::InitKit),
            ("Save kit", Binding::SaveKit),
            ("Compare", Binding::Compare),
            ("Undo", Binding::Undo),
            ("Redo", Binding::Redo),
            ("Panic", Binding::Panic),
        ];
        for (c, (n, bd)) in tops.into_iter().enumerate() {
            m.set(t(c as u8 + 1), n, Global, bd);
        }
        pads(&mut m);
        m
    }
}
