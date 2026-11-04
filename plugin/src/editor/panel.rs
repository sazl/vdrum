// SPDX-License-Identifier: GPL-3.0-only
//! The editor's mode tabs: a drawing of the Launch Control XL 3 front panel with the current mode's
//! controls, labelled and in sync with the hardware, the "now editing" and context pills, and the
//! tab bar.
//!
//! The drawing comes from the controller thread's [`View`]; mouse actions go back to the surface as
//! device input ([`EditorInput`]), so they behave as on the device. Positions follow the design's
//! `docs/lcxl3-daw-mode/source/render.py` (panel units, scaled to the window).

use super::{Color32, FontId, Rect, Sense, Stroke, StrokeKind, Ui};
use crate::params::{EditorInput, Shared, VDrumParams};
use lcxl3::{Button, Control, Input};
use nih_plug::prelude::ParamSetter;
use nih_plug_egui::egui::{self, pos2, vec2, Align2, CornerRadius, Pos2, RichText};
use std::sync::atomic::Ordering;
use vdrum_surface::{ControlView, Mode, View};

/// How long the "now editing" pill stays after the last hardware edit, in seconds.
pub const EDIT_PILL_S: f64 = 1.5;
/// Mouse travel per encoder step, in points.
const ENCODER_PX: f32 = 4.0;

/// Panel units (from `render.py`): column centres, encoder row centres, fader and button rows.
const COLX0: f32 = 378.0;
const COLDX: f32 = 153.3;
const ROWY: [f32; 3] = [185.0, 378.0, 572.0];
const FADER_TOP: f32 = 730.0;
const FADER_BOTTOM: f32 = 1180.0;
const TOP_Y: f32 = 1300.0;
const BOT_Y: f32 = 1420.0;
const CELL_W: f32 = 146.0;
const CELL_H: f32 = 150.0;
const BTN_H: f32 = 72.0;
/// The panel's extent in panel units.
const PANEL: Rect = Rect { min: Pos2 { x: 50.0, y: 110.0 }, max: Pos2 { x: 1540.0, y: 1470.0 } };

/// A button to draw: the button, its rectangle, its control view, its LED and its label.
type ButtonCell<'a> = (Button, Rect, Option<&'a ControlView>, Option<(u8, u8, u8)>, String);

/// Editor-side state of the mode tabs.
#[derive(Default)]
pub struct PanelState {
    /// Encoder drag: the control and the travel not yet sent.
    encoder_drag: Option<(Control, f32)>,
    /// Fader drag: the fader and its position (0..=127) as last sent.
    fader_drag: Option<(u8, f32)>,
    /// The button pressed with the left mouse button (released with it).
    pressed: Option<Button>,
    /// Buttons latched with the right mouse button (held until right-clicked again).
    pub latched: Vec<Button>,
    /// The hardware edit last shown and when it arrived (egui time).
    edit_seen: u32,
    edit_at: f64,
}

fn sk(width: f32, colour: Color32) -> Stroke {
    Stroke::new(width, colour)
}

fn rgb(c: (u8, u8, u8)) -> Color32 {
    Color32::from_rgb(c.0, c.1, c.2)
}

/// Black or white text, whichever reads on `bg`.
fn text_on(bg: (u8, u8, u8)) -> Color32 {
    let l = 0.299 * bg.0 as f32 + 0.587 * bg.1 as f32 + 0.114 * bg.2 as f32;
    if l > 140.0 {
        Color32::from_rgb(20, 20, 24)
    } else {
        Color32::from_rgb(245, 245, 245)
    }
}

/// The tab of a mode: 1..=13 (0 is Params).
pub fn tab_of(mode: Mode) -> u8 {
    Mode::ALL.iter().position(|m| *m == mode).map_or(0, |i| i as u8 + 1)
}

pub fn mode_of(tab: u8) -> Option<Mode> {
    (tab as usize).checked_sub(1).and_then(|i| Mode::ALL.get(i).copied())
}

/// Short tab names.
fn tab_name(mode: Mode) -> String {
    format!("{} {}", mode.number(), mode.name())
}

