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
    todo!("port: Core/Src/Soundengine/DSPAudio/automationNode.c:17")
}

// ---- functions that use globals (methods on `Lxr`)
impl Lxr {
    /// C: `void autoNode_setDestination(AutomationNode* node, uint16_t dest)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/automationNode.c:22`. Reference build: out-of-line in flash. Group P1.
    /// Globals: `midiParser_originalCcValues`.
    /// Calls: `Lxr::midiParser_ccHandler`.
    pub fn autoNode_setDestination(&mut self, node: &mut AutomationNode, dest: u16) {
        todo!("port: Core/Src/Soundengine/DSPAudio/automationNode.c:22")
    }

    /// C: `void autoNode_updateValue(AutomationNode* node, uint8_t val)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/automationNode.c:44`. Reference build: out-of-line in flash. Group P1.
    /// Calls: `Lxr::midiParser_ccHandler`.
    pub fn autoNode_updateValue(&mut self, node: &mut AutomationNode, val: u8) {
        todo!("port: Core/Src/Soundengine/DSPAudio/automationNode.c:44")
    }
}
