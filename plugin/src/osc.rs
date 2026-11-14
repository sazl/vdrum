// SPDX-License-Identifier: GPL-3.0-only
//! VDrum's OSC 1.0 and OSCQuery server (`docs/osc-oscquery-plan.md`).
//!
//! The protocol (HTTP, WebSocket, UDP, mDNS) is the `surface-control-oscquery` crate; this
//! module is VDrum's address space and what a value set by a client does:
//!
//! | Address | Type | Access | |
//! |---|---|---|---|
//! | `/vdrum/param/<id>` | `i` | read-write | every host parameter, `<id>` as in `lxr_params` |
//! | `/vdrum/param/<id>/text` | `s` | read | the value as VDrum displays it |
//! | `/vdrum/fader/<1..7>` (and `/text`) | `i` | read-write | voice faders, 0..`FADER_MAX` |
//! | `/vdrum/switch/sum_aux`, `/vdrum/switch/hardware_clock` | `T` | read-write | |
//! | `/vdrum/midi` | `m` | write | a MIDI message, played like host MIDI |
//! | `/vdrum/kit/name` | `s` | read | |
//!
//! Values set by clients go to the audio thread as [`Command`]s, which treat them like a
//! learned MIDI CC: the editor copies them to the host parameters. Changes are read from
//! [`Shared`] every 10 ms, whoever made them (the DAW, the editor, MIDI, kit loads, OSC).
//!
//! Environment: `VDRUM_OSC=0` turns the server off; `VDRUM_OSC_PORT` fixes the TCP and the UDP
//! port (the same number); `VDRUM_OSC_BIND` is the address to bind (`127.0.0.1` by default);
//! `VDRUM_OSC_MDNS=0` turns mDNS off.

use crate::params::{self, fader_text, value_text, Command, Shared, FADER_MAX, FADER_NAMES, NUM_FADERS};
use lxr_params::ParamInfo;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use surface_control_oscquery as oscq;
use surface_control_oscquery::{Node, Value};

/// The root of the address space.
pub const ROOT: &str = "/vdrum";

const SWITCH_NAMES: [&str; 2] = ["sum_aux", "hardware_clock"];
const SWITCH_LABELS: [&str; 2] = ["Sum Aux Into Main", "Hardware Clock (44140.625 Hz)"];

/// What an address is.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Kind {
    Param(&'static ParamInfo),
    /// A voice fader, 0-based.
    Fader(usize),
    /// `sum_aux` (0) or `hardware_clock` (1).
    Switch(usize),
    Midi,
    KitName,
}

/// Every address, in the order of the address space.
fn kinds() -> Vec<Kind> {
    let mut v: Vec<Kind> = params::host_param_infos().into_iter().map(Kind::Param).collect();
    v.extend((0..NUM_FADERS).map(Kind::Fader));
    v.extend((0..2).map(Kind::Switch));
    v.push(Kind::Midi);
    v.push(Kind::KitName);
    v
}

fn node(k: Kind) -> Node {
    match k {
        Kind::Param(i) => Node::int(format!("{ROOT}/param/{}", i.id), i.min as i32, i.max as i32)
            .description(i.name)
            .tag(params::group_of(i))
            .text(),
        Kind::Fader(f) => Node::int(format!("{ROOT}/fader/{}", f + 1), 0, FADER_MAX)
            .description(FADER_NAMES[f])
            .tag("Faders")
            .text(),
        Kind::Switch(s) => {
            Node::bool(format!("{ROOT}/switch/{}", SWITCH_NAMES[s])).description(SWITCH_LABELS[s]).tag("Switches")
        }
        Kind::Midi => Node::midi(format!("{ROOT}/midi"))
            .description("A MIDI message for the engine, as host MIDI (notes play the voices)"),
        Kind::KitName => Node::string(format!("{ROOT}/kit/name")).description("Kit name"),
    }
}

/// VDrum as an OSCQuery application.
pub struct Vdrum {
    shared: Arc<Shared>,
    kinds: Vec<Kind>,
}

impl Vdrum {
    pub fn new(shared: Arc<Shared>) -> Vdrum {
        Vdrum { shared, kinds: kinds() }
    }

    /// The command for a value a client set; `None` for read-only addresses.
    fn command(&self, index: usize, v: Value) -> Option<Command> {
        Some(match (self.kinds.get(index)?, v) {
            (Kind::Param(i), Value::Int(x)) => Command::Param(i.index, x as u16),
            (Kind::Fader(f), Value::Int(x)) => Command::Fader(*f as u8, x),
            (Kind::Switch(s), Value::Bool(b)) => Command::Switch(*s as u8, b),
            (Kind::Midi, Value::Midi(m)) => Command::Midi(m),
            _ => return None,
        })
    }
}

