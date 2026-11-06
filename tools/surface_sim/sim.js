// SPDX-License-Identifier: GPL-3.0-only
// VDrum surface simulator page. The simulator itself is crates/vdrum-sim compiled to WebAssembly:
// this file only draws what the virtual device shows, turns pointer and keyboard input into
// gestures on the virtual device, and moves MIDI between the simulator and a real LCXL3 (Web MIDI).
"use strict";

const SIM = {};
const enc = new TextEncoder();
const dec = new TextDecoder();
const OFF = "#2a292d"; // LED_OFF in tools/surface_spec/surface_spec/device_svg.py
const OFF_ICON = "#4a494e";

let ex = null; // wasm exports
let state = null;
let spec = null;
let logFrom = 0;
let t0 = performance.now();
let latched = new Set();
let midi = { access: null, input: null, output: null };
let screenKey = "";
let liveVersion = null;
let scenarios = [];

function call(cmd) {
  const bytes = enc.encode(JSON.stringify(cmd));
  const p = ex.sim_alloc(bytes.length);
  new Uint8Array(ex.memory.buffer, p, bytes.length).set(bytes);
  ex.sim_command(p, bytes.length);
  return JSON.parse(dec.decode(new Uint8Array(ex.memory.buffer, ex.sim_result_ptr(), ex.sim_result_len())));
}
SIM.call = call;

function now() {
  return Math.round(performance.now() - t0);
}

function setStatus(text, kind) {
  const el = document.getElementById("spec-status");
  el.textContent = text;
  el.title = text;
  el.className = "status " + (kind || "");
}

// ---------------------------------------------------------------- loading

async function load() {
  const [svg, wasm, specText] = await Promise.all([
    fetch("device.svg").then((r) => r.text()),
    fetch("vdrum_sim.wasm").then((r) => r.arrayBuffer()),
    fetch("surface.json").then((r) => (r.ok ? r.text() : null)).catch(() => null),
  ]);
  document.getElementById("device").innerHTML = svg;
  const { instance } = await WebAssembly.instantiate(wasm, {});
  ex = instance.exports;
  state = call({ cmd: "reset" });
  if (specText) loadSpec(specText, "surface.json");
  else setStatus("built-in spec", "ok");
  prepareDevice();
  bindInput();
  await loadScenarios();
  requestAnimationFrame(frame);
  pollLive();
}

function loadSpec(text, from) {
  const r = call({ cmd: "spec", json: text });
  if (r.error) {
    setStatus(`${from}: ${r.error} (the previous spec is still running)`, "error");
    return false;
  }
  spec = JSON.parse(text);
  setStatus(`${from} loaded ${new Date().toLocaleTimeString()}`, "ok");
  drawRules();
  return true;
}
SIM.loadSpec = loadSpec;

// Live reload: `run.py serve` answers /status with the build version and errors. A hosted page has
// no such endpoint, so polling stops at the first failure.
async function pollLive() {
  try {
    const r = await fetch("status", { cache: "no-store" });
    if (!r.ok) return;
    const s = await r.json();
    if (s.errors && s.errors.length) {
      setStatus("build failed: " + s.errors.join("; ") + " (the previous spec is still running)", "error");
    } else if (s.version !== liveVersion) {
      const first = liveVersion === null;
      liveVersion = s.version;
      if (!first) {
        const text = await fetch("surface.json", { cache: "no-store" }).then((x) => x.text());
        loadSpec(text, "rebuilt surface.json");
        await loadScenarios();
      }
    }
  } catch (e) {
    return;
  }
  setTimeout(pollLive, 500);
}

async function loadScenarios() {
  try {
    const r = await fetch("scenarios.json", { cache: "no-store" });
    if (!r.ok) return;
    scenarios = (await r.json()).scenarios || [];
  } catch (e) {
    scenarios = [];
  }
  const sel = document.getElementById("scenario");
  if (!sel) return;
  sel.innerHTML = scenarios.map((s, i) => `<option value="${i}">${esc(s.name)}</option>`).join("");
}

