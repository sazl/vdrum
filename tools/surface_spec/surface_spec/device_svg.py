"""Draw the Launch Control XL 3 top panel as SVG from hardware.py (units: mm).

Every control is a <g id="ctl-KEY" data-control="KEY" data-kind="..."> and every LED an element
with id "led-KEY" and class "led", so a simulator can colour LEDs, move faders and catch input by
id. The screen is <g id="screen"> with the 128 x 64 active area as <rect id="screen-active">.

    python3 tools/surface_spec/run.py svg OUT.svg [--demo]

--demo lights the LEDs and sets fader positions and screen text to show every element.
"""

import sys
from html import escape

from . import hardware as hw

FONT = "Inter, 'Helvetica Neue', Arial, sans-serif"
PANEL = "#3c3b3f"
HOUSING = "#252428"
HOUSING_EDGE = "#151417"
PRINT = "#c9c9cd"
CAP = "#58575c"
CAP_EDGE = "#6c6b70"
LED_OFF = "#2a292d"


def f(v):
    return f"{v:.2f}".rstrip("0").rstrip(".")


def rect(x0, y0, x1, y1, r=0.0, **a):
    attrs = " ".join(f'{k.replace("_", "-")}="{v}"' for k, v in a.items())
    return f'<rect x="{f(x0)}" y="{f(y0)}" width="{f(x1 - x0)}" height="{f(y1 - y0)}" rx="{f(r)}" {attrs}/>'


def text(x, y, s, size, weight=400, anchor="middle", fill=PRINT, extra=""):
    return (f'<text x="{f(x)}" y="{f(y)}" font-family="{FONT}" font-size="{f(size)}" font-weight="{weight}" '
            f'text-anchor="{anchor}" dominant-baseline="central" fill="{fill}" {extra}>{escape(s)}</text>')


def led_colour(state, key, default=LED_OFF):
    rgb = state.get("leds", {}).get(key)
    return default if rgb is None else "#%02x%02x%02x" % rgb


