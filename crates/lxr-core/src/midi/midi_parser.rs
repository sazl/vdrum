//! Port of `Core/Inc/Soundengine/MIDI/MidiParser.h`, `Core/Src/Soundengine/MIDI/MidiParser.c`.
//!
//! Porting group: P3. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;

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

// ---- functions without access to globals
/// C: `static inline float calcPitchModAmount(uint8_t data2)`
///
/// Source: `Core/Src/Soundengine/MIDI/MidiParser.c:95`. Reference build: inlined into its callers only. Group P3.
pub fn calcPitchModAmount(data2: u8) -> f32 {
    todo!("port: Core/Src/Soundengine/MIDI/MidiParser.c:95")
}

/// C: `float midiParser_calcDetune(uint8_t value)`
///
/// Source: `Core/Src/Soundengine/MIDI/MidiParser.c:118`. Reference build: out-of-line in flash. Group P3.
pub fn midiParser_calcDetune(value: u8) -> f32 {
    // Ported by P1 (needed by osc_setBaseNote / osc_recalcFreq; difftested in p1_oscillator.rs).
    // frac = vfma(-0.5, value, 1/127.f); return vfma(1, frac, SEMITONE_UP - 1)
    let frac = vfma(-0.5, value as i32 as f32, f32::from_bits(0x3c010204));
    vfma(1.0, frac, f32::from_bits(0x3d738fa0))
}

/// C: `uint8_t midiParser_cc2voiceNr(uint8_t cc)`
///
/// Source: `Core/Src/Soundengine/MIDI/MidiParser.c:1553`. Reference build: inlined into its callers only. Group P3.
pub fn midiParser_cc2voiceNr(cc: u8) -> u8 {
    todo!("port: Core/Src/Soundengine/MIDI/MidiParser.c:1553")
}

// ---- functions that use globals (methods on `Lxr`)
impl Lxr {
    /// C: `static void midiParser_nrpnHandler(uint16_t value)`
    ///
    /// Source: `Core/Src/Soundengine/MIDI/MidiParser.c:134`. Reference build: inlined into its callers only. Group P3.
    /// Globals: `midiParser_activeNrpnNumber`.
    /// Calls: `Lxr::midiParser_ccHandler`.
    pub fn midiParser_nrpnHandler(&mut self, value: u16) {
        todo!("port: Core/Src/Soundengine/MIDI/MidiParser.c:134")
    }

    /// C: `void midiParser_ccHandler(MidiMsg msg, uint8_t updateOriginalValue, uint8_t isFloatModulator, float v)`
    ///
    /// Source: `Core/Src/Soundengine/MIDI/MidiParser.c:144`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `cymbalVoice`, `fxBus`, `hatVoice`, `menu_activePage`, `midiParser_activeNrpnNumber`, `midiParser_originalCcValues`, `mixer_audioRouting`, `mixer_decimation_rate`, `snareVoice`, `velocityModulators`, `voiceArray`.
    /// Calls: `dsp::fx::compressor::Compressor_setRatio`, `Lxr::Cymbal_setPan`, `dsp::decay::DecayEg_setDecay`, `dsp::decay::DecayEg_setSlope`, `fx_bus::FxBus_prepareFxChange`, `Lxr::FxBus_setParameter`, `Lxr::FxBus_updateParameter`, `Lxr::HiHat_setPan`, `dsp::fx::ringmod::Ringmod_setWave`, `dsp::resonant_filter::SVF_directSetFilterValue`, `dsp::resonant_filter::SVF_setDrive`, `dsp::resonant_filter::SVF_setReso`, `Lxr::Snare_setPan`, `midi::midi_parser::calcPitchModAmount`, `Lxr::lfo_setFreq`, `Lxr::lfo_setSync`, `Lxr::menu_getSubPage`, `Lxr::menu_switchSubPage`, `Lxr::midiParser_nrpnHandler`, `Lxr::modNode_originalValueChanged`, `dsp::oscillator::osc_recalcFreq`, `Lxr::seq_setMute`, `dsp::distortion::setDistortionShape`, `Lxr::setPan`, `dsp::slope_eg2::slopeEg2_calcDecay`, `dsp::slope_eg2::slopeEg2_setAttack`, `dsp::slope_eg2::slopeEg2_setDecay`, `dsp::slope_eg2::slopeEg2_setSlope`, `dsp::transient_generator::transient_setWaveform`, `midi::value_shaper::valueShaperF2F`.
    pub fn midiParser_ccHandler(&mut self, msg: MidiMsg, updateOriginalValue: u8, isFloatModulator: u8, v: f32) {
        todo!("port: Core/Src/Soundengine/MIDI/MidiParser.c:144")
    }

