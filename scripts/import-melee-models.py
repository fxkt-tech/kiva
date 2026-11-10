#!/usr/bin/env python3
"""Build deterministic KIVAWP01 melee assets from frozen portable sources.

uv run --offline --with numpy --with pillow python scripts/import-melee-models.py
uv run --offline --with numpy --with pillow python scripts/import-melee-models.py --verify

The Zhexing output is an explicitly authored Kiva static assembly. It does not
interpret the original packed/Float deformation textures or execute star rules.
"""
import argparse
import importlib.util
import io
import json
from pathlib import Path
import struct
import tempfile

import numpy as np
from PIL import Image

SPEC = importlib.util.spec_from_file_location('prepared_runtime_helpers', Path(__file__).with_name('import-prepared-models.py'))
P = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(P)
N, require, sha, points, normals = P.N, P.require, P.sha, P.points, P.normals
REPO = Path(__file__).resolve().parent.parent
PACKAGE_HASHES = {
    'fulangmao': '71bf61d35756e4b655e156d952756640c8e8476bc42315c1fdb4c128d1a57b80',
    'zhexingcuishan': '81891e9d41c1fb9333c510c398c022e0389fae1c73c7c480385c9447545fdd7e',
}
HELPER_HASHES = {
    'import-prepared-models.py': 'c74db0c4fea9fa0fdd90c13eaef2ba91dcb6ef9fdb1f4a598164dcc51d65c007',
    'import-naraka.py': '6466eb88a4f577686d412c7f8b56f96f4d56e85250865b810a5b49de091757d9',
    'import-bailingmiao.py': '4eb46962ca602db4125aec5f3986e0abd6bfbf712ec2f7c18a9f213c66630dea',
}
SOURCE_TO_MELEE = np.array([[1., 0., 0.], [0., 0., 1.], [0., -1., 0.]])
PIECE_TO_SOURCE = SOURCE_TO_MELEE.T
IDENTITY = {'fulangmao': '伏浪锚2310478', 'zhexingcuishan': '谪星·摧山2311638'}


def check_helpers():
    for name, digest in HELPER_HASHES.items():
        require(sha(Path(__file__).with_name(name).read_bytes()) == digest,
                'review imported helper changes before rebuilding melee: ' + name)


def check_sources(source, names):
    packages = {}
    for name in names:
        directory = source / name
        raw = (directory / 'portable_manifest.json').read_bytes()
        require(sha(raw) == PACKAGE_HASHES[name], 'frozen portable manifest changed: ' + name)
        manifest = json.loads(raw)
        verification = P.PORTABLE.verify_package(directory)
        models = {row['role']: N.Glb(directory / row['file'], row['sha256']) for row in manifest['models']}
        packages[name] = {'manifest': manifest, 'models': models, 'verification': verification,
                          'directory': directory}
    return packages


def check_sources_unchanged(source, packages):
    for name, package in packages.items():
        directory = source / name
        require(sha((directory / 'portable_manifest.json').read_bytes()) == PACKAGE_HASHES[name],
                'source manifest changed during conversion')
        for row in package['manifest']['files']:
            raw = P.PORTABLE.checked_path(directory, row['file']).read_bytes()
            require(len(raw) == row['bytes'] and sha(raw) == row['sha256'],
                    'frozen source changed during conversion: ' + row['file'])


def authored_piece(scale, origin):
    result = np.eye(4)
    result[:3, :3] = PIECE_TO_SOURCE @ np.diag(scale)
    result[:3, 3] = origin
    require(np.linalg.det(result[:3, :3]) > 0, 'authored piece transform reverses winding')
    return result


