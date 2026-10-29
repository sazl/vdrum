//! Port of `Core/Inc/Soundengine/Sequencer/sequencer.h`, `Core/Src/Soundengine/Sequencer/sequencer.c`.
//!
//! Porting group: P3. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

pub mod clock_sync;
pub mod euklid_generator;

use crate::prelude::*;
use crate::cmem::CMem;
use crate::dsp::automation_node::autoNode_init;
use crate::memmap::ADDR_seq_patternSet;
use crate::midi::midi_parser::{MIDI_FILTER_NOTES, MIDI_FILTER_PROG_C, MIDI_FILTER_SYSTEM, RECIP_127};
use crate::preset::{PRESET_KIT_TIMER, PRESET_PAT_TIMER, PRESET_STOP_PAT_TIMER, SKIP_NEXT_KIT_AUTOLOAD};

// ---- #define constants
/// `#define NUM_TRACKS 7` (Core/Inc/Soundengine/Sequencer/sequencer.h:32)
pub const NUM_TRACKS: i32 = 7;
/// `#define NUM_PATTERN 16*4` (Core/Inc/Soundengine/Sequencer/sequencer.h:33)
pub const NUM_PATTERN: i32 = 64;
/// `#define NUM_STEPS 64` (Core/Inc/Soundengine/Sequencer/sequencer.h:34)
pub const NUM_STEPS: i32 = 64;
/// `#define NUM_BARS 4` (Core/Inc/Soundengine/Sequencer/sequencer.h:35)
pub const NUM_BARS: i32 = 4;
/// `#define MAX_PATTERN 64` (Core/Inc/Soundengine/Sequencer/sequencer.h:36)
pub const MAX_PATTERN: i32 = 64;
/// `#define MAX_SONG_LENGTH 64` (Core/Inc/Soundengine/Sequencer/sequencer.h:38)
pub const MAX_SONG_LENGTH: i32 = 64;
/// `#define SONG_END MAX_PATTERN` (Core/Inc/Soundengine/Sequencer/sequencer.h:39)
pub const SONG_END: i32 = 64;
/// `#define SONG_LOOP SONG_END+1` (Core/Inc/Soundengine/Sequencer/sequencer.h:40)
pub const SONG_LOOP: i32 = 65;
/// `#define SEQ_DEFAULT_NOTE 60` (Core/Inc/Soundengine/Sequencer/sequencer.h:42)
pub const SEQ_DEFAULT_NOTE: i32 = 60;
/// `#define STEP_ACTIVE_MASK 0x80` (Core/Inc/Soundengine/Sequencer/sequencer.h:44)
pub const STEP_ACTIVE_MASK: i32 = 128;
/// `#define STEP_VOLUME_MASK 0x7f` (Core/Inc/Soundengine/Sequencer/sequencer.h:45)
pub const STEP_VOLUME_MASK: i32 = 127;
/// `#define ROLL_VOLUME 100` (Core/Inc/Soundengine/Sequencer/sequencer.h:54)
pub const ROLL_VOLUME: i32 = 100;
/// `#define USE_PATTERN_EDIT_BUFFER 0` (Core/Inc/Soundengine/Sequencer/sequencer.h:137)
pub const USE_PATTERN_EDIT_BUFFER: i32 = 0;
/// `#define SEQ_PRESCALER_MASK 0x03` (Core/Src/Soundengine/Sequencer/sequencer.c:45)
pub const SEQ_PRESCALER_MASK: i32 = 3;
/// `#define MIDI_PRESCALER_MASK 0x04` (Core/Src/Soundengine/Sequencer/sequencer.c:46)
pub const MIDI_PRESCALER_MASK: i32 = 4;

// ---- enums
/// C: `enum Seq_QuantisationEnum` (Core/Inc/Soundengine/Sequencer/sequencer.h:56), 1 bytes, underlying type `u8`.
/// Representation: integer type alias plus one `pub const` per enumerator.
pub type Seq_QuantisationEnum = u8;
pub const NO_QUANTISATION: Seq_QuantisationEnum = 0;
pub const QUANT_8: Seq_QuantisationEnum = 1;
pub const QUANT_16: Seq_QuantisationEnum = 2;
pub const QUANT_32: Seq_QuantisationEnum = 3;
pub const QUANT_64: Seq_QuantisationEnum = 4;

// ---- types
/// C: `typedef union { uint16_t word[NUM_BARS]; uint64_t quadword; } ActiveSteps;` (Core/Inc/Soundengine/Sequencer/sequencer.h:81), 8 bytes.
///
/// Representation: the storage is `word`; `quadword` is accessed through `quadword()`/`set_quadword()`
/// (little endian, as on the Cortex-M7).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct ActiveSteps {
    /// `uint16_t word[NUM_BARS];`
    pub word: [u16; 4],
}
impl ActiveSteps {
    pub const ZERO: Self = Self { word: [0; 4] };
    /// `uint64_t quadword;` (read)
    #[inline(always)]
    pub fn quadword(&self) -> u64 {
        (self.word[0] as u64) | (self.word[1] as u64) << 16 | (self.word[2] as u64) << 32 | (self.word[3] as u64) << 48
    }
    /// `uint64_t quadword;` (write)
    #[inline(always)]
    pub fn set_quadword(&mut self, v: u64) {
        for i in 0..4 {
            self.word[i] = (v >> (16 * i)) as u16;
        }
    }
}

/// C: `struct PatternBackupBufferStruct` (Core/Inc/Soundengine/Sequencer/sequencer.h:108), 3650 bytes.
/// C typedef name `PatternBackupBuffer`; the struct tag `PatternBackupBufferStruct` is a type alias below.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PatternBackupBuffer {
    /// `Step seq_subStepPattern[NUM_TRACKS][NUM_STEPS]; //contains step data`
    pub seq_subStepPattern: [[Step; 64]; 7],
    /// `uint8_t seq_stepTiming[NUM_TRACKS][NUM_STEPS];`
    pub seq_stepTiming: [[u8; 64]; 7],
    /// `uint16_t seq_mainSteps[NUM_BARS][NUM_TRACKS]; //bits represent if step is active`
    pub seq_mainSteps: [[u16; 7]; 4],
    /// `uint8_t length[NUM_TRACKS]; //only used in euklid generator?`
    pub length: [u8; 7],
    /// `uint8_t shuffle;`
    pub shuffle: u8,
    /// `uint8_t kitNr;`
    pub kitNr: u8,
}
/// C struct tag of `PatternBackupBuffer`.
pub type PatternBackupBufferStruct = PatternBackupBuffer;
impl PatternBackupBuffer {
    /// All-zero value (C static zero initialisation).
    pub const ZERO: Self = Self { seq_subStepPattern: [[Step::ZERO; 64]; 7], seq_stepTiming: [[0; 64]; 7], seq_mainSteps: [[0; 7]; 4], length: [0; 7], shuffle: 0, kitNr: 0 };
}
impl Default for PatternBackupBuffer {
    fn default() -> Self {
        Self::ZERO
    }
}

/// C: `struct PatternSetStruct` (Core/Inc/Soundengine/Sequencer/sequencer.h:97), 233536 bytes.
/// C typedef name `PatternSet`; the struct tag `PatternSetStruct` is a type alias below.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PatternSet {
    /// `Step seq_subStepPattern[NUM_PATTERN][NUM_TRACKS][NUM_STEPS]; //contains step data`
    pub seq_subStepPattern: [[[Step; 64]; 7]; 64],
    /// `StepTiming seq_stepTiming[NUM_PATTERN][NUM_TRACKS][NUM_STEPS];`
    pub seq_stepTiming: [[[StepTiming; 64]; 7]; 64],
    /// `uint16_t seq_mainSteps[NUM_BARS][NUM_PATTERN][NUM_TRACKS]; //bits represent if step is active`
    pub seq_mainSteps: [[[u16; 7]; 64]; 4],
    /// `uint8_t length[NUM_PATTERN][NUM_TRACKS]; //only used in euklid generator?`
    pub length: [[u8; 7]; 64],
    /// `uint8_t shuffle[NUM_PATTERN];`
    pub shuffle: [u8; 64],
    /// `uint8_t kitNr[NUM_PATTERN];`
    pub kitNr: [u8; 64],
}
/// C struct tag of `PatternSet`.
pub type PatternSetStruct = PatternSet;
impl PatternSet {
    /// All-zero value (C static zero initialisation).
    pub const ZERO: Self = Self { seq_subStepPattern: [[[Step::ZERO; 64]; 7]; 64], seq_stepTiming: [[[StepTiming::ZERO; 64]; 7]; 64], seq_mainSteps: [[[0; 7]; 64]; 4], length: [[0; 7]; 64], shuffle: [0; 64], kitNr: [0; 64] };
}
impl Default for PatternSet {
    fn default() -> Self {
        Self::ZERO
    }
}

/// C: `struct {...}` (Core/Src/Soundengine/Sequencer/sequencer.c:99), 3 bytes.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct seq_rndValue_t {
    /// `uint8_t prev;`
    pub prev: u8,
    /// `uint8_t current;`
    pub current: u8,
    /// `uint8_t next;`
    pub next: u8,
}
impl seq_rndValue_t {
    /// All-zero value (C static zero initialisation).
    pub const ZERO: Self = Self { prev: 0, current: 0, next: 0 };
}
impl Default for seq_rndValue_t {
    fn default() -> Self {
        Self::ZERO
    }
}

// ---- functions without access to globals
/// C: `int8_t getShiftValue(uint8_t in)`
///
/// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:845`. Reference build: not linked (removed by --gc-sections). Group P3.
pub fn getShiftValue(r#in: u8) -> i8 {
    match r#in {
        0 => -7,
        1 => -6,
        2 => -5,
        3 => -4,
        4 => -3,
        5 => -2,
        6 => -1,
        8 => 1,
        9 => 2,
        10 => 3,
        11 => 4,
        12 => 5,
        13 => 6,
        14 => 7,
        _ => 0, // default / 7
    }
}

