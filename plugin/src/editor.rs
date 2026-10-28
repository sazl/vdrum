// SPDX-License-Identifier: GPL-3.0-only
//! The egui editor: every parameter on one screen as a matrix of value bars (voices × sections),
//! plus a column with the FX bus, performance and MIDI settings and the output meters.
//!
//! Interaction: drag a bar horizontally or vertically (Shift: fine), double-click resets to the
//! default, list parameters open their choices on click, right-click opens MIDI learn.

use crate::engine::track_trigger_note;
use crate::params::{is_list, value_text, Command, Shared, VDrumParams, N};
use lxr_params::kit::Kit;
use lxr_params::{fx_param_text, par, Dtype, ParamInfo, Section, Voice, MOD_TARGET_VOICE_OFFSETS, VOICE_PAGES};
use nih_plug::prelude::*;
use nih_plug_egui::egui::{
    self, pos2, vec2, Align2, Color32, FontId, PopupCloseBehavior, Rect, RichText, ScrollArea, Sense, Stroke,
    StrokeKind, Ui,
};
use nih_plug_egui::resizable_window::ResizableWindow;
use nih_plug_egui::{create_egui_editor, EguiState};
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

const CELL_W: f32 = 116.0;
const ROW_H: f32 = 17.0;
const GLOBAL_W: f32 = 250.0;

pub const TRACK_NAMES: [&str; 7] = ["Drum 1", "Drum 2", "Drum 3", "Snare", "Cymbal", "Closed hi-hat", "Open hi-hat"];
pub const PAD_LABELS: [&str; 7] = ["D1", "D2", "D3", "SN", "CY", "CH", "OH"];
const MIDI_CHAN: [u16; 7] =
    [par::MIDI_CHAN_1, par::MIDI_CHAN_2, par::MIDI_CHAN_3, par::MIDI_CHAN_4, par::MIDI_CHAN_5, par::MIDI_CHAN_6, par::MIDI_CHAN_7];
const MIDI_NOTE: [u16; 7] =
    [par::MIDI_NOTE1, par::MIDI_NOTE2, par::MIDI_NOTE3, par::MIDI_NOTE4, par::MIDI_NOTE5, par::MIDI_NOTE6, par::MIDI_NOTE7];

/// Where every parameter is shown.
pub struct Layout {
    /// `matrix[voice][section]`: parameter indices in menu order (sections as
    /// [`Section::VOICE_SECTIONS`]). The hi-hat column merges the closed and open hi-hat pages.
    pub matrix: Vec<Vec<Vec<u16>>>,
    /// Rows per section.
    pub rows: Vec<usize>,
    pub fx: Vec<u16>,
    pub performance: Vec<u16>,
}

impl Layout {
    pub fn build(params: &VDrumParams) -> Layout {
        let exposed = |p: u16| (p as usize) < N && params.slot_of[p as usize] >= 0 && p < lxr_params::END_OF_SOUND_PARAMETERS;
        let mut matrix = Vec::new();
        #[allow(clippy::needless_range_loop)]
        for v in 0..6 {
            let mut secs = Vec::new();
            for s in 0..Section::VOICE_SECTIONS.len() {
                let mut list: Vec<u16> = VOICE_PAGES[v][s].iter().map(|(_, p)| *p).filter(|p| exposed(*p)).collect();
                if v == 5 {
                    for (_, p) in VOICE_PAGES[6][s] {
                        if exposed(*p) && !list.contains(p) {
                            list.push(*p);
                        }
                    }
                }
                secs.push(list);
            }
            matrix.push(secs);
        }
        let rows = (0..Section::VOICE_SECTIONS.len())
            .map(|s| matrix.iter().map(|v: &Vec<Vec<u16>>| v[s].len()).max().unwrap_or(0))
            .collect();
        let placed: Vec<u16> = matrix.iter().flatten().flatten().copied().collect();
        let mut fx = Vec::new();
        let mut performance = Vec::new();
        for info in params.infos.iter() {
            if info.voice == Voice::Fx {
                fx.push(info.index);
            } else if info.kind != lxr_params::ParamKind::Global && !placed.contains(&info.index) {
                performance.push(info.index);
            }
        }
        Layout { matrix, rows, fx, performance }
    }
}

