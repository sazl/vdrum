// SPDX-License-Identifier: GPL-3.0-only
//! Modes 9 to 12 beyond plain parameters: momentary effects, mutes, LFO scaling, snapshots,
//! per-voice morph and macros.
//!
//! Changes made here (except the LFO offset) bypass the undo history, because they are
//! performances that restore themselves or are recalled as a whole.

use super::*;
use crate::layout::voice_param;
use lxr_params::{oh_params, ParamKind, PARAMS};
use serde::{Deserialize, Serialize};

/// Number of snapshot slots and macros.
pub const SLOTS: usize = 8;
/// Momentary effects: stutter 1/16, stutter 1/32, filter sweep, reso up, swell, long decay, lo-fi,
/// FX throw.
pub const STUTTER_16: u8 = 0;
pub const STUTTER_32: u8 = 1;
pub const FILTER_SWEEP: u8 = 2;
/// Tracks hit this long before a stutter starts are repeated, in milliseconds.
pub const STUTTER_RECENT_MS: u64 = 1000;

/// A momentary effect while its button is held.
#[derive(Clone, Debug)]
pub(super) struct ActiveMoment {
    pub k: u8,
    pub since: u64,
    /// (parameter, value before the effect, value at full effect).
    pub params: Vec<(u16, u16, u16)>,
    /// Stutter: tracks to repeat and the time of the next repeat.
    pub tracks: u8,
    pub next: u64,
}

/// A non-destructive multiplier of one parameter kind of all voices (LFO rate, LFO depth,
/// velocity amount): the values before scaling are kept while the scale is not 100 %.
#[derive(Clone, Debug, Default)]
pub(super) struct Scaler {
    pub pct: i32,
    /// (parameter, value at 100 %, value last written).
    pub base: Vec<(u16, u16, u16)>,
}

/// A macro: parameters with the range learned for them, a depth and the fader position.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Macro {
    /// (parameter, value at fader bottom, value at fader top).
    pub assign: Vec<(u16, u16, u16)>,
    /// Range scale in percent, 0..=200.
    pub depth: u16,
}

/// Per-voice morph toward a snapshot.
#[derive(Clone, Debug, Default)]
pub(super) struct Morph {
    pub amount: u8,
    pub target: u8,
    /// The voice's values before the morph started.
    pub base: Option<Vec<(u16, u16)>>,
}

/// What the plugin saves of the controller besides the settings.
#[derive(Default, Serialize, Deserialize)]
struct SavedState {
    snapshots: Vec<Option<Vec<(u16, u16)>>>,
    macros: Vec<Macro>,
}

/// The kit parameters (sound parameters of the seven voices, including the FX bus).
pub fn kit_params() -> Vec<u16> {
    PARAMS.iter().filter(|p| p.kind == ParamKind::Sound && p.min < p.max).chain(oh_params().iter()).map(|p| p.index).collect()
}

/// The kit parameters of voice `t` (0..7).
pub fn voice_params(t: u8) -> Vec<u16> {
    PARAMS
        .iter()
        .filter(|p| p.kind == ParamKind::Sound && p.min < p.max)
        .chain(oh_params().iter())
        .filter(|p| p.voice.number() == Some(t))
        .map(|p| p.index)
        .collect()
}

fn range(idx: u16) -> (u16, u16) {
    lxr_params::info(idx).map_or((0, 127), |i| (i.min, i.max))
}

/// Interpolates a parameter: numbers linearly, lists (16 entries or fewer, modulation targets)
/// switch at the middle.
fn lerp(idx: u16, a: u16, b: u16, num: u32, den: u32) -> u16 {
    let list = lxr_params::info(idx).is_some_and(|i| i.steps() <= 16 || i.is_mod_target());
    if list {
        return if num * 2 >= den { b } else { a };
    }
    let (a, b) = (a as i64, b as i64);
    (a + (b - a) * num as i64 / den.max(1) as i64) as u16
}

impl Surface {
    /// Push a parameter change that does not go into the undo history.
    pub(super) fn set_raw(&mut self, idx: u16, value: u16, model: &dyn Model, out: &mut Vec<Action>) {
        if model.param(idx) != value {
            out.push(Action::SetParam { idx, value });
        }
    }

    // ---- mutes and Perform triggers

