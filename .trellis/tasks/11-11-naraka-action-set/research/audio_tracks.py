#!/usr/bin/env python3
"""Extract bounded, labelled wire evidence of real animation->sound links."""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path
import struct

ROOT=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('animation_proto',ROOT/'probe-animation-proto.py')
proto=importlib.util.module_from_spec(spec);spec.loader.exec_module(proto)
KEYS={
 'male_hengdao_attack_light_01','male_hengdao_attack_light_02','male_hengdao_attack_light_03',
 'male_hengdao_attack_shock_light_01','male_hengdao_shock_01','male_hengdao_shock_04',
 'male_hengdao_dodge_front_01','male_hengdao_dodge_back_01','male_hengdao_dodge_left_01','male_hengdao_dodge_right_01',
 'male_hengdao_jump_inplace_up_01','male_hengdao_jump_forward_up_01','male_hengdao_jump_down_to_land_01',
 'male_hengdao_run_01','male_hengdao_walk_01','male_hengdao_run_start_01','male_hengdao_sprint_01',
}

def field_value(fields,number,wire,default=None):
    matches=[v for n,w,v,_,_ in fields if (n,w)==(number,wire)]
    if len(matches)>1:raise ValueError('expected singular field')
    return matches[0] if matches else default

def sound_payload(data):
    f=proto.fields(data)
    described=proto.describe(data)
    for item in described:
        if item['field']==8:
            raw=field_value(f,8,2)
            item.pop('message',None)
            item.update(bytes=len(raw),sha256=hashlib.sha256(raw).hexdigest(),note='Original position/rotation/scale submessage omitted from compact report; full original payload remains reproducible.')
    return dict(event=field_value(f,1,2,b'').decode(),fields=described,sha256=hashlib.sha256(data).hexdigest())

def main():
    ap=argparse.ArgumentParser();ap.add_argument('--source-root',type=Path,default=ROOT);args=ap.parse_args()
    raw=(args.source_root/'proto_track_archive.bytes.payload.bin').read_bytes();rows=[]
    for _,w,value,start,end in proto.fields(raw):
        assert w==2
        entry=proto.fields(value);key=field_value(entry,1,2).decode()
        if key not in KEYS:continue
        groups=[]
        for n,w,group,groupstart,groupend in entry:
            if (n,w)!=(2,2):continue
            gf=proto.fields(group);group_id=field_value(gf,2,0);events=[]
            for nn,ww,event,eventstart,eventend in gf:
                if (nn,ww)!=(1,2):continue
                ef=proto.fields(event);time=field_value(ef,1,5);kind=field_value(ef,2,0);payload=field_value(ef,3,2)
                if payload is None:continue
                for pn,pw,pv,_,_ in proto.fields(payload):
                    if pw!=2:continue
                    sounds=[]
                    if pn in [110,111]:sounds.append(dict(path=[pn],sound=sound_payload(pv)))
                    elif pn==225:
                        for bn,bw,bv,_,_ in proto.fields(pv):
                            if bw!=2:continue
                            bf=proto.fields(bv);sp=field_value(bf,1,2)
                            if sp is not None:sounds.append(dict(path=[225,bn,1],condition_numeric_field_2=field_value(bf,2,0),sound=sound_payload(sp)))
                    if sounds:
                        events.append(dict(event_offset_in_group=eventstart,event_end_in_group=eventend,original_time_float=struct.unpack('<f',time)[0] if time else None,original_time_hex=time.hex() if time else None,track_type_numeric=kind,event_sha256=hashlib.sha256(event).hexdigest(),sounds=sounds))
            groups.append(dict(group_numeric_2=group_id,group_offset_in_entry=groupstart,group_end_in_entry=groupend,sound_events=events))
        rows.append(dict(key=key,archive_offset=start,archive_end=end,entry_sha256=hashlib.sha256(value).hexdigest(),groups=groups))
    report=dict(archive_sha256=hashlib.sha256(raw).hexdigest(),note='Numeric fields retain source wire labels. Field1 float is not declared seconds or normalized time: schema and group numeric 0/1/2 meanings are unresolved. Source groups may differ; do not flatten all groups into one runtime track. Conditional numeric field2 is retained without inventing its meaning.',selected=rows)
    (ROOT/'audio-tracks-selected.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
    for row in rows:
        print(row['key'],[(g['group_numeric_2'],e['original_time_float'],[s['sound']['event'] for s in e['sounds']]) for g in row['groups'] for e in g['sound_events']])

if __name__=='__main__':main()
