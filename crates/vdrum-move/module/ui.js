/*
 * VDrum on Ableton Move (Schwung 1.9.2 takeover module). SPDX-License-Identifier: GPL-3.0-only
 *
 * A thin bridge: everything VDrum does runs in dsp.so. This file
 *  - forwards Move's controls (CCs, and notes as a fallback) to the DSP: set_param("in", hex),
 *    3 bytes per message, once per tick;
 *  - reads the DSP's frame, get_param("ui"): "rev\n<LEDs, 2 hex digits each>\n<5 display lines>",
 *  - sends the LEDs that changed, a few per tick (Move's MIDI out buffer holds about 64 packets),
 *  - draws the display lines when they change;
 *  - asks the DSP to save the session when the module is unloaded.
 *
 * LED order (crates/vdrum-move/src/hw.rs): pads 68..99, steps 16..31, track buttons CC 43..40,
 * then the buttons below.
 */

import { setLED, setButtonLED, invalidateLedCache } from '/data/UserData/schwung/shared/input_filter.mjs';

const BUTTON_CCS = [85, 86, 49, 50, 51, 56, 58, 60, 88, 119, 62, 63, 54, 55, 52, 118];
const LEDS = [];
for (let n = 68; n <= 99; n++) LEDS.push({ note: true, id: n });
for (let n = 16; n <= 31; n++) LEDS.push({ note: true, id: n });
for (let cc = 43; cc >= 40; cc--) LEDS.push({ note: false, id: cc });
for (const cc of BUTTON_CCS) LEDS.push({ note: false, id: cc });

const LEDS_PER_TICK = 8;
const FRAME_EVERY = 6;      /* ticks between frame reads (~17 ms) */
const RECHECK_EVERY = 344;  /* ticks between full LED passes (~1 s): an LED whose send was dropped
                               (buffer full) is sent again; the others are skipped by setLED's cache */

let pending = [];           /* hex messages to forward */
let wanted = new Array(LEDS.length).fill(0);
let shown = new Array(LEDS.length).fill(-1);
let cursor = 0;
let rev = "";
let lines = ["VDrum", "loading...", "", "", ""];
let linesDirty = true;
let ticks = 0;

function hex2(v) {
    return (v & 0xff).toString(16).padStart(2, "0");
}

function setParam(key, value) {
    if (typeof host_module_set_param_blocking === "function") {
        host_module_set_param_blocking(key, value, 100);
    } else if (typeof host_module_set_param === "function") {
        host_module_set_param(key, value);
    }
}

function repaint() {
    invalidateLedCache();
    shown.fill(-1);
    linesDirty = true;
}

globalThis.init = function () {
    repaint();
};

globalThis.onResume = function () {
    repaint();
};

globalThis.onUnload = function () {
    setParam("save", "1");
};

globalThis.onMidiMessageInternal = function (data) {
    const status = data[0] & 0xf0;
    if (status === 0xb0) {
        pending.push(hex2(data[0]) + hex2(data[1]) + hex2(data[2]));
    } else if ((status === 0x90 || status === 0x80) && data[1] >= 10) {
        /* knob touches (notes 0..9) are left out; the DSP drops these notes when it gets them
         * directly */
        pending.push(hex2(data[0]) + hex2(data[1]) + hex2(data[2]));
    }
};

globalThis.onMidiMessageExternal = function (data) {};

function readFrame() {
    if (typeof host_module_get_param !== "function") return;
    const f = host_module_get_param("ui");
    if (!f) return;
    const parts = String(f).split("\n");
    if (parts.length < 2 || parts[0] === rev) return;
    rev = parts[0];
    const leds = parts[1];
    for (let i = 0; i < LEDS.length && 2 * i + 1 < leds.length; i++) {
        wanted[i] = parseInt(leds.substr(2 * i, 2), 16) || 0;
    }
    const next = parts.slice(2, 7);
    while (next.length < 5) next.push("");
    for (let i = 0; i < 5; i++) {
        if (next[i] !== lines[i]) {
            lines = next;
            linesDirty = true;
            break;
        }
    }
}

function sendLeds() {
    let sent = 0;
    for (let k = 0; k < LEDS.length && sent < LEDS_PER_TICK; k++) {
        const i = (cursor + k) % LEDS.length;
        if (shown[i] === wanted[i]) continue;
        const l = LEDS[i];
        if (l.note) setLED(l.id, wanted[i]);
        else setButtonLED(l.id, wanted[i]);
        shown[i] = wanted[i];
        sent++;
        cursor = (i + 1) % LEDS.length;
    }
}

function draw() {
    clear_screen();
    fill_rect(0, 0, 128, 11, 1);
    print(1, 2, lines[0], 0);
    for (let i = 1; i < 5; i++) {
        print(1, 2 + 12 * i + 1, lines[i], 1);
    }
    linesDirty = false;
}

globalThis.tick = function () {
    if (pending.length > 0) {
        setParam("in", pending.join(""));
        pending = [];
    }
    if (ticks % RECHECK_EVERY === 0) shown.fill(-1);
    if (ticks++ % FRAME_EVERY === 0) readFrame();
    sendLeds();
    if (linesDirty) draw();
};
