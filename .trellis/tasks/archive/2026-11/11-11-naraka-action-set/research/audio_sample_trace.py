#!/usr/bin/env python3
"""Trace selected v135 Event->PlayAction->Random/Sound->DIDX media and decode."""
import hashlib
import json
import math
from pathlib import Path
import struct
import subprocess
import wave

from audio_probe import OUT, AUDIO, Reader, chunks, fnv, pck, selected_bank, tables

VGM = Path('/home/asuka/kiva-asset-research/vgmstream-r2168/vgmstream-cli')
EVENTS = [
    'gman_hengdao_attack_light_01',
    'gman_hengdao_attack_light_02',
    'gman_hengdao_attack_light_03',
    'gman_hengdao_attack_shock_light_01',
    'gman_all_weapon_touch_shock_katana',
    'gman_all_weapon_touch_bounce_katana',
    'gman_all_jump_start_pre_01',
    'gman_all_jump_down_recover_01',
    'gman_dodge_01',
    'impact_body',
    'move_run',
    'move_sprint',
    'move_jump_takeoff_cloth_fast',
    'move_jump_land',
    'gman_all_break_hurt',
    'gman_all_die_drop',
    'gman_dying_voice_male',
]

def random_tail(obj, objects):
    body = bytes.fromhex(obj['body_hex']); found=[]
    for count in range(1, min(512,len(objects))+1):
        start=len(body)-(4+count*4+2+count*8)
        if start<4 or struct.unpack_from('<I',body,start)[0]!=count: continue
        child_ids=list(struct.unpack_from('<'+'I'*count,body,start+4))
        play_at=start+4+count*4
        if struct.unpack_from('<H',body,play_at)[0]!=count: continue
        playlist=[dict(id=i,weight=w) for i,w in struct.iter_unpack('<Ii',body[play_at+2:])]
        if len(set(child_ids))!=count or not set(child_ids)<=set(objects) or set(child_ids)!={p['id'] for p in playlist} or any(p['weight']<=0 for p in playlist): continue
        found.append(dict(offset=start,children=child_ids,playlist=playlist))
    if len(found)!=1: raise ValueError('random tail not uniquely bounded')
    return found[0]

def switch_tail(obj, objects):
    body=bytes.fromhex(obj['body_hex']);found=[]
    for start in range(4,len(body)-14):
        try:
            r=Reader(body,start);kind=r.take(1)[0];group=r.u32();default=r.u32();continuous=r.take(1)[0]
            if kind not in [0,1] or continuous not in [0,1]: continue
            count=r.u32()
            if not 1<=count<=512: continue
            ids=[r.u32() for _ in range(count)]
            if len(set(ids))!=count or not set(ids)<=set(objects): continue
            ngroups=r.u32()
            if not 1<=ngroups<=512:continue
            groups=[]
            for _ in range(ngroups):
                sid=r.u32();n=r.u32()
                if not 0<=n<=count:raise ValueError('switch list range')
                children=[r.u32() for _ in range(n)]
                if not set(children)<=set(ids):raise ValueError('switch child range')
                groups.append(dict(switch=sid,children=children))
            if len({g['switch'] for g in groups})!=ngroups: continue
            np=r.u32()
            if not 0<=np<=count: continue
            params=[]
            for _ in range(np):
                node=r.u32();flags=r.take(1)[0];on_switch=r.take(1)[0];fade_out,fade_in=struct.unpack('<ii',r.take(8))
                if node not in ids or flags>3 or on_switch>7:raise ValueError('switch params')
                params.append(dict(node=node,flags=flags,on_switch=on_switch,fade_out=fade_out,fade_in=fade_in))
            if r.at!=len(body):continue
            if default not in [g['switch'] for g in groups]:continue
            found.append(dict(offset=start,group_type=kind,group_id=group,default_switch=default,continuous=continuous,children=ids,groups=groups,params=params))
        except (ValueError,struct.error):continue
    if len(found)!=1:raise ValueError(('switch tail not uniquely bounded',len(found)))
    return found[0]

def terminals(ident,objects,path=None):
    path=[] if path is None else path
    if ident in path:raise ValueError('container cycle')
    obj=objects[ident]
    if obj['type']==2:return [ident],[]
    if obj['type']==5:
        tail=random_tail(obj,objects);chosen=tail['children']
    elif obj['type']==6:
        tail=switch_tail(obj,objects)
        chosen=next(g['children'] for g in tail['groups'] if g['switch']==tail['default_switch'])
    else:raise ValueError(('unsupported container',obj['type'],ident))
    all_ids=[];branches=[dict(id=ident,type=obj['type'],tail=tail)]
    for child in chosen:
        ids,sub=terminals(child,objects,path+[ident]);all_ids.extend(ids);branches.extend(sub)
    return all_ids,branches

