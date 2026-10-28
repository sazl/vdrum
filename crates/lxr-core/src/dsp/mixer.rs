//! Port of `Core/Inc/Soundengine/DSPAudio/mixer.h`, `Core/Src/Soundengine/DSPAudio/mixer.c`.
//!
//! Porting group: P4. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;

// ---- #define constants
/// `#define USE_SWITCH_ROUTING 1` (Core/Inc/Soundengine/DSPAudio/mixer.h:20)
pub const USE_SWITCH_ROUTING: i32 = 1;
/// `#define USE_DECIMATOR 1` (Core/Inc/Soundengine/DSPAudio/mixer.h:21)
pub const USE_DECIMATOR: i32 = 1;
/// `#define USE_TRIGGERED_PAN 1` (Core/Inc/Soundengine/DSPAudio/mixer.h:22)
pub const USE_TRIGGERED_PAN: i32 = 1;

// ---- enums
// C: `enum {...}` (Core/Inc/Soundengine/DSPAudio/mixer.h:31), 1 bytes, underlying type `u8`. (anonymous: enumerators only)
pub const MIXER_ROUTING_DAC1_STEREO: u8 = 0;
pub const MIXER_ROUTING_DAC2_STEREO: u8 = 1;
pub const MIXER_ROUTING_DAC1_L: u8 = 2;
pub const MIXER_ROUTING_DAC1_R: u8 = 3;
pub const MIXER_ROUTING_DAC2_L: u8 = 4;
pub const MIXER_ROUTING_DAC2_R: u8 = 5;
pub const MIXER_ROUTING_FX_BUS: u8 = 6;

// ---- functions that use globals (methods on `Lxr`)
impl Lxr {
    /// C: `void mixer_init()`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/mixer.c:43`. Reference build: out-of-line in flash. Group P4.
    /// Globals: `mixer_audioRouting`, `mixer_decimation_cnt`, `mixer_decimation_rate`, `mixer_voice_samples`.
    /// Calls: `Lxr::FxBus_init`.
    pub fn mixer_init(&mut self) {
        todo!("port: Core/Src/Soundengine/DSPAudio/mixer.c:43")
    }

    /// C: `IN_ITCM_RAM void mixer_decimateBlock(const uint8_t voiceNr, int16_t* buffer)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/mixer.c:60`. Reference build: out-of-line in ITCM. Group P4.
    /// Globals: `mixer_decimation_cnt`, `mixer_decimation_rate`, `mixer_voice_samples`.
    pub fn mixer_decimateBlock(&mut self, voiceNr: u8, buffer: &mut [i16]) {
        todo!("port: Core/Src/Soundengine/DSPAudio/mixer.c:60")
    }

    /// C: `uint8_t mixer_checkOutJackAvailable(uint8_t dest)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/mixer.c:79`. Reference build: out-of-line in flash. Group P4.
    /// Globals: `l1_Available`, `l2_Available`, `r1_Available`, `r2_Available`.
    pub fn mixer_checkOutJackAvailable(&mut self, dest: u8) -> u8 {
        todo!("port: Core/Src/Soundengine/DSPAudio/mixer.c:79")
    }

    /// C: `IN_ITCM_RAM static inline void mixer_addDataToOutputFilteredPan(uint8_t dest, const float pan, int16_t* data,int16_t* outL,int16_t* outR,int16_t* outL2, int16_t* outR2, int16_t *fxBuffer, OnePoleFilter* filter)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/mixer.c:310`. Reference build: not in the reference ELF (file not linked or function optimised away). Group P4.
    /// Calls: `cmsis::__QADD16`, `dsp::one_pole_lp::calcOnePole`, `Lxr::mixer_checkOutJackAvailable`.
    pub fn mixer_addDataToOutputFilteredPan(&mut self, dest: u8, pan: f32, data: &mut [i16], outL: &mut [i16], outR: &mut [i16], outL2: &mut [i16], outR2: &mut [i16], fxBuffer: &mut [i16], filter: &mut OnePoleFilter) {
        todo!("port: Core/Src/Soundengine/DSPAudio/mixer.c:310")
    }

    /// C: `IN_ITCM_RAM static inline void mixer_addDataToOutput(uint8_t dest, const float panL, const float panR, int16_t* data,int16_t* outL,int16_t* outR,int16_t* outL2, int16_t* outR2, int16_t *fxBuffer)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/mixer.c:438`. Reference build: inlined into its callers only. Group P4.
    /// Calls: `cmsis::__QADD16`, `Lxr::mixer_checkOutJackAvailable`.
    pub fn mixer_addDataToOutput(&mut self, dest: u8, panL: f32, panR: f32, data: &mut [i16], outL: &mut [i16], outR: &mut [i16], outL2: &mut [i16], outR2: &mut [i16], fxBuffer: &mut [i16]) {
        todo!("port: Core/Src/Soundengine/DSPAudio/mixer.c:438")
    }

    /// C: `IN_ITCM_RAM void mixer_calcNextSampleBlock(int16_t* output,int16_t* output2)`
    ///
    /// Source: `Core/Src/Soundengine/DSPAudio/mixer.c:548`. Reference build: out-of-line in ITCM. Group P4.
    /// Globals: `cymbalVoice`, `hatVoice`, `l1_Available`, `l2_Available`, `mixer_audioRouting`, `mixer_decimation_rate`, `r1_Available`, `r2_Available`, `snareVoice`, `voiceArray`.
    /// Calls: `Lxr::Cymbal_calcAsync`, `Lxr::Cymbal_calcSyncBlock`, `Lxr::FxBus_process`, `Lxr::HiHat_calcAsync`, `Lxr::HiHat_calcSyncBlock`, `dsp::resonant_filter::SVF_recalcFreq`, `Lxr::Snare_calcAsync`, `Lxr::Snare_calcSyncBlock`, `dsp::buffer_tools::bufferTool_clearBuffer`, `Lxr::calcDrumVoiceAsync`, `Lxr::calcDrumVoiceSyncBlock`, `Lxr::lfo_dispatchNextValue`, `Lxr::mixer_addDataToOutput`, `Lxr::mixer_decimateBlock`, `Lxr::modNode_reassignVeloMod`, `Lxr::modNode_resetTargets`.
    pub fn mixer_calcNextSampleBlock(&mut self, output: &mut [i16], output2: &mut [i16]) {
        todo!("port: Core/Src/Soundengine/DSPAudio/mixer.c:548")
    }
}