/// C: `uint32_t flamIdxToPattern(uint8_t flamIdx)`
///
/// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:911`. Reference build: inlined into its callers only. Group P3.
pub fn flamIdxToPattern(flamIdx: u8) -> u32 {
    match flamIdx {
        1 => 0b100000000000100000000000, // x2
        2 => 0b100000001000000010000000, // x3
        3 => 0b100000100000100000100000, // x4
        4 => 0b100010001000100010001000, // x6
        5 => 0b100100100100100100100100, // x8
        _ => 0b100000000000000000000000, // off / default
    }
}

/// C: `static void seq_resetNote(Step *step)`
///
/// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:2526`. Reference build: inlined into its callers only. Group P3.
pub fn seq_resetNote(step: &mut Step) {
    step.note = SEQ_DEFAULT_NOTE as u8;
    step.param1Nr = NO_AUTOMATION as u8;
    step.param1Val = 0;
    step.param2Nr = NO_AUTOMATION as u8;
    step.param2Val = 0;
    step.prob = 100;
    step.volume = 127; // clears active bit as well
}

// ---- functions that use globals (methods on `Lxr`)
impl Lxr {
    /// C: `uint8_t seq_getMasterStepCnt()`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:176`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_masterStepCnt`.
    pub fn seq_getMasterStepCnt(&mut self) -> u8 {
        self.seq_masterStepCnt
    }

    /// C: `void seq_init()`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:181`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_activeSong`, `seq_automationNodes`, `seq_lastMasterStep`, `seq_patternSet`, `seq_songData`, `seq_stepIndex`, `seq_stepToSkip`.
    /// Calls: `dsp::automation_node::autoNode_init`, `Lxr::seq_clearPattern`.
    pub fn seq_init(&mut self) {
        // give active song pointer a proper value
        self.seq_activeSong = Some(0);

        for row in self.seq_songData.iter_mut() {
            *row = [SONG_END as u8; 64];
        }
        self.seq_songData[0][0] = 0;

        for i in 0..NUM_TRACKS as usize {
            autoNode_init(&mut self.seq_automationNodes[i][0]);
            autoNode_init(&mut self.seq_automationNodes[i][1]);
            self.seq_stepToSkip[i] = -1;
        }

        self.seq_stepIndex = [0; 7];
        self.seq_lastMasterStep = [0; 7];

        for i in 0..NUM_PATTERN as u8 {
            self.seq_clearPattern(i); // will clear all tracks in the pattern
        }
    }

    /// C: `void seq_copyPatternToBackupBuffer(uint8_t patternNr)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:225`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `patternBackupBuffer`, `seq_patternSet`.
    pub fn seq_copyPatternToBackupBuffer(&mut self, patternNr: u8) {
        let p = patternNr as usize;
        let ps = &self.seq_patternSet;
        let bb = &mut self.patternBackupBuffer;
        bb.seq_subStepPattern = ps.seq_subStepPattern[p];
        for t in 0..7 {
            for s in 0..64 {
                bb.seq_stepTiming[t][s] = ps.seq_stepTiming[p][t][s].bits;
            }
        }
        for i in 0..NUM_BARS as usize {
            bb.seq_mainSteps[i] = ps.seq_mainSteps[i][p];
        }
        bb.length = ps.length[p];
        bb.shuffle = ps.shuffle[p];
        bb.kitNr = ps.kitNr[p];
    }

    /// C: `void seq_copyBackupBufferToPattern(uint8_t patternNr)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:242`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `patternBackupBuffer`, `seq_patternSet`.
    pub fn seq_copyBackupBufferToPattern(&mut self, patternNr: u8) {
        let p = patternNr as usize;
        let bb = &self.patternBackupBuffer;
        let ps = &mut self.seq_patternSet;
        ps.seq_subStepPattern[p] = bb.seq_subStepPattern;
        for t in 0..7 {
            for s in 0..64 {
                ps.seq_stepTiming[p][t][s].bits = bb.seq_stepTiming[t][s];
            }
        }
        for i in 0..NUM_BARS as usize {
            ps.seq_mainSteps[i][p] = bb.seq_mainSteps[i];
        }
        ps.length[p] = bb.length;
        ps.shuffle[p] = bb.shuffle;
        ps.kitNr[p] = bb.kitNr;
    }

    /// C: `void seq_setShuffle(float shuffle)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:269`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_shuffle`.
    pub fn seq_setShuffle(&mut self, shuffle: f32) {
        self.seq_shuffle = shuffle;
    }

    /// C: `void seq_setTrackLength(uint8_t trackNr, uint8_t length)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:274`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `frontParser_shownPattern`, `seq_patternSet`.
    pub fn seq_setTrackLength(&mut self, trackNr: u8, length: u8) {
        let sp = self.frontParser_shownPattern as usize;
        self.seq_patternSet.length[sp][trackNr as usize] = length;
    }

    /// C: `uint8_t seq_getTrackLength(uint8_t trackNr)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:293`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `frontParser_shownPattern`, `seq_patternSet`.
    pub fn seq_getTrackLength(&mut self, trackNr: u8) -> u8 {
        self.seq_patternSet.length[self.frontParser_shownPattern as usize][trackNr as usize]
    }

    /// C: `float seq_getDeltaTShuffleDelay(uint16_t bpm)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:354`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_shuffle`.
    pub fn seq_getDeltaTShuffleDelay(&mut self, bpm: u16) -> f32 {
        // Compiled (fast-math) form: (float)(int)(60000/bpm) * (seq_shuffle * (4/96*16 = 0.6666667f)).
        if bpm == 0 {
            return 1.0;
        }
        let dt = (60000u32 / bpm as u32) as i32 as f32;
        dt * (self.seq_shuffle * f32::from_bits(0x3f2aaaab))
    }

    /// C: `static void seq_calcDeltaT(uint16_t bpm, uint8_t isShuffled)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:386`. Reference build: inlined into its callers only. Group P3.
    /// Globals: `isStepShuffled`, `seq_deltaT`, `seq_lastShuffle`, `seq_shuffle`.
    pub fn seq_calcDeltaT(&mut self, bpm: u16, isShuffled: u8) {
        // Compiled (fast-math) form, as inlined into seq_tick / seq_MidiSyncQueue:
        //   seq_deltaT = (float)(int)(60000/bpm) * (1/24.f)                (udiv: bpm 0 gives 0)
        //   if (seq_shuffle != 0) {
        //     f = (float)(int)isShuffled * seq_shuffle; old = seq_lastShuffle; seq_lastShuffle = f;
        //     seq_deltaT *= fma(fms(1, old, 16), ...) i.e. (1 - old*16 [fused]) + f*16 [fused]
        //   }
        // For isShuffled == 0 the compiler folds f to 0 and the factor to the single fused `1 - old*16`.
        let q = if bpm == 0 { 0 } else { 60000u32 / bpm as u32 };
        let dt = (q as i32 as f32) * f32::from_bits(0x3d2aaaab);
        self.seq_deltaT = dt;
        if self.seq_shuffle != 0.0 {
            let old = self.seq_lastShuffle;
            let k = (-old).mul_add(16.0, 1.0);
            if isShuffled == 0 {
                self.seq_lastShuffle = 0.0;
                self.seq_deltaT = k * dt;
            } else {
                let f = (isShuffled as i32 as f32) * self.seq_shuffle;
                self.seq_lastShuffle = f;
                self.seq_deltaT = dt * f.mul_add(16.0, k);
            }
        }
    }

    /// C: `void seq_setBpm(uint16_t bpm)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:437`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_tempo`.
    /// Calls: `Lxr::lfo_recalcSync`.
    pub fn seq_setBpm(&mut self, bpm: u16) {
        self.seq_tempo = bpm;
        self.lfo_recalcSync();
    }

    /// C: `uint16_t seq_getBpm()`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:444`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_tempo`.
    pub fn seq_getBpm(&mut self) -> u16 {
        self.seq_tempo
    }

    /// C: `void seq_sync()`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:449`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_isSyncExternal`.
    /// Calls: `Lxr::seq_getExtSync`, `Lxr::sync_tick`.
    pub fn seq_sync(&mut self) {
        if self.seq_getExtSync() == 2 {
            self.sync_tick();
        }
    }

    /// C: `void seq_setNextPattern(const uint8_t patNr)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:457`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_loadPendigFlag`, `seq_pendingPattern`, `seq_running`.
    /// Calls: `Lxr::seq_setPendingPatternActive`.
    pub fn seq_setNextPattern(&mut self, patNr: u8) {
        if patNr as i32 >= MAX_PATTERN {
            return;
        }
        self.seq_pendingPattern = patNr;
        self.seq_loadPendigFlag = 1;

        if self.seq_running == 0 {
            // immediately change pattern
            self.seq_setPendingPatternActive();
        }
    }

    /// C: `static void seq_sendMidi(MidiMsg msg)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:470`. Reference build: inlined into its callers only. Group P3.
    /// Calls: `Lxr::uart_sendMidi`, `Lxr::usb_sendMidi`.
    pub fn seq_sendMidi(&mut self, msg: MidiMsg) {
        // send to usb midi
        self.usb_sendMidi(msg);
        // send to hardware midi out jack
        self.uart_sendMidi(msg);
    }

