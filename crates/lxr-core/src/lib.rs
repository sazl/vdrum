//! `lxr-core`: bit-exact Rust port of the sound engine of the Erica Synths LXR-02 (LXR V2) firmware.
//!
//! The module tree mirrors the C files (docs/ARCHITECTURE.md, porting rule 1): one module per C file
//! (header and source together), C names for types, fields, constants and functions. C globals are fields of
//! [`Lxr`]; functions that use globals are methods on `Lxr`, the others are free functions.
//!
//! | C | Rust |
//! |---|---|
//! | `Core/{Inc,Src}/Soundengine/DSPAudio/X` | `dsp::x` (`1PoleLp` → `dsp::one_pole_lp`) |
//! | `Core/{Inc,Src}/Soundengine/DSPAudio/FX/X` | `dsp::fx::x` |
//! | `Core/{Inc,Src}/Soundengine/DSPAudio/FX/VerbTools/X` | `dsp::fx::verb_tools::x` |
//! | `Core/{Inc,Src}/Soundengine/MIDI/X` | `midi::x` |
//! | `Core/{Inc,Src}/Soundengine/Sequencer/sequencer` | `sequencer` |
//! | `Core/{Inc,Src}/Soundengine/Sequencer/X` | `sequencer::x` |
//! | `Core/{Inc,Src}/Soundengine/X` (FxBus, Soundengine, FIFO, config, ...) | `x` |
//! | `Core/{Inc,Src}/Preset/presetManager`, `PresetManager.h` | `preset` |
//! | `Core/{Inc,Src}/Preset/X` (SeqStep, initKit) | `preset::x` |
//! | `Core/Inc/Parameters.h`, `Core/Src/Parameters.c` | `parameters` |
//! | `Core/Src/frontPanelParser.c`, `Core/Inc/frontPanelParser_ui.h` | `front_panel_parser` |
//! | `Core/Inc/menu/menu.h`, `Core/Src/menu/Cc2Text.c` | `menu`, `menu::cc2_text` |
//! | CMSIS intrinsics (`cmsis_gcc.h`) | `cmsis` |
//! | hardware / UI functions called by engine code | `external` |
//!
//! Non-C modules: [`engine`] (`Lxr`), [`fpu`] (fused multiply-add helpers, flush-to-zero, libm),
//! [`rng`] (deterministic `GetRngValue`), [`memmap`] (`CPtr`, firmware addresses), [`place`]
//! (struct pointer parameters), [`tables`] (generated data tables), [`prelude`].

#![allow(non_snake_case, non_camel_case_types, non_upper_case_globals, clippy::all)]
// skeleton: stubs do not use their parameters yet
#![allow(unused_variables, unused_imports, dead_code, unreachable_code)]

pub mod engine;
pub mod fpu;
pub mod memmap;
pub mod place;
pub mod prelude;
pub mod rng;
pub mod tables;

pub mod cmsis;
pub mod config;
pub mod datatypes;
pub mod dsp;
pub mod external;
pub mod fifo;
pub mod front_panel_parser;
pub mod fx_bus;
pub mod globals;
pub mod menu;
pub mod midi;
pub mod parameters;
pub mod preset;
pub mod sequencer;
pub mod soundengine;

pub use engine::{Block, Lxr};
