//! Sequencer access for the VDrum plugin: writing pattern data and driving the sequencer from a
//! host transport. Not part of the firmware port.
//!
//! The firmware edits patterns through its front panel; the plugin keeps its own copy of the
//! pattern data and writes it here. Writes to the pattern that plays also go to
//! `patternBackupBuffer`, because `seq_setPendingPatternActive` copies that buffer back over the
//! playing pattern when the pattern changes (the firmware's "discard unsaved changes").

use crate::midi::midi_parser::RECIP_127;
use crate::prelude::*;
use crate::sequencer::NUM_BARS;

/// The firmware's `NO_AUTOMATION` step parameter number.
const NO_AUTOMATION: u8 = 255;

/// One step as the plugin writes it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VdrumStep {
    pub on: bool,
    /// 0..=127.
    pub velocity: u8,
    /// 0..=100 (compared with a random value 1..=100 per step).
    pub prob: u8,
    pub note: u8,
    /// `StepTiming.flam`, 0..=5.
    pub flam: u8,
    /// `StepTiming.shift`, 0..=14 (7 = none).
    pub shift: u8,
    /// Automation slots: (parameter number or 255, value 0..=127).
    pub locks: [(u8, u8); 2],
}

impl Lxr {
    fn vdrum_seq_sync_backup(&mut self, pat: u8) {
        if pat == self.seq_activePattern {
            self.seq_copyPatternToBackupBuffer(pat);
        }
    }

    /// Writes one step (`pat` < 64, `track` < 7, `step` < 64; others are ignored). With `sync`, the
    /// playing pattern's backup copy is updated too (see the module notes); a bulk load can pass
    /// false and call [`Lxr::vdrum_seq_loaded`] at the end.
    pub fn vdrum_seq_set_step(&mut self, pat: u8, track: u8, step: u8, s: VdrumStep, sync: bool) {
        if pat >= 64 || track >= 7 || step >= 64 {
            return;
        }
        let mut st = self.seq_step_read(pat, track, step as i32);
        st.volume = (s.velocity & 0x7f) | if s.on { STEP_ACTIVE_MASK as u8 } else { 0 };
        st.prob = s.prob.min(100);
        st.note = s.note & 0x7f;
        let lock = |l: (u8, u8)| if l.0 == NO_AUTOMATION { (NO_AUTOMATION, 0) } else { (l.0, l.1 & 0x7f) };
        (st.param1Nr, st.param1Val) = lock(s.locks[0]);
        (st.param2Nr, st.param2Val) = lock(s.locks[1]);
        self.seq_step_write(pat, track, step as i32, st);
        let mut t = StepTiming::ZERO;
        t.set_flam(s.flam.min(5));
        t.set_shift(s.shift.min(14));
        self.seq_stepTiming_write(pat, track, step as i32, t);
        self.seq_setMainStep(pat, track, step, s.on as u8);
        if sync {
            self.vdrum_seq_sync_backup(pat);
        }
    }

    /// Sets only a step's probability (density changes).
    pub fn vdrum_seq_set_prob(&mut self, pat: u8, track: u8, step: u8, prob: u8, sync: bool) {
        if pat >= 64 || track >= 7 || step >= 64 {
            return;
        }
        let mut st = self.seq_step_read(pat, track, step as i32);
        st.prob = prob.min(100);
        self.seq_step_write(pat, track, step as i32, st);
        if sync {
            self.vdrum_seq_sync_backup(pat);
        }
    }

    /// Sets the on/off state of all 64 steps of a track (bit `n` = step `n`).
    pub fn vdrum_seq_set_bits(&mut self, pat: u8, track: u8, bits: u64) {
        if pat >= 64 || track >= 7 {
            return;
        }
        for bar in 0..NUM_BARS as usize {
            self.seq_patternSet.seq_mainSteps[bar][pat as usize][track as usize] = (bits >> (16 * bar)) as u16;
        }
        for step in 0..64u8 {
            let mut st = self.seq_step_read(pat, track, step as i32);
            if bits >> step & 1 != 0 {
                st.volume |= STEP_ACTIVE_MASK as u8;
            } else {
                st.volume &= STEP_VOLUME_MASK as u8;
            }
            self.seq_step_write(pat, track, step as i32, st);
        }
        self.vdrum_seq_sync_backup(pat);
    }

