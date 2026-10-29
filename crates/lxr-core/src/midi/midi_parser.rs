//! Port of `Core/Inc/Soundengine/MIDI/MidiParser.h`, `Core/Src/Soundengine/MIDI/MidiParser.c`.
//!
//! Porting group: P3. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;
use crate::cmem::CMem;
use crate::dsp::decay::{DecayEg_setDecay, DecayEg_setSlope};
use crate::dsp::distortion::setDistortionShape;
use crate::dsp::fx::compressor::Compressor_setRatio;
use crate::dsp::fx::ringmod::Ringmod_setWave;
use crate::dsp::oscillator::osc_recalcFreq;
use crate::dsp::resonant_filter::{SVF_directSetFilterValue, SVF_setDrive, SVF_setReso};
use crate::dsp::slope_eg2::{slopeEg2_calcDecay, slopeEg2_setAttack, slopeEg2_setDecay, slopeEg2_setSlope};
use crate::dsp::transient_generator::transient_setWaveform;
use crate::fx_bus::FxBus_prepareFxChange;
use crate::midi::value_shaper::valueShaperF2F;

// ---- #define constants
/// `#define SEMITONE_UP 1.0594630943592952645618252949463f` (Core/Src/Soundengine/MIDI/MidiParser.c:70) = 1.0594631433486938
pub const SEMITONE_UP: f32 = f32::from_bits(0x3f879c7d);
/// `#define SEMITONE_DOWN 0.94387431268169349664191315666753f` (Core/Src/Soundengine/MIDI/MidiParser.c:71) = 0.9438742995262146
pub const SEMITONE_DOWN: f32 = f32::from_bits(0x3f71a1bf);
/// `#define NUM_LFO 6` (Core/Src/Soundengine/MIDI/MidiParser.c:73)
pub const NUM_LFO: i32 = 6;

// ---- enums
/// C: `enum TxRxFilterValues` (Core/Inc/Soundengine/MIDI/MidiParser.h:67), 1 bytes, underlying type `u8`.
/// Representation: integer type alias plus one `pub const` per enumerator.
pub type TxRxFilterValues = u8;
pub const MIDI_FILTER_NOTES: TxRxFilterValues = 1;
pub const MIDI_FILTER_SYSTEM: TxRxFilterValues = 2;
pub const MIDI_FILTER_CC: TxRxFilterValues = 4;
pub const MIDI_FILTER_PROG_C: TxRxFilterValues = 8;
/// C: `enum State` (Core/Src/Soundengine/MIDI/MidiParser.c:60), 1 bytes, underlying type `u8`.
/// Representation: integer type alias plus one `pub const` per enumerator.
pub type State = u8;
pub const MIDI_STATUS: State = 0;
pub const MIDI_DATA1: State = 1;
pub const MIDI_DATA2: State = 2;
pub const SYSEX_DATA: State = 3;
pub const IGNORE: State = 4;

// ---- types
/// C: the anonymous type of `midiParser_routing` (Core/Src/Soundengine/MIDI/MidiParser.c:44):
/// `static union { uint8_t value; struct { unsigned usb2midi:1; unsigned usb2usb:1; unsigned midi2midi:1;
/// unsigned midi2usb:1; unsigned :4; } route; } midiParser_routing;`, 4 bytes.
///
/// Representation: the union storage as a 32 bit word (`value` is its low byte); the `route` bit-fields are bits 0..3.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct midiParser_routing_t {
    pub bits: u32,
}
impl midiParser_routing_t {
    pub const ZERO: Self = Self { bits: 0 };
    /// `uint8_t value;` (read)
    #[inline(always)]
    pub fn value(&self) -> u8 { self.bits as u8 }
    /// `uint8_t value;` (write; only the low byte changes)
    #[inline(always)]
    pub fn set_value(&mut self, v: u8) { self.bits = (self.bits & !0xff) | v as u32 }
    /// `route.usb2midi`
    #[inline(always)]
    pub fn route_usb2midi(&self) -> u32 { self.bits & 1 }
    #[inline(always)]
    pub fn set_route_usb2midi(&mut self, v: u32) { self.bits = (self.bits & !1) | (v & 1) }
    /// `route.usb2usb`
    #[inline(always)]
    pub fn route_usb2usb(&self) -> u32 { (self.bits >> 1) & 1 }
    #[inline(always)]
    pub fn set_route_usb2usb(&mut self, v: u32) { self.bits = (self.bits & !2) | ((v & 1) << 1) }
    /// `route.midi2midi`
    #[inline(always)]
    pub fn route_midi2midi(&self) -> u32 { (self.bits >> 2) & 1 }
    #[inline(always)]
    pub fn set_route_midi2midi(&mut self, v: u32) { self.bits = (self.bits & !4) | ((v & 1) << 2) }
    /// `route.midi2usb`
    #[inline(always)]
    pub fn route_midi2usb(&self) -> u32 { (self.bits >> 3) & 1 }
    #[inline(always)]
    pub fn set_route_midi2usb(&mut self, v: u32) { self.bits = (self.bits & !8) | ((v & 1) << 3) }
}

/// `1/127.f` as the firmware uses it: GCC turns `x/127.f` into a multiplication by this constant
/// (`-freciprocal-math`), bits 0x3c010204.
pub const RECIP_127: f32 = f32::from_bits(0x3c010204);

/// Offset of the out-of-range store `midiParser_originalCcValues[65536]` (MIDI CC 0 = bank change, see
/// `midiParser_ccHandler`) inside `seq_patternSet` in the shipped v1.71 image (0x200315a0 - 0x200230b8).
/// The GCC 11.3 ELF has the same store at offset 0xe4f0 (0x200319c0 - 0x200234d0).
pub const CC0_OOB_PATTERNSET_OFFSET: u32 = 0xe4e8;

/// `parameterArray` follows `midiParser_originalCcValues` at this offset in both builds (after
/// `midiParser_routing`, `midi_MidiChannels`, `active_voices` and padding).
pub const PARAMETER_ARRAY_CC_OFFSET: u32 = 0x110;

// ---- functions without access to globals
/// C: `static inline float calcPitchModAmount(uint8_t data2)`
///
/// Source: `Core/Src/Soundengine/MIDI/MidiParser.c:95`. Reference build: inlined into its callers only. Group P3.
/// Machine code: `v = (float)(int)data2 * (1/127.f)`, `v*v`, `* 32.f` (three roundings, no fusing).
pub fn calcPitchModAmount(data2: u8) -> f32 {
    let val = (data2 as i32 as f32) * RECIP_127;
    let sq = val * val;
    sq * PITCH_AMOUNT_FACTOR as f32
}

