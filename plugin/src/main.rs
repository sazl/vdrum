// SPDX-License-Identifier: GPL-3.0-only
//! Standalone VDrum (JACK/ALSA/CoreAudio/WASAPI via nih-plug's standalone wrapper).
//! Run with `--help` for the audio/MIDI backend options.
//!
//! nih-plug's CoreAudio/WASAPI/ALSA backend (cpal) opens the default output device only at the
//! exact `--sample-rate` (default 48000) and `--period-size` (default 512) it is given, and falls
//! back to the silent dummy backend when the device runs at another rate (for example 44100 Hz on
//! a Mac). Unless they are passed on the command line, both are taken from the default output
//! device's own configuration here.

use nih_plug::prelude::*;
use vdrum::VDrum;

/// nih-plug's default period size; used when the device accepts it.
const PERIOD_SIZE: u32 = 512;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let args = with_device_defaults(args, default_output_config);
    nih_export_standalone_with_args::<VDrum, _>(args);
}

/// The default output device's sample rate and the period size range it accepts.
#[derive(Debug, Clone, Copy, PartialEq)]
struct DeviceConfig {
    sample_rate: u32,
    period_range: Option<(u32, u32)>,
}

fn default_output_config() -> Option<DeviceConfig> {
    use cpal::traits::{DeviceTrait, HostTrait};
    let device = cpal::default_host().default_output_device()?;
    let config = device.default_output_config().ok()?;
    let period_range = match *config.buffer_size() {
        cpal::SupportedBufferSize::Range { min, max } => Some((min, max)),
        cpal::SupportedBufferSize::Unknown => None,
    };
    Some(DeviceConfig {
        sample_rate: config.sample_rate().0,
        period_range,
    })
}

/// Whether `args` already sets the option with the short form `short` and the long form `long`.
fn has_option(args: &[String], short: &str, long: &str) -> bool {
    args.iter().skip(1).any(|a| {
        a == long
            || a.starts_with(&format!("{long}="))
            || a == short
            || (a.starts_with(short) && !a.starts_with("--"))
    })
}

/// Appends `--sample-rate` and `--period-size` from the default output device when they are not
/// given and the backend is not JACK (which uses the JACK server's settings).
fn with_device_defaults(
    mut args: Vec<String>,
    device: impl FnOnce() -> Option<DeviceConfig>,
) -> Vec<String> {
    let backend_jack = args
        .windows(2)
        .any(|w| (w[0] == "-b" || w[0] == "--backend") && w[1] == "jack")
        || args.iter().any(|a| a == "--backend=jack" || a == "-bjack");
    let want_rate = !has_option(&args, "-r", "--sample-rate");
    let want_period = !has_option(&args, "-p", "--period-size");
    let help = args
        .iter()
        .skip(1)
        .any(|a| a == "-h" || a == "--help" || a == "-V" || a == "--version");
    if backend_jack || help || !(want_rate || want_period) {
        return args;
    }
    let Some(dev) = device() else {
        return args;
    };
    if want_rate && dev.sample_rate > 0 {
        args.push("--sample-rate".into());
        args.push(dev.sample_rate.to_string());
    }
    if want_period {
        if let Some((min, max)) = dev.period_range {
            if min <= max && !(min..=max).contains(&PERIOD_SIZE) {
                args.push("--period-size".into());
                args.push(PERIOD_SIZE.clamp(min, max).to_string());
            }
        }
    }
    args
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    const MAC: DeviceConfig = DeviceConfig {
        sample_rate: 44100,
        period_range: Some((15, 4096)),
    };

    #[test]
    fn adds_the_device_rate() {
        assert_eq!(
            with_device_defaults(s(&["vdrum"]), || Some(MAC)),
            s(&["vdrum", "--sample-rate", "44100"])
        );
    }

    #[test]
    fn keeps_a_given_rate() {
        for a in [
            &["vdrum", "-r", "96000"][..],
            &["vdrum", "--sample-rate", "96000"],
            &["vdrum", "--sample-rate=96000"],
            &["vdrum", "-r96000"],
        ] {
            assert_eq!(with_device_defaults(s(a), || Some(MAC)), s(a));
        }
    }

    #[test]
    fn clamps_the_period_into_the_device_range() {
        let dev = DeviceConfig {
            sample_rate: 48000,
            period_range: Some((1024, 4096)),
        };
        assert_eq!(
            with_device_defaults(s(&["vdrum"]), || Some(dev)),
            s(&["vdrum", "--sample-rate", "48000", "--period-size", "1024"])
        );
    }

    #[test]
    fn leaves_jack_help_and_no_device_alone() {
        for a in [
            &["vdrum", "--backend", "jack"][..],
            &["vdrum", "-b", "jack"],
            &["vdrum", "--help"],
        ] {
            assert_eq!(with_device_defaults(s(a), || Some(MAC)), s(a));
        }
        assert_eq!(with_device_defaults(s(&["vdrum"]), || None), s(&["vdrum"]));
    }
}
