//! ARM emulation harness for the LXR V2 firmware (reference for the Rust DSP port).
pub mod engine;
pub mod firmware;
pub mod scenario;
pub mod symbols;
pub use engine::{AudioContext, Block, Build, EmuEngine};
pub use firmware::Firmware;