def assembly(package, name):
    if name == 'fulangmao':
        return {'main': np.eye(4)}, {
            'selection': 'Both original configured LOD0 skinned instances, with their actual current source pose.',
            'authored_piece_assembly': False,
        }
    directory = package['directory']
    rule = json.loads((directory / 'source-evidence/additional/rule_source.json').read_bytes())
    shape_rules = rule['ruleGroups'][6]['ruleDatas']
    head_rule = shape_rules[1]['subRules'][0]
    require(head_rule['subRuleStartNumber'] == 0 and head_rule['subRuleEndNumber'] == 99,
            'source head-anchor rule changed')
    head = [head_rule['vectorStartValue'][key] for key in ('x', 'y', 'z')]
    require(head == [-0.007568359375, 0.0243988037109375, 1.656005859375], 'source head position changed')
    # These four inputs are complementary source material groups, not four
    # alternative complete blades. Their serialized positions are compact and
    # require an unrecovered shader. The transforms below are authored Kiva
    # composition, preserving every original vertex/triangle/UV and material.
    transforms = {
        'main': np.eye(4),
        'source_variant_input_01': authored_piece([1.5, 3.5, 2.4], [0., .12, 1.27]),
        'source_variant_input_02': authored_piece([1.5, 2.6, 2.], [0., -.03, 1.24]),
        'source_variant_input_03': authored_piece([1., 1., 1.], head),
        'source_variant_input_04': authored_piece([.8, .8, .8], [0., 0., .32]),
    }
    return transforms, {
        'selection': 'Kiva static assembly of source-proven complementary blade/back/head/handle groups plus original main shaft; both FX inputs excluded.',
        'authored_piece_assembly': True,
        'source_group_roles': {'source_variant_input_01': 'blade back / 刀背',
                               'source_variant_input_02': 'blade / 刀刃',
                               'source_variant_input_03': 'head / 人头',
                               'source_variant_input_04': 'handle / 刀柄', 'main': 'handle shaft / 刀柄'},
        'source_head_anchor': {'file': 'source-evidence/additional/rule_source.json',
                               'path': 'ruleGroups[6].ruleDatas[1].subRules[0].vectorStartValue',
                               'interval': [0, 99], 'xyz': head,
                               'usage': 'Position reference only; choosing this anchor is authored, not an executed original star result.'},
        'source_shape_index_reference': 8,
        'source_shape_index_executed': False,
        'Float_or_packed_shape_textures_sampled': False,
        'original_default_or_randomized_star_appearance_claimed': False,
        'piece_transform_note': 'Translations/rotations/scales are explicitly authored Kiva assembly; original source shader lookup and morph behavior are not reconstructed.',
        'excluded_roles': ['source_fx_input_01', 'source_fx_input_02'],
    }


def source_lbs(glb, mesh, primitive):
    """Independent per-bone accumulation, retaining every actual weighted slot."""
    actual = glb.posed(mesh, primitive)
    slots, bones = [], []
    if mesh['skin']:
        independent = np.zeros_like(actual)
        for slot, (joint, inverse) in enumerate(zip(mesh['skin']['joints'], mesh['binds'], strict=True)):
            weight = np.where(primitive['joints'] == slot, primitive['weights'], 0.).sum(1)
            vertices = int((weight > 0).sum())
            if vertices:
                slots.append(slot)
            current = glb.model(joint)
            independent += points(current @ inverse, primitive['POSITION']) * weight[:, None]
            bones.append({'slot': slot, 'name': glb.doc['nodes'][joint]['name'],
                          'weighted_vertices': vertices, 'source_current_model': current.tolist(),
                          'original_inverse_bind': inverse.tolist(),
                          'current_model_times_IBM_identity_residual': float(abs(current @ inverse - np.eye(4)).max())})
    else:
        independent = points(glb.model(mesh['node']), primitive['POSITION'])
    error = float(abs(independent - actual).max())
    require(error < 1e-6, 'independent actual source LBS mismatch')
    return actual, independent, slots, bones, error


def assembly_contacts(parts):
    """Real f32 vertex pairs bound surface gaps without claiming mesh welding."""
    positions = [np.concatenate([primitive['positions'] for primitive in part['primitives']]).astype('<f4').astype(np.float64)
                 for part in parts]
    tolerance, contacts, adjacency = .002, [], [set() for _ in parts]
    for left in range(len(parts)):
        for right in range(left + 1, len(parts)):
            a, b = positions[left], positions[right]
            minimum, pair = float('inf'), None
            for offset in range(0, len(a), 64):
                distances = ((a[offset:offset + 64, None, :] - b[None, :, :]) ** 2).sum(2)
                i, j = np.unravel_index(distances.argmin(), distances.shape)
                if distances[i, j] < minimum:
                    minimum, pair = float(distances[i, j]), (offset + int(i), int(j))
            gap = float(np.sqrt(minimum))
            connected = gap <= tolerance
            if connected:
                adjacency[left].add(right)
                adjacency[right].add(left)
            contacts.append({'parts': [parts[left]['name'], parts[right]['name']],
                             'nearest_actual_f32_vertices': [a[pair[0]].tolist(), b[pair[1]].tolist()],
                             'surface_gap_upper_bound_m': gap, 'within_contact_tolerance': connected})
    reached, pending = set(), [0]
    while pending:
        current = pending.pop()
        if current not in reached:
            reached.add(current)
            pending.extend(adjacency[current] - reached)
    require(len(reached) == len(parts), 'authored assembly contains a detached source region')
    return {'actual_binary_f32_positions': True, 'contact_tolerance_m': tolerance,
            'all_parts_connected_within_tolerance': True, 'pair_checks': contacts,
            'interpretation': 'Minimum original triangle-vertex distance is an upper bound on inter-surface gap. Contact graph is connected at 2mm tolerance; this does not claim welded topology or exact triangle intersections.'}


