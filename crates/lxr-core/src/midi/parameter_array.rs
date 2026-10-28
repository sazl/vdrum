//! Port of `Core/Inc/Soundengine/MIDI/ParameterArray.h`, `Core/Src/Soundengine/MIDI/ParameterArray.c`.
//!
//! Porting group: P3. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;

// ---- #define constants
/// `#define TYPE_UINT8 0` (Core/Inc/Soundengine/MIDI/ParameterArray.h:18)
pub const TYPE_UINT8: i32 = 0;
/// `#define TYPE_FLT 1` (Core/Inc/Soundengine/MIDI/ParameterArray.h:19)
pub const TYPE_FLT: i32 = 1;
/// `#define TYPE_SPECIAL_F 2` (Core/Inc/Soundengine/MIDI/ParameterArray.h:20)
pub const TYPE_SPECIAL_F: i32 = 2;
/// `#define TYPE_UINT32 3` (Core/Inc/Soundengine/MIDI/ParameterArray.h:21)
pub const TYPE_UINT32: i32 = 3;
/// `#define TYPE_SPECIAL_P 4` (Core/Inc/Soundengine/MIDI/ParameterArray.h:22)
pub const TYPE_SPECIAL_P: i32 = 4;
/// `#define TYPE_SPECIAL_FILTER_F 5` (Core/Inc/Soundengine/MIDI/ParameterArray.h:23)
pub const TYPE_SPECIAL_FILTER_F: i32 = 5;

// ---- types
/// C: `typedef union { float flt; uint32_t itg; } ptrValue;` (Core/Inc/Soundengine/MIDI/ParameterArray.h:367), 4 bytes.
///
/// Representation: the union's storage as the raw 32 bit pattern (`itg`); `flt` is accessed through
/// `flt()`/`set_flt()` (`f32::from_bits`/`to_bits`), which is exactly what the C union does.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct ptrValue {
    /// `uint32_t itg;`
    pub itg: u32,
}
impl ptrValue {
    pub const ZERO: Self = Self { itg: 0 };
    /// `float flt;` (read)
    #[inline(always)]
    pub fn flt(&self) -> f32 { f32::from_bits(self.itg) }
    /// `float flt;` (write)
    #[inline(always)]
    pub fn set_flt(&mut self, v: f32) { self.itg = v.to_bits() }
}

/// C: `struct ParameterStruct` (Core/Inc/Soundengine/MIDI/ParameterArray.h:373), 8 bytes.
/// C typedef name `Parameter`; the struct tag `ParameterStruct` is a type alias below.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Parameter {
    /// `void* ptr;`
    ///
    /// C pointer: stored as the address in the reference firmware memory map (`CPtr`, 0 = NULL), see `memmap`.
    pub ptr: CPtr,
    /// `uint8_t type;`
    pub r#type: u8,
}
/// C struct tag of `Parameter`.
pub type ParameterStruct = Parameter;
impl Parameter {
    /// All-zero value (C static zero initialisation).
    pub const ZERO: Self = Self { ptr: 0, r#type: 0 };
}
impl Default for Parameter {
    fn default() -> Self {
        Self::ZERO
    }
}

// ---- functions that use globals (methods on `Lxr`)
impl Lxr {
    /// C: `IN_ITCM_RAM void paramArray_setParameter(uint16_t idx, ptrValue newValue)`
    ///
    /// Source: `Core/Src/Soundengine/MIDI/ParameterArray.c:32`. Reference build: out-of-line in ITCM. Group P3.
    /// Globals: `activeParameters`, `parameterArray`.
    /// Calls: `Lxr::midiParser_ccHandler`.
    pub fn paramArray_setParameter(&mut self, idx: u16, newValue: ptrValue) {
        let _ = newValue; // unused in the compiled (`#if 1`) variant
        let p = self.parameterArray[idx as usize];

        // **PATROT avoid setting this if it's not set to something good
        if p.ptr == 0 {
            return;
        }

        // `msg` is an uninitialised local in C; midiParser_ccHandler only reads status/data1/data2.
        let mut msg = MidiMsg::ZERO;
        let val = self.activeParameters[idx as usize];

        if idx < 128 {
            msg.status = MIDI_CC as u8;
            msg.data1 = idx as u8;
            msg.data2 = val;

            msg.data1 = msg.data1.wrapping_add(1);
            msg.data1 &= 0x7f;
            self.midiParser_ccHandler(msg, 0, 0, 0.0);
        } else {
            msg.status = crate::front_panel_parser::CC_2 as u8;
            msg.data1 = idx.wrapping_sub(128) as u8; // stupid offset only in CC not CC2
            msg.data2 = val;

            msg.data1 &= 0x7f;
            self.midiParser_ccHandler(msg, 0, 0, 0.0);
        }
    }

