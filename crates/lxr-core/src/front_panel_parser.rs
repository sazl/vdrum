//! Port of `Core/Inc/frontPanelParser_ui.h`, `Core/Src/frontPanelParser.c`.
//!
//! Porting group: P3. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;

// ---- #define constants
/// `#define CC_2 0xd6` (Core/Inc/frontPanelParser_ui.h:48)
pub const CC_2: i32 = 214;
/// `#define CC_LFO_TARGET 0xd7` (Core/Inc/frontPanelParser_ui.h:49)
pub const CC_LFO_TARGET: i32 = 215;
/// `#define CC_VELO_TARGET 0xd8` (Core/Inc/frontPanelParser_ui.h:50)
pub const CC_VELO_TARGET: i32 = 216;
/// `#define NRPN_MUTE_1 1000` (Core/Src/frontPanelParser.c:44)
pub const NRPN_MUTE_1: i32 = 1000;
/// `#define NRPN_MUTE_7 1006` (Core/Src/frontPanelParser.c:45)
pub const NRPN_MUTE_7: i32 = 1006;

// ---- functions that use globals (methods on `Lxr`)
impl Lxr {
    /// C: `void frontPanel_sendData(uint8_t status, uint8_t data1, uint8_t data2)`
    ///
    /// Source: `Core/Src/frontPanelParser.c:617`. Reference build: out-of-line in flash. Group P3.
    /// Calls: `Lxr::frontParser_parseUartData`.
    pub fn frontPanel_sendData(&mut self, status: u8, data1: u8, data2: u8) {
        todo!("port: Core/Src/frontPanelParser.c:617")
    }
}
