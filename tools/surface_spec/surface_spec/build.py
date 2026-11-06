"""Load surface/surface.py, check it, and write the JSON the plugin loads."""

import importlib
import json
import os
import sys

from .behaviours import BEHAVIOURS, SETTINGS, STEP_FIELDS
from .dsl import CONTROLS, MODE_KEYS, PLAIN_SECTIONS
from .rules import check_rules

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "..", ".."))
SOURCE_DIR = os.path.join(ROOT, "surface")
OUTPUT = os.path.join(ROOT, "crates", "vdrum-surface", "spec", "surface.json")
PARAM_IDS = os.path.join(os.path.dirname(__file__), "param_ids.json")
SCENARIOS_OUTPUT = os.path.join(ROOT, "crates", "vdrum-sim", "scenarios.json")


def load(source_dir=SOURCE_DIR):
    """Import surface.py fresh (so a watcher picks up edits) and return its `surface`."""
    if source_dir not in sys.path:
        sys.path.insert(0, source_dir)
    for name in list(sys.modules):
        mod = sys.modules[name]
        path = getattr(mod, "__file__", None) or ""
        if path and os.path.abspath(path).startswith(source_dir + os.sep):
            del sys.modules[name]
    importlib.invalidate_caches()
    return importlib.import_module("surface").surface


def check(spec):
    """Errors in a built spec (a list of strings; empty when it is valid)."""
    errors = []
    with open(PARAM_IDS) as fh:
        param_ids = set(json.load(fh))
    slots = {}
    for m in spec["modes"]:
        if m["mode"] not in MODE_KEYS:
            errors.append(f"mode {m['mode']}: unknown mode key")
        if not 1 <= m["slot"] <= 16:
            errors.append(f"mode {m['mode']}: slot {m['slot']} outside 1..16")
        if m["slot"] in slots:
            errors.append(f"slot {m['slot']}: used by {slots[m['slot']]} and {m['mode']}")
        slots[m["slot"]] = m["mode"]
    seen = [m["mode"] for m in spec["modes"]]
    for key in MODE_KEYS:
        if seen.count(key) != 1:
            errors.append(f"mode {key}: defined {seen.count(key)} times (every mode the plugin knows needs one slot)")
    for name in ("steps", "step-held"):
        if name not in spec["overlays"]:
            errors.append(f"overlay {name} missing")

    def check_cells(where, cells):
        for c in cells:
            w = f"{where} {c['control']}"
            if c["control"] not in CONTROLS:
                errors.append(f"{w}: unknown control")
            sec = c["section"]
            base, _, tr = sec.partition(":")
            if not (sec in PLAIN_SECTIONS or (base in ("seq_track", "perf", "pad") and tr.isdigit() and int(tr) < 7)):
                errors.append(f"{w}: unknown section {sec}")
            d = c["do"]
            if d not in BEHAVIOURS:
                errors.append(f"{w}: unknown behaviour {d}")
                continue
            kind, arg = BEHAVIOURS[d], c.get("arg")
            ok = {
                None: arg is None,
                "param": arg in param_ids,
                "track": isinstance(arg, int) and 0 <= arg < 7,
                "step_field": arg in STEP_FIELDS,
                "setting": arg in SETTINGS,
            }.get(kind, isinstance(arg, int) and isinstance(kind, int) and 0 <= (arg if isinstance(arg, int) else -1) < kind)
            if not ok:
                errors.append(f"{w}: {d}: bad argument {arg!r} (expected {kind})")

    for lid, cells in spec["layouts"].items():
        check_cells(f"layout {lid}", cells)
    for oid, cells in spec["overlays"].items():
        check_cells(f"overlay {oid}", cells)
    errors += check_rules(spec)
    return errors


def dumps(spec):
    """JSON with one cell per line, so diffs of the built file stay readable."""
    def cells(lst, ind):
        return "[\n" + ",\n".join(ind + "  " + json.dumps(c, ensure_ascii=False) for c in lst) + "\n" + ind + "]"

    out = ['{', '  "format": "vdrum-surface-spec",', f'  "version": {spec["version"]},', '  "modes": [']
    out.append(",\n".join("    " + json.dumps(m, ensure_ascii=False) for m in spec["modes"]))
    out.append("  ],")
    out.append('  "layouts": {')
    out.append(",\n".join(f"    {json.dumps(k)}: {cells(v, '    ')}" for k, v in spec["layouts"].items()))
    out.append("  },")
    out.append('  "overlays": {')
    out.append(",\n".join(f"    {json.dumps(k)}: {cells(v, '    ')}" for k, v in spec["overlays"].items()))
    out.append("  },")
    out.append('  "navigation": ' + cells(spec["navigation"], "  ") + ",")
    out.append('  "leds": ' + cells(spec["leds"], "  ") + ",")
    out.append('  "slot_leds": ' + json.dumps(spec["slot_leds"]) + ",")
    out.append('  "screen": ' + cells(spec["screen"], "  ") + ",")
    pal = spec["palette"]
    out.append('  "palette": {')
    out.append('    "sections": ' + cells(pal["sections"], "    ") + ",")
    out.append('    "tracks": ' + cells(pal["tracks"], "    ") + ",")
    out.append('    "named": ' + cells(pal["named"], "    "))
    out.append("  }")
    out.append("}")
    return "\n".join(out) + "\n"


def load_scenarios(source_dir=SOURCE_DIR):
    """Every scenario in surface/scenarios/*.py (each module's `scenarios()`), in file order."""
    load(source_dir)  # puts surface/ on the path and drops stale modules
    d = os.path.join(source_dir, "scenarios")
    out = []
    for f in sorted(os.listdir(d)) if os.path.isdir(d) else []:
        if f.endswith(".py") and not f.startswith("_"):
            mod = importlib.import_module("scenarios." + f[:-3])
            out += [s.to_json() for s in mod.scenarios()]
    return out


def dumps_scenarios(scs):
    """One step per line."""
    out = ['{', '  "scenarios": [']
    items = []
    for sc in scs:
        steps = ",\n".join("        " + json.dumps(st, ensure_ascii=False) for st in sc["steps"])
        items.append('    {"name": ' + json.dumps(sc["name"], ensure_ascii=False) + ', "steps": [\n' + steps + "\n    ]}")
    out.append(",\n".join(items))
    out.append("  ]")
    out.append("}")
    return "\n".join(out) + "\n"


def build(source_dir=SOURCE_DIR):
    from .scenario import check_scenarios
    spec = load(source_dir).to_json()
    scs = load_scenarios(source_dir)
    return spec, scs, check(spec) + check_scenarios(scs)