fn send(shared: &Shared, i: EditorInput) {
    let _ = shared.surface_input.push(i);
}

/// The tab bar: Params and the 13 modes, "Follow controller" and the connection state. Returns
/// the tab clicked.
pub fn tab_bar(ui: &mut Ui, params: &VDrumParams, shared: &Shared, tab: u8) -> Option<u8> {
    let mut clicked = None;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 2.0;
        if ui.selectable_label(tab == 0, "Params").on_hover_text("All parameters").clicked() {
            clicked = Some(0);
        }
        ui.add_space(4.0);
        for m in Mode::ALL {
            let t = tab_of(m);
            let r = ui.selectable_label(tab == t, tab_name(m)).on_hover_text(format!("Launch Control XL 3 mode {}: {}", m.number(), m.name()));
            if r.clicked() {
                clicked = Some(t);
            }
        }
        ui.add_space(8.0);
        let mut follow = params.follow_controller.load(Ordering::Relaxed);
        if ui
            .checkbox(&mut follow, "Follow controller")
            .on_hover_text("Switch to the controller's mode tab when the mode changes on the controller, also from Params")
            .changed()
        {
            params.follow_controller.store(follow, Ordering::Relaxed);
        }
        let (text, col) = if !shared.surface_running.load(Ordering::Relaxed) {
            ("Controller: off", Color32::from_gray(120))
        } else if shared.controller_connected.load(Ordering::Relaxed) {
            ("Controller: connected", Color32::from_rgb(110, 200, 120))
        } else {
            ("Controller: not connected", Color32::from_rgb(200, 160, 90))
        };
        ui.label(RichText::new(text).size(10.5).color(col)).on_hover_text(
            "The mode tabs work without a controller: they show the values and accept mouse edits. VDRUM_LCXL3=0 turns the controller off.",
        );
    });
    clicked
}

/// The pill row: the last hardware edit (filled, 1.5 s) and the active context layers (outlined).
pub fn pills(ui: &mut Ui, view: Option<&View>, st: &mut PanelState) {
    let now = ui.input(|i| i.time);
    ui.horizontal(|ui| {
        ui.set_min_height(18.0);
        ui.spacing_mut().item_spacing.x = 6.0;
        let Some(v) = view else { return };
        if let Some(e) = &v.edit {
            if e.gen != st.edit_seen {
                st.edit_seen = e.gen;
                st.edit_at = now;
            }
            if now - st.edit_at < EDIT_PILL_S {
                let text = format!("{} · {} · {}", e.section.to_uppercase(), e.name, e.value);
                pill(ui, &text, Some(e.colour), rgb(e.colour));
                ui.ctx().request_repaint_after(std::time::Duration::from_millis(100));
            }
        }
        for l in &v.layers {
            pill(ui, l, None, Color32::from_rgb(200, 205, 215));
        }
        for b in &st.latched {
            pill(ui, &format!("LATCHED: {}", button_name(*b)), None, Color32::from_rgb(240, 170, 60));
        }
    });
}

fn pill(ui: &mut Ui, text: &str, fill: Option<(u8, u8, u8)>, stroke: Color32) {
    let font = FontId::proportional(11.0);
    let galley = ui.painter().layout_no_wrap(text.to_string(), font, fill.map_or(stroke, text_on));
    let size = galley.size() + vec2(14.0, 4.0);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    let r = CornerRadius::same((size.y / 2.0) as u8);
    match fill {
        Some(c) => {
            ui.painter().rect_filled(rect, r, rgb(c));
        }
        None => {
            ui.painter().rect_stroke(rect, r, sk(1.0, stroke), StrokeKind::Inside);
        }
    }
    ui.painter().galley(rect.center() - galley.size() / 2.0, galley, Color32::WHITE);
}

