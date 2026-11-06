"""Launch Control XL 3 hardware: every control, LED, label and the screen, with its position in mm.

Coordinates: origin at the top-left corner of the top panel, x to the right, y towards the player,
in millimetres. The panel is 250 x 239 mm (user guide v1, "Weight and Dimensions").

Positions were measured from Novation's top-view product render (1600 x 1530 px, uploaded to the
project). The panel spans x 7..1594 px and y 6..1524 px of that image, which gives 0.1575 mm/px in
both directions (250 / 1587 = 0.15753, 239 / 1518 = 0.15744), so the render is to scale within
0.1 %. Positions are expected to be within about 0.5 mm; they are not from a drawing by Novation.
Items marked UNVERIFIED are assumptions to check on the device.

Control keys follow layout.py: K<r><c> encoders, F<c> faders, T<c>/B<c> button rows, and the named
side buttons.
"""

PANEL_W = 250.0
PANEL_H = 239.0
_PX_X0, _PX_Y0 = 7.0, 6.0
_MM_PER_PX = 0.1575


def mm(px_x, px_y):
    """Render pixel position to panel mm."""
    return (round((px_x - _PX_X0) * _MM_PER_PX, 2), round((px_y - _PX_Y0) * _MM_PER_PX, 2))


def mm_len(px):
    return round(px * _MM_PER_PX, 2)


# Column centres (encoder LEDs, fader slots and button LEDs line up): 378.6 px + 153.0 px per column.
COLUMN_X = [mm(378.6 + 153.0 * c, 0)[0] for c in range(8)]
COLUMN_PITCH = mm_len(153.0)

PANEL = {"w": PANEL_W, "h": PANEL_H, "corner_r": mm_len(12), "colour": "#3c3b3f"}

LOGO = {  # printed "LAUNCHCONTROL XL", baseline and spans measured
    "launch": {"x0": mm(61, 0)[0], "x1": mm(227.5, 0)[0]},
    "control": {"x0": mm(231, 0)[0], "x1": mm(411, 0)[0]},
    "xl": {"x0": mm(419, 0)[0], "x1": mm(445, 0)[0], "top": mm(0, 43)[1], "bottom": mm(0, 54)[1]},
    "cap_top": mm(0, 42)[1],
    "baseline": mm(0, 65)[1],
}

# Recessed Novation logo plate, top-right corner (decorative; not a control).
NOVATION_PLATE = {"x0": mm(1514, 0)[0], "y0": mm(0, 27)[1], "x1": mm(1587, 0)[0], "y1": mm(0, 102)[1]}

SCREEN = {
    # Bezel window as seen on the render.
    "window": {"x0": mm(59.5, 0)[0], "y0": mm(0, 134)[1], "x1": mm(240, 0)[0], "y1": mm(0, 246)[1], "r": mm_len(9)},
    # OLED 128 x 64 pixels, monochrome (programmer's reference v1.0, "Bitmap"). Active-area size and
    # position UNVERIFIED: assumed 2:1, centred in the window, 25.2 x 12.6 mm.
    "pixels": (128, 64),
    "active": {"x0": mm(69.75, 0)[0], "y0": mm(0, 150)[1], "x1": mm(229.75, 0)[0], "y1": mm(0, 230)[1]},
}

ENCODER_ROW_Y = [mm(0, y)[1] for y in (182.0, 375.5, 570.0)]  # knob cap centres
ENCODER_LED_Y = [mm(0, y)[1] for y in (253.4, 444.0, 635.5)]
ENCODER = {"cap_d": mm_len(70), "led_w": mm_len(21), "led_h": mm_len(9)}

FADER = {
    "slot_w": mm_len(17),
    "slot_y0": mm(0, 738)[1],
    "slot_y1": mm(0, 1170)[1],
    # Cap travel: centre line from the top tick to the bottom tick, 60 mm throw (user guide v1).
    "travel_y0": mm(0, 763)[1],
    "travel_y1": mm(0, 1146)[1],
    "cap_w": mm_len(113),
    "cap_h": mm_len(54),
}
# Scale ticks: 7 sets, centred between adjacent faders, 9 ticks each, long/short alternating.
FADER_TICKS = {
    "y": [mm(0, 763 + 47.875 * i)[1] for i in range(9)],
    "long": mm_len(38),
    "short": mm_len(25),
    "x": [round((COLUMN_X[c] + COLUMN_X[c + 1]) / 2, 2) for c in range(7)],
}

# Button strips (two rows of 10 segments: row button, Novation/blank, buttons 1..8).
STRIP = {
    "x0": mm(71, 0)[0],
    "x1": mm(1528, 0)[0],
    "rows": [
        {"y0": mm(0, 1259)[1], "y1": mm(0, 1337)[1]},
        {"y0": mm(0, 1381)[1], "y1": mm(0, 1457)[1]},
    ],
    "r": mm_len(9),
    "dividers_x": [mm(226, 0)[0], mm(301, 0)[0]] + [mm(301 + 153 * k, 0)[0] for k in range(1, 8)],
    "label_seg": {"x0": mm(71, 0)[0], "x1": mm(226, 0)[0]},
    "logo_seg": {"x0": mm(226, 0)[0], "x1": mm(301, 0)[0]},
    "button_x0": mm(301, 0)[0],
    "button_w": COLUMN_PITCH,
    "led_w": mm_len(59),
    "led_h": mm_len(7),
    "caption_y": [mm(0, 1358)[1], mm(0, 1478)[1]],
}

