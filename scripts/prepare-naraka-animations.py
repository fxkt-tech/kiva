#!/usr/bin/env python3
"""Freeze original Naraka clips/Avatar, build oracle inputs, verify prepared data.

Extraction uses the user's read-only game and existing validated container tool.
Verification and native-local packing use only the Python standard library.
No source muscle approximation is accepted as a verified local-pose bake.
"""
from __future__ import annotations

import argparse
from collections import defaultdict
import json
import gzip
from pathlib import Path
import sys
import tempfile

from naraka_animation import (ScalarClip, avatar_rig, canonical_json,
                              pack_verified_locals, require, sha256)

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_PREPARED = ROOT / 'assets/prepared/naraka/animations'
MAX_JSON_BYTES = 128 * 1024**2
H0_NAMES = {'male_hengdao_attack_light_01', 'male_hengdao_idle_01',
            'male_hengdao_jump_down_loop_01', 'male_hengdao_finger_left_01',
            'male_hengdao_finger_right_01', 'male_bow_finger_default_left_01',
            'male_bow_finger_default_right_01'}


def load_json(path: Path):
    if not path.exists() and path.suffix == '.json' and path.with_suffix('.json.gz').exists():
        path = path.with_suffix('.json.gz')
    require(path.is_file() and path.stat().st_size <= MAX_JSON_BYTES, f'missing/oversized JSON: {path}')
    if path.suffix == '.gz':
        with gzip.open(path, 'rb') as stream:
            data = stream.read(MAX_JSON_BYTES + 1)
        require(len(data) <= MAX_JSON_BYTES, 'oversized decompressed JSON')
        return json.loads(data)
    return json.loads(path.read_text())


