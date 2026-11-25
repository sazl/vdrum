"""Generate Prism-style HTML mockups for the three consolidation UX proposals."""
import math, random, sys, os

OUT = sys.argv[1]
FONTS = sys.argv[2]
random.seed(7)

CSS = f"""
@font-face {{ font-family: BCond; src: url('{FONTS}/BarlowCondensed-SemiBold.ttf'); font-weight: 600; }}
@font-face {{ font-family: BCond; src: url('{FONTS}/BarlowCondensed-Bold.ttf'); font-weight: 700; }}
@font-face {{ font-family: BSemi; src: url('{FONTS}/BarlowSemiCondensed-Regular.ttf'); font-weight: 400; }}
@font-face {{ font-family: BSemi; src: url('{FONTS}/BarlowSemiCondensed-SemiBold.ttf'); font-weight: 600; }}
:root {{ --well:#020108; --bg:#06050F; --raised:#100D26; --raised2:#15122E; --hover:#1B1838;
  --rule:#2B2850; --edge:#4A4680; --control:#68638A; --text:#DDDCEB; --muted:#A4A1BD; --disabled:#7F7AA6;
  --blue:#6AB1F3; --green:#65D097; --yellow:#E3CA69; --amber:#EFA056; --red:#E66E68; --bluetint:#1F3048; }}
* {{ box-sizing: border-box; margin: 0; padding: 0; }}
body {{ background: var(--bg); color: var(--text); font-family: BSemi, sans-serif; font-size: 15px; width: 1440px; height: 900px; overflow: hidden; position: relative; }}
.bar {{ height: 56px; display: flex; align-items: center; gap: 8px; padding: 0 20px; border-bottom: 1px solid var(--rule); }}
.title {{ font-family: BCond; font-weight: 700; font-size: 26px; letter-spacing: .5px; margin-right: 12px; }}
.btn {{ height: 32px; min-width: 32px; padding: 0 10px; display: inline-flex; align-items: center; justify-content: center; font-size: 13px; border: 1px solid var(--control); background: var(--bg); color: var(--text); white-space: nowrap; }}
.btn.ghost {{ background: var(--raised2); border: none; border-bottom: 2px solid var(--rule); }}
.btn.prim {{ background: var(--green); color: var(--bg); border: none; font-family: BCond; font-weight: 700; }}
.btn.seg {{ background: var(--raised2); }}
.btn.seg.on {{ background: var(--yellow); color: var(--bg); border: none; font-family: BCond; font-weight: 700; }}
.btn.tog.on {{ background: var(--blue); color: var(--bg); border: none; font-family: BCond; font-weight: 700; }}
.btn.sel {{ justify-content: space-between; gap: 10px; }}
.btn.sel::after {{ content: '▾'; color: var(--muted); }}
.btn.dis {{ color: var(--disabled); border-color: var(--edge); }}
.spacer {{ flex: 1; }}
.muted {{ color: var(--muted); }} .small {{ font-size: 13px; }} .label {{ font-size: 12px; color: var(--muted); }}
.head {{ font-family: BCond; font-weight: 600; font-size: 20px; }}
.cap {{ font-family: BCond; font-weight: 700; font-size: 13px; letter-spacing: 1px; color: var(--muted); text-transform: uppercase; }}
.knob {{ display: flex; flex-direction: column; align-items: center; width: 64px; height: 68px; }}
.knob svg {{ display: block; }}
.knob .kl {{ font-size: 12px; color: var(--muted); margin-top: 1px; white-space: nowrap; }}
.knob .kl.sel {{ color: var(--blue); border-bottom: 2px solid var(--blue); }}
.knob .kv {{ font-size: 13px; }}
.list {{ display: flex; flex-direction: column; width: 64px; align-items: center; height: 68px; justify-content: center; }}
.list .btn {{ height: 26px; width: 60px; padding: 0 4px; font-size: 12px; margin-top: 6px; }}
.step {{ width: 36px; height: 30px; background: var(--raised2); border: 1px solid var(--rule); position: relative; }}
.step.on {{ background: var(--blue); border-color: var(--blue); }}
.step.off {{ background: var(--well); border-color: var(--rule); opacity: .5; }}
.step.cur {{ outline: 2px solid var(--text); outline-offset: -2px; }}
.step .lv {{ position: absolute; left: 3px; right: 3px; bottom: 3px; background: var(--green); }}
.row {{ display: flex; align-items: center; gap: 4px; }}
.beat {{ width: 6px; }}
.num {{ width: 26px; height: 30px; display: flex; align-items: center; justify-content: center; font-family: BCond; font-weight: 700; font-size: 18px; }}
.hit {{ width: 6px; height: 6px; background: var(--green); display: inline-block; }}
.tag {{ font-family: BCond; font-weight: 700; font-size: 11px; letter-spacing: 1px; color: var(--muted); border: 1px solid var(--edge); padding: 0 4px; height: 18px; display: inline-flex; align-items: center; }}
.rule {{ border-top: 1px solid var(--rule); }}
.panel {{ background: var(--raised); border: 1px solid var(--rule); }}
.status {{ position: absolute; left: 20px; right: 20px; bottom: 0; height: 30px; display: flex; align-items: center; font-size: 13px; color: var(--muted); border-top: 1px solid var(--rule); }}
.annot {{ position: absolute; background: var(--amber); color: var(--bg); font-family: BCond; font-weight: 700; font-size: 14px; width: 22px; height: 22px; display: flex; align-items: center; justify-content: center; border-radius: 11px; z-index: 5; }}
"""

