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
            {
            let v = vm(self);
            v.lastVal = 0.0;
            v.amount = 0.0;
        }
        self.modNode_setDestination(vm, 0);
    }

    /// C: `static void modNode_setOriginalValueChanged(ModulationNode* vm, uint16_t idx)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/modulationNode.c:37`. Reference build: inlined into its callers only. Group P1.
    /// Globals: `parameterArray`.
    pub fn modNode_setOriginalValueChanged(&mut self, vm: &mut ModulationNode, idx: u16) {
            if vm.destination == idx {
            let p = self.parameterArray[idx as usize];
            match p.r#type as i32 {
                TYPE_UINT8 => vm.originalValue.itg = self.cptr_read_u8(p.ptr) as u32,
                TYPE_SPECIAL_F => {}
                TYPE_SPECIAL_P | TYPE_SPECIAL_FILTER_F | TYPE_FLT => vm.originalValue.set_flt(self.cptr_read_f32(p.ptr)),
                TYPE_UINT32 => vm.originalValue.itg = self.cptr_read_u32(p.ptr),
                _ => {}
            }
        }
    }

    /// C: `void modNode_originalValueChanged(uint16_t idx)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/modulationNode.c:74`. Reference build: out-of-line in flash. Group P1.
    /// Globals: `cymbalVoice`, `hatVoice`, `parameterArray`, `snareVoice`, `velocityModulators`, `voiceArray`.
    /// Calls: `Lxr::modNode_setOriginalValueChanged`.
    pub fn modNode_originalValueChanged(&mut self, idx: u16) {
            for i in 0..6 {
            let mut vm = self.velocityModulators[i];
            self.modNode_setOriginalValueChanged(&mut vm, idx);
            self.velocityModulators[i] = vm;
        }
        for v in 0..3 {
            let mut vm = self.voiceArray[v].lfo.modTarget;
            self.modNode_setOriginalValueChanged(&mut vm, idx);
            self.voiceArray[v].lfo.modTarget = vm;
        }
        let mut vm = self.snareVoice.lfo.modTarget;
        self.modNode_setOriginalValueChanged(&mut vm, idx);
        self.snareVoice.lfo.modTarget = vm;
        let mut vm = self.cymbalVoice.lfo.modTarget;
        self.modNode_setOriginalValueChanged(&mut vm, idx);
        self.cymbalVoice.lfo.modTarget = vm;
        let mut vm = self.hatVoice.lfo.modTarget;
        self.modNode_setOriginalValueChanged(&mut vm, idx);
        self.hatVoice.lfo.modTarget = vm;
    }

    /// C: `IN_ITCM_RAM void modNode_resetTargets()`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/modulationNode.c:90`. Reference build: out-of-line in ITCM. Group P1.
    /// Globals: `cymbalVoice`, `hatVoice`, `snareVoice`, `velocityModulators`, `voiceArray`.
    /// Calls: `Lxr::paramArray_setParameter`.
    pub fn modNode_resetTargets(&mut self) {
            // every node is read again after each call (paramArray_setParameter can change them)
        for i in 0..6 {
            let n = self.velocityModulators[i];
            self.paramArray_setParameter(n.destination, n.originalValue);
        }
        for v in 0..3 {
            let n = self.voiceArray[v].lfo.modTarget;
            self.paramArray_setParameter(n.destination, n.originalValue);
        }
        let n = self.snareVoice.lfo.modTarget;
        self.paramArray_setParameter(n.destination, n.originalValue);
        let n = self.cymbalVoice.lfo.modTarget;
        self.paramArray_setParameter(n.destination, n.originalValue);
        let n = self.hatVoice.lfo.modTarget;
        self.paramArray_setParameter(n.destination, n.originalValue);
    }

    /// C: `IN_ITCM_RAM void modNode_reassignVeloMod()`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/modulationNode.c:108`. Reference build: out-of-line in ITCM. Group P1.
    /// Globals: `activeParameters`, `parameterArray`, `velocityModulators`.
    /// Calls: `Lxr::modNode_updateValue`.
    pub fn modNode_reassignVeloMod(&mut self) {
            // inlined modNode_updateValue (the store of lastVal is dropped since it is unchanged)
        for i in 0..6 {
            let n = self.velocityModulators[i];
            if self.parameterArray[n.destination as usize].ptr == 0 {
                continue;
            }
            let newValue = modNode_calcNewValue(n.amount, n.lastVal, self.activeParameters[n.destination as usize]);
            self.midiParser_ccHandler(modNode_ccMsg(n.destination), 0, 1, newValue);
        }
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
            self.modNode_resetTargets();
        vm(self).destination = dest;
        let p = self.parameterArray[dest as usize];
        if p.ptr == 0 {
            return;
        }
        match p.r#type as i32 {
            TYPE_UINT8 => {
                let v = self.cptr_read_u8(p.ptr) as u32;
                vm(self).originalValue.itg = v;
            }
            TYPE_SPECIAL_F => vm(self).originalValue.set_flt(1.0),
            TYPE_SPECIAL_FILTER_F | TYPE_FLT => {
                let v = self.cptr_read_f32(p.ptr);
                vm(self).originalValue.set_flt(v);
            }
            TYPE_UINT32 => {
                let v = self.cptr_read_u32(p.ptr);
                vm(self).originalValue.itg = v;
            }
            _ => {}
        }
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
            vm(self).lastVal = val;
        let dest = vm(self).destination;
        if self.parameterArray[dest as usize].ptr == 0 {
            return;
        }
        let newValue = modNode_calcNewValue(vm(self).amount, val, self.activeParameters[dest as usize]);
        self.midiParser_ccHandler(modNode_ccMsg(dest), 0, 1, newValue);
    }
}

