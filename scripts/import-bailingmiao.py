#!/usr/bin/env python3
"""Prepare portable original Bai/Lianmu resources; never integrate them at runtime.

  python scripts/import-bailingmiao.py --source /path/to/naraka
  python scripts/import-bailingmiao.py --verify assets/prepared/naraka/bailingmiao

Requires numpy/Pillow, and reuses the published import-naraka GLB reader read-only.
No game installation is needed to verify an already prepared portable package.
"""
import argparse
import hashlib
import importlib.util
import json
from pathlib import Path

import numpy as np
from PIL import Image

SPEC = importlib.util.spec_from_file_location('naraka_reader', Path(__file__).with_name('import-naraka.py'))
NARAKA = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(NARAKA)
require = NARAKA.require
sha = NARAKA.sha
COMPONENTS = {
    'outfit': ('outfit.glb', '3606106f8f8f55e76eab8063f733725f221363c50faa4945469d98feaf3bd8f2'),
    'lianmu': ('lianmu.glb', 'fddd7ce6283909ff017fc6c0c27d37eb57db8c2f8147529b84b95559971b8d05'),
    'generic_female_face': ('generic_female_face.glb', '5053332a25a1291465077e389b6f15844c48ad13c1310e03490f63fe466c02a8'),
}


def checked_path(root, relative):
    require(not Path(relative).is_absolute(), 'portable path must be relative')
    path = (root / relative).resolve()
    require(path.is_relative_to(root.resolve()), 'portable path leaves package')
    return path


def copy_verified(source, root, relative, role, records, expected=None):
    data = source.read_bytes()
    require(expected is None or sha(data) == expected, f'source hash mismatch: {source.name}')
    target = checked_path(root, relative)
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_bytes(data)
    record = {'file': str(Path(relative).as_posix()), 'sha256': sha(data), 'bytes': len(data), 'role': role}
    if record not in records:
        records.append(record)
    return record


def write_json(root, relative, value, role, records):
    target = checked_path(root, relative)
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(json.dumps(value, ensure_ascii=False, indent=2) + '\n')
    record = {'file': str(Path(relative).as_posix()), 'sha256': sha(target.read_bytes()),
              'bytes': target.stat().st_size, 'role': role}
    records.append(record)
    return record


def copy_identity(directory, root, records):
    documents = []
    names = ['identity_manifest.json', 'target_parts.json', 'face_identity_records.json', 'colour_identity_limits.json']
    raw_records = {}

    def visit(value):
        if isinstance(value, dict):
            if value.get('record_file') and value.get('record_sha256'):
                original = Path(value['record_file'])
                path = directory / original.name
                if not path.exists():
                    path = original
                require(path.is_file() and path.suffix == '.bin' and path.stat().st_size < 1024 * 1024,
                        'missing or oversized original identity record')
                raw_records[path.name] = (path, value['record_sha256'])
            for nested in value.values():
                visit(nested)
        elif isinstance(value, list):
            for nested in value:
                visit(nested)
    for name in names:
        path = directory / name
        if not path.exists():
            continue
        documents.append(copy_verified(path, root, 'source-evidence/' + name, 'historical-original-identity-json', records))
        visit(json.loads(path.read_text()))
    for name, (path, expected) in sorted(raw_records.items()):
        copy_verified(path, root, 'source-evidence/records/' + name, 'original-identity-record', records, expected)
    return documents


def verify_glb(path, expected):
    """Read actual accessors/skins and perturb a genuinely weighted source joint."""
    glb = NARAKA.Glb(path, expected)
    local = [np.array(n['matrix']).reshape((4, 4), order='F') for n in glb.doc['nodes']]
    checks = []
    vertices, triangles, skins = 0, 0, 0
    for mesh in glb.meshes():
        skin = mesh['skin']
        if skin:
            skins += 1
            require(np.isfinite(mesh['binds']).all() and (abs(np.linalg.det(mesh['binds'])) > 1e-8).all(), 'invalid source inverse binds')
        for p in mesh['primitives']:
            count = len(p['POSITION'])
            vertices += count
            triangles += len(p['indices']) // 3
            posed = glb.posed(mesh, p)
            require(np.isfinite(posed).all(), 'nonfinite source posed vertices')
            check = {'mesh': mesh['name'], 'vertices': count, 'native_skin': bool(skin),
                     'source_pose_bounds': [posed.min(0).tolist(), posed.max(0).tolist()]}
            if skin:
                contribution = np.bincount(p['joints'].reshape(-1), weights=p['weights'].reshape(-1), minlength=len(skin['joints']))
                slot = int(np.argmax(contribution))
                joint = skin['joints'][slot]
                changed = [m.copy() for m in local]
                c, s = np.cos(.2), np.sin(.2)
                rotation = np.eye(4)
                rotation[:3, :3] = [[c, -s, 0], [s, c, 0], [0, 0, 1]]
                changed[joint] = changed[joint] @ rotation
                cache = {}
                def world(i):
                    if i not in cache:
                        cache[i] = world(glb.parents[i]) @ changed[i] if i in glb.parents else changed[i]
                    return cache[i]
                matrices = np.array([world(j) @ b for j, b in zip(skin['joints'], mesh['binds'], strict=True)])
                points = np.column_stack((p['POSITION'], np.ones(count)))
                after = (np.einsum('nvij,nj->nvi', matrices[p['joints']], points) * p['weights'][:, :, None]).sum(1)[:, :3]
                movement = np.linalg.norm(after - posed, axis=1)
                require(np.isfinite(after).all() and movement.max() > 1e-7, 'source weighted joint perturbation failed')
                check.update({'palette_slots': len(skin['joints']), 'perturbed_source_joint': glb.doc['nodes'][joint]['name'],
                              'radians': .2, 'maximum_deformation_source_units': float(movement.max())})
            checks.append(check)
    require(vertices > 0 and triangles > 0, 'empty prepared geometry')
    return {'file': path.name, 'sha256': expected, 'vertices': vertices, 'triangles': triangles,
            'meshes': len(list(glb.meshes())), 'native_skinned_meshes': skins, 'primitive_checks': checks,
            'original_units_and_poses_preserved': True, 'browser_or_screenshot_test': False}