def knob(label, val, norm, sel=False, size=34):
    r = size / 2 - 2
    c = size / 2 + 1
    a0 = math.radians(135); sweep = math.radians(270)
    def pt(a): return (c + r * math.cos(a), c + r * math.sin(a))
    def arc(a1, a2):
        x1, y1 = pt(a1); x2, y2 = pt(a2)
        large = 1 if (a2 - a1) > math.pi else 0
        return f"M{x1:.1f},{y1:.1f} A{r},{r} 0 {large} 1 {x2:.1f},{y2:.1f}"
    a = a0 + sweep * norm
    px1 = c + 3 * math.cos(a); py1 = c + 3 * math.sin(a)
    px2 = c + (r - 4) * math.cos(a); py2 = c + (r - 4) * math.sin(a)
    fill = f'<path d="{arc(a0, a)}" stroke="#65D097" stroke-width="4" fill="none"/>' if norm > 0.01 else ''
    s = size + 2
    return (f'<div class="knob"><svg width="{s}" height="{s}"><path d="{arc(a0, a0 + sweep)}" stroke="#15122E" stroke-width="4" fill="none"/>'
            f'{fill}<line x1="{px1:.1f}" y1="{py1:.1f}" x2="{px2:.1f}" y2="{py2:.1f}" stroke="#A4A1BD" stroke-width="2"/></svg>'
            f'<div class="kl{" sel" if sel else ""}">{label}</div><div class="kv">{val}</div></div>')

def lst(label, val):
    return f'<div class="list"><div class="kl label">{label}</div><div class="btn sel">{val}</div></div>'

def rk(label, val=None):
    n = random.random()
    if val is None:
        val = str(int(n * 127))
    return knob(label, val, n)

def steps(n_on, length=16, cur=None, lane=False, pattern=None):
    out = []
    pat = pattern or [random.random() < n_on for _ in range(16)]
    for s in range(16):
        if s and s % 4 == 0:
            out.append('<div class="beat"></div>')
        cls = 'step'
        inner = ''
        if s >= length:
            cls += ' off'
        elif lane:
            inner = f'<div class="lv" style="height:{int(random.random()*24)}px"></div>'
        elif pat[s]:
            cls += ' on'
        if cur is not None and s == cur:
            cls += ' cur'
        out.append(f'<div class="{cls}">{inner}</div>')
    return ''.join(out)

