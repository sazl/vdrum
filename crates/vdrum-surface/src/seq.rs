// SPDX-License-Identifier: GPL-3.0-only
//! The sequencer's pattern data as the plugin keeps it (the copy the controller edits and the plugin
//! saves), the edits the surface asks for, and the Euclid generator.
//!
//! The engine (the ported LXR-02 sequencer) plays a copy of this data. The plugin writes every edit
//! to both; see `plugin/README.md`.

use serde::{Deserialize, Serialize};

pub const NUM_PATTERNS: usize = 64;
pub const NUM_TRACKS: usize = 7;
/// Steps per track: 4 bars of 16.
pub const NUM_STEPS: usize = 64;
/// Parameter number of an unused lock slot (the firmware's `NO_AUTOMATION`).
pub const NO_LOCK: u8 = 255;
/// The firmware's default step note (`SEQ_DEFAULT_NOTE`).
pub const DEFAULT_NOTE: u8 = 60;
/// Micro-shift value meaning "no shift" (`getShiftValue`: 0..6 = -7..-1, 7 = 0, 8..14 = +1..+7).
pub const NO_SHIFT: u8 = 7;
/// Highest flam / ratchet index (`flamIdxToPattern`: off, x2, x3, x4, x6, x8).
pub const MAX_FLAM: u8 = 5;
pub const FLAM_NAMES: [&str; 6] = ["off", "x2", "x3", "x4", "x6", "x8"];

/// One step of one track.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StepData {
    pub on: bool,
    /// 1..=127.
    pub velocity: u8,
    /// Playback probability in percent, 0..=100.
    pub prob: u8,
    /// 0..=127.
    pub note: u8,
    /// 0..=[`MAX_FLAM`].
    pub flam: u8,
    /// 0..=14, [`NO_SHIFT`] = none.
    pub shift: u8,
    /// Parameter locks: (engine parameter index or [`NO_LOCK`], value 0..=127 as a MIDI CC value).
    /// Slot 0 is the older one.
    pub locks: [(u8, u8); 2],
}

impl Default for StepData {
    /// The firmware's cleared step (`seq_resetNote`), off.
    fn default() -> Self {
        StepData {
            on: false,
            velocity: 127,
            prob: 100,
            note: DEFAULT_NOTE,
            flam: 0,
            shift: NO_SHIFT,
            locks: [(NO_LOCK, 0); 2],
        }
    }
}

impl StepData {
    /// The step's fields packed as saved: on, velocity, prob, note, flam, shift, lock 1, lock 2.
    fn pack(&self) -> [u8; 10] {
        [
            self.on as u8,
            self.velocity,
            self.prob,
            self.note,
            self.flam,
            self.shift,
            self.locks[0].0,
            self.locks[0].1,
            self.locks[1].0,
            self.locks[1].1,
        ]
    }

    fn unpack(b: [u8; 10]) -> StepData {
        StepData {
            on: b[0] != 0,
            velocity: b[1].clamp(1, 127),
            prob: b[2].min(100),
            note: b[3].min(127),
            flam: b[4].min(MAX_FLAM),
            shift: b[5].min(14),
            locks: [(b[6], b[7].min(127)), (b[8], b[9].min(127))],
        }
    }
}

/// One track of one pattern.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TrackData {
    pub steps: [StepData; NUM_STEPS],
    /// 1..=64.
    pub length: u8,
    /// Euclid pulses (0 = steps set by hand) and rotation.
    pub fills: u8,
    pub rotate: u8,
}

impl Default for TrackData {
    fn default() -> Self {
        TrackData { steps: [StepData::default(); NUM_STEPS], length: 16, fills: 0, rotate: 0 }
    }
}

impl TrackData {
    /// The on/off state of all steps, bit `n` = step `n`.
    pub fn bits(&self) -> u64 {
        self.steps.iter().enumerate().fold(0, |b, (i, s)| b | ((s.on as u64) << i))
    }

