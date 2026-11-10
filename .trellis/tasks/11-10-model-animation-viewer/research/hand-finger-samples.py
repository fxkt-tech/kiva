"""Read source local finger quaternions, semantic names, and rest segment facts."""
from bisect import bisect_right
import importlib.util
import json
import math
from pathlib import Path
import sys

sys.dont_write_bytecode = True

SPEC = importlib.util.spec_from_file_location('hand_asset_counts', Path(__file__).with_name('hand-asset-counts.py'))
COUNTS = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(COUNTS)
ROOT, Reader = COUNTS.ROOT, COUNTS.Reader


def normalize(q):
    length = math.sqrt(sum(v * v for v in q))
    return [v / length for v in q]


def dot(a, b):
    return sum(x * y for x, y in zip(a, b))


def angle(a, b):
    return math.degrees(2 * math.acos(min(1., abs(dot(normalize(a), normalize(b))))))


def slerp(a, b, weight):
    a, b = normalize(a), normalize(b)
    product = dot(a, b)
    if product < 0:
        b, product = [-v for v in b], -product
    if product > .9995:
        return normalize([x + (y - x) * weight for x, y in zip(a, b)])
    theta = math.acos(max(-1., min(1., product)))
    return normalize([(x * math.sin((1 - weight) * theta) + y * math.sin(weight * theta)) / math.sin(theta) for x, y in zip(a, b)])


def sample(keys, time, rest):
    if not keys:
        return rest
    end = bisect_right([key[0] for key in keys], time)
    if end == 0:
        return list(keys[0][1:])
    if end == len(keys):
        return list(keys[-1][1:])
    a, b = keys[end-1], keys[end]
    return slerp(a[1:], b[1:], (time-a[0])/(b[0]-a[0]))


def matrix(trs):
    tx, ty, tz, x, y, z, w, sx, sy, sz = trs
    return [[(1-2*(y*y+z*z))*sx, 2*(x*y-z*w)*sy, 2*(x*z+y*w)*sz, tx],
            [2*(x*y+z*w)*sx, (1-2*(x*x+z*z))*sy, 2*(y*z-x*w)*sz, ty],
            [2*(x*z-y*w)*sx, 2*(y*z+x*w)*sy, (1-2*(x*x+y*y))*sz, tz], [0,0,0,1]]


def multiply(a,b):
    return [[sum(a[r][k]*b[k][c] for k in range(4)) for c in range(4)] for r in range(4)]


def models(joints, basis):
    result=[]
    for joint in joints:
        parent=joint['parent']
        result.append(multiply(basis if parent==65535 else result[parent],matrix(joint['trs'])))
    return {joint['name']: [model[i][3] for i in range(3)] for joint,model in zip(joints,result)}


def source():
    reader=Reader(ROOT/'assets/choulingke/animations.bin')
    assert reader.bytes(8)==b'KIVASR01'
    n,k=reader.read('HH'); joints=[]
    for _ in range(n):
        joints.append({'name':reader.string(),'parent':reader.read('H')[0],'trs':reader.read('10f')})
    clips={}
    for _ in range(k):
        name=reader.string();duration,tracks=reader.read('HH'); values={}
        for _ in range(tracks):
            joint=reader.read('H')[0]
            for width in [3,4,3]:
                count=reader.read('H')[0]
                keys=[reader.read('H'+str(width)+'f') for _ in range(count)]
                if width==4:
                    values[joints[joint]['name']]=keys
        if name=='skill_ylk_1':
            clips[name]={'duration_ms':duration,'rotations':values}
    assert reader.offset==len(reader.data)
    return joints,clips


def cc0_rest():
    reader=Reader(ROOT/'assets/animations/stickman.anim');assert reader.bytes(8)==b'KIVAAN01'
    n,_=reader.read('HH'); joints=[]
    for _ in range(n):
        joints.append({'name':reader.string(),'parent':reader.read('H')[0],'trs':reader.read('10f')})
    # Match AnimationLibrary::model_pose basis (Y=PI; uniform scale1.1).
    basis=[[-1.1,0,0,0],[0,1.1,0,0],[0,0,-1.1,0],[0,0,0,1]]
    return models(joints,basis)


