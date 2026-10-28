# VDrum plugin

CLAP, VST3 and standalone builds of VDrum, built on a port of the LXR-02 sound engine. Uses
[nih-plug](https://github.com/robbert-vdh/nih-plug), pinned to rev `de421011f41a6d10fc8c7a6084e4f4dee0143683`.
An AUv2 component and a macOS standalone app come from [clap-wrapper](../au/CMakeLists.txt).

![VDrum editor](../docs/screenshot.png)

The plugin talks to the sound engine through the `Engine` trait in `src/engine.rs`. It runs
`LxrEngine`, a wrapper around `lxr_core::Lxr` (the port of the LXR-02 engine, `ActiveEngine` in
`src/lib.rs`):

* `set_parameter` is the menu path of the contract (`menu_encoderChangeParameter`); `get_parameter`
  reads `activeParameters[]`. Out-of-range indices are ignored.
* `load_kit` / `save_kit` use the `.snd` format: 8 name bytes, then the 247 kit bytes
  (`preset_loadDrumsetSD` / `preset_saveDrumsetSD`).
* Faders: see *Voice faders* below.
* `StubEngine` (a test sound, not the LXR-02) is kept for tests of the plugin layer.
* **Safety net.** A panic inside the engine (an engine bug) must not unwind into the host, which
  would abort it. `LxrEngine` catches it, marks itself faulted, then renders silence and ignores
  input. `initialize()` replaces a faulted engine; the host parameters restore the kit. This path
  is not real-time safe (unwinding allocates), and it only runs after a bug.
* **Temporary workaround.** MIDI CC 0 (bank change) values 64–127 are dropped before the engine.
  The firmware has 64 kit slots and reads past them; `lxr-core` currently panics there. See
  `apply_event` in `src/lib.rs`. Remove this once `lxr-core` handles it.

### Performance

`cargo run --release -p vdrum --example engine_bench` retriggers all seven tracks every 8 ticks.
All voices are routed to the FX bus, which runs each FX type at 100 % wet. On the Linux x86_64 build
machine (generic `x86_64` target, no FMA, so `mul_add` is a software call) one `tick()` (64 frames)
took 23.5 µs on average, p99 39.8 µs. That is **1.6 % of one core** at 44.1 kHz, so no profiling
was needed. `new()` takes about 0.3 ms.

## Building

```sh
cargo xtask bundle vdrum --release
# target/bundled/VDrum.clap, target/bundled/VDrum.vst3, target/bundled/VDrum (standalone)

cargo xtask bundle vdrum --release --target x86_64-pc-windows-gnu    # Windows cross-build (mingw-w64)
cargo test -p vdrum -p lxr-params
```

On Linux the build needs the ALSA, JACK, X11/xcb, GL and Wayland client development packages:
`libasound2-dev libjack-jackd2-dev libgl-dev libx11-xcb-dev libxcb*-dev libxcursor-dev
libxkbcommon-dev libwayland-dev`. Wayland is required because the rfd file dialog depends on it.

The standalone takes `--backend {auto,jack,alsa,dummy}`, `--sample-rate` and `--period-size`
(run `--help` for the full list). Its default rate is 48000 Hz; pass `--sample-rate 44100` to hear
the engine output without resampling.

### AUv2 and standalone app (macOS, untested)

```sh
cargo xtask bundle-universal vdrum --release
cmake -S au -B target/au -G Xcode -DVDRUM_CLAP=$PWD/target/bundled/VDrum.clap
cmake --build target/au --config Release
```

This uses clap-wrapper v0.9.1 (commit `eb8b1901`) and produces `VDrum.component` (type `aumu`,
manufacturer `VDrm`, subtype `VDr1`) and `VDrum.app`. Each embeds a copy of `VDrum.clap`. Nobody has
built this on macOS yet, so it should be treated as unverified until macOS CI runs it.

## Audio

* **Engine rate.** The engine renders 64-frame ticks at 44100 Hz. Its output (DAC 1 and DAC 2,
  i16) is converted to f32 by dividing by 32768.
* **Outputs.**
  * Main output: DAC 1.
  * Aux output: DAC 2.
  * The *Sum aux into main* switch (parameter ID `vdrum_sum_aux`) adds DAC 2 to the main output
    and silences the aux output.
  * A second layout ("Main only") has no aux port and always sums.
* **Sample rate.** At a 44100 Hz host rate the samples pass through unmodified. At any other rate
  they are resampled with rubato (`SincFixedIn`: 128-tap sinc, BlackmanHarris² window, cubic
  interpolation, oversampling 256). The resampler delay is included in the latency the plugin
  reports to the host.
* **Hardware clock** (parameter `vdrum_hardware_clock`, not automatable). The LXR-02's real sample
  clock is 44140.625 Hz, and the engine's coefficients use `REAL_FS` = 44140. When this switch is
  on, the engine runs at 44140.625 Hz and is always resampled to the host rate, so pitches and
  times match the hardware. It is off by default, which keeps 44100 Hz pass-through bit-exact.
  Both output stages are prepared in `initialize()`, so switching does not allocate. The output
  restarts and the plugin reports the new latency.
* **Floating point mode.**
  * The firmware runs the engine with flush-to-zero off (IEEE subnormals).
  * Plugin wrappers turn FTZ/DAZ on during `process()`.
  * `fp::IeeeGuard` clears them around every engine call (`tick`, `midi_byte`, `set_parameter`,
    `load_kit`, `get_parameter`, `new`) and restores the previous state afterwards:
    * x86: MXCSR bits 15 and 6.
    * aarch64: FPCR bit 24.
* **Bit-exactness test.** `src/real_engine_tests.rs` runs the scenario corpus (`tests/scenarios`)
  through the plugin's event scheduling and output path at 44100 Hz:
  * events are placed at jittered host sample positions;
  * five host buffer patterns are used, including 1, 7, 13, 947 and 4096 frames;
  * FTZ/DAZ is switched on, as a plugin wrapper would.
  
  The output, converted back to i16, matches the reference hashes of the shipped v1.71 firmware for
  all 299 scenarios without kit loads. `global_morph_no_target` and the 11 kit-load scenarios are
  skipped. A second test checks the plugin's MIDI defaults with all voices and the FX bus against
  `Lxr` called directly.
* **No allocations or locks on the audio thread.**
  * Fixed-capacity event queue and output FIFO.
  * Preallocated resampler buffers.
  * A lock-free `ArrayQueue` for commands from the GUI.
  * Kits travel inline.
  * This was checked with the real engine, in a debug build with the `assert_process_allocs`
    feature, in the standalone on Xvfb. Pads were played, parameters and faders dragged, and the
    init kit loaded while audio ran, with no allocation panics.

## Timing and latency

MIDI events and parameter changes are scheduled onto the 64-frame ticks:

* The output is delayed by a FIFO that starts with 64 frames of silence. At other rates it is about
  64 · rate / 44100 frames, plus the resampler delay.
* A tick runs when the FIFO is empty at host time `h`. Before it runs, every pending event with a
  host time below `h` is applied.
* As a result, an event inside a tick's time span is applied at the start of that tick. With the
  reported latency (64 samples at 44100 Hz), the host lines it up at most 63 engine frames early
  and never late.
* At 44100 Hz the output depends only on the absolute sample position, not on the host's buffer
  sizes (see the `pipeline` tests).

Host parameter values are polled once per host buffer (`SAMPLE_ACCURATE_AUTOMATION = false`). A
change is applied at the start of the next tick.

## MIDI

* **Raw MIDI to the engine.** All MIDI from the host goes to the engine as raw bytes through
  `Engine::midi_byte`: note on/off, CC, program change, pitch bend and pressure. The firmware's MIDI
  parser decides what happens with them.
* **Firmware behaviours worth knowing:**
  * A track triggers when the message is on its channel and the note matches. A track note of 0
    means any note.
  * **Default note map (plugin choice).** The firmware's default globals
    (`preset_loadDefaultGlobals`, which `preset_createGlobals` also writes to a new `glo.cfg`) put all
    seven tracks on channel 1 with note 48, so one note triggers every voice. Hardware with a
    `glo.cfg` behaves the same until the user changes it.
    * The plugin instead defaults to one General MIDI drum note per track, all on channel 1:
      Drum 1 = 36 (Bass Drum 1), Drum 2 = 45 (Low Tom), Drum 3 = 50 (High Tom), Snare = 38,
      Cymbal = 49 (Crash 1), closed hi-hat = 42, open hi-hat = 46.
    * It sets them after `Lxr::new()` through the firmware's own parameter path
      (`set_parameter(PAR_MIDI_NOTEx)`, which only writes `activeParameters`). These are also the
      host parameters' defaults.
    * The sound does not change: with a track note set, a matching note plays the voice at
      `SEQ_DEFAULT_NOTE`, as note 48 did before.
    * The engine itself keeps the firmware defaults, so the scenario reference hashes still hold.
    * Projects saved with older versions restore their stored notes.
  * The global channel defaults to 0, which means off.
  * CCs 2–127 control parameters 1–126. The channel rule is `midiParser_cc2voiceNr`: drum 1's CCs
    follow the drum 2 channel (`midi_MidiChannels[n]` is the channel of track n+1).
  * Parameters 128–246 use NRPN `index − 128`: CC 99 is the MSB, CC 98 the LSB and CC 6 carries
    the 7-bit value.
  * Note-off is ignored. CC 0 loads a kit (bank change). Program change on the global channel
    selects the next pattern.
  * Each control's tooltip shows its CC/NRPN.
* **MIDI learn.**
  * To learn: right-click a control, choose *MIDI learn*, then move a controller. The next CC on
    any channel is assigned.
  * A learned (channel, CC) pair is taken out of the stream and is not passed to the engine. Its
    7-bit value is scaled linearly onto the parameter's range.
  * The learn map is saved in the plugin state under the key `midi_learn` (parameter IDs, not
    indices).
  * The context menu can forget a learned CC; the MIDI learn section has *Forget all*.

## Parameters and GUI sync

* **Host parameters.** There are 249 engine parameters: 233 sound and 16 global. Each has the ID
  of its firmware enum name, lower case and without `PAR_` (for example `coarse1` or
  `midi_chan_global`). Groups follow voice and section (`Drum 1/Oscillator`, `FX/FX Bus`,
  `Global/MIDI`). Values show the firmware's edit-mode text; modulation targets are spelled out.
* **Voice faders.** The plugin adds six parameters, `vdrum_fader_1` … `vdrum_fader_6`, for the
  hardware's volume faders (Drum 1–3, Snare, Cymbal, Hi-hat). They are not part of the kit.
  * They have steps 0–99 and go through the hardware path, `adc_checkFaders` (`set_fader_adc`).
  * Step `s` reads ADC `36 + 41·s` (12 bit). Every step therefore clears the 40-count
    hysteresis, step 0 is silence, and step 99 is the top.
  * The top (0xFFF0, gain 0.99951) is the power-on value of the contract, and it is the default.
  * The gain is the square of the fader position, displayed in dB.
* **Not exposed.** `PAR_FX_REVERB_TYPE` has a single possible value (the firmware lists one reverb
  type), and a host parameter cannot have a zero-width range. It stays in the kit and the GUI shows
  it read-only. The 13 unused kit slots are also kept in the kit but not exposed.
* **Changes from MIDI and kit loads.** The audio thread compares the engine's values after every
  tick that applied MIDI, and after kit loads, then flags the values that changed. The editor
  copies them to the host parameters with begin/set/end gestures.
  * **nih-plug limitation:** only the GUI context can set host parameters. While the editor is
    closed, host parameters fall behind such changes until the editor is next opened.
  * The engine keeps its value whenever the host parameter is stale. Host values are only sent to
    the engine when the host changes them.
  * Saved state takes not-yet-copied values from the engine, so nothing is lost.

## State and kit files

* **Plugin state.** It holds:
  * the host parameters;
  * `kit`: the `.snd` bytes (8-byte name plus 247 parameters), hex encoded;
  * `globals`: the global settings by ID;
  * `midi_learn`;
  * the editor size and zoom.
* **Restoring state.** The kit and the globals are authoritative. They are loaded into the engine
  in `initialize()`.
* **Kit files.**
  * *Load…* and *Save…* read and write the firmware's `.snd` format (`preset_loadDrumsetSD`,
    `preset_saveDrumsetSD`).
  * *Save…* suggests the firmware's file name `NN-XXXXX.snd`.
  * *Init* loads the init kit.

