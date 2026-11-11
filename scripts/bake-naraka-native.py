#!/usr/bin/env python3
"""Validate actual native captures, freeze verified locals and pack KIVASR02.

This validation compares the packed/FK data against independently exported
Unity world matrices and actual source weighted vertices, not a muscle guess.
"""
import argparse
import gzip
import importlib.util
import json
from pathlib import Path

import numpy as np
from naraka_animation import (canonical_json, pack_verified_locals, quaternion,
                              read_native_capture, require, sha256, unpack_runtime)

ROOT = Path(__file__).resolve().parents[1]


def matrix(pose):
    t = np.asarray(pose[:3], dtype=np.float64)
    x, y, z, w = quaternion(pose[3:7])
    scale = np.asarray(pose[7:], dtype=np.float64)
    r = np.array([[1 - 2*(y*y+z*z), 2*(x*y-z*w), 2*(x*z+y*w)],
                  [2*(x*y+z*w), 1 - 2*(x*x+z*z), 2*(y*z-x*w)],
                  [2*(x*z-y*w), 2*(y*z+x*w), 1 - 2*(x*x+y*y)]])
    result = np.eye(4)
    result[:3, :3] = r * scale
    result[:3, 3] = t
    return result


def fk(poses, nodes):
    result = []
    for pose, node in zip(poses, nodes, strict=True):
        local = matrix(pose)
        result.append(result[node['parent']] @ local if node['parent'] >= 0 else local)
    return np.array(result)


def runtime_pose(clip, time, nodes):
    result = [n['rest'][:] for n in nodes]
    for joint, channels in clip['tracks'].items():
        values = []
        for keys, start, width in zip(channels, (0, 3, 7), (3, 4, 3), strict=True):
            if not keys:
                values += nodes[joint]['rest'][start:start + width]
            elif len(keys) == 1:
                values += keys[0][1]
            else:
                # Native key timestamps are already f32, so the same captured
                # instant must be present without an interpolation assumption.
                found = next((v for t, v in keys if t == time), None)
                require(found is not None, 'packed clip lost a native sample instant')
                values += found
        result[joint] = values
    return result