// ---------------------------------------------------------------- device drawing

const leds = new Map(); // key -> { el, attr, off }
let labelLayer = null;

function prepareDevice() {
  const svg = document.querySelector("#device svg");
  svg.removeAttribute("width");
  svg.removeAttribute("height");
  for (const el of svg.querySelectorAll(".led")) {
    const key = el.id.replace(/^led-/, "");
    const attr = el.tagName === "polyline" ? "stroke" : "fill";
    // Off: the bar LEDs keep the drawing's unlit colour; the lit icons and texts of the side
    // buttons (chevrons, dot, triangle, Solo/Arm and Mute/Select) go dark grey.
    const bar = el.tagName === "rect";
    leds.set(key, { el, attr, off: bar ? el.getAttribute(attr) || OFF : OFF_ICON });
  }
  // The screen: a 128 x 64 image over the active area, drawn pixel by pixel.
  const a = svg.querySelector("#screen-active");
  const img = document.createElementNS("http://www.w3.org/2000/svg", "image");
  for (const k of ["x", "y", "width", "height"]) img.setAttribute(k, a.getAttribute(k));
  img.setAttribute("id", "screen-image");
  img.setAttribute("preserveAspectRatio", "none");
  img.style.imageRendering = "pixelated";
  a.after(img);
  labelLayer = document.createElementNS("http://www.w3.org/2000/svg", "g");
  labelLayer.setAttribute("id", "labels");
  svg.appendChild(labelLayer);
}

function rgb(c) {
  return `rgb(${c[0]},${c[1]},${c[2]})`;
}

function drawLeds() {
  for (const [key, l] of leds) {
    const c = state.leds[key];
    const v = c && (c[0] || c[1] || c[2]) ? rgb(c) : l.off;
    if (l.el.getAttribute(l.attr) !== v) l.el.setAttribute(l.attr, v);
  }
}

function drawFaders() {
  for (let i = 0; i < 8; i++) {
    const cap = document.querySelector(`#ctl-F${i + 1} .fader-cap`);
    if (!cap) continue;
    const [y0, y1] = cap.dataset.travel.split(" ").map(Number);
    const y = y1 - ((y1 - y0) * state.faders[i]) / 127;
    cap.setAttribute("transform", `translate(0 ${y.toFixed(2)})`);
  }
}

// The device's font and line positions are not documented (docs/lcxl3-daw-mode/
// declarative-modes-brainstorm.md, section 7): text is drawn with a generic face at fixed sizes and
// cut to one bit per pixel. Bitmaps the host sends are shown exactly.
const SCREEN_LAYOUT = {
  1: [[13, 22], [18, 50]],
  2: [[9, 10], [13, 31], [18, 55]],
  4: [[13, 22], [18, 50]],
};

function drawScreen() {
  const s = state.screen;
  const key = JSON.stringify(s);
  if (key === screenKey) return;
  screenKey = key;
  const c = document.createElement("canvas");
  c.width = 128;
  c.height = 64;
  const g = c.getContext("2d");
  g.fillStyle = "#000";
  g.fillRect(0, 0, 128, 64);
  if (s.bitmap) {
    const img = g.createImageData(128, 64);
    s.bitmap.forEach((row, y) => {
      for (let x = 0; x < 128; x++) {
        const on = row[x] === "1" ? 255 : 0;
        const i = (y * 128 + x) * 4;
        img.data[i] = img.data[i + 1] = img.data[i + 2] = on;
        img.data[i + 3] = 255;
      }
    });
    g.putImageData(img, 0, 0);
  } else {
    const lay = SCREEN_LAYOUT[s.arrangement] || s.lines.map((_, i) => [9, 9 + i * 8]);
    g.fillStyle = "#fff";
    g.textAlign = "center";
    g.textBaseline = "middle";
    s.lines.forEach((line, i) => {
      const [size, y] = lay[i] || [9, 9 + i * 8];
      let px = size;
      g.font = `600 ${px}px Barlow, Arial, sans-serif`;
      while (px > 7 && g.measureText(line).width > 124) {
        px -= 1;
        g.font = `600 ${px}px Barlow, Arial, sans-serif`;
      }
      g.fillText(line, 64, y);
    });
    // one bit per pixel, as on the OLED
    const img = g.getImageData(0, 0, 128, 64);
    for (let i = 0; i < img.data.length; i += 4) {
      const on = img.data[i] > 110 ? 255 : 0;
      img.data[i] = img.data[i + 1] = img.data[i + 2] = on;
    }
    g.putImageData(img, 0, 0);
  }
  const im = document.getElementById("screen-image");
  im.setAttribute("href", c.toDataURL());
  im.dataset.lines = s.lines.join("|");
  im.dataset.source = s.source;
}

