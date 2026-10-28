//! Port of `Core/Inc/Soundengine/MIDI/MidiMessages.h`, `Core/Src/Soundengine/MIDI/MidiMessages.c`.
//!
//! Porting group: P3. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;

// ---- #define constants
/// `#define NOTE_OFF 0x80` (Core/Inc/Soundengine/MIDI/MidiMessages.h:46)
pub const NOTE_OFF: i32 = 128;
/// `#define NOTE_ON 0x90` (Core/Inc/Soundengine/MIDI/MidiMessages.h:47)
pub const NOTE_ON: i32 = 144;
/// `#define MIDI_CC 0xB0` (Core/Inc/Soundengine/MIDI/MidiMessages.h:48)
pub const MIDI_CC: i32 = 176;
/// `#define MIDI_CC2 0xF4` (Core/Inc/Soundengine/MIDI/MidiMessages.h:49)
pub const MIDI_CC2: i32 = 244;
/// `#define PROG_CHANGE 0xC0` (Core/Inc/Soundengine/MIDI/MidiMessages.h:50)
pub const PROG_CHANGE: i32 = 192;
/// `#define MIDI_PITCH_WHEEL 0xE0` (Core/Inc/Soundengine/MIDI/MidiMessages.h:51)
pub const MIDI_PITCH_WHEEL: i32 = 224;
/// `#define MIDI_AT 0xA0` (Core/Inc/Soundengine/MIDI/MidiMessages.h:52)
pub const MIDI_AT: i32 = 160;
/// `#define CHANNEL_PRESSURE 0xD0` (Core/Inc/Soundengine/MIDI/MidiMessages.h:53)
pub const CHANNEL_PRESSURE: i32 = 208;
/// `#define CC_MOD_WHEEL 0x01` (Core/Inc/Soundengine/MIDI/MidiMessages.h:59)
pub const CC_MOD_WHEEL: i32 = 1;
/// `#define CC_ALL_SOUND_OFF 0x78` (Core/Inc/Soundengine/MIDI/MidiMessages.h:60)
pub const CC_ALL_SOUND_OFF: i32 = 120;
/// `#define CC_ALL_NOTES_OFF 0x7B` (Core/Inc/Soundengine/MIDI/MidiMessages.h:61)
pub const CC_ALL_NOTES_OFF: i32 = 123;
/// `#define MIDI_CLOCK 0xF8` (Core/Inc/Soundengine/MIDI/MidiMessages.h:65)
pub const MIDI_CLOCK: i32 = 248;
/// `#define SYSEX_START 0xF0` (Core/Inc/Soundengine/MIDI/MidiMessages.h:66)
pub const SYSEX_START: i32 = 240;
/// `#define SYSEX_END 0xF7` (Core/Inc/Soundengine/MIDI/MidiMessages.h:67)
pub const SYSEX_END: i32 = 247;
/// `#define MIDI_START 0xFA` (Core/Inc/Soundengine/MIDI/MidiMessages.h:68)
pub const MIDI_START: i32 = 250;
/// `#define MIDI_STOP 0xFC` (Core/Inc/Soundengine/MIDI/MidiMessages.h:69)
pub const MIDI_STOP: i32 = 252;
/// `#define MIDI_CONTINUE 0xFB` (Core/Inc/Soundengine/MIDI/MidiMessages.h:70)
pub const MIDI_CONTINUE: i32 = 251;
/// `#define MIDI_MTC_QFRAME 0xF1` (Core/Inc/Soundengine/MIDI/MidiMessages.h:71)
pub const MIDI_MTC_QFRAME: i32 = 241;
/// `#define MIDI_SONG_POSITION_POINTER 0xF2` (Core/Inc/Soundengine/MIDI/MidiMessages.h:72)
pub const MIDI_SONG_POSITION_POINTER: i32 = 242;
/// `#define MIDI_SONG_SEL 0xF3` (Core/Inc/Soundengine/MIDI/MidiMessages.h:73)
pub const MIDI_SONG_SEL: i32 = 243;
/// `#define NO_AUTOMATION 0xff` (Core/Inc/Soundengine/MIDI/MidiMessages.h:78)
pub const NO_AUTOMATION: i32 = 255;
/// `#define FRONT_SEQ_CC 0xd2` (Core/Inc/Soundengine/MIDI/MidiMessages.h:418)
pub const FRONT_SEQ_CC: i32 = 210;
/// `#define FRONT_CODEC_CONTROL 0xd3` (Core/Inc/Soundengine/MIDI/MidiMessages.h:419)
pub const FRONT_CODEC_CONTROL: i32 = 211;
/// `#define VOICE_CC 0xd4` (Core/Inc/Soundengine/MIDI/MidiMessages.h:420)
pub const VOICE_CC: i32 = 212;
/// `#define FRONT_SET_BPM 0xd5` (Core/Inc/Soundengine/MIDI/MidiMessages.h:421)
pub const FRONT_SET_BPM: i32 = 213;
/// `#define FRONT_CC_2 0xd6` (Core/Inc/Soundengine/MIDI/MidiMessages.h:422)
pub const FRONT_CC_2: i32 = 214;
/// `#define FRONT_CC_LFO_TARGET 0xd7` (Core/Inc/Soundengine/MIDI/MidiMessages.h:423)
pub const FRONT_CC_LFO_TARGET: i32 = 215;
/// `#define FRONT_CC_VELO_TARGET 0xd8` (Core/Inc/Soundengine/MIDI/MidiMessages.h:424)
pub const FRONT_CC_VELO_TARGET: i32 = 216;
/// `#define FRONT_SET_P1_DEST 0xda` (Core/Inc/Soundengine/MIDI/MidiMessages.h:426)
pub const FRONT_SET_P1_DEST: i32 = 218;
/// `#define FRONT_SET_P2_DEST 0xdb` (Core/Inc/Soundengine/MIDI/MidiMessages.h:427)
pub const FRONT_SET_P2_DEST: i32 = 219;
/// `#define FRONT_SET_P1_VAL 0xdc` (Core/Inc/Soundengine/MIDI/MidiMessages.h:428)
pub const FRONT_SET_P1_VAL: i32 = 220;
/// `#define FRONT_SET_P2_VAL 0xdd` (Core/Inc/Soundengine/MIDI/MidiMessages.h:429)
pub const FRONT_SET_P2_VAL: i32 = 221;
/// `#define FRONT_ARM_AUTOMATION_STEP 0xdf` (Core/Inc/Soundengine/MIDI/MidiMessages.h:431)
pub const FRONT_ARM_AUTOMATION_STEP: i32 = 223;
/// `#define SAMPLE_CC 0xc0` (Core/Inc/Soundengine/MIDI/MidiMessages.h:433)
pub const SAMPLE_CC: i32 = 192;
/// `#define FRONT_SAMPLE_START_UPLOAD 0x01` (Core/Inc/Soundengine/MIDI/MidiMessages.h:434)
pub const FRONT_SAMPLE_START_UPLOAD: i32 = 1;
/// `#define FRONT_SAMPLE_COUNT 0x02` (Core/Inc/Soundengine/MIDI/MidiMessages.h:435)
pub const FRONT_SAMPLE_COUNT: i32 = 2;
/// `#define FRONT_LED_SEQ_BUTTON 0x02` (Core/Inc/Soundengine/MIDI/MidiMessages.h:439)
pub const FRONT_LED_SEQ_BUTTON: i32 = 2;
/// `#define FRONT_LED_PULSE_BEAT 0x04` (Core/Inc/Soundengine/MIDI/MidiMessages.h:441)
pub const FRONT_LED_PULSE_BEAT: i32 = 4;
/// `#define FRONT_SEQ_RUN_STOP 0x01` (Core/Inc/Soundengine/MIDI/MidiMessages.h:448)
pub const FRONT_SEQ_RUN_STOP: i32 = 1;
/// `#define FRONT_SEQ_MUTE_TRACK 0x09` (Core/Inc/Soundengine/MIDI/MidiMessages.h:450)
pub const FRONT_SEQ_MUTE_TRACK: i32 = 9;
/// `#define FRONT_SEQ_UNMUTE_TRACK 0x0a` (Core/Inc/Soundengine/MIDI/MidiMessages.h:451)
pub const FRONT_SEQ_UNMUTE_TRACK: i32 = 10;
/// `#define FRONT_SEQ_CHANGE_PAT 0x0b` (Core/Inc/Soundengine/MIDI/MidiMessages.h:452)
pub const FRONT_SEQ_CHANGE_PAT: i32 = 11;
/// `#define FRONT_SEQ_ROLL_ON 0x0c` (Core/Inc/Soundengine/MIDI/MidiMessages.h:453)
pub const FRONT_SEQ_ROLL_ON: i32 = 12;
/// `#define FRONT_SEQ_ROLL_OFF 0x0d` (Core/Inc/Soundengine/MIDI/MidiMessages.h:454)
pub const FRONT_SEQ_ROLL_OFF: i32 = 13;
/// `#define FRONT_SEQ_REQUEST_STEP_PARAMS 0x0f` (Core/Inc/Soundengine/MIDI/MidiMessages.h:455)
pub const FRONT_SEQ_REQUEST_STEP_PARAMS: i32 = 15;
/// `#define FRONT_SEQ_ROLL_ON_OFF 0x10` (Core/Inc/Soundengine/MIDI/MidiMessages.h:456)
pub const FRONT_SEQ_ROLL_ON_OFF: i32 = 16;
/// `#define FRONT_SEQ_ROLL_RATE 0x11` (Core/Inc/Soundengine/MIDI/MidiMessages.h:457)
pub const FRONT_SEQ_ROLL_RATE: i32 = 17;
/// `#define FRONT_SEQ_VOLUME 0x12` (Core/Inc/Soundengine/MIDI/MidiMessages.h:458)
pub const FRONT_SEQ_VOLUME: i32 = 18;
/// `#define FRONT_SEQ_NOTE 0x13` (Core/Inc/Soundengine/MIDI/MidiMessages.h:459)
pub const FRONT_SEQ_NOTE: i32 = 19;
/// `#define FRONT_SEQ_PROB 0x14` (Core/Inc/Soundengine/MIDI/MidiMessages.h:460)
pub const FRONT_SEQ_PROB: i32 = 20;
/// `#define FRONT_SEQ_SET_ACTIVE_TRACK 0x15` (Core/Inc/Soundengine/MIDI/MidiMessages.h:461)
pub const FRONT_SEQ_SET_ACTIVE_TRACK: i32 = 21;
/// `#define FRONT_SEQ_FLAM 0x16` (Core/Inc/Soundengine/MIDI/MidiMessages.h:462)
pub const FRONT_SEQ_FLAM: i32 = 22;
/// `#define FRONT_SEQ_EUKLID_LENGTH 0x17` (Core/Inc/Soundengine/MIDI/MidiMessages.h:464)
pub const FRONT_SEQ_EUKLID_LENGTH: i32 = 23;
/// `#define FRONT_SEQ_EUKLID_STEPS 0x18` (Core/Inc/Soundengine/MIDI/MidiMessages.h:465)
pub const FRONT_SEQ_EUKLID_STEPS: i32 = 24;
/// `#define FRONT_SEQ_REQUEST_EUKLID_PARAMS 0x19` (Core/Inc/Soundengine/MIDI/MidiMessages.h:466)
pub const FRONT_SEQ_REQUEST_EUKLID_PARAMS: i32 = 25;
/// `#define FRONT_SEQ_SET_SHOWN_PATTERN 0x1A` (Core/Inc/Soundengine/MIDI/MidiMessages.h:467)
pub const FRONT_SEQ_SET_SHOWN_PATTERN: i32 = 26;
/// `#define FRONT_SEQ_REC_ON_OFF 0x1B` (Core/Inc/Soundengine/MIDI/MidiMessages.h:469)
pub const FRONT_SEQ_REC_ON_OFF: i32 = 27;
/// `#define FRONT_SEQ_SHIFT 0x1C` (Core/Inc/Soundengine/MIDI/MidiMessages.h:471)
pub const FRONT_SEQ_SHIFT: i32 = 28;
/// `#define FRONT_SEQ_CLEAR_TRACK 0x1f` (Core/Inc/Soundengine/MIDI/MidiMessages.h:474)
pub const FRONT_SEQ_CLEAR_TRACK: i32 = 31;
/// `#define FRONT_SEQ_COPY_TRACK 0x20` (Core/Inc/Soundengine/MIDI/MidiMessages.h:475)
pub const FRONT_SEQ_COPY_TRACK: i32 = 32;
/// `#define FRONT_SEQ_COPY_PATTERN 0x21` (Core/Inc/Soundengine/MIDI/MidiMessages.h:476)
pub const FRONT_SEQ_COPY_PATTERN: i32 = 33;
/// `#define FRONT_SEQ_SET_QUANT 0x22` (Core/Inc/Soundengine/MIDI/MidiMessages.h:477)
pub const FRONT_SEQ_SET_QUANT: i32 = 34;
/// `#define FRONT_SEQ_SET_AUTOM_TRACK 0x23` (Core/Inc/Soundengine/MIDI/MidiMessages.h:478)
pub const FRONT_SEQ_SET_AUTOM_TRACK: i32 = 35;
/// `#define FRONT_SEQ_SELECT_ACTIVE_STEP 0x24` (Core/Inc/Soundengine/MIDI/MidiMessages.h:479)
pub const FRONT_SEQ_SELECT_ACTIVE_STEP: i32 = 36;
/// `#define FRONT_SEQ_SHUFFLE 0x25` (Core/Inc/Soundengine/MIDI/MidiMessages.h:480)
pub const FRONT_SEQ_SHUFFLE: i32 = 37;
/// `#define FRONT_SEQ_TRACK_LENGTH 0x26` (Core/Inc/Soundengine/MIDI/MidiMessages.h:481)
pub const FRONT_SEQ_TRACK_LENGTH: i32 = 38;
/// `#define FRONT_SEQ_CLEAR_AUTOM 0x28` (Core/Inc/Soundengine/MIDI/MidiMessages.h:483)
pub const FRONT_SEQ_CLEAR_AUTOM: i32 = 40;
/// `#define FRONT_SEQ_POSX 0x29` (Core/Inc/Soundengine/MIDI/MidiMessages.h:485)
pub const FRONT_SEQ_POSX: i32 = 41;
/// `#define FRONT_SEQ_POSY 0x2a` (Core/Inc/Soundengine/MIDI/MidiMessages.h:486)
pub const FRONT_SEQ_POSY: i32 = 42;
/// `#define FRONT_SEQ_FLUX 0x2b` (Core/Inc/Soundengine/MIDI/MidiMessages.h:487)
pub const FRONT_SEQ_FLUX: i32 = 43;
/// `#define FRONT_SEQ_SOM_FREQ 0x2c` (Core/Inc/Soundengine/MIDI/MidiMessages.h:488)
pub const FRONT_SEQ_SOM_FREQ: i32 = 44;
/// `#define FRONT_SEQ_MIDI_CHAN 0x2d` (Core/Inc/Soundengine/MIDI/MidiMessages.h:489)
pub const FRONT_SEQ_MIDI_CHAN: i32 = 45;
/// `#define FRONT_SEQ_MIDI_MODE 0x2e` (Core/Inc/Soundengine/MIDI/MidiMessages.h:490)
pub const FRONT_SEQ_MIDI_MODE: i32 = 46;
/// `#define FRONT_SEQ_MIDI_ROUTING 0x2f` (Core/Inc/Soundengine/MIDI/MidiMessages.h:491)
pub const FRONT_SEQ_MIDI_ROUTING: i32 = 47;
/// `#define FRONT_SEQ_MIDI_TX_FILTER 0x30` (Core/Inc/Soundengine/MIDI/MidiMessages.h:492)
pub const FRONT_SEQ_MIDI_TX_FILTER: i32 = 48;
/// `#define FRONT_SEQ_MIDI_RX_FILTER 0x31` (Core/Inc/Soundengine/MIDI/MidiMessages.h:493)
pub const FRONT_SEQ_MIDI_RX_FILTER: i32 = 49;
/// `#define FRONT_SEQ_BAR_RESET_MODE 0x32` (Core/Inc/Soundengine/MIDI/MidiMessages.h:494)
pub const FRONT_SEQ_BAR_RESET_MODE: i32 = 50;
/// `#define FRONT_SEQ_ERASE_ON_OFF 0x33` (Core/Inc/Soundengine/MIDI/MidiMessages.h:495)
pub const FRONT_SEQ_ERASE_ON_OFF: i32 = 51;
/// `#define FRONT_SEQ_EUKLID_ROTATION 0x35` (Core/Inc/Soundengine/MIDI/MidiMessages.h:497)
pub const FRONT_SEQ_EUKLID_ROTATION: i32 = 53;
/// `#define FRONT_SEQ_TRIGGER_IN_PPQ 0x36` (Core/Inc/Soundengine/MIDI/MidiMessages.h:499)
pub const FRONT_SEQ_TRIGGER_IN_PPQ: i32 = 54;
/// `#define FRONT_SEQ_TRIGGER_OUT1_PPQ 0x37` (Core/Inc/Soundengine/MIDI/MidiMessages.h:500)
pub const FRONT_SEQ_TRIGGER_OUT1_PPQ: i32 = 55;
/// `#define FRONT_SEQ_TRIGGER_OUT2_PPQ 0x38` (Core/Inc/Soundengine/MIDI/MidiMessages.h:501)
pub const FRONT_SEQ_TRIGGER_OUT2_PPQ: i32 = 56;
/// `#define FRONT_SEQ_TRIGGER_GATE_MODE 0x39` (Core/Inc/Soundengine/MIDI/MidiMessages.h:502)
pub const FRONT_SEQ_TRIGGER_GATE_MODE: i32 = 57;
/// `#define EQ_ON_OFF 0x01` (Core/Inc/Soundengine/MIDI/MidiMessages.h:505)
pub const EQ_ON_OFF: i32 = 1;
/// `#define EQ_BASS_F 0x02` (Core/Inc/Soundengine/MIDI/MidiMessages.h:506)
pub const EQ_BASS_F: i32 = 2;
/// `#define EQ_TREB_F 0x03` (Core/Inc/Soundengine/MIDI/MidiMessages.h:507)
pub const EQ_TREB_F: i32 = 3;
/// `#define EQ_TREB_GAIN 0x04` (Core/Inc/Soundengine/MIDI/MidiMessages.h:508)
pub const EQ_TREB_GAIN: i32 = 4;
/// `#define EQ_BASS_GAIN 0x05` (Core/Inc/Soundengine/MIDI/MidiMessages.h:509)
pub const EQ_BASS_GAIN: i32 = 5;
/// `#define LIMIT_ENABLE 0x06` (Core/Inc/Soundengine/MIDI/MidiMessages.h:511)
pub const LIMIT_ENABLE: i32 = 6;
/// `#define LIMIT_ATT 0x07` (Core/Inc/Soundengine/MIDI/MidiMessages.h:512)
pub const LIMIT_ATT: i32 = 7;
/// `#define LIMIT_REL 0x08` (Core/Inc/Soundengine/MIDI/MidiMessages.h:513)
pub const LIMIT_REL: i32 = 8;
/// `#define LIMIT_MAX 0x09` (Core/Inc/Soundengine/MIDI/MidiMessages.h:514)
pub const LIMIT_MAX: i32 = 9;
/// `#define LIMIT_MIN 0x0A` (Core/Inc/Soundengine/MIDI/MidiMessages.h:515)
pub const LIMIT_MIN: i32 = 10;
/// `#define SYSEX_INACTIVE 0x00` (Core/Inc/Soundengine/MIDI/MidiMessages.h:520)
pub const SYSEX_INACTIVE: i32 = 0;
/// `#define SYSEX_REQUEST_STEP_DATA 0x01` (Core/Inc/Soundengine/MIDI/MidiMessages.h:521)
pub const SYSEX_REQUEST_STEP_DATA: i32 = 1;
/// `#define SYSEX_RECEIVE_STEP_DATA 0x02` (Core/Inc/Soundengine/MIDI/MidiMessages.h:522)
pub const SYSEX_RECEIVE_STEP_DATA: i32 = 2;
/// `#define SYSEX_RECEIVE_PAT_LEN_DATA 0x06` (Core/Inc/Soundengine/MIDI/MidiMessages.h:526)
pub const SYSEX_RECEIVE_PAT_LEN_DATA: i32 = 6;
/// `#define SYSEX_ACTIVE_MODE_NONE 0x7f` (Core/Inc/Soundengine/MIDI/MidiMessages.h:527)
pub const SYSEX_ACTIVE_MODE_NONE: i32 = 127;

