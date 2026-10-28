// SPDX-License-Identifier: GPL-3.0-only
//! The egui editor: one card per voice with all of that voice's sections, and a column with the
//! FX bus, performance, MIDI and output settings.
//!
//! Interaction: drag a value bar horizontally or vertically (Shift: fine), double-click resets to
//! the default, list parameters open their choices on click, right-click opens MIDI learn. Values
//! that differ from their default are marked with an amber stripe.

use crate::engine::track_trigger_note;
use crate::params::{fader_text, is_list, plugin_default, value_text, Command, Shared, VDrumParams, FADER_MAX, N};
use lxr_params::kit::Kit;
use lxr_params::{fx_param_text, par, Dtype, ParamInfo, Section, Voice, MOD_TARGET_VOICE_OFFSETS, VOICE_PAGES};
use nih_plug::prelude::*;
use nih_plug_egui::egui::{
    self, pos2, vec2, Align2, Color32, CornerRadius, FontId, Frame, Margin, PopupCloseBehavior, Rect, RichText,
    ScrollArea, Sense, Stroke, StrokeKind, Ui,
};
use nih_plug_egui::resizable_window::ResizableWindow;
use nih_plug_egui::{create_egui_editor, EguiState};
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

/// Width of a voice card's rows.
const ROW_W: f32 = 232.0;
/// Width of the global column's rows.
const GLOBAL_W: f32 = 290.0;
const ROW_H: f32 = 18.0;
const ROW_GAP: f32 = 1.0;
const CARD_PAD: f32 = 6.0;
const CARD_GAP: f32 = 6.0;
/// Default window size: everything fits at 100 % (about 1850 × 960 logical pixels).
pub const DEFAULT_SIZE: (u32, u32) = (1856, 952);

pub const TRACK_NAMES: [&str; 7] = ["Drum 1", "Drum 2", "Drum 3", "Snare", "Cymbal", "Closed hi-hat", "Open hi-hat"];
pub const PAD_LABELS: [&str; 7] = ["D1", "D2", "D3", "SN", "CY", "CH", "OH"];
const MIDI_CHAN: [u16; 7] =
    [par::MIDI_CHAN_1, par::MIDI_CHAN_2, par::MIDI_CHAN_3, par::MIDI_CHAN_4, par::MIDI_CHAN_5, par::MIDI_CHAN_6, par::MIDI_CHAN_7];
const MIDI_NOTE: [u16; 7] =
    [par::MIDI_NOTE1, par::MIDI_NOTE2, par::MIDI_NOTE3, par::MIDI_NOTE4, par::MIDI_NOTE5, par::MIDI_NOTE6, par::MIDI_NOTE7];

// Colours
const ACCENT: Color32 = Color32::from_rgb(120, 170, 230);
const BAR: Color32 = Color32::from_rgb(44, 86, 138);
const CHANGED: Color32 = Color32::from_rgb(240, 170, 60);
const CARD_BG: Color32 = Color32::from_rgb(34, 36, 41);
const ROW_BG: Color32 = Color32::from_rgb(48, 51, 58);
const ROW_HOVER: Color32 = Color32::from_rgb(62, 66, 76);
const LABEL: Color32 = Color32::from_rgb(170, 175, 185);
const LABEL_CHANGED: Color32 = Color32::from_rgb(232, 232, 236);
const VALUE: Color32 = Color32::from_rgb(240, 240, 244);

/// One row of a card.
#[derive(Clone, Debug, PartialEq)]
pub enum Item {
    /// A parameter (index) with its row label.
    Param(u16, String),
    /// Voice fader `i` (plugin parameter, not part of the kit).
    Fader(usize),
}

/// A titled group of rows.
#[derive(Clone, Debug)]
pub struct Group {
    pub title: &'static str,
    pub items: Vec<Item>,
}

/// Where every parameter is shown.
pub struct Layout {
    /// The groups of each of the six voice cards.
    pub voices: Vec<Vec<Group>>,
    pub fx: Vec<u16>,
    pub performance: Vec<u16>,
}

/// Groups of a voice card, in order: (title, matcher on the menu section / category).
fn group_of(info: &ParamInfo) -> usize {
    match info.section {
        Section::Oscillator => 0,
        Section::Fm => 1,
        Section::AmpEg => 2,
        Section::Modulation if info.category.trim() == "Velocity" => 7,
        Section::Modulation => 3,
        Section::Filter => 4,
        Section::Transient => 5,
        Section::Lfo => 6,
        _ => 8,
    }
}

fn group_titles(voice: usize) -> [&'static str; 9] {
    let fm = if voice >= 4 { "Mod oscillators (FM)" } else { "FM" };
    ["Oscillator", fm, "Amp EG", "Mod EG (pitch)", "Filter", "Transient", "LFO", "Velocity modulation", "Mix"]
}

