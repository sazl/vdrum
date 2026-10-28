//! Port of `Core/Inc/Soundengine/Sequencer/sequencer.h`, `Core/Src/Soundengine/Sequencer/sequencer.c`.
//!
//! Porting group: P3. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

pub mod clock_sync;
pub mod euklid_generator;

use crate::prelude::*;

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
    todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:845")
}

/// C: `uint32_t flamIdxToPattern(uint8_t flamIdx)`
///
/// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:911`. Reference build: inlined into its callers only. Group P3.
pub fn flamIdxToPattern(flamIdx: u8) -> u32 {
    todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:911")
}

/// C: `static void seq_resetNote(Step *step)`
///
/// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:2526`. Reference build: inlined into its callers only. Group P3.
pub fn seq_resetNote(step: &mut Step) {
    todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:2526")
}

// ---- functions that use globals (methods on `Lxr`)
impl Lxr {
    /// C: `uint8_t seq_getMasterStepCnt()`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:176`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_masterStepCnt`.
    pub fn seq_getMasterStepCnt(&mut self) -> u8 {
        todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:176")
    }

    /// C: `void seq_init()`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:181`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_activeSong`, `seq_automationNodes`, `seq_lastMasterStep`, `seq_patternSet`, `seq_songData`, `seq_stepIndex`, `seq_stepToSkip`.
    /// Calls: `dsp::automation_node::autoNode_init`, `Lxr::seq_clearPattern`.
    pub fn seq_init(&mut self) {
        todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:181")
    }

    /// C: `void seq_copyPatternToBackupBuffer(uint8_t patternNr)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:225`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `patternBackupBuffer`, `seq_patternSet`.
    pub fn seq_copyPatternToBackupBuffer(&mut self, patternNr: u8) {
        todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:225")
    }

    /// C: `void seq_copyBackupBufferToPattern(uint8_t patternNr)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:242`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `patternBackupBuffer`, `seq_patternSet`.
    pub fn seq_copyBackupBufferToPattern(&mut self, patternNr: u8) {
        todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:242")
    }

    /// C: `void seq_setShuffle(float shuffle)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:269`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_shuffle`.
    pub fn seq_setShuffle(&mut self, shuffle: f32) {
        todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:269")
    }

    /// C: `void seq_setTrackLength(uint8_t trackNr, uint8_t length)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:274`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `frontParser_shownPattern`, `seq_patternSet`.
    pub fn seq_setTrackLength(&mut self, trackNr: u8, length: u8) {
        todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:274")
    }

    /// C: `uint8_t seq_getTrackLength(uint8_t trackNr)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:293`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `frontParser_shownPattern`, `seq_patternSet`.
    pub fn seq_getTrackLength(&mut self, trackNr: u8) -> u8 {
        todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:293")
    }

    /// C: `float seq_getDeltaTShuffleDelay(uint16_t bpm)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:354`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_shuffle`.
    pub fn seq_getDeltaTShuffleDelay(&mut self, bpm: u16) -> f32 {
        todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:354")
    }

    /// C: `static void seq_calcDeltaT(uint16_t bpm, uint8_t isShuffled)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:386`. Reference build: inlined into its callers only. Group P3.
    /// Globals: `isStepShuffled`, `seq_deltaT`, `seq_lastShuffle`, `seq_shuffle`.
    pub fn seq_calcDeltaT(&mut self, bpm: u16, isShuffled: u8) {
        todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:386")
    }

    /// C: `void seq_setBpm(uint16_t bpm)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:437`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_tempo`.
    /// Calls: `Lxr::lfo_recalcSync`.
    pub fn seq_setBpm(&mut self, bpm: u16) {
        todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:437")
    }

    /// C: `uint16_t seq_getBpm()`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:444`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_tempo`.
    pub fn seq_getBpm(&mut self) -> u16 {
            // Ported by P1 (needed by lfo_calcPhaseInc; difftested in p1_lfo.rs)
        self.seq_tempo
    }

