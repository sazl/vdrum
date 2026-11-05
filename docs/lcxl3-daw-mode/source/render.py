import sys
from PIL import Image, ImageDraw, ImageFont, ImageEnhance

sys.path.insert(0, sys.argv[1])
from layout import MODES, SECTIONS, MODE_NAMES, COLUMN_TITLES, MODE_COLS, DAW_CC, VOICE_COLOURS, PERF_VOICE_OF_COL, FILE_NAMES, COLOURED_BY_TRACK

SRC = sys.argv[2]
OUT = sys.argv[3]

F_B = "/usr/share/fonts/opentype/inter/Inter-Bold.otf"
F_R = "/usr/share/fonts/opentype/inter/Inter-Regular.otf"
F_M = "/usr/share/fonts/truetype/dejavu/DejaVuSansMono.ttf"


def font(p, s):
    return ImageFont.truetype(p, s)


COLX = [378 + 153.3 * i for i in range(8)]
ROWY = {1: 185, 2: 378, 3: 572}
FADER_Y = 955
TOP_Y, BOT_Y = 1300, 1420
SIDE = {
    "PAGE": (149, 380, 156, 78), "TRACK": (149, 572, 156, 78),
    "REC": (149, 783, 156, 74), "PLAY": (149, 862, 156, 74),
    "SHIFT": (149, 1050, 156, 74), "MODE": (149, 1127, 156, 74),
    "SOLO": (149, 1300, 150, 64), "MUTE": (149, 1420, 150, 64),
    "LOGO": (263, 1300, 70, 64),
}
HEADER = 120
FOOTER = 190
W = 146
H = 66


def text_colour(rgb):
    r, g, b = rgb
    return (20, 20, 24) if (0.299 * r + 0.587 * g + 0.114 * b) > 140 else (245, 245, 245)


def fit(draw, text, path, size, maxw):
    while size > 9:
        f = font(path, size)
        if draw.textlength(text, font=f) <= maxw:
            return f
        size -= 1
    return font(path, size)


def box(draw, cx, cy, w, h, entry, fill, kind_tag=True):
    label, detail, section, kind = entry
    x0, y0, x1, y1 = cx - w / 2, cy - h / 2, cx + w / 2, cy + h / 2
    outline = (255, 255, 255) if kind in ("cycle", "toggle", "hold", "press") else None
    draw.rounded_rectangle((x0, y0, x1, y1), radius=9, fill=fill + (238,),
                           outline=outline, width=2 if outline else 0)
    tc = text_colour(fill)
    f1 = fit(draw, label, F_B, 19 if w > 100 else 14, w - 10)
    sub = detail
    if kind in ("cycle", "toggle", "hold", "press"):
        sub = {"cycle": "cycle", "toggle": "on/off", "hold": "hold", "press": "press"}[kind]
        if detail and len(detail) <= 13:
            sub += f" | {detail}"
    f2 = fit(draw, sub, F_M, 12 if w > 100 else 10, w - 10)
    if sub:
        draw.text((cx, cy - 10), label, font=f1, fill=tc, anchor="mm")
        draw.text((cx, cy + 15), sub, font=f2, fill=tc, anchor="mm")
    else:
        draw.text((cx, cy), label, font=f1, fill=tc, anchor="mm")


def colour_for(mode, key, entry):
    section = entry[2]
    if section in COLOURED_BY_TRACK and key[0] in "KFTB" and key[-1].isdigit():
        return VOICE_COLOURS[PERF_VOICE_OF_COL[int(key[-1])]]
    return SECTIONS[section][1]