/// Label of a parameter inside its voice card: the readable name without the voice and the
/// section words, spelled out.
pub fn row_label(info: &ParamInfo) -> String {
    let mut s = info.name.strip_prefix(info.voice.label()).unwrap_or(info.name).trim_start();
    for prefix in ["Oscillator ", "Amp EG ", "Mod EG ", "Modulation ", "FM ", "Transient ", "Filter ", "LFO ", "Voice ", "Velocity ", "Mod Osc "] {
        if let Some(r) = s.strip_prefix(prefix) {
            if !r.trim().is_empty() {
                s = r;
                break;
            }
        }
    }
    match s {
        "Volume Mod" => "Volume Mod (on/off)".to_string(),
        "Pan" | "Panning" => "Pan".to_string(),
        _ => s.to_string(),
    }
}

impl Layout {
    pub fn build() -> Layout {
        let exposed = |p: u16| lxr_params::info(p).is_some_and(|i| i.kind == lxr_params::ParamKind::Sound);
        let mut voices = Vec::new();
        for v in 0..6 {
            let mut params: Vec<u16> = Vec::new();
            let pages: &[usize] = if v == 5 { &[5, 6] } else { &[v] };
            for s in 0..Section::VOICE_SECTIONS.len() {
                for &page in pages {
                    for (_, p) in VOICE_PAGES[page][s] {
                        if exposed(*p) && !params.contains(p) {
                            params.push(*p);
                        }
                    }
                }
            }
            let titles = group_titles(v);
            let mut groups: Vec<Group> = titles.iter().map(|t| Group { title: t, items: Vec::new() }).collect();
            for p in params {
                let info = lxr_params::info(p).unwrap();
                groups[group_of(info)].items.push(Item::Param(p, row_label(info)));
            }
            groups[8].items.push(Item::Fader(v));
            groups.retain(|g| !g.items.is_empty());
            voices.push(groups);
        }
        let placed: Vec<u16> = voices
            .iter()
            .flatten()
            .flat_map(|g| g.items.iter())
            .filter_map(|i| if let Item::Param(p, _) = i { Some(*p) } else { None })
            .collect();
        let mut fx = Vec::new();
        let mut performance = Vec::new();
        for info in lxr_params::sound_params() {
            if info.voice == Voice::Fx {
                fx.push(info.index);
            } else if !placed.contains(&info.index) {
                performance.push(info.index);
            }
        }
        Layout { voices, fx, performance }
    }
}

enum FileReply {
    Load(PathBuf),
    Save(PathBuf),
}

struct Data {
    layout: Layout,
    name_buf: String,
    pad_seen: [u32; 7],
    pad_lit: [f64; 7],
    file_reply: Arc<Mutex<Option<FileReply>>>,
    status: String,
}

pub fn create(params: Arc<VDrumParams>, shared: Arc<Shared>) -> Option<Box<dyn Editor>> {
    let egui_state = params.editor_state.clone();
    let data = Data {
        layout: Layout::build(),
        name_buf: String::new(),
        pad_seen: [0; 7],
        pad_lit: [-10.0; 7],
        file_reply: Arc::new(Mutex::new(None)),
        status: String::new(),
    };
    create_egui_editor(
        params.editor_state.clone(),
        data,
        |ctx, _| {
            let mut v = egui::Visuals::dark();
            v.panel_fill = Color32::from_rgb(24, 25, 29);
            v.window_fill = Color32::from_rgb(24, 25, 29);
            ctx.set_visuals(v);
            ctx.style_mut(|s| {
                s.spacing.item_spacing = vec2(4.0, ROW_GAP);
                s.spacing.button_padding = vec2(6.0, 2.0);
                s.interaction.tooltip_delay = 0.35;
            });
        },
        move |ctx, setter, data| update(ctx, setter, data, &params, &shared, &egui_state),
    )
}

fn update(ctx: &egui::Context, setter: &ParamSetter, data: &mut Data, params: &VDrumParams, shared: &Shared, egui_state: &EguiState) {
    // Values the engine changed itself (MIDI, kit loads) go to the host parameters.
    for (slot, info) in params.infos.iter().enumerate() {
        let i = info.index as usize;
        if shared.dirty[i].swap(false, Ordering::Relaxed) {
            let v = shared.values[i].load(Ordering::Relaxed) as i32;
            let p = &params.ints[slot];
            if p.value() != v {
                setter.begin_set_parameter(p);
                setter.set_parameter(p, v);
                setter.end_set_parameter(p);
            }
        }
    }

    let zoom = params.ui_scale.load(Ordering::Relaxed);
    if (ctx.zoom_factor() - zoom).abs() > 1e-3 {
        ctx.set_zoom_factor(zoom);
    }

    let reply = data.file_reply.lock().ok().and_then(|mut r| r.take());
    match reply {
        Some(FileReply::Load(path)) => match std::fs::read(&path) {
            Ok(bytes) => match Kit::from_file_bytes(&bytes) {
                Ok(kit) => {
                    shared.set_name(&kit.name);
                    let _ = shared.commands.push(Command::LoadKit(kit.to_file_bytes()));
                    data.status = format!("Loaded {}", path.display());
                }
                Err(e) => data.status = format!("{}: {e}", path.display()),
            },
            Err(e) => data.status = format!("{}: {e}", path.display()),
        },
        Some(FileReply::Save(path)) => {
            data.status = match std::fs::write(&path, params.kit().to_file_bytes()) {
                Ok(()) => format!("Saved {}", path.display()),
                Err(e) => format!("{}: {e}", path.display()),
            }
        }
        None => {}
    }

    ResizableWindow::new("vdrum-window").min_size(vec2(480.0, 320.0)).show(ctx, egui_state, |ui| {
        top_bar(ui, data, params, shared);
        ui.add_space(4.0);
        ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = CARD_GAP;
                for v in 0..6 {
                    voice_card(ui, setter, data, params, shared, v);
                }
                global_column(ui, setter, data, params, shared);
            });
        });
    });
    ctx.request_repaint_after(std::time::Duration::from_millis(33));
}

