//! Port of `Core/Inc/Soundengine/Sequencer/EuklidGenerator.h`, `Core/Src/Soundengine/Sequencer/EuklidGenerator.c`.
//!
//! Porting group: P3. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;

// ---- functions without access to globals
/// C: `static inline uint8_t getObjSize(uint16_t group)`
///
/// Source: `Core/Src/Soundengine/Sequencer/EuklidGenerator.c:53`. Reference build: inlined into its callers only. Group P3.
pub fn getObjSize(group: u16) -> u8 {
    todo!("port: Core/Src/Soundengine/Sequencer/EuklidGenerator.c:53")
}

// ---- functions that use globals (methods on `Lxr`)
impl Lxr {
    /// C: `void euklid_init()`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/EuklidGenerator.c:36`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `euklid_length`, `euklid_patternBuffer`, `euklid_rotation`, `euklid_steps`.
    pub fn euklid_init(&mut self) {
        todo!("port: Core/Src/Soundengine/Sequencer/EuklidGenerator.c:36")
    }

    /// C: `void euklid_calcRecursive(uint8_t length, uint8_t steps, uint8_t iteration, uint16_t group1, uint16_t group2)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/EuklidGenerator.c:60`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `euklid_nextCnt1`, `euklid_originalLength`, `euklid_patternBuffer`.
    /// Calls: `sequencer::euklid_generator::getObjSize`.
    pub fn euklid_calcRecursive(&mut self, length: u8, steps: u8, iteration: u8, group1: u16, group2: u16) {
        todo!("port: Core/Src/Soundengine/Sequencer/EuklidGenerator.c:60")
    }

    /// C: `void euklid_generate(uint8_t trackNr, uint8_t patternNr)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/EuklidGenerator.c:142`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `euklid_length`, `euklid_nextCnt1`, `euklid_originalLength`, `euklid_patternBuffer`, `euklid_rotation`, `euklid_steps`.
    /// Calls: `Lxr::euklid_calcRecursive`, `Lxr::euklid_rotatePattern`, `Lxr::euklid_transferPattern`.
    pub fn euklid_generate(&mut self, trackNr: u8, patternNr: u8) {
        todo!("port: Core/Src/Soundengine/Sequencer/EuklidGenerator.c:142")
    }

    /// C: `uint8_t euklid_getLength(uint8_t trackNr)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/EuklidGenerator.c:169`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `euklid_length`.
    pub fn euklid_getLength(&mut self, trackNr: u8) -> u8 {
        todo!("port: Core/Src/Soundengine/Sequencer/EuklidGenerator.c:169")
    }

    /// C: `uint8_t euklid_getSteps(uint8_t trackNr)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/EuklidGenerator.c:174`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `euklid_steps`.
    pub fn euklid_getSteps(&mut self, trackNr: u8) -> u8 {
        todo!("port: Core/Src/Soundengine/Sequencer/EuklidGenerator.c:174")
    }

    /// C: `void euklid_setLength(uint8_t trackNr, uint8_t value)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/EuklidGenerator.c:180`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `euklid_length`.
    pub fn euklid_setLength(&mut self, trackNr: u8, value: u8) {
        todo!("port: Core/Src/Soundengine/Sequencer/EuklidGenerator.c:180")
    }

    /// C: `void euklid_setSteps(uint8_t trackNr, uint8_t value, uint8_t patternNr)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/EuklidGenerator.c:187`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `euklid_length`, `euklid_steps`.
    /// Calls: `Lxr::euklid_generate`.
    pub fn euklid_setSteps(&mut self, trackNr: u8, value: u8, patternNr: u8) {
        todo!("port: Core/Src/Soundengine/Sequencer/EuklidGenerator.c:187")
    }

    /// C: `uint8_t euklid_getRotation(uint8_t trackNr)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/EuklidGenerator.c:197`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `euklid_rotation`.
    pub fn euklid_getRotation(&mut self, trackNr: u8) -> u8 {
        todo!("port: Core/Src/Soundengine/Sequencer/EuklidGenerator.c:197")
    }

    /// C: `void euklid_setRotation(uint8_t trackNr, uint8_t value, uint8_t patternNr)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/EuklidGenerator.c:202`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `euklid_length`, `euklid_rotation`.
    /// Calls: `Lxr::euklid_generate`.
    pub fn euklid_setRotation(&mut self, trackNr: u8, value: u8, patternNr: u8) {
        todo!("port: Core/Src/Soundengine/Sequencer/EuklidGenerator.c:202")
    }

    /// C: `void euklid_rotatePattern(uint8_t length, uint8_t amount)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/EuklidGenerator.c:210`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `euklid_patternBuffer`.
    pub fn euklid_rotatePattern(&mut self, length: u8, amount: u8) {
        todo!("port: Core/Src/Soundengine/Sequencer/EuklidGenerator.c:210")
    }

    /// C: `void euklid_transferPattern(uint8_t trackNr, uint8_t patternNr)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/EuklidGenerator.c:221`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `euklid_length`, `euklid_patternBuffer`, `seq_patternSet`.
    pub fn euklid_transferPattern(&mut self, trackNr: u8, patternNr: u8) {
        todo!("port: Core/Src/Soundengine/Sequencer/EuklidGenerator.c:221")
    }
}
