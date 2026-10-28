// SPDX-License-Identifier: GPL-3.0-only
//! Cross-checks of the parameter table against the firmware source.
//!
//! The firmware source is looked up in `$LXR_FW_SRC`, then in the `third_party/lxr-firmware`
//! submodule. Without it the tests print a note and pass.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use lxr_params::*;

fn fw_root() -> Option<PathBuf> {
    let candidates = [
        std::env::var_os("LXR_FW_SRC").map(PathBuf::from),
        Some(Path::new(env!("CARGO_MANIFEST_DIR")).join("../../third_party/lxr-firmware")),
    ];
    for c in candidates.into_iter().flatten() {
        if c.join("Core/Inc/Parameters.h").exists() {
            return Some(c);
        }
    }
    eprintln!("firmware source not found (set LXR_FW_SRC): skipping cross-check");
    None
}

fn read(root: &Path, p: &str) -> String {
    let b = std::fs::read(root.join("Core").join(p)).unwrap();
    b.iter().map(|&c| c as char).collect()
}

/// Remove `/* */` and `//` comments (string literals kept).
fn strip_comments(s: &str) -> String {
    let b: Vec<char> = s.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < b.len() {
        if b[i] == '"' {
            out.push(b[i]);
            i += 1;
            while i < b.len() && b[i] != '"' {
                if b[i] == '\\' {
                    out.push(b[i]);
                    i += 1;
                }
                out.push(b[i]);
                i += 1;
            }
            if i < b.len() {
                out.push('"');
                i += 1;
            }
        } else if b[i] == '/' && i + 1 < b.len() && b[i + 1] == '*' {
            i += 2;
            while i + 1 < b.len() && !(b[i] == '*' && b[i + 1] == '/') {
                i += 1;
            }
            i += 2;
            out.push(' ');
        } else if b[i] == '/' && i + 1 < b.len() && b[i + 1] == '/' {
            while i < b.len() && b[i] != '\n' {
                i += 1;
            }
        } else {
            out.push(b[i]);
            i += 1;
        }
    }
    out
}