// ---- enums
/// C: `enum MidiSource` (Core/Inc/Soundengine/MIDI/MidiMessages.h:20), 1 bytes, underlying type `u8`.
/// Representation: integer type alias plus one `pub const` per enumerator.
pub type MidiSource = u8;
pub const midiSourceMIDI: MidiSource = 0;
pub const midiSourceUSB: MidiSource = 1;
/// C: `enum {...}` (Core/Inc/Soundengine/MIDI/MidiMessages.h:83), 1 bytes, underlying type `u8`.
/// Representation: integer type alias plus one `pub const` per enumerator.
pub type ParamEnumsDefinition = u8;
pub const CC_BANK_CHANGE: ParamEnumsDefinition = 0;
pub const CC_MODWHEEL: ParamEnumsDefinition = 1;
pub const OSC_WAVE_DRUM1: ParamEnumsDefinition = 2;
pub const OSC_WAVE_DRUM2: ParamEnumsDefinition = 3;
pub const OSC_WAVE_DRUM3: ParamEnumsDefinition = 4;
pub const OSC_WAVE_SNARE: ParamEnumsDefinition = 5;
pub const NRPN_DATA_ENTRY_COARSE: ParamEnumsDefinition = 6;
pub const CYM_WAVE1: ParamEnumsDefinition = 7;
pub const WAVE1_HH: ParamEnumsDefinition = 8;
pub const F_OSC1_COARSE: ParamEnumsDefinition = 9;
pub const F_OSC1_FINE: ParamEnumsDefinition = 10;
pub const F_OSC2_COARSE: ParamEnumsDefinition = 11;
pub const F_OSC2_FINE: ParamEnumsDefinition = 12;
pub const F_OSC3_COARSE: ParamEnumsDefinition = 13;
pub const F_OSC3_FINE: ParamEnumsDefinition = 14;
pub const F_OSC4_COARSE: ParamEnumsDefinition = 15;
pub const F_OSC4_FINE: ParamEnumsDefinition = 16;
pub const F_OSC5_COARSE: ParamEnumsDefinition = 17;
pub const F_OSC5_FINE: ParamEnumsDefinition = 18;
pub const F_OSC6_COARSE: ParamEnumsDefinition = 19;
pub const F_OSC6_FINE: ParamEnumsDefinition = 20;
pub const MOD_WAVE_DRUM1: ParamEnumsDefinition = 21;
pub const MOD_WAVE_DRUM2: ParamEnumsDefinition = 22;
pub const MOD_WAVE_DRUM3: ParamEnumsDefinition = 23;
pub const CYM_WAVE2: ParamEnumsDefinition = 24;
pub const CYM_WAVE3: ParamEnumsDefinition = 25;
pub const WAVE2_HH: ParamEnumsDefinition = 26;
pub const WAVE3_HH: ParamEnumsDefinition = 27;
pub const SNARE_NOISE_F: ParamEnumsDefinition = 28;
pub const SNARE_MIX: ParamEnumsDefinition = 29;
pub const CYM_MOD_OSC_F1: ParamEnumsDefinition = 30;
pub const CYM_MOD_OSC_F2: ParamEnumsDefinition = 31;
pub const CYM_MOD_OSC_GAIN1: ParamEnumsDefinition = 32;
pub const CYM_MOD_OSC_GAIN2: ParamEnumsDefinition = 33;
pub const MOD_OSC_F1: ParamEnumsDefinition = 34;
pub const MOD_OSC_F2: ParamEnumsDefinition = 35;
pub const MOD_OSC_GAIN1: ParamEnumsDefinition = 36;
pub const MOD_OSC_GAIN2: ParamEnumsDefinition = 37;
pub const FILTER_FREQ_DRUM1: ParamEnumsDefinition = 38;
pub const FILTER_FREQ_DRUM2: ParamEnumsDefinition = 39;
pub const FILTER_FREQ_DRUM3: ParamEnumsDefinition = 40;
pub const SNARE_FILTER_F: ParamEnumsDefinition = 41;
pub const CYM_FIL_FREQ: ParamEnumsDefinition = 42;
pub const HAT_FILTER_F: ParamEnumsDefinition = 43;
pub const RESO_DRUM1: ParamEnumsDefinition = 44;
pub const RESO_DRUM2: ParamEnumsDefinition = 45;
pub const RESO_DRUM3: ParamEnumsDefinition = 46;
pub const SNARE_RESO: ParamEnumsDefinition = 47;
pub const CYM_RESO: ParamEnumsDefinition = 48;
pub const HAT_RESO: ParamEnumsDefinition = 49;
pub const VELOA1: ParamEnumsDefinition = 50;
pub const VELOD1: ParamEnumsDefinition = 51;
pub const VELOA2: ParamEnumsDefinition = 52;
pub const VELOD2: ParamEnumsDefinition = 53;
pub const VELOA3: ParamEnumsDefinition = 54;
pub const VELOD3: ParamEnumsDefinition = 55;
pub const VELOA4: ParamEnumsDefinition = 56;
pub const VELOD4: ParamEnumsDefinition = 57;
pub const VELOA5: ParamEnumsDefinition = 58;
pub const VELOD5: ParamEnumsDefinition = 59;
pub const VELOA6: ParamEnumsDefinition = 60;
pub const VELOD6: ParamEnumsDefinition = 61;
pub const VELOD6_OPEN: ParamEnumsDefinition = 62;
pub const VOL_SLOPE1: ParamEnumsDefinition = 63;
pub const VOL_SLOPE2: ParamEnumsDefinition = 64;
pub const VOL_SLOPE3: ParamEnumsDefinition = 65;
pub const EG_SNARE1_SLOPE: ParamEnumsDefinition = 66;
pub const CYM_SLOPE: ParamEnumsDefinition = 67;
pub const VOL_SLOPE6: ParamEnumsDefinition = 68;
pub const REPEAT1: ParamEnumsDefinition = 69;
pub const CYM_REPEAT: ParamEnumsDefinition = 70;
pub const PITCHD1: ParamEnumsDefinition = 71;
pub const PITCHD2: ParamEnumsDefinition = 72;
pub const PITCHD3: ParamEnumsDefinition = 73;
pub const PITCHD4: ParamEnumsDefinition = 74;
pub const MODAMNT1: ParamEnumsDefinition = 75;
pub const MODAMNT2: ParamEnumsDefinition = 76;
pub const MODAMNT3: ParamEnumsDefinition = 77;
pub const MODAMNT4: ParamEnumsDefinition = 78;
pub const PITCH_SLOPE1: ParamEnumsDefinition = 79;
pub const PITCH_SLOPE2: ParamEnumsDefinition = 80;
pub const PITCH_SLOPE3: ParamEnumsDefinition = 81;
pub const PITCH_SLOPE4: ParamEnumsDefinition = 82;
pub const FMAMNT1: ParamEnumsDefinition = 83;
pub const FMDTN1: ParamEnumsDefinition = 84;
pub const FMAMNT2: ParamEnumsDefinition = 85;
pub const FMDTN2: ParamEnumsDefinition = 86;
pub const FMAMNT3: ParamEnumsDefinition = 87;
pub const FMDTN3: ParamEnumsDefinition = 88;
pub const VOL1: ParamEnumsDefinition = 89;
pub const VOL2: ParamEnumsDefinition = 90;
pub const VOL3: ParamEnumsDefinition = 91;
pub const VOL4: ParamEnumsDefinition = 92;
pub const VOL5: ParamEnumsDefinition = 93;
pub const VOL6: ParamEnumsDefinition = 94;
pub const PAN1: ParamEnumsDefinition = 95;
pub const PAN2: ParamEnumsDefinition = 96;
pub const PAN3: ParamEnumsDefinition = 97;
pub const NRPN_FINE: ParamEnumsDefinition = 98;
pub const NRPN_COARSE: ParamEnumsDefinition = 99;
pub const PAN4: ParamEnumsDefinition = 100;
pub const PAN5: ParamEnumsDefinition = 101;
pub const PAN6: ParamEnumsDefinition = 102;
pub const OSC1_DIST: ParamEnumsDefinition = 103;
pub const OSC2_DIST: ParamEnumsDefinition = 104;
pub const OSC3_DIST: ParamEnumsDefinition = 105;
pub const SNARE_DISTORTION: ParamEnumsDefinition = 106;
pub const CYMBAL_DISTORTION: ParamEnumsDefinition = 107;
pub const HAT_DISTORTION: ParamEnumsDefinition = 108;
pub const VOICE_DECIMATION1: ParamEnumsDefinition = 109;
pub const VOICE_DECIMATION2: ParamEnumsDefinition = 110;
pub const VOICE_DECIMATION3: ParamEnumsDefinition = 111;
pub const VOICE_DECIMATION4: ParamEnumsDefinition = 112;
pub const VOICE_DECIMATION5: ParamEnumsDefinition = 113;
pub const VOICE_DECIMATION6: ParamEnumsDefinition = 114;
pub const VOICE_DECIMATION_ALL: ParamEnumsDefinition = 115;
pub const FREQ_LFO1: ParamEnumsDefinition = 116;
pub const FREQ_LFO2: ParamEnumsDefinition = 117;
pub const FREQ_LFO3: ParamEnumsDefinition = 118;
pub const FREQ_LFO4: ParamEnumsDefinition = 119;
pub const FREQ_LFO5: ParamEnumsDefinition = 120;
pub const FREQ_LFO6: ParamEnumsDefinition = 121;
pub const AMOUNT_LFO1: ParamEnumsDefinition = 122;
pub const AMOUNT_LFO2: ParamEnumsDefinition = 123;
pub const AMOUNT_LFO3: ParamEnumsDefinition = 124;
pub const AMOUNT_LFO4: ParamEnumsDefinition = 125;
pub const AMOUNT_LFO5: ParamEnumsDefinition = 126;
pub const AMOUNT_LFO6: ParamEnumsDefinition = 127;
pub const RESERVED4: ParamEnumsDefinition = 128;
/// C: `enum {...}` (Core/Inc/Soundengine/MIDI/MidiMessages.h:240), 1 bytes, underlying type `u8`.
/// Representation: integer type alias plus one `pub const` per enumerator.
pub type Param2EnumsDefinition = u8;
pub const CC2_FILTER_DRIVE_1: Param2EnumsDefinition = 0;
pub const CC2_FILTER_DRIVE_2: Param2EnumsDefinition = 1;
pub const CC2_FILTER_DRIVE_3: Param2EnumsDefinition = 2;
pub const CC2_FILTER_DRIVE_4: Param2EnumsDefinition = 3;
pub const CC2_FILTER_DRIVE_5: Param2EnumsDefinition = 4;
pub const CC2_FILTER_DRIVE_6: Param2EnumsDefinition = 5;
pub const CC2_MIX_MOD_1: Param2EnumsDefinition = 6;
pub const CC2_MIX_MOD_2: Param2EnumsDefinition = 7;
pub const CC2_MIX_MOD_3: Param2EnumsDefinition = 8;
pub const CC2_VOLUME_MOD_ON_OFF1: Param2EnumsDefinition = 9;
pub const CC2_VOLUME_MOD_ON_OFF2: Param2EnumsDefinition = 10;
pub const CC2_VOLUME_MOD_ON_OFF3: Param2EnumsDefinition = 11;
pub const CC2_VOLUME_MOD_ON_OFF4: Param2EnumsDefinition = 12;
pub const CC2_VOLUME_MOD_ON_OFF5: Param2EnumsDefinition = 13;
pub const CC2_VOLUME_MOD_ON_OFF6: Param2EnumsDefinition = 14;
pub const CC2_VELO_MOD_AMT_1: Param2EnumsDefinition = 15;
pub const CC2_VELO_MOD_AMT_2: Param2EnumsDefinition = 16;
pub const CC2_VELO_MOD_AMT_3: Param2EnumsDefinition = 17;
pub const CC2_VELO_MOD_AMT_4: Param2EnumsDefinition = 18;
pub const CC2_VELO_MOD_AMT_5: Param2EnumsDefinition = 19;
pub const CC2_VELO_MOD_AMT_6: Param2EnumsDefinition = 20;
pub const CC2_VEL_DEST_1: Param2EnumsDefinition = 21;
pub const CC2_VEL_DEST_2: Param2EnumsDefinition = 22;
pub const CC2_VEL_DEST_3: Param2EnumsDefinition = 23;
pub const CC2_VEL_DEST_4: Param2EnumsDefinition = 24;
pub const CC2_VEL_DEST_5: Param2EnumsDefinition = 25;
pub const CC2_VEL_DEST_6: Param2EnumsDefinition = 26;
pub const CC2_WAVE_LFO1: Param2EnumsDefinition = 27;
pub const CC2_WAVE_LFO2: Param2EnumsDefinition = 28;
pub const CC2_WAVE_LFO3: Param2EnumsDefinition = 29;
pub const CC2_WAVE_LFO4: Param2EnumsDefinition = 30;
pub const CC2_WAVE_LFO5: Param2EnumsDefinition = 31;
pub const CC2_WAVE_LFO6: Param2EnumsDefinition = 32;
pub const CC2_VOICE_LFO1: Param2EnumsDefinition = 33;
pub const CC2_VOICE_LFO2: Param2EnumsDefinition = 34;
pub const CC2_VOICE_LFO3: Param2EnumsDefinition = 35;
pub const CC2_VOICE_LFO4: Param2EnumsDefinition = 36;
pub const CC2_VOICE_LFO5: Param2EnumsDefinition = 37;
pub const CC2_VOICE_LFO6: Param2EnumsDefinition = 38;
pub const CC2_TARGET_LFO1: Param2EnumsDefinition = 39;
pub const CC2_TARGET_LFO2: Param2EnumsDefinition = 40;
pub const CC2_TARGET_LFO3: Param2EnumsDefinition = 41;
pub const CC2_TARGET_LFO4: Param2EnumsDefinition = 42;
pub const CC2_TARGET_LFO5: Param2EnumsDefinition = 43;
pub const CC2_TARGET_LFO6: Param2EnumsDefinition = 44;
pub const CC2_RETRIGGER_LFO1: Param2EnumsDefinition = 45;
pub const CC2_RETRIGGER_LFO2: Param2EnumsDefinition = 46;
pub const CC2_RETRIGGER_LFO3: Param2EnumsDefinition = 47;
pub const CC2_RETRIGGER_LFO4: Param2EnumsDefinition = 48;
pub const CC2_RETRIGGER_LFO5: Param2EnumsDefinition = 49;
pub const CC2_RETRIGGER_LFO6: Param2EnumsDefinition = 50;
pub const CC2_SYNC_LFO1: Param2EnumsDefinition = 51;
pub const CC2_SYNC_LFO2: Param2EnumsDefinition = 52;
pub const CC2_SYNC_LFO3: Param2EnumsDefinition = 53;
pub const CC2_SYNC_LFO4: Param2EnumsDefinition = 54;
pub const CC2_SYNC_LFO5: Param2EnumsDefinition = 55;
pub const CC2_SYNC_LFO6: Param2EnumsDefinition = 56;
pub const CC2_OFFSET_LFO1: Param2EnumsDefinition = 57;
pub const CC2_OFFSET_LFO2: Param2EnumsDefinition = 58;
pub const CC2_OFFSET_LFO3: Param2EnumsDefinition = 59;
pub const CC2_OFFSET_LFO4: Param2EnumsDefinition = 60;
pub const CC2_OFFSET_LFO5: Param2EnumsDefinition = 61;
pub const CC2_OFFSET_LFO6: Param2EnumsDefinition = 62;
pub const CC2_FILTER_TYPE_1: Param2EnumsDefinition = 63;
pub const CC2_FILTER_TYPE_2: Param2EnumsDefinition = 64;
pub const CC2_FILTER_TYPE_3: Param2EnumsDefinition = 65;
pub const CC2_FILTER_TYPE_4: Param2EnumsDefinition = 66;
pub const CC2_FILTER_TYPE_5: Param2EnumsDefinition = 67;
pub const CC2_FILTER_TYPE_6: Param2EnumsDefinition = 68;
pub const CC2_TRANS1_VOL: Param2EnumsDefinition = 69;
pub const CC2_TRANS2_VOL: Param2EnumsDefinition = 70;
pub const CC2_TRANS3_VOL: Param2EnumsDefinition = 71;
pub const CC2_TRANS4_VOL: Param2EnumsDefinition = 72;
pub const CC2_TRANS5_VOL: Param2EnumsDefinition = 73;
pub const CC2_TRANS6_VOL: Param2EnumsDefinition = 74;
pub const CC2_TRANS1_WAVE: Param2EnumsDefinition = 75;
pub const CC2_TRANS2_WAVE: Param2EnumsDefinition = 76;
pub const CC2_TRANS3_WAVE: Param2EnumsDefinition = 77;
pub const CC2_TRANS4_WAVE: Param2EnumsDefinition = 78;
pub const CC2_TRANS5_WAVE: Param2EnumsDefinition = 79;
pub const CC2_TRANS6_WAVE: Param2EnumsDefinition = 80;
pub const CC2_TRANS1_FREQ: Param2EnumsDefinition = 81;
pub const CC2_TRANS2_FREQ: Param2EnumsDefinition = 82;
pub const CC2_TRANS3_FREQ: Param2EnumsDefinition = 83;
pub const CC2_TRANS4_FREQ: Param2EnumsDefinition = 84;
pub const CC2_TRANS5_FREQ: Param2EnumsDefinition = 85;
pub const CC2_TRANS6_FREQ: Param2EnumsDefinition = 86;
pub const CC2_AUDIO_OUT1: Param2EnumsDefinition = 87;
pub const CC2_AUDIO_OUT2: Param2EnumsDefinition = 88;
pub const CC2_AUDIO_OUT3: Param2EnumsDefinition = 89;
pub const CC2_AUDIO_OUT4: Param2EnumsDefinition = 90;
pub const CC2_AUDIO_OUT5: Param2EnumsDefinition = 91;
pub const CC2_AUDIO_OUT6: Param2EnumsDefinition = 92;
pub const CC2_UNUSED1: Param2EnumsDefinition = 93;
pub const CC2_UNUSED2: Param2EnumsDefinition = 94;
pub const CC2_UNUSED3: Param2EnumsDefinition = 95;
pub const CC2_UNUSED4: Param2EnumsDefinition = 96;
pub const CC2_UNUSED5: Param2EnumsDefinition = 97;
pub const CC2_UNUSED6: Param2EnumsDefinition = 98;
pub const CC2_UNUSED7: Param2EnumsDefinition = 99;
pub const CC2_PWM_1: Param2EnumsDefinition = 100;
pub const CC2_PWM_2: Param2EnumsDefinition = 101;
pub const CC2_PWM_3: Param2EnumsDefinition = 102;
pub const CC2_PWM_4: Param2EnumsDefinition = 103;
pub const CC2_PWM_5: Param2EnumsDefinition = 104;
pub const CC2_PWM_6: Param2EnumsDefinition = 105;
pub const CC2_FX_TYPE: Param2EnumsDefinition = 106;
pub const CC2_FX_ROUTING: Param2EnumsDefinition = 107;
pub const CC2_FX_AMOUNT: Param2EnumsDefinition = 108;
pub const CC2_FX_DIST_TYPE: Param2EnumsDefinition = 109;
pub const CC2_FX_P1: Param2EnumsDefinition = 110;
pub const CC2_FX_P2: Param2EnumsDefinition = 111;
pub const CC2_FX_P3: Param2EnumsDefinition = 112;
pub const CC2_FX_P4: Param2EnumsDefinition = 113;
pub const CC2_FX_RATIO: Param2EnumsDefinition = 114;
pub const CC2_FX_WAVE: Param2EnumsDefinition = 115;
pub const CC2_FX_REVERB_TYPE: Param2EnumsDefinition = 116;
pub const CC2_FX_DELAY_TYPE: Param2EnumsDefinition = 117;
pub const CC2_MUTE_1: Param2EnumsDefinition = 200;
pub const CC2_MUTE_2: Param2EnumsDefinition = 201;
pub const CC2_MUTE_3: Param2EnumsDefinition = 202;
pub const CC2_MUTE_4: Param2EnumsDefinition = 203;
pub const CC2_MUTE_5: Param2EnumsDefinition = 204;
pub const CC2_MUTE_6: Param2EnumsDefinition = 205;
pub const CC2_MUTE_7: Param2EnumsDefinition = 206;

