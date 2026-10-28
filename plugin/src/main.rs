// SPDX-License-Identifier: GPL-3.0-only
//! Standalone VDrum (JACK/ALSA/CoreAudio/WASAPI via nih-plug's standalone wrapper).
//! Run with `--help` for the audio/MIDI backend options.

use nih_plug::prelude::*;
use vdrum::VDrum;

fn main() {
    nih_export_standalone::<VDrum>();
}
