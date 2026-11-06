# Generic control-surface library: review and plan

Status: proposal, 2026-11-06. Nothing here is implemented. It answers Sami's request of 2026-11-06:
everything in Rust, with a library that is independent of VDrum and of the Launch Control XL 3
(LCXL3). The LCXL3 becomes the first of several devices that use the library to drive a plugin.

Crate names below use the placeholder prefix `surface-`; the final name is open (question 8).

## 1. What exists today

These are the measured starting points, on branch `claude/project-thread-7553h1` (PR #26).

| Part | Where | Size | Coupling |
|---|---|---|---|
| LCXL3 codec and emulator | `crates/lcxl3` | 1,154 lines | Generic in spirit, LCXL3 only. No VDrum dependency. |
| Surface runtime | `crates/vdrum-surface` | 4,500 lines (`surface/mod.rs` 1,625) | Depends on `lcxl3` and `lxr-params`. The `Model` trait and the `Action` enum are VDrum's (voice faders, kits, sequencer, LFOs, mutes). |
| Spec loader and rule interpreter | `vdrum-surface/src/{spec,rules}.rs` | 1,107 lines | A string-guard rule language (`"shift"`, `"mode:seq"`, `"held:play"`) interpreted at run time. |
| Python spec, DSL, SVG, docs | `surface/`, `tools/surface_spec/` | about 2,000 lines | Produces `surface.json`. Hand-mirrors the Rust catalogues (`BUTTONS`, `PREDICATES`, `NAV_STEPS`) and needs `param_ids.json` exported back from Rust. |
| Simulator | `crates/vdrum-sim`, `tools/surface_sim` | 742 lines Rust, plus JS/CSS | WASM build of the runtime with a fake VDrum model. |
| Plugin integration | `plugin/src/controller.rs` | 924 lines | Opens the ports with `midir`, runs `Surface` on a controller thread, and applies `Action`s to shared atomics and the engine command queue. |

The complexity has three sources.

1. **Two languages for one model.** Every rule catalogue exists twice, in Python and in Rust, and is
   kept in sync by hand. `param_ids.json` makes the dependency circular.
2. **A home-made rule language.** Guards and steps are strings checked twice: by the Python checker
   and by the Rust parser.
3. **VDrum in the core.** `Model` and `Action` describe VDrum, so nothing below the plugin can be
   reused.

## 2. Review of the proposed components

### 2.1 Surface definition (physical: controls, layout; used for SVGs and overlays)

Keep it. Some points need fixing.

- **Control identity is shared.** The definition and the interface must name the same controls. If
  they are separate crates, a third crate is needed for the ids anyway. Proposal: put both in one
  device-model crate, as two modules (section 3).
- **Say what the drawing contains.** It must have enough geometry to draw the device exactly (the
  LCXL3 SVG work needed panel size, positions, cap shapes and LED placement). It must also stay
  free of rendering code, so that a device can be defined without pulling in an SVG writer.
- **Capabilities belong here, not in the protocol.** Examples: whether a button has an LED, whether
  that LED is RGB, single-colour or none, encoder detents, whether a fader is motorised, whether a
  display is text-field or bitmap. The behaviours in the control layer read these to pick their
  feedback. Today `lcxl3::emulator` and `device_svg.py` each hold part of this knowledge.

### 2.2 Surface interface (protocol: everything readable, writable, configurable)

Keep it. It has three gaps.

- **The transport is missing.** Protocol (bytes ↔ events) is one thing. Port discovery, matching
  a device by name or by SysEx identity reply, the mode handshake (LCXL3 DAW mode) and reconnection
  are another. They need `midir`, which does not exist for `wasm32-unknown-unknown`, so they must
  sit in a separate, feature-gated crate. Otherwise the simulator cannot build.
- **Pure functions.** The protocol should be pure: `decode(&[u8]) -> Option<Event>` and
  `encode(&Output) -> Vec<Vec<u8>>`, plus `init()`/`shutdown()` message lists. This is how
  `crates/lcxl3` already works, and it is what makes protocol tests and emulators cheap.
- **Bound the scope of "anything configurable".** Device-specific configuration, such as LCXL3
  custom modes, factory templates and global settings, should be optional typed extensions on the
  device crate. They should not be part of the generic interface. A generic API for every possible
  setting of every device has no stable shape.

### 2.3 Surface control (predefined behaviours: toggle + LED, knob + LED ring)

Keep it. This is the layer that saves the most work per new device. Behaviours the plan should
name, because each has a known pitfall:

- **Toggle, momentary, latch-on-long-press, radio group.** Press and release semantics and timing
  (long-press, double-tap) need a clock, so they depend on the runtime's time source.
- **Absolute fader or knob → parameter, with takeover.** Choices are jump, pickup or scaling. The
  LCXL3 faders are not motorised, so a parameter changed elsewhere leaves the fader at the wrong
  position. VDrum handles this today; the generic version must make it a choice.
- **Endless encoder → parameter.** Options: relative encoding (two's complement, offset 64,
  sign-magnitude), acceleration, and steps for discrete parameters.
- **Value feedback.** LED brightness or colour follows the value, and must update when the host or
  automation changes the parameter, not only when the control moves.
- **Host gestures.** A move must open and close an automation gesture (`begin_set_parameter` /
  `end_set_parameter` in nih-plug; the editor already does this in `plugin/src/editor.rs`). The
  library has to emit begin/end explicitly. Otherwise DAW automation recording breaks.

### 2.4 Surface display (drawing on displays)

Keep it, with two corrections.

- **Two kinds of display.** Some devices render text fields themselves, and the host sends strings
  to slots. Others need a bitmap, which the library renders and sends. The LCXL3's screen layout is
  undocumented, which is still open from the earlier work. The display layer needs both kinds
  behind one widget API ("title, name, value") so that a layout does not care which kind the device
  has.
- **Do not write a raster library.** `embedded-graphics` (0.8.2 on crates.io, checked 2026-11-06)
  already provides monochrome and colour canvases, primitives and bitmap fonts, and it builds for
  WASM. The display crate should provide the widgets and the device-independent display model, and
  rasterise through `embedded-graphics`.

### 2.5 Surface state (a state-machine library, event driven)

Change this one. A general-purpose state-machine library is the component most likely to grow
without paying off.

- **The state a surface needs is narrow.** It is: the active mode stack (mode, sub-mode or page,
  momentary overlays such as "while Shift is held"), held modifiers, latched toggles, per-control
  takeover state, timers, and a small store of named values. Everything in today's
  `vdrum-surface` fits that list.
- **Hierarchical state machines already exist in Rust.** `statig` is one (0.4.1 on crates.io).
  Writing a new general one costs more than adopting one or keeping the model narrow.
- **Rust code is opaque to the tools.** If state transitions are arbitrary Rust closures, the
  diagram generator and the simulator's rule highlighting cannot see them. This was the reason for
  the declarative rules in PR #26.

Proposal: no standalone state crate. The runtime (section 3, `surface-runtime`) owns:

- one event loop, `event → dispatch → outputs`;
- the mode stack;
- a typed store;
- timers.

Mode transitions are declared as data (`on Shift+Track1 press → push mode Voice(0)`), so diagrams
come from the same structure that runs. Leaf behaviours, such as what a sequencer step button does,
are Rust code.

### 2.6 Surface mode (modes, snapshots, linking, conditional behaviour)

Keep it as the authoring layer, but merge it with the runtime that executes it. Modes are the
states of 2.5. Splitting "the thing that defines modes" from "the thing that runs them" across two
crates gives a crate boundary with every type shared across it. Snapshots are a store
serialisation. Linking means one input drives several bindings. Conditional behaviour means guards
on bindings.

### 2.7 What the plan is missing

1. **The target side, which is the crux of decoupling.** "Map any MIDI control to a VST control"
   needs a target abstraction:
   - a parameter list (id, name, range, step count, value-to-text);
   - a read of the current value, for feedback;
   - a write with gesture begin and end;
   - application commands for everything that is not a parameter (VDrum: kit browse, sequencer
     edits, panic, mutes).

   Today this role is played by VDrum's `Model` and `Action`. Without a generic target trait the
   library stays tied to one plugin.
2. **A scope decision: inside the plugin, or controlling any plugin.** A library compiled into a
   plugin can drive that plugin's own parameters. It cannot control a different plugin inside a
   DAW. That needs the DAW's own control-surface API, or a wrapper plugin that hosts other plugins.
   The plan must say which one is meant (question 1).
3. **Extension point for custom behaviours.** VDrum's sequencer mode, step locks, momentary effects
   and morph are application-specific. The library needs a `Behaviour` trait that applications
   implement, registered by name so that data-defined layouts can refer to them.
4. **Testing and simulation as first-class parts.** These cover a virtual device per device crate
   (today's `lcxl3::emulator`), a scenario runner and golden checks, and the WASM simulator. The
   plan lists none of them. They are what keeps a generic library honest across devices.
5. **A second device early.** A library designed against one device takes that device's shape.
   One second device, even a synthetic "8 knobs, 8 buttons, plain CC" definition, should go
   through every layer from the first phase.
6. **Threading.** Surface work must never run on the audio thread. The runtime should be
   single-threaded, deterministic and driven by `handle(event, now)` / `tick(now)` calls. The host
   adapter connects it to the plugin with lock-free queues, as `controller.rs` does today.
7. **Where layouts live.** "Everything in Rust" is ambiguous (question 2):
   - Rust code compiled into the plugin. A layout change needs a rebuild, and hot reload and loading
     a layout in the browser are lost.
   - Rust types serialised to data (RON or JSON). These are authored either in Rust builder code or
     in the data file and loaded at run time. One language, no code generation, and hot reload and
     simulator file loading stay.

## 3. Proposed design

### 3.1 Principles

- **One language.** Rust everywhere. The only non-Rust code is the simulator page's DOM glue, kept
  small.
- **Data for structure, code for behaviour.** Device definitions, layouts, modes, bindings and
  transitions are serde types. They can be written in Rust builders (type-checked, and the default)
  or loaded from a data file (hot reload, the browser). Behaviours are Rust trait implementations
  referenced by name. The rule language of PR #26 disappears: guards become a small typed enum, not
  strings.
- **Layers point downwards only.** The library never depends on a device crate or an application.
  A device crate never depends on an application.
- **Pure core.** Everything above the I/O crate is `no I/O, no threads, no clock of its own`, so it
  builds for WASM and is testable with a fake clock.
- **Numeric ids with names.** Controls, parameters and commands are `u16` ids with a name table in
  their definition. Device crates export typed constants. Generic tools (SVG, simulator, docs) then
  need no generic type parameters.

### 3.2 Components

| Crate | Role | Sami's component |
|---|---|---|
| `surface-core` | Ids, `Event` (press, release, absolute value with resolution, relative delta, touch), `Output` (LED colour or brightness or animation, display content, value echo), colours, time, the `Target` trait (parameters, values, gestures, commands). | (new) |
| `surface-device` | `physical`: geometry, controls, capabilities, displays. `protocol`: the `Protocol` trait (pure decode, encode, init, shutdown), identity matching. | definition + interface |
| `surface-display` | Display model (text-field vs bitmap), widgets (title/name/value, bars, lists), rasterising through `embedded-graphics`. | display |
| `surface-control` | The `Behaviour` trait and the stock behaviours of 2.3 (toggle, momentary, radio, param knob or fader with takeover, relative encoder, value feedback, gesture handling). | control |
| `surface-runtime` | Event loop, mode stack, overlays, guards, transitions, store, timers, snapshots, linking; loads a `Profile` (device + layouts + modes + bindings). | state + mode |
| `surface-io` | `midir` transport, port matching, handshake, reconnection. Not built for WASM. | (new) |
| `surface-render` | SVG of a device, overlay labels per mode, Mermaid diagram of the mode graph, Markdown control tables. Used by the CLI, the docs and the simulator. | part of definition |
| `surface-test` | Virtual-device harness, scenario DSL in Rust, golden-file checks. | (new) |
| `surface-sim` | WASM build of the runtime plus a device's emulator, and the static web page. | (new) |
| `surface-cli` | `svg`, `docs`, `diagram`, `check`, `serve` (simulator with hot reload), `bundle`. Replaces `tools/surface_spec/run.py`. | (new) |

Device crates live outside the library: `devices/novation-lcxl3` (definition, protocol, emulator,
typed constants) and `devices/generic-cc` (the synthetic second device).

The application side is VDrum's adapter crate, `crates/vdrum-control`:

- `impl Target` over the plugin's shared parameter state;
- VDrum's commands;
- VDrum's custom behaviours (sequencer steps, step locks, momentary effects, morph, macros, kit
  browser);
- VDrum's LCXL3 profile: the 13 modes, layouts, palette and screen templates.

`plugin/` keeps only the thread and queue glue.

### 3.3 Architecture diagram

```
                 ┌──────────────────────── application (VDrum) ────────────────────────┐
                 │  plugin/ (nih-plug)            crates/vdrum-control                 │
                 │  audio thread ◄─queues─► controller thread ── impl Target            │
                 │                                 │            custom Behaviours       │
                 │                                 │            LCXL3 profile (modes)   │
                 └─────────────────────────────────┼────────────────────────────────────┘
                                                   │ uses
 ┌──────────────────────────────── library (device- and app-agnostic) ─────────────────┐
 │                                                 ▼                                   │
 │   surface-io ──────────────► surface-runtime ◄──────── surface-control              │
 │   (midir, ports,            (event loop, modes,        (Behaviour trait, toggle,    │
 │    handshake; native only)   guards, store, timers,     knob+LED, takeover,         │
 │        │                     snapshots, Profile)        encoder, gestures)          │
 │        │                       │        │                      │                    │
 │        │                       │        └──────► surface-display (widgets,          │
 │        │                       │                 text/bitmap, embedded-graphics)    │
 │        ▼                       ▼                               │                    │
 │   surface-device (physical + protocol traits) ◄────────────────┘                    │
 │        │                                                                            │
 │        ▼                                                                            │
 │   surface-core (ids, Event, Output, colour, time, Target trait)                     │
 │                                                                                     │
 │   tooling:  surface-render (SVG, overlays, Mermaid, tables)                         │
 │             surface-test (virtual device, scenarios, goldens)                       │
 │             surface-sim (WASM + page)      surface-cli (svg/docs/check/serve)       │
 └─────────────────────────────────────────────────────────────────────────────────────┘
                                                   ▲ implemented by
                 ┌──────────────────────── devices ─┴───────────────────────────────────┐
                 │  devices/novation-lcxl3 (definition, protocol, emulator)             │
                 │  devices/generic-cc     (synthetic second device)                    │
                 └──────────────────────────────────────────────────────────────────────┘

 Data flow at run time:
   MIDI bytes ─► Protocol::decode ─► Event ─► Runtime (mode stack ─► binding ─► Behaviour)
              ─► Target ops (set param + gesture, command)  and  Outputs (LED, display)
   Outputs ─► Protocol::encode ─► MIDI bytes          Target changes ─► Behaviour feedback
```

### 3.4 Key interfaces (sketch, not final)

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
    fn params(&self) -> &[ParamInfo];          // id, name, range, steps
    fn value(&self, p: ParamId) -> f32;         // normalised 0..=1
    fn text(&self, p: ParamId, v: f32) -> String;
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
    fn on_event(&mut self, ev: &Event, cx: &mut Cx) -> Handled;   // cx: store, target, time, emit
    fn render(&self, cx: &RenderCx) -> Vec<Output>;               // feedback from current state
}