    /// C: `void seq_sync()`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:449`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_isSyncExternal`.
    /// Calls: `Lxr::seq_getExtSync`, `Lxr::sync_tick`.
    pub fn seq_sync(&mut self) {
        todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:449")
    }

    /// C: `void seq_setNextPattern(const uint8_t patNr)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:457`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_loadPendigFlag`, `seq_pendingPattern`, `seq_running`.
    /// Calls: `Lxr::seq_setPendingPatternActive`.
    pub fn seq_setNextPattern(&mut self, patNr: u8) {
        todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:457")
    }

    /// C: `static void seq_sendMidi(MidiMsg msg)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:470`. Reference build: inlined into its callers only. Group P3.
    /// Calls: `Lxr::uart_sendMidi`, `Lxr::usb_sendMidi`.
    pub fn seq_sendMidi(&mut self, msg: MidiMsg) {
        todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:470")
    }

    /// C: `static void seq_parseAutomationNodes(uint8_t track, Step* stepData)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:482`. Reference build: inlined into its callers only. Group P3.
    /// Globals: `seq_automationNodes`.
    /// Calls: `Lxr::autoNode_setDestination`, `Lxr::autoNode_updateValue`.
    pub fn seq_parseAutomationNodes(&mut self, track: u8, stepData: &mut Step) {
        todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:482")
    }

    /// C: `void seq_triggerVoice(uint8_t voiceNr, uint8_t vol, uint8_t note)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:492`. Reference build: out-of-line as a GCC clone (.part/.isra/.constprop). Group P3.
    /// Globals: `activeParameters`, `midi_MidiChannels`, `seq_activePattern`, `seq_patternSet`, `seq_stepIndex`.
    /// Calls: `Lxr::seq_parseAutomationNodes`, `Lxr::seq_sendMidiNoteOn`, `Lxr::voiceControl_noteOff`, `Lxr::voiceControl_noteOn`.
    pub fn seq_triggerVoice(&mut self, voiceNr: u8, vol: u8, note: u8) {
        todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:492")
    }

    /// C: `static uint8_t seq_determineNextPattern(uint8_t autoIncrement)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:565`. Reference build: inlined into its callers only. Group P3.
    /// Globals: `seq_activePattern`, `seq_activeSong`, `seq_songModeEnabled`, `seq_songPositionPointer`.
    pub fn seq_determineNextPattern(&mut self, autoIncrement: u8) -> u8 {
        todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:565")
    }

    /// C: `IN_ITCM_RAM void seq_setPendingPatternActive()`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:602`. Reference build: out-of-line in ITCM. Group P3.
    /// Globals: `activeParameters`, `frontParser_shownPattern`, `fxBus`, `menu_activePage`, `menu_loadedPresetNr`, `menu_playedPattern`, `menu_selectedPresetNr`, `menu_shownPattern`, `menu_songMenuPosition`, `midiParser_txRxFilter`, `midi_MidiChannels`, `midi_process_block`, `seq_activePattern`, `seq_lastMasterStep`, `seq_loadPendigFlag`, `seq_patternSet`, `seq_pendingPattern`, `seq_shuffle`, `seq_songPositionPointer`, `seq_stepIndex`.
    /// Calls: `Lxr::buttonHandler_getMode`, `Lxr::led_clearAllBlinkLeds`, `Lxr::led_clearSequencerLeds`, `Lxr::led_idx2StepLed`, `Lxr::led_initPatternSelectLeds`, `Lxr::led_setBlinkLed`, `Lxr::led_updateSeqLeds`, `Lxr::menu_getActiveVoice`, `Lxr::menu_repaint`, `Lxr::menu_repaintAll`, `Lxr::menu_setShownPattern`, `Lxr::preset_loadDrumsetRam`, `Lxr::preset_touchTimer`, `Lxr::seq_copyBackupBufferToPattern`, `Lxr::seq_copyPatternToBackupBuffer`, `Lxr::seq_getTrackLength`, `Lxr::seq_sendProgChg`, `Lxr::seq_setShuffle`, `Lxr::seq_setStepIndexToStart`, `Lxr::voiceControl_noteOff`.
    pub fn seq_setPendingPatternActive(&mut self) {
        todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:602")
    }

