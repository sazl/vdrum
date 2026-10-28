//! `Lxr`: the state of the whole engine. Every C global variable that the ported code uses is a field
//! with the C name (function-local `static` variables are named `<function>_<variable>`). Large buffers are
//! boxed and allocated once in the constructor.
//!
//! Generated from the DWARF and the `.data` / `.dtcm` contents of the reference ELF (`lxr_v1.7_gcc11.3.elf`):
//! `new_zeroed()` is the state right after the firmware's startup code (`.data`/`.dtcm` copied from flash,
//! `.bss`/`.dtcmz` zeroed) and before `main()`.

use crate::prelude::*;

/// One TIM2 interrupt worth of audio: `OUTPUT_DMA_SIZE` = 64 stereo frames per DAC, int16, L/R interleaved.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Block {
    pub dac1: [i16; 128],
    pub dac2: [i16; 128],
}

impl Default for Block {
    fn default() -> Self {
        Block { dac1: [0; 128], dac2: [0; 128] }
    }
}

/// The engine state: one field per C global (same name and C type). See the module documentation.
#[derive(Clone)]
pub struct Lxr {
    // ---------------- Core/Src/Preset/presetManager.c
    /// `static uint8_t morph_index = 0;`
    ///
    /// Declared at `Core/Src/Preset/presetManager.c:42`; firmware address 0x20021165 (`.bss`), 1 bytes.
    pub morph_index: u8,
    /// `static uint8_t morph_state = 0; //flag for non blocking morph`
    ///
    /// Declared at `Core/Src/Preset/presetManager.c:41`; firmware address 0x20021166 (`.bss`), 1 bytes.
    pub morph_state: u8,
    /// `static MidiMsg msg = {0};`
    ///
    /// Declared at `Core/Src/Preset/presetManager.c:2148`; firmware address 0x20021168 (`.bss`), 8 bytes.
    /// Function-local `static` of `preset_morph_tick`.
    pub preset_morph_tick_msg: MidiMsg,
    /// `static MidiMsg msg = {0};`
    ///
    /// Declared at `Core/Src/Preset/presetManager.c:1289`; firmware address 0x20021170 (`.bss`), 8 bytes.
    /// Function-local `static` of `preset_sendDrumsetParametersImmediate`.
    pub preset_sendDrumsetParametersImmediate_msg: MidiMsg,
    /// `uint8_t preset_kitDirty = 0;`
    ///
    /// Declared at `Core/Src/Preset/presetManager.c:84`; firmware address 0x20021178 (`.bss`), 1 bytes.
    pub preset_kitDirty: u8,
    /// `uint32_t preset_kitTimer = 0;`
    ///
    /// Declared at `Core/Src/Preset/presetManager.c:81`; firmware address 0x2002117c (`.bss`), 4 bytes.
    pub preset_kitTimer: u32,
    /// `uint8_t preset_patDirty = 0;`
    ///
    /// Declared at `Core/Src/Preset/presetManager.c:85`; firmware address 0x20021180 (`.bss`), 1 bytes.
    pub preset_patDirty: u8,
    /// `uint32_t preset_patTimer = 0;`
    ///
    /// Declared at `Core/Src/Preset/presetManager.c:82`; firmware address 0x20021184 (`.bss`), 4 bytes.
    pub preset_patTimer: u32,
    /// `extern char savefileNames[NUM_KIT_SLOTS][NUM_SAVEFILE_NAMES][8];`
    ///
    /// Declared at `Core/Inc/Preset/PresetManager.h:37`; firmware address 0x20021188 (`.bss`), 2048 bytes.
    pub savefileNames: [[[u8; 8]; 4]; 64],
    // ---------------- Core/Src/Soundengine/AudioCodecManager.c
    /// `extern INDTCMZ int16_t dma_buffer[OUTPUT_DMA_SIZE*4]; // *4 because we need 16 stereo samples = *2`
    ///
    /// Declared at `Core/Inc/Soundengine/cs4344_cs5343.h:19`; firmware address 0x200090b4 (`.dtcmz`), 512 bytes.
    pub dma_buffer: [i16; 256],
    /// `extern INDTCMZ int16_t dma_buffer2[OUTPUT_DMA_SIZE*4]; // *4 because we need 16 stereo samples = *2`
    ///
    /// Declared at `Core/Inc/Soundengine/cs4344_cs5343.h:20`; firmware address 0x200092b4 (`.dtcmz`), 512 bytes.
    pub dma_buffer2: [i16; 256],
    /// `extern volatile uint8_t bCurrentSampleValid;`
    ///
    /// Declared at `Core/Inc/Soundengine/globals.h:20`; firmware address 0x200094b4 (`.dtcmz`), 1 bytes.
    pub bCurrentSampleValid: u8,
    // ---------------- Core/Src/Soundengine/DSPAudio/CymbalVoice.c
    /// `extern CymbalVoice cymbalVoice;`
    ///
    /// Declared at `Core/Inc/Soundengine/DSPAudio/CymbalVoice.h:73`; firmware address 0x200094b8 (`.dtcmz`), 384 bytes.
    pub cymbalVoice: CymbalVoice,
    // ---------------- Core/Src/Soundengine/DSPAudio/DrumVoice.c
    /// `extern DrumVoice voiceArray[NUM_DRUM_VOICES];`
    ///
    /// Declared at `Core/Inc/Soundengine/DSPAudio/DrumVoice.h:101`; firmware address 0x20009638 (`.dtcmz`), 2112 bytes.
    pub voiceArray: [DrumVoice; 3],
    // ---------------- Core/Src/Soundengine/DSPAudio/FX/FxRamBuffer.c
    /// `extern INDTCMZ __fp16 FX_BUFFER[FX_BUFFER_SIZE+1];`
    ///
    /// Declared at `Core/Inc/Soundengine/DSPAudio/FX/FxRamBuffer.h:25`; firmware address 0x20009e78 (`.dtcmz`), 65536 bytes.
    pub FX_BUFFER: Box<[u16; 32768]>,
    // ---------------- Core/Src/Soundengine/DSPAudio/HiHat.c
    /// `extern HiHatVoice hatVoice;`
    ///
    /// Declared at `Core/Inc/Soundengine/DSPAudio/HiHat.h:78`; firmware address 0x20019e78 (`.dtcmz`), 392 bytes.
    pub hatVoice: HiHatVoice,
    // ---------------- Core/Src/Soundengine/DSPAudio/Snare.c
    /// `extern SnareVoice snareVoice;`
    ///
    /// Declared at `Core/Inc/Soundengine/DSPAudio/Snare.h:82`; firmware address 0x2001a000 (`.dtcmz`), 368 bytes.
    pub snareVoice: SnareVoice,
    // ---------------- Core/Src/Soundengine/DSPAudio/mixer.c
    /// `extern float mixer_decimation_rate[7]; / **<sets the sample rate decimation. 0..1 = full rate* /`
    ///
    /// Declared at `Core/Inc/Soundengine/DSPAudio/mixer.h:25`; firmware address 0x2001a170 (`.dtcmz`), 28 bytes.
    pub mixer_decimation_rate: [f32; 7],
    /// `INDTCMZ float mixer_decimation_cnt[6]; / **<s'n'h counter for decimator* /`
    ///
    /// Declared at `Core/Src/Soundengine/DSPAudio/mixer.c:39`; firmware address 0x2001a18c (`.dtcmz`), 24 bytes.
    pub mixer_decimation_cnt: [f32; 6],
    /// `INDTCMZ int16_t mixer_voice_samples[6]; / **< stores the last outputted sample of the 6 voices* /`
    ///
    /// Declared at `Core/Src/Soundengine/DSPAudio/mixer.c:40`; firmware address 0x2001a1a4 (`.dtcmz`), 12 bytes.
    pub mixer_voice_samples: [i16; 6],
    /// `extern uint8_t mixer_audioRouting[6];`
    ///
    /// Declared at `Core/Inc/Soundengine/DSPAudio/mixer.h:28`; firmware address 0x2001a1b0 (`.dtcmz`), 6 bytes.
    pub mixer_audioRouting: [u8; 6],
    /// `uint8_t l1_Available;`
    ///
    /// Declared at `Core/Src/Soundengine/DSPAudio/mixer.c:29`; firmware address 0x20021988 (`.bss`), 1 bytes.
    pub l1_Available: u8,
    /// `uint8_t l2_Available;`
    ///
    /// Declared at `Core/Src/Soundengine/DSPAudio/mixer.c:30`; firmware address 0x20021989 (`.bss`), 1 bytes.
    pub l2_Available: u8,
    /// `uint8_t r1_Available;`
    ///
    /// Declared at `Core/Src/Soundengine/DSPAudio/mixer.c:31`; firmware address 0x2002198a (`.bss`), 1 bytes.
    pub r1_Available: u8,
    /// `uint8_t r2_Available;`
    ///
    /// Declared at `Core/Src/Soundengine/DSPAudio/mixer.c:28`; firmware address 0x2002198b (`.bss`), 1 bytes.
    pub r2_Available: u8,
    // ---------------- Core/Src/Soundengine/DSPAudio/modulationNode.c
    /// `extern ModulationNode velocityModulators[6];`
    ///
    /// Declared at `Core/Inc/Soundengine/DSPAudio/modulationNode.h:33`; firmware address 0x2001a1b8 (`.dtcmz`), 96 bytes.
    pub velocityModulators: [ModulationNode; 6],
    // ---------------- Core/Src/Soundengine/FxBus.c
    /// `extern INDTCMZ FxBus fxBus;`
    ///
    /// Declared at `Core/Inc/Soundengine/FxBus.h:63`; firmware address 0x2001a218 (`.dtcmz`), 352 bytes.
    pub fxBus: FxBus,
    // ---------------- Core/Src/Soundengine/MIDI/CortexParser.c
    /// `MidiMsg cortex_midiMsg;`
    ///
    /// Declared at `Core/Src/Soundengine/MIDI/CortexParser.c:44`; firmware address 0x2002198c (`.bss`), 8 bytes.
    pub cortex_midiMsg: MidiMsg,
    /// `uint8_t frontParser_activeStep=0;`
    ///
    /// Declared at `Core/Src/Soundengine/MIDI/CortexParser.c:61`; firmware address 0x20021994 (`.bss`), 1 bytes.
    pub frontParser_activeStep: u8,
    /// `extern uint8_t frontParser_activeTrack;`
    ///
    /// Declared at `Core/Inc/Soundengine/MIDI/CortexParser.h:22`; firmware address 0x20021995 (`.bss`), 1 bytes.
    pub frontParser_activeTrack: u8,
    /// `uint8_t frontParser_rxCnt=0;`
    ///
    /// Declared at `Core/Src/Soundengine/MIDI/CortexParser.c:41`; firmware address 0x20021996 (`.bss`), 1 bytes.
    pub frontParser_rxCnt: u8,
    /// `extern uint8_t frontParser_shownPattern;`
    ///
    /// Declared at `Core/Inc/Soundengine/MIDI/CortexParser.h:23`; firmware address 0x20021997 (`.bss`), 1 bytes.
    pub frontParser_shownPattern: u8,
    /// `extern uint8_t frontParser_sysexActive;`
    ///
    /// Declared at `Core/Inc/Soundengine/MIDI/CortexParser.h:24`; firmware address 0x20021998 (`.bss`), 1 bytes.
    pub frontParser_sysexActive: u8,
    /// `static uint8_t frontParser_sysexBuffer[7];`
    ///
    /// Declared at `Core/Src/Soundengine/MIDI/CortexParser.c:54`; firmware address 0x2002199c (`.bss`), 7 bytes.
    pub frontParser_sysexBuffer: [u8; 7],
    /// `uint16_t frontParser_sysexSeqStepNr=0;`
    ///
    /// Declared at `Core/Src/Soundengine/MIDI/CortexParser.c:57`; firmware address 0x200219a4 (`.bss`), 2 bytes.
    pub frontParser_sysexSeqStepNr: u16,
    /// `uint16_t frontParser_twoByteData=0;`
    ///
    /// Declared at `Core/Src/Soundengine/MIDI/CortexParser.c:52`; firmware address 0x200219a6 (`.bss`), 2 bytes.
    pub frontParser_twoByteData: u16,
    // ---------------- Core/Src/Soundengine/MIDI/MidiParser.c
    /// `static uint8_t midiParser_mtcIgnore=1;`
    ///
    /// Declared at `Core/Src/Soundengine/MIDI/MidiParser.c:40`; firmware address 0x200202d3 (`.data`), 1 bytes.
    pub midiParser_mtcIgnore: u8,
    /// `extern uint8_t midiParser_txRxFilter;`
    ///
    /// Declared at `Core/Inc/Soundengine/MIDI/MidiParser.h:76`; firmware address 0x200202d4 (`.data`), 1 bytes.
    pub midiParser_txRxFilter: u8,
    /// `static uint8_t parserState = IGNORE; // state of the parser state machine. Set to what it's expecting next`
    ///
    /// Declared at `Core/Src/Soundengine/MIDI/MidiParser.c:112`; firmware address 0x200202d5 (`.data`), 1 bytes.
    pub parserState: u8,
    /// `volatile static int8_t checkMIDIprocPAR = 0;`
    ///
    /// Declared at `Core/Src/Soundengine/MIDI/MidiParser.c:1960`; firmware address 0x200219a8 (`.bss`), 1 bytes.
    pub checkMIDIprocPAR: i8,
    /// `volatile static uint8_t checkMIDIprocPAR_t = 0;`
    ///
    /// Declared at `Core/Src/Soundengine/MIDI/MidiParser.c:1959`; firmware address 0x200219a9 (`.bss`), 1 bytes.
    pub checkMIDIprocPAR_t: u8,
    /// `MidiMsg midiMsg_tmp; // buffer message where the incoming data is stored`
    ///
    /// Declared at `Core/Src/Soundengine/MIDI/MidiParser.c:109`; firmware address 0x200219ac (`.bss`), 8 bytes.
    pub midiMsg_tmp: MidiMsg,
    /// `static uint16_t midiParser_activeNrpnNumber = 0;`
    ///
    /// Declared at `Core/Src/Soundengine/MIDI/MidiParser.c:35`; firmware address 0x200219b4 (`.bss`), 2 bytes.
    pub midiParser_activeNrpnNumber: u16,
    /// `static uint32_t volatile midiParser_lastMtcReceived=0x0;`
    ///
    /// Declared at `Core/Src/Soundengine/MIDI/MidiParser.c:41`; firmware address 0x200219b8 (`.bss`), 4 bytes.
    pub midiParser_lastMtcReceived: u32,
    /// `static uint8_t midiParser_mtcIsRunning=0;`
    ///
    /// Declared at `Core/Src/Soundengine/MIDI/MidiParser.c:42`; firmware address 0x200219bc (`.bss`), 1 bytes.
    pub midiParser_mtcIsRunning: u8,
    /// `extern uint8_t midiParser_originalCcValues[0xff];`
    ///
    /// Declared at `Core/Inc/Soundengine/MIDI/MidiParser.h:53`; firmware address 0x200219c0 (`.bss`), 255 bytes.
    pub midiParser_originalCcValues: [u8; 255],
    /// `} midiParser_routing = {0};`
    ///
    /// Declared at `Core/Src/Soundengine/MIDI/MidiParser.c:53`; firmware address 0x20021ac0 (`.bss`), 4 bytes.
    pub midiParser_routing: midiParser_routing_t,
    /// `extern uint8_t midi_MidiChannels[8]; // last element is global channel`
    ///
    /// Declared at `Core/Inc/Soundengine/MIDI/MidiParser.h:56`; firmware address 0x20021ac4 (`.bss`), 8 bytes.
    pub midi_MidiChannels: [u8; 8],
    // ---------------- Core/Src/Soundengine/MIDI/MidiVoiceControl.c
    /// `static uint8_t active_voices=0; // which voices are currently playing a note`
    ///
    /// Declared at `Core/Src/Soundengine/MIDI/MidiVoiceControl.c:27`; firmware address 0x20021acc (`.bss`), 1 bytes.
    pub active_voices: u8,
    // ---------------- Core/Src/Soundengine/MIDI/ParameterArray.c
    /// `extern Parameter parameterArray[END_OF_SOUND_PARAMETERS];`
    ///
    /// Declared at `Core/Inc/Soundengine/MIDI/ParameterArray.h:380`; firmware address 0x20021ad0 (`.bss`), 1976 bytes.
    pub parameterArray: [Parameter; 247],
    // ---------------- Core/Src/Soundengine/MIDI/Uart.c
    /// `static Fifo fifo_midiRx;`
    ///
    /// Declared at `Core/Src/Soundengine/MIDI/Uart.c:36`; firmware address 0x200223d0 (`.bss`), 258 bytes.
    pub fifo_midiRx: Fifo,
    /// `extern uint8_t midi_process_block;`
    ///
    /// Declared at `Core/Inc/Soundengine/MIDI/Uart.h:21`; firmware address 0x200225d6 (`.bss`), 1 bytes.
    pub midi_process_block: u8,
    // ---------------- Core/Src/Soundengine/Sequencer/EuklidGenerator.c
    /// `static uint8_t euklid_length[NUM_TRACKS];`
    ///
    /// Declared at `Core/Src/Soundengine/Sequencer/EuklidGenerator.c:21`; firmware address 0x200225d8 (`.bss`), 7 bytes.
    pub euklid_length: [u8; 7],
    /// `static uint16_t euklid_nextCnt1=0;`
    ///
    /// Declared at `Core/Src/Soundengine/Sequencer/EuklidGenerator.c:33`; firmware address 0x200225e0 (`.bss`), 2 bytes.
    pub euklid_nextCnt1: u16,
    /// `static uint16_t euklid_originalLength=0;`
    ///
    /// Declared at `Core/Src/Soundengine/Sequencer/EuklidGenerator.c:34`; firmware address 0x200225e2 (`.bss`), 2 bytes.
    pub euklid_originalLength: u16,
    /// `static ActiveSteps euklid_patternBuffer; / **< 16 bits for maximum 16 steps. 1 is a note 0 is a pause* /`
    ///
    /// Declared at `Core/Src/Soundengine/Sequencer/EuklidGenerator.c:31`; firmware address 0x200225e8 (`.bss`), 8 bytes.
    pub euklid_patternBuffer: ActiveSteps,
    /// `static uint8_t euklid_rotation[NUM_TRACKS];`
    ///
    /// Declared at `Core/Src/Soundengine/Sequencer/EuklidGenerator.c:23`; firmware address 0x200225f0 (`.bss`), 7 bytes.
    pub euklid_rotation: [u8; 7],
    /// `static uint8_t euklid_steps[NUM_TRACKS];`
    ///
    /// Declared at `Core/Src/Soundengine/Sequencer/EuklidGenerator.c:22`; firmware address 0x200225f8 (`.bss`), 7 bytes.
    pub euklid_steps: [u8; 7],
    // ---------------- Core/Src/Soundengine/Sequencer/clockSync.c
    /// `uint8_t sync_clockCnt = 0;`
    ///
    /// Declared at `Core/Src/Soundengine/Sequencer/clockSync.c:23`; firmware address 0x200225ff (`.bss`), 1 bytes.
    pub sync_clockCnt: u8,
    /// `extern uint32_t sync_delayedStepTimer;`
    ///
    /// Declared at `Core/Inc/Soundengine/Sequencer/clockSync.h:43`; firmware address 0x20022600 (`.bss`), 4 bytes.
    pub sync_delayedStepTimer: u32,
    /// `float sync_timeLast = 0;`
    ///
    /// Declared at `Core/Src/Soundengine/Sequencer/clockSync.c:19`; firmware address 0x20022604 (`.bss`), 4 bytes.
    pub sync_timeLast: f32,
    /// `static float sync_window[WINDOW_SIZE];`
    ///
    /// Declared at `Core/Src/Soundengine/Sequencer/clockSync.c:18`; firmware address 0x20022608 (`.bss`), 32 bytes.
    pub sync_window: [f32; 8],
    /// `uint8_t sync_writePos = 0;`
    ///
    /// Declared at `Core/Src/Soundengine/Sequencer/clockSync.c:20`; firmware address 0x20022628 (`.bss`), 1 bytes.
    pub sync_writePos: u8,
    // ---------------- Core/Src/Soundengine/Sequencer/sequencer.c
    /// `static MidiMsg msg = { .bits = { .length = 2 } }; // status + 2 data bytes`
    ///
    /// Declared at `Core/Src/Soundengine/Sequencer/sequencer.c:2727`; firmware address 0x200202d8 (`.data`), 8 bytes.
    /// Function-local `static` of `seq_sendMidiNoteOn`.
    pub seq_sendMidiNoteOn_msg: MidiMsg,
    /// `static MidiMsg msg = { .bits = { .length = 1 } }; // status + 1 data byte`
    ///
    /// Declared at `Core/Src/Soundengine/Sequencer/sequencer.c:2755`; firmware address 0x200202e0 (`.data`), 8 bytes.
    /// Function-local `static` of `seq_sendProgChg`.
    pub seq_sendProgChg_msg: MidiMsg,
    /// `static int8_t seq_armedArmedAutomationStep = -1;`
    ///
    /// Declared at `Core/Src/Soundengine/Sequencer/sequencer.c:78`; firmware address 0x200202e8 (`.data`), 1 bytes.
    pub seq_armedArmedAutomationStep: i8,
    /// `static int8_t seq_armedArmedAutomationTrack = -1;`
    ///
    /// Declared at `Core/Src/Soundengine/Sequencer/sequencer.c:79`; firmware address 0x200202e9 (`.data`), 1 bytes.
    pub seq_armedArmedAutomationTrack: i8,
    /// `uint8_t seq_quantisation = QUANT_16;`
    ///
    /// Declared at `Core/Src/Soundengine/Sequencer/sequencer.c:96`; firmware address 0x200202ea (`.data`), 1 bytes.
    pub seq_quantisation: u8,
    /// `uint8_t seq_rollRate = 1; //start with roll rate = 1/16`
    ///
    /// Declared at `Core/Src/Soundengine/Sequencer/sequencer.c:50`; firmware address 0x200202eb (`.data`), 1 bytes.
    pub seq_rollRate: u8,
    /// `static uint16_t seq_tempo = 120; / **< seq speed in bpm* /`
    ///
    /// Declared at `Core/Src/Soundengine/Sequencer/sequencer.c:56`; firmware address 0x200202ec (`.data`), 2 bytes.
    pub seq_tempo: u16,
    /// `uint8_t chokeCurrent=0;`
    ///
    /// Declared at `Core/Src/Soundengine/Sequencer/sequencer.c:1089`; firmware address 0x20022629 (`.bss`), 1 bytes.
    pub chokeCurrent: u8,
    /// `uint8_t chokeNext=0;`
    ///
    /// Declared at `Core/Src/Soundengine/Sequencer/sequencer.c:1090`; firmware address 0x2002262a (`.bss`), 1 bytes.
    pub chokeNext: u8,
    /// `uint8_t isStepShuffled=0;`
    ///
    /// Declared at `Core/Src/Soundengine/Sequencer/sequencer.c:59`; firmware address 0x2002262b (`.bss`), 1 bytes.
    pub isStepShuffled: u8,
    /// `uint8_t lastWasNoteOn[NUM_TRACKS];`
    ///
    /// Declared at `Core/Src/Soundengine/Sequencer/sequencer.c:174`; firmware address 0x2002262c (`.bss`), 7 bytes.
    pub lastWasNoteOn: [u8; 7],
    /// `static uint8_t midi_chan_notes[16]; / **< what note is playing on each channel * /`
    ///
    /// Declared at `Core/Src/Soundengine/Sequencer/sequencer.c:116`; firmware address 0x20022634 (`.bss`), 16 bytes.
    pub midi_chan_notes: [u8; 16],
    /// `static uint16_t midi_notes_on=0; / **< which channels have a note currently playing * /`
    ///
    /// Declared at `Core/Src/Soundengine/Sequencer/sequencer.c:117`; firmware address 0x20022644 (`.bss`), 2 bytes.
    pub midi_notes_on: u16,
    /// `static MidiMsg msg = {0};`
    ///
    /// Declared at `Core/Src/Soundengine/Sequencer/sequencer.c:2714`; firmware address 0x20022648 (`.bss`), 8 bytes.
    /// Function-local `static` of `seq_sendRealtime`.
    pub seq_sendRealtime_msg: MidiMsg,
    /// `extern PatternBackupBuffer patternBackupBuffer;`
    ///
    /// Declared at `Core/Inc/Soundengine/Sequencer/sequencer.h:149`; firmware address 0x20022650 (`.bss`), 3650 bytes.
    pub patternBackupBuffer: PatternBackupBuffer,
    /// `uint8_t seq_activeAutomTrack=0;`
    ///
    /// Declared at `Core/Src/Soundengine/Sequencer/sequencer.c:63`; firmware address 0x20023492 (`.bss`), 1 bytes.
    pub seq_activeAutomTrack: u8,
    /// `extern uint8_t seq_activePattern;`
    ///
    /// Declared at `Core/Inc/Soundengine/Sequencer/sequencer.h:128`; firmware address 0x20023493 (`.bss`), 1 bytes.
    pub seq_activePattern: u8,
    /// `extern uint8_t *seq_activeSong;`
    ///
    /// Declared at `Core/Inc/Soundengine/Sequencer/sequencer.h:159`; firmware address 0x20023494 (`.bss`), 4 bytes.
    /// C: `uint8_t *seq_activeSong`. Representation: `Some(i)` = `seq_songData[i]` (the only values the firmware assigns), `None` = NULL (power-on value, set by `seq_init`).
    pub seq_activeSong: Option<usize>,
    /// `static AutomationNode seq_automationNodes[NUM_TRACKS][2];`
    ///
    /// Declared at `Core/Src/Soundengine/Sequencer/sequencer.c:157`; firmware address 0x20023498 (`.bss`), 28 bytes.
    pub seq_automationNodes: [[AutomationNode; 2]; 7],
    /// `static float seq_deltaT; / **< time in [ms] until the next step`
    ///
    /// Declared at `Core/Src/Soundengine/Sequencer/sequencer.c:60`; firmware address 0x200234b4 (`.bss`), 4 bytes.
    pub seq_deltaT: f32,
    /// `uint8_t seq_eraseActive=0; / **RECORD will be 1 if live erasing the active voice * /`
    ///
    /// Declared at `Core/Src/Soundengine/Sequencer/sequencer.c:94`; firmware address 0x200234b8 (`.bss`), 1 bytes.
    pub seq_eraseActive: u8,
    /// `uint8_t seq_isSyncExternal = 0;`
    ///
    /// Declared at `Core/Src/Soundengine/Sequencer/sequencer.c:68`; firmware address 0x200234b9 (`.bss`), 1 bytes.
    pub seq_isSyncExternal: u8,
    /// `uint8_t seq_lastMasterStep[NUM_TRACKS]; //keeps track of the last triggered master sync step of each track`
    ///
    /// Declared at `Core/Src/Soundengine/Sequencer/sequencer.c:69`; firmware address 0x200234bc (`.bss`), 7 bytes.
    pub seq_lastMasterStep: [u8; 7],
    /// `float seq_lastShuffle = 0;`
    ///
    /// Declared at `Core/Src/Soundengine/Sequencer/sequencer.c:139`; firmware address 0x200234c4 (`.bss`), 4 bytes.
    pub seq_lastShuffle: f32,
    /// `static uint32_t seq_lastTick = 0; / **< stores the time the last step change occured* /`
    ///
    /// Declared at `Core/Src/Soundengine/Sequencer/sequencer.c:58`; firmware address 0x200234c8 (`.bss`), 4 bytes.
    pub seq_lastTick: u32,
    /// `static uint8_t seq_loadPendigFlag = 0;`
    ///
    /// Declared at `Core/Src/Soundengine/Sequencer/sequencer.c:109`; firmware address 0x200234cc (`.bss`), 1 bytes.
    pub seq_loadPendigFlag: u8,
    /// `uint8_t seq_masterStepCnt=0; / ** keeps track of the played steps between 0 and 127 independent from the track counters* /`
    ///
    /// Declared at `Core/Src/Soundengine/Sequencer/sequencer.c:49`; firmware address 0x200234cd (`.bss`), 1 bytes.
    pub seq_masterStepCnt: u8,
    /// `static uint8_t seq_mutedTracks=0; / **< indicate which tracks are muted * /`
    ///
    /// Declared at `Core/Src/Soundengine/Sequencer/sequencer.c:81`; firmware address 0x200234ce (`.bss`), 1 bytes.
    pub seq_mutedTracks: u8,
    /// `extern PatternSet seq_patternSet;`
    ///
    /// Declared at `Core/Inc/Soundengine/Sequencer/sequencer.h:143`; firmware address 0x200234d0 (`.bss`), 233536 bytes.
    pub seq_patternSet: Box<PatternSet>,
    /// `extern uint8_t seq_pendingPattern;`
    ///
    /// Declared at `Core/Inc/Soundengine/Sequencer/sequencer.h:129`; firmware address 0x2005c510 (`.bss`), 1 bytes.
    pub seq_pendingPattern: u8,
    /// `extern uint8_t seq_prescaleCounter;`
    ///
    /// Declared at `Core/Inc/Soundengine/Sequencer/sequencer.h:131`; firmware address 0x2005c511 (`.bss`), 1 bytes.
    pub seq_prescaleCounter: u8,
    /// `uint8_t seq_recordActive = 0; / **< set to 1 to activate the reording mode* /`
    ///
    /// Declared at `Core/Src/Soundengine/Sequencer/sequencer.c:92`; firmware address 0x2005c512 (`.bss`), 1 bytes.
    pub seq_recordActive: u8,
    /// `extern uint8_t seq_resetBarOnPatternChange;`
    ///
    /// Declared at `Core/Inc/Soundengine/Sequencer/sequencer.h:166`; firmware address 0x2005c513 (`.bss`), 1 bytes.
    pub seq_resetBarOnPatternChange: u8,
    /// `}seq_rndValue[NUM_TRACKS];`
    ///
    /// Declared at `Core/Src/Soundengine/Sequencer/sequencer.c:104`; firmware address 0x2005c514 (`.bss`), 21 bytes.
    pub seq_rndValue: [seq_rndValue_t; 7],
    /// `uint8_t seq_rollState = 0; / **< each bit represents a voice. if bit is set, roll is active* /`
    ///
    /// Declared at `Core/Src/Soundengine/Sequencer/sequencer.c:51`; firmware address 0x2005c529 (`.bss`), 1 bytes.
    pub seq_rollState: u8,
    /// `uint8_t seq_running = 0; / **< 1 running, 0 if stopped* /`
    ///
    /// Declared at `Core/Src/Soundengine/Sequencer/sequencer.c:83`; firmware address 0x2005c52a (`.bss`), 1 bytes.
    pub seq_running: u8,
    /// `extern uint8_t seq_selectedStep;`
    ///
    /// Declared at `Core/Inc/Soundengine/Sequencer/sequencer.h:164`; firmware address 0x2005c52b (`.bss`), 1 bytes.
    pub seq_selectedStep: u8,
    /// `float seq_shuffle = 0;`
    ///
    /// Declared at `Core/Src/Soundengine/Sequencer/sequencer.c:72`; firmware address 0x2005c52c (`.bss`), 4 bytes.
    pub seq_shuffle: f32,
    /// `extern uint8_t seq_songData[NUM_KIT_SLOTS][MAX_SONG_LENGTH];`
    ///
    /// Declared at `Core/Inc/Soundengine/Sequencer/sequencer.h:158`; firmware address 0x2005c530 (`.bss`), 4096 bytes.
    pub seq_songData: Box<[[u8; 64]; 64]>,
    /// `uint8_t seq_songModeEnabled=0;`
    ///
    /// Declared at `Core/Src/Soundengine/Sequencer/sequencer.c:82`; firmware address 0x2005d530 (`.bss`), 1 bytes.
    pub seq_songModeEnabled: u8,
    /// `extern uint8_t seq_songPositionPointer;`
    ///
    /// Declared at `Core/Inc/Soundengine/Sequencer/sequencer.h:160`; firmware address 0x2005d531 (`.bss`), 1 bytes.
    pub seq_songPositionPointer: u8,
    /// `static int8_t seq_stepIndex[NUM_TRACKS]; / **< we have 16 steps con 4 bars = 64 steps.`
    ///
    /// Declared at `Core/Src/Soundengine/Sequencer/sequencer.c:53`; firmware address 0x2005d534 (`.bss`), 7 bytes.
    pub seq_stepIndex: [i8; 7],
    /// `int8_t seq_stepToSkip[NUM_TRACKS];`
    ///
    /// Declared at `Core/Src/Soundengine/Sequencer/sequencer.c:74`; firmware address 0x2005d53c (`.bss`), 7 bytes.
    pub seq_stepToSkip: [i8; 7],
    // ---------------- Core/Src/Soundengine/TriggerOut.c
    /// `extern uint8_t trigger_dividerClockOut1;`
    ///
    /// Declared at `Core/Inc/Soundengine/TriggerOut.h:92`; firmware address 0x200202ee (`.data`), 1 bytes.
    pub trigger_dividerClockOut1: u8,
    /// `extern uint8_t trigger_prescalerClockInput;`
    ///
    /// Declared at `Core/Inc/Soundengine/TriggerOut.h:94`; firmware address 0x200202f0 (`.data`), 1 bytes.
    pub trigger_prescalerClockInput: u8,
    /// `extern uint8_t trigger_clockReceived;`
    ///
    /// Declared at `Core/Inc/Soundengine/TriggerOut.h:43`; firmware address 0x2005d565 (`.bss`), 1 bytes.
    pub trigger_clockReceived: u8,
    // ---------------- Core/Src/menu/menu.c
    /// `extern uint8_t activeParameters[NUM_PARAMS]; // keeps the currently used kit in memory -> editBuffer`
    ///
    /// Declared at `Core/Inc/menu/menu.h:616`; firmware address 0x2005dfbc (`.bss`), 302 bytes.
    pub activeParameters: [u8; 302],
    /// `extern uint8_t mainParameters[NUM_KIT_SLOTS][END_OF_SOUND_PARAMETERS]; // keeps all kits from sd card in memory`
    ///
    /// Declared at `Core/Inc/menu/menu.h:615`; firmware address 0x2005e134 (`.bss`), 15808 bytes.
    pub mainParameters: Box<[[u8; 247]; 64]>,
    /// `extern uint8_t menu_activeBar;`
    ///
    /// Declared at `Core/Inc/menu/menu.h:51`; firmware address 0x20061ef5 (`.bss`), 1 bytes.
    pub menu_activeBar: u8,
    /// `extern uint8_t menu_activePage;`
    ///
    /// Declared at `Core/Inc/menu/menu.h:45`; firmware address 0x20061ef6 (`.bss`), 1 bytes.
    pub menu_activePage: u8,
    /// `extern uint8_t menu_loadedPresetNr[NUM_CURRENT];`
    ///
    /// Declared at `Core/Inc/menu/menu.h:66`; firmware address 0x20061efc (`.bss`), 4 bytes.
    pub menu_loadedPresetNr: [u8; 4],
    /// `extern uint8_t menu_playedPattern;`
    ///
    /// Declared at `Core/Inc/menu/menu.h:47`; firmware address 0x20061f02 (`.bss`), 1 bytes.
    pub menu_playedPattern: u8,
    /// `extern uint8_t menu_selectedPresetNr[NUM_PRESET_LOCATIONS];`
    ///
    /// Declared at `Core/Inc/menu/menu.h:55`; firmware address 0x20061f08 (`.bss`), 6 bytes.
    pub menu_selectedPresetNr: [u8; 6],
    /// `extern uint8_t menu_shownPattern;`
    ///
    /// Declared at `Core/Inc/menu/menu.h:49`; firmware address 0x20061f0e (`.bss`), 1 bytes.
    pub menu_shownPattern: u8,
    /// `extern uint8_t menu_songMenuPosition; //used to show the list in the display and scroll through it`
    ///
    /// Declared at `Core/Inc/menu/menu.h:52`; firmware address 0x20061f0f (`.bss`), 1 bytes.
    pub menu_songMenuPosition: u8,
    /// `extern uint8_t *morphParameters;`
    ///
    /// Declared at `Core/Inc/menu/menu.h:618`; firmware address 0x20061f10 (`.bss`), 4 bytes.
    /// C: `uint8_t* morphParameters`. Representation: `Some(i)` = `mainParameters[i]` (the only values the firmware assigns), `None` = NULL (power-on value; the firmware never assigns it without an SD card, so `preset_morph_tick` then reads through a NULL pointer, i.e. ITCM code bytes at 0x00000000 + index).
    pub morphParameters: Option<usize>,
    // ---------------- Core/Src/stm32f7xx_it.c
    /// `extern INDTCMZ volatile uint32_t systick_ticks;`
    ///
    /// Declared at `Core/Inc/Soundengine/globals.h:18`; firmware address 0x2001a378 (`.dtcmz`), 4 bytes.
    pub systick_ticks: u32,
    // ---------------- P3 additions: globals of UI-side files that engine code reaches
    // (`uart_sendFrontpanelByte` -> `frontPanel_parseData`, `menu_init`, `HAL_I2S_TxCpltCallback`)
    /// `uint8_t frontPanel_sysexMode = 0;` (Core/Src/frontPanelParser.c); firmware address 0x2005d594, 1 bytes.
    pub frontPanel_sysexMode: u8,
    /// `volatile MidiMsg frontParser_midiMsg;` (Core/Src/frontPanelParser.c); firmware address 0x2005d598, 8 bytes.
    pub frontParser_midiMsg: MidiMsg,
    /// `volatile uint8_t frontParser_newSeqDataAvailable = 0;` (Core/Src/frontPanelParser.c); 0x2005d5a0, 1 bytes.
    pub frontParser_newSeqDataAvailable: u8,
    /// `static uint16_t frontParser_nrpnNr = 0;` (Core/Src/frontPanelParser.c); firmware address 0x2005d5a2, 2 bytes.
    pub frontParser_nrpnNr: u16,
    /// `static uint8_t frontParser_rxCnt=0;` of Core/Src/frontPanelParser.c (a different variable than
    /// `frontParser_rxCnt` of CortexParser.c); firmware address 0x2005d5a4, 1 bytes.
    pub frontPanelParser_rxCnt: u8,
    /// `volatile Step frontParser_stepData;` (Core/Src/frontPanelParser.c); firmware address 0x2005d5a8, 7 bytes.
    pub frontParser_stepData: Step,
    /// `volatile static uint8_t frontParser_sysexBuffer[7];` of Core/Src/frontPanelParser.c (a different
    /// variable than `frontParser_sysexBuffer` of CortexParser.c); firmware address 0x2005d5b0, 7 bytes.
    pub frontPanelParser_sysexBuffer: [u8; 7],
    /// `uint8_t paramToModTarget[END_OF_SOUND_PARAMETERS];` (Core/Src/menu/Cc2Text.c); 0x2005dec0, 247 bytes.
    pub paramToModTarget: [u8; 247],
    /// `uint8_t menu_activeVoice = 0;` (Core/Src/menu/menu.c); firmware address 0x20061ef8, 1 bytes.
    pub menu_activeVoice: u8,
    /// `static uint8_t menu_TargetVoiceGapIndex = 0xFF;` (Core/Src/menu/menu.c); firmware address 0x200202f7, 1 bytes.
    pub menu_TargetVoiceGapIndex: u8,
    /// `uint8_t dmaPtr;` (Core/Src/Soundengine/cs4344_cs5343.c); firmware address 0x2005d56c, 1 bytes.
    pub dmaPtr: u8,
    // ---------------- not C globals
    /// State of the deterministic replacement of the hardware RNG (`GetRngValue`, see `crate::rng`).
    pub rng_state: u32,
}

