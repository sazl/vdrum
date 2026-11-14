// SPDX-License-Identifier: GPL-3.0-only
//! The egui editor: one card per voice with all of that voice's sections, and a column with the
//! FX bus, performance, MIDI and output settings.
//!
//! Interaction: drag a value bar horizontally or vertically (Shift: fine), double-click resets to
//! the default, list parameters open their choices on click, right-click opens MIDI learn. Values
//! that differ from their default are marked with an amber stripe.

use crate::engine::track_trigger_note;

use crate::library::{self, ImportPlan};
use crate::lxr_files::{self, PatternPlan};
use crate::params::{
    fader_text, is_list, learn_name, plugin_default, value_text, Command, Shared, VDrumParams, FADER_MAX, LEARN_FADER,
    LEARN_HW_CLOCK, LEARN_SUM_AUX,
};
use lxr_params::kit::Kit;
use lxr_params::open_hat::{oh_index, OH_CHOKE, OH_SPECS, OH_VELOD};
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
const ROW_W: f32 = 180.0;
/// Width of the global column's rows.
const GLOBAL_W: f32 = 206.0;
const ROW_H: f32 = 15.0;
const ROW_GAP: f32 = 1.0;
const CARD_PAD: f32 = 5.0;
const CARD_GAP: f32 = 5.0;
/// Height of a section header.
const HEADER_H: f32 = 11.0;
/// Text sizes: value rows, section headers and notes, card titles.
const ROW_FONT: f32 = 11.0;
const SMALL_FONT: f32 = 9.5;
const TITLE_FONT: f32 = 14.0;
/// Width of the track names in the global column's MIDI table.
const TRACK_COL_W: f32 = 72.0;
/// Default window size (logical pixels, i.e. macOS points) at 100 %: everything fits without
/// scrolling (see the `default_size_fits_everything` test), with one card per voice (the closed and
/// the open hi-hat have their own cards). It fits the MacBook Pro 16" default (1728 × 1117) at 100 %
/// and the MacBook Air 13" (1470 × 956) at 90 % zoom, leaving room for the menu bar, a host's
/// plugin window header and the Dock.
pub const DEFAULT_SIZE: (u32, u32) = (1584, 790);
/// The default size of layout version 4, which had a tab bar and a pill row above the cards. Saved
/// states with this size are migrated to [`DEFAULT_SIZE`].
pub const TABS_SIZE: (u32, u32) = (1584, 836);
/// The default size of layout version 2, where the two hi-hats shared one card. Saved states with
/// this size are migrated to [`DEFAULT_SIZE`].
pub const SHARED_HAT_SIZE: (u32, u32) = (1392, 790);
/// The default size of the first layout, which did not fit laptop screens. Saved states with this
/// size are migrated to [`DEFAULT_SIZE`].
pub const LEGACY_SIZE: (u32, u32) = (1812, 960);
/// Saved with the plugin state; states without it come from the first layout. 3: one card per
/// hi-hat. 4: tab bar and pill row. 5: no tab bar or pill row.
pub const LAYOUT_VERSION: u32 = 5;
pub const MIN_ZOOM: f32 = 0.5;
pub const MAX_ZOOM: f32 = 3.0;
const ZOOM_STEP: f32 = 0.1;

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
    /// The groups of each voice: Drum 1..3, Snare, Cymbal, closed hi-hat (the firmware's hi-hat
    /// voice), open hi-hat (VDrum's separate voice). Each voice has its own card; the hi-hats'
    /// shared choke switch is on both hi-hat cards.
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
        let mut hat_params: Vec<u16> = Vec::new();
        for v in 0..7 {
            let mut params: Vec<u16> = Vec::new();
            if v == 6 {
                // the open hi-hat: the twins of the hi-hat's rows (its decay where the open decay was),
                // the choke after the decay
                for &p in &hat_params {
                    if let Some(k) = OH_SPECS.iter().position(|(_, t)| *t == Some(p)) {
                        params.push(oh_index(k));
                        if k == OH_VELOD {
                            params.push(oh_index(OH_CHOKE));
                        }
                    }
                }
            }
            let pages: &[usize] = match v {
                5 => &[5, 6],
                6 => &[],
                _ => std::slice::from_ref(&[0, 1, 2, 3, 4][v]),
            };
            #[allow(clippy::needless_range_loop)]
            for s in 0..Section::VOICE_SECTIONS.len() {
                for &page in pages {
                    for (_, p) in VOICE_PAGES[page][s] {
                        if exposed(*p) && !params.contains(p) {
                            params.push(*p);
                        }
                    }
                }
            }
            if v == 5 {
                // with the separate open hi-hat the firmware's open decay has no effect: the open
                // hi-hat's card shows its own decay
                hat_params = params.clone();
                params.retain(|p| *p != par::VELOD6_OPEN);
                // the choke switch is shared by both hi-hats and shown on both cards
                if let Some(i) = params.iter().position(|p| *p == par::VELOD6_CLOSED) {
                    params.insert(i + 1, oh_index(OH_CHOKE));
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
            } else if !placed.contains(&info.index) && info.index != par::VELOD6_OPEN {
                performance.push(info.index);
            }
        }
        Layout { voices, fx, performance }
    }
}

enum FileReply {
    Load(PathBuf),
    Save(PathBuf),
    /// Kit files to import into the library.
    ImportKits(Vec<PathBuf>),
    /// An LXR-02 project folder: its `.snd` files are imported into the library.
    ImportFolder(PathBuf),
    /// Export a library slot's kit as a `.snd` file.
    ExportKit(usize, PathBuf),
    /// Pattern files to import into the sequencer.
    ImportPatterns(Vec<PathBuf>),
    /// Export a pattern as a `.pat` file.
    ExportPattern(usize, PathBuf),
    /// Export everything as an LXR-02 project folder.
    ExportProject(PathBuf),
}

/// An import waiting for confirmation.
struct PendingImport {
    kits: ImportPlan,
    patterns: PatternPlan,
    /// The patterns came with a project folder: ask before importing them.
    ask_patterns: bool,
}

struct Data {
    layout: Layout,
    name_buf: String,
    pad_seen: [u32; 7],
    pad_lit: [f64; 7],
    file_reply: Arc<Mutex<Option<FileReply>>>,
    status: String,
    /// The kit library panel is open.
    library_open: bool,
    /// An import that replaces kits or patterns, waiting for confirmation.
    pending_import: Option<PendingImport>,
    /// A project export waiting for confirmation: the folder and the LXR-02 files already there.
    pending_export: Option<(PathBuf, usize)>,
    /// The library panel shows the patterns (otherwise the kits).
    show_patterns: bool,
    /// The slot being renamed and the new name.
    renaming: Option<(usize, String)>,
    /// Size of the scrolled content and of its viewport in the last frame, in points.
    content_size: egui::Vec2,
    viewport_size: egui::Vec2,
}

