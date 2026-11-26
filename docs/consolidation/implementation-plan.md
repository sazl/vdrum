# VDrum consolidation: implementation plan

Status: draft 1, 2026-11-26. This plan builds on [review.md](review.md), which has the inventory, the conflicts and the earlier questions. It is based on Sami's answers of 2026-11-26 (section 1) and the chosen layout: **proposal B (tracks and inspector) with the Mix, FX, Modulation and Library pages from proposal C**. Section 11 lists the questions that are still open, together with the assumptions this draft makes for them.

Goal: one product and one plugin, **VDrum**, in `sazl/vdrum`. It includes every voice, parameter, effect, modulation source and feature of VDrum, Hexa FM Drum and nano-thump, except the vdrum features that were removed earlier. Section 10 is the traceability matrix that checks this.

## 1. Decisions (Sami, 2026-11-26)

| # | Decision |
|---|---|
| D1 | The work happens in the `sazl/vdrum` repository. |
| D2 | There is one product name and one plugin: VDrum. The hxfmdrum and nano-thump code is merged into VDrum. |
| D3 | The nano-thump lookup tables and factory kits are ported to Rust source. nano-thump is not optional and the build does not need `NT_FIRMWARE`. |
| D4 | Features already removed from vdrum stay out: the sequencer port, Perform, Morph/Macro, LCXL3 and Move. |
| D5 | There are **10 tracks**. Any voice type from any of the three machines can be assigned to any track. |
| D6 | FX buses, the compressor and LFOs are **decoupled from tracks**. The compressor is an ordinary FX. |
| D7 | Each track has **2 FX slots** and can use any FX type. There is also a **master bus with 4 FX slots**, each of which can use any FX type. |
| D8 | The sequencer has 16 steps, 16 patterns (A–P) and per-track length and speed, as in hxfmdrum. |
| D9 | Every parameter can have a modulation lane, created on demand. |
| D10 | Every track has a Velocity lane. |
| D11 | Output does not have to be bit-exact against the firmware. Merging the three machines comes first. |
| D12 | Everything works across engines: modulation and FX can reach any track. There is one engine. |
| D13 | Choke groups can include any track and are flexible. Mute and solo are flexible on every track. |
| D14 | MIDI: one channel per track (1, 2, 3, … ascending), vdrum's CC handling and MIDI learn on all tracks, program change selects the kit/pattern, and MTS-ESP and pitch bend apply to every track. |
| D15 | The LXR St1/St2/L1/R1/L2/R2 routing is removed. Instead there are 10 stereo aux outputs (one per track) and 1 stereo master/mix output. |
| D16 | Storage: a **preset** is the configuration of one track/voice. A **kit** is a set of presets for all tracks. A **pattern** is the sequencer state for all 10 tracks. A **project** holds kits, presets and patterns together. |
| D17 | Look: hxfmdrum's Prism style, with vdrum's resizable window and 50–300 % zoom. |
| D18 | Layout: proposal B is the main screen, with pages from C: Tracks (B), Mix, FX, Modulation and Library. |

## 2. Target architecture

### 2.1 Workspace

| Crate | Status | Content |
|---|---|---|
| `crates/vdrum-dsp` | new | Real-time engine: voice models, FX units, LFOs, modulation, choke, mixer and outputs. f32, runs at the host rate, `no_std` + `alloc`, no allocation in `process`. |
| `crates/vdrum-seq` | new (from `hxfmdrum/src/seq`) | Patterns, tracks, lanes, the clock, MIDI export. |
| `crates/vdrum-model` | new; absorbs `lxr-params` metadata | Parameter registry (IDs, names, ranges, display text), and the Preset, Kit, Pattern and Project types with serde and versioning. |
| `crates/vdrum-import` | new; absorbs `vdrum-engine/lxr_files.rs` and `lxr-params/kit.rs` | Import of LXR `.snd` files and LXR-02 project folders, Hexa JSON presets and nano-thump factory kits. Export of LXR-02 projects. |
| `crates/nt-tables` | new | The nano-thump tables (`CURVE`, `SINE`, `WAVE`, `COEF`, `SHAPE`, `PITCH`, `RANGE`, CC maps, `NOTE_PART`) and the 16 factory kits/patterns, as generated Rust source (see 3.1). |
| `plugin/` | rewritten | nice-plug shell: host parameters, state, MIDI, OSC and the editor. |
| `crates/lxr-core`, `lxr-emu`, `lxr-difftest` | kept as the reference | No longer linked into the plugin. Used as test oracles by the sound-alike tests (section 9) until those are stable. Removing them is a separate decision for later. |
| `crates/vdrum-engine` | removed after migration | Its library and file-format code moves to `vdrum-import` and `vdrum-model`. |
| `crates/lxr-params` | folded into `vdrum-model` | The generated tables stay, under `vdrum-model/src/lxr/`. |

