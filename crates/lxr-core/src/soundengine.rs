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
        if self.bCurrentSampleValid as i32 != SAMPLE_VALID {
            // USE_DAC2: mixer_calcNextSampleBlock(&dma_buffer[half*128], &dma_buffer2[half*128])
            const N: usize = (OUTPUT_DMA_SIZE * 2) as usize;
            let off = self.bCurrentSampleValid as usize * N;
            let mut out: [i16; N] = self.dma_buffer[off..off + N].try_into().unwrap();
            let mut out2: [i16; N] = self.dma_buffer2[off..off + N].try_into().unwrap();
            self.mixer_calcNextSampleBlock(&mut out, &mut out2);
            self.dma_buffer[off..off + N].copy_from_slice(&out);
            self.dma_buffer2[off..off + N].copy_from_slice(&out2);
            self.bCurrentSampleValid = SAMPLE_VALID as u8;
            return 1;
        }
        0
    }

    /// C: `void soundengine_init()`
    ///
    /// Source: `Core/Src/Soundengine/Soundengine.c:38`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `bCurrentSampleValid`, `dma_buffer`, `dma_buffer2`, `midi_MidiChannels`, `velocityModulators`.
    /// Calls: `Lxr::CodecInit`, `Lxr::Cymbal_init`, `Lxr::HiHat_init`, `Lxr::Snare_init`, `Lxr::euklid_init`, `Lxr::initDrumVoice`, `Lxr::mixer_init`, `Lxr::modNode_init`, `Lxr::parameterArray_init`, `Lxr::seq_init`, `Lxr::seq_setBpm`, `Lxr::soundengine_calcNextSampleBlock`.
    pub fn soundengine_init(&mut self) {
        self.mixer_init();

        // start the audio codec
        self.CodecInit();

        self.parameterArray_init();

        self.euklid_init();

        for i in 0..6usize {
            self.modNode_init(move |s: &mut Lxr| &mut s.velocityModulators[i]);
        }

        // init the seq speed
        self.seq_init();
        self.seq_setBpm(120);

        self.midi_MidiChannels = [0; 8];

        self.initDrumVoice();
        self.Snare_init();
        self.HiHat_init();
        self.Cymbal_init();

        // precalc the first dma buffer block
        self.soundengine_calcNextSampleBlock();

        // FPU->FPDSCR |= FZ: only affects exception handlers, which the engine does not model
        // (all engine code runs in thread mode with FZ = 0, docs/ENGINE_CONTRACT.md section 1).
    }

    /// C: `void soundengine_process()`
    ///
    /// Source: `Core/Src/Soundengine/Soundengine.c:77`. Reference build: out-of-line in flash. Group P3.
    /// Globals: `bCurrentSampleValid`, `dma_buffer`, `dma_buffer2`.
    /// Calls: `Lxr::preset_morph_tick`, `Lxr::seq_tick`, `Lxr::soundengine_calcNextSampleBlock`, `Lxr::uart_processMidi`, `Lxr::usb_tick`.
    pub fn soundengine_process(&mut self) {
        // The reference build has CALC_SAMPLES_IN_INTERRUPT = 1: TIM2 update interrupt disabled around
        // seq_tick + soundengine_calcNextSampleBlock and its counter set to 1950 (hardware only).
        self.seq_tick();
        self.soundengine_calcNextSampleBlock();

        // process midi on midi port
        self.uart_processMidi();

        self.usb_tick();

        self.preset_morph_tick();
    }
}