def track_tail(length, speed):
    return (f'<div class="btn" style="margin-left:10px">−</div><div class="btn ghost" style="width:34px">{length}</div>'
            f'<div class="btn">+</div><div class="btn sel" style="width:70px">{speed}</div>'
            f'<div class="btn" title="randomize">⚄</div><div class="btn">✕</div>')

PATTERNS = 'ABCDEFGHIJKLMNOP'
def bank(sel=0, w=26):
    return ''.join(f'<div class="btn seg{" on" if i == sel else ""}" style="min-width:{w}px;width:{w}px;padding:0">{c}</div>' for i, c in enumerate(PATTERNS))

def topbar(extra=''):
    return f'''<div class="bar"><div class="title">FM DRUM <span class="muted" style="font-size:15px;font-family:BSemi;font-weight:400">(name TBD)</span></div>
<div class="btn sel" style="width:200px">Starter Kit</div><div class="btn ghost">Save</div>{extra}
<div class="spacer"></div><span class="label">Tempo</span><div class="btn ghost" style="width:70px">120.0</div><div class="btn prim" style="width:56px">RUN</div>
<span class="label" style="margin-left:10px">MTS-ESP: off</span><div class="btn" style="margin-left:10px">−</div><span class="small">100 %</span><div class="btn">+</div></div>'''

def page(name, body):
    html = f'<!doctype html><html><head><meta charset="utf-8"><title>{name}</title><style>{CSS}</style></head><body>{body}</body></html>'
    with open(os.path.join(OUT, name + '.html'), 'w') as f:
        f.write(html)

def annot(n, x, y):
    return f'<div class="annot" style="left:{x}px;top:{y}px">{n}</div>'

# ------------------------------------------------------------------------------------------
# A: Engine banks

LXR = ['Drum 1', 'Drum 2', 'Drum 3', 'Snare', 'Cymbal', 'Hat cl.', 'Hat op.']
def lxr_osc_knobs(i):
    if i < 3:
        return [lst('Wave', ['Sin', 'Tri', 'Sin'][i]), rk('Coarse'), rk('Fine', '+0'), rk('Pulse W'), lst('FM Mode', 'FM'),
                lst('FM Wave', 'Sin'), rk('FM Amt'), rk('FM Freq')]
    if i == 3:
        return [lst('Wave', 'Tri'), rk('Coarse'), rk('Fine', '+1'), rk('Pulse W'), rk('Noise Fq'), rk('Mix')]
    return [lst('Wave 1', 'Noi'), rk('Coarse'), rk('Fine', '+0'), rk('Pulse W'), lst('Wave 2', 'Sin'), lst('Wave 3', 'Sqr'),
            rk('Mod F 1'), rk('Mod F 2'), rk('Mod G 1'), rk('Mod G 2')]

def voice_col(n, name, knobs, width, hit=False, m=False, s=False, sub=''):
    ks = ''.join(knobs)
    return f'''<div style="width:{width}px;border-right:1px solid var(--rule);padding:8px 6px 0 8px">
<div class="row" style="gap:6px"><div class="num" style="width:20px">{n}</div><div style="flex:1;line-height:1.05"><div class="small" style="font-weight:600">{name}</div><div class="label">{sub}</div></div>
{'<span class="hit"></span>' if hit else ''}<div class="btn{' tog on' if m else ''}" style="width:26px;min-width:26px;height:26px;padding:0">M</div><div class="btn{' tog on' if s else ''}" style="width:26px;min-width:26px;height:26px;padding:0">S</div></div>
<div class="rule" style="margin:8px 0 6px -8px;margin-right:-6px"></div>
<div style="display:grid;grid-template-columns:repeat(2,64px);gap:4px 10px;justify-content:center">{ks}</div></div>'''