The nano-thump `nt-syro` and `nt-emu` crates are not imported. `nt-core` is imported as `crates/nt-core` (reference only), with its `build.rs` reading `nt-tables` instead of the firmware.

### 2.2 Framework

* Move the plugin to **nice-plug 0.4.2 / nice-plug-egui 0.6 / egui 0.37**, which hxfmdrum uses. nice-plug is a fork of nih-plug with nearly the same API, so the plugin shell ports mostly unchanged.
* Retire the vendored `third_party/nih_plug_egui` and `egui-baseview` (from vdrum) and `crates/nih_plug_egui` (from nano-thump). Before deleting them, re-check vdrum's zoom and resize fixes against nice-plug-egui (`third_party/README.md` lists them). Port any fix that is still needed as a small patch crate.
* New crates use Rust edition 2024 and Rust 1.88 as the minimum version, matching hxfmdrum.
* The licence stays **GPL-3.0-only**. hxfmdrum and nano-thump code is relicensed when it is moved in; Sami is the author of both.

### 2.3 Signal flow

```
            ┌──────── track n (n = 1..10) ────────┐
trigger ──► │ voice (any type) → FX slot 1 → FX slot 2 → level / pan / mute ─┼──► aux out n (stereo)
            └──────────────────────────────────────┘                  │
                                                     (track "to mix" on)│
                                                                       ▼
                                            Σ tracks → master FX 1..4 → master level → main out (Mix)
```

* The engine runs at the host sample rate. Control-rate work happens every 32 samples: modulation, LFOs, parameter smoothing and voice coefficients. Triggers and MIDI are sample-accurate. Nothing is resampled and the plugin reports no latency (resolves C3).
* Aux out n carries track n after its FX, level and pan, and before the master bus. The per-track "to mix" switch (default on) controls whether the track is also summed into the master. Layout 2, "Mix only", is kept for hosts without multi-out.
* The nano-thump output stage (chorus, soft clip, compressor) and the LXR FX bus are no longer fixed. They become FX types (section 4.3). The **default kit** puts NT Chorus → NT Soft Clip → Compressor on the master, so that NT kits imported unchanged sound close to the original.

### 2.4 Voice types

The voice-type field of a track selects one of these models, all in `vdrum-dsp/src/voice/`:

| Voice type | Source | Parameters (approx.) | Notes |
|---|---|---|---|
| `lxr.drum` | LXR Drum 1–3 (`DrumVoice`) | 30 | Osc (wave, coarse, fine, PW), FM (mode, wave, amount, freq), Amp EG, Mod EG, Filter, Transient, Voice drive, Decimation, Velocity mod |
| `lxr.snare` | LXR Snare | 30 | adds Noise freq, Noise/Osc mix and Repeat; no FM |
| `lxr.cymbal` | LXR Cymbal | 30 | 3 waves, 2 mod oscs, Repeat |
| `lxr.hihat` | LXR Hi-hat (closed and open) | 31 | Decay Closed, Decay Open, **Mode** (Closed / Open / Velocity: velocity ≥ 64 plays open) |
| `nt.part` | nano-thump part | 21 + Link | 2 layers (Type, Level, Mod Amount, Mod Rate, Pitch, Attack, Release) and the part stage (Bit Reduction, Fold, Drive, Pitch Mode). The part's **FX Send becomes the send level to a track FX slot** (section 4.2). The part's Pan and Level map to the track's pan and level. |
| `hx.fm` | Hexa FM voice | 10 | Start, End, Speed, Ratio, Wave, Feedback, Release 1/2, Algorithm. Level and Pan map to the track's level and pan. |
| `none` | – | 0 | empty track |

Each type can be used on any number of tracks at once: every track owns its own voice instance (C5 no longer applies). The per-voice LXR LFO parameters (waveform, frequency, sync, amount, offset, retrigger, destination voice, destination) move into the global LFO pool (section 5). Velocity modulation (amount, destination, volume-mod switch) stays per track and becomes general (section 5.2).

**Common track parameters**: Voice type, Level (fader, with the vdrum square-law taper in dB), Pan, Mute, Solo, To mix, MIDI channel, Note mode, Root note, Choke group and Pitch bend range.

### 2.5 Porting the voice models

