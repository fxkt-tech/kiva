#!/usr/bin/env python3
"""Planning-only approximate muscle -> quaternion spike, NOT a Mecanim decoder.

Inputs are read-only original-source probes. Outputs prove binding consistency,
finite articulated motion and the size of the neutral-vs-import difference.
They do NOT prove native twist distribution, body-COM solve or humanoid IK.
"""
import hashlib
import json
import math
from pathlib import Path
import zlib

import numpy as np

HERE = Path(__file__).resolve().parent


def unwrap(value):
    while isinstance(value, dict) and set(value) == {'data'}:
        value = value['data']
    return value


def vector(value, keys='xyz'):
    return np.array([value[k] for k in keys], dtype=np.float64)


def quat(value):
    q = vector(value, 'xyzw')
    return q / np.linalg.norm(q)


def mul(a, b):
    xyz = a[3] * b[:3] + b[3] * a[:3] + np.cross(a[:3], b[:3])
    return np.r_[xyz, a[3] * b[3] - a[:3].dot(b[:3])]


def inverse(q):
    return np.r_[-q[:3], q[3]] / q.dot(q)


def rotate(q, v):
    return mul(mul(q, np.r_[v, 0.]), inverse(q))[:3]


def exp_quat(rotation_vector):
    angle = np.linalg.norm(rotation_vector)
    if angle < 1e-14:
        return np.array([0., 0., 0., 1.])
    return np.r_[rotation_vector * (math.sin(angle / 2) / angle), math.cos(angle / 2)]


def swing_twist(angles):
    return mul(exp_quat(np.array([0., angles[1], angles[2]])),
               exp_quat(np.array([angles[0], 0., 0.])))


def angular_distance(a, b):
    return math.degrees(2 * math.acos(float(np.clip(abs(a.dot(b)), -1, 1))))


def muscle_axes():
    """Candidate HumanTrait mapping; explicit, reviewable, not native queried."""
    result = {
        'Spine': (2, 1, 0), 'Chest': (5, 4, 3), 'UpperChest': (8, 7, 6),
        'Neck': (11, 10, 9), 'Head': (14, 13, 12),
    }
    for side, leg, arm, hand in [('Left', 21, 37, 55), ('Right', 29, 46, 75)]:
        result.update({
            side + 'UpperLeg': (leg + 2, leg + 1, leg),
            side + 'LowerLeg': (leg + 4, None, leg + 3),
            side + 'Foot': (None, leg + 6, leg + 5),
            side + 'Toes': (None, None, leg + 7),
            side + 'Shoulder': (None, arm + 1, arm),
            side + 'UpperArm': (arm + 4, arm + 3, arm + 2),
            side + 'LowerArm': (arm + 6, None, arm + 5),
            side + 'Hand': (None, arm + 8, arm + 7),
        })
        for digit, finger in enumerate(('Thumb', 'Index', 'Middle', 'Ring', 'Little')):
            base = hand + digit * 4
            result[side + ' ' + finger + ' Proximal'] = (None, base + 1, base)
            result[side + ' ' + finger + ' Intermediate'] = (None, None, base + 2)
            result[side + ' ' + finger + ' Distal'] = (None, None, base + 3)
    return result


def fk(nodes, translations, rotations):
    positions, world_rotations = [], []
    for node, t, q in zip(nodes, translations, rotations, strict=True):
        parent = node['m_ParentId']
        positions.append(t if parent < 0 else positions[parent] + rotate(world_rotations[parent], t))
        world_rotations.append(q if parent < 0 else mul(world_rotations[parent], q))
    return np.array(positions), np.array(world_rotations)