    /// C: `static void seq_parseAutomationNodes(uint8_t track, Step* stepData)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:482`. Reference build: inlined into its callers only. Group P3.
    /// Globals: `seq_automationNodes`.
    /// Calls: `Lxr::autoNode_setDestination`, `Lxr::autoNode_updateValue`.
    pub fn seq_parseAutomationNodes(&mut self, track: u8, stepData: &mut Step) {
        let t = track as usize;
        let mut n0 = self.seq_automationNodes[t][0];
        self.autoNode_setDestination(&mut n0, stepData.param1Nr as u16);
        self.seq_automationNodes[t][0] = n0;
        let mut n1 = self.seq_automationNodes[t][1];
        self.autoNode_setDestination(&mut n1, stepData.param2Nr as u16);
        self.seq_automationNodes[t][1] = n1;
        let mut n0 = self.seq_automationNodes[t][0];
        self.autoNode_updateValue(&mut n0, stepData.param1Val);
        self.seq_automationNodes[t][0] = n0;
        let mut n1 = self.seq_automationNodes[t][1];
        self.autoNode_updateValue(&mut n1, stepData.param2Val);
        self.seq_automationNodes[t][1] = n1;
    }

    /// C: `void seq_triggerVoice(uint8_t voiceNr, uint8_t vol, uint8_t note)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:492`. Reference build: out-of-line as a GCC clone (.part/.isra/.constprop). Group P3.
    /// Globals: `activeParameters`, `midi_MidiChannels`, `seq_activePattern`, `seq_patternSet`, `seq_stepIndex`.
    /// Calls: `Lxr::seq_parseAutomationNodes`, `Lxr::seq_sendMidiNoteOn`, `Lxr::voiceControl_noteOff`, `Lxr::voiceControl_noteOn`.
    pub fn seq_triggerVoice(&mut self, voiceNr: u8, vol: u8, note: u8) {
        if voiceNr > 6 {
            return;
        }
        let v = voiceNr as usize;
        let pat = self.seq_activePattern;
        let mut step = self.seq_step_read(pat, voiceNr, self.seq_stepIndex[v] as i32);
        self.seq_parseAutomationNodes(voiceNr, &mut step);

        // if a note is on for that channel send note-off first
        self.voiceControl_noteOff(voiceNr);

        // Trigger internal synth voice
        self.voiceControl_noteOn(voiceNr, note, vol);

        let midiChan = self.midi_MidiChannels[v];

        // the note that is played will be whatever is received unless we have a note override set
        let ov = self.activeParameters[PAR_MIDI_NOTE1 as usize + v];
        let midiNote = if ov == 0 { note } else { ov };

        // send the new note to midi/usb out
        let pat = self.seq_activePattern;
        let vel = self.seq_step_read(pat, voiceNr, self.seq_stepIndex[v] as i32).volume & STEP_VOLUME_MASK as u8;
        self.seq_sendMidiNoteOn(midiChan, midiNote, vel);
    }

    /// C: `static uint8_t seq_determineNextPattern(uint8_t autoIncrement)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:565`. Reference build: inlined into its callers only. Group P3.
    /// Globals: `seq_activePattern`, `seq_activeSong`, `seq_songModeEnabled`, `seq_songPositionPointer`.
    pub fn seq_determineNextPattern(&mut self, autoIncrement: u8) -> u8 {
        if self.seq_songModeEnabled != 0 {
            let song = self.seq_activeSong.expect("seq_activeSong is NULL");
            if autoIncrement != 0 {
                self.seq_songPositionPointer = self.seq_songPositionPointer.wrapping_add(1);
                if self.seq_songPositionPointer as i32 == MAX_SONG_LENGTH {
                    self.seq_songPositionPointer = 0;
                }
                self.seq_songData[song][self.seq_songPositionPointer as usize]
            } else {
                let mut pos = self.seq_songPositionPointer.wrapping_add(1);
                if pos as i32 == MAX_SONG_LENGTH {
                    pos = 0;
                }
                self.seq_songData[song][pos as usize]
            }
        } else {
            self.seq_activePattern
        }
    }

    /// C: `IN_ITCM_RAM void seq_setPendingPatternActive()`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:602`. Reference build: out-of-line in ITCM. Group P3.
    /// Globals: `activeParameters`, `frontParser_shownPattern`, `fxBus`, `menu_activePage`, `menu_loadedPresetNr`, `menu_playedPattern`, `menu_selectedPresetNr`, `menu_shownPattern`, `menu_songMenuPosition`, `midiParser_txRxFilter`, `midi_MidiChannels`, `midi_process_block`, `seq_activePattern`, `seq_lastMasterStep`, `seq_loadPendigFlag`, `seq_patternSet`, `seq_pendingPattern`, `seq_shuffle`, `seq_songPositionPointer`, `seq_stepIndex`.
    /// Calls: `Lxr::buttonHandler_getMode`, `Lxr::led_clearAllBlinkLeds`, `Lxr::led_clearSequencerLeds`, `Lxr::led_idx2StepLed`, `Lxr::led_initPatternSelectLeds`, `Lxr::led_setBlinkLed`, `Lxr::led_updateSeqLeds`, `Lxr::menu_getActiveVoice`, `Lxr::menu_repaint`, `Lxr::menu_repaintAll`, `Lxr::menu_setShownPattern`, `Lxr::preset_loadDrumsetRam`, `Lxr::preset_touchTimer`, `Lxr::seq_copyBackupBufferToPattern`, `Lxr::seq_copyPatternToBackupBuffer`, `Lxr::seq_getTrackLength`, `Lxr::seq_sendProgChg`, `Lxr::seq_setShuffle`, `Lxr::seq_setStepIndexToStart`, `Lxr::voiceControl_noteOff`.
    pub fn seq_setPendingPatternActive(&mut self) {
        self.midi_process_block = 1;
        if self.seq_activePattern != self.seq_pendingPattern || self.seq_loadPendigFlag != 0 {
            self.seq_setStepIndexToStart();

            self.seq_loadPendigFlag = 0;

            // copy current backup to current slot (discarding unsaved changes)
            let ap = self.seq_activePattern;
            self.seq_copyBackupBufferToPattern(ap);
            // copy pending pattern to backup buffer
            let pp = self.seq_pendingPattern;
            self.seq_copyPatternToBackupBuffer(pp);

            self.seq_activePattern = self.seq_pendingPattern;

            let ap = self.seq_activePattern as usize;
            self.activeParameters[PAR_SHUFFLE as usize] = self.seq_patternSet.shuffle[ap];
            // `x/127.f` is compiled as a multiplication by the reciprocal
            let sh = (self.activeParameters[PAR_SHUFFLE as usize] as i32 as f32) * RECIP_127;
            self.seq_setShuffle(sh);

            // A new pattern is activated here -> Load corresponding drumkit
            if self.activeParameters[PAR_PATTERN_MODE as usize] as i32 == SKIP_NEXT_KIT_AUTOLOAD {
                self.activeParameters[PAR_PATTERN_MODE as usize] = 1;
            } else if self.activeParameters[PAR_PATTERN_MODE as usize] == 1 {
                // turn FX off to save cycles
                self.fxBus.fxType = FX_OFF as _;
                let kit = self.seq_patternSet.kitNr[self.seq_activePattern as usize];
                self.preset_loadDrumsetRam(kit, 0, 1);
                let kit = self.seq_patternSet.kitNr[self.seq_activePattern as usize];
                self.menu_selectedPresetNr[SAVE_TYPE_KIT as usize] = kit;
                self.menu_loadedPresetNr[CURRENT_KIT as usize] = kit;
                self.preset_touchTimer(PRESET_KIT_TIMER);
            }

            // stop blinking pattern led
            let led = self.led_idx2StepLed(self.seq_activePattern);
            self.led_setBlinkLed(led, 0);

            self.menu_playedPattern = self.seq_activePattern;

            // send a pattern change message to midi/usb out
            let ap = self.seq_activePattern;
            self.seq_sendProgChg(ap);

            // all notes off here since we are switching patterns
            self.voiceControl_noteOff(0xFF);
        } else if self.activeParameters[PAR_PATTERN_MODE as usize] as i32 == SKIP_NEXT_KIT_AUTOLOAD {
            self.activeParameters[PAR_PATTERN_MODE as usize] = 1;
        }

        // update for song screen to follow patterns
        self.menu_songMenuPosition = self.seq_songPositionPointer;

        if self.menu_activePage < PATTERN_LOAD_PAGE || self.menu_activePage > SONG_MODE_PAGE {
            let ap = self.seq_activePattern;
            self.menu_setShownPattern(ap);
            self.led_clearSequencerLeds();
            let trackNr = self.menu_getActiveVoice();
            self.led_updateSeqLeds();
            self.activeParameters[PAR_TRACK_LENGTH as usize] = self.seq_getTrackLength(trackNr);
            self.menu_repaint();
        } else {
            // store the pending pattern update for shift button release handler
            self.menu_shownPattern = self.seq_activePattern;
        }

        if self.buttonHandler_getMode() == SELECT_MODE_PATTERN {
            self.led_clearSequencerLeds();
            self.led_clearAllBlinkLeds();
            self.led_initPatternSelectLeds();
            self.menu_repaint();
        } else if self.buttonHandler_getMode() == SELECT_MODE_SONG {
            self.menu_repaintAll();
        }

        self.midi_process_block = 0;
    }

    /// C: `uint8_t seq_getLongestTrack()`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:831`. Reference build: inlined into its callers only. Group P3.
    /// Globals: `seq_activePattern`, `seq_patternSet`.
    pub fn seq_getLongestTrack(&mut self) -> u8 {
        let ap = self.seq_activePattern as usize;
        let mut trackId: u8 = 0;
        for i in 1..NUM_TRACKS as usize {
            if self.seq_patternSet.length[ap][i] > self.seq_patternSet.length[ap][trackId as usize] {
                trackId = i as u8;
            }
        }
        trackId
    }