1. **Hexa FM**: copied as it is (`dsp/voice.rs`, `dsp/wave.rs`). It already runs at the host rate.
2. **LXR voices**: extracted from `lxr-core` into instance structs.
   * Firmware globals indexed by `voiceNr` become fields of the instance.
   * `activeParameters[]` lookups become a typed parameter struct.
   * Constants based on `REAL_FS` take the sample rate.
   * Lookup tables are reused from `lxr-core/src/tables`.
   * i16 buffers become f32, with the level calibrated so that full scale is unchanged (1/32768).
   * The `fpu::fma` and `libm` ports are no longer needed; std f32 math is enough.
3. **nano-thump part**: extracted from `nt-core/src/dsp.rs`.
   * `Voice` and `PartMix` become instance structs.
   * The 24 kHz voice rate becomes the host rate. Phase increments and envelope rates are scaled by `24000 / fs`.
   * The 2× upsampling allpasses are removed, because the voices already run at the output rate.
   * The `stale` carrier types 5..=7 are a firmware quirk that reads voice 0's state. They are not reachable from the 45 valid select codes, so they are dropped.
   * The Q31 phase arithmetic stays.
4. **Hi-hat open/closed**: vdrum's split open hat becomes two tracks of `lxr.hihat` (Mode Closed and Mode Open) in one choke group. The import does this automatically (section 7.3).

Each port gets sound-alike tests against the reference crate (section 9.2).

## 3. Firmware data in Rust (D3)

### 3.1 nano-thump tables

* `tools/nt_gen_tables.py` (new) reads the firmware WAV or flash image. It uses the same offsets as `nt-core/build.rs` and `nt-syro`.
* It writes `crates/nt-tables/src/generated.rs`: `pub static CURVE: [f32; N] = [f32::from_bits(0x…), …];` for each table, plus the 16 factory kits and patterns as `[u8; 0xe0]` and `[u8; 0xcf8]` byte arrays.
* The generated file is committed. A header comment records the firmware file's SHA-256 and the generator version.
* `nt-core/build.rs` is removed; `nt-core::fw` re-exports `nt_tables`.
* **Input needed:** the firmware file (`nano_thum_sys_0114.wav` or the decoded flash image) is not in this environment. Either Sami runs the generator once on their computer, or I run it once the file is given to the session (question Q1).
* **Note:** this commits data extracted from proprietary firmware to a public GPL repository. Sami decided this (D3); it is recorded here so that it is not a surprise later.

### 3.2 LXR

The LXR tables are already Rust source (`lxr-core/src/tables`, generated from the GPL firmware source). No change is needed.

## 4. FX

### 4.1 The FX trait

```rust
pub trait Fx {
    fn reset(&mut self);
    fn set_sample_rate(&mut self, fs: f32);
    fn set_param(&mut self, p: u8, v: f32);   // p = 0..FX_PARAMS
    fn process(&mut self, l: &mut [f32], r: &mut [f32], ctx: &FxCtx);  // ctx: tempo, transport
}
```

* An `FxSlot` holds `Type` (enum), `Mix` (dry/wet), `P1..P6` and a preallocated instance of **every** FX type. All delay lines are allocated in `initialize()`, so changing the type does not allocate. Memory per slot is up to about 3 MB (the NT reverb ring holds 0x40000 stereo frames). 24 slots therefore use about 70 MB.
* Alternative: allocate only the selected type on a background thread and swap it in lock-free. This saves memory, but a type change takes effect one buffer late. Recommendation: preallocate first; measure memory in phase 4 and switch if it is a problem.

### 4.2 Sends

* nano-thump's per-part FX Send, and vdrum's voice routing to the FX bus (Audio Out = FX), are both sends into a shared effect.
* To keep this, each FX slot has **Mode: Insert / Send**.
  * **Insert** processes the track's signal (dry/wet mix inside the slot).
  * **Send** on a master slot sums `track × send_level[track][slot]` for every track and adds 100 % wet output to the master.
* Each track therefore has 4 send levels, one per master slot, which are 0 unless set.
* This is the only way to get "several tracks share one reverb" without a dedicated FX bus. It is the superset of both machines' routing.
* An NT part's FX Send maps to the send level for the master slot that holds the imported NT reverb or delay.

### 4.3 FX types

| FX type | Source | Parameters (P1–P6) |
|---|---|---|
| Off | – | – |
| Drive | LXR FX drive | Type (Tube / Fold / Clip), Drive, Tone, Level |
| Ring Mod | LXR | Wave (6), Frequency |
| Compressor | LXR FX compressor | Ratio (1:1–8:1), Threshold, Attack, Release, Makeup |
| Bus Compressor | nano-thump output compressor | Amount |
| Delay | LXR FX delay | Type (Delay / Ping-pong), Time, Feedback, Tone, Sync |
| Sync Delay | nano-thump delay | Amount, Tone, Time (tempo-synced) |
| Reverb | nano-thump reverb | Amount, Tone, Time |
| Plate | LXR `reverb.rs` (in the port, compiled out of the firmware with `USE_REVERB = 0`) | Size, Damp, Predelay, Reflections |
| Chorus | nano-thump output stage | Depth, Rate, Mix |
| Soft Clip | nano-thump output stage | Drive |
| Decimator | LXR voice and global sample rate | Rate |
| Bit Reducer / Fold | nano-thump part stage | Bits, Fold, Drive |