    fn set_bits(&mut self, bits: u64) {
        for (i, s) in self.steps.iter_mut().enumerate() {
            s.on = bits >> i & 1 != 0;
        }
    }

    fn is_default(&self) -> bool {
        *self == TrackData::default()
    }
}

/// One pattern.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PatternData {
    pub tracks: [TrackData; NUM_TRACKS],
    /// 0..=127 (`PAR_SHUFFLE`).
    pub shuffle: u8,
}

impl PatternData {
    pub fn is_empty(&self) -> bool {
        self.tracks.iter().all(|t| t.steps.iter().all(|s| !s.on))
    }
}

/// All patterns, and the playback density of every track.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SeqData {
    pub patterns: Vec<PatternData>,
    /// Playback density per track, 0..=100 (100 = every step plays with its own probability).
    pub density: [u8; NUM_TRACKS],
    /// Density of all tracks, 0..=100, multiplied with the track's.
    pub density_all: u8,
}

impl Default for SeqData {
    fn default() -> Self {
        SeqData { patterns: vec![PatternData::default(); NUM_PATTERNS], density: [100; NUM_TRACKS], density_all: 100 }
    }
}

/// A change the surface asks for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SeqEdit {
    /// Replace one step.
    Step { pat: u8, track: u8, step: u8, data: StepData },
    /// Set Euclid pulses and rotation; rewrites the track's on/off states (pulses 0 clears them).
    Euclid { pat: u8, track: u8, fills: u8, rotate: u8 },
    /// Track length 1..=64. A track with Euclid pulses is generated again for the new length.
    Length { pat: u8, track: u8, length: u8 },
    Shuffle { pat: u8, value: u8 },
    /// Playback density of a track (0..7) or of all tracks (7), 0..=100.
    Density { track: u8, value: u8 },
    /// Switch to a pattern at the end of the bar (at once when stopped).
    NextPattern(u8),
    /// Start or stop the sequencer (internal clock).
    Run(bool),
    /// Silence the sequencer or let it play again (host clock: the host transport keeps running).
    Silence(bool),
}

impl SeqData {
    pub fn track(&self, pat: u8, track: u8) -> &TrackData {
        &self.patterns[(pat as usize).min(NUM_PATTERNS - 1)].tracks[(track as usize).min(NUM_TRACKS - 1)]
    }

    pub fn step(&self, pat: u8, track: u8, step: u8) -> StepData {
        self.track(pat, track).steps[(step as usize).min(NUM_STEPS - 1)]
    }

    fn track_mut(&mut self, pat: u8, track: u8) -> &mut TrackData {
        &mut self.patterns[(pat as usize).min(NUM_PATTERNS - 1)].tracks[(track as usize).min(NUM_TRACKS - 1)]
    }

    /// The probability the engine plays a step with: its own, scaled by the densities.
    pub fn effective_prob(&self, track: u8, prob: u8) -> u8 {
        let d = self.density[(track as usize).min(NUM_TRACKS - 1)] as u32 * self.density_all as u32;
        ((prob.min(100) as u32 * d + 5000) / 10000) as u8
    }

    /// Apply an edit (transport edits change nothing here). Values are limited to their ranges.
    pub fn apply(&mut self, e: &SeqEdit) {
        match *e {
            SeqEdit::Step { pat, track, step, data } => {
                if (step as usize) < NUM_STEPS {
                    self.track_mut(pat, track).steps[step as usize] = StepData::unpack(data.pack());
                }
            }
            SeqEdit::Euclid { pat, track, fills, rotate } => {
                let t = self.track_mut(pat, track);
                t.fills = fills.min(t.length);
                t.rotate = rotate % t.length.max(1);
                let bits = euclid(t.length, t.fills, t.rotate);
                t.set_bits(bits);
            }
            SeqEdit::Length { pat, track, length } => {
                let t = self.track_mut(pat, track);
                t.length = length.clamp(1, NUM_STEPS as u8);
                if t.fills > 0 {
                    t.fills = t.fills.min(t.length);
                    t.rotate %= t.length;
                    let bits = euclid(t.length, t.fills, t.rotate);
                    t.set_bits(bits);
                }
            }
            SeqEdit::Shuffle { pat, value } => {
                self.patterns[(pat as usize).min(NUM_PATTERNS - 1)].shuffle = value.min(127);
            }
            SeqEdit::Density { track, value } => {
                let v = value.min(100);
                match self.density.get_mut(track as usize) {
                    Some(d) => *d = v,
                    None => self.density_all = v,
                }
            }
            SeqEdit::NextPattern(_) | SeqEdit::Run(_) | SeqEdit::Silence(_) => {}
        }
    }