impl Data {
    fn new() -> Data {
        Data {
            layout: Layout::build(),
            name_buf: String::new(),
            pad_seen: [0; 7],
            pad_lit: [-10.0; 7],
            file_reply: Arc::new(Mutex::new(None)),
            status: String::new(),
            library_open: false,
            pending_import: None,
            pending_export: None,
            show_patterns: false,
            renaming: None,
            content_size: egui::Vec2::ZERO,
            viewport_size: egui::Vec2::ZERO,
        }
    }
}

pub fn create(params: Arc<VDrumParams>, shared: Arc<Shared>) -> Option<Box<dyn Editor>> {
    let egui_state = params.editor_state.clone();
    let data = Data::new();
    let zoom = params.ui_scale.load(Ordering::Relaxed);
    create_egui_editor(
        params.editor_state.clone(),
        data,
        move |ctx, _| setup(ctx, zoom),
        move |ctx, setter, data| update(ctx, setter, data, &params, &shared, &egui_state),
    )
}

fn setup(ctx: &egui::Context, zoom: f32) {
    let mut v = egui::Visuals::dark();
    v.panel_fill = Color32::from_rgb(24, 25, 29);
    v.window_fill = Color32::from_rgb(24, 25, 29);
    ctx.set_visuals(v);
    ctx.style_mut(|s| {
        s.spacing.item_spacing = vec2(4.0, ROW_GAP);
        s.spacing.button_padding = vec2(5.0, 1.0);
        s.spacing.interact_size.y = ROW_H;
        s.interaction.tooltip_delay = 0.35;
        for (style, size) in [
            (egui::TextStyle::Body, 12.0),
            (egui::TextStyle::Button, 12.0),
            (egui::TextStyle::Small, SMALL_FONT),
            (egui::TextStyle::Heading, TITLE_FONT),
        ] {
            s.text_styles.insert(style, FontId::proportional(size));
        }
    });
    // The zoom buttons own the zoom factor (it is saved with the plugin and sizes the window).
    ctx.options_mut(|o| o.zoom_with_keyboard = false);
    ctx.set_zoom_factor(zoom);
}

/// Changes the zoom and asks the host to scale the window by the same ratio, so the content keeps
/// filling it. If the host refuses, the content is zoomed inside the unchanged window.
fn set_zoom(params: &VDrumParams, egui_state: &EguiState, zoom: f32) {
    let old = params.ui_scale.load(Ordering::Relaxed);
    let zoom = ((zoom / ZOOM_STEP).round() * ZOOM_STEP).clamp(MIN_ZOOM, MAX_ZOOM);
    if (zoom - old).abs() < 1e-3 {
        return;
    }
    params.ui_scale.store(zoom, Ordering::Relaxed);
    let (w, h) = egui_state.requested_size().unwrap_or_else(|| egui_state.size());
    let r = zoom / old;
    egui_state.set_requested_size(((w as f32 * r).round() as u32, (h as f32 * r).round() as u32));
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

    // Plugin controls changed by a learned CC.
    for (k, flag) in shared.extra_dirty.iter().enumerate() {
        if flag.swap(false, Ordering::Relaxed) {
            let v = shared.extra_values[k].load(Ordering::Relaxed);
            let target = LEARN_FADER + k as u16;
            if target < LEARN_SUM_AUX {
                let p = &params.faders[k];
                setter.begin_set_parameter(p);
                setter.set_parameter(p, v);
                setter.end_set_parameter(p);
            } else {
                let p = if target == LEARN_SUM_AUX { &params.sum_aux } else { &params.hardware_clock };
                setter.begin_set_parameter(p);
                setter.set_parameter(p, v != 0);
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
                    shared.deselect_library();
                    shared.set_kit_file(Some(path.clone()), kit.to_file_bytes().to_vec());
                    let _ = shared.commands.push(Command::LoadKit(kit.to_file_bytes()));
                    data.status = format!("Loaded {}", path.display());
                }
                Err(e) => data.status = format!("{}: {e}", path.display()),
            },
            Err(e) => data.status = format!("{}: {e}", path.display()),
        },
        Some(FileReply::Save(path)) => {
            let bytes = params.kit().to_file_bytes();
            data.status = match std::fs::write(&path, bytes) {
                Ok(()) => {
                    // the kit is now that file's, not a library slot's
                    shared.deselect_library();
                    shared.set_kit_file(Some(path.clone()), bytes.to_vec());
                    format!("Saved {}", path.display())
                }
                Err(e) => format!("{}: {e}", path.display()),
            }
        }
        Some(FileReply::ImportKits(paths)) => plan_import(data, shared, &paths, &[], false),
        Some(FileReply::ImportPatterns(paths)) => plan_import(data, shared, &[], &paths, false),
        Some(FileReply::ImportFolder(dir)) => {
            let kits = library::snd_files(&dir);
            let pats = lxr_files::pat_files(&dir);
            if kits.is_empty() && pats.is_empty() {
                data.status = format!("No .snd or .pat files in {}", dir.display());
            } else {
                plan_import(data, shared, &kits, &pats, true);
            }
        }
        Some(FileReply::ExportPattern(n, path)) => {
            let file = pattern_file(shared, n);
            data.status = match file.map(|f| std::fs::write(&path, lxr_files::pat_to_bytes(&f))) {
                Some(Ok(())) => format!("Exported {}", path.display()),
                Some(Err(e)) => format!("{}: {e}", path.display()),
                None => "Pattern not available".into(),
            };
        }
        Some(FileReply::ExportProject(dir)) => {
            let existing = lxr_files::existing_project_files(&dir).len();
            data.pending_export = Some((dir, existing));
            data.library_open = true;
        }
        Some(FileReply::ExportKit(slot, path)) => {
            let kit = shared.library.lock().ok().and_then(|l| l.slots.get(slot).cloned().flatten()).map(|e| e.kit);
            data.status = match kit {
                Some(k) => match std::fs::write(&path, k.to_file_bytes()) {
                    Ok(()) => format!("Exported {}", path.display()),
                    Err(e) => format!("{}: {e}", path.display()),
                },
                None => format!("Slot {slot:02} is empty"),
            };
        }
        None => {}
    }

    ResizableWindow::new("vdrum-window").min_size(vec2(480.0, 320.0)).show(ctx, egui_state, |ui| {
        top_bar(ui, data, params, shared, egui_state);
        let out = ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = CARD_GAP;
                for v in 0..7 {
                    voice_card(ui, setter, data, params, shared, v);
                }
                global_column(ui, setter, data, params, shared);
            });
        });
        data.content_size = out.content_size;
        data.viewport_size = out.inner_rect.size();
    });
    let mut open = data.library_open;
    egui::Window::new("Library")
        .open(&mut open)
        .default_size(vec2(360.0, 520.0))
        .collapsible(false)
        .show(ctx, |ui| library_panel(ui, data, shared));
    data.library_open = open && data.library_open;
    ctx.request_repaint_after(std::time::Duration::from_millis(33));
}

fn spawn_dialog(reply: Arc<Mutex<Option<FileReply>>>, save: Option<String>) {
    run_dialog(reply, move || {
        let d = kit_dialog();
        match save {
            Some(name) => d.set_file_name(name).save_file().map(FileReply::Save),
            None => d.pick_file().map(FileReply::Load),
        }
    });
}

