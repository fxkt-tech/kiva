#!/usr/bin/env python3
"""Import verified named Messiah v15 channels and the recovered original rig.

Run with the extraction's numpy-equipped venv; the game remains read-only:
  /path/to/research/.venv/bin/python scripts/import-choulingke-animations.py --source /path/to/research

KIVASR01 (little endian): joint/clip counts u16; each joint has a u16-length
UTF-8 name, i16 parent, 10f local TRS (T3/Qxyzw4/S3); clips have a name,
u16 duration in ms and u16 track count. Each track has a u16 joint followed
by translation/rotation/scale: u16 key count, then (u16 ms, 3f/4f/3f) keys.
No runtime resource paths, Python, JSON, game files or mutable game state.
"""
import argparse
import hashlib
import json
from pathlib import Path
import struct
import numpy as np
from messiah_anim_v15 import parse_animation


def string(value):
    data = value.encode('utf-8')
    return struct.pack('<H', len(data)) + data


def quaternion(rotation):
    # Stable matrix->xyzw conversion, including rotations near 180 degrees.
    result = np.empty(4)
    trace = np.trace(rotation)
    if trace > 0:
        s = np.sqrt(trace + 1) * 2
        result[:] = [(rotation[2, 1] - rotation[1, 2]) / s,
                     (rotation[0, 2] - rotation[2, 0]) / s,
                     (rotation[1, 0] - rotation[0, 1]) / s, s / 4]
    else:
        i = int(np.argmax(np.diag(rotation)))
        j, k = (i + 1) % 3, (i + 2) % 3
        s = np.sqrt(1 + rotation[i, i] - rotation[j, j] - rotation[k, k]) * 2
        result[i] = s / 4
        result[j] = (rotation[j, i] + rotation[i, j]) / s
        result[k] = (rotation[k, i] + rotation[i, k]) / s
        result[3] = (rotation[k, j] - rotation[j, k]) / s
    return result / np.linalg.norm(result)


