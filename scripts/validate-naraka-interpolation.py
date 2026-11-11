#!/usr/bin/env python3
"""Measure SRT interpolation against independently evaluated native instants.

The reference capture must contain extra times, not just the packed key grid.
This reports measured temporal error; it does not imply a continuous error bound.
"""
import argparse
from bisect import bisect_left
import importlib.util
import json
from pathlib import Path

import numpy as np
from naraka_animation import canonical_json, read_native_capture, require, sha256

spec = importlib.util.spec_from_file_location('bake', Path(__file__).with_name('bake-naraka-native.py'))
bake = importlib.util.module_from_spec(spec)
spec.loader.exec_module(bake)


def interpolate(a, b, factor):
    result = a + factor * (b - a)
    qa, qb = a[:, 3:7], b[:, 3:7].copy()
    qa = qa / np.linalg.norm(qa, axis=1)[:, None]
    qb = qb / np.linalg.norm(qb, axis=1)[:, None]
    dots = np.sum(qa * qb, axis=1)
    qb[dots < 0] *= -1
    dots = np.clip(abs(dots), 0, 1)
    theta = np.arccos(dots)
    sine = np.sin(theta)
    spherical = sine > .001
    result[:, 3:7] = qa + factor * (qb - qa)
    result[spherical, 3:7] = ((np.sin((1-factor)*theta[spherical]) / sine[spherical])[:, None] * qa[spherical]
                             + (np.sin(factor*theta[spherical]) / sine[spherical])[:, None] * qb[spherical])
    result[:, 3:7] /= np.linalg.norm(result[:, 3:7], axis=1)[:, None]
    return result


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--coarse', type=Path, required=True)
    p.add_argument('--reference', type=Path, required=True)
    p.add_argument('--real-hand', type=Path, required=True)
    p.add_argument('--output', type=Path, required=True)
    args = p.parse_args()
    coarse_raw, reference_raw = args.coarse.read_bytes(), args.reference.read_bytes()
    coarse, reference = read_native_capture(coarse_raw), read_native_capture(reference_raw)
    require(coarse['rig'] == reference['rig'], 'native interpolation rig differs')
    by_name = {c['name']: c for c in coarse['clips']}
    require(set(by_name) == {c['name'] for c in reference['clips']}, 'native interpolation clip sets differ')
    nodes = reference['rig']['nodes']
    mesh = json.loads(args.real_hand.read_text())['primitives'][0]
    node_index = {node['name']: i for i, node in enumerate(nodes)}
    palette = np.array([node_index[name] for name in mesh['bone_names']])
    vertices = np.array(mesh['positions']).reshape((-1, 3))
    positions = np.concatenate((vertices, np.ones((len(vertices), 1))), axis=1)
    joints = np.array(mesh['joints']).reshape((-1, 4))
    weights = np.array(mesh['weights']).reshape((-1, 4))
    inverse = np.array(mesh['bindposes_column_major']).reshape((-1, 4, 4)).transpose((0, 2, 1))
    rows = []
    for clip in reference['clips']:
        source = by_name[clip['name']]
        require(source['source_sha256'] == clip['source_sha256'], 'native interpolation original clip differs')
        times = [f['time'] for f in source['frames']]
        row = {'name': clip['name'], 'source_sha256': clip['source_sha256'], 'extra_native_instants': 0,
               'shared_native_instants': 0, 'shared_native_local_max_error_degrees': 0.,
               'local_quaternion_max_error_degrees': 0., 'local_position_max_error_m': 0.,
               'world_joint_position_max_error_m': 0., 'real_hand_lbs_vertex_max_error_m': 0.}
        for frame in clip['frames']:
            at = bisect_left(times, frame['time'])
            if at < len(times) and times[at] == frame['time']:
                row['shared_native_instants'] += 1
                poses = np.array(source['frames'][at]['locals'])
                poses[:, 3:7] /= np.linalg.norm(poses[:, 3:7], axis=1)[:, None]
            else:
                require(0 < at < len(times), 'reference instant outside coarse clip')
                row['extra_native_instants'] += 1
                factor = (frame['time'] - times[at-1]) / (times[at] - times[at-1])
                poses = interpolate(np.array(source['frames'][at-1]['locals']), np.array(source['frames'][at]['locals']), factor)
            exact = np.array(frame['locals'])
            q = exact[:, 3:7] / np.linalg.norm(exact[:, 3:7], axis=1)[:, None]
            dots = np.clip(abs(np.sum(q * poses[:, 3:7], axis=1)), 0, 1)
            angle = float(np.max(2 * np.degrees(np.arccos(dots))))
            if at < len(times) and times[at] == frame['time']:
                row['shared_native_local_max_error_degrees'] = max(row['shared_native_local_max_error_degrees'], angle)
            position = float(np.max(np.linalg.norm(exact[:, :3] - poses[:, :3], axis=1)))
            world = bake.fk(poses.tolist(), nodes)
            native = np.array(frame['native_world_column_major']).reshape((-1, 4, 4)).transpose((0, 2, 1))
            joint_error = float(np.max(np.linalg.norm(world[:, :3, 3] - native[:, :3, 3], axis=1)))
            skin_delta = (world[palette] - native[palette]) @ inverse
            influences = np.einsum('vkij,vj->vki', skin_delta[joints], positions)
            vertex_error = float(np.max(np.linalg.norm(np.sum(influences[:, :, :3] * weights[:, :, None], axis=1), axis=1)))
            for key, value in [('local_quaternion_max_error_degrees', angle), ('local_position_max_error_m', position),
                               ('world_joint_position_max_error_m', joint_error), ('real_hand_lbs_vertex_max_error_m', vertex_error)]:
                row[key] = max(row[key], value)
        require(row['extra_native_instants'] + row['shared_native_instants'] > 0, 'reference has no instants')
        rows.append(row)
    result = {'format': 'KIVA_NATIVE_INTERPOLATION_1', 'coarse_capture_sha256': sha256(coarse_raw),
              'reference_capture_sha256': sha256(reference_raw), 'real_hand_sha256': sha256(args.real_hand.read_bytes()),
              'method': 'Local linear translation/scale and shortest-path quaternion slerp; independent native reference at times absent from coarse capture; raw Unity metres; actual original 1116 weighted hand vertices',
              'scope': 'Measured intermediate instants only; no claim of a continuous error bound', 'clips': rows,
              'maximum': {key: max(row[key] for row in rows) for key in rows[0] if key.endswith(('_m', '_degrees'))}}
    args.output.write_bytes(canonical_json(result))
    print(json.dumps(result['maximum'], indent=2))


if __name__ == '__main__':
    main()
