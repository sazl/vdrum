// SPDX-License-Identifier: GPL-3.0-only
//! A C ABI for WebAssembly (`wasm32-unknown-unknown`), without bindings generators: the page
//! writes a UTF-8 JSON command into memory it got from [`sim_alloc`], calls [`sim_command`], and
//! reads the UTF-8 JSON result at [`sim_result_ptr`] / [`sim_result_len`].
//!
//! ```js
//! const enc = new TextEncoder(), dec = new TextDecoder();
//! function call(cmd) {
//!   const bytes = enc.encode(JSON.stringify(cmd));
//!   const p = ex.sim_alloc(bytes.length);
//!   new Uint8Array(ex.memory.buffer, p, bytes.length).set(bytes);
//!   ex.sim_command(p, bytes.length);              // frees the command
//!   return JSON.parse(dec.decode(new Uint8Array(ex.memory.buffer, ex.sim_result_ptr(), ex.sim_result_len())));
//! }
//! ```
//!
//! One simulator per module instance, created on the first command (built-in spec). Also builds
//! natively, so the tests exercise the same functions.

use crate::Sim;
use std::cell::RefCell;

thread_local! {
    static SIM: RefCell<Option<Sim>> = const { RefCell::new(None) };
    static RESULT: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

/// Memory for a command of `len` bytes.
#[no_mangle]
pub extern "C" fn sim_alloc(len: usize) -> *mut u8 {
    let mut v = Vec::<u8>::with_capacity(len.max(1));
    let p = v.as_mut_ptr();
    std::mem::forget(v);
    p
}

/// Free memory from [`sim_alloc`] that was not passed to [`sim_command`].
///
/// # Safety
/// `ptr` and `len` must come from one [`sim_alloc`] call.
#[no_mangle]
pub unsafe extern "C" fn sim_free(ptr: *mut u8, len: usize) {
    drop(Vec::from_raw_parts(ptr, 0, len.max(1)));
}

/// Run the JSON command at `ptr` (from [`sim_alloc`], freed here). The result is at
/// [`sim_result_ptr`] until the next call.
///
/// # Safety
/// `ptr` and `len` must come from one [`sim_alloc`] call, with `len` bytes written.
#[no_mangle]
pub unsafe extern "C" fn sim_command(ptr: *mut u8, len: usize) {
    let bytes = Vec::from_raw_parts(ptr, len, len.max(1));
    let out = run(&bytes);
    RESULT.with(|r| *r.borrow_mut() = out.into_bytes());
}

#[no_mangle]
pub extern "C" fn sim_result_ptr() -> *const u8 {
    RESULT.with(|r| r.borrow().as_ptr())
}

#[no_mangle]
pub extern "C" fn sim_result_len() -> usize {
    RESULT.with(|r| r.borrow().len())
}

/// Run one command on the module's simulator (the native side of [`sim_command`]).
pub fn run(bytes: &[u8]) -> String {
    let cmd: serde_json::Value = match serde_json::from_slice(bytes) {
        Ok(v) => v,
        Err(e) => return serde_json::json!({"error": format!("command: {e}")}).to_string(),
    };
    SIM.with(|s| {
        let mut s = s.borrow_mut();
        if cmd.get("cmd").and_then(|c| c.as_str()) == Some("reset") || s.is_none() {
            *s = Some(Sim::default());
        }
        s.as_mut().map(|sim| sim.command(&cmd)).unwrap_or_default().to_string()
    })
}