def texture_pixel_checks(materials, package, roles):
    originals = {}
    for role, glb in package['models'].items():
        if role not in roles:
            continue
        for material in glb.doc['materials']:
            texture = material.get('pbrMetallicRoughness', {}).get('baseColorTexture')
            if texture is not None:
                raw = glb.image(texture['index'])
                originals[sha(raw)] = raw
    results = []
    for evidence in materials.evidence:
        if 'source_png_sha256' not in evidence:
            continue
        source = Image.open(io.BytesIO(originals[evidence['source_png_sha256']])).convert('RGBA')
        # The pinned melee base images are already <=1024. Unlike a mere
        # encoded-PNG hash, this proves every actual RGB/alpha pixel survived.
        require(max(source.size) <= 1024, 'review any new source base-texture resizing explicitly')
        _, srgb, raw = next(row for row in materials.textures if row[0] == evidence['name'])
        actual = Image.open(io.BytesIO(raw)).convert('RGBA')
        expected_pixels, actual_pixels = np.asarray(source), np.asarray(actual)
        require(srgb and source.size == actual.size and np.array_equal(expected_pixels, actual_pixels),
                'source base RGB/alpha pixels changed')
        results.append({'name': evidence['name'], 'source_RGBA_pixels_sha256': sha(expected_pixels.tobytes()),
                        'runtime_RGBA_pixels_sha256': sha(actual_pixels.tobytes()),
                        'all_RGB_and_alpha_pixels_identical': True, 'resized': False, 'size': source.size})
    require(len(results) == len(materials.textures), 'base-texture pixel inventory incomplete')
    return results