/// Drop `#if USE_CODEC_EQ` blocks (the macro is not defined in the firmware) and other
/// preprocessor lines.
fn preprocess(src: &str) -> String {
    let mut out = String::new();
    let mut skipping = false;
    for line in src.lines() {
        let t = line.trim();
        if t.starts_with("#if USE_CODEC_EQ") {
            skipping = true;
            continue;
        }
        if skipping && t.starts_with("#else") {
            skipping = false;
            continue;
        }
        if t.starts_with("#endif") {
            skipping = false;
            continue;
        }
        if skipping || t.starts_with('#') {
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

/// Parse the body of `enum <name> { ... }` into (name, value) pairs.
fn parse_enum(src: &str, name: &str) -> Vec<(String, i64)> {
    let s = preprocess(&strip_comments(src));
    let start = s.find(&format!("enum {name}")).expect(name);
    let open = start + s[start..].find('{').unwrap();
    let close = open + s[open..].find('}').unwrap();
    let mut out: Vec<(String, i64)> = Vec::new();
    let mut cur = -1i64;
    for item in s[open + 1..close].split(',') {
        let item = item.trim();
        if item.is_empty() || item.starts_with('#') {
            continue;
        }
        if let Some((n, e)) = item.split_once('=') {
            let e = e.trim();
            cur = if let Some(h) = e.strip_prefix("0x") {
                i64::from_str_radix(h, 16).unwrap()
            } else if let Ok(v) = e.parse() {
                v
            } else {
                out.iter().find(|(k, _)| k == e).unwrap().1
            };
            out.push((n.trim().to_string(), cur));
        } else {
            cur += 1;
            out.push((item.to_string(), cur));
        }
    }
    out
}

/// The items of the C initializer `<decl> = { ... };`, split at top level commas.
fn initializer(src: &str, decl: &str) -> Vec<String> {
    let s = strip_comments(src);
    let start = s.find(decl).expect(decl);
    let open = start + s[start..].find('{').unwrap();
    let mut depth = 0;
    let mut items = Vec::new();
    let mut cur = String::new();
    for c in s[open..].chars() {
        match c {
            '{' => {
                depth += 1;
                if depth > 1 {
                    cur.push(c);
                }
            }
            '}' => {
                depth -= 1;
                if depth == 0 {
                    if !cur.trim().is_empty() {
                        items.push(cur.trim().to_string());
                    }
                    break;
                }
                cur.push(c);
            }
            ',' if depth == 1 => {
                items.push(cur.trim().to_string());
                cur.clear();
            }
            _ => cur.push(c),
        }
    }
    items.into_iter().filter(|i| !i.is_empty()).collect()
}

#[test]
fn param_enum_matches_parameters_h() {
    let Some(root) = fw_root() else { return };
    let e = parse_enum(&read(&root, "Inc/Parameters.h"), "ParamEnums");
    let map: BTreeMap<_, _> = e.iter().cloned().collect();
    assert_eq!(map["END_OF_SOUND_PARAMETERS"], END_OF_SOUND_PARAMETERS as i64);
    assert_eq!(map["NUM_PARAMS"], NUM_PARAMS as i64);
    assert_eq!(e.len(), PARAM_ENUM.len());
    for ((n, v), (gn, gv)) in e.iter().zip(PARAM_ENUM.iter()) {
        assert_eq!((n.as_str(), *v), (*gn, *gv as i64));
    }
    // every sound parameter's enum name has the value of its index
    for p in PARAMS.iter().chain(GLOBAL_PARAMS.iter()) {
        assert_eq!(map[p.enum_name], p.index as i64, "{}", p.enum_name);
    }
    // all indices below END_OF_SOUND_PARAMETERS are covered
    let n_sound = e.iter().filter(|(_, v)| *v < END_OF_SOUND_PARAMETERS as i64).map(|(_, v)| *v).max().unwrap() + 1;
    assert_eq!(n_sound as usize, PARAMS.len());
}

#[test]
fn dtypes_match_menu_c() {
    let Some(root) = fw_root() else { return };
    let items = initializer(&read(&root, "Src/menu/menu.c"), "parameter_dtypes[NUM_PARAMS]");
    assert_eq!(items.len(), NUM_PARAMS as usize);
    for p in PARAMS.iter().chain(GLOBAL_PARAMS.iter()) {
        let it = &items[p.index as usize];
        let base = it.split('|').next().unwrap().trim();
        let expect = match p.dtype {
            Dtype::U0to255 => "DTYPE_0B255",
            Dtype::U0to127 => "DTYPE_0B127",
            Dtype::Pm100 => "DTYPE_PM100",
            Dtype::Menu => "DTYPE_MENU",
            Dtype::Pm63 => "DTYPE_PM63",
            Dtype::U1to16 => "DTYPE_1B16",
            Dtype::OnOff => "DTYPE_ON_OFF",
            Dtype::TargetSelectionLfo => "DTYPE_TARGET_SELECTION_LFO",
            Dtype::TargetSelectionVelo => "DTYPE_TARGET_SELECTION_VELO",
            Dtype::VoiceLfo => "DTYPE_VOICE_LFO",
            Dtype::AutomTarget => "DTYPE_AUTOM_TARGET",
            Dtype::U0to1 => "DTYPE_0b1",
            Dtype::NoteName => "DTYPE_NOTE_NAME",
            Dtype::U0to15 => "DTYPE_0B15",
            Dtype::U1to64 => "DTYPE_1B64",
            Dtype::U0to100 => "DTYPE_0B100",
        };
        assert_eq!(base, expect, "{}", p.enum_name);
        if let Some(m) = p.menu {
            let id = it.split("MENU_").nth(1).unwrap().split('<').next().unwrap().trim();
            let ids = read(&root, "Inc/menu/MenuText.h");
            let line = ids.lines().find(|l| l.contains(&format!("#define MENU_{id}"))).unwrap();
            let num: u32 = line.split_whitespace().last().unwrap().parse().unwrap();
            assert_eq!(num, m as u32, "{}", p.enum_name);
        }
    }
}

#[test]
fn mod_targets_match_cc2text() {
    let Some(root) = fw_root() else { return };
    let src = read(&root, "Src/menu/Cc2Text.c");
    let items = initializer(&src, "modTargets[]");
    assert_eq!(items.len(), MOD_TARGETS.len());
    let pe: BTreeMap<_, _> = parse_enum(&read(&root, "Inc/Parameters.h"), "ParamEnums").into_iter().collect();
    let ne: BTreeMap<_, _> = parse_enum(&read(&root, "Inc/menu/menu.h"), "NamesEnum").into_iter().collect();
    for (it, (t, p)) in items.iter().zip(MOD_TARGETS.iter()) {
        let inner = it.trim_matches(|c| c == '{' || c == '}');
        let (a, b) = inner.split_once(',').unwrap();
        assert_eq!(ne[a.trim()], *t as i64);
        assert_eq!(pe[b.trim()], *p as i64);
    }
    let offs = initializer(&src, "modTargetVoiceOffsets[6]");
    assert_eq!(offs.len(), 6);
}

#[test]
fn init_kit_matches() {
    let Some(root) = fw_root() else { return };
    let items = initializer(&read(&root, "Src/Preset/initKit.c"), "initKit[INIT_KIT_SIZE]");
    let bytes: Vec<u8> = items.iter().map(|s| u8::from_str_radix(s.trim_start_matches("0x"), 16).unwrap()).collect();
    assert_eq!(bytes, INIT_KIT.to_vec());
}

#[test]
fn every_sound_param_is_on_a_menu_page() {
    let Some(root) = fw_root() else { return };
    let pages = strip_comments(&read(&root, "Inc/menu/menuPages.h"));
    let idents: std::collections::BTreeSet<&str> =
        pages.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_')).collect();
    for p in PARAMS.iter() {
        let on_page = idents.contains(p.enum_name);
        let in_mod_targets = MOD_TARGETS.iter().any(|t| t.1 == p.index);
        match p.kind {
            ParamKind::Sound => assert!(on_page || in_mod_targets, "{}", p.enum_name),
            ParamKind::Unused => assert!(!on_page || p.index == 0, "{}", p.enum_name),
            ParamKind::Global => unreachable!(),
        }
    }
}

#[test]
fn cc_numbers_match_midi_messages_h() {
    let Some(root) = fw_root() else { return };
    let src = read(&root, "Inc/Soundengine/MIDI/MidiMessages.h");
    let s = strip_comments(&src);
    // typedef enum { ... } ParamEnumsDefinition;
    let end = s.find("} ParamEnumsDefinition").unwrap();
    let start = s[..end].rfind("typedef enum").unwrap();
    let body = format!("enum X {}}}", &s[start + "typedef enum".len()..end]);
    let cc = parse_enum(&body, "X");
    let cc_vals: Vec<i64> = cc.iter().map(|(_, v)| *v).collect();
    for p in PARAMS.iter() {
        if let Some(c) = p.midi.cc {
            assert_eq!(c as u16, p.index + 1);
            assert!(cc_vals.contains(&(c as i64)), "{}", p.enum_name);
        }
        if let Some(n) = p.midi.nrpn {
            assert_eq!(n, p.index - 128);
        }
    }
}

/// Re-runs the generator on the firmware source and compares the output with the committed file.
#[test]
fn generated_file_is_up_to_date() {
    let Some(root) = fw_root() else { return };
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let out = std::env::temp_dir().join(format!("lxr-params-gen-{}.rs", std::process::id()));
    let status = std::process::Command::new("python3")
        .arg(dir.join("gen/gen_params.py"))
        .arg(&root)
        .arg(&out)
        .status();
    match status {
        Ok(s) if s.success() => {
            let a = std::fs::read_to_string(&out).unwrap();
            let b = std::fs::read_to_string(dir.join("src/generated.rs")).unwrap();
            let _ = std::fs::remove_file(&out);
            assert!(a == b, "src/generated.rs is out of date: run gen/gen_params.py");
        }
        _ => eprintln!("python3 not available: generator check skipped"),
    }
}