function drawLabels() {
  const show = document.getElementById("show-labels").checked;
  labelLayer.style.display = show ? "" : "none";
  if (!show) return;
  const key = JSON.stringify(Object.entries(state.cells).map(([k, c]) => [k, c.label]));
  if (labelLayer.dataset.key === key) return;
  labelLayer.dataset.key = key;
  labelLayer.innerHTML = "";
  for (const [control, c] of Object.entries(state.cells)) {
    const g = document.getElementById("ctl-" + control);
    if (!g) continue;
    const b = g.getBBox();
    const t = document.createElementNS("http://www.w3.org/2000/svg", "text");
    const kind = g.dataset.kind;
    const x = b.x + b.width / 2;
    const y = kind === "encoder" ? b.y + b.height + 3.6 : kind === "fader" ? b.y + b.height + 2.4 : b.y + b.height - 1.2;
    t.setAttribute("x", x.toFixed(2));
    t.setAttribute("y", y.toFixed(2));
    t.setAttribute("text-anchor", "middle");
    t.setAttribute("class", "cell-label");
    const col = c.colour[0] + c.colour[1] + c.colour[2] > 40 ? c.colour : [200, 200, 205];
    t.setAttribute("fill", rgb(col.map((v) => Math.round(v * 0.55 + 255 * 0.45))));
    t.textContent = c.label.length > 11 ? c.label.slice(0, 10) + "…" : c.label;
    labelLayer.appendChild(t);
  }
}

// ---------------------------------------------------------------- panels

function esc(s) {
  return String(s).replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]);
}

let hoverKey = null;

function drawState() {
  const s = state.surface;
  const rows = [
    ["Mode", `${s.slot} ${s.name}`],
    ["Sequencer view", s.seq_view ? (s.sticky ? "on (sticky)" : "on") : "off"],
    ["Held", s.held.length ? s.held.join(", ") : "none"],
    ["Steps held", s.held_steps.length ? s.held_steps.map((x) => x + 1).join(", ") : "none"],
    ["Seq track / bar", `${s.seq_track + 1} / ${s.seq_bar + 1}`],
    ["Sequencer", `${state.seq.running ? "running" : "stopped"}${state.seq.silent ? ", silent" : ""}, P${String(state.seq.pattern + 1).padStart(2, "0")}`],
    ["Kit", state.kit + (s.browsing ? " (browsing)" : "")],
    ["Layers", s.layers.length ? s.layers.join("; ") : "none"],
    ["MIDI", midi.output ? `${midi.input ? midi.input.name : "?"} / ${midi.output.name}` : "virtual device only"],
  ];
  let html = rows.map(([k, v]) => `<dt>${esc(k)}</dt><dd>${esc(v)}</dd>`).join("");
  if (hoverKey) {
    const c = state.cells[hoverKey];
    html += `<div class="hover">${esc(hoverKey)}: ${c ? esc(`${c.label} · ${c.section}${c.value != null ? " · " + c.value : ""}${c.param ? " · " + c.param : ""}`) : "navigation"}</div>`;
  }
  const el = document.getElementById("state");
  if (el.innerHTML !== html) el.innerHTML = `<dl id="state-list" style="display:contents">${html}</dl>`;
}

