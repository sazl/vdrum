//! Port of `Core/Inc/Soundengine/Sequencer/clockSync.h`, `Core/Src/Soundengine/Sequencer/clockSync.c`.
//!
//! Porting group: P3. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;

// ---- #define constants
/// `#define WINDOW_SIZE 8` (Core/Inc/Soundengine/Sequencer/clockSync.h:26)
pub const WINDOW_SIZE: i32 = 8;
/// `#define WINDOW_MASK 0x07` (Core/Inc/Soundengine/Sequencer/clockSync.h:27)
pub const WINDOW_MASK: i32 = 7;

// ---- functions without access to globals
/// C: `uint16_t sync_calcBpm(float timePerPulse)`
///
/// Source: `Core/Src/Soundengine/Sequencer/clockSync.c:29`. Reference build: inlined into its callers only. Group P3.
pub fn sync_calcBpm(timePerPulse: f32) -> u16 {
    todo!("port: Core/Src/Soundengine/Sequencer/clockSync.c:29")
}

// ---- functions that use globals (methods on `Lxr`)
impl Lxr {
    /// C: `void sync_tick()`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/clockSync.c:42`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `isStepShuffled`, `seq_prescaleCounter`, `sync_clockCnt`, `sync_delayedStepTimer`, `sync_timeLast`, `sync_window`, `sync_writePos`, `systick_ticks`.
    /// Calls: `Lxr::seq_MidiSyncQueue`, `Lxr::seq_getBpm`, `Lxr::seq_getDeltaTShuffleDelay`, `Lxr::seq_getMasterStepCnt`, `Lxr::seq_nextStep`, `Lxr::seq_setBpm`, `sequencer::clock_sync::sync_calcBpm`.
    pub fn sync_tick(&mut self) {
        todo!("port: Core/Src/Soundengine/Sequencer/clockSync.c:42")
    }

    /// C: `void sync_midiResetPosition()`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/clockSync.c:129`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `sync_clockCnt`.
    /// Calls: `Lxr::seq_resetPosition`.
    pub fn sync_midiResetPosition(&mut self) {
        todo!("port: Core/Src/Soundengine/Sequencer/clockSync.c:129")
    }

    /// C: `void sync_midiStartStop(uint8_t isStart)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/clockSync.c:135`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `sync_timeLast`, `systick_ticks`.
    /// Calls: `Lxr::seq_setRunning`.
    pub fn sync_midiStartStop(&mut self, isStart: u8) {
        todo!("port: Core/Src/Soundengine/Sequencer/clockSync.c:135")
    }

    /// C: `uint8_t sync_getClockCnt()`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/clockSync.c:147`. Reference build: not linked (removed by --gc-sections). Group P3.
    /// Globals: `sync_clockCnt`.
    pub fn sync_getClockCnt(&mut self) -> u8 {
        todo!("port: Core/Src/Soundengine/Sequencer/clockSync.c:147")
    }
}