/// C: `float midiParser_calcDetune(uint8_t value)`
///
/// Source: `Core/Src/Soundengine/MIDI/MidiParser.c:118`. Reference build: out-of-line in flash. Group P3.
/// Machine code: `frac = vfma(-0.5, value, 1/127.f)`, `return vfma(1.0, frac, SEMITONE_UP-1)` (constant 0x3d738fa0).
pub fn midiParser_calcDetune(value: u8) -> f32 {
    let frac = vfma(-0.5, value as i32 as f32, RECIP_127);
    vfma(1.0, frac, f32::from_bits(0x3d738fa0))
}

/// C: `uint8_t midiParser_cc2voiceNr(uint8_t cc)`
///
/// Source: `Core/Src/Soundengine/MIDI/MidiParser.c:1553`. Reference build: inlined into its callers only. Group P3.
pub fn midiParser_cc2voiceNr(cc: u8) -> u8 {
    match cc {
        0 | 1 | 6 | 98 | 99 => 0,
        2 | 9 | 10 | 21 | 38 | 44 | 50 | 51 | 63 | 71 | 75 | 79 | 83 | 84 | 89 | 95 | 103 | 109 | 116 | 122 => 1,
        3 | 11 | 12 | 22 | 39 | 45 | 52 | 53 | 64 | 72 | 76 | 80 | 85 | 86 | 90 | 96 | 104 | 110 | 117 | 123 => 2,
        4 | 13 | 14 | 23 | 40 | 46 | 54 | 55 | 65 | 73 | 77 | 81 | 87 | 88 | 91 | 97 | 105 | 111 | 118 | 124 => 3,
        5 | 15 | 16 | 28 | 29 | 41 | 47 | 56 | 57 | 66 | 69 | 74 | 78 | 82 | 92 | 100 | 106 | 112 | 119 | 125 => 4,
        7 | 17 | 18 | 24 | 25 | 30 | 31 | 32 | 33 | 42 | 48 | 58 | 59 | 67 | 70 | 93 | 101 | 107 | 113 | 120 | 126 => 5,
        8 | 19 | 20 | 26 | 27 | 34 | 35 | 36 | 37 | 43 | 49 | 60 | 61 | 62 | 68 | 94 | 102 | 108 | 114 | 121 | 127 => 6,
        115 => 7,
        _ => 0,
    }
}

/// `uint8_t wave = N * value.flt; wave = wave >= N ? N-1 : wave;` as compiled: `vcvt.u32.f32` (truncating,
/// saturating), low byte, then the clamp to `max` (= N-1).
#[inline(always)]
fn float_to_clamped_u8(n: f32, flt: f32, max: u8) -> u8 {
    let w = ((n * flt) as u32) as u8;
    if w > max { max } else { w }
}

/// `FILTER_SHAPER` (-0.9f) as passed to `valueShaperF2F`.
const FILTER_SHAPER: f32 = -0.9;

/// Copies an `OscInfo` out of `$place`, updates `midiFreq` and calls `osc_recalcFreq`.
macro_rules! set_midi_freq {
    ($osc:expr, coarse, $itg:expr) => {{
        let osc = &mut $osc;
        osc.midiFreq &= 0x00ff;
        osc.midiFreq |= ($itg as u16) << 8;
        osc_recalcFreq(osc);
    }};
    ($osc:expr, fine, $itg:expr) => {{
        let osc = &mut $osc;
        osc.midiFreq &= 0xff00;
        osc.midiFreq |= $itg as u16;
        osc_recalcFreq(osc);
    }};
}

// ---- functions that use globals (methods on `Lxr`)
impl Lxr {
    /// Store into `midiParser_originalCcValues[idx]` with the firmware's out-of-range behaviour: indices
    /// 256..=383 (CC2 path with an NRPN number whose low byte is above 126) land in the globals that follow
    /// the array (`midiParser_routing`, `midi_MidiChannels`, `active_voices`, `parameterArray[0..14]`; index
    /// 255 and the other padding bytes are not modelled), and index 65536 (MIDI CC 0: `uint16_t paramNr =
    /// data1-1` wraps) lands in `seq_patternSet` (shipped layout). Non-C helper, group P3.
    pub fn midiParser_storeOriginalCcValue(&mut self, idx: u32, v: u8) {
        if idx < 255 {
            self.midiParser_originalCcValues[idx as usize] = v;
        } else if idx == 0x10000 {
            self.seq_patternSet.set_byte(CC0_OOB_PATTERNSET_OFFSET, v);
        } else if let Some(k) = idx.checked_sub(PARAMETER_ARRAY_CC_OFFSET).filter(|o| o % 8 < 4) {
            // a byte of `parameterArray[k / 8].ptr`: the firmware changes a byte of the shipped address of the
            // pointer target, so do the byte arithmetic on that address (the relative layout of the target
            // objects differs between the shipped image and the reference ELF, whose addresses `CPtr` holds)
            let e = &mut self.parameterArray[(k / 8) as usize];
            let sh = (k % 8) * 8;
            match crate::midi::shipped_layout::to_shipped(e.ptr) {
                Some(s) => {
                    let s2 = (s & !(0xff << sh)) | ((v as u32) << sh);
                    e.ptr = match crate::midi::shipped_layout::from_shipped(s2) {
                        Some(p) => p,
                        // outside the modelled objects (e.g. beyond the end of RAM): the pointer keeps the
                        // shipped address; accesses through it read 0 / are dropped in `cptr_*` (on the device:
                        // a bus fault or unmodelled memory)
                        None => s2,
                    };
                }
                None => e.ptr = (e.ptr & !(0xff << sh)) | ((v as u32) << sh),
            }
        } else {
            self.global_set_byte(crate::memmap::ADDR_midiParser_originalCcValues + idx, v);
        }
    }

    /// C: `static void midiParser_nrpnHandler(uint16_t value)`
    ///
    /// Source: `Core/Src/Soundengine/MIDI/MidiParser.c:134`. Reference build: inlined into its callers only. Group P3.
    pub fn midiParser_nrpnHandler(&mut self, value: u16) {
        let mut msg2 = MidiMsg::ZERO;
        msg2.status = MIDI_CC2 as u8;
        msg2.data1 = self.midiParser_activeNrpnNumber as u8;
        msg2.data2 = value as u8;
        self.midiParser_ccHandler(msg2, 1, 0, 0.0);
    }