enum FileReply {
    Load(PathBuf),
    Save(PathBuf),
}

struct Data {
    layout: Layout,
    labels: Vec<String>,
    name_buf: String,
    pad_seen: [u32; 7],
    pad_lit: [f64; 7],
    file_reply: Arc<Mutex<Option<FileReply>>>,
    status: String,
}

/// Label of a parameter inside its voice column and section: the readable name without the voice
/// and section prefix.
fn cell_label(info: &ParamInfo) -> String {
    let mut s = info.name;
    if let Some(r) = s.strip_prefix(info.voice.label()) {
        s = r.trim_start();
    }
    if let Some(r) = s.strip_prefix(info.section.label()) {
        if !r.trim().is_empty() {
            s = r.trim_start();
        }
    }
    s.to_string()
}

pub fn create(params: Arc<VDrumParams>, shared: Arc<Shared>) -> Option<Box<dyn Editor>> {
    let egui_state = params.editor_state.clone();
    let mut labels = vec![String::new(); N];
    for info in params.infos.iter() {
        labels[info.index as usize] = cell_label(info);
    }
    let data = Data {
        layout: Layout::build(&params),
        labels,
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
            ctx.set_visuals(egui::Visuals::dark());
            ctx.style_mut(|s| {
                s.spacing.item_spacing = vec2(4.0, 2.0);
                s.spacing.button_padding = vec2(6.0, 2.0);
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
        top_bar(ui, setter, data, params, shared);
        ui.separator();
        ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
            ui.horizontal_top(|ui| {
                band(ui, setter, data, params, shared, 0..4);
                ui.add_space(10.0);
                band(ui, setter, data, params, shared, 4..8);
                ui.add_space(10.0);
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

fn top_bar(ui: &mut Ui, setter: &ParamSetter, data: &mut Data, params: &VDrumParams, shared: &Shared) {
    let _ = setter;
    ui.horizontal(|ui| {
        ui.label(RichText::new("VDrum").heading().strong());
        ui.add_space(8.0);
        ui.label("Kit");
        let edit = ui.add(egui::TextEdit::singleline(&mut data.name_buf).char_limit(8).desired_width(76.0));
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
        ui.separator();
        let now = ui.input(|i| i.time);
        for t in 0..7 {
            let hits = shared.pad_hits[t].load(Ordering::Relaxed);
            if hits != data.pad_seen[t] {
                data.pad_seen[t] = hits;
                data.pad_lit[t] = now;
            }
            let (rect, resp) = ui.allocate_exact_size(vec2(34.0, 24.0), Sense::click());
            let glow = (1.0 - (now - data.pad_lit[t]) / 0.25).clamp(0.0, 1.0) as f32;
            let base = Color32::from_rgb(52, 56, 64);
            let lit = Color32::from_rgb(240, 160, 40);
            let fill = lerp_color(base, lit, glow);
            ui.painter().rect_filled(rect, 3.0, fill);
            if resp.hovered() {
                ui.painter().rect_stroke(rect, 3.0, Stroke::new(1.0_f32, Color32::from_gray(200)), StrokeKind::Inside);
            }
            ui.painter().text(rect.center(), Align2::CENTER_CENTER, PAD_LABELS[t], FontId::proportional(12.0), Color32::WHITE);
            let get = |p: u16| params.param(p).map(|x| x.value() as u16).unwrap_or(0);
            let (ch, note) = track_trigger_note(t, get);
            let resp = resp.on_hover_text(format!(
                "Trigger {}: note-on channel {}, note {} ({})",
                TRACK_NAMES[t],
                ch + 1,
                note,
                lxr_params::format::set_note_name(note).iter().map(|b| *b as char).collect::<String>()
            ));
            if resp.hovered() && ui.input(|i| i.pointer.primary_pressed()) {
                let _ = shared.commands.push(Command::Midi([0x90 | ch, note, 127]));
            }
        }
        ui.separator();
        if ui.small_button("−").on_hover_text("Smaller").clicked() {
            let z = (params.ui_scale.load(Ordering::Relaxed) - 0.1).max(0.5);
            params.ui_scale.store(z, Ordering::Relaxed);
        }
        ui.label(format!("{:.0}%", params.ui_scale.load(Ordering::Relaxed) * 100.0));
        if ui.small_button("+").on_hover_text("Larger").clicked() {
            let z = (params.ui_scale.load(Ordering::Relaxed) + 0.1).min(3.0);
            params.ui_scale.store(z, Ordering::Relaxed);
        }
        ui.separator();
        let target = shared.learn_target.load(Ordering::Relaxed);
        if target >= 0 {
            let name = lxr_params::info(target as u16).map(|i| i.name).unwrap_or("?");
            ui.label(RichText::new(format!("MIDI learn: move a controller for {name}")).color(Color32::from_rgb(255, 170, 60)));
            if ui.small_button("Cancel").clicked() {
                shared.learn_target.store(-1, Ordering::Relaxed);
            }
        } else if !data.status.is_empty() {
            ui.label(RichText::new(&data.status).weak());
        }
    });
}

fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let l = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t) as u8;
    Color32::from_rgb(l(a.r(), b.r()), l(a.g(), b.g()), l(a.b(), b.b()))
}

fn band(ui: &mut Ui, setter: &ParamSetter, data: &mut Data, params: &VDrumParams, shared: &Shared, sections: std::ops::Range<usize>) {
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            for v in Voice::VOICES {
                let (rect, _) = ui.allocate_exact_size(vec2(CELL_W, ROW_H + 2.0), Sense::hover());
                let label = if v == Voice::HiHat { "Hi-hat (CH/OH)" } else { v.label() };
                ui.painter().text(rect.left_center(), Align2::LEFT_CENTER, label, FontId::proportional(13.0), ui.visuals().strong_text_color());
            }
        });
        for s in sections {
            ui.add_space(3.0);
            ui.label(RichText::new(Section::VOICE_SECTIONS[s].label()).small().color(Color32::from_rgb(130, 170, 220)));
            for r in 0..data.layout.rows[s] {
                ui.horizontal(|ui| {
                    for v in 0..6 {
                        match data.layout.matrix[v][s].get(r).copied() {
                            Some(idx) => {
                                let label = data.labels[idx as usize].clone();
                                param_cell(ui, setter, params, shared, idx, CELL_W, &label, false);
                            }
                            None => {
                                ui.allocate_exact_size(vec2(CELL_W, ROW_H), Sense::hover());
                            }
                        }
                    }
                });
            }
        }
    });
}

