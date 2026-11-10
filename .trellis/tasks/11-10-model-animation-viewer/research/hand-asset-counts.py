"""Read-only inventory of hand channels and actual bundled palette mappings."""
from pathlib import Path
import collections
import json
import re
import struct

ROOT = Path(__file__).resolve().parents[4]


class Reader:
    def __init__(self, path):
        self.data = Path(path).read_bytes()
        self.offset = 0

    def bytes(self, count):
        result = self.data[self.offset:self.offset + count]
        assert len(result) == count
        self.offset += count
        return result

    def read(self, fmt):
        return struct.unpack('<' + fmt, self.bytes(struct.calcsize('<' + fmt)))

    def string(self):
        return self.bytes(self.read('H')[0]).decode()


def category(name):
    if 'Finger' in name:
        return 'finger'
    if 'Hand' in name or 'hand' in name:
        return 'hand'
    if re.search(r'ForeArm|Forearm|ForeTwist', name):
        return 'forearm'
    if 'UpperArm' in name or 'UpperTwist' in name:
        return 'upperarm'
    return None


def character(path):
    reader = Reader(path)
    magic = reader.bytes(8)
    assert magic in (b'KIVAAV02', b'KIVAAV03')
    metadata = json.loads(reader.bytes(reader.read('I')[0]))
    drivers = []
    for _ in range(reader.read('H')[0]):
        drivers.append({'name': reader.string(), 'anchor': reader.read('3f')})
    for _ in range(reader.read('H')[0]):
        reader.string()
        reader.read('B')
        reader.bytes(reader.read('I')[0])
    for _ in range(reader.read('H')[0]):
        reader.string()
        reader.read('hhhB11f')
        if magic == b'KIVAAV03':
            reader.read('B')
    parts = []
    for _ in range(reader.read('H')[0]):
        part = {'name': reader.string()}
        blade, count = reader.read('BH')
        bones = []
        for _ in range(count):
            name, driver = reader.string(), reader.read('H')[0]
            reader.read('16f')
            rest = reader.read('16f') if magic == b'KIVAAV03' else None
            bones.append({'name': name, 'driver': drivers[driver]['name'], 'category': category(name),
                          'rest_model_position': list(rest[12:15]) if rest else None})
        part['blade'] = bool(blade)
        part['bone_count'] = len(bones)
        part['hand_related_bones'] = [bone for bone in bones if bone['category']]
        influenced = collections.Counter()
        sums = collections.Counter()
        bone_vertices = collections.Counter()
        bone_weights = collections.Counter()
        vertices = 0
        for _ in range(reader.read('H')[0]):
            _, vertex_count, index_count = reader.read('HII')
            vertices += vertex_count
            for _ in range(vertex_count):
                values = reader.read('8f8B')
                touched = set()
                for index, weight in zip(values[8:12], values[12:16]):
                    label = bones[index]['category']
                    if weight and label:
                        touched.add(label)
                        sums[label] += weight
                        bone_vertices[index] += 1
                        bone_weights[index] += weight
                influenced.update(touched)
            reader.bytes(index_count * 4)
        part['vertices'] = vertices
        for index, bone in enumerate(bones):
            if bone['category']:
                bone['weighted_vertex_slots'] = bone_vertices[index]
                bone['weights255_sum'] = bone_weights[index]
        part['vertices_with_category_influence'] = dict(influenced)
        part['weights255_sums_by_category'] = dict(sums)
        parts.append(part)
    assert reader.offset == len(reader.data)
    return {'format': magic.decode(), 'drivers': drivers, 'parts': parts,
            'source_animation_name_matching': metadata.get('source_animation_name_matching'),
            'matrix_contract': metadata.get('matrix_contract')}


def source_animation(path):
    reader = Reader(path)
    assert reader.bytes(8) == b'KIVASR01'
    joint_count, clip_count = reader.read('HH')
    joints = []
    for _ in range(joint_count):
        joints.append({'name': reader.string(), 'parent': reader.read('H')[0], 'trs': reader.read('10f')})
    clips = []
    for _ in range(clip_count):
        clip = {'name': reader.string()}
        clip['duration_ms'], tracks = reader.read('HH')
        clip['tracks'] = tracks
        hand_tracks = []
        for _ in range(tracks):
            joint = joints[reader.read('H')[0]]
            keys = []
            varying = []
            for width in (3, 4, 3):
                count = reader.read('H')[0]
                values = [reader.read('H' + str(width) + 'f') for _ in range(count)]
                keys.append(count)
                varying.append(bool(values) and any(any(abs(a-b) > 1e-6 for a,b in zip(value[1:],values[0][1:])) for value in values[1:]))
            if category(joint['name']):
                hand_tracks.append({'name': joint['name'], 'category': category(joint['name']), 'keys_TRS': keys, 'varying_TRS': varying})
        clip['hand_related_tracks'] = hand_tracks
        clips.append(clip)
    assert reader.offset == len(reader.data)
    return {'joint_count': joint_count, 'clip_count': clip_count,
            'hand_related_joints': [{key:value for key,value in joint.items() if key != 'trs'} for joint in joints if category(joint['name'])],
            'clips': clips}


if __name__ == '__main__':
    result = {'characters': {name: character(ROOT / 'assets' / name / 'character.bin') for name in ['choulingke', 'lihuowang', 'bailingmiao']},
              'source_animation': source_animation(ROOT / 'assets/choulingke/animations.bin')}
    for clip in result['source_animation']['clips']:
        tracks = clip['hand_related_tracks']
        clip['hand_category_track_counts'] = dict(collections.Counter(track['category'] for track in tracks))
        clip['varying_finger_rotation_tracks'] = sum(track['category'] == 'finger' and track['varying_TRS'][1] for track in tracks)
        if clip['name'] not in ['skill_ylk_1', 'skill_huajie_01__shuangqiangt']:
            del clip['hand_related_tracks']
    target = Path(__file__).with_name('hand-asset-counts.json')
    target.write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n')
    for name, body in result['characters'].items():
        print(name, 'driver count', len(body['drivers']))
        for part in body['parts']:
            if part['hand_related_bones']:
                categories = collections.Counter(bone['category'] for bone in part['hand_related_bones'])
                print(' ',part['name'], 'bones',dict(categories),'influenced vertices',part['vertices_with_category_influence'])
        mappings = collections.defaultdict(set)
        for part in body['parts']:
            for bone in part['hand_related_bones']:
                mappings[bone['driver']].add(bone['name'].split('::')[-1])
        print(' hand-related unique mappings', {key: sorted(value) for key,value in mappings.items()})
    source = result['source_animation']
    print('source joints', source['joint_count'], 'categories', dict(collections.Counter(category(joint['name']) for joint in source['hand_related_joints'])))
    for clip in source['clips']:
        if clip['name'] in ['skill_ylk_1','skill_huajie_01__shuangqiangt']:
            print('clip',clip['name'],'tracks',clip['tracks'],'hand categories',dict(collections.Counter(track['category'] for track in clip['hand_related_tracks'])))
    print('Wrote read-only inventory',target)
