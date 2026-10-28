//! C pointers and the firmware memory map.
//!
//! Struct fields of C pointer type are stored as `CPtr`: the address the pointer would hold in the reference
//! firmware (`lxr_v1.7_gcc11.3.elf`), 0 = NULL. This keeps NULL checks, pointer comparisons and the values
//! themselves identical to the firmware (the emulator can compare them directly). Dereferencing goes through
//! `Lxr::cptr_*`, which map an address back to the `Lxr` field (implement with the address and offset tables
//! below).
//!
//! Generated from the DWARF of the reference ELF: `ADDR_<global>` are the addresses of the C globals that are
//! fields of `Lxr`, `size_of_c` / `offset_of_c` the C sizes and member offsets of every C struct.

use crate::prelude::*;

/// A C pointer stored as its address in the reference firmware's memory map (0 = NULL).
pub type CPtr = u32;

/// `dma_buffer` (512 bytes)
pub const ADDR_dma_buffer: CPtr = 0x200090b4;
/// `dma_buffer2` (512 bytes)
pub const ADDR_dma_buffer2: CPtr = 0x200092b4;
/// `bCurrentSampleValid` (1 bytes)
pub const ADDR_bCurrentSampleValid: CPtr = 0x200094b4;
/// `cymbalVoice` (384 bytes)
pub const ADDR_cymbalVoice: CPtr = 0x200094b8;
/// `voiceArray` (2112 bytes)
pub const ADDR_voiceArray: CPtr = 0x20009638;
/// `FX_BUFFER` (65536 bytes)
pub const ADDR_FX_BUFFER: CPtr = 0x20009e78;
/// `hatVoice` (392 bytes)
pub const ADDR_hatVoice: CPtr = 0x20019e78;
/// `snareVoice` (368 bytes)
pub const ADDR_snareVoice: CPtr = 0x2001a000;
/// `mixer_decimation_rate` (28 bytes)
pub const ADDR_mixer_decimation_rate: CPtr = 0x2001a170;
/// `mixer_decimation_cnt` (24 bytes)
pub const ADDR_mixer_decimation_cnt: CPtr = 0x2001a18c;
/// `mixer_voice_samples` (12 bytes)
pub const ADDR_mixer_voice_samples: CPtr = 0x2001a1a4;
/// `mixer_audioRouting` (6 bytes)
pub const ADDR_mixer_audioRouting: CPtr = 0x2001a1b0;
/// `velocityModulators` (96 bytes)
pub const ADDR_velocityModulators: CPtr = 0x2001a1b8;
/// `fxBus` (352 bytes)
pub const ADDR_fxBus: CPtr = 0x2001a218;
/// `systick_ticks` (4 bytes)
pub const ADDR_systick_ticks: CPtr = 0x2001a378;
// P3 additions (globals of UI-side files reached by engine code)
/// `frontPanel_sysexMode` (1 bytes)
pub const ADDR_frontPanel_sysexMode: CPtr = 0x2005d594;
/// `frontParser_midiMsg` (8 bytes)
pub const ADDR_frontParser_midiMsg: CPtr = 0x2005d598;
/// `frontParser_newSeqDataAvailable` (1 bytes)
pub const ADDR_frontParser_newSeqDataAvailable: CPtr = 0x2005d5a0;
/// `frontParser_nrpnNr` (2 bytes)
pub const ADDR_frontParser_nrpnNr: CPtr = 0x2005d5a2;
/// `frontParser_rxCnt` of frontPanelParser.c (1 bytes)
pub const ADDR_frontPanelParser_rxCnt: CPtr = 0x2005d5a4;
/// `frontParser_stepData` (7 bytes)
pub const ADDR_frontParser_stepData: CPtr = 0x2005d5a8;
/// `frontParser_sysexBuffer` of frontPanelParser.c (7 bytes)
pub const ADDR_frontPanelParser_sysexBuffer: CPtr = 0x2005d5b0;
/// `paramToModTarget` (247 bytes)
pub const ADDR_paramToModTarget: CPtr = 0x2005dec0;
/// `menu_activeVoice` (1 bytes)
pub const ADDR_menu_activeVoice: CPtr = 0x20061ef8;
/// `menu_TargetVoiceGapIndex` (1 bytes)
pub const ADDR_menu_TargetVoiceGapIndex: CPtr = 0x200202f7;
/// `dmaPtr` (1 bytes)
pub const ADDR_dmaPtr: CPtr = 0x2005d56c;
/// `midiParser_mtcIgnore` (1 bytes)
pub const ADDR_midiParser_mtcIgnore: CPtr = 0x200202d3;
/// `midiParser_txRxFilter` (1 bytes)
pub const ADDR_midiParser_txRxFilter: CPtr = 0x200202d4;
/// `parserState` (1 bytes)
pub const ADDR_parserState: CPtr = 0x200202d5;
/// `seq_sendMidiNoteOn_msg` (8 bytes)
pub const ADDR_seq_sendMidiNoteOn_msg: CPtr = 0x200202d8;
/// `seq_sendProgChg_msg` (8 bytes)
pub const ADDR_seq_sendProgChg_msg: CPtr = 0x200202e0;
/// `seq_armedArmedAutomationStep` (1 bytes)
pub const ADDR_seq_armedArmedAutomationStep: CPtr = 0x200202e8;
/// `seq_armedArmedAutomationTrack` (1 bytes)
pub const ADDR_seq_armedArmedAutomationTrack: CPtr = 0x200202e9;
/// `seq_quantisation` (1 bytes)
pub const ADDR_seq_quantisation: CPtr = 0x200202ea;
/// `seq_rollRate` (1 bytes)
pub const ADDR_seq_rollRate: CPtr = 0x200202eb;
/// `seq_tempo` (2 bytes)
pub const ADDR_seq_tempo: CPtr = 0x200202ec;
/// `trigger_dividerClockOut1` (1 bytes)
pub const ADDR_trigger_dividerClockOut1: CPtr = 0x200202ee;
/// `trigger_prescalerClockInput` (1 bytes)
pub const ADDR_trigger_prescalerClockInput: CPtr = 0x200202f0;
/// `morph_index` (1 bytes)
pub const ADDR_morph_index: CPtr = 0x20021165;
/// `morph_state` (1 bytes)
pub const ADDR_morph_state: CPtr = 0x20021166;
/// `preset_morph_tick_msg` (8 bytes)
pub const ADDR_preset_morph_tick_msg: CPtr = 0x20021168;
/// `preset_sendDrumsetParametersImmediate_msg` (8 bytes)
pub const ADDR_preset_sendDrumsetParametersImmediate_msg: CPtr = 0x20021170;
/// `preset_kitDirty` (1 bytes)
pub const ADDR_preset_kitDirty: CPtr = 0x20021178;
/// `preset_kitTimer` (4 bytes)
pub const ADDR_preset_kitTimer: CPtr = 0x2002117c;
/// `preset_patDirty` (1 bytes)
pub const ADDR_preset_patDirty: CPtr = 0x20021180;
/// `preset_patTimer` (4 bytes)
pub const ADDR_preset_patTimer: CPtr = 0x20021184;
/// `savefileNames` (2048 bytes)
pub const ADDR_savefileNames: CPtr = 0x20021188;
/// `l1_Available` (1 bytes)
pub const ADDR_l1_Available: CPtr = 0x20021988;
/// `l2_Available` (1 bytes)
pub const ADDR_l2_Available: CPtr = 0x20021989;
/// `r1_Available` (1 bytes)
pub const ADDR_r1_Available: CPtr = 0x2002198a;
/// `r2_Available` (1 bytes)
pub const ADDR_r2_Available: CPtr = 0x2002198b;
/// `adc_faderValues` (24 bytes)
pub const ADDR_adc_faderValues: CPtr = 0x20021134;
/// `cortex_midiMsg` (8 bytes)
pub const ADDR_cortex_midiMsg: CPtr = 0x2002198c;
/// `frontParser_activeStep` (1 bytes)
pub const ADDR_frontParser_activeStep: CPtr = 0x20021994;
/// `frontParser_activeTrack` (1 bytes)
pub const ADDR_frontParser_activeTrack: CPtr = 0x20021995;
/// `frontParser_rxCnt` (1 bytes)
pub const ADDR_frontParser_rxCnt: CPtr = 0x20021996;
/// `frontParser_shownPattern` (1 bytes)
pub const ADDR_frontParser_shownPattern: CPtr = 0x20021997;
/// `frontParser_sysexActive` (1 bytes)
pub const ADDR_frontParser_sysexActive: CPtr = 0x20021998;
/// `frontParser_sysexBuffer` (7 bytes)
pub const ADDR_frontParser_sysexBuffer: CPtr = 0x2002199c;
/// `frontParser_sysexSeqStepNr` (2 bytes)
pub const ADDR_frontParser_sysexSeqStepNr: CPtr = 0x200219a4;
/// `frontParser_twoByteData` (2 bytes)
pub const ADDR_frontParser_twoByteData: CPtr = 0x200219a6;
/// `checkMIDIprocPAR` (1 bytes)
pub const ADDR_checkMIDIprocPAR: CPtr = 0x200219a8;
/// `checkMIDIprocPAR_t` (1 bytes)
pub const ADDR_checkMIDIprocPAR_t: CPtr = 0x200219a9;
/// `midiMsg_tmp` (8 bytes)
pub const ADDR_midiMsg_tmp: CPtr = 0x200219ac;
/// `midiParser_activeNrpnNumber` (2 bytes)
pub const ADDR_midiParser_activeNrpnNumber: CPtr = 0x200219b4;
/// `midiParser_lastMtcReceived` (4 bytes)
pub const ADDR_midiParser_lastMtcReceived: CPtr = 0x200219b8;
/// `midiParser_mtcIsRunning` (1 bytes)
pub const ADDR_midiParser_mtcIsRunning: CPtr = 0x200219bc;
/// `midiParser_originalCcValues` (255 bytes)
pub const ADDR_midiParser_originalCcValues: CPtr = 0x200219c0;
/// `midiParser_routing` (4 bytes)
pub const ADDR_midiParser_routing: CPtr = 0x20021ac0;
/// `midi_MidiChannels` (8 bytes)
pub const ADDR_midi_MidiChannels: CPtr = 0x20021ac4;
/// `active_voices` (1 bytes)
pub const ADDR_active_voices: CPtr = 0x20021acc;
/// `parameterArray` (1976 bytes)
pub const ADDR_parameterArray: CPtr = 0x20021ad0;
/// `fifo_midiRx` (258 bytes)
pub const ADDR_fifo_midiRx: CPtr = 0x200223d0;
/// `midi_process_block` (1 bytes)
pub const ADDR_midi_process_block: CPtr = 0x200225d6;
/// `euklid_length` (7 bytes)
pub const ADDR_euklid_length: CPtr = 0x200225d8;
/// `euklid_nextCnt1` (2 bytes)
pub const ADDR_euklid_nextCnt1: CPtr = 0x200225e0;
/// `euklid_originalLength` (2 bytes)
pub const ADDR_euklid_originalLength: CPtr = 0x200225e2;
/// `euklid_patternBuffer` (8 bytes)
pub const ADDR_euklid_patternBuffer: CPtr = 0x200225e8;
/// `euklid_rotation` (7 bytes)
pub const ADDR_euklid_rotation: CPtr = 0x200225f0;
/// `euklid_steps` (7 bytes)
pub const ADDR_euklid_steps: CPtr = 0x200225f8;
/// `sync_clockCnt` (1 bytes)
pub const ADDR_sync_clockCnt: CPtr = 0x200225ff;
/// `sync_delayedStepTimer` (4 bytes)
pub const ADDR_sync_delayedStepTimer: CPtr = 0x20022600;
/// `sync_timeLast` (4 bytes)
pub const ADDR_sync_timeLast: CPtr = 0x20022604;
/// `sync_window` (32 bytes)
pub const ADDR_sync_window: CPtr = 0x20022608;
/// `sync_writePos` (1 bytes)
pub const ADDR_sync_writePos: CPtr = 0x20022628;
/// `chokeCurrent` (1 bytes)
pub const ADDR_chokeCurrent: CPtr = 0x20022629;
/// `chokeNext` (1 bytes)
pub const ADDR_chokeNext: CPtr = 0x2002262a;
/// `isStepShuffled` (1 bytes)
pub const ADDR_isStepShuffled: CPtr = 0x2002262b;
/// `lastWasNoteOn` (7 bytes)
pub const ADDR_lastWasNoteOn: CPtr = 0x2002262c;
/// `midi_chan_notes` (16 bytes)
pub const ADDR_midi_chan_notes: CPtr = 0x20022634;
/// `midi_notes_on` (2 bytes)
pub const ADDR_midi_notes_on: CPtr = 0x20022644;
/// `seq_sendRealtime_msg` (8 bytes)
pub const ADDR_seq_sendRealtime_msg: CPtr = 0x20022648;
/// `patternBackupBuffer` (3650 bytes)
pub const ADDR_patternBackupBuffer: CPtr = 0x20022650;
/// `seq_activeAutomTrack` (1 bytes)
pub const ADDR_seq_activeAutomTrack: CPtr = 0x20023492;
/// `seq_activePattern` (1 bytes)
pub const ADDR_seq_activePattern: CPtr = 0x20023493;
/// `seq_activeSong` (4 bytes)
pub const ADDR_seq_activeSong: CPtr = 0x20023494;
/// `seq_automationNodes` (28 bytes)
pub const ADDR_seq_automationNodes: CPtr = 0x20023498;
/// `seq_deltaT` (4 bytes)
pub const ADDR_seq_deltaT: CPtr = 0x200234b4;
/// `seq_eraseActive` (1 bytes)
pub const ADDR_seq_eraseActive: CPtr = 0x200234b8;
/// `seq_isSyncExternal` (1 bytes)
pub const ADDR_seq_isSyncExternal: CPtr = 0x200234b9;
/// `seq_lastMasterStep` (7 bytes)
pub const ADDR_seq_lastMasterStep: CPtr = 0x200234bc;
/// `seq_lastShuffle` (4 bytes)
pub const ADDR_seq_lastShuffle: CPtr = 0x200234c4;
/// `seq_lastTick` (4 bytes)
pub const ADDR_seq_lastTick: CPtr = 0x200234c8;
/// `seq_loadPendigFlag` (1 bytes)
pub const ADDR_seq_loadPendigFlag: CPtr = 0x200234cc;
/// `seq_masterStepCnt` (1 bytes)
pub const ADDR_seq_masterStepCnt: CPtr = 0x200234cd;
/// `seq_mutedTracks` (1 bytes)
pub const ADDR_seq_mutedTracks: CPtr = 0x200234ce;
/// `seq_patternSet` (233536 bytes)
pub const ADDR_seq_patternSet: CPtr = 0x200234d0;
/// `seq_pendingPattern` (1 bytes)
pub const ADDR_seq_pendingPattern: CPtr = 0x2005c510;
/// `seq_prescaleCounter` (1 bytes)
pub const ADDR_seq_prescaleCounter: CPtr = 0x2005c511;
/// `seq_recordActive` (1 bytes)
pub const ADDR_seq_recordActive: CPtr = 0x2005c512;
/// `seq_resetBarOnPatternChange` (1 bytes)
pub const ADDR_seq_resetBarOnPatternChange: CPtr = 0x2005c513;
/// `seq_rndValue` (21 bytes)
pub const ADDR_seq_rndValue: CPtr = 0x2005c514;
/// `seq_rollState` (1 bytes)
pub const ADDR_seq_rollState: CPtr = 0x2005c529;
/// `seq_running` (1 bytes)
pub const ADDR_seq_running: CPtr = 0x2005c52a;
/// `seq_selectedStep` (1 bytes)
pub const ADDR_seq_selectedStep: CPtr = 0x2005c52b;
/// `seq_shuffle` (4 bytes)
pub const ADDR_seq_shuffle: CPtr = 0x2005c52c;
/// `seq_songData` (4096 bytes)
pub const ADDR_seq_songData: CPtr = 0x2005c530;
/// `seq_songModeEnabled` (1 bytes)
pub const ADDR_seq_songModeEnabled: CPtr = 0x2005d530;
/// `seq_songPositionPointer` (1 bytes)
pub const ADDR_seq_songPositionPointer: CPtr = 0x2005d531;
/// `seq_stepIndex` (7 bytes)
pub const ADDR_seq_stepIndex: CPtr = 0x2005d534;
/// `seq_stepToSkip` (7 bytes)
pub const ADDR_seq_stepToSkip: CPtr = 0x2005d53c;
/// `trigger_clockReceived` (1 bytes)
pub const ADDR_trigger_clockReceived: CPtr = 0x2005d565;
/// `activeParameters` (302 bytes)
pub const ADDR_activeParameters: CPtr = 0x2005dfbc;
/// `mainParameters` (15808 bytes)
pub const ADDR_mainParameters: CPtr = 0x2005e134;
/// `menu_activeBar` (1 bytes)
pub const ADDR_menu_activeBar: CPtr = 0x20061ef5;
/// `menu_activePage` (1 bytes)
pub const ADDR_menu_activePage: CPtr = 0x20061ef6;
/// `menu_loadedPresetNr` (4 bytes)
pub const ADDR_menu_loadedPresetNr: CPtr = 0x20061efc;
/// `menu_playedPattern` (1 bytes)
pub const ADDR_menu_playedPattern: CPtr = 0x20061f02;
/// `menu_selectedPresetNr` (6 bytes)
pub const ADDR_menu_selectedPresetNr: CPtr = 0x20061f08;
/// `menu_shownPattern` (1 bytes)
pub const ADDR_menu_shownPattern: CPtr = 0x20061f0e;
/// `menu_songMenuPosition` (1 bytes)
pub const ADDR_menu_songMenuPosition: CPtr = 0x20061f0f;
/// `morphParameters` (4 bytes)
pub const ADDR_morphParameters: CPtr = 0x20061f10;
/// table `initKit`
pub const ADDR_initKit: CPtr = 0x0802a23c;
/// table `MidiNoteFrequencies`
pub const ADDR_MidiNoteFrequencies: CPtr = 0x0802a408;
/// table `recTable`
pub const ADDR_recTable: CPtr = 0x0802a634;
/// table `sawTable`
pub const ADDR_sawTable: CPtr = 0x0802fe34;
/// table `triTable`
pub const ADDR_triTable: CPtr = 0x08035634;
/// table `modTargets`
pub const ADDR_modTargets: CPtr = 0x0803b068;
/// table `table_1n914`
pub const ADDR_table_1n914: CPtr = 0x20000000;
/// table `ecc83Table`
pub const ADDR_ecc83Table: CPtr = 0x20000408;
/// table `transientData`
pub const ADDR_transientData: CPtr = 0x20000810;
/// table `transientVolumeTable`
pub const ADDR_transientVolumeTable: CPtr = 0x20006f6c;
/// table `sine_table`
pub const ADDR_sine_table: CPtr = 0x20007080;
/// table `parameterOrder`
pub const ADDR_parameterOrder: CPtr = 0x200201dc;

