// SPDX-License-Identifier: GPL-3.0-only
//! The Schwung plugin C ABI (`src/host/plugin_api_v1.h` of Schwung 1.9.2, MIT): `host_api_v1_t`,
//! which the host passes in, and `plugin_api_v2_t`, which `move_plugin_init_v2` returns.
//!
//! Every entry point runs on Move's SPI audio callback (SCHED_FIFO 70, core 3), except module
//! loading (`dlopen`, `create_instance`), which runs on the shim's worker thread. The layouts must
//! match the header exactly; the offset of `reserved` (120) is checked at compile time, as the
//! header does.

use std::ffi::{c_char, c_int, c_void};

pub const MOVE_PLUGIN_API_VERSION_2: u32 = 2;
pub const MOVE_SAMPLE_RATE: c_int = 44100;
pub const MOVE_FRAMES_PER_BLOCK: usize = 128;

/// `MOVE_MIDI_SOURCE_*`.
pub const MIDI_SOURCE_INTERNAL: c_int = 0;
pub const MIDI_SOURCE_EXTERNAL: c_int = 2;

pub type ModEmitValueFn = Option<
    unsafe extern "C" fn(*mut c_void, *const c_char, *const c_char, *const c_char, f32, f32, f32, c_int, c_int) -> c_int,
>;
pub type ModClearSourceFn = Option<unsafe extern "C" fn(*mut c_void, *const c_char)>;

/// `host_api_v1_t`.
#[repr(C)]
pub struct HostApiV1 {
    pub api_version: u32,
    pub sample_rate: c_int,
    pub frames_per_block: c_int,
    pub mapped_memory: *mut u8,
    pub audio_out_offset: c_int,
    pub audio_in_offset: c_int,
    pub log: Option<unsafe extern "C" fn(*const c_char)>,
    pub midi_send_internal: Option<unsafe extern "C" fn(*const u8, c_int) -> c_int>,
    pub midi_send_external: Option<unsafe extern "C" fn(*const u8, c_int) -> c_int>,
    pub get_clock_status: Option<unsafe extern "C" fn() -> c_int>,
    pub mod_emit_value: ModEmitValueFn,
    pub mod_clear_source: ModClearSourceFn,
    pub mod_host_ctx: *mut c_void,
    pub get_bpm: Option<unsafe extern "C" fn() -> f32>,
    pub midi_inject_to_move: Option<unsafe extern "C" fn(*const u8, c_int) -> c_int>,
    pub slot_recv_channel: Option<unsafe extern "C" fn(*mut c_void) -> c_int>,
    pub get_beat_position: Option<unsafe extern "C" fn() -> f64>,
    pub reserved: [*mut c_void; 8],
}

#[cfg(target_pointer_width = "64")]
const _: () = assert!(std::mem::offset_of!(HostApiV1, reserved) == 120);

/// `plugin_api_v2_t`.
#[repr(C)]
pub struct PluginApiV2 {
    pub api_version: u32,
    pub create_instance: unsafe extern "C" fn(*const c_char, *const c_char) -> *mut c_void,
    pub destroy_instance: unsafe extern "C" fn(*mut c_void),
    pub on_midi: unsafe extern "C" fn(*mut c_void, *const u8, c_int, c_int),
    pub set_param: unsafe extern "C" fn(*mut c_void, *const c_char, *const c_char),
    pub get_param: unsafe extern "C" fn(*mut c_void, *const c_char, *mut c_char, c_int) -> c_int,
    pub get_error: unsafe extern "C" fn(*mut c_void, *mut c_char, c_int) -> c_int,
    pub render_block: unsafe extern "C" fn(*mut c_void, *mut i16, c_int),
}