def build_weapon(name, package):
    materials, parts, checks = P.Materials(), [], []
    transforms, selection = assembly(package, name)
    # Both weapons use their actual source shaft centre (0,0,0) as the Kiva
    # grip. Normalization changes only display size, never simulation reach.
    geometry = np.concatenate([points(transforms[role], glb.posed(mesh, primitive))
                               for role, glb in package['models'].items() if role in transforms
                               for mesh in glb.meshes() for primitive in mesh['primitives']])
    source_bounds = [geometry.min(0).tolist(), geometry.max(0).tolist()]
    scale = 1.10 / np.ptp(geometry[:, 2])
    conversion = P.matrix(SOURCE_TO_MELEE, scale)
    adaptations = []
    for role, glb in package['models'].items():
        if role not in transforms:
            continue
        piece = transforms[role]
        combined = conversion @ piece
        adaptations.append({'role': role, 'piece_to_source_space': piece.tolist(),
                            'source_to_runtime': conversion.tolist(), 'combined': combined.tolist(),
                            'mount': 'Grip0', 'authored': 'Kiva size/grip/axis adaptation and explicitly documented static composition.'})
        for mesh in glb.meshes():
            primitives = []
            for primitive in mesh['primitives']:
                posed, independent, slots, bones, error = source_lbs(glb, mesh, primitive)
                positions = points(combined, posed)
                posed_normals = normals(combined, N.posed_normals(glb, mesh, primitive))
                bake_error = float(np.linalg.norm(points(combined, independent) - positions.astype(np.float32), axis=1).max())
                require(bake_error < 1e-5, 'current-LBS rigid bake exceeds f32 tolerance')
                checks.append({'role': role, 'mesh': mesh['name'], 'vertices': len(positions),
                               'triangles': len(primitive['indices']) // 3, 'native_source_skin': bool(mesh['skin']),
                               'weighted_source_slots': slots, 'original_bones': bones,
                               'source_four_influence_weights_preserved_before_bake': bool(mesh['skin']),
                               'maximum_independent_current_LBS_error_m': error,
                               'maximum_independent_LBS_and_f32_bake_error_m': bake_error,
                               'source_pose_bounds': [posed.min(0).tolist(), posed.max(0).tolist()],
                               'runtime_bounds': [positions.min(0).tolist(), positions.max(0).tolist()],
                               'nondegenerate_UV_triangles': P.uv_check(primitive),
                               'original_indices_sha256': sha(primitive['indices'].astype('<u4').tobytes()),
                               'original_UV_f32_sha256': sha(primitive['TEXCOORD_0'].astype('<f4').tobytes())})
                if name == 'fulangmao' and len(positions) == 448:
                    original_indices = np.frombuffer((package['directory'] / 'source-evidence/additional/main_3583192603019751291.single-indices.u32.bin').read_bytes(), dtype='<u4')
                    require(np.array_equal(original_indices, primitive['joints'][:, 0])
                            and np.array_equal(primitive['weights'], np.tile([1., 0., 0., 0.], (448, 1))),
                            'original 448 single-index implicit-weight vertices changed')
                    checks[-1]['original_single_index_implicit_weight_vertices_verified'] = 448
                    checks[-1]['source_four_influence_weights_preserved_before_bake'] = False
                primitives.append({'material': materials.add(name + '::' + role, glb, mesh, primitive),
                                   'positions': positions, 'normals': posed_normals,
                                   'uv': primitive['TEXCOORD_0'], 'indices': primitive['indices']})
            parts.append({'name': f'naraka::{name}::{role}::{mesh["name"]}', 'mount': 0, 'primitives': primitives})
    if name == 'fulangmao':
        require(len(parts) == 2 and checks[0]['weighted_source_slots'] == list(range(5))
                and checks[1].get('original_single_index_implicit_weight_vertices_verified') == 448,
                'Fulangmao original two-skin inventory incomplete')
    else:
        require(len(parts) == 5 and sum(row['native_source_skin'] for row in checks) == 1
                and checks[0]['weighted_source_slots'] == [0, 1], 'Zhexing complementary composition or original two-bone main incomplete')
    header = {'format': 'KIVAWP01', 'identity': IDENTITY[name], 'mount0': 'Grip',
              'main_axis': '+Y', 'right_hand_grip': [0., 0., 0.], 'left_hand_grip': [0., 0., 0.],
              'unused_second_grip': True, 'source_grip': [0., 0., 0.],
              'source_grip_note': 'Authored Kiva shaft-centre grip, not a recovered original Actor attachment.',
              'runtime_length_m': 1.10, 'uniform_source_to_runtime_scale': scale,
              'assembly_source_bounds': source_bounds, 'max_texture_size': 1024,
              'selection': selection, 'adaptations': adaptations,
              'limits': 'Original current-LBS/static geometry with source base RGB/alpha; neutral prepared PBR. No source attack clips, packed/normal/Float shader, randomized stars, dissolve, spirit-light or reflection FX reconstruction.'}
    writer = N.Writer()
    writer.data = bytearray(b'KIVAWP01')
    P.write_header(writer, header)
    materials.write(writer)
    writer.write('6f', *([0.] * 6))
    writer.write('H', len(parts))
    for part in parts:
        writer.string(part['name'])
        writer.write('B', part['mount'])
        P.write_primitives(writer, part['primitives'], False)
    raw = bytes(writer.data)
    binary = verify_binary(raw, parts)
    return raw, {'primitives': checks, 'materials_and_textures': materials.evidence,
                 'source_base_texture_pixel_checks': texture_pixel_checks(materials, package, transforms),
                 'assembly_contact_checks': assembly_contacts(parts),
                 'adaptations': adaptations, 'selection': selection, 'binary': binary}, header