def main():
    dbbl,ts=tables();banks={r['key']:r for r in ts['WwiseBanks_Bank']};events={r['key']:r for r in ts['WwiseBanks_Event']}
    packages={};cache={};report=[];decoder_sha=hashlib.sha256(VGM.read_bytes()).hexdigest()
    for name in EVENTS:
        row=events[name];r=Reader(row['raw']);assert r.string()==name;bank_name=r.string()
        if bank_name not in cache:
            br=Reader(banks[bank_name]['raw']);assert br.string()==bank_name;package_name=br.string()
            pkg=packages.setdefault(package_name,pck(AUDIO/package_name))
            matches=[e for e in pkg['entries'] if e['section']=='banks' and e['id']==fnv(bank_name)]
            assert len(matches)==1
            info,data=selected_bank(pkg,matches[0]);cache[bank_name]=(info,data)
        info,data=cache[bank_name]
        objs={o['id']:o for c in info['chunks'] for o in c.get('objects',[])}
        event=objs[fnv(name)];assert event['type']==4
        eb=bytes.fromhex(event['body_hex']);assert eb[0]<128 and len(eb)==1+4*eb[0]
        actions=list(struct.unpack_from('<'+'I'*eb[0],eb,1));paths=[];sounds=[]
        for action_id in actions:
            action=objs[action_id];assert action['type']==3
            ab=bytes.fromhex(action['body_hex']);kind,target,bus=struct.unpack_from('<HIB',ab)
            assert kind==0x403 and bus==0
            obj=objs[target]
            ids,branches=terminals(target,objs)
            paths.append(dict(action_id=action_id,play_action_type=kind,target_id=target,target_type=obj['type'],containers=branches,sounds=ids,action_body_hex=ab.hex(),action_properties_count=ab[7],action_ranged_properties_count=ab[8]))
            sounds.extend(ids)
        didx=next(c['media'] for c in info['chunks'] if c['tag']=='DIDX')
        datastart,mediadata=next((at+8,body) for tag,at,body in chunks(data) if tag==b'DATA')
        media=[]
        for sid in sounds:
            s=objs[sid];assert s['type']==2
            sb=bytes.fromhex(s['body_hex']);plugin,stream,source,memory,bits=struct.unpack_from('<IBIIB',sb)
            assert stream==0 and plugin&15==1 and bits&1==0
            slots=[x for x in didx if x['id']==source];assert len(slots)==1
            slot=slots[0];assert slot['offset']+slot['bytes']<=len(mediadata)
            raw=mediadata[slot['offset']:slot['offset']+slot['bytes']]
            assert len(raw)==memory and raw[:4]==b'RIFF' and raw[8:12]==b'WAVE' and len(raw)==struct.unpack_from('<I',raw,4)[0]+8
            parts={tag:body for tag,_,body in chunks(raw,12)};codec,ch,rate=struct.unpack_from('<HHI',parts[b'fmt '])
            media.append(dict(sound_id=sid,plugin_id=plugin,stream_type=stream,source_id=source,source_bytes=len(raw),source_sha256=hashlib.sha256(raw).hexdigest(),bank_data_offset=datastart+slot['offset'],physical_offset=info['entry']['offset']+datastart+slot['offset'],codec=codec,channels=ch,rate=rate))
        # Decode one original variant per event, retaining all variant provenance above.
        chosen=media[0];raw=mediadata[next(x['offset'] for x in didx if x['id']==chosen['source_id']):][:chosen['source_bytes']]
        stem=OUT/f'audio-sample-{name}';wem=stem.with_suffix('.wem');wav=stem.with_suffix('.wav');ogg=stem.with_suffix('.ogg')
        wem.write_bytes(raw)
        proc=subprocess.run([str(VGM),'-i','-o',str(wav),str(wem)],capture_output=True,text=True,check=True)
        with wave.open(str(wav),'rb') as w:
            params=w.getparams();pcm=w.readframes(w.getnframes())
        assert params.sampwidth==2 and params.nchannels==chosen['channels'] and params.framerate==chosen['rate']
        values=struct.unpack('<'+'h'*(len(pcm)//2),pcm);peak=max(map(abs,values));rms=math.sqrt(sum(v*v for v in values)/len(values))/32768
        subprocess.run(['ffmpeg','-v','error','-y','-i',str(wav),'-c:a','libvorbis','-q:a','4',str(ogg)],check=True)
        converted=json.loads(subprocess.run(['ffprobe','-v','error','-show_entries','stream=sample_rate,channels:format=duration','-of','json',str(ogg)],capture_output=True,text=True,check=True).stdout)
        measurement=dict(source_id=chosen['source_id'],duration=params.nframes/params.framerate,samples=params.nframes,channels=params.nchannels,rate=params.framerate,peak=peak/32768,rms_dbfs=20*math.log10(rms) if rms else None,wav_sha256=hashlib.sha256(wav.read_bytes()).hexdigest(),ogg_bytes=ogg.stat().st_size,ogg_sha256=hashlib.sha256(ogg.read_bytes()).hexdigest(),ogg_probe=converted,vgmstream_stdout=proc.stdout)
        # Keep just two compact audible feasibility samples; all full raw data remains reproducible.
        keep=name in ['gman_hengdao_attack_light_01','gman_all_weapon_touch_shock_katana']
        wem.unlink();wav.unlink()
        if not keep:ogg.unlink()
        report.append(dict(event=name,event_id=fnv(name),bank=bank_name,bank_id=fnv(bank_name),bank_sha256=info['sha256'],bank_entry=info['entry'],package=info['package'],event_record_offset=row['offset'],event_record_bytes=row['bytes'],event_record_sha256=hashlib.sha256(row['raw']).hexdigest(),paths=paths,media=media,decoded=measurement,retained_sample=ogg.name if keep else None))
    result=dict(source_dbbl_sha256=hashlib.sha256(dbbl).hexdigest(),scope='Selected event chains; source name is not alone semantic proof. Switch selections follow original default and remain explicitly marked. No original parameter/RTPC/timing reconstruction claimed.',decoder=str(VGM),decoder_sha256=decoder_sha,conversion='ffmpeg -v error -y -i full-source.wav -c:a libvorbis -q:a 4 output.ogg; no normalization, trimming or splicing',chains=report)
    (OUT/'audio_sample_trace.json').write_text(json.dumps(result,ensure_ascii=False,indent=2)+'\n')
    print(json.dumps([dict(event=r['event'],variants=len(r['media']),seconds=r['decoded']['duration'],ogg_bytes=r['decoded']['ogg_bytes']) for r in report],indent=2))

if __name__=='__main__':main()