def render(mode):
    m = MODES[mode]
    base = Image.open(SRC).convert("RGBA")
    base = ImageEnhance.Brightness(base).enhance(0.42)
    bw, bh = base.size
    img = Image.new("RGBA", (bw, bh + HEADER + FOOTER), (24, 25, 29, 255))
    img.paste(base, (0, HEADER))
    ov = Image.new("RGBA", img.size, (0, 0, 0, 0))
    d = ImageDraw.Draw(ov)
    oy = HEADER

    # header
    num = 8 if mode == "8h" else mode
    d.text((40, 30), f"Mode {num}: {MODE_NAMES[mode]}", font=font(F_B, 44), fill=(245, 245, 245))
    d.text((40, 84), "VDrum on Launch Control XL 3 (DAW mode), proposed mapping. Not implemented or tested on hardware.",
           font=font(F_R, 20), fill=(185, 190, 200))

    # column titles (over the LAUNCHCONTROL logo strip, just above the knob row)
    titles = COLUMN_TITLES[MODE_COLS[mode]]
    for c, t in enumerate(titles):
        d.rounded_rectangle((COLX[c] - 70, oy + 98, COLX[c] + 70, oy + 126), radius=6, fill=(12, 12, 14, 230))
        d.text((COLX[c], oy + 112), t, font=font(F_B, 16), fill=(235, 235, 235), anchor="mm")

    for key, entry in m.items():
        fill = colour_for(mode, key, entry)
        if key[0] == "K":
            r, c = int(key[1]), int(key[2])
            box(d, COLX[c - 1], oy + ROWY[r], W, H, entry, fill)
            d.text((COLX[c - 1] + W / 2 - 4, oy + ROWY[r] + H / 2 + 12), f"CC {DAW_CC[key]}",
                   font=font(F_M, 11), fill=(170, 175, 185), anchor="rm")
        elif key[0] == "F" and key[1:].isdigit():
            c = int(key[1:])
            box(d, COLX[c - 1], oy + FADER_Y, W, 84, entry, fill)
            d.text((COLX[c - 1], oy + FADER_Y + 56), f"CC {DAW_CC[key]}", font=font(F_M, 11),
                   fill=(170, 175, 185), anchor="mm")
        elif key[0] in "TB" and key[1:].isdigit():
            c = int(key[1:])
            y = TOP_Y if key[0] == "T" else BOT_Y
            box(d, COLX[c - 1], oy + y, W, 62, entry, fill)
            d.text((COLX[c - 1], oy + y + 42), f"CC {DAW_CC[key]}", font=font(F_M, 11),
                   fill=(170, 175, 185), anchor="mm")
        elif key in SIDE:
            cx, cy, w, h = SIDE[key]
            box(d, cx, oy + cy, w, h, entry, fill)

    # screen mock
    d.rounded_rectangle((62, oy + 137, 238, oy + 243), radius=6, fill=(0, 0, 0, 255), outline=(90, 90, 90), width=2)
    lines = {
        10: ("10 PERFORM", "Kit: Initkit", "SD Decay  64"),
        16: ("16 GLOBAL", "D1 note", "C1 (36)"),
        9: ("9 MIX + FX", "FX type", "del"),
        8: ("8 SEQ  P01 B1", "Track 1 D1", "SD Fills 4/16"),
        "8h": ("8 SEQ  step 5", "Lock Cutoff", "D1 64 > 90"),
        11: ("11 LFO + MOD", "SD LFO dest", "SD Coarse"),
        12: ("12 MORPH", "D1 Morph > S2", "50%"),
    }.get(mode, (f"{mode} {MODE_NAMES[mode].upper()}", "Filter: Cutoff", "127"))
    for i, t in enumerate(lines):
        d.text((150, oy + 160 + i * 30), t, font=fit(d, t, F_M, 17, 165), fill=(235, 235, 235), anchor="mm")

    # footer legend
    fy = bh + HEADER + 18
    used = []
    for e in m.values():
        if e[2] not in used:
            used.append(e[2])
    track_coloured = any(u in COLOURED_BY_TRACK for u in used)
    used = [u for u in used if u not in COLOURED_BY_TRACK]
    x = 40
    for s in used:
        name, rgb = SECTIONS[s]
        tw = d.textlength(name, font=font(F_R, 18))
        if x + tw + 40 > bw - 30:
            x = 40
            fy += 34
        d.rounded_rectangle((x, fy, x + 22, fy + 22), radius=4, fill=rgb + (255,))
        d.text((x + 30, fy + 11), name, font=font(F_R, 18), fill=(220, 222, 228), anchor="lm")
        x += tw + 64
    if track_coloured:
        for tag, rgb in [(t, c) for t, c in VOICE_COLOURS.items() if t != "FX"]:
            tw = d.textlength(tag, font=font(F_R, 18))
            if x + tw + 40 > bw - 30:
                x = 40
                fy += 34
            d.rounded_rectangle((x, fy, x + 22, fy + 22), radius=4, fill=rgb + (255,))
            d.text((x + 30, fy + 11), tag, font=font(F_R, 18), fill=(220, 222, 228), anchor="lm")
            x += tw + 64
    fy += 44
    notes = ("White outline = button. cycle: steps through a list. on/off: latching toggle. "
             "hold: active while held. Shift + button 1-16 selects mode 1-16. Encoders are relative;")
    notes2 = ("faders use pickup after a mode change. CC = DAW-mode CC from Novation's programmer's reference v1.0 "
              "(encoders/faders ch 16, buttons ch 1).")
    d.text((40, fy), notes, font=font(F_R, 17), fill=(170, 175, 185))
    d.text((40, fy + 26), notes2, font=font(F_R, 17), fill=(170, 175, 185))

    out = Image.alpha_composite(img, ov).convert("RGB")
    num = 8 if mode == "8h" else mode
    out.save(f"{OUT}/mode-{num:02d}-{FILE_NAMES[mode]}.png", optimize=True)


for mode in MODES:
    render(mode)