/// C `sizeof` of every C struct type (bytes).
pub mod size_of_c {
    pub const AllpassDelay: u32 = 28;
    pub const AllpassModDelay: u32 = 64;
    pub const AutomationNode: u32 = 2;
    pub const Compressor: u32 = 32;
    pub const CymbalVoice: u32 = 384;
    pub const DecayEg: u32 = 12;
    pub const Delay: u32 = 96;
    pub const Distortion: u32 = 24;
    pub const Dither: u32 = 24;
    pub const DrumVoice: u32 = 704;
    pub const Fifo: u32 = 258;
    pub const FifoBig: u32 = 66;
    pub const FxBus: u32 = 352;
    pub const HiHatVoice: u32 = 392;
    pub const Lfo: u32 = 48;
    pub const MasterDistortion: u32 = 152;
    pub const MidiMsg: u32 = 8;
    pub const ModTarg: u32 = 4;
    pub const ModulationNode: u32 = 16;
    pub const OnePoleFilter: u32 = 16;
    pub const OscInfo: u32 = 40;
    pub const Parameter: u32 = 8;
    pub const PatternBackupBuffer: u32 = 3650;
    pub const PatternSet: u32 = 233536;
    pub const ResonantFilter: u32 = 32;
    pub const Reverb: u32 = 520;
    pub const Ringmod: u32 = 44;
    pub const SineCosine: u32 = 12;
    pub const SlopeEg2: u32 = 56;
    pub const SnapEg: u32 = 4;
    pub const SnareVoice: u32 = 368;
    pub const Step: u32 = 7;
    pub const TiltEq: u32 = 28;
    pub const TransientGenerator: u32 = 24;
    pub const VerbDelay: u32 = 16;
    pub const VerbModDelay: u32 = 48;
    pub const pair: u32 = 8;
    pub const seq_rndValue_t: u32 = 3;
}

