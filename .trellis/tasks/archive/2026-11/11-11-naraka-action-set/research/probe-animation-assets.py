#!/usr/bin/env python3
"""Read-only, bounded Naraka animation object probe; outputs beside this script."""
from pathlib import Path
import argparse, collections, hashlib, json, sqlite3, sys

OUT = Path(__file__).resolve().parent
BASE = Path('/home/asuka/kiva-asset-research/naraka')
SOURCE = Path('/mnt/c/Program Files (x86)/Steam/steamapps/common/NARAKA BLADEPOINT/NarakaBladepoint_Data/StreamingAssets')
sys.path.insert(0, str(BASE/'containers'))
from naraka_sparse import SparseBundle
import UnityPy

def json_default(value):
    if isinstance(value,(bytes,bytearray)): return {'bytes':len(value),'hex':bytes(value).hex()}
    raise TypeError(type(value).__name__)

def run(name,metadata_only=False):
    db=sqlite3.connect('file:'+str(BASE/'identity/appres.sqlite')+'?mode=ro',uri=True)
    rows=db.execute('select r.path,b.key,b.declared_size from resources r join bundles b on r.bundle_id=b.id where r.path like ?',('%/'+name,)).fetchall()
    if len(rows)!=1:raise ValueError(rows)
    path,key,size=rows[0]
    # animation_info is a verified 10,394,312-byte object; no blanket full-game limit increase.
    limits={'max_read':16*1024**2} if name=='animation_info_archive.bytes' else {}
    # Verified object 4241003489660708546 is 17,542,124 bytes.
    if name=='playable_runtime_override_mapping_archive.bytes':limits={'max_read':32*1024**2}
    with SparseBundle(SOURCE/key,**limits) as bundle:
        env=UnityPy.Environment()
        cab=next(n['name'] for n in bundle.nodes if n['flags']&4)
        sf=env.load_file(bundle.open_node(cab),name=cab)
        ptr=sf.container[path]
        obj=sf.objects[ptr.m_PathID]
        tree=obj.read_typetree(check_read=True)
        raw=obj.get_raw_data()
        if obj.type.name=='TextAsset':
            value=tree['m_Script']
            payload=value.encode('utf-8',errors='surrogateescape') if isinstance(value,str) else bytes(value)
            (OUT/(name+'.payload.bin')).write_bytes(payload)
            tree['m_Script']=dict(bytes=len(payload),sha256=hashlib.sha256(payload).hexdigest(),prefix_hex=payload[:160].hex())
        elif obj.type.name=='AnimationClip' and metadata_only:
            muscle=tree['m_MuscleClip'];curves=muscle['m_Clip']['data']
            tree=dict(m_Name=tree['m_Name'],m_SampleRate=tree['m_SampleRate'],m_Events=tree['m_Events'],
                      duration=muscle['m_StopTime']-muscle['m_StartTime'],m_LoopTime=muscle['m_LoopTime'],
                      scalar_curves=dict(streamed=curves['m_StreamedClip']['curveCount'],dense=curves['m_DenseClip']['m_CurveCount'],constant=len(curves['m_ConstantClip']['data'])),
                      dense_frames=curves['m_DenseClip']['m_FrameCount'],
                      binding_types=dict(collections.Counter(str((b['typeID'],b['customType'])) for b in tree['m_ClipBindingConstant']['genericBindings'])),
                      humanoid_muscles=sum(b['typeID']==95 and 42<=b['attribute']<=136 for b in tree['m_ClipBindingConstant']['genericBindings']))
        report=dict(path=path,bundle_key=key,cab=cab,declared_size=size,path_id=obj.path_id,type=obj.type.name,
                    byte_start=obj.byte_start,byte_size=obj.byte_size,raw_sha256=hashlib.sha256(raw).hexdigest(),
                    old_type_hash=obj.serialized_type.old_type_hash.hex(),
                    type_counts=dict(collections.Counter(o.type.name for o in sf.objects.values())),
                    externals=[dict(file_id=i+1,path=r.path) for i,r in enumerate(sf.externals)],tree=tree,
                    proof=bundle.report())
        target=OUT/(name+'.probe.json')
        target.write_text(json.dumps(report,ensure_ascii=False,indent=2,default=json_default)+'\n')
        print(json.dumps({k:report[k] for k in ('path','bundle_key','path_id','type','byte_size','type_counts')},ensure_ascii=False))
        print('tree keys',list(tree))
        print('output',target)

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('names',nargs='+');p.add_argument('--metadata-only',action='store_true');args=p.parse_args()
    for name in args.names:run(name,args.metadata_only)