    /// C: `uint8_t seq_getLongestTrack()`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:831`. Reference build: inlined into its callers only. Group P3.
    /// Globals: `seq_activePattern`, `seq_patternSet`.
    pub fn seq_getLongestTrack(&mut self) -> u8 {
        todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:831")
    }

    /// C: `uint32_t seq_getShiftFlamPattern(const uint8_t track, uint8_t step, StepPosition stepPos)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:942`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_activePattern`, `seq_patternSet`, `seq_rndValue`.
    /// Calls: `sequencer::flamIdxToPattern`, `sequencer::getShiftValue`, `Lxr::seq_intIsMainStepActive`.
    pub fn seq_getShiftFlamPattern(&mut self, track: u8, step: u8, stepPos: StepPosition) -> u32 {
        todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:942")
    }

    /// C: `void seq_processFlam(uint8_t idx)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:1091`. Reference build: out-of-line as a GCC clone (.part/.isra/.constprop). Group P3.
    /// Globals: `chokeCurrent`, `chokeNext`, `lastWasNoteOn`, `seq_activePattern`, `seq_mutedTracks`, `seq_patternSet`, `seq_running`, `seq_stepIndex`.
    /// Calls: `Lxr::seq_getShiftFlamPattern`, `Lxr::seq_triggerVoice`, `Lxr::voiceControl_noteOff`.
    pub fn seq_processFlam(&mut self, idx: u8) {
        todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:1091")
    }

    /// C: `void seq_MidiSyncQueue(uint8_t cnt)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:1236`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_deltaT`, `seq_lastShuffle`, `seq_lastTick`, `seq_prescaleCounter`, `seq_shuffle`, `seq_tempo`, `systick_ticks`.
    /// Calls: `Lxr::seq_calcDeltaT`, `Lxr::trigger_reset`.
    pub fn seq_MidiSyncQueue(&mut self, cnt: u8) {
        todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:1236")
    }

    /// C: `IN_ITCM_RAM void seq_nextStep()`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:1254`. Reference build: out-of-line in ITCM. Group P3.
    /// Globals: `activeParameters`, `frontParser_activeTrack`, `menu_activeBar`, `menu_activePage`, `menu_playedPattern`, `midi_process_block`, `seq_activePattern`, `seq_activeSong`, `seq_eraseActive`, `seq_lastMasterStep`, `seq_masterStepCnt`, `seq_mutedTracks`, `seq_patternSet`, `seq_pendingPattern`, `seq_recordActive`, `seq_rndValue`, `seq_rollRate`, `seq_rollState`, `seq_running`, `seq_songModeEnabled`, `seq_songPositionPointer`, `seq_stepIndex`.
    /// Calls: `Lxr::GetRngValue`, `Lxr::led_blinkHandler`, `Lxr::led_clearActive_step`, `Lxr::led_clearSequencerLeds`, `Lxr::led_idx2StepLed`, `Lxr::led_setActive_step`, `Lxr::led_setValue`, `Lxr::led_updateSeqLeds`, `Lxr::menu_getActiveVoice`, `Lxr::menu_getViewedPattern`, `Lxr::menu_setShownPattern`, `Lxr::midiParser_checkMtc`, `Lxr::seq_addNote`, `Lxr::seq_determineNextPattern`, `Lxr::seq_getLongestTrack`, `Lxr::seq_intIsMainStepActive`, `Lxr::seq_setMainStep`, `Lxr::seq_setPendingPatternActive`, `Lxr::seq_setRunning`, `Lxr::seq_triggerVoice`.
    pub fn seq_nextStep(&mut self) {
        todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:1254")
    }