/// C `offsetof(<struct>, <member>)` of every C struct type, named `<struct>_<member>`.
pub mod offset_of_c {
    pub const AllpassDelay_g: u32 = 0;
    pub const AllpassDelay_out: u32 = 4;
    pub const AllpassDelay_in: u32 = 8;
    pub const AllpassDelay_delay: u32 = 12;
    pub const AllpassModDelay_g: u32 = 0;
    pub const AllpassModDelay_out: u32 = 4;
    pub const AllpassModDelay_in: u32 = 8;
    pub const AllpassModDelay_delay: u32 = 16;
    pub const AutomationNode_destination: u32 = 0;
    pub const Compressor_tresh: u32 = 0;
    pub const Compressor_attack: u32 = 4;
    pub const Compressor_release: u32 = 8;
    pub const Compressor_ratio: u32 = 12;
    pub const Compressor_markupGain: u32 = 16;
    pub const Compressor_env: u32 = 24;
    pub const CymbalVoice_osc: u32 = 0;
    pub const CymbalVoice_modOsc: u32 = 40;
    pub const CymbalVoice_modOsc2: u32 = 80;
    pub const CymbalVoice_fmModAmount1: u32 = 120;
    pub const CymbalVoice_fmModAmount2: u32 = 124;
    pub const CymbalVoice_vol: u32 = 128;
    pub const CymbalVoice_pan: u32 = 132;
    pub const CymbalVoice_triggeredPanL: u32 = 136;
    pub const CymbalVoice_triggeredPanR: u32 = 140;
    pub const CymbalVoice_velo: u32 = 144;
    pub const CymbalVoice_noiseSample: u32 = 148;
    pub const CymbalVoice_filter: u32 = 152;
    pub const CymbalVoice_filterType: u32 = 184;
    pub const CymbalVoice_lfo: u32 = 188;
    pub const CymbalVoice_transGen: u32 = 236;
    pub const CymbalVoice_distortion: u32 = 264;
    pub const CymbalVoice_oscVolEg: u32 = 288;
    pub const CymbalVoice_egValueOscVol: u32 = 344;
    pub const CymbalVoice_volumeMod: u32 = 348;
    pub const CymbalVoice_snapEg: u32 = 352;
    pub const CymbalVoice_combinedVolume: u32 = 356;
    pub const CymbalVoice_FaderVolumeFilter: u32 = 360;
    pub const CymbalVoice_pwm: u32 = 376;
    pub const DecayEg_decay: u32 = 0;
    pub const DecayEg_slope: u32 = 4;
    pub const DecayEg_value: u32 = 8;
    pub const Delay_oldRead: u32 = 0;
    pub const Delay_newRead: u32 = 4;
    pub const Delay_write: u32 = 8;
    pub const Delay_fracReadNew: u32 = 12;
    pub const Delay_fracReadOld: u32 = 16;
    pub const Delay_interpValue: u32 = 20;
    pub const Delay_interpStep: u32 = 24;
    pub const Delay_tmpBuffer: u32 = 28;
    pub const Delay_tmpBuffer2: u32 = 32;
    pub const Delay_feedback: u32 = 36;
    pub const Delay_range: u32 = 40;
    pub const Delay_delayTime: u32 = 44;
    pub const Delay_pendingDelayTime: u32 = 48;
    pub const Delay_timePending: u32 = 52;
    pub const Delay_tone: u32 = 56;
    pub const Delay_tone2: u32 = 72;
    pub const Delay_delayType: u32 = 88;
    pub const Distortion_shape: u32 = 0;
    pub const Distortion_inv_shape: u32 = 4;
    pub const Distortion_DriveParamFilter: u32 = 8;
    pub const Dither_r1: u32 = 0;
    pub const Dither_r2: u32 = 2;
    pub const Dither_s1: u32 = 4;
    pub const Dither_s2: u32 = 8;
    pub const Dither_in: u32 = 12;
    pub const Dither_tmp: u32 = 16;
    pub const Dither_out: u32 = 20;
    pub const DrumVoice_osc: u32 = 0;
    pub const DrumVoice_modOsc: u32 = 40;
    pub const DrumVoice_fmModAmount: u32 = 80;
    pub const DrumVoice_vol: u32 = 84;
    pub const DrumVoice_velo: u32 = 88;
    pub const DrumVoice_pan: u32 = 92;
    pub const DrumVoice_triggeredPanL: u32 = 96;
    pub const DrumVoice_triggeredPanR: u32 = 100;
    pub const DrumVoice_oscSample: u32 = 104;
    pub const DrumVoice_oscPitchEg: u32 = 108;
    pub const DrumVoice_egPitchModAmount: u32 = 120;
    pub const DrumVoice_offset: u32 = 124;
    pub const DrumVoice_transGen: u32 = 128;
    pub const DrumVoice_lfo: u32 = 152;
    pub const DrumVoice_oscVolEg: u32 = 200;
    pub const DrumVoice_egValueOscVol: u32 = 256;
    pub const DrumVoice_volEgValueBlock: u32 = 260;
    pub const DrumVoice_distortion: u32 = 520;
    pub const DrumVoice_filter: u32 = 544;
    pub const DrumVoice_filterType: u32 = 576;
    pub const DrumVoice_ampFilterInput: u32 = 584;
    pub const DrumVoice_lastGain: u32 = 592;
    pub const DrumVoice_mixOscs: u32 = 600;
    pub const DrumVoice_decimationCnt: u32 = 604;
    pub const DrumVoice_decimationRate: u32 = 608;
    pub const DrumVoice_snapEg: u32 = 612;
    pub const DrumVoice_volumeMod: u32 = 616;
    pub const DrumVoice_dither: u32 = 620;
    pub const DrumVoice_combinedVolume: u32 = 644;
    pub const DrumVoice_FaderVolumeFilter: u32 = 648;
    pub const DrumVoice_MixerPageFilter: u32 = 664;
    pub const DrumVoice_pwm: u32 = 696;
    pub const Fifo_data: u32 = 0;
    pub const Fifo_read: u32 = 256;
    pub const Fifo_write: u32 = 257;
    pub const FifoBig_data: u32 = 0;
    pub const FifoBig_read: u32 = 64;
    pub const FifoBig_write: u32 = 65;
    pub const FxBus_fxType: u32 = 0;
    pub const FxBus_audioRouting: u32 = 1;
    pub const FxBus_dryWet: u32 = 4;
    pub const FxBus_parameter: u32 = 8;
    pub const FxBus_distortion: u32 = 24;
    pub const FxBus_ringmod: u32 = 176;
    pub const FxBus_compressor: u32 = 224;
    pub const FxBus_delay: u32 = 256;
    pub const HiHatVoice_osc: u32 = 0;
    pub const HiHatVoice_modOsc: u32 = 40;
    pub const HiHatVoice_modOsc2: u32 = 80;
    pub const HiHatVoice_fmModAmount1: u32 = 120;
    pub const HiHatVoice_fmModAmount2: u32 = 124;
    pub const HiHatVoice_vol: u32 = 128;
    pub const HiHatVoice_pan: u32 = 132;
    pub const HiHatVoice_triggeredPanL: u32 = 136;
    pub const HiHatVoice_triggeredPanR: u32 = 140;
    pub const HiHatVoice_velo: u32 = 144;
    pub const HiHatVoice_noiseSample: u32 = 148;
    pub const HiHatVoice_filter: u32 = 152;
    pub const HiHatVoice_filterType: u32 = 184;
    pub const HiHatVoice_transGen: u32 = 188;
    pub const HiHatVoice_decayClosed: u32 = 212;
    pub const HiHatVoice_decayOpen: u32 = 216;
    pub const HiHatVoice_distortion: u32 = 224;
    pub const HiHatVoice_oscVolEg: u32 = 248;
    pub const HiHatVoice_egValueOscVol: u32 = 304;
    pub const HiHatVoice_lfo: u32 = 308;
    pub const HiHatVoice_snapEg: u32 = 356;
    pub const HiHatVoice_volumeMod: u32 = 360;
    pub const HiHatVoice_isOpen: u32 = 361;
    pub const HiHatVoice_combinedVolume: u32 = 364;
    pub const HiHatVoice_FaderVolumeFilter: u32 = 368;
    pub const HiHatVoice_pwm: u32 = 384;
    pub const Lfo_phase: u32 = 0;
    pub const Lfo_phaseInc: u32 = 4;
    pub const Lfo_waveform: u32 = 8;
    pub const Lfo_retrigger: u32 = 9;
    pub const Lfo_phaseOffset: u32 = 12;
    pub const Lfo_rnd: u32 = 16;
    pub const Lfo_sync: u32 = 20;
    pub const Lfo_freq: u32 = 24;
    pub const Lfo_modTarget: u32 = 28;
    pub const Lfo_modNodeValue: u32 = 44;
    pub const MasterDistortion_distortionType: u32 = 0;
    pub const MasterDistortion_drive: u32 = 4;
    pub const MasterDistortion_p1: u32 = 8;
    pub const MasterDistortion_p2: u32 = 12;
    pub const MasterDistortion_p3: u32 = 16;
    pub const MasterDistortion_hp1_state: u32 = 24;
    pub const MasterDistortion_hp2_state: u32 = 40;
    pub const MasterDistortion_hp3_state: u32 = 56;
    pub const MasterDistortion_lp1_state: u32 = 72;
    pub const MasterDistortion_lp2_state: u32 = 88;
    pub const MasterDistortion_lp3_state: u32 = 104;
    pub const MasterDistortion_slewState: u32 = 120;
    pub const MasterDistortion_lastOut: u32 = 136;
    pub const MidiMsg_status: u32 = 0;
    pub const MidiMsg_lastNonSysStatus: u32 = 1;
    pub const MidiMsg_data1: u32 = 2;
    pub const MidiMsg_data2: u32 = 3;
    pub const MidiMsg_bits: u32 = 4;
    pub const ModTarg_nameIdx: u32 = 0;
    pub const ModTarg_param: u32 = 2;
    pub const ModulationNode_destination: u32 = 0;
    pub const ModulationNode_type: u32 = 2;
    pub const ModulationNode_originalValue: u32 = 4;
    pub const ModulationNode_amount: u32 = 8;
    pub const ModulationNode_lastVal: u32 = 12;
    pub const OnePoleFilter_r: u32 = 0;
    pub const OnePoleFilter_b: u32 = 8;
    pub const OscInfo_output: u32 = 0;
    pub const OscInfo_phaseInc: u32 = 4;
    pub const OscInfo_phase: u32 = 8;
    pub const OscInfo_freq: u32 = 12;
    pub const OscInfo_waveform: u32 = 16;
    pub const OscInfo_tableOffset: u32 = 17;
    pub const OscInfo_pitchMod: u32 = 20;
    pub const OscInfo_fmMod: u32 = 24;
    pub const OscInfo_modNodeValue: u32 = 28;
    pub const OscInfo_midiFreq: u32 = 32;
    pub const OscInfo_baseNote: u32 = 34;
    pub const OscInfo_startPhase: u32 = 36;
    pub const Parameter_ptr: u32 = 0;
    pub const Parameter_type: u32 = 4;
    pub const PatternBackupBuffer_seq_subStepPattern: u32 = 0;
    pub const PatternBackupBuffer_seq_stepTiming: u32 = 3136;
    pub const PatternBackupBuffer_seq_mainSteps: u32 = 3584;
    pub const PatternBackupBuffer_length: u32 = 3640;
    pub const PatternBackupBuffer_shuffle: u32 = 3647;
    pub const PatternBackupBuffer_kitNr: u32 = 3648;
    pub const PatternSet_seq_subStepPattern: u32 = 0;
    pub const PatternSet_seq_stepTiming: u32 = 200704;
    pub const PatternSet_seq_mainSteps: u32 = 229376;
    pub const PatternSet_length: u32 = 232960;
    pub const PatternSet_shuffle: u32 = 233408;
    pub const PatternSet_kitNr: u32 = 233472;
    pub const ResonantFilter_f: u32 = 0;
    pub const ResonantFilter_g: u32 = 4;
    pub const ResonantFilter_q: u32 = 8;
    pub const ResonantFilter_s1: u32 = 12;
    pub const ResonantFilter_s2: u32 = 16;
    pub const ResonantFilter_a: u32 = 20;
    pub const ResonantFilter_b: u32 = 24;
    pub const ResonantFilter_drive: u32 = 28;
    pub const Reverb_apDelay: u32 = 0;
    pub const Reverb_apModDelay: u32 = 56;
    pub const Reverb_inputAllpass: u32 = 184;
    pub const Reverb_delay: u32 = 296;
    pub const Reverb_earlyReflectionDelay: u32 = 360;
    pub const Reverb_preDelay: u32 = 376;
    pub const Reverb_lfo: u32 = 392;
    pub const Reverb_feedback: u32 = 404;
    pub const Reverb_delayTime: u32 = 408;
    pub const Reverb_earlyVolume: u32 = 412;
    pub const Reverb_lpFeedback: u32 = 416;
    pub const Reverb_lpImpulse: u32 = 432;
    pub const Reverb_reverbBufferSram: u32 = 448;
    pub const Reverb_lastOut: u32 = 452;
    pub const Reverb_lastOut2: u32 = 456;
    pub const Reverb_eq: u32 = 460;
    pub const Reverb_eqFb: u32 = 488;
    pub const Ringmod_freq: u32 = 0;
    pub const Ringmod_osc: u32 = 4;
    pub const SineCosine_a: u32 = 0;
    pub const SineCosine_s: u32 = 4;
    pub const SlopeEg2_attack: u32 = 0;
    pub const SlopeEg2_decay: u32 = 8;
    pub const SlopeEg2_slope: u32 = 16;
    pub const SlopeEg2_invSlope: u32 = 24;
    pub const SlopeEg2_repeat: u32 = 32;
    pub const SlopeEg2_repeatCnt: u32 = 33;
    pub const SlopeEg2_value: u32 = 40;
    pub const SlopeEg2_state: u32 = 48;
    pub const SnapEg_value: u32 = 0;
    pub const SnareVoice_osc: u32 = 0;
    pub const SnareVoice_noiseOsc: u32 = 40;
    pub const SnareVoice_filterType: u32 = 80;
    pub const SnareVoice_vol: u32 = 84;
    pub const SnareVoice_pan: u32 = 88;
    pub const SnareVoice_triggeredPanL: u32 = 92;
    pub const SnareVoice_triggeredPanR: u32 = 96;
    pub const SnareVoice_mix: u32 = 100;
    pub const SnareVoice_velo: u32 = 104;
    pub const SnareVoice_oscSample: u32 = 108;
    pub const SnareVoice_noiseSample: u32 = 112;
    pub const SnareVoice_filter: u32 = 116;
    pub const SnareVoice_oscPitchEg: u32 = 148;
    pub const SnareVoice_egPitchModAmount: u32 = 160;
    pub const SnareVoice_lfo: u32 = 164;
    pub const SnareVoice_transGen: u32 = 212;
    pub const SnareVoice_oscVolEg: u32 = 240;
    pub const SnareVoice_egValueOscVol: u32 = 296;
    pub const SnareVoice_distortion: u32 = 304;
    pub const SnareVoice_snapEg: u32 = 328;
    pub const SnareVoice_volumeMod: u32 = 332;
    pub const SnareVoice_combinedVolume: u32 = 336;
    pub const SnareVoice_FaderVolumeFilter: u32 = 344;
    pub const SnareVoice_pwm: u32 = 360;
    pub const Step_volume: u32 = 0;
    pub const Step_prob: u32 = 1;
    pub const Step_note: u32 = 2;
    pub const Step_param1Nr: u32 = 3;
    pub const Step_param1Val: u32 = 4;
    pub const Step_param2Nr: u32 = 5;
    pub const Step_param2Val: u32 = 6;
    pub const TiltEq_amp: u32 = 0;
    pub const TiltEq_gfactor: u32 = 4;
    pub const TiltEq_a0: u32 = 8;
    pub const TiltEq_b1: u32 = 12;
    pub const TiltEq_lp_out: u32 = 16;
    pub const TiltEq_lgain: u32 = 20;
    pub const TiltEq_hgain: u32 = 24;
    pub const TransientGenerator_output: u32 = 0;
    pub const TransientGenerator_phase: u32 = 4;
    pub const TransientGenerator_pitch: u32 = 8;
    pub const TransientGenerator_waveform: u32 = 12;
    pub const TransientGenerator_volume: u32 = 16;
    pub const TransientGenerator_midiFreq: u32 = 20;
    pub const VerbDelay_ramBuffer: u32 = 0;
    pub const VerbDelay_length: u32 = 4;
    pub const VerbDelay_readPtr: u32 = 6;
    pub const VerbDelay_writePtr: u32 = 8;
    pub const VerbDelay_roomSz: u32 = 12;
    pub const VerbModDelay_ramBuffer: u32 = 0;
    pub const VerbModDelay_length: u32 = 4;
    pub const VerbModDelay_readPtr: u32 = 8;
    pub const VerbModDelay_writePtr: u32 = 12;
    pub const VerbModDelay_lp: u32 = 16;
    pub const VerbModDelay_frac: u32 = 32;
    pub const VerbModDelay_depth: u32 = 36;
    pub const VerbModDelay_modVal: u32 = 40;
    pub const VerbModDelay_roomSz: u32 = 44;
    pub const pair_point: u32 = 0;
    pub const pair_value: u32 = 4;
    pub const seq_rndValue_t_prev: u32 = 0;
    pub const seq_rndValue_t_current: u32 = 1;
    pub const seq_rndValue_t_next: u32 = 2;
}

