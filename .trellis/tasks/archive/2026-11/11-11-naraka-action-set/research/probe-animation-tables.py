#!/usr/bin/env python3
"""Bounded read-only target-table decoding, with raw target record evidence."""
from pathlib import Path
import hashlib, importlib.util, json, struct

BASE=Path('/home/asuka/kiva-asset-research/naraka')
OUT=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('dbbl',BASE/'identity/dbbl_tables.py')
dbbl=importlib.util.module_from_spec(spec);spec.loader.exec_module(dbbl)
salt=(BASE/'identity/xls_salt575_candidate.bin').read_bytes()
manifest=json.loads((BASE/'identity/xls/textassets.json').read_text())

def strings(raw):
    result=[]
    for at in range(len(raw)-4):
        n=struct.unpack_from('<I',raw,at)[0]
        if not 2<=n<=512 or at+4+2*n>len(raw):continue
        try:s=raw[at+4:at+4+2*n].decode('utf-16-le')
        except UnicodeError:continue
        if all(c.isprintable() and c!='\x00' for c in s) and any(c.isalpha() for c in s):
            result.append(dict(offset=at,characters=n,value=s))
    return result

def main():
    mapping=OUT/'animation-state-mapping.json'
    selected_hashes={r['clip_hash_u64'] for r in json.loads(mapping.read_text())['rows']} if mapping.exists() else set()
    for name in ('AutoAnimationClip','AutoAnimatorStates','AutoAnimTrackAnalysis','BattleAction','WeaponConfig','SkillConfig'):
        src=next(r for r in manifest if r['name']==name)
        raw=Path(src['file']).read_bytes()
        if hashlib.sha256(raw).hexdigest()!=src['sha256']:raise ValueError(name+' source changed')
        data=dbbl.xor_decode(raw,name,salt)
        item=dict(name=name,source_path_id=src['path_id'],source_file=src['file'],source_sha256=src['sha256'],
                  decoded_sha256=hashlib.sha256(data).hexdigest(),decoded_bytes=len(data))
        needle='hengdao'.encode('utf-16-le')
        try:
            idx=dbbl.parse(data,name)
            item['framing_verified']=True
            item['tables']=[{k:v for k,v in t.items() if k!='records'} for t in idx['tables']]
            item['records']=[]
            for table in idx['tables']:
                for rec in table['records']:
                    r=data[rec['offset']:rec['end']]
                    if needle not in r:continue
                    item['records'].append(dict(table=table['name'],**rec,sha256=hashlib.sha256(r).hexdigest(),
                                                strings=strings(r),raw_hex=r.hex()))
        except (ValueError,UnicodeError,struct.error) as e:
            item['framing_verified']=False;item['error']=str(e)
            hits=[];pos=0
            while (pos:=data.find(needle,pos))!=-1:hits.append(pos);pos+=len(needle)
            item['hits']=hits
            if name.startswith('Auto'):
                r=dbbl.Reader(data);r.take(4);r.u32();r.string();n=r.u32();headers=[(r.u32(),r.string()) for _ in range(n)]
                tables=[];records=[]
                nonempty=[(a,b) for a,b in headers if a]
                for ti,(start,sub) in enumerate(nonempty):
                    end=nonempty[ti+1][0] if ti+1<len(nonempty) else len(data)
                    r.pos=start;count=r.u32();index_size=r.u32();entries=[]
                    for _ in range(count):
                        key=struct.unpack('<q',r.take(8))[0] if name=='AutoAnimationClip' else r.string()
                        entries.append((key,r.u32()))
                    if r.pos!=start+8+index_size:raise ValueError(('index not fully consumed',sub,r.pos,start+8+index_size))
                    for key,size in entries:
                        at=r.pos;rec=r.take(size);rr=dbbl.Reader(rec)
                        own_key=struct.unpack('<q',rr.take(8))[0] if name=='AutoAnimationClip' else rr.string()
                        if own_key!=key:raise ValueError(('record key mismatch',sub,key,own_key))
                        if name=='AutoAnimationClip' and key&((1<<64)-1) in selected_hashes or 'hengdao' in str(key) or needle in rec or name=='AutoAnimatorStates' and str(key) in ('attack_1000','attack_1000_charge','attack_1000_prepare'):
                            records.append(dict(table=sub,key=key,offset=at,end=r.pos,bytes=size,sha256=hashlib.sha256(rec).hexdigest(),strings=strings(rec),raw_hex=rec.hex()))
                    if r.pos!=end:raise ValueError(('table not fully consumed',sub,r.pos,end))
                    tables.append(dict(name=sub,offset=start,end=end,count=count,index_size=index_size))
                item['framing_verified']=True;item.pop('error');item['tables']=tables;item['records']=records
        target=OUT/(name+'.hengdao.json')
        target.write_text(json.dumps(item,ensure_ascii=False,indent=2)+'\n')
        print(name,'framing',item['framing_verified'],'records',len(item.get('records',[])),'hits',len(item.get('hits',[])))
        if not item['framing_verified']:print(item['error'])

if __name__=='__main__':main()