    /// C: `uint8_t seq_getExtSync()`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:1518`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_isSyncExternal`.
    pub fn seq_getExtSync(&mut self) -> u8 {
        todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:1518")
    }

    /// C: `void seq_setExtSync(uint8_t isExt)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:1523`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_isSyncExternal`.
    pub fn seq_setExtSync(&mut self, isExt: u8) {
        todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:1523")
    }

    /// C: `void seq_armAutomationStep(uint8_t stepNr, uint8_t track,uint8_t isArmed)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:1528`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_armedArmedAutomationStep`, `seq_armedArmedAutomationTrack`.
    pub fn seq_armAutomationStep(&mut self, stepNr: u8, track: u8, isArmed: u8) {
        todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:1528")
    }

    /// C: `float seq_getElapsedStepTimePercent()`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:1642`. Reference build: inlined into its callers only. Group P3.
    /// Globals: `seq_prescaleCounter`.
    pub fn seq_getElapsedStepTimePercent(&mut self) -> f32 {
        todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:1642")
    }

    /// C: `void seq_tick()`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:1650`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `isStepShuffled`, `midiParser_txRxFilter`, `seq_deltaT`, `seq_isSyncExternal`, `seq_lastShuffle`, `seq_lastTick`, `seq_masterStepCnt`, `seq_prescaleCounter`, `seq_running`, `seq_shuffle`, `seq_tempo`, `sync_delayedStepTimer`, `systick_ticks`, `trigger_clockReceived`.
    /// Calls: `Lxr::seq_MidiSyncQueue`, `Lxr::seq_calcDeltaT`, `Lxr::seq_getBpm`, `Lxr::seq_getDeltaTShuffleDelay`, `Lxr::seq_getExtSync`, `Lxr::seq_getMasterStepCnt`, `Lxr::seq_isRunning`, `Lxr::seq_nextStep`, `Lxr::seq_processFlam`, `Lxr::seq_sendRealtime`, `Lxr::trigger_clockTick`.
    pub fn seq_tick(&mut self) {
        todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:1650")
    }

    /// C: `void seq_setQuantisation(uint8_t value)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:1860`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_quantisation`.
    pub fn seq_setQuantisation(&mut self, value: u8) {
        todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:1860")
    }

    /// C: `void seq_setMainStep(uint8_t patternNr, uint8_t voice, uint8_t stepNr, uint8_t onOff)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:1882`. Reference build: inlined into its callers only. Group P3.
    /// Globals: `seq_patternSet`.
    pub fn seq_setMainStep(&mut self, patternNr: u8, voice: u8, stepNr: u8, onOff: u8) {
        todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:1882")
    }

    /// C: `uint8_t seq_isRunning()`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:1910`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_running`.
    pub fn seq_isRunning(&mut self) -> u8 {
        todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:1910")
    }

    /// C: `void seq_resetPosition()`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:1914`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `midiParser_txRxFilter`, `seq_deltaT`, `seq_lastMasterStep`, `seq_lastShuffle`, `seq_masterStepCnt`, `seq_songPositionPointer`, `seq_stepIndex`.
    /// Calls: `Lxr::led_resetBlinkTimer`, `Lxr::midiParser_checkMtc`, `Lxr::seq_sendRealtime`, `Lxr::seq_setStepIndexToStart`, `Lxr::trigger_reset`, `Lxr::voiceControl_noteOff`.
    pub fn seq_resetPosition(&mut self) {
        todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:1914")
    }

    /// C: `void seq_setRunning(uint8_t isRunning)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:1956`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `midiParser_txRxFilter`, `seq_lastMasterStep`, `seq_prescaleCounter`, `seq_running`, `seq_stepIndex`, `trigger_clockReceived`.
    /// Calls: `Lxr::preset_touchTimer`, `Lxr::seq_sendRealtime`, `Lxr::seq_setStepIndexToStart`, `Lxr::trigger_reset`.
    pub fn seq_setRunning(&mut self, isRunning: u8) {
        todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:1956")
    }