/// Byte at firmware address `p` of a DTCM table (initialised RAM holding constant data), for reads through
/// pointers that out-of-range stores have moved into a table (`midi::shipped_layout::SHIPPED_TABLES`).
/// Non-C helper, group P3.
pub fn table_byte(p: CPtr) -> Option<u8> {
    use crate::tables::distortion::{ecc83Table, table_1n914};
    use crate::tables::transient_tables::{transientData, transientVolumeTable};
    use crate::tables::wavetable::sine_table;
    fn at<const W: usize>(off: u32, el: impl Fn(usize) -> [u8; W]) -> u8 {
        el(off as usize / W)[off as usize % W]
    }
    let inr = |a: CPtr, n: u32| p >= a && p < a + n;
    Some(if inr(ADDR_transientData, 26460) {
        let o = (p - ADDR_transientData) as usize;
        transientData[o / 2205][o % 2205] as u8
    } else if inr(ADDR_transientVolumeTable, 69 * 4) {
        at(p - ADDR_transientVolumeTable, |i| transientVolumeTable[i].to_le_bytes())
    } else if inr(ADDR_sine_table, 4097 * 2) {
        at(p - ADDR_sine_table, |i| sine_table[i].to_le_bytes())
    } else if inr(ADDR_table_1n914, 257 * 4) {
        at(p - ADDR_table_1n914, |i| table_1n914[i].to_le_bytes())
    } else if inr(ADDR_ecc83Table, 129 * 8) {
        at(p - ADDR_ecc83Table, |i| ecc83Table[i].to_le_bytes())
    } else {
        return None;
    })
}

