//! Port of `Core/Inc/Soundengine/Soundengine.h`, `Core/Src/Soundengine/Soundengine.c`.
//!
//! Porting group: P3. See `docs/PORTING_STATUS.md` and the rules in `docs/ARCHITECTURE.md`.
//! Generated skeleton: names, field order and C types follow the firmware sources and the DWARF of the
//! reference ELF. Function bodies are `todo!()` until ported.

use crate::prelude::*;

// ---- functions that use globals (methods on `Lxr`)
impl Lxr {
    /// C: `uint8_t soundengine_calcNextSampleBlock()`
    ///
    /// Source: `Core/Src/Soundengine/Soundengine.c:21`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `bCurrentSampleValid`, `dma_buffer`, `dma_buffer2`.
    /// Calls: `Lxr::mixer_calcNextSampleBlock`.
    pub fn soundengine_calcNextSampleBlock(&mut self) -> u8 {
        todo!("port: Core/Src/Soundengine/Soundengine.c:21")
    }

    /// C: `void soundengine_init()`
    ///
    /// Source: `Core/Src/Soundengine/Soundengine.c:38`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `bCurrentSampleValid`, `dma_buffer`, `dma_buffer2`, `midi_MidiChannels`, `velocityModulators`.
    /// Calls: `Lxr::CodecInit`, `Lxr::Cymbal_init`, `Lxr::HiHat_init`, `Lxr::Snare_init`, `Lxr::euklid_init`, `Lxr::initDrumVoice`, `Lxr::mixer_init`, `Lxr::modNode_init`, `Lxr::parameterArray_init`, `Lxr::seq_init`, `Lxr::seq_setBpm`, `Lxr::soundengine_calcNextSampleBlock`.
    pub fn soundengine_init(&mut self) {
        todo!("port: Core/Src/Soundengine/Soundengine.c:38")
    }

    /// C: `void soundengine_process()`
    ///
    /// Source: `Core/Src/Soundengine/Soundengine.c:77`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `bCurrentSampleValid`, `dma_buffer`, `dma_buffer2`.
    /// Calls: `Lxr::preset_morph_tick`, `Lxr::seq_tick`, `Lxr::soundengine_calcNextSampleBlock`, `Lxr::uart_processMidi`, `Lxr::usb_tick`.
    pub fn soundengine_process(&mut self) {
        todo!("port: Core/Src/Soundengine/Soundengine.c:77")
    }
}
