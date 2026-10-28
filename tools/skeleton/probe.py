"""Evaluate object-like macros with GCC 11.3 (same flags as the firmware) to get their C type and value.

For each translation unit, a probe file includes the .c file and defines
    static const __typeof__((M)) __probe_M = (M);
for each macro M first seen in that TU. Macros that do not compile as constant
initialisers are dropped iteratively. Values and types are read from the object file.
"""
import json, os, re, subprocess, sys
from elftools.elf.elffile import ELFFile

S = os.path.dirname(os.path.abspath(__file__)) + '/'
SRC = '/home/user/lxr-ref/lxr-desktop-app/'
GCC = '/home/user/lxr-ref/toolchains/arm-gnu-toolchain-11.3.rel1-x86_64-arm-none-eabi/bin/arm-none-eabi-gcc'
INC = [SRC + x for x in ['Core/Inc', 'Drivers/CMSIS/Include', 'Drivers/CMSIS/Device/ST/STM32F7xx/Include',
                         'Drivers/STM32F7xx_HAL_Driver/Inc', 'Drivers/STM32F7xx_HAL_Driver/Inc/Legacy',
                         'Middlewares/Third_Party/FatFs/src', 'FATFS/App', 'FATFS/Target',
                         'Middlewares/JS/STM32_USB_Device_Library/Core/Inc',
                         'Middlewares/JS/STM32_USB_Device_Library/Class/CDC/Inc', 'USB_DEVICE_MIDI/App', 'USB_DEVICE_MIDI/Target']]
FLAGS = ['-mcpu=cortex-m7', '-mthumb', '-mfpu=fpv5-d16', '-mfloat-abi=hard', '--specs=nano.specs', '-DSTM32F765xx',
         '-DUSE_HAL_DRIVER', '-O0', '-g', '-gdwarf-4', '-ffast-math', '-freciprocal-math', '-mfp16-format=ieee', '-w',
         '-fno-common']
macros = json.load(open(S + 'macros.json'))
macros.pop('__conflicts__', None)
files = {l.strip().replace('/', '_'): l.strip() for l in open(S + 'files.txt')}
OUT = S + 'probe/'
os.makedirs(OUT, exist_ok=True)

wanted = [k for k, v in macros.items() if v[2] is None and v[3] != '' and not k.endswith('_H_') and not k.endswith('_H')
          and '__attribute__' not in v[3]]
by_tu = {}
for k in wanted:
    by_tu.setdefault(macros[k][4], []).append(k)

BASE = {('DW_ATE_float', 4): 'f32', ('DW_ATE_float', 8): 'f64'}
results = {}
failed = {}
for tu, names in sorted(by_tu.items()):
    src = files[tu]
    names = list(names)
    for attempt in range(60):
        lines = ['#include "%s"' % (SRC + src)]
        lineof = {}
        for n in names:
            lineof[len(lines) + 1] = n
            lines.append('__attribute__((used)) static const __typeof__((%s)) __probe_%s = (%s);' % (n, n, n))
        pf = OUT + tu + '.probe.c'
        open(pf, 'w').write('\n'.join(lines) + '\n')
        r = subprocess.run([GCC] + FLAGS + ['-I' + i for i in INC] + ['-c', pf, '-o', pf + '.o'], capture_output=True, text=True)
        if r.returncode == 0:
            break
        bad = set()
        for m in re.finditer(re.escape(pf) + r':(\d+):\d+:', r.stderr):
            ln = int(m.group(1))
            if ln in lineof:
                bad.add(lineof[ln])
        if not bad:
            print('cannot attribute errors for', tu, r.stderr[:2000], file=sys.stderr)
            names = []
            break
        for b in bad:
            failed[b] = macros[b][3]
        names = [n for n in names if n not in bad]
    if not names:
        continue
    # read values + types
    e = ELFFile(open(pf + '.o', 'rb'))
    symtab = e.get_section_by_name('.symtab')
    d = e.get_dwarf_info()
    vtypes = {}
    for cu in d.iter_CUs():
        for die in cu.get_top_DIE().iter_children():
            if die.tag == 'DW_TAG_variable' and 'DW_AT_name' in die.attributes:
                nm = die.attributes['DW_AT_name'].value.decode()
                if nm.startswith('__probe_'):
                    t = die.get_DIE_from_attribute('DW_AT_type')
                    # strip const/volatile/typedef
                    chain = []
                    while t.tag in ('DW_TAG_const_type', 'DW_TAG_volatile_type', 'DW_TAG_typedef'):
                        if t.tag == 'DW_TAG_typedef':
                            chain.append(t.attributes['DW_AT_name'].value.decode())
                        if 'DW_AT_type' not in t.attributes:
                            break
                        t = t.get_DIE_from_attribute('DW_AT_type')
                    desc = {'tag': t.tag, 'typedefs': chain}
                    if t.tag == 'DW_TAG_base_type':
                        enc = t.attributes['DW_AT_encoding'].value
                        desc['enc'] = enc; desc['size'] = t.attributes['DW_AT_byte_size'].value
                        desc['name'] = t.attributes['DW_AT_name'].value.decode()
                    elif t.tag == 'DW_TAG_enumeration_type':
                        desc['size'] = t.attributes['DW_AT_byte_size'].value
                        desc['name'] = t.attributes['DW_AT_name'].value.decode() if 'DW_AT_name' in t.attributes else None
                        u = t.get_DIE_from_attribute('DW_AT_type') if 'DW_AT_type' in t.attributes else None
                        desc['enc'] = u.attributes['DW_AT_encoding'].value if u else None
                    elif t.tag == 'DW_TAG_array_type':
                        desc['size'] = None
                    vtypes[nm[8:]] = desc
    for s in symtab.iter_symbols():
        if s.name.startswith('__probe_'):
            n = s.name[8:]
            sec = e.get_section(s['st_shndx'])
            data = sec.data()[s['st_value']:s['st_value'] + s['st_size']]
            results[n] = {'type': vtypes.get(n), 'bytes': data.hex(), 'size': s['st_size']}
json.dump({'ok': results, 'failed': failed}, open(S + 'macro_values.json', 'w'), indent=0)
print('ok', len(results), 'failed', len(failed))
