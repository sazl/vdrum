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
    // Not linked out of line in the reference build (inlined into sync_tick, see there for the compiled
    // form). C semantics: quarterDuration = 24 * (t/4); bpm = 60000 / quarterDuration.
    let quarterDuration = 24.0 * (timePerPulse / 4.0);
    let bpm = (60.0 * 1000.0) / quarterDuration;
    if bpm > 0.0 { bpm as u32 as u16 } else { 1 }
}

// ---- functions that use globals (methods on `Lxr`)
impl Lxr {
    /// C: `void sync_tick()`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/clockSync.c:42`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `isStepShuffled`, `seq_prescaleCounter`, `sync_clockCnt`, `sync_delayedStepTimer`, `sync_timeLast`, `sync_window`, `sync_writePos`, `systick_ticks`.
    /// Calls: `Lxr::seq_MidiSyncQueue`, `Lxr::seq_getBpm`, `Lxr::seq_getDeltaTShuffleDelay`, `Lxr::seq_getMasterStepCnt`, `Lxr::seq_nextStep`, `Lxr::seq_setBpm`, `sequencer::clock_sync::sync_calcBpm`.
    pub fn sync_tick(&mut self) {
        // with the new sequencer without substeps we have classical 4ppq resolution, MIDI clock is 24ppq:
        // advance the sequencer every 6 clock pulses
        if self.sync_clockCnt == 0 {
            let isStepShuffled = self.seq_getMasterStepCnt() % 2 == 1;
            if isStepShuffled {
                let bpm = self.seq_getBpm();
                let dt = self.seq_getDeltaTShuffleDelay(bpm);
                self.sync_delayedStepTimer = (self.systick_ticks as f32 + dt) as u32;
            } else {
                self.seq_nextStep();
                self.seq_prescaleCounter = 0;
                self.seq_MidiSyncQueue(0);
            }
        }

        self.sync_clockCnt = self.sync_clockCnt.wrapping_add(1);
        if self.sync_clockCnt >= 6 {
            self.sync_clockCnt = 0;
        }

        // Fix for external sync LFO sync calculation
        let time = self.systick_ticks as f32 - self.sync_timeLast;
        self.sync_timeLast = self.systick_ticks as f32;

        self.sync_window[self.sync_writePos as usize] = time;
        self.sync_writePos = self.sync_writePos.wrapping_add(1) & WINDOW_MASK as u8;

        // Machine code (fast-math): the window is summed in index order, avg = sum * 0.125, and
        // sync_calcBpm(avg) is folded to `avg > 0 ? (uint16_t)(uint32_t)(80000.f / sum) : 1`.
        let w = self.sync_window;
        let sum = ((((((w[1] + w[0]) + w[2]) + w[3]) + w[4]) + w[5]) + w[6]) + w[7];
        let avg = sum * 0.125;
        let bpm: u16 = if avg > 0.0 { (80000.0f32 / sum) as u32 as u16 } else { 1 };
        self.seq_setBpm(bpm);
    }

    /// C: `void sync_midiResetPosition()`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/clockSync.c:129`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `sync_clockCnt`.
    /// Calls: `Lxr::seq_resetPosition`.
    pub fn sync_midiResetPosition(&mut self) {
        self.sync_clockCnt = 0;
        self.seq_resetPosition();
    }

    /// C: `void sync_midiStartStop(uint8_t isStart)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/clockSync.c:135`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `sync_timeLast`, `systick_ticks`.
    /// Calls: `Lxr::seq_setRunning`.
    pub fn sync_midiStartStop(&mut self, isStart: u8) {
        if isStart != 0 {
            // this message does NOT mean start immediately! it just 'arms' the seq to start at the next MIDI_CLOCK received
            self.seq_setRunning(1);
            self.sync_timeLast = self.systick_ticks as f32;
        } else {
            self.seq_setRunning(0);
        }
    }

    /// C: `uint8_t sync_getClockCnt()`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/clockSync.c:147`. Reference build: not linked (removed by --gc-sections). Group P3.
    /// Globals: `sync_clockCnt`.
    pub fn sync_getClockCnt(&mut self) -> u8 {
        self.sync_clockCnt
    }
}
