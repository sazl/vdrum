//! Port of `Core/Inc/Soundengine/MIDI/Uart.h`, `Core/Src/Soundengine/MIDI/Uart.c`.
//!
//! Porting group: P3. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;
use crate::fifo::fifo_bufferOut;

// ---- functions that use globals (methods on `Lxr`)
impl Lxr {
    /// C: `void uart_clearFrontFifo()`
    ///
    /// Source: `Core/Src/Soundengine/MIDI/Uart.c:42`. Reference build: out-of-line in flash. Group P3.
    ///
    /// Hardware / UI function outside the engine, called from engine code. Decide its behaviour
    /// (no-op, fixed value, or an emulation of its effect on engine state) when porting the caller.
    pub fn uart_clearFrontFifo(&mut self) {
        // Front panel FIFOs are not modelled (front panel bytes are parsed synchronously): no-op.
    }

    /// C: `void uart_processMidi()`
    ///
    /// Source: `Core/Src/Soundengine/MIDI/Uart.c:88`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `fifo_midiRx`.
    /// Calls: `fifo::fifo_bufferOut`, `Lxr::midiParser_parseUartData`.
    pub fn uart_processMidi(&mut self) {
        let mut data: u8 = 0;
        while fifo_bufferOut(&mut self.fifo_midiRx, &mut data) != 0 {
            self.midiParser_parseUartData(data);
        }
    }

    /// C: `void uart_sendMidi(MidiMsg msg)`
    ///
    /// Source: `Core/Src/Soundengine/MIDI/Uart.c:97`. Reference build: out-of-line in flash. Group P3.
    ///
    /// Hardware / UI function outside the engine, called from engine code. Decide its behaviour
    /// (no-op, fixed value, or an emulation of its effect on engine state) when porting the caller.
    pub fn uart_sendMidi(&mut self, msg: MidiMsg) {
        // MIDI output (fifo_midiTx + USART3) is not modelled: it never feeds back into engine state.
        let _ = msg;
    }

    /// C: `void uart_sendFrontpanelByte(uint8_t data)`
    ///
    /// Source: `Core/Src/Soundengine/MIDI/Uart.c:137`. Reference build: out-of-line in flash. Group P3.
    ///
    /// Hardware / UI function outside the engine, called from engine code. Decide its behaviour
    /// (no-op, fixed value, or an emulation of its effect on engine state) when porting the caller.
    pub fn uart_sendFrontpanelByte(&mut self, data: u8) {
        self.frontPanel_parseData(data);
    }

    /// C: `void uart_sendFrontpanelSysExByte(uint8_t data)`
    ///
    /// Source: `Core/Src/Soundengine/MIDI/Uart.c:155`. Reference build: out-of-line in flash. Group P3.
    ///
    /// Hardware / UI function outside the engine, called from engine code. Decide its behaviour
    /// (no-op, fixed value, or an emulation of its effect on engine state) when porting the caller.
    pub fn uart_sendFrontpanelSysExByte(&mut self, data: u8) {
        // fifo_frontTx is not modelled (bytes to the AVR front panel never come back): no-op.
        let _ = data;
    }
}
