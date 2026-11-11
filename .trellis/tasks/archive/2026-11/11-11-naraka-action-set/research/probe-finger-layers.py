#!/usr/bin/env python3
"""Read original finger masks/values; does not invent original layer order/weights."""
from pathlib import Path
import hashlib, importlib.util, json, math, sqlite3, struct, sys
ROOT=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('asset_probe',ROOT/'probe-animation-assets.py');a=importlib.util.module_from_spec(spec);spec.loader.exec_module(a)

def samples(tree,times):
    clip=tree['m_MuscleClip']['m_Clip']['data'];st=clip['m_StreamedClip'];de=clip['m_DenseClip'];co=clip['m_ConstantClip']['data'];raw=struct.pack('<'+'I'*len(st['data']),*st['data']);at=0;frames=[]
    while at<len(raw):
        time,n=struct.unpack_from('<fi',raw,at);at+=8;keys=[]
        for _ in range(n):
            idx,*coeff=struct.unpack_from('<i4f',raw,at);at+=20;keys.append((idx,coeff))
        frames.append((time,keys))
    if at!=len(raw):raise ValueError('stream EOF')
    outputs=[]
    for time in times:
        vals=[None]*st['curveCount']
        for ft,keys in frames:
            if ft>time:break
            for idx,c in keys:
                delta=time-ft if ft>-1e20 else 0.
                vals[idx]=((c[0]*delta+c[1])*delta+c[2])*delta+c[3]
        cc=de['m_CurveCount'];f=(time-de['m_BeginTime'])*de['m_SampleRate'];lo=max(0,min(de['m_FrameCount']-1,math.floor(f)));hi=min(de['m_FrameCount']-1,lo+1);mix=max(0,min(1,f-lo))
        vals.extend(de['m_SampleArray'][lo*cc+i]*(1-mix)+de['m_SampleArray'][hi*cc+i]*mix for i in range(cc));vals.extend(co)
        if not all(v is not None and math.isfinite(v) for v in vals):raise ValueError('nonfinite sample')
        out={};index=0
        for binding in tree['m_ClipBindingConstant']['genericBindings']:
            width={1:3,2:4,3:3}.get(binding['attribute'],1) if binding['typeID']==4 else 1
            if binding['typeID']==95 and 42<=binding['attribute']<=136:out[binding['attribute']]=vals[index]
            index+=width
        if index!=len(vals):raise ValueError('binding width')
        outputs.append(out)
    return outputs

def main():
    names=['male_hengdao_finger_left_01','male_hengdao_finger_right_01','male_bow_finger_default_left_01','male_bow_finger_default_right_01','male_hengdao_idle_01','male_hengdao_jump_down_loop_01','male_hengdao_attack_light_01']
    db=sqlite3.connect('file:'+str(a.BASE/'identity/appres.sqlite')+'?mode=ro',uri=True);rows=[]
    for name in names:
        matches=db.execute('select r.path,b.key from resources r join bundles b on b.id=r.bundle_id where r.path like ?',('%/'+name+'.anim',)).fetchall()
        if len(matches)!=1:raise ValueError(matches)
        path,key=matches[0]
        with a.SparseBundle(a.SOURCE/key) as bundle:
            cab=next(n['name'] for n in bundle.nodes if n['flags']&4);env=a.UnityPy.Environment();sf=env.load_file(bundle.open_node(cab),name=cab);obj=sf.objects[sf.container[path].m_PathID];tree=obj.read_typetree(check_read=True);raw=obj.get_raw_data();duration=tree['m_MuscleClip']['m_StopTime']-tree['m_MuscleClip']['m_StartTime'];times=[0,duration/2,duration];ss=samples(tree,times)
            rows.append(dict(name=name,path=path,bundle_key=key,cab=cab,path_id=obj.path_id,object_sha256=hashlib.sha256(raw).hexdigest(),duration=duration,
                             muscle_attribute_mask=sorted(ss[0]),samples=[dict(time=t,values=v) for t,v in zip(times,ss)],source_proof=bundle.report()))
    left=set(rows[0]['muscle_attribute_mask']);right=set(rows[1]['muscle_attribute_mask'])
    if len(left)!=20 or len(right)!=20 or left&right:raise ValueError('finger masks')
    for row in rows:
        mask=set(row['muscle_attribute_mask']);row['left_finger_present_attributes']=sorted(mask&left);row['right_finger_present_attributes']=sorted(mask&right)
        row['authored_zero_attributes_at_start']=[k for k,v in row['samples'][0]['values'].items() if v==0.]
    report=dict(mask_semantics='Left/right sets proven by actual left/right named source clip bindings. Presence is separate from authored zero. No original layer order, weight, additive mode or AvatarMask recovered here.',left_attributes=sorted(left),right_attributes=sorted(right),clips=rows)
    (ROOT/'finger-layer-values.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
    for r in rows:print(r['name'],'muscles',len(r['muscle_attribute_mask']),'left',len(r['left_finger_present_attributes']),'right',len(r['right_finger_present_attributes']))

if __name__=='__main__':main()