fn kit_dialog() -> rfd::FileDialog {
    rfd::FileDialog::new().add_filter("Kit (.snd)", &["snd", "SND"])
}

fn pat_dialog() -> rfd::FileDialog {
    rfd::FileDialog::new().add_filter("LXR-02 pattern (.pat)", &["pat", "PAT"])
}

/// Runs a file dialog on its own thread; its answer is handled in the next frame.
fn run_dialog(reply: Arc<Mutex<Option<FileReply>>>, dialog: impl FnOnce() -> Option<FileReply> + Send + 'static) {
    std::thread::spawn(move || {
        if let (Some(r), Ok(mut slot)) = (dialog(), reply.lock()) {
            *slot = Some(r);
        }
    });
}

/// Imports at once, or asks first when the import replaces kits or patterns, or brings patterns
/// with a project folder.
fn plan_import(data: &mut Data, shared: &Shared, kit_paths: &[PathBuf], pat_paths: &[PathBuf], ask_patterns: bool) {
    let kits = match shared.library.lock() {
        Ok(l) => l.plan_import_paths(kit_paths),
        Err(_) => return,
    };
    let (files, errors) = lxr_files::read_files(pat_paths);
    let (patterns, replaces_patterns) = match shared.seq.lock() {
        Ok(seq) => {
            let mut p = lxr_files::plan_patterns(&seq, &files);
            p.errors.extend(errors);
            let r = !p.replaces(&seq).is_empty();
            (p, r)
        }
        Err(_) => return,
    };
    let pending = PendingImport { kits, patterns, ask_patterns };
    let ask = !pending.kits.replaces().is_empty() || replaces_patterns || (ask_patterns && !pending.patterns.puts.is_empty());
    if ask {
        data.pending_import = Some(pending);
        data.library_open = true;
    } else {
        data.status = apply_import(shared, &pending, true);
    }
}

/// Applies an import; returns the status line.
fn apply_import(shared: &Shared, p: &PendingImport, with_patterns: bool) -> String {
    let mut parts = Vec::new();
    if let Ok(mut l) = shared.library.lock() {
        l.apply(&p.kits);
        let kits_found = !p.kits.puts.is_empty() || p.kits.init_skipped + p.kits.duplicates + p.kits.no_room + p.kits.errors.len() > 0;
        if kits_found {
            parts.push(p.kits.summary());
        }
        if with_patterns {
            let found = !p.patterns.puts.is_empty() || p.patterns.empty_skipped + p.patterns.no_room + p.patterns.errors.len() > 0;
            if let (true, Ok(mut seq)) = (found, shared.seq.lock()) {
                p.patterns.apply(&mut seq, &mut l);
                parts.push(p.patterns.summary());
            }
            if !p.patterns.puts.is_empty() {
                let _ = shared.commands.push(Command::SeqReload);
            }
        } else if !p.patterns.puts.is_empty() {
            parts.push("patterns skipped".into());
        }
    }
    shared.library_generation.fetch_add(1, Ordering::Relaxed);
    let s = parts.join(", ");
    if s.starts_with("Imported") {
        s
    } else {
        format!("Imported {s}")
    }
}

/// A pattern as a pattern file (the sequencer's data, the library's name and kit number).
fn pattern_file(shared: &Shared, n: usize) -> Option<lxr_files::PatFile> {
    let lib = shared.library.lock().ok()?;
    let seq = shared.seq.lock().ok()?;
    Some(lxr_files::PatFile { name: *lib.pattern_names.get(n)?, pattern: seq.patterns.get(n)?.clone(), kit_nr: lib.pattern_kits[n] })
}

/// Writes the project export; returns the status line.
fn export_project(shared: &Shared, dir: &std::path::Path) -> String {
    let files = {
        let (Ok(lib), Ok(seq)) = (shared.library.lock(), shared.seq.lock()) else { return "Export failed".into() };
        let globals: Vec<u8> = lxr_files::GLOBALS.map(|i| shared.values[i].load(Ordering::Relaxed) as u8).collect();
        let mut name = [0u8; 8];
        name.copy_from_slice(&shared.name_bytes()[..8]);
        lxr_files::project_files(&lib, &seq, &globals, &name)
    };
    match lxr_files::write_project(dir, &files) {
        Ok(n) => format!("Exported {n} files to {}", dir.display()),
        Err(e) => format!("{}: {e}", dir.display()),
    }
}

/// Library actions chosen in a frame, done after the library lock is released.
enum LibAction {
    Load(usize),
    Store(usize),
    Clear(usize),
    Move(usize, i32),
    Rename(usize, String),
    Export(usize),
    RenamePattern(usize, String),
    ExportPattern(usize),
}

fn do_lib_action(data: &mut Data, shared: &Shared, a: LibAction) {
    let bump = || shared.library_generation.fetch_add(1, Ordering::Relaxed);
    match a {
        LibAction::Load(s) => {
            if let Some(label) = shared.load_library_slot(s) {
                data.status = format!("Loaded {label} from the library");
            }
        }
        LibAction::Store(s) => {
            if let Some(label) = shared.store_in_library(s) {
                data.status = format!("Saved the kit to library slot {label}");
            }
        }
        LibAction::Clear(s) => {
            if let Ok(mut l) = shared.library.lock() {
                l.slots[s] = None;
                if l.selected == Some(s) {
                    l.selected = None;
                }
            }
            bump();
        }
        LibAction::Move(s, d) => {
            if let Ok(mut l) = shared.library.lock() {
                l.move_slot(s, d);
            }
            bump();
        }
        LibAction::Rename(s, name) => {
            let selected = shared.library.lock().ok().and_then(|mut l| {
                let e = l.slots.get_mut(s)?.as_mut()?;
                e.kit.set_name(&name);
                let k = e.kit.clone();
                (l.selected == Some(s)).then_some(k)
            });
            // the loaded kit is the slot's: its name follows
            if let Some(k) = selected {
                shared.set_name(&k.name);
            }
            bump();
        }
        LibAction::RenamePattern(n, name) => {
            if let Ok(mut l) = shared.library.lock() {
                l.pattern_names[n] = lxr_files::name_bytes(&name);
            }
            bump();
        }
        LibAction::ExportPattern(n) => {
            let name = shared.library.lock().ok().map(|l| lxr_files::file_name(n, &l.pattern_names[n], "PAT"));
            if let Some(name) = name {
                run_dialog(data.file_reply.clone(), move || pat_dialog().set_file_name(name).save_file().map(|p| FileReply::ExportPattern(n, p)));
            }
        }
        LibAction::Export(s) => {
            let name = shared.library.lock().ok().and_then(|l| l.slots.get(s).cloned().flatten()).map(|e| e.kit.file_name(s as u8));
            if let Some(name) = name {
                run_dialog(data.file_reply.clone(), move || kit_dialog().set_file_name(name).save_file().map(|p| FileReply::ExportKit(s, p)));
            }
        }
    }
}

