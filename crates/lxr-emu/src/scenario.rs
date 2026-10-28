//! Scenario files: a list of timed events (MIDI bytes, parameter changes, kit loads,
//! fader moves) and a block count. Rendering a scenario produces, for every block,
//! DAC1 (128 int16, L/R interleaved) followed by DAC2 (128 int16), little endian.
//!
//! ```json
//! { "name": "bd_decay", "ticks": 300,
//!   "events": [ { "tick": 0, "events": [ {"set_parameter": {"param": 50, "value": 20}},
//!                                       {"midi": [144, 48, 100]} ] } ] }
//! ```
//! Events of an entry with `tick` = n are applied in order before the (n+1)-th call of
//! `tick()` (n = 0: before the first block). Entries need not be sorted; entries with
//! the same tick keep their file order.

use crate::engine::{Block, EmuEngine};
use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Scenario {
    pub name: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// Number of blocks to render.
    pub ticks: u64,
    pub events: Vec<TimedEvents>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimedEvents {
    pub tick: u64,
    pub events: Vec<Event>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Event {
    /// Raw DIN MIDI bytes, fed one by one to the parser.
    Midi(Vec<u8>),
    /// Parameter change through the menu path (`EmuEngine::set_parameter`).
    SetParameter { param: u16, value: u8 },
    /// Kit file contents (8 byte name + parameters) as hex.
    LoadKit { hex: String },
    /// Fader `index` (0..6) to ADC value `adc` (0..=0xFFF0, 12 bit left aligned).
    Fader { index: u8, adc: u16 },
}

impl Scenario {
    pub fn from_json(s: &str) -> Result<Self> {
        let sc: Scenario = serde_json::from_str(s).context("parse scenario")?;
        for e in &sc.events {
            if e.tick >= sc.ticks {
                bail!("event at tick {} >= ticks {}", e.tick, sc.ticks);
            }
        }
        Ok(sc)
    }

    pub fn load(path: impl AsRef<std::path::Path>) -> Result<Self> {
        let s = std::fs::read_to_string(path.as_ref())
            .with_context(|| format!("read {:?}", path.as_ref()))?;
        Self::from_json(&s).with_context(|| format!("{:?}", path.as_ref()))
    }

    /// Events sorted by tick (stable).
    pub fn schedule(&self) -> Vec<(u64, &Event)> {
        let mut v: Vec<(u64, &Event)> = self
            .events
            .iter()
            .flat_map(|t| t.events.iter().map(move |e| (t.tick, e)))
            .collect();
        v.sort_by_key(|(t, _)| *t);
        v
    }
}

pub fn decode_hex(s: &str) -> Result<Vec<u8>> {
    if !s.len().is_multiple_of(2) {
        bail!("odd hex length");
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).context("hex"))
        .collect()
}

pub fn apply_event(eng: &mut EmuEngine, e: &Event) -> Result<()> {
    match e {
        Event::Midi(bytes) => eng.midi(bytes),
        Event::SetParameter { param, value } => eng.set_parameter(*param, *value),
        Event::LoadKit { hex } => eng.load_kit(&decode_hex(hex)?),
        Event::Fader { index, adc } => eng.set_fader_adc(*index as usize, *adc),
    }
}

/// Run a scenario; `sink` receives every block.
pub fn run(eng: &mut EmuEngine, sc: &Scenario, mut sink: impl FnMut(&Block)) -> Result<()> {
    let sched = sc.schedule();
    let mut i = 0;
    for t in 0..sc.ticks {
        while i < sched.len() && sched[i].0 == t {
            apply_event(eng, sched[i].1)
                .with_context(|| format!("event at tick {t}: {:?}", sched[i].1))?;
            i += 1;
        }
        let b = eng.tick()?;
        sink(&b);
    }
    Ok(())
}

pub fn block_bytes(b: &Block, out: &mut Vec<u8>) {
    for s in b.dac1.iter().chain(b.dac2.iter()) {
        out.extend_from_slice(&s.to_le_bytes());
    }
}

/// Render a scenario to the raw output format and return (bytes, sha256 hex).
pub fn render(eng: &mut EmuEngine, sc: &Scenario) -> Result<(Vec<u8>, String)> {
    let mut out = Vec::with_capacity(sc.ticks as usize * 512);
    run(eng, sc, |b| block_bytes(b, &mut out))?;
    let h = crate::engine::hex(&Sha256::digest(&out));
    Ok((out, h))
}