fn spawn_dialog(reply: Arc<Mutex<Option<FileReply>>>, save: Option<String>) {
    std::thread::spawn(move || {
        let d = rfd::FileDialog::new().add_filter("Kit (.snd)", &["snd", "SND"]);
        let r = match save {
            Some(name) => d.set_file_name(name).save_file().map(FileReply::Save),
            None => d.pick_file().map(FileReply::Load),
        };
        if let (Some(r), Ok(mut slot)) = (r, reply.lock()) {
            *slot = Some(r);
        }
    });
}

fn top_bar(ui: &mut Ui, data: &mut Data, params: &VDrumParams, shared: &Shared) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        ui.label(RichText::new("VDrum").size(18.0).strong().color(Color32::WHITE));
        ui.label(RichText::new("LXR-02 engine").size(11.0).color(LABEL));
        ui.separator();
        ui.label(RichText::new("Kit").color(LABEL));
        let edit = ui.add(egui::TextEdit::singleline(&mut data.name_buf).char_limit(8).desired_width(80.0))
            .on_hover_text("Kit name (8 characters, saved in the .snd file)");
        if edit.changed() {
            let mut k = Kit::init();
            k.set_name(&data.name_buf);
            shared.set_name(&k.name);
        }
        if !edit.has_focus() {
            let k = Kit { name: shared.name_bytes(), values: [0; lxr_params::kit::KIT_PARAMS] };
            data.name_buf = k.name_str();
        }
        if ui.button("Load…").on_hover_text("Load a kit file (.snd)").clicked() {
            spawn_dialog(data.file_reply.clone(), None);
        }
        if ui.button("Save…").on_hover_text("Save the kit as a .snd file").clicked() {
            spawn_dialog(data.file_reply.clone(), Some(params.kit().file_name(1)));
        }
        if ui.button("Init").on_hover_text("Load the init kit").clicked() {
            let k = Kit::init();
            shared.set_name(&k.name);
            let _ = shared.commands.push(Command::LoadKit(k.to_file_bytes()));
        }
        let changed = params
            .infos
            .iter()
            .filter(|i| i.kind == lxr_params::ParamKind::Sound)
            .filter(|i| params.param(i.index).is_some_and(|p| p.value() as u16 != plugin_default(i)))
            .count();
        ui.label(RichText::new(format!("{changed} values differ from the init kit")).size(11.0).color(LABEL))
            .on_hover_text("Rows with an amber stripe differ from their default. Double-click a row to reset it.");
        ui.separator();
        let target = shared.learn_target.load(Ordering::Relaxed);
        if target >= 0 {
            let name = lxr_params::info(target as u16).map(|i| i.name).unwrap_or("?");
            ui.label(RichText::new(format!("MIDI learn: move a controller for {name}")).color(CHANGED));
            if ui.small_button("Cancel").clicked() {
                shared.learn_target.store(-1, Ordering::Relaxed);
            }
        } else if !data.status.is_empty() {
            ui.label(RichText::new(&data.status).size(11.0).color(LABEL));
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let z = params.ui_scale.load(Ordering::Relaxed);
            if ui.small_button("+").on_hover_text("Larger").clicked() {
                params.ui_scale.store((z + 0.1).min(3.0), Ordering::Relaxed);
            }
            if ui.small_button(format!("{:.0}%", z * 100.0)).on_hover_text("Reset to 100 %").clicked() {
                params.ui_scale.store(1.0, Ordering::Relaxed);
            }
            if ui.small_button("−").on_hover_text("Smaller").clicked() {
                params.ui_scale.store((z - 0.1).max(0.5), Ordering::Relaxed);
            }
            ui.label(RichText::new("Zoom").size(11.0).color(LABEL));
        });
    });
}

fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let l = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t) as u8;
    Color32::from_rgb(l(a.r(), b.r()), l(a.g(), b.g()), l(a.b(), b.b()))
}

fn note_text(note: u8) -> String {
    lxr_params::format::set_note_name(note).iter().map(|b| *b as char).collect::<String>().trim().to_string()
}