    /// The data as saved with the plugin state (JSON, only what differs from the defaults).
    pub fn to_text(&self) -> String {
        let patterns = self
            .patterns
            .iter()
            .enumerate()
            .filter(|(_, p)| **p != PatternData::default())
            .map(|(i, p)| SavedPattern {
                p: i as u8,
                shuffle: p.shuffle,
                tracks: p
                    .tracks
                    .iter()
                    .enumerate()
                    .filter(|(_, t)| !t.is_default())
                    .map(|(ti, t)| SavedTrack {
                        t: ti as u8,
                        length: t.length,
                        fills: t.fills,
                        rotate: t.rotate,
                        steps: t
                            .steps
                            .iter()
                            .enumerate()
                            .filter(|(_, s)| **s != StepData::default())
                            .map(|(si, s)| SavedStep { s: si as u8, d: s.pack() })
                            .collect(),
                    })
                    .collect(),
            })
            .collect();
        let saved = Saved { version: 1, density: self.density, density_all: self.density_all, patterns };
        serde_json::to_string(&saved).unwrap_or_default()
    }

    /// Inverse of [`SeqData::to_text`]; `None` if the text is not valid. Out-of-range values are
    /// limited.
    pub fn from_text(text: &str) -> Option<SeqData> {
        let saved: Saved = serde_json::from_str(text).ok()?;
        let mut d = SeqData {
            density: saved.density.map(|v| v.min(100)),
            density_all: saved.density_all.min(100),
            ..SeqData::default()
        };
        for p in saved.patterns {
            let Some(pd) = d.patterns.get_mut(p.p as usize) else { continue };
            pd.shuffle = p.shuffle.min(127);
            for t in p.tracks {
                let Some(td) = pd.tracks.get_mut(t.t as usize) else { continue };
                td.length = t.length.clamp(1, NUM_STEPS as u8);
                td.fills = t.fills.min(td.length);
                td.rotate = t.rotate % td.length;
                for s in t.steps {
                    if let Some(sd) = td.steps.get_mut(s.s as usize) {
                        *sd = StepData::unpack(s.d);
                    }
                }
            }
        }
        Some(d)
    }
}

#[derive(Serialize, Deserialize)]
struct Saved {
    version: u32,
    density: [u8; NUM_TRACKS],
    density_all: u8,
    patterns: Vec<SavedPattern>,
}

#[derive(Serialize, Deserialize)]
struct SavedPattern {
    p: u8,
    shuffle: u8,
    tracks: Vec<SavedTrack>,
}

#[derive(Serialize, Deserialize)]
struct SavedTrack {
    t: u8,
    length: u8,
    fills: u8,
    rotate: u8,
    steps: Vec<SavedStep>,
}

#[derive(Serialize, Deserialize)]
struct SavedStep {
    s: u8,
    d: [u8; 10],
}

/// Euclidean rhythm: `fills` pulses spread evenly over `length` steps (step 0 is a pulse when
/// `fills > 0`), rotated later by `rotate` steps. Bit `n` = step `n`.
///
/// This is the plugin's generator, not the engine's `euklid_generate`: the engine's recursion only
/// covers lengths up to 16 (longer lengths overflow its 16 bit groups; see
/// `crates/lxr-core/src/sequencer/euklid_generator.rs`), and it would have to run on the audio
/// thread. For lengths up to 16 the two can differ in where the gaps fall; both give evenly spread
/// pulses.
pub fn euclid(length: u8, fills: u8, rotate: u8) -> u64 {
    let n = length.clamp(1, NUM_STEPS as u8) as u32;
    let k = fills.min(n as u8) as u32;
    if k == 0 {
        return 0;
    }
    let mut bits = 0u64;
    for i in 0..n {
        if (i * k) % n < k {
            bits |= 1 << ((i + rotate as u32) % n);
        }
    }
    bits
}

