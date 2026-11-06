// SPDX-License-Identifier: GPL-3.0-only
//! Simulator of VDrum's Launch Control XL 3 surface.
//!
//! A [`Sim`] runs the real [`vdrum_surface::Surface`] against a virtual device
//! ([`lcxl3::emulator::Device`]) and a stand-in for the plugin ([`SimModel`]). Every exchange goes
//! through the device's MIDI bytes, as with the hardware:
//!
//! ```text
//!   gesture ─► virtual device ─► MIDI (DAW Out) ─► decode ─► Surface ─► Actions ─► SimModel
//!                    ▲                                          │
//!                    └──── MIDI (DAW In) ◄── DeviceState ◄── Frame (LEDs, screen)
//! ```
//!
//! The same messages can go to and come from a real device instead ([`Sim::midi_in`],
//! [`Sim::take_midi_out`]); the browser page does that over Web MIDI.
//!
//! [`Sim::command`] takes and returns JSON, so the browser (through [`wasm`]) and the tests drive
//! the simulator the same way. Commands are listed at [`Sim::command`].

pub mod model;
pub mod wasm;

use lcxl3::emulator::Device;
use lcxl3::{decode, Button, Control, Input};
use serde_json::{json, Value};
use std::sync::Arc;
use vdrum_surface::rules::button_name;
use vdrum_surface::spec::{control_key, mode_name, Spec};
use vdrum_surface::{Action, DeviceState, Model, Surface};

pub use model::SimModel;

/// One log line.
#[derive(Clone, Debug, PartialEq)]
pub struct LogEntry {
    /// Simulator time in milliseconds.
    pub t: u64,
    /// `input` (a decoded device message), `action`, `param`, `midi_in`, `midi_out`, `rule`,
    /// `spec`, `error`.
    pub kind: &'static str,
    pub text: String,
}

/// Panel keys of the side buttons, as in the device drawing (`tools/surface_spec`).
pub const SIDE_KEYS: [(&str, Button); 10] = [
    ("SOLO", Button::SoloArm),
    ("MUTE", Button::MuteSelect),
    ("LOGO", Button::Logo),
    ("PAGE_UP", Button::PageUp),
    ("PAGE_DOWN", Button::PageDown),
    ("TRACK_LEFT", Button::TrackLeft),
    ("TRACK_RIGHT", Button::TrackRight),
    ("REC", Button::Record),
    ("PLAY", Button::Play),
    ("SHIFT", Button::Shift),
];

/// A control from its panel key (`K11`, `F1`, `T1`, `B8`, `SOLO`, ...).
pub fn control_from_panel_key(k: &str) -> Option<Control> {
    if let Some((_, b)) = SIDE_KEYS.iter().find(|(n, _)| *n == k) {
        return Some(Control::Button(*b));
    }
    vdrum_surface::spec::control_from_key(k)
}

/// The panel key of a control.
pub fn panel_key(c: Control) -> String {
    match c {
        Control::Button(b) => match SIDE_KEYS.iter().find(|(_, x)| *x == b) {
            Some((n, _)) => n.to_string(),
            None => control_key(c),
        },
        _ => control_key(c),
    }
}

fn hex(m: &[u8]) -> String {
    m.iter().map(|b| format!("{b:02X}")).collect::<Vec<_>>().join(" ")
}

/// Check one scenario expectation against a state from [`Sim::state`]. `path` is a dotted path
/// (`surface.mode`, `screen.lines.0`, `leds.K11`); a path ending in `~` checks that the value's text
/// (lines joined with ` | `) contains `want`. For an LED, `"lit"` and `"off"` check that it is on
/// or off at any colour.
pub fn check(state: &Value, path: &str, want: &Value) -> Result<(), String> {
    let (p, contains) = match path.strip_suffix('~') {
        Some(p) => (p, true),
        None => (path, false),
    };
    let mut v = state;
    for part in p.split('.') {
        v = match v {
            Value::Object(m) => m.get(part),
            Value::Array(a) => part.parse::<usize>().ok().and_then(|i| a.get(i)),
            _ => None,
        }
        .ok_or_else(|| format!("{path}: no such value in the state"))?;
    }
    let ok = if contains {
        let text = match v {
            Value::Array(a) => a.iter().map(|x| x.as_str().map(String::from).unwrap_or_else(|| x.to_string())).collect::<Vec<_>>().join(" | "),
            Value::String(s) => s.clone(),
            x => x.to_string(),
        };
        text.contains(want.as_str().unwrap_or_default())
    } else if want == "lit" || want == "off" {
        let lit = v.as_array().is_some_and(|a| a.iter().any(|x| x.as_u64().unwrap_or(0) > 0));
        lit == (want == "lit")
    } else {
        v == want
    };
    if ok {
        Ok(())
    } else {
        Err(format!("{path} is {v}, expected {}{want}", if contains { "text containing " } else { "" }))
    }
}

