# Plan: OSC and OSCQuery server in the VDrum plugin

Status: implemented on 2026-11-14 (`plugin/src/osc.rs`). Written 2026-11-14 against commit 60f8abc; the differences from the plan are under "As implemented" at the end.

## Purpose

External programs control VDrum over a public protocol. Sami chose OSC 1.0 with OSCQuery on
2026-11-14 (in place of the surface-control wrapper plugin). The first client is
[surface-control](https://github.com/sazl/surface-control), whose matching plan is
`docs/oscquery-plan.md` in that repository. Any other OSCQuery client (for example ossia score
or Vidvox's OSCQuery browser) works the same way.

OSCQuery is the Vidvox proposal, <https://github.com/Vidvox/OSCQueryProposal> (read
2026-11-14). The parts used here:

- An HTTP/1.1 server. `GET /path` returns the OSC address space under `path` as JSON, with
  the attributes `FULL_PATH`, `CONTENTS`, `TYPE`, `ACCESS`, `VALUE`, `RANGE`, `DESCRIPTION`,
  `TAGS`, `UNIT` and `CLIPMODE`. `GET /?HOST_INFO` returns the server's name, its extensions
  and where its OSC and WebSocket servers are.
- A WebSocket on the same port. A client sends `{"COMMAND": "LISTEN", "DATA": "/path"}`, and
  the server then sends every value of that address as a binary OSC packet. `IGNORE` stops
  this.
- Discovery: the server registers the mDNS service type `_oscjson._tcp`.

## Address space

The root is `/vdrum`. The names are the existing host parameter IDs, so a client sees the
same names as a DAW.

| Address | TYPE | ACCESS | Contents |
|---|---|---|---|
| `/vdrum/param/<id>` | `i` | 3 | The 286 engine parameters (`coarse1`, `filter_freq_1`, ...). `RANGE` is `{MIN, MAX}` of `lxr_params`. `VALUE` is the engine value. `DESCRIPTION` is the parameter name. `TAGS` holds the group (`Drum 1/Oscillator`). |
| `/vdrum/param/<id>/text` | `s` | 1 | The value as VDrum displays it (`value_text`). |
| `/vdrum/fader/<n>` | `i` | 3 | The seven voice faders, 0 to 99 (`vdrum_fader_<n>`). |
| `/vdrum/fader/<n>/text` | `s` | 1 | The fader value in dB. |
| `/vdrum/switch/sum_aux`, `/vdrum/switch/hardware_clock` | `T` | 3 | The two switches. |
| `/vdrum/midi` | `m` | 2 | One MIDI message (OSC type `m`, 4 bytes), given to the engine as host MIDI is. This is how a client plays notes. |
| `/vdrum/kit/name` | `s` | 1 | The kit name. |
| `/vdrum/param/<id>/values` | `s` | 1 | Modulation destinations only: the values the destination's voice allows ("off" and that voice's targets, as the editor offers), decimal, separated by spaces. Changes with the LFO's voice. |
| `/vdrum/hits/<n>` | `i` | 1 | Counts track `n`'s triggers from any source (host MIDI, OSC, the editor's pads); wraps at 10000. |
| `/vdrum/mute/<n>` | `T` | 3 | Track mute (`seq_setMute`): the track's notes are dropped from any source; a sound that plays decays. |
| `/vdrum/lfo/<n>` | `f` | 1 | Track `n`'s LFO output, 0 to 1 in steps of 0.01 (`vdrum_lfo_values`). |
| `/vdrum/lfo/reset` | `T` | 2 | True restarts every LFO at its phase offset. |
| `/vdrum/fx/p<n>/name` | `s` | 1 | What FX parameter `n` is for the current FX type (`FX Drive`), as the editor's FX page names it. |
| `/vdrum/fx/sub` | `i` | 3 | The FX type's own type parameter (distortion type, ring modulator waveform, compressor ratio, delay type): the fourth entry of the FX page when it is not P1 to P4. `/text`: name and value, or `none for this FX`. |
| `/vdrum/transport/beat` | `f` | 1 | The host's position in beats when its transport last started (before `playing` in the address space, so a client has it first). |
| `/vdrum/transport/playing` | `T` | 1 | The host's transport plays. |
| `/vdrum/transport/tempo` | `f` | 1 | The host's tempo in bpm; 0 while unknown. |
| `/vdrum/kit/browse` | `i` | 3 | The place of the kit shown (library slot, or file in the folder of the current kit file). A higher value shows the next kit, a lower one the previous: the library's filled slots, or with an empty library the `.snd` files of that folder. `/text` is the kit shown. A set of any other address ends browsing. |
| `/vdrum/kit/browsing` | `T` | 1 | True while browse shows a kit that is not loaded; false after a load or a set of any other address. |
| `/vdrum/kit/load` | `T` | 2 | True loads the kit browse shows. |
| `/vdrum/kit/init` | `T` | 2 | True loads the init kit. |
| `/vdrum/kit/save` | `T` | 2 | True saves the kit into its library slot when it came from the library, else into its kit file. |
| `/vdrum/kit/compare` | `T` | 3 | True plays the kit as last loaded or saved; false the edited kit again. |
| `/vdrum/kit/status` | `s` | 1 | The result of the last browse, load, init, save or compare (`Kit saved: library 05 MALWA`). |
| `/vdrum/panic` | `T` | 2 | True silences every voice. |

