#!/usr/bin/env python3
"""Surface spec tool.

    python3 tools/surface_spec/run.py build          build surface/ into crates/vdrum-surface/spec/surface.json
                                                    and crates/vdrum-sim/scenarios.json
    python3 tools/surface_spec/run.py check          fail if a built file is missing changes or the spec has errors
    python3 tools/surface_spec/run.py watch          rebuild whenever a file in surface/ changes
    python3 tools/surface_spec/run.py serve [PORT] [--source DIR --out DIR]
                                                    simulator page at http://localhost:PORT (8737), rebuilt and
                                                    reloaded on every save
    python3 tools/surface_spec/run.py bundle DIR     the simulator page as static files (hosting)
    python3 tools/surface_spec/run.py docs           write the generated docs (mode images, control tables,
                                                    navigation diagram) into docs/lcxl3-daw-mode/generated/
    python3 tools/surface_spec/run.py svg OUT.svg [--demo]   draw the device
"""

import json
import os
import shutil
import subprocess
import sys
import threading
import time
import traceback

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from surface_spec import build as b  # noqa: E402

SIM_DIR = os.path.join(b.ROOT, "tools", "surface_sim")
SIM_BUILD = os.path.join(SIM_DIR, "build")
WASM = os.path.join(b.ROOT, "target", "wasm32-unknown-unknown", "release", "vdrum_sim.wasm")


# Sources and outputs; `serve --source DIR --out DIR` changes them (the browser tests use a copy).
PATHS = {"source": b.SOURCE_DIR, "spec": b.OUTPUT, "scenarios": b.SCENARIOS_OUTPUT}


def outputs():
    """(spec, scenarios, errors, {path: text}) for the current sources."""
    spec, scs, errors = b.build(PATHS["source"])
    files = {}
    if not errors:
        files[PATHS["spec"]] = b.dumps(spec)
        files[PATHS["scenarios"]] = b.dumps_scenarios(scs)
    return spec, scs, errors, files


def cmd_build(write=True, quiet=False):
    try:
        spec, scs, errors, files = outputs()
    except Exception:
        traceback.print_exc()
        return 1, [traceback.format_exc().strip().splitlines()[-1]]
    for e in errors:
        print("error:", e, file=sys.stderr)
    if errors:
        return 1, errors
    if write:
        for path, text in files.items():
            try:
                with open(path) as fh:
                    if fh.read() == text:
                        continue
            except FileNotFoundError:
                pass
            with open(path, "w") as fh:
                fh.write(text)
        if not quiet:
            print(f"wrote {os.path.relpath(PATHS['spec'])} ({len(spec['modes'])} modes), "
                  f"{os.path.relpath(PATHS['scenarios'])} ({len(scs)} scenarios)")
        return 0, []
    stale = []
    for path, text in files.items():
        try:
            with open(path) as fh:
                if fh.read() != text:
                    stale.append(path)
        except FileNotFoundError:
            stale.append(path)
    if stale:
        for p in stale:
            print(f"{os.path.relpath(p)} is out of date: run `python3 tools/surface_spec/run.py build`", file=sys.stderr)
        return 1, []
    print("spec ok, built files up to date")
    return 0, []


def stamp():
    s = []
    for d, _, files in os.walk(PATHS["source"]):
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


def build_wasm():
    """Build the simulator for the browser (needs the wasm32-unknown-unknown target)."""
    r = subprocess.run(["cargo", "build", "-q", "-p", "vdrum-sim", "--release", "--target", "wasm32-unknown-unknown"],
                       cwd=b.ROOT)
    if r.returncode != 0:
        raise SystemExit("cargo build for wasm32-unknown-unknown failed (rustup target add wasm32-unknown-unknown)")


def write_device_svg(path):
    from surface_spec import device_svg
    with open(path, "w") as fh:
        fh.write(device_svg.draw({}))


def page_files():
    """The simulator page's files: published path -> source path."""
    return {
        "index.html": os.path.join(SIM_DIR, "index.html"),
        "sim.js": os.path.join(SIM_DIR, "sim.js"),
        "sim.css": os.path.join(SIM_DIR, "sim.css"),
        "device.svg": os.path.join(SIM_BUILD, "device.svg"),
        "vdrum_sim.wasm": WASM,
        "surface.json": PATHS["spec"],
        "scenarios.json": PATHS["scenarios"],
    }


def prepare_page():
    os.makedirs(SIM_BUILD, exist_ok=True)
    cmd_build()
    build_wasm()
    write_device_svg(os.path.join(SIM_BUILD, "device.svg"))


def cmd_serve(port=8737):
    """Serve the simulator; rebuild the spec on every save and tell the page (GET /status)."""
    import http.server

    prepare_page()
    files = page_files()
    live = {"version": 0, "errors": []}

    def watcher():
        last = stamp()
        while True:
            time.sleep(0.3)
            now = stamp()
            if now == last:
                continue
            last = now
            code, errors = cmd_build(quiet=True)
            live["errors"] = errors
            if code == 0:
                live["version"] += 1
            print(time.strftime("%H:%M:%S"), "rebuilt" if code == 0 else f"build failed: {errors}")

    threading.Thread(target=watcher, daemon=True).start()

    class Handler(http.server.BaseHTTPRequestHandler):
        def do_GET(self):
            path = self.path.split("?")[0].lstrip("/") or "index.html"
            if path == "status":
                body = json.dumps(live).encode()
                ctype = "application/json"
            elif path in files and os.path.exists(files[path]):
                with open(files[path], "rb") as fh:
                    body = fh.read()
                ctype = {"html": "text/html", "js": "text/javascript", "css": "text/css", "svg": "image/svg+xml",
                         "wasm": "application/wasm", "json": "application/json"}[path.rsplit(".", 1)[1]]
            else:
                self.send_error(404)
                return
            self.send_response(200)
            self.send_header("Content-Type", ctype + ("; charset=utf-8" if ctype.startswith("text") or ctype.endswith("json") else ""))
            self.send_header("Cache-Control", "no-store")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def log_message(self, *a):
            pass

    srv = http.server.ThreadingHTTPServer(("127.0.0.1", port), Handler)
    print(f"simulator at http://localhost:{port} (Ctrl+C to stop); edits in surface/ reload the page's spec")
    srv.serve_forever()


def cmd_bundle(out_dir):
    prepare_page()
    os.makedirs(out_dir, exist_ok=True)
    for name, src in page_files().items():
        shutil.copyfile(src, os.path.join(out_dir, name))
    print(f"wrote the simulator page to {out_dir}")


def main(argv):
    if len(argv) < 2:
        print(__doc__)
        return 2
    c = argv[1]
    if c == "build":
        return cmd_build()[0]
    if c == "check":
        return cmd_build(write=False)[0]
    if c == "watch":
        return cmd_watch()
    if c == "serve":
        args = argv[2:]
        port = 8737
        while args:
            a = args.pop(0)
            if a == "--source":
                PATHS["source"] = os.path.abspath(args.pop(0))
            elif a == "--out":
                out = os.path.abspath(args.pop(0))
                os.makedirs(out, exist_ok=True)
                PATHS["spec"] = os.path.join(out, "surface.json")
                PATHS["scenarios"] = os.path.join(out, "scenarios.json")
            else:
                port = int(a)
        return cmd_serve(port)
    if c == "bundle" and len(argv) > 2:
        return cmd_bundle(argv[2])
    if c == "docs":
        from surface_spec import docs
        return docs.write_all()
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