    /// C: `uint32_t seq_getShiftFlamPattern(const uint8_t track, uint8_t step, StepPosition stepPos)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:942`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_activePattern`, `seq_patternSet`, `seq_rndValue`.
    /// Calls: `sequencer::flamIdxToPattern`, `sequencer::getShiftValue`, `Lxr::seq_intIsMainStepActive`.
    pub fn seq_getShiftFlamPattern(&mut self, track: u8, step: u8, stepPos: StepPosition) -> u32 {
        let ap = self.seq_activePattern;
        let mut currentStepActive = self.seq_intIsMainStepActive(track, step, ap);
        if currentStepActive == 0 {
            return 0;
        }

        let timing = self.seq_stepTiming_read(ap, track, step as i32);
        let flamCurrentStep = timing.flam();
        let mut shiftCurrentStep: i8 = getShiftValue(timing.shift());

        // check probability
        let prob = self.seq_step_read(ap, track, step as i32).prob;
        let rnd = self.seq_rndValue[track as usize];
        let r = match stepPos {
            STEP_PREV => rnd.prev,
            STEP_NEXT => rnd.next,
            _ => rnd.current,
        };
        if r > prob {
            currentStepActive = 0;
        }
        if currentStepActive == 0 {
            return 0;
        }

        // add step pattern for active steps
        let stepPattern = flamIdxToPattern(flamCurrentStep);

        // we have 24 pulses per step (96ppq) but only 8 possible shift values (0-7)
        shiftCurrentStep = shiftCurrentStep.wrapping_mul(3);
        let sh = shiftCurrentStep as i32;

        if sh == 0 {
            match stepPos {
                STEP_PREV => 0,
                STEP_NEXT => 0,
                _ => stepPattern & 0xFFFFFF,
            }
        } else if sh > 0 {
            match stepPos {
                // data from prev step is shifted forward to this one
                STEP_PREV => (stepPattern << (24 - sh)) & 0xFFFFFF,
                STEP_NEXT => 0,
                // data is shifted forward, but a rest remains in this step
                _ => (stepPattern >> sh) & 0xFFFFFF,
            }
        } else {
            let sh = -sh;
            match stepPos {
                STEP_PREV => 0,
                // data from next step is shifted backwards to this one
                STEP_NEXT => (stepPattern >> (24 - sh)) & 0xFFFFFF,
                // data is shifted backwards, but a rest remains in this step
                _ => (stepPattern << sh) & 0xFFFFFF,
            }
        }
    }

    /// C: `void seq_processFlam(uint8_t idx)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:1091`. Reference build: out-of-line as a GCC clone (.part/.isra/.constprop). Group P3.
    /// Globals: `chokeCurrent`, `chokeNext`, `lastWasNoteOn`, `seq_activePattern`, `seq_mutedTracks`, `seq_patternSet`, `seq_running`, `seq_stepIndex`.
    /// Calls: `Lxr::seq_getShiftFlamPattern`, `Lxr::seq_triggerVoice`, `Lxr::voiceControl_noteOff`.
    pub fn seq_processFlam(&mut self, idx: u8) {
        // Ported from the SHIPPED v1.71 binary (seq_processFlam.part.0 at 0x080187f8, called from seq_tick),
        // which differs from the C source: it never reads or writes `lastWasNoteOn` and has no final
        // `else if (lastWasNoteOn[track]) { lastWasNoteOn[track] = 0; voiceControl_noteOff(track); }` branch.
        // The trigger branches (next > current > prev with the choke flags) are as in the C source.
        if self.seq_running == 0 {
            return;
        }

        if idx == 0 {
            self.chokeCurrent = 0;
            self.chokeNext = 0;
        }

        // 1UL << (23-idx), as `lsl.w` by a register (0 for shift amounts >= 32)
        let sa = 23u32.wrapping_sub(idx as u32) & 0xff;
        let mask: u32 = if sa < 32 { 1u32 << sa } else { 0 };

        for track in 0..NUM_TRACKS as u8 {
            // if track is not muted
            if (self.seq_mutedTracks as u32 >> track) & 1 != 0 {
                continue;
            }
            let ap = self.seq_activePattern;
            let length = self.seq_patternSet.length[ap as usize][track as usize];
            let si = self.seq_stepIndex[track as usize] as i32;
            let currentStep = si as u8;
            let nextStep = arm_srem(si + 1, length as i32) as u8;
            let prevStep: u8 = if si != 0 { currentStep.wrapping_sub(1) } else { length.wrapping_sub(1) };

            let currentStepPattern = self.seq_getShiftFlamPattern(track, currentStep, STEP_CURRENT);
            let nextStepPattern = self.seq_getShiftFlamPattern(track, nextStep, STEP_NEXT);
            let prevStepPattern = self.seq_getShiftFlamPattern(track, prevStep, STEP_PREV);

            if nextStepPattern & mask != 0 {
                self.chokeNext = 1;
                let s = self.seq_step_read(ap, track, nextStep as i32);
                self.seq_triggerVoice(track, s.volume & STEP_VOLUME_MASK as u8, s.note);
            } else if currentStepPattern & mask != 0 {
                if self.chokeNext == 0 {
                    self.chokeCurrent = 1;
                    let s = self.seq_step_read(ap, track, currentStep as i32);
                    self.seq_triggerVoice(track, s.volume & STEP_VOLUME_MASK as u8, s.note);
                }
            } else if prevStepPattern & mask != 0 {
                if (self.chokeNext | self.chokeCurrent) == 0 {
                    let s = self.seq_step_read(ap, track, prevStep as i32);
                    self.seq_triggerVoice(track, s.volume & STEP_VOLUME_MASK as u8, s.note);
                }
            }
        }
    }

    /// C: `void seq_MidiSyncQueue(uint8_t cnt)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:1236`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_deltaT`, `seq_lastShuffle`, `seq_lastTick`, `seq_prescaleCounter`, `seq_shuffle`, `seq_tempo`, `systick_ticks`.
    /// Calls: `Lxr::seq_calcDeltaT`, `Lxr::trigger_reset`.
    pub fn seq_MidiSyncQueue(&mut self, cnt: u8) {
        self.seq_prescaleCounter = cnt;
        self.trigger_reset(1);
        self.seq_lastTick = self.systick_ticks;

        if self.seq_tempo != 0 {
            let t = self.seq_tempo;
            self.seq_calcDeltaT(t, 0);
        }
    }