// ---- helpers reproducing the inlined machine code of modNode_updateValue / modNode_reassignVeloMod

/// `VMINNM.F32`: IEEE 754-2008 minNum (a NaN operand yields the other operand; -0 < +0).
#[inline(always)]
pub(crate) fn vminnm(a: f32, b: f32) -> f32 {
    if a.is_nan() {
        b
    } else if b.is_nan() {
        a
    } else if a == b {
        if a.is_sign_negative() { a } else { b }
    } else if a < b {
        a
    } else {
        b
    }
}

/// `VMAXNM.F32`: IEEE 754-2008 maxNum (a NaN operand yields the other operand; +0 > -0).
#[inline(always)]
pub(crate) fn vmaxnm(a: f32, b: f32) -> f32 {
    if a.is_nan() {
        b
    } else if b.is_nan() {
        a
    } else if a == b {
        if a.is_sign_negative() { b } else { a }
    } else if a > b {
        a
    } else {
        b
    }
}

/// `newValue` of `modNode_updateValue`: `activeParameters[dest]/127.f + amount*(val*2.f-1.f)`, clamped to
/// [0, 1]. Compiled as `vfma(amount*vfma(-1, val, 2), active, 1/127.f)`, then `vminnm(.., 1)`, `vmaxnm(.., 0)`
/// (a NaN gives 1). Not a C function (inlined code), public for the differential tests.
#[inline]
pub fn modNode_calcNewValue(amount: f32, val: f32, active: u8) -> f32 {
    let bip = vfma(-1.0, val, 2.0);
    let v = vfma(bip * amount, active as i32 as f32, f32::from_bits(0x3c010204));
    vmaxnm(vminnm(v, 1.0), 0.0)
}

/// The `MidiMsg` that `modNode_updateValue` passes to `midiParser_ccHandler` (only `status` and `data1` are
/// set in the firmware; the other bytes are uninitialised stack memory there and 0 here).
#[inline]
pub fn modNode_ccMsg(destination: u16) -> MidiMsg {
    if destination < 128 {
        MidiMsg { status: MIDI_CC as u8, data1: ((destination + 1) & 0x7f) as u8, ..MidiMsg::ZERO }
    } else {
        MidiMsg { status: CC_2 as u8, data1: (destination & 0x7f) as u8, ..MidiMsg::ZERO }
    }
}
