//! Shared helpers for differential tests (lxr-core vs firmware in lxr-emu).
//! Each porting group adds its own test file under `tests/` (e.g. `tests/p1_oscillator.rs`).
pub use lxr_emu::Firmware;

/// Load the GCC 11.3 reference ELF from `LXR_FW_ELF`; `None` (test should return early) if unset.
pub fn reference_elf() -> Option<Firmware> {
    let path = std::env::var("LXR_FW_ELF").ok()?;
    let data = std::fs::read(&path).unwrap_or_else(|e| panic!("read {path}: {e}"));
    Some(Firmware::from_elf(&data).expect("load reference ELF"))
}

/// Macro: get the reference firmware or skip the test with a message.
#[macro_export]
macro_rules! fw_or_skip {
    () => {
        match $crate::reference_elf() {
            Some(fw) => fw,
            None => {
                eprintln!("LXR_FW_ELF not set; skipping differential test");
                return;
            }
        }
    };
}

/// Simple deterministic PRNG for input sweeps.
pub struct Sweep(pub u64);
impl Sweep {
    pub fn next_u32(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        (self.0 >> 16) as u32
    }
    pub fn u8(&mut self) -> u8 { self.next_u32() as u8 }
    pub fn f32_in(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (hi - lo) * (self.next_u32() as f32 / u32::MAX as f32)
    }
}