/// A trigger pad for track `t`.
fn pad(ui: &mut Ui, data: &mut Data, params: &VDrumParams, shared: &Shared, t: usize, size: egui::Vec2) {
    let now = ui.input(|i| i.time);
    let hits = shared.pad_hits[t].load(Ordering::Relaxed);
    if hits != data.pad_seen[t] {
        data.pad_seen[t] = hits;
        data.pad_lit[t] = now;
    }
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let glow = (1.0 - (now - data.pad_lit[t]) / 0.25).clamp(0.0, 1.0) as f32;
    let fill = lerp_color(Color32::from_rgb(56, 60, 70), Color32::from_rgb(240, 160, 40), glow);
    ui.painter().rect_filled(rect, 3.0, fill);
    if resp.hovered() {
        ui.painter().rect_stroke(rect, 3.0, Stroke::new(1.0_f32, Color32::from_gray(200)), StrokeKind::Inside);
    }
    ui.painter().text(rect.center(), Align2::CENTER_CENTER, PAD_LABELS[t], FontId::proportional(12.0), Color32::WHITE);
    let get = |p: u16| params.param(p).map(|x| x.value() as u16).unwrap_or(0);
    let (ch, note) = track_trigger_note(t, get);
    let resp = resp.on_hover_text(format!(
        "Trigger {}: note-on channel {}, note {} ({}). Set the note and channel in the MIDI section.",
        TRACK_NAMES[t],
        ch + 1,
        note,
        note_text(note)
    ));
    if resp.hovered() && ui.input(|i| i.pointer.primary_pressed()) {
        let _ = shared.commands.push(Command::Midi([0x90 | ch, note, 127]));
    }
}

fn card<R>(ui: &mut Ui, width: f32, add: impl FnOnce(&mut Ui) -> R) -> R {
    Frame::new()
        .fill(CARD_BG)
        .corner_radius(CornerRadius::same(5))
        .inner_margin(Margin::same(CARD_PAD as i8))
        .show(ui, |ui| {
            ui.set_width(width);
            ui.spacing_mut().item_spacing = vec2(4.0, ROW_GAP);
            add(ui)
        })
        .inner
}

fn voice_card(ui: &mut Ui, setter: &ParamSetter, data: &mut Data, params: &VDrumParams, shared: &Shared, v: usize) {
    card(ui, ROW_W, |ui| {
        ui.vertical(|ui| {
            // Header: name, trigger note(s), pads
            let tracks: &[usize] = if v == 5 { &[5, 6] } else if v == 4 { &[4] } else { std::slice::from_ref(&[0, 1, 2, 3][v]) };
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.set_width(ROW_W - 40.0 * tracks.len() as f32 - 4.0);
                    ui.label(RichText::new(Voice::VOICES[v].label()).size(16.0).strong().color(Color32::WHITE));
                    let get = |p: u16| params.param(p).map(|x| x.value() as u16).unwrap_or(0);
                    let notes: Vec<String> = tracks
                        .iter()
                        .map(|&t| {
                            let (ch, note) = track_trigger_note(t, get);
                            let n = get(MIDI_NOTE[t]);
                            let what = if n == 0 { "any note".to_string() } else { format!("{} ({note})", note_text(note)) };
                            if tracks.len() > 1 {
                                format!("{}: {what} ch {}", PAD_LABELS[t], ch + 1)
                            } else {
                                format!("{what}, channel {}", ch + 1)
                            }
                        })
                        .collect();
                    ui.label(RichText::new(notes.join("  ")).size(10.5).color(LABEL));
                });
                for &t in tracks {
                    pad(ui, data, params, shared, t, vec2(36.0, 30.0));
                }
            });
            for g in &data.layout.voices[v] {
                section_header(ui, g.title, ROW_W);
                for item in &g.items {
                    match item {
                        Item::Param(idx, label) => param_cell(ui, setter, params, shared, *idx, ROW_W, label, false),
                        Item::Fader(i) => fader_cell(ui, setter, params, *i, ROW_W),
                    }
                }
            }
        });
    });
}

fn section_header(ui: &mut Ui, title: &str, width: f32) {
    ui.add_space(3.0);
    let (rect, _) = ui.allocate_exact_size(vec2(width, 15.0), Sense::hover());
    let p = ui.painter();
    let g = p.layout_no_wrap(title.to_uppercase(), FontId::proportional(10.5), ACCENT);
    let y = rect.center().y;
    let tw = g.size().x;
    p.galley(pos2(rect.min.x, y - g.size().y / 2.0), g, ACCENT);
    p.line_segment([pos2(rect.min.x + tw + 6.0, y), pos2(rect.max.x, y)], Stroke::new(1.0_f32, Color32::from_rgb(60, 70, 88)));
}