    pub(super) fn toggle_mute(&mut self, cell: &Cell, track: Option<u8>, model: &dyn Model, out: &mut Vec<Action>) {
        let cur = model.mutes() & 0x7f;
        let new = match track {
            Some(t) => cur ^ (1 << t),
            None if cur == 0x7f => 0,
            None => 0x7f,
        };
        out.push(Action::SetMutes(new));
        let text = match track {
            Some(t) => if new >> t & 1 != 0 { "muted" } else { "on" }.to_string(),
            None => if new == 0 { "all on" } else { "all muted" }.to_string(),
        };
        self.show(cell, text);
    }

    // ---- momentary effects

    /// The parameters a momentary effect changes: (parameter, value at full effect).
    fn moment_targets(&self, k: u8, model: &dyn Model) -> Vec<(u16, u16)> {
        let d = self.moment_depth as u32;
        let up = |idx: u16| {
            let (_, max) = range(idx);
            let cur = model.param(idx) as u32;
            (idx, (cur + (max as u32).saturating_sub(cur) * d / 127) as u16)
        };
        let voices = |kind: &str| (0..7u8).map(|t| voice_param(t, kind)).filter(|i| lxr_params::info(*i).is_some()).collect::<Vec<_>>();
        match k {
            FILTER_SWEEP => voices("cutoff")
                .into_iter()
                .map(|i| {
                    let (min, _) = range(i);
                    let cur = model.param(i) as u32;
                    (i, (cur - (cur - min as u32) * d / 127) as u16)
                })
                .collect(),
            3 => voices("reso").into_iter().map(up).collect(),
            4 => voices("attack").into_iter().map(up).collect(),
            5 => voices("decay").into_iter().map(up).collect(),
            6 => {
                let i = par::VOICE_DECIMATION_ALL;
                let cur = model.param(i) as u32;
                vec![(i, (cur * (127 - d) / 127).max(4) as u16)]
            }
            7 => {
                let mut v: Vec<(u16, u16)> = voices("out").into_iter().map(|i| (i, AUDIO_OUT_FX)).collect();
                v.push(up(par::FX_DRY_WET));
                v
            }
            _ => Vec::new(),
        }
    }

    pub(super) fn moment_press(&mut self, cell: &Cell, k: u8, model: &dyn Model, out: &mut Vec<Action>) {
        if self.moments.iter().any(|m| m.k == k) {
            return;
        }
        let now = self.now;
        let mut m = ActiveMoment { k, since: now, params: Vec::new(), tracks: 0, next: now };
        if k == STUTTER_16 || k == STUTTER_32 {
            for t in 0..7u8 {
                if self.hit_at[t as usize].is_some_and(|at| now.saturating_sub(at) <= STUTTER_RECENT_MS) {
                    m.tracks |= 1 << t;
                }
            }
            if m.tracks == 0 {
                self.show(cell, "no recent hits".to_string());
                return;
            }
        } else {
            for (idx, full) in self.moment_targets(k, model) {
                m.params.push((idx, model.param(idx), full));
            }
            if k != FILTER_SWEEP {
                for &(idx, _, full) in &m.params {
                    self.set_raw(idx, full, model, out);
                }
            }
        }
        self.moments.push(m);
        self.show(cell, "on".to_string());
        // stutters start at once
        self.moment_tick(model, out);
    }

    pub(super) fn moment_release(&mut self, cell: &Cell, k: u8, model: &dyn Model, out: &mut Vec<Action>) {
        let Some(pos) = self.moments.iter().position(|m| m.k == k) else { return };
        let m = self.moments.remove(pos);
        for (idx, before, _) in m.params {
            self.set_raw(idx, before, model, out);
        }
        self.show(cell, "off".to_string());
    }

    /// Stutter repeats and the filter sweep's movement.
    pub(super) fn moment_tick(&mut self, model: &dyn Model, out: &mut Vec<Action>) {
        let now = self.now;
        let beat_ms = 60000.0 / model.tempo().clamp(20.0, 999.0);
        let sweep_ms = 50 + (127 - self.moment_rate.min(127) as u64) * 15;
        let mut sets = Vec::new();
        for m in &mut self.moments {
            match m.k {
                STUTTER_16 | STUTTER_32 => {
                    let interval = beat_ms / if m.k == STUTTER_16 { 4.0 } else { 8.0 };
                    while now >= m.next {
                        for t in 0..7u8 {
                            if m.tracks >> t & 1 != 0 {
                                out.push(Action::Trigger { track: t, velocity: 100 });
                            }
                        }
                        m.next += interval.max(1.0) as u64;
                    }
                }
                FILTER_SWEEP => {
                    let e = now.saturating_sub(m.since).min(sweep_ms) as u32;
                    for &(idx, before, full) in &m.params {
                        sets.push((idx, lerp(idx, before, full, e, sweep_ms as u32)));
                    }
                }
                _ => {}
            }
        }
        for (idx, v) in sets {
            self.set_raw(idx, v, model, out);
        }
    }