def design_a():
    sections = ['Osc + FM', 'Amp EG', 'Pitch EG', 'Filter', 'Transient', 'LFO', 'Velocity', 'Mix']
    body = topbar()
    body += '<div class="row" style="padding:10px 20px 0 20px;gap:6px">'
    for i, (b, n) in enumerate([('LXR', 7), ('NANO', 6), ('HEXA', 6)]):
        hits = '<span class="hit" style="margin-left:6px"></span>' if i != 2 else ''
        body += f'<div class="btn seg{" on" if i == 0 else ""}" style="width:120px">{b} · {n}{hits}</div>'
    body += '<div style="width:24px"></div><span class="label">Section</span>'
    for i, s in enumerate(sections):
        body += f'<div class="btn seg{" on" if i == 0 else ""}">{s}</div>'
    body += '</div>'
    # voice columns + FX column
    body += '<div style="display:flex;margin:10px 20px 0 20px;border:1px solid var(--rule);height:420px;overflow:hidden">'
    for i, nme in enumerate(LXR):
        body += voice_col(i + 1, nme, lxr_osc_knobs(min(i, 5)), 152, hit=i in (0, 3, 5), sub=['ch 1 · C1', 'ch 1 · A1', 'ch 1 · D2', 'ch 1 · D1', 'ch 1 · C#2', 'ch 1 · F#1', 'ch 1 · A#1'][i])
    fx = (f'<div style="flex:1;padding:8px 10px" class="panel"><div class="cap">LXR FX bus</div>'
          f'<div class="row" style="margin-top:8px;gap:2px">' + ''.join(f'<div class="btn seg{" on" if j == 4 else ""}" style="padding:0 5px">{t}</div>' for j, t in enumerate(['off', 'drv', 'rng', 'cmp', 'del'])) + '</div>'
          f'<div style="display:grid;grid-template-columns:repeat(2,64px);gap:6px 12px;margin-top:10px">{lst("Type", "Pp")}{lst("Out", "St1")}{rk("Dry/Wet", "35")}{rk("P1")}{rk("P2")}{rk("P3")}{rk("P4")}<div></div></div>'
          f'<div class="rule" style="margin:10px 0"></div><div class="cap">Output</div><div class="label" style="margin-top:6px">Main ▮▮▮▮▮▯▯ −6 dB<br>Aux ▮▮▯▯▯▯▯ −18 dB</div>'
          f'<div class="btn" style="margin-top:8px;width:100%">Kit library…</div></div>')
    body += fx + '</div>'
    # sequencer
    body += f'''<div class="row" style="margin:12px 20px 0 20px;gap:6px"><div class="btn sel" style="width:150px">Gates</div>
<span class="label" style="margin-left:10px">Pattern</span>{bank(0)}<div class="spacer"></div>
<div class="btn tog">All 19 tracks</div><div class="btn ghost">Export MIDI</div></div>'''
    body += '<div style="margin:8px 20px 0 20px">'
    for i, nme in enumerate(LXR):
        body += f'<div class="row" style="margin-bottom:4px"><div class="num">{i+1}</div><div class="small" style="width:84px">{nme}</div>{steps(0.3, length=16 if i != 2 else 12, cur=5)}{track_tail(16 if i != 2 else 12, "x4")}</div>'
    body += '</div><div class="status">LXR bank: 7 voices, 1 FX bus. Section applies to every column of the bank. hxfm bank shows its 10 knobs; nano-thump bank sections are Layer 1, Layer 2, Part.</div>'
    body += annot(1, 12, 64) + annot(2, 360, 64) + annot(3, 1210, 120) + annot(4, 1232, 545)
    page('design-a-engine-banks', body)

# ------------------------------------------------------------------------------------------
# B: Track slots + inspector

SLOTS = [('LXR', 'Drum 1'), ('HX', 'Voice 1'), ('LXR', 'Snare'), ('NT', 'Part 2'), ('HX', 'Voice 3'), ('LXR', 'Hat cl.'),
         ('LXR', 'Hat op.'), ('NT', 'Part 5'), ('HX', 'Voice 4'), ('LXR', 'Cymbal'), ('NT', 'Part 1'), ('—', 'empty')]

