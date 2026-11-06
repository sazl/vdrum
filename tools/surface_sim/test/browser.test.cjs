// SPDX-License-Identifier: GPL-3.0-only
// Browser tests of the simulator page (Playwright, Chromium):
//
//   node tools/surface_sim/test/browser.test.cjs            run
//   node tools/surface_sim/test/browser.test.cjs --update   rewrite the reference screenshots
//
// The script copies surface/ to a temporary folder, starts `run.py serve` on that copy (so the
// live-reload test can edit it), and checks: the page starts without errors; screenshots of the
// device against references in test/golden/ (visual); clicks, wheel and drags reach the simulator;
// a spec loaded from a file and a spec rebuilt on save are used at once, and a broken one is
// refused; Web MIDI in and out with a mocked MIDI port (the messages a real LCXL3 would get and
// send); scenario replay; no horizontal scroll at phone width.
"use strict";

const { chromium } = require("playwright");
const { spawn, execFileSync } = require("child_process");
const fs = require("fs");
const os = require("os");
const path = require("path");

const ROOT = path.resolve(__dirname, "..", "..", "..");
const GOLDEN = path.join(__dirname, "golden");
const UPDATE = process.argv.includes("--update");
const PORT = 8790 + Math.floor(Math.random() * 100);
const URL = `http://localhost:${PORT}/`;
// Share of pixels that may differ from a reference screenshot (font rendering differs between
// machines; the device drawing itself does not).
const MAX_DIFF = 0.004;

let failures = 0;
async function test(name, f) {
  try {
    await f();
    console.log("ok   " + name);
  } catch (e) {
    failures++;
    console.log("FAIL " + name + "\n     " + (e && e.stack ? e.stack.split("\n").slice(0, 3).join("\n     ") : e));
  }
}
function assert(cond, msg) {
  if (!cond) throw new Error(msg || "assertion failed");
}
function eq(a, b, msg) {
  const x = JSON.stringify(a), y = JSON.stringify(b);
  if (x !== y) throw new Error(`${msg || "values differ"}: ${x} != ${y}`);
}
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

// A Web MIDI stand-in: one input and one output named like the device's DAW ports. Everything the
// page sends is kept in window.__sent; window.__midiIn(bytes) delivers a message from the "device".
const MIDI_MOCK = () => {
  window.__sent = [];
  const input = { id: "in-1", name: "LCXL3 1 DAW Out", type: "input", onmidimessage: null };
  const output = { id: "out-1", name: "LCXL3 1 DAW In", type: "output", send: (m) => window.__sent.push(Array.from(m)) };
  window.__midiIn = (bytes) => input.onmidimessage && input.onmidimessage({ data: new Uint8Array(bytes) });
  window.__midiRequests = [];
  navigator.requestMIDIAccess = async (opts) => {
    window.__midiRequests.push(opts);
    return { inputs: new Map([["in-1", input]]), outputs: new Map([["out-1", output]]) };
  };
};

async function openPage(browser, opts = {}) {
  const page = await browser.newPage({ viewport: opts.viewport || { width: 1500, height: 1100 }, deviceScaleFactor: 1 });
  page.errors = [];
  // Aborted font requests show up as "Failed to load resource" without the URL.
  page.on("console", (m) => m.type() === "error" && !/Failed to load resource: net::ERR_FAILED/.test(m.text()) && page.errors.push(m.text()));
  page.on("requestfailed", (r) => !/fonts\.(googleapis|gstatic)/.test(r.url()) && page.errors.push("request failed: " + r.url()));
  page.on("pageerror", (e) => page.errors.push(String(e)));
  // Google Fonts may be unreachable (tests run offline too); block them so every run uses the
  // same fallback faces.
  await page.route(/fonts\.(googleapis|gstatic)\.com/, (r) => r.abort());
  if (opts.midi) await page.addInitScript(MIDI_MOCK);
  await page.goto(URL);
  await page.waitForFunction(() => window.SIM && window.SIM.state && window.SIM.state() && document.querySelector("#screen-image"));
  await page.waitForTimeout(150);
  return page;
}