def verify_binary(raw, expected=None, offsets=None):
    require(0 < len(raw) <= 64 * 1024 * 1024, 'runtime size outside bound')
    reader = N.Reader(raw)
    require(reader.bytes(8) == b'KIVAWP01', 'runtime magic mismatch')
    length = reader.read('I')[0]
    require(length < 128 * 1024, 'runtime header exceeds bound')
    header = json.loads(reader.bytes(length))
    require(header['format'] == 'KIVAWP01' and header['mount0'] == 'Grip' and header['main_axis'] == '+Y',
            'wrong melee runtime contract')
    textures, materials, pngs = P.read_materials(reader)
    grips = reader.read('6f')
    require(grips == (0.,) * 6, 'melee grip must remain origin')
    count = reader.read('H')[0]
    require(0 < count <= 64 and (expected is None or count == len(expected)), 'runtime part count')
    geometry, stats = [], []
    for index in range(count):
        part_name = reader.string()
        require(part_name.startswith('naraka::'), 'wrong part namespace')
        if offsets is not None:
            offsets.setdefault('mount', reader.at)
        require(reader.read('B')[0] == 0, 'melee parts must use Grip0')
        primitives = reader.read('H')[0]
        require(0 < primitives <= 64, 'runtime primitive count')
        if expected is not None:
            require(expected[index]['name'] == part_name and primitives == len(expected[index]['primitives']), 'binary lost source part')
        for primitive_index in range(primitives):
            mat, vc, ic = reader.read('HII')
            require(mat < materials and 0 < vc <= 250000 and 0 < ic <= 1500000 and ic % 3 == 0,
                    'runtime geometry/material counts')
            if offsets is not None:
                offsets.setdefault('vertex', reader.at)
                offsets.setdefault('vertex_count', vc)
            vertices = np.frombuffer(reader.bytes(vc * 32), dtype=P.VERTEX)
            require(all(np.isfinite(vertices[field]).all() for field in ('p', 'n', 'uv')), 'nonfinite runtime vertex')
            require(abs(np.linalg.norm(vertices['n'], axis=1) - 1).max() < 1e-4, 'runtime normal not unit')
            if offsets is not None:
                offsets.setdefault('index', reader.at)
            indices = np.frombuffer(reader.bytes(ic * 4), dtype='<u4')
            require(indices.max() < vc, 'runtime index out of range')
            if expected is not None:
                source = expected[index]['primitives'][primitive_index]
                require(mat == source['material'] and np.array_equal(indices, source['indices']), 'binary source indices/material changed')
                require(np.array_equal(vertices['uv'], source['uv'].astype('<f4')), 'binary original UV changed')
                require(np.array_equal(vertices['p'], source['positions'].astype('<f4'))
                        and np.array_equal(vertices['n'], source['normals'].astype('<f4')), 'binary source geometry changed')
            geometry.append(vertices['p'])
            stats.append({'name': part_name, 'mount': 0, 'vertices': vc, 'triangles': ic // 3})
    require(reader.at == len(raw), 'runtime trailing bytes')
    combined = np.concatenate(geometry)
    extent = np.ptp(combined, axis=0)
    require(extent.argmax() == 1 and abs(extent[1] - 1.1) < 1e-5 and np.max(abs(combined)) < 1.1,
            'melee axis/size bounds invalid')
    return {'format': 'KIVAWP01', 'sha256': sha(raw), 'bytes': len(raw), 'textures': textures,
            'materials': materials, 'parts': count, 'vertices': sum(row['vertices'] for row in stats),
            'triangles': sum(row['triangles'] for row in stats), 'part_statistics': stats,
            'actual_runtime_mount0_bounds': [combined.min(0).tolist(), combined.max(0).tolist()],
            'PNG_checks': pngs, 'right_left_grips': grips, 'exact_EOF': True}


def corrupt_checks(raw):
    offsets = {}
    verify_binary(raw, offsets=offsets)
    def replace(at, value):
        return raw[:at] + value + raw[at + len(value):]
    mutations = {'truncated': raw[:-1], 'trailing': raw + b'!', 'magic': b'INVALID!' + raw[8:],
                 'oversized_header': replace(8, struct.pack('<I', 0xffffffff)),
                 'nonfinite_position': replace(offsets['vertex'], struct.pack('<f', float('nan'))),
                 'invalid_index': replace(offsets['index'], struct.pack('<I', offsets['vertex_count'])),
                 'wrong_mount': replace(offsets['mount'], bytes([1]))}
    for reason, damaged in mutations.items():
        try:
            verify_binary(damaged)
        except (ValueError, struct.error, UnicodeError):
            continue
        raise ValueError('corrupt melee binary accepted: ' + reason)
    return list(mutations)


def build(source, output, names):
    check_helpers()
    packages = check_sources(source, names)
    outputs = {}
    for name, package in packages.items():
        raw, report, header = build_weapon(name, package)
        report.update({'source_verification': package['verification'],
                       'checks': ['pinned_portable_manifest_and_all_file_SHA_before_and_after',
                                  'native_source_current_LBS_all_weighted_slots',
                                  'static_source_hierarchy_before_authored_assembly', 'original_indices_UV_and_material_reference',
                                  'f32_bake_tolerance_and_unit_normals', 'bounded_sRGB_source_base_RGB_alpha',
                                  'source_base_RGBA_every_pixel_identical', 'actual_f32_assembly_contact_graph_connected_2mm',
                                  'Grip0_Y_axis_1.10m', 'bounded_binary_exact_EOF']
                                  + (['original_448_single_index_implicit_weight1'] if name == 'fulangmao' else []),
                       'browser_or_screenshot_test': False})
        provenance = {'package': 'assets/prepared/naraka/' + name, 'portable_manifest_sha256': PACKAGE_HASHES[name],
                      'files': package['manifest']['files'], 'identity': package['manifest']['identity'],
                      'generator': 'scripts/import-melee-models.py', 'generator_sha256': sha(Path(__file__).read_bytes()),
                      'imported_helpers_sha256': HELPER_HASHES, 'runtime': 'weapon.bin', 'runtime_sha256': sha(raw),
                      'adaptation': header, 'game_reads': False, 'runtime_external_paths_or_downloads': False}
        readme = ('# ' + IDENTITY[name] + '\n\n'
                  'Generated from frozen `assets/prepared/naraka/' + name + '` by `scripts/import-melee-models.py`. '
                  '`weapon.bin` is existing KIVAWP01; every part uses Grip0, +Y length, 1.10m total length. '
                  'The right grip and unused second grip are zero. Held and flying instances share this asset.\n\n'
                  + header['selection']['selection'] + '\n\n'
                  + ('The four non-FX inputs are complementary original controller groups (back, blade, head, handle), not four interchangeable whole blades. '
                     'Their per-piece transforms are explicitly authored Kiva static composition, documented in both JSON reports. '
                     'The head anchor references source rule interval0..99; neither that star rule nor its shape-index8 is executed. '
                     'This is not a recovered original default/randomized appearance. No Float/packed deformation texture is sampled; the clamped PNG previews are never used as geometry.\n\n' if name == 'zhexingcuishan' else
                     'Both original skinned instances are baked from their actual hierarchy/current-LBS. The five-slot native four-weight body and the 448 original UInt32 single-index/implicit-weight1 attachment are retained.\n\n')
                  + 'Original base RGB/alpha, UV, triangles, cull/cutoff and prepared neutral PBR factors are retained. '
                  'Normal/packed/flow/ID/Float maps are not guessed as colors or ORM. Original attack/morph, stars, dissolve, spirit-light and cubemap/FX behavior are not restored. '
                  'Grip, display size and static composition are Kiva adaptations; simulation reach, timing and physics do not change. '
                  'No browser/rendering acceptance is claimed.\n\n'
                  '`uv run --offline --with numpy --with pillow python scripts/import-melee-models.py --verify` '
                  'validates pinned source files and helpers, current-LBS, exact binary geometry, and rebuilds all four files byte for byte in a temporary directory.\n')
        outputs[name] = {'weapon.bin': raw, 'README.md': readme.encode(),
                         'source-manifest.json': P.json_bytes(provenance), 'conversion-report.json': P.json_bytes(report)}
        directory = output / name
        directory.mkdir(parents=True, exist_ok=True)
        for filename, contents in outputs[name].items():
            (directory / filename).write_bytes(contents)
    check_sources_unchanged(source, packages)
    check_helpers()
    return outputs


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path, default=REPO / 'assets/prepared/naraka')
    parser.add_argument('--output', type=Path, default=REPO / 'assets')
    parser.add_argument('--target', choices=tuple(PACKAGE_HASHES), help='rebuild one of the two melee assets')
    parser.add_argument('--verify', action='store_true', help='rebuild to temp and compare all four files byte for byte')
    args = parser.parse_args()
    source, output = args.source.resolve(), args.output.resolve()
    names = [args.target] if args.target else list(PACKAGE_HASHES)
    require(not output.is_relative_to(source) and all(not source.is_relative_to(output / name) for name in names),
            'runtime output must not overlap prepared source')
    if args.verify:
        with tempfile.TemporaryDirectory(prefix='kiva-melee-') as temporary:
            built = build(source, Path(temporary), names)
            for name, files in built.items():
                for filename, contents in files.items():
                    require((output / name / filename).read_bytes() == contents, 'nonreproducible/stale melee output: ' + name + '/' + filename)
                print(json.dumps({'verified': name, 'sha256': sha(files['weapon.bin']), 'byte_identical_files': len(files),
                                  'corrupt_inputs_rejected': corrupt_checks(files['weapon.bin'])}))
    else:
        for name, files in build(source, output, names).items():
            print(json.dumps({'generated': name + '/weapon.bin', 'bytes': len(files['weapon.bin']), 'sha256': sha(files['weapon.bin'])}))


if __name__ == '__main__':
    main()