function stepText(list) {
  if (list === undefined) return '<span class="muted">next rule</span>';
  if (!list.length) return '<span class="muted">nothing</span>';
  return list.map((st) => esc(st.do + (st.arg !== undefined ? "(" + JSON.stringify(st.arg) + ")" : ""))).join(", ");
}

function drawRules() {
  if (!spec) return;
  const rows = spec.navigation
    .map((r, i) => `<tr data-rule="${i}"><td>${i + 1}</td><td>${esc(r.button)}</td><td class="mono">${esc((r.when || []).join(" "))}</td>`
      + `<td class="mono">${stepText(r.press)}</td><td class="mono">${stepText(r.release)}</td><td class="note">${esc(r.note || "")}</td></tr>`)
    .join("");
  document.getElementById("rules").innerHTML =
    `<div class="table-wrap"><table><thead><tr><th>#</th><th>Button</th><th>When</th><th>Press</th><th>Release</th><th>Note</th></tr></thead><tbody>${rows}</tbody></table></div>`;
}

let lastRule = undefined;
function markRule() {
  const r = state.surface.last_rule;
  if (r === lastRule) return;
  lastRule = r;
  for (const tr of document.querySelectorAll("#rules tr.hit")) tr.classList.remove("hit");
  if (r !== null && r !== undefined) {
    const tr = document.querySelector(`#rules tr[data-rule="${r}"]`);
    if (tr) {
      tr.classList.add("hit");
      tr.scrollIntoView({ block: "nearest" });
    }
  }
}

// The live state diagram: the mode slots as on Shift + button N, the sequencer view and Shift.
function drawDiagram() {
  if (!spec) return;
  const s = state.surface;
  const key = JSON.stringify([s.mode, s.seq_view, s.sticky, s.shift, s.layers, spec.modes]);
  const el = document.getElementById("diagram");
  if (el.dataset.key === key) return;
  el.dataset.key = key;
  const bySlot = {};
  for (const m of spec.modes) bySlot[m.slot] = m;
  const W = 560, bw = 64, bh = 34, gx = 4;
  let out = `<svg viewBox="0 0 ${W} 250" role="img" aria-label="State diagram">`;
  out += `<defs><marker id="arrow" viewBox="0 0 8 8" refX="7" refY="4" markerWidth="6" markerHeight="6" orient="auto"><path d="M0,0 L8,4 L0,8 z" fill="currentColor"/></marker></defs>`;
  out += `<rect class="region" x="1" y="1" width="${W - 2}" height="112" rx="6"/><text class="region-title" x="10" y="16">Mode (Shift + button N)</text>`;
  for (let slot = 1; slot <= 16; slot++) {
    const m = bySlot[slot];
    const x = 10 + ((slot - 1) % 8) * (bw + gx), y = slot <= 8 ? 24 : 64;
    const cls = "st" + (m ? (m.mode === s.mode ? " active" : "") : " empty");
    const name = m ? m.name : "empty";
    out += `<g class="${cls}" data-state="${m ? esc(m.mode) : "slot-" + slot}"><rect x="${x}" y="${y}" width="${bw}" height="${bh}" rx="4"/>`
      + `<text class="num" x="${x + 5}" y="${y + 11}">${slot}</text><text x="${x + bw / 2}" y="${y + 25}" text-anchor="middle">${esc(name.length > 11 ? name.slice(0, 10) + "…" : name)}</text></g>`;
  }
  const sv = s.sticky ? "sticky" : s.seq_view ? "on" : "off";
  out += `<rect class="region" x="1" y="122" width="${W / 2 - 6}" height="74" rx="6"/><text class="region-title" x="10" y="137">Sequencer view (Solo/Arm)</text>`;
  [["off", 14], ["on", 104], ["sticky", 194]].forEach(([n, x]) => {
    out += `<g class="st${sv === n ? " active" : ""}" data-state="seq-${n}"><rect x="${x}" y="146" width="70" height="30" rx="4"/><text x="${x + 35}" y="165" text-anchor="middle">${n}</text></g>`;
  });
  out += `<path class="edge" d="M84 156 L104 156"/><path class="edge" d="M104 168 L84 168"/><path class="edge" d="M174 161 L194 161"/>`;
  out += `<text class="edge-label" x="14" y="190">Solo/Arm toggles; Shift + Solo/Arm makes it sticky</text>`;
  const x2 = W / 2 + 4;
  out += `<rect class="region" x="${x2}" y="122" width="${W / 2 - 5}" height="74" rx="6"/><text class="region-title" x="${x2 + 9}" y="137">Shift</text>`;
  [["up", x2 + 14], ["held", x2 + 104]].forEach(([n, x]) => {
    const on = (n === "held") === s.shift;
    out += `<g class="st${on ? " active" : ""}" data-state="shift-${n}"><rect x="${x}" y="146" width="70" height="30" rx="4"/><text x="${x + 35}" y="165" text-anchor="middle">${n}</text></g>`;
  });
  out += `<text class="edge-label" x="${x2 + 14}" y="190">held: the 16 buttons show the mode slots</text>`;
  out += `<text class="region-title" x="10" y="218">Active layers</text>`;
  out += `<text class="layers" x="10" y="236">${esc(s.layers.length ? s.layers.join("  ·  ") : "none")}</text>`;
  out += `</svg>`;
  el.innerHTML = out;
}

