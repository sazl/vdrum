# Generic control-surface library: review and plan

Status: proposal, revision 2 (2026-11-09). Nothing here is implemented.

This answers Sami's request of 2026-11-06: everything in Rust, and a library that is independent of
VDrum and of the Launch Control XL 3 (LCXL3). The LCXL3 becomes the first of several devices that
use the library to drive plugins.

Crate names below use the placeholder prefix `surface-`; the final name is open (question 6).

Decisions so far (Sami, 2026-11-09):

| # | Question | Answer |
|---|---|---|
| 1 | Only plugins that embed the library, or any plugin? | Ideally any VST/plugin. |
| 2 | Layouts and modes as compiled Rust or as data files? | Rust code for now. |
| 3 | PR #26 (Python spec, simulator)? | Closed without merging. |

## 1. Starting point

These measurements are on `main` as of 2026-11-09.

| Part | Where | Size | Coupling |
|---|---|---|---|
| LCXL3 codec | `crates/lcxl3` | 661 lines | LCXL3 only, no VDrum dependency. |
| Surface runtime | `crates/vdrum-surface` | 3,703 lines (`surface/mod.rs` 1,594) | Depends on `lcxl3` and `lxr-params`. The layouts are hand-written Rust tables. The navigation state machine is implicit in `Surface` fields and in the order of match arms. The `Model` trait and the `Action` enum describe VDrum (voice faders, kits, sequencer, LFOs, mutes). |
| Plugin integration | `plugin/src/controller.rs` | about 900 lines | Opens the ports with `midir` and runs `Surface` on a controller thread. It applies `Action`s to shared atomics and the engine command queue. |
| Design docs | `docs/lcxl3-daw-mode/` | — | Hand-kept control tables, mode PNGs, and `source/layout.py`, which duplicates the Rust layouts. |

PR #26 was closed, but its commits stay reachable on GitHub (`refs/pull/26/head`, last commit
`545adba`). Parts worth porting from it:

- The LCXL3 emulator (`crates/lcxl3/src/emulator.rs`, 492 lines).
- The measured device geometry behind the confirmed SVG (`tools/surface_spec/surface_spec/hardware.py`).
- The browser simulator page and its Playwright tests.
- The nine scenarios.
- The differential parity method: the old and new code are fed the same 3,000 random sequences of
  120 inputs and their outputs compared.

The coupling has two sources.

1. **VDrum in the core.** `Model` and `Action` describe VDrum, so nothing below the plugin can be
   reused.
2. **No device abstraction.** `vdrum-surface` calls `lcxl3` types directly: `Button`, `Control`,
   the LED and screen messages.

PR #26's Python layer added a third: two languages for one model.

## 2. Review of the proposed components

### 2.1 Surface definition (physical: controls, layout; used for SVGs and overlays)

Keep it, with three changes.

- **Control identity is shared.** The definition and the interface must name the same controls.
  As separate crates they would need a third crate for the ids anyway. Proposal: one device-model
  crate with two modules (section 4).
- **Enough geometry for an exact drawing, and no rendering code.** The definition needs panel size,
  positions, cap shapes and LED placement, as the confirmed LCXL3 SVG did. It must not contain the
  SVG writer, so that defining a device does not pull one in.
- **Capabilities belong here, not in the protocol.** Examples: LED (RGB, single colour, none),
  encoder detents, motorised fader, display kind. The control layer reads these to choose feedback.

### 2.2 Surface interface (protocol: everything readable, writable, configurable)

Keep it, with three changes.

- **Separate the transport.** Port discovery, matching by name or SysEx identity reply, the mode
  handshake (LCXL3 DAW mode) and reconnection need `midir`. `midir` does not build for
  `wasm32-unknown-unknown`, so the transport must sit in its own crate or the simulator cannot be
  built.
- **Keep the protocol pure.** It should be `decode(&[u8]) -> Option<Event>`,
  `encode(&Output) -> Vec<Vec<u8>>`, and `init()`/`shutdown()` message lists, as `crates/lcxl3`
  already does. That makes protocol tests and emulators cheap.