const st = (page) => page.evaluate(() => window.SIM.state());
const ledFill = (page, key) => page.evaluate((k) => {
  const el = document.getElementById("led-" + k);
  return el.getAttribute(el.tagName === "polyline" ? "stroke" : "fill");
}, key);

async function center(page, sel) {
  const b = await page.locator(sel).boundingBox();
  return { x: b.x + b.width / 2, y: b.y + b.height / 2 };
}

// Compare a screenshot with its reference inside the browser (no image libraries needed).
async function visual(page, name, locator) {
  const file = path.join(GOLDEN, name + ".png");
  const shot = await locator.screenshot();
  if (UPDATE || !fs.existsSync(file)) {
    fs.mkdirSync(GOLDEN, { recursive: true });
    fs.writeFileSync(file, shot);
    console.log("     wrote " + path.relative(ROOT, file));
    return;
  }
  const diff = await page.evaluate(async ([a, b]) => {
    const load = (src) => new Promise((res, rej) => { const i = new Image(); i.onload = () => res(i); i.onerror = rej; i.src = src; });
    const [x, y] = await Promise.all([load(a), load(b)]);
    if (x.width !== y.width || x.height !== y.height) return { size: [x.width, x.height, y.width, y.height] };
    const px = (img) => { const c = document.createElement("canvas"); c.width = img.width; c.height = img.height; const g = c.getContext("2d"); g.drawImage(img, 0, 0); return g.getImageData(0, 0, img.width, img.height).data; };
    const p = px(x), q = px(y);
    let n = 0;
    for (let i = 0; i < p.length; i += 4) if (Math.abs(p[i] - q[i]) + Math.abs(p[i + 1] - q[i + 1]) + Math.abs(p[i + 2] - q[i + 2]) > 48) n++;
    return { share: n / (p.length / 4) };
  }, ["data:image/png;base64," + shot.toString("base64"), "data:image/png;base64," + fs.readFileSync(file).toString("base64")]);
  if (diff.size) throw new Error(`${name}: size ${diff.size[0]}x${diff.size[1]}, reference ${diff.size[2]}x${diff.size[3]}`);
  if (diff.share > MAX_DIFF) {
    const out = path.join(os.tmpdir(), "vdrum-sim-" + name + ".png");
    fs.writeFileSync(out, shot);
    throw new Error(`${name}: ${(diff.share * 100).toFixed(2)} % of pixels differ (limit ${MAX_DIFF * 100} %); screenshot in ${out}`);
  }
}

