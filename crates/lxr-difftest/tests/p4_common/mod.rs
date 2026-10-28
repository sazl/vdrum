//! Shared helpers for the P4 (voices, mixer) differential tests: C-layout marshalling of the voice structs
//! (offsets from `lxr_core::memmap::offset_of_c`, i.e. the DWARF of the reference ELF), and syncing the P4
//! globals between `Lxr` and the emulator.
#![allow(dead_code, non_snake_case)]

use lxr_core::memmap::{self, offset_of_c as O, size_of_c as S};
use lxr_core::prelude::*;
use lxr_difftest::Firmware;
use std::cell::Cell;
use std::rc::Rc;

/// A value with a C memory layout in the firmware.
pub trait CMem: Sized {
    const SIZE: usize;
    fn rd(b: &[u8], o: usize) -> Self;
    fn wr(&self, b: &mut [u8], o: usize);
}

macro_rules! prim {
    ($t:ty, $n:expr) => {
        impl CMem for $t {
            const SIZE: usize = $n;
            fn rd(b: &[u8], o: usize) -> Self {
                <$t>::from_le_bytes(b[o..o + $n].try_into().unwrap())
            }
            fn wr(&self, b: &mut [u8], o: usize) {
                b[o..o + $n].copy_from_slice(&self.to_le_bytes());
            }
        }
    };
}
prim!(u8, 1);
prim!(i8, 1);
prim!(u16, 2);
prim!(i16, 2);
prim!(u32, 4);
prim!(i32, 4);
prim!(f32, 4);
prim!(f64, 8);

impl CMem for ptrValue {
    const SIZE: usize = 4;
    fn rd(b: &[u8], o: usize) -> Self {
        ptrValue { itg: u32::rd(b, o) }
    }
    fn wr(&self, b: &mut [u8], o: usize) {
        self.itg.wr(b, o)
    }
}

impl<T: CMem, const N: usize> CMem for [T; N] {
    const SIZE: usize = T::SIZE * N;
    fn rd(b: &[u8], o: usize) -> Self {
        core::array::from_fn(|i| T::rd(b, o + i * T::SIZE))
    }
    fn wr(&self, b: &mut [u8], o: usize) {
        for (i, x) in self.iter().enumerate() {
            x.wr(b, o + i * T::SIZE);
        }
    }
}

// Generated from the lxr-core struct definitions (field order) and `offset_of_c` (DWARF offsets).
include!("layout_gen.rs");

/// Bytes of `x` in C layout (padding zero): bitwise comparison of whole structs.
pub fn bytes<T: CMem>(x: &T) -> Vec<u8> {
    let mut b = vec![0u8; T::SIZE];
    x.wr(&mut b, 0);
    b
}

/// Read a `T` from firmware memory.
pub fn get<T: CMem>(fw: &Firmware, addr: u32) -> T {
    let b = fw.read(addr, T::SIZE).unwrap();
    T::rd(&b, 0)
}

/// Write a `T` into firmware memory (padding bytes keep their contents).
pub fn put<T: CMem>(fw: &mut Firmware, addr: u32, x: &T) {
    let mut b = fw.read(addr, T::SIZE).unwrap();
    x.wr(&mut b, 0);
    fw.write(addr, &b).unwrap();
}

/// Assert that two values are bit-identical in C layout; on mismatch, name the first differing byte offset.
#[track_caller]
pub fn assert_same<T: CMem>(what: &str, rust: &T, fw: &T) {
    let a = bytes(rust);
    let b = bytes(fw);
    if a != b {
        let i = a.iter().zip(&b).position(|(x, y)| x != y).unwrap();
        let s = i & !3;
        let e = (s + 8).min(a.len());
        panic!("{what}: differs at byte offset {i} (rust {:02x?} vs fw {:02x?})", &a[s..e], &b[s..e]);
    }
}

pub const FADER_FULL: f32 = f32::from_bits(0x3F7F_E001);