    /// C: `void midiParser_ccHandler(MidiMsg msg, uint8_t updateOriginalValue, uint8_t isFloatModulator, float v)`
    ///
    /// Source: `Core/Src/Soundengine/MIDI/MidiParser.c:144`. Reference build: out-of-line in flash. Group P3.
    /// Ported from the machine code (0x08016dc4): `value.itg = (uint8_t)(uint32_t)(v*127.f)` for float
    /// modulators; `value.flt = (float)(int)itg * (1/127.f)` otherwise; `x/33.9f` and `x/100.f` are
    /// multiplications by reciprocal constants and `1 + (x/33.9 - 0.75)` is one `vfma(0.25, x, 1/33.9)`;
    /// float to `uint8_t` conversions are `vcvt.u32.f32` followed by the low byte. Out-of-range stores into
    /// `midiParser_originalCcValues` are emulated (see `midiParser_storeOriginalCcValue`).
    pub fn midiParser_ccHandler(&mut self, msg: MidiMsg, updateOriginalValue: u8, isFloatModulator: u8, v: f32) {
        let itg: u8;
        let flt: f32;
        if isFloatModulator != 0 {
            itg = ((v * 127.0) as u32) as u8;
            flt = v;
        } else {
            itg = msg.data2;
            flt = (itg as i32 as f32) * RECIP_127;
        }
        let isf = isFloatModulator != 0;
        let d1 = msg.data1;
        let nosc = NUM_OSC_WAVEFORMS as f32;
        let mosc = NUM_OSC_WAVEFORMS - 1;

        if (msg.status & 0xF0) as i32 == MIDI_CC {
            let paramNr: u16 = (d1 as u16).wrapping_sub(1);
            if updateOriginalValue != 0 && !isf {
                self.midiParser_storeOriginalCcValue(paramNr as u32 + 1, itg);
            }
            match d1 {
                CC_MODWHEEL => {}
                CC_BANK_CHANGE => {}
                NRPN_DATA_ENTRY_COARSE => {
                    self.midiParser_nrpnHandler(itg as u16);
                    return;
                }
                NRPN_FINE => {
                    self.midiParser_activeNrpnNumber &= !0x7f;
                    self.midiParser_activeNrpnNumber |= (itg & 0x7f) as u16;
                }
                NRPN_COARSE => {
                    self.midiParser_activeNrpnNumber &= 0x7f;
                    self.midiParser_activeNrpnNumber |= (itg as u16) << 7;
                }
                VOL_SLOPE1 => slopeEg2_setSlope(&mut self.voiceArray[0].oscVolEg, itg),
                PITCH_SLOPE1 => DecayEg_setSlope(&mut self.voiceArray[0].oscPitchEg, itg),
                VOL_SLOPE2 => slopeEg2_setSlope(&mut self.voiceArray[1].oscVolEg, itg),
                PITCH_SLOPE2 => DecayEg_setSlope(&mut self.voiceArray[1].oscPitchEg, itg),
                VOL_SLOPE3 => slopeEg2_setSlope(&mut self.voiceArray[2].oscVolEg, itg),
                PITCH_SLOPE3 => DecayEg_setSlope(&mut self.voiceArray[2].oscPitchEg, itg),
                PITCH_SLOPE4 => DecayEg_setSlope(&mut self.snareVoice.oscPitchEg, itg),
                VOL_SLOPE6 => slopeEg2_setSlope(&mut self.hatVoice.oscVolEg, itg),
                FILTER_FREQ_DRUM1 | FILTER_FREQ_DRUM2 | FILTER_FREQ_DRUM3 => SVF_directSetFilterValue(
                    &mut self.voiceArray[(d1 - FILTER_FREQ_DRUM1) as usize].filter,
                    valueShaperF2F(flt, FILTER_SHAPER),
                ),
                RESO_DRUM1 | RESO_DRUM2 | RESO_DRUM3 => {
                    SVF_setReso(&mut self.voiceArray[(d1 - RESO_DRUM1) as usize].filter, flt)
                }
                F_OSC1_COARSE => set_midi_freq!(self.voiceArray[0].osc, coarse, itg),
                F_OSC2_COARSE => set_midi_freq!(self.voiceArray[1].osc, coarse, itg),
                F_OSC3_COARSE => set_midi_freq!(self.voiceArray[2].osc, coarse, itg),
                F_OSC4_COARSE => set_midi_freq!(self.snareVoice.osc, coarse, itg),
                F_OSC5_COARSE => set_midi_freq!(self.cymbalVoice.osc, coarse, itg),
                F_OSC1_FINE => set_midi_freq!(self.voiceArray[0].osc, fine, itg),
                F_OSC2_FINE => set_midi_freq!(self.voiceArray[1].osc, fine, itg),
                F_OSC3_FINE => set_midi_freq!(self.voiceArray[2].osc, fine, itg),
                F_OSC4_FINE => set_midi_freq!(self.snareVoice.osc, fine, itg),
                F_OSC5_FINE => set_midi_freq!(self.cymbalVoice.osc, fine, itg),
                F_OSC6_FINE => set_midi_freq!(self.hatVoice.osc, fine, itg),
                OSC_WAVE_DRUM1 | OSC_WAVE_DRUM2 | OSC_WAVE_DRUM3 => {
                    let w = if isf { float_to_clamped_u8(nosc, flt, mosc) } else { itg };
                    self.voiceArray[(d1 - OSC_WAVE_DRUM1) as usize].osc.waveform = w;
                }
                MOD_WAVE_DRUM1 | MOD_WAVE_DRUM2 | MOD_WAVE_DRUM3 => {
                    let w = if isf { float_to_clamped_u8(nosc, flt, mosc) } else { itg };
                    self.voiceArray[(d1 - MOD_WAVE_DRUM1) as usize].modOsc.waveform = w;
                }
                OSC_WAVE_SNARE => {
                    let w = if isf { float_to_clamped_u8(nosc, flt, mosc) } else { itg };
                    self.snareVoice.osc.waveform = w;
                }
                OSC1_DIST | OSC2_DIST | OSC3_DIST => {
                    setDistortionShape(&mut self.voiceArray[(d1 - OSC1_DIST) as usize].distortion, flt)
                }
                VELOA1 => slopeEg2_setAttack(&mut self.voiceArray[0].oscVolEg, itg, AMP_EG_SYNC as u8),
                VELOD1 => slopeEg2_setDecay(&mut self.voiceArray[0].oscVolEg, itg, AMP_EG_SYNC as u8),
                PITCHD1 => DecayEg_setDecay(&mut self.voiceArray[0].oscPitchEg, itg),
                MODAMNT1 => self.voiceArray[0].egPitchModAmount = calcPitchModAmount(itg),
                FMAMNT1 => self.voiceArray[0].fmModAmount = flt,
                FMDTN1 => set_midi_freq!(self.voiceArray[0].modOsc, coarse, itg),
                VOL1 => self.voiceArray[0].vol = flt,
                PAN1 => self.setPan(0, itg),
                VELOA2 => slopeEg2_setAttack(&mut self.voiceArray[1].oscVolEg, itg, AMP_EG_SYNC as u8),
                VELOD2 => slopeEg2_setDecay(&mut self.voiceArray[1].oscVolEg, itg, AMP_EG_SYNC as u8),
                PITCHD2 => DecayEg_setDecay(&mut self.voiceArray[1].oscPitchEg, itg),
                MODAMNT2 => self.voiceArray[1].egPitchModAmount = calcPitchModAmount(itg),
                FMAMNT2 => self.voiceArray[1].fmModAmount = flt,
                FMDTN2 => set_midi_freq!(self.voiceArray[1].modOsc, coarse, itg),
                VOL2 => self.voiceArray[1].vol = flt,
                PAN2 => self.setPan(1, itg),
                VELOA3 => slopeEg2_setAttack(&mut self.voiceArray[2].oscVolEg, itg, AMP_EG_SYNC as u8),
                VELOD3 => slopeEg2_setDecay(&mut self.voiceArray[2].oscVolEg, itg, AMP_EG_SYNC as u8),
                PITCHD3 => DecayEg_setDecay(&mut self.voiceArray[2].oscPitchEg, itg),
                MODAMNT3 => self.voiceArray[2].egPitchModAmount = calcPitchModAmount(itg),
                FMAMNT3 => self.voiceArray[2].fmModAmount = flt,
                FMDTN3 => set_midi_freq!(self.voiceArray[2].modOsc, coarse, itg),
                VOL3 => self.voiceArray[2].vol = flt,
                PAN3 => self.setPan(2, itg),
                VOL4 => self.snareVoice.vol = flt,
                PAN4 => self.Snare_setPan(itg),
                SNARE_NOISE_F => self.snareVoice.noiseOsc.freq = flt * 22000.0,
                VELOA4 => slopeEg2_setAttack(&mut self.snareVoice.oscVolEg, itg, 0),
                VELOD4 => slopeEg2_setDecay(&mut self.snareVoice.oscVolEg, itg, 0),
                PITCHD4 => DecayEg_setDecay(&mut self.snareVoice.oscPitchEg, itg),
                MODAMNT4 => self.snareVoice.egPitchModAmount = calcPitchModAmount(itg),
                SNARE_FILTER_F => {
                    SVF_directSetFilterValue(&mut self.snareVoice.filter, valueShaperF2F(flt, FILTER_SHAPER))
                }
                SNARE_RESO => SVF_setReso(&mut self.snareVoice.filter, flt),
                SNARE_MIX => self.snareVoice.mix = flt,
                VOL5 => self.cymbalVoice.vol = flt,
                PAN5 => self.Cymbal_setPan(itg),
                VELOA5 => slopeEg2_setAttack(&mut self.cymbalVoice.oscVolEg, itg, 0),
                VELOD5 => slopeEg2_setDecay(&mut self.cymbalVoice.oscVolEg, itg, 0),
                CYM_WAVE1 => {
                    let w = if isf { float_to_clamped_u8(nosc, flt, mosc) } else { itg };
                    self.cymbalVoice.osc.waveform = w;
                }
                CYM_WAVE2 => {
                    let w = if isf { float_to_clamped_u8(nosc, flt, mosc) } else { itg };
                    self.cymbalVoice.modOsc.waveform = w;
                }
                CYM_WAVE3 => {
                    let w = if isf { float_to_clamped_u8(nosc, flt, mosc) } else { itg };
                    self.cymbalVoice.modOsc2.waveform = w;
                }
                CYM_MOD_OSC_F1 => set_midi_freq!(self.cymbalVoice.modOsc, coarse, itg),
                CYM_MOD_OSC_F2 => set_midi_freq!(self.cymbalVoice.modOsc2, coarse, itg),
                CYM_MOD_OSC_GAIN1 => self.cymbalVoice.fmModAmount1 = flt,
                CYM_MOD_OSC_GAIN2 => self.cymbalVoice.fmModAmount2 = flt,
                CYM_FIL_FREQ => {
                    SVF_directSetFilterValue(&mut self.cymbalVoice.filter, valueShaperF2F(flt, FILTER_SHAPER))
                }
                CYM_RESO => SVF_setReso(&mut self.cymbalVoice.filter, flt),
                CYM_REPEAT => self.cymbalVoice.oscVolEg.repeat = itg,
                CYM_SLOPE => slopeEg2_setSlope(&mut self.cymbalVoice.oscVolEg, itg),
                WAVE1_HH => {
                    let w = if isf { float_to_clamped_u8(nosc, flt, mosc) } else { itg };
                    self.hatVoice.osc.waveform = w;
                }
                WAVE2_HH => {
                    let w = if isf { float_to_clamped_u8(nosc, flt, mosc) } else { itg };
                    self.hatVoice.modOsc.waveform = w;
                }
                WAVE3_HH => {
                    let w = if isf { float_to_clamped_u8(nosc, flt, mosc) } else { itg };
                    self.hatVoice.modOsc2.waveform = w;
                }
                VELOD6 => self.hatVoice.decayClosed = slopeEg2_calcDecay(itg) as f32,
                VELOD6_OPEN => self.hatVoice.decayOpen = slopeEg2_calcDecay(itg) as f32,
                HAT_FILTER_F => {
                    SVF_directSetFilterValue(&mut self.hatVoice.filter, valueShaperF2F(flt, FILTER_SHAPER))
                }
                HAT_RESO => SVF_setReso(&mut self.hatVoice.filter, flt),
                VOL6 => self.hatVoice.vol = flt,
                PAN6 => self.HiHat_setPan(itg),
                F_OSC6_COARSE => set_midi_freq!(self.hatVoice.osc, coarse, itg),
                MOD_OSC_F1 => set_midi_freq!(self.hatVoice.modOsc, coarse, itg),
                MOD_OSC_F2 => set_midi_freq!(self.hatVoice.modOsc2, coarse, itg),
                MOD_OSC_GAIN1 => self.hatVoice.fmModAmount1 = flt,
                MOD_OSC_GAIN2 => self.hatVoice.fmModAmount2 = flt,
                VELOA6 => slopeEg2_setAttack(&mut self.hatVoice.oscVolEg, itg, 0),
                REPEAT1 => self.snareVoice.oscVolEg.repeat = itg,
                EG_SNARE1_SLOPE => slopeEg2_setSlope(&mut self.snareVoice.oscVolEg, itg),
                SNARE_DISTORTION => setDistortionShape(&mut self.snareVoice.distortion, flt),
                CYMBAL_DISTORTION => setDistortionShape(&mut self.cymbalVoice.distortion, flt),
                HAT_DISTORTION => setDistortionShape(&mut self.hatVoice.distortion, flt),
                FREQ_LFO1 | FREQ_LFO2 | FREQ_LFO3 => {
                    let i = (d1 - FREQ_LFO1) as usize;
                    let mut lfo = self.voiceArray[i].lfo;
                    self.lfo_setFreq(&mut lfo, flt);
                    self.voiceArray[i].lfo = lfo;
                }
                FREQ_LFO4 => {
                    let mut lfo = self.snareVoice.lfo;
                    self.lfo_setFreq(&mut lfo, flt);
                    self.snareVoice.lfo = lfo;
                }
                FREQ_LFO5 => {
                    let mut lfo = self.cymbalVoice.lfo;
                    self.lfo_setFreq(&mut lfo, flt);
                    self.cymbalVoice.lfo = lfo;
                }
                FREQ_LFO6 => {
                    let mut lfo = self.hatVoice.lfo;
                    self.lfo_setFreq(&mut lfo, flt);
                    self.hatVoice.lfo = lfo;
                }
                AMOUNT_LFO1 | AMOUNT_LFO2 | AMOUNT_LFO3 => {
                    self.voiceArray[(d1 - AMOUNT_LFO1) as usize].lfo.modTarget.amount = flt
                }
                AMOUNT_LFO4 => self.snareVoice.lfo.modTarget.amount = flt,
                AMOUNT_LFO5 => self.cymbalVoice.lfo.modTarget.amount = flt,
                AMOUNT_LFO6 => self.hatVoice.lfo.modTarget.amount = flt,
                VOICE_DECIMATION1..=VOICE_DECIMATION_ALL => {
                    self.mixer_decimation_rate[(d1 - VOICE_DECIMATION1) as usize] = valueShaperF2F(flt, -0.7);
                }
                _ => {}
            }
            if !isf {
                self.modNode_originalValueChanged(paramNr);
            }
        } else {
            // MIDI_CC2
            let paramNr: u16 = d1 as u16 + 1 + 127;
            if updateOriginalValue != 0 && !isf {
                self.midiParser_storeOriginalCcValue(paramNr as u32, itg);
            }
            match d1 {
                CC2_TRANS1_WAVE | CC2_TRANS2_WAVE | CC2_TRANS3_WAVE | CC2_TRANS4_WAVE | CC2_TRANS5_WAVE
                | CC2_TRANS6_WAVE => {
                    let w = if isf {
                        float_to_clamped_u8(NUM_TRANSIENTS as f32, flt, (NUM_TRANSIENTS - 1) as u8)
                    } else {
                        itg
                    };
                    let tg = match d1 {
                        CC2_TRANS1_WAVE => &mut self.voiceArray[0].transGen,
                        CC2_TRANS2_WAVE => &mut self.voiceArray[1].transGen,
                        CC2_TRANS3_WAVE => &mut self.voiceArray[2].transGen,
                        CC2_TRANS4_WAVE => &mut self.snareVoice.transGen,
                        CC2_TRANS5_WAVE => &mut self.cymbalVoice.transGen,
                        _ => &mut self.hatVoice.transGen,
                    };
                    transient_setWaveform(tg, w);
                }
                CC2_TRANS1_VOL => self.voiceArray[0].transGen.volume = flt,
                CC2_TRANS2_VOL => self.voiceArray[1].transGen.volume = flt,
                CC2_TRANS3_VOL => self.voiceArray[2].transGen.volume = flt,
                CC2_TRANS4_VOL => self.snareVoice.transGen.volume = flt,
                CC2_TRANS5_VOL => self.cymbalVoice.transGen.volume = flt,
                CC2_TRANS6_VOL => self.hatVoice.transGen.volume = flt,
                CC2_TRANS1_FREQ | CC2_TRANS2_FREQ | CC2_TRANS3_FREQ | CC2_TRANS4_FREQ | CC2_TRANS5_FREQ
                | CC2_TRANS6_FREQ => {
                    // 1.f + ((itg/33.9f) - 0.75f) == vfma(0.25, (float)(int)itg, 1/33.9f)
                    let p = vfma(0.25, itg as i32 as f32, f32::from_bits(0x3cf1a6e3));
                    match d1 {
                        CC2_TRANS1_FREQ => self.voiceArray[0].transGen.pitch = p,
                        CC2_TRANS2_FREQ => self.voiceArray[1].transGen.pitch = p,
                        CC2_TRANS3_FREQ => self.voiceArray[2].transGen.pitch = p,
                        CC2_TRANS4_FREQ => self.snareVoice.transGen.pitch = p,
                        CC2_TRANS5_FREQ => self.cymbalVoice.transGen.pitch = p,
                        _ => self.hatVoice.transGen.pitch = p,
                    }
                }
                CC2_FILTER_TYPE_1 | CC2_FILTER_TYPE_2 | CC2_FILTER_TYPE_3 | CC2_FILTER_TYPE_4 | CC2_FILTER_TYPE_5
                | CC2_FILTER_TYPE_6 => {
                    let t = if isf {
                        let t = (vfma(1.0, flt, FILTER_NAIVE_2_POLE as f32) as u32) as u8;
                        if t >= FILTER_OFF { FILTER_OFF - 1 } else { t }
                    } else {
                        itg.wrapping_add(1)
                    };
                    match d1 {
                        CC2_FILTER_TYPE_4 => self.snareVoice.filterType = t,
                        CC2_FILTER_TYPE_5 => self.cymbalVoice.filterType = t,
                        CC2_FILTER_TYPE_6 => self.hatVoice.filterType = t,
                        _ => self.voiceArray[(d1 - CC2_FILTER_TYPE_1) as usize].filterType = t,
                    }
                }
                CC2_FILTER_DRIVE_1 | CC2_FILTER_DRIVE_2 | CC2_FILTER_DRIVE_3 => {
                    SVF_setDrive(&mut self.voiceArray[(d1 - CC2_FILTER_DRIVE_1) as usize].filter, flt)
                }
                CC2_FILTER_DRIVE_4 => SVF_setDrive(&mut self.snareVoice.filter, flt),
                CC2_FILTER_DRIVE_5 => SVF_setDrive(&mut self.cymbalVoice.filter, flt),
                CC2_FILTER_DRIVE_6 => SVF_setDrive(&mut self.hatVoice.filter, flt),
                CC2_MIX_MOD_1 | CC2_MIX_MOD_2 | CC2_MIX_MOD_3 => {
                    self.voiceArray[(d1 - CC2_MIX_MOD_1) as usize].mixOscs = (itg != 0) as u8
                }
                CC2_VOLUME_MOD_ON_OFF1 | CC2_VOLUME_MOD_ON_OFF2 | CC2_VOLUME_MOD_ON_OFF3 => {
                    self.voiceArray[(d1 - CC2_VOLUME_MOD_ON_OFF1) as usize].volumeMod = (itg != 0) as u8
                }
                CC2_VOLUME_MOD_ON_OFF4 => self.snareVoice.volumeMod = (itg != 0) as u8,
                CC2_VOLUME_MOD_ON_OFF5 => self.cymbalVoice.volumeMod = (itg != 0) as u8,
                CC2_VOLUME_MOD_ON_OFF6 => self.hatVoice.volumeMod = (itg != 0) as u8,
                CC2_VELO_MOD_AMT_1 | CC2_VELO_MOD_AMT_2 | CC2_VELO_MOD_AMT_3 | CC2_VELO_MOD_AMT_4
                | CC2_VELO_MOD_AMT_5 | CC2_VELO_MOD_AMT_6 => {
                    self.velocityModulators[(d1 - CC2_VELO_MOD_AMT_1) as usize].amount = flt
                }
                CC2_WAVE_LFO1 | CC2_WAVE_LFO2 | CC2_WAVE_LFO3 | CC2_WAVE_LFO4 | CC2_WAVE_LFO5 | CC2_WAVE_LFO6 => {
                    let w = if isf {
                        float_to_clamped_u8(NUM_LFO_WAVEFORMS as f32, flt, NUM_LFO_WAVEFORMS - 1)
                    } else {
                        itg
                    };
                    match d1 {
                        CC2_WAVE_LFO4 => self.snareVoice.lfo.waveform = w,
                        CC2_WAVE_LFO5 => self.cymbalVoice.lfo.waveform = w,
                        CC2_WAVE_LFO6 => self.hatVoice.lfo.waveform = w,
                        _ => self.voiceArray[(d1 - CC2_WAVE_LFO1) as usize].lfo.waveform = w,
                    }
                }
                CC2_RETRIGGER_LFO1 | CC2_RETRIGGER_LFO2 | CC2_RETRIGGER_LFO3 | CC2_RETRIGGER_LFO4
                | CC2_RETRIGGER_LFO5 | CC2_RETRIGGER_LFO6 => {
                    let r = if isf { ((7.0 * flt) as u32) as u8 } else { itg };
                    match d1 {
                        CC2_RETRIGGER_LFO4 => self.snareVoice.lfo.retrigger = r,
                        CC2_RETRIGGER_LFO5 => self.cymbalVoice.lfo.retrigger = r,
                        CC2_RETRIGGER_LFO6 => self.hatVoice.lfo.retrigger = r,
                        _ => self.voiceArray[(d1 - CC2_RETRIGGER_LFO1) as usize].lfo.retrigger = r,
                    }
                }
                CC2_SYNC_LFO1 | CC2_SYNC_LFO2 | CC2_SYNC_LFO3 | CC2_SYNC_LFO4 | CC2_SYNC_LFO5 | CC2_SYNC_LFO6 => {
                    let mut lfo = match d1 {
                        CC2_SYNC_LFO4 => self.snareVoice.lfo,
                        CC2_SYNC_LFO5 => self.cymbalVoice.lfo,
                        CC2_SYNC_LFO6 => self.hatVoice.lfo,
                        _ => self.voiceArray[(d1 - CC2_SYNC_LFO1) as usize].lfo,
                    };
                    self.lfo_setSync(&mut lfo, itg);
                    match d1 {
                        CC2_SYNC_LFO4 => self.snareVoice.lfo = lfo,
                        CC2_SYNC_LFO5 => self.cymbalVoice.lfo = lfo,
                        CC2_SYNC_LFO6 => self.hatVoice.lfo = lfo,
                        _ => self.voiceArray[(d1 - CC2_SYNC_LFO1) as usize].lfo = lfo,
                    }
                }
                CC2_VOICE_LFO1 | CC2_VOICE_LFO2 | CC2_VOICE_LFO3 | CC2_VOICE_LFO4 | CC2_VOICE_LFO5 | CC2_VOICE_LFO6 => {}
                CC2_TARGET_LFO1 | CC2_TARGET_LFO2 | CC2_TARGET_LFO3 | CC2_TARGET_LFO4 | CC2_TARGET_LFO5
                | CC2_TARGET_LFO6 => {}
                CC2_OFFSET_LFO1 | CC2_OFFSET_LFO2 | CC2_OFFSET_LFO3 | CC2_OFFSET_LFO4 | CC2_OFFSET_LFO5
                | CC2_OFFSET_LFO6 => {
                    // value.flt * 0xffffffff: the constant is 4294967296.f; vcvt.u32 saturates
                    let o = (flt * 4294967296.0) as u32;
                    match d1 {
                        CC2_OFFSET_LFO4 => self.snareVoice.lfo.phaseOffset = o,
                        CC2_OFFSET_LFO5 => self.cymbalVoice.lfo.phaseOffset = o,
                        CC2_OFFSET_LFO6 => self.hatVoice.lfo.phaseOffset = o,
                        _ => self.voiceArray[(d1 - CC2_OFFSET_LFO1) as usize].lfo.phaseOffset = o,
                    }
                }
                CC2_AUDIO_OUT1 | CC2_AUDIO_OUT2 | CC2_AUDIO_OUT3 | CC2_AUDIO_OUT4 | CC2_AUDIO_OUT5 | CC2_AUDIO_OUT6 => {
                    self.mixer_audioRouting[(d1 - CC2_AUDIO_OUT1) as usize] = itg
                }
                CC2_MUTE_1 | CC2_MUTE_2 | CC2_MUTE_3 | CC2_MUTE_4 | CC2_MUTE_5 | CC2_MUTE_6 | CC2_MUTE_7 => {
                    let voiceNr = d1 - CC2_MUTE_1;
                    if itg == 0 {
                        self.seq_setMute(voiceNr, 0);
                    } else {
                        self.seq_setMute(voiceNr, 1);
                    }
                }
                CC2_PWM_1 | CC2_PWM_2 | CC2_PWM_3 => self.voiceArray[(d1 - CC2_PWM_1) as usize].pwm = flt,
                CC2_PWM_4 => self.snareVoice.pwm = flt,
                CC2_PWM_5 => self.cymbalVoice.pwm = flt,
                CC2_PWM_6 => self.hatVoice.pwm = flt,
                CC2_FX_TYPE => {
                    if self.fxBus.fxType != itg {
                        FxBus_prepareFxChange();
                    }
                    self.fxBus.fxType = itg;
                    self.FxBus_updateParameter();
                    if self.menu_activePage == FX_MODE_PAGE {
                        if self.menu_getSubPage() != self.fxBus.fxType {
                            let t = self.fxBus.fxType;
                            self.menu_switchSubPage(t, 1);
                        }
                    }
                }
                CC2_FX_ROUTING => self.fxBus.audioRouting = itg,
                CC2_FX_AMOUNT => self.fxBus.dryWet = (itg as i32 as f32) * f32::from_bits(0x3c23d70a),
                CC2_FX_DIST_TYPE => self.fxBus.distortion.distortionType = itg,
                CC2_FX_P1 => self.FxBus_setParameter(0, flt),
                CC2_FX_P2 => self.FxBus_setParameter(1, flt),
                CC2_FX_P3 => self.FxBus_setParameter(2, flt),
                CC2_FX_P4 => self.FxBus_setParameter(3, flt),
                CC2_FX_RATIO => Compressor_setRatio(&mut self.fxBus.compressor, itg),
                CC2_FX_WAVE => Ringmod_setWave(&mut self.fxBus.ringmod, itg),
                CC2_FX_REVERB_TYPE => {}
                CC2_FX_DELAY_TYPE => self.fxBus.delay.delayType = itg,
                _ => {}
            }
            if !isf {
                self.modNode_originalValueChanged(paramNr);
            }
        }
    }