    /// C: `IN_ITCM_RAM void seq_nextStep()`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:1254`. Reference build: out-of-line in ITCM. Group P3.
    /// Globals: `activeParameters`, `frontParser_activeTrack`, `menu_activeBar`, `menu_activePage`, `menu_playedPattern`, `midi_process_block`, `seq_activePattern`, `seq_activeSong`, `seq_eraseActive`, `seq_lastMasterStep`, `seq_masterStepCnt`, `seq_mutedTracks`, `seq_patternSet`, `seq_pendingPattern`, `seq_recordActive`, `seq_rndValue`, `seq_rollRate`, `seq_rollState`, `seq_running`, `seq_songModeEnabled`, `seq_songPositionPointer`, `seq_stepIndex`.
    /// Calls: `Lxr::GetRngValue`, `Lxr::led_blinkHandler`, `Lxr::led_clearActive_step`, `Lxr::led_clearSequencerLeds`, `Lxr::led_idx2StepLed`, `Lxr::led_setActive_step`, `Lxr::led_setValue`, `Lxr::led_updateSeqLeds`, `Lxr::menu_getActiveVoice`, `Lxr::menu_getViewedPattern`, `Lxr::menu_setShownPattern`, `Lxr::midiParser_checkMtc`, `Lxr::seq_addNote`, `Lxr::seq_determineNextPattern`, `Lxr::seq_getLongestTrack`, `Lxr::seq_intIsMainStepActive`, `Lxr::seq_setMainStep`, `Lxr::seq_setPendingPatternActive`, `Lxr::seq_setRunning`, `Lxr::seq_triggerVoice`.
    pub fn seq_nextStep(&mut self) {
        self.midi_process_block = 1;
        self.led_blinkHandler(); // blink leds in sync to tempo/seq

        if self.seq_running == 0 {
            return;
        }

        self.seq_masterStepCnt = self.seq_masterStepCnt.wrapping_add(1);

        // ---- calc master step position. max value is 64. also take in regard the pattern length -----
        let longestTrack = self.seq_getLongestTrack();
        let mut seqlen = self.seq_patternSet.length[self.seq_activePattern as usize][longestTrack as usize];
        if seqlen == 0 {
            seqlen = 16;
        }

        let lsi = self.seq_stepIndex[longestTrack as usize] as i32;
        let masterStepPos: u8 = if ((lsi + 1) & 0x3f) == 0 || (lsi + 1) >= seqlen as i32 {
            0 // a bar has passed
        } else {
            (lsi + 1) as u8
        };

        // -------- check if the master track has ended and check if a pattern switch is necessary --------
        if masterStepPos == 0 {
            if self.seq_activePattern == self.seq_pendingPattern {
                // check pattern settings if we have to auto change patterns
                self.seq_pendingPattern = self.seq_determineNextPattern(1);
                if self.seq_pendingPattern as i32 == SONG_END {
                    self.seq_setRunning(0);
                    // make sure pending pattern does not remain at 64 resulting in a seg fault.
                    let song = self.seq_activeSong.expect("seq_activeSong is NULL");
                    self.seq_pendingPattern = self.seq_songData[song][0];
                    if self.seq_pendingPattern >= 64 {
                        self.seq_pendingPattern = 0;
                    }
                    self.seq_songPositionPointer = 0;
                    return;
                } else if self.seq_pendingPattern as i32 == SONG_LOOP {
                    // reset to song start
                    let song = self.seq_activeSong.expect("seq_activeSong is NULL");
                    self.seq_pendingPattern = self.seq_songData[song][0];
                    self.seq_songPositionPointer = 0;
                }
            }

            // a new pattern is about to start: set pendingPattern active
            self.seq_setPendingPatternActive();

            self.midi_process_block = 0;
        }

        // ---------- now check if the master track is at a full beat position to flash the start/stop button --------
        if (masterStepPos & 3) == 0 {
            self.led_setValue(1, LED_START_STOP);
        } else if (masterStepPos & 3) == 1 {
            self.led_setValue(0, LED_START_STOP);
        }

        // --------- Time to process the single tracks -------------------------
        for i in 0..NUM_TRACKS as usize {
            // increment the step index
            self.seq_stepIndex[i] = self.seq_stepIndex[i].wrapping_add(1);

            seqlen = self.seq_patternSet.length[self.seq_activePattern as usize][i];
            if seqlen == 0 {
                seqlen = 16;
            }

            if (self.seq_stepIndex[i] as i32) >= seqlen as i32 || (self.seq_stepIndex[i] & 0x3f) == 0 {
                // if end is reached reset track to step 0
                self.seq_stepIndex[i] = 0;
            }

            // if track is not muted
            if (self.seq_mutedTracks as u32 >> i) & 1 == 0 {
                // we only have mainsteps now
                let ap = self.seq_activePattern;
                if self.seq_intIsMainStepActive(i as u8, self.seq_stepIndex[i] as u8, ap) != 0 {
                    // **RECORD erase mode
                    if self.seq_eraseActive != 0 && i == self.frontParser_activeTrack as usize {
                        let (ap, at, si) = (self.seq_activePattern, self.frontParser_activeTrack, self.seq_stepIndex[i] as u8);
                        self.seq_setMainStep(ap, at, si, 0);
                        let s = self.seq_stepIndex[i] as i32;
                        let bar = self.menu_activeBar as i32;
                        if s >= bar * 16 && s < (bar + 1) * 16 {
                            let led = self.led_idx2StepLed((s % 16) as u8);
                            self.led_setValue(0, led);
                        }
                    } else {
                        // every step gets a random value
                        self.seq_rndValue[i].prev = self.seq_rndValue[i].current;
                        self.seq_rndValue[i].current = self.seq_rndValue[i].next;
                        self.seq_rndValue[i].next = (self.GetRngValue() % 100 + 1) as u8; // value between 0 and 100
                    }
                }
            }

            // ---- check if the roll mode has to trigger the voice
            if self.seq_rollRate != 0xff {
                // not in oneshot mode
                if (self.seq_rollState as u32 >> i) & 1 != 0 {
                    if arm_srem(self.seq_stepIndex[i] as i32, self.seq_rollRate as i32) == 0 {
                        let vol = ROLL_VOLUME as u8;
                        let ap = self.seq_activePattern;
                        let note = self.seq_step_read(ap, i as u8, self.seq_stepIndex[i] as i32).note;
                        self.seq_triggerVoice(i as u8, vol, note);
                        self.seq_addNote(i as u8, vol, note);
                    }
                }
            }
        }

        // step display follows bars
        if self.activeParameters[PAR_FOLLOW as usize] & FOLLOW_BAR as u8 != 0 {
            let av = self.menu_getActiveVoice();
            // only update if bar changed
            if self.menu_activeBar as i32 != (self.seq_stepIndex[av as usize] as i32) / 16 {
                if self.menu_activePage < PATTERN_LOAD_PAGE || self.menu_activePage > SONG_MODE_PAGE {
                    let ap = self.seq_activePattern;
                    self.menu_setShownPattern(ap);
                    self.led_clearSequencerLeds();
                    let trackNr = self.menu_getActiveVoice();
                    // update menu_activeBar and Bar LEDs
                    self.menu_activeBar = ((self.seq_stepIndex[trackNr as usize] as i32) / 16) as u8;
                    self.led_updateSeqLeds();
                }
            }
        }

        let shownPattern = self.menu_getViewedPattern();
        let playedPattern = self.menu_playedPattern;

        let stepIdx = self.seq_stepIndex[self.frontParser_activeTrack as usize] as u8;

        let bar = self.menu_activeBar as i32;
        let isOnActivePage = (stepIdx as i32) >= bar * 16 && (stepIdx as i32) < (bar * 16 + 16);

        if shownPattern == playedPattern && isOnActivePage {
            // only update chaselight LED when in step edit mode
            let page = self.menu_activePage;
            if page < MENU_MIDI_PAGE || page == PERF_PAGE || page == SEQ_PAGE || page == EUKLID_PAGE {
                self.led_setActive_step(stepIdx % 16);
            }
        } else {
            self.led_clearActive_step();
        }

        // check mtc, which might stop the sequencer if we haven't seen one in a while
        self.midiParser_checkMtc();
    }

    /// C: `uint8_t seq_getExtSync()`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:1518`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_isSyncExternal`.
    pub fn seq_getExtSync(&mut self) -> u8 {
        self.seq_isSyncExternal
    }

    /// C: `void seq_setExtSync(uint8_t isExt)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:1523`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_isSyncExternal`.
    pub fn seq_setExtSync(&mut self, isExt: u8) {
        self.seq_isSyncExternal = isExt;
    }

    /// C: `void seq_armAutomationStep(uint8_t stepNr, uint8_t track,uint8_t isArmed)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:1528`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_armedArmedAutomationStep`, `seq_armedArmedAutomationTrack`.
    pub fn seq_armAutomationStep(&mut self, stepNr: u8, track: u8, isArmed: u8) {
        if isArmed != 0 {
            self.seq_armedArmedAutomationStep = stepNr as i8;
            self.seq_armedArmedAutomationTrack = track as i8;
        } else {
            self.seq_armedArmedAutomationStep = -1;
            self.seq_armedArmedAutomationTrack = -1;
        }
    }

    /// C: `float seq_getElapsedStepTimePercent()`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:1642`. Reference build: inlined into its callers only. Group P3.
    /// Globals: `seq_prescaleCounter`.
    pub fn seq_getElapsedStepTimePercent(&mut self) -> f32 {
        // not linked in the reference build; its only caller (seq_addNote) compiles `>= 50.f` to `prescale >= 11.5f`
        self.seq_prescaleCounter as i32 as f32 / 23.0 * 100.0
    }

    /// C: `void seq_tick()`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:1650`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `isStepShuffled`, `midiParser_txRxFilter`, `seq_deltaT`, `seq_isSyncExternal`, `seq_lastShuffle`, `seq_lastTick`, `seq_masterStepCnt`, `seq_prescaleCounter`, `seq_running`, `seq_shuffle`, `seq_tempo`, `sync_delayedStepTimer`, `systick_ticks`, `trigger_clockReceived`.
    /// Calls: `Lxr::seq_MidiSyncQueue`, `Lxr::seq_calcDeltaT`, `Lxr::seq_getBpm`, `Lxr::seq_getDeltaTShuffleDelay`, `Lxr::seq_getExtSync`, `Lxr::seq_getMasterStepCnt`, `Lxr::seq_isRunning`, `Lxr::seq_nextStep`, `Lxr::seq_processFlam`, `Lxr::seq_sendRealtime`, `Lxr::trigger_clockTick`.
    pub fn seq_tick(&mut self) {
        // Ported from the machine code of the reference build (byte-identical in the shipped v1.71): GCC's
        // fast-math rewrites of seq_calcDeltaT / seq_getDeltaTShuffleDelay and of the `rest` arithmetic are
        // reproduced exactly (see seq_calcDeltaT).
        if self.seq_getExtSync() != 0 {
            if self.sync_delayedStepTimer != 0 {
                if self.systick_ticks >= self.sync_delayedStepTimer {
                    self.sync_delayedStepTimer = 0;
                    self.seq_nextStep();
                    self.seq_prescaleCounter = 0;
                    self.seq_MidiSyncQueue(0);
                }
            }

            // analogue clock (trigger_clockReceived is only incremented by the clock input interrupt)
            let mut first = true;
            while self.trigger_clockReceived != 0 {
                self.trigger_clockReceived -= 1;

                let isStepShuffled = self.seq_getMasterStepCnt() % 2 == 1;
                if isStepShuffled {
                    let bpm = self.seq_getBpm();
                    let t = self.systick_ticks as f32;
                    self.sync_delayedStepTimer = if first || bpm == 0 {
                        let dt = self.seq_getDeltaTShuffleDelay(bpm);
                        (t + dt) as u32
                    } else {
                        // the compiler's loop for the following iterations fuses the multiply-add
                        let k = self.seq_shuffle * f32::from_bits(0x3f2aaaab);
                        k.mul_add((60000u32 / bpm as u32) as i32 as f32, t) as u32
                    };
                    first = false;
                } else {
                    self.seq_nextStep();
                    self.seq_prescaleCounter = 0;
                    self.seq_MidiSyncQueue(0);
                    first = true;
                }
            }

            // midi clock flam stuff
            let diff = self.systick_ticks.wrapping_sub(self.seq_lastTick) as f32;
            let old = self.seq_deltaT;
            if diff >= old {
                let diff = self.systick_ticks.wrapping_sub(self.seq_lastTick) as f32;
                self.seq_lastTick = self.systick_ticks;

                let newdt = if self.seq_tempo != 0 {
                    let t = self.seq_tempo;
                    self.seq_calcDeltaT(t, 0);
                    self.seq_deltaT
                } else {
                    old
                };
                // deltaT = calc - (diff - deltaT_old), compiled as (old - diff) + calc
                self.seq_deltaT = (old - diff) + newdt;

                // do not autorun, internal clock used to upscale 24ppq MIDI clock for flam processing
                if self.seq_prescaleCounter != 24 {
                    if self.seq_isRunning() == 1 {
                        let pc = self.seq_prescaleCounter;
                        self.trigger_clockTick(0, pc);
                    }
                    let pc = self.seq_prescaleCounter;
                    self.seq_processFlam(pc);
                    self.seq_prescaleCounter = self.seq_prescaleCounter.wrapping_add(1);
                }
            }
        } else {
            if self.seq_deltaT == -1.0 {
                self.seq_deltaT = 32000.0;
                self.seq_nextStep();
                return;
            }

            let old = self.seq_deltaT;
            let diff = self.systick_ticks.wrapping_sub(self.seq_lastTick) as f32;
            if diff >= old {
                let diff = self.systick_ticks.wrapping_sub(self.seq_lastTick) as f32;
                self.seq_lastTick = self.systick_ticks;

                let t = self.seq_tempo;
                let s = self.isStepShuffled;
                self.seq_calcDeltaT(t, s);
                self.isStepShuffled = 0;
                // deltaT = calc - (diff - deltaT_old), compiled as (old - diff) + calc
                self.seq_deltaT = (old - diff) + self.seq_deltaT;

                if self.seq_prescaleCounter == 0 {
                    if self.seq_getExtSync() == 0 {
                        self.seq_nextStep();
                    }
                }

                if self.seq_isRunning() != 0 {
                    let pc = self.seq_prescaleCounter;
                    self.trigger_clockTick(0, pc);
                }

                let pc = self.seq_prescaleCounter;
                self.seq_processFlam(pc);

                if self.seq_getExtSync() == 0 {
                    // only send internal MIDI clock to output when external sync is off
                    if (self.seq_prescaleCounter as i32 % MIDI_PRESCALER_MASK) == 0 {
                        self.seq_sendRealtime(MIDI_CLOCK as u8);
                    }
                }
                self.seq_prescaleCounter = self.seq_prescaleCounter.wrapping_add(1);
                if self.seq_prescaleCounter == 23 {
                    self.isStepShuffled = (self.seq_masterStepCnt % 2 == 1) as u8;
                }
                if self.seq_prescaleCounter >= 24 {
                    self.seq_prescaleCounter = 0;
                }
            }
        }
    }