async function main() {
  const tmp = fs.mkdtempSync(path.join(os.tmpdir(), "vdrum-sim-test-"));
  const src = path.join(tmp, "surface");
  fs.cpSync(path.join(ROOT, "surface"), src, { recursive: true, filter: (p) => !p.includes("__pycache__") });
  const server = spawn("python3", [path.join(ROOT, "tools/surface_spec/run.py"), "serve", String(PORT), "--source", src, "--out", path.join(tmp, "out")],
    { cwd: ROOT, stdio: ["ignore", "pipe", "inherit"] });
  let serverOut = "";
  server.stdout.on("data", (d) => (serverOut += d));
  for (let i = 0; i < 600 && !serverOut.includes("simulator at"); i++) await sleep(200);
  if (!serverOut.includes("simulator at")) throw new Error("server did not start:\n" + serverOut);

  const browser = await chromium.launch();
  try {
    await test("page starts with the built spec and no errors", async () => {
      const page = await openPage(browser);
      const s = await st(page);
      eq(s.surface.name, "Drum 1");
      // 48 drawn LEDs: the surface also colours the Back button right of Solo/Arm (CC 104), whose
      // logo was taken out of the drawing; its colour is in the state.
      eq(await page.locator(".led").count(), 48, "LED elements");
      eq(Object.keys(s.leds).length, 49, "LEDs the surface sets");
      assert(/surface\.json loaded/.test(await page.textContent("#spec-status")), await page.textContent("#spec-status"));
      eq(await page.locator("#rules tbody tr").count(), JSON.parse(fs.readFileSync(path.join(tmp, "out", "surface.json"))).navigation.length);
      eq(page.errors, [], "console errors");
      await page.close();
    });

    await test("visual: the device at start, with Shift held, and in the sequencer view", async () => {
      const page = await openPage(browser);
      await page.uncheck("#show-labels");
      await page.evaluate(() => { window.SIM.apply({ cmd: "reset" }); });
      // freeze the clock so the pulsing mode LED is the same in every run
      await page.evaluate(() => { window.SIM.apply({ cmd: "tick", now: 0 }); });
      const dev = page.locator("#device");
      await page.waitForTimeout(100);
      await visual(page, "device-start", dev);
      await page.evaluate(() => window.SIM.apply({ cmd: "button", key: "SHIFT", pressed: true }));
      await page.waitForTimeout(100);
      const s = await st(page);
      eq(s.leds.T2, [40, 40, 40], "other mode slot");
      eq(s.leds.B5, [0, 0, 0], "empty slot");
      await page.evaluate(() => window.SIM.apply({ cmd: "button", key: "SHIFT", pressed: false }));
      await page.evaluate(() => window.SIM.apply({ cmd: "press", key: "SOLO" }));
      await page.waitForTimeout(100);
      await visual(page, "device-seq-view", dev);
      await page.close();
    });

    await test("clicks, wheel and drags reach the simulator", async () => {
      const page = await openPage(browser);
      const solo = await center(page, "#ctl-SOLO");
      await page.mouse.click(solo.x, solo.y);
      await page.waitForTimeout(80);
      eq(await ledFill(page, "SOLO"), "rgb(254,254,254)", "Solo/Arm LED");
      eq((await st(page)).surface.seq_view, true);
      await page.mouse.click(solo.x, solo.y);
      const k = await center(page, "#ctl-K11 circle");
      await page.mouse.move(k.x, k.y);
      await page.mouse.wheel(0, -100);
      await page.mouse.wheel(0, -100);
      await page.waitForTimeout(80);
      let s = await st(page);
      eq(s.screen.lines, ["DRUM 1 Oscillator", "Fine", "2"]);
      eq(await page.getAttribute("#screen-image", "data-lines"), "DRUM 1 Oscillator|Fine|2");
      // drag K12 up by 3 detents (6 px each)
      const k2 = await center(page, "#ctl-K12 circle");
      await page.mouse.move(k2.x, k2.y);
      await page.mouse.down();
      await page.mouse.move(k2.x, k2.y - 19, { steps: 5 });
      await page.mouse.up();
      s = await st(page);
      eq(s.turns[1], 3, "K12 detents");
      // fader 8 to the top
      const capBefore = await page.getAttribute("#ctl-F8 .fader-cap", "transform");
      const f = await center(page, "#ctl-F8 .fader-cap");
      await page.mouse.move(f.x, f.y);
      await page.mouse.down();
      await page.mouse.move(f.x, f.y - 400, { steps: 8 });
      await page.mouse.up();
      await page.waitForTimeout(80);
      s = await st(page);
      eq(s.faders[7], 127, "fader 8 position");
      assert((await page.getAttribute("#ctl-F8 .fader-cap", "transform")) !== capBefore, "fader cap moved");
      const log = await page.evaluate(() => window.SIM.call({ cmd: "log", from: 0 }));
      assert(log.some((e) => e.kind === "param" && e.text.startsWith("fine1 ")), "param log entry");
      assert(await page.locator("#log li.param").count() > 0, "log shows parameter changes");
      // hold a button with the mouse: Record lights bright while held in the sequencer view
      await page.mouse.click(solo.x, solo.y);
      const rec = await center(page, "#ctl-REC");
      await page.mouse.move(rec.x, rec.y);
      await page.mouse.down();
      await page.waitForTimeout(50);
      eq((await st(page)).leds.REC, [254, 0, 0], "Record held");
      await page.mouse.up();
      await page.waitForTimeout(50);
      eq((await st(page)).leds.REC, [64, 0, 0], "Record released");
      // right-click latches Shift
      const sh = await center(page, "#ctl-SHIFT");
      await page.mouse.click(sh.x, sh.y, { button: "right" });
      eq((await st(page)).surface.shift, true, "Shift latched");
      await page.mouse.click(sh.x, sh.y);
      eq((await st(page)).surface.shift, false, "Shift released");
      // Space holds Shift
      await page.keyboard.down("Space");
      eq((await st(page)).surface.shift, true, "Space = Shift");
      await page.keyboard.up("Space");
      eq(page.errors, [], "console errors");
      await page.close();
    });

    await test("a spec loaded from a file is used at once; a broken one is refused", async () => {
      const page = await openPage(browser);
      const text = fs.readFileSync(path.join(tmp, "out", "surface.json"), "utf8").replace('"slot": 10,', '"slot": 13,');
      const f = path.join(tmp, "moved.json");
      fs.writeFileSync(f, text);
      await page.setInputFiles("#spec-file", f);
      await page.waitForTimeout(100);
      assert(/moved\.json loaded/.test(await page.textContent("#spec-status")));
      await page.evaluate(() => { const A = window.SIM.apply; A({ cmd: "button", key: "SHIFT", pressed: true }); A({ cmd: "press", key: "B5" }); A({ cmd: "button", key: "SHIFT", pressed: false }); });
      eq((await st(page)).surface.mode, "perform");
      fs.writeFileSync(f, "{ not json");
      await page.setInputFiles("#spec-file", f);
      await page.waitForTimeout(100);
      assert(/previous spec is still running/.test(await page.textContent("#spec-status")));
      eq((await st(page)).surface.slot, 13, "the running spec stays");
      await page.close();
    });

    await test("live reload: a saved change to surface/ reaches the open page; a build error is shown", async () => {
      const page = await openPage(browser);
      await page.waitForTimeout(700);
      const file = path.join(src, "rules", "screen.py");
      const orig = fs.readFileSync(file, "utf8");
      fs.writeFileSync(file, orig.replace('s.show(["{mode_slot} {mode_name}", "{kit}", "{last_touched}"])', 's.show(["Mode {mode_slot}: {mode_name}", "{kit}", "{last_touched}"])'));
      await page.waitForFunction(() => window.SIM.state().screen.lines[0] === "Mode 1: Drum 1", null, { timeout: 8000 })
        .catch((e) => { throw new Error(e.message + "\nserver: " + serverOut.slice(-400)); });
      assert(/rebuilt surface\.json loaded/.test(await page.textContent("#spec-status")));
      fs.writeFileSync(file, orig.replace('"{kit}", "{last_touched}"])', '"{kit}", "{no_such_variable}"])'));
      await page.waitForFunction(() => /build failed/.test(document.getElementById("spec-status").textContent), null, { timeout: 8000 });
      assert(/no_such_variable/.test(await page.textContent("#spec-status")));
      eq((await st(page)).screen.lines[0], "Mode 1: Drum 1", "the previous spec keeps running");
      fs.writeFileSync(file, orig);
      await page.waitForFunction(() => window.SIM.state().screen.lines[0] === "1 Drum 1", null, { timeout: 8000 });
      await page.close();
    });

    await test("Web MIDI out: connecting sends the device the connect sequence and every LED", async () => {
      const page = await openPage(browser, { midi: true });
      await page.click("#midi-connect");
      await page.waitForTimeout(150);
      eq(await page.evaluate(() => window.__midiRequests), [{ sysex: true }], "asked for SysEx access");
      const sent = await page.evaluate(() => window.__sent);
      eq(sent[0], [0x9f, 0x0c, 0x7f], "DAW mode on first");
      const ledMsgs = sent.filter((m) => m.length === 13 && m[6] === 0x01 && m[7] === 0x53);
      eq(new Set(ledMsgs.map((m) => m[8])).size, 49, "one RGB message per LED");
      assert(sent.some((m) => m[0] === 0xb6 && m[1] === 113 && m[2] === 10), "display timeout 1 s (CC 113 on channel 7)");
      eq(await page.textContent("#midi-connect"), "Disconnect");
      await page.close();
    });

    await test("Web MIDI in: the device's messages drive the surface; the answers go back out", async () => {
      const page = await openPage(browser, { midi: true });
      await page.click("#midi-connect");
      await page.waitForTimeout(100);
      await page.evaluate(() => (window.__sent = []));
      // Solo/Arm press and release (CC 65 on channel 1)
      await page.evaluate(() => { window.__midiIn([0xb0, 65, 127]); window.__midiIn([0xb0, 65, 0]); });
      await page.waitForTimeout(100);
      eq((await st(page)).surface.seq_view, true);
      let sent = await page.evaluate(() => window.__sent);
      assert(sent.some((m) => m.length === 13 && m[7] === 0x53 && m[8] === 65 && m[9] === 127 && m[10] === 127 && m[11] === 127), "Solo/Arm LED white");
      const text = (m) => String.fromCharCode(...m.slice(9, -1));
      assert(sent.some((m) => m[6] === 0x06 && m[7] === 0x36 && text(m) === "Sequencer view"), "popup title to the overlay");
      assert(sent.some((m) => m[6] === 0x04 && m[7] === 0x36 && m[8] === 0x7f), "overlay triggered");
      // a relative encoder message (K11, CC 77 on channel 16, +1)
      await page.evaluate(() => (window.__sent = []));
      await page.evaluate(() => window.__midiIn([0xbf, 77, 65]));
      await page.waitForTimeout(100);
      sent = await page.evaluate(() => window.__sent);
      assert(sent.some((m) => m[6] === 0x06 && m[7] === 0x36 && m[8] === 1 && text(m) === "Fine"), "popup name Fine");
      // with a device attached the on-screen controls do nothing (the device sends its own input)
      const before = (await st(page)).turns[0];
      const k = await center(page, "#ctl-K11 circle");
      await page.mouse.move(k.x, k.y);
      await page.mouse.wheel(0, -100);
      eq((await st(page)).turns[0], before, "virtual turn ignored");
      await page.click("#midi-connect");
      sent = await page.evaluate(() => window.__sent);
      eq(sent[sent.length - 1], [0x9f, 0x0c, 0x00], "DAW mode off on disconnect");
      eq(page.errors, [], "console errors");
      await page.close();
    });

    await test("scenario replay passes in the page", async () => {
      const page = await openPage(browser);
      const n = await page.locator("#scenario option").count();
      assert(n >= 5, "scenarios listed: " + n);
      for (let i = 0; i < Math.min(n, 3); i++) {
        await page.selectOption("#scenario", String(i));
        await page.click("#scenario-run");
        await page.waitForFunction(() => /passed|failed/.test(document.getElementById("scenario-result").textContent), null, { timeout: 30000 });
        const r = await page.textContent("#scenario-result");
        assert(/^passed/.test(r), r);
      }
      await page.close();
    });

    await test("phone width: no horizontal scroll", async () => {
      const page = await openPage(browser, { viewport: { width: 400, height: 900 } });
      const w = await page.evaluate(() => [document.documentElement.scrollWidth, window.innerWidth]);
      assert(w[0] <= w[1], `page is ${w[0]} px wide in a ${w[1]} px window`);
      await page.close();
    });
  } finally {
    await browser.close();
    server.kill();
    fs.rmSync(tmp, { recursive: true, force: true });
  }
  console.log(failures ? `${failures} failed` : "all passed");
  process.exit(failures ? 1 : 0);
}

main().catch((e) => {
  console.error(e);
  process.exit(1);
});