fn button_name(b: Button) -> String {
    match b {
        Button::Top(c) => format!("button {}", c + 1),
        Button::Bottom(c) => format!("button {}", c + 9),
        Button::SoloArm => "Solo/Arm".into(),
        Button::MuteSelect => "Mute/Select".into(),
        Button::Logo => "Logo".into(),
        Button::PageUp => "Page up".into(),
        Button::PageDown => "Page down".into(),
        Button::TrackLeft => "Track left".into(),
        Button::TrackRight => "Track right".into(),
        Button::Play => "Play".into(),
        Button::Record => "Record".into(),
        Button::Shift => "Shift".into(),
    }
}

/// Maps panel units into the drawing area.
struct Map {
    area: Rect,
    sx: f32,
    sy: f32,
}

impl Map {
    fn new(area: Rect) -> Map {
        Map { area, sx: area.width() / PANEL.width(), sy: area.height() / PANEL.height() }
    }
    fn p(&self, x: f32, y: f32) -> Pos2 {
        pos2(self.area.min.x + (x - PANEL.min.x) * self.sx, self.area.min.y + (y - PANEL.min.y) * self.sy)
    }
    /// A rectangle of `w` × `h` panel units centred at (`x`, `y`).
    fn r(&self, x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::from_center_size(self.p(x, y), vec2(w * self.sx, h * self.sy))
    }
    fn font(&self, size: f32) -> FontId {
        FontId::proportional((size * self.sx.min(self.sy) * 2.2).clamp(8.0, 14.0))
    }
}

fn colx(c: u8) -> f32 {
    COLX0 + COLDX * c as f32
}

/// The side buttons: (button, centre x, centre y, width, height) in panel units.
fn side_buttons() -> Vec<(Button, f32, f32, f32, f32)> {
    vec![
        (Button::PageUp, 110.0, 380.0, 76.0, 78.0),
        (Button::PageDown, 190.0, 380.0, 76.0, 78.0),
        (Button::TrackLeft, 110.0, 572.0, 76.0, 78.0),
        (Button::TrackRight, 190.0, 572.0, 76.0, 78.0),
        (Button::Record, 149.0, 783.0, 156.0, 74.0),
        (Button::Play, 149.0, 862.0, 156.0, 74.0),
        (Button::Shift, 149.0, 1050.0, 156.0, 74.0),
        (Button::SoloArm, 149.0, TOP_Y, 150.0, BTN_H),
        (Button::MuteSelect, 149.0, BOT_Y, 150.0, BTN_H),
        (Button::Logo, 263.0, TOP_Y, 70.0, BTN_H),
    ]
}

/// What a mouse action on the panel does.
enum Act {
    Send(EditorInput),
    /// Double-click: reset an engine parameter to its default.
    Reset(u16),
}