    /// C: `void seq_setQuantisation(uint8_t value)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:1860`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_quantisation`.
    pub fn seq_setQuantisation(&mut self, value: u8) {
        self.seq_quantisation = value;
    }

    /// C: `void seq_setMainStep(uint8_t patternNr, uint8_t voice, uint8_t stepNr, uint8_t onOff)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:1882`. Reference build: inlined into its callers only. Group P3.
    /// Globals: `seq_patternSet`.
    pub fn seq_setMainStep(&mut self, patternNr: u8, voice: u8, stepNr: u8, onOff: u8) {
        let bar = stepNr / 16;
        let shift = stepNr - bar * 16;
        let off = PS_MAINSTEPS_OFF + ((bar as i64 * 64 + patternNr as i64) * 7 + voice as i64) * 2;
        let mut w = self.ps_get(off) as u16 | (self.ps_get(off + 1) as u16) << 8;
        if onOff != 0 {
            w |= 1u16 << shift;
        } else {
            w &= !(1u16 << shift);
        }
        self.ps_set(off, w as u8);
        self.ps_set(off + 1, (w >> 8) as u8);
    }

    /// C: `uint8_t seq_isRunning()`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:1910`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_running`.
    pub fn seq_isRunning(&mut self) -> u8 {
        self.seq_running
    }

    /// C: `void seq_resetPosition()`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:1914`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `midiParser_txRxFilter`, `seq_deltaT`, `seq_lastMasterStep`, `seq_lastShuffle`, `seq_masterStepCnt`, `seq_songPositionPointer`, `seq_stepIndex`.
    /// Calls: `Lxr::led_resetBlinkTimer`, `Lxr::midiParser_checkMtc`, `Lxr::seq_sendRealtime`, `Lxr::seq_setStepIndexToStart`, `Lxr::trigger_reset`, `Lxr::voiceControl_noteOff`.
    pub fn seq_resetPosition(&mut self) {
        self.led_resetBlinkTimer();

        // reset song position bar counter
        self.seq_lastShuffle = 0.0;
        self.seq_masterStepCnt = 0;
        self.seq_songPositionPointer = 0xff; // -1
        // so the next seq_tick call will trigger the next step immediately
        self.seq_deltaT = 0.0;
        self.seq_sendRealtime(MIDI_STOP as u8);

        // send notes off on all channels that have notes playing and reset our bitmap to reflect that
        self.voiceControl_noteOff(0xFF);

        self.trigger_reset(0);

        // if mtc was doing it's thing, tell it to stop it.
        self.midiParser_checkMtc();

        self.seq_setStepIndexToStart();
    }

    /// C: `void seq_setRunning(uint8_t isRunning)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:1956`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `midiParser_txRxFilter`, `seq_lastMasterStep`, `seq_prescaleCounter`, `seq_running`, `seq_stepIndex`, `trigger_clockReceived`.
    /// Calls: `Lxr::preset_touchTimer`, `Lxr::seq_sendRealtime`, `Lxr::seq_setStepIndexToStart`, `Lxr::trigger_reset`.
    pub fn seq_setRunning(&mut self, isRunning: u8) {
        self.seq_running = isRunning;
        if self.seq_running == 0 {
            self.preset_touchTimer(PRESET_PAT_TIMER);
        } else {
            // abort pattern save timer, since seq is running again...
            self.preset_touchTimer(PRESET_STOP_PAT_TIMER);

            self.trigger_clockReceived = 0;
            self.seq_prescaleCounter = 0;
            self.seq_sendRealtime(MIDI_START as u8);
            self.trigger_reset(1);
        }
        self.seq_setStepIndexToStart();
    }

    /// C: `static uint8_t seq_intIsMainStepActive(uint8_t voice, uint8_t mainStepNr, uint8_t pattern)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:2034`. Reference build: inlined into its callers only. Group P3.
    /// Globals: `seq_patternSet`.
    pub fn seq_intIsMainStepActive(&mut self, voice: u8, mainStepNr: u8, pattern: u8) -> u8 {
        let bar = mainStepNr / 16;
        let shift = mainStepNr - bar * 16;
        let w = self.seq_mainSteps_read(bar, pattern, voice);
        ((w as u32 & (1u32 << shift)) > 0) as u8
    }

    /// C: `IN_ITCM_RAM void seq_setMute(uint8_t trackNr, uint8_t isMuted)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:2047`. Reference build: out-of-line in ITCM. Group P3.
    /// Globals: `midi_MidiChannels`, `seq_mutedTracks`.
    /// Calls: `Lxr::voiceControl_noteOff`.
    pub fn seq_setMute(&mut self, trackNr: u8, isMuted: u8) {
        if trackNr == 7 {
            // unmute all
            self.seq_mutedTracks = 0;
        } else if isMuted != 0 {
            // mute track
            self.seq_mutedTracks |= voice_bit(trackNr);
            // turn off the midi note that may be playing on that track
            let ch = self.midi_MidiChannels[trackNr as usize];
            self.voiceControl_noteOff(ch);
        } else {
            // unmute track
            self.seq_mutedTracks &= !voice_bit(trackNr);
        }
    }

    /// C: `uint8_t seq_isTrackMuted(uint8_t trackNr)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:2067`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_mutedTracks`.
    pub fn seq_isTrackMuted(&mut self, trackNr: u8) -> u8 {
        if self.seq_mutedTracks & voice_bit(trackNr) != 0 { 1 } else { 0 }
    }

    /// C: `void seq_sendStepInfoToFront(uint16_t stepNr)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:2115`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_patternSet`.
    /// Calls: `Lxr::uart_sendFrontpanelSysExByte`.
    pub fn seq_sendStepInfoToFront(&mut self, stepNr: u16) {
        let absPat = (stepNr / NUM_STEPS as u16) as u8;
        let currentTrack = absPat / 8;
        let currentPattern = absPat - currentTrack * 8;
        let currentStep = (stepNr - absPat as u16 * NUM_STEPS as u16) as u8;

        let d = self.seq_step_read(currentPattern, currentTrack, currentStep as i32);

        self.uart_sendFrontpanelSysExByte(d.volume & 0x7f);
        self.uart_sendFrontpanelSysExByte(d.prob & 0x7f);
        self.uart_sendFrontpanelSysExByte(d.note & 0x7f);
        self.uart_sendFrontpanelSysExByte(d.param1Nr & 0x7f);
        self.uart_sendFrontpanelSysExByte(d.param1Val & 0x7f);
        self.uart_sendFrontpanelSysExByte(d.param2Nr & 0x7f);
        self.uart_sendFrontpanelSysExByte(d.param2Val & 0x7f);

        // now the MSBs from all 7 values
        self.uart_sendFrontpanelSysExByte(
            (((d.volume & 0x80) >> 7)
                | ((d.prob & 0x80) >> 6)
                | ((d.note & 0x80) >> 5)
                | ((d.param1Nr & 0x80) >> 4)
                | ((d.param1Val & 0x80) >> 3)
                | ((d.param2Nr & 0x80) >> 2)
                | ((d.param2Val & 0x80) >> 1))
                & 0x7f,
        );
    }

    /// C: `void seq_setRoll(uint8_t voice, uint8_t onOff)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:2153`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_recordActive`, `seq_rollRate`, `seq_rollState`, `seq_running`.
    /// Calls: `Lxr::seq_addNote`, `Lxr::seq_isRunning`, `Lxr::seq_triggerVoice`.
    pub fn seq_setRoll(&mut self, voice: u8, onOff: u8) {
        if voice >= 7 {
            return;
        }
        if onOff != 0 {
            self.seq_rollState |= voice_bit(voice);
            // if seq is stopped, always use oneshot mode
            if self.seq_rollRate == 0xff || self.seq_isRunning() == 0 {
                // trigger one shot
                self.seq_triggerVoice(voice, ROLL_VOLUME as u8, SEQ_DEFAULT_NOTE as u8);
                // record roll notes
                self.seq_addNote(voice, ROLL_VOLUME as u8, SEQ_DEFAULT_NOTE as u8);
            }
        } else {
            self.seq_rollState &= !voice_bit(voice);
        }
    }

    /// C: `void seq_setRollRate(uint8_t rate)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:2171`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_rollRate`.
    pub fn seq_setRollRate(&mut self, rate: u8) {
        self.seq_rollRate = match rate {
            0 => 0xfe,
            1 => 15, // 1/1
            2 => 7,  // 1/2
            3 => 3,  // 1/4
            4 => 1,  // 1/8
            _ => 0,  // 1/16 / default
        };
        self.seq_rollRate = self.seq_rollRate.wrapping_add(1);
    }