/// The library's menu entries (top bar).
fn library_menu(ui: &mut Ui, data: &mut Data, shared: &Shared) {
    if ui.button("Show library…").clicked() {
        data.library_open = true;
        ui.close_menu();
    }
    ui.separator();
    if ui.button("Import kits…").on_hover_text("Add .snd files to the library (NN-NAME.SND goes to slot NN)").clicked() {
        run_dialog(data.file_reply.clone(), || kit_dialog().pick_files().map(FileReply::ImportKits));
        ui.close_menu();
    }
    if ui.button("Import patterns…").on_hover_text("Load LXR-02 .pat files into the sequencer (NN-NAME.PAT goes to pattern NN + 1)").clicked() {
        run_dialog(data.file_reply.clone(), || pat_dialog().pick_files().map(FileReply::ImportPatterns));
        ui.close_menu();
    }
    if ui.button("Import LXR-02 project folder…").on_hover_text("Add the kits (.snd) of a project folder and, if you confirm, its patterns (.pat); init kits and empty patterns are skipped").clicked() {
        run_dialog(data.file_reply.clone(), || rfd::FileDialog::new().pick_folder().map(FileReply::ImportFolder));
        ui.close_menu();
    }
    if ui.button("Export LXR-02 project…").on_hover_text("Write all kits, patterns and the globals as an LXR-02 project folder").clicked() {
        run_dialog(data.file_reply.clone(), || rfd::FileDialog::new().pick_folder().map(FileReply::ExportProject));
        ui.close_menu();
    }
    ui.separator();
    let selected = shared.library_selected();
    let first_empty = shared.library.lock().ok().and_then(|l| (0..library::SLOTS).find(|&s| l.slots[s].is_none()));
    if let Some(s) = selected {
        if ui.button(format!("Save kit to slot {s:02}")).clicked() {
            do_lib_action(data, shared, LibAction::Store(s));
            ui.close_menu();
        }
    }
    let r = ui.add_enabled(first_empty.is_some(), egui::Button::new("Store kit in an empty slot"));
    if r.clicked() {
        if let Some(s) = first_empty {
            do_lib_action(data, shared, LibAction::Store(s));
        }
        ui.close_menu();
    }
}

/// Import and export confirmations at the top of the library panel.
fn confirmations(ui: &mut Ui, data: &mut Data, shared: &Shared) {
    if let Some(p) = &data.pending_import {
        let replaced = p.kits.replaces();
        if !replaced.is_empty() {
            let n = replaced.len();
            ui.label(RichText::new(format!("This import replaces {n} kit{}:", if n == 1 { "" } else { "s" })).color(CHANGED));
            ui.label(RichText::new(replaced.join(", ")).size(SMALL_FONT));
        }
        let numbers: Vec<usize> = p.patterns.puts.iter().map(|(n, _)| *n).collect();
        let has_patterns = !numbers.is_empty();
        let has_kits = !p.kits.puts.is_empty();
        if has_patterns {
            let n = numbers.len();
            let q = if p.ask_patterns { "Also import" } else { "Import" };
            ui.label(
                RichText::new(format!(
                    "{q} {n} pattern{}? This replaces {} in the sequencer.",
                    if n == 1 { "" } else { "s" },
                    PatternPlan::range_text(&numbers)
                ))
                .color(CHANGED),
            );
        }
        let mut done = None;
        ui.horizontal(|ui| {
            if has_patterns && has_kits {
                if ui.button("Import kits and patterns").clicked() {
                    done = Some(Some(true));
                }
                if ui.button("Kits only").on_hover_text("Skip the patterns").clicked() {
                    done = Some(Some(false));
                }
            } else if ui.button("Import").clicked() {
                done = Some(Some(true));
            }
            if ui.button("Cancel").clicked() {
                done = Some(None);
            }
        });
        match done {
            Some(Some(with_patterns)) => {
                if let Some(p) = data.pending_import.take() {
                    data.status = apply_import(shared, &p, with_patterns);
                }
            }
            Some(None) => {
                data.pending_import = None;
                data.status = "Import cancelled".into();
            }
            None => {}
        }
        ui.separator();
    }
    if let Some((dir, existing)) = &data.pending_export {
        ui.label(RichText::new(format!("Export an LXR-02 project to {}", dir.display())).color(CHANGED));
        if *existing > 0 {
            ui.label(RichText::new(format!("This replaces the {existing} LXR-02 project files in the folder.")).color(CHANGED));
        }
        ui.label(RichText::new("The LXR-02 cannot hold:").size(SMALL_FONT).color(LABEL));
        for l in lxr_files::EXPORT_LOSSES {
            ui.label(RichText::new(format!("• {l}")).size(SMALL_FONT).color(LABEL));
        }
        let mut done = None;
        ui.horizontal(|ui| {
            if ui.button("Export").clicked() {
                done = Some(true);
            }
            if ui.button("Cancel").clicked() {
                done = Some(false);
            }
        });
        match done {
            Some(true) => {
                if let Some((dir, _)) = data.pending_export.take() {
                    data.status = export_project(shared, &dir);
                }
            }
            Some(false) => {
                data.pending_export = None;
                data.status = "Export cancelled".into();
            }
            None => {}
        }
        ui.separator();
    }
}

/// The 64 sequencer patterns with their LXR-02 names and kit numbers. Right-click: rename,
/// export `.pat`.
fn pattern_list(ui: &mut Ui, data: &mut Data, shared: &Shared, lib: &library::Library) {
    ui.label(RichText::new("Right-click: rename, export .pat. The kit number is kept for export; VDrum does not change kits with patterns.").size(SMALL_FONT).color(LABEL));
    let steps: Vec<usize> = match shared.seq.lock() {
        Ok(seq) => seq.patterns.iter().map(|p| p.tracks.iter().map(|t| t.bits().count_ones() as usize).sum()).collect(),
        Err(_) => return,
    };
    let mut actions = Vec::new();
    ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        for (n, &steps) in steps.iter().enumerate() {
            if let Some((rn, buf)) = data.renaming.as_mut().filter(|(rn, _)| *rn == n) {
                let r = ui.add(egui::TextEdit::singleline(buf).char_limit(8).desired_width(120.0));
                r.request_focus();
                if r.lost_focus() {
                    if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        actions.push(LibAction::RenamePattern(*rn, buf.clone()));
                    }
                    data.renaming = None;
                }
                continue;
            }
            let name = lxr_files::name_text(&lib.pattern_names[n]);
            let text = format!("{:02} {:<8}  {steps} steps, kit {:02}", n + 1, name.trim_end(), lib.pattern_kits[n]);
            let rich = if steps == 0 { RichText::new(text).color(Color32::from_gray(110)) } else { RichText::new(text) };
            let r = ui.selectable_label(false, rich);
            r.context_menu(|ui| {
                if ui.button("Rename").clicked() {
                    data.renaming = Some((n, name.trim_end().to_string()));
                    ui.close_menu();
                }
                if ui.button("Export .pat…").clicked() {
                    actions.push(LibAction::ExportPattern(n));
                    ui.close_menu();
                }
            });
        }
    });
    for a in actions {
        do_lib_action(data, shared, a);
    }
}