/// Draws the front panel for `view` and handles mouse edits.
pub fn front_panel(ui: &mut Ui, setter: &ParamSetter, params: &VDrumParams, shared: &Shared, view: Option<&View>, st: &mut PanelState) {
    let avail = ui.available_rect_before_wrap();
    let (area, _) = ui.allocate_exact_size(avail.size(), Sense::hover());
    let painter = ui.painter_at(area);
    painter.rect_filled(area, 10.0, Color32::from_rgb(14, 14, 16));
    painter.rect(area.shrink(3.0), 9.0, PANEL_FILL, sk(1.5, PANEL_EDGE), StrokeKind::Inside);
    painter.line_segment([pos2(area.min.x + 14.0, area.min.y + 5.0), pos2(area.max.x - 14.0, area.min.y + 5.0)], sk(1.0, Color32::from_white_alpha(18)));
    let Some(view) = view else {
        painter.text(area.center(), Align2::CENTER_CENTER, "Waiting for the controller surface…", FontId::proportional(14.0), Color32::GRAY);
        return;
    };
    let map = Map::new(area.shrink(8.0));
    let mut acts: Vec<Act> = Vec::new();

    // screen
    let screen = map.r(150.0, 190.0, 176.0, 106.0);
    painter.rect(screen.expand(4.0), 6.0, Color32::from_rgb(10, 10, 12), sk(1.0, Color32::from_gray(60)), StrokeKind::Inside);
    painter.rect_filled(screen, 2.0, Color32::BLACK);
    for (i, line) in view.screen.iter().enumerate() {
        let y = screen.min.y + screen.height() * (0.2 + 0.3 * i as f32);
        painter.text(pos2(screen.center().x, y), Align2::CENTER_CENTER, line, FontId::monospace(map.font(15.0).size), Color32::from_rgb(225, 232, 245));
    }

    // encoders
    for row in 0..3u8 {
        for col in 0..8u8 {
            let c = Control::Encoder { row, col };
            let rect = map.r(colx(col), ROWY[row as usize], CELL_W, CELL_H);
            let id = ui.id().with(("enc", row, col));
            let resp = ui.interact(rect, id, Sense::click_and_drag());
            let cv = view.control(c);
            draw_encoder(&painter, &map, rect, cv, resp.hovered() || resp.dragged());
            if resp.drag_started() {
                st.encoder_drag = Some((c, 0.0));
            }
            if resp.dragged() {
                if let Some((dc, acc)) = st.encoder_drag.as_mut().filter(|(dc, _)| *dc == c) {
                    let _ = dc;
                    let d = resp.drag_delta();
                    *acc += d.x - d.y;
                    let steps = (*acc / ENCODER_PX).trunc();
                    if steps != 0.0 {
                        *acc -= steps * ENCODER_PX;
                        let fine = ui.input(|i| i.modifiers.shift);
                        let delta = if fine { steps.signum() } else { steps }.clamp(-63.0, 63.0) as i8;
                        acts.push(Act::Send(EditorInput::Input(Input::Relative { row, col, delta })));
                    }
                }
            }
            if resp.drag_stopped() {
                st.encoder_drag = None;
            }
            control_extras(&resp, cv, &mut acts);
        }
    }

    // faders
    for col in 0..8u8 {
        let c = Control::Fader(col);
        let cv = view.control(c);
        let track = map.r(colx(col), (FADER_TOP + FADER_BOTTOM) / 2.0, 40.0, FADER_BOTTOM - FADER_TOP);
        let hit = map.r(colx(col), (FADER_TOP + FADER_BOTTOM) / 2.0, CELL_W, FADER_BOTTOM - FADER_TOP + 60.0);
        let resp = ui.interact(hit, ui.id().with(("fader", col)), Sense::click_and_drag());
        draw_fader(&painter, &map, col, track, cv, resp.hovered() || resp.dragged());
        let y_to_raw = |y: f32| ((track.max.y - y) / track.height() * 127.0).clamp(0.0, 127.0);
        if resp.drag_started() {
            // start at the parameter's value, so the fader picks it up at once
            let start = cv.and_then(|v| v.frac).map(|f| f * 127.0).unwrap_or_else(|| resp.interact_pointer_pos().map_or(0.0, |p| y_to_raw(p.y)));
            st.fader_drag = Some((col, start));
            acts.push(Act::Send(EditorInput::Input(Input::Analogue { control: c, value: start.round() as u8 })));
        }
        if resp.dragged() {
            if let Some((_, pos)) = st.fader_drag.as_mut().filter(|(dc, _)| *dc == col) {
                let fine = ui.input(|i| i.modifiers.shift);
                let dy = -resp.drag_delta().y / track.height() * 127.0 * if fine { 0.25 } else { 1.0 };
                let old = pos.round();
                *pos = (*pos + dy).clamp(0.0, 127.0);
                if pos.round() != old {
                    acts.push(Act::Send(EditorInput::Input(Input::Analogue { control: c, value: pos.round() as u8 })));
                }
            }
        }
        if resp.drag_stopped() {
            st.fader_drag = None;
        }
        control_extras(&resp, cv, &mut acts);
    }

    // button rows and side buttons
    let mut buttons: Vec<ButtonCell> = Vec::new();
    for col in 0..8u8 {
        for (b, y) in [(Button::Top(col), TOP_Y), (Button::Bottom(col), BOT_Y)] {
            let cv = view.control(Control::Button(b));
            buttons.push((b, map.r(colx(col), y, CELL_W, BTN_H), cv, cv.map(|v| v.led), cv.map(|v| v.label.clone()).unwrap_or_default()));
        }
    }
    for (b, x, y, w, h) in side_buttons() {
        let led = view.nav_leds.iter().find(|(c, _)| *c == Control::Button(b)).map(|(_, l)| *l);
        buttons.push((b, map.r(x, y, w, h), None, led, button_name(b)));
    }
    let primary_down = ui.input(|i| i.pointer.primary_down());
    for (b, rect, cv, led, label) in buttons {
        let resp = ui.interact(rect, ui.id().with(("btn", format!("{b:?}"))), Sense::click_and_drag());
        let held = st.pressed == Some(b) || st.latched.contains(&b);
        draw_button(&painter, &map, rect, cv, led, &label, held, resp.hovered());
        if resp.is_pointer_button_down_on() && ui.input(|i| i.pointer.primary_pressed()) && st.pressed.is_none() && !st.latched.contains(&b) {
            st.pressed = Some(b);
            acts.push(Act::Send(EditorInput::Input(Input::Button { button: b, pressed: true })));
        }
        if resp.secondary_clicked() {
            if let Some(i) = st.latched.iter().position(|x| *x == b) {
                st.latched.remove(i);
                acts.push(Act::Send(EditorInput::Input(Input::Button { button: b, pressed: false })));
            } else {
                st.latched.push(b);
                acts.push(Act::Send(EditorInput::Input(Input::Button { button: b, pressed: true })));
            }
        }
        let tip = match cv {
            Some(v) => format!("{} ({})\nClick: press; hold the mouse button to hold. Right-click: latch.", v.label, v.section),
            None => format!("{label}\nClick: press. Right-click: latch (for example Shift, then a button)."),
        };
        resp.on_hover_text(tip);
    }
    if let Some(b) = st.pressed {
        if !primary_down {
            st.pressed = None;
            acts.push(Act::Send(EditorInput::Input(Input::Button { button: b, pressed: false })));
        }
    }

    for a in acts {
        match a {
            Act::Send(i) => send(shared, i),
            Act::Reset(idx) => {
                if let (Some(p), Some(info)) = (params.param(idx), lxr_params::info(idx)) {
                    let d = crate::params::plugin_default(info) as i32;
                    setter.begin_set_parameter(p);
                    setter.set_parameter(p, d);
                    setter.end_set_parameter(p);
                }
            }
        }
    }
}

