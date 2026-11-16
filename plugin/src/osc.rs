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
//! | `/vdrum/param/<id>/values` | `s` | read | modulation destinations only: the values the destination's voice allows, as decimal numbers separated by spaces ("off" and the voice's targets, as the editor offers) |
//! | `/vdrum/fader/<1..7>` (and `/text`) | `i` | read-write | voice faders, 0..`FADER_MAX` |
//! | `/vdrum/switch/sum_aux`, `/vdrum/switch/hardware_clock` | `T` | read-write | |
//! | `/vdrum/midi` | `m` | write | a MIDI message, played like host MIDI |
//! | `/vdrum/hits/<1..7>` | `i` | read | counts the track's triggers from any source (wraps at 10000) |
//! | `/vdrum/mute/<1..7>` | `T` | read-write | the track's notes are dropped from any source |
//! | `/vdrum/lfo/<1..7>` | `f` | read | the voice's LFO output, 0 to 1, in steps of 0.01 |
//! | `/vdrum/lfo/reset` | `T` | write | true restarts every LFO at its phase offset |
//! | `/vdrum/fx/p<1..4>/name` | `s` | read | what FX parameter 1 to 4 is for the current FX type ("FX Drive") |
//! | `/vdrum/fx/sub` (and `/text`) | `i` | read-write | the FX type's own type parameter (distortion type, ring modulator waveform, compressor ratio, delay type); text "none for this FX" when the type has none |
//! | `/vdrum/transport/beat` | `f` | read | the host's position in beats when its transport last started |
//! | `/vdrum/transport/playing` | `T` | read | the host's transport plays |
//! | `/vdrum/transport/tempo` | `f` | read | the host's tempo in bpm (0: unknown) |
//! | `/vdrum/kit/name` | `s` | read | |
//! | `/vdrum/kit/browse` (and `/text`) | `i` | read-write | the place of the kit shown (library slot, or file in its folder); a higher value shows the next kit, a lower one the previous one: the library's filled slots, or without any the `.snd` files in the folder of the current kit file; the text is the kit shown |
//! | `/vdrum/kit/browsing` | `T` | read | true while browse shows a kit that is not loaded (until a load or a set of any other address) |
//! | `/vdrum/kit/load` | `T` | write | true loads the kit shown by browse |
//! | `/vdrum/kit/init` | `T` | write | true loads the init kit |
//! | `/vdrum/kit/save` | `T` | write | true saves the kit into its library slot, else into its kit file |
//! | `/vdrum/kit/compare` | `T` | read-write | true: the kit as last loaded or saved plays; false: the edited kit again |
//! | `/vdrum/kit/status` | `s` | read | the result of the last browse, load, init, save or compare |
//! | `/vdrum/panic` | `T` | write | true silences every voice |
//!
//! Values set by clients go to the audio thread as [`Command`]s, which treat them like a
//! learned MIDI CC: the editor copies them to the host parameters. Changes are read from
//! [`Shared`] every 10 ms, whoever made them (the DAW, the editor, MIDI, kit loads, OSC).
//! A set of any other address ends kit browsing.
//!
//! Environment: `VDRUM_OSC=0` turns the server off; `VDRUM_OSC_PORT` fixes the TCP and the UDP
//! port (the same number); `VDRUM_OSC_BIND` is the address to bind (`127.0.0.1` by default);
//! `VDRUM_OSC_MDNS=0` turns mDNS off.

use crate::library;
use crate::params::{self, fader_text, value_text, Command, Shared, FADER_MAX, FADER_NAMES, NUM_FADERS};
use lxr_params::kit::Kit;
use lxr_params::{par, Dtype, ParamInfo, MOD_TARGET_VOICE_OFFSETS};
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use surface_control_oscquery as oscq;
use surface_control_oscquery::{Node, Value};

/// The root of the address space.
pub const ROOT: &str = "/vdrum";

const SWITCH_NAMES: [&str; 2] = ["sum_aux", "hardware_clock"];
const SWITCH_LABELS: [&str; 2] = ["Sum Aux Into Main", "Hardware Clock (44140.625 Hz)"];
const TRACK_NAMES: [&str; 7] = ["Drum 1", "Drum 2", "Drum 3", "Snare", "Cymbal", "Hi-hat", "Open Hi-hat"];
/// `/vdrum/hits/<t>` wraps at this.
pub const HITS_WRAP: u32 = 10_000;
const FX_PARAMS: [u16; 4] = [par::FX_P1, par::FX_P2, par::FX_P3, par::FX_P4];