- **Bound "anything configurable".** Device-specific configuration (LCXL3 custom modes, templates,
  global settings) should be optional typed extensions in the device crate. A generic API for every
  setting of every device has no stable shape.

### 2.3 Surface control (predefined behaviours: toggle + LED, knob + LED ring)

Keep it. This layer saves the most work per new device. The plan should name these behaviours,
since each has a known pitfall:

- **Toggle, momentary, long-press latch, radio group.** These need the runtime's clock for timing.
- **Absolute fader or knob to a parameter, with a takeover choice: jump, pickup or scale.** The
  LCXL3 faders are not motorised. VDrum has pickup today (`Settings::fader_pickup`).
- **Endless encoder.** Covers the relative encodings (two's complement, offset 64,
  sign-magnitude), acceleration, and stepping for discrete parameters.
- **Value feedback.** It must follow host and automation changes, not only control moves.
- **Host gestures.** A move must open and close an automation gesture (`begin_set_parameter` /
  `end_set_parameter` in nih-plug, which `plugin/src/editor.rs` already uses). The library must
  emit begin and end explicitly, or DAW automation recording breaks.

### 2.4 Surface display (drawing on displays)

Keep it, with two corrections.

- **Two display kinds.** Text-field displays render text themselves, and the host sends strings to
  slots. Bitmap displays need the library to render. One widget API (title, name, value) should
  cover both. The LCXL3 screen layout is still undocumented.
- **Do not write a raster library.** `embedded-graphics` (0.8.2 on crates.io, checked 2026-11-06)
  provides canvases, primitives and bitmap fonts, and builds for WASM.

### 2.5 Surface state (a state-machine library, event driven)

Change it. A general state-machine library is the component most likely to grow without paying off.

- The state a surface needs is narrow:
  - the mode stack (mode, page, momentary overlays such as "while Shift is held");
  - held modifiers and latched toggles;
  - takeover state;
  - timers;
  - a small typed store.

  Everything in today's `vdrum-surface` fits this list.
- Hierarchical state machines already exist in Rust. One example is `statig`, 0.4.1 on crates.io.
- If transitions are arbitrary closures, the diagram generator and the simulator cannot see them.

Proposal: no standalone state crate. The runtime owns:

- the event loop;
- the mode stack;
- the store;
- the timers.

Transitions are declared through a typed builder:

```rust
on(SHIFT + TRACK[0]).press(push_mode(VOICE_1))
```

The builder produces plain Rust values. The diagram is generated from the same values that run.
Leaf behaviours are Rust code.

### 2.6 Surface mode (modes, snapshots, linking, conditional behaviour)

Keep it as the authoring API, in the same crate as the runtime that executes it. Modes are the
states of 2.5, and splitting the definition from the execution would put every type across a crate
boundary.

- Snapshots are a store serialisation.
- Linking is one input driving several bindings.
- Conditional behaviour is guards on bindings.

### 2.7 What the plan is missing

1. **The target abstraction, which is the crux of decoupling.** It has four parts:
   - a parameter list (id, name, range, steps, value-to-text);
   - value read-back for feedback;
   - writes with gesture begin and end;
   - application commands for everything that is not a parameter.

   VDrum's `Model` and `Action` play this role today. Controlling any plugin (decision 1) depends on
   this trait being host-neutral, since the same layout must drive VDrum's own parameters and a
   hosted plugin's parameters.
2. **How "any plugin" is reached.** See section 3. This is the largest addition to the plan.
3. **One owner per device.** Several plugin instances, or several plugins, may want the same
   controller. Only one process should own the MIDI port. Historically WinMM on Windows opens a
   MIDI port for one client only, and Windows MIDI Services adds multi-client support; neither was
   tested here. Some DAWs run each plugin in its own process; Bitwig is one example.
4. **An extension point for custom behaviours.** VDrum's sequencer, step locks, momentary effects
   and morph are application code. The `Behaviour` trait lets applications add them.
