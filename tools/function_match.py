# Match functions of a relocatable-linked ELF against the shipped binary with relocated bytes masked.
import sys, json
from elftools.elf.elffile import ELFFile
from elftools.elf.relocation import RelocationSection
S, b = sys.argv[1], sys.argv[2]
ship = open(S+'/fw/fw171.bin','rb').read()
f = ELFFile(open(f'{S}/build-{b}/fw.elf','rb'))
segs=[(s['p_vaddr'],s['p_paddr'],s['p_filesz']) for s in f.iter_segments() if s['p_type']=='PT_LOAD']
def v2off(a):
    for va,pa,fs in segs:
        if va<=a<va+fs: return pa+(a-va)-0x08008000
binf=open(f'{S}/build-{b}/fw.bin','rb').read()
# masked addresses from relocations (VMA)
masked=set()
for sec in f.iter_sections():
    if isinstance(sec, RelocationSection):
        tgt=f.get_section(sec['sh_info']); base=tgt['sh_addr']
        if not tgt['sh_flags'] & 2: continue
        for r in sec.iter_relocations():
            a=r['r_offset']  # final link: r_offset is VMA
            for k in range(4): masked.add(a+k)
syms=[]
for s in f.get_section_by_name('.symtab').iter_symbols():
    if s['st_info']['type']=='STT_FUNC' and s['st_size']>=8:
        syms.append((s.name, s['st_value']&~1, s['st_size']))
syms=sorted(set(syms), key=lambda x:x[1])
res={}; matched=0; mb=0; tb=0; amb=0
for name,a,sz in syms:
    o=v2off(a)
    if o is None: continue
    body=binf[o:o+sz]; mask=[(a+i) in masked for i in range(sz)]
    tb+=sz
    # longest unmasked run
    best=(0,0); i=0
    while i<sz:
        if mask[i]: i+=1; continue
        j=i
        while j<sz and not mask[j]: j+=1
        if j-i>best[1]-best[0]: best=(i,j)
        i=j
    seg=body[best[0]:best[1]]
    hits=[]
    if len(seg)>=4:
        p=ship.find(seg)
        while p>=0 and len(hits)<50:
            st=p-best[0]
            if st>=0 and st+sz<=len(ship) and all(mask[k] or ship[st+k]==body[k] for k in range(sz)):
                hits.append(st)
            p=ship.find(seg,p+1)
    hits=sorted(set(hits))
    if len(hits)==1: matched+=1; mb+=sz; res[name]={'ship_addr':hex(0x08008000+hits[0]),'size':sz,'our_addr':hex(a)}
    elif len(hits)>1: amb+=1; res[name]={'ambiguous':[hex(0x08008000+h) for h in hits[:5]],'size':sz}
    else: res[name]={'unmatched':True,'size':sz}
print(b,'funcs',len(syms),'unique',matched,'ambiguous',amb,'bytes',tb,mb)
json.dump(res,open(f'{S}/match-{b}.json','w'),indent=1)