/// The simulator.
pub struct Sim {
    pub surface: Surface,
    pub model: SimModel,
    pub link: DeviceState,
    pub device: Device,
    now: u64,
    log: Vec<LogEntry>,
    /// Messages for a real device (Web MIDI), when one is attached.
    out: Vec<Vec<u8>>,
    /// A real device is attached: its input replaces the virtual device's gestures, and the
    /// surface's messages also go to it.
    pub hardware: bool,
    /// Detents turned per encoder (the cap's drawn rotation; the encoders are endless).
    turns: [i32; 24],
}

impl Default for Sim {
    fn default() -> Self {
        Sim::new(vdrum_surface::spec::builtin_arc().clone())
    }
}

impl Sim {
    /// A simulator running `spec`, connected (DAW mode on, device settings sent).
    pub fn new(spec: Arc<Spec>) -> Sim {
        let mut s = Sim {
            surface: Surface::with_spec(spec),
            model: SimModel::new(),
            link: DeviceState::default(),
            device: Device::new(),
            now: 0,
            log: Vec::new(),
            out: Vec::new(),
            hardware: false,
            turns: [0; 24],
        };
        s.connect();
        s
    }

    /// Send the connect sequence (DAW mode, relative encoders, display setup, device settings)
    /// and a full frame.
    pub fn connect(&mut self) {
        let mut msgs = self.link.connect();
        msgs.extend(self.surface.device_settings());
        self.send(msgs);
        self.tick(self.now);
    }

    pub fn now(&self) -> u64 {
        self.now
    }

    pub fn log(&self) -> &[LogEntry] {
        &self.log
    }

    fn note(&mut self, kind: &'static str, text: String) {
        self.log.push(LogEntry { t: self.now, kind, text });
    }

    /// Messages from the surface to the device: the virtual device takes them, and so does the
    /// real one when attached.
    fn send(&mut self, msgs: Vec<Vec<u8>>) {
        for m in msgs {
            self.note("midi_out", hex(&m));
            let replies = self.device.receive(&m, self.now);
            for r in replies {
                // The virtual device's replies (bitmap acknowledgements) go back to the surface
                // unless a real device answers instead.
                if !self.hardware {
                    self.receive(&r);
                }
            }
            if self.hardware {
                self.out.push(m);
            }
        }
    }

    /// One message from the device (virtual or real) to the surface.
    fn receive(&mut self, msg: &[u8]) {
        self.note("midi_in", hex(msg));
        let input = decode(msg);
        if matches!(input, Input::Other) {
            return;
        }
        self.note("input", format!("{input:?}"));
        if let Input::SurfaceMode(_) = input {
            // The device left DAW mode (its own Mode button): bring it back, as the plugin does.
            self.link.invalidate();
        }
        let before = self.model.params.borrow().clone();
        let actions = self.surface.handle(&input, self.now, &self.model);
        if let Input::Button { .. } = input {
            let r = self.surface.last_rule();
            let text = match r.and_then(|i| self.surface.spec().rules.navigation.get(i).map(|n| (i, n.note.clone()))) {
                Some((i, note)) => format!("navigation rule {}{}", i + 1, if note.is_empty() { String::new() } else { format!(": {note}") }),
                None => "no navigation rule".to_string(),
            };
            self.note("rule", text);
        }
        self.act(actions, &before);
        self.frame();
    }

    fn act(&mut self, actions: Vec<Action>, before: &std::collections::HashMap<u16, u16>) {
        let mut device = Vec::new();
        for a in &actions {
            match a {
                Action::SetParam { idx, value } => {
                    let (id, name, text) = match lxr_params::info(*idx) {
                        Some(i) => (i.id, i.name, lxr_params::format_value(i, (*value).min(255) as u8)),
                        None => ("?", "?", value.to_string()),
                    };
                    let old = before.get(idx).copied().unwrap_or(0);
                    self.note("param", format!("{id} ({name}): {old} -> {value} ({})", text.trim()));
                }
                Action::Device(m) => device.push(m.clone()),
                a => self.note("action", format!("{a:?}")),
            }
        }
        if let Some([t, n, v]) = self.model.apply(&actions) {
            self.surface.notify(&t, &n, &v);
        }
        self.send(device);
    }