def verify_package(directory):
    directory = directory.resolve()
    manifest = json.loads((directory / 'portable_manifest.json').read_text())
    require(manifest['prepared_only'] and not manifest['runtime_integrated'], 'package must remain resource-only')
    paths = set()
    glbs = []
    pngs = 0
    for record in manifest['files']:
        path = checked_path(directory, record['file'])
        require(record['file'] not in paths, 'duplicate portable file entry')
        paths.add(record['file'])
        raw = path.read_bytes()
        require(len(raw) == record['bytes'] and sha(raw) == record['sha256'], 'portable file hash/size mismatch: ' + record['file'])
        if path.suffix == '.glb':
            glbs.append(verify_glb(path, record['sha256']))
        elif path.suffix == '.png':
            with Image.open(path) as image:
                image.verify()
            pngs += 1
    require(glbs, 'prepared package lacks original model')
    indexed = {record['file']: record for record in manifest['files']}
    declared_models = manifest.get('components', manifest.get('models', [])) + manifest.get('related_models', [])
    require(declared_models, 'prepared package lacks declared model inventory')
    for model in declared_models:
        require(model['file'] in indexed and indexed[model['file']]['sha256'] == model['sha256'],
                'declared source model is absent or differs from portable inventory')
    return {'passed': True, 'files': len(paths), 'glbs': glbs, 'png_files': pngs,
            'portable_paths_only': True, 'game_or_research_directory_required_for_verify': False,
            'browser_or_screenshot_test': False}


