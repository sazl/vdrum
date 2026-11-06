"""Docs generated from the spec: one device image per mode (labels in their section colours), the
control tables, and the navigation (state diagram, rules, LEDs, screen). Written by `run.py build`
into docs/lcxl3-daw-mode/generated/; `run.py check` fails when they are out of date."""

import os
import re

from . import device_svg
from .rules import mermaid

DOCS_DIR = os.path.join(os.path.dirname(__file__), "..", "..", "..", "docs", "lcxl3-daw-mode", "generated")
DOCS_DIR = os.path.abspath(DOCS_DIR)

ROWS = [("Knob row 1", "K1{}"), ("Knob row 2", "K2{}"), ("Knob row 3", "K3{}"), ("Fader", "F{}"),
        ("Top button", "T{}"), ("Bottom button", "B{}")]


def slug(name):
    return re.sub(r"[^a-z0-9]+", "-", name.lower()).strip("-")


def colour_of(spec, section):
    pal = spec["palette"]
    base, _, t = section.partition(":")
    if t:
        return tuple(pal["tracks"][int(t)])
    return tuple(dict((k, v) for k, v in pal["sections"])[section])


def layout_cells(spec, layout_id, overlay=None):
    cells = {}
    for c in spec["layouts"][layout_id]:
        cells[c["control"]] = c
    for c in overlay or []:
        cells[c["control"]] = c
    return cells


def does(c):
    d, a = c["do"], c.get("arg")
    if d == "param":
        return f"`{a}`"
    if d == "spare":
        return "(unassigned)"
    return f"{d}" + (f" {a}" if a is not None else "")


def mode_image(spec, cells):
    leds, labels = {}, {}
    for key, c in cells.items():
        if c["do"] == "spare":
            continue
        rgb = colour_of(spec, c["section"])
        labels[key] = (c["label"], rgb)
        if not key.startswith("F"):
            leds[key] = rgb
    return device_svg.draw({"leds": leds, "labels": labels})


def table(cells):
    out = ["| Row | " + " | ".join(str(i) for i in range(1, 9)) + " |", "| --- |" + " --- |" * 8]
    for name, pat in ROWS:
        row = []
        for col in range(1, 9):
            c = cells.get(pat.format(col))
            row.append("" if c is None else (f"{c['label']} {does(c)}" if c["do"] != "spare" else "(unassigned)"))
        out.append(f"| {name} | " + " | ".join(x.replace("|", "\\|") for x in row) + " |")
    return "\n".join(out)


def outputs(spec):
    """{path: text} of every generated doc."""
    files = {}
    pages = []
    modes = sorted(spec["modes"], key=lambda m: m["slot"])
    for m in modes:
        variants = [(m["name"], layout_cells(spec, m["layout"]))]
        if m["mode"] == "sequencer":
            variants.append((m["name"] + ", step held", layout_cells(spec, m["layout"], spec["overlays"]["step-held"])))
        for title, cells in variants:
            img = f"mode-{m['slot']:02d}-{slug(title)}.svg"
            files[os.path.join(DOCS_DIR, "images", img)] = mode_image(spec, cells)
            pages.append(f"## Mode {m['slot']}: {title}\n\n![{title}](images/{img})\n\n{table(cells)}\n")
    steps = {c["control"]: c for c in spec["overlays"]["steps"]}
    pages.append("## Sequencer view (Solo/Arm), laid over any mode\n\n" + table(steps) + "\n")
    head = ("# Control tables (generated)\n\nGenerated from the surface spec (`surface/`) by "
            "`python3 tools/surface_spec/run.py build`; do not edit. Each cell is the label shown on the "
            "screen and what the control does: a parameter id in backticks, or a behaviour "
            "(`docs/lcxl3-daw-mode/surface-spec.md`, Behaviours). LED colours in the images are the "
            "section colours; the device dims or lights them by state.\n\n")
    files[os.path.join(DOCS_DIR, "control-tables.md")] = head + "\n".join(pages)

    nav = ["# Navigation (generated)\n",
           "Generated from `surface/rules/` by `python3 tools/surface_spec/run.py build`; do not edit.\n",
           "## State diagram\n", "```mermaid", mermaid(spec).rstrip(), "```\n",
           "## Rules\n", "The first rule whose button and guards match, and which has an entry for the event, "
           "handles it. \"next rule\": the event falls through.\n",
           "| # | Button | When | Press | Release | Note |", "| --- | --- | --- | --- | --- | --- |"]

    def steps_text(lst):
        if lst is None:
            return "next rule"
        if not lst:
            return "nothing"
        return ", ".join(s["do"] + (f"({s['arg']})" if "arg" in s else "") for s in lst).replace("|", "\\|")

    for i, r in enumerate(spec["navigation"]):
        nav.append(f"| {i + 1} | {r['button']} | {' '.join(r.get('when', []))} | {steps_text(r.get('press'))} | "
                   f"{steps_text(r.get('release'))} | {r.get('note', '')} |")
    nav += ["", "## Side-button LEDs\n", "First matching rule; no match is off. "
            f"The 16 buttons show the mode slots while: {' '.join(spec['slot_leds']['when']) or 'never'}.\n",
            "| Button | Rules |", "| --- | --- |"]
    for b in spec["leds"]:
        rules = "; ".join((" ".join(r["when"]) or "otherwise") + " → " + r["colour"] for r in b["rules"])
        nav.append(f"| {b['button']} | {rules} |")
    nav += ["", "## Stationary screen\n", "| When | Title | Name | Value |", "| --- | --- | --- | --- |"]
    for r in spec["screen"]:
        nav.append(f"| {' '.join(r['when']) or 'otherwise'} | " + " | ".join(f"`{x}`" for x in r["lines"]) + " |")
    files[os.path.join(DOCS_DIR, "navigation.md")] = "\n".join(nav) + "\n"
    return files


