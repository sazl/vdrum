// SPDX-License-Identifier: GPL-3.0-only
//! The parts of VDrum that every front end shares: the engine wrapper around the `lxr-core` port
//! (with VDrum's sequencer extensions), the sequencer's pattern data and edits, the kit library and
//! the LXR-02 file formats (`.SND`, `.PAT`, project folders), VDrum's defaults, and the guard that
//! keeps the engine's floating point mode IEEE.
//!
//! User: the plugin (`plugin/`). Nothing here starts threads or depends on a plugin framework.

pub mod defaults;
pub mod engine;
pub mod fp;
pub mod library;
pub mod lxr_files;
pub mod seq;

use engine::SeqEv;
use seq::{SeqData, SeqEdit};

/// The engine operations that bring the engine's copy of the sequencer data up to date after
/// `edit` was applied to `d`.
pub fn seq_events(d: &SeqData, edit: &SeqEdit) -> Vec<SeqEv> {
    match *edit {
        SeqEdit::Step { pat, track, step, .. } => vec![SeqEv::Step { pat, track, step, data: d.step(pat, track, step) }],
        SeqEdit::Euclid { pat, track, .. } => vec![SeqEv::Bits { pat, track, bits: d.track(pat, track).bits() }],
        SeqEdit::Length { pat, track, .. } => {
            let t = d.track(pat, track);
            let mut v = vec![SeqEv::Length { pat, track, length: t.length }];
            if t.fills > 0 {
                v.push(SeqEv::Bits { pat, track, bits: t.bits() });
            }
            v
        }
        SeqEdit::Shuffle { pat, .. } => {
            let value = d.patterns.get(pat as usize).map_or(0, |p| p.shuffle);
            vec![SeqEv::Shuffle { pat, value }]
        }
        SeqEdit::Density { track, .. } => {
            let value = d.density.get(track as usize).copied().unwrap_or(d.density_all);
            vec![SeqEv::Density { track, value }]
        }
        SeqEdit::NextPattern(p) => vec![SeqEv::NextPattern(p.min(63))],
        SeqEdit::Run(on) => vec![SeqEv::Run(on)],
        SeqEdit::Silence(on) => vec![SeqEv::Silence(on)],
    }
}