def prepare(source, output):
    source = source.resolve()
    origin = source / 'export/character/bailingmiao'
    manifest = json.loads((origin / 'manifest.json').read_text())
    require(manifest['identity']['body'] == 3402139 and manifest['identity']['hair'] == 100408
            and manifest['identity']['body_size'] == 90005, 'wrong Bai/Lianmu/female source identity')
    output.mkdir(parents=True, exist_ok=True)
    records, checks = [], []
    components = []
    for role, (filename, expected) in COMPONENTS.items():
        path = origin / filename
        check = verify_glb(path, expected)
        require(check['native_skinned_meshes'] > 0, 'original character component lost its skin')
        checks.append(check)
        components.append(copy_verified(path, output, 'models/' + filename, 'original-source-skin-glb:' + role, records, expected))
        for suffix in ('.source.json', '.cpu-verification.json'):
            evidence = path.with_suffix(suffix)
            copy_verified(evidence, output, 'source-evidence/components/' + evidence.name, 'historical-source-model-evidence', records)
    for texture in manifest['textures']:
        path = Path(texture['png'])
        copy_verified(path, output, 'textures/' + path.name, 'original-material-texture', records, texture['png_sha256'])
    cosmetic = json.loads((origin / 'cosmetics/manifest.json').read_text())
    for texture in cosmetic['textures']:
        path = Path(texture['png'])
        copy_verified(path, output, 'cosmetics/' + path.name, 'original-cosmetic-texture', records, texture['png_sha256'])
    for path in sorted((origin / 'derived_preview').glob('*.png')):
        copy_verified(path, output, 'derived-preview/' + path.name, 'derived-neutral-alpha-preview-mask', records)
    identity = copy_identity(source / 'identity/bailingmiao', output, records)
    for name in ('manifest.json', 'female_rig.json', 'female_face_source.json'):
        copy_verified(origin / name, output, 'source-evidence/origin-' + name, 'historical-origin-metadata', records)
    copy_verified(origin / 'cosmetics/manifest.json', output, 'source-evidence/cosmetic-textures.json', 'historical-origin-metadata', records)
    icons = source / 'identity/bailingmiao/reference_icons'
    if icons.is_dir():
        for path in sorted(icons.iterdir()):
            if path.suffix in ('.json', '.png'):
                copy_verified(path, output, 'reference-icons/' + path.name, 'original-reference-icon', records)
    report = {'prepared_only': True, 'runtime_integrated': False, 'source_identity': manifest['identity'],
              'components': checks, 'original_material_pngs': len(manifest['textures']),
              'original_cosmetic_pngs': len(cosmetic['textures']),
              'specific_face_preset_morph_and_cosmetics_shader_applied': False,
              'lianmu_shader_dye_recovered': False, 'normal_packed_maps_reinterpreted': False,
              'female_source_avatar': 'l22_female_skeletonAvatar',
              'browser_or_screenshot_test': False}
    write_json(output, 'verification-report.json', report, 'offline-verification-report', records)
    readme = '''# 白灵淼衣装与莲母：仅资源准备

本目录没有接入 Kiva 外观、WASM、include_bytes 或武器玩法。保留原坐标/单位的三个原蒙皮组件：白灵淼 s16 衣装、莲母 hair_s19、独立通用女性基脸。此包不是已还原专属捏脸的完整运行时角色，也没有 AV03 或新武器。

身份：FashionBody3402139（排除新喜3402140）→ActorTemp1210776→BodySize90005→原 l22_female_skeletonAvatar；ActorVisualPart11602 为 s16。莲母为 HairItem100408 / HairVisual100471。女性也沿用引擎 gMan 骨名，实际矩阵来自女性源，未套男性比例/脸或NPC。

衣装保留16个原可视LOD0实例和各自原FBX骨序（按确切Mesh引用与骨路径CRC验证），空材质模拟代理不显示。莲母原SMR骨列表为空，28槽由原female63骨与原18aux矩阵唯一匹配，7条外部父链接为明确的离线重建，非原序列化骨列表。所有原local SRT、UV、四权重与IBMs保留，当前源姿态可能不同于bind pose。

textures/中的27张原材质PNG、cosmetics/中的3张确切原妆容PNG均保留原尺寸/通道。normal/packed/em/ID/SH/flow/LUT仅保留其真实来源/槽名，未猜作ORM。眼影是通道mask；原iris引用中的male命名保留，不改成李火旺虹膜。专属FacePreset1416403形变和妆容shader组合未应用。莲母染色/SH未恢复：GLB只有明确的中性浅灰alpha预览（derived-preview/白RGB仅留原alpha），不是把ID彩色数据当albedo，也未沿用李火旺黑发/棕眼。原参考图标单独留作源配色对照，不声称取得shader颜色参数。

portable_manifest.json所有实际交付文件均用本目录相对路径与SHA覆盖，验证不需要游戏或原WSL目录。source-evidence/里的原JSON含历史绝对源定位；它们只是来源证据，验证器不读取这些路径。小型原配置record.bin已随包，CAB/GPU压缩数据留在项目外研究目录。

在仓库根目录用有numpy/Pillow的Python执行：

    python scripts/import-bailingmiao.py --verify assets/prepared/naraka/bailingmiao

从原研究成果重新准备：

    python scripts/import-bailingmiao.py --source /path/to/naraka

检验包括实际GLB accessor/四权重/骨索引/IBMs、源层级计算、实际加权骨扰动、每个交付文件SHA和PNG解码。仅离线检查，没有运行游戏、浏览器或截图测试。
'''
    (output / 'README.md').write_text(readme)
    copy_verified(output / 'README.md', output, 'README.md', 'readme', records)
    portable = {'schema': 'naraka-prepared-v1', 'name': '白灵淼 / 莲母', 'prepared_only': True,
                'runtime_integrated': False, 'av03_generated': False, 'new_weapon_generated': False,
                'identity': manifest['identity'], 'components': components, 'identity_documents': identity,
                'files': records, 'origin_provenance': 'Archived original metadata only; historical absolute paths are not verification dependencies.',
                'limits': report}
    (output / 'portable_manifest.json').write_text(json.dumps(portable, ensure_ascii=False, indent=2) + '\n')
    result = verify_package(output)
    print(json.dumps({'output': str(output), 'passed': result['passed'], 'files': result['files'],
                      'models': len(result['glbs']), 'pngs': result['png_files'],
                      'runtime_integrated': False}, ensure_ascii=False))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path)
    parser.add_argument('--output', type=Path, default=Path(__file__).resolve().parents[1] / 'assets/prepared/naraka/bailingmiao')
    parser.add_argument('--verify', type=Path)
    args = parser.parse_args()
    if args.verify:
        print(json.dumps(verify_package(args.verify), ensure_ascii=False))
    elif args.source:
        prepare(args.source, args.output)
    else:
        parser.error('--source or --verify required')


if __name__ == '__main__':
    main()