fn global_column(ui: &mut Ui, setter: &ParamSetter, data: &mut Data, params: &VDrumParams, shared: &Shared) {
    card(ui, GLOBAL_W, |ui| {
        ui.vertical(|ui| {
            ui.label(RichText::new("FX bus & global").size(16.0).strong().color(Color32::WHITE));
            let fx_type = params.param(par::FX_TYPE).map(|p| p.value() as u8).unwrap_or(0);
            let fx_name = lxr_params::info(par::FX_TYPE).map(|i| value_text(i, fx_type as u16)).unwrap_or_default();
            ui.label(RichText::new(format!("Type {fx_name}: unused controls are dimmed")).size(10.5).color(LABEL));
            section_header(ui, "FX bus", GLOBAL_W);
            for &idx in &data.layout.fx {
                let text = fx_param_text(fx_type, idx);
                let label = match text {
                    Some(t) if (par::FX_P1..=par::FX_P4).contains(&idx) => {
                        format!("P{}: {}", idx - par::FX_P1 + 1, lxr_params::format::text_label(t))
                    }
                    _ => lxr_params::info(idx).map(|i| i.name.strip_prefix("FX ").unwrap_or(i.name).to_string()).unwrap_or_default(),
                };
                let relevant = text.is_some() || matches!(idx, par::FX_TYPE | par::FX_ROUTING | par::FX_DRY_WET);
                param_cell(ui, setter, params, shared, idx, GLOBAL_W, &label, !relevant);
            }
            if !data.layout.performance.is_empty() {
                section_header(ui, "Performance", GLOBAL_W);
                for &idx in &data.layout.performance.clone() {
                    let label = lxr_params::info(idx).map(|i| i.name.to_string()).unwrap_or_default();
                    param_cell(ui, setter, params, shared, idx, GLOBAL_W, &label, false);
                }
            }
            section_header(ui, "MIDI", GLOBAL_W);
            param_cell(ui, setter, params, shared, par::BPM, GLOBAL_W, "Tempo (BPM)", false);
            param_cell(ui, setter, params, shared, par::MIDI_CHAN_GLOBAL, GLOBAL_W, "Global channel", false);
            ui.horizontal(|ui| {
                let w = (GLOBAL_W - 92.0 - 8.0) / 2.0;
                for (text, width) in [("Track", 92.0), ("Channel", w), ("Trigger note", w)] {
                    let (r, _) = ui.allocate_exact_size(vec2(width, 14.0), Sense::hover());
                    ui.painter().text(r.left_center(), Align2::LEFT_CENTER, text, FontId::proportional(10.5), LABEL);
                }
            });
            for t in 0..7 {
                ui.horizontal(|ui| {
                    let (rect, _) = ui.allocate_exact_size(vec2(92.0, ROW_H), Sense::hover());
                    ui.painter().text(rect.left_center(), Align2::LEFT_CENTER, TRACK_NAMES[t], FontId::proportional(12.0), LABEL_CHANGED);
                    let w = (GLOBAL_W - 92.0 - 8.0) / 2.0;
                    param_cell(ui, setter, params, shared, MIDI_CHAN[t], w, "Ch", false);
                    param_cell(ui, setter, params, shared, MIDI_NOTE[t], w, "Note", false);
                });
            }
            ui.label(
                RichText::new("Plugin default: one General MIDI note per track on channel 1 (the hardware default is note C4 for all tracks).")
                    .size(10.0)
                    .color(LABEL),
            );
            section_header(ui, "Output", GLOBAL_W);
            meters(ui, shared);
            let has_aux = shared.has_aux.load(Ordering::Relaxed);
            let mut sum = params.sum_aux.value();
            let resp = ui.add_enabled(has_aux, egui::Checkbox::new(&mut sum, "Sum aux (DAC 2) into main"));
            if resp.changed() {
                setter.begin_set_parameter(&params.sum_aux);
                setter.set_parameter(&params.sum_aux, sum);
                setter.end_set_parameter(&params.sum_aux);
            }
            let mut hw = params.hardware_clock.value();
            let resp = ui.checkbox(&mut hw, "Hardware clock (44140.625 Hz)").on_hover_text(
                "Run the engine at the LXR-02's real sample clock and resample to the host rate. \
                 Off: the engine runs at 44100 Hz and its samples pass through unmodified at a 44100 Hz host rate.",
            );
            if resp.changed() {
                setter.begin_set_parameter(&params.hardware_clock);
                setter.set_parameter(&params.hardware_clock, hw);
                setter.end_set_parameter(&params.hardware_clock);
            }
            let rate = shared.host_rate.load(Ordering::Relaxed);
            let lat = shared.latency.load(Ordering::Relaxed);
            if rate > 0.0 {
                let mode = if hw {
                    "resampled from 44140.625 Hz"
                } else if (rate - 44100.0).abs() < 0.5 {
                    "engine rate, bit-exact"
                } else {
                    "resampled from 44100 Hz"
                };
                ui.label(RichText::new(format!("{rate:.0} Hz ({mode}), latency {lat} samples")).size(10.5).color(LABEL));
            }
            section_header(ui, "MIDI learn", GLOBAL_W);
            let n = shared.learn.iter().filter(|s| s.load(Ordering::Relaxed) != 0).count();
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("{n} controller(s) learned")).size(11.0).color(LABEL_CHANGED));
                if n > 0 && ui.small_button("Forget all").clicked() {
                    shared.forget_all();
                }
            });
            ui.label(RichText::new("Right-click a control to learn a CC. Tooltips show each control's CC/NRPN.").size(10.5).color(LABEL));
        });
    });
}