## GUI

* **Why egui.** The editor uses nih_plug_egui. Every parameter has to be on one screen (about 250
  controls generated from the `lxr-params` table), and immediate mode builds that from data without
  per-widget state or bindings. Vizia and iced would need a retained widget tree for every cell.
* **Layout.**
  * One card per voice. Each card holds all of that voice's sections:
    * Oscillator, FM (or the mod oscillators for the cymbal and hi-hat), Amp EG, Mod EG (pitch);
    * Filter, Transient, LFO, Velocity modulation;
    * Mix, including the fader.
  * Card headers show the trigger note and channel, with trigger pads that light when the track
    plays. The hi-hat card has CH and OH and shows the closed and open hi-hat pages.
  * Labels are spelled out ("Pitch Mod Amount", "Destination Voice"). Values are shown as the
    LXR-02 display shows them in edit mode. The tooltip has the full name and, for modulation
    destinations, the spelled-out target.
  * Values that differ from their default (the init kit, or the plugin default for globals) have an
    amber stripe. The top bar counts them.
  * The default window is 1812 × 960 and fits everything at 100 % on a 1920 × 1080 display.
  * A right-hand column holds:
    * the FX bus, with P1–P4 labelled for the current FX type from the firmware's FX page and
      controls not used by the type dimmed;
    * performance;
    * MIDI (BPM, global channel, channel and note per track);
    * output meters, the sum and hardware clock switches, and status.
  * The top bar has the kit name, Load/Save/Init, the count of changed values, status and zoom.
