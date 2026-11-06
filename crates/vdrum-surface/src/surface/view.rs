// SPDX-License-Identifier: GPL-3.0-only
//! A snapshot of what the surface shows, for the plugin editor's mode tabs: every control of the
//! current mode with its label, colours and value, the active context layers, the last hardware
//! edit and the screen.
//!
//! The editor never runs surface logic itself: it draws a [`View`] and sends its mouse actions back
//! as [`lcxl3::Input`]s, which go through [`Surface::handle_editor`].

use super::*;

/// One control of the current mode.
#[derive(Clone, Debug, PartialEq)]
pub struct ControlView {
    pub control: Control,
    /// The label (FX parameters are named after the FX type).
    pub label: String,
    /// The section's name (popup title) and colour.
    pub section: &'static str,
    pub colour: Rgb,
    /// The LED colour the device shows (encoders and buttons; faders have no LED).
    pub led: Rgb,
    /// The value as the screen shows it, if the control has one.
    pub value: Option<String>,
    /// The value's position in its range, 0..=1 (encoder arcs, fader value markers).
    pub frac: Option<f32>,
    /// Parameter ID and full name, for engine parameters.
    pub param: Option<(&'static str, &'static str)>,
    /// Faders: the hardware position (0..=127) if the fader has moved, and whether it has picked
    /// up the value.
    pub fader: Option<(Option<u8>, bool)>,
}

/// The last edit made on the hardware (not through [`Surface::handle_editor`]).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EditView {
    /// Changes with every edit.
    pub gen: u32,
    pub section: String,
    pub name: String,
    pub value: String,
    pub colour: Rgb,
}

/// What the surface shows.
#[derive(Clone, Debug, PartialEq)]
pub struct View {
    pub mode: Mode,
    pub controls: Vec<ControlView>,
    /// LEDs of the buttons outside the layout (navigation, Play, Record).
    pub nav_leds: Vec<(Control, Rgb)>,
    /// Active context layers, as short texts ("SHIFT: select mode", "STEP 5 HELD: locks").
    pub layers: Vec<String>,
    pub edit: Option<EditView>,
    /// The stationary screen lines.
    pub screen: [String; 3],
}

impl View {
    pub fn control(&self, c: Control) -> Option<&ControlView> {
        self.controls.iter().find(|v| v.control == c)
    }
}

/// The range and current value of a control that has one.
pub(super) fn range_of(s: &Surface, cell: &Cell, model: &dyn Model) -> Option<(u16, u16, u16)> {
    Some(match cell.binding {
        Binding::Param(idx) => {
            let info = lxr_params::info(idx)?;
            (info.min, info.max, model.param(idx))
        }
        Binding::VoiceFader(t) => (0, FADER_MAX as u16, model.voice_fader(t).clamp(0, FADER_MAX) as u16),
        Binding::SeqDensity(t) => (0, 100, model.seq().density[t as usize % 7] as u16),
        Binding::Macro(n) => (0, 127, s.macro_pos[n as usize % SLOTS] as u16),
        Binding::MacroDepth(n) => (0, 200, s.macros[n as usize % SLOTS].depth),
        Binding::LfoRateScale => (0, 200, s.scalers[0].pct as u16),
        Binding::LfoDepthScale => (0, 200, s.scalers[1].pct as u16),
        Binding::VelScale => (0, 200, s.scalers[2].pct as u16),
        Binding::SeqDensityAll => (0, 100, model.seq().density_all as u16),
        Binding::MomentDepth => (0, 127, s.moment_depth as u16),
        Binding::MomentRate => (0, 127, s.moment_rate as u16),
        Binding::SeqEntryVelocity => (1, 127, s.entry_velocity as u16),
        Binding::TrackMorph(t) => (0, 127, s.morph[t as usize % 7].amount as u16),
        Binding::MorphAll => (0, 127, s.morph_all as u16),
        Binding::SeqShuffle => (0, 127, model.seq().patterns.get(model.seq_status().pattern as usize).map_or(0, |p| p.shuffle) as u16),
        Binding::SeqLength(t) => {
            let st = model.seq_status();
            (1, 64, model.seq().track(st.pattern, t).length as u16)
        }
        Binding::SeqFills(t) => {
            let st = model.seq_status();
            let d = model.seq();
            let tr = d.track(st.pattern, t);
            (0, tr.length as u16, tr.fills as u16)
        }
        Binding::SeqRotate(t) => {
            let st = model.seq_status();
            let d = model.seq();
            let tr = d.track(st.pattern, t);
            (0, tr.length.saturating_sub(1) as u16, tr.rotate as u16)
        }
        _ => return None,
    })
}