    /// C: `uint8_t seq_isAutomationStepArmed()`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:2301`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_armedArmedAutomationStep`.
    pub fn seq_isAutomationStepArmed(&mut self) -> u8 {
        if self.seq_armedArmedAutomationStep != -1 { 1 } else { 0 }
    }

    /// C: `void seq_recordAutomation(uint8_t voice, uint8_t dest, uint8_t value)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:2307`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_activeAutomTrack`, `seq_activePattern`, `seq_armedArmedAutomationStep`, `seq_armedArmedAutomationTrack`, `seq_patternSet`, `seq_recordActive`, `seq_stepIndex`.
    /// Calls: `Lxr::seq_intIsMainStepActive`.
    pub fn seq_recordAutomation(&mut self, voice: u8, dest: u8, value: u8) {
        if self.seq_recordActive != 0 {
            let step = self.seq_stepIndex[voice as usize] as u8;
            let ap = self.seq_activePattern;
            if self.seq_intIsMainStepActive(voice, step, ap) != 0 {
                let mut s = self.seq_step_read(ap, voice, step as i32);
                if self.seq_activeAutomTrack == 0 {
                    s.param1Nr = dest;
                    s.param1Val = value;
                } else {
                    s.param2Nr = dest;
                    s.param2Val = value;
                }
                self.seq_step_write(ap, voice, step as i32, s);
            }
        }

        if self.seq_armedArmedAutomationStep != -1 && self.seq_armedArmedAutomationTrack != -1 {
            // step button is held down -> set step automation parameters
            let ap = self.seq_activePattern;
            let tr = self.seq_armedArmedAutomationTrack as u8;
            let st = self.seq_armedArmedAutomationStep as i32;
            let mut s = self.seq_step_read(ap, tr, st);
            if self.seq_activeAutomTrack == 0 {
                s.param1Nr = dest;
                s.param1Val = value;
            } else {
                s.param2Nr = dest;
                s.param2Val = value;
            }
            self.seq_step_write(ap, tr, st, s);
        }
    }

    /// C: `void seq_addNote(uint8_t trackNr,uint8_t vel, uint8_t note)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:2364`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `frontParser_activeTrack`, `frontParser_shownPattern`, `menu_activeBar`, `menu_activePage`, `seq_activePattern`, `seq_patternSet`, `seq_prescaleCounter`, `seq_recordActive`, `seq_running`, `seq_stepIndex`, `seq_stepToSkip`.
    /// Calls: `Lxr::led_setValue`, `Lxr::seq_determineNextPattern`, `Lxr::seq_getElapsedStepTimePercent`, `Lxr::seq_setMainStep`.
    pub fn seq_addNote(&mut self, trackNr: u8, vel: u8, note: u8) {
        // Machine code: `seq_getElapsedStepTimePercent() >= 50.f` is `(float)seq_prescaleCounter >= 11.5f`, and the
        // cross-pattern branch (`step == 0 && seq_stepIndex > 32`) is removed as unreachable: the target pattern
        // is always seq_activePattern.
        if self.seq_running != 0 && self.seq_recordActive != 0 {
            let t = trackNr as usize;
            let mut step = self.seq_stepIndex[t] as u8;
            let mut quantizedStep: i8 = if (self.seq_prescaleCounter as i32 as f32) >= 11.5 {
                (self.seq_stepIndex[t] as u8).wrapping_add(1) as i8
            } else {
                self.seq_stepIndex[t]
            };

            let targetPattern = self.seq_activePattern;

            // limit to track length
            let len = self.seq_patternSet.length[targetPattern as usize][t];
            quantizedStep = arm_srem(quantizedStep as i32, len as i32) as i8;
            if step as i32 != quantizedStep as i32 {
                step = quantizedStep as u8;
                self.seq_stepToSkip[t] = step as i8;
            }

            // set the current step in the requested track active
            let mut s = self.seq_step_read(targetPattern, trackNr, step as i32);
            s.note = note;
            s.volume = vel;
            s.prob = 100;
            s.volume |= STEP_ACTIVE_MASK as u8;
            self.seq_step_write(targetPattern, trackNr, step as i32, s);

            // activate corresponding main step
            self.seq_setMainStep(targetPattern, trackNr, step, 1);

            if self.frontParser_shownPattern == targetPattern
                && self.frontParser_activeTrack == trackNr
                && self.menu_activeBar == step / 16
            {
                if self.menu_activePage != PERF_PAGE {
                    let stepNr = step % 16;
                    let led = self.led_idx2StepLed(stepNr);
                    self.led_setValue(1, led);
                }
            }
        }
    }

    /// C: `void seq_setRecordingMode(uint8_t active)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:2514`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_recordActive`.
    pub fn seq_setRecordingMode(&mut self, active: u8) {
        self.seq_recordActive = active;
    }

    /// C: `void seq_setErasingMode(uint8_t active)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:2519`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_eraseActive`.
    pub fn seq_setErasingMode(&mut self, active: u8) {
        self.seq_eraseActive = active;
    }

    /// C: `void seq_clearTrack(uint8_t trackNr, uint8_t pattern)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:2537`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_patternSet`.
    /// Calls: `sequencer::seq_resetNote`.
    pub fn seq_clearTrack(&mut self, trackNr: u8, pattern: u8) {
        let p = pattern as usize;
        let t = trackNr as usize;
        for k in 0..NUM_STEPS as usize {
            seq_resetNote(&mut self.seq_patternSet.seq_subStepPattern[p][t][k]);

            self.seq_patternSet.seq_stepTiming[p][t][k].set_flam(0);
            self.seq_patternSet.seq_stepTiming[p][t][k].set_shift(7); // 7 = no shift

            // every 1st step in a substep pattern active
            if k % 8 == 0 {
                self.seq_patternSet.seq_subStepPattern[p][t][k].volume |= STEP_ACTIVE_MASK as u8;
            }
        }

        // all main steps off for this track
        for i in 0..NUM_BARS as usize {
            self.seq_patternSet.seq_mainSteps[i][p][t] = 0;
        }

        // **PATROT all pattern rotations off and all patterns set to length 16
        self.seq_patternSet.length[p][t] = 16;
    }

    /// C: `void seq_clearPattern(uint8_t pattern)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:2560`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_patternSet`.
    /// Calls: `Lxr::seq_clearTrack`.
    pub fn seq_clearPattern(&mut self, pattern: u8) {
        for i in 0..NUM_TRACKS as u8 {
            self.seq_clearTrack(i, pattern);
        }
    }

    /// C: `void seq_clearAutomation(uint8_t trackNr, uint8_t pattern, uint8_t automTrack)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:2568`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_patternSet`.
    pub fn seq_clearAutomation(&mut self, trackNr: u8, pattern: u8, automTrack: u8) {
        let p = pattern as usize;
        let t = trackNr as usize;
        for k in 0..NUM_STEPS as usize {
            let s = &mut self.seq_patternSet.seq_subStepPattern[p][t][k];
            if automTrack == 0 {
                s.param1Nr = NO_AUTOMATION as u8;
                s.param1Val = 0;
            } else {
                s.param2Nr = NO_AUTOMATION as u8;
                s.param2Val = 0;
            }
        }
    }

    /// C: `void seq_copyTrack(uint8_t srcNr, uint8_t dstNr, uint8_t pattern)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:2588`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_patternSet`.
    pub fn seq_copyTrack(&mut self, srcNr: u8, dstNr: u8, pattern: u8) {
        let p = pattern as usize;
        let (src, dst) = (srcNr as usize, dstNr as usize);
        for k in 0..NUM_STEPS as usize {
            let s = self.seq_patternSet.seq_subStepPattern[p][src][k];
            self.seq_patternSet.seq_subStepPattern[p][dst][k] = s;
            self.seq_patternSet.seq_stepTiming[p][dst][k] = self.seq_patternSet.seq_stepTiming[p][src][k];
        }
        for i in 0..NUM_BARS as usize {
            self.seq_patternSet.seq_mainSteps[i][p][dst] = self.seq_patternSet.seq_mainSteps[i][p][src];
        }
        self.seq_patternSet.length[p][dst] = self.seq_patternSet.length[p][src];
    }

    /// C: `void seq_copyPattern(uint8_t src, uint8_t dst)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:2618`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_patternSet`.
    pub fn seq_copyPattern(&mut self, src: u8, dst: u8) {
        let (s, d) = (src as usize, dst as usize);
        for j in 0..NUM_TRACKS as usize {
            for k in 0..NUM_STEPS as usize {
                let st = self.seq_patternSet.seq_subStepPattern[s][j][k];
                self.seq_patternSet.seq_subStepPattern[d][j][k] = st;
                self.seq_patternSet.seq_stepTiming[d][j][k] = self.seq_patternSet.seq_stepTiming[s][j][k];
            }
            for i in 0..NUM_BARS as usize {
                self.seq_patternSet.seq_mainSteps[i][d][j] = self.seq_patternSet.seq_mainSteps[i][s][j];
            }
            self.seq_patternSet.length[d][j] = self.seq_patternSet.length[s][j];
            self.seq_patternSet.shuffle[d] = self.seq_patternSet.shuffle[s];
            self.seq_patternSet.kitNr[d] = self.seq_patternSet.kitNr[s];
        }
    }

    /// C: `void seq_setActiveAutomationTrack(uint8_t trackNr)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:2652`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_activeAutomTrack`.
    pub fn seq_setActiveAutomationTrack(&mut self, trackNr: u8) {
        self.seq_activeAutomTrack = trackNr;
    }