fn global_column(ui: &mut Ui, setter: &ParamSetter, data: &mut Data, params: &VDrumParams, shared: &Shared) {
    ui.vertical(|ui| {
        ui.set_width(GLOBAL_W);
        heading(ui, "FX bus");
        let fx_type = params.param(par::FX_TYPE).map(|p| p.value() as u8).unwrap_or(0);
        for &idx in &data.layout.fx {
            let text = fx_param_text(fx_type, idx);
            let label = match text {
                Some(t) if (par::FX_P1..=par::FX_P4).contains(&idx) => {
                    format!("P{}: {}", idx - par::FX_P1 + 1, lxr_params::format::text_label(t))
                }
                _ => data.labels[idx as usize].clone(),
            };
            let relevant = text.is_some() || matches!(idx, par::FX_TYPE | par::FX_ROUTING | par::FX_DRY_WET);
            param_cell(ui, setter, params, shared, idx, GLOBAL_W, &label, !relevant);
        }
        if !data.layout.performance.is_empty() {
            heading(ui, "Performance");
            for &idx in &data.layout.performance.clone() {
                let label = lxr_params::info(idx).map(|i| i.name.to_string()).unwrap_or_default();
                param_cell(ui, setter, params, shared, idx, GLOBAL_W, &label, false);
            }
        }
        heading(ui, "MIDI");
        param_cell(ui, setter, params, shared, par::BPM, GLOBAL_W, "BPM", false);
        param_cell(ui, setter, params, shared, par::MIDI_CHAN_GLOBAL, GLOBAL_W, "Global channel", false);
        for t in 0..7 {
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(vec2(78.0, ROW_H), Sense::hover());
                ui.painter().text(rect.left_center(), Align2::LEFT_CENTER, TRACK_NAMES[t], FontId::proportional(11.0), ui.visuals().text_color());
                let w = (GLOBAL_W - 78.0 - 8.0) / 2.0;
                param_cell(ui, setter, params, shared, MIDI_CHAN[t], w, "Ch", false);
                param_cell(ui, setter, params, shared, MIDI_NOTE[t], w, "Note", false);
            });
        }
        heading(ui, "Output");
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
        let resp = ui
            .checkbox(&mut hw, "Hardware clock (44140.625 Hz, resampled)")
            .on_hover_text(
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
            let hw = params.hardware_clock.value();
            let mode = if hw { "resampled from 44140.625 Hz" } else if (rate - 44100.0).abs() < 0.5 { "engine rate, no resampling" } else { "resampled from 44100 Hz" };
            ui.label(RichText::new(format!("{rate:.0} Hz ({mode}), latency {lat} samples")).small().weak());
        }
        heading(ui, "MIDI learn");
        let n = shared.learn.iter().filter(|s| s.load(Ordering::Relaxed) != 0).count();
        ui.horizontal(|ui| {
            ui.label(RichText::new(format!("{n} controller(s) learned")).small());
            if n > 0 && ui.small_button("Forget all").clicked() {
                shared.forget_all();
            }
        });
        ui.label(RichText::new("Right-click a control to learn a CC.").small().weak());
    });
}

