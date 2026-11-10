#!/usr/bin/env python3
"""Prepare source Naraka weapon resources, without gameplay integration.

  python scripts/prepare-naraka-weapons.py --source /path/to/naraka
  python scripts/prepare-naraka-weapons.py --verify assets/prepared/naraka/yizhiying

Reads frozen project-external exports; verification needs only portable files,
numpy and Pillow. It never reads a game installation or modifies runtime code.
"""
import argparse
import importlib.util
import json
from pathlib import Path

SPEC = importlib.util.spec_from_file_location('naraka_portable', Path(__file__).with_name('import-bailingmiao.py'))
PORTABLE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(PORTABLE)
require = PORTABLE.require
TARGETS = {
    'yinbingluo': {'item_id': 2311228, 'name': '引冰落', 'part_ids': (3110620,),
                  'inventory_note': '引冰落场景保留两个真实原蒙皮renderer。',
                  'encoding_note': '引冰落原顶点蒙皮通道是 UInt32/dim1 单索引，没有独立的原权重数组。导出仅作已验证的窄展开：逐顶点索引数值保留，implicit weight=1 显式写成 glTF 四槽 [1,0,0,0]。其中 135 个原 joint1 顶点仍指向原骨槽1；没有将不明值回退到joint0。原类型、索引证据与展开范围保留在 source-evidence/weapon-export.original.json。'},
    'yizhiying': {'item_id': 2310557, 'name': '羿之影', 'part_ids': (3112195, 3112196, 3112197),
                 'inventory_note': '羿之影的9骨弓本体、静态quiver与arrowbag三个配置部件均保留。',
                 'related_note': 'related/另保留独立原箭体Mesh（635顶点/658三角形）。它由唯一原资源路径及同原stem关联，未恢复原Actor装配或材质绑定；保持静态/中性展示，不算第4个配置装备part，不按同名arrow材质猜绑定。'},
    'fulangmao': {'item_id': 2310478, 'name': '伏浪锚', 'part_ids': (3112349,),
                 'encoding_note': '伏浪锚主体保留原四权重/四索引流；附属448顶点原为UInt32/dim1单索引、无独立权重通道，仅将原索引与implicit weight1窄展开为glTF四槽。原索引字节、布局、骨路径CRC与逐骨LBS数值核验均在source-evidence/additional及model-checks内，不把派生展开称原四权重。'},
    'zhexingcuishan': {'item_id': 2311638, 'name': '谪星·摧山', 'part_ids': (3112012,),
                     'model_intro': 'models/区分唯一配置主部件与受原控制器引用的独立刀刃/特效输入；各GLB单独保存，不把全部输入叠加成一份默认外观，也不算新增WeaponTemp。',
                     'encoding_note': '原物品属于谪星：RareSkin1022引用颜色、刃形、神威、凶魂溯四类词条。随包保留原记录、规则及实际材质/数值输入；原随机星格选择、刀刃变式、溶解与灵光行为未执行。基础几何本身不证明已还原默认完整刀形或任一随机星格结果，未知通道未猜成颜色或形变算法。'},
}


def check_source_parts(original, identity, config):
    """Check the exact configuration chain; static parts need no synthetic skin."""
    item = config['item_id']
    expected = set(config['part_ids'])
    require(original['identity']['item_id'] == identity['item_id'] == item,
            'wrong original weapon identity')
    require({part['id'] for part in identity['parts']} == expected
            and set(original['identity']['weapon_temp_ids']) == expected,
            'weapon configuration part identity mismatch')
    require({record['weapon_temp_id'] for record in original['glbs']} == expected,
            'configuration-confirmed weapon attachments missing or unconfirmed part added')
    require(any(record['role'] == 'main' and record['weapon_temp_id'] == config['part_ids'][0]
                for record in original['glbs']), 'configuration-confirmed main weapon missing')
    require(len({Path(record['file']).name for record in original['glbs']}) == len(original['glbs']),
            'source model filenames collide')