def design_b():
    body = topbar()
    body += f'''<div class="row" style="margin:10px 20px 0 20px;gap:6px"><div class="btn sel" style="width:150px">Gates</div>
<span class="label" style="margin-left:10px">Pattern</span>{bank(2)}<div class="spacer"></div><div class="btn ghost">Export MIDI</div></div>'''
    body += '<div style="display:flex;margin:8px 20px 0 20px;gap:14px"><div>'
    for i, (eng, v) in enumerate(SLOTS):
        selected = i == 3
        bg = 'background:var(--bluetint);' if selected else ''
        empty = eng == '—'
        chip = f'<span class="tag" style="width:34px;justify-content:center">{eng}</span>'
        name = f'<div class="btn sel{" dis" if empty else ""}" style="width:128px;height:28px;font-size:13px">{v}</div>'
        body += (f'<div class="row" style="margin-bottom:3px;{bg}padding:1px 4px"><div class="num">{i+1}</div>{chip}{name}'
                 f'<div class="btn" style="width:26px;min-width:26px;height:26px;padding:0">M</div><div class="btn" style="width:26px;min-width:26px;height:26px;padding:0">S</div>'
                 f'<div style="width:6px"></div>{steps(0 if empty else 0.3, length=16, cur=None if empty else 9)}{track_tail(16, "x4")}</div>')
    body += '</div>'
    # voice pool
    pool = [('LXR', ['Drum 1', 'Drum 2', 'Drum 3', 'Snare', 'Cymbal', 'Hat cl', 'Hat op']), ('NT', ['Part 1', 'Part 2', 'Part 3', 'Part 4', 'Part 5', 'Part 6']), ('HX', ['Voice 1', 'Voice 2', 'Voice 3', 'Voice 4', 'Voice 5', 'Voice 6'])]
    used = {('LXR', 'Drum 1'), ('LXR', 'Snare'), ('LXR', 'Hat cl'), ('LXR', 'Hat op'), ('LXR', 'Cymbal'), ('NT', 'Part 2'), ('NT', 'Part 5'), ('NT', 'Part 1'), ('HX', 'Voice 1'), ('HX', 'Voice 3'), ('HX', 'Voice 4')}
    body += '<div class="panel" style="flex:1;padding:8px 10px;height:438px"><div class="cap">Voice pool</div><div class="label" style="margin:4px 0 8px">Each voice can be on one slot. Grey = in use.</div>'
    for eng, vs in pool:
        body += f'<div class="label" style="margin-top:6px">{eng}</div><div style="display:flex;flex-wrap:wrap;gap:4px;margin-top:4px">'
        for v in vs:
            u = (eng, v) in used
            body += f'<div class="btn{" dis" if u else ""}" style="height:26px;font-size:12px;padding:0 6px">{v}</div>'
        body += '</div>'
    body += '<div class="rule" style="margin:12px 0 8px"></div><div class="cap">Engine FX</div><div class="label" style="margin-top:4px">LXR bus: del · NT: reverb · HX: none</div></div></div>'
    # inspector
    body += '<div style="margin:8px 20px 0 20px;border:1px solid var(--rule);height:300px;display:flex">'
    body += '<div style="width:180px;padding:10px;border-right:1px solid var(--rule)"><div class="cap">Track 4</div><div class="head" style="margin-top:2px">NT · Part 2</div>'
    body += '<div class="label" style="margin-top:10px">MIDI</div><div class="row" style="margin-top:4px"><div class="btn sel" style="width:70px">Ch 1</div><div class="btn sel" style="width:70px">D1</div></div>'
    body += '<div class="label" style="margin-top:10px">Choke group</div><div class="btn sel" style="width:144px;margin-top:4px">None</div>'
    body += '<div class="label" style="margin-top:10px">Output</div><div class="btn sel" style="width:144px;margin-top:4px">Mix</div>'
    body += '<div class="label" style="margin-top:10px">Lanes in use</div><div class="small" style="margin-top:2px">Velocity, L1 Pitch</div></div>'
    def card(title, knobs, w):
        return f'<div style="width:{w}px;padding:10px;border-right:1px solid var(--rule)"><div class="cap">{title}</div><div style="display:grid;grid-template-columns:repeat(4,64px);gap:6px 8px;margin-top:8px">{"".join(knobs)}</div></div>'
    l1 = [lst('Carrier', 'Sine'), lst('Mod Osc', 'Osc 2'), lst('Env', 'Env 1'), rk('Level'), rk('Mod Amt', '+12'), rk('Mod Rate'), knob('Pitch', '64', .5, sel=True), rk('Attack'), rk('Release')]
    l2 = [lst('Carrier', 'Noise LP'), lst('Mod Osc', 'Osc 3'), lst('Env', 'Env 2'), rk('Level'), rk('Mod Amt', '−8'), rk('Mod Rate'), rk('Pitch'), rk('Attack'), rk('Release')]
    pt = [rk('FX Send'), rk('Bit Red'), rk('Fold'), rk('Drive'), rk('Pan', 'C'), rk('Level', '+0'), lst('Pitch', 'Cont.'), '<div class="list"><div class="kl label">Link</div><div class="btn tog on" style="height:26px;width:60px;margin-top:6px">L1+L2</div></div>']
    body += card('Layer 1', l1, 310) + card('Layer 2', l2, 310) + card('Part', pt, 310)
    body += '<div style="flex:1;padding:10px" class="panel"><div class="cap">NT FX (shared)</div><div class="row" style="margin-top:8px;gap:2px"><div class="btn seg on">Reverb</div><div class="btn seg">Delay</div></div><div style="display:grid;grid-template-columns:repeat(2,64px);gap:6px 8px;margin-top:8px">' + rk('Amount') + rk('Tone') + rk('Time') + '</div></div>'
    body += '</div><div class="status">Selected track: 4. The inspector shows the assigned voice\'s parameters grouped by its engine\'s sections. Right-click a knob to open its lane.</div>'
    body += annot(1, 2, 100) + annot(2, 1180, 100) + annot(3, 20, 556) + annot(4, 1240, 556)
    page('design-b-track-slots', body)