def direction(a,b):
    delta=[y-x for x,y in zip(a,b)];length=math.sqrt(dot(delta,delta))
    return [v/length for v in delta]


if __name__=='__main__':
    joints,clips=source();by_name={joint['name']:joint for joint in joints}
    source_rest=models(joints,[[1,0,0,0],[0,1,0,0],[0,0,1,0],[0,0,0,1]])
    samples=[]
    clip=clips['skill_ylk_1']; times=[0.,.2,.9,1.35,1.8]
    for side in ['L','R']:
        for finger in range(5):
            for tail in ['', '1','2']:
                name=f'Bip001 {side} Finger{finger}{tail}'
                rest=by_name[name]['trs'][3:7]
                quats=[sample(clip['rotations'].get(name,[]),min(t*1000,clip['duration_ms']),rest) for t in times]
                samples.append({'name':name,'source_parent':joints[by_name[name]['parent']]['name'],
                                'times_seconds':times,'local_quaternions_xyzw':quats,
                                'angle_degrees_from_rest':[angle(rest,q) for q in quats],
                                'angle_degrees_from_first_sample':[angle(quats[0],q) for q in quats]})
    counts=json.loads(Path(__file__).with_name('hand-asset-counts.json').read_text())
    target_names={}
    target_twists={}
    segments=[]; donor=cc0_rest()
    for name in ['lihuowang','bailingmiao']:
        body=counts['characters'][name]; anchors={d['name']:d['anchor'] for d in body['drivers']}; names={}
        for part in body['parts']:
            for bone in part['hand_related_bones']:
                suffix=bone['name'].split('::')[-1]
                names.setdefault(suffix,[]).append({'palette_part':part['name'],'full_name':bone['name'],'rest_position':bone['rest_model_position']})
        chains=[]
        for side in ['L','R']:
            for finger in range(5):
                chain=[f'gMan {side} Finger{finger}{tail}' for tail in ['', '1','2']]
                assert all(node in names for node in chain)
                chains.append({'source':[f'Bip001 {side} Finger{finger}{tail}' for tail in ['', '1','2']], 'target_semantic_chain':chain})
            lower=side.lower()
            for parent,child in [('upperarm','lowerarm'),('lowerarm','hand')]:
                a,b=f'{parent}_{lower}',f'{child}_{lower}'
                target=direction(anchors[a],anchors[b]);source_dir=direction(donor[a],donor[b])
                segments.append({'appearance':name,'segment':a+'->'+b,'donor_reference_direction':source_dir,'target_anchor_direction':target,
                                 'direction_difference_degrees':math.degrees(math.acos(max(-1.,min(1.,dot(target,source_dir)))))})
        target_names[name]=chains
        target_twists[name]={suffix:entries for suffix,entries in names.items() if 'Twist' in suffix or suffix.endswith(('UpperArm','Forearm','Hand'))}
    source_twists={joint['name']:{'parent':joints[joint['parent']]['name'],'rest_position':source_rest[joint['name']]} for joint in joints if joint['name'] in [f'Bone {side} {part} Twist{tail}' for side in ['L','R'] for part in ['UpperArm','ForeArm'] for tail in ['', '1']]}
    out={'sampled_clip':'skill_ylk_1','duration_ms':clip['duration_ms'],'local_finger_samples':samples,
         'semantic_finger_chains':target_names,'target_twist_palette_positions':target_twists,
         'source_twist_positions':source_twists,'cc0_reference_positions':donor,'rest_segment_comparison':segments}
    target=Path(__file__).with_suffix('.json');target.write_text(json.dumps(out,indent=2)+'\n')
    for record in samples:
        if record['name'] in ['Bip001 L Finger11','Bip001 L Finger12','Bip001 R Finger11','Bip001 R Finger12','Bip001 R Finger01']:
            print(record['name'],'from_rest_deg',[round(x,2) for x in record['angle_degrees_from_rest']], 'from_t0_deg',[round(x,2) for x in record['angle_degrees_from_first_sample']])
    print('Rest segment angles:',[(s['appearance'],s['segment'],round(s['direction_difference_degrees'],3)) for s in segments])
    print('Wrote',target)