def prepare_target(source, output_root, target):
    config = TARGETS[target]
    item, label = config['item_id'], config['name']
    origin = source / 'export/weapon' / target
    original = json.loads((origin / 'manifest.json').read_text())
    identity_parts = json.loads((source / 'identity' / target / 'target_parts.json').read_text())
    check_source_parts(original, identity_parts, config)
    source_identity_hash = original['identity'].get('source_identity_sha256')
    if source_identity_hash:
        require(PORTABLE.sha((source / 'identity' / target / 'identity_manifest.json').read_bytes()) == source_identity_hash,
                'source identity document differs from export provenance')
    output = output_root / target
    output.mkdir(parents=True, exist_ok=True)
    records, models, textures, checks, related_models, related_checks = [], [], [], [], [], []
    for record in original['glbs']:
        path = Path(record['file'])
        if not path.is_absolute():
            path = origin / path
        check = PORTABLE.verify_glb(path, record['sha256'])
        require(bool(record['native_skin']) == bool(check['native_skinned_meshes']),
                'source native/static skin type differs from exported geometry')
        require(record.get('skins', check['native_skinned_meshes']) == check['native_skinned_meshes'],
                'source native skin count differs from exported geometry')
        checks.append(check)
        local = PORTABLE.copy_verified(path, output, 'models/' + path.name,
                                       'original-weapon-glb:' + record['role'], records, record['sha256'])
        models.append({'file': local['file'], 'sha256': local['sha256'], 'role': record['role'],
                       'native_skinned_meshes': check['native_skinned_meshes']})
        verification = record.get('cpu_verification')
        if verification:
            require(verification['passed'], 'source LBS/static original-geometry verification failed')
            path = Path(verification['file'])
            if not path.is_absolute():
                path = origin / path
            PORTABLE.copy_verified(path, output, 'source-evidence/model-checks/' + path.name,
                                   'original-source-geometry-verification', records, verification.get('sha256'))
        khronos = record.get('khronos_validation')
        if khronos:
            require(khronos['errors'] == 0, 'source Khronos validation failed')
            path = Path(khronos['file'])
            if not path.is_absolute():
                path = origin / path
            PORTABLE.copy_verified(path, output, 'source-evidence/model-checks/' + path.name,
                                   'original-source-khronos-verification', records, khronos.get('sha256'))
    for record in original['textures']:
        path = Path(record['png'])
        if not path.is_absolute():
            path = origin / path
        local = PORTABLE.copy_verified(path, output, 'textures/' + path.name,
                                       'original-material-texture', records, record['png_sha256'])
        texture = {'file': local['file'], 'sha256': local['sha256'],
                   'source': record.get('source'), 'source_binding': record.get('slot', record.get('slots')),
                   'width': record.get('width'), 'height': record.get('height'),
                   'dimensions_channels': 'Original decoded PNG unchanged; no resize/recolor.',
                   'runtime_channel_mapping': 'Only source bindings are retained; unknown packed/normal/em/ID semantics are not guessed.'}
        if record.get('display_only', record.get('preview_only', False)):
            texture.update(display_only=True, original_format=record.get('original_format', record.get('format')),
                           pixel_editing=record.get('pixel_editing'),
                           dimensions_channels='PNG is a display-only copy; original numerical precision is retained separately in the declared raw/npy/layout evidence, not in this PNG.')
        textures.append(texture)
    PORTABLE.copy_verified(origin / 'manifest.json', output, 'source-evidence/weapon-export.original.json',
                           'historical-original-export-manifest', records)
    identity = PORTABLE.copy_identity(source / 'identity' / target, output, records)
    additional = []
    for evidence in original.get('additional_evidence', []):
        path = Path(evidence['file'])
        if not path.is_absolute():
            path = origin / path
        require(path.stat().st_size <= 4 * 1024 * 1024, 'source evidence must remain a small metadata artifact')
        relative = 'source-evidence/additional/' + path.name
        require(not any(record['file'] == relative for record in records), 'additional source evidence filename collision')
        additional.append(PORTABLE.copy_verified(path, output, relative, evidence['role'], records, evidence['sha256']))
    related_path = origin / 'related/manifest.json'
    if related_path.is_file():
        related = json.loads(related_path.read_text())
        require(related['identity']['related_to_item_id'] == item and not related['identity']['equipped_part'],
                'related geometry falsely listed as a configured equipment part')
        for record in related['glbs']:
            path = Path(record['file'])
            if not path.is_absolute():
                path = related_path.parent / path
            related_checks.append(PORTABLE.verify_glb(path, record['sha256']))
            local = PORTABLE.copy_verified(path, output, 'related/models/' + path.name,
                                           'related-original-geometry-not-equipped', records, record['sha256'])
            related_models.append({'file': local['file'], 'sha256': local['sha256'], 'role': record['role'],
                                   'equipped_part': False, 'native_skin': record.get('native_skin', False)})
            for key in ('cpu_verification', 'khronos_validation'):
                evidence = record.get(key)
                if not evidence:
                    continue
                path = Path(evidence['file'])
                if not path.is_absolute():
                    path = related_path.parent / path
                PORTABLE.copy_verified(path, output, 'related/source-evidence/' + path.name,
                                       'related-source-geometry-verification', records, evidence.get('sha256'))
        evidence = Path(related['source_evidence'])
        if not evidence.is_absolute():
            evidence = related_path.parent / evidence
        PORTABLE.copy_verified(evidence, output, 'related/source-evidence/' + evidence.name,
                               'related-original-source-evidence', records)
        PORTABLE.copy_verified(related_path, output, 'related/source-evidence/manifest.original.json',
                               'related-historical-original-manifest', records)
    report = {'prepared_only': True, 'runtime_integrated': False, 'identity': original['identity'],
              'models': checks, 'related_geometry_not_equipped': related_checks, 'original_texture_entries': len(textures),
              'source_shader_reconstructed': False, 'new_weapon_actions_or_grip_bake': False,
              'browser_or_screenshot_test': False}
    PORTABLE.write_json(output, 'verification-report.json', report, 'offline-verification-report', records)
    encoding_note = config.get('encoding_note', '')
    related_note = config.get('related_note', '') if related_models else ''
    if related_models and not related_note:
        related_note = f'related/单列{len(related_models)}份有来源证据的关联几何，不算原配置装备part；原装配及材质证据的边界见各自source-evidence。'
    inventory_note = config.get('inventory_note')
    if inventory_note is None:
        roles = '、'.join(record['role'] for record in original['glbs'])
        skins = sum(check['native_skinned_meshes'] for check in checks)
        static = sum(check['meshes'] - check['native_skinned_meshes'] for check in checks)
        inventory_note = f'原配置部件ID为{list(config["part_ids"])}；GLB源输入标签为{roles}，共{skins}个原蒙皮实例与{static}个原静态实例。'
    model_intro = config.get('model_intro', 'models/按原配置明确部件分别提供GLB。')
    precision_note = ''
    gpu_note = '原GPU压缩数据/CAB留在外部研究目录，未塞入本包。'
    if any(texture.get('display_only') for texture in textures):
        precision_note += '\n\n标记display_only的Float/形变PNG仅供显示，含明确clamp等预览转换；原数值以随包raw/npy与布局/SHA证据为准，未用8bitPNG冒充原Float/signed精度。'
    if original.get('non_png_textures'):
        precision_note += '\n\n原绑定Cubemap等非普通2D纹理没有伪展平为PNG；完整原GPU/对象/face与mip布局证据随additional_source_evidence保存，尚未执行其原反射/FX材质行为。'
    if precision_note:
        gpu_note = '完整CAB及普通纹理GPU压缩数据留在外部研究目录；本目标必要的形变数值及非2D纹理原数据作为明确例外随包保存。'
    readme = f'''# {label}：原武器资源准备

原物品ID {item}。本目录仅供后续接入，未改变 Kiva 枪械、弓箭、动作、输入、WASM 或任何 runtime/include_bytes。原坐标/比例、原层级、UV、材质引用与源骨槽保留，未做 Kiva 握柄轴重定向或刚性烘焙。

{model_intro}原本静态的附件保持静态，不补造骨；有蒙皮的主体保留源骨序、逆绑定和规范化 glTF 蒙皮表示，以及原当前姿态。原导入姿态未必为bind pose。{inventory_note}

{encoding_note}

{related_note}

textures/保留全部实际绑定的原decoded PNG尺寸/通道（含normal、packed、em、ID等）；GLB基础色是中性PBR预览，原特殊shader、未知packed通道/法线约定及动态效果并未恢复。{gpu_note}{precision_note}

portable_manifest.json用相对路径、SHA和字节数覆盖所有交付文件。source-evidence/原JSON中的绝对路径仅是历史来源定位，验证器不会访问原WSL或游戏目录；已包含小型原身份record.bin与源元数据。没有浏览器、截图或游戏执行验收。

在仓库根目录，用numpy/Pillow环境离线验证：

    python scripts/prepare-naraka-weapons.py --verify assets/prepared/naraka/{target}

从已冻结研究export重新准备：

    python scripts/prepare-naraka-weapons.py --source /path/to/naraka --target {target}
'''
    path = output / 'README.md'
    path.write_text(readme)
    PORTABLE.copy_verified(path, output, 'README.md', 'readme', records)
    manifest = {'schema': 'naraka-prepared-v1', 'name': label, 'prepared_only': True,
                'runtime_integrated': False, 'identity': original['identity'], 'models': models,
                'textures': textures, 'related_models': related_models, 'identity_documents': identity, 'files': records,
                'origin_provenance': 'Archived original JSON paths are historical evidence, never local verification dependencies.',
                'limits': original.get('limits', []), 'original_coordinate_and_skin_preserved': True}
    if additional:
        manifest['additional_source_evidence'] = additional
    (output / 'portable_manifest.json').write_text(json.dumps(manifest, ensure_ascii=False, indent=2) + '\n')
    result = PORTABLE.verify_package(output)
    print(json.dumps({'target': target, 'output': str(output), 'passed': result['passed'],
                      'files': result['files'], 'models': len(result['glbs']), 'pngs': result['png_files'],
                      'runtime_integrated': False}, ensure_ascii=False))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path)
    parser.add_argument('--target', choices=['all', *TARGETS], default='all')
    parser.add_argument('--output', type=Path, default=Path(__file__).resolve().parents[1] / 'assets/prepared/naraka')
    parser.add_argument('--verify', type=Path)
    args = parser.parse_args()
    if args.verify:
        print(json.dumps(PORTABLE.verify_package(args.verify), ensure_ascii=False))
    elif args.source:
        for target in TARGETS if args.target == 'all' else [args.target]:
            prepare_target(args.source.resolve(), args.output, target)
    else:
        parser.error('--source or --verify required')


if __name__ == '__main__':
    main()
