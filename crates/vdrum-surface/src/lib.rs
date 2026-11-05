// SPDX-License-Identifier: GPL-3.0-only
//! VDrum's Launch Control XL 3 surface: the design's 16 mode slots, what every control does in each
//! mode, fader pickup, held buttons, and the LED and screen contents, as a state machine without
//! I/O.
//!
//! * Input: messages decoded by [`lcxl3::decode`], with a millisecond clock.
//! * Output: [`Action`]s for the plugin (parameter changes, triggers) and [`Frame`]s (LEDs, screen),
//!   which [`DeviceState`] turns into device messages, sending only what changed.
//! * The plugin's state is read through the [`Model`] trait.
//!
//! The mapping tables follow the design's `lcxl3-daw-mode/source/layout.py` (revision 2).

pub mod frame;
pub mod layout;
pub mod model;
pub mod seq;
pub mod surface;

pub use frame::{DeviceState, Frame, Popup};
pub use layout::{Binding, Cell, Layout, Mode, Section};
pub use model::{Action, Model, Switch};
pub use seq::{SeqData, SeqEdit, SeqStatus, StepData};
pub use surface::{lfo_brightness, pulse, value_brightness, LFO_STEADY, MODE_ACTIVE, MODE_OTHER, ControlView, EditView, Settings, Surface, View, ENCODER_LED_FLOOR, PULSE_FLOOR, PULSE_MS};
