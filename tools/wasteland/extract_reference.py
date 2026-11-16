#!/usr/bin/env python3
"""Extract compatibility metadata, never source geometry, from Kiva's male avatar.

Run from any directory: python3 tools/wasteland/extract_reference.py [--verify]
Only Python's standard library is required. Output lives outside runtime assets.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import struct
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(REPO / 'scripts'))
from naraka_animation import unpack_runtime


def require(condition, message):
    if not condition:
        raise ValueError(message)


def identity():
    return [[float(r == c) for c in range(4)] for r in range(4)]


def matrix(values):
    """AV03 column-major bytes to explicitly row-major JSON matrices."""
    return [[values[c * 4 + r] for c in range(4)] for r in range(4)]


def multiply(a, b):
    return [[sum(a[r][k] * b[k][c] for k in range(4)) for c in range(4)] for r in range(4)]


def point(a, p):
    return [sum(a[r][k] * p[k] for k in range(3)) + a[r][3] for r in range(3)]


def distance(a, b):
    return math.sqrt(sum((x - y) ** 2 for x, y in zip(a, b)))


def max_difference(a, b):
    return max(abs(x - y) for row_a, row_b in zip(a, b) for x, y in zip(row_a, row_b))


class Reader:
    def __init__(self, data):
        self.data, self.at = data, 0

    def take(self, count):
        require(0 <= count <= len(self.data) - self.at, 'truncated AV03 data')
        result = self.data[self.at:self.at + count]
        self.at += count
        return result

    def read(self, fmt):
        return struct.unpack('<' + fmt, self.take(struct.calcsize('<' + fmt)))

    def string(self):
        return self.take(self.read('H')[0]).decode('utf-8')


def parse_avatar(raw):
    r = Reader(raw)
    require(r.take(8) == b'KIVAAV03', 'only AV03 explicitly retaining source rest is supported')
    header = json.loads(r.take(r.read('I')[0]))
    drivers = [{'name': r.string(), 'position_game': list(r.read('3f'))}
               for _ in range(r.read('H')[0])]
    require(len(drivers) == 23, 'expected the existing 23-driver body contract')
    for _ in range(r.read('H')[0]):
        r.string()
        srgb, length = r.read('BI')
        require(srgb in (0, 1), 'invalid texture color space')
        r.take(length)
    for _ in range(r.read('H')[0]):
        r.string()
        r.read('hhhB11fB')
    palettes = []
    low, high = [math.inf] * 3, [-math.inf] * 3
    nonbind, vertices = 0, 0
    for _ in range(r.read('H')[0]):
        part, blade, count = r.string(), *r.read('BH')
        require(blade in (0, 1) and 0 < count <= 256, 'invalid palette size')
        slots = []
        for index in range(count):
            name, driver = r.string(), r.read('H')[0]
            inverse, rest = matrix(r.read('16f')), matrix(r.read('16f'))
            require(driver < len(drivers), 'invalid structural driver')
            require(all(math.isfinite(x) for m in (inverse, rest) for row in m for x in row),
                    'nonfinite reference matrix')
            nonbind += max_difference(multiply(rest, inverse), identity()) > 1e-3
            slots.append({'name': name, 'native_name': name.rsplit('::', 1)[-1],
                          'palette_index': index, 'driver_index': driver,
                          'inverse_bind_game': inverse, 'source_rest_game': rest})
        transforms = [multiply(s['source_rest_game'], s['inverse_bind_game']) for s in slots]
        for _ in range(r.read('H')[0]):
            _, vc, ic = r.read('HII')
            require(vc > 0 and ic > 0 and ic % 3 == 0, 'invalid primitive size')
            data = r.take(vc * 40)
            # Match the source report's full body/head/hair baseline, excluding
            # independent weapon, sheath and hat. Do not write these vertices.
            body = not blade and not part.startswith(('naraka::sheath', 'naraka::hat'))
            for values in struct.iter_unpack('<8f8B', data):
                p, joints, weights = values[:3], values[8:12], values[12:16]
                require(all(j < count for j in joints) and sum(weights) == 255, 'invalid skin weights')
                if body:
                    out = [0.0] * 3
                    for joint, weight in zip(joints, weights):
                        if weight:
                            q = point(transforms[joint], p)
                            for axis in range(3):
                                out[axis] += q[axis] * weight / 255
                    for axis in range(3):
                        low[axis], high[axis] = min(low[axis], out[axis]), max(high[axis], out[axis])
                    vertices += 1
            indices = r.take(ic * 4)
            require(max(x[0] for x in struct.iter_unpack('<I', indices)) < vc, 'invalid triangle index')
        if part.startswith('naraka::outfit::'):
            palettes.append({'part': part, 'slots': slots})
    require(r.at == len(raw), 'trailing AV03 bytes')
    require(abs(high[1] - low[1] - 1.73) < 0.002 and abs(low[1]) < 0.002,
            'default avatar no longer matches the approved 1.73m baseline')
    return header, drivers, palettes, {'body_bounds_game': [low, high],
                                     'body_height_m': high[1] - low[1],
                                     'body_vertex_samples': vertices,
                                     'nonbind_palette_slots': nonbind}


def extract():
    avatar_path, animation_path = REPO / 'assets/characters/lihuowang.bin', REPO / 'assets/animations/naraka.bin'
    avatar_raw, animation_raw = avatar_path.read_bytes(), animation_path.read_bytes()
    header, drivers, palettes, evidence = parse_avatar(avatar_raw)
    source = unpack_runtime(animation_raw)
    require(source['rig_id'] == 'naraka/l22_male_skeleton', 'unexpected animation hierarchy')
    native = {}
    for palette in palettes:
        for slot in palette['slots']:
            name = slot['native_name']
            if name.startswith('gMan '):
                if name in native:
                    require(max_difference(native[name]['source_rest_game'], slot['source_rest_game']) < 1e-4,
                            f'ambiguous target rest for {name}')
                else:
                    native[name] = slot
    selected = [node for node in source['nodes'] if node['name'].startswith('gMan ')]
    require(len(selected) == 52, 'expected 22 body and 30 finger deform joints')
    nodes = [{'name': 'root', 'parent': None, 'driver_index': 0,
              'source_rest_game': identity(), 'reference_only_root': True}]
    for node in selected:
        name = node['name']
        require(name in native, f'target body reference missing {name}')
        parent = source['nodes'][node['parent']]['name']
        slot = native[name]
        nodes.append({'name': name, 'parent': parent if parent.startswith('gMan ') else 'root',
                      'driver_index': slot['driver_index'], 'source_rest_game': slot['source_rest_game']})
    by_name = {n['name']: n for n in nodes}
    positions = {name: [n['source_rest_game'][axis][3] for axis in range(3)] for name, n in by_name.items()}
    names = ['root', 'gMan Pelvis', 'gMan Spine', 'gMan Spine1', 'gMan Spine2', 'gMan Neck', 'gMan Head',
             'gMan L Clavicle', 'gMan L UpperArm', 'gMan L Forearm', 'gMan L Hand',
             'gMan R Clavicle', 'gMan R UpperArm', 'gMan R Forearm', 'gMan R Hand',
             'gMan L Thigh', 'gMan L Calf', 'gMan L Foot', 'gMan L Toe0',
             'gMan R Thigh', 'gMan R Calf', 'gMan R Foot', 'gMan R Toe0']
    anchor_differences = [{'driver': d['name'], 'native_joint': name,
                           'distance_m': distance(d['position_game'], positions[name])}
                          for d, name in zip(drivers, names)]
    anchor_error = max(row['distance_m'] for row in anchor_differences)
    palms = {}
    for side in ('L', 'R'):
        palm_names = [f'gMan {side} {suffix}' for suffix in ('Hand', 'Finger1', 'Finger2', 'Finger4')]
        fingers = [[f'gMan {side} Finger{digit}{suffix}' for suffix in ('', '1', '2')] for digit in range(5)]
        palms[side] = {'frame_point_names': palm_names, 'frame_points_game': [positions[n] for n in palm_names],
                       'finger_chains': fingers,
                       'terminal_forward_local': [1, 0, 0],
                       'frame_contract': '+X wrist->middle MCP; +Y index->little after orthogonalization; +Z cross(X,Y)',
                       'attachment_contract': 'Runtime actual wrist/MCP frame; existing +0.055m melee offset, not a static Blender weapon bone'}
    evidence.update({'driver_anchor_max_error_m': anchor_error, 'deform_joint_count': 52,
                     'reference_root_count': 1, 'finger_joint_count': 30,
                     'source_animation_clips_validated': len(source['clips']),
                     'anchor_vs_actual_skin_joint': anchor_differences})
    return {'schema': 'kiva-compatibility-reference-v1',
            'purpose': 'Reference skeleton only. No new character mesh, animation compatibility acceptance, or original geometry included.',
            'matrix_layout': 'row-major nested lists; source binary is column-major',
            'coordinates': {'game': 'meters, +Y up, -Z forward', 'blender': 'meters, +Z up, +Y forward',
                            'game_to_blender': [[1, 0, 0, 0], [0, 0, -1, 0], [0, 1, 0, 0], [0, 0, 0, 1]]},
            'provenance': [{'path': str(path.relative_to(REPO)), 'sha256': hashlib.sha256(raw).hexdigest()}
                           for path, raw in ((avatar_path, avatar_raw), (animation_path, animation_raw))],
            'height_m': 1.73, 'target_original_conversion': header['conversion'],
            'nodes': nodes, 'drivers': drivers, 'palms': palms,
            'original_outfit_palettes': palettes, 'evidence': evidence,
            'limits': ['Original outfit twist/accessory palette matrices retained as metadata only; their parent hierarchy is not present in AV03 and is not invented.',
                       'Structural driver anchors and actual skin-joint rests are distinct existing data: calf positions differ about 3.1cm. Both are preserved, neither is silently substituted.',
                       'Native source animation hierarchy establishes the 52 gMan joint parent links; target positions/orientations come from the actual default avatar, not donor animation rest.',
                       'Original IBM is evidence, not inverse(current rest). New authored mesh must have its own valid bind matrices.']}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--output', type=Path, default=REPO / 'art/wasteland/reference-rig.json')
    parser.add_argument('--verify', action='store_true', help='rebuild in memory and compare exact JSON bytes')
    args = parser.parse_args()
    data = extract()
    raw = (json.dumps(data, ensure_ascii=False, indent=2, allow_nan=False) + '\n').encode()
    if args.verify:
        require(args.output.read_bytes() == raw, 'reference JSON is stale; regenerate explicitly')
    else:
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_bytes(raw)
    print(json.dumps({'file': str(args.output), 'verify': args.verify, **data['evidence']}, indent=2))


if __name__ == '__main__':
    main()