// surface-runtime
pub struct Runtime { /* profile, mode stack, store, timers, behaviours */ }
impl Runtime {
    pub fn handle(&mut self, ev: Event, now: Millis, target: &dyn Target) -> Step; // ops + outputs
    pub fn tick(&mut self, now: Millis, target: &dyn Target) -> Step;
}
```

### 3.5 Proposed project structure

The library is placed inside this repository first, as its own workspace folder, and moved to its
own repository once a second application or device needs it (question 7).

```
vdrum/
├── Cargo.toml                      workspace: lib/*, devices/*, crates/*, plugin, xtask
├── lib/                            the generic library (no VDrum, no LCXL3)
│   ├── surface-core/
│   │   └── src/{lib.rs, id.rs, event.rs, output.rs, colour.rs, time.rs, target.rs}
│   ├── surface-device/
│   │   └── src/{lib.rs, physical.rs, capability.rs, protocol.rs, identity.rs}
│   ├── surface-display/
│   │   └── src/{lib.rs, model.rs, widget.rs, raster.rs}
│   ├── surface-control/
│   │   └── src/{lib.rs, behaviour.rs, button.rs, knob.rs, fader.rs, encoder.rs,
│   │            feedback.rs, takeover.rs}
│   ├── surface-runtime/
│   │   └── src/{lib.rs, profile.rs, mode.rs, guard.rs, binding.rs, store.rs,
│   │            timer.rs, snapshot.rs, dispatch.rs}
│   ├── surface-io/                 native only (midir)
│   │   └── src/{lib.rs, ports.rs, session.rs}
│   ├── surface-render/
│   │   └── src/{lib.rs, svg.rs, overlay.rs, diagram.rs, tables.rs}
│   ├── surface-test/
│   │   └── src/{lib.rs, virtual_device.rs, scenario.rs, golden.rs}
│   ├── surface-sim/
│   │   ├── src/{lib.rs, wasm.rs}
│   │   └── web/{index.html, sim.js, sim.css}
│   └── surface-cli/
│       └── src/main.rs             svg | docs | diagram | check | serve | bundle
├── devices/
│   ├── novation-lcxl3/
│   │   ├── src/{lib.rs, physical.rs, protocol.rs, controls.rs, emulator.rs}
│   │   └── tests/{protocol.rs, emulator.rs}
│   └── generic-cc/
│       └── src/lib.rs
├── crates/
│   ├── lxr-core/ lxr-emu/ lxr-difftest/ lxr-params/      unchanged
│   └── vdrum-control/              VDrum's adapter
│       ├── src/{lib.rs, target.rs, commands.rs}
│       ├── src/behaviours/{sequencer.rs, step_lock.rs, momentary_fx.rs, morph.rs,
│       │                   macros.rs, kit_browser.rs, lfo_view.rs}
│       ├── src/profiles/lcxl3/{mod.rs, palette.rs, navigation.rs, screen.rs,
│       │                       voices.rs, sequencer.rs, mix_fx.rs, perform.rs,
│       │                       lfo_mod.rs, morph_macro.rs, global.rs}
│       └── tests/{parity.rs, scenarios.rs}
├── plugin/                         nih-plug; controller.rs shrinks to thread + queue glue
└── docs/surface-library/           this plan, the authoring guide, generated docs
```

Removed at the end of the migration:

- `crates/lcxl3`, which moves to `devices/novation-lcxl3`;
- `crates/vdrum-surface` and `crates/vdrum-sim`, replaced by the `lib/*` crates and
  `vdrum-control`;
- `surface/`, `tools/surface_spec/` and `tools/surface_sim/`.

## 4. Migration plan

Each phase is one pull request and leaves the plugin working.

1. **Core and devices.**
   - Create `surface-core` and `surface-device`.
   - Move `crates/lcxl3` to `devices/novation-lcxl3`, implementing `Protocol`, and port the
     physical data from `hardware.py`.
   - Add `devices/generic-cc`.
   - Tests: protocol round trips and the emulator, for both devices.
2. **Rendering and CLI.**
   - `surface-render` and `surface-cli svg`.
   - The LCXL3 SVG must equal the one Sami confirmed (byte-compare against the Python output).
3. **Controls and display.**
   - `surface-control` stock behaviours and `surface-display`.
   - Unit tests with a fake target and a fake clock, run on both devices.
4. **Runtime.**
   - `surface-runtime`, `surface-test`.
   - A small example profile for `generic-cc` (one mode, knobs to params, toggles with LEDs) as the
     reference example.
5. **VDrum adapter.**
   - `vdrum-control`: `Target`, the commands, the custom behaviours, and the 13-mode LCXL3
     profile.
   - Parity test: the old `vdrum-surface` and the new runtime are fed the same random input
     sequences, and the device MIDI out and target operations are compared. This is the method that
     verified PR #26 (3,000 sequences × 120 inputs). It is required because "reproduce today's
     behaviour exactly" is a recorded decision.
6. **Plugin switch-over.**
   - `surface-io`; `controller.rs` uses the new runtime.
   - Delete `vdrum-surface`, the Python and the old simulator.
   - The VDrum-side hot reload (`VDRUM_SURFACE_SPEC`) loads a profile data file.
7. **Simulator and docs.**
   - `surface-sim` and `surface-cli serve`/`bundle`, with Playwright tests ported from
     `tools/surface_sim/test`.
   - Generated docs.
   - An authoring guide: how to add a device, a behaviour, a mode.

## 5. Risks

- **Size.** About 9,300 lines exist across the surface, simulator and tooling, Python included.
  Much of it is ported rather than rewritten, but phases 4 and 5 are large.
- **Over-generalising.** The second device and the parity test are the checks against it. Features
  without a second user stay in `vdrum-control`.
- **Data vs code drift.** If profiles can be authored both in Rust and in data files, the two must
  be one type, with the Rust builder producing the data. Otherwise PR #26's duplication comes back.
- **PR #26.** The Python parts are replaced. What is reused:
  - the emulator;
  - the simulator page;
  - the scenarios (as Rust);
  - the SVG geometry;
  - the parity method.

## 6. Open questions

1. **Scope.** Is the library only for plugins that embed it (VDrum, later others of yours), or is
   it also meant to control third-party plugins in a DAW? The second needs a host-wrapper plugin or
   DAW control-surface integrations, and is a different project.
2. **Where layouts live.** Should layouts and modes be Rust code compiled in, or Rust types loaded
   from data files (RON or JSON) with Rust builders as the default authoring? Data files keep hot
   reload and the browser simulator's "load a spec" feature.
3. **User-editable layouts.** Should end users of a plugin edit layouts and modes (in an editor UI
   or files), or only developers?
4. **PR #26.** Merge it as an interim state, or close it and start the library on a new branch from
   main?
5. **Exact parity.** Does "reproduce today's behaviour exactly" still apply to the rewrite?
6. **Second device.** Is a synthetic generic CC device enough, or is there a second physical device
   you own that should be the second target?
7. **Repository.** Inside `vdrum` for now (proposed), or a separate repository from the start?
8. **Name and licence.** What should the library be called? VDrum is GPL-3.0-only. A library meant
   for many devices and other projects may need MIT or Apache-2.0, which is your call as the
   author.