A parameter whose range has width zero (`PAR_FX_REVERB_TYPE`) is not listed, as it is not a host
parameter. The address space does not change while the plugin runs, so `PATH_CHANGED` is never
sent.

On 2026-11-14 Sami said that none of the removed controller features come back. On 2026-11-16
Sami chose to add the state and actions a controller profile needs as OSC addresses (hits, mutes,
LFO outputs and reset, FX names, modulation destination values, host transport, kit browse, load,
init, save, compare, panic; the rows above from `/vdrum/param/<id>/values` on). Undo stays in the
controller (surface-control's `history` component). The sequencer is removed from VDrum (Sami,
2026-11-14); surface-control's sequencer plays VDrum through `/vdrum/midi`.

## Threads and data flow

1. **Server thread.** `initialize()` starts one server thread per plugin instance, as the
   removed controller thread was started. Dropping the plugin stops it. It owns:
   - a `TcpListener` for HTTP and WebSocket,
   - a `UdpSocket` for OSC,
   - the mDNS registration.
2. **Incoming values** (UDP, or binary WebSocket frames) are decoded. A value may be `i`, `f`,
   `d`, `h`, `T` or `F`; a number is rounded to the nearest step and clamped to the range. Values are pushed
   onto `Shared::commands` as new variants: `Command::Param(idx, value)`, `Command::Fader(n, step)`,
   `Command::Switch(which, on)` and `Command::Midi(bytes)`. `process()` applies them as it
   applies a learned CC: `Ev::Param { learned: true }` sets the dirty flag, so an open editor
   copies the value to the host parameter. `Command::Midi` already exists for the editor's pads.
3. **Outgoing values.** Every 10 ms the server thread compares `Shared::values`,
   `fader_values`, `sum_aux_on` and `hw_clock_on` with what it last sent. It sends each changed
   value (and its text) to every client that listens to that address. Changes from the DAW,
   the editor, MIDI, kit loads and OSC are all seen this way. Clients that use plain OSC without
   OSCQuery get the same values over UDP: the last 8 distinct addresses and ports that have sent
   a UDP message since the server started receive each changed value as `/vdrum/param/<id> <value> "<text>"` (an int32 and
   a string; the faders and switches likewise). surface-control's example target until it reads
   OSCQuery (`examples/vdrum-lcxl3` there) relies on this.
4. **Ports.** The ports are chosen by the system (bind to port 0), so several instances do not
   collide. The editor shows the ports. The mDNS instance name is `VDrum` with a number when
   more than one instance runs (`VDrum 2`). An environment variable, `VDRUM_OSC_PORT`, fixes both
   the HTTP port and the UDP port, which are then the same number (TCP and UDP ports are
   separate), for clients without mDNS. Binding is to `127.0.0.1` by default;
   `VDRUM_OSC_BIND=0.0.0.0` opens it to the network.
