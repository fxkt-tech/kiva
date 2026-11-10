#!/usr/bin/env python3
"""Offline prepared Naraka -> bounded KIVAAV03/KIVAWP01 runtime assets.

uv run --offline --with numpy --with pillow python scripts/import-prepared-models.py
uv run --offline --with numpy --with pillow python scripts/import-prepared-models.py --verify

Only reads the frozen portable packages, never a game/research installation.
NumPy/Pillow are build-time dependencies; the client has no new GLTF/JSON dependency.
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

SPEC = importlib.util.spec_from_file_location('naraka_runtime_helpers', Path(__file__).with_name('import-naraka.py'))
N = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(N)
PORTABLE_SPEC = importlib.util.spec_from_file_location('naraka_portable_helpers', Path(__file__).with_name('import-bailingmiao.py'))
PORTABLE = importlib.util.module_from_spec(PORTABLE_SPEC)
PORTABLE_SPEC.loader.exec_module(PORTABLE)
require, sha, points, normals = N.require, N.sha, N.points, N.normals
PACKAGE_HASHES = {
    'bailingmiao': 'bdf24f30a81916a3864e2c62f6a29fbeb2975e4fb20c26e4426b46ca1319a7a6',
    'yinbingluo': '3ab476488aef89294c1f95a88e42afbc1e7ea6c03ce14d843a7a54f30582d3fc',
    'yizhiying': 'fbe98256865375192c3f97d4b2b7a0452e6352bf22dfa59ac3db25e493ed7136',
}
REPO = Path(__file__).resolve().parent.parent
VERTEX = np.dtype([('p', '<f4', 3), ('n', '<f4', 3), ('uv', '<f4', 2)])
SKIN_VERTEX = np.dtype(VERTEX.descr + [('j', 'u1', 4), ('w', 'u1', 4)])
ROTATE_WEAPON = np.diag([-1., 1., -1.])
ROTATE_BOW = np.array([[-1., 0., 0.], [0., 0., 1.], [0., 1., 0.]])


def json_bytes(value):
    return (json.dumps(value, ensure_ascii=False, indent=2, allow_nan=False) + '\n').encode()


def matrix(rotation, scale=1., source_origin=None, target_origin=None):
    result = np.eye(4)
    result[:3, :3] = rotation * scale
    if source_origin is not None:
        result[:3, 3] = -result[:3, :3] @ np.asarray(source_origin)
    if target_origin is not None:
        result[:3, 3] += target_origin
    require(np.linalg.det(result[:3, :3]) > 0, 'axis adaptation must preserve winding')
    return result


def checked_packages(source):
    """Pin manifests, then verify every portable file and real skin/PNG data."""
    packages = {}
    for name, digest in PACKAGE_HASHES.items():
        path = source / name
        raw = (path / 'portable_manifest.json').read_bytes()
        require(sha(raw) == digest, f'frozen portable manifest changed: {name}')
        manifest = json.loads(raw)
        verification = PORTABLE.verify_package(path)
        declared = manifest.get('components', manifest.get('models'))
        models = {Path(row['file']).stem: N.Glb(path / row['file'], row['sha256']) for row in declared}
        packages[name] = {'manifest': manifest, 'models': models, 'verification': verification}
    return packages


class Materials:
    def __init__(self):
        self.textures, self.materials = [], []
        self.texture_keys, self.material_keys = {}, {}
        self.evidence = []

    def texture(self, raw, label, adaptation='Source RGB/alpha preserved; resized only.'):
        key = sha(raw)
        if key not in self.texture_keys:
            image = Image.open(io.BytesIO(raw)).convert('RGBA')
            original_size = image.size
            image.thumbnail((1024, 1024), Image.Resampling.LANCZOS)
            encoded = io.BytesIO()
            image.save(encoded, format='PNG', optimize=True)
            data = encoded.getvalue()
            self.texture_keys[key] = len(self.textures)
            self.textures.append((label, True, data))
            self.evidence.append({'name': label, 'source_png_sha256': key,
                                  'source_size': original_size, 'runtime_size': image.size,
                                  'runtime_png_sha256': sha(data), 'color_space': 'sRGB',
                                  'adaptation': adaptation})
        return self.texture_keys[key]

    def add(self, role, glb, mesh, primitive, face_skin=None):
        # Female eye/face/lash tints are separate even if a future GLB shares a material.
        key = (role, primitive['material'], mesh['name'] if face_skin is not None else '')
        if key in self.material_keys:
            return self.material_keys[key]
        original = glb.doc['materials'][primitive['material']]
        pbr = original.get('pbrMetallicRoughness', {})
        base = -1
        if 'baseColorTexture' in pbr:
            base = self.texture(glb.image(pbr['baseColorTexture']['index']), f'naraka::{role}::{original["name"]}')
        tint = pbr.get('baseColorFactor', [1., 1., 1., 1.])[:3]
        require(pbr.get('baseColorFactor', [1., 1., 1., 1.])[3] == 1, 'unsupported source constant alpha')
        roughness, metallic, reflectance = pbr.get('roughnessFactor', .75), pbr.get('metallicFactor', 0.), .38
        alpha, cutoff = original.get('alphaMode', 'OPAQUE') == 'MASK', original.get('alphaCutoff', .5)
        require(original.get('alphaMode', 'OPAQUE') in ('OPAQUE', 'MASK'), 'unhandled source blend mode')
        double = bool(original.get('doubleSided', False))
        adaptation = 'Source baseColorFactor/base PNG, cutoff and cull retained; neutral PBR scalars from prepared GLB.'
        if face_skin is not None:
            require(base == -1, 'generic female face unexpectedly has recovered base texture')
            if mesh['name'].startswith('face_'):
                tint = face_skin.tolist()
                adaptation = 'Generic female source face; neutral tint sampled from actual female outfit hand UV0, not Li male skin.'
            elif mesh['name'].startswith('eyes_'):
                tint, roughness, reflectance = [.28, .30, .32], .3, .5
                adaptation = 'Neutral gray PBR tint on original female eye geometry; no authored atlas or claimed source cosmetics.'
            elif mesh['name'].startswith('eyelash_'):
                tint, roughness = [.012, .012, .012], .9
                adaptation = 'Neutral charcoal tint on original female eyelash geometry; source cosmetic shader is not reconstructed.'
            else:
                raise ValueError('unknown female face mesh')
        response = [roughness, metallic, reflectance, 0., 0., .18, 0., *tint, cutoff]
        require(len(response) == 11 and np.isfinite(response).all(), 'nonfinite material response')
        name = f'naraka::{role}::{original["name"]}'
        result = len(self.materials)
        self.material_keys[key] = result
        self.materials.append((name, base, -1, -1, alpha, response, double))
        self.evidence.append({'material': name, 'prepared_factor_linear': pbr.get('baseColorFactor', [1, 1, 1, 1]),
                              'runtime_factor_linear': list(tint), 'alpha_mask': alpha, 'cutoff': cutoff,
                              'double_sided': double, 'adaptation': adaptation})
        return result

    def write(self, writer):
        writer.write('H', len(self.textures))
        for name, srgb, raw in self.textures:
            writer.string(name)
            writer.write('BI', srgb, len(raw))
            writer.data.extend(raw)
        writer.write('H', len(self.materials))
        for name, base, normal, surface, alpha, response, double in self.materials:
            writer.string(name)
            writer.write('hhhB11fB', base, normal, surface, alpha, *response, double)


def source_record(name, package):
    return {'package': 'assets/prepared/naraka/' + name,
            'portable_manifest_sha256': PACKAGE_HASHES[name],
            'identity': {key: value for key, value in package['manifest']['identity'].items()
                         if not key.startswith('source_identity')},
            'verified_files': package['verification']['files'],
            'verified_pngs': package['verification']['png_files'],
            'files': package['manifest']['files']}


def uv_check(p):
    uv = p['TEXCOORD_0']
    triangles = p['indices'].reshape(-1, 3)
    a, b = uv[triangles[:, 1]] - uv[triangles[:, 0]], uv[triangles[:, 2]] - uv[triangles[:, 0]]
    count = int((abs(a[:, 0] * b[:, 1] - a[:, 1] * b[:, 0]) > 1e-10).sum())
    require(count > 0, 'source UV entirely degenerate')
    return count


def skin_pose(matrices, positions, joints, weights):
    homogeneous = np.column_stack((positions, np.ones(len(positions))))
    return (np.einsum('nvij,nj->nvi', matrices[joints], homogeneous) * weights[:, :, None]).sum(1)[:, :3]


def packed_vertices(p, skinned):
    data = np.empty(len(p['positions']), dtype=SKIN_VERTEX if skinned else VERTEX)
    for field, source in (('p', 'positions'), ('n', 'normals'), ('uv', 'uv')):
        data[field] = p[source]
    if skinned:
        data['j'], data['w'] = p['joints'], p['weights']
    return data.tobytes()


def write_header(writer, value):
    raw = json_bytes(value)
    writer.write('I', len(raw))
    writer.data.extend(raw)


def write_primitives(writer, primitives, skinned):
    writer.write('H', len(primitives))
    for p in primitives:
        writer.write('HII', p['material'], len(p['positions']), len(p['indices']))
        writer.data.extend(packed_vertices(p, skinned))
        writer.data.extend(p['indices'].astype('<u4').tobytes())


def driver_for(glb, node, role):
    # The vocabulary is shared; these are female matrices, not male indices.
    return N.driver_for(glb, node, 'face' if role == 'generic_female_face' else role)


def build_female(package):
    models = package['models']
    full = np.concatenate([g.posed(m, p) for g in models.values() for m in g.meshes() for p in m['primitives']])
    floor, top = float(full[:, 1].min()), float(full[:, 1].max())
    scale = 1.73 / (top - floor)
    conversion = matrix(ROTATE_WEAPON, scale)
    conversion[1, 3] = -floor * scale
    inverse_conversion = np.linalg.inv(conversion)
    female_reference = models['lianmu']
    anchors = []
    for _, source in N.DRIVERS:
        if source is None:
            anchors.append(np.zeros(3))
        else:
            matches = [i for i, node in enumerate(female_reference.doc['nodes']) if node['name'] == source]
            require(len(matches) == 1, 'missing/ambiguous female structural anchor: ' + source)
            anchors.append((conversion @ female_reference.model(matches[0]))[:3, 3])
    anchors = np.array(anchors)
    require(anchors[10, 0] < 0 < anchors[14, 0], 'female hand sides swapped')
    require(anchors[18, 2] < anchors[17, 2] and anchors[22, 2] < anchors[21, 2], 'female feet not forward-Z')
    face_skin, skin_evidence = N.skin_color(models['outfit'])
    skin_evidence['method'] = 'Median original female_Body embedded base PNG at actual female hand/finger/forearm-weighted UV0, decoded sRGB to linear.'
    materials, parts, checks = Materials(), [], []
    for role, glb in models.items():
        for mesh in glb.meshes():
            require(mesh['skin'], 'female component lost native skin')
            rest = np.array([conversion @ glb.model(j) for j in mesh['skin']['joints']])
            binds = mesh['binds'] @ inverse_conversion
            bones = [(f'naraka::female::{role}::{glb.doc["nodes"][j]["name"]}', driver_for(glb, j, role), inverse, model)
                     for j, inverse, model in zip(mesh['skin']['joints'], binds, rest, strict=True)]
            require(0 < len(bones) <= 256, 'female palette outside u8 range')
            primitives = []
            for p in mesh['primitives']:
                positions = points(conversion, p['POSITION'])
                weights = N.quantize_weights(p['weights'])
                expected = points(conversion, glb.posed(mesh, p))
                actual = skin_pose(rest @ binds, positions, p['joints'], p['weights'])
                float_error = float(abs(actual - expected).max())
                serialized = rest.astype(np.float32).astype(float) @ binds.astype(np.float32).astype(float)
                encoded = skin_pose(serialized, positions.astype(np.float32), p['joints'], weights / 255.)
                byte_error = float(np.linalg.norm(encoded - expected, axis=1).max())
                require(float_error < 1e-6 and byte_error < .005, 'female source pose LBS conversion exceeds tolerance')
                # Rotate the actual most-weighted structural region around its own
                # female anchor. Compare original high precision and runtime bytes.
                contribution = np.bincount(p['joints'].ravel(), weights=p['weights'].ravel(), minlength=len(bones))
                driver = bones[int(contribution.argmax())][1]
                angle = .35
                rotate = np.array([[np.cos(angle), 0, np.sin(angle)], [0, 1, 0], [-np.sin(angle), 0, np.cos(angle)]])
                delta = matrix(rotate, source_origin=anchors[driver], target_origin=anchors[driver])
                driven = rest.copy()
                for slot, bone in enumerate(bones):
                    if bone[1] == driver:
                        driven[slot] = delta @ rest[slot]
                motion_float = skin_pose(driven @ binds, positions, p['joints'], p['weights'])
                motion_serialized = skin_pose(driven.astype(np.float32).astype(float) @ binds.astype(np.float32).astype(float),
                                              positions.astype(np.float32), p['joints'], weights / 255.)
                movement = float(np.linalg.norm(motion_float - actual, axis=1).max())
                motion_error = float(np.linalg.norm(motion_serialized - motion_float, axis=1).max())
                require(movement > 1e-5 and motion_error < .005, 'actual female driven deformation failed')
                checks.append({'role': role, 'mesh': mesh['name'], 'vertices': len(positions), 'triangles': len(p['indices']) // 3,
                               'palette_slots': len(bones), 'maximum_C_LBS_error_m': float_error,
                               'maximum_f32_weights255_error_m': byte_error,
                               'perturbed_female_driver': N.DRIVERS[driver][0], 'perturbation_radians': angle,
                               'maximum_driven_deformation_m': movement, 'maximum_driven_serialization_error_m': motion_error,
                               'nondegenerate_UV_triangles': uv_check(p)})
                primitives.append({'material': materials.add(role, glb, mesh, p, face_skin if role == 'generic_female_face' else None),
                                   'positions': positions, 'normals': normals(conversion, p['NORMAL']), 'uv': p['TEXCOORD_0'],
                                   'joints': p['joints'].astype(np.uint8), 'weights': weights, 'indices': p['indices']})
            parts.append({'name': f'naraka::female::{role}::{mesh["node"]}::{mesh["name"]}', 'bones': bones, 'primitives': primitives})
    header = {'format': 'KIVAAV03', 'identity': '白灵淼衣装3402139 + 莲母100408 + 通用女性基脸',
              'height_m': 1.73, 'max_texture_size': 1024, 'conversion': conversion.tolist(),
              'source_scale': scale, 'source_floor': floor, 'source_top': top,
              'matrix_contract': 'P=C*P; IBM=originalIBM*C^-1; sourceRest=C*originalBoneWorld; source current pose differs from bind.',
              'female_driver_reference': 'Actual Lianmu l22_female_skeletonAvatar current source structural anchors; palette current rests remain independent.',
              'face': 'Generic FEMALE source face. Female-body-derived skin; neutral gray eye tint. No Bai preset/cosmetic shader recovered.',
              'hair': 'Neutral light-grey alpha preview from frozen GLB, actual accessory base PNG; no ID RGB as albedo, no reconstructed dye/SH.',
              'limits': 'Neutral prepared PBR, no original cloth/hair physics, shader, specific facial preset or source fire/draw animation.'}
    writer = N.Writer()
    write_header(writer, header)
    writer.write('H', len(N.DRIVERS))
    for (name, _), anchor in zip(N.DRIVERS, anchors, strict=True):
        writer.string(name)
        writer.write('3f', *anchor)
    materials.write(writer)
    writer.write('H', len(parts))
    for part in parts:
        writer.string(part['name'])
        writer.write('BH', 0, len(part['bones']))
        for name, driver, inverse, rest in part['bones']:
            writer.string(name)
            writer.write('H16f16f', driver, *inverse.T.ravel(), *rest.T.ravel())
        write_primitives(writer, part['primitives'], True)
    report = {'normalization': {'source_bounds': [full.min(0).tolist(), full.max(0).tolist()],
                                'scale': scale, 'C': conversion.tolist(), 'female_anchors': anchors.tolist(),
                                'female_source_reference': 'models/lianmu.glb; no male indices/proportions'},
              'skin_tint': skin_evidence, 'primitives': checks, 'materials_and_textures': materials.evidence,
              'maximum_quantized_LBS_error_m': max(x['maximum_f32_weights255_error_m'] for x in checks),
              'maximum_driven_LBS_error_m': max(x['maximum_driven_serialization_error_m'] for x in checks)}
    return bytes(writer.data), report, header


def build_weapon(name, package):
    materials, parts, checks, adaptations = Materials(), [], [], []
    if name == 'yinbingluo':
        conversions = {'yinbingluo_main': matrix(ROTATE_WEAPON, source_origin=[0., .0582, 1.01312304])}
        right, left = [0., -.0582, .76312304], [0., -.0282, .41312304]
    else:
        conversions = {'yizhiying_main': matrix(ROTATE_BOW, .66, target_origin=[0., 0., .08])}
        right, left = [0., 0., .35], [0., 0., .08]
    for role, glb in package['models'].items():
        mount = 0 if role.endswith('_main') else 1
        if mount == 1:
            # These are independent original configured parts. There is no
            # recovered Actor attachment: center each and author a Kiva mount.
            geometry = np.concatenate([glb.posed(m, p) for m in glb.meshes() for p in m['primitives']])
            center = (geometry.min(0) + geometry.max(0)) * .5
            offset = [.18, -.13, .16] if role.endswith('_quiver') else [-.18, -.13, .18]
            conversions[role] = matrix(ROTATE_BOW, .55, center, offset)
        conversion = conversions[role]
        adaptations.append({'part': role, 'mount': 'muzzle/arrow source' if mount == 0 else 'spine_03 in Kiva body axes',
                            'conversion': conversion.tolist(),
                            'authored': 'Kiva axis/grip/muzzle or body mount adaptation; not a recovered original Actor attachment.'})
        for mesh in glb.meshes():
            primitives = []
            for p in mesh['primitives']:
                source_pose = glb.posed(mesh, p)
                positions = points(conversion, source_pose)
                # Independent per-slot accumulation checks against the source
                # GLB skinning reader, including all original gun joint1 vertices.
                weighted_slots, joint1_vertices = [], 0
                nonbind = 0
                if mesh['skin']:
                    independent = np.zeros_like(source_pose)
                    for slot, (joint, inverse) in enumerate(zip(mesh['skin']['joints'], mesh['binds'], strict=True)):
                        contribution = np.where(p['joints'] == slot, p['weights'], 0.).sum(1)
                        if contribution.max() > 0:
                            weighted_slots.append(slot)
                        nonbind += int(abs(glb.model(joint) @ inverse - np.eye(4)).max() > 1e-4)
                        independent += points(glb.model(joint) @ inverse, p['POSITION']) * contribution[:, None]
                    require(np.max(abs(independent - source_pose)) < 1e-6, 'independent actual source LBS bake mismatch')
                    joint1_vertices = int(((p['joints'] == 1) & (p['weights'] > 0)).any(1).sum())
                else:
                    independent = points(glb.model(mesh['node']), p['POSITION'])
                error = float(np.linalg.norm(points(conversion, independent) - positions.astype(np.float32), axis=1).max())
                require(error < 1e-5, 'rigid source current-LBS bake f32 error')
                checks.append({'role': role, 'mesh': mesh['name'], 'vertices': len(positions), 'triangles': len(p['indices']) // 3,
                               'native_source_skin': bool(mesh['skin']), 'weighted_source_slots': weighted_slots,
                               'joint1_vertices': joint1_vertices, 'nonidentity_sourceRest_IBM_matrices': nonbind,
                               'maximum_independent_LBS_and_f32_bake_error_m': error,
                               'source_pose_bounds': [source_pose.min(0).tolist(), source_pose.max(0).tolist()],
                               'runtime_bounds': [positions.min(0).tolist(), positions.max(0).tolist()],
                               'nondegenerate_UV_triangles': uv_check(p)})
                primitives.append({'material': materials.add(role, glb, mesh, p), 'positions': positions,
                                   'normals': normals(conversion, N.posed_normals(glb, mesh, p)),
                                   'uv': p['TEXCOORD_0'], 'indices': p['indices']})
            parts.append({'name': f'naraka::{role}::{mesh["name"]}', 'mount': mount, 'primitives': primitives})
    if name == 'yinbingluo':
        require(any(x['joint1_vertices'] == 135 for x in checks), 'the 135 original gun joint1 vertices were lost')
    else:
        require(any(x['weighted_source_slots'] == list(range(9)) for x in checks), 'bow native9 weighted joints lost')
        require(sum(x['mount'] == 1 for x in parts) == 2, 'configured bow quiver/arrowbag missing')
    header = {'format': 'KIVAWP01', 'identity': '引冰落2311228' if name == 'yinbingluo' else '羿之影2310557',
              'max_texture_size': 1024, 'forward': '-Z', 'muzzle_or_arrow_source': [0, 0, 0],
              'right_hand_grip': right, 'left_hand_grip': left, 'adaptations': adaptations,
              'limits': 'Original current source pose LBS baked rigid; no source fire/draw clips, special shader, emission or packed mapping.',
              'related_arrow': 'Independent related neutral geometry remains prepared only; no recovered Actor/material binding, not fourth configured part.' if name == 'yizhiying' else None}
    writer = N.Writer()
    writer.data = bytearray(b'KIVAWP01')
    write_header(writer, header)
    materials.write(writer)
    writer.write('6f', *right, *left)
    writer.write('H', len(parts))
    for part in parts:
        writer.string(part['name'])
        writer.write('B', part['mount'])
        write_primitives(writer, part['primitives'], False)
    return bytes(writer.data), {'primitives': checks, 'materials_and_textures': materials.evidence, 'adaptations': adaptations}, header


def read_materials(reader):
    textures = reader.read('H')[0]
    require(0 < textures <= 64, 'runtime texture count outside limit')
    texture_checks = []
    for _ in range(textures):
        name = reader.string()
        srgb, length = reader.read('BI')
        require(srgb == 1 and 0 < length < 8 * 1024 * 1024, 'invalid runtime PNG size/color space')
        raw = reader.bytes(length)
        with Image.open(io.BytesIO(raw)) as image:
            require(image.format == 'PNG' and max(image.size) <= 1024, 'PNG size/format contract')
            image.verify()
            texture_checks.append({'name': name, 'size': image.size, 'sRGB': True, 'sha256': sha(raw)})
    materials = reader.read('H')[0]
    require(0 < materials <= 128, 'runtime material count outside limit')
    for _ in range(materials):
        reader.string()
        base, normal, surface, alpha, *response = reader.read('hhhB11f')
        double = reader.read('B')[0]
        require(-1 <= base < textures and normal == surface == -1, 'unknown texture channels or invalid indices')
        require(alpha in (0, 1) and double in (0, 1) and np.isfinite(response).all(), 'material response invalid')
    return textures, materials, texture_checks


def verify_binary(raw, name, offsets=None):
    require(0 < len(raw) <= 64 * 1024 * 1024, 'runtime file size outside limit')
    reader = N.Reader(raw)
    skinned = name == 'bailingmiao'
    require(reader.bytes(8) == (b'KIVAAV03' if skinned else b'KIVAWP01'), 'runtime magic mismatch')
    header_length = reader.read('I')[0]
    require(header_length < 128 * 1024, 'runtime header exceeds bound')
    header = json.loads(reader.bytes(header_length))
    anchors = None
    if skinned:
        require(reader.read('H')[0] == 23, 'female driver count')
        anchors = []
        for expected, _ in N.DRIVERS:
            require(reader.string() == expected, 'female structural driver order')
            anchors.append(reader.read('3f'))
        require(np.isfinite(anchors).all(), 'nonfinite female anchors')
    texture_count, material_count, texture_checks = read_materials(reader)
    grips = None if skinned else reader.read('6f')
    require(skinned or np.isfinite(grips).all(), 'nonfinite ranged grips')
    part_count = reader.read('H')[0]
    require(0 < part_count <= 64, 'runtime part count outside bound')
    points_by_mount, stats, nonbind = {}, [], 0
    for _ in range(part_count):
        part = reader.string()
        require(part.startswith('naraka::'), 'runtime part namespace')
        if skinned:
            blade, bones = reader.read('BH')
            require(blade == 0 and 0 < bones <= 256, 'female cannot contain blade or empty palette')
            transforms = []
            for _ in range(bones):
                require(reader.string().startswith('naraka::female::'), 'female palette namespace')
                require(reader.read('H')[0] < 23, 'female palette driver index')
                inverse = np.array(reader.read('16f')).reshape((4, 4), order='F')
                if offsets is not None:
                    offsets.setdefault('current_rest', reader.at)
                rest = np.array(reader.read('16f')).reshape((4, 4), order='F')
                require(np.isfinite([inverse, rest]).all() and min(abs(np.linalg.det(inverse)), abs(np.linalg.det(rest))) > 1e-4,
                        'invalid female IBM/current matrix')
                transforms.append(rest @ inverse)
                nonbind += int(abs(rest @ inverse - np.eye(4)).max() > 1e-3)
            mount = 0
        else:
            mount = reader.read('B')[0]
            require(mount in (0, 1), 'unknown weapon attachment')
        primitive_count = reader.read('H')[0]
        require(0 < primitive_count <= 64, 'runtime primitive count')
        for _ in range(primitive_count):
            mat, vc, ic = reader.read('HII')
            require(mat < material_count and 0 < vc <= 250000 and 0 < ic <= 1500000 and ic % 3 == 0,
                    'runtime geometry counts/material')
            if offsets is not None:
                offsets.setdefault('vertex', reader.at)
                offsets.setdefault('vertex_count', vc)
            data = np.frombuffer(reader.bytes(vc * (40 if skinned else 32)), dtype=SKIN_VERTEX if skinned else VERTEX)
            require(np.isfinite(data['p']).all() and np.isfinite(data['n']).all() and np.isfinite(data['uv']).all(), 'nonfinite runtime vertex')
            require(abs(np.linalg.norm(data['n'], axis=1) - 1).max() < 1e-4, 'runtime normal not unit')
            if offsets is not None:
                offsets.setdefault('index', reader.at)
            indices = np.frombuffer(reader.bytes(ic * 4), dtype='<u4')
            require(indices.max() < vc, 'runtime index out of range')
            if skinned:
                require((data['j'] < bones).all() and (data['w'].astype(int).sum(1) == 255).all(), 'runtime four-weight/palette invalid')
                posed = skin_pose(np.array(transforms), data['p'], data['j'], data['w'] / 255.)
            else:
                posed = data['p']
            require(np.isfinite(posed).all(), 'nonfinite runtime LBS')
            points_by_mount.setdefault(mount, []).append(posed)
            stats.append({'name': part, 'mount': mount, 'vertices': vc, 'triangles': ic // 3})
    require(reader.at == len(raw), 'runtime binary trailing bytes')
    geometry = np.concatenate(points_by_mount[0])
    if skinned:
        require(nonbind > 0 and abs(np.ptp(geometry[:, 1]) - 1.73) < .002 and abs(geometry[:, 1].min()) < .002,
                'female source pose replaced or actual normalized bounds invalid')
    elif name == 'yinbingluo':
        require(np.ptp(geometry, axis=0).argmax() == 2 and abs(geometry[:, 2].min()) < .001,
                'gun does not point down-Z from actual muzzle0')
    else:
        require(np.ptp(geometry, axis=0).argmax() == 1 and 1.5 < np.ptp(geometry[:, 1]) < 1.6,
                'bow vertical span invalid')
    return {'format': header['format'], 'sha256': sha(raw), 'bytes': len(raw), 'textures': texture_count,
            'materials': material_count, 'parts': part_count, 'vertices': sum(x['vertices'] for x in stats),
            'triangles': sum(x['triangles'] for x in stats), 'part_statistics': stats,
            'actual_runtime_mount0_bounds': [geometry.min(0).tolist(), geometry.max(0).tolist()],
            'source_pose_nonbind_bones': nonbind, 'PNG_checks': texture_checks,
            'right_left_grips': grips, 'exact_EOF': True}


def build(source, output):
    packages = checked_packages(source)
    outputs = {}
    for name, package in packages.items():
        raw, report, header = build_female(package) if name == 'bailingmiao' else build_weapon(name, package)
        filename = 'character.bin' if name == 'bailingmiao' else 'weapon.bin'
        binary = verify_binary(raw, name)
        report.update({'binary': binary, 'source_verification': package['verification'],
                       'checks': ['pinned_portable_manifest_and_every_file_SHA', 'actual_source_accessor_weights_indices',
                                  'original_source_hierarchy_and_current_LBS', 'source_f32_or_weights255_deviation',
                                  'UV_and_unit_normals', 'bounded_PNG_sRGB_base_only', 'finite_matrices_and_bounds', 'exact_binary_EOF'],
                       'browser_or_screenshot_test': False})
        provenance = source_record(name, package)
        provenance.update({'generator': 'scripts/import-prepared-models.py',
                           'generator_sha256': sha(Path(__file__).read_bytes()), 'runtime': filename,
                           'runtime_sha256': sha(raw), 'adaptation': header,
                           'runtime_external_paths_or_downloads': False, 'game_reads': False})
        readme = ('# ' + header['identity'] + '\n\nGenerated offline by `scripts/import-prepared-models.py` from frozen '
                  '`assets/prepared/naraka/' + name + '`. Runtime file: `' + filename + '` (' + header['format'] + ').\n\n'
                  + header.get('limits', '') + '\n\nOnly source base-color PNGs / alpha and explicit neutral PBR adaptations are used; '
                  'normal/packed/SH/emission maps are not reinterpreted. Source palette and geometry provenance, '
                  'grips/body mounts, texture resizing and numerical verification are in the two JSON reports. '
                  'No browser, rendering or interaction acceptance is claimed.\n\n'
                  '`uv run --offline --with numpy --with pillow python scripts/import-prepared-models.py --verify` '
                  'revalidates every frozen source file and rebuilds all outputs to compare byte for byte.\n')
        outputs[name] = {filename: raw, 'source-manifest.json': json_bytes(provenance),
                         'conversion-report.json': json_bytes(report), 'README.md': readme.encode()}
        directory = output / name
        directory.mkdir(parents=True, exist_ok=True)
        for file, contents in outputs[name].items():
            (directory / file).write_bytes(contents)
    # Verify after all writes: generated output must never overwrite a frozen input.
    for name, digest in PACKAGE_HASHES.items():
        path = source / name
        require(sha((path / 'portable_manifest.json').read_bytes()) == digest, 'source manifest changed during conversion')
        for record in packages[name]['manifest']['files']:
            require(sha(PORTABLE.checked_path(path, record['file']).read_bytes()) == record['sha256'], 'frozen source mutated during conversion')
    return outputs


def corrupt_binary_checks(raw, name):
    offsets = {}
    verify_binary(raw, name, offsets)
    def replace(at, data):
        return raw[:at] + data + raw[at + len(data):]
    mutations = {'truncated': raw[:-1], 'trailing': raw + b'!', 'bad_magic': b'INVALID!' + raw[8:],
                 'oversized_header': replace(8, struct.pack('<I', 0xffffffff)),
                 'nonfinite_position': replace(offsets['vertex'], struct.pack('<f', float('nan'))),
                 'index_outside_primitive': replace(offsets['index'], struct.pack('<I', offsets['vertex_count']))}
    if name == 'bailingmiao':
        mutations['nonfinite_current_rest'] = replace(offsets['current_rest'], struct.pack('<f', float('nan')))
        mutations['joint_outside_palette'] = replace(offsets['vertex'] + 32, bytes([255]))
        weight_offset = offsets['vertex'] + 36
        old_weight = raw[weight_offset]
        mutations['weight_sum_not255'] = replace(weight_offset, bytes([old_weight - 1 if old_weight else 1]))
    for reason, damaged in mutations.items():
        try:
            verify_binary(damaged, name)
        except (ValueError, struct.error, UnicodeError):
            continue
        raise ValueError('corrupt runtime binary accepted: ' + reason)
    return list(mutations)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path, default=REPO / 'assets/prepared/naraka')
    parser.add_argument('--output', type=Path, default=REPO / 'assets')
    parser.add_argument('--verify', action='store_true', help='rebuild to temp; compare all checked-in runtime files byte for byte')
    args = parser.parse_args()
    source, output = args.source.resolve(), args.output.resolve()
    require(not output.is_relative_to(source) and not source.is_relative_to(output / 'bailingmiao'), 'runtime output must not overwrite prepared source')
    if args.verify:
        with tempfile.TemporaryDirectory(prefix='kiva-prepared-models-') as temporary:
            built = build(source, Path(temporary))
            for name, files in built.items():
                for filename, contents in files.items():
                    require((output / name / filename).read_bytes() == contents, f'nonreproducible/stale output: {name}/{filename}')
                binary = files['character.bin' if name == 'bailingmiao' else 'weapon.bin']
                rejected = corrupt_binary_checks(binary, name)
                print(json.dumps({'verified': name, 'sha256': sha(binary), 'byte_identical_files': len(files), 'corrupt_inputs_rejected': rejected}))
    else:
        built = build(source, output)
        for name, files in built.items():
            filename = 'character.bin' if name == 'bailingmiao' else 'weapon.bin'
            print(json.dumps({'generated': name + '/' + filename, 'bytes': len(files[filename]), 'sha256': sha(files[filename])}))


if __name__ == '__main__':
    main()
