# Plan: OSC and OSCQuery server in the VDrum plugin

Status: plan, not implemented. Written 2026-11-14 against commit 60f8abc.

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

A parameter whose range has width zero (`PAR_FX_REVERB_TYPE`) is not listed, as it is not a host
parameter. The address space does not change while the plugin runs, so `PATH_CHANGED` is never
sent.

Not included, because Sami said on 2026-11-14 that none of the removed controller features come
back: kit browsing, kit save, undo, mutes and sequencer editing.

## Threads and data flow

1. **Server thread.** `initialize()` starts one server thread per plugin instance, as the
   removed controller thread was started. Dropping the plugin stops it. It owns:
   - a `TcpListener` for HTTP and WebSocket,
   - a `UdpSocket` for OSC,
   - the mDNS registration.
2. **Incoming values** (UDP, or binary WebSocket frames) are decoded and pushed onto
   `Shared::commands` as new variants: `Command::Param(idx, value)`, `Command::Fader(n, step)`,
   `Command::Switch(which, on)` and `Command::Midi(bytes)`. `process()` applies them as it
   applies a learned CC: `Ev::Param { learned: true }` sets the dirty flag, so an open editor
   copies the value to the host parameter. `Command::Midi` already exists for the editor's pads.
3. **Outgoing values.** Every 10 ms the server thread compares `Shared::values`,
   `fader_values`, `sum_aux_on` and `hw_clock_on` with what it last sent. It sends each changed
   value (and its text) to every client that listens to that address. Changes from the DAW,
   the editor, MIDI, kit loads and OSC are all seen this way.
4. **Ports.** The ports are chosen by the system (bind to port 0), so several instances do not
   collide. The editor shows the ports. The mDNS instance name is `VDrum` with a number when
   more than one instance runs (`VDrum 2`). An environment variable, `VDRUM_OSC_PORT`, fixes the
   HTTP port for clients without mDNS. Binding is to `127.0.0.1` by default;
   `VDRUM_OSC_BIND=0.0.0.0` opens it to the network.
5. **Off switch.** `VDRUM_OSC=0` disables the server, as `VDRUM_LCXL3=0` did for the
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

## Open points

1. Address naming: `/vdrum/param/<id>`, or the flat `/vdrum/<id>`.
2. Whether the server should be on by default or enabled by a switch in the editor.
3. Whether to use `TAGS` for groups, or a container per group (`/vdrum/Drum 1/Oscillator/...`).
   Containers are closer to how other OSCQuery servers group, but then the names are no longer
   the host IDs.