    /// C: `void midiParser_checkMtc()`
    ///
    /// Source: `Core/Src/Soundengine/MIDI/MidiParser.c:1448`. Reference build: out-of-line in flash. Group P3.
    pub fn midiParser_checkMtc(&mut self) {
        if self.midiParser_mtcIsRunning == 0 {
            return;
        }
        if self.seq_isRunning() == 0 {
            self.midiParser_mtcIsRunning = 0;
            return;
        }
        if self.systick_ticks.wrapping_sub(self.midiParser_lastMtcReceived) > 100 * 4 {
            self.midiParser_mtcIsRunning = 0;
            self.uart_sendFrontpanelByte(FRONT_SEQ_CC as u8);
            self.uart_sendFrontpanelByte(FRONT_SEQ_RUN_STOP as u8);
            self.uart_sendFrontpanelByte(0);
            self.seq_setRunning(0);
        }
    }

    /// C: `static void midiParser_noteOn(uint8_t voice, uint8_t note, uint8_t vel, uint8_t do_rec)`
    ///
    /// Source: `Core/Src/Soundengine/MIDI/MidiParser.c:1477`. Reference build: out-of-line as a GCC clone (.part/.isra/.constprop). Group P3.
    pub fn midiParser_noteOn(&mut self, voice: u8, mut note: u8, vel: u8, do_rec: u8) {
        if self.menu_activePage == SEQ_PAGE {
            let step = self.activeParameters[PAR_ACTIVE_STEP as usize];
            let trackNr = self.menu_getActiveVoice();
            let patternNr = self.menu_getViewedPattern();
            // C indexes unchecked (PAR_ACTIVE_STEP can be set to 0..127 over MIDI): flat helpers
            let mut st = self.seq_step_read(patternNr, trackNr, step as i32);
            st.note = note;
            self.seq_step_write(patternNr, trackNr, step as i32, st);
            self.activeParameters[PAR_STEP_NOTE as usize] = note;
            self.menu_repaint();
        } else {
            if self.seq_isTrackMuted(voice) != 0 {
                return;
            }
            let ov = self.activeParameters[(PAR_MIDI_NOTE1 + voice as u16) as usize];
            if ov != 0 {
                if note == ov {
                    note = SEQ_DEFAULT_NOTE as u8;
                } else {
                    return;
                }
            }
            self.voiceControl_noteOn(voice, note, vel);
            if do_rec != 0 {
                let chan = self.midi_MidiChannels[voice as usize];
                self.seq_addNote(voice, vel, note);
                self.seq_midiNoteOff(chan);
                self.seq_sendMidiNoteOn(chan, note, vel);
            }
        }
    }