5. **On by default.** `VDRUM_OSC=0` disables the server, as `VDRUM_LCXL3=0` did for the
   controller.

## Crates

| Crate | Version (crates.io, 2026-11-14) | Use |
|---|---|---|
| `rosc` | 0.11.4 | OSC 1.0 encode and decode, including type `m`. |
| `tungstenite` | 0.30.0 | WebSocket server on a `std::net::TcpStream`, no async runtime. |
| `mdns-sd` | 0.21.7 | mDNS registration without an async runtime. |
| `serde_json` | already a dependency | The JSON replies. |

The HTTP part is a small hand-written GET handler on the same `TcpListener`. A request with
`Upgrade: websocket` is passed to `tungstenite`'s server handshake. The `oscquery` crate
(0.2.0, 2025-02-08) was not chosen: it depends on `tokio` and `axum`.

## Limitations

- **The DAW does not see OSC changes while the editor is closed.** In nih-plug only the GUI
  context can set host parameters ("Parameters and GUI sync" in `plugin/README.md`). An OSC
  change reaches the engine at once, but the host parameter, and with it DAW automation
  recording, follows only while the editor is open. MIDI learn has the same limitation today.
  Removing it needs nih-plug to send CLAP parameter output events or VST3 `performEdit` from
  the audio thread. That is not part of this plan.
- **Timing.** An incoming value or note takes effect at the start of the next audio block. The
  DAW does not record notes sent over OSC.
- **Real-time safety.** Only the server thread allocates. The audio thread pops commands from
  the existing lock-free queue (`ArrayQueue`, 1024 entries). A full queue drops the command,
  and the server thread counts the drops.

## Tests

- **Unit tests:** address-space JSON for one parameter of each kind; OSC decode to
  `Command`; the 10 ms diff.
- **Integration test:** start the server on a `Shared` without the plugin; fetch `/vdrum` over
  HTTP; `LISTEN` on one address over WebSocket; send a value over UDP; check that the command
  queue and the streamed value match.
- **Manual checks:** Vidvox's OSCQuery browser or libossia's client on macOS; two instances in
  one DAW; `clap-validator` passes as before.

## Decisions (Sami, 2026-11-14)

1. Addresses are `/vdrum/param/<id>`, as in the table above.
2. The server is on by default. `VDRUM_OSC=0` turns it off.
3. Groups go in `TAGS`. The names stay the host IDs.

Point 3 was not one of Sami's answers. It follows from decision 1, which keeps the host IDs as
names.

## As implemented (2026-11-14)

- **The protocol is a surface-control crate.** Sami asked that other plugins and apps make the
  same integration with standard, importable code, so HTTP, WebSocket, UDP and mDNS are the crate
  `surface-control-oscquery` (sazl/surface-control `crates/surface-control-oscquery`, MIT OR
  Apache-2.0), a git dependency pinned by commit in `plugin/Cargo.toml`. It uses the crates of
  the table above. `plugin/src/osc.rs` holds only VDrum's address space and implements the
  crate's `App` trait: `nodes`, `value`, `text`, `set`.
- **Texts** are read-only string nodes `<address>/text` for every parameter and fader.
- **Values from clients** become `Command::Param`, `Command::Fader` and `Command::Switch` on the
  audio thread, applied like a learned MIDI CC; `/vdrum/midi` becomes `Command::Midi`, as the
  editor's pads.
- **Environment:** `VDRUM_OSC=0`, `VDRUM_OSC_PORT`, `VDRUM_OSC_BIND` as planned, and
  `VDRUM_OSC_MDNS=0` to skip mDNS. The editor shows the name and ports under the sample rate.
- **Instances:** the first server in a process is `VDrum`, the next `VDrum 2`, and so on; a name
  is free again when its instance is removed.
- **Tested:** unit tests of the address space and commands; end to end on Linux, VDrum's CLAP in
  a headless host driven by `surfacectl run --simulate` (sazl/surface-control
  `docs/guide-vdrum-lcxl3.md` section 1). Not tested: a DAW, macOS, Windows, the editor's label.