    /// Send what changed in the frame.
    fn frame(&mut self) {
        let f = self.surface.frame(self.now, &self.model);
        let msgs = self.link.update(&f);
        self.send(msgs);
    }

    /// Advance time to `now` (milliseconds; never backwards): hold actions, momentary effects,
    /// the sequencer clock, animations.
    pub fn tick(&mut self, now: u64) {
        self.now = self.now.max(now);
        self.model.tick(self.now);
        let before = self.model.params.borrow().clone();
        let actions = self.surface.tick(self.now, &self.model);
        self.act(actions, &before);
        self.frame();
    }

    /// Messages from a real device (Web MIDI).
    pub fn midi_in(&mut self, msg: &[u8]) {
        self.receive(msg);
    }

    /// Messages for the real device since the last call.
    pub fn take_midi_out(&mut self) -> Vec<Vec<u8>> {
        std::mem::take(&mut self.out)
    }

    /// Turn an encoder by `detents` on the virtual device.
    pub fn turn(&mut self, row: u8, col: u8, detents: i32) {
        if row < 3 && col < 8 {
            self.turns[(row * 8 + col) as usize] += detents;
        }
        for m in self.device.turn(row, col, detents, self.now) {
            self.receive(&m);
        }
    }

    /// Move a fader of the virtual device to `value` (0..=127).
    pub fn fader(&mut self, col: u8, value: u8) {
        for m in self.device.move_fader(col, value, self.now) {
            self.receive(&m);
        }
    }

    /// Press or release a button of the virtual device.
    pub fn button(&mut self, b: Button, pressed: bool) {
        for m in self.device.button(b, pressed) {
            self.receive(&m);
        }
    }

    /// Load another spec (JSON). On error the running spec stays and the error is returned.
    pub fn load_spec(&mut self, json: &str) -> Result<(), String> {
        match Spec::parse(json) {
            Ok(spec) => {
                self.surface.set_spec(Arc::new(spec));
                self.note("spec", "spec loaded".into());
                self.frame();
                Ok(())
            }
            Err(e) => {
                self.note("error", e.clone());
                Err(e)
            }
        }
    }

    /// What the virtual device shows and the surface's state, as JSON.
    pub fn state(&self) -> Value {
        let mut leds = serde_json::Map::new();
        for c in vdrum_surface::frame::led_controls() {
            let (r, g, b) = self.device.led(c).rgb8();
            leds.insert(panel_key(c), json!([r, g, b]));
        }
        let screen = self.device.screen(self.now);
        let bitmap = screen.bitmap.as_ref().map(|b| {
            (0..64).map(|y| (0..128).map(|x| if b.get(x, y) { '1' } else { '0' }).collect::<String>()).collect::<Vec<_>>()
        });
        let s = &self.surface;
        let spec = s.spec();
        let (slot, name) = spec.mode_info(s.mode());
        let view = s.view(self.now, &self.model);
        let mut cells = serde_json::Map::new();
        for c in &view.controls {
            cells.insert(
                panel_key(c.control),
                json!({
                    "label": c.label,
                    "section": c.section,
                    "colour": [c.colour.0, c.colour.1, c.colour.2],
                    "value": c.value,
                    "frac": c.frac,
                    "param": c.param.map(|p| p.0),
                    "fader": c.fader.map(|(pos, picked)| json!({"pos": pos, "picked": picked})),
                }),
            );
        }
        let st = self.model.seq_status();
        json!({
            "now": self.now,
            "leds": leds,
            "screen": { "source": screen.source, "arrangement": screen.arrangement, "lines": screen.lines, "bitmap": bitmap },
            "faders": self.device.fader_pos,
            "turns": self.turns.to_vec(),
            "daw_mode": self.device.daw_mode,
            "unknown": self.device.unknown.iter().map(|m| hex(m)).collect::<Vec<_>>(),
            "surface": {
                "mode": mode_name(s.mode()),
                "slot": slot,
                "name": name,
                "shift": s.shift(),
                "seq_view": s.seq_view(),
                "sticky": s.seq_view_sticky(),
                "seq_track": s.seq_track(),
                "seq_bar": s.seq_bar(),
                "held": s.held_buttons().into_iter().map(button_name).collect::<Vec<_>>(),
                "held_steps": s.held_steps(),
                "browsing": s.browsing(),
                "last_rule": s.last_rule(),
                "layers": view.layers,
            },
            "cells": cells,
            "seq": { "running": st.running, "pattern": st.pattern, "pending": st.pending, "silent": st.silent, "step": st.step.to_vec() },
            "kit": self.model.kit_name(),
            "log_len": self.log.len(),
        })
    }