    // ---- LFO + Mod, column 8

    /// Rate scale (0), depth scale (1), velocity scale (2): percent of the values before scaling.
    pub(super) fn scale(&mut self, cell: &Cell, which: usize, pct: i32, model: &dyn Model, out: &mut Vec<Action>) {
        let kind = ["lfo_rate", "lfo_amt", "velamt"][which];
        let (lo, hi) = if which == 0 { (10, 400) } else { (0, 200) };
        let s = &mut self.scalers[which];
        let old = s.pct;
        s.pct = pct.clamp(lo, hi);
        if s.pct == old {
            let text = format!("{} %", s.pct);
            self.show(cell, text);
            return;
        }
        if s.base.is_empty() {
            s.base = (0..7u8)
                .map(|t| voice_param(t, kind))
                .filter(|i| lxr_params::info(*i).is_some())
                .map(|i| (i, model.param(i), model.param(i)))
                .collect();
        }
        let pct = s.pct as u32;
        let mut sets = Vec::new();
        for b in &mut s.base {
            // a value changed elsewhere since it was scaled becomes the new value at 100 %
            let cur = model.param(b.0);
            if cur != b.2 {
                b.1 = cur;
            }
            let (min, max) = range(b.0);
            let v = (b.1 as u32 * pct / 100).clamp(min as u32, max as u32) as u16;
            b.2 = v;
            sets.push((b.0, v));
        }
        if s.pct == 100 {
            s.base.clear();
        }
        let text = format!("{} %", s.pct);
        for (i, v) in sets {
            self.set_raw(i, v, model, out);
        }
        self.show(cell, text);
    }

    pub(super) fn lfo_offset_all(&mut self, cell: &Cell, delta: i32, model: &dyn Model, out: &mut Vec<Action>) {
        let mut first = None;
        for t in 0..7u8 {
            let i = voice_param(t, "lfo_ofs");
            let Some(info) = lxr_params::info(i) else { continue };
            let v = (model.param(i) as i32 + delta).clamp(info.min as i32, info.max as i32) as u16;
            first.get_or_insert(v);
            self.set(i, v, model, out);
        }
        self.show(cell, first.map(|v| v.to_string()).unwrap_or_default());
    }

    /// LFOs that run free get clock sync 1/1 (the sequencer clock follows the host tempo in the
    /// Host clock setting); pressed again, the previous settings return.
    pub(super) fn lfo_sync_host(&mut self, cell: &Cell, model: &dyn Model, out: &mut Vec<Action>) {
        match self.lfo_sync_saved.take() {
            Some(saved) => {
                for (i, v) in saved {
                    self.set_raw(i, v, model, out);
                }
                self.show(cell, "off".to_string());
            }
            None => {
                let mut saved = Vec::new();
                for t in 0..7u8 {
                    let i = voice_param(t, "lfo_sync");
                    if lxr_params::info(i).is_none() {
                        continue;
                    }
                    saved.push((i, model.param(i)));
                    if model.param(i) == 0 {
                        self.set_raw(i, 3, model, out);
                    }
                }
                self.lfo_sync_saved = Some(saved);
                self.show(cell, "free LFOs at 1/1".to_string());
            }
        }
    }

    // ---- snapshots, morph, macros

    pub(super) fn snapshot_recall(&mut self, cell: &Cell, n: u8, model: &dyn Model, out: &mut Vec<Action>) {
        match self.snapshots[n as usize % SLOTS].clone() {
            Some(values) => {
                for (i, v) in values {
                    self.set_raw(i, v, model, out);
                }
                for m in &mut self.morph {
                    *m = Morph { target: m.target, ..Morph::default() };
                }
                self.show(cell, "recalled".to_string());
            }
            None => self.show(cell, "empty (hold 1 s: store)".to_string()),
        }
    }

    pub(super) fn snapshot_store(&mut self, cell: &Cell, n: u8, model: &dyn Model) {
        self.snapshots[n as usize % SLOTS] = Some(kit_params().into_iter().map(|i| (i, model.param(i))).collect());
        self.state_changed = true;
        self.show(cell, "stored".to_string());
    }