* **Controls.**
  * Drag a value bar to change it; hold Shift for fine steps.
  * Double-click resets to the default.
  * List parameters (waveforms, filter types, LFO and velocity targets, …) open a list on click.
    Mod-target lists offer the destination voice's range (`modTargetVoiceOffsets`).
  * Tooltips show the full name, value, range, default, CC/NRPN, learned CC and stable ID.
* **Window.** Resizable from the corner. Zoom with −/+ (persisted); click the percentage to reset
  it to 100 %. The content scrolls if it does not fit.

## Validation (Linux, x86_64)

These results are for the real engine (`lxr-core`), from `cargo xtask bundle vdrum --release`.

* **clap-validator 0.3.2** on `VDrum.clap`: 18 passed, 0 failed, 3 skipped (preset discovery is
  not implemented).
  * Before the CC 0 workaround, the parameter fuzz test failed with the `lxr-core` panic
    described above.
* **pluginval 1.0.4** on `VDrum.vst3`: strictness 10, editor tests included, run on Xvfb: SUCCESS.
  * The log still shows two panics on the editor thread when windows close: an X11 `BadDrawable`
    from `unwrap()` in `baseview::gl::GlContext::swap_buffers` (baseview `src/gl/x11.rs:237`).
  * The host destroys the parent window while baseview's render thread draws one more frame.
  * The panic ends only that GUI thread, and pluginval passes.
  * A fix needs a change in baseview: handle the error in `swap_buffers`, or stop rendering
    before the parent window goes away. It is not in the plugin's code.
* **Standalone.** Ran with `--backend dummy` at 44100 and 48000 Hz on Xvfb. Checked by driving
  the GUI with xdotool: pads trigger notes (meters move), dragging changes values, and the MIDI
  learn menu opens.
* **Windows.** The `x86_64-pc-windows-gnu` cross-build produces `VDrum.clap`, `VDrum.vst3` and
  `VDrum.exe`. They have not been run.
* **Software GL caveat.** baseview asks for an sRGB-capable GLX framebuffer, and Xvfb's software
  GLX has none. GUI runs on Xvfb therefore used a small `LD_PRELOAD` shim that drops that one
  attribute. Real X servers with GPU drivers provide such configs.