    /// C: `uint8_t midiParser_isValidChannel(uint8_t chan)`
    ///
    /// Source: `Core/Src/Soundengine/MIDI/MidiParser.c:1543`. Reference build: inlined into its callers only. Group P3.
    pub fn midiParser_isValidChannel(&mut self, chan: u8) -> u8 {
        for i in 0..8 {
            if chan == self.midi_MidiChannels[i] {
                return 1;
            }
        }
        0
    }

    /// C: `void midiParser_parseMidiMessage(MidiMsg msg)`
    ///
    /// Source: `Core/Src/Soundengine/MIDI/MidiParser.c:1717`. Reference build: out-of-line in flash. Group P3.
    pub fn midiParser_parseMidiMessage(&mut self, mut msg: MidiMsg) {
        if self.midiParser_routing.value() != 0 {
            if msg.bits.source() == midiSourceUSB as u32 {
                if self.midiParser_routing.route_usb2midi() != 0 {
                    self.uart_sendMidi(msg);
                }
            } else if msg.bits.source() == midiSourceMIDI as u32 {
                if self.midiParser_routing.route_midi2midi() != 0 {
                    self.uart_sendMidi(msg);
                }
                if self.midiParser_routing.route_midi2usb() != 0 {
                    self.usb_sendMidi(msg);
                }
            }
        }

        if msg.bits.sysxbyte() != 0 {
            return;
        }

        let sys = (self.midiParser_txRxFilter & MIDI_FILTER_SYSTEM) != 0;
        if (msg.status & 0xF0) == 0xF0 {
            match msg.status as i32 {
                MIDI_CLOCK => {
                    if sys && self.seq_getExtSync() != 0 {
                        self.seq_sync();
                    }
                }
                MIDI_START => {
                    if sys && self.seq_getExtSync() != 0 {
                        self.sync_midiResetPosition();
                        self.sync_midiStartStop(1);
                    }
                }
                MIDI_CONTINUE => {
                    if sys && self.seq_getExtSync() != 0 {
                        self.sync_midiStartStop(1);
                    }
                }
                MIDI_STOP => {
                    if sys && self.seq_getExtSync() != 0 {
                        self.sync_midiStartStop(0);
                    }
                }
                MIDI_SONG_POSITION_POINTER => {}
                MIDI_MTC_QFRAME => {
                    if !sys {
                        return;
                    }
                    if (msg.data1 & 0x70) == 0 {
                        self.midiParser_lastMtcReceived = self.systick_ticks;
                    }
                    if self.seq_isRunning() != 0 {
                        return;
                    }
                    if self.seq_getExtSync() != 0 {
                        return;
                    }
                    if (msg.data1 & 0x7F) == 0 {
                        self.midiParser_mtcIgnore = 0;
                    } else if self.midiParser_mtcIgnore != 0 {
                        return;
                    } else if (msg.data1 & 0x70) != 0x70 {
                        if (msg.data1 & 0x0F) != 0 {
                            self.midiParser_mtcIgnore = 1;
                            return;
                        }
                    } else if (msg.data1 & 0x01) == 0 {
                        self.uart_sendFrontpanelByte(FRONT_SEQ_CC as u8);
                        self.uart_sendFrontpanelByte(FRONT_SEQ_RUN_STOP as u8);
                        self.uart_sendFrontpanelByte(1);
                        self.midiParser_mtcIgnore = 1;
                        self.midiParser_mtcIsRunning = 1;
                        self.midiParser_lastMtcReceived = self.systick_ticks;
                        self.seq_setRunning(1);
                    }
                }
                _ => {}
            }
        } else {
            let msgonly = msg.status & 0xF0;
            let chanonly = msg.status & 0x0F;
            msg.lastNonSysStatus = msg.status;

            if (msgonly & 0xE0) == 0x80 {
                if (self.midiParser_txRxFilter & MIDI_FILTER_NOTES) != 0 {
                    let mut q: i8 = -1;
                    if self.midi_MidiChannels[7] == chanonly {
                        q = self.frontParser_activeTrack as i8;
                        if msgonly as i32 == NOTE_ON && msg.data2 != 0 {
                            let t = self.frontParser_activeTrack;
                            self.midiParser_noteOn(t, msg.data1, msg.data2, 1);
                        }
                    }
                    for v in 0i8..7 {
                        if self.midi_MidiChannels[v as usize] == chanonly && v != q {
                            if msgonly as i32 == NOTE_ON && msg.data2 != 0 {
                                self.midiParser_noteOn(v as u8, msg.data1, msg.data2, 1);
                            }
                        }
                    }
                }
            } else if msgonly as i32 == PROG_CHANGE {
                if (self.midiParser_txRxFilter & MIDI_FILTER_PROG_C) != 0 && chanonly == self.midi_MidiChannels[7] {
                    self.seq_setNextPattern(msg.data1);
                }
            } else if msgonly as i32 == MIDI_CC {
                let voiceNr = midiParser_cc2voiceNr(msg.data1);
                let accept = if voiceNr == 0 || voiceNr >= 7 {
                    (self.midiParser_txRxFilter & MIDI_FILTER_CC) != 0 && self.midiParser_isValidChannel(chanonly) != 0
                } else {
                    (self.midiParser_txRxFilter & MIDI_FILTER_CC) != 0
                        && (chanonly == self.midi_MidiChannels[voiceNr as usize] || chanonly == self.midi_MidiChannels[7])
                };
                if accept {
                    let t = self.frontParser_activeTrack;
                    self.seq_recordAutomation(t, msg.data1, msg.data2);
                    self.midiParser_ccHandler(msg, 1, 0, 0.0);
                    self.uart_sendFrontpanelByte(msg.status);
                    self.uart_sendFrontpanelByte(msg.data1);
                    self.uart_sendFrontpanelByte(msg.data2);
                }
            }
        }
    }

