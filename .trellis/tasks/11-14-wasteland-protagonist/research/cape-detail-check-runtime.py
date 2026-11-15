import hashlib
import io
import json
import math
import struct
from collections import defaultdict
from itertools import product
from pathlib import Path

import numpy as np
from PIL import Image

root=Path.cwd(); runtime=root/'art/wasteland/runtime'; authored=json.loads((runtime/'authored-mesh.json').read_text()); proof=json.loads((runtime/'conversion-report.json').read_text())
raw=(root/'assets/characters/zhuque.bin').read_bytes(); offset=0

def take(n):
 global offset
 assert 0<=n<=len(raw)-offset
 out=raw[offset:offset+n];offset+=n;return out

def read(f):return struct.unpack('<'+f,take(struct.calcsize('<'+f)))
def string():return take(read('H')[0]).decode()
def mat():
 x=read('16f');return [[x[c*4+r] for c in range(4)]for r in range(4)]
def multiply(a,b):return [[sum(a[r][k]*b[k][c]for k in range(4))for c in range(4)]for r in range(4)]
assert take(8)==b'KIVAAV04';header=json.loads(take(read('I')[0]));assert header==authored['header']
assert header['triangle_budget']==150000
assert header['development_projected_uv']==[] and header['development_material_factors_only'] is False
assert header['source_blend_sha256']==hashlib.sha256((root/'art/wasteland/cape-refined/zhuque-cape-runtime-source.blend').read_bytes()).hexdigest()
assert header['reference_sha256']==hashlib.sha256((root/'art/wasteland/reference-rig.json').read_bytes()).hexdigest()
assert header['pose_sha256']==hashlib.sha256((root/'art/wasteland/greybox/runtime-poses.json').read_bytes()).hexdigest()
reference=json.loads((root/'art/wasteland/reference-rig.json').read_text()); rest={n['name']:n['source_rest_game']for n in reference['nodes']}
drivers=[(string(),read('3f'))for _ in range(read('H')[0])];assert len(drivers)==23
for (name,p),driver in zip(drivers,reference['drivers']):assert name==driver['name'] and max(abs(x-y)for x,y in zip(p,driver['position_game']))<1e-7
source_pixels=defaultdict(list)
for folder in [root/'art/wasteland/textured/textures',root/'art/wasteland/cape-refined/textures']:
 for source in folder.glob('*.png'):
  pixels=Image.open(source).convert('RGBA').tobytes()
  source_pixels[hashlib.sha256(pixels).hexdigest()].append(str(source.relative_to(root)))
textures=[];texture_payloads=[];maxpixel=0
for _ in range(read('H')[0]):
 name=string();srgb,length=read('BI');png=take(length);assert srgb in(0,1)
 img=Image.open(io.BytesIO(png));img.load();assert img.size==(2048,2048)
 pixel_sha=hashlib.sha256(img.convert('RGBA').tobytes()).hexdigest()
 assert pixel_sha in source_pixels,(name,'PNG pixels do not match any frozen source map')
 textures.append({'name':name,'srgb':bool(srgb),'bytes':length,'sha256':hashlib.sha256(png).hexdigest(),'matching_source_pngs':source_pixels[pixel_sha]})
 texture_payloads.append(png)
assert len(textures)==20
materials=[]
for _ in range(read('H')[0]):
 name=string();base,normal,orm,mask=read('hhhB');factors=read('11f');double,mode=read('BB');opacity,*emission=read('4f');emissive,effect=read('hB')
 assert all(math.isfinite(v)for v in(*factors,opacity,*emission))
 assert mask==0 and double==1 and mode in(0,2) and effect in(0,1)
 assert all(0<=i<20 for i in(base,normal,orm,emissive))
 assert [textures[i]['srgb']for i in(base,normal,orm,emissive)]==[True,False,False,True]
 source_material=next(m for m in authored['materials'] if m['name']==name)
 assert mode==source_material['alpha_mode'] and effect==source_material['surface_effect']
 assert (effect==0 and mode==0) or (effect==1 and mode==2)
 materials.append({'name':name,'base':base,'normal':normal,'orm':orm,'emission':emissive,'alpha_mode':mode,'effect':effect,'emissive_factor':emission})