fn meters(ui: &mut Ui, shared: &Shared) {
    let names = ["Main L", "Main R", "Aux L", "Aux R"];
    for (c, name) in names.iter().enumerate() {
        let v = shared.meters[c].load(Ordering::Relaxed);
        let db = if v > 0.0 { 20.0 * v.log10() } else { -120.0 };
        let frac = ((db + 60.0) / 60.0).clamp(0.0, 1.0);
        let (rect, _) = ui.allocate_exact_size(vec2(GLOBAL_W, 13.0), Sense::hover());
        let p = ui.painter();
        p.rect_filled(rect, 2.0, Color32::from_gray(30));
        let mut fill = rect;
        fill.set_width(rect.width() * frac);
        let col = if db > -0.1 {
            Color32::from_rgb(230, 70, 60)
        } else if db > -12.0 {
            Color32::from_rgb(220, 190, 60)
        } else {
            Color32::from_rgb(80, 190, 110)
        };
        p.rect_filled(fill, 2.0, col);
        let txt = if db > -120.0 { format!("{name}  {db:.1} dBFS") } else { format!("{name}  -inf") };
        p.text(rect.left_center() + vec2(4.0, 0.0), Align2::LEFT_CENTER, txt, FontId::proportional(10.5), Color32::WHITE);
    }
}

/// The values a list parameter offers.
fn options(info: &ParamInfo, params: &VDrumParams) -> Vec<u16> {
    if info.is_mod_target() {
        let voice = if info.dtype == Dtype::TargetSelectionLfo {
            // The LFO's destination voice (PAR_VOICE_LFOx, 1..=6).
            let lfo = info.index - par::TARGET_LFO1;
            params.param(par::VOICE_LFO1 + lfo).map(|p| p.value().clamp(1, 6) as u8 - 1).unwrap_or(0)
        } else {
            info.voice.number().unwrap_or(0)
        };
        let (lo, hi) = MOD_TARGET_VOICE_OFFSETS[voice.min(5) as usize];
        let mut v: Vec<u16> = std::iter::once(0).chain(lo as u16..=hi as u16).collect();
        let cur = params.param(info.index).map(|p| p.value() as u16).unwrap_or(0);
        if !v.contains(&cur) {
            v.push(cur);
        }
        v
    } else {
        (info.min..=info.max).collect()
    }
}

fn midi_help(info: &ParamInfo, params: &VDrumParams) -> String {
    let m = &info.midi;
    let mut s = String::new();
    if let Some(cc) = m.cc {
        let rule = match m.cc_voice {
            v @ 1..=6 => {
                let ch = params.param(MIDI_CHAN[v as usize]).map(|p| p.value()).unwrap_or(0);
                format!("on the {} channel ({ch}) or the global channel", TRACK_NAMES[v as usize])
            }
            _ => "on any channel set for a track or as global channel".to_string(),
        };
        s += &format!("MIDI: CC {cc} {rule}");
    } else if let Some(n) = m.nrpn {
        s += &format!("MIDI: NRPN {n} (CC 99 = {}, CC 98 = {}, value on CC 6)", n >> 7, n & 127);
        if m.front_translates_nrpn {
            s += "; the value is a parameter number, translated to a modulation target";
        }
    } else {
        s += "MIDI: no CC/NRPN";
    }
    if (m.cc.is_some() || m.nrpn.is_some()) && !m.effective() {
        s += " (ignored by the firmware)";
    }
    s
}

/// The value as the LXR-02's display shows it in edit mode.
fn hw_text(info: &ParamInfo, v: u16) -> String {
    let s = lxr_params::format_value(info, v.min(255) as u8);
    let s = s.trim();
    if s.is_empty() {
        v.to_string()
    } else {
        s.to_string()
    }
}

/// Paints a row: background, value bar (or list marker), changed stripe, label and value.
#[allow(clippy::too_many_arguments)]
fn paint_row(ui: &Ui, rect: Rect, resp: &egui::Response, frac: Option<f32>, changed: bool, dim: bool, label: &str, value: &str) {
    let painter = ui.painter().with_clip_rect(rect.intersect(ui.clip_rect()));
    let bg = if resp.hovered() || resp.dragged() { ROW_HOVER } else { ROW_BG };
    painter.rect_filled(rect, 2.0, bg);
    match frac {
        Some(f) => {
            let mut r = rect;
            r.set_width(rect.width() * f.clamp(0.0, 1.0));
            painter.rect_filled(r, 2.0, BAR.gamma_multiply(if dim { 0.45 } else { 1.0 }));
        }
        None => {
            let tri = Rect::from_min_size(pos2(rect.max.x - 10.0, rect.center().y - 2.0), vec2(6.0, 4.0));
            painter.add(egui::Shape::convex_polygon(vec![tri.left_top(), tri.right_top(), tri.center_bottom()], LABEL, Stroke::NONE));
        }
    }
    if changed {
        painter.rect_filled(Rect::from_min_size(rect.min, vec2(3.0, rect.height())), 1.0, CHANGED);
    }
    let alpha = if dim { 105 } else { 255 };
    let col = |c: Color32| Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), alpha);
    let font = FontId::proportional(12.0);
    let vg = painter.layout_no_wrap(value.to_string(), font.clone(), col(VALUE));
    let right = rect.max.x - if frac.is_none() { 13.0 } else { 5.0 };
    // The value keeps at most 60 % of the row; the label gets the rest.
    let vw = vg.size().x.min(rect.width() * 0.6);
    let vpos = pos2(right - vw, rect.center().y - vg.size().y / 2.0);
    painter.with_clip_rect(Rect::from_min_max(pos2(vpos.x, rect.min.y), pos2(right, rect.max.y)).intersect(ui.clip_rect())).galley(vpos, vg, VALUE);
    let lg = painter.layout_no_wrap(label.to_string(), font, col(if changed { LABEL_CHANGED } else { LABEL }));
    let lclip = Rect::from_min_max(rect.min, pos2((vpos.x - 6.0).max(rect.min.x), rect.max.y));
    painter.with_clip_rect(lclip.intersect(ui.clip_rect())).galley(pos2(rect.min.x + 6.0, rect.center().y - lg.size().y / 2.0), lg, VALUE);
}

