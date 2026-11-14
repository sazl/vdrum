//! `Lxr`: the state of the whole engine. Every C global variable that the ported code uses is a field
//! with the C name (function-local `static` variables are named `<function>_<variable>`). Large buffers are
//! boxed and allocated once in the constructor.
//!
//! Generated from the DWARF and the `.data` / `.dtcm` contents of the reference ELF (`lxr_v1.7_gcc11.3.elf`):
//! `new_zeroed()` is the state right after the firmware's startup code (`.data`/`.dtcm` copied from flash,
//! `.bss`/`.dtcmz` zeroed) and before `main()`.

use crate::prelude::*;

/// `adc_buf[8+i]` for a fader at the top (12 bits, left aligned), `docs/ENGINE_CONTRACT.md` section 2.
pub const FADER_FULL_ADC: u16 = 0xFFF0;

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
    // ---------------- Core/Src/IO/adcPots.c
    /// `float adc_faderValues[6];`
    ///
    /// Declared at `Core/Src/IO/adcPots.c:33`; firmware address 0x20021134 (`.bss`), 24 bytes.
    /// Read by `adc_getFaderVolume` (voice volume faders). Stays 0.0 without ADC input.
    pub adc_faderValues: [f32; 6],
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
    /// `uint16_t frontParser_twoByteData=0;`
    ///
    /// Declared at `Core/Src/Soundengine/MIDI/CortexParser.c:52`; firmware address 0x200219a6 (`.bss`), 2 bytes.
    pub frontParser_twoByteData: u16,
    // ---------------- Core/Src/Soundengine/MIDI/MidiParser.c
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
    // ---------------- Core/Src/Soundengine/Sequencer/sequencer.c (only the tempo and the track mutes
    // are ported; VDrum has no sequencer)
    /// `static uint16_t seq_tempo = 120; / **< seq speed in bpm* /`
    ///
    /// Declared at `Core/Src/Soundengine/Sequencer/sequencer.c:56`; firmware address 0x200202ec (`.data`), 2 bytes.
    pub seq_tempo: u16,
    /// `static uint8_t seq_mutedTracks=0; / **< indicate which tracks are muted * /`
    ///
    /// Declared at `Core/Src/Soundengine/Sequencer/sequencer.c:81`; firmware address 0x200234ce (`.bss`), 1 bytes.
    pub seq_mutedTracks: u8,
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
    /// `volatile MidiMsg frontParser_midiMsg;` (Core/Src/frontPanelParser.c); firmware address 0x2005d598, 8 bytes.
    pub frontParser_midiMsg: MidiMsg,
    /// `static uint16_t frontParser_nrpnNr = 0;` (Core/Src/frontPanelParser.c); firmware address 0x2005d5a2, 2 bytes.
    pub frontParser_nrpnNr: u16,
    /// `static uint8_t frontParser_rxCnt=0;` of Core/Src/frontPanelParser.c (a different variable than
    /// `frontParser_rxCnt` of CortexParser.c); firmware address 0x2005d5a4, 1 bytes.
    pub frontPanelParser_rxCnt: u8,
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
    /// Number of blocks rendered by `tick()` since power-on (drives `systick_ticks`, see `tick`).
    pub blocks: u64,
    /// `adc_buf_old[8..14]` of `Core/Src/IO/adcPots.c` (last accepted fader ADC readings, UI side; used by
    /// `set_fader_adc` for the hysteresis of `adc_checkFaders`).
    pub fader_adc_old: [u16; 6],
    /// VDrum extensions (open hi-hat voice), see `crate::vdrum`. Off after `new()`.
    pub vdrum: crate::vdrum::VdrumState,
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
            adc_faderValues: [0.0; 6],
            velocityModulators: [ModulationNode::ZERO; 6],
            fxBus: FxBus::ZERO,
            cortex_midiMsg: MidiMsg::ZERO,
            frontParser_activeStep: 0,
            frontParser_activeTrack: 0,
            frontParser_rxCnt: 0,
            frontParser_shownPattern: 0,
            frontParser_sysexActive: 0,
            frontParser_sysexBuffer: [0; 7],
            frontParser_twoByteData: 0,
            midiParser_txRxFilter: 255,
            parserState: 4,
            checkMIDIprocPAR: 0,
            checkMIDIprocPAR_t: 0,
            midiMsg_tmp: MidiMsg::ZERO,
            midiParser_activeNrpnNumber: 0,
            midiParser_originalCcValues: [0; 255],
            midiParser_routing: midiParser_routing_t::ZERO,
            midi_MidiChannels: [0; 8],
            active_voices: 0,
            parameterArray: [Parameter::ZERO; 247],
            fifo_midiRx: Fifo::ZERO,
            midi_process_block: 0,
            seq_tempo: 120,
            seq_mutedTracks: 0,
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
            frontParser_midiMsg: MidiMsg::ZERO,
            frontParser_nrpnNr: 0,
            frontPanelParser_rxCnt: 0,
            paramToModTarget: [0; 247],
            menu_activeVoice: 0,
            menu_TargetVoiceGapIndex: 0xFF,
            dmaPtr: 0,
            rng_state: crate::rng::RNG_SEED,
            blocks: 0,
            fader_adc_old: [0; 6],
            vdrum: crate::vdrum::VdrumState::ZERO,
        }
    }

    /// Power-on, exactly as the reference emulator (`docs/ENGINE_CONTRACT.md` section 2): the no-SD-card
    /// path of `main()` (menu_init, soundengine_init (renders one block into buffer half 0),
    /// copyClear_clearCurrentPattern, preset_init(0), preset_loadDefaultGlobals, initMidiUart), the faders
    /// at the top, then the pending kit transfer is completed. Everything runs with FZ = 0. Group P3.
    pub fn new() -> Self {
        let mut s = Lxr::new_zeroed();
        s.menu_init();
        s.soundengine_init();
        // no SD card: main() takes the else branch
        s.copyClear_clearCurrentPattern();
        s.preset_init(0);
        s.preset_loadDefaultGlobals();
        s.initMidiUart();
        // main loop: faders at the top (adc_buf[8+i] = 0xFFF0, adc_checkFaders)
        for i in 0..6 {
            s.set_fader_adc(i, FADER_FULL_ADC);
        }
        // preset_init -> preset_sendDrumsetParameters -> preset_morph started a non-blocking transfer
        s.drain_morph();
        // VDrum extensions: only `s.vdrum` changes (hat split stays off)
        s.vdrum_init();
        s
    }

    /// Runs `preset_morph_tick` until the pending kit/morph transfer is complete (on the hardware this
    /// happens within the next main-loop iterations).
    pub fn drain_morph(&mut self) {
        while self.morph_state != 0 {
            self.preset_morph_tick();
        }
    }

    /// One byte of DIN MIDI input, as `uart_processMidi` handles it (main-loop context, FZ = 0), then the
    /// morph transfer it may have started is completed. Group P3.
    pub fn midi_byte(&mut self, b: u8) {
        self.midiParser_parseUartData(b);
        self.drain_morph();
    }

    /// Several bytes of DIN MIDI input (`midi_byte` for each).
    pub fn midi(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.midi_byte(b);
        }
    }

    /// Renders one block (`docs/ENGINE_CONTRACT.md` section 3): advances `systick_ticks` (TIM14 at 4 kHz,
    /// 16384/2825 ticks per 64-frame block at fs = 44140.625 Hz), runs the SPI3 DMA transfer-complete
    /// interrupt (`HAL_I2S_TxCpltCallback`: `dmaPtr++`, `bCurrentSampleValid = 1 - (dmaPtr & 1)`), then the
    /// main loop's `soundengine_calcNextSampleBlock();` (the firmware's `seq_tick()` before it is removed: VDrum has no sequencer) with FZ = 0. `dac1` is `dma_buffer2`
    /// (the mixer's `output2`, DAC 1 routing), `dac2` is `dma_buffer`. Group P3.
    pub fn tick(&mut self) -> Block {
        self.blocks += 1;
        self.systick_ticks = ((self.blocks * 16384) / 2825) as u32;
        self.HAL_I2S_TxCpltCallback_SPI3();
        let half = self.bCurrentSampleValid as usize;
        assert!(half <= 1, "bCurrentSampleValid = {half} after the DMA callback");
        // The firmware calls `seq_tick()` here; VDrum has no sequencer.
        self.soundengine_calcNextSampleBlock();
        let mut blk = Block::default();
        blk.dac1.copy_from_slice(&self.dma_buffer2[half * 128..half * 128 + 128]);
        blk.dac2.copy_from_slice(&self.dma_buffer[half * 128..half * 128 + 128]);
        blk
    }

    /// C: `HAL_I2S_TxCpltCallback(&hi2s3)` (`Core/Src/Soundengine/cs4344_cs5343.c:86`, SPI3 branch, SWITCH_FLAG = 1).
    /// The `error` counter and `HAL_I2S_Transmit_DMA` are hardware only.
    fn HAL_I2S_TxCpltCallback_SPI3(&mut self) {
        self.dmaPtr = self.dmaPtr.wrapping_add(1);
        self.bCurrentSampleValid = 1u8.wrapping_sub(self.dmaPtr & 1);
    }

    /// Changes a parameter the way the menu does after the encoder chose a new value
    /// (`menu_encoderChangeParameter`, `docs/ENGINE_CONTRACT.md` section 5), then completes the morph
    /// transfer. `value` is not range-limited. Group P3.
    pub fn set_parameter(&mut self, param: u16, value: u8) {
        use crate::menu::cc2_text::{getModTargetGapIndex, getModTargetIdxFromGapIdx, modTargets};
        use crate::menu::{parameter_dtypes, DTYPE_TARGET_SELECTION_LFO, DTYPE_TARGET_SELECTION_VELO, DTYPE_VOICE_LFO};
        use crate::front_panel_parser::{CC_2, CC_LFO_TARGET, CC_VELO_TARGET};
        assert!(param < NUM_PARAMS, "parameter {param} out of range");
        let dtype = (parameter_dtypes[param as usize] & 0xff) as u8;
        // the menu limits the LFO voice number to 1..6 (menu_encoderChangeParameter, menu.c:2899)
        let value = if dtype == DTYPE_VOICE_LFO { value.clamp(1, 6) } else { value };
        self.activeParameters[param as usize] = value;
        // a mod target index past `modTargets` (the menu never produces one) sends target 0
        let mod_param = |idx: u8| modTargets.get(idx as usize).map_or(0, |t| t.param as u8);
        match dtype {
            DTYPE_TARGET_SELECTION_VELO => {
                let voice = param.wrapping_sub(PAR_VEL_DEST_1) as u8;
                let v = mod_param(value);
                let upper = ((v & 0x80) >> 7) | ((voice & 0x3f) << 1);
                self.frontPanel_sendData(CC_VELO_TARGET as u8, upper, v & 0x7f);
            }
            DTYPE_VOICE_LFO => {
                let lfo = param.wrapping_sub(PAR_VOICE_LFO1) as u8;
                let t = getModTargetIdxFromGapIdx(value.wrapping_sub(1), self.menu_TargetVoiceGapIndex);
                self.activeParameters[PAR_TARGET_LFO1 as usize + lfo as usize] = t;
                let v = mod_param(t);
                let upper = ((v & 0x80) >> 7) | ((lfo & 0x3f) << 1);
                self.frontPanel_sendData(CC_LFO_TARGET as u8, upper, v & 0x7f);
            }
            DTYPE_TARGET_SELECTION_LFO => {
                let lfo = param.wrapping_sub(PAR_TARGET_LFO1) as u8;
                self.menu_TargetVoiceGapIndex = getModTargetGapIndex(value);
                let v = mod_param(value);
                let upper = ((v & 0x80) >> 7) | ((lfo & 0x3f) << 1);
                self.frontPanel_sendData(CC_LFO_TARGET as u8, upper, v & 0x7f);
            }
            _ => {}
        }
        if param < 128 {
            self.frontPanel_sendData(MIDI_CC as u8, param as u8, value);
        } else if param < END_OF_SOUND_PARAMETERS {
            self.frontPanel_sendData(CC_2 as u8, (param - 128) as u8, value);
        } else {
            self.menu_parseGlobalParam(param, value);
        }
        self.drain_morph();
    }

    /// Loads a kit file (8 byte name + up to 247 parameter bytes) into kit slot 0 as
    /// `preset_loadDrumsetSD` reads it (missing bytes 0, invalid mod targets 0) and applies it as the LOAD
    /// menu does (`preset_loadDrumsetRam(0, 0, 0)`), then completes the morph transfer
    /// (`docs/ENGINE_CONTRACT.md` section 6). Group P3.
    pub fn load_kit(&mut self, kit: &[u8]) {
        use crate::menu::cc2_text::getNumModTargets;
        assert!((8..=8 + END_OF_SOUND_PARAMETERS as usize).contains(&kit.len()), "kit must be 8..=255 bytes");
        let mut name = [0u8; 8];
        name.copy_from_slice(&kit[..8]);
        if name.iter().all(|&c| c == b' ' || c == 0) {
            name = *b"Empty\0\0\0";
        }
        self.savefileNames[0][0] = name;
        let mut params = [0u8; END_OF_SOUND_PARAMETERS as usize];
        params[..kit.len() - 8].copy_from_slice(&kit[8..]);
        let nmt = getNumModTargets();
        for i in 0..6usize {
            for base in [PAR_VEL_DEST_1, PAR_TARGET_LFO1] {
                let k = base as usize + i;
                if params[k] >= nmt {
                    params[k] = 0;
                }
            }
        }
        self.mainParameters[0] = params;
        self.preset_loadDrumsetRam(0, 0, 0);
        self.drain_morph();
    }

    /// Sets fader `i` (0..6) to a 16 bit ADC reading (12 bits, left aligned) and runs `adc_checkFaders` over
    /// all six faders (`Core/Src/IO/adcPots.c:256`, hysteresis 40 on `adc >> 4`), which updates
    /// `adc_faderValues[i] = v*v` with `v = (adc >> 4) / 4096` (0 below 0.01) when the reading moved.
    ///
    /// VDrum extension: fader 6 is the open hi-hat's own fader (`crate::vdrum`, used with the hat split).
    pub fn set_fader_adc(&mut self, i: usize, adc: u16) {
        assert!(i < 7, "fader index {i}");
        if i == 6 {
            return self.oh_set_fader_adc(adc);
        }
        const HYSTERSIS: i32 = 40;
        let (new, old) = ((adc >> 4) as i32, (self.fader_adc_old[i] >> 4) as i32);
        if new > old + HYSTERSIS || old > new + HYSTERSIS {
            let mut v = new as f32 / 4096.0;
            self.fader_adc_old[i] = adc;
            if v < 0.01 {
                v = 0.0;
            }
            self.adc_faderValues[i] = v * v;
        }
    }
}
