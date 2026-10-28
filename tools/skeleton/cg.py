"""Call graph + data references per function from the reference ELF (uses --emit-relocs relocations)."""
import json, bisect, sys
from elftools.elf.elffile import ELFFile
from elftools.elf.relocation import RelocationSection

e = ELFFile(open('/home/user/lxr-ref/lxr_v1.7_gcc11.3.elf', 'rb'))
symtab = e.get_section_by_name('.symtab')
funcs = []  # (start, end, name)
objs = {}
syms_by_idx = list(symtab.iter_symbols())
for s in syms_by_idx:
    t = s['st_info']['type']
    if t == 'STT_FUNC' and s['st_size'] > 0:
        a = s['st_value'] & ~1
        funcs.append((a, a + s['st_size'], s.name))
    elif t == 'STT_OBJECT':
        objs[s['st_value']] = (s.name, s['st_size'])
funcs.sort()
starts = [f[0] for f in funcs]

def func_at(addr):
    i = bisect.bisect_right(starts, addr) - 1
    if i >= 0 and funcs[i][0] <= addr < funcs[i][1]:
        return funcs[i][2]
    return None

obj_sorted = sorted((a, n, sz) for a, (n, sz) in objs.items())
ostarts = [o[0] for o in obj_sorted]
def obj_at(addr):
    i = bisect.bisect_right(ostarts, addr) - 1
    if i >= 0 and obj_sorted[i][0] <= addr < obj_sorted[i][0] + max(obj_sorted[i][2], 1):
        return obj_sorted[i][1]
    return None

calls = {}; data = {}
for sec in e.iter_sections():
    if not isinstance(sec, RelocationSection):
        continue
    target = e.get_section(sec['sh_info'])
    if target.name not in ('.text', '.itcm_text'):
        continue
    for r in sec.iter_relocations():
        off = r['r_offset']
        fn = func_at(off)
        if fn is None:
            continue
        s = syms_by_idx[r['r_info_sym']]
        tp = r['r_info_type']
        val = s['st_value']
        # section symbols: need the addend from the instruction/literal; read literal for ABS32
        if s['st_info']['type'] == 'STT_SECTION' or s.name == '':
            if tp == 2:  # R_ARM_ABS32: read the literal from the section data
                data_bytes = target.data()
                v = int.from_bytes(data_bytes[off - target['sh_addr']: off - target['sh_addr'] + 4], 'little')
                f2 = func_at(v & ~1)
                if f2 and (v & ~1) == [f for f in funcs if f[2] == f2][0][0]:
                    calls.setdefault(fn, set()).add(f2)
                else:
                    o = obj_at(v)
                    data.setdefault(fn, set()).add(o or ('@%x' % v))
            continue
        name = s.name
        if s['st_info']['type'] == 'STT_FUNC':
            calls.setdefault(fn, set()).add(name)
        else:
            if tp == 2:
                data_bytes = target.data()
                v = int.from_bytes(data_bytes[off - target['sh_addr']: off - target['sh_addr'] + 4], 'little')
                o = obj_at(v)
                data.setdefault(fn, set()).add(o or name)
            else:
                data.setdefault(fn, set()).add(name)

out = {'funcs': {f[2]: [f[0], f[1]] for f in funcs},
       'calls': {k: sorted(v) for k, v in calls.items()},
       'data': {k: sorted(v) for k, v in data.items()},
       'objs': {n: [a, sz] for a, (n, sz) in objs.items()}}
json.dump(out, open(sys.argv[1], 'w'), indent=0)
print(len(funcs), len(calls), len(data))