/// Tooltip and double-click reset for encoders and faders.
fn control_extras(resp: &egui::Response, cv: Option<&ControlView>, acts: &mut Vec<Act>) {
    let Some(v) = cv else { return };
    if resp.double_clicked() {
        if let Some((id, _)) = v.param {
            if let Some(info) = lxr_params::by_id(id) {
                acts.push(Act::Reset(info.index));
            }
        }
    }
    let mut tip = format!("{} ({})", v.label, v.section);
    if let Some((id, name)) = v.param {
        tip += &format!("\n{name} [{id}]");
    }
    if let Some(val) = &v.value {
        tip += &format!("\nValue: {val}");
    }
    if let Some((_, picked)) = v.fader {
        if !picked {
            tip += "\nThe hardware fader has not picked up this value yet.";
        }
    }
    tip += "\nDrag to change (Shift: fine). Double-click: default.";
    resp.clone().on_hover_text(tip);
}

/// Colours of the drawn hardware.
const PANEL_FILL: Color32 = Color32::from_rgb(24, 25, 28);
const PANEL_EDGE: Color32 = Color32::from_rgb(58, 60, 66);
const CAP_DARK: Color32 = Color32::from_rgb(22, 22, 25);
const CAP_RING: Color32 = Color32::from_rgb(52, 53, 58);
const SLOT: Color32 = Color32::from_rgb(8, 8, 10);
const TICK: Color32 = Color32::from_rgb(70, 72, 78);
const BTN_OFF: Color32 = Color32::from_rgb(34, 35, 39);