# ------------------------------------------------------------------------------------------
# C: Function pages

PAGES = ['Sequence', 'Sound', 'Modulation', 'FX', 'Mix', 'Library']
def page_tabs(sel):
    return '<div class="row" style="padding:10px 20px 0 20px;gap:6px">' + ''.join(f'<div class="btn seg{" on" if i == sel else ""}" style="width:118px">{p}</div>' for i, p in enumerate(PAGES)) + '</div>'

ALL = [('LXR', n) for n in LXR] + [('NT', f'Part {i}') for i in range(1, 7)] + [('HX', f'Voice {i}') for i in range(1, 7)]

def design_c_seq():
    body = topbar() + page_tabs(0)
    body += f'''<div class="row" style="margin:10px 20px 0 20px;gap:6px"><div class="btn sel" style="width:150px">Gates</div>
<span class="label" style="margin-left:10px">Pattern</span>{bank(0)}<div class="spacer"></div><div class="btn tog on">Group</div><div class="btn ghost">Export MIDI</div></div>'''
    body += '<div style="margin:8px 20px 0 20px">'
    prev = None
    for i, (eng, n) in enumerate(ALL):
        if eng != prev and prev is not None:
            body += '<div class="rule" style="margin:3px 0 3px 0"></div>'
        prev = eng
        body += (f'<div class="row" style="margin-bottom:2px;height:28px"><div class="num" style="height:26px;font-size:15px">{i+1}</div><span class="tag" style="width:34px;justify-content:center">{eng}</span>'
                 f'<div class="small" style="width:84px;margin-left:4px">{n}</div><span class="hit" style="visibility:{"visible" if i in (0,3,8,14) else "hidden"}"></span>'
                 f'<div class="btn" style="width:24px;min-width:24px;height:24px;padding:0;font-size:12px">M</div><div class="btn" style="width:24px;min-width:24px;height:24px;padding:0;font-size:12px">S</div><div style="width:6px"></div>'
                 + steps(0.28, cur=3).replace('class="step', 'style="height:24px" class="step') + track_tail(16, 'x4').replace('class="btn', 'style="height:24px" class="btn') + '</div>')
    body += '</div><div class="status">Sequence page: all 19 tracks, grouped by engine. Other pages: Sound (one voice at a time, all its parameters), Modulation, FX, Mix, Library.</div>'
    body += annot(1, 12, 64) + annot(2, 1262, 112)
    page('design-c-pages-sequence', body)

