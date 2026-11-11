"""Portable interpolation/FK/actual-weight verification, with no Unity/NumPy.

This consumes the runtime binary, not just baked key poses. Every independently
captured native reference instant is checked, including all 1116 hand vertices.
"""
from bisect import bisect_right
import math
import struct

from naraka_animation import (quaternion, read_native_capture, require, sha256,
                              unpack_runtime)


def slerp(a, b, u):
    a, b = quaternion(a), quaternion(b)
    dot = sum(x*y for x, y in zip(a, b))
    if dot < 0:
        b, dot = [-x for x in b], -dot
    if dot > .9995:
        return quaternion([x+(y-x)*u for x, y in zip(a, b)])
    angle = math.acos(min(1., dot))
    aa, bb = math.sin((1-u)*angle)/math.sin(angle), math.sin(u*angle)/math.sin(angle)
    return [x*aa+y*bb for x, y in zip(a, b)]


def index_clip(clip):
    return {joint: [(keys, [key[0] for key in keys]) for keys in channels]
            for joint, channels in clip['tracks'].items()}


def sample_runtime(index, nodes, time):
    poses = [node['rest'][:] for node in nodes]
    for joint, channels in index.items():
        for start, width, (keys, times) in zip((0, 3, 7), (3, 4, 3), channels, strict=True):
            if not keys:
                continue
            at = bisect_right(times, time)
            if at == 0:
                value = keys[0][1]
            elif at == len(keys) or keys[at-1][0] == time:
                value = keys[at-1][1]
            else:
                a, b = keys[at-1], keys[at]
                # Adjacent f32 times are meaningful cuts. Never clamp this
                # denominator to EPSILON: there is no reachable f32 in between.
                u = (time-a[0])/(b[0]-a[0])
                value = slerp(a[1], b[1], u) if width == 4 else [x+(y-x)*u for x, y in zip(a[1], b[1])]
            poses[joint][start:start+width] = value
    return poses


def matrix(pose):
    x, y, z, w = quaternion(pose[3:7])
    a, b, c = pose[7:]
    return [(1-2*(y*y+z*z))*a, 2*(x*y-z*w)*b, 2*(x*z+y*w)*c, pose[0],
            2*(x*y+z*w)*a, (1-2*(x*x+z*z))*b, 2*(y*z-x*w)*c, pose[1],
            2*(x*z-y*w)*a, 2*(y*z+x*w)*b, (1-2*(x*x+y*y))*c, pose[2]]


def multiply(a, b):
    result = []
    for at in (0, 4, 8):
        x, y, z, t = a[at:at+4]
        result.extend((x*b[0]+y*b[4]+z*b[8], x*b[1]+y*b[5]+z*b[9],
                       x*b[2]+y*b[6]+z*b[10], x*b[3]+y*b[7]+z*b[11]+t))
    return result


def fk(poses, nodes):
    worlds = []
    for pose, node in zip(poses, nodes, strict=True):
        local = matrix(pose)
        worlds.append(multiply(worlds[node['parent']], local) if node['parent'] >= 0 else local)
    return worlds


def raw_poses(poses, normalization):
    result, factor = [], normalization['scale']
    for i, p in enumerate(poses):
        result.append([p[0]/factor, (p[1]-(normalization['floor_offset_y'] if i == 0 else 0))/factor,
                       -p[2]/factor, -p[3], -p[4], p[5], p[6], *p[7:]])
    return result