def draw(state=None):
    state = state or {}
    out = []
    W, H = hw.PANEL_W, hw.PANEL_H
    out.append(f'<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {f(W)} {f(H)}" width="{f(W * 4)}" height="{f(H * 4)}">')
    out.append("<title>Launch Control XL 3 top panel</title>")
    out.append('<defs><radialGradient id="cap" cx="45%" cy="40%" r="60%"><stop offset="0" stop-color="#626166"/>'
               f'<stop offset="1" stop-color="{CAP}"/></radialGradient>'
               '<filter id="sh" x="-30%" y="-30%" width="160%" height="160%"><feDropShadow dx="0.8" dy="1.2" '
               'stdDeviation="0.9" flood-color="#000" flood-opacity="0.55"/></filter>'
               '<filter id="glow" x="-50%" y="-50%" width="200%" height="200%"><feGaussianBlur stdDeviation="0.35"/>'
               '</filter></defs>')
    out.append(rect(0, 0, W, H, hw.PANEL["corner_r"], fill=PANEL, id="panel"))

    # Screen
    sc = hw.SCREEN
    w, a = sc["window"], sc["active"]
    out.append('<g id="screen">' + rect(w["x0"] - 0.6, w["y0"] - 0.6, w["x1"] + 0.6, w["y1"] + 0.6, w["r"] + 0.6, fill="#1b1a1d")
               + rect(w["x0"], w["y0"], w["x1"], w["y1"], w["r"], fill="#050507")
               + rect(a["x0"], a["y0"], a["x1"], a["y1"], 0, id="screen-active", fill="#050507",
                      data_pixels="%dx%d" % sc["pixels"]))
    lines = state.get("screen")
    if lines:
        px = (a["y1"] - a["y0"]) / sc["pixels"][1]
        # (text, font size in OLED pixels, centre row in OLED pixels). The device's own font and
        # line positions are not documented; these are placeholders (see the doc, "Screen").
        for s_, size_px, row in lines:
            out.append(text((a["x0"] + a["x1"]) / 2, a["y0"] + px * row, s_, px * size_px, 500, fill="#e8ecf2",
                            extra='class="screen-line"'))
    out.append("</g>")

    # Side housings and labels
    x0, x1 = hw.SIDE_X0, hw.SIDE_X1
    for name, h in hw.SIDE.items():
        out.append(rect(x0, h["y0"], x1, h["y1"], 1.4, fill=HOUSING, stroke=HOUSING_EDGE, stroke_width="0.4", filter="url(#sh)"))
        if h["split"] == "v":
            mx = (x0 + x1) / 2
            out.append(f'<line x1="{f(mx)}" y1="{f(h["y0"] + 2)}" x2="{f(mx)}" y2="{f(h["y1"] - 2)}" stroke="{HOUSING_EDGE}" stroke-width="1.2" stroke-linecap="round"/>')
        else:
            out.append(f'<line x1="{f(x0 + 2)}" y1="{f(h["split_y"])}" x2="{f(x1 - 2)}" y2="{f(h["split_y"])}" stroke="{HOUSING_EDGE}" stroke-width="1" stroke-linecap="round"/>')
        if "label" in h:
            s_, (lx, ly) = h["label"]
            out.append(text(lx, ly, s_, 2.3))
        if "sublabel" in h:
            s_, (lx, ly) = h["sublabel"]
            out.append(text(lx, ly, "└ " + s_ + " ┘", 2.3))

    # Fader ticks
    t = hw.FADER_TICKS
    for tx in t["x"]:
        for i, ty in enumerate(t["y"]):
            half = (t["long"] if i % 2 == 0 else t["short"]) / 2
            out.append(f'<line x1="{f(tx - half)}" y1="{f(ty)}" x2="{f(tx + half)}" y2="{f(ty)}" stroke="#d6d6da" stroke-width="0.35"/>')

    # Strips
    st = hw.STRIP
    for i, row in enumerate(st["rows"]):
        out.append(rect(st["x0"], row["y0"], st["x1"], row["y1"], st["r"], fill=HOUSING, stroke=HOUSING_EDGE, stroke_width="0.4"))
        for dx in st["dividers_x"]:
            out.append(f'<line x1="{f(dx)}" y1="{f(row["y0"] + 2.1)}" x2="{f(dx)}" y2="{f(row["y1"] - 2.1)}" stroke="{HOUSING_EDGE}" stroke-width="0.5"/>')
    cap0, cap1 = st["caption_y"]
    lx = (st["label_seg"]["x0"] + st["label_seg"]["x1"]) / 2
    out.append(text(lx, cap0, "DAW Control", 2.3))
    out.append(text(lx, cap1, "DAW Mixer", 2.3))
    for c in range(8):
        out.append(text(hw.COLUMN_X[c], cap0, str(c + 1), 2.3))
        out.append(text(hw.COLUMN_X[c], cap1, str(c + 9), 2.3))

    # Controls
    faders = state.get("faders", {})
    for c in hw.controls():
        key, kind = c["key"], c["kind"]
        g = [f'<g id="ctl-{key}" data-control="{key}" data-kind="{kind}">']
        if kind == "encoder":
            r = hw.ENCODER["cap_d"] / 2
            g.append(f'<circle cx="{f(c["x"])}" cy="{f(c["y"])}" r="{f(r)}" fill="url(#cap)" stroke="{CAP_EDGE}" stroke-width="0.3" filter="url(#sh)"/>')
            lw, lh = hw.ENCODER["led_w"], hw.ENCODER["led_h"]
            col = led_colour(state, key)
            g.append(rect(c["x"] - lw / 2, c["led"]["y"] - lh / 2, c["x"] + lw / 2, c["led"]["y"] + lh / 2, 0.2,
                          id=f"led-{key}", **{"class": "led"}, fill=col))
        elif kind == "fader":
            fd = hw.FADER
            g.append(rect(c["x"] - fd["slot_w"] / 2, fd["slot_y0"], c["x"] + fd["slot_w"] / 2, fd["slot_y1"], fd["slot_w"] / 2, fill="#0c0b0e"))
            pos = faders.get(key, 0.0)  # 0 = bottom, 1 = top
            cy = fd["travel_y1"] - (fd["travel_y1"] - fd["travel_y0"]) * pos
            g.append(f'<g class="fader-cap" data-travel="{f(fd["travel_y0"])} {f(fd["travel_y1"])}" transform="translate(0 {f(cy)})" filter="url(#sh)">'
                     + rect(c["x"] - fd["cap_w"] / 2, -fd["cap_h"] / 2, c["x"] + fd["cap_w"] / 2, fd["cap_h"] / 2, 0.8, fill="#48474c", stroke="#5c5b60", stroke_width="0.3")
                     + rect(c["x"] - fd["cap_w"] / 2 + 0.6, -0.3, c["x"] + fd["cap_w"] / 2 - 0.6, 0.3, 0.2, fill="#d8d8dc") + "</g>")
        else:
            g.append(rect(c["x0"], c["y0"], c["x1"], c["y1"], 0, fill="transparent", **{"class": "hit"}))
            led = c["led"]
            col = led_colour(state, key, None)
            shape = led["shape"] if led else None
            if shape == "bar":
                lw, lh = st["led_w"], st["led_h"]
                g.append(rect(led["x"] - lw / 2, led["y"] - lh / 2, led["x"] + lw / 2, led["y"] + lh / 2, lh / 2,
                              id=f"led-{key}", **{"class": "led"}, fill=col or LED_OFF))
            elif shape == "text":
                g.append(text((c["x0"] + c["x1"]) / 2, (c["y0"] + c["y1"]) / 2, c["text"], 2.4, 500, fill=col or PRINT,
                              extra=f'id="led-{key}" class="led"'))
            elif shape and shape.startswith("chevron"):
                ic = hw.ICONS[shape]
                (cx, cy), w2, h2 = ic["c"], ic["w"] / 2, ic["h"] / 2
                pts = {"chevron_up": [(-w2, h2), (0, -h2), (w2, h2)], "chevron_down": [(-w2, -h2), (0, h2), (w2, -h2)],
                       "chevron_left": [(w2, -h2), (-w2, 0), (w2, h2)], "chevron_right": [(-w2, -h2), (w2, 0), (-w2, h2)]}[shape]
                d = " ".join(f"{f(cx + x)},{f(cy + y)}" for x, y in pts)
                g.append(f'<polyline id="led-{key}" class="led" points="{d}" fill="none" stroke="{col or PRINT}" stroke-width="0.45" stroke-linecap="round" stroke-linejoin="round"/>')
            elif shape == "dot":
                ic = hw.ICONS["dot"]
                g.append(f'<circle id="led-{key}" class="led" cx="{f(ic["c"][0])}" cy="{f(ic["c"][1])}" r="{f(ic["d"] / 2)}" fill="{col or "#e8264f"}"/>')
            elif shape == "triangle":
                ic = hw.ICONS["triangle"]
                ym = (ic["y0"] + ic["y1"]) / 2
                g.append(f'<polygon id="led-{key}" class="led" points="{f(ic["x0"])},{f(ic["y0"])} {f(ic["x1"])},{f(ym)} {f(ic["x0"])},{f(ic["y1"])}" fill="{col or "#5ef0c8"}"/>')
            elif key in ("SHIFT", "MODE"):
                tx, ty = hw.ICONS["shift_text" if key == "SHIFT" else "mode_text"]
                g.append(text(tx, ty, c["text"], 2.4, 500))
        g.append("</g>")
        out.append("".join(g))
    # Labels (generated mode images): text under each control, in its section colour.
    for c in hw.controls():
        lab = state.get("labels", {}).get(c["key"])
        if not lab:
            continue
        s_, rgb = lab
        col = "#%02x%02x%02x" % tuple(int(v * 0.55 + 255 * 0.45) for v in rgb)
        if c["kind"] == "encoder":
            x, y = c["x"], c["led"]["y"] + 2.6
        elif c["kind"] == "fader":
            x, y = c["x"], hw.FADER["slot_y1"] + 2.4
        else:
            x, y = (c["x0"] + c["x1"]) / 2, c["y1"] - 1.3
        out.append(text(x, y, s_ if len(s_) <= 12 else s_[:11] + "…", 1.75, 500, fill=col, extra='class="cell-label"'))
    out.append("</svg>")
    return "\n".join(out)


def demo_state():
    """LED colours and fader positions as on Novation's product render (for comparison)."""
    leds = {}
    rows = [(201, 95, 230), (110, 160, 240), (80, 235, 200)]
    for r in range(3):
        for c in range(8):
            leds[f"K{r + 1}{c + 1}"] = rows[r]
    ramp = [(201, 95, 230), (150, 130, 235), (120, 150, 235), (130, 190, 225), (120, 215, 210), (95, 235, 200), (95, 235, 200), (95, 235, 200)]
    for c in range(8):
        leds[f"T{c + 1}"] = ramp[c]
        leds[f"B{c + 1}"] = ramp[c]
    faders = {f"F{i + 1}": v for i, v in enumerate([0.08, 0.84, 0.69, 0.92, 0.23, 0.58, 0.69, 0.43])}
    return {"leds": leds, "faders": faders, "screen": [("Mode Select", 11, 14), ("Peak", 14, 32)]}


if __name__ == "__main__":
    st = demo_state() if "--demo" in sys.argv else {}
    with open(sys.argv[1], "w") as fh:
        fh.write(draw(st))