/// Kit actions (`/vdrum/kit/*`, `/vdrum/panic`, `/vdrum/lfo/reset`).
#[derive(Clone, Copy, Debug, PartialEq)]
enum KitAction {
    Load,
    Init,
    Save,
}

/// What an address is.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Kind {
    Param(&'static ParamInfo),
    /// The allowed values of a modulation destination.
    Values(&'static ParamInfo),
    /// A voice fader, 0-based.
    Fader(usize),
    /// `sum_aux` (0) or `hardware_clock` (1).
    Switch(usize),
    Midi,
    Hits(usize),
    Mute(usize),
    Lfo(usize),
    LfoReset,
    /// The name of FX parameter 1 to 4 (0-based).
    FxName(usize),
    FxSub,
    Beat,
    Playing,
    Tempo,
    KitName,
    Browse,
    Browsing,
    Kit(KitAction),
    Compare,
    Status,
    Panic,
}

/// Every address, in the order of the address space. The transport's beat comes before
/// `playing`, so that a client has it when the transport starts.
fn kinds() -> Vec<Kind> {
    let infos = params::host_param_infos();
    let mut v: Vec<Kind> = infos.iter().copied().map(Kind::Param).collect();
    v.extend(infos.iter().copied().filter(|i| i.is_mod_target()).map(Kind::Values));
    v.extend((0..NUM_FADERS).map(Kind::Fader));
    v.extend((0..2).map(Kind::Switch));
    v.push(Kind::Midi);
    v.extend((0..7).map(Kind::Hits));
    v.extend((0..7).map(Kind::Mute));
    v.extend((0..7).map(Kind::Lfo));
    v.push(Kind::LfoReset);
    v.extend((0..4).map(Kind::FxName));
    v.push(Kind::FxSub);
    v.extend([Kind::Beat, Kind::Playing, Kind::Tempo]);
    v.push(Kind::KitName);
    v.push(Kind::Browse);
    v.push(Kind::Browsing);
    v.extend([KitAction::Load, KitAction::Init, KitAction::Save].map(Kind::Kit));
    v.extend([Kind::Compare, Kind::Status, Kind::Panic]);
    v
}

/// The FX type's own type parameter (distortion type, ring modulator waveform, compressor ratio,
/// delay type), if the type has one: the fourth entry of its FX page when that is not P1 to P4.
pub fn fx_sub_param(fx_type: u16) -> Option<u16> {
    let page = lxr_params::FX_PAGE.get(fx_type as usize)?;
    page.get(3).map(|(p, _)| *p).filter(|p| !FX_PARAMS.contains(p))
}

/// The highest value of any FX type's own type parameter.
fn fx_sub_max() -> i32 {
    (0..lxr_params::FX_PAGE.len() as u16)
        .filter_map(fx_sub_param)
        .filter_map(lxr_params::info)
        .map(|i| i.max as i32)
        .max()
        .unwrap_or(0)
}

/// The name of FX parameter `k` (0..4) for an FX type, as the editor's FX page shows it.
pub fn fx_name(fx_type: u16, k: usize) -> String {
    lxr_params::fx_param_text(fx_type.min(255) as u8, FX_PARAMS[k])
        .map(|t| format!("FX {}", lxr_params::text_names(t).2.trim()))
        .unwrap_or_else(|| format!("FX P{}", k + 1))
}

/// The values a modulation destination may take: "off" and the targets of its destination voice
/// (as the editor and the firmware's menu offer), and the current value.
pub fn mod_values(info: &ParamInfo, get: impl Fn(u16) -> u16) -> Vec<u16> {
    use lxr_params::open_hat::{oh_index, oh_k, OH_MAX_TARGET, OH_TARGET_BASE, OH_VOICE_LFO};
    let oh = oh_k(info.index).is_some();
    let voice = if oh && info.dtype == Dtype::TargetSelectionLfo {
        get(oh_index(OH_VOICE_LFO)).clamp(1, 7) as u8 - 1
    } else if oh {
        6
    } else if info.dtype == Dtype::TargetSelectionLfo {
        get(par::VOICE_LFO1 + (info.index - par::TARGET_LFO1)).clamp(1, 6) as u8 - 1
    } else {
        info.voice.number().unwrap_or(0)
    };
    let (lo, hi) = if voice == 6 { (OH_TARGET_BASE, OH_MAX_TARGET) } else { MOD_TARGET_VOICE_OFFSETS[voice.min(5) as usize] };
    let mut v: Vec<u16> = std::iter::once(0).chain(lo as u16..=hi as u16).collect();
    let cur = get(info.index);
    if !v.contains(&cur) {
        v.push(cur);
        v.sort_unstable();
    }
    v
}

fn node(k: Kind) -> Node {
    match k {
        Kind::Param(i) => Node::int(format!("{ROOT}/param/{}", i.id), i.min as i32, i.max as i32)
            .description(i.name)
            .tag(params::group_of(i))
            .text(),
        Kind::Values(i) => Node::string(format!("{ROOT}/param/{}/values", i.id))
            .description(format!("{} values", i.name)),
        Kind::Fader(f) => Node::int(format!("{ROOT}/fader/{}", f + 1), 0, FADER_MAX)
            .description(FADER_NAMES[f])
            .tag("Faders")
            .text(),
        Kind::Switch(s) => {
            Node::bool(format!("{ROOT}/switch/{}", SWITCH_NAMES[s])).description(SWITCH_LABELS[s]).tag("Switches")
        }
        Kind::Midi => Node::midi(format!("{ROOT}/midi"))
            .description("A MIDI message for the engine, as host MIDI (notes play the voices)"),
        Kind::Hits(t) => Node::int(format!("{ROOT}/hits/{}", t + 1), 0, HITS_WRAP as i32 - 1)
            .read_only()
            .description(format!("{} Hits", TRACK_NAMES[t]))
            .tag("Tracks"),
        Kind::Mute(t) => Node::bool(format!("{ROOT}/mute/{}", t + 1)).description(format!("{} Mute", TRACK_NAMES[t])).tag("Tracks"),
        Kind::Lfo(t) => Node::float(format!("{ROOT}/lfo/{}", t + 1), 0.0, 1.0)
            .read_only()
            .description(format!("{} LFO Output", TRACK_NAMES[t]))
            .tag("LFO"),
        Kind::LfoReset => write_only(Node::bool(format!("{ROOT}/lfo/reset"))).description("LFO Reset").tag("LFO"),
        Kind::FxName(k) => Node::string(format!("{ROOT}/fx/p{}/name", k + 1)).description(format!("FX P{} Name", k + 1)),
        Kind::FxSub => Node::int(format!("{ROOT}/fx/sub"), 0, fx_sub_max()).description("FX Sub-type").tag("FX").text(),
        Kind::Beat => Node::float(format!("{ROOT}/transport/beat"), 0.0, 1.0e6)
            .read_only()
            .description("Transport Start Beat")
            .tag("Transport"),
        Kind::Playing => Node::bool(format!("{ROOT}/transport/playing"))
            .read_only()
            .description("Transport Playing")
            .tag("Transport"),
        Kind::Tempo => Node::float(format!("{ROOT}/transport/tempo"), 0.0, 999.0)
            .read_only()
            .unit("bpm")
            .description("Transport Tempo")
            .tag("Transport"),
        Kind::KitName => Node::string(format!("{ROOT}/kit/name")).description("Kit name"),
        Kind::Browse => Node::int(format!("{ROOT}/kit/browse"), 0, 1023).description("Kit Browse").tag("Kit").text(),
        Kind::Browsing => Node::bool(format!("{ROOT}/kit/browsing")).read_only().description("Kit Browsing").tag("Kit"),
        Kind::Kit(a) => {
            let (n, d) = match a {
                KitAction::Load => ("load", "Kit Load"),
                KitAction::Init => ("init", "Kit Init"),
                KitAction::Save => ("save", "Kit Save"),
            };
            write_only(Node::bool(format!("{ROOT}/kit/{n}"))).description(d).tag("Kit")
        }
        Kind::Compare => Node::bool(format!("{ROOT}/kit/compare")).description("Kit Compare").tag("Kit"),
        Kind::Status => Node::string(format!("{ROOT}/kit/status")).description("Kit Status"),
        Kind::Panic => write_only(Node::bool(format!("{ROOT}/panic"))).description("Panic"),
    }
}

fn write_only(n: Node) -> Node {
    n.access(oscq::Access::Write)
}

/// What kit browsing shows.
#[derive(Clone, Debug, PartialEq)]
enum Browsed {
    Slot(usize),
    File(PathBuf),
}

/// State of the kit actions.
#[derive(Default)]
struct KitState {
    browsed: Option<Browsed>,
    /// The edited kit while compare plays the saved one.
    stash: Option<Kit>,
    status: String,
    /// The place of the kit browsing shows.
    pos: i32,
}

/// VDrum as an OSCQuery application.
pub struct Vdrum {
    shared: Arc<Shared>,
    kinds: Vec<Kind>,
    kit: Mutex<KitState>,
}

impl Vdrum {
    pub fn new(shared: Arc<Shared>) -> Vdrum {
        Vdrum { shared, kinds: kinds(), kit: Mutex::new(KitState::default()) }
    }

    /// The command for a value a client set; `None` for read-only addresses and the kit
    /// actions.
    fn command(&self, index: usize, v: Value) -> Option<Command> {
        let sh = &*self.shared;
        Some(match (self.kinds.get(index)?, v) {
            (Kind::Param(i), Value::Int(x)) => Command::Param(i.index, x as u16),
            (Kind::Fader(f), Value::Int(x)) => Command::Fader(*f as u8, x),
            (Kind::Switch(s), Value::Bool(b)) => Command::Switch(*s as u8, b),
            (Kind::Midi, Value::Midi(m)) => Command::Midi(m),
            (Kind::Mute(t), Value::Bool(b)) => Command::Mute(*t as u8, b),
            (Kind::LfoReset, Value::Bool(true)) => Command::LfoReset,
            (Kind::Panic, Value::Bool(true)) => Command::Panic,
            (Kind::FxSub, Value::Int(x)) => {
                let p = fx_sub_param(sh.value(par::FX_TYPE))?;
                let i = lxr_params::info(p)?;
                Command::Param(p, (x.max(0) as u16).clamp(i.min, i.max))
            }
            _ => return None,
        })
    }

    fn state(&self) -> std::sync::MutexGuard<'_, KitState> {
        self.kit.lock().unwrap_or_else(|e| e.into_inner())
    }

    fn load(&self, kit: &Kit) {
        self.shared.set_name(&kit.name);
        let _ = self.shared.commands.push(Command::LoadKit(kit.to_file_bytes()));
    }

    /// A kit action; returns the new status text.
    fn kit_action(&self, k: Kind, v: &Value) -> Option<String> {
        let sh = &*self.shared;
        let mut st = self.state();
        Some(match (k, v) {
            (Kind::Browse, Value::Int(x)) if *x != st.pos => {
                let dir = (x - st.pos).signum();
                let lib = sh.library.lock().ok().filter(|l| !l.is_empty()).map(|l| {
                    let from = match &st.browsed {
                        Some(Browsed::Slot(s)) => Some(*s),
                        _ => l.selected,
                    };
                    l.step(from, dir).map(|s| (s, l.label(s)))
                });
                match lib {
                    Some(Some((s, label))) => {
                        st.browsed = Some(Browsed::Slot(s));
                        st.pos = s as i32;
                        format!("Kit {label} · Play loads")
                    }
                    Some(None) => "no kits".into(),
                    None => {
                        let path = match &st.browsed {
                            Some(Browsed::File(p)) => Some(p.clone()),
                            _ => sh.kit_file.lock().ok().and_then(|f| f.path.clone()),
                        };
                        let Some(cur) = path else { return Some("no kit folder: load a kit in the editor".into()) };
                        let folder = cur.parent().map(Path::to_path_buf).unwrap_or_default();
                        let files = library::snd_files(&folder);
                        if files.is_empty() {
                            return Some(format!("no kits in {}", folder.display()));
                        }
                        let n = files.len() as i32;
                        let i = match files.iter().position(|f| *f == cur) {
                            Some(i) => (i as i32 + dir).rem_euclid(n),
                            None => 0,
                        } as usize;
                        let label = file_label(&files[i]);
                        st.browsed = Some(Browsed::File(files[i].clone()));
                        st.pos = i as i32;
                        format!("{label} · Play loads")
                    }
                }
            }
            (Kind::Kit(KitAction::Load), Value::Bool(true)) => match st.browsed.take() {
                Some(Browsed::Slot(s)) => match sh.load_library_slot(s) {
                    Some(label) => format!("Kit loaded: {label}"),
                    None => format!("Kit not loaded: {s:02} is empty"),
                },
                Some(Browsed::File(p)) => match std::fs::read(&p).map_err(|e| e.to_string()).and_then(|b| Kit::from_file_bytes(&b).map_err(|e| e.to_string())) {
                    Ok(kit) => {
                        sh.deselect_library();
                        sh.set_kit_file(Some(p.clone()), kit.to_file_bytes().to_vec());
                        self.load(&kit);
                        format!("Kit loaded: {}", file_label(&p))
                    }
                    Err(e) => format!("Kit not loaded: {e}"),
                },
                None => "nothing browsed".into(),
            },
            (Kind::Kit(KitAction::Init), Value::Bool(true)) => {
                let k = Kit::init();
                sh.deselect_library();
                self.load(&k);
                "Init kit loaded".into()
            }
            (Kind::Kit(KitAction::Save), Value::Bool(true)) => {
                if let Some(s) = sh.library_selected() {
                    match sh.store_in_library(s) {
                        Some(label) => format!("Kit saved: library {label}"),
                        None => "Kit not saved".into(),
                    }
                } else {
                    let path = sh.kit_file.lock().ok().and_then(|f| f.path.clone());
                    match path {
                        None => "Kit not saved: save once in the editor".into(),
                        Some(p) => {
                            let bytes = sh.kit().to_file_bytes();
                            match std::fs::write(&p, bytes) {
                                Ok(()) => {
                                    sh.set_kit_file(Some(p.clone()), bytes.to_vec());
                                    format!("Kit saved: {}", file_label(&p))
                                }
                                Err(e) => format!("Kit not saved: {e}"),
                            }
                        }
                    }
                }
            }
            (Kind::Compare, Value::Bool(true)) if st.stash.is_none() => {
                let saved = sh.kit_file.lock().ok().and_then(|f| f.saved.clone());
                match saved.and_then(|b| Kit::from_file_bytes(&b).ok()) {
                    Some(k) => {
                        st.stash = Some(sh.kit());
                        self.load(&k);
                        "Compare: saved kit".into()
                    }
                    None => "Compare: no saved kit, load or save a kit first".into(),
                }
            }
            (Kind::Compare, Value::Bool(false)) => match st.stash.take() {
                Some(k) => {
                    self.load(&k);
                    "Compare: edited kit".into()
                }
                None => return None,
            },
            _ => return None,
        })
    }
}

