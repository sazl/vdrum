"""Collect #defines (with their defining file) from the -dD preprocessor output of the in-scope files."""
import re, glob, os, json, sys
S = os.path.dirname(os.path.abspath(__file__)) + '/'
LM = re.compile(r'^# (\d+) "([^"]*)"')
DEF = re.compile(r'^#define (\w+)(\([^)]*\))?\s?(.*)$')
UNDEF = re.compile(r'^#undef (\w+)')
macros = {}  # name -> (file, line, params, body)
for p in sorted(glob.glob(S + 'pp/*.dd')):
    cur_f, cur_l = None, 0
    for raw in open(p, errors='replace'):
        raw = raw.rstrip('\n')
        m = LM.match(raw)
        if m:
            cur_l = int(m.group(1)); cur_f = os.path.normpath(m.group(2)); continue
        m = DEF.match(raw)
        if m and cur_f and cur_f.startswith('Core/'):
            name, params, body = m.group(1), m.group(2), m.group(3).strip()
            key = name
            prev = macros.get(key)
            if prev and prev[3] != body:
                macros.setdefault('__conflicts__', []).append((name, prev[:2], (cur_f, cur_l), prev[3], body))
            elif not prev:
                macros[key] = (cur_f, cur_l, params, body, os.path.basename(p)[:-3])
        cur_l += 1
json.dump(macros, open(S + 'macros.json', 'w'), indent=0)
if __name__ == '__main__':
    from collections import Counter
    c = Counter(v[0] for k, v in macros.items() if k != '__conflicts__')
    for f, n in sorted(c.items()):
        print(n, f)
    for x in macros.get('__conflicts__', []):
        print('CONFLICT', x)