    /// C: `static uint8_t seq_intIsMainStepActive(uint8_t voice, uint8_t mainStepNr, uint8_t pattern)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:2034`. Reference build: inlined into its callers only. Group P3.
    /// Globals: `seq_patternSet`.
    pub fn seq_intIsMainStepActive(&mut self, voice: u8, mainStepNr: u8, pattern: u8) -> u8 {
        todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:2034")
    }

    /// C: `IN_ITCM_RAM void seq_setMute(uint8_t trackNr, uint8_t isMuted)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:2047`. Reference build: out-of-line in ITCM. Group P3.
    /// Globals: `midi_MidiChannels`, `seq_mutedTracks`.
    /// Calls: `Lxr::voiceControl_noteOff`.
    pub fn seq_setMute(&mut self, trackNr: u8, isMuted: u8) {
        todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:2047")
    }

    /// C: `uint8_t seq_isTrackMuted(uint8_t trackNr)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:2067`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_mutedTracks`.
    pub fn seq_isTrackMuted(&mut self, trackNr: u8) -> u8 {
        todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:2067")
    }

    /// C: `void seq_sendStepInfoToFront(uint16_t stepNr)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:2115`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_patternSet`.
    /// Calls: `Lxr::uart_sendFrontpanelSysExByte`.
    pub fn seq_sendStepInfoToFront(&mut self, stepNr: u16) {
        todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:2115")
    }

    /// C: `void seq_setRoll(uint8_t voice, uint8_t onOff)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:2153`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_recordActive`, `seq_rollRate`, `seq_rollState`, `seq_running`.
    /// Calls: `Lxr::seq_addNote`, `Lxr::seq_isRunning`, `Lxr::seq_triggerVoice`.
    pub fn seq_setRoll(&mut self, voice: u8, onOff: u8) {
        todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:2153")
    }

    /// C: `void seq_setRollRate(uint8_t rate)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:2171`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_rollRate`.
    pub fn seq_setRollRate(&mut self, rate: u8) {
        todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:2171")
    }

    /// C: `uint8_t seq_isAutomationStepArmed()`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:2301`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_armedArmedAutomationStep`.
    pub fn seq_isAutomationStepArmed(&mut self) -> u8 {
        todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:2301")
    }

    /// C: `void seq_recordAutomation(uint8_t voice, uint8_t dest, uint8_t value)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:2307`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_activeAutomTrack`, `seq_activePattern`, `seq_armedArmedAutomationStep`, `seq_armedArmedAutomationTrack`, `seq_patternSet`, `seq_recordActive`, `seq_stepIndex`.
    /// Calls: `Lxr::seq_intIsMainStepActive`.
    pub fn seq_recordAutomation(&mut self, voice: u8, dest: u8, value: u8) {
        todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:2307")
    }

    /// C: `void seq_addNote(uint8_t trackNr,uint8_t vel, uint8_t note)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:2364`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `frontParser_activeTrack`, `frontParser_shownPattern`, `menu_activeBar`, `menu_activePage`, `seq_activePattern`, `seq_patternSet`, `seq_prescaleCounter`, `seq_recordActive`, `seq_running`, `seq_stepIndex`, `seq_stepToSkip`.
    /// Calls: `Lxr::led_setValue`, `Lxr::seq_determineNextPattern`, `Lxr::seq_getElapsedStepTimePercent`, `Lxr::seq_setMainStep`.
    pub fn seq_addNote(&mut self, trackNr: u8, vel: u8, note: u8) {
        todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:2364")
    }

    /// C: `void seq_setRecordingMode(uint8_t active)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:2514`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_recordActive`.
    pub fn seq_setRecordingMode(&mut self, active: u8) {
        todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:2514")
    }

