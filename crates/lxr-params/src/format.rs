// SPDX-License-Identifier: GPL-3.0-only
//! Value display of the firmware menu (`menu_repaintGeneric()` in menu.c and the helpers
//! `numtostrpu()`, `numtostrps()`, `numtostru()`, `setNoteName()`, `getMenuItemNameForValue()`,
//! `menu_displayModTargetFull()`, `menu_displayModTargetShort()`), ported to strings.
//!
//! The LCD shows fixed width fields; the functions here return the same characters with leading
//! and trailing blanks removed and stop at a NUL byte (the firmware copies 3 bytes of texts such as
//! `"x2"` including their terminating NUL to the LCD buffer).
//!
//! Values outside the range the firmware can display (it would read past the end of its text
//! arrays) are shown as `"---"` (the firmware's `menuText_dash`).

use crate::{
    mod_target_voice, par, text_names, Dtype, MenuList, ParamInfo, CAT_NAMES, LONG_NAMES, MOD_TARGETS,
    SHORT_NAMES, VALUE_NAMES,
};

/// `numtostrpu(buf, num, ' ')`: 3 characters, space padded unsigned.
pub fn numtostrpu(num: u8, pad: u8) -> [u8; 3] {
    let mut buf = [0u8; 3];
    let mut num = num;
    if num > 99 {
        buf[0] = b'0' + num / 100;
        num %= 100;
    } else {
        buf[0] = pad;
    }
    if num > 9 {
        buf[1] = b'0' + num / 10;
        num %= 10;
    } else if buf[0] == pad {
        buf[1] = pad;
    } else {
        buf[1] = b'0';
    }
    buf[2] = b'0' + num;
    buf
}

/// `numtostrps(buf, num)`: 3 characters, space padded signed.
pub fn numtostrps(num: i8) -> [u8; 3] {
    let mut buf = [0u8; 3];
    let mut num = num as i16;
    if num > 99 {
        buf[0] = b'0' + (num / 100) as u8;
        num %= 100;
    } else if num < 0 {
        buf[0] = b'-';
        num = -num;
    } else {
        buf[0] = b' ';
    }
    if num > 9 {
        buf[1] = b'0' + (num / 10) as u8;
        num %= 10;
    } else if buf[0] < b'0' {
        buf[1] = b' ';
    } else {
        buf[1] = b'0';
    }
    buf[2] = b'0' + num as u8;
    buf
}

/// `numtostru(buf, num)`: unsigned without padding.
pub fn numtostru(num: u8) -> String {
    num.to_string()
}

/// `setNoteName(num, buf)`: e.g. `"C#4"`, `"A 3"` (octave = num / 12, as an ASCII digit offset; the
/// firmware prints `':'` for octave 10).
pub fn set_note_name(num: u8) -> [u8; 3] {
    let n = num % 12;
    let c0 = (if n > 8 { 60 } else { 67 }) + ((n + (n > 4) as u8) / 2);
    let sharp = (n < 5 && (n & 1) != 0) || (n > 5 && (n & 1) == 0);
    [c0, if sharp { b'#' } else { b' ' }, 48 + num / 12]
}

fn lcd(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|b| *b == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).trim().to_string()
}

fn three(s: &str) -> String {
    let b = s.as_bytes();
    lcd(&b[..b.len().min(3)])
}

/// `getMenuItemNameForValue(menuId, value, buf)`.
pub fn menu_item_name(menu: MenuList, value: u8) -> String {
    let texts = menu.texts();
    if menu == MenuList::Waveform && value as usize >= texts.len() {
        // user sample: 's' + number (non padded) at buf[1], buf[2] = ' ' first
        let mut b = vec![b's', b' ', b' '];
        let n = numtostru(value - texts.len() as u8);
        for (i, c) in n.bytes().enumerate() {
            if 1 + i < b.len() {
                b[1 + i] = c;
            } else {
                b.push(c);
            }
        }
        // numtostru writes its digits and a terminating 0 from buf[1]; the LCD shows 3 chars
        return lcd(&b[..3.min(b.len())]);
    }
    match texts.get(value as usize) {
        Some(t) => three(t),
        None => "---".to_string(),
    }
}

/// Short (3 character) name of a modulation destination (`menu_displayModTargetShort()`).
pub fn mod_target_short(idx: u8, include_voice: bool) -> String {
    let Some(&(name, param)) = MOD_TARGETS.get(idx as usize) else {
        return "---".to_string();
    };
    if param == par::NONE {
        return "off".to_string();
    }
    let short = SHORT_NAMES[VALUE_NAMES[name as usize].0 as usize].as_bytes();
    let mut out = Vec::new();
    let mut n = 3;
    if include_voice {
        out.push(b'0' + mod_target_voice(idx));
        n -= 1;
    }
    out.extend_from_slice(&short[..short.len().min(n)]);
    lcd(&out)
}