/// The library panel: the 64 slots. Click loads; right-click for the other actions.
fn library_panel(ui: &mut Ui, data: &mut Data, shared: &Shared) {
    let mut actions = Vec::new();
    confirmations(ui, data, shared);
    let Ok(lib) = shared.library.lock().map(|l| l.clone()) else { return };
    ui.horizontal(|ui| {
        ui.menu_button("Library ▾", |ui| library_menu(ui, data, shared));
        for (patterns, text) in [(false, "Kits"), (true, "Patterns")] {
            if ui.selectable_label(data.show_patterns == patterns, text).clicked() && data.show_patterns != patterns {
                data.show_patterns = patterns;
                data.renaming = None;
            }
        }
        ui.label(RichText::new(format!("{} of {} slots used", lib.len(), library::SLOTS)).size(10.5).color(LABEL));
    });
    if data.show_patterns {
        pattern_list(ui, data, shared, &lib);
        return;
    }
    ui.label(RichText::new("Click loads a kit. Right-click: save, rename, move, export, clear.").size(SMALL_FONT).color(LABEL));
    ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
        for s in 0..library::SLOTS {
            let entry = lib.slots[s].as_ref();
            if let Some((rs, buf)) = data.renaming.as_mut().filter(|(rs, _)| *rs == s) {
                let r = ui.add(egui::TextEdit::singleline(buf).char_limit(8).desired_width(120.0));
                r.request_focus();
                if r.lost_focus() {
                    if ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                        actions.push(LibAction::Rename(*rs, buf.clone()));
                    }
                    data.renaming = None;
                }
                continue;
            }
            let text = match entry {
                Some(e) if !e.source.is_empty() => RichText::new(format!("{}   ({})", lib.label(s), e.source)),
                Some(_) => RichText::new(lib.label(s)),
                None => RichText::new(lib.label(s)).color(Color32::from_gray(110)),
            };
            let r = ui.selectable_label(lib.selected == Some(s), text);
            if r.clicked() && entry.is_some() {
                actions.push(LibAction::Load(s));
            }
            r.context_menu(|ui| {
                if ui.button(if entry.is_some() { "Save current kit here (replace)" } else { "Save current kit here" }).clicked() {
                    actions.push(LibAction::Store(s));
                    ui.close_menu();
                }
                if entry.is_some() {
                    if ui.button("Rename").clicked() {
                        data.renaming = Some((s, entry.map(|e| e.kit.name_str().trim_end().to_string()).unwrap_or_default()));
                        ui.close_menu();
                    }
                    if ui.button("Export .snd…").clicked() {
                        actions.push(LibAction::Export(s));
                        ui.close_menu();
                    }
                    if ui.button("Clear").clicked() {
                        actions.push(LibAction::Clear(s));
                        ui.close_menu();
                    }
                }
                if s > 0 && ui.button("Move up").clicked() {
                    actions.push(LibAction::Move(s, -1));
                    ui.close_menu();
                }
                if s + 1 < library::SLOTS && ui.button("Move down").clicked() {
                    actions.push(LibAction::Move(s, 1));
                    ui.close_menu();
                }
            });
        }
    });
    for a in actions {
        do_lib_action(data, shared, a);
    }
}

fn top_bar(ui: &mut Ui, data: &mut Data, params: &VDrumParams, shared: &Shared, egui_state: &EguiState) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        ui.label(RichText::new("VDrum").size(16.0).strong().color(Color32::WHITE));
        ui.label(RichText::new("LXR-02 engine").size(10.5).color(LABEL));
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
            let k = Kit::from_values(shared.name_bytes(), [0; lxr_params::kit::KIT_PARAMS]);
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
            shared.deselect_library();
            let _ = shared.commands.push(Command::LoadKit(k.to_file_bytes()));
        }
        ui.separator();
        let (selected, filled) = shared.library.lock().map(|l| (l.selected, l.len())).unwrap_or((None, 0));
        for (text, dir, tip) in [("◀", -1, "Load the previous kit of the library"), ("▶", 1, "Load the next kit of the library")] {
            let r = ui.add_enabled(filled > 0, egui::Button::new(text).small()).on_hover_text(tip);
            if r.clicked() {
                let next = shared.library.lock().ok().and_then(|l| l.step(selected, dir));
                if let Some(s) = next {
                    do_lib_action(data, shared, LibAction::Load(s));
                }
            }
        }
        let title = match selected {
            Some(s) => format!("Library {s:02} ▾"),
            None => "Library ▾".to_string(),
        };
        ui.menu_button(title, |ui| library_menu(ui, data, shared))
            .response
            .on_hover_text(format!("{filled} of {} library slots used", library::SLOTS));
        if let Some(s) = selected {
            if ui.button("Save kit").on_hover_text(format!("Write the kit into library slot {s:02}")).clicked() {
                do_lib_action(data, shared, LibAction::Store(s));
            }
        }
        let changed = params
            .infos
            .iter()
            .filter(|i| i.kind == lxr_params::ParamKind::Sound)
            .filter(|i| params.param(i.index).is_some_and(|p| p.value() as u16 != plugin_default(i)))
            .count();
        ui.label(RichText::new(format!("{changed} values differ from the init kit")).size(10.5).color(LABEL))
            .on_hover_text("Rows with an amber stripe differ from their default. Double-click a row to reset it.");
        ui.separator();
        let target = shared.learn_target.load(Ordering::Relaxed);
        if target >= 0 {
            let name = learn_name(target as u16);
            ui.label(RichText::new(format!("MIDI learn: move a controller for {name}")).color(CHANGED));
            if ui.small_button("Cancel").clicked() {
                shared.learn_target.store(-1, Ordering::Relaxed);
            }
        } else if !data.status.is_empty() {
            ui.label(RichText::new(&data.status).size(10.5).color(LABEL));
        }
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let z = params.ui_scale.load(Ordering::Relaxed);
            if ui.small_button("+").on_hover_text("Larger (window and content)").clicked() {
                set_zoom(params, egui_state, z + ZOOM_STEP);
            }
            if ui.small_button(format!("{:.0}%", z * 100.0)).on_hover_text("Reset to 100 %").clicked() {
                set_zoom(params, egui_state, 1.0);
            }
            if ui.small_button("−").on_hover_text("Smaller (window and content)").clicked() {
                set_zoom(params, egui_state, z - ZOOM_STEP);
            }
            ui.label(RichText::new("Zoom").size(10.5).color(LABEL));
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
    ui.painter().text(rect.center(), Align2::CENTER_CENTER, PAD_LABELS[t], FontId::proportional(11.0), Color32::WHITE);
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
            // Header: name, trigger note, pad
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.set_width(ROW_W - 32.0 - 4.0);
                    let title = if v < 5 { Voice::VOICES[v].label() } else { TRACK_NAMES[v] };
                    ui.label(RichText::new(title).size(TITLE_FONT).strong().color(Color32::WHITE));
                    let get = |p: u16| params.param(p).map(|x| x.value() as u16).unwrap_or(0);
                    let (ch, note) = track_trigger_note(v, get);
                    let what = if get(MIDI_NOTE[v]) == 0 { "any note".to_string() } else { format!("{} ({note})", note_text(note)) };
                    ui.label(RichText::new(format!("{what}, channel {}", ch + 1)).size(SMALL_FONT).color(LABEL));
                });
                pad(ui, data, params, shared, v, vec2(28.0, 26.0));
            });
            for g in &data.layout.voices[v] {
                section_header(ui, g.title, ROW_W);
                for item in &g.items {
                    match item {
                        Item::Param(idx, label) => param_cell(ui, setter, params, shared, *idx, ROW_W, label, false),
                        Item::Fader(i) => fader_cell(ui, setter, params, shared, *i, ROW_W),
                    }
                }
            }
        });
    });
}