// ---- types
/// C: `struct MidiBits { enum MidiSource source:1; unsigned sysxbyte:1; unsigned length:2; unsigned :4; };`
/// (Core/Inc/Soundengine/MIDI/MidiMessages.h:25), 4 bytes (the bit-fields live in an `unsigned int` unit).
///
/// Representation: the 32 bit storage unit `bits`; `source` = bit 0, `sysxbyte` = bit 1, `length` = bits 2..3
/// (GCC little endian allocation). Use the accessors.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct MidiBits {
    pub bits: u32,
}
impl MidiBits {
    pub const ZERO: Self = Self { bits: 0 };
    /// `enum MidiSource source:1;`
    #[inline(always)]
    pub fn source(&self) -> u32 { self.bits & 1 }
    #[inline(always)]
    pub fn set_source(&mut self, v: u32) { self.bits = (self.bits & !1) | (v & 1) }
    /// `unsigned sysxbyte:1;`
    #[inline(always)]
    pub fn sysxbyte(&self) -> u32 { (self.bits >> 1) & 1 }
    #[inline(always)]
    pub fn set_sysxbyte(&mut self, v: u32) { self.bits = (self.bits & !2) | ((v & 1) << 1) }
    /// `unsigned length:2;`
    #[inline(always)]
    pub fn length(&self) -> u32 { (self.bits >> 2) & 3 }
    #[inline(always)]
    pub fn set_length(&mut self, v: u32) { self.bits = (self.bits & !0xc) | ((v & 3) << 2) }
}

/// C: `struct MidiStruct` (Core/Inc/Soundengine/MIDI/MidiMessages.h:34), 8 bytes.
/// C typedef name `MidiMsg`; the struct tag `MidiStruct` is a type alias below.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MidiMsg {
    /// `uint8_t status;`
    pub status: u8,
    /// `uint8_t lastNonSysStatus;`
    pub lastNonSysStatus: u8,
    /// `uint8_t data1;`
    pub data1: u8,
    /// `uint8_t data2;`
    pub data2: u8,
    /// `struct MidiBits bits;`
    pub bits: MidiBits,
}
/// C struct tag of `MidiMsg`.
pub type MidiStruct = MidiMsg;
impl MidiMsg {
    /// All-zero value (C static zero initialisation).
    pub const ZERO: Self = Self { status: 0, lastNonSysStatus: 0, data1: 0, data2: 0, bits: MidiBits::ZERO };
}
impl Default for MidiMsg {
    fn default() -> Self {
        Self::ZERO
    }
}
