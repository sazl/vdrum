//! Port of `Core/Inc/Soundengine/MIDI/Uart.h`, `Core/Src/Soundengine/MIDI/Uart.c`.
//!
//! Porting group: P3. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;

// ---- functions that use globals (methods on `Lxr`)
impl Lxr {
    /// C: `void uart_clearFrontFifo()`
    ///
    /// Source: `Core/Src/Soundengine/MIDI/Uart.c:42`. Reference build: out-of-line in flash. Group P3.
    ///
    /// Hardware / UI function outside the engine, called from engine code. Decide its behaviour
    /// (no-op, fixed value, or an emulation of its effect on engine state) when porting the caller.
    pub fn uart_clearFrontFifo(&mut self) {
        todo!("port: Core/Src/Soundengine/MIDI/Uart.c:42")
    }

    /// C: `void uart_processMidi()`
    ///
    /// Source: `Core/Src/Soundengine/MIDI/Uart.c:88`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `fifo_midiRx`.
    /// Calls: `fifo::fifo_bufferOut`, `Lxr::midiParser_parseUartData`.
    pub fn uart_processMidi(&mut self) {
        todo!("port: Core/Src/Soundengine/MIDI/Uart.c:88")
    }

    /// C: `void uart_sendMidi(MidiMsg msg)`
    ///
    /// Source: `Core/Src/Soundengine/MIDI/Uart.c:97`. Reference build: out-of-line in flash. Group P3.
    ///
    /// Hardware / UI function outside the engine, called from engine code. Decide its behaviour
    /// (no-op, fixed value, or an emulation of its effect on engine state) when porting the caller.
    pub fn uart_sendMidi(&mut self, msg: MidiMsg) {
        todo!("port: Core/Src/Soundengine/MIDI/Uart.c:97")
    }

    /// C: `void uart_sendFrontpanelByte(uint8_t data)`
    ///
    /// Source: `Core/Src/Soundengine/MIDI/Uart.c:137`. Reference build: out-of-line in flash. Group P3.
    ///
    /// Hardware / UI function outside the engine, called from engine code. Decide its behaviour
    /// (no-op, fixed value, or an emulation of its effect on engine state) when porting the caller.
    pub fn uart_sendFrontpanelByte(&mut self, data: u8) {
        todo!("port: Core/Src/Soundengine/MIDI/Uart.c:137")
    }

    /// C: `void uart_sendFrontpanelSysExByte(uint8_t data)`
    ///
    /// Source: `Core/Src/Soundengine/MIDI/Uart.c:155`. Reference build: out-of-line in flash. Group P3.
    ///
    /// Hardware / UI function outside the engine, called from engine code. Decide its behaviour
    /// (no-op, fixed value, or an emulation of its effect on engine state) when porting the caller.
    pub fn uart_sendFrontpanelSysExByte(&mut self, data: u8) {
        todo!("port: Core/Src/Soundengine/MIDI/Uart.c:155")
    }
}
