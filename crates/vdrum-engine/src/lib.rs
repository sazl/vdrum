// SPDX-License-Identifier: GPL-3.0-only
//! The parts of VDrum that every front end shares: the engine wrapper around the `lxr-core` port,
//! the kit library and the LXR-02 file formats (`.SND`, project folders), VDrum's defaults, and the
//! guard that keeps the engine's floating point mode IEEE.
//!
//! User: the plugin (`plugin/`). Nothing here starts threads or depends on a plugin framework.

pub mod defaults;
pub mod engine;
pub mod fp;
pub mod library;
pub mod lxr_files;