SIDE_X0, SIDE_X1 = mm(71, 0)[0], mm(227.5, 0)[0]
SIDE = {
    # Paired buttons share one housing split by a divider.
    "page": {"y0": mm(0, 341)[1], "y1": mm(0, 420)[1], "split": "v", "label": ("Page", mm(150, 323.5))},
    "track": {"y0": mm(0, 532.5)[1], "y1": mm(0, 611.5)[1], "split": "v", "label": ("Track", mm(150, 514)),
              "sublabel": ("Settings", mm(150, 631))},
    "transport": {"y0": mm(0, 745)[1], "y1": mm(0, 900)[1], "split": "h", "split_y": mm(0, 822.5)[1]},
    "shiftmode": {"y0": mm(0, 1009)[1], "y1": mm(0, 1165)[1], "split": "h", "split_y": mm(0, 1085)[1],
                  "label": ("Edit", mm(150, 1187))},
}

# Every control with its kind, hit area, and LED (where the light appears on the part).
# led: None = no LED. Side-button LED colours are set by the surface as RGB (crates/lcxl3); whether
# the hardware shows full RGB on every side button is UNVERIFIED.
def controls():
    c = []
    for r in range(3):
        for col in range(8):
            c.append({"key": f"K{r + 1}{col + 1}", "kind": "encoder", "x": COLUMN_X[col], "y": ENCODER_ROW_Y[r],
                      "led": {"shape": "bar", "x": COLUMN_X[col], "y": ENCODER_LED_Y[r]}})
    for col in range(8):
        c.append({"key": f"F{col + 1}", "kind": "fader", "x": COLUMN_X[col], "led": None})
    for row, prefix in enumerate("TB"):
        y0, y1 = STRIP["rows"][row]["y0"], STRIP["rows"][row]["y1"]
        for col in range(8):
            x0 = round(STRIP["button_x0"] + STRIP["button_w"] * col, 2)
            c.append({"key": f"{prefix}{col + 1}", "kind": "button", "x0": x0, "x1": round(x0 + STRIP["button_w"], 2),
                      "y0": y0, "y1": y1, "led": {"shape": "bar", "x": COLUMN_X[col], "y": round((y0 + y1) / 2, 2)}})
    s0, s1 = STRIP["rows"][0], STRIP["rows"][1]
    seg = STRIP["label_seg"]
    c.append({"key": "SOLO", "kind": "button", "x0": seg["x0"], "x1": seg["x1"], "y0": s0["y0"], "y1": s0["y1"],
              "text": "Solo / Arm", "led": {"shape": "text"}})
    c.append({"key": "MUTE", "kind": "button", "x0": seg["x0"], "x1": seg["x1"], "y0": s1["y0"], "y1": s1["y1"],
              "text": "Mute / Select", "led": {"shape": "text"}})
    lg = STRIP["logo_seg"]
    c.append({"key": "LOGO", "kind": "button", "x0": lg["x0"], "x1": lg["x1"], "y0": s0["y0"], "y1": s0["y1"],
              "led": {"shape": "logo"}})
    p, t, tr, sm = SIDE["page"], SIDE["track"], SIDE["transport"], SIDE["shiftmode"]
    mid = round((SIDE_X0 + SIDE_X1) / 2, 2)
    c += [
        {"key": "PAGE_UP", "kind": "button", "x0": SIDE_X0, "x1": mid, "y0": p["y0"], "y1": p["y1"], "led": {"shape": "chevron_up"}},
        {"key": "PAGE_DOWN", "kind": "button", "x0": mid, "x1": SIDE_X1, "y0": p["y0"], "y1": p["y1"], "led": {"shape": "chevron_down"}},
        {"key": "TRACK_LEFT", "kind": "button", "x0": SIDE_X0, "x1": mid, "y0": t["y0"], "y1": t["y1"], "led": {"shape": "chevron_left"}},
        {"key": "TRACK_RIGHT", "kind": "button", "x0": mid, "x1": SIDE_X1, "y0": t["y0"], "y1": t["y1"], "led": {"shape": "chevron_right"}},
        {"key": "REC", "kind": "button", "x0": SIDE_X0, "x1": SIDE_X1, "y0": tr["y0"], "y1": tr["split_y"], "led": {"shape": "dot"}},
        {"key": "PLAY", "kind": "button", "x0": SIDE_X0, "x1": SIDE_X1, "y0": tr["split_y"], "y1": tr["y1"], "led": {"shape": "triangle"}},
        {"key": "SHIFT", "kind": "button", "x0": SIDE_X0, "x1": SIDE_X1, "y0": sm["y0"], "y1": sm["split_y"], "text": "Shift", "led": None},
        {"key": "MODE", "kind": "button", "x0": SIDE_X0, "x1": SIDE_X1, "y0": sm["split_y"], "y1": sm["y1"], "text": "Mode", "led": None},
    ]
    return c


# Icon geometry (render px converted to mm).
ICONS = {
    "chevron_up": {"c": mm(110, 381.5), "w": mm_len(27), "h": mm_len(14)},
    "chevron_down": {"c": mm(189, 381.5), "w": mm_len(27), "h": mm_len(14)},
    "chevron_left": {"c": mm(109, 573), "w": mm_len(14), "h": mm_len(27)},
    "chevron_right": {"c": mm(189, 573), "w": mm_len(14), "h": mm_len(27)},
    "dot": {"c": mm(148.5, 782.6), "d": mm_len(26)},
    "triangle": {"x0": mm(137.5, 0)[0], "x1": mm(159, 0)[0], "y0": mm(0, 851)[1], "y1": mm(0, 875)[1]},
    "shift_text": mm(150, 1050),
    "mode_text": mm(150, 1127),
}