impl Lxr {
    /// Byte at firmware address `p`, which must lie in an `Lxr` global (panics otherwise: the firmware
    /// would read memory the port does not model).
    #[inline]
    fn cptr_byte(&self, p: CPtr) -> u8 {
        // NULL pointer targets (`parameterArray` entries that `parameterArray_init` leaves zero): address
        // 0x00000000 is ITCM RAM holding the shipped code, see `preset::SHIPPED_ITCM_HEAD`.
        if (p as usize) < crate::preset::SHIPPED_ITCM_HEAD.len() {
            return crate::preset::SHIPPED_ITCM_HEAD[p as usize];
        }
        match self.global_get_byte(p).or_else(|| table_byte(p)) {
            Some(b) => b,
            None => panic!("cptr: read of unmapped firmware address {p:#010x}"),
        }
    }
    #[inline]
    fn cptr_set_byte(&mut self, p: CPtr, v: u8) {
        if !self.global_set_byte(p, v) {
            panic!("cptr: write to unmapped firmware address {p:#010x}");
        }
    }
    /// Reads the `f32` at firmware address `p` (a field of an `Lxr` global). Non-C helper, group P3
    /// (used by ParameterArray / modulationNode code). Byte-wise, so a load from a field of another
    /// type (e.g. the low word of a `double`) reinterprets the bytes exactly like the firmware.
    pub fn cptr_read_f32(&self, p: CPtr) -> f32 {
        f32::from_bits(self.cptr_read_u32(p))
    }
    /// Writes the `f32` at firmware address `p`. Non-C helper, group P3.
    pub fn cptr_write_f32(&mut self, p: CPtr, v: f32) {
        self.cptr_write_u32(p, v.to_bits())
    }
    /// Reads the `u8` at firmware address `p`. Non-C helper, group P3.
    pub fn cptr_read_u8(&self, p: CPtr) -> u8 {
        self.cptr_byte(p)
    }
    /// Writes the `u8` at firmware address `p`. Non-C helper, group P3.
    pub fn cptr_write_u8(&mut self, p: CPtr, v: u8) {
        self.cptr_set_byte(p, v)
    }
    /// Reads the `u32` at firmware address `p`. Non-C helper, group P3.
    pub fn cptr_read_u32(&self, p: CPtr) -> u32 {
        u32::from_le_bytes([self.cptr_byte(p), self.cptr_byte(p + 1), self.cptr_byte(p + 2), self.cptr_byte(p + 3)])
    }
    /// Writes the `u32` at firmware address `p`. Non-C helper, group P3.
    pub fn cptr_write_u32(&mut self, p: CPtr, v: u32) {
        for (i, b) in v.to_le_bytes().into_iter().enumerate() {
            self.cptr_set_byte(p + i as u32, b);
        }
    }
}