/// A label chip in the section colour (as the printed labels of the design images).
fn label_chip(p: &egui::Painter, map: &Map, rect: Rect, text: &str, colour: (u8, u8, u8)) {
    p.rect_filled(rect, rect.height() * 0.3, rgb(colour));
    p.text(rect.center(), Align2::CENTER_CENTER, text, map.font(14.0), text_on(colour));
}

fn draw_encoder(p: &egui::Painter, map: &Map, rect: Rect, cv: Option<&ControlView>, hot: bool) {
    let v = cv.filter(|v| !v.label.is_empty());
    let c = pos2(rect.center().x, rect.min.y + rect.height() * 0.57);
    let r = (rect.height() * 0.19).min(rect.width() * 0.2);
    // knob cap: outer skirt, ring, top
    p.circle_filled(c + vec2(0.0, r * 0.08), r * 1.08, Color32::from_rgb(10, 10, 12));
    p.circle_filled(c, r, CAP_RING);
    p.circle_filled(c, r * 0.82, CAP_DARK);
    let Some(v) = v else {
        p.circle_stroke(c, r, sk(1.0, Color32::from_gray(40)));
        return;
    };
    let chip = Rect::from_min_size(pos2(rect.min.x + 2.0, rect.min.y), vec2(rect.width() - 4.0, rect.height() * 0.24));
    label_chip(p, map, chip, &v.label, v.colour);
    // value arc around the cap, in the LED colour
    let ring = r * 1.22;
    let start = std::f32::consts::PI * 0.75;
    let full = std::f32::consts::PI * 1.5;
    let arc = |from: f32, to: f32| -> Vec<Pos2> {
        let n = 32;
        (0..=n).map(|i| {
            let a = start + full * (from + (to - from) * i as f32 / n as f32);
            c + vec2(a.cos(), a.sin()) * ring
        }).collect()
    };
    p.add(egui::Shape::line(arc(0.0, 1.0), sk(3.0, Color32::from_gray(45))));
    let pointer = match v.frac {
        Some(f) => {
            let f = f.clamp(0.0, 1.0);
            if f > 0.0 {
                p.add(egui::Shape::line(arc(0.0, f), sk(3.0, rgb(v.led))));
            }
            start + full * f
        }
        None => {
            // no value: the LED lights the cap's centre
            p.circle_filled(c, r * 0.3, rgb(v.led));
            start
        }
    };
    if v.frac.is_some() {
        let d = vec2(pointer.cos(), pointer.sin());
        p.line_segment([c + d * r * 0.25, c + d * r * 0.8], sk(2.0, Color32::from_gray(225)));
    }
    if hot {
        p.circle_stroke(c, r * 1.42, sk(1.5, Color32::from_gray(200)));
    }
    if let Some(val) = &v.value {
        p.text(pos2(rect.center().x, rect.max.y - rect.height() * 0.04), Align2::CENTER_BOTTOM, val, map.font(13.0), Color32::from_gray(225));
    }
}