fn section_header(ui: &mut Ui, title: &str, width: f32) {
    ui.add_space(1.0);
    let (rect, _) = ui.allocate_exact_size(vec2(width, HEADER_H), Sense::hover());
    let p = ui.painter();
    let g = p.layout_no_wrap(title.to_uppercase(), FontId::proportional(SMALL_FONT), ACCENT);
    let y = rect.center().y;
    let tw = g.size().x;
    p.galley(pos2(rect.min.x, y - g.size().y / 2.0), g, ACCENT);
    p.line_segment([pos2(rect.min.x + tw + 6.0, y), pos2(rect.max.x, y)], Stroke::new(1.0_f32, Color32::from_rgb(60, 70, 88)));
}

fn global_column(ui: &mut Ui, setter: &ParamSetter, data: &mut Data, params: &VDrumParams, shared: &Shared) {
    card(ui, GLOBAL_W, |ui| {
        ui.vertical(|ui| {
            ui.label(RichText::new("FX bus & global").size(TITLE_FONT).strong().color(Color32::WHITE));
            let fx_type = params.param(par::FX_TYPE).map(|p| p.value() as u8).unwrap_or(0);
            let fx_name = lxr_params::info(par::FX_TYPE).map(|i| value_text(i, fx_type as u16)).unwrap_or_default();
            ui.label(RichText::new(format!("Type {fx_name}: unused controls are dimmed")).size(SMALL_FONT).color(LABEL));
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
                let w = (GLOBAL_W - TRACK_COL_W - 8.0) / 2.0;
                for (text, width) in [("Track", TRACK_COL_W), ("Channel", w), ("Trigger note", w)] {
                    let (r, _) = ui.allocate_exact_size(vec2(width, HEADER_H), Sense::hover());
                    ui.painter().text(r.left_center(), Align2::LEFT_CENTER, text, FontId::proportional(SMALL_FONT), LABEL);
                }
            });
            for t in 0..7 {
                ui.horizontal(|ui| {
                    let (rect, _) = ui.allocate_exact_size(vec2(TRACK_COL_W, ROW_H), Sense::hover());
                    ui.painter().text(rect.left_center(), Align2::LEFT_CENTER, TRACK_NAMES[t], FontId::proportional(ROW_FONT), LABEL_CHANGED);
                    let w = (GLOBAL_W - TRACK_COL_W - 8.0) / 2.0;
                    param_cell(ui, setter, params, shared, MIDI_CHAN[t], w, "Ch", false);
                    param_cell(ui, setter, params, shared, MIDI_NOTE[t], w, "Note", false);
                });
            }
            ui.label(
                RichText::new("Plugin default: one General MIDI note per track on channel 1 (the hardware default is note C4 for all tracks).")
                    .size(SMALL_FONT)
                    .color(LABEL),
            );
            section_header(ui, "Output", GLOBAL_W);
            meters(ui, shared);
            let has_aux = shared.has_aux.load(Ordering::Relaxed);
            let mut sum = params.sum_aux.value();
            let resp = ui.add_enabled(has_aux, egui::Checkbox::new(&mut sum, "Sum aux (DAC 2) into main"));
            learn_marks(ui, resp.rect, shared, LEARN_SUM_AUX);
            resp.context_menu(|ui| learn_menu(ui, shared, LEARN_SUM_AUX));
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
            learn_marks(ui, resp.rect, shared, LEARN_HW_CLOCK);
            resp.context_menu(|ui| learn_menu(ui, shared, LEARN_HW_CLOCK));
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
                ui.label(RichText::new(format!("{rate:.0} Hz ({mode}), latency {lat} samples")).size(SMALL_FONT).color(LABEL));
            }
            section_header(ui, "MIDI learn", GLOBAL_W);
            let n = shared.learn.iter().filter(|s| s.load(Ordering::Relaxed) != 0).count();
            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("{n} controller(s) learned")).size(10.5).color(LABEL_CHANGED));
                if n > 0 && ui.small_button("Forget all").clicked() {
                    shared.forget_all();
                }
            });
            ui.label(RichText::new("Right-click a control to learn a CC. Tooltips show each control's CC/NRPN.").size(SMALL_FONT).color(LABEL));
        });
    });
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
        let col = if db > -0.1 {
            Color32::from_rgb(230, 70, 60)
        } else if db > -12.0 {
            Color32::from_rgb(220, 190, 60)
        } else {
            Color32::from_rgb(80, 190, 110)
        };
        p.rect_filled(fill, 2.0, col);
        let txt = if db > -120.0 { format!("{name}  {db:.1} dBFS") } else { format!("{name}  -inf") };
        p.text(rect.left_center() + vec2(4.0, 0.0), Align2::LEFT_CENTER, txt, FontId::proportional(SMALL_FONT), Color32::WHITE);
    }
}

