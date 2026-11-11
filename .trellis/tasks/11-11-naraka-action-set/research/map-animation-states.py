#!/usr/bin/env python3
"""Bridge original named states to original clip names via CRC32 and override records."""
from pathlib import Path
import argparse, collections, hashlib, importlib.util, json, re, sqlite3, struct, zlib
ROOT=Path(__file__).resolve().parent

def load(name,file):
    s=importlib.util.spec_from_file_location(name,ROOT/file);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
pb=load('probe_pb','probe-animation-proto.py');tab=load('probe_tab','probe-animation-tables.py')

def state_index():
    name='AutoAnimatorStates';src=next(r for r in tab.manifest if r['name']==name)
    raw=Path(src['file']).read_bytes();data=tab.dbbl.xor_decode(raw,name,tab.salt);r=tab.dbbl.Reader(data)
    if r.take(4)!=b'DDBL' or r.u32()!=1 or r.string()!=name:raise ValueError('header')
    count=r.u32();headers=[(r.u32(),r.string()) for _ in range(count)]
    if len(headers)!=1 or r.pos!=headers[0][0]:raise ValueError('header range')
    count=r.u32();isz=r.u32();entries=[(r.string(),r.u32()) for _ in range(count)]
    if r.pos!=headers[0][0]+8+isz:raise ValueError('index range')
    byhash=collections.defaultdict(list)
    for name,size in entries:
        start=r.pos;rec=r.take(size);rr=tab.dbbl.Reader(rec)
        if rr.string()!=name:raise ValueError('record key')
        byhash[zlib.crc32(name.encode())].append(dict(state=name,offset=start,end=r.pos,bytes=size,sha256=hashlib.sha256(rec).hexdigest()))
    if r.pos!=len(data):raise ValueError('table EOF')
    return byhash,dict(source_file=src['file'],source_sha256=src['sha256'],source_path_id=src['path_id'],decoded_sha256=hashlib.sha256(data).hexdigest(),records=count,index_size=isz,complete_framing=True)

