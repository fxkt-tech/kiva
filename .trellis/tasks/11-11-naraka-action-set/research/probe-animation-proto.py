#!/usr/bin/env python3
"""Inspect explicit protobuf wire fields; numeric fields remain unnamed without schema."""
from pathlib import Path
import argparse, collections, hashlib, json, struct
ROOT=Path(__file__).resolve().parent

def varint(data,pos):
    value=0
    for shift in range(0,70,7):
        if pos>=len(data):raise ValueError('truncated varint')
        b=data[pos];pos+=1;value|=(b&127)<<shift
        if b<128:return value,pos
    raise ValueError('excessive varint')

def fields(data):
    result=[];pos=0
    while pos<len(data):
        start=pos;tag,pos=varint(data,pos);number,wire=tag>>3,tag&7
        if not 1<=number<2**29:raise ValueError('field number')
        if wire==0:value,pos=varint(data,pos)
        elif wire==1:
            if pos+8>len(data):raise ValueError('fixed64 bounds')
            value=data[pos:pos+8];pos+=8
        elif wire==2:
            size,pos=varint(data,pos)
            if pos+size>len(data):raise ValueError('length bounds')
            value=data[pos:pos+size];pos+=size
        elif wire==5:
            if pos+4>len(data):raise ValueError('fixed32 bounds')
            value=data[pos:pos+4];pos+=4
        else:raise ValueError('unsupported wire')
        result.append((number,wire,value,start,pos))
    return result

def describe(data,depth=0):
    out=[]
    for number,wire,v,start,end in fields(data):
        row={'field':number,'wire':wire,'offset':start,'end':end}
        if wire==0:row['value']=v
        elif wire in (1,5):row.update(hex=v.hex(),float=struct.unpack('<d' if wire==1 else '<f',v)[0])
        else:
            try:
                s=v.decode('utf-8')
                if not s or not all(c.isprintable() for c in s):raise ValueError('not text')
                row['text']=s
            except (ValueError,UnicodeError):
                if depth<8:
                    try:row['message']=describe(v,depth+1)
                    except (ValueError,struct.error):row.update(bytes=len(v),hex=v.hex())
                else:row.update(bytes=len(v),prefix_hex=v[:64].hex())
        out.append(row)
    return out

def numeric_paths(data,targets,path=(),depth=0):
    if depth>12:return []
    out=[];ordinal=collections.Counter()
    for n,w,v,start,end in fields(data):
        at=path+((n,ordinal[n]),);ordinal[n]+=1
        if w==0 and v in targets:out.append(dict(path=at,value=v,message=describe(data)))
        if w==2:
            try:out.extend(numeric_paths(v,targets,at,depth+1))
            except (ValueError,struct.error):pass
    return out

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--source-root',type=Path,default=ROOT);args=parser.parse_args()
    source_root=args.source_root
    archive=(source_root/'animation_info_archive.bytes.payload.bin').read_bytes()
    selected=[];names={}
    mapping=ROOT/'animation-state-mapping.json'
    chosen={r['clip_names'][0] for r in json.loads(mapping.read_text())['rows']} if mapping.exists() else {f'male_hengdao_attack_light_0{i}' for i in range(1,6)}|{'male_hengdao_shock_01','male_hengdao_shock_04','male_hengdao_attack_shock_light_01'}
    for n,w,v,start,end in fields(archive):
        if w!=2:raise ValueError('expected archive message')
        f=fields(v);name=next((x[2].decode() for x in f if x[0]==2 and x[1]==2),'')
        if name in chosen:
            ident=next(x[2] for x in f if x[0]==1 and x[1]==0)
            names[name]=ident
            selected.append(dict(offset=start,end=end,name=name,hash_u64=ident,sha256=hashlib.sha256(v).hexdigest(),fields=describe(v)))
    (ROOT/'animation-info-selected.json').write_text(json.dumps(selected,ensure_ascii=False,indent=2)+'\n')
    wanted=[f'male_hengdao_attack_light_0{i}' for i in range(1,6)]+['male_hengdao_shock_01','male_hengdao_shock_04','male_hengdao_attack_shock_light_01']
    targets={names[n] for n in wanted if n in names}
    for basename in ['all_flat_playable_animators_archive','anim_state_set_archive','weapon_battle_config_archive']:
        raw=(source_root/(basename+'.bytes.payload.bin')).read_bytes()
        rows=numeric_paths(raw,targets)
        (ROOT/(basename+'.selected.json')).write_text(json.dumps(rows,ensure_ascii=False,indent=2)+'\n')
        print(basename,'targets',len(rows))
        for r in rows[:8]:print('path',r['path'],'clip',next(n for n,h in names.items() if h==r['value']))
    print('clips',len(selected),'identities',{n:names.get(n) for n in wanted})

if __name__=='__main__':main()