impl Surface {
    /// Handle input that comes from the plugin editor (mouse edits in a mode tab): the same as
    /// [`Surface::handle`], except that it does not count as a hardware edit in [`View::edit`].
    pub fn handle_editor(&mut self, input: &Input, now: u64, model: &dyn Model) -> Vec<Action> {
        self.editor_input = true;
        let out = self.handle(input, now, model);
        self.editor_input = false;
        out
    }

    /// The last hardware edit was made at `gen`; recorded by `show`.
    pub(super) fn record_edit(&mut self, section: String, name: &str, value: &str, colour: Rgb) {
        if self.editor_input {
            return;
        }
        self.edit_gen = self.edit_gen.wrapping_add(1);
        self.last_edit = Some(EditView { gen: self.edit_gen, section, name: name.to_string(), value: value.to_string(), colour });
    }

    /// The active context layers.
    fn layers(&self, model: &dyn Model) -> Vec<String> {
        let mut v = Vec::new();
        if self.shift {
            v.push("SHIFT: select mode".to_string());
        }
        if self.mode != Mode::Sequencer && self.view {
            let sticky = if self.sticky { " (sticky)" } else { "" };
            v.push(format!("STEPS: {} {}{sticky}", self.seq_track + 1, TRACKS[self.seq_track as usize]));
        }
        if self.seq_view() {
            if self.is_held(Control::Button(Button::MuteSelect)) {
                v.push("PATTERN: select".to_string());
            }
            for h in &self.held_steps {
                v.push(format!("STEP {} HELD: locks", h.step + 1));
            }
        }
        if self.accent {
            v.push("ACCENT".to_string());
        }
        if self.browsing {
            v.push("BROWSING KITS: Play loads".to_string());
        }
        if self.comparing {
            v.push("COMPARE: saved kit".to_string());
        }
        if let Some((n, _)) = &self.learning {
            v.push(format!("LEARNING MACRO {}", n + 1));
        }
        for m in &self.moments {
            let c = Control::Button(if m.k < 8 { Button::Top(m.k) } else { Button::Bottom(m.k - 8) });
            let name = self.layout.get(c).map(|c| c.label.clone()).unwrap_or_else(|| format!("{}", m.k + 1));
            v.push(format!("MOMENT: {name}"));
        }
        for h in &self.held {
            if h.fired || matches!(h.control, Control::Button(Button::Shift)) {
                continue;
            }
            if let Some(cell) = self.layout.get(h.control) {
                let holds = matches!(cell.binding, Binding::PasteVoice | Binding::Snapshot(_) | Binding::InitKit | Binding::SaveKit);
                if holds {
                    v.push(format!("HOLD 1 s: {}", self.label(h.control, model).unwrap_or_default()));
                }
            }
        }
        v
    }

    /// The snapshot the editor draws.
    pub fn view(&self, now: u64, model: &dyn Model) -> View {
        let frame = self.frame(now, model);
        let mut controls = Vec::new();
        for (c, cell) in self.layout.iter() {
            let led = frame.leds.iter().find(|(k, _)| k == c).map(|(_, rgb)| *rgb).unwrap_or((0, 0, 0));
            let range = range_of(self, cell, model);
            let value = range.map(|(_, _, cur)| match cell.binding {
                Binding::Param(_) => self.value_text(cell, cur, model),
                _ => cur.to_string(),
            });
            let frac = range.map(|(min, max, cur)| if max > min { (cur.saturating_sub(min)) as f32 / (max - min) as f32 } else { 0.0 });
            let param = match cell.binding {
                Binding::Param(idx) => lxr_params::info(idx).map(|i| (i.id, i.name)),
                _ => None,
            };
            let fader = match c {
                Control::Fader(col) => {
                    let f = self.faders[*col as usize];
                    Some((f.raw, f.picked || !self.settings.fader_pickup))
                }
                _ => None,
            };
            controls.push(ControlView {
                control: *c,
                label: self.label(*c, model).unwrap_or_default(),
                section: cell.section.label(),
                colour: self.colour(cell.section),
                led,
                value,
                frac,
                param,
                fader,
            });
        }
        let in_layout = |c: &Control| self.layout.get(*c).is_some();
        let nav_leds = frame.leds.iter().filter(|(c, _)| !in_layout(c)).copied().collect();
        View { mode: self.mode, controls, nav_leds, layers: self.layers(model), edit: self.last_edit.clone(), screen: frame.stationary }
    }
}
