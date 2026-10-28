//! Port of `Core/Src/Soundengine/DSPAudio/` (voices, oscillators, filters, envelopes, mixer).

pub mod automation_node;
pub mod buffer_tools;
pub mod cymbal_voice;
pub mod decay;
pub mod distortion;
pub mod dither;
pub mod drum_voice;
pub mod fx;
pub mod hi_hat;
pub mod lfo;
pub mod median_filter;
pub mod mixer;
pub mod modulation_node;
pub mod one_pole_lp;
pub mod oscillator;
pub mod random;
pub mod resonant_filter;
pub mod samples;
pub mod slope_eg2;
pub mod snap_eg;
pub mod snare;
pub mod square_root_lut;
pub mod tilt_eq;
pub mod transient_generator;
pub mod transient_tables;
pub mod wavetable;