    /// C: `void seq_setErasingMode(uint8_t active)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:2519`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_eraseActive`.
    pub fn seq_setErasingMode(&mut self, active: u8) {
        todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:2519")
    }

    /// C: `void seq_clearTrack(uint8_t trackNr, uint8_t pattern)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:2537`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_patternSet`.
    /// Calls: `sequencer::seq_resetNote`.
    pub fn seq_clearTrack(&mut self, trackNr: u8, pattern: u8) {
        todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:2537")
    }

    /// C: `void seq_clearPattern(uint8_t pattern)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:2560`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_patternSet`.
    /// Calls: `Lxr::seq_clearTrack`.
    pub fn seq_clearPattern(&mut self, pattern: u8) {
        todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:2560")
    }

    /// C: `void seq_clearAutomation(uint8_t trackNr, uint8_t pattern, uint8_t automTrack)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:2568`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_patternSet`.
    pub fn seq_clearAutomation(&mut self, trackNr: u8, pattern: u8, automTrack: u8) {
        todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:2568")
    }

    /// C: `void seq_copyTrack(uint8_t srcNr, uint8_t dstNr, uint8_t pattern)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:2588`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_patternSet`.
    pub fn seq_copyTrack(&mut self, srcNr: u8, dstNr: u8, pattern: u8) {
        todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:2588")
    }

    /// C: `void seq_copyPattern(uint8_t src, uint8_t dst)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:2618`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_patternSet`.
    pub fn seq_copyPattern(&mut self, src: u8, dst: u8) {
        todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:2618")
    }

    /// C: `void seq_setActiveAutomationTrack(uint8_t trackNr)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:2652`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `seq_activeAutomTrack`.
    pub fn seq_setActiveAutomationTrack(&mut self, trackNr: u8) {
        todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:2652")
    }

    /// C: `void seq_midiNoteOff(uint8_t chan)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:2679`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `midi_chan_notes`, `midi_notes_on`.
    /// Calls: `Lxr::seq_sendMidi`.
    pub fn seq_midiNoteOff(&mut self, chan: u8) {
        todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:2679")
    }

    /// C: `static void seq_sendRealtime(const uint8_t status)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:2712`. Reference build: inlined into its callers only. Group P3.
    /// Globals: `midiParser_txRxFilter`, `seq_sendRealtime_msg`.
    /// Calls: `Lxr::seq_sendMidi`.
    pub fn seq_sendRealtime(&mut self, status: u8) {
        todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:2712")
    }

    /// C: `void seq_sendMidiNoteOn(const uint8_t channel, const uint8_t note, const uint8_t veloc)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:2725`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `midiParser_txRxFilter`, `midi_chan_notes`, `midi_notes_on`, `seq_sendMidiNoteOn_msg`.
    /// Calls: `Lxr::seq_sendMidi`.
    pub fn seq_sendMidiNoteOn(&mut self, channel: u8, note: u8, veloc: u8) {
        todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:2725")
    }

    /// C: `static void seq_sendProgChg(const uint8_t ptn)`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:2753`. Reference build: inlined into its callers only. Group P3.
    /// Globals: `midiParser_txRxFilter`, `midi_MidiChannels`, `seq_sendProgChg_msg`.
    /// Calls: `Lxr::seq_sendMidi`.
    pub fn seq_sendProgChg(&mut self, ptn: u8) {
        todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:2753")
    }

    /// C: `static void seq_setStepIndexToStart()`
    ///
    /// Source: `Core/Src/Soundengine/Sequencer/sequencer.c:2774`. Reference build: inlined into its callers only. Group P3.
    /// Globals: `seq_lastMasterStep`, `seq_stepIndex`.
    pub fn seq_setStepIndexToStart(&mut self) {
        todo!("port: Core/Src/Soundengine/Sequencer/sequencer.c:2774")
    }
}