const LOG_MAX = 3000;
function drawLog() {
  if (state.log_len <= logFrom) return;
  const entries = call({ cmd: "log", from: logFrom });
  logFrom += entries.length;
  const ol = document.getElementById("log");
  const atBottom = ol.scrollTop + ol.clientHeight >= ol.scrollHeight - 4;
  const frag = document.createDocumentFragment();
  for (const e of entries) {
    const li = document.createElement("li");
    li.className = e.kind;
    li.innerHTML = `<span class="t">${(e.t / 1000).toFixed(2)}</span><span class="k">${esc(e.kind)}</span><span>${esc(e.text)}</span>`;
    frag.appendChild(li);
  }
  ol.appendChild(frag);
  while (ol.children.length > LOG_MAX) ol.firstChild.remove();
  applyLogFilter();
  if (atBottom) ol.scrollTop = ol.scrollHeight;
}

function applyLogFilter() {
  const off = new Set([...document.querySelectorAll(".filters input")].filter((i) => !i.checked).map((i) => i.dataset.kind));
  for (const li of document.getElementById("log").children) li.hidden = off.has(li.className);
}

// ---------------------------------------------------------------- input

function svgPoint(evt) {
  const svg = document.querySelector("#device svg");
  const p = svg.createSVGPoint();
  p.x = evt.clientX;
  p.y = evt.clientY;
  return p.matrixTransform(svg.getScreenCTM().inverse());
}

function apply(cmd) {
  state = call(cmd);
  return state;
}
SIM.apply = apply;

function button(key, pressed) {
  const g = document.getElementById("ctl-" + key);
  if (g) g.classList.toggle("pressed", pressed);
  if (key === "MODE") return; // the Mode button sends nothing in DAW mode
  if (midi.output) return; // the real device sends its own input
  apply({ cmd: "button", key, pressed });
}