5. **Testing and simulation.** This means a virtual device per device crate, a scenario runner,
   golden checks and the WASM simulator.
6. **A second device early**, even a synthetic "8 knobs, 8 buttons, plain CC" one. Otherwise the
   library takes the LCXL3's shape.
7. **Threading.** Surface work never runs on the audio thread. The runtime is single-threaded and
   deterministic, and is driven by `handle(event, now)` / `tick(now)`.

## 3. Controlling any plugin

A library compiled into one plugin can only reach that plugin's parameters. To control other
plugins there are four routes. They are not exclusive: the `Target` trait is the seam, and each
route is one `Target` implementation.

| Route | How it works | Works in | Cost | Limits |
|---|---|---|---|---|
| A. Embedded | The plugin links the library (VDrum). | Any DAW | Lowest | Only that plugin. |
| B. Wrapper plugin | A VST3/CLAP plugin that loads another plugin inside itself, passes audio and MIDI through, and implements `Target` over the hosted plugin's parameters. | Any DAW that loads the wrapper | High | See below. |
| C. Standalone host | A desktop application that hosts plugins and the surface, outside a DAW. | No DAW | Medium | Not inside a DAW session. |
| D. DAW bridge | A small DAW-specific script or extension (Bitwig controller API, REAPER control-surface API, Ableton remote scripts) reports the focused device's parameters to the library and applies changes. | One DAW per bridge | Medium per DAW | Each DAW exposes a different subset. Some DAWs expose only parameter pages, such as CLAP's remote-controls extension. |

Facts that bear on route B:

- **Licensing.** VST3 is licensed under MIT since SDK 3.8 (released October 2025, per Steinberg's
  licensing page). CLAP is MIT. Neither licence blocks a wrapper.
- **CLAP hosting in Rust.** `clack-host` (prokopyl/clack) is a low-level, safe CLAP host library
  built on `clap-sys`. It is the most direct base.
- **VST3 hosting in Rust.** It is younger. `vst3-host` 0.5.0 reports working hosting tested
  against real plugins on macOS. `rack` 0.4.8 covers VST3, AU and CLAP, but lists VST3 GUI
  hosting as in progress. Both need an evaluation before being relied on. VDrum already depends on
  `vst3-sys` through nih-plug.
- **nih-plug fixes the parameter list.** nih-plug builds its parameter table once, when the plugin
  is created (`src/wrapper/vst3/inner.rs` and `src/wrapper/clap/wrapper.rs` at the pinned revision
  `de42101`). A wrapper built on nih-plug can therefore expose the hosted plugin's parameters to
  the DAW only as a fixed pool of generic slots ("Param 1..N"). The other option is to build the
  wrapper on `clack-plugin` directly, where CLAP's params extension can ask the host to rescan.
  The surface itself does not need the DAW to see the parameters; only DAW automation of the
  hosted plugin through the wrapper does.
- **Hosted plugin windows.** Embedding a hosted plugin's window inside the wrapper's window is the
  most platform-specific part. Opening it as a separate window is simpler.
- **State.** The wrapper must save and restore the hosted plugin's state inside its own.

Recommendation: build route A first (VDrum), with the `Target` trait designed for route B from the
start. Then build route B as CLAP-first: host CLAP plugins through `clack-host`, then add VST3
hosting after an evaluation of `vst3-host` and `rack`. Route D is optional later, for DAW-wide
control such as mixer and transport, which no plugin can reach.

Device ownership (section 2.7, item 3) becomes concrete with route B, because a session may hold
many wrapper instances. Proposal: a small `surface-daemon` process owns the devices and runs the
runtime. Each plugin instance (embedded or wrapper) connects over local IPC and registers as a
target. The surface switches between targets with a focus control. Route A can run without the
daemon at first, as VDrum does today, and move to it in phase 7.

## 4. Proposed design

### 4.1 Principles

- **One language.** Rust everywhere. The simulator page keeps a small amount of JavaScript for
  the DOM.
