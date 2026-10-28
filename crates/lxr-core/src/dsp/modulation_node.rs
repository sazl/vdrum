//! Port of `Core/Inc/Soundengine/DSPAudio/modulationNode.h`, `Core/Src/Soundengine/DSPAudio/modulationNode.c`.
//!
//! Porting group: P1. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;

// ---- types
/// C: `struct ModulatorStruct` (Core/Inc/Soundengine/DSPAudio/modulationNode.h:21), 16 bytes.
/// C typedef name `ModulationNode`; the struct tag `ModulatorStruct` is a type alias below.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ModulationNode {
    /// `uint16_t destination; / **< dest param nr * /`
    pub destination: u16,
    /// `uint8_t type; / **< pointer type * /`
    pub r#type: u8,
    /// `ptrValue originalValue; / **< stores the original value of the parameter* /`
    pub originalValue: ptrValue,
    /// `float amount; / **< modulation amount* /`
    pub amount: f32,
    /// `float lastVal;`
    pub lastVal: f32,
}
/// C struct tag of `ModulationNode`.
pub type ModulatorStruct = ModulationNode;
impl ModulationNode {
    /// All-zero value (C static zero initialisation).
    pub const ZERO: Self = Self { destination: 0, r#type: 0, originalValue: ptrValue::ZERO, amount: 0.0, lastVal: 0.0 };
}
impl Default for ModulationNode {
    fn default() -> Self {
        Self::ZERO
    }
}

// ---- functions that use globals (methods on `Lxr`)
impl Lxr {
    /// C: `void modNode_init(ModulationNode* vm)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/modulationNode.c:30`. Reference build: out-of-line in flash. Group P1.
    /// Globals: `parameterArray`.
    /// Calls: `Lxr::modNode_setDestination`.
    ///
    /// `vm` is a place accessor (see `crate::place`): the C pointer may point into `Lxr` and this
    /// function can also reach the pointee through globals, so a plain `&mut` would not be faithful.
    pub fn modNode_init(&mut self, vm: impl Fn(&mut Lxr) -> &mut ModulationNode + Copy) {
        todo!("port: Core/Src/Soundengine/DSPAudio/modulationNode.c:30")
    }

    /// C: `static void modNode_setOriginalValueChanged(ModulationNode* vm, uint16_t idx)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/modulationNode.c:37`. Reference build: inlined into its callers only. Group P1.
    /// Globals: `parameterArray`.
    pub fn modNode_setOriginalValueChanged(&mut self, vm: &mut ModulationNode, idx: u16) {
        todo!("port: Core/Src/Soundengine/DSPAudio/modulationNode.c:37")
    }

    /// C: `void modNode_originalValueChanged(uint16_t idx)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/modulationNode.c:74`. Reference build: out-of-line in flash. Group P1.
    /// Globals: `cymbalVoice`, `hatVoice`, `parameterArray`, `snareVoice`, `velocityModulators`, `voiceArray`.
    /// Calls: `Lxr::modNode_setOriginalValueChanged`.
    pub fn modNode_originalValueChanged(&mut self, idx: u16) {
        todo!("port: Core/Src/Soundengine/DSPAudio/modulationNode.c:74")
    }

    /// C: `IN_ITCM_RAM void modNode_resetTargets()`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/modulationNode.c:90`. Reference build: out-of-line in ITCM. Group P1.
    /// Globals: `cymbalVoice`, `hatVoice`, `snareVoice`, `velocityModulators`, `voiceArray`.
    /// Calls: `Lxr::paramArray_setParameter`.
    pub fn modNode_resetTargets(&mut self) {
        todo!("port: Core/Src/Soundengine/DSPAudio/modulationNode.c:90")
    }

    /// C: `IN_ITCM_RAM void modNode_reassignVeloMod()`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/modulationNode.c:108`. Reference build: out-of-line in ITCM. Group P1.
    /// Globals: `activeParameters`, `parameterArray`, `velocityModulators`.
    /// Calls: `Lxr::modNode_updateValue`.
    pub fn modNode_reassignVeloMod(&mut self) {
        todo!("port: Core/Src/Soundengine/DSPAudio/modulationNode.c:108")
    }

    /// C: `void modNode_setDestination(ModulationNode* vm, uint16_t dest)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/modulationNode.c:120`. Reference build: out-of-line in flash. Group P1.
    /// Globals: `parameterArray`.
    /// Calls: `Lxr::modNode_resetTargets`.
    ///
    /// `vm` is a place accessor (see `crate::place`): the C pointer may point into `Lxr` and this
    /// function can also reach the pointee through globals, so a plain `&mut` would not be faithful.
    pub fn modNode_setDestination(&mut self, vm: impl Fn(&mut Lxr) -> &mut ModulationNode + Copy, dest: u16) {
        todo!("port: Core/Src/Soundengine/DSPAudio/modulationNode.c:120")
    }

    /// C: `void modNode_updateValue(ModulationNode* vm, float val)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/modulationNode.c:164`. Reference build: out-of-line in flash. Group P1.
    /// Globals: `activeParameters`, `parameterArray`.
    /// Calls: `Lxr::midiParser_ccHandler`.
    ///
    /// `vm` is a place accessor (see `crate::place`): the C pointer may point into `Lxr` and this
    /// function can also reach the pointee through globals, so a plain `&mut` would not be faithful.
    pub fn modNode_updateValue(&mut self, vm: impl Fn(&mut Lxr) -> &mut ModulationNode + Copy, val: f32) {
        todo!("port: Core/Src/Soundengine/DSPAudio/modulationNode.c:164")
    }
}