/// The sequencer's playback state, as the plugin reports it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SeqStatus {
    pub running: bool,
    /// Pattern playing.
    pub pattern: u8,
    /// Pattern queued for the end of the bar (equals `pattern` when none is queued).
    pub pending: u8,
    /// Step each track is on (-1 before the first step).
    pub step: [i8; NUM_TRACKS],
    /// More than half of the current step has passed (recorded notes go to the next step).
    pub late: bool,
    /// Silenced with Play under the host clock: the sequencer follows the host but plays no steps.
    pub silent: bool,
}

impl Default for SeqStatus {
    fn default() -> Self {
        SeqStatus { running: false, pattern: 0, pending: 0, step: [-1; NUM_TRACKS], late: false, silent: false }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn euclid_spreads_pulses() {
        assert_eq!(euclid(8, 3, 0), 0b0100_1001);
        assert_eq!(euclid(16, 4, 0), 0x1111);
        assert_eq!(euclid(16, 4, 1), 0x2222);
        assert_eq!(euclid(16, 0, 0), 0);
        assert_eq!(euclid(16, 16, 3), 0xffff);
        assert_eq!(euclid(64, 64, 0), u64::MAX);
        for n in 1..=64u8 {
            for k in 0..=n {
                assert_eq!(euclid(n, k, 0).count_ones(), k as u32, "{k} of {n}");
            }
        }
    }

    #[test]
    fn text_round_trip() {
        let mut d = SeqData::default();
        assert_eq!(SeqData::from_text(&d.to_text()), Some(d.clone()));
        d.apply(&SeqEdit::Euclid { pat: 3, track: 2, fills: 5, rotate: 2 });
        let mut s = d.step(3, 2, 0);
        s.locks[1] = (12, 99);
        s.velocity = 40;
        d.apply(&SeqEdit::Step { pat: 3, track: 2, step: 0, data: s });
        d.apply(&SeqEdit::Length { pat: 9, track: 6, length: 64 });
        d.apply(&SeqEdit::Shuffle { pat: 9, value: 30 });
        d.apply(&SeqEdit::Density { track: 4, value: 50 });
        d.apply(&SeqEdit::Density { track: 7, value: 80 });
        let text = d.to_text();
        assert!(text.len() < 1000, "{}", text.len());
        assert_eq!(SeqData::from_text(&text), Some(d));
        assert_eq!(SeqData::from_text("nonsense"), None);
    }

    #[test]
    fn length_regenerates_euclid_tracks_only() {
        let mut d = SeqData::default();
        d.apply(&SeqEdit::Euclid { pat: 0, track: 0, fills: 4, rotate: 0 });
        d.apply(&SeqEdit::Length { pat: 0, track: 0, length: 8 });
        assert_eq!(d.track(0, 0).bits(), euclid(8, 4, 0));
        let on = StepData { on: true, ..StepData::default() };
        d.apply(&SeqEdit::Step { pat: 0, track: 1, step: 5, data: on });
        d.apply(&SeqEdit::Length { pat: 0, track: 1, length: 4 });
        assert_eq!(d.track(0, 1).bits(), 1 << 5);
    }

    #[test]
    fn density_scales_probability() {
        let mut d = SeqData::default();
        assert_eq!(d.effective_prob(0, 100), 100);
        d.apply(&SeqEdit::Density { track: 0, value: 50 });
        d.apply(&SeqEdit::Density { track: 7, value: 50 });
        assert_eq!(d.effective_prob(0, 100), 25);
        assert_eq!(d.effective_prob(1, 80), 40);
    }
}