- **Layouts are Rust code** (decision 2), written with typed builders that produce plain values.
  The diagram, SVG overlays and docs are generated from those values. The string rule language of
  PR #26 is gone; guards are a typed enum. Hot reload of layouts is dropped for now.
- **Layers point downwards only.** The library never depends on a device crate, a host or an
  application. A device crate never depends on an application.
- **Pure core.** Everything above `surface-io` does no I/O, starts no threads and has no clock of
  its own. It builds for WASM and is tested with a fake clock.
- **Numeric ids with names.** Controls, parameters and commands are numeric ids with a name table.
  Device crates export typed constants. Generic tools then need no type parameters.

### 4.2 Components

| Crate | Role | Proposed component |
|---|---|---|
| `surface-core` | Ids, `Event` (press, release, absolute value with resolution, relative delta, touch), `Output` (LED, display content, value echo), colours, time, the `Target` trait. | (new) |
| `surface-device` | `physical`: geometry, controls, capabilities, displays. `protocol`: the `Protocol` trait and identity matching. | definition + interface |
| `surface-display` | Display model (text-field or bitmap), widgets, rasterising through `embedded-graphics`. | display |
| `surface-control` | The `Behaviour` trait and the stock behaviours of 2.3. | control |
| `surface-runtime` | Event loop, mode stack, overlays, guards, transitions, store, timers, snapshots, linking; the builder API for profiles. | state + mode |
| `surface-io` | `midir` transport, port matching, handshake, reconnection. Native only. | (new) |
| `surface-render` | SVG of a device, per-mode overlays, Mermaid diagram of the modes, Markdown control tables. | part of definition |
| `surface-test` | Virtual-device harness, scenarios in Rust, golden-file checks. | (new) |
| `surface-sim` | WASM build of the runtime with a device emulator and a fake target, plus the web page. | (new) |
| `surface-cli` | `svg`, `docs`, `diagram`, `serve`, `bundle`. | (new) |
| `surface-host-clap` | `Target` over a CLAP plugin loaded with `clack-host` (route B). | (new) |
| `surface-host-vst3` | `Target` over a VST3 plugin (route B, after evaluation). | (new) |
| `surface-link` | Local IPC between plugin instances and the daemon: target registration, parameter lists, value changes. | (new) |

The applications are:

- `apps/surface-daemon`, which owns the devices and runs the runtime;
- `apps/surface-wrapper`, the route-B wrapper plugin (VST3/CLAP).

Device crates:

- `devices/novation-lcxl3`: definition, protocol, emulator, typed constants.
- `devices/generic-cc`: the synthetic second device.

VDrum's side is `crates/vdrum-control`:

- an implementation of `Target` over the plugin's shared parameter state;
- VDrum's commands;
- VDrum's custom behaviours (sequencer steps, step locks, momentary effects, morph, macros, kit
  browser);
- VDrum's LCXL3 profile (the 13 modes, palette, screen templates).

`plugin/` keeps only the thread and queue glue.

### 4.3 Architecture diagram