/// The values a list parameter offers.
fn options(info: &ParamInfo, params: &VDrumParams) -> Vec<u16> {
    if info.is_mod_target() {
        let oh = lxr_params::open_hat::oh_k(info.index).is_some();
        let voice = if oh && info.dtype == Dtype::TargetSelectionLfo {
            // the open hi-hat's LFO voice (1..=7, 7 = the open hi-hat)
            let p = oh_index(lxr_params::open_hat::OH_VOICE_LFO);
            params.param(p).map(|p| p.value().clamp(1, 7) as u8 - 1).unwrap_or(0)
        } else if oh {
            6
        } else if info.dtype == Dtype::TargetSelectionLfo {
            // The LFO's destination voice (PAR_VOICE_LFOx, 1..=6).
            let lfo = info.index - par::TARGET_LFO1;
            params.param(par::VOICE_LFO1 + lfo).map(|p| p.value().clamp(1, 6) as u8 - 1).unwrap_or(0)
        } else {
            info.voice.number().unwrap_or(0)
        };
        let (lo, hi) = if voice == 6 {
            (lxr_params::open_hat::OH_TARGET_BASE, lxr_params::open_hat::OH_MAX_TARGET)
        } else {
            MOD_TARGET_VOICE_OFFSETS[voice.min(5) as usize]
        };
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
    let font = FontId::proportional(ROW_FONT);
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

/// Outline while a control waits for a CC, green dot when it has a learned CC.
fn learn_marks(ui: &Ui, rect: Rect, shared: &Shared, target: u16) {
    if shared.learn_target.load(Ordering::Relaxed) == target as i32 {
        ui.painter().rect_stroke(rect, 2.0, Stroke::new(1.5_f32, CHANGED), StrokeKind::Inside);
    }
    if shared.learned_for(target).next().is_some() {
        ui.painter().circle_filled(pos2(rect.max.x - 3.0, rect.min.y + 3.0), 2.2, Color32::from_rgb(110, 210, 140));
    }
}

/// The MIDI learn entries of a control's context menu.
fn learn_menu(ui: &mut Ui, shared: &Shared, target: u16) {
    ui.label(RichText::new(learn_name(target)).strong());
    if shared.learn_target.load(Ordering::Relaxed) == target as i32 {
        if ui.button("Cancel MIDI learn").clicked() {
            shared.learn_target.store(-1, Ordering::Relaxed);
            ui.close_menu();
        }
    } else if ui.button("MIDI learn (move a controller)").clicked() {
        shared.learn_target.store(target as i32, Ordering::Relaxed);
        ui.close_menu();
    }
    let learned: Vec<(u8, u8)> = shared.learned_for(target).collect();
    for (ch, cc) in &learned {
        ui.label(format!("Learned: channel {} CC {cc}", ch + 1));
    }
    if !learned.is_empty() && ui.button("Forget learned controller").clicked() {
        shared.forget(target);
        ui.close_menu();
    }
}

fn fader_cell(ui: &mut Ui, setter: &ParamSetter, params: &VDrumParams, shared: &Shared, i: usize, width: f32) {
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
    let target = LEARN_FADER + i as u16;
    learn_marks(ui, rect, shared, target);
    resp.context_menu(|ui| {
        learn_menu(ui, shared, target);
        if ui.button("Reset to top (0 dB)").clicked() {
            setter.begin_set_parameter(p);
            setter.set_parameter(p, FADER_MAX);
            setter.end_set_parameter(p);
            ui.close_menu();
        }
    });
    resp.on_hover_ui(|ui| {
        for (ch, cc) in shared.learned_for(target) {
            ui.label(format!("Learned: channel {} CC {cc}", ch + 1));
        }
        let voice = Voice::VOICES.get(i).copied().unwrap_or(Voice::OpenHat);
        ui.label(RichText::new(format!("{} Fader", voice.label())).strong());
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
    learn_marks(ui, rect, shared, idx);

    // MIDI learn menu
    resp.context_menu(|ui| {
        learn_menu(ui, shared, idx);
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
        let learned: Vec<(u8, u8)> = shared.learned_for(idx).collect();
        if learned.is_empty() {
            ui.label("MIDI: not learned (right-click for MIDI learn)");
        }
        for (ch, cc) in learned {
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
                            // (the hi-hats' shared choke switch is on both hi-hat cards)
                            let choke = *idx == oh_index(OH_CHOKE);
                            let voice = Voice::VOICES.get(v).copied().unwrap_or(Voice::OpenHat);
                            assert!(lxr_params::info(*idx).unwrap().voice == voice || (choke && v == 5), "{idx}");
                            assert!(!label.is_empty());
                            assert!(!shown.contains(idx) || choke, "{idx} shown twice");
                            shown.push(*idx);
                        }
                        Item::Fader(i) => faders.push(*i),
                    }
                }
            }
        }
        assert_eq!(faders, vec![0, 1, 2, 3, 4, 5, 6]);
        shown.extend(&l.fx);
        shown.extend(&l.performance);
        shown.extend(MIDI_CHAN);
        shown.extend(MIDI_NOTE);
        shown.extend([par::BPM, par::MIDI_CHAN_GLOBAL]);
        for info in params.infos.iter() {
            // the firmware's open decay has no effect with the separate open hi-hat (its decay is
            // `velod7` on the open hi-hat's card); it stays a host parameter for old automation
            if info.index == par::VELOD6_OPEN {
                assert!(!shown.contains(&info.index));
                continue;
            }
            assert!(shown.contains(&info.index), "{} not shown", info.enum_name);
        }
    }

    /// A host that accepts every change and resize.
    struct NullContext;
    impl GuiContext for NullContext {
        fn plugin_api(&self) -> PluginApi {
            PluginApi::Clap
        }
        fn request_resize(&self) -> bool {
            true
        }
        unsafe fn raw_begin_set_parameter(&self, _: ParamPtr) {}
        unsafe fn raw_set_parameter_normalized(&self, _: ParamPtr, _: f32) {}
        unsafe fn raw_end_set_parameter(&self, _: ParamPtr) {}
        fn get_state(&self) -> PluginState {
            unimplemented!()
        }
        fn set_state(&self, _: PluginState) {}
    }

    /// Lays the editor out (without rendering) in a window of `size` logical pixels and returns
    /// the scrolled content's size and its viewport's size, in points.
    fn layout(size: (u32, u32), zoom: f32) -> (egui::Vec2, egui::Vec2) {
        let shared = Arc::new(Shared::new());
        let params = VDrumParams::new(shared.clone());
        params.ui_scale.store(zoom, Ordering::Relaxed);
        let egui_state = EguiState::from_size(size.0, size.1);
        let mut data = Data::new();
        let ctx = egui::Context::default();
        setup(&ctx, zoom);
        let setter = ParamSetter::new(&NullContext);
        // The integration passes the window in points: logical pixels / zoom.
        for _ in 0..3 {
            let input = egui::RawInput {
                screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(size.0 as f32, size.1 as f32) / ctx.zoom_factor())),
                ..Default::default()
            };
            let _ = ctx.run(input, |ctx| update(ctx, &setter, &mut data, &params, &shared, &egui_state));
        }
        assert!((ctx.zoom_factor() - zoom).abs() < 1e-6);
        (data.content_size, data.viewport_size)
    }

    #[test]
    fn library_panel_imports_after_confirmation() {
        let shared = Arc::new(Shared::new());
        let params = VDrumParams::new(shared.clone());
        let egui_state = EguiState::from_size(DEFAULT_SIZE.0, DEFAULT_SIZE.1);
        let kit = |name: &str, coarse: u8| {
            let mut k = Kit::init();
            k.set_name(name);
            k.values[par::COARSE1 as usize] = coarse;
            k.to_file_bytes().to_vec()
        };
        shared.library.lock().unwrap().slots[1] = Some(library::Entry { kit: Kit::from_file_bytes(&kit("OLD", 3)).unwrap(), source: String::new() });
        let dir = std::env::temp_dir().join(format!("vdrum-libpanel-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("01-NEW.SND"), kit("NEW", 5)).unwrap();
        std::fs::write(dir.join("02-INITK.SND"), Kit::init().to_file_bytes()).unwrap();
        let mut pat = crate::seq::PatternData::default();
        pat.tracks[0].steps[4].on = true;
        let pat_file = |p: &crate::seq::PatternData| lxr_files::pat_to_bytes(&lxr_files::PatFile { name: *b"GROOVE\0\0", pattern: p.clone(), kit_nr: 1 });
        std::fs::write(dir.join("03-GROOV.PAT"), pat_file(&pat)).unwrap();
        std::fs::write(dir.join("04-.PAT"), pat_file(&Default::default())).unwrap();
        let mut data = Data::new();
        *data.file_reply.lock().unwrap() = Some(FileReply::ImportFolder(dir.clone()));
        let ctx = egui::Context::default();
        setup(&ctx, 1.0);
        let setter = ParamSetter::new(&NullContext);
        let frame = |data: &mut Data| {
            let input = egui::RawInput {
                screen_rect: Some(Rect::from_min_size(pos2(0.0, 0.0), vec2(DEFAULT_SIZE.0 as f32, DEFAULT_SIZE.1 as f32))),
                ..Default::default()
            };
            let _ = ctx.run(input, |ctx| update(ctx, &setter, data, &params, &shared, &egui_state));
        };
        frame(&mut data);
        // slot 01 is taken: the import waits for confirmation in the open panel
        assert!(data.library_open);
        let pending = data.pending_import.as_ref().unwrap();
        assert_eq!(pending.kits.replaces(), vec!["01 OLD".to_string()]);
        assert_eq!(pending.patterns.puts.len(), 1);
        assert!(pending.ask_patterns);
        assert_eq!(shared.library.lock().unwrap().slots[1].as_ref().unwrap().kit.name_str().trim_end(), "OLD");
        frame(&mut data);
        // confirmed, with the patterns
        let p = data.pending_import.take().unwrap();
        assert_eq!(apply_import(&shared, &p, true), "Imported 1 kit (1 init kits skipped), 1 pattern (1 empty skipped)");
        assert_eq!(shared.library.lock().unwrap().len(), 1);
        assert_eq!(shared.seq.lock().unwrap().patterns[3], pat);
        assert_eq!(lxr_files::name_text(&shared.library.lock().unwrap().pattern_names[3]), "GROOVE");
        assert!(matches!(shared.commands.pop(), Some(Command::SeqReload)));
        // a pattern import that replaces pattern 04 asks first; one into empty patterns does not
        *data.file_reply.lock().unwrap() = Some(FileReply::ImportPatterns(vec![dir.join("03-GROOV.PAT")]));
        frame(&mut data);
        assert!(data.pending_import.take().is_some());
        std::fs::write(dir.join("loose.pat"), pat_file(&pat)).unwrap();
        *data.file_reply.lock().unwrap() = Some(FileReply::ImportPatterns(vec![dir.join("loose.pat")]));
        frame(&mut data);
        assert!(data.pending_import.is_none());
        assert_eq!(data.status, "Imported 1 pattern");
        assert_eq!(shared.seq.lock().unwrap().patterns[0], pat, "first empty pattern");
        // project export asks first, listing the losses
        let out = dir.join("export");
        *data.file_reply.lock().unwrap() = Some(FileReply::ExportProject(out.clone()));
        frame(&mut data);
        assert_eq!(data.pending_export, Some((out.clone(), 0)));
        data.show_patterns = true;
        frame(&mut data);
        data.pending_export = None;
        assert!(export_project(&shared, &out).starts_with("Exported 194 files"));
        assert_eq!(std::fs::read(out.join("03-GROOV.PAT")).unwrap(), pat_file(&pat));
        // loading and storing go through the shared state
        do_lib_action(&mut data, &shared, LibAction::Load(1));
        assert_eq!(shared.kit_name(), "NEW");
        do_lib_action(&mut data, &shared, LibAction::Rename(1, "RENAMED".into()));
        assert_eq!(shared.kit_name(), "RENAMED");
        do_lib_action(&mut data, &shared, LibAction::Move(1, 1));
        assert_eq!(shared.library_selected(), Some(2));
        do_lib_action(&mut data, &shared, LibAction::Clear(2));
        assert!(shared.library.lock().unwrap().is_empty());
        assert_eq!(shared.library_selected(), None);
        frame(&mut data);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn default_size_fits_everything() {
        let (content, viewport) = layout(DEFAULT_SIZE, 1.0);
        println!("content {content:?}, viewport {viewport:?}");
        assert!(content.x <= viewport.x && content.y <= viewport.y, "content {content:?} viewport {viewport:?}");
        // Not much spare room either: the default size is the content's size.
        assert!(viewport.x - content.x < 24.0 && viewport.y - content.y < 24.0, "content {content:?} viewport {viewport:?}");
    }

    #[test]
    fn zoomed_window_fits_everything() {
        // The zoom buttons scale the window with the zoom, so the content still fits.
        for z in [0.5, 0.8, 1.2, 1.5, 2.0] {
            let size = ((DEFAULT_SIZE.0 as f32 * z).round() as u32, (DEFAULT_SIZE.1 as f32 * z).round() as u32);
            let (content, viewport) = layout(size, z);
            assert!(content.x <= viewport.x + 1.0 && content.y <= viewport.y + 1.0, "{z}: content {content:?} viewport {viewport:?}");
        }
    }

    #[test]
    fn zoom_scales_the_window_request() {
        let shared = Arc::new(Shared::new());
        let params = VDrumParams::new(shared);
        let state = EguiState::from_size(1000, 500);
        set_zoom(&params, &state, 1.2);
        assert_eq!(state.requested_size(), Some((1200, 600)));
        assert!((params.ui_scale.load(Ordering::Relaxed) - 1.2).abs() < 1e-6);
        // A second click before the host has resized scales the pending request.
        set_zoom(&params, &state, 1.5);
        assert_eq!(state.requested_size(), Some((1500, 750)));
        set_zoom(&params, &state, 9.0);
        assert!((params.ui_scale.load(Ordering::Relaxed) - MAX_ZOOM).abs() < 1e-6);
        set_zoom(&params, &state, 0.0);
        assert!((params.ui_scale.load(Ordering::Relaxed) - MIN_ZOOM).abs() < 1e-6);
    }

    #[test]
    fn default_size_fits_laptop_screens() {
        // Logical resolutions of MacBook defaults minus the menu bar (37 pt with the notch), a
        // host's plugin window header (about 60 pt in Logic) and the Dock (about 60 pt when not
        // hidden): the MacBook Pro 16" (1728 × 1117) at 100 %, the MacBook Air 13" (1470 × 956)
        // at 90 % zoom.
        let fits = |(w, h): (u32, u32), z: f32| DEFAULT_SIZE.0 as f32 * z <= w as f32 && DEFAULT_SIZE.1 as f32 * z <= (h - 37 - 60 - 60) as f32;
        assert!(fits((1728, 1117), 1.0), "{DEFAULT_SIZE:?}");
        assert!(fits((1470, 956), 0.9), "{DEFAULT_SIZE:?}");
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