* Parameter labels change with the type, as vdrum's FX bus panel already does. Host parameters are named "T3 FX1 P2" and similar, because host parameter names cannot change at run time (C18).
* The exact P1–P6 assignment per type is fixed in phase 4 from each source's code. The table above lists what each source has.

## 5. Modulation (D6, D9, D12)

### 5.1 Global LFOs

* There is a pool of **8 LFOs**, not tied to any track (question Q2 asks whether 8 is the right number).
* Each LFO uses the LXR algorithm: 8 waveforms (sin, tri, saw up, saw down, square, random, exp up, exp down), Frequency, Clock sync (off, 4/1 … 1/32), Amount, Offset, Retrigger (off, or any track 1–10), and a **Destination**.
* A destination can be any modulatable parameter of any track, voice, FX slot, master FX slot, or another LFO.
* Phase/retrigger and sync follow `lfo.rs`, which takes the tempo from the clock.

### 5.2 Velocity modulation

* Per track: Velocity Amount, Destination (any parameter, on any track), and Velocity → Volume on/off.
* This keeps the LXR semantics and extends the target range. nano-thump's built-in "velocity scales layer level" becomes the `nt.part` default of Velocity → Volume on.

### 5.3 Voice-internal modulation

These stay inside their voice models: the LXR Mod EG and FM, nano-thump's per-layer modulator oscillator (Mod Rate/Amount), and Hexa's Start→End glide.

### 5.4 Step lanes

These are the sequencer lanes from section 6.

### 5.5 Applying modulation

* `vdrum-model` gives every modulatable parameter a `ModTarget(u32)`: (scope, track, slot, index).
* Each control block, the modulation matrix sums the LFO, velocity and lane offsets on the normalized value. It clamps the result, and the voice or FX converts it to plain units.
* Lane semantics follow Hexa: the knob is the minimum, and value = knob + lane × (1 − knob).
* LFOs are bipolar around the knob value, and velocity mod adds `amount × velocity`, as in LXR.

## 6. Sequencer (D8, D9, D10)

* Port `hxfmdrum/src/seq` to `vdrum-seq` with `NUM_TRACKS = 10`. The lock-free atomic store, the clock (`for_each_due_step`), the speeds (/2 … x12), lengths 1–16, Delay, Random Skip, dice, clear, the 16 patterns with copy/paste/clear, the `Selected Pattern` host parameter and MIDI export are all kept.
* **Lanes per track:**
  * fixed lanes: Gates, Velocity (default 0.8 for new steps), Delay, Random Skip;
  * **dynamic parameter lanes**: up to 16 per track, created by right-clicking a knob. A lane is keyed by `ModTarget`, so a lane can also target the track's FX slots and send levels.
  * The view dropdown lists the fixed lanes and the lanes in use.
  * Storage per pattern and track: a small map from target to `[f32; 16]`, preallocated with capacity 16. It is written from the editor through the existing atomic slots, which hold the target ID and the values.