```
 ┌──────────────────────────── applications and hosts ───────────────────────────────┐
 │                                                                                   │
 │  VDrum plugin (nih-plug)           surface-wrapper plugin        DAW bridge       │
 │  └ vdrum-control                   └ surface-host-clap /         (optional,       │
 │    impl Target, custom               surface-host-vst3            route D)        │
 │    Behaviours, LCXL3 profile         impl Target over the                         │
 │    (route A)                         hosted plugin (route B)                      │
 │        │                                   │                          │           │
 │        └──────────── surface-link (local IPC: register target, ───────┘           │
 │                      parameter lists, values, focus)                              │
 │                                     │                                             │
 │                       apps/surface-daemon (owns devices)                          │
 └─────────────────────────────────────┼─────────────────────────────────────────────┘
                                       │ uses
 ┌──────────────────────────── library (device- and host-agnostic) ──────────────────┐
 │                                     ▼                                             │
 │   surface-io ─────────────► surface-runtime ◄──────── surface-control             │
 │   (midir, ports,            (event loop, modes,       (Behaviour trait, toggle,   │
 │    handshake; native)        guards, store, timers,    knob+LED, takeover,        │
 │        │                     snapshots, builders)      encoder, gestures)         │
 │        │                       │        │                     │                   │
 │        │                       │        └─────► surface-display (widgets,         │
 │        │                       │                text/bitmap, embedded-graphics)   │
 │        ▼                       ▼                              │                   │
 │   surface-device (physical + protocol traits) ◄───────────────┘                   │
 │        │                                                                          │
 │        ▼                                                                          │
 │   surface-core (ids, Event, Output, colour, time, Target trait)                   │
 │                                                                                   │
 │   tooling: surface-render (SVG, overlays, Mermaid, tables)   surface-cli          │
 │            surface-test (virtual device, scenarios, goldens) surface-sim (WASM)   │
 └───────────────────────────────────────────────────────────────────────────────────┘
                                       ▲ implemented by
 ┌──────────────────────────── devices ┴─────────────────────────────────────────────┐
 │   devices/novation-lcxl3 (definition, protocol, emulator)                         │
 │   devices/generic-cc     (synthetic second device)                                │
 └───────────────────────────────────────────────────────────────────────────────────┘

 Run-time flow:
   MIDI in ─► Protocol::decode ─► Event ─► Runtime: mode stack ─► binding ─► Behaviour
         ─► TargetOp (gesture begin/set/end, command) ─► focused Target (VDrum or hosted plugin)
         ─► Output (LED, display) ─► Protocol::encode ─► MIDI out
   Target value change (automation, editor) ─► Runtime ─► Behaviour feedback ─► Output
```

### 4.4 Key interfaces (sketch, not final)

```rust
// surface-core
pub struct ControlId(pub u16);
pub struct ParamId(pub u32);
pub struct CommandId(pub u16);
pub enum Event { Press(ControlId), Release(ControlId), Abs { id: ControlId, value: u16, max: u16 },
                 Rel { id: ControlId, delta: i32 }, Touch(ControlId, bool) }
pub enum Output { Led { id: ControlId, light: Light }, Display { id: DisplayId, content: DisplayContent },
                  Echo { id: ControlId, value: u16 } }
pub trait Target {
    fn params(&self) -> &[ParamInfo];           // id, name, range, steps, group
    fn value(&self, p: ParamId) -> f32;          // normalised 0..=1
    fn text(&self, p: ParamId, v: f32) -> String;
    fn commands(&self) -> &[CommandInfo] { &[] } // application commands, if any
}
pub enum TargetOp { Begin(ParamId), Set(ParamId, f32), End(ParamId), Command(CommandId, i32) }

// surface-device
pub trait Protocol {
    fn decode(&mut self, bytes: &[u8]) -> Option<Event>;
    fn encode(&mut self, out: &Output) -> Vec<Vec<u8>>;
    fn init(&self) -> Vec<Vec<u8>>;
    fn shutdown(&self) -> Vec<Vec<u8>>;
}

// surface-control
pub trait Behaviour {
    fn on_event(&mut self, ev: &Event, cx: &mut Cx) -> Handled; // cx: store, target, time, emit ops
    fn render(&self, cx: &RenderCx) -> Vec<Output>;              // feedback from current state
}

// surface-runtime
impl Runtime {
    pub fn handle(&mut self, ev: Event, now: Millis, target: &dyn Target) -> Step; // ops + outputs
    pub fn tick(&mut self, now: Millis, target: &dyn Target) -> Step;
}
```

A layout for an arbitrary hosted plugin cannot name that plugin's parameters at build time. Profiles
therefore bind to parameters in three ways:

- by id (VDrum);
- by position ("parameter 1..8 of page n");
- through a per-plugin map, written in Rust for now.

### 4.5 Proposed project structure

The library starts inside this repository and moves to its own repository once the wrapper or a
second application uses it (question 5).

