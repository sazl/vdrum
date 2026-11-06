#!/usr/bin/env python3
"""Surface spec tool.

    python3 tools/surface_spec/run.py build          build surface/ into crates/vdrum-surface/spec/surface.json
    python3 tools/surface_spec/run.py check          fail if the built file is missing changes or the spec has errors
    python3 tools/surface_spec/run.py watch          rebuild whenever a file in surface/ changes
    python3 tools/surface_spec/run.py svg OUT.svg [--demo]   draw the device
"""

import os
import sys
import time
import traceback

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from surface_spec import build as b  # noqa: E402


def cmd_build(write=True):
    try:
        spec, errors = b.build()
    except Exception:
        traceback.print_exc()
        return 1
    for e in errors:
        print("error:", e, file=sys.stderr)
    if errors:
        return 1
    text = b.dumps(spec)
    if write:
        with open(b.OUTPUT, "w") as fh:
            fh.write(text)
        print(f"wrote {os.path.relpath(b.OUTPUT)} ({len(spec['modes'])} modes)")
        return 0
    try:
        with open(b.OUTPUT) as fh:
            current = fh.read()
    except FileNotFoundError:
        current = ""
    if current != text:
        print(f"{os.path.relpath(b.OUTPUT)} is out of date: run `python3 tools/surface_spec/run.py build`", file=sys.stderr)
        return 1
    print("spec ok, built file up to date")
    return 0


def stamp():
    s = []
    for d, _, files in os.walk(b.SOURCE_DIR):
        for f in files:
            if f.endswith(".py"):
                p = os.path.join(d, f)
                s.append((p, os.stat(p).st_mtime_ns))
    return sorted(s)


def cmd_watch():
    last = None
    print(f"watching {os.path.relpath(b.SOURCE_DIR)} (Ctrl+C to stop)")
    while True:
        now = stamp()
        if now != last:
            last = now
            print(time.strftime("%H:%M:%S"), end=" ")
            cmd_build()
        time.sleep(0.3)


def main(argv):
    if len(argv) < 2:
        print(__doc__)
        return 2
    c = argv[1]
    if c == "build":
        return cmd_build()
    if c == "check":
        return cmd_build(write=False)
    if c == "watch":
        return cmd_watch()
    if c == "svg":
        from surface_spec import device_svg
        st = device_svg.demo_state() if "--demo" in argv else {}
        with open(argv[2], "w") as fh:
            fh.write(device_svg.draw(st))
        return 0
    print(__doc__)
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv))