impl Lxr {
    /// The state after the firmware's startup code: every global at its static initial value (generated from the
    /// reference ELF), the RNG seeded with `crate::rng::RNG_SEED`. Nothing of `main()` has run yet.
    pub fn new_zeroed() -> Self {
        Lxr {
            morph_index: 0,
            morph_state: 0,
            preset_morph_tick_msg: MidiMsg::ZERO,
            preset_sendDrumsetParametersImmediate_msg: MidiMsg::ZERO,
            preset_kitDirty: 0,
            preset_kitTimer: 0,
            preset_patDirty: 0,
            preset_patTimer: 0,
            savefileNames: [[[0; 8]; 4]; 64],
            dma_buffer: [0; 256],
            dma_buffer2: [0; 256],
            bCurrentSampleValid: 0,
            cymbalVoice: CymbalVoice::ZERO,
            voiceArray: [DrumVoice::ZERO; 3],
            FX_BUFFER: Box::new([0; 32768]),
            hatVoice: HiHatVoice::ZERO,
            snareVoice: SnareVoice::ZERO,
            mixer_decimation_rate: [0.0; 7],
            mixer_decimation_cnt: [0.0; 6],
            mixer_voice_samples: [0; 6],
            mixer_audioRouting: [0; 6],
            l1_Available: 0,
            l2_Available: 0,
            r1_Available: 0,
            r2_Available: 0,
            velocityModulators: [ModulationNode::ZERO; 6],
            fxBus: FxBus::ZERO,
            cortex_midiMsg: MidiMsg::ZERO,
            frontParser_activeStep: 0,
            frontParser_activeTrack: 0,
            frontParser_rxCnt: 0,
            frontParser_shownPattern: 0,
            frontParser_sysexActive: 0,
            frontParser_sysexBuffer: [0; 7],
            frontParser_sysexSeqStepNr: 0,
            frontParser_twoByteData: 0,
            midiParser_mtcIgnore: 1,
            midiParser_txRxFilter: 255,
            parserState: 4,
            checkMIDIprocPAR: 0,
            checkMIDIprocPAR_t: 0,
            midiMsg_tmp: MidiMsg::ZERO,
            midiParser_activeNrpnNumber: 0,
            midiParser_lastMtcReceived: 0,
            midiParser_mtcIsRunning: 0,
            midiParser_originalCcValues: [0; 255],
            midiParser_routing: midiParser_routing_t::ZERO,
            midi_MidiChannels: [0; 8],
            active_voices: 0,
            parameterArray: [Parameter::ZERO; 247],
            fifo_midiRx: Fifo::ZERO,
            midi_process_block: 0,
            euklid_length: [0; 7],
            euklid_nextCnt1: 0,
            euklid_originalLength: 0,
            euklid_patternBuffer: ActiveSteps::ZERO,
            euklid_rotation: [0; 7],
            euklid_steps: [0; 7],
            sync_clockCnt: 0,
            sync_delayedStepTimer: 0,
            sync_timeLast: 0.0,
            sync_window: [0.0; 8],
            sync_writePos: 0,
            seq_sendMidiNoteOn_msg: MidiMsg { status: 0, lastNonSysStatus: 0, data1: 0, data2: 0, bits: MidiBits { bits: 0x8 } },
            seq_sendProgChg_msg: MidiMsg { status: 0, lastNonSysStatus: 0, data1: 0, data2: 0, bits: MidiBits { bits: 0x4 } },
            seq_armedArmedAutomationStep: -1,
            seq_armedArmedAutomationTrack: -1,
            seq_quantisation: 2,
            seq_rollRate: 1,
            seq_tempo: 120,
            chokeCurrent: 0,
            chokeNext: 0,
            isStepShuffled: 0,
            lastWasNoteOn: [0; 7],
            midi_chan_notes: [0; 16],
            midi_notes_on: 0,
            seq_sendRealtime_msg: MidiMsg::ZERO,
            patternBackupBuffer: PatternBackupBuffer::ZERO,
            seq_activeAutomTrack: 0,
            seq_activePattern: 0,
            seq_activeSong: None,
            seq_automationNodes: [[AutomationNode::ZERO; 2]; 7],
            seq_deltaT: 0.0,
            seq_eraseActive: 0,
            seq_isSyncExternal: 0,
            seq_lastMasterStep: [0; 7],
            seq_lastShuffle: 0.0,
            seq_lastTick: 0,
            seq_loadPendigFlag: 0,
            seq_masterStepCnt: 0,
            seq_mutedTracks: 0,
            seq_patternSet: Box::new(PatternSet::ZERO),
            seq_pendingPattern: 0,
            seq_prescaleCounter: 0,
            seq_recordActive: 0,
            seq_resetBarOnPatternChange: 0,
            seq_rndValue: [seq_rndValue_t::ZERO; 7],
            seq_rollState: 0,
            seq_running: 0,
            seq_selectedStep: 0,
            seq_shuffle: 0.0,
            seq_songData: Box::new([[0; 64]; 64]),
            seq_songModeEnabled: 0,
            seq_songPositionPointer: 0,
            seq_stepIndex: [0; 7],
            seq_stepToSkip: [0; 7],
            trigger_dividerClockOut1: 1,
            trigger_prescalerClockInput: 1,
            trigger_clockReceived: 0,
            activeParameters: [0; 302],
            mainParameters: Box::new([[0; 247]; 64]),
            menu_activeBar: 0,
            menu_activePage: 0,
            menu_loadedPresetNr: [0; 4],
            menu_playedPattern: 0,
            menu_selectedPresetNr: [0; 6],
            menu_shownPattern: 0,
            menu_songMenuPosition: 0,
            morphParameters: None,
            systick_ticks: 0,
            frontPanel_sysexMode: 0,
            frontParser_midiMsg: MidiMsg::ZERO,
            frontParser_newSeqDataAvailable: 0,
            frontParser_nrpnNr: 0,
            frontPanelParser_rxCnt: 0,
            frontParser_stepData: Step::ZERO,
            frontPanelParser_sysexBuffer: [0; 7],
            paramToModTarget: [0; 247],
            menu_activeVoice: 0,
            menu_TargetVoiceGapIndex: 0xFF,
            dmaPtr: 0,
            rng_state: crate::rng::RNG_SEED,
        }
    }

    /// Power-on: reproduces the firmware start without SD card (see `docs/ENGINE_CONTRACT.md`), then loads
    /// the init kit. Group P3.
    pub fn new() -> Self {
        todo!("port: Core/Src/main.c main() without SD card, then load the init kit")
    }

    /// Feeds one byte of DIN MIDI through the firmware parser (main-loop context, FZ = 0). Group P3.
    pub fn midi_byte(&mut self, b: u8) {
        todo!("port: Core/Src/Soundengine/MIDI/Uart.c uart_processMidi -> midiParser_parseUartData")
    }

    /// Runs one TIM2 interrupt: `seq_tick(); soundengine_calcNextSampleBlock();` with FZ = 1 (use
    /// `crate::fpu::FtzGuard`). Group P3.
    pub fn tick(&mut self) -> Block {
        todo!("port: Core/Src/Soundengine/Soundengine.c soundengine_process (interrupt part)")
    }
}