function bindInput() {
  const svg = document.querySelector("#device svg");
  let drag = null;
  svg.addEventListener("contextmenu", (e) => e.preventDefault());
  svg.addEventListener("pointerdown", (e) => {
    const g = e.target.closest("[data-control]");
    if (!g) return;
    const key = g.dataset.control, kind = g.dataset.kind;
    svg.setPointerCapture(e.pointerId);
    if (kind === "button") {
      if (e.button === 2) {
        // latch: held until the next click
        if (latched.has(key)) {
          latched.delete(key);
          g.classList.remove("latched");
          button(key, false);
        } else {
          latched.add(key);
          g.classList.add("latched");
          button(key, true);
        }
        return;
      }
      if (latched.has(key)) {
        latched.delete(key);
        g.classList.remove("latched");
        button(key, false);
        return;
      }
      button(key, true);
      drag = { key, kind };
    } else if (kind === "encoder") {
      drag = { key, kind, y: e.clientY, acc: 0 };
    } else if (kind === "fader") {
      drag = { key, kind };
      moveFader(key, e);
    }
    e.preventDefault();
  });
  svg.addEventListener("pointermove", (e) => {
    if (!drag) return;
    if (drag.kind === "encoder") {
      const dy = drag.y - e.clientY;
      const steps = Math.trunc((dy - drag.acc) / 6);
      if (steps) {
        drag.acc += steps * 6;
        turn(drag.key, steps);
      }
    } else if (drag.kind === "fader") moveFader(drag.key, e);
  });
  const end = () => {
    if (drag && drag.kind === "button") button(drag.key, false);
    drag = null;
  };
  svg.addEventListener("pointerup", end);
  svg.addEventListener("pointercancel", end);
  svg.addEventListener("wheel", (e) => {
    const g = e.target.closest("[data-control]");
    if (!g) return;
    const key = g.dataset.control, kind = g.dataset.kind;
    const d = e.deltaY < 0 ? 1 : -1;
    if (kind === "encoder") turn(key, d);
    else if (kind === "fader") {
      const col = Number(key.slice(1)) - 1;
      if (!midi.output) apply({ cmd: "fader", col, value: Math.max(0, Math.min(127, state.faders[col] + d * 4)) });
    } else return;
    e.preventDefault();
  }, { passive: false });
  svg.addEventListener("pointerover", (e) => {
    const g = e.target.closest("[data-control]");
    hoverKey = g ? g.dataset.control : null;
  });
  addEventListener("keydown", (e) => {
    if (e.code === "Space" && !e.repeat && !(e.target instanceof HTMLInputElement) && !(e.target instanceof HTMLSelectElement)) {
      button("SHIFT", true);
      e.preventDefault();
    }
  });
  addEventListener("keyup", (e) => {
    if (e.code === "Space") button("SHIFT", false);
  });
  document.getElementById("reset").addEventListener("click", () => {
    const text = spec ? JSON.stringify(spec) : null;
    state = call({ cmd: "reset" });
    logFrom = 0;
    document.getElementById("log").innerHTML = "";
    latched.clear();
    for (const g of document.querySelectorAll(".latched")) g.classList.remove("latched");
    if (text) loadSpec(text, "spec");
  });
  document.getElementById("spec-file").addEventListener("change", async (e) => {
    const f = e.target.files[0];
    if (f) loadSpec(await f.text(), f.name);
    e.target.value = "";
  });
  document.getElementById("log-clear").addEventListener("click", () => (document.getElementById("log").innerHTML = ""));
  for (const i of document.querySelectorAll(".filters input")) i.addEventListener("change", applyLogFilter);
  document.getElementById("midi-connect").addEventListener("click", toggleMidi);
  document.getElementById("midi-in").addEventListener("focus", listPorts);
  document.getElementById("midi-out").addEventListener("focus", listPorts);
  const run = document.getElementById("scenario-run");
  if (run) run.addEventListener("click", () => runScenario(Number(document.getElementById("scenario").value)));
}

function turn(key, detents) {
  if (midi.output) return;
  const row = Number(key[1]) - 1, col = Number(key[2]) - 1;
  apply({ cmd: "turn", row, col, detents });
}

function moveFader(key, e) {
  if (midi.output) return;
  const cap = document.querySelector(`#ctl-${key} .fader-cap`);
  const [y0, y1] = cap.dataset.travel.split(" ").map(Number);
  const y = svgPoint(e).y;
  const v = Math.round(((y1 - y) / (y1 - y0)) * 127);
  apply({ cmd: "fader", col: Number(key.slice(1)) - 1, value: Math.max(0, Math.min(127, v)) });
}

// ---------------------------------------------------------------- Web MIDI