/// Drag handling for a value bar. Returns the new value when it changed.
fn drag_value(ui: &Ui, resp: &egui::Response, v: i32, min: i32, max: i32) -> Option<i32> {
    let id = resp.id;
    if resp.drag_started() {
        ui.data_mut(|d| d.insert_temp(id, v as f32));
    }
    if resp.dragged() {
        let d = resp.drag_delta();
        let fine = ui.input(|i| i.modifiers.shift);
        let span = (max - min) as f32;
        let rate = if fine { 0.1 } else { (span / 180.0).max(0.04) };
        let acc = ui.data(|d| d.get_temp::<f32>(id)).unwrap_or(v as f32);
        let acc = (acc + (d.x - d.y) * rate).clamp(min as f32, max as f32);
        ui.data_mut(|d| d.insert_temp(id, acc));
        let nv = acc.round() as i32;
        if nv != v {
            return Some(nv);
        }
    }
    None
}

fn fader_cell(ui: &mut Ui, setter: &ParamSetter, params: &VDrumParams, i: usize, width: f32) {
    let p = &params.faders[i];
    let (rect, resp) = ui.allocate_exact_size(vec2(width, ROW_H), Sense::click_and_drag());
    let v = p.value();
    if resp.drag_started() {
        setter.begin_set_parameter(p);
    }
    if let Some(nv) = drag_value(ui, &resp, v, 0, FADER_MAX) {
        setter.set_parameter(p, nv);
    }
    if resp.drag_stopped() {
        setter.end_set_parameter(p);
    }
    if resp.double_clicked() {
        setter.begin_set_parameter(p);
        setter.set_parameter(p, FADER_MAX);
        setter.end_set_parameter(p);
    }
    let v = p.value();
    paint_row(ui, rect, &resp, Some(v as f32 / FADER_MAX as f32), v != FADER_MAX, false, "Fader", &fader_text(v));
    resp.on_hover_ui(|ui| {
        ui.label(RichText::new(format!("{} Fader", Voice::VOICES[i].label())).strong());
        ui.label(format!("Gain: {}  (position {v} of {FADER_MAX})", fader_text(v)));
        ui.label("The hardware's volume fader for this voice (adc_checkFaders: gain = position²). Default: top (0 dB).");
        ui.label("Not part of the kit; saved with the plugin state and automatable.");
    });
}