def decompose(matrix):
    scales = np.linalg.norm(matrix[:3, :3], axis=0)
    if np.min(scales) < 1e-8:
        raise ValueError('singular source local matrix')
    rotation = matrix[:3, :3] / scales
    if np.linalg.det(rotation) < 0:
        scales[0] *= -1
        rotation[:, 0] *= -1
    u, _, vh = np.linalg.svd(rotation)
    rotation = u @ vh
    error = float(np.max(abs(rotation * scales - matrix[:3, :3])))
    if error > 0.00001:
        raise ValueError(f'source matrix shear exceeds XML rounding: {error}')
    return [*matrix[:3, 3], *quaternion(rotation), *scales], error


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path, required=True)
    parser.add_argument('--output', type=Path, default=Path(__file__).resolve().parents[1] / 'assets/choulingke/animations.bin')
    args = parser.parse_args()
    source = args.source.resolve()
    hierarchy = json.loads((source / 'animation_yulingke_hierarchy.json').read_text())
    raw_skeleton = (source / 'extracted/animation/yulingke.skeleton').read_bytes()
    if hashlib.sha256(raw_skeleton).hexdigest() != hierarchy['source_sha256']:
        raise ValueError('hierarchy/source skeleton digest mismatch')
    body = (Path(__file__).resolve().parents[1] / 'assets/choulingke/character.bin').read_bytes()
    if body[:8] != b'KIVAAV02':
        raise ValueError('expected current normalized avatar asset')
    body_meta = json.loads(body[12:12 + struct.unpack_from('<I', body, 8)[0]])
    scale, floor = body_meta['source_scale'], body_meta['source_floor']
    conversion = np.diag([-scale, scale, -scale, 1.0])
    conversion[1, 3] = -floor * scale
    joints = hierarchy['joints']
    by_name = {joint['name']: joint for joint in joints}
    registry = json.loads((source / 'animation_yulingke_clips.json').read_text())
    extraction = json.loads((source / 'extracted/animation/yulingke_clips_extraction_manifest.json').read_text())
    physical = {path: record for record in extraction['resources'] for path in record['paths']}
    supplemental = {
        'skill_huajie_01__shuangqiangt',
        'b_fight_env_jump_01_start_Lfoot__zuiquan',
        'b_fight_env_jump_02_loop__zuiquan',
        'b_fight_env_jump_03_to_idle__zuiquan',
        'skill_shoujia_dodge_f',
    }
    for filename in ['animation_common_motion_probe.json', 'animation_dodge_motion_probe.json']:
        for entry in json.loads((source / filename).read_text())['entries']:
            item = entry['source']
            if item['name'] not in supplemental:
                continue
            supplemental.remove(item['name'])
            item = dict(item, resolved_path=item['res'], shared_retarget=True)
            registry.append(item)
            physical[item['resolved_path']] = dict(entry['metadata'], file=entry['file'])
    if supplemental:
        raise ValueError(f'missing explicitly selected shared clips: {supplemental}')
    clips, skipped, maximum_error = [], [], 0.0
    for item in registry:
        resource = physical.get(item['resolved_path'])
        if resource is None:
            skipped.append(dict(name=item['name'], reason='exact VFS not present in extracted indexes'))
            continue
        data = Path(resource['file']).read_bytes()
        if hashlib.sha256(data).hexdigest() != resource['sha256']:
            raise ValueError(f"changed extracted clip {item['name']}")
        clip = parse_animation(data)
        if any(b.name is None for b in clip.bones):
            skipped.append(dict(name=item['name'], reason='unresolved FC/FD/FE/FF bone-name dictionary',
                                explicit=sum(b.name is not None for b in clip.bones), referenced=sum(b.name is None for b in clip.bones)))
            continue
        accepted, omitted = [], []
        for bone in clip.bones:
            joint = by_name.get(bone.name)
            if joint is None:
                omitted.append(dict(name=bone.name, reason='not in target skeleton'))
                continue
            source_parent = clip.bones[bone.parent].name if bone.parent >= 0 else None
            target_parent = joints[joint['parent']]['name'] if joint['parent'] >= 0 else None
            # Clip roots omit Scene Root; all other local-space channels need an
            # exact named parent. Earring/sleeve variants are not safe to reuse.
            if source_parent != target_parent and not (source_parent is None and target_parent == 'Scene Root'):
                omitted.append(dict(name=bone.name, reason='different explicit parent',
                                    clip_parent=source_parent, rig_parent=target_parent))
                continue
            if item.get('shared_retarget') and bone.translation.values:
                # Common humanoid axes are explicitly named, but limb lengths
                # differ. Carry animated offsets, preserving the target rest
                # segment instead of stretching the target to the donor body.
                rest = np.array(joint['local_matrix_gltf']).reshape(4, 4).T[:3, 3]
                initial = np.array(bone.translation.values[0])
                ratio = np.linalg.norm(rest) / np.linalg.norm(initial) if np.linalg.norm(initial) > 1e-5 else 1
                bone.translation.values = [list(rest + (np.array(value) - initial) * ratio)
                                           for value in bone.translation.values]
            accepted.append((joint['index'], bone))
        if 'Bip001' not in [bone.name for _, bone in accepted]:
            raise ValueError('named clip lacks verified root channel')
        clips.append((item, clip, accepted, omitted, resource))
    output = bytearray(b'KIVASR01' + struct.pack('<HH', len(joints), len(clips)))
    for joint in joints:
        matrix = np.array(joint['local_matrix_gltf']).reshape(4, 4).T
        if joint['parent'] == -1:
            matrix = conversion @ matrix
        trs, error = decompose(matrix)
        maximum_error = max(maximum_error, error)
        output += string(joint['name']) + struct.pack('<h10f', joint['parent'], *trs)
    manifests = []
    for item, clip, tracks, omitted, resource in clips:
        output += string(item['name']) + struct.pack('<HH', clip.duration_ms, len(tracks))
        total_keys = 0
        for index, bone in tracks:
            output += struct.pack('<H', index)
            for keys, size in [(bone.translation, 3), (bone.rotation, 4), (bone.scale, 3)]:
                output += struct.pack('<H', len(keys.times_ms))
                total_keys += len(keys.times_ms)
                for time, values in zip(keys.times_ms, keys.values):
                    output += struct.pack('<H' + 'f' * size, time, *values)
        manifests.append(dict(name=item['name'], source_vfs=item['resolved_path'],
                              duration_ms=clip.duration_ms, registered_duration_ms=item['durationMS'],
                              source_sha256=resource['sha256'], physical={key: resource[key] for key in ('key', 'pack', 'offset', 'stored_size', 'index_sources')},
                              source_track_count=len(clip.bones), mapped_track_count=len(tracks),
                              source_sections=clip.sections, imported_key_count=total_keys, shared_humanoid_retarget=item.get('shared_retarget', False), omitted_tracks=omitted))
    args.output.write_bytes(output)
    manifest = dict(format='KIVASR01', binary_sha256=hashlib.sha256(output).hexdigest(),
                    bytes=len(output), skeleton_vfs=hierarchy['source_vfs'], skeleton_sha256=hierarchy['source_sha256'],
                    joint_count=len(joints), clip_count=len(clips), source_scale=scale, source_floor=floor,
                    source_yaw_degrees=180, maximum_local_matrix_TRS_error=maximum_error,
                    quaternion_encoding='20-bit smallest-three cyclic component order, source milliseconds retained; no temporal resampling',
                    quaternion_component_quantization_half_step=2 ** 0.5 / (2 * ((1 << 20) - 1)),
                    root_motion='Runtime strips planar Bip001 translation only; vertical waist motion retained, initial yaw aligned to rest. Simulation owns actor movement.',
                    runtime_adaptations=[
                        dict(state='melee', clip='skill_ylk_1', range_seconds=[0.90, 1.80], playback_seconds=0.42, role='Kiva visual crop, original skill-slot/hit-frame mapping unresolved'),
                        dict(state='heavy melee', clip='skill_ylk_1', range_seconds=[0.90, 1.80], playback_seconds=0.60, role='Kiva visual crop'),
                        dict(state='hit', clip='weakhit_b', range_seconds=[0, 0.30], playback_seconds=0.24, role='Kiva reaction crop'),
                        dict(state='parry', clip='skill_huajie_01__shuangqiangt', range_seconds=[0.1, 0.65], playback_seconds=0.25, role='original shared player parry, Kiva timing/limb-length retarget'),
                        dict(state='air start/loop/land', clips=['b_fight_env_jump_01_start_Lfoot__zuiquan', 'b_fight_env_jump_02_loop__zuiquan', 'b_fight_env_jump_03_to_idle__zuiquan'], role='original male shared jump sequence, physics owns root height'),
                        dict(state='dodge', clip='skill_shoujia_dodge_f', range_seconds=[0.2, 1.0], playback_seconds=0.42, role='original NPC dodge, Kiva crop and facing adaptation'),
                    ],
                    unresolved=['FC/FD/FE/FF shared bone-name dictionary', 'target-specific locomotion/jump/dodge/guard shared bone-name dictionary; selected common named jump/dodge/huajie clips are humanoid retargets',
                                'boss skill-slot to original clip/hit-frame mapping', 'independent accessory skeletons and cloth physics (fallback)'],
                    clips=manifests, skipped=skipped,
                    reproduce='research/.venv/bin/python scripts/import-choulingke-animations.py --source research')
    args.output.with_name('animation_manifest.json').write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n')
    print(f'{len(joints)} joints, {len(clips)} clips, {len(output):,} bytes; TRS max error {maximum_error:.3g}')


if __name__ == '__main__':
    main()