def main():
    avatar_file = HERE / 'humanoid-avatar.probe.json'
    clip_file = HERE / 'male_hengdao_attack_light_01.anim.probe.json'
    avatar_probe = json.loads(avatar_file.read_text())
    clip_probe = json.loads(clip_file.read_text())
    tree, clip_tree = avatar_probe['tree'], clip_probe['tree']
    avatar = tree['m_Avatar']
    human = unwrap(avatar['m_Human'])
    skeleton = unwrap(human['m_Skeleton'])
    poses = unwrap(human['m_SkeletonPose'])['m_X']
    nodes = skeleton['m_Node']
    axes = skeleton['m_AxesArray']

    by_name_hash = {}
    for _, path in tree['m_TOS']:
        name = path.rsplit('/', 1)[-1]
        digest = zlib.crc32(name.encode())
        assert digest not in by_name_hash or by_name_hash[digest] == name
        by_name_hash[digest] = name
    names = [by_name_hash[avatar['m_SkeletonNameIDArray'][i]]
             for i in avatar['m_HumanSkeletonIndexArray']]
    assert len(set(names)) == len(names) == 53
    name_to_human = {x['m_BoneName']: x['m_HumanName']
                     for x in tree['m_HumanDescription']['m_Human']}
    assert len(name_to_human) == 52
    for index, native in enumerate(avatar['m_HumanSkeletonIndexArray']):
        assert avatar['m_HumanSkeletonReverseIndexArray'][native] == index

    muscle_clip = clip_tree['m_MuscleClip']
    data = unwrap(muscle_clip['m_Clip'])
    streamed = data['m_StreamedClip']['curveCount']
    dense = data['m_DenseClip']
    constant = data['m_ConstantClip']['data']
    curves = streamed + dense['m_CurveCount'] + len(constant)
    offset, by_attribute = 0, {}
    for binding in clip_tree['m_ClipBindingConstant']['genericBindings']:
        dims = {1: 3, 2: 4, 3: 3, 4: 3}.get(binding['attribute'], 1) if binding['typeID'] == 4 else 1
        if binding['typeID'] == 95 and binding['customType'] == 8:
            attribute = binding['attribute']
            assert attribute not in by_attribute
            by_attribute[attribute] = offset
            assert muscle_clip['m_IndexArray'][attribute] == offset
        offset += dims
    assert offset == curves == 179
    assert all(42 + i in by_attribute for i in range(95))
    all_dense = np.array(dense['m_SampleArray']).reshape(dense['m_FrameCount'], dense['m_CurveCount'])
    muscles = np.zeros((dense['m_FrameCount'], 95))
    for index in range(95):
        column = by_attribute[index + 42]
        assert column >= streamed, 'this bounded probe does not implement streamed muscles'
        muscles[:, index] = (all_dense[:, column - streamed] if column < streamed + dense['m_CurveCount']
                             else constant[column - streamed - dense['m_CurveCount']])

    # m_DeltaPose is start minus end, NOT the initial pose for missing muscles.
    delta = muscle_clip['m_DeltaPose']
    stored_delta = np.array(delta['m_DoFArray'] + delta['m_LeftHandPose']['m_DoFArray']
                            + delta['m_RightHandPose']['m_DoFArray'])
    delta_error = float(np.abs(stored_delta - (muscles[0] - muscles[-1])).max())

    mapping = muscle_axes()
    used = {m for ids in mapping.values() for m in ids if m is not None}
    unused = sorted(set(range(95)) - used)
    assert unused == list(range(15, 21))
    assert not np.any(muscles[:, unused])
    translations = np.array([vector(x['t']) for x in poses])
    reference_rotations = np.array([quat(x['q']) for x in poses])
    reference_positions, _ = fk(nodes, translations, reference_rotations)
    neutral_rotations = reference_rotations.copy()
    bindings = []
    for i, name in enumerate(names):
        if name == 'MotionRoot' or name_to_human[name] == 'Hips':
            continue
        axis = axes[nodes[i]['m_AxesId']]
        pre, post = quat(axis['m_PreQ']), quat(axis['m_PostQ'])
        neutral_rotations[i] = mul(pre, inverse(post))
        bindings.append((i, axis, mapping[name_to_human[name]]))
    neutral_positions, _ = fk(nodes, translations, neutral_rotations)
    all_positions, all_rotations = [], []
    for values in muscles:
        local = reference_rotations.copy()
        for index, axis, ids in bindings:
            v = np.array([0. if m is None else values[m] for m in ids])
            # An explicit approximation to normalized muscle -> angular limits.
            # Keep out-of-range original muscle values; clamping loses motion.
            low, high = vector(axis['m_Limit']['m_Min']), vector(axis['m_Limit']['m_Max'])
            angle = v * np.where(v >= 0, high, -low) * vector(axis['m_Sgn'])
            q = mul(mul(quat(axis['m_PreQ']), swing_twist(angle)), inverse(quat(axis['m_PostQ'])))
            local[index] = q / np.linalg.norm(q)
        positions, _ = fk(nodes, translations, local)
        all_positions.append(positions)
        all_rotations.append(local)
    all_positions, all_rotations = np.array(all_positions), np.array(all_rotations)
    assert np.isfinite(all_positions).all() and np.isfinite(all_rotations).all()
    actual_lengths = np.array([np.linalg.norm(all_positions[:, i] - all_positions[:, n['m_ParentId']], axis=1)
                              for i, n in enumerate(nodes) if n['m_ParentId'] >= 0])
    reference_lengths = np.array([np.linalg.norm(translations[i])
                                 for i, n in enumerate(nodes) if n['m_ParentId'] >= 0])
    length_error = float(np.abs(actual_lengths - reference_lengths[:, None]).max())
    neutral_vs_import = {name: angular_distance(neutral_rotations[i], reference_rotations[i])
                         for i, name in enumerate(names)}
    local_span = {name: max(angular_distance(all_rotations[0, i], q) for q in all_rotations[:, i])
                  for i, name in enumerate(names)}
    trajectories = {name: float(np.linalg.norm(all_positions[:, i] - all_positions[0, i], axis=1).max())
                    for i, name in enumerate(names) if 'Hand' in name or 'Foot' in name}
    finger_names = [name for name in names if 'Finger' in name]
    body_names = [name for name in names if name not in finger_names and name != 'MotionRoot']
    report = {
        'purpose': 'approximate planning spike; NOT native Mecanim equivalence proof',
        'input_sha256': {f.name: hashlib.sha256(f.read_bytes()).hexdigest() for f in (avatar_file, clip_file)},
        'source_raw_sha256': {'avatar': avatar_probe['raw_sha256'], 'clip': clip_probe['raw_sha256']},
        'binding_checks': {'native_human_nodes': len(names), 'axes': len(axes), 'description_bones': len(name_to_human),
                           'full_curve_width': curves, 'streamed': streamed, 'dense': dense['m_CurveCount'],
                           'constant': len(constant), 'muscle_count': 95, 'candidate_mapped_muscles': len(used),
                           'missing_eye_jaw_zero_muscles': unused, 'deltaPose_start_minus_end_max_error': delta_error},
        'muscle_extrema': {'min': float(muscles.min()), 'max': float(muscles.max()),
                          'samples_outside_normalized_range': int((np.abs(muscles) > 1).sum())},
        'sample_count': len(muscles), 'frame_rate': dense['m_SampleRate'],
        'native_human_scale': human['m_Scale'],
        'length_error_normalized_units': length_error,
        'max_quaternion_unit_error': float(np.abs(np.linalg.norm(all_rotations, axis=2) - 1).max()),
        'moving_body_local_joints_gt_1_degree': sum(local_span[n] > 1 for n in body_names),
        'moving_finger_local_joints_gt_1_degree': sum(local_span[n] > 1 for n in finger_names),
        'neutral_vs_import_angle_degrees': neutral_vs_import,
        'candidate_motion_span_degrees': local_span,
        'candidate_extremity_motion_normalized_units': trajectories,
        'sample_frames': [{'frame': f, 'seconds': f / dense['m_SampleRate'],
                           'positions': {name: all_positions[f, i].tolist() for i, name in enumerate(names)},
                           'local_rotations_xyzw': {name: all_rotations[f, i].tolist() for i, name in enumerate(names)}}
                          for f in (0, 6, 12, 18, 23)],
        'not_implemented_or_proven': [
            'native HumanTrait axis and muscle normalization oracle; candidate explicit mapping only',
            'Mecanim twist distribution; Avatar arm/forearm/upperleg/lowerleg twist values are all 0.5',
            'root bodyPosition/bodyRotation to pelvis/center-of-mass solve and source goal IK',
            'human-scale/full native skeleton/basis conversion; this probe uses normalized human skeleton',
            'generic transform curves, beyond validating full curve width',
            'native Unity / original-game frame-by-frame quaternion or weighted-skin comparison',
        ],
    }
    target = HERE / 'humanoid-muscle-math.probe.json'
    target.write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
    print(json.dumps({k: v for k, v in report.items() if k not in ('sample_frames', 'neutral_vs_import_angle_degrees', 'candidate_motion_span_degrees')}, indent=2))


if __name__ == '__main__':
    main()