def main():
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument('--capture', type=Path, required=True)
    p.add_argument('--native-report', type=Path, required=True)
    p.add_argument('--reference', type=Path, required=True,
                   help='independent offset native capture, not used to choose adaptive keys')
    p.add_argument('--interpolation-report', type=Path, required=True)
    p.add_argument('--real-hand', type=Path, required=True)
    p.add_argument('--prepared', type=Path, default=ROOT / 'assets/prepared/naraka/animations')
    p.add_argument('--sampler', type=Path, default=ROOT / 'scripts/naraka-native/NativeSampler.cs')
    args = p.parse_args()
    raw = args.capture.read_bytes()
    captured = read_native_capture(raw)
    report = json.loads(args.native_report.read_text())
    interpolation = json.loads(args.interpolation_report.read_text())
    reference_bytes = args.reference.read_bytes()
    selection = json.loads((args.prepared / 'selection.json').read_text())
    require(report.get('passed') is True and report.get('avatar_is_human') is True,
            'native source evaluation did not pass')
    require(report.get('real_skin_checked') is True and report.get('real_skin_vertices', 0) >= 1000,
            'native check did not use the real weighted hand vertices')
    require(report.get('adaptive') is True, 'production bake requires bounded native adaptive sampling')
    require(interpolation['coarse_capture_sha256'] == sha256(raw)
            and interpolation['reference_capture_sha256'] == sha256(reference_bytes), 'interpolation capture identity mismatch')
    require(interpolation['real_hand_sha256'] == sha256(args.real_hand.read_bytes()), 'interpolation real hand source changed')
    require(interpolation['maximum']['local_quaternion_max_error_degrees'] <= .1
            and all(value <= .001 for key, value in interpolation['maximum'].items() if key.endswith('_m')),
            'independent native interpolation fidelity threshold failed')
    require(report['avatar_source_sha256'] == selection['avatar']['raw_sha256'], 'native Avatar source changed')
    expected = {c['name']: c for c in selection['clips']}
    checks = {c['name']: c for c in report['clips']}
    require(set(checks) == {c['name'] for c in captured['clips']}, 'native report/capture clip set mismatch')
    require({'male_hengdao_attack_light_01', 'male_hengdao_idle_01', 'male_hengdao_jump_down_loop_01'} <= set(checks),
            'native H0 did not cover 95/75/55 channel cases')
    nodes = captured['rig']['nodes']
    frozen_rig = json.loads((args.prepared / 'source/rig.json').read_text())
    require([(n['name'], n['parent']) for n in nodes] == [(n['name'], n['parent']) for n in frozen_rig['nodes']],
            'native rig hierarchy differs from original Avatar')
    normalization = selection['runtime_normalization']
    captured['runtime_normalization'] = normalization
    captured['avatar_source_sha256'] = selection['avatar']['raw_sha256']
    captured['evaluator'].update(source_unity_version=selection['unity_version'],
                                base_sample_fps=report['capture_fps'], adaptive=report['adaptive'],
                                adaptive_rule=report['adaptive_rule'],
                                source_evaluator_patch_difference=selection['unity_version'] != report['unity_version'],
                                sampler_sha256=sha256(args.sampler.read_bytes()),
                                method='Original AnimationClip AssetBundle -> original Avatar -> manual native AnimationClipPlayable Graph; direct local TRS, no muscle approximation',
                                foot_ik=False, apply_root_motion=False)
    maximum_native_fk = 0.0
    for clip in captured['clips']:
        require(clip['name'] in expected and clip['source_sha256'] == expected[clip['name']]['raw_sha256'],
                'native capture source clip changed')
        check = checks[clip['name']]
        for key, limit in [('main_reference_local_degrees', .1), ('main_reference_local_position_m', .001),
                           ('default_reference_local_degrees', .1), ('default_reference_local_position_m', .001),
                           ('repeat_degrees', .1), ('repeat_position_m', .001), ('real_vertex_lbs_error_m', .001),
                           ('adaptive_replay_degrees', .1), ('adaptive_replay_position_m', .001)]:
            require(key in check and 0 <= check[key] <= limit, 'native fidelity check failed: ' + key)
        clip['composition'] = {'rule': check['composition'], 'left_layer': check.get('left_layer') or None,
                               'right_layer': check.get('right_layer') or None,
                               'authored_muscle_count': expected[clip['name']]['muscle_binding_count'],
                               'original_controller_blend_restored': False}
        for frame in clip['frames']:
            native = np.array(frame['native_world_column_major']).reshape((-1, 4, 4)).transpose((0, 2, 1))
            maximum_native_fk = max(maximum_native_fk, float(np.max(np.abs(fk(frame['locals'], nodes) - native))))
    require(maximum_native_fk <= .001, 'local TRS disagrees with native world-matrix FK')
    captured['h0_verified'] = True
    binary = pack_verified_locals(captured)
    decoded = unpack_runtime(binary)
    packed_clips = {c['name']: c for c in decoded['clips']}
    reflect = np.diag([1., 1., -1., 1.])
    maximum_packed_fk, maximum_local_angle, maximum_local_position = 0.0, 0.0, 0.0
    for clip in captured['clips']:
        packed = packed_clips[clip['name']]
        for frame in clip['frames']:
            poses = runtime_pose(packed, frame['time'], decoded['nodes'])
            actual = fk(poses, decoded['nodes'])
            native = np.array(frame['native_world_column_major']).reshape((-1, 4, 4)).transpose((0, 2, 1))
            want = reflect @ native @ reflect
            want[:, :3, 3] *= normalization['scale']
            want[:, 1, 3] += normalization['floor_offset_y']
            maximum_packed_fk = max(maximum_packed_fk, float(np.max(np.abs(actual - want))))
            for i, (pose, original) in enumerate(zip(poses, frame['locals'], strict=True)):
                q = np.asarray(quaternion(original[3:7])); q[:3] *= np.array([-1., -1., 1.])
                delta = abs(float(np.dot(q, quaternion(pose[3:7]))))
                angle = 2*np.degrees(np.arccos(min(1., delta)))
                maximum_local_angle = max(maximum_local_angle, float(angle))
                target = np.array(original[:3]) * np.array([1., 1., -1.]) * normalization['scale']
                if i == 0: target[1] += normalization['floor_offset_y']
                maximum_local_position = max(maximum_local_position, float(np.linalg.norm(np.array(pose[:3]) - target)))
    require(maximum_packed_fk <= .001 and maximum_local_angle <= .1 and maximum_local_position <= .001,
            'packed pose roundtrip exceeds native fidelity threshold')
    # The native world matrices are kept independently in the compressed raw
    # capture. Runtime/frozen local JSON never gains a second posed FK table.
    for clip in captured['clips']:
        for frame in clip['frames']:
            del frame['native_world_column_major']
    locals_bytes = canonical_json(captured)
    h0 = {'format': 'KIVA_NATIVE_H0_1', 'passed': True,
          'avatar_source_sha256': captured['avatar_source_sha256'],
          'locals_sha256': sha256(locals_bytes), 'native_capture_sha256': sha256(raw),
          'native_report_sha256': sha256(canonical_json(report)),
          'independent_reference_sha256': sha256(reference_bytes),
          'interpolation_report_sha256': sha256(canonical_json(interpolation)),
          'independent_interpolation': interpolation,
          'native_local_to_world_FK_max_abs_error': maximum_native_fk,
          'packed_native_world_FK_max_abs_error': maximum_packed_fk,
          'packed_local_quaternion_max_error_degrees': maximum_local_angle,
          'packed_local_position_max_error_m': maximum_local_position,
          'native_source_checks': report,
          'scope': 'Direct native original main graph and native original default chains; no claim of exact f2/f1 patch equivalence or recovered original controller weights. Inverse HumanPose diagnostic is intentionally excluded from bake.'}
    baked = args.prepared / 'baked'
    baked.mkdir(exist_ok=True)
    (baked / 'native-capture.bin.gz').write_bytes(gzip.compress(raw, mtime=0))
    (baked / 'locals.json.gz').write_bytes(gzip.compress(locals_bytes, mtime=0))
    (baked / 'h0-report.json').write_bytes(canonical_json(h0))
    (baked / 'native-report.json').write_bytes(canonical_json(report))
    (baked / 'reference-capture.bin.gz').write_bytes(gzip.compress(reference_bytes, mtime=0))
    (baked / 'interpolation-report.json').write_bytes(canonical_json(interpolation))
    (args.prepared / 'source/real-hand-lbs.json').write_bytes(args.real_hand.read_bytes())
    output = ROOT / 'assets/naraka/animations.bin'
    output.parent.mkdir(exist_ok=True)
    output.write_bytes(binary)
    manifest = {'format': 'KIVASR02', 'bytes': len(binary), 'sha256': sha256(binary),
                'rig_id': captured['rig']['rig_id'], 'joint_count': len(nodes), 'clip_count': len(captured['clips']),
                'runtime_normalization': normalization, 'h0_report_sha256': sha256(canonical_json(h0)),
                'native_evaluator': captured['evaluator'], 'clips': [{k:c[k] for k in ['name','source_sha256','duration','composition']} for c in captured['clips']]}
    output.with_name('animations-manifest.json').write_bytes(canonical_json(manifest))
    (baked / 'runtime-manifest.json').write_bytes(canonical_json(manifest))
    spec = importlib.util.spec_from_file_location('prepare', ROOT / 'scripts/prepare-naraka-animations.py')
    prepare = importlib.util.module_from_spec(spec); spec.loader.exec_module(prepare)
    prepare.update_manifest(args.prepared)
    print(json.dumps({k:v for k,v in h0.items() if k not in ('native_source_checks', 'independent_interpolation')},ensure_ascii=False,indent=2))
    print('runtime bytes',len(binary),'clips',len(captured['clips']),'baked compressed bytes',sum(p.stat().st_size for p in baked.iterdir()))


if __name__ == '__main__':
    main()
