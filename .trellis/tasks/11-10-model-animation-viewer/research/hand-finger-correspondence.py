"""Persist semantic chain, actual weighted-leaf, and twist-position evidence."""
import importlib.util
import json
import math
from pathlib import Path
import struct
import sys

sys.dont_write_bytecode = True

BASE = Path(__file__).resolve().parent
SPEC = importlib.util.spec_from_file_location('hand_sample', BASE / 'hand-finger-samples.py')
M = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(M)


if __name__ == '__main__':
    joints, _ = M.source()
    source_positions = M.models(joints, [[1,0,0,0],[0,1,0,0],[0,0,1,0],[0,0,0,1]])
    counts = json.loads((BASE / 'hand-asset-counts.json').read_text())
    weights = {}
    for who, body in counts['characters'].items():
        for part in body['parts']:
            for bone in part['hand_related_bones']:
                key = (who, bone['name'].split('::')[-1])
                entry = weights.setdefault(key, {'slots':0, 'weight255':0, 'rest_position':bone['rest_model_position']})
                entry['slots'] += bone['weighted_vertex_slots']
                entry['weight255'] += bone['weights255_sum']
    result = {'fingers':[], 'twists':{}, 'target_parent_verification':{}}
    for side in ['L','R']:
        for digit in range(5):
            source = [f'Bip001 {side} Finger{digit}{tail}' for tail in ['', '1','2','Nub']]
            targets = [f'gMan {side} Finger{digit}{tail}' for tail in ['', '1','2']]
            entry = {'side':side, 'digit':digit, 'source_names':source,
                     'source_rest_positions':[source_positions[name] for name in source],
                     'source_segment_lengths':[math.dist(source_positions[a],source_positions[b]) for a,b in zip(source,source[1:])],
                     'source_weighted_slots':[weights.get(('choulingke',name),{}).get('slots',0) for name in source], 'targets':{}}
            for who in ['lihuowang','bailingmiao']:
                records = [weights[(who,name)] for name in targets]
                entry['targets'][who] = {'names':targets, 'rest_positions':[r['rest_position'] for r in records],
                                        'segment_lengths':[math.dist(a['rest_position'],b['rest_position']) for a,b in zip(records,records[1:])],
                                        'weighted_slots':[r['slots'] for r in records]}
            result['fingers'].append(entry)
    for who in ['lihuowang','bailingmiao']:
        anchors = {d['name']:d['anchor'] for d in counts['characters'][who]['drivers']}
        rows = []
        for side in ['L','R']:
            for part,start,end in [('UpperArm','upperarm','lowerarm'),('Forearm','lowerarm','hand')]:
                a,b = anchors[f'{start}_{side.lower()}'],anchors[f'{end}_{side.lower()}']
                vector = [y-x for x,y in zip(a,b)]
                length2 = sum(value*value for value in vector)
                suffixes = [f'gMan {side} {part}'] + [f'gMan {side} {part} Twist{i}' for i in range(1,5 if part=='UpperArm' else 4)]
                for suffix in suffixes:
                    record = weights[(who,suffix)]
                    position = record['rest_position']
                    delta = [x-y for x,y in zip(position,a)]
                    fraction = sum(x*y for x,y in zip(delta,vector)) / length2
                    projected = [x+fraction*y for x,y in zip(a,vector)]
                    rows.append({'name':suffix, **record, 'projected_fraction_of_limb':fraction, 'off_axis_m':math.dist(position,projected)})
        result['twists'][who] = rows
    for filename in ['assets/prepared/naraka/bailingmiao/models/lianmu.glb', 'assets/prepared/naraka/bailingmiao/models/outfit.glb']:
        raw = (M.ROOT / filename).read_bytes()
        size = struct.unpack_from('<I',raw,12)[0]
        nodes = json.loads(raw[20:20+size])['nodes']
        parents = {child:index for index,node in enumerate(nodes) for child in node.get('children',[])}
        records = {}
        for side in ['L','R']:
            for digit in range(5):
                for tail in ['', '1','2','Nub']:
                    name = f'gMan {side} Finger{digit}{tail}'
                    records[name] = [nodes[parents[index]]['name'] if index in parents else None
                                     for index,node in enumerate(nodes) if node.get('name')==name]
        result['target_parent_verification'][filename] = records
    target = Path(__file__).with_suffix('.json')
    target.write_text(json.dumps(result,indent=2)+'\n')
    assert all(entry['source_weighted_slots'][-1]==0 and all(v>0 for v in entry['source_weighted_slots'][:3]) for entry in result['fingers'])
    print('Verified source first-three joints weighted / Nub unweighted for all ten fingers; wrote',target)