async function listPorts() {
  if (!navigator.requestMIDIAccess) {
    setStatus("this browser has no Web MIDI (Chrome, Edge; Firefox with the site permission add-on)", "error");
    return;
  }
  try {
    midi.access = midi.access || (await navigator.requestMIDIAccess({ sysex: true }));
  } catch (err) {
    setStatus("MIDI access refused: " + err.message, "error");
    return;
  }
  const fill = (id, ports) => {
    const sel = document.getElementById(id);
    const cur = sel.value;
    sel.innerHTML = `<option value="">${id === "midi-in" ? "MIDI in" : "MIDI out"}</option>`
      + [...ports.values()].map((p) => `<option value="${esc(p.id)}">${esc(p.name)}</option>`).join("");
    // The LCXL3's DAW port names are not checked here; a port whose name has "DAW" is offered first.
    const daw = [...ports.values()].find((p) => /DAW/i.test(p.name) && /(LCXL|Launch Control)/i.test(p.name));
    sel.value = cur || (daw ? daw.id : "");
  };
  fill("midi-in", midi.access.inputs);
  fill("midi-out", midi.access.outputs);
}

async function toggleMidi() {
  const btn = document.getElementById("midi-connect");
  if (midi.output) {
    apply({ cmd: "hardware", on: false });
    flushMidi(); // the device back to standalone mode
    if (midi.input) midi.input.onmidimessage = null;
    midi.input = midi.output = null;
    btn.textContent = "Connect LCXL3";
    btn.classList.remove("on");
    return;
  }
  await listPorts();
  if (!midi.access) return;
  const inId = document.getElementById("midi-in").value, outId = document.getElementById("midi-out").value;
  midi.input = midi.access.inputs.get(inId) || null;
  midi.output = midi.access.outputs.get(outId) || null;
  if (!midi.output || !midi.input) {
    midi.input = midi.output = null;
    setStatus("choose the LCXL3's DAW in and out ports", "error");
    return;
  }
  midi.input.onmidimessage = (m) => apply({ cmd: "midi_in", bytes: [...m.data] });
  apply({ cmd: "hardware", on: true });
  flushMidi();
  btn.textContent = "Disconnect";
  btn.classList.add("on");
}
SIM.toggleMidi = toggleMidi;

function flushMidi() {
  if (!midi.output) return;
  const out = call({ cmd: "midi_out" });
  for (const m of out) midi.output.send(m);
}

// ---------------------------------------------------------------- scenarios

// Run a scenario from scenarios.json (built from surface/scenarios/*.py) on a fresh simulator,
// step by step so it can be watched; the same steps run in `cargo test -p vdrum-sim`.
async function runScenario(i) {
  const sc = scenarios[i];
  if (!sc) return;
  const res = document.getElementById("scenario-result");
  const text = spec ? JSON.stringify(spec) : null;
  state = call({ cmd: "reset" });
  if (text) call({ cmd: "spec", json: text });
  logFrom = 0;
  document.getElementById("log").innerHTML = "";
  paused = true;
  let failed = null;
  for (const [n, step] of sc.steps.entries()) {
    if (step.expect) {
      const r = call({ cmd: "expect", checks: step.expect });
      if (r.failures.length) failed = `step ${n + 1}: ${r.failures.join("; ")}`;
    } else {
      state = call(step);
    }
    render();
    res.textContent = `step ${n + 1} of ${sc.steps.length}`;
    if (failed) break;
    await new Promise((r) => setTimeout(r, step.cmd === "advance" ? Math.min(step.ms, 800) : 250));
  }
  res.className = "result " + (failed ? "fail" : "pass");
  res.textContent = failed ? "failed: " + failed : `passed (${sc.steps.length} steps)`;
  // continue in real time from the scenario's clock
  t0 = performance.now() - state.now;
  paused = false;
}
SIM.runScenario = runScenario;

// ---------------------------------------------------------------- loop

let paused = false;
function render() {
  drawLeds();
  drawFaders();
  drawScreen();
  drawLabels();
  drawState();
  drawDiagram();
  markRule();
  drawLog();
}

function frame() {
  if (!paused) {
    state = call({ cmd: "tick", now: now() });
    flushMidi();
    render();
  }
  requestAnimationFrame(frame);
}

window.SIM = SIM;
SIM.state = () => state;
load().catch((e) => setStatus("could not start: " + e.message, "error"));