/// Write the firmware globals that P4 code reads or writes from `Lxr`.
pub fn push_state(fw: &mut Firmware, l: &Lxr) {
    put(fw, memmap::ADDR_voiceArray, &l.voiceArray);
    put(fw, memmap::ADDR_snareVoice, &l.snareVoice);
    put(fw, memmap::ADDR_hatVoice, &l.hatVoice);
    put(fw, memmap::ADDR_cymbalVoice, &l.cymbalVoice);
    put(fw, memmap::ADDR_velocityModulators, &l.velocityModulators);
    put(fw, memmap::ADDR_mixer_decimation_rate, &l.mixer_decimation_rate);
    put(fw, memmap::ADDR_mixer_decimation_cnt, &l.mixer_decimation_cnt);
    put(fw, memmap::ADDR_mixer_voice_samples, &l.mixer_voice_samples);
    put(fw, memmap::ADDR_mixer_audioRouting, &l.mixer_audioRouting);
    put(fw, memmap::ADDR_adc_faderValues, &l.adc_faderValues);
    put(fw, memmap::ADDR_l1_Available, &l.l1_Available);
    put(fw, memmap::ADDR_l2_Available, &l.l2_Available);
    put(fw, memmap::ADDR_r1_Available, &l.r1_Available);
    put(fw, memmap::ADDR_r2_Available, &l.r2_Available);
}

/// Compare every P4 global of `Lxr` with the firmware memory, bit for bit.
#[track_caller]
pub fn check_state(ctx: &str, fw: &Firmware, l: &Lxr) {
    for v in 0..3u32 {
        let f: DrumVoice = get(fw, memmap::ADDR_voiceArray + v * S::DrumVoice);
        assert_same(&format!("{ctx}: voiceArray[{v}]"), &l.voiceArray[v as usize], &f);
    }
    assert_same(&format!("{ctx}: snareVoice"), &l.snareVoice, &get(fw, memmap::ADDR_snareVoice));
    assert_same(&format!("{ctx}: hatVoice"), &l.hatVoice, &get(fw, memmap::ADDR_hatVoice));
    assert_same(&format!("{ctx}: cymbalVoice"), &l.cymbalVoice, &get(fw, memmap::ADDR_cymbalVoice));
    assert_same(&format!("{ctx}: velocityModulators"), &l.velocityModulators, &get(fw, memmap::ADDR_velocityModulators));
    assert_same(&format!("{ctx}: mixer_decimation_rate"), &l.mixer_decimation_rate, &get(fw, memmap::ADDR_mixer_decimation_rate));
    assert_same(&format!("{ctx}: mixer_decimation_cnt"), &l.mixer_decimation_cnt, &get(fw, memmap::ADDR_mixer_decimation_cnt));
    assert_same(&format!("{ctx}: mixer_voice_samples"), &l.mixer_voice_samples, &get(fw, memmap::ADDR_mixer_voice_samples));
    assert_same(&format!("{ctx}: mixer_audioRouting"), &l.mixer_audioRouting, &get(fw, memmap::ADDR_mixer_audioRouting));
    let jacks: [u8; 4] = get(fw, memmap::ADDR_l1_Available);
    assert_eq!([l.l1_Available, l.l2_Available, l.r1_Available, l.r2_Available], jacks, "{ctx}: jack flags");
}

/// Firmware set up like the engine contract: FPSCR = 0 (thread mode, FZ=0), `GetRngValue` replaced by the
/// xorshift32 generator (state shared through the returned cell, keep it equal to `Lxr::rng_state`), and all
/// four jack-detect inputs plugged.
pub fn setup(mut fw: Firmware) -> (Firmware, Rc<Cell<u32>>) {
    use unicorn_engine::RegisterARM;
    fw.fpscr = 0;
    fw.insn_limit = 200_000_000;
    let st = Rc::new(Cell::new(lxr_core::rng::RNG_SEED));
    let s2 = st.clone();
    let a = fw.addr("GetRngValue").unwrap();
    fw.intercept(a, move |uc| {
        let mut x = s2.get();
        let v = lxr_core::rng::xorshift32(&mut x);
        s2.set(x);
        uc.reg_write(RegisterARM::R0, v as u64).unwrap();
    })
    .unwrap();
    fw.uc.get_data_mut().mmio_values.insert(0x4002_0410, (1 << 4) | (1 << 6));
    fw.uc.get_data_mut().mmio_values.insert(0x4002_0C10, (1 << 6) | (1 << 7));
    (fw, st)
}
