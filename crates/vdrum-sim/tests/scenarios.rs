// SPDX-License-Identifier: GPL-3.0-only
//! Runs every scenario in `scenarios.json` (built from `surface/scenarios/*.py` by
//! `python3 tools/surface_spec/run.py build`) on a fresh simulator with the built-in spec.

use serde_json::Value;
use vdrum_sim::{check, Sim};

#[test]
fn scenarios_pass() {
    let file: Value = serde_json::from_str(include_str!("../scenarios.json")).expect("scenarios.json");
    let list = file["scenarios"].as_array().expect("scenarios");
    assert!(!list.is_empty());
    let mut failures = Vec::new();
    for sc in list {
        let name = sc["name"].as_str().unwrap_or("?");
        let mut sim = Sim::default();
        for (i, step) in sc["steps"].as_array().unwrap().iter().enumerate() {
            if let Some(checks) = step.get("expect").and_then(Value::as_object) {
                let state = sim.state();
                for (path, want) in checks {
                    if let Err(e) = check(&state, path, want) {
                        failures.push(format!("{name}, step {}: {e}", i + 1));
                    }
                }
            } else {
                let r = sim.command(step);
                if let Some(e) = r.get("error") {
                    failures.push(format!("{name}, step {}: {e}", i + 1));
                }
            }
        }
    }
    assert!(failures.is_empty(), "{} failed checks:\n{}", failures.len(), failures.join("\n"));
}

#[test]
fn check_reports_what_differs() {
    let state = serde_json::json!({"screen": {"lines": ["Mode", "Slot 13", "empty"]}, "leds": {"T1": [0, 0, 0]}});
    assert!(check(&state, "screen.lines.1", &"Slot 13".into()).is_ok());
    assert!(check(&state, "screen.lines~", &"Slot 13 | empty".into()).is_ok());
    assert!(check(&state, "leds.T1", &"off".into()).is_ok());
    let e = check(&state, "leds.T1", &"lit".into()).unwrap_err();
    assert_eq!(e, "leds.T1 is [0,0,0], expected \"lit\"");
    assert!(check(&state, "screen.nope", &Value::Null).unwrap_err().contains("no such value"));
}