    /// Track length 1..=64.
    pub fn vdrum_seq_set_length(&mut self, pat: u8, track: u8, length: u8, sync: bool) {
        if pat >= 64 || track >= 7 {
            return;
        }
        self.seq_patternSet.length[pat as usize][track as usize] = length.clamp(1, 64);
        if sync {
            self.vdrum_seq_sync_backup(pat);
        }
    }

    /// Pattern shuffle 0..=127 (`PAR_SHUFFLE`); applied at once to the playing pattern.
    pub fn vdrum_seq_set_shuffle(&mut self, pat: u8, value: u8, sync: bool) {
        if pat >= 64 {
            return;
        }
        let v = value.min(127);
        self.seq_patternSet.shuffle[pat as usize] = v;
        if pat == self.seq_activePattern {
            self.activeParameters[PAR_SHUFFLE as usize] = v;
            self.seq_setShuffle(v as i32 as f32 * RECIP_127);
        }
        if sync {
            self.vdrum_seq_sync_backup(pat);
        }
    }

    /// Ends a bulk load (`sync` = false writes): refreshes the playing pattern's backup copy and
    /// shuffle.
    pub fn vdrum_seq_loaded(&mut self) {
        let ap = self.seq_activePattern;
        let sh = self.seq_patternSet.shuffle[ap as usize];
        self.vdrum_seq_set_shuffle(ap, sh, true);
    }

    /// Starts the sequencer on the host transport: steps then advance only on
    /// [`Lxr::vdrum_seq_host_pulse`] (the firmware's external clock input path, `seq_isSyncExternal`
    /// = 1). `next` is the position of the next pulse in 16th notes from the song start; every
    /// track is placed so that this pulse plays its step `next % length`.
    pub fn vdrum_seq_host_start(&mut self, next: u32) {
        self.seq_setExtSync(1);
        self.sync_delayedStepTimer = 0;
        self.seq_setRunning(1);
        self.vdrum_seq_align(next);
    }

    /// Re-places the tracks after a jump of the host position (see [`Lxr::vdrum_seq_host_start`]).
    pub fn vdrum_seq_align(&mut self, next: u32) {
        let ap = self.seq_activePattern as usize;
        for t in 0..7 {
            let len = match self.seq_patternSet.length[ap][t] {
                0 => 16,
                l => l as i64,
            };
            // `seq_nextStep` increments first; len - 1 wraps to 0
            self.seq_stepIndex[t] = (next as i64 - 1).rem_euclid(len) as i8;
        }
        self.trigger_clockReceived = 0;
        // no flam/ratchet processing (which plays the current step) until the next pulse
        self.seq_prescaleCounter = 24;
        // shuffle delays odd 16ths: `seq_tick` reads the count before `seq_nextStep` increments it
        self.seq_masterStepCnt = next as u8;
    }

    /// One 16th note of the host transport.
    pub fn vdrum_seq_host_pulse(&mut self) {
        self.trigger_clockReceived = self.trigger_clockReceived.wrapping_add(1);
    }

    /// Stops the sequencer.
    pub fn vdrum_seq_stop(&mut self) {
        self.seq_setRunning(0);
        self.trigger_clockReceived = 0;
        self.sync_delayedStepTimer = 0;
    }

    /// Runs the sequencer on its own clock again, as `PAR_BPM` says (0: external clock input,
    /// 1: MIDI clock, otherwise that tempo).
    pub fn vdrum_seq_internal_clock(&mut self) {
        match self.activeParameters[PAR_BPM as usize] {
            0 => self.seq_setExtSync(1),
            1 => self.seq_setExtSync(2),
            bpm => {
                self.seq_setBpm(bpm as u16);
                self.seq_setExtSync(0);
            }
        }
    }

    /// Starts (from step 1) or stops the sequencer on its own clock.
    pub fn vdrum_seq_run(&mut self, on: bool) {
        if on {
            self.seq_masterStepCnt = 0;
            self.seq_setRunning(1);
        } else {
            self.vdrum_seq_stop();
        }
    }
}
