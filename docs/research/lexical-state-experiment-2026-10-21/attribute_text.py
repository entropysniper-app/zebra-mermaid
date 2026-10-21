from pathlib import Path
import struct,json,hashlib,re,sys,collections
binary=Path(sys.argv[1]);dest=Path(sys.argv[2]);data=binary.read_bytes()
assert data[:4]==bytes.fromhex('cffaedfe'),'Expected little-endian Mach-O64'
ncmds=struct.unpack_from('<I',data,16)[0];off=32;sections=[];symtab=None;function_starts=None;text_base=None
for _ in range(ncmds):
 command,length=struct.unpack_from('<II',data,off);assert length>=8
 if command==0x19:
  seg=data[off+8:off+24].split(b'\0')[0].decode();vmaddr=struct.unpack_from('<Q',data,off+24)[0];nsect=struct.unpack_from('<I',data,off+64)[0]
  if seg=='__TEXT':text_base=vmaddr
  for j in range(nsect):
   pos=off+72+j*80;name=data[pos:pos+16].split(b'\0')[0].decode();segment=data[pos+16:pos+32].split(b'\0')[0].decode();addr,size=struct.unpack_from('<QQ',data,pos+32);sections.append({'name':name,'segment':segment,'address':addr,'bytes':size,'index':len(sections)+1})
 elif command==2:symtab=struct.unpack_from('<IIII',data,off+8)
 elif command==0x26:function_starts=struct.unpack_from('<II',data,off+8)
 off+=length
text=next(s for s in sections if s['segment']=='__TEXT' and s['name']=='__text');assert symtab and function_starts and text_base is not None
symoff,nsyms,stroff,strsize=symtab;strings=data[stroff:stroff+strsize];names=collections.defaultdict(list)
def demangle(s):
 value=s[1:] if s.startswith('__ZN') else s
 if not(value.startswith('_ZN') and value.endswith('E')):return s
 body=value[3:-1];parts=[]
 while body:
  m=re.match(r'([0-9]+)',body)
  if not m:return s
  n=int(m[1]);body=body[len(m[1]):]
  if n>len(body):return s
  part=body[:n];body=body[n:]
  if re.fullmatch(r'h[0-9a-f]{16}',part):continue
  for old,new in [('$LT$','<'),('$GT$','>'),('$u20$',' '),('$C$',','),('$RF$','&'),('$BP$','*'),('$LP$','('),('$RP$',')'),('$u7b$','{'),('$u7d$','}')]:part=part.replace(old,new)
  parts.append(part.replace('..','::'))
 return '::'.join(parts)
for i in range(nsyms):
 strx,ntype,nsect,desc,value=struct.unpack_from('<IBBHQ',data,symoff+i*16)
 if ntype&0xe0 or ntype&0xe!=0xe or nsect!=text['index']:continue
 end=strings.find(b'\0',strx);name=strings[strx:end].decode('utf-8','replace');names[value].append(demangle(name))
fo,fs=function_starts;addr=text_base;starts=[];n=0;shift=0
for byte in data[fo:fo+fs]:
 n|=(byte&0x7f)<<shift
 if byte&0x80:shift+=7;continue
 if n==0:break
 addr+=n
 if text['address']<=addr<text['address']+text['bytes']:starts.append(addr)
 n=0;shift=0
assert starts and starts==sorted(set(starts));rows=[]
for i,addr in enumerate(starts):
 end=starts[i+1] if i+1<len(starts) else text['address']+text['bytes']
 rows.append({'address':hex(addr),'spanBytesIncludingAlignment':end-addr,'symbols':names.get(addr,[])})
report={'sha256':hashlib.sha256(data).hexdigest(),'instructionTextBytes':text['bytes'],'functionStartSpans':len(rows),'method':'Mach-O LC_FUNCTION_STARTS intervals intersecting __TEXT/__text, paired with N_SECT symbol addresses. Spans include inter-function alignment and may have aliases or missing local names. They estimate attribution, not disassembled exclusive instruction size. Artifact bytes are unmodified. No paths/host identifiers recorded.','rows':sorted(rows,key=lambda r:r['spanBytesIncludingAlignment'],reverse=True)}
dest.write_text(json.dumps(report,indent=2)+'\n')
for r in report['rows'][:35]:print(r['spanBytesIncludingAlignment'],' | '.join(r['symbols']))