    /// C: `void midiParser_checkMtc()`
    ///
    /// Source: `Core/Src/Soundengine/MIDI/MidiParser.c:1448`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `midiParser_lastMtcReceived`, `midiParser_mtcIsRunning`, `systick_ticks`.
    /// Calls: `Lxr::seq_isRunning`, `Lxr::seq_setRunning`, `Lxr::uart_sendFrontpanelByte`.
    pub fn midiParser_checkMtc(&mut self) {
        todo!("port: Core/Src/Soundengine/MIDI/MidiParser.c:1448")
    }

    /// C: `static void midiParser_noteOn(uint8_t voice, uint8_t note, uint8_t vel, uint8_t do_rec)`
    ///
    /// Source: `Core/Src/Soundengine/MIDI/MidiParser.c:1477`. Reference build: out-of-line as a GCC clone (.part/.isra/.constprop). Group P3.
    /// Globals: `activeParameters`, `menu_activePage`, `midi_MidiChannels`, `seq_patternSet`.
    /// Calls: `Lxr::menu_getActiveVoice`, `Lxr::menu_getViewedPattern`, `Lxr::menu_repaint`, `Lxr::seq_addNote`, `Lxr::seq_isTrackMuted`, `Lxr::seq_midiNoteOff`, `Lxr::seq_sendMidiNoteOn`, `Lxr::voiceControl_noteOn`.
    pub fn midiParser_noteOn(&mut self, voice: u8, note: u8, vel: u8, do_rec: u8) {
        todo!("port: Core/Src/Soundengine/MIDI/MidiParser.c:1477")
    }

    /// C: `uint8_t midiParser_isValidChannel(uint8_t chan)`
    ///
    /// Source: `Core/Src/Soundengine/MIDI/MidiParser.c:1543`. Reference build: inlined into its callers only. Group P3.
    /// Globals: `midi_MidiChannels`.
    pub fn midiParser_isValidChannel(&mut self, chan: u8) -> u8 {
        todo!("port: Core/Src/Soundengine/MIDI/MidiParser.c:1543")
    }

    /// C: `void midiParser_parseMidiMessage(MidiMsg msg)`
    ///
    /// Source: `Core/Src/Soundengine/MIDI/MidiParser.c:1717`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `frontParser_activeTrack`, `midiParser_lastMtcReceived`, `midiParser_mtcIgnore`, `midiParser_mtcIsRunning`, `midiParser_routing`, `midiParser_txRxFilter`, `midi_MidiChannels`, `systick_ticks`.
    /// Calls: `midi::midi_parser::midiParser_cc2voiceNr`, `Lxr::midiParser_ccHandler`, `Lxr::midiParser_isValidChannel`, `Lxr::midiParser_noteOn`, `Lxr::seq_getExtSync`, `Lxr::seq_isRunning`, `Lxr::seq_recordAutomation`, `Lxr::seq_setNextPattern`, `Lxr::seq_setRunning`, `Lxr::seq_sync`, `Lxr::sync_midiResetPosition`, `Lxr::sync_midiStartStop`, `Lxr::uart_sendFrontpanelByte`, `Lxr::uart_sendMidi`, `Lxr::usb_sendMidi`.
    pub fn midiParser_parseMidiMessage(&mut self, msg: MidiMsg) {
        todo!("port: Core/Src/Soundengine/MIDI/MidiParser.c:1717")
    }

    /// C: `void midiParser_handleStatusByte(unsigned char data)`
    ///
    /// Source: `Core/Src/Soundengine/MIDI/MidiParser.c:1923`. Reference build: inlined into its callers only. Group P3.
    /// Globals: `midiMsg_tmp`, `parserState`.
    pub fn midiParser_handleStatusByte(&mut self, data: u8) {
        todo!("port: Core/Src/Soundengine/MIDI/MidiParser.c:1923")
    }

    /// C: `void midiParser_parseUartData(unsigned char data)`
    ///
    /// Source: `Core/Src/Soundengine/MIDI/MidiParser.c:1962`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `checkMIDIprocPAR`, `checkMIDIprocPAR_t`, `midiMsg_tmp`, `parserState`.
    /// Calls: `Lxr::midiParser_handleStatusByte`, `Lxr::midiParser_parseMidiMessage`.
    pub fn midiParser_parseUartData(&mut self, data: u8) {
        todo!("port: Core/Src/Soundengine/MIDI/MidiParser.c:1962")
    }

    /// C: `void midiParser_setRouting(uint8_t value)`
    ///
    /// Source: `Core/Src/Soundengine/MIDI/MidiParser.c:2103`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `midiParser_routing`.
    pub fn midiParser_setRouting(&mut self, value: u8) {
        todo!("port: Core/Src/Soundengine/MIDI/MidiParser.c:2103")
    }

    /// C: `void midiParser_setFilter(uint8_t is_tx, uint8_t value)`
    ///
    /// Source: `Core/Src/Soundengine/MIDI/MidiParser.c:2133`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `midiParser_txRxFilter`.
    pub fn midiParser_setFilter(&mut self, is_tx: u8, value: u8) {
        todo!("port: Core/Src/Soundengine/MIDI/MidiParser.c:2133")
    }
}