def design_c_mix():
    body = topbar() + page_tabs(4)
    body += '<div style="display:flex;margin:12px 20px 0 20px;border:1px solid var(--rule);height:720px">'
    prev = None
    for i, (eng, n) in enumerate(ALL):
        border = 'border-left:2px solid var(--edge);' if (prev and eng != prev) else ''
        prev = eng
        lvl = random.random()
        rows = [('Level', f'{int(-30 + lvl * 30)} dB'), ('Pan', 'C'), ('Drive', str(int(random.random() * 127)))]
        if eng == 'LXR':
            rows.append(('Decim.', '127'))
        if eng == 'NT':
            rows.append(('Bit red.', '0'))
        out = '<div class="label" style="margin-top:6px">Out</div><div class="btn sel" style="width:58px;height:24px;font-size:12px;padding:0 4px;margin-top:2px">' + ('St1' if eng == 'LXR' else 'Mix') + '</div>'
        choke = '<div class="label" style="margin-top:6px">Choke</div><div class="btn sel" style="width:58px;height:24px;font-size:12px;padding:0 4px;margin-top:2px">' + ('1' if n.startswith('Hat') else '—') + '</div>'
        midi = '<div class="label" style="margin-top:6px">MIDI</div><div class="small">ch 1 · ' + ['C1','A1','D2','D1','C#2','F#1','A#1','E1','F1','G1','A1','B1','C2','C3','C3','C3','C3','C3','C3'][i] + '</div>'
        fader_h = 200
        fader = (f'<div style="height:{fader_h}px;width:10px;background:var(--well);border:1px solid var(--rule);position:relative;margin:8px auto 0">'
                 f'<div style="position:absolute;left:0;right:0;bottom:0;height:{int(lvl*fader_h)}px;background:var(--green)"></div></div>')
        body += (f'<div style="width:68px;{border}border-right:1px solid var(--rule);padding:6px 6px;display:flex;flex-direction:column;align-items:center">'
                 f'<span class="tag">{eng}</span><div class="small" style="margin-top:4px;text-align:center;line-height:1.05;height:30px">{n}</div>'
                 + knob('Pan', 'C', .5, size=28) + knob('Drive', '63', .5, size=28) + fader + out + choke + midi +
                 '<div class="row" style="margin-top:8px;gap:2px"><div class="btn" style="width:26px;min-width:26px;height:24px;padding:0">M</div><div class="btn" style="width:26px;min-width:26px;height:24px;padding:0">S</div></div></div>')
    body += '<div style="flex:1;padding:10px" class="panel"><div class="cap">Masters</div><div class="label" style="margin-top:8px">LXR main / aux<br>NT volume<br>Mix out</div></div></div>'
    body += '<div class="status">Mix page: one strip per track. Faders map to vdrum faders (LXR), Part Level (NT) and Level (HX).</div>'
    body += annot(3, 12, 64)
    page('design-c-pages-mix', body)

design_a(); design_b(); design_c_seq(); design_c_mix()