assert len(materials)==5
vertices=triangles=0;maxbind=0;maxnormal=0
for pi in range(read('H')[0]):
 name=string();blade,bones=read('BH');assert blade==0 and 0<bones<=52
 bone_names=[]
 for _ in range(bones):
  bone_name=string().rsplit('::',1)[-1];driver=read('H')[0];inverse=mat();reference_matrix=mat()
  assert driver<23 and bone_name in rest
  assert max(abs(reference_matrix[r][c]-rest[bone_name][r][c])for r in range(4)for c in range(4))<1e-6
  bind=multiply(reference_matrix,inverse);maxbind=max(maxbind,max(abs(bind[r][c]-float(r==c))for r in range(4)for c in range(4)));bone_names.append(bone_name)
 sourcepart=authored['parts'][pi];assert name=='naraka::zhuque::'+sourcepart['name']
 for pri in range(read('H')[0]):
  mi,vc,ic=read('HII');assert mi<5 and vc>0 and ic>0 and ic%3==0
  sourceprimitive=sourcepart['primitives'][pri];assert materials[mi]['name']==sourceprimitive['material'];assert vc==len(sourceprimitive['vertices'])
  for vi in range(vc):
   vals=read('8f8B');pos,norm,uv=vals[:3],vals[3:6],vals[6:8];indices,weights=vals[8:12],vals[12:16]
   assert all(math.isfinite(v)for v in vals[:8]);assert all(i<bones for i in indices);assert sum(weights)==255
   length=math.sqrt(sum(v*v for v in norm));maxnormal=max(maxnormal,abs(length-1));assert abs(length-1)<1e-5
   sv=sourceprimitive['vertices'][vi]
   assert max(abs(a-b)for a,b in zip((*pos,*norm,*uv),(*sv['position'],*sv['normal'],*sv['uv'])))<1e-6
   assert list(weights)==sv['weights']
   for i,w,j in zip(indices,weights,sv['joints']):
    if w: assert bone_names[i]==authored['bones'][j]['name']
  indices=read(str(ic)+'I');assert all(i<vc for i in indices);assert list(indices)==sourceprimitive['indices']
  vertices+=vc;triangles+=ic//3
assert offset==len(raw) and triangles==139152 and maxbind<1e-5
assert proof['runtime_sha256']==hashlib.sha256(raw).hexdigest()
assert proof['source_sha256']==header['source_blend_sha256']
assert proof['glb_sha256']==hashlib.sha256((runtime/'zhuque.glb').read_bytes()).hexdigest()
glbraw=(runtime/'zhuque.glb').read_bytes()
assert struct.unpack_from('<4sII',glbraw)==(b'glTF',2,len(glbraw))
chunks={};at=12
while at<len(glbraw):
 size,kind=struct.unpack_from('<I4s',glbraw,at);assert size%4==0
 chunks[kind]=glbraw[at+8:at+8+size];at+=8+size
assert at==len(glbraw)
doc=json.loads(chunks[b'JSON']);glbbin=chunks[b'BIN\0']; orm_count=0
for material in doc['materials']:
 mr=material['pbrMetallicRoughness']['metallicRoughnessTexture']; ao=material['occlusionTexture']
 assert ao['index']==mr['index'] and ao.get('texCoord',0)==mr.get('texCoord',0) and ao.get('strength',1)==1
 image=doc['images'][doc['textures'][mr['index']]['source']];view=doc['bufferViews'][image['bufferView']]
 data=glbbin[view.get('byteOffset',0):view.get('byteOffset',0)+view['byteLength']]
 decoded=next(m for m in materials if m['name']==material['name'])
 assert data==texture_payloads[decoded['orm']],material['name']
 orm_count+=1
assert orm_count==5

# Independent actual GLB skinning, topology count and symmetric geometry coverage.
def accessor(index):
 entry=doc['accessors'][index];view=doc['bufferViews'][entry['bufferView']]
 components={'SCALAR':1,'VEC2':2,'VEC3':3,'VEC4':4,'MAT4':16}[entry['type']]
 kind={5121:'u1',5123:'<u2',5125:'<u4',5126:'<f4'}[entry['componentType']]
 dt=np.dtype(kind);start=view.get('byteOffset',0)+entry.get('byteOffset',0)
 arr=np.ndarray((entry['count'],components),dtype=dt,buffer=glbbin,offset=start,strides=(view.get('byteStride',dt.itemsize*components),dt.itemsize)).copy()
 if entry.get('normalized') and kind!='<f4': arr=arr/np.iinfo(dt).max
 return arr