fn kit_name(shared: &Shared) -> String {
    String::from_utf8_lossy(&shared.name_bytes()).trim_end_matches(['\0', ' ']).to_string()
}

impl oscq::App for Vdrum {
    fn nodes(&self) -> Vec<Node> {
        self.kinds.iter().map(|k| node(*k)).collect()
    }

    fn value(&self, index: usize) -> Option<Value> {
        let sh = &*self.shared;
        Some(match self.kinds.get(index)? {
            Kind::Param(i) => Value::Int(sh.value(i.index) as i32),
            Kind::Fader(f) => Value::Int(sh.fader_values[*f].load(Ordering::Relaxed)),
            Kind::Switch(0) => Value::Bool(sh.sum_aux_on.load(Ordering::Relaxed)),
            Kind::Switch(_) => Value::Bool(sh.hw_clock_on.load(Ordering::Relaxed)),
            Kind::KitName => Value::String(kit_name(sh)),
            Kind::Midi => return None,
        })
    }

    fn text(&self, index: usize, v: &Value) -> Option<String> {
        let Value::Int(x) = v else { return None };
        match self.kinds.get(index)? {
            Kind::Param(i) => Some(value_text(i, *x as u16)),
            Kind::Fader(_) => Some(fader_text(*x)),
            _ => None,
        }
    }

    fn set(&self, index: usize, v: Value) {
        if let Some(c) = self.command(index, v) {
            // A full queue drops the command, as for the editor's.
            let _ = self.shared.commands.push(c);
        }
    }
}

/// Starts the server with the settings from the environment; `None` when `VDRUM_OSC=0` or it
/// cannot start (the reason is logged).
pub fn start(shared: Arc<Shared>) -> Option<oscq::Server> {
    let cfg = oscq::Config::from_env("VDrum", "VDRUM")?;
    match oscq::Server::start(Vdrum::new(shared), cfg) {
        Ok(s) => Some(s),
        Err(e) => {
            nih_plug::nih_log!("VDrum: OSC server not started: {e}");
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oscq::rosc::{OscMidiMessage, OscType};
    use oscq::App;

    fn index(app: &Vdrum, path: &str) -> usize {
        app.nodes().iter().position(|n| n.path == path).unwrap()
    }

    #[test]
    fn address_space() {
        let app = Vdrum::new(Arc::new(Shared::new()));
        let nodes = app.nodes();
        assert_eq!(nodes.len(), params::host_param_infos().len() + NUM_FADERS + 2 + 2);
        let space = oscq::address_space(&app, &nodes, "/vdrum/param/coarse1").unwrap();
        assert_eq!(space["TYPE"], "i");
        assert_eq!(space["ACCESS"], 3);
        assert_eq!(space["RANGE"][0]["MAX"], 127);
        assert_eq!(space["CONTENTS"]["text"]["TYPE"], "s");
        assert_eq!(space["TAGS"][0], "Drum 1/Oscillator");
        let all = oscq::address_space(&app, &nodes, "/").unwrap();
        assert_eq!(all["CONTENTS"]["vdrum"]["CONTENTS"]["midi"]["TYPE"], "m");
        assert_eq!(all["CONTENTS"]["vdrum"]["CONTENTS"]["fader"]["CONTENTS"]["7"]["DESCRIPTION"], "Open Hi-hat Fader");
    }

    #[test]
    fn values_become_commands() {
        let app = Vdrum::new(Arc::new(Shared::new()));
        let nodes = app.nodes();
        let info = lxr_params::by_id("coarse1").unwrap();
        let cmd = |path: &str, a: OscType| {
            let i = index(&app, path);
            app.command(i, nodes[i].accept(&a)?)
        };
        assert!(matches!(cmd("/vdrum/param/coarse1", OscType::Float(1000.0)), Some(Command::Param(i, v)) if i == info.index && v == info.max));
        assert!(matches!(cmd("/vdrum/param/coarse1", OscType::Double(3.4)), Some(Command::Param(_, 3))));
        assert!(matches!(cmd("/vdrum/fader/2", OscType::Int(-5)), Some(Command::Fader(1, 0))));
        assert!(matches!(cmd("/vdrum/switch/sum_aux", OscType::Bool(true)), Some(Command::Switch(0, true))));
        let midi = OscType::Midi(OscMidiMessage { port: 0, status: 0x90, data1: 36, data2: 100 });
        assert!(matches!(cmd("/vdrum/midi", midi), Some(Command::Midi([0x90, 36, 100]))));
        assert!(cmd("/vdrum/kit/name", OscType::String("x".into())).is_none());
    }
}