    /// Morph amount (`amount`) or target snapshot (`target`) of voice `t`.
    pub(super) fn morph_set(&mut self, t: u8, amount: Option<u8>, target: Option<u8>, model: &dyn Model, out: &mut Vec<Action>) -> String {
        let m = &mut self.morph[t as usize];
        if let Some(a) = amount {
            m.amount = a.min(127);
        }
        if let Some(g) = target {
            m.target = g % SLOTS as u8;
        }
        let (amount, target) = (m.amount, m.target);
        let Some(snap) = self.snapshots[target as usize].clone() else {
            return format!("snapshot {} empty", target + 1);
        };
        let m = &mut self.morph[t as usize];
        if m.base.is_none() {
            if amount == 0 {
                return format!("0 > S{}", target + 1);
            }
            m.base = Some(voice_params(t).into_iter().map(|i| (i, model.param(i))).collect());
        }
        let base = m.base.clone().unwrap_or_default();
        if amount == 0 {
            m.base = None;
        }
        let audio_out = voice_param(t, "out");
        for (i, from) in base {
            let to = snap.iter().find(|(j, _)| *j == i).map_or(from, |(_, v)| *v);
            // like the firmware's morph, the audio routing stays
            let v = if i == audio_out { from } else { lerp(i, from, to, amount as u32, 127) };
            self.set_raw(i, v, model, out);
        }
        format!("{} > S{}", amount, target + 1)
    }

    pub(super) fn macro_move(&mut self, n: u8, pos: u8, model: &dyn Model, out: &mut Vec<Action>) -> String {
        let m = self.macros[n as usize % SLOTS].clone();
        if m.assign.is_empty() {
            return "not learned".to_string();
        }
        for (i, from, to) in m.assign {
            let (min, max) = range(i);
            let span = (to as i64 - from as i64) * m.depth as i64 / 100;
            let v = (from as i64 + span * pos as i64 / 127).clamp(min as i64, max as i64) as u16;
            self.set_raw(i, v, model, out);
        }
        format!("{}", pos as u32 * 100 / 127)
    }

    pub(super) fn learn_press(&mut self, cell: &Cell, n: u8) {
        match self.learning.take() {
            Some((l, rec)) if l == n => {
                let count = rec.len();
                self.macros[n as usize % SLOTS].assign = rec;
                if self.macros[n as usize % SLOTS].depth == 0 {
                    self.macros[n as usize % SLOTS].depth = 100;
                }
                self.state_changed = true;
                self.show(cell, format!("{count} parameters"));
            }
            _ => {
                self.learning = Some((n, Vec::new()));
                self.show(cell, "move controls, press again".to_string());
            }
        }
    }

    /// Record a controller change for the macro being learned.
    pub(super) fn learn_record(&mut self, idx: u16, old: u16, new: u16) {
        if let Some((_, rec)) = &mut self.learning {
            match rec.iter_mut().find(|r| r.0 == idx) {
                Some(r) => r.2 = new,
                None => rec.push((idx, old, new)),
            }
        }
    }

    pub(super) fn macro_depth(&mut self, cell: &Cell, n: u8, delta: i32) {
        let m = &mut self.macros[n as usize % SLOTS];
        m.depth = (m.depth as i32 + delta * 5).clamp(0, 200) as u16;
        let text = format!("{} %", m.depth);
        self.state_changed = true;
        self.show(cell, text);
    }

    /// Snapshots and macros as saved with the plugin state (JSON).
    pub fn state_text(&self) -> String {
        let s = SavedState { snapshots: self.snapshots.to_vec(), macros: self.macros.to_vec() };
        serde_json::to_string(&s).unwrap_or_default()
    }

    /// Restore [`Surface::state_text`]; invalid text is ignored.
    pub fn load_state(&mut self, text: &str) {
        let Ok(s) = serde_json::from_str::<SavedState>(text) else { return };
        for (slot, v) in self.snapshots.iter_mut().zip(s.snapshots) {
            *slot = v;
        }
        for (slot, v) in self.macros.iter_mut().zip(s.macros) {
            *slot = Macro { depth: v.depth.min(200), ..v };
        }
    }

    /// Whether snapshots or macros changed since the last call.
    pub fn take_state_changed(&mut self) -> bool {
        std::mem::take(&mut self.state_changed)
    }
}