/// Full name of a modulation destination (`menu_displayModTargetFull()`): category (8 columns)
/// followed by the long name.
pub fn mod_target_full(idx: u8) -> String {
    let Some(&(name, param)) = MOD_TARGETS.get(idx as usize) else {
        return "---".to_string();
    };
    if param == par::NONE {
        return "off".to_string();
    }
    let (_, c, l) = VALUE_NAMES[name as usize];
    let cat = CAT_NAMES[c as usize];
    let long = LONG_NAMES[l as usize];
    format!("{} {}", cat.trim(), long.trim()).trim().to_string()
}

fn coarse_like(p: u16) -> bool {
    [
        par::COARSE1,
        par::COARSE2,
        par::COARSE3,
        par::COARSE4,
        par::COARSE5,
        par::COARSE6,
        par::FM_FREQ1,
        par::FM_FREQ2,
        par::FM_FREQ3,
        par::MOD_OSC_F1_CYM,
        par::MOD_OSC_F2_CYM,
        par::MOD_OSC_F1,
        par::MOD_OSC_F2,
    ]
    .contains(&p)
}

/// Offset subtracted from the value for display by `DTYPE_PM63` parameters (60 for the coarse
/// tune / FM frequency parameters, 63 otherwise).
pub fn pm63_offset(p: u16) -> i16 {
    if coarse_like(p) {
        60
    } else {
        63
    }
}

fn common(info: &ParamInfo, v: u8) -> Option<String> {
    let p = info.index;
    Some(match info.dtype {
        Dtype::OnOff => if v == 1 { "on" } else { "off" }.to_string(),
        Dtype::Menu => menu_item_name(info.menu?, v),
        Dtype::Pm63 => lcd(&numtostrps((v as i16 - pm63_offset(p)) as i8)),
        Dtype::NoteName => {
            if (par::MIDI_NOTE1..=par::MIDI_NOTE7).contains(&p) && v == 0 {
                "any".to_string()
            } else {
                lcd(&set_note_name(v))
            }
        }
        Dtype::U0to1 => lcd(&numtostrpu(v.wrapping_add(1), b' ')),
        Dtype::U0to127
        | Dtype::U0to100
        | Dtype::U0to255
        | Dtype::U1to16
        | Dtype::U0to15
        | Dtype::U1to64
        | Dtype::VoiceLfo
        | Dtype::Pm100 => {
            if p == par::BPM && v == 0 {
                "EXT".to_string()
            } else if p == par::BPM && v == 1 {
                "MID".to_string()
            } else {
                lcd(&numtostrpu(v, b' '))
            }
        }
        _ => return None,
    })
}

/// The value text the menu shows in edit mode (one parameter with its full name).
pub fn format_value(info: &ParamInfo, v: u8) -> String {
    match info.dtype {
        Dtype::TargetSelectionLfo | Dtype::TargetSelectionVelo | Dtype::AutomTarget => mod_target_full(v),
        _ => common(info, v).unwrap_or_else(|| "---".to_string()),
    }
}

/// The 3 character value text of the parameter overview (four parameters per LCD line).
pub fn format_value_short(info: &ParamInfo, v: u8) -> String {
    match info.dtype {
        Dtype::TargetSelectionLfo | Dtype::TargetSelectionVelo => mod_target_short(v, false),
        Dtype::AutomTarget => mod_target_short(v, true),
        _ => common(info, v).unwrap_or_else(|| "---".to_string()),
    }
}

fn norm(s: &str) -> String {
    s.chars().filter(|c| !c.is_whitespace()).flat_map(|c| c.to_lowercase()).collect()
}

/// Inverse of [`format_value`] / [`format_value_short`]: finds the value in the parameter's range
/// whose text matches `text` (case and blanks ignored). Numbers are also accepted for numeric
/// types, as displayed (`-3` for a `DTYPE_PM63` parameter means stored value 60 or 57).
/// For modulation destinations the [`MOD_TARGETS`] index (`#12`) is accepted as well.
pub fn parse_value(info: &ParamInfo, text: &str) -> Option<u8> {
    let t = norm(text);
    if t.is_empty() {
        return None;
    }
    let range = info.min..=info.max;
    for v in range.clone() {
        let v = v as u8;
        if norm(&format_value(info, v)) == t || norm(&format_value_short(info, v)) == t {
            return Some(v);
        }
    }
    if info.is_mod_target() {
        if let Some(n) = t.strip_prefix('#').and_then(|n| n.parse::<u16>().ok()) {
            return range.contains(&n).then_some(n as u8);
        }
        // readable form used by VDrum: "<voice> <parameter name>"
        for v in range.clone() {
            if let Some(name) = mod_target_readable(v as u8) {
                if norm(&name) == t {
                    return Some(v as u8);
                }
            }
        }
        return None;
    }
    let n: i32 = t.trim_start_matches('+').parse().ok()?;
    let raw = match info.dtype {
        Dtype::Pm63 => n + pm63_offset(info.index) as i32,
        Dtype::U0to1 => n - 1,
        Dtype::OnOff => n,
        Dtype::Menu => return None,
        Dtype::NoteName => n,
        _ => n,
    };
    (raw >= info.min as i32 && raw <= info.max as i32).then_some(raw as u8)
}