parents={child:i for i,node in enumerate(doc['nodes'])for child in node.get('children',[])};worlds={}
def world(index):
 if index in worlds:return worlds[index]
 node=doc['nodes'][index]
 if 'matrix'in node:local=np.array(node['matrix']).reshape((4,4),order='F')
 else:
  x,y,z,w=node.get('rotation',[0,0,0,1]);local=np.eye(4)
  local[:3,:3]=np.array([[1-2*(y*y+z*z),2*(x*y-z*w),2*(x*z+y*w)],[2*(x*y+z*w),1-2*(x*x+z*z),2*(y*z-x*w)],[2*(x*z-y*w),2*(y*z+x*w),1-2*(x*x+y*y)]])@np.diag(node.get('scale',[1,1,1]))
  local[:3,3]=node.get('translation',[0,0,0])
 worlds[index]=world(parents[index])@local if index in parents else local
 return worlds[index]
glb_vertices=[];glb_triangles=0;glb_meshes=0
for ni,node in enumerate(doc['nodes']):
 if 'mesh'not in node:continue
 assert 'skin'in node
 skin=doc['skins'][node['skin']];ibms=accessor(skin['inverseBindMatrices'])
 palette=np.array([world(joint)@bind.reshape((4,4),order='F')for joint,bind in zip(skin['joints'],ibms)])
 for primitive in doc['meshes'][node['mesh']]['primitives']:
  assert primitive.get('mode',4)==4
  attrs=primitive['attributes'];points=accessor(attrs['POSITION']);joints=accessor(attrs['JOINTS_0']).astype(int);weights=accessor(attrs['WEIGHTS_0'])
  assert np.isfinite(points).all() and np.max(np.abs(weights.sum(axis=1)-1))<1e-5
  assert joints.min()>=0 and joints.max()<len(palette)
  full=np.concatenate((points,np.ones((len(points),1))),axis=1)
  posed=np.einsum('nvij,nj,nv->ni',palette[joints],full,weights)[:,:3]
  assert np.isfinite(posed).all();glb_vertices.append(posed)
  indices=accessor(primitive['indices']);assert len(indices)%3==0 and indices.max()<len(points)
  glb_triangles+=len(indices)//3
 glb_meshes+=1
assert glb_triangles==triangles,'GLB omitted evaluated cloth thickness/topology'
source_points=np.array([v['position']for part in authored['parts']for primitive in part['primitives']for v in primitive['vertices']])
glb_points=np.concatenate(glb_vertices,axis=0)
def coverage(query,target):
 cell=1e-4;tolerance=2e-6
 query=np.unique(query,axis=0);target=np.unique(target,axis=0);grid=defaultdict(list)
 for value in target:grid[tuple(np.floor(value/cell).astype(int))].append(value)
 offsets=list(product((-1,0,1),repeat=3));maximum=0
 for value in query:
  key=tuple(np.floor(value/cell).astype(int));distance=float('inf')
  for offset in offsets:
   for candidate in grid.get(tuple(key[k]+offset[k]for k in range(3)),()):
    delta=value-candidate;distance=min(distance,float(delta@delta))
  assert distance<=tolerance*tolerance,('missing evaluated bind vertex',value,distance)
  maximum=max(maximum,distance**.5)
 return {'unique_query_vertices':len(query),'unique_target_vertices':len(target),'maximum_nearest_error_m':maximum}
forward=coverage(glb_points,source_points);reverse=coverage(source_points,glb_points)
report={'runtime_sha256':hashlib.sha256(raw).hexdigest(),'bytes':len(raw),'strict_development_flags_absent':True,'actual_glb_explicit_ORM_AO_materials_checked':orm_count,'glb_sha256':hashlib.sha256(glbraw).hexdigest(),'textures':textures,'materials':materials,'serialized_vertices':vertices,'triangles':triangles,'actual_GLB_triangles':glb_triangles,'actual_GLB_skinned_meshes':glb_meshes,'actual_GLB_bind_to_runtime':forward,'runtime_to_actual_GLB_bind':reverse,'maximum_own_rest_inverse_product_error':maxbind,'maximum_unit_normal_error':maxnormal,'maximum_export_vs_source_png_pixel_delta_u8':maxpixel,'eof_exact':True,'source_bind_UV_weights_and_indices_match_authored_mesh':True,'limits':'Actual binary/material/PNG/source correspondence; motionLBS and GPU visual acceptance independently recorded elsewhere.'}
Path('.trellis/tasks/11-14-wasteland-protagonist/research/cape-detail-check-runtime-final.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
