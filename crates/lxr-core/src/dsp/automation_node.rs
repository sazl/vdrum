//! Port of `Core/Inc/Soundengine/DSPAudio/automationNode.h`, `Core/Src/Soundengine/DSPAudio/automationNode.c`.
//!
//! Porting group: P1. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;

// ---- types
/// C: `struct AutomationStruct` (Core/Inc/Soundengine/DSPAudio/automationNode.h:21), 2 bytes.
/// C typedef name `AutomationNode`; the struct tag `AutomationStruct` is a type alias below.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AutomationNode {
    /// `uint16_t destination; / **< dest param nr * /`
    pub destination: u16,
}
/// C struct tag of `AutomationNode`.
pub type AutomationStruct = AutomationNode;
impl AutomationNode {
    /// All-zero value (C static zero initialisation).
    pub const ZERO: Self = Self { destination: 0 };
}
impl Default for AutomationNode {
    fn default() -> Self {
        Self::ZERO
    }
}

// ---- functions without access to globals
/// C: `void autoNode_init(AutomationNode* node)`
///
/// Source: `Core/Src/Soundengine/DSPAudio/automationNode.c:17`. Reference build: out-of-line in flash. Group P1.
pub fn autoNode_init(node: &mut AutomationNode) {
    node.destination = NO_AUTOMATION as u16;
}

// ---- functions that use globals (methods on `Lxr`)
impl Lxr {
    /// C: `void autoNode_setDestination(AutomationNode* node, uint16_t dest)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/automationNode.c:22`. Reference build: out-of-line in flash. Group P1.
    /// Globals: `midiParser_originalCcValues`.
    /// Calls: `Lxr::midiParser_ccHandler`.
    pub fn autoNode_setDestination(&mut self, node: &mut AutomationNode, dest: u16) {
            if node.destination != NO_AUTOMATION as u16 {
            let d = node.destination;
            let mut msg = autoNode_ccMsg(d);
            msg.data2 = self.midiParser_originalCcValues[d as usize];
            self.midiParser_ccHandler(msg, 0, 0, 0.0);
        }
        node.destination = dest;
    }

    /// C: `void autoNode_updateValue(AutomationNode* node, uint8_t val)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/automationNode.c:44`. Reference build: out-of-line in flash. Group P1.
    /// Calls: `Lxr::midiParser_ccHandler`.
    pub fn autoNode_updateValue(&mut self, node: &mut AutomationNode, val: u8) {
            if node.destination != NO_AUTOMATION as u16 {
            let mut msg = autoNode_ccMsg(node.destination);
            msg.data2 = val;
            self.midiParser_ccHandler(msg, 0, 0, 0.0);
        }
    }
}

/// The `MidiMsg` (without `data2`) that automationNode passes to `midiParser_ccHandler` for a destination
/// (`lastNonSysStatus` and `bits` are uninitialised stack memory in the firmware and 0 here).
#[inline]
pub fn autoNode_ccMsg(destination: u16) -> MidiMsg {
    if destination > 127 {
        MidiMsg { status: MIDI_CC2 as u8, data1: (destination - 127 - 1) as u8, ..MidiMsg::ZERO }
    } else {
        MidiMsg { status: MIDI_CC as u8, data1: destination as u8, ..MidiMsg::ZERO }
    }
}