/// Readable name of a modulation destination used by VDrum: the readable name of the parameter it
/// stands for (`"Drum 1 Oscillator Coarse"`), `"off"` for index 0. Not a firmware text.
pub fn mod_target_readable(idx: u8) -> Option<String> {
    let &(_, param) = MOD_TARGETS.get(idx as usize)?;
    if param == par::NONE {
        return Some("off".to_string());
    }
    Some(crate::info(param)?.name.to_string())
}

/// Readable text of a TEXT_* index ("category long name"), for labels.
pub fn text_label(text: u8) -> String {
    let (_, c, l) = text_names(text);
    format!("{c} {l}").trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{by_enum_name, info};

    fn f(name: &str, v: u8) -> String {
        format_value(by_enum_name(name).unwrap(), v)
    }

    #[test]
    fn numbers() {
        assert_eq!(&numtostrpu(5, b' '), b"  5");
        assert_eq!(&numtostrpu(105, b' '), b"105");
        assert_eq!(&numtostrpu(100, b' '), b"100");
        assert_eq!(&numtostrps(-5), b"- 5");
        assert_eq!(&numtostrps(-63), b"-63");
        assert_eq!(&numtostrps(7), b"  7");
        assert_eq!(&numtostrps(67), b" 67");
        assert_eq!(f("PAR_COARSE1", 60), "0");
        assert_eq!(f("PAR_COARSE1", 0), "-60");
        assert_eq!(f("PAR_FINE1", 0), "-63");
        assert_eq!(f("PAR_FINE1", 58), "- 5");
        assert_eq!(f("PAR_PAN1", 127), "64");
        assert_eq!(f("PAR_VOL1", 127), "127");
    }

    #[test]
    fn menus() {
        assert_eq!(f("PAR_OSC_WAVE_DRUM1", 0), "Sin");
        assert_eq!(f("PAR_OSC_WAVE_DRUM1", 5), "PWM");
        assert_eq!(f("PAR_OSC_WAVE_DRUM1", 6), "s0");
        assert_eq!(f("PAR_FILTER_TYPE_1", 7), "off");
        assert_eq!(f("PAR_FILTER_TYPE_1", 3), "UBP");
        assert_eq!(f("PAR_FX_TYPE", 4), "del");
        assert_eq!(f("PAR_VOLUME_MOD_ON_OFF1", 1), "on");
        assert_eq!(f("PAR_SYNC_LFO1", 0), "off");
        assert_eq!(f("PAR_TRANS1_WAVE", 13), "Sp2");
        assert_eq!(f("PAR_AUDIO_OUT1", 6), "FX");
        assert_eq!(f("PAR_FX_ROUTING", 6), "---");
    }

    #[test]
    fn mod_targets() {
        assert_eq!(f("PAR_VEL_DEST_1", 0), "off");
        assert_eq!(f("PAR_VEL_DEST_1", 2), "Oscilltr Coarse");
        assert_eq!(format_value_short(by_enum_name("PAR_VEL_DEST_1").unwrap(), 2), "coa");
        assert_eq!(mod_target_short(2, true), "1co");
        assert_eq!(mod_target_readable(2).unwrap(), "Drum 1 Oscillator Coarse");
    }

    #[test]
    fn notes_and_globals() {
        assert_eq!(&set_note_name(48), b"C 4");
        assert_eq!(&set_note_name(61), b"C#5");
        assert_eq!(&set_note_name(69), b"A 5");
        assert_eq!(&set_note_name(70), b"A#5");
        assert_eq!(&set_note_name(71), b"B 5");
        assert_eq!(&set_note_name(64), b"E 5");
        assert_eq!(&set_note_name(65), b"F 5");
        assert_eq!(f("PAR_MIDI_NOTE1", 0), "any");
        assert_eq!(f("PAR_MIDI_NOTE1", 48), "C 4");
        assert_eq!(f("PAR_BPM", 0), "EXT");
        assert_eq!(f("PAR_BPM", 1), "MID");
        assert_eq!(f("PAR_BPM", 120), "120");
    }

    #[test]
    fn roundtrip_all() {
        for p in crate::PARAMS.iter().chain(crate::GLOBAL_PARAMS.iter()) {
            for v in p.min..=p.max {
                let v = v as u8;
                let s = format_value(p, v);
                let back = parse_value(p, &s).unwrap_or_else(|| panic!("{} {v} {s:?}", p.enum_name));
                // several values can share a text (e.g. "off" and mod targets of the FX
                // parameters that appear for every voice): the text must at least be the same
                assert_eq!(format_value(p, back), s, "{} {v}", p.enum_name);
                let ss = format_value_short(p, v);
                let back = parse_value(p, &ss).unwrap();
                assert_eq!(format_value_short(p, back), ss, "{} {v}", p.enum_name);
            }
        }
        let _ = info(0);
    }

    #[test]
    fn numeric_parse() {
        let p = by_enum_name("PAR_COARSE1").unwrap();
        assert_eq!(parse_value(p, "-3"), Some(57));
        assert_eq!(parse_value(p, "+3"), Some(63));
        let p = by_enum_name("PAR_VEL_DEST_2").unwrap();
        assert_eq!(parse_value(p, "#40"), Some(40));
        assert_eq!(parse_value(p, "Drum 2 Oscillator Coarse"), Some(40));
    }
}