#[allow(clippy::too_many_arguments)]
fn param_cell(ui: &mut Ui, setter: &ParamSetter, params: &VDrumParams, shared: &Shared, idx: u16, width: f32, label: &str, dim: bool) {
    let Some(info) = lxr_params::info(idx) else {
        ui.allocate_exact_size(vec2(width, ROW_H), Sense::hover());
        return;
    };
    let Some(p) = params.param(idx) else {
        // Not a host parameter (single possible value): shown read-only.
        let (rect, resp) = ui.allocate_exact_size(vec2(width, ROW_H), Sense::hover());
        paint_row(ui, rect, &resp, Some(0.0), false, true, label, &hw_text(info, shared.value(idx)));
        resp.on_hover_text(format!("{}\nOnly one value exists in the firmware; not a host parameter.\nID: {}", info.name, info.id));
        return;
    };
    let (rect, resp) = ui.allocate_exact_size(vec2(width, ROW_H), Sense::click_and_drag());
    let v = p.value();
    let (min, max) = (info.min as i32, info.max as i32);
    let list = is_list(info);
    let default = plugin_default(info) as i32;

    if !list {
        if resp.drag_started() {
            setter.begin_set_parameter(p);
        }
        if let Some(nv) = drag_value(ui, &resp, v, min, max) {
            setter.set_parameter(p, nv);
        }
        if resp.drag_stopped() {
            setter.end_set_parameter(p);
        }
    }
    if resp.double_clicked() {
        setter.begin_set_parameter(p);
        setter.set_parameter(p, default);
        setter.end_set_parameter(p);
    }

    // List popup
    let popup_id = resp.id.with("list");
    if list && resp.clicked() {
        ui.memory_mut(|m| m.toggle_popup(popup_id));
    }
    if list {
        egui::popup_below_widget(ui, popup_id, &resp, PopupCloseBehavior::CloseOnClick, |ui| {
            ui.set_min_width(width.max(220.0));
            ScrollArea::vertical().max_height(360.0).show(ui, |ui| {
                for o in options(info, params) {
                    let text = if info.is_mod_target() { value_text(info, o) } else { hw_text(info, o) };
                    if ui.selectable_label(o as i32 == v, text).clicked() {
                        setter.begin_set_parameter(p);
                        setter.set_parameter(p, o as i32);
                        setter.end_set_parameter(p);
                    }
                }
            });
        });
    }

    let v = p.value();
    let frac = if list || max <= min { None } else { Some((v - min) as f32 / (max - min) as f32) };
    paint_row(ui, rect, &resp, frac, v != default, dim, label, &hw_text(info, v as u16));
    let learning = shared.learn_target.load(Ordering::Relaxed) == idx as i32;
    let has_learn = shared.learned_for(idx).next().is_some();
    if learning {
        ui.painter().rect_stroke(rect, 2.0, Stroke::new(1.5_f32, CHANGED), StrokeKind::Inside);
    }
    if has_learn {
        ui.painter().circle_filled(pos2(rect.max.x - 3.0, rect.min.y + 3.0), 2.2, Color32::from_rgb(110, 210, 140));
    }

    // MIDI learn menu
    resp.context_menu(|ui| {
        ui.label(RichText::new(info.name).strong());
        if learning {
            if ui.button("Cancel MIDI learn").clicked() {
                shared.learn_target.store(-1, Ordering::Relaxed);
                ui.close_menu();
            }
        } else if ui.button("MIDI learn (move a controller)").clicked() {
            shared.learn_target.store(idx as i32, Ordering::Relaxed);
            ui.close_menu();
        }
        let learned: Vec<(u8, u8)> = shared.learned_for(idx).collect();
        for (ch, cc) in &learned {
            ui.label(format!("Learned: channel {} CC {cc}", ch + 1));
        }
        if !learned.is_empty() && ui.button("Forget learned controller").clicked() {
            shared.forget(idx);
            ui.close_menu();
        }
        if ui.button("Reset to default").clicked() {
            setter.begin_set_parameter(p);
            setter.set_parameter(p, default);
            setter.end_set_parameter(p);
            ui.close_menu();
        }
    });

    resp.on_hover_ui(|ui| {
        ui.label(RichText::new(info.name).strong());
        let readable = value_text(info, v as u16);
        let shown = hw_text(info, v as u16);
        if readable != shown {
            ui.label(format!("Value: {shown}  ({readable}, {v})"));
        } else {
            ui.label(format!("Value: {shown}  ({v})"));
        }
        ui.label(format!(
            "Range: {} … {}  ({min}…{max}); default {}{}",
            hw_text(info, info.min),
            hw_text(info, info.max),
            hw_text(info, default as u16),
            if v != default { "  (changed)" } else { "" }
        ));
        ui.label(midi_help(info, params));
        for (ch, cc) in shared.learned_for(idx) {
            ui.label(format!("Learned: channel {} CC {cc}", ch + 1));
        }
        if !info.description.is_empty() {
            ui.label(info.description);
        }
        ui.label(RichText::new(format!("ID: {}  ({})", info.id, info.enum_name)).small().weak());
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_host_parameter_is_on_screen() {
        let params = VDrumParams::new(Arc::new(Shared::new()));
        let l = Layout::build();
        let mut shown: Vec<u16> = Vec::new();
        let mut faders = Vec::new();
        for (v, groups) in l.voices.iter().enumerate() {
            for g in groups {
                for item in &g.items {
                    match item {
                        Item::Param(idx, label) => {
                            // Voice cards only show parameters of their own voice, with a label.
                            assert_eq!(lxr_params::info(*idx).unwrap().voice, Voice::VOICES[v], "{idx}");
                            assert!(!label.is_empty());
                            assert!(!shown.contains(idx), "{idx} shown twice");
                            shown.push(*idx);
                        }
                        Item::Fader(i) => faders.push(*i),
                    }
                }
            }
        }
        assert_eq!(faders, vec![0, 1, 2, 3, 4, 5]);
        shown.extend(&l.fx);
        shown.extend(&l.performance);
        shown.extend(MIDI_CHAN);
        shown.extend(MIDI_NOTE);
        shown.extend([par::BPM, par::MIDI_CHAN_GLOBAL]);
        for info in params.infos.iter() {
            assert!(shown.contains(&info.index), "{} not shown", info.enum_name);
        }
    }

    #[test]
    fn row_labels_are_spelled_out() {
        let l = |p| row_label(lxr_params::info(p).unwrap());
        assert_eq!(l(par::COARSE1), "Coarse");
        assert_eq!(l(par::NOISE_FREQ1), "Noise Frequency");
        assert_eq!(l(par::VEL_DEST_1), "Destination");
        assert_eq!(l(par::VOL1), "Volume");
        let all: Vec<String> = Layout::build()
            .voices
            .iter()
            .flatten()
            .flat_map(|g| g.items.clone())
            .filter_map(|i| if let Item::Param(_, s) = i { Some(s) } else { None })
            .collect();
        for s in all {
            assert!(s.len() <= 20, "{s}");
        }
    }
}