def write_json(path: Path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(canonical_json(value))


def safe_path(root: Path, relative: str) -> Path:
    require(isinstance(relative, str) and not Path(relative).is_absolute(), 'absolute prepared path')
    result = (root / relative).resolve()
    require(result.is_relative_to(root.resolve()), 'prepared path escape')
    return result


def source_record(obj, tree, bundle, key, path):
    raw = obj.get_raw_data()
    record = {'path': path, 'bundle_key': key, 'cab': obj.assets_file.name,
              'path_id': obj.path_id, 'byte_start': obj.byte_start,
              'byte_size': len(raw), 'raw_sha256': sha256(raw),
              'unity_version': obj.assets_file.unity_version,
              'type_hash': obj.serialized_type.old_type_hash.hex(),
              'original_type_tree': bool(obj.serialized_type.nodes),
              'proof': bundle.report(), 'tree': tree}
    require(record['original_type_tree'], 'source lacks original TypeTree')
    return record, raw


def extract(args, prepared: Path):
    require(args.source is not None and args.tools is not None, 'extraction needs --source and --tools')
    source = args.source.resolve()
    require(source.is_dir() and (args.tools / 'containers/naraka_sparse.py').is_file(), 'invalid source/tools directory')
    require(not prepared.resolve().is_relative_to(source), 'output cannot be inside the game installation')
    sys.path.insert(0, str(args.tools.resolve() / 'containers'))
    from naraka_sparse import SparseBundle
    import UnityPy

    selection = load_json(prepared / 'selection.json')
    require(selection['format'] == 'KIVA_NARAKA_SELECTION_1', 'invalid selection version')
    groups = defaultdict(list)
    for clip in selection['clips']:
        groups[clip['bundle_key']].append(clip)
    groups[selection['avatar']['bundle_key']].append(selection['avatar'])
    records, validation = [], []
    for key, items in sorted(groups.items()):
        relative = Path(key)
        require(not relative.is_absolute() and '..' not in relative.parts, 'unsafe source bundle key')
        with SparseBundle(source / key) as bundle:
            cab = next(n['name'] for n in bundle.nodes if n['flags'] & 4)
            env = UnityPy.Environment()
            sf = env.load_file(bundle.open_node(cab), name=cab)
            require(sf.unity_version == selection['unity_version'], 'source Unity version changed')
            for item in items:
                obj = sf.objects[item['path_id']]
                tree = obj.read_typetree(check_read=True)
                require(tree['m_Name'] == item['name'], 'source object identity changed')
                record, raw = source_record(obj, tree, bundle, key, item.get('path', item.get('asset_path')))
                require(record['raw_sha256'] == item['raw_sha256'], 'source object hash changed')
                raw_path = 'source/objects/' + record['raw_sha256'] + '.bin'
                record['raw_path'] = raw_path
                target = safe_path(prepared, raw_path)
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(raw)
                if obj.type.name == 'Avatar':
                    write_json(prepared / 'source/avatar.json', record)
                    write_json(prepared / 'source/rig.json', avatar_rig(tree))
                else:
                    require(obj.type.name == 'AnimationClip', 'selected object is not an AnimationClip')
                    clip = ScalarClip(tree)
                    require(abs(clip.duration - item['duration']) < 1e-5, 'source clip duration changed')
                    require(sum(clip.muscle_mask) == item['muscle_binding_count'], 'source authored mask changed')
                    errors = clip.endpoint_errors()
                    # Native endpoint descriptors independently validate scalar
                    # reading; this is deliberately not a Humanoid pose gate.
                    require(max(errors.values()) < 0.002, 'scalar endpoint reference disagrees')
                    validation.append({'name': clip.name, 'scalar_count': clip.scalar_count,
                                       'muscle_count': sum(clip.muscle_mask),
                                       'left_finger_count': sum(clip.muscle_mask[55:75]),
                                       'right_finger_count': sum(clip.muscle_mask[75:95]),
                                       'endpoint_max_abs_error': errors})
                    write_json(prepared / 'source/clips' / (clip.name + '.json'), record)
                records.append(record['raw_sha256'])
    write_json(prepared / 'source/curve-validation.json', {'scope': 'source scalar values only; H0 native pose validation is separate',
                                                         'clips': sorted(validation, key=lambda x: x['name'])})
    make_oracle_input(prepared)
    update_manifest(prepared)
    if args.native_bundles is not None:
        make_native_bundles(args, selection)
    print(f'Frozen {len(records) - 1} clips and original Avatar; native H0 is still required')


def make_native_bundles(args, selection):
    """Rebuild original native inputs externally, never inside the game/repo."""
    from naraka_container import decode_bundle
    output = args.native_bundles.resolve()
    require(not output.is_relative_to(args.source.resolve())
            and not output.is_relative_to(ROOT), 'native bundles must be outside the game installation and repository')
    output.mkdir(parents=True, exist_ok=True)
    groups = defaultdict(list)
    for clip in selection['clips']:
        groups[clip['bundle_key']].append(clip)
    rig_key = selection['avatar']['bundle_key']
    paths, bundles = {}, []
    for key in sorted(set(groups) | {rig_key}):
        original = decode_bundle(args.source / key)
        canonical = original.canonical()
        target = output / (key.rsplit('/', 1)[-1] + '.unityfs')
        target.write_bytes(canonical)
        paths[key] = str(target)
        if key in groups:
            bundles.append({'bundle_key': key, 'path': str(target),
                            'source_sha256': original.source_sha256,
                            'canonical_sha256': sha256(canonical), 'bytes': len(canonical),
                            'selected_clips': groups[key]})
    write_json(output / 'native-input-bundles.json', {
        'rig_bundle': paths[rig_key], 'rig_path': selection['avatar']['asset_path'],
        'animation_bundles': bundles})


def make_oracle_input(prepared: Path) -> bytes:
    selection = load_json(prepared / 'selection.json')
    rig = load_json(prepared / 'source/rig.json')
    clips = []
    for item in selection['clips']:
        record = load_json(prepared / 'source/clips' / (item['name'] + '.json'))
        clip = ScalarClip(record['tree'])
        entry = {'name': clip.name, 'source_sha256': record['raw_sha256'],
                 'source_path': record['path'], 'source_bundle_key': record['bundle_key'],
                 'duration': clip.duration, 'start': clip.start,
                 'authored_muscle_mask': clip.muscle_mask,
                 'animator_attributes': sorted(clip.animator_bindings)}
        if clip.name in H0_NAMES:
            entry.update(clip.oracle_clip())
        clips.append(entry)
    result = {'format': 'KIVA_NARAKA_ORACLE_1',
              'status': 'source-values-only; not a native bone pose',
              'avatar_source_sha256': selection['avatar']['raw_sha256'],
              'unity_source_version': selection['unity_version'],
              'rig': rig, 'runtime_normalization': selection['runtime_normalization'], 'clips': clips,
              'finger_layers': {'left': 'male_hengdao_finger_left_01',
                                'right': 'male_hengdao_finger_right_01',
                                'original_blend_order_weight_mask': 'unresolved; do not claim original runtime layering'},
              'animator_attribute_layout': 'Original Animator scalar encoding and authored mask; muscle index=attribute-42 for attributes42..136. Extra attributes remain opaque. These scalars are not assumed to be normalized HumanPose API values.',
              'native_bundle_paths': 'external native-input-bundles.json; canonical bundle bytes are not embedded into game assets'}
    data = canonical_json(result)
    (prepared / 'oracle-input.json').write_bytes(data)
    return data


def update_manifest(prepared: Path):
    records = []
    for p in sorted(prepared.rglob('*')):
        if p.is_file() and p.name != 'prepared-manifest.json' and p.suffix != '.md':
            data = p.read_bytes()
            records.append({'path': p.relative_to(prepared).as_posix(), 'bytes': len(data), 'sha256': sha256(data)})
    write_json(prepared / 'prepared-manifest.json', {
        'format': 'KIVA_NARAKA_PREPARED_1',
        'native_h0_verified': (prepared / 'baked/h0-report.json').is_file()
                              and load_json(prepared / 'baked/h0-report.json').get('passed') is True,
        'scope': 'source scalar/Avatar freeze and, only if separately verified, native-local bake',
        'files': records})


def verify(prepared: Path, *, repackage: bool = False):
    manifest = load_json(prepared / 'prepared-manifest.json')
    require(manifest['format'] == 'KIVA_NARAKA_PREPARED_1', 'invalid prepared manifest')
    seen = set()
    for record in manifest['files']:
        relative = record['path']
        require(relative not in seen, 'duplicate prepared file')
        seen.add(relative)
        path = safe_path(prepared, relative)
        require(path.is_file() and path.stat().st_size == record['bytes'], f'prepared size mismatch: {relative}')
        require(sha256(path.read_bytes()) == record['sha256'], f'prepared hash mismatch: {relative}')
    actual = {p.relative_to(prepared).as_posix() for p in prepared.rglob('*')
              if p.is_file() and p.name != 'prepared-manifest.json' and p.suffix != '.md'}
    require(actual == seen, 'prepared manifest file set mismatch')
    selection = load_json(prepared / 'selection.json')
    avatar = load_json(prepared / 'source/avatar.json')
    require(sha256(safe_path(prepared, avatar['raw_path']).read_bytes()) == selection['avatar']['raw_sha256'], 'original Avatar hash mismatch')
    require(canonical_json(avatar_rig(avatar['tree'])) == (prepared / 'source/rig.json').read_bytes(), 'Avatar rig reconstruction mismatch')
    maximum = 0.0
    for item in selection['clips']:
        record = load_json(prepared / 'source/clips' / (item['name'] + '.json'))
        require(sha256(safe_path(prepared, record['raw_path']).read_bytes()) == item['raw_sha256'], 'original clip hash mismatch')
        clip = ScalarClip(record['tree'])
        require(clip.name == item['name'] and sum(clip.muscle_mask) == item['muscle_binding_count'], 'prepared clip identity/mask mismatch')
        errors = clip.endpoint_errors()
        maximum = max(maximum, *errors.values())
        require(max(errors.values()) < 0.002, 'prepared scalar endpoint mismatch')
    before = (prepared / 'oracle-input.json').read_bytes()
    rebuilt = oracle_bytes(prepared)
    require(before == rebuilt, 'oracle input reconstruction mismatch')
    if manifest['native_h0_verified']:
        locals_doc = load_json(prepared / 'baked/locals.json')
        report = load_json(prepared / 'baked/h0-report.json')
        verify_native_identity(prepared, locals_doc, report, selection)
        native_report = load_json(prepared / 'baked/native-report.json')
        require(report['native_report_sha256'] == sha256(canonical_json(native_report)), 'native source report digest mismatch')
        require(report['native_source_checks'] == native_report, 'native source report differs from H0')
        with gzip.open(prepared / 'baked/native-capture.bin.gz', 'rb') as stream:
            capture = stream.read(256 * 1024**2 + 1)
        require(len(capture) <= 256 * 1024**2 and report['native_capture_sha256'] == sha256(capture), 'native capture digest mismatch')
        binary = pack_verified_locals(locals_doc)
        runtime = ROOT / 'assets/naraka/animations.bin'
        require(repackage or (runtime.is_file() and runtime.read_bytes() == binary), 'runtime binary rebuild mismatch')
        runtime_manifest = load_json(prepared / 'baked/runtime-manifest.json')
        require(runtime_manifest['sha256'] == sha256(binary) and runtime_manifest['bytes'] == len(binary), 'runtime manifest digest mismatch')
        require(runtime_manifest['h0_report_sha256'] == sha256(canonical_json(report)), 'runtime H0 report digest mismatch')
        interpolation = load_json(prepared / 'baked/interpolation-report.json')
        require(report['interpolation_report_sha256'] == sha256(canonical_json(interpolation)), 'interpolation report digest mismatch')
        require(interpolation['coarse_capture_sha256'] == report['native_capture_sha256'], 'interpolation native capture mismatch')
        with gzip.open(prepared / 'baked/reference-capture.bin.gz', 'rb') as stream:
            reference = stream.read(256 * 1024**2 + 1)
        require(len(reference) <= 256 * 1024**2 and report['independent_reference_sha256'] == sha256(reference), 'independent reference capture mismatch')
        mesh_path = prepared / 'source/real-hand-lbs.json'
        require(interpolation['real_hand_sha256'] == sha256(mesh_path.read_bytes()), 'independent real hand input mismatch')
        from naraka_animation_validation import verify_cuts, verify_sampling
        cut_count = verify_cuts(locals_doc, native_report, binary)
        sampling = verify_sampling(binary, locals_doc, reference, interpolation, load_json(mesh_path))
        print(f'Native interpolation: {sampling}; adjacent f32 cuts verified: {cut_count}')
        if repackage:
            runtime.parent.mkdir(exist_ok=True)
            runtime.write_bytes(binary)
            runtime.with_name('animations-manifest.json').write_bytes(canonical_json(runtime_manifest))
        else:
            require(runtime.with_name('animations-manifest.json').read_bytes() == canonical_json(runtime_manifest),
                    'runtime manifest differs from verified frozen bake')
    else:
        require(not repackage, 'native H0 required before runtime repackaging')
    print(f'Verified {len(selection["clips"])} original scalar clips, Avatar, oracle inputs; max endpoint error {maximum:.3g}; native H0={manifest["native_h0_verified"]}')


def oracle_bytes(prepared: Path) -> bytes:
    # Same builder as make_oracle_input, with output redirected into a temporary
    # tree of symlinked read-only input files. Never rewrite frozen inputs.
    with tempfile.TemporaryDirectory() as temp:
        stage = Path(temp)
        (stage / 'source').symlink_to((prepared / 'source').resolve(), target_is_directory=True)
        (stage / 'selection.json').symlink_to((prepared / 'selection.json').resolve())
        return make_oracle_input(stage)


def verify_native_identity(prepared: Path, document: dict, report: dict, selection: dict):
    require(report.get('passed') is True, 'native H0 report did not pass')
    require(report.get('avatar_source_sha256') == selection['avatar']['raw_sha256'], 'H0 Avatar source mismatch')
    require(document.get('avatar_source_sha256') == selection['avatar']['raw_sha256'], 'native-local Avatar source mismatch')
    require(report.get('locals_sha256') == sha256(canonical_json(document)), 'H0/native-local digest mismatch')
    expected = {c['name']: c['raw_sha256'] for c in selection['clips']}
    require(set(expected) == {c['name'] for c in document['clips']}, 'native-local clip selection mismatch')
    for clip in document['clips']:
        require(clip['name'] in expected and clip.get('source_sha256') == expected[clip['name']], 'native clip source mismatch')
    require(document.get('runtime_normalization') == selection['runtime_normalization'], 'native-local runtime normalization mismatch')
    source_checks = report['native_source_checks']
    require(source_checks.get('passed') is True and source_checks.get('avatar_is_human') is True
            and source_checks.get('real_skin_checked') is True and source_checks.get('real_skin_vertices', 0) >= 1000,
            'native report lacks actual original Avatar / weighted hand proof')
    require(document['rig']['rig_id'] == selection['rig_id'], 'native rig identity mismatch')
    frozen_rig = load_json(prepared / 'source/rig.json')
    require([(n['name'], n['parent']) for n in document['rig']['nodes']]
            == [(n['name'], n['parent']) for n in frozen_rig['nodes']], 'native-local hierarchy mismatch')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    group = parser.add_mutually_exclusive_group(required=True)
    group.add_argument('--extract', action='store_true')
    group.add_argument('--verify', action='store_true')
    group.add_argument('--repackage', action='store_true', help='fully verify frozen native evidence and regenerate runtime files')
    parser.add_argument('--source', type=Path, help='read-only StreamingAssets directory')
    parser.add_argument('--tools', type=Path, help='existing research root containing containers/naraka_sparse.py')
    parser.add_argument('--native-bundles', type=Path,
                        help='with --extract, rebuild canonical original bundles in an external cache')
    parser.add_argument('--prepared', type=Path, default=DEFAULT_PREPARED)
    args = parser.parse_args()
    prepared = args.prepared.resolve()
    if args.extract:
        extract(args, prepared)
    elif args.verify:
        verify(prepared)
    else:
        verify(prepared, repackage=True)


if __name__ == '__main__':
    main()