def wanted(clip,states):
    core={f'male_hengdao_attack_light_0{i}' for i in range(1,6)}|{'male_hengdao_attack_shock_light_01','male_hengdao_shock_01','male_hengdao_shock_04','male_hengdao_idle_01','male_hengdao_promenade_01','male_hengdao_run_01','male_hengdao_crouch_idle_01','male_hengdao_crouch_promenade_01','male_hengdao_crouch_run_01'}
    if clip in core:return True
    if clip in {'male_hengdao_finger_left_01','male_hengdao_finger_right_01','male_bow_finger_default_left_01','male_bow_finger_default_right_01'}:return True
    if clip in {'male_sniper_idle_01','male_sniper_ads_idle_01','male_sniper_ads_attack_01','male_sniper_attack_01','male_sniper_attack_01_recover','male_sniper_reload_01','male_bow_idle_01','male_bow_hold_idle_01','male_bow_attack_01','male_bow_attack_01_charge','male_bow_attack_01_charge_hold_01','male_bow_attack_01_recover','male_bow_attack_aim_01','male_bow_attack_aim_01_pre'}:return True
    if re.fullmatch(r'male_hengdao_(jump_(inplace|forward)_(up|down)_0[12]|jump_down_loop_01|jump_down_to_land_0[1-6]|dodge_(front|back|left|right)(_double)?_01)',clip):return True
    shared={'common_hurt_front_1','common_hurt_back_1','common_hurt_left_1','common_hurt_right_1','common_die_hurt_front_1','common_die_hurt_back_1','dinner_slide_01','dinner_sprint_battle'}
    return any(s in shared for s in states)

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--source-root',type=Path,default=ROOT);args=parser.parse_args()
    source_root=args.source_root
    byhash,stateproof=state_index();data=(source_root/'playable_runtime_override_mapping_archive.bytes.payload.bin').read_bytes()
    infos={r['name']:r for r in json.loads((ROOT/'animation-info-selected.json').read_text())}
    # Read all info records to cover shared hurt/death clips, keeping only selected outputs.
    for _,w,v,start,end in pb.fields((source_root/'animation_info_archive.bytes.payload.bin').read_bytes()):
        fs=pb.fields(v);name=next((x[2].decode() for x in fs if x[0]==2 and x[1]==2),'')
        if not name:continue
        if name not in infos:
            infos[name]=dict(name=name,hash_u64=next(x[2] for x in fs if x[0]==1 and x[1]==0),fields=pb.describe(v))
    db=sqlite3.connect('file:'+str(tab.BASE/'identity/appres.sqlite')+'?mode=ro',uri=True);rows=[]
    paths=collections.defaultdict(list)
    for path,key,size in db.execute("select r.path,b.key,b.declared_size from resources r join bundles b on b.id=r.bundle_id where r.path like '%animclipadapter%'"):
        paths[path.rsplit('/',1)[-1]].append((path,key,size))
    groups={'male_hd_weapon_player','dinner_male_movement_player','dinner_female_movement_player','dinner_male_hurt_act_player','female_bow_weapon_player'}
    for _,w,v,start,end in pb.fields(data):
        fs=pb.fields(v);name=next(x[2].decode() for x in fs if x[0]==1 and x[1]==2)
        if name not in groups:continue
        for n,w,c,at,en in fs:
            if n!=5 or w!=2:continue
            ff=pb.fields(c);h=next(x[2] for x in ff if x[0]==1 and x[1]==0)&0xffffffff
            clipnames=[x[2].decode() for x in ff if x[1]==2];states=[x['state'] for x in byhash[h]]
            if not clipnames or not wanted(clipnames[0],states):continue
            clip=clipnames[0];info=infos.get(clip)
            refs=paths[clip+'.anim']
            rows.append(dict(group=name,group_archive_offset=start,group_archive_end=end,override_entry_offset=at,override_entry_end=en,
                             override_entry_sha256=hashlib.sha256(c).hexdigest(),state_crc32=h,state_records=byhash[h],clip_names=clipnames,
                             clip_string_fields={str(i+2):value for i,value in enumerate(clipnames)},
                             selection_evidence='same-name-both-fields' if len(set(clipnames))==1 else 'distinct-name-candidates-runtime-selection-unproven',
                             identity_details_apply_to=clip,
                             clip_hash_u64=info['hash_u64'] if info else None,
                             duration=next((f['float'] for f in info['fields'] if f['field']==3),None) if info else None,
                             source_paths=[dict(path=p,bundle_key=k,declared_bundle_size=s) for p,k,s in refs]))
    result=dict(method='CRC32 of exact original AutoAnimatorStates string key equals low32 of original override field5 key; all table records consumed and key equality verified. Field5 child strings 2/3 retained without inventing their selection rule. When strings differ, original runtime selection is unproven; hash/duration/paths describe identity_details_apply_to (the first-listed candidate) only.',
                state_source=stateproof,override_source_sha256=hashlib.sha256(data).hexdigest(),rows=rows)
    (ROOT/'animation-state-mapping.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
    graphdata=(source_root/'combo_graph_archive.bytes.payload.bin').read_bytes()
    for _,_,v,start,end in pb.fields(graphdata):
        fs=pb.fields(v);name=next(x[2].decode() for x in fs if x[0]==1 and x[1]==2)
        if name!='male_hd_combo_graph':continue
        graph=next(x[2] for x in fs if x[0]==2 and x[1]==2);edges=[];nodes=[]
        for n,w,c,off,en in pb.fields(graph):
            if n==3 and w==2:
                ff=pb.fields(c);left=next((x[2].decode() for x in ff if x[0]==1 and x[1]==2),'');right=next((x[2].decode() for x in ff if x[0]==2 and x[1]==2),'')
                if (left,right) in {('','101'),('101','102'),('102','103')} or right=='141':
                    edges.append(dict(source=left,target=right,offset=off,end=en,fields=pb.describe(c)))
            elif n==4 and w==2:
                ff=pb.fields(c);key=next((x[2].decode() for x in ff if x[0]==1 and x[1]==2),'')
                if key in {'101','102','103','141'}:nodes.append(dict(key=key,offset=off,end=en,fields=pb.describe(c)))
        (ROOT/'combo-graph-hd-compact.json').write_text(json.dumps(dict(name=name,archive_offset=start,archive_end=end,source_sha256=hashlib.sha256(graphdata).hexdigest(),selected_edges=edges,selected_nodes=nodes),ensure_ascii=False,indent=2)+'\n')
    print('selected mappings',len(rows))
    for r in rows:
        if r['group']=='male_hd_weapon_player' and r['clip_names'][0] in {f'male_hengdao_attack_light_0{i}' for i in range(1,6)}|{'male_hengdao_shock_01','male_hengdao_shock_04','male_hengdao_attack_shock_light_01'}:
            print([x['state'] for x in r['state_records']],r['clip_names'][0],r['duration'])

if __name__=='__main__':main()