fn heading(ui: &mut Ui, s: &str) {
    ui.add_space(4.0);
    ui.label(RichText::new(s).small().color(Color32::from_rgb(130, 170, 220)));
}

fn meters(ui: &mut Ui, shared: &Shared) {
    let names = ["Main L", "Main R", "Aux L", "Aux R"];
    for (c, name) in names.iter().enumerate() {
        let v = shared.meters[c].load(Ordering::Relaxed);
        let db = if v > 0.0 { 20.0 * v.log10() } else { -120.0 };
        let frac = ((db + 60.0) / 60.0).clamp(0.0, 1.0);
        let (rect, _) = ui.allocate_exact_size(vec2(GLOBAL_W, 12.0), Sense::hover());
        let p = ui.painter();
        p.rect_filled(rect, 2.0, Color32::from_gray(30));
        let mut fill = rect;
        fill.set_width(rect.width() * frac);
        let col = if db > -0.1 { Color32::from_rgb(230, 70, 60) } else if db > -12.0 { Color32::from_rgb(220, 190, 60) } else { Color32::from_rgb(80, 190, 110) };
        p.rect_filled(fill, 2.0, col);
        let txt = if db > -120.0 { format!("{name} {db:.1} dBFS") } else { format!("{name} -inf") };
        p.text(rect.left_center() + vec2(4.0, 0.0), Align2::LEFT_CENTER, txt, FontId::proportional(10.0), Color32::WHITE);
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

#[allow(clippy::too_many_arguments)]
fn param_cell(ui: &mut Ui, setter: &ParamSetter, params: &VDrumParams, shared: &Shared, idx: u16, width: f32, label: &str, dim: bool) {
    let (Some(info), Some(p)) = (lxr_params::info(idx), params.param(idx)) else {
        ui.allocate_exact_size(vec2(width, ROW_H), Sense::hover());
        return;
    };
    let (rect, resp) = ui.allocate_exact_size(vec2(width, ROW_H), Sense::click_and_drag());
    let id = resp.id;
    let v = p.value();
    let (min, max) = (info.min as i32, info.max as i32);
    let list = is_list(info);

    // Dragging
    if resp.drag_started() {
        setter.begin_set_parameter(p);
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
        if nv != p.value() {
            setter.set_parameter(p, nv);
        }
    }
    if resp.drag_stopped() {
        setter.end_set_parameter(p);
    }
    if resp.double_clicked() {
        setter.begin_set_parameter(p);
        setter.set_parameter(p, info.default as i32);
        setter.end_set_parameter(p);
    }

    // List popup
    let popup_id = id.with("list");
    if list && resp.clicked() {
        ui.memory_mut(|m| m.toggle_popup(popup_id));
    }
    if list {
        egui::popup_below_widget(ui, popup_id, &resp, PopupCloseBehavior::CloseOnClick, |ui| {
            ui.set_min_width(width.max(180.0));
            ScrollArea::vertical().max_height(320.0).show(ui, |ui| {
                for o in options(info, params) {
                    if ui.selectable_label(o as i32 == v, value_text(info, o)).clicked() {
                        setter.begin_set_parameter(p);
                        setter.set_parameter(p, o as i32);
                        setter.end_set_parameter(p);
                    }
                }
            });
        });
    }

    // Drawing
    let learning = shared.learn_target.load(Ordering::Relaxed) == idx as i32;
    let has_learn = shared.learned_for(idx).next().is_some();
    let visuals = ui.visuals();
    let bg = if resp.hovered() || resp.dragged() { visuals.widgets.hovered.bg_fill } else { visuals.widgets.inactive.bg_fill };
    let painter = ui.painter().with_clip_rect(rect.intersect(ui.clip_rect()));
    painter.rect_filled(rect, 2.0, bg);
    if !list && max > min {
        let frac = (v - min) as f32 / (max - min) as f32;
        let mut f = rect;
        f.set_width(rect.width() * frac);
        painter.rect_filled(f, 2.0, Color32::from_rgba_unmultiplied(70, 130, 200, if dim { 40 } else { 90 }));
    } else if list {
        let tri = Rect::from_min_size(pos2(rect.max.x - 9.0, rect.center().y - 2.0), vec2(6.0, 4.0));
        painter.add(egui::Shape::convex_polygon(
            vec![tri.left_top(), tri.right_top(), tri.center_bottom()],
            visuals.weak_text_color(),
            Stroke::NONE,
        ));
    }
    let alpha = if dim { 110 } else { 255 };
    let text_col = |c: Color32| Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), alpha);
    let font = FontId::proportional(11.0);
    let value = value_text(info, v as u16);
    let vg = painter.layout_no_wrap(value, font.clone(), text_col(visuals.strong_text_color()));
    let right = rect.max.x - if list { 12.0 } else { 4.0 };
    let vpos = pos2(right - vg.size().x, rect.center().y - vg.size().y / 2.0);
    let lg = painter.layout_no_wrap(label.to_string(), font, text_col(visuals.text_color()));
    let lclip = Rect::from_min_max(rect.min, pos2((vpos.x - 4.0).max(rect.min.x), rect.max.y));
    painter
        .with_clip_rect(lclip.intersect(ui.clip_rect()))
        .galley(pos2(rect.min.x + 4.0, rect.center().y - lg.size().y / 2.0), lg, Color32::WHITE);
    painter.galley(vpos, vg, Color32::WHITE);
    if learning {
        painter.rect_stroke(rect, 2.0, Stroke::new(1.5_f32, Color32::from_rgb(255, 170, 60)), StrokeKind::Inside);
    }
    if has_learn {
        painter.circle_filled(pos2(rect.min.x + 2.5, rect.min.y + 2.5), 2.0, Color32::from_rgb(255, 170, 60));
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
            setter.set_parameter(p, info.default as i32);
            setter.end_set_parameter(p);
            ui.close_menu();
        }
    });

    resp.on_hover_ui(|ui| {
        ui.label(RichText::new(info.name).strong());
        ui.label(format!("Value: {}  ({v})", value_text(info, v as u16)));
        ui.label(format!(
            "Range: {} … {}  ({min}…{max}), default {}",
            value_text(info, info.min),
            value_text(info, info.max),
            value_text(info, info.default)
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
        let l = Layout::build(&params);
        let mut shown: Vec<u16> = l.matrix.iter().flatten().flatten().copied().collect();
        shown.extend(&l.fx);
        shown.extend(&l.performance);
        shown.extend(MIDI_CHAN);
        shown.extend(MIDI_NOTE);
        shown.extend([par::BPM, par::MIDI_CHAN_GLOBAL]);
        for info in params.infos.iter() {
            assert!(shown.contains(&info.index), "{} not shown", info.enum_name);
        }
        // Voice columns only show parameters of their own voice.
        for (v, secs) in l.matrix.iter().enumerate() {
            for idx in secs.iter().flatten() {
                assert_eq!(lxr_params::info(*idx).unwrap().voice, Voice::VOICES[v], "{idx}");
            }
        }
    }

    #[test]
    fn labels_drop_voice_and_section() {
        assert_eq!(cell_label(lxr_params::info(par::COARSE1).unwrap()), "Coarse");
        assert_eq!(cell_label(lxr_params::info(par::NOISE_FREQ1).unwrap()), "Noise Frequency");
    }
}