```
vdrum/
├── Cargo.toml                       workspace: lib/*, hosts/*, apps/*, devices/*, crates/*, plugin, xtask
├── lib/                             the generic library (no VDrum, no LCXL3, no plugin format)
│   ├── surface-core/
│   │   └── src/{lib.rs, id.rs, event.rs, output.rs, colour.rs, time.rs, target.rs}
│   ├── surface-device/
│   │   └── src/{lib.rs, physical.rs, capability.rs, protocol.rs, identity.rs}
│   ├── surface-display/
│   │   └── src/{lib.rs, model.rs, widget.rs, raster.rs}
│   ├── surface-control/
│   │   └── src/{lib.rs, behaviour.rs, button.rs, knob.rs, fader.rs, encoder.rs,
│   │            feedback.rs, takeover.rs, gesture.rs}
│   ├── surface-runtime/
│   │   └── src/{lib.rs, builder.rs, profile.rs, mode.rs, guard.rs, binding.rs, store.rs,
│   │            timer.rs, snapshot.rs, dispatch.rs, focus.rs}
│   ├── surface-io/                  native only (midir)
│   │   └── src/{lib.rs, ports.rs, session.rs}
│   ├── surface-link/                local IPC between instances and the daemon
│   │   └── src/{lib.rs, message.rs, client.rs, server.rs}
│   ├── surface-render/
│   │   └── src/{lib.rs, svg.rs, overlay.rs, diagram.rs, tables.rs}
│   ├── surface-test/
│   │   └── src/{lib.rs, virtual_device.rs, fake_target.rs, scenario.rs, golden.rs}
│   ├── surface-sim/
│   │   ├── src/{lib.rs, wasm.rs}
│   │   └── web/{index.html, sim.js, sim.css, test/}
│   └── surface-cli/
│       └── src/main.rs              svg | docs | diagram | serve | bundle
├── hosts/                           Target implementations over hosted plugins (route B)
│   ├── surface-host-clap/           clack-host
│   │   └── src/{lib.rs, load.rs, params.rs, process.rs, state.rs, gui.rs}
│   └── surface-host-vst3/           after evaluating vst3-host / rack
│       └── src/{lib.rs, load.rs, params.rs, process.rs, state.rs, gui.rs}
├── apps/
│   ├── surface-daemon/              owns the devices, runs the runtime, serves surface-link
│   │   └── src/main.rs
│   └── surface-wrapper/             VST3/CLAP plugin that hosts one plugin (route B)
│       └── src/{lib.rs, params.rs, editor.rs}
├── devices/
│   ├── novation-lcxl3/
│   │   ├── src/{lib.rs, physical.rs, protocol.rs, controls.rs, emulator.rs}
│   │   └── tests/{protocol.rs, emulator.rs}
│   └── generic-cc/
│       └── src/lib.rs
├── crates/
│   ├── lxr-core/ lxr-emu/ lxr-difftest/ lxr-params/   unchanged
│   └── vdrum-control/               VDrum's adapter (route A)
│       ├── src/{lib.rs, target.rs, commands.rs}
│       ├── src/behaviours/{sequencer.rs, step_lock.rs, momentary_fx.rs, morph.rs,
│       │                   macros.rs, kit_browser.rs, lfo_view.rs}
│       ├── src/profiles/lcxl3/{mod.rs, palette.rs, navigation.rs, screen.rs, voices.rs,
│       │                       sequencer.rs, mix_fx.rs, perform.rs, lfo_mod.rs,
│       │                       morph_macro.rs, global.rs}
│       └── tests/{parity.rs, scenarios.rs}
├── plugin/                          nih-plug; controller.rs shrinks to thread and queue glue
└── docs/surface-library/            this plan, the authoring guide, generated docs
```

Removed at the end of migration:

- `crates/lcxl3`, which moves to `devices/novation-lcxl3`;
- `crates/vdrum-surface`, replaced by `lib/*` and `vdrum-control`;
- `docs/lcxl3-daw-mode/source/`, plus the hand-kept tables and PNGs, which are replaced by
  generated docs.

## 5. Migration plan

