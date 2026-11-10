"""Measure finger native axes using own-rest transforms and actual weighted LBS geometry."""
import importlib.util
import json
import math
from pathlib import Path
import re
import sys

sys.dont_write_bytecode = True
BASE = Path(__file__).resolve().parent
SPEC = importlib.util.spec_from_file_location('hand_counts', BASE / 'hand-asset-counts.py')
M = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(M)


def rows(values):
    return [[values[c*4+r] for c in range(4)] for r in range(4)]


def multiply(a,b):
    return [[sum(a[r][k]*b[k][c] for k in range(4)) for c in range(4)] for r in range(4)]


def inverse(matrix):
    values=[list(row)+[float(r==c) for c in range(4)] for r,row in enumerate(matrix)]
    for c in range(4):
        pivot=max(range(c,4),key=lambda r:abs(values[r][c]));values[c],values[pivot]=values[pivot],values[c]
        amount=values[c][c];assert abs(amount)>1e-12
        values[c]=[v/amount for v in values[c]]
        for r in range(4):
            if r!=c:
                factor=values[r][c];values[r]=[v-factor*p for v,p in zip(values[r],values[c])]
    return [row[4:] for row in values]


def point(matrix,p):
    return [sum(matrix[r][c]*p[c] for c in range(3))+matrix[r][3] for r in range(3)]


def length(vector):
    return math.sqrt(sum(v*v for v in vector))


def unit(vector):
    size=length(vector);return [v/size for v in vector]


def det(matrix):
    a,b,c=matrix[:3]
    return a[0]*(b[1]*c[2]-b[2]*c[1])-a[1]*(b[0]*c[2]-b[2]*c[0])+a[2]*(b[0]*c[1]-b[1]*c[0])


def audit(appearance):
    reader=M.Reader(M.ROOT/'assets'/appearance/'character.bin');assert reader.bytes(8)==b'KIVAAV03'
    reader.bytes(reader.read('I')[0])
    for _ in range(reader.read('H')[0]):reader.string();reader.read('3f')
    for _ in range(reader.read('H')[0]):reader.string();reader.read('B');reader.bytes(reader.read('I')[0])
    for _ in range(reader.read('H')[0]):reader.string();reader.read('hhhB11fB')
    result={'links':[],'distal_clouds':[]}
    for _ in range(reader.read('H')[0]):
        part=reader.string();blade,n=reader.read('BH');bones=[]
        for _ in range(n):
            name=reader.string();reader.read('H');ibm=rows(reader.read('16f'));rest=rows(reader.read('16f'))
            bones.append({'name':name,'semantic':name.split('::')[-1],'rest':rest,'ibm':ibm,'skin':multiply(rest,ibm)})
        by_name={bone['semantic']:bone for bone in bones}
        for side in ['L','R']:
            for d in range(5):
                names=[f'gMan {side} Finger{d}{tail}' for tail in ['', '1','2']]
                for a,b in zip(names,names[1:]):
                    if a not in by_name or b not in by_name:continue
                    parent,child=by_name[a],by_name[b];parent_matrix=parent['rest'];child_position=[child['rest'][r][3] for r in range(3)]
                    offset=point(inverse(parent_matrix),child_position)
                    result['links'].append({'part':part,'parent':a,'child':b,'child_offset_parent_native':offset,'native_direction':unit(offset),
                                            'parent_basis_column_lengths':[length([parent_matrix[r][c] for r in range(3)]) for c in range(3)],'parent_det3':det(parent_matrix)})
        terminal={index:{'bone':bone,'points':[],'dominant':[]} for index,bone in enumerate(bones) if re.fullmatch(r'gMan [LR] Finger[0-4]2',bone['semantic'])}
        for _ in range(reader.read('H')[0]):
            _,nv,ni=reader.read('HII')
            for _ in range(nv):
                vertex=reader.read('8f8B');indices=vertex[8:12];weights=vertex[12:16]
                targets=[(index,weight) for index,weight in zip(indices,weights) if index in terminal and weight>0]
                if not targets:continue
                output=[0.,0.,0.]
                for index,weight in zip(indices,weights):
                    if weight:
                        transformed=point(bones[index]['skin'],vertex[:3])
                        output=[v+weight/255*x for v,x in zip(output,transformed)]
                for index,weight in targets:
                    terminal[index]['points'].append((output,weight))
                    if weight>=128:terminal[index]['dominant'].append((output,weight))
            reader.bytes(ni*4)
        for value in terminal.values():
            if not value['points']:continue
            bone=value['bone'];inv=inverse(bone['rest']);points=[(point(inv,p),w) for p,w in value['points']]
            dominant=[(point(inv,p),w) for p,w in value['dominant']]
            total=sum(w for p,w in points);center=[sum(p[c]*w for p,w in points)/total for c in range(3)]
            row={'part':part,'name':bone['semantic'],'weighted_vertex_slots':len(points),'weights255_sum':total,
                 'distal_native_weighted_centroid':center,'centroid_direction_native':unit(center),'positive_native_X_weight_fraction':sum(w for p,w in points if p[0]>0)/total,
                 'native_bounds':[[min(p[c] for p,w in points) for c in range(3)],[max(p[c] for p,w in points) for c in range(3)]],
                 'basis_column_lengths':[length([bone['rest'][r][c] for r in range(3)]) for c in range(3)],'det3':det(bone['rest'])}
            if dominant:
                total_d=sum(w for p,w in dominant);center_d=[sum(p[c]*w for p,w in dominant)/total_d for c in range(3)]
                row['dominant_vertex_slots']=len(dominant);row['dominant_native_weighted_centroid']=center_d;row['dominant_centroid_direction_native']=unit(center_d)
            result['distal_clouds'].append(row)
    assert reader.offset==len(reader.data)
    return result


if __name__=='__main__':
    result={appearance:audit(appearance) for appearance in ['lihuowang','bailingmiao']}
    target=Path(__file__).with_suffix('.json');target.write_text(json.dumps(result,indent=2)+'\n')
    for appearance,rows_ in result.items():
        links=rows_['links'];clouds=rows_['distal_clouds']
        print(appearance,'links',len(links),'min child native +X cosine',min(link['native_direction'][0] for link in links),'positive determinant',all(link['parent_det3']>0 for link in links))
        for row in clouds:
            if row['part'].endswith('::body') or row['part'].endswith('::1::body'):
                print(' ',row['name'],'centroid_native_mm',[round(v*1000,3) for v in row['distal_native_weighted_centroid']],'+Xweight',round(row['positive_native_X_weight_fraction'],6),'directionX',round(row['centroid_direction_native'][0],6),'dominant',row.get('dominant_vertex_slots'))
    print('Wrote',target)