    /// Run one JSON command and return its result. Commands (`cmd`):
    ///
    /// * `turn` `{row, col, detents}`; `fader` `{col, value}`; `button` `{key, pressed}` (`key`: a
    ///   panel key, `K11`..., `T1`, `B8`, `SOLO`, `SHIFT`, ...); `press` `{key}` (press and release).
    /// * `tick` `{now}`; `advance` `{ms}`.
    /// * `midi_in` `{bytes: [..]}` (from a real device); `midi_out` (returns and clears the
    ///   messages for it); `hardware` `{on}` (attach / detach a real device; attaching sends the
    ///   connect sequence and a full frame to it).
    /// * `spec` `{json}` (load a spec; the result has `error` on failure).
    /// * `state`; `log` `{from}` (entries from index `from`).
    ///
    /// Every command except `log`, `midi_out` and `spec` returns the state.
    pub fn command(&mut self, cmd: &Value) -> Value {
        let u = |k: &str| cmd.get(k).and_then(Value::as_u64).unwrap_or(0);
        let i = |k: &str| cmd.get(k).and_then(Value::as_i64).unwrap_or(0);
        let key = cmd.get("key").and_then(Value::as_str).unwrap_or("");
        match cmd.get("cmd").and_then(Value::as_str).unwrap_or("") {
            "turn" => self.turn(u("row") as u8, u("col") as u8, i("detents") as i32),
            "fader" => self.fader(u("col") as u8, u("value").min(127) as u8),
            "button" | "press" => {
                let Some(Control::Button(b)) = control_from_panel_key(key) else {
                    return json!({"error": format!("unknown button {key}")});
                };
                if cmd["cmd"] == "press" {
                    self.button(b, true);
                    self.button(b, false);
                } else {
                    self.button(b, cmd.get("pressed").and_then(Value::as_bool).unwrap_or(true));
                }
            }
            "tick" => self.tick(u("now")),
            "advance" => {
                let to = self.now + u("ms");
                // Step through time so hold actions and the sequencer see every 10 ms.
                while self.now + 10 < to {
                    self.tick(self.now + 10);
                }
                self.tick(to);
            }
            "midi_in" => {
                let bytes: Vec<u8> = cmd
                    .get("bytes")
                    .and_then(Value::as_array)
                    .map(|a| a.iter().filter_map(|v| v.as_u64().map(|v| v as u8)).collect())
                    .unwrap_or_default();
                self.midi_in(&bytes);
            }
            "midi_out" => {
                return json!(self.take_midi_out());
            }
            "hardware" => {
                let on = cmd.get("on").and_then(Value::as_bool).unwrap_or(false);
                if on && !self.hardware {
                    self.hardware = true;
                    self.out.clear();
                    self.connect();
                } else if !on && self.hardware {
                    // Return the real device to standalone mode (the page sends `midi_out`
                    // before closing the port), then bring the virtual device up to date.
                    self.out = self.link.disconnect();
                    self.hardware = false;
                    self.connect();
                }
            }
            "spec" => {
                let text = cmd.get("json").and_then(Value::as_str).unwrap_or("");
                return match self.load_spec(text) {
                    Ok(()) => json!({"ok": true}),
                    Err(e) => json!({"error": e}),
                };
            }
            "log" => {
                let from = (u("from") as usize).min(self.log.len());
                return Value::Array(
                    self.log[from..].iter().map(|e| json!({"t": e.t, "kind": e.kind, "text": e.text})).collect(),
                );
            }
            "expect" => {
                let state = self.state();
                let failures: Vec<String> = cmd
                    .get("checks")
                    .and_then(Value::as_object)
                    .map(|m| m.iter().filter_map(|(path, want)| check(&state, path, want).err()).collect())
                    .unwrap_or_default();
                return json!({ "failures": failures });
            }
            "state" | "reset" => {}
            other => return json!({"error": format!("unknown command {other}")}),
        }
        self.state()
    }
}
