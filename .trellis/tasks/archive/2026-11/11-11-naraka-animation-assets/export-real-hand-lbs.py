"""Export actual pinned source hand vertices/IBMs for native BakeMesh checks."""
from pathlib import Path
import hashlib, importlib.util, json
import numpy as np

ROOT=Path(__file__).resolve().parents[3]
spec=importlib.util.spec_from_file_location('existing_naraka_import',ROOT/'scripts/import-naraka.py')
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
source=Path('/home/asuka/kiva-asset-research/naraka/export/character/lihuowang_s20_outfit.glb')
g=m.Glb(source,m.SOURCES['outfit'][1])
rig=json.loads((ROOT/'assets/prepared/naraka/animations/source/rig.json').read_text())
known={n['name'] for n in rig['nodes']}
mesh=next(x for x in g.meshes() if x['name']=='body')
names=[g.doc['nodes'][i]['name'] for i in mesh['skin']['joints']]
records=[]
X=np.diag([-1.,1.,1.,1.])
for pi,p in enumerate(mesh['primitives']):
    joints=p['joints'];weights=p['weights']
    hand=np.array([any(s in n for s in ['Hand','Finger','Forearm']) for n in names])
    supported=np.array([n in known for n in names])
    choose=((hand[joints]*weights).sum(1)>.98) & (((~supported[joints])*weights).sum(1)<1e-10)
    ids=np.flatnonzero(choose)
    if not len(ids):continue
    selected_joints=joints[ids];selected_weights=weights[ids]
    used=sorted(set(int(j) for js,ws in zip(selected_joints,selected_weights) for j,w in zip(js,ws) if w>0))
    remap={j:i for i,j in enumerate(used)}
    indices=np.full(len(p['POSITION']),-1,dtype=np.int64);indices[ids]=np.arange(len(ids))
    tris=p['indices'].reshape((-1,3));tris=tris[(indices[tris]>=0).all(1)]
    newj=np.array([[remap[int(j)] if w>0 else 0 for j,w in zip(js,ws)] for js,ws in zip(selected_joints,selected_weights)])
    positions=p['POSITION'][ids].copy();positions[:,0]*=-1 # glTF->original rawUnity X reflection
    binds=[X @ mesh['binds'][j] @ X for j in used]
    records.append({'primitive':pi,'bone_names':[names[j] for j in used],
                    'positions':positions.reshape(-1).tolist(),'joints':newj.reshape(-1).tolist(),
                    'weights':selected_weights.reshape(-1).tolist(),
                    'bindposes_column_major':np.array([b.T.reshape(-1) for b in binds]).reshape(-1).tolist(),
                    'triangles':indices[tris][:,[0,2,1]].reshape(-1).tolist(),
                    'source_vertex_ids':ids.tolist(),'source_palette_indices':used})
output=Path('/home/asuka/kiva-asset-research/naraka/action-set-2026-11-11/native-bundles/real-hand-lbs.json')
result={'format':'KIVA_REAL_HAND_LBS_1','source_glb':str(source),'source_glb_sha256':hashlib.sha256(source.read_bytes()).hexdigest(),
        'source_mesh':'body','coordinate_system':'original rawUnity; glTF X reflection reversed once',
        'skin_equation':'renderer at world identity: sum(weight * donorBoneWorld * originalIBM * originalVertex)',
        'limitation':'actual original vertex/weight/IBM data selected by supported donor bone names, cross-rig test binding; not original actor render-component assembly',
        'primitives':records}
output.write_text(json.dumps(result,ensure_ascii=False,separators=(',',':'))+'\n')
print(output)
print('primitives',len(records),'vertices',sum(len(r['source_vertex_ids']) for r in records),'triangles',sum(len(r['triangles'])//3 for r in records),'bytes',output.stat().st_size)
