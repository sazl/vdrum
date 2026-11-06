// SPDX-License-Identifier: GPL-3.0-only
//! A stand-in for the plugin: parameter values, voice faders, hit counters, switches, mutes, the
//! sequencer's data, and a simple sequencer clock. It has no sound engine.

use std::cell::RefCell;
use std::collections::HashMap;
use vdrum_surface::seq::NUM_TRACKS;
use vdrum_surface::{Action, Model, SeqData, SeqEdit, SeqStatus, Switch};

/// Kit names offered by Shift + Page up/down (the plugin browses the kit folder).
pub const KITS: [&str; 4] = ["Init", "Kit A", "Kit B", "Kit C"];

pub struct SimModel {
    pub params: RefCell<HashMap<u16, u16>>,
    pub faders: RefCell<[i32; 7]>,
    pub hits: RefCell<[u32; 7]>,
    pub switches: RefCell<[bool; 2]>,
    pub seq: RefCell<SeqData>,
    pub status: RefCell<SeqStatus>,
    pub mutes: RefCell<u8>,
    pub kit: RefCell<String>,
    pub tempo: f64,
    /// Time the internal clock started (ms) and the last 16th it played.
    run_at: RefCell<Option<(u64, i64)>>,
    now: RefCell<u64>,
    browse: RefCell<usize>,
}

impl Default for SimModel {
    fn default() -> Self {
        SimModel::new()
    }
}

fn defaults() -> HashMap<u16, u16> {
    lxr_params::PARAMS
        .iter()
        .chain(lxr_params::GLOBAL_PARAMS.iter())
        .chain(lxr_params::oh_params().iter())
        .map(|p| (p.index, p.default))
        .collect()
}

impl SimModel {
    pub fn new() -> SimModel {
        SimModel {
            params: RefCell::new(defaults()),
            faders: RefCell::new([99; 7]),
            hits: RefCell::new([0; 7]),
            switches: RefCell::new([false; 2]),
            seq: RefCell::new(SeqData::default()),
            status: RefCell::new(SeqStatus::default()),
            mutes: RefCell::new(0),
            kit: RefCell::new("Init".into()),
            tempo: 120.0,
            run_at: RefCell::new(None),
            now: RefCell::new(0),
            browse: RefCell::new(0),
        }
    }

    /// Apply the surface's actions. Returns a popup to show for the actions the plugin answers
    /// (kit browsing), as the plugin does with `Surface::notify`.
    pub fn apply(&self, actions: &[Action]) -> Option<[String; 3]> {
        let mut notify = None;
        for a in actions {
            match a {
                Action::SetParam { idx, value } => {
                    self.params.borrow_mut().insert(*idx, *value);
                }
                Action::SetVoiceFader { track, step } => self.faders.borrow_mut()[*track as usize % 7] = *step,
                Action::Trigger { track, .. } => self.hit(*track),
                Action::SetSwitch(sw, on) => self.switches.borrow_mut()[(*sw == Switch::HwClock) as usize] = *on,
                Action::Seq(e) => {
                    self.seq.borrow_mut().apply(e);
                    let mut st = self.status.borrow_mut();
                    match *e {
                        SeqEdit::NextPattern(p) => {
                            st.pending = p;
                            if !st.running {
                                st.pattern = p;
                            }
                        }
                        SeqEdit::Run(r) => {
                            st.running = r;
                            if r {
                                *self.run_at.borrow_mut() = Some((*self.now.borrow(), -1));
                            } else {
                                *self.run_at.borrow_mut() = None;
                                st.step = [-1; NUM_TRACKS];
                            }
                        }
                        SeqEdit::Silence(on) => st.silent = on,
                        _ => {}
                    }
                }
                Action::SetMutes(m) => *self.mutes.borrow_mut() = *m,
                Action::KitBrowse(d) => {
                    let mut b = self.browse.borrow_mut();
                    *b = (*b as i32 + *d as i32).rem_euclid(KITS.len() as i32) as usize;
                    notify = Some(["Kit".to_string(), KITS[*b].to_string(), "Play loads".to_string()]);
                }
                Action::KitLoadBrowsed => {
                    *self.kit.borrow_mut() = KITS[*self.browse.borrow()].to_string();
                    notify = Some(["Kit".to_string(), self.kit.borrow().clone(), "loaded".to_string()]);
                }
                Action::InitKit => {
                    *self.params.borrow_mut() = defaults();
                    *self.kit.borrow_mut() = "Init".into();
                }
                _ => {}
            }
        }
        notify
    }

    fn hit(&self, track: u8) {
        if *self.mutes.borrow() >> track & 1 == 0 {
            self.hits.borrow_mut()[track as usize % 7] += 1;
        }
    }

    /// Advance the internal sequencer clock to `now` (16ths at the tempo): steps move, steps that
    /// are on count as hits, a queued pattern starts at the end of the bar.
    pub fn tick(&self, now: u64) {
        *self.now.borrow_mut() = now;
        let Some((start, last)) = *self.run_at.borrow() else { return };
        let sixteenth = 60_000.0 / self.tempo / 4.0;
        let n = ((now.saturating_sub(start)) as f64 / sixteenth) as i64;
        if n == last {
            return;
        }
        for k in (last + 1)..=n {
            let mut st = self.status.borrow_mut();
            if k > 0 && k % 16 == 0 && st.pending != st.pattern {
                st.pattern = st.pending;
            }
            let pat = st.pattern;
            let mut hits = Vec::new();
            {
                let d = self.seq.borrow();
                for t in 0..NUM_TRACKS as u8 {
                    let len = d.track(pat, t).length.max(1) as i64;
                    let step = (k % len) as u8;
                    st.step[t as usize] = step as i8;
                    if d.step(pat, t, step).on && !st.silent {
                        hits.push(t);
                    }
                }
            }
            drop(st);
            for t in hits {
                self.hit(t);
            }
        }
        *self.run_at.borrow_mut() = Some((start, n));
    }
}

impl Model for SimModel {
    fn param(&self, idx: u16) -> u16 {
        self.params.borrow().get(&idx).copied().unwrap_or(0)
    }
    fn voice_fader(&self, t: u8) -> i32 {
        self.faders.borrow()[t as usize % 7]
    }
    fn kit_name(&self) -> String {
        self.kit.borrow().clone()
    }
    fn hits(&self, t: u8) -> u32 {
        self.hits.borrow()[t as usize % 7]
    }
    fn switch(&self, sw: Switch) -> bool {
        self.switches.borrow()[(sw == Switch::HwClock) as usize]
    }
    fn seq(&self) -> Box<dyn std::ops::Deref<Target = SeqData> + '_> {
        Box::new(self.seq.borrow())
    }
    fn seq_status(&self) -> SeqStatus {
        *self.status.borrow()
    }
    fn lfo(&self, t: u8) -> f32 {
        // A 0.5 Hz sine per track, phase-shifted by track (the plugin reports the engine's LFOs).
        let ph = (*self.now.borrow() as f32 / 2000.0 + t as f32 / 7.0) * std::f32::consts::TAU;
        0.5 + 0.5 * ph.sin()
    }
    fn mutes(&self) -> u8 {
        *self.mutes.borrow()
    }
    fn tempo(&self) -> f64 {
        self.tempo
    }
}