    /// C: `void midiParser_handleStatusByte(unsigned char data)`
    ///
    /// Source: `Core/Src/Soundengine/MIDI/MidiParser.c:1923`. Reference build: inlined into its callers only. Group P3.
    pub fn midiParser_handleStatusByte(&mut self, data: u8) {
        match (data & 0xF0) as i32 {
            NOTE_OFF | NOTE_ON | MIDI_CC | MIDI_PITCH_WHEEL | MIDI_AT => {
                self.midiMsg_tmp.status = data;
                self.parserState = MIDI_DATA1;
                self.midiMsg_tmp.bits.set_length(2);
            }
            PROG_CHANGE | CHANNEL_PRESSURE => {
                self.midiMsg_tmp.status = data;
                self.parserState = MIDI_DATA1;
                self.midiMsg_tmp.bits.set_length(1);
            }
            _ => {
                self.parserState = IGNORE;
                self.midiMsg_tmp.bits.set_length(0);
            }
        }
        self.midiMsg_tmp.lastNonSysStatus = data;
    }

    /// C: `void midiParser_parseUartData(unsigned char data)`
    ///
    /// Source: `Core/Src/Soundengine/MIDI/MidiParser.c:1962`. Reference build: out-of-line in flash. Group P3.
    pub fn midiParser_parseUartData(&mut self, data: u8) {
        self.checkMIDIprocPAR_t = (self.checkMIDIprocPAR_t ^ 1) & 0x01;
        self.checkMIDIprocPAR = (self.checkMIDIprocPAR_t as i8).wrapping_sub(6);

        let mut parseMsg = false;
        if data & 0x80 != 0 {
            self.midiMsg_tmp.bits.set_sysxbyte(0);
            if (data & 0xf0) == 0xF0 {
                self.midiMsg_tmp.status = data;
                match data as i32 {
                    MIDI_CLOCK => parseMsg = true,
                    SYSEX_START => {
                        self.parserState = SYSEX_DATA;
                        self.midiMsg_tmp.bits.set_length(0);
                        parseMsg = true;
                    }
                    SYSEX_END => {
                        if self.parserState == SYSEX_DATA {
                            self.parserState = MIDI_STATUS;
                            self.midiMsg_tmp.bits.set_length(0);
                            parseMsg = true;
                        }
                    }
                    MIDI_SONG_SEL => parseMsg = true,
                    MIDI_MTC_QFRAME => {
                        self.parserState = MIDI_DATA1;
                        self.midiMsg_tmp.bits.set_length(1);
                    }
                    MIDI_SONG_POSITION_POINTER => {
                        self.parserState = MIDI_DATA1;
                        self.midiMsg_tmp.bits.set_length(2);
                    }
                    _ => parseMsg = true,
                }
            } else {
                self.midiParser_handleStatusByte(data);
            }
        } else {
            match self.parserState {
                MIDI_STATUS => {
                    self.midiMsg_tmp.status = self.midiMsg_tmp.lastNonSysStatus;
                    if self.midiMsg_tmp.bits.length() != 0 {
                        self.midiMsg_tmp.data1 = data;
                        if self.midiMsg_tmp.bits.length() == 2 {
                            self.parserState = MIDI_DATA2;
                        } else {
                            self.midiMsg_tmp.data2 = 0;
                            parseMsg = true;
                        }
                    }
                }
                MIDI_DATA1 => {
                    self.midiMsg_tmp.data1 = data;
                    if self.midiMsg_tmp.bits.length() == 2 {
                        self.parserState = MIDI_DATA2;
                    } else {
                        parseMsg = true;
                    }
                }
                MIDI_DATA2 => {
                    self.midiMsg_tmp.data2 = data;
                    parseMsg = true;
                }
                SYSEX_DATA => {
                    self.midiMsg_tmp.bits.set_sysxbyte(1);
                    self.midiMsg_tmp.status = data;
                    self.midiMsg_tmp.bits.set_length(0);
                    parseMsg = true;
                }
                _ => {}
            }
        }

        if parseMsg {
            if self.parserState != SYSEX_DATA {
                self.parserState = MIDI_STATUS;
            }
            self.midiMsg_tmp.bits.set_source(midiSourceMIDI as u32);
            let m = self.midiMsg_tmp;
            self.midiParser_parseMidiMessage(m);
        }
    }