* **Voice type change:** lanes whose targets the new type does not have are kept, but greyed out and not applied, so switching back restores them.
* **Triggering:** a due step with its gate on fires `trigger(velocity)` on the track's voice, applies the lanes for that hit (held until the next hit, as in Hexa), and applies chokes (section 8.1).
* **Pattern → kit link** (nano-thump's pattern → kit): each pattern has an optional *Kit* field. When the pattern becomes active and the field is set, that kit is loaded at the next bar line. See Q3.
* **Clock:** host transport, or in the standalone app the internal clock (Run, Tempo). The tempo range is 10–600 BPM, the superset of Hexa's 40–300 and nano-thump's 10–600. The tempo also drives LFO sync and the tempo-synced FX.

## 7. Data model and files (D16)

### 7.1 Types

```text
Preset   { name, voice_type, voice_params, track_params (level, pan, note mode, root,
           bend range, velocity mod), fx_slots[2] (type, mode, mix, P1..P6), sends[4] }
Kit      { name, presets[10] (inline copies), master_fx[4], lfos[8], choke matrix, mute/solo }
Pattern  { name, tracks[10] { length, speed, gates[16], velocity[16], delay[16],
           skip[16], lanes: [(ModTarget, [f32;16])] }, kit: Option<KitRef> }
Project  { kits: Vec<Kit> (64 slots), presets: Vec<Preset> (library), patterns[16],
           current kit, selected pattern, global settings (tempo, MIDI, OSC) }
```

* LFOs and the master FX belong to the kit, so that a kit sounds the same wherever it is loaded. This needs confirming (Q4).
* Choke and mute/solo are also stored in the kit, and can be overridden per pattern (Q4).
* Files are JSON with `"format": "vdrum", "version": 1`. The extensions are `.vdpreset`, `.vdkit`, `.vdpattern` and `.vdproject`.
* The plugin state is the Project.

### 7.2 Library

* There are 64 kit slots per project (as in vdrum and the LXR-02), and a preset library inside the project.
* User presets and kits also live as files in a user folder, as Hexa's user presets do.
* The vdrum library functions are kept: ◀ ▶ browse, save to slot, rename, move, clear, export one file, and listing what an import replaces.
* Factory content:
  * Hexa's 5 presets (Starter Kit, Polymeter Study, Clonk Machine, Noise Garden, Blank Slate), each as a kit plus patterns;
  * the 16 nano-thump factory kits and its Init kit;
  * the LXR Init kit.

### 7.3 Imports and exports

| Format | Direction | Mapping |
|---|---|---|
| LXR `.snd` (255 B) and VDrum `.snd` (+ `VDX1` open-hat block, 297 B) | import | Tracks 1–7: Drum 1–3, Snare, Cymbal, Hat (Closed), Hat (Open). Choke group 1 = tracks 6, 7. LXR LFO 1–6 (and the open hat's LFO) → global LFOs 1–7, with each destination translated to (track, parameter). The FX bus → master slot 1 in Send mode, with the voices' Audio Out = FX turned into send levels. Faders → track levels. |
| LXR-02 project folder | import | `.snd` files → kit slots, as in the vdrum library import (init kits skipped, duplicates detected). |
| LXR-02 project folder | export | Only for kits whose tracks 1–7 match the LXR layout and whose FX can be expressed by the LXR FX bus. Anything else is listed as "not exported", as vdrum already does for the open hat. |
| Hexa JSON preset | import | Voices 1–6 → tracks 1–6 (`hx.fm`). The 16 patterns → patterns A–P: Hexa's 10 parameter lanes become dynamic lanes, and Delay and Skip are kept. Mute/solo are kept. |
| nano-thump factory kits and patterns | import | Parts 1–6 → tracks 1–6 (`nt.part`). FX bytes → a master slot (Reverb or Sync Delay) in Send mode, with each part's FX Send as its send level. The fixed output stage → master Chorus, Soft Clip and Bus Compressor. Pattern mute mask → track mutes. Choke mask → one choke group. |
| `.mid` | export | Hexa's MIDI export, extended to 10 tracks. Velocity comes from the Velocity lane, and pitch from the root note for every voice type. |

## 8. Plugin shell

### 8.1 Choke and mute (D13)

* Choke is a **10 × 10 matrix**: "a trigger on track *i* silences track *j*".
* "Group" presets fill it symmetrically, which reproduces nano-thump's choke mask. A one-way entry reproduces vdrum's closed-chokes-open behaviour.
* A choked voice gets a fast release (2 ms) instead of being cut hard.
* Mute and solo: one host parameter each per track, as in Hexa. Muted voices keep running silently. Solo is exclusive-additive: any track soloed means only soloed tracks are heard.

### 8.2 MIDI (D14)

* **Channel:** the default for track *n* is channel *n*. It can be set to 1–16 or Off, and tracks may share a channel.
* **Note mode** per track:
  * *Pitch* (the default): the note transposes the voice by `note − root` semitones. MTS-ESP, when connected, gives the frequency ratio `f(note)/f(root)`. The default root is C3 (60).
    * `hx.fm` multiplies Start and End by the ratio (Hexa used absolute note frequency; with root = the note of Start this is the same).
    * LXR voices scale oscillator, FM and mod-osc frequencies.
    * `nt.part` offsets the layer pitch.
  * *Trigger*: any note triggers at the programmed pitch.
  * *Note filter*: only one note triggers. This keeps the old vdrum behaviour of several tracks on one channel with different notes.
* **Pitch bend:** per-track range, default ±2 semitones, on every voice type.
* **Velocity:** the note-on velocity drives the track's velocity modulation and Velocity → Volume, as a sequencer hit's Velocity lane does.
* **CC:**
  * vdrum's MIDI learn is extended to every host parameter. It works by right-click, saves (channel, CC) → parameter ID, and has Forget and Forget all.
  * A per-track **CC layout** option (default Off) maps CCs on the track's channel to the voice type's parameters in a fixed order.
  * The LXR and NT firmware CC tables are used as the starting order for their types. Q5 asks whether this is wanted, or whether MIDI learn alone is enough.
* **Program change:** on any track channel, or on a global "Control channel" (default Off), program *p* selects pattern *p* (A–P for 0–15). Through the pattern's kit link this also selects its kit. Q3 asks whether the kit should instead be selected directly with CC 0 (bank) + program.
* **MTS-ESP:** the client moves from Hexa (`vendor/mts-esp`, 0BSD) and is used for every track.

### 8.3 Host parameters

| Group | Count |
|---|---|
| Per track: voice parameters, as the union of all types with stable IDs `t{n}_{type}_{param}` | ≈ 150 |
| Per track: common parameters (type, level, pan, mute, solo, to mix, bend, velocity mod 3) | 10 |
| Per track: FX slots 2 × (type, mode, mix, P1–P6) | 18 |
| Per track: sends | 4 |
| **Per track total** | **≈ 182** |
| 10 tracks | ≈ 1820 |
| Master FX 4 × 9, master level | 37 |
| LFOs 8 × 8 | 64 |
| Selected Pattern, standalone tempo and run | 3 |
| **Total** | **≈ 1925** |

* A union per track is used because nice-plug cannot rename or re-range host parameters at run time. Parameters of a type that is not selected have no effect, and their automation is ignored.
* Alternative: 48 generic voice slots per track ("T1 V01"…) whose meaning changes with the type. That gives ≈ 730 parameters in total, but generic names in the host's automation lists.
* Recommendation: the union. CLAP and VST3 handle this count. AU in Logic should be tested in phase 9. See Q6.
* MIDI channel, note mode, root and choke are not automatable (`non_automatable`), as nano-thump's channel parameters are.

### 8.4 Outputs (D15)

* Layout 1: main "Mix" plus aux "Track 1" … "Track 10", all stereo.
* Layout 2: "Mix" only.
* `vdrum_sum_aux` and the LXR Audio Out parameters are removed.

### 8.5 Retained vdrum features

* OSC and OSCQuery: the address scheme becomes `/vdrum/track/<n>/…`, `/vdrum/fx/master/<k>/…`, `/vdrum/lfo/<k>/…`, `/vdrum/pattern`, `/vdrum/kit/…`, `/vdrum/midi`, plus transport and panic. `docs/osc-oscquery-plan.md` is updated.
* The engine fault safety net, the FTZ handling (FTZ is now allowed in the new engine, because it no longer has to be bit-exact), no allocation on the audio thread, editor sync of values changed by MIDI, OSC or kit loads, the hardware clock option (see Q7) and pads.

## 9. Verification

### 9.1 Unit and integration tests

* **Sequencer:** the Hexa tests (timing, wrap, delay, block-size independence), extended to 10 tracks, the velocity lane and dynamic lanes.
* **Model:** serde round-trips, version migration, and import golden files: one LXR `.snd`, the DLC project folder if it is provided, a Hexa JSON file, and the 16 NT factory kits.
* **Modulation:** each LFO wave, sync, retrigger, and cross-track targets, including FX slots.
* **Choke matrix, mute/solo, MIDI routing** (channel, note modes, bend, MTS fallback), **program change** and **MIDI learn**.

### 9.2 Sound-alike tests

* Each ported voice model and FX is checked against its reference implementation (`lxr-core::Lxr`, `nt-core::Engine`, Hexa `Voice`).
* The same trigger and parameters are rendered at 44.1 kHz (LXR) or 48 kHz (NT) through both implementations, and compared on:
  * the RMS envelope in 5 ms frames (within 1 dB until −60 dB);
  * the spectral centroid per frame (within 5 %);
  * the onset time (within 1 sample).
* This runs over a parameter sweep of each type (1000 random settings) and over all factory and init kits.
* The tolerances are proposals and will be adjusted per model where a deliberate change (for example removing the NT 24 kHz path) makes a difference.

### 9.3 Plugin validation

clap-validator, pluginval (strictness 10, with the editor), and `auval` for the AU. The allocation check (`assert_process_allocs`) runs in the standalone app.

### 9.4 Performance

The worst case is 10 tracks of the most expensive type, 2 Plate/Reverb FX per track, and 4 master FX with every send active. The target is below 15 % of one core at 48 kHz with 64-sample buffers on the CI Linux runner. For reference, the current LXR engine uses 1.6 % of a core for 7 voices and the FX bus.

### 9.5 Feature-superset test

`tests/superset.rs` asserts that every row of the traceability matrix (section 10) has an implementing parameter, type or feature flag in `vdrum-model`. The list is kept in `docs/consolidation/superset.toml` and checked against the registry.

## 10. Traceability matrix (superset check)

The ✓ in the "Kept" column means the feature exists in the result; "changed" means it exists in a generalised form.

| Source | Feature | Kept as |
|---|---|---|
| vdrum | Drum 1–3 voices, all parameters except LFO | `lxr.drum` ✓ |
| vdrum | Snare, Cymbal, Hi-hat voices | `lxr.snare`, `lxr.cymbal`, `lxr.hihat` ✓ |
| vdrum | Separate open hat with its own parameters and choke | `lxr.hihat` Mode Open on its own track + choke matrix (changed) |
| vdrum | 6 + 1 voice LFOs, cross-voice targets | 8 global LFOs, any target (changed) |
| vdrum | Velocity modulation, velocity → volume | per-track velocity mod, any target ✓ |
| vdrum | FX bus: drive, ring, compressor, delay, dry/wet, P1–P4 | FX types Drive, Ring Mod, Compressor, Delay; master slot Send mode (changed) |
| vdrum | Voice Audio Out St1/St2/L1/R1/L2/R2/FX, FX Audio Out | removed by D15; replaced by 10 aux + mix, and sends for "FX" |
| vdrum | Global voice sample rate | Decimator FX on the master (changed) |
| vdrum | Global tempo | standalone tempo ✓ |
| vdrum | Voice faders 0–99 (square law, dB) | track Level with the same taper ✓ |
| vdrum | Sum aux / Main-only layout | Layout 2 "Mix only" ✓ (sum switch replaced by "to mix") |
| vdrum | Hardware clock 44140.625 Hz | Q7 |
| vdrum | Per-track MIDI channel and note, global channel | per-track channel + note mode Note filter ✓ |
| vdrum | MIDI learn on every parameter, saved in state | ✓ (all ~1900 parameters) |
| vdrum | `.snd` load/save, Init, 64-slot kit library, LXR-02 project import/export, ◀ ▶ | ✓ (section 7) |
| vdrum | OSC/OSCQuery with mDNS | ✓ (new address scheme) |
| vdrum | Changed-value marker and count, tooltips (ID, CC, default), double-click reset, Shift fine, zoom 50–300 %, resizable | ✓ (marker uses a muted colour; review C22) |
| vdrum | Pads with trigger lights, meters, status line, panic | ✓ |
| vdrum | Engine fault safety net, no allocation in process | ✓ |
| hxfm | 6 FM voices, 13 parameters each | `hx.fm` on any track ✓ |
| hxfm | 8 waves, 9 ratios, FM/Mix, Start/End/Speed glide | ✓ |
| hxfm | Mute / Solo | per track ✓ |
| hxfm | Sequencer: 16 steps, length, speed /2–x12, Delay, Random Skip, dice, clear | ✓ (10 tracks) |
| hxfm | 10 parameter lanes, right-click knob → lane | dynamic lanes for every parameter ✓ |
| hxfm | 16 patterns A–P, copy/paste/clear, Selected Pattern parameter | ✓ |
| hxfm | Right-click voice/track: reset, copy, paste | ✓ (track = preset copy/paste) |
| hxfm | Host transport / standalone Run + Tempo | ✓ |
| hxfm | MIDI note → pitch, pitch bend ±2, MTS-ESP | all tracks ✓ |
| hxfm | MIDI export (pattern or track, drag) | ✓ |
| hxfm | Multi-output Mix + 6 stereo | Mix + 10 stereo ✓ |
| hxfm | JSON presets, 5 factory presets, user preset folder | ✓ (as kits + patterns) |
| hxfm | Prism theme, Barlow fonts, widgets | ✓ |
| hxfm | `render_demo` example | ✓ (renders factory kits) |
| nano-thump | 6 parts × 2 layers, 21 parameters, 45 select codes | `nt.part` ✓ |
| nano-thump | Layer link (edit both layers) | inspector Link toggle ✓ |
| nano-thump | Bit reduction, fold, drive | inside `nt.part` ✓, also as an FX type |
| nano-thump | Reverb / tempo-synced delay send, Amount, Tone, Time | FX types Reverb, Sync Delay; Send mode ✓ |
| nano-thump | Chorus stage, soft clip, compressor, master volume | FX types Chorus, Soft Clip, Bus Compressor; master level (changed) |
| nano-thump | 16 factory kits and patterns, Init store, Kit source | factory kits ✓; Kit source becomes "Factory / Init" in the library |
| nano-thump | Pattern = kit + mute mask + choke mask; program change | pattern kit link, pattern mutes, choke matrix, program change ✓ |
| nano-thump | Part MIDI channel (1–16 / Off) | track channel ✓ |
| nano-thump | Firmware CC map, layer-linked CCs | CC layout option (Q5) |
| nano-thump | Volume, Tempo 10–600, Host tempo switch | master level, tempo 10–600, host tempo follow ✓ |
| nano-thump | Velocity → level | `nt.part` default velocity → volume ✓ |
| nano-thump | Pads, level meter, redraw on change, icon | ✓ (the VDrum icon stays; Q8) |
| all | CLAP, VST3, standalone, AUv2, macOS app | ✓ |

## 11. Questions still open (assumptions in this draft)

| # | Question | Assumed |
|---|---|---|
| Q1 | Where is the nano-thump firmware file, so that the table generator can run once? | Sami runs `tools/nt_gen_tables.py` locally, or provides the file to the session. |
| Q2 | How many global LFOs? | 8 |
| Q3 | Program change: should it select the pattern (which loads its linked kit), or the kit directly via CC 0 bank + program? | Pattern, with an optional kit link |
| Q4 | Do LFOs, master FX, choke and mute belong to the kit? | Yes; mutes can also be overridden per pattern |
| Q5 | Should the per-track CC layout use the firmware CC orders (default off), or is MIDI learn alone enough? | Include it, default off |
| Q6 | Host parameters: the union per track (≈ 1900, named) or generic slots (≈ 730, generic names)? | Union |
| Q7 | Should the hardware clock option (LXR pitch × 44140.625/44100) stay, as a per-track LXR tuning switch? | Keep it as a global "LXR hardware tuning" switch |
| Q8 | Which icon and branding: vdrum's current (none), or nano-thump's icon? | Keep VDrum's; add an icon later |

## 12. Phases

Each phase is one PR into `main`, opened and not merged until Sami reviews it. Docs are updated in the same PR.

| Phase | Content | Depends on | Size |
|---|---|---|---|
| 0 | This plan and the review (PR #33) | – | done when approved |
| 1 | Framework move to nice-plug / egui 0.37; Prism theme and widgets copied from hxfmdrum; the current VDrum keeps working; CI green | 0 | M |
| 2 | `nt-tables`: generator plus generated source; `nt-core` imported as a reference crate; NT factory kits available | 0, Q1 | S |
| 3 | `vdrum-model`: parameter registry, `ModTarget`, the Preset/Kit/Pattern/Project types, serde | 0 | M |
| 4 | `vdrum-dsp` voices: `hx.fm`, `lxr.*`, `nt.part`, with sound-alike tests | 2, 3 | L |
| 5 | `vdrum-dsp` FX: the FX trait, all 13 types, slots, insert/send, master bus; tests | 3 | L |
| 6 | Modulation: LFO pool, velocity mod, matrix; tests | 3, 4, 5 | M |
| 7 | `vdrum-seq`: Hexa sequencer at 10 tracks, Velocity lane, dynamic lanes, pattern kit link, MIDI export | 3 | M |
| 8 | `vdrum-import`: LXR, Hexa and NT imports, LXR-02 export, factory content | 3, 4, 5 | M |
| 9 | Plugin shell: host parameters, state = Project, outputs, MIDI (channels, note modes, bend, MTS, learn, CC layout, program change), choke/mute, OSC; validators | 4–8 | L |
| 10 | Editor: Tracks page (B), Mix, FX, Modulation and Library pages; zoom/resize; mockups first for Sami's approval | 9 | L |
| 11 | Hardening: performance, allocation check, AU host test with the parameter count, superset test, docs (README, ARCHITECTURE, plugin README) | 10 | M |
| 12 | Retire `vdrum-engine`; decide whether to keep `lxr-emu` and `lxr-difftest` | 11 | S |

Size: S under 1 day, M 1–3 days, L 3–6 days of agent work, as a rough estimate. Phases 3 and 7 can run in parallel with 4 and 5.

## 13. Risks

| Risk | Mitigation |
|---|---|
| Sound of LXR and NT voices drifts during the port (bit-exactness dropped) | Sound-alike tests against the reference crates; A/B renders of every factory kit attached to each PR |
| About 1900 host parameters are slow in some hosts (AU/Logic) | Test in phase 9; the fallback is generic slots (Q6) |
| FX preallocation memory (about 70 MB) | Measure in phase 5; the fallback is lazy allocation off the audio thread |
| nice-plug-egui lacks vdrum's zoom/resize fixes | Port the fixes as a patch in phase 1 |
| Proprietary NT data in a public repository | Sami's decision (D3), recorded in 3.1 |
| CI currently fails on all jobs within seconds (PR #33, docs-only), cause not investigated | Fix in phase 1, since that phase needs green CI |