fn file_label(p: &Path) -> String {
    p.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default()
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
        let ld = Ordering::Relaxed;
        Some(match self.kinds.get(index)? {
            Kind::Param(i) => Value::Int(sh.value(i.index) as i32),
            Kind::Values(i) => {
                let v = mod_values(i, |p| sh.value(p));
                Value::String(v.iter().map(u16::to_string).collect::<Vec<_>>().join(" "))
            }
            Kind::Fader(f) => Value::Int(sh.fader_values[*f].load(ld)),
            Kind::Switch(0) => Value::Bool(sh.sum_aux_on.load(ld)),
            Kind::Switch(_) => Value::Bool(sh.hw_clock_on.load(ld)),
            Kind::Hits(t) => Value::Int((sh.pad_hits[*t].load(ld) % HITS_WRAP) as i32),
            Kind::Mute(t) => Value::Bool(sh.mutes.load(ld) >> t & 1 != 0),
            Kind::Lfo(t) => Value::Float((sh.lfo[*t].load(ld).clamp(0.0, 1.0) * 100.0).round() / 100.0),
            Kind::FxName(k) => Value::String(fx_name(sh.value(par::FX_TYPE), *k)),
            Kind::FxSub => Value::Int(fx_sub_param(sh.value(par::FX_TYPE)).map_or(0, |p| sh.value(p) as i32)),
            Kind::Beat => Value::Float(sh.transport_start_beat.load(ld)),
            Kind::Playing => Value::Bool(sh.transport_playing.load(ld)),
            Kind::Tempo => Value::Float(sh.transport_tempo.load(ld)),
            Kind::KitName => Value::String(kit_name(sh)),
            Kind::Browse => Value::Int(self.state().pos),
            Kind::Browsing => Value::Bool(self.state().browsed.is_some()),
            Kind::Compare => Value::Bool(self.state().stash.is_some()),
            Kind::Status => Value::String(self.state().status.clone()),
            Kind::Midi | Kind::LfoReset | Kind::Kit(_) | Kind::Panic => return None,
        })
    }

    fn text(&self, index: usize, v: &Value) -> Option<String> {
        let Value::Int(x) = v else { return None };
        let sh = &*self.shared;
        match self.kinds.get(index)? {
            Kind::Param(i) => Some(value_text(i, *x as u16)),
            Kind::Fader(_) => Some(fader_text(*x)),
            Kind::FxSub => Some(match fx_sub_param(sh.value(par::FX_TYPE)).and_then(lxr_params::info) {
                Some(i) => format!("{} {}", i.name, value_text(i, *x as u16)),
                None => "none for this FX".into(),
            }),
            Kind::Browse => {
                let st = self.state();
                Some(match &st.browsed {
                    Some(_) => st.status.clone(),
                    None => "Shift + Page: browse".into(),
                })
            }
            _ => None,
        }
    }

    fn set(&self, index: usize, v: Value) {
        let Some(&k) = self.kinds.get(index) else { return };
        if !matches!(k, Kind::Browse | Kind::Kit(KitAction::Load)) {
            self.state().browsed = None;
        }
        if let Some(status) = self.kit_action(k, &v) {
            self.state().status = status;
            return;
        }
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
    fn the_server_starts() {
        let cfg = oscq::Config::new("VDrum test").mdns(false);
        let server = oscq::Server::start(Vdrum::new(Arc::new(Shared::new())), cfg);
        assert!(server.is_ok(), "{:?}", server.err());
    }

    #[test]
    fn address_space() {
        let app = Vdrum::new(Arc::new(Shared::new()));
        let nodes = app.nodes();
        let infos = params::host_param_infos();
        let targets = infos.iter().filter(|i| i.is_mod_target()).count();
        assert!(targets > 0);
        // params, values, faders, switches, midi, hits, mutes, lfos, lfo reset, fx names, fx sub,
        // transport, kit name, browse, browsing, load/init/save, compare, status, panic
        assert_eq!(nodes.len(), infos.len() + targets + NUM_FADERS + 2 + 1 + 7 + 7 + 7 + 1 + 4 + 1 + 3 + 1 + 1 + 1 + 3 + 3);
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
        assert!(matches!(cmd("/vdrum/mute/3", OscType::Bool(true)), Some(Command::Mute(2, true))));
        assert!(matches!(cmd("/vdrum/lfo/reset", OscType::Bool(true)), Some(Command::LfoReset)));
        assert!(cmd("/vdrum/lfo/reset", OscType::Bool(false)).is_none());
        assert!(matches!(cmd("/vdrum/panic", OscType::Bool(true)), Some(Command::Panic)));
    }

    #[test]
    fn state_reads_back() {
        let shared = Arc::new(Shared::new());
        let app = Vdrum::new(shared.clone());
        let v = |path: &str| app.value(index(&app, path));
        shared.pad_hits[1].store(HITS_WRAP + 3, Ordering::Relaxed);
        assert_eq!(v("/vdrum/hits/2"), Some(Value::Int(3)));
        shared.mutes.store(0b100, Ordering::Relaxed);
        assert_eq!(v("/vdrum/mute/3"), Some(Value::Bool(true)));
        assert_eq!(v("/vdrum/mute/1"), Some(Value::Bool(false)));
        shared.lfo[0].store(0.123, Ordering::Relaxed);
        assert_eq!(v("/vdrum/lfo/1"), Some(Value::Float(0.12)));
        shared.transport_playing.store(true, Ordering::Relaxed);
        assert_eq!(v("/vdrum/transport/playing"), Some(Value::Bool(true)));
        // A beat comes before playing in the address space.
        assert!(index(&app, "/vdrum/transport/beat") < index(&app, "/vdrum/transport/playing"));
    }

    #[test]
    fn fx_names_and_sub_type_follow_the_fx_type() {
        let shared = Arc::new(Shared::new());
        let app = Vdrum::new(shared.clone());
        let i = index(&app, "/vdrum/fx/p1/name");
        let with_sub = (0..lxr_params::FX_PAGE.len() as u16).find(|t| fx_sub_param(*t).is_some()).unwrap();
        let without = (0..lxr_params::FX_PAGE.len() as u16).find(|t| fx_sub_param(*t).is_none()).unwrap();
        shared.values[par::FX_TYPE as usize].store(without, Ordering::Relaxed);
        let sub = index(&app, "/vdrum/fx/sub");
        assert_eq!(app.text(sub, &Value::Int(0)).as_deref(), Some("none for this FX"));
        assert!(app.command(sub, Value::Int(1)).is_none());
        shared.values[par::FX_TYPE as usize].store(with_sub, Ordering::Relaxed);
        let p = fx_sub_param(with_sub).unwrap();
        assert!(matches!(app.command(sub, Value::Int(1)), Some(Command::Param(q, 1)) if q == p));
        let Some(Value::String(name)) = app.value(i) else { panic!() };
        assert!(name.starts_with("FX "), "{name}");
    }

    #[test]
    fn mod_destinations_offer_their_voice_targets() {
        let shared = Arc::new(Shared::new());
        let app = Vdrum::new(shared.clone());
        let info = lxr_params::by_id("target_lfo1").unwrap();
        shared.values[par::VOICE_LFO1 as usize].store(2, Ordering::Relaxed);
        let Some(Value::String(v)) = app.value(index(&app, "/vdrum/param/target_lfo1/values")) else { panic!() };
        let (lo, hi) = MOD_TARGET_VOICE_OFFSETS[1];
        let want: Vec<String> = std::iter::once(0u16).chain(lo as u16..=hi as u16).map(|x| x.to_string()).collect();
        assert_eq!(v, want.join(" "));
        assert!(info.is_mod_target());
    }

    #[test]
    fn kit_actions_report_their_status() {
        let shared = Arc::new(Shared::new());
        let app = Vdrum::new(shared.clone());
        let set = |path: &str, v: Value| app.set(index(&app, path), v);
        let status = || match app.value(index(&app, "/vdrum/kit/status")) {
            Some(Value::String(s)) => s,
            _ => panic!(),
        };
        set("/vdrum/kit/browse", Value::Int(1));
        assert!(status().starts_with("no kit folder"), "{}", status());
        set("/vdrum/kit/compare", Value::Bool(true));
        assert!(status().contains("no saved kit"));
        // A library kit: browse shows it, load loads it.
        let k = Kit::init();
        shared.library.lock().unwrap().slots[5] = Some(library::Entry { kit: k.clone(), source: String::new() });
        set("/vdrum/kit/browse", Value::Int(1));
        assert!(status().starts_with("Kit 05"), "{}", status());
        assert_eq!(app.value(index(&app, "/vdrum/kit/browse")), Some(Value::Int(5)));
        assert_eq!(app.value(index(&app, "/vdrum/kit/browsing")), Some(Value::Bool(true)));
        while shared.commands.pop().is_some() {}
        set("/vdrum/kit/load", Value::Bool(true));
        assert!(status().starts_with("Kit loaded"), "{}", status());
        assert_eq!(app.value(index(&app, "/vdrum/kit/browsing")), Some(Value::Bool(false)));
        assert!(matches!(shared.commands.pop(), Some(Command::LoadKit(_))));
        // Compare plays the kit as last loaded, then the edited one again.
        set("/vdrum/kit/compare", Value::Bool(true));
        assert_eq!(app.value(index(&app, "/vdrum/kit/compare")), Some(Value::Bool(true)));
        set("/vdrum/kit/compare", Value::Bool(false));
        assert_eq!(status(), "Compare: edited kit");
        set("/vdrum/kit/save", Value::Bool(true));
        assert!(status().starts_with("Kit saved: library 05"), "{}", status());
    }
}