Each phase is one pull request and leaves the plugin working.

1. **Core and devices.**
   - Create `surface-core` and `surface-device`.
   - Move `crates/lcxl3` to `devices/novation-lcxl3` behind `Protocol`.
   - Port the emulator and the measured geometry from PR #26.
   - Add `devices/generic-cc`.
   - Protocol and emulator tests for both devices.
2. **Rendering and CLI.**
   - `surface-render` and `surface-cli svg`.
   - The LCXL3 SVG must match the one Sami confirmed in PR #26.
3. **Controls and display.**
   - `surface-control` stock behaviours and `surface-display`.
   - Unit tests with a fake target and a fake clock, on both devices.
4. **Runtime.**
   - `surface-runtime` with the builder API, and `surface-test`.
   - A reference profile for `generic-cc`: one mode, knobs to parameters, toggles with LEDs.
5. **VDrum adapter (route A).**
   - `vdrum-control`: `Target`, the commands, the custom behaviours and the 13-mode LCXL3 profile.
   - Parity test against the current `vdrum-surface`, comparing device MIDI out and target
     operations on random input sequences.
6. **Plugin switch-over.**
   - `surface-io`; `controller.rs` uses the new runtime.
   - Delete `vdrum-surface`.
   - Generated docs replace the hand-kept tables and PNGs.
7. **Simulator.** `surface-sim` and `surface-cli serve`/`bundle`, with Playwright tests ported from
   PR #26.
8. **Daemon and link.** `surface-link`, `surface-daemon`, and focus switching between targets.
   VDrum connects through the link.
9. **Wrapper, CLAP (route B).**
   - `surface-host-clap` and `surface-wrapper`: audio and MIDI pass-through, state, a separate
     window for the hosted plugin's GUI, and a fixed pool of automatable slots.
   - Tested against at least one free CLAP plugin.
10. **Wrapper, VST3.**
    - Evaluate `vst3-host` and `rack`, then build `surface-host-vst3`.
    - Tested on macOS and Windows with at least one free VST3 plugin.

## 6. Risks

- **Size.** Phases 1 to 7 port about 4,400 lines of existing Rust plus PR #26's emulator and
  simulator. Phases 9 and 10 (plugin hosting) are new work, and the largest unknown.
- **Plugin hosting quirks.** Plugins differ in threading, parameter flags, state and GUI handling. A
  wrapper needs testing against many plugins. CLAP is the smaller first step.
- **Multi-process.** The daemon adds IPC, startup ordering and crash recovery. Route A works
  without it until phase 8.
- **Over-generalising.** The second device and the parity test check against it. Features with no
  second user stay in `vdrum-control`.

## 7. Open questions

1. **Exact parity.** Does "reproduce today's behaviour exactly" still apply to the rewrite?
2. **Second device.** Is a synthetic generic CC device enough, or is there a second physical
   device you own that should be the second target?
3. **Route for "any plugin".** Is the wrapper plugin (route B) the intended route, and is
   CLAP-first acceptable? Are routes C (standalone host) and D (DAW bridges) wanted at all?
4. **Daemon.** Is a separate daemon process acceptable? It is needed for one device to reach
   several plugin instances.
5. **Repository.** Inside `vdrum` for now (proposed), or a separate repository from the start?
6. **Name and licence.** What should the library be called? VDrum is GPL-3.0-only. A library for
   many devices and projects may need MIT or Apache-2.0. That is your call as the author.

## Sources

- Steinberg, VST 3 licensing: https://steinbergmedia.github.io/vst3_dev_portal/pages/VST+3+Licensing/VST3+License
- clack (CLAP in Rust, plugin and host): https://github.com/prokopyl/clack, https://docs.rs/clack-host
- vst3-host: https://docs.rs/crate/vst3-host/0.5.0
- rack: https://docs.rs/crate/rack/latest
- nih-plug at the pinned revision `de42101`: `src/params.rs`, `src/wrapper/vst3/inner.rs`,
  `src/wrapper/clap/wrapper.rs`