fn draw_fader(p: &egui::Painter, map: &Map, col: u8, track: Rect, cv: Option<&ControlView>, hot: bool) {
    let _ = col;
    let slot = Rect::from_center_size(track.center(), vec2(6.0, track.height()));
    // scale marks on both sides
    for i in 0..=10 {
        let y = track.max.y - track.height() * i as f32 / 10.0;
        let w = if i % 5 == 0 { 9.0 } else { 6.0 };
        p.line_segment([pos2(slot.min.x - 5.0 - w, y), pos2(slot.min.x - 5.0, y)], sk(1.0, TICK));
        p.line_segment([pos2(slot.max.x + 5.0, y), pos2(slot.max.x + 5.0 + w, y)], sk(1.0, TICK));
    }
    p.rect(slot, 3.0, SLOT, sk(1.0, Color32::from_gray(48)), StrokeKind::Outside);
    let Some(v) = cv.filter(|v| !v.label.is_empty()) else {
        let cap = Rect::from_center_size(pos2(track.center().x, track.max.y), vec2(track.width() * 1.2, 20.0));
        p.rect(cap, 3.0, CAP_DARK, sk(1.0, Color32::from_gray(40)), StrokeKind::Inside);
        return;
    };
    let chip_h = 22.0 * map.sy.max(0.3) * 1.6;
    let chip = Rect::from_center_size(pos2(track.center().x, track.min.y - 15.0 - chip_h * 0.5), vec2(track.width() * 3.2, chip_h));
    label_chip(p, map, chip, &v.label, v.colour);
    if let Some(val) = &v.value {
        p.text(pos2(track.center().x, track.max.y + 15.0), Align2::CENTER_TOP, val, map.font(13.0), Color32::from_gray(225));
    }
    let y_of = |raw: f32| track.max.y - raw / 127.0 * track.height();
    let (hw, picked) = v.fader.unwrap_or((None, true));
    // the parameter's value: a lit marker beside the slot
    if let Some(f) = v.frac {
        let y = y_of(f * 127.0);
        p.line_segment([pos2(slot.min.x - 16.0, y), pos2(slot.max.x + 16.0, y)], sk(2.0, rgb(v.colour)));
    }
    // the cap at the hardware position (at the value once picked up, or never moved)
    let cap_raw = match (hw, picked) {
        (Some(h), false) => h as f32,
        _ => v.frac.map_or(0.0, |f| f * 127.0),
    };
    let cap = Rect::from_center_size(pos2(track.center().x, y_of(cap_raw)), vec2(track.width() * 1.2, 22.0));
    p.rect_filled(cap.translate(vec2(0.0, 3.0)), 3.0, Color32::from_rgba_unmultiplied(0, 0, 0, 140));
    p.rect(cap, 3.0, if picked { Color32::from_rgb(40, 41, 46) } else { Color32::from_rgb(70, 70, 74) }, sk(if hot { 2.0 } else { 1.0 }, if hot { Color32::from_gray(220) } else { Color32::from_gray(85) }), StrokeKind::Inside);
    // ribs and the centre line (section colour once picked up)
    for dy in [-6.0, 6.0] {
        p.line_segment([pos2(cap.min.x + 4.0, cap.center().y + dy), pos2(cap.max.x - 4.0, cap.center().y + dy)], sk(1.0, Color32::from_gray(62)));
    }
    let centre = if picked { rgb(v.colour) } else { Color32::from_gray(160) };
    p.line_segment([pos2(cap.min.x + 3.0, cap.center().y), pos2(cap.max.x - 3.0, cap.center().y)], sk(2.0, centre));
    if !picked && hw.is_some() {
        p.text(pos2(cap.max.x + 4.0, cap.center().y), Align2::LEFT_CENTER, "pickup", map.font(11.0), Color32::from_gray(170));
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_button(p: &egui::Painter, map: &Map, rect: Rect, cv: Option<&ControlView>, led: Option<(u8, u8, u8)>, label: &str, held: bool, hot: bool) {
    let led = led.unwrap_or((0, 0, 0));
    let lit = led != (0, 0, 0);
    let unused = cv.is_some_and(|v| v.label.is_empty());
    // shadow, body, top highlight
    p.rect_filled(rect.translate(vec2(0.0, 2.0)), 6.0, Color32::from_rgb(8, 8, 10));
    let fill = if unused || !lit { BTN_OFF } else { rgb(led) };
    let stroke = if held {
        sk(2.5, Color32::WHITE)
    } else if hot {
        sk(1.5, Color32::from_gray(210))
    } else {
        sk(1.0, Color32::from_gray(if lit { 120 } else { 62 }))
    };
    p.rect(rect, 6.0, fill, stroke, StrokeKind::Inside);
    p.line_segment([pos2(rect.min.x + 6.0, rect.min.y + 2.5), pos2(rect.max.x - 6.0, rect.min.y + 2.5)], sk(1.0, Color32::from_white_alpha(if lit { 70 } else { 25 })));
    if !unused {
        let tc = if !lit { Color32::from_gray(215) } else { text_on(led) };
        p.text(rect.center(), Align2::CENTER_CENTER, label, map.font(14.0), tc);
    }
}