    /// C: `void midiParser_setRouting(uint8_t value)`
    ///
    /// Source: `Core/Src/Soundengine/MIDI/MidiParser.c:2103`. Reference build: out-of-line in flash. Group P3.
    pub fn midiParser_setRouting(&mut self, value: u8) {
        self.midiParser_routing.set_value(0);
        match value {
            1 => self.midiParser_routing.set_route_usb2midi(1),
            2 => self.midiParser_routing.set_route_midi2midi(1),
            3 => {
                self.midiParser_routing.set_route_usb2midi(1);
                self.midiParser_routing.set_route_midi2midi(1);
            }
            4 => self.midiParser_routing.set_route_midi2usb(1),
            5 => {
                self.midiParser_routing.set_route_midi2midi(1);
                self.midiParser_routing.set_route_midi2usb(1);
            }
            _ => {}
        }
    }

    /// C: `void midiParser_setFilter(uint8_t is_tx, uint8_t value)`
    ///
    /// Source: `Core/Src/Soundengine/MIDI/MidiParser.c:2133`. Reference build: out-of-line in flash. Group P3.
    pub fn midiParser_setFilter(&mut self, is_tx: u8, value: u8) {
        if is_tx != 0 {
            self.midiParser_txRxFilter = (value << 4) | (self.midiParser_txRxFilter & 0x0F);
        } else {
            self.midiParser_txRxFilter = (value & 0x0F) | (self.midiParser_txRxFilter & 0xF0);
        }
    }
}