    /// C: `void parameterArray_init()`
    ///
    /// Source: `Core/Src/Soundengine/MIDI/ParameterArray.c:108`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `cymbalVoice`, `fxBus`, `hatVoice`, `mixer_audioRouting`, `mixer_decimation_rate`, `parameterArray`, `snareVoice`, `velocityModulators`, `voiceArray`.
    pub fn parameterArray_init(&mut self) {
        // Generated from the C source by tools/gen_paramarray.py: every `&global.field` is the field address in
        // the reference ELF (the `CPtr` convention of `memmap`).
        self.parameterArray[PAR_OSC_WAVE_DRUM1 as usize].ptr = 0x20009648; // &voiceArray[0].osc.waveform
        self.parameterArray[PAR_OSC_WAVE_DRUM1 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_OSC_WAVE_DRUM2 as usize].ptr = 0x20009908; // &voiceArray[1].osc.waveform
        self.parameterArray[PAR_OSC_WAVE_DRUM2 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_OSC_WAVE_DRUM3 as usize].ptr = 0x20009bc8; // &voiceArray[2].osc.waveform
        self.parameterArray[PAR_OSC_WAVE_DRUM3 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_OSC_WAVE_SNARE as usize].ptr = 0x2001a010; // &snareVoice.osc.waveform
        self.parameterArray[PAR_OSC_WAVE_SNARE as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_WAVE1_CYM as usize].ptr = 0x200094c8; // &cymbalVoice.osc.waveform
        self.parameterArray[PAR_WAVE1_CYM as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_WAVE1_HH as usize].ptr = 0x20019e88; // &hatVoice.osc.waveform
        self.parameterArray[PAR_WAVE1_HH as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_COARSE1 as usize].ptr = 0x20009654; // &voiceArray[0].osc.modNodeValue
        self.parameterArray[PAR_COARSE1 as usize].r#type = TYPE_SPECIAL_F as u8;
        self.parameterArray[PAR_FINE1 as usize].ptr = 0x20009654; // &voiceArray[0].osc.modNodeValue
        self.parameterArray[PAR_FINE1 as usize].r#type = TYPE_SPECIAL_F as u8;
        self.parameterArray[PAR_COARSE2 as usize].ptr = 0x20009914; // &voiceArray[1].osc.modNodeValue
        self.parameterArray[PAR_COARSE2 as usize].r#type = TYPE_SPECIAL_F as u8;
        self.parameterArray[PAR_FINE2 as usize].ptr = 0x20009914; // &voiceArray[1].osc.modNodeValue
        self.parameterArray[PAR_FINE2 as usize].r#type = TYPE_SPECIAL_F as u8;
        self.parameterArray[PAR_COARSE3 as usize].ptr = 0x20009bd4; // &voiceArray[2].osc.modNodeValue
        self.parameterArray[PAR_COARSE3 as usize].r#type = TYPE_SPECIAL_F as u8;
        self.parameterArray[PAR_FINE3 as usize].ptr = 0x20009bd4; // &voiceArray[2].osc.modNodeValue
        self.parameterArray[PAR_FINE3 as usize].r#type = TYPE_SPECIAL_F as u8;
        self.parameterArray[PAR_COARSE4 as usize].ptr = 0x2001a01c; // &snareVoice.osc.modNodeValue
        self.parameterArray[PAR_COARSE4 as usize].r#type = TYPE_SPECIAL_F as u8;
        self.parameterArray[PAR_FINE4 as usize].ptr = 0x2001a01c; // &snareVoice.osc.modNodeValue
        self.parameterArray[PAR_FINE4 as usize].r#type = TYPE_SPECIAL_F as u8;
        self.parameterArray[PAR_COARSE5 as usize].ptr = 0x200094d4; // &cymbalVoice.osc.modNodeValue
        self.parameterArray[PAR_COARSE5 as usize].r#type = TYPE_SPECIAL_F as u8;
        self.parameterArray[PAR_FINE5 as usize].ptr = 0x200094d4; // &cymbalVoice.osc.modNodeValue
        self.parameterArray[PAR_FINE5 as usize].r#type = TYPE_SPECIAL_F as u8;
        self.parameterArray[PAR_COARSE6 as usize].ptr = 0x20019e94; // &hatVoice.osc.modNodeValue
        self.parameterArray[PAR_COARSE6 as usize].r#type = TYPE_SPECIAL_F as u8;
        self.parameterArray[PAR_FINE6 as usize].ptr = 0x20019e94; // &hatVoice.osc.modNodeValue
        self.parameterArray[PAR_FINE6 as usize].r#type = TYPE_SPECIAL_F as u8;
        self.parameterArray[PAR_MOD_WAVE_DRUM1 as usize].ptr = 0x20009670; // &voiceArray[0].modOsc.waveform
        self.parameterArray[PAR_MOD_WAVE_DRUM1 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_MOD_WAVE_DRUM2 as usize].ptr = 0x20009930; // &voiceArray[1].modOsc.waveform
        self.parameterArray[PAR_MOD_WAVE_DRUM2 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_MOD_WAVE_DRUM3 as usize].ptr = 0x20009bf0; // &voiceArray[2].modOsc.waveform
        self.parameterArray[PAR_MOD_WAVE_DRUM3 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_WAVE2_CYM as usize].ptr = 0x200094f0; // &cymbalVoice.modOsc.waveform
        self.parameterArray[PAR_WAVE2_CYM as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_WAVE3_CYM as usize].ptr = 0x20009518; // &cymbalVoice.modOsc2.waveform
        self.parameterArray[PAR_WAVE3_CYM as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_WAVE2_HH as usize].ptr = 0x20019eb0; // &hatVoice.modOsc.waveform
        self.parameterArray[PAR_WAVE2_HH as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_WAVE3_HH as usize].ptr = 0x20019ed8; // &hatVoice.modOsc2.waveform
        self.parameterArray[PAR_WAVE3_HH as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_NOISE_FREQ1 as usize].ptr = 0x2001a044; // &snareVoice.noiseOsc.modNodeValue
        self.parameterArray[PAR_NOISE_FREQ1 as usize].r#type = TYPE_SPECIAL_F as u8;
        self.parameterArray[PAR_MIX1 as usize].ptr = 0x2001a064; // &snareVoice.mix
        self.parameterArray[PAR_MIX1 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_MOD_OSC_F1_CYM as usize].ptr = 0x200094fc; // &cymbalVoice.modOsc.modNodeValue
        self.parameterArray[PAR_MOD_OSC_F1_CYM as usize].r#type = TYPE_SPECIAL_F as u8;
        self.parameterArray[PAR_MOD_OSC_F2_CYM as usize].ptr = 0x20009524; // &cymbalVoice.modOsc2.modNodeValue
        self.parameterArray[PAR_MOD_OSC_F2_CYM as usize].r#type = TYPE_SPECIAL_F as u8;
        self.parameterArray[PAR_MOD_OSC_GAIN1_CYM as usize].ptr = 0x20009530; // &cymbalVoice.fmModAmount1
        self.parameterArray[PAR_MOD_OSC_GAIN1_CYM as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_MOD_OSC_GAIN2_CYM as usize].ptr = 0x20009534; // &cymbalVoice.fmModAmount2
        self.parameterArray[PAR_MOD_OSC_GAIN2_CYM as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_MOD_OSC_F1 as usize].ptr = 0x20019ebc; // &hatVoice.modOsc.modNodeValue
        self.parameterArray[PAR_MOD_OSC_F1 as usize].r#type = TYPE_SPECIAL_F as u8;
        self.parameterArray[PAR_MOD_OSC_F2 as usize].ptr = 0x20019ee4; // &hatVoice.modOsc2.modNodeValue
        self.parameterArray[PAR_MOD_OSC_F2 as usize].r#type = TYPE_SPECIAL_F as u8;
        self.parameterArray[PAR_MOD_OSC_GAIN1 as usize].ptr = 0x20019ef0; // &hatVoice.fmModAmount1
        self.parameterArray[PAR_MOD_OSC_GAIN1 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_MOD_OSC_GAIN2 as usize].ptr = 0x20019ef4; // &hatVoice.fmModAmount2
        self.parameterArray[PAR_MOD_OSC_GAIN2 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_FILTER_FREQ_1 as usize].ptr = 0x20009858; // &voiceArray[0].filter.f
        self.parameterArray[PAR_FILTER_FREQ_1 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_FILTER_FREQ_2 as usize].ptr = 0x20009b18; // &voiceArray[1].filter.f
        self.parameterArray[PAR_FILTER_FREQ_2 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_FILTER_FREQ_3 as usize].ptr = 0x20009dd8; // &voiceArray[2].filter.f
        self.parameterArray[PAR_FILTER_FREQ_3 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_FILTER_FREQ_4 as usize].ptr = 0x2001a074; // &snareVoice.filter.f
        self.parameterArray[PAR_FILTER_FREQ_4 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_FILTER_FREQ_5 as usize].ptr = 0x20009550; // &cymbalVoice.filter.f
        self.parameterArray[PAR_FILTER_FREQ_5 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_FILTER_FREQ_6 as usize].ptr = 0x20019f10; // &hatVoice.filter.f
        self.parameterArray[PAR_FILTER_FREQ_6 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_RESO_1 as usize].ptr = 0x20009860; // &voiceArray[0].filter.q
        self.parameterArray[PAR_RESO_1 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_RESO_2 as usize].ptr = 0x20009b20; // &voiceArray[1].filter.q
        self.parameterArray[PAR_RESO_2 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_RESO_3 as usize].ptr = 0x20009de0; // &voiceArray[2].filter.q
        self.parameterArray[PAR_RESO_3 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_RESO_4 as usize].ptr = 0x2001a07c; // &snareVoice.filter.q
        self.parameterArray[PAR_RESO_4 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_RESO_5 as usize].ptr = 0x20009558; // &cymbalVoice.filter.q
        self.parameterArray[PAR_RESO_5 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_RESO_6 as usize].ptr = 0x20019f18; // &hatVoice.filter.q
        self.parameterArray[PAR_RESO_6 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_VELOA1 as usize].ptr = 0x20009700; // &voiceArray[0].oscVolEg.attack
        self.parameterArray[PAR_VELOA1 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_VELOD1 as usize].ptr = 0x20009708; // &voiceArray[0].oscVolEg.decay
        self.parameterArray[PAR_VELOD1 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_VELOA2 as usize].ptr = 0x200099c0; // &voiceArray[1].oscVolEg.attack
        self.parameterArray[PAR_VELOA2 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_VELOD2 as usize].ptr = 0x200099c8; // &voiceArray[1].oscVolEg.decay
        self.parameterArray[PAR_VELOD2 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_VELOA3 as usize].ptr = 0x20009c80; // &voiceArray[2].oscVolEg.attack
        self.parameterArray[PAR_VELOA3 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_VELOD3 as usize].ptr = 0x20009c88; // &voiceArray[2].oscVolEg.decay
        self.parameterArray[PAR_VELOD3 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_VELOA4 as usize].ptr = 0x2001a0f0; // &snareVoice.oscVolEg.attack
        self.parameterArray[PAR_VELOA4 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_VELOD4 as usize].ptr = 0x2001a0f8; // &snareVoice.oscVolEg.decay
        self.parameterArray[PAR_VELOD4 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_VELOA5 as usize].ptr = 0x200095d8; // &cymbalVoice.oscVolEg.attack
        self.parameterArray[PAR_VELOA5 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_VELOD5 as usize].ptr = 0x200095e0; // &cymbalVoice.oscVolEg.decay
        self.parameterArray[PAR_VELOD5 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_VELOA6 as usize].ptr = 0x20019f70; // &hatVoice.oscVolEg.attack
        self.parameterArray[PAR_VELOA6 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_VELOD6_CLOSED as usize].ptr = 0x20019f4c; // &hatVoice.decayClosed
        self.parameterArray[PAR_VELOD6_CLOSED as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_VELOD6_OPEN as usize].ptr = 0x20019f50; // &hatVoice.decayOpen
        self.parameterArray[PAR_VELOD6_OPEN as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_VOL_SLOPE1 as usize].ptr = 0x20009710; // &voiceArray[0].oscVolEg.slope
        self.parameterArray[PAR_VOL_SLOPE1 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_VOL_SLOPE2 as usize].ptr = 0x200099d0; // &voiceArray[1].oscVolEg.slope
        self.parameterArray[PAR_VOL_SLOPE2 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_VOL_SLOPE3 as usize].ptr = 0x20009c90; // &voiceArray[2].oscVolEg.slope
        self.parameterArray[PAR_VOL_SLOPE3 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_VOL_SLOPE4 as usize].ptr = 0x2001a100; // &snareVoice.oscVolEg.slope
        self.parameterArray[PAR_VOL_SLOPE4 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_VOL_SLOPE5 as usize].ptr = 0x200095e8; // &cymbalVoice.oscVolEg.slope
        self.parameterArray[PAR_VOL_SLOPE5 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_VOL_SLOPE6 as usize].ptr = 0x20019f80; // &hatVoice.oscVolEg.slope
        self.parameterArray[PAR_VOL_SLOPE6 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_REPEAT4 as usize].ptr = 0x2001a110; // &snareVoice.oscVolEg.repeat
        self.parameterArray[PAR_REPEAT4 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_REPEAT5 as usize].ptr = 0x200095f8; // &cymbalVoice.oscVolEg.repeat
        self.parameterArray[PAR_REPEAT5 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_MOD_EG1 as usize].ptr = 0x200096a4; // &voiceArray[0].oscPitchEg.decay
        self.parameterArray[PAR_MOD_EG1 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_MOD_EG2 as usize].ptr = 0x20009964; // &voiceArray[1].oscPitchEg.decay
        self.parameterArray[PAR_MOD_EG2 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_MOD_EG3 as usize].ptr = 0x20009c24; // &voiceArray[2].oscPitchEg.decay
        self.parameterArray[PAR_MOD_EG3 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_MOD_EG4 as usize].ptr = 0x2001a094; // &snareVoice.oscPitchEg.decay
        self.parameterArray[PAR_MOD_EG4 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_MODAMNT1 as usize].ptr = 0x200096b0; // &voiceArray[0].egPitchModAmount
        self.parameterArray[PAR_MODAMNT1 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_MODAMNT2 as usize].ptr = 0x20009970; // &voiceArray[1].egPitchModAmount
        self.parameterArray[PAR_MODAMNT2 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_MODAMNT3 as usize].ptr = 0x20009c30; // &voiceArray[2].egPitchModAmount
        self.parameterArray[PAR_MODAMNT3 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_MODAMNT4 as usize].ptr = 0x2001a0a0; // &snareVoice.egPitchModAmount
        self.parameterArray[PAR_MODAMNT4 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_PITCH_SLOPE1 as usize].ptr = 0x200096a8; // &voiceArray[0].oscPitchEg.slope
        self.parameterArray[PAR_PITCH_SLOPE1 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_PITCH_SLOPE2 as usize].ptr = 0x20009968; // &voiceArray[1].oscPitchEg.slope
        self.parameterArray[PAR_PITCH_SLOPE2 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_PITCH_SLOPE3 as usize].ptr = 0x20009c28; // &voiceArray[2].oscPitchEg.slope
        self.parameterArray[PAR_PITCH_SLOPE3 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_PITCH_SLOPE4 as usize].ptr = 0x2001a098; // &snareVoice.oscPitchEg.slope
        self.parameterArray[PAR_PITCH_SLOPE4 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_FMAMNT1 as usize].ptr = 0x20009688; // &voiceArray[0].fmModAmount
        self.parameterArray[PAR_FMAMNT1 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_FM_FREQ1 as usize].ptr = 0x2000967c; // &voiceArray[0].modOsc.modNodeValue
        self.parameterArray[PAR_FM_FREQ1 as usize].r#type = TYPE_SPECIAL_F as u8;
        self.parameterArray[PAR_FMAMNT2 as usize].ptr = 0x20009948; // &voiceArray[1].fmModAmount
        self.parameterArray[PAR_FMAMNT2 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_FM_FREQ2 as usize].ptr = 0x2000993c; // &voiceArray[1].modOsc.modNodeValue
        self.parameterArray[PAR_FM_FREQ2 as usize].r#type = TYPE_SPECIAL_F as u8;
        self.parameterArray[PAR_FMAMNT3 as usize].ptr = 0x20009c08; // &voiceArray[2].fmModAmount
        self.parameterArray[PAR_FMAMNT3 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_FM_FREQ3 as usize].ptr = 0x20009bfc; // &voiceArray[2].modOsc.modNodeValue
        self.parameterArray[PAR_FM_FREQ3 as usize].r#type = TYPE_SPECIAL_F as u8;
        self.parameterArray[PAR_VOL1 as usize].ptr = 0x2000968c; // &voiceArray[0].vol
        self.parameterArray[PAR_VOL1 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_VOL2 as usize].ptr = 0x2000994c; // &voiceArray[1].vol
        self.parameterArray[PAR_VOL2 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_VOL3 as usize].ptr = 0x20009c0c; // &voiceArray[2].vol
        self.parameterArray[PAR_VOL3 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_VOL4 as usize].ptr = 0x2001a054; // &snareVoice.vol
        self.parameterArray[PAR_VOL4 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_VOL5 as usize].ptr = 0x20009538; // &cymbalVoice.vol
        self.parameterArray[PAR_VOL5 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_VOL6 as usize].ptr = 0x20019ef8; // &hatVoice.vol
        self.parameterArray[PAR_VOL6 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_PAN1 as usize].ptr = 0x20009694; // &voiceArray[0].pan
        self.parameterArray[PAR_PAN1 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_PAN2 as usize].ptr = 0x20009954; // &voiceArray[1].pan
        self.parameterArray[PAR_PAN2 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_PAN3 as usize].ptr = 0x20009c14; // &voiceArray[2].pan
        self.parameterArray[PAR_PAN3 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_PAN4 as usize].ptr = 0x2001a058; // &snareVoice.pan
        self.parameterArray[PAR_PAN4 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_PAN5 as usize].ptr = 0x2000953c; // &cymbalVoice.pan
        self.parameterArray[PAR_PAN5 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_PAN6 as usize].ptr = 0x20019efc; // &hatVoice.pan
        self.parameterArray[PAR_PAN6 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_DRIVE1 as usize].ptr = 0x20009840; // &voiceArray[0].distortion.shape
        self.parameterArray[PAR_DRIVE1 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_DRIVE2 as usize].ptr = 0x20009b00; // &voiceArray[1].distortion.shape
        self.parameterArray[PAR_DRIVE2 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_DRIVE3 as usize].ptr = 0x20009dc0; // &voiceArray[2].distortion.shape
        self.parameterArray[PAR_DRIVE3 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_SNARE_DISTORTION as usize].ptr = 0x2001a130; // &snareVoice.distortion.shape
        self.parameterArray[PAR_SNARE_DISTORTION as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_CYMBAL_DISTORTION as usize].ptr = 0x200095c0; // &cymbalVoice.distortion.shape
        self.parameterArray[PAR_CYMBAL_DISTORTION as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_HAT_DISTORTION as usize].ptr = 0x20019f58; // &hatVoice.distortion.shape
        self.parameterArray[PAR_HAT_DISTORTION as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_VOICE_DECIMATION1 as usize].ptr = 0x2001a170; // &mixer_decimation_rate[0]
        self.parameterArray[PAR_VOICE_DECIMATION1 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_VOICE_DECIMATION2 as usize].ptr = 0x2001a174; // &mixer_decimation_rate[1]
        self.parameterArray[PAR_VOICE_DECIMATION2 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_VOICE_DECIMATION3 as usize].ptr = 0x2001a178; // &mixer_decimation_rate[2]
        self.parameterArray[PAR_VOICE_DECIMATION3 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_VOICE_DECIMATION4 as usize].ptr = 0x2001a17c; // &mixer_decimation_rate[3]
        self.parameterArray[PAR_VOICE_DECIMATION4 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_VOICE_DECIMATION5 as usize].ptr = 0x2001a180; // &mixer_decimation_rate[4]
        self.parameterArray[PAR_VOICE_DECIMATION5 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_VOICE_DECIMATION6 as usize].ptr = 0x2001a184; // &mixer_decimation_rate[5]
        self.parameterArray[PAR_VOICE_DECIMATION6 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_VOICE_DECIMATION_ALL as usize].ptr = 0x2001a188; // &mixer_decimation_rate[6]
        self.parameterArray[PAR_VOICE_DECIMATION_ALL as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_FREQ_LFO1 as usize].ptr = 0x200096fc; // &voiceArray[0].lfo.modNodeValue
        self.parameterArray[PAR_FREQ_LFO1 as usize].r#type = TYPE_SPECIAL_F as u8;
        self.parameterArray[PAR_FREQ_LFO2 as usize].ptr = 0x200099bc; // &voiceArray[1].lfo.modNodeValue
        self.parameterArray[PAR_FREQ_LFO2 as usize].r#type = TYPE_SPECIAL_F as u8;
        self.parameterArray[PAR_FREQ_LFO3 as usize].ptr = 0x20009c7c; // &voiceArray[2].lfo.modNodeValue
        self.parameterArray[PAR_FREQ_LFO3 as usize].r#type = TYPE_SPECIAL_F as u8;
        self.parameterArray[PAR_FREQ_LFO4 as usize].ptr = 0x2001a0d0; // &snareVoice.lfo.modNodeValue
        self.parameterArray[PAR_FREQ_LFO4 as usize].r#type = TYPE_SPECIAL_F as u8;
        self.parameterArray[PAR_FREQ_LFO5 as usize].ptr = 0x200095a0; // &cymbalVoice.lfo.modNodeValue
        self.parameterArray[PAR_FREQ_LFO5 as usize].r#type = TYPE_SPECIAL_F as u8;
        self.parameterArray[PAR_FREQ_LFO6 as usize].ptr = 0x20019fd8; // &hatVoice.lfo.modNodeValue
        self.parameterArray[PAR_FREQ_LFO6 as usize].r#type = TYPE_SPECIAL_F as u8;
        self.parameterArray[PAR_AMOUNT_LFO1 as usize].ptr = 0x200096f4; // &voiceArray[0].lfo.modTarget.amount
        self.parameterArray[PAR_AMOUNT_LFO1 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_AMOUNT_LFO2 as usize].ptr = 0x200099b4; // &voiceArray[1].lfo.modTarget.amount
        self.parameterArray[PAR_AMOUNT_LFO2 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_AMOUNT_LFO3 as usize].ptr = 0x20009c74; // &voiceArray[2].lfo.modTarget.amount
        self.parameterArray[PAR_AMOUNT_LFO3 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_AMOUNT_LFO4 as usize].ptr = 0x2001a0c8; // &snareVoice.lfo.modTarget.amount
        self.parameterArray[PAR_AMOUNT_LFO4 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_AMOUNT_LFO5 as usize].ptr = 0x20009598; // &cymbalVoice.lfo.modTarget.amount
        self.parameterArray[PAR_AMOUNT_LFO5 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_AMOUNT_LFO6 as usize].ptr = 0x20019fd0; // &hatVoice.lfo.modTarget.amount
        self.parameterArray[PAR_AMOUNT_LFO6 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_FILTER_DRIVE_1 as usize].ptr = 0x20009874; // &voiceArray[0].filter.drive
        self.parameterArray[PAR_FILTER_DRIVE_1 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_FILTER_DRIVE_2 as usize].ptr = 0x20009b34; // &voiceArray[1].filter.drive
        self.parameterArray[PAR_FILTER_DRIVE_2 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_FILTER_DRIVE_3 as usize].ptr = 0x20009df4; // &voiceArray[2].filter.drive
        self.parameterArray[PAR_FILTER_DRIVE_3 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_FILTER_DRIVE_4 as usize].ptr = 0x2001a090; // &snareVoice.filter.drive
        self.parameterArray[PAR_FILTER_DRIVE_4 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_FILTER_DRIVE_5 as usize].ptr = 0x2000956c; // &cymbalVoice.filter.drive
        self.parameterArray[PAR_FILTER_DRIVE_5 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_FILTER_DRIVE_6 as usize].ptr = 0x20019f2c; // &hatVoice.filter.drive
        self.parameterArray[PAR_FILTER_DRIVE_6 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_MIX_MOD_1 as usize].ptr = 0x20009890; // &voiceArray[0].mixOscs
        self.parameterArray[PAR_MIX_MOD_1 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_MIX_MOD_2 as usize].ptr = 0x20009b50; // &voiceArray[1].mixOscs
        self.parameterArray[PAR_MIX_MOD_2 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_MIX_MOD_3 as usize].ptr = 0x20009e10; // &voiceArray[2].mixOscs
        self.parameterArray[PAR_MIX_MOD_3 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_VOLUME_MOD_ON_OFF1 as usize].ptr = 0x200098a0; // &voiceArray[0].volumeMod
        self.parameterArray[PAR_VOLUME_MOD_ON_OFF1 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_VOLUME_MOD_ON_OFF2 as usize].ptr = 0x20009b60; // &voiceArray[1].volumeMod
        self.parameterArray[PAR_VOLUME_MOD_ON_OFF2 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_VOLUME_MOD_ON_OFF3 as usize].ptr = 0x20009e20; // &voiceArray[2].volumeMod
        self.parameterArray[PAR_VOLUME_MOD_ON_OFF3 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_VOLUME_MOD_ON_OFF4 as usize].ptr = 0x2001a14c; // &snareVoice.volumeMod
        self.parameterArray[PAR_VOLUME_MOD_ON_OFF4 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_VOLUME_MOD_ON_OFF5 as usize].ptr = 0x20009614; // &cymbalVoice.volumeMod
        self.parameterArray[PAR_VOLUME_MOD_ON_OFF5 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_VOLUME_MOD_ON_OFF6 as usize].ptr = 0x20019fe0; // &hatVoice.volumeMod
        self.parameterArray[PAR_VOLUME_MOD_ON_OFF6 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_VELO_MOD_AMT_1 as usize].ptr = 0x2001a1c0; // &velocityModulators[0].amount
        self.parameterArray[PAR_VELO_MOD_AMT_1 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_VELO_MOD_AMT_2 as usize].ptr = 0x2001a1d0; // &velocityModulators[1].amount
        self.parameterArray[PAR_VELO_MOD_AMT_2 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_VELO_MOD_AMT_3 as usize].ptr = 0x2001a1e0; // &velocityModulators[2].amount
        self.parameterArray[PAR_VELO_MOD_AMT_3 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_VELO_MOD_AMT_4 as usize].ptr = 0x2001a1f0; // &velocityModulators[3].amount
        self.parameterArray[PAR_VELO_MOD_AMT_4 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_VELO_MOD_AMT_5 as usize].ptr = 0x2001a200; // &velocityModulators[4].amount
        self.parameterArray[PAR_VELO_MOD_AMT_5 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_VELO_MOD_AMT_6 as usize].ptr = 0x2001a210; // &velocityModulators[5].amount
        self.parameterArray[PAR_VELO_MOD_AMT_6 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_VEL_DEST_1 as usize].ptr = 0x2001a1b8; // &velocityModulators[0].destination
        self.parameterArray[PAR_VEL_DEST_1 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_VEL_DEST_2 as usize].ptr = 0x2001a1c8; // &velocityModulators[1].destination
        self.parameterArray[PAR_VEL_DEST_2 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_VEL_DEST_3 as usize].ptr = 0x2001a1d8; // &velocityModulators[2].destination
        self.parameterArray[PAR_VEL_DEST_3 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_VEL_DEST_4 as usize].ptr = 0x2001a1e8; // &velocityModulators[3].destination
        self.parameterArray[PAR_VEL_DEST_4 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_VEL_DEST_5 as usize].ptr = 0x2001a1f8; // &velocityModulators[4].destination
        self.parameterArray[PAR_VEL_DEST_5 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_VEL_DEST_6 as usize].ptr = 0x2001a208; // &velocityModulators[5].destination
        self.parameterArray[PAR_VEL_DEST_6 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_WAVE_LFO1 as usize].ptr = 0x200096d8; // &voiceArray[0].lfo.waveform
        self.parameterArray[PAR_WAVE_LFO1 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_WAVE_LFO2 as usize].ptr = 0x20009998; // &voiceArray[1].lfo.waveform
        self.parameterArray[PAR_WAVE_LFO2 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_WAVE_LFO3 as usize].ptr = 0x20009c58; // &voiceArray[2].lfo.waveform
        self.parameterArray[PAR_WAVE_LFO3 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_WAVE_LFO4 as usize].ptr = 0x2001a0ac; // &snareVoice.lfo.waveform
        self.parameterArray[PAR_WAVE_LFO4 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_WAVE_LFO5 as usize].ptr = 0x2000957c; // &cymbalVoice.lfo.waveform
        self.parameterArray[PAR_WAVE_LFO5 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_WAVE_LFO6 as usize].ptr = 0x20019fb4; // &hatVoice.lfo.waveform
        self.parameterArray[PAR_WAVE_LFO6 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_RETRIGGER_LFO1 as usize].ptr = 0x200096d9; // &voiceArray[0].lfo.retrigger
        self.parameterArray[PAR_RETRIGGER_LFO1 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_RETRIGGER_LFO2 as usize].ptr = 0x20009999; // &voiceArray[1].lfo.retrigger
        self.parameterArray[PAR_RETRIGGER_LFO2 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_RETRIGGER_LFO3 as usize].ptr = 0x20009c59; // &voiceArray[2].lfo.retrigger
        self.parameterArray[PAR_RETRIGGER_LFO3 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_RETRIGGER_LFO4 as usize].ptr = 0x2001a0ad; // &snareVoice.lfo.retrigger
        self.parameterArray[PAR_RETRIGGER_LFO4 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_RETRIGGER_LFO5 as usize].ptr = 0x2000957d; // &cymbalVoice.lfo.retrigger
        self.parameterArray[PAR_RETRIGGER_LFO5 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_RETRIGGER_LFO6 as usize].ptr = 0x20019fb5; // &hatVoice.lfo.retrigger
        self.parameterArray[PAR_RETRIGGER_LFO6 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_SYNC_LFO1 as usize].ptr = 0x200096e4; // &voiceArray[0].lfo.sync
        self.parameterArray[PAR_SYNC_LFO1 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_SYNC_LFO2 as usize].ptr = 0x200099a4; // &voiceArray[1].lfo.sync
        self.parameterArray[PAR_SYNC_LFO2 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_SYNC_LFO3 as usize].ptr = 0x20009c64; // &voiceArray[2].lfo.sync
        self.parameterArray[PAR_SYNC_LFO3 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_SYNC_LFO4 as usize].ptr = 0x2001a0b8; // &snareVoice.lfo.sync
        self.parameterArray[PAR_SYNC_LFO4 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_SYNC_LFO5 as usize].ptr = 0x20009588; // &cymbalVoice.lfo.sync
        self.parameterArray[PAR_SYNC_LFO5 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_SYNC_LFO6 as usize].ptr = 0x20019fc0; // &hatVoice.lfo.sync
        self.parameterArray[PAR_SYNC_LFO6 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_OFFSET_LFO1 as usize].ptr = 0x200096dc; // &voiceArray[0].lfo.phaseOffset
        self.parameterArray[PAR_OFFSET_LFO1 as usize].r#type = TYPE_UINT32 as u8;
        self.parameterArray[PAR_OFFSET_LFO2 as usize].ptr = 0x2000999c; // &voiceArray[1].lfo.phaseOffset
        self.parameterArray[PAR_OFFSET_LFO2 as usize].r#type = TYPE_UINT32 as u8;
        self.parameterArray[PAR_OFFSET_LFO3 as usize].ptr = 0x20009c5c; // &voiceArray[2].lfo.phaseOffset
        self.parameterArray[PAR_OFFSET_LFO3 as usize].r#type = TYPE_UINT32 as u8;
        self.parameterArray[PAR_OFFSET_LFO4 as usize].ptr = 0x2001a0b0; // &snareVoice.lfo.phaseOffset
        self.parameterArray[PAR_OFFSET_LFO4 as usize].r#type = TYPE_UINT32 as u8;
        self.parameterArray[PAR_OFFSET_LFO5 as usize].ptr = 0x20009580; // &cymbalVoice.lfo.phaseOffset
        self.parameterArray[PAR_OFFSET_LFO5 as usize].r#type = TYPE_UINT32 as u8;
        self.parameterArray[PAR_OFFSET_LFO6 as usize].ptr = 0x20019fb8; // &hatVoice.lfo.phaseOffset
        self.parameterArray[PAR_OFFSET_LFO6 as usize].r#type = TYPE_UINT32 as u8;
        self.parameterArray[PAR_FILTER_TYPE_1 as usize].ptr = 0x20009878; // &voiceArray[0].filterType
        self.parameterArray[PAR_FILTER_TYPE_1 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_FILTER_TYPE_2 as usize].ptr = 0x20009b38; // &voiceArray[1].filterType
        self.parameterArray[PAR_FILTER_TYPE_2 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_FILTER_TYPE_3 as usize].ptr = 0x20009df8; // &voiceArray[2].filterType
        self.parameterArray[PAR_FILTER_TYPE_3 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_FILTER_TYPE_4 as usize].ptr = 0x2001a050; // &snareVoice.filterType
        self.parameterArray[PAR_FILTER_TYPE_4 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_FILTER_TYPE_5 as usize].ptr = 0x20009570; // &cymbalVoice.filterType
        self.parameterArray[PAR_FILTER_TYPE_5 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_FILTER_TYPE_6 as usize].ptr = 0x20019f30; // &hatVoice.filterType
        self.parameterArray[PAR_FILTER_TYPE_6 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_TRANS1_VOL as usize].ptr = 0x200096c8; // &voiceArray[0].transGen.volume
        self.parameterArray[PAR_TRANS1_VOL as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_TRANS2_VOL as usize].ptr = 0x20009988; // &voiceArray[1].transGen.volume
        self.parameterArray[PAR_TRANS2_VOL as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_TRANS3_VOL as usize].ptr = 0x20009c48; // &voiceArray[2].transGen.volume
        self.parameterArray[PAR_TRANS3_VOL as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_TRANS4_VOL as usize].ptr = 0x2001a0e4; // &snareVoice.transGen.volume
        self.parameterArray[PAR_TRANS4_VOL as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_TRANS5_VOL as usize].ptr = 0x200095b4; // &cymbalVoice.transGen.volume
        self.parameterArray[PAR_TRANS5_VOL as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_TRANS6_VOL as usize].ptr = 0x20019f44; // &hatVoice.transGen.volume
        self.parameterArray[PAR_TRANS6_VOL as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_TRANS1_WAVE as usize].ptr = 0x200096c4; // &voiceArray[0].transGen.waveform
        self.parameterArray[PAR_TRANS1_WAVE as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_TRANS2_WAVE as usize].ptr = 0x20009984; // &voiceArray[1].transGen.waveform
        self.parameterArray[PAR_TRANS2_WAVE as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_TRANS3_WAVE as usize].ptr = 0x20009c44; // &voiceArray[2].transGen.waveform
        self.parameterArray[PAR_TRANS3_WAVE as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_TRANS4_WAVE as usize].ptr = 0x2001a0e0; // &snareVoice.transGen.waveform
        self.parameterArray[PAR_TRANS4_WAVE as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_TRANS5_WAVE as usize].ptr = 0x200095b0; // &cymbalVoice.transGen.waveform
        self.parameterArray[PAR_TRANS5_WAVE as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_TRANS6_WAVE as usize].ptr = 0x20019f40; // &hatVoice.transGen.waveform
        self.parameterArray[PAR_TRANS6_WAVE as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_TRANS1_FREQ as usize].ptr = 0x200096c0; // &voiceArray[0].transGen.pitch
        self.parameterArray[PAR_TRANS1_FREQ as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_TRANS2_FREQ as usize].ptr = 0x20009980; // &voiceArray[1].transGen.pitch
        self.parameterArray[PAR_TRANS2_FREQ as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_TRANS3_FREQ as usize].ptr = 0x20009c40; // &voiceArray[2].transGen.pitch
        self.parameterArray[PAR_TRANS3_FREQ as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_TRANS4_FREQ as usize].ptr = 0x2001a0dc; // &snareVoice.transGen.pitch
        self.parameterArray[PAR_TRANS4_FREQ as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_TRANS5_FREQ as usize].ptr = 0x200095ac; // &cymbalVoice.transGen.pitch
        self.parameterArray[PAR_TRANS5_FREQ as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_TRANS6_FREQ as usize].ptr = 0x20019f3c; // &hatVoice.transGen.pitch
        self.parameterArray[PAR_TRANS6_FREQ as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_AUDIO_OUT1 as usize].ptr = 0x2001a1b0; // &mixer_audioRouting[0]
        self.parameterArray[PAR_AUDIO_OUT1 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_AUDIO_OUT2 as usize].ptr = 0x2001a1b1; // &mixer_audioRouting[1]
        self.parameterArray[PAR_AUDIO_OUT2 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_AUDIO_OUT3 as usize].ptr = 0x2001a1b2; // &mixer_audioRouting[2]
        self.parameterArray[PAR_AUDIO_OUT3 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_AUDIO_OUT4 as usize].ptr = 0x2001a1b3; // &mixer_audioRouting[3]
        self.parameterArray[PAR_AUDIO_OUT4 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_AUDIO_OUT5 as usize].ptr = 0x2001a1b4; // &mixer_audioRouting[4]
        self.parameterArray[PAR_AUDIO_OUT5 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_AUDIO_OUT6 as usize].ptr = 0x2001a1b5; // &mixer_audioRouting[5]
        self.parameterArray[PAR_AUDIO_OUT6 as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_PWM_1 as usize].ptr = 0x200098f0; // &voiceArray[0].pwm
        self.parameterArray[PAR_PWM_1 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_PWM_2 as usize].ptr = 0x20009bb0; // &voiceArray[1].pwm
        self.parameterArray[PAR_PWM_2 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_PWM_3 as usize].ptr = 0x20009e70; // &voiceArray[2].pwm
        self.parameterArray[PAR_PWM_3 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_PWM_4 as usize].ptr = 0x2001a168; // &snareVoice.pwm
        self.parameterArray[PAR_PWM_4 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_PWM_5 as usize].ptr = 0x20009630; // &cymbalVoice.pwm
        self.parameterArray[PAR_PWM_5 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_PWM_6 as usize].ptr = 0x20019ff8; // &hatVoice.pwm
        self.parameterArray[PAR_PWM_6 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_FX_TYPE as usize].ptr = 0x2001a218; // &fxBus.fxType
        self.parameterArray[PAR_FX_TYPE as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_FX_ROUTING as usize].ptr = 0x2001a219; // &fxBus.audioRouting
        self.parameterArray[PAR_FX_ROUTING as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_FX_DRY_WET as usize].ptr = 0x2001a21c; // &fxBus.dryWet
        self.parameterArray[PAR_FX_DRY_WET as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_FX_DIST_TYPE as usize].ptr = 0x2001a230; // &fxBus.distortion.distortionType
        self.parameterArray[PAR_FX_DIST_TYPE as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_FX_P1 as usize].ptr = 0x2001a220; // &fxBus.parameter[0]
        self.parameterArray[PAR_FX_P1 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_FX_P2 as usize].ptr = 0x2001a224; // &fxBus.parameter[1]
        self.parameterArray[PAR_FX_P2 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_FX_P3 as usize].ptr = 0x2001a228; // &fxBus.parameter[2]
        self.parameterArray[PAR_FX_P3 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_FX_P4 as usize].ptr = 0x2001a22c; // &fxBus.parameter[3]
        self.parameterArray[PAR_FX_P4 as usize].r#type = TYPE_FLT as u8;
        self.parameterArray[PAR_FX_RATIO as usize].ptr = 0x2001a304; // &fxBus.compressor.ratio
        self.parameterArray[PAR_FX_RATIO as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_FX_WAVE as usize].ptr = 0x2001a2dc; // &fxBus.ringmod.osc.waveform
        self.parameterArray[PAR_FX_WAVE as usize].r#type = TYPE_UINT8 as u8;
        self.parameterArray[PAR_FX_DELAY_TYPE as usize].ptr = 0x2001a370; // &fxBus.delay.delayType
        self.parameterArray[PAR_FX_DELAY_TYPE as usize].r#type = TYPE_UINT8 as u8;
    }
}
