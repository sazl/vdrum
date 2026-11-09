// SPDX-License-Identifier: GPL-3.0-only
//! The module's state as [`vdrum_surface::Surface`] reads it, and carrying out the surface's
//! actions that only need the shared state (parameters, faders, triggers, sequencer edits).

use crate::shared::{Command, Shared, N};
use lxr_params::kit::Kit;
use std::sync::atomic::Ordering;
use vdrum_engine::engine::{track_trigger_note, SeqEv};
use vdrum_surface::{Action, Model, SeqData, SeqStatus};

/// The shared state as the surface reads it (worker thread).
pub struct SharedModel<'a>(pub &'a Shared);

impl Model for SharedModel<'_> {
    fn param(&self, idx: u16) -> u16 {
        if (idx as usize) < N {
            self.0.value(idx)
        } else {
            0
        }
    }
    fn voice_fader(&self, t: u8) -> i32 {
        self.0.faders.get(t as usize).map_or(0, |f| f.load(Ordering::Relaxed))
    }
    fn kit_name(&self) -> String {
        let n = self.0.kit_name_bytes();
        let end = n.iter().position(|b| *b == 0).unwrap_or(8);
        n[..end].iter().map(|b| *b as char).collect()
    }
    fn hits(&self, t: u8) -> u32 {
        self.0.hits.get(t as usize).map_or(0, |h| h.load(Ordering::Relaxed))
    }
    fn seq(&self) -> Box<dyn std::ops::Deref<Target = SeqData> + '_> {
        // A poisoned lock still holds valid data (edits are applied whole).
        Box::new(self.0.seq.lock().unwrap_or_else(|e| e.into_inner()))
    }
    fn seq_status(&self) -> SeqStatus {
        self.0.seq_status.load()
    }
    fn lfo(&self, t: u8) -> f32 {
        self.0.lfo.get(t as usize).map_or(0.5, |v| f32::from_bits(v.load(Ordering::Relaxed)))
    }
    fn mutes(&self) -> u8 {
        self.0.mutes.load(Ordering::Relaxed)
    }
    fn tempo(&self) -> f64 {
        (self.0.value(lxr_params::par::BPM) as f64).max(20.0)
    }
}

/// Load a kit: name and engine (the mirror follows when the audio thread has loaded it).
pub fn load_kit(shared: &Shared, kit: &Kit) {
    shared.set_kit_name(kit.name);
    shared.send(Command::LoadKit(kit.to_file_bytes()));
}

/// Carry out a surface action that needs only the shared state. Returns the action back if it
/// needs the file system ([`Action::KitBrowse`], [`Action::KitLoadBrowsed`], [`Action::SaveKit`],
/// [`Action::Compare`]); device messages, switches and the host clock do not apply to Move and are
/// dropped.
pub fn apply(shared: &Shared, action: Action) -> Option<Action> {
    match action {
        Action::SetParam { idx, value } => {
            if (idx as usize) < N {
                // The mirror is updated at once so that the next detent starts from the new value.
                shared.values[idx as usize].store(value, Ordering::Relaxed);
                shared.send(Command::Param(idx, value));
            }
        }
        Action::SetVoiceFader { track, step } => {
            if let Some(f) = shared.faders.get(track as usize) {
                f.store(step, Ordering::Relaxed);
                shared.send(Command::Fader(track, step));
            }
        }
        Action::Trigger { track, velocity } => {
            let (ch, note) = track_trigger_note(track as usize, |p| shared.value(p));
            shared.send(Command::Midi([0x90 | ch, note, velocity.clamp(1, 127)]));
        }
        Action::Panic => shared.send(Command::Panic),
        Action::InitKit => load_kit(shared, &Kit::init()),
        Action::Seq(edit) => {
            let evs = {
                let mut d = shared.seq.lock().unwrap_or_else(|e| e.into_inner());
                d.apply(&edit);
                vdrum_engine::seq_events(&d, &edit)
            };
            for ev in evs {
                shared.send(Command::Seq(ev));
            }
        }
        Action::SetMutes(m) => {
            shared.mutes.store(m & 0x7f, Ordering::Relaxed);
            shared.send(Command::Seq(SeqEv::Mutes(m & 0x7f)));
        }
        Action::LfoReset => shared.send(Command::Seq(SeqEv::LfoReset)),
        Action::Device(_) | Action::SetSwitch(..) | Action::SeqHostClock(_) => {}
        a @ (Action::KitBrowse(_) | Action::KitLoadBrowsed | Action::SaveKit | Action::Compare(_)) => return Some(a),
    }
    None
}