def skin_influences(mesh, nodes):
    primitive = mesh['primitives'][0]
    names = {node['name']: i for i, node in enumerate(nodes)}
    palette = [names[name] for name in primitive['bone_names']]
    positions, joints, weights = primitive['positions'], primitive['joints'], primitive['weights']
    inverse = primitive['bindposes_column_major']
    vertices = []
    for i in range(len(positions)//3):
        x, y, z = positions[3*i:3*i+3]
        influences = []
        for k in range(4):
            joint, weight = joints[4*i+k], weights[4*i+k]
            if weight == 0:
                continue
            m = inverse[16*joint:16*(joint+1)]
            influences.append((palette[joint], (m[0]*x+m[4]*y+m[8]*z+m[12])*weight,
                               (m[1]*x+m[5]*y+m[9]*z+m[13])*weight,
                               (m[2]*x+m[6]*y+m[10]*z+m[14])*weight, weight))
        vertices.append(influences)
    require(len(vertices) >= 1000, 'portable skin check lacks the actual weighted hand subset')
    return vertices


def skin_error(delta, vertices):
    maximum = 0.
    for vertex in vertices:
        dx = dy = dz = 0.
        for joint, x, y, z, w in vertex:
            m = delta[joint]
            dx += m[0]*x+m[1]*y+m[2]*z+m[3]*w
            dy += m[4]*x+m[5]*y+m[6]*z+m[7]*w
            dz += m[8]*x+m[9]*y+m[10]*z+m[11]*w
        maximum = max(maximum, dx*dx+dy*dy+dz*dz)
    return math.sqrt(maximum)


def verify_sampling(binary, document, reference_bytes, proof, mesh):
    require(proof['reference_capture_sha256'] == sha256(reference_bytes), 'interpolation reference digest mismatch')
    packed = unpack_runtime(binary)
    reference = read_native_capture(reference_bytes)
    nodes = packed['nodes']
    require([(n['name'], n['parent']) for n in nodes]
            == [(n['name'], n['parent']) for n in reference['rig']['nodes']], 'sampling reference hierarchy mismatch')
    clips = {c['name']: c for c in packed['clips']}
    source = {c['name']: c['source_sha256'] for c in document['clips']}
    require(set(clips) == {c['name'] for c in reference['clips']}, 'sampling reference clip set mismatch')
    vertices = skin_influences(mesh, nodes)
    maximum = {'local_quaternion_max_error_degrees': 0., 'local_position_max_error_m': 0.,
               'world_joint_position_max_error_m': 0., 'real_hand_lbs_vertex_max_error_m': 0.}
    sample_count = 0
    for clip in reference['clips']:
        require(source[clip['name']] == clip['source_sha256'], 'sampling reference source SHA mismatch')
        index = index_clip(clips[clip['name']])
        for frame in clip['frames']:
            poses = raw_poses(sample_runtime(index, nodes, frame['time']), document['runtime_normalization'])
            for actual, native in zip(poses, frame['locals'], strict=True):
                a, b = quaternion(actual[3:7]), quaternion(native[3:7])
                angle = 2*math.degrees(math.acos(min(1., abs(sum(x*y for x, y in zip(a, b))))))
                distance = math.sqrt(sum((x-y)**2 for x, y in zip(actual[:3], native[:3])))
                maximum['local_quaternion_max_error_degrees'] = max(maximum['local_quaternion_max_error_degrees'], angle)
                maximum['local_position_max_error_m'] = max(maximum['local_position_max_error_m'], distance)
            worlds = fk(poses, nodes)
            # Native matrices are serialized column-major; portable FK is 3x4
            # row-major. The difference also directly evaluates weighted LBS.
            delta = [[a[i]-b[(i % 4)*4+i//4] for i in range(12)]
                     for a, b in zip(worlds, frame['native_world_column_major'], strict=True)]
            joint = max(math.sqrt(m[3]**2+m[7]**2+m[11]**2) for m in delta)
            maximum['world_joint_position_max_error_m'] = max(maximum['world_joint_position_max_error_m'], joint)
            maximum['real_hand_lbs_vertex_max_error_m'] = max(maximum['real_hand_lbs_vertex_max_error_m'], skin_error(delta, vertices))
            sample_count += 1
        require(maximum['local_quaternion_max_error_degrees'] <= .1, 'runtime interpolation exceeds native angular threshold')
        require(all(value <= .001 for key, value in maximum.items() if key.endswith('_m')),
                'runtime interpolation exceeds native spatial / real hand LBS threshold')
    require(sample_count > 0, 'empty independent native sampling reference')
    return {'independent_native_instants': sample_count, 'maximum': maximum}


def verify_cuts(document, native_report, binary):
    packed = unpack_runtime(binary)
    clips = {c['name']: c for c in packed['clips']}
    node_indices = {node['name']: i for i, node in enumerate(packed['nodes'])}
    count = 0
    for check in native_report['clips']:
        index = index_clip(clips[check['name']])
        for cut in check.get('precision_cuts', []):
            a, b = cut['left_time'], cut['right_time']
            bits = struct.unpack('<I', struct.pack('<f', a))[0]
            require(struct.unpack('<f', struct.pack('<I', bits+1))[0] == b,
                    'native precision cut is not adjacent at f32 precision')
            for time, field in [(a, 'left_local'), (b, 'right_local')]:
                pose = raw_poses(sample_runtime(index, packed['nodes'], time), document['runtime_normalization'])
                for bone in cut['changed_bones']:
                    require(max(abs(x-y) for x, y in zip(pose[node_indices[bone['bone']]][:3], bone[field][:3])) <= 1e-6,
                            'runtime lost native precision cut position')
                    q, r = quaternion(pose[node_indices[bone['bone']]][3:7]), quaternion(bone[field][3:7])
                    require(abs(sum(x*y for x, y in zip(q, r))) > .999999, 'runtime lost native precision cut rotation')
            count += 1
    return count