    /// C: `void seq_midiNoteOff(uint8_t chan)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:2679`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `midi_chan_notes`, `midi_notes_on`.
    /// Calls: `Lxr::seq_sendMidi`.
    pub fn seq_midiNoteOff(&mut self, chan: u8) {
        // `msg` is a local: only status/data1/data2/bits.length are set; MIDI output is not observed.
        let mut msg = MidiMsg::ZERO;
        msg.bits.set_length(2);
        msg.data2 = 0;

        if chan == 0xff {
            // all notes off
            for i in 0..16u32 {
                if (1u32 << i) & self.midi_notes_on as u32 != 0 {
                    msg.status = NOTE_OFF as u8 | i as u8;
                    msg.data1 = self.midi_chan_notes[i as usize];
                    self.seq_sendMidi(msg);
                }
            }
            // reset all
            self.midi_notes_on = 0;
            return;
        }
        let bit: u32 = if (chan as u32) < 32 { 1u32 << chan } else { 0 };
        if bit & self.midi_notes_on as u32 != 0 {
            msg.status = NOTE_OFF as u8 | chan;
            msg.data1 = self.midi_chan_notes[chan as usize];
            self.seq_sendMidi(msg);
            // turn off our knowledge of that note playing
            self.midi_notes_on &= !(bit as u16);
        }
    }

    /// C: `static void seq_sendRealtime(const uint8_t status)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:2712`. Reference build: inlined into its callers only. Group P3.
    /// Globals: `midiParser_txRxFilter`, `seq_sendRealtime_msg`.
    /// Calls: `Lxr::seq_sendMidi`.
    pub fn seq_sendRealtime(&mut self, status: u8) {
        // FILT filter out realtime msgs if appropriate
        if (self.midiParser_txRxFilter & (MIDI_FILTER_SYSTEM << 4)) == 0 {
            return;
        }
        self.seq_sendRealtime_msg.status = status;
        let m = self.seq_sendRealtime_msg;
        self.seq_sendMidi(m);
    }

    /// C: `void seq_sendMidiNoteOn(const uint8_t channel, const uint8_t note, const uint8_t veloc)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:2725`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `midiParser_txRxFilter`, `midi_chan_notes`, `midi_notes_on`, `seq_sendMidiNoteOn_msg`.
    /// Calls: `Lxr::seq_sendMidi`.
    pub fn seq_sendMidiNoteOn(&mut self, channel: u8, note: u8, veloc: u8) {
        // FILT filter out note msgs if appropriate
        if (self.midiParser_txRxFilter & (MIDI_FILTER_NOTES << 4)) == 0 {
            return;
        }
        self.seq_sendMidiNoteOn_msg.status = NOTE_ON as u8 | channel;
        self.seq_sendMidiNoteOn_msg.data1 = note;
        self.seq_sendMidiNoteOn_msg.data2 = veloc;
        let m = self.seq_sendMidiNoteOn_msg;
        self.seq_sendMidi(m);

        // keep track of which notes are on so we can turn them off later
        if (channel as usize) < self.midi_chan_notes.len() {
            self.midi_chan_notes[channel as usize] = note;
        } else if let Some(a) = crate::midi::shipped_layout::shipped_offset(crate::memmap::ADDR_midi_chan_notes, channel as u32) {
            // MIDI channel parameters set above 15 over MIDI: C writes past the array (shipped layout);
            // bytes outside the modelled memory are dropped
            self.global_set_byte(a, note);
        }
        let bit: u32 = if (channel as u32) < 32 { 1u32 << channel } else { 0 };
        self.midi_notes_on |= bit as u16;
    }

    /// C: `static void seq_sendProgChg(const uint8_t ptn)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:2753`. Reference build: inlined into its callers only. Group P3.
    /// Globals: `midiParser_txRxFilter`, `midi_MidiChannels`, `seq_sendProgChg_msg`.
    /// Calls: `Lxr::seq_sendMidi`.
    pub fn seq_sendProgChg(&mut self, ptn: u8) {
        // FILT filter out PC msgs if appropriate
        if (self.midiParser_txRxFilter & (MIDI_FILTER_PROG_C << 4)) == 0 {
            return;
        }
        self.seq_sendProgChg_msg.status = PROG_CHANGE as u8 | self.midi_MidiChannels[7];
        self.seq_sendProgChg_msg.data1 = ptn;
        self.seq_sendProgChg_msg.bits.set_length(1);
        let m = self.seq_sendProgChg_msg;
        self.seq_sendMidi(m);
    }

    /// C: `static void seq_setStepIndexToStart()`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:2774`. Reference build: inlined into its callers only. Group P3.
    /// Globals: `seq_lastMasterStep`, `seq_stepIndex`.
    pub fn seq_setStepIndexToStart(&mut self) {
        for i in 0..NUM_TRACKS as usize {
            // this is for external clock sync via trigger expansion kit (the ext tick will adjust this -1)
            self.seq_lastMasterStep[i] = 0;
            // -1 here because we increment it first thing when we start
            self.seq_stepIndex[i] = -1;
        }
    }
}

/// `1 << n` truncated to u8, as the Cortex-M7 computes it (`lsl` by a register: 0 for n >= 32).
#[inline(always)]
fn voice_bit(n: u8) -> u8 {
    if n < 32 { (1u32 << n) as u8 } else { 0 }
}

/// C `a % b` for `int` operands as compiled (`sdiv` + `mls`): the Cortex-M7 `sdiv` returns 0 for a
/// zero divisor, so `a % 0 == a`.
#[inline(always)]
pub fn arm_srem(a: i32, b: i32) -> i32 {
    if b == 0 { a } else { a.wrapping_rem(b) }
}

// Byte offsets of the members of `PatternSet` (C layout, see `cmem`).
const PS_STEPTIMING_OFF: i64 = 64 * 7 * 64 * 7;
const PS_MAINSTEPS_OFF: i64 = PS_STEPTIMING_OFF + 64 * 7 * 64;

impl Lxr {
    /// Reads byte `off` of the C object `seq_patternSet`. Offsets outside the object (from negative step
    /// indices or step numbers >= 64, which the firmware does not range check) read the neighbouring
    /// globals at the reference ELF addresses. Such reads only reach `seq_songData`, `seq_stepIndex`, ...
    /// or kit slots >= 7 of `mainParameters`, which hold the same values in the shipped layout
    /// (unused kit slots are zero without an SD card).
    fn ps_get(&self, off: i64) -> u8 {
        if off >= 0 && off < PatternSet::SIZE as i64 {
            self.seq_patternSet.get_byte(off as u32)
        } else {
            let a = (ADDR_seq_patternSet as i64 + off) as u32;
            // memory the engine does not model reads 0 (see `memmap::cptr_byte`)
            self.global_get_byte(a).unwrap_or(0)
        }
    }

    fn ps_set(&mut self, off: i64, v: u8) {
        if off >= 0 && off < PatternSet::SIZE as i64 {
            self.seq_patternSet.set_byte(off as u32, v)
        } else {
            let a = (ADDR_seq_patternSet as i64 + off) as u32;
            // stores to memory the engine does not model are dropped (see `memmap::cptr_set_byte`)
            let _ = self.global_set_byte(a, v);
        }
    }

    /// `seq_patternSet.seq_subStepPattern[pat][track][step]` with C (unchecked, flat) indexing.
    pub fn seq_step_read(&self, pat: u8, track: u8, step: i32) -> Step {
        let flat = (pat as i64 * 7 + track as i64) * 64 + step as i64;
        if (0..64 * 7 * 64).contains(&flat) {
            let f = flat as usize;
            return self.seq_patternSet.seq_subStepPattern[f / 448][(f / 64) % 7][f % 64];
        }
        let o = flat * 7;
        Step {
            volume: self.ps_get(o),
            prob: self.ps_get(o + 1),
            note: self.ps_get(o + 2),
            param1Nr: self.ps_get(o + 3),
            param1Val: self.ps_get(o + 4),
            param2Nr: self.ps_get(o + 5),
            param2Val: self.ps_get(o + 6),
        }
    }

    /// Write counterpart of `seq_step_read`.
    pub fn seq_step_write(&mut self, pat: u8, track: u8, step: i32, s: Step) {
        let flat = (pat as i64 * 7 + track as i64) * 64 + step as i64;
        if (0..64 * 7 * 64).contains(&flat) {
            let f = flat as usize;
            self.seq_patternSet.seq_subStepPattern[f / 448][(f / 64) % 7][f % 64] = s;
            return;
        }
        let o = flat * 7;
        for (i, b) in [s.volume, s.prob, s.note, s.param1Nr, s.param1Val, s.param2Nr, s.param2Val].into_iter().enumerate() {
            self.ps_set(o + i as i64, b);
        }
    }

    /// `seq_patternSet.seq_stepTiming[pat][track][step]` with C (unchecked, flat) indexing.
    pub fn seq_stepTiming_read(&self, pat: u8, track: u8, step: i32) -> StepTiming {
        let flat = (pat as i64 * 7 + track as i64) * 64 + step as i64;
        StepTiming { bits: self.ps_get(PS_STEPTIMING_OFF + flat) }
    }

    /// Write counterpart of `seq_stepTiming_read`.
    pub fn seq_stepTiming_write(&mut self, pat: u8, track: u8, step: i32, t: StepTiming) {
        let flat = (pat as i64 * 7 + track as i64) * 64 + step as i64;
        self.ps_set(PS_STEPTIMING_OFF + flat, t.bits);
    }

    /// `seq_patternSet.seq_mainSteps[bar][pattern][voice]` with C (unchecked, flat) indexing.
    fn seq_mainSteps_read(&self, bar: u8, pattern: u8, voice: u8) -> u16 {
        if bar < 4 && pattern < 64 && voice < 7 {
            return self.seq_patternSet.seq_mainSteps[bar as usize][pattern as usize][voice as usize];
        }
        let off = PS_MAINSTEPS_OFF + ((bar as i64 * 64 + pattern as i64) * 7 + voice as i64) * 2;
        self.ps_get(off) as u16 | (self.ps_get(off + 1) as u16) << 8
    }
}
