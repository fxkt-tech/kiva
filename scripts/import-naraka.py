#!/usr/bin/env python3
"""Build trusted offline KIVAAV03 Li Huowang appearance assets.

  /path/to/naraka/.venv/bin/python scripts/import-naraka.py --source /path/to/naraka
  /path/to/naraka/.venv/bin/python scripts/import-naraka.py --verify assets/lihuowang/character.bin

KIVAAV03 retains AV02's little-endian order, adding one material double-sided
byte after hhhB11f, and one source-rest-model MAT4 after each palette IBM.
This importer reads pinned, previously exported GLBs, never a game installation.
"""
import argparse
import hashlib
import io
import json
from pathlib import Path
import struct

import numpy as np
from PIL import Image

DRIVERS = [
    ('root', None), ('pelvis', 'gMan Pelvis'),
    ('spine_01', 'gMan Spine'), ('spine_02', 'gMan Spine1'),
    ('spine_03', 'gMan Spine2'), ('neck_01', 'gMan Neck'),
    ('Head', 'gMan Head'), ('clavicle_l', 'gMan L Clavicle'),
    ('upperarm_l', 'gMan L UpperArm'), ('lowerarm_l', 'gMan L Forearm'),
    ('hand_l', 'gMan L Hand'), ('clavicle_r', 'gMan R Clavicle'),
    ('upperarm_r', 'gMan R UpperArm'), ('lowerarm_r', 'gMan R Forearm'),
    ('hand_r', 'gMan R Hand'), ('thigh_l', 'gMan L Thigh'),
    ('calf_l', 'gMan L Calf'), ('foot_l', 'gMan L Foot'),
    ('ball_l', 'gMan L Toe0'), ('thigh_r', 'gMan R Thigh'),
    ('calf_r', 'gMan R Calf'), ('foot_r', 'gMan R Foot'),
    ('ball_r', 'gMan R Toe0'),
]
SOURCES = {
    'outfit': ('character/lihuowang_s20_outfit.glb', '931bd444e2d8e64f704d5ef67715d60317c6fc88f00ce23d5647fa89cad5e0dc'),
    'hair': ('character/xinsu_reconstructed_binding.glb', '74fea423645617a4cdf498948118a18c747f3cb3c0f870158291d5e51021e98f'),
    'face': ('character/generic_base_face_neutral.glb', '82b5677e844a551c3219e307b86fd6701a8e69a72ae9f6c384f4d45b25dccbee'),
    'blade': ('weapon/zhulei_basecolor.glb', '8638c119db28f47f434c3b1924972fe61b542b9977f8926bc5b4ec37e3ea5d40'),
    'sheath': ('weapon/zhulei_sheath.glb', '192ea395e59b44dab38ad59cb590fb202887a4dc2797fc363f166035dca144c1'),
    'ornament': ('character/guidao_source_basecolor.glb', '9e50c35b8e9da98576948fea87dd7cab3634efe107675ae3879080af59883e4f'),
}


def require(test, message):
    if not test:
        raise ValueError(message)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def points(matrix, values):
    return (matrix @ np.column_stack((values, np.ones(len(values)))).T).T[:, :3]


def normals(matrix, values):
    result = (np.linalg.inv(matrix[:3, :3]).T @ np.asarray(values).T).T
    lengths = np.linalg.norm(result, axis=1)
    require((lengths > 1e-8).all(), 'degenerate normal')
    return result / lengths[:, None]


class Glb:
    def __init__(self, path, expected_hash):
        self.path = path
        raw = path.read_bytes()
        require(sha(raw) == expected_hash, f'unrecognized source GLB hash: {path}')
        magic, version, size = struct.unpack_from('<4sII', raw)
        require((magic, version, size) == (b'glTF', 2, len(raw)), 'invalid GLB header')
        length, tag = struct.unpack_from('<I4s', raw, 12)
        require(tag == b'JSON', 'missing GLB JSON')
        self.doc = json.loads(raw[20:20 + length])
        at = 20 + length
        count, tag = struct.unpack_from('<I4s', raw, at)
        require(tag == b'BIN\0' and at + 8 + count == len(raw), 'invalid GLB BIN/EOF')
        self.binary = raw[at + 8:]
        self.parents = {}
        for parent, node in enumerate(self.doc['nodes']):
            for child in node.get('children', []):
                require(child not in self.parents, 'multiple node parents')
                self.parents[child] = parent
        self.world = {}
        self._pending = set()
        for i in range(len(self.doc['nodes'])):
            self.model(i)

    def model(self, i):
        if i not in self.world:
            require(i not in self._pending, 'cyclic original source hierarchy')
            self._pending.add(i)
            node = self.doc['nodes'][i]
            require('matrix' in node, 'pinned source expected explicit node matrix')
            local = np.array(node['matrix'], dtype=np.float64).reshape((4, 4), order='F')
            self.world[i] = self.model(self.parents[i]) @ local if i in self.parents else local
            require(np.isfinite(self.world[i]).all(), 'nonfinite original node matrix')
            self._pending.remove(i)
        return self.world[i]

    def array(self, i):
        acc = self.doc['accessors'][i]
        require(not acc.get('normalized') and 'sparse' not in acc, 'unexpected pinned accessor layout')
        view = self.doc['bufferViews'][acc['bufferView']]
        require('byteStride' not in view, 'unexpected interleaved pinned accessor')
        widths = {'SCALAR': 1, 'VEC2': 2, 'VEC3': 3, 'VEC4': 4, 'MAT4': 16}
        types = {5126: '<f4', 5123: '<u2', 5125: '<u4'}
        start = view.get('byteOffset', 0) + acc.get('byteOffset', 0)
        width = widths[acc['type']]
        array = np.frombuffer(self.binary, dtype=types[acc['componentType']], count=acc['count'] * width, offset=start).reshape((acc['count'], width)).copy()
        require(np.isfinite(array).all(), 'nonfinite original accessor')
        return array

    def image(self, texture):
        image = self.doc['images'][self.doc['textures'][texture]['source']]
        view = self.doc['bufferViews'][image['bufferView']]
        start = view.get('byteOffset', 0)
        raw = self.binary[start:start + view['byteLength']]
        require(image['mimeType'] == 'image/png', 'pinned source texture is not PNG')
        return raw

    def meshes(self):
        for i, node in enumerate(self.doc['nodes']):
            if 'mesh' not in node:
                continue
            mesh = self.doc['meshes'][node['mesh']]
            skin = self.doc.get('skins', [])[node['skin']] if 'skin' in node else None
            binds = self.array(skin['inverseBindMatrices']).reshape((-1, 4, 4)).transpose((0, 2, 1)) if skin else None
            primitives = []
            for primitive in mesh['primitives']:
                require(primitive.get('mode', 4) == 4, 'source primitive must be triangles')
                attrs = primitive['attributes']
                p = {key: self.array(attrs[key]) for key in ('POSITION', 'NORMAL', 'TEXCOORD_0')}
                p['indices'] = self.array(primitive['indices']).reshape(-1).astype(np.uint32)
                p['material'] = primitive['material']
                if skin:
                    p['joints'] = self.array(attrs['JOINTS_0']).astype(np.int64)
                    p['weights'] = self.array(attrs['WEIGHTS_0']).astype(np.float64)
                    require(p['joints'].shape == p['weights'].shape == (len(p['POSITION']), 4), 'source four-influence skin incomplete')
                    require(p['joints'].min() >= 0 and p['joints'].max() < len(binds), 'source joint range')
                    require((p['weights'] >= 0).all() and abs(p['weights'].sum(1) - 1).max() < 1e-5, 'source skin weight sum')
                    p['weights'] /= p['weights'].sum(1, keepdims=True)
                require(len(p['indices']) % 3 == 0 and p['indices'].max() < len(p['POSITION']), 'source triangle indices')
                primitives.append(p)
            yield {'node': i, 'name': mesh['name'], 'skin': skin, 'binds': binds, 'primitives': primitives}

    def posed(self, mesh, primitive):
        if not mesh['skin']:
            return points(self.model(mesh['node']), primitive['POSITION'])
        matrices = np.array([self.model(joint) @ bind for joint, bind in zip(mesh['skin']['joints'], mesh['binds'], strict=True)])
        homogeneous = np.column_stack((primitive['POSITION'], np.ones(len(primitive['POSITION']))))
        return (np.einsum('nvij,nj->nvi', matrices[primitive['joints']], homogeneous) * primitive['weights'][:, :, None]).sum(1)[:, :3]


def quantize_weights(weights):
    """Largest-remainder quantization: preserve palette, exactly 255 per vertex."""
    normalized = weights / weights.sum(1, keepdims=True)
    scaled = normalized * 255
    result = np.floor(scaled).astype(np.int64)
    remaining = 255 - result.sum(1)
    order = np.argsort(-(scaled - result), axis=1, kind='stable')
    for i in range(4):
        rows = np.flatnonzero(remaining > i)
        result[rows, order[rows, i]] += 1
    require((result >= 0).all() and (result <= 255).all() and (result.sum(1) == 255).all(), 'weight quantization failed')
    return result.astype(np.uint8)


def driver_for(glb, node, role):
    exact = {source: i for i, (_, source) in enumerate(DRIVERS) if source}
    current = node
    while current is not None:
        if glb.doc['nodes'][current]['name'] in exact:
            return exact[glb.doc['nodes'][current]['name']]
        current = glb.parents.get(current)
    if role == 'face':
        return 6  # Original generic face's isolated facial nodes follow Head.
    if glb.doc['nodes'][node]['name'] in ('MotionRoot', 'body_dummy'):
        return 0
    raise ValueError(f'no explicit structural ancestor for {role}/{glb.doc["nodes"][node]["name"]}')


def srgb_linear(values):
    values = np.asarray(values, dtype=np.float64)
    return np.where(values <= 0.04045, values / 12.92, ((values + 0.055) / 1.055) ** 2.4)


def skin_color(glb):
    mesh = next(m for m in glb.meshes() if m['name'] == 'body')
    primitive = mesh['primitives'][0]
    mat = glb.doc['materials'][primitive['material']]
    raw = glb.image(mat['pbrMetallicRoughness']['baseColorTexture']['index'])
    image = np.array(Image.open(io.BytesIO(raw)).convert('RGB'))
    names = [glb.doc['nodes'][i]['name'] for i in mesh['skin']['joints']]
    palette = np.array([('Hand' in n or 'Finger' in n or 'Forearm' in n) for n in names])
    choose = (palette[primitive['joints']] * primitive['weights']).sum(1) > 0.8
    uv = primitive['TEXCOORD_0'][choose] % 1
    require(len(uv) > 50, 'insufficient original hand skin UV samples')
    colors = image[np.minimum((uv[:, 1] * image.shape[0]).astype(int), image.shape[0] - 1),
                   np.minimum((uv[:, 0] * image.shape[1]).astype(int), image.shape[1] - 1)]
    median = np.median(colors, axis=0) / 255
    require(median.max() - median.min() > 0.04 and 0.15 < median.mean() < 0.9, 'implausible source hand skin tone')
    return srgb_linear(median), {'method': 'Median original male_body_02 _MainTex pixels at actual hand/finger/forearm-weighted body UV0; sRGB decoded to linear material tint.',
                                'texture_sha256': sha(raw), 'samples': len(uv),
                                'median_srgb': median.tolist(), 'linear_tint': srgb_linear(median).tolist()}


def neutral_eye_png(glb, mesh, primitive, size=512):
    """Authored neutral iris baked into the unchanged original eye UV charts.

    Rasterize source UV triangles and evaluate a small brown iris/pupil on the
    source eye's front (+Z). This is Kiva appearance art, not a recovered atlas.
    """
    world = glb.posed(mesh, primitive)
    uv = primitive['TEXCOORD_0'].astype(np.float64)
    require(uv.min() >= -1e-5 and uv.max() <= 1.00001, 'eye UV outside expected source atlas')
    rgb = np.full((size, size, 4), [199, 196, 180, 255], dtype=np.uint8)
    centers = {}
    radii = {}
    for side in (-1, 1):
        values = world[world[:, 0] * side > 0]
        require(len(values) > 20, 'original eye geometry missing a side')
        centers[side] = (values.min(0) + values.max(0)) / 2
        radii[side] = min(np.ptp(values[:, 0]), np.ptp(values[:, 1])) / 2
    for tri in primitive['indices'].reshape(-1, 3):
        coords = uv[tri] * (size - 1)
        minimum = np.maximum(np.floor(coords.min(0)).astype(int), 0)
        maximum = np.minimum(np.ceil(coords.max(0)).astype(int), size - 1)
        matrix = np.column_stack((coords[0] - coords[2], coords[1] - coords[2]))
        if abs(np.linalg.det(matrix)) < 1e-8:
            continue
        xx, yy = np.meshgrid(np.arange(minimum[0], maximum[0] + 1), np.arange(minimum[1], maximum[1] + 1))
        coords_pixel = np.stack((xx, yy), axis=-1)
        bary = (coords_pixel - coords[2]) @ np.linalg.inv(matrix).T
        inside = (bary >= -1e-5).all(-1) & (bary.sum(-1) <= 1.00001)
        if not inside.any():
            continue
        bary = np.column_stack((bary[inside], 1 - bary[inside].sum(-1)))
        values = bary @ world[tri]
        side = 1 if world[tri, 0].mean() > 0 else -1
        center, radius = centers[side], radii[side]
        radial = np.linalg.norm(values[:, :2] - center[:2], axis=1) / radius
        front = values[:, 2] > center[2]
        iris = (radial < 0.48) & front
        pupil = (radial < 0.22) & front
        colors = np.tile([199, 196, 180, 255], (len(values), 1))
        colors[iris] = [76, 54, 34, 255]
        colors[pupil] = [10, 9, 8, 255]
        rgb[yy[inside], xx[inside]] = colors
    dark = int((rgb[:, :, :3].mean(2) < 100).sum())
    require(10 < dark < size * size / 2, 'neutral eye atlas lacks bounded iris/pupil')
    image = Image.fromarray(rgb)
    encoded = io.BytesIO()
    image.save(encoded, format='PNG', optimize=True)
    return encoded.getvalue(), {'adaptation': 'Kiva neutral brown iris/pupil and warm sclera baked to original source UV0; no source eye/makeup/face-preset texture recovered.', 'dark_pixels': dark}


class Writer:
    def __init__(self):
        self.data = bytearray(b'KIVAAV03')

    def write(self, fmt, *values):
        self.data.extend(struct.pack('<' + fmt, *values))

    def string(self, value):
        raw = value.encode()
        require(len(raw) < 65536, 'name exceeds u16')
        self.write('H', len(raw))
        self.data.extend(raw)


class Reader:
    def __init__(self, data):
        self.data, self.at = data, 0

    def bytes(self, length):
        require(length >= 0 and self.at + length <= len(self.data), 'truncated avatar binary')
        raw = self.data[self.at:self.at + length]
        self.at += length
        return raw

    def read(self, fmt):
        return struct.unpack('<' + fmt, self.bytes(struct.calcsize('<' + fmt)))

    def string(self):
        return self.bytes(self.read('H')[0]).decode()


def verify_binary(path, raw=None):
    raw = path.read_bytes() if raw is None else raw
    reader = Reader(raw)
    require(reader.bytes(8) == b'KIVAAV03', 'unexpected avatar magic')
    header = json.loads(reader.bytes(reader.read('I')[0]))
    n = reader.read('H')[0]
    require(n == len(DRIVERS), 'expected 23 structural drivers')
    anchors = []
    for name, _ in DRIVERS:
        require(reader.string() == name, 'driver order changed')
        anchors.append(reader.read('3f'))
    require(np.isfinite(anchors).all(), 'nonfinite anchors')
    texture_count = reader.read('H')[0]
    for _ in range(texture_count):
        reader.string()
        srgb, length = reader.read('BI')
        require(srgb in (0, 1), 'texture color-space flag')
        image = Image.open(io.BytesIO(reader.bytes(length)))
        require(image.format == 'PNG' and max(image.size) <= header['max_texture_size'], 'invalid/oversized bundled PNG')
        image.verify()
    material_count = reader.read('H')[0]
    double_sided = 0
    for _ in range(material_count):
        reader.string()
        base, normal, surface, alpha, *response = reader.read('hhhB11f')
        double = reader.read('B')[0]
        require(all(-1 <= i < texture_count for i in (base, normal, surface)), 'material texture index')
        require(normal == surface == -1, 'unverified normal/packed map accidentally connected')
        require(alpha in (0, 1) and double in (0, 1) and np.isfinite(response).all(), 'invalid material response')
        double_sided += double
    part_count = reader.read('H')[0]
    vertices, indices, blade_count, source_pose_nonbind_bones = 0, 0, 0, 0
    posed_points, part_stats = [], []
    for _ in range(part_count):
        name = reader.string()
        blade, bones = reader.read('BH')
        require(blade in (0, 1) and 1 <= bones <= 256, 'invalid part/palette header')
        blade_count += blade
        transforms = []
        for _ in range(bones):
            require(reader.string().startswith('naraka::'), 'unprefixed Naraka palette could alias source animations')
            driver = reader.read('H')[0]
            require(driver < n, 'driver index outside structural map')
            inverse = np.array(reader.read('16f')).reshape((4, 4), order='F')
            rest = np.array(reader.read('16f')).reshape((4, 4), order='F')
            require(np.isfinite(inverse).all() and np.isfinite(rest).all(), 'nonfinite matrix')
            require(abs(np.linalg.det(inverse)) > 1e-4 and abs(np.linalg.det(rest)) > 1e-4, 'singular source IBM/rest')
            transforms.append(rest @ inverse)
            source_pose_nonbind_bones += int(abs(rest @ inverse - np.eye(4)).max() > 1e-3)
        count = reader.read('H')[0]
        total = 0
        for _ in range(count):
            material, vc, ic = reader.read('HII')
            require(material < material_count and vc > 0 and ic > 0 and ic % 3 == 0, 'invalid primitive counts')
            data = np.frombuffer(reader.bytes(vc * 40), dtype=np.dtype([('p', '<f4', 3), ('n', '<f4', 3), ('uv', '<f4', 2), ('j', 'u1', 4), ('w', 'u1', 4)]))
            require(np.isfinite(data['p']).all() and np.isfinite(data['n']).all() and np.isfinite(data['uv']).all(), 'nonfinite vertex')
            require((data['j'] < bones).all() and (data['w'].astype(int).sum(1) == 255).all(), 'joint index/weights255 invalid')
            require(abs(np.linalg.norm(data['n'], axis=1) - 1).max() < 1e-4, 'invalid unit vertex normal')
            tris = np.frombuffer(reader.bytes(ic * 4), dtype='<u4')
            require(tris.max() < vc, 'triangle index outside primitive')
            homogeneous = np.column_stack((data['p'], np.ones(vc)))
            posed = (np.einsum('nvij,nj->nvi', np.array(transforms)[data['j']], homogeneous) * (data['w'] / 255)[:, :, None]).sum(1)[:, :3]
            if blade:
                extent = np.ptp(data['p'], axis=0)
                require(extent.argmax() == 1 and extent[1] > .9 and data['p'][:, 1].max() > .8,
                        'serialized rigid blade axis is not+Y')
            if not blade and not name.startswith('naraka::sheath'):
                posed_points.append(posed)
            vertices += vc
            indices += ic
            total += vc
        part_stats.append({'name': name, 'blade': bool(blade), 'bones': bones, 'vertices': total})
    require(reader.at == len(raw), 'avatar binary has trailing bytes')
    require(blade_count == 1, 'expected one rigid melee blade')
    require(source_pose_nonbind_bones > 0, 'source imported pose unexpectedly replaced by bind-pose identity')
    blade_c = np.array(header['blade_conversion'])
    require(np.allclose(points(blade_c, np.array([header['blade_grip_source']])), 0, atol=1e-6),
            'serialized authored grip correction does not center hilt')
    axis = blade_c[:3, :3] @ [0., 0., 1.]
    require(np.dot(axis / np.linalg.norm(axis), [0, 1, 0]) > .99999, 'serialized blade forward axis')
    geometry = np.concatenate(posed_points)
    bounds = [geometry.min(0).tolist(), geometry.max(0).tolist()]
    height = float(np.ptp(geometry[:, 1]))
    require(abs(height - 1.73) < 0.002 and abs(geometry[:, 1].min()) < 0.002, 'actual source-LBS full character is not height1.73/floor0')
    return {'file': str(path), 'bytes': len(raw), 'sha256': sha(raw), 'drivers': n,
            'textures': texture_count, 'materials': material_count, 'double_sided_materials': double_sided,
            'parts': part_count, 'vertices': vertices, 'triangles': indices // 3,
            'actual_binary_lbs_character_bounds': bounds, 'actual_binary_character_height': height,
            'part_statistics': part_stats, 'exact_EOF': True,
            'source_pose_nonbind_bones_preserved': source_pose_nonbind_bones,
            'browser_or_screenshot_test': False}


def posed_normals(glb, mesh, primitive):
    if not mesh['skin']:
        return normals(glb.model(mesh['node']), primitive['NORMAL'])
    matrices = np.array([glb.model(j) @ b for j, b in zip(mesh['skin']['joints'], mesh['binds'], strict=True)])
    value = (np.einsum('nvij,nj->nvi', matrices[primitive['joints'], :3, :3], primitive['NORMAL']) * primitive['weights'][:, :, None]).sum(1)
    return value / np.linalg.norm(value, axis=1)[:, None]


def build(source, output, max_texture_size):
    require(max_texture_size in (512, 1024), 'texture limit must be 512 or1024')
    export = source / 'export' if (source / 'export').is_dir() else source
    declared = json.loads((export / 'manifest.json').read_text())
    models = {role: Glb(export / filename, digest) for role, (filename, digest) in SOURCES.items()}
    for entry in declared['deliverables']:
        local = export / ('weapon' if '/weapon/' in entry['glb']['file'] else 'character') / Path(entry['glb']['file']).name
        require(sha(local.read_bytes()) == entry['glb']['sha256'], 'source manifest GLB changed')
    full = np.concatenate([models[role].posed(mesh, p) for role in ('outfit', 'hair', 'face') for mesh in models[role].meshes() for p in mesh['primitives']])
    floor, top = float(full[:, 1].min()), float(full[:, 1].max())
    scale = 1.73 / (top - floor)
    conversion = np.diag([-scale, scale, -scale, 1.])  # Already-glTF source +Z -> gameplay -Z; no extra Unity reflection.
    conversion[1, 3] = -floor * scale
    inverse_conversion = np.linalg.inv(conversion)
    common = models['hair']
    anchors = []
    for name, bone in DRIVERS:
        if bone is None:
            anchors.append(np.zeros(3))
        else:
            found = [i for i, n in enumerate(common.doc['nodes']) if n['name'] == bone]
            require(len(found) == 1, f'configured original l22 anchor missing/ambiguous: {bone}')
            anchors.append((conversion @ common.model(found[0]))[:3, 3])
    anchors = np.array(anchors)
    require(anchors[10, 0] < 0 < anchors[14, 0], 'left/right driver anchors flipped')
    require(anchors[18, 2] < anchors[17, 2] and anchors[22, 2] < anchors[21, 2], 'toe/foot source evidence does not face gameplay-Z')
    skin_tint, skin_evidence = skin_color(models['outfit'])
    textures, texture_keys, materials, material_keys, texture_evidence = [], {}, [], {}, []
    parts, errors = [], []
    eye_evidence = None

    def texture(raw, label, adapted=False):
        key = sha(raw)
        if key not in texture_keys:
            image = Image.open(io.BytesIO(raw)).convert('RGBA')
            original_size = image.size
            image.thumbnail((max_texture_size, max_texture_size), Image.Resampling.LANCZOS)
            data = io.BytesIO()
            image.save(data, format='PNG', optimize=True)
            texture_keys[key] = len(textures)
            textures.append((f'naraka::{label}', True, data.getvalue()))
            texture_evidence.append({'name': label, 'source_png_sha256': key,
                                     'source_size': original_size, 'embedded_size': image.size,
                                     'embedded_png_sha256': sha(data.getvalue()),
                                     'adaptation': 'Authored Kiva neutral eye atlas' if adapted else 'Resize only, source RGB/alpha retained; normal/packed maps not connected.'})
        return texture_keys[key]

    def material(role, glb, mesh, primitive):
        nonlocal eye_evidence
        key = (role, primitive['material'])
        if key in material_keys:
            return material_keys[key]
        original = glb.doc['materials'][primitive['material']]
        pbr = original.get('pbrMetallicRoughness', {})
        base = -1
        if 'baseColorTexture' in pbr:
            base = texture(glb.image(pbr['baseColorTexture']['index']), role + '::' + original['name'])
        tint = pbr.get('baseColorFactor', [1, 1, 1, 1])[:3]
        roughness, metallic, reflectance, transmission = 0.78, 0., 0.38, 0.
        alpha = original.get('alphaMode') == 'MASK'
        cutoff = original.get('alphaCutoff', .5)
        double = bool(original.get('doubleSided', False))
        if role == 'face':
            base, alpha, double = -1, False, False
            if mesh['name'].startswith('face_'):
                tint, transmission = skin_tint.tolist(), .02
            elif mesh['name'].startswith('eyes_'):
                raw, eye_evidence = neutral_eye_png(glb, mesh, primitive)
                base = texture(raw, 'authored_neutral_brown_eyes', True)
                tint, roughness, reflectance = [1., 1., 1.], .3, .5
            elif mesh['name'].startswith('eyelash_'):
                tint, roughness, double = [.008, .006, .004], .9, True
            else:
                raise ValueError('unexpected generic face component')
        elif role == 'blade':
            roughness, metallic, reflectance = .4, .7, .5
        elif role == 'sheath':
            roughness, metallic = .65, .15
        response = [roughness, metallic, reflectance, 0., 0., .18, transmission, *tint, cutoff]
        require(len(response) == 11 and np.isfinite(response).all(), 'material response shape')
        index = len(materials)
        materials.append((f'naraka::{role}::{original["name"]}', base, -1, -1, alpha, response, double))
        material_keys[key] = index
        return index

    for role in ('outfit', 'hair', 'face'):
        glb = models[role]
        for mesh in glb.meshes():
            require(mesh['skin'] is not None, 'character component must retain actual source skin')
            names = [glb.doc['nodes'][i]['name'] for i in mesh['skin']['joints']]
            rest = np.array([conversion @ glb.model(i) for i in mesh['skin']['joints']])
            binds = np.array([b @ inverse_conversion for b in mesh['binds']])
            bones = [(f'naraka::{role}::{name}', driver_for(glb, i, role), inverse, model)
                     for name, i, inverse, model in zip(names, mesh['skin']['joints'], binds, rest, strict=True)]
            require(len(bones) <= 256, 'palette does not fit four u8 joints')
            primitives = []
            for primitive in mesh['primitives']:
                positions = points(conversion, primitive['POSITION'])
                weights = quantize_weights(primitive['weights'])
                joints = primitive['joints']
                homogeneous = np.column_stack((positions, np.ones(len(positions))))
                transforms = np.einsum('nij,njk->nik', rest, binds)
                float_pose = (np.einsum('nvij,nj->nvi', transforms[joints], homogeneous) * primitive['weights'][:, :, None]).sum(1)[:, :3]
                expected = points(conversion, glb.posed(mesh, primitive))
                float_error = float(abs(float_pose - expected).max())
                require(float_error < 1e-6, 'C/rest/IBM source-LBS equivalence broken')
                # Repeat with exactly the float32 matrices, positions and bytes
                # that are serialized, rather than validating higher precision only.
                serialized = np.einsum('nij,njk->nik', rest.astype(np.float32).astype(float), binds.astype(np.float32).astype(float))
                encoded_p = np.column_stack((positions.astype(np.float32), np.ones(len(positions))))
                byte_pose = (np.einsum('nvij,nj->nvi', serialized[joints], encoded_p) * (weights / 255)[:, :, None]).sum(1)[:, :3]
                byte_error = float(np.linalg.norm(byte_pose - expected, axis=1).max())
                require(byte_error < .005, 'weights255 source-pose error exceeds5mm')
                uv = primitive['TEXCOORD_0']
                triangles = primitive['indices'].reshape(-1, 3)
                edge_a = uv[triangles[:, 1]] - uv[triangles[:, 0]]
                edge_b = uv[triangles[:, 2]] - uv[triangles[:, 0]]
                determinant = edge_a[:, 0] * edge_b[:, 1] - edge_a[:, 1] * edge_b[:, 0]
                require((abs(determinant) > 1e-10).any(), 'all source UV triangles degenerate; tangent generation unsafe')
                errors.append({'role': role, 'mesh': mesh['name'], 'vertices': len(positions),
                               'float_C_source_LBS_error_max': float_error,
                               'serialized_f32_weights255_error_max_m': byte_error,
                               'nondegenerate_UV_triangles': int((abs(determinant) > 1e-10).sum()),
                               'triangles': len(triangles)})
                primitives.append({'material': material(role, glb, mesh, primitive),
                                   'positions': positions, 'normals': normals(conversion, primitive['NORMAL']),
                                   'uv': uv, 'joints': joints.astype(np.uint8), 'weights': weights,
                                   'indices': primitive['indices']})
            source_node_name = glb.doc['nodes'][mesh['node']]['name']
            parts.append({'name': f'naraka::{role}::{source_node_name}', 'blade': False, 'bones': bones, 'primitives': primitives})

    # The red wrapped hilt occupies source z approximately[-.55,-.25], with the
    # broad guard at[-.2,-.1] and silver blade toward+Z. This offline authored
    # grip lies inside the actual narrow hilt; no source wrist offset is claimed.
    grip = np.array([0., 0., -.395])
    rotation = np.array([[1., 0., 0.], [0., 0., 1.], [0., -1., 0.]])
    blade_conversion = np.eye(4)
    blade_conversion[:3, :3] = scale * rotation
    blade_conversion[:3, 3] = -scale * rotation @ grip
    glb = models['blade']
    mesh = next(glb.meshes())
    primitive = mesh['primitives'][0]
    original_posed = glb.posed(mesh, primitive)
    blade_positions = points(blade_conversion, original_posed)
    require(np.allclose(points(blade_conversion, grip[None, :]), 0, atol=1e-12), 'blade grip not at origin')
    require(blade_positions[:, 1].max() > .8 and blade_positions[:, 1].min() < 0, 'rigid blade does not extend+Y from hilt')
    source_rig = [{'name': glb.doc['nodes'][i]['name'], 'source_rest_model': glb.model(i).tolist(), 'inverse_bind': b.tolist()}
                  for i, b in zip(mesh['skin']['joints'], mesh['binds'], strict=True)]
    parts.append({'name': 'naraka::zhulei::rigid_blade', 'blade': True,
                  'bones': [('naraka::zhulei::baked_rigid_adapter', 14, np.eye(4), np.eye(4))],
                  'primitives': [{'material': material('blade', glb, mesh, primitive),
                                  'positions': blade_positions, 'normals': normals(blade_conversion, posed_normals(glb, mesh, primitive)),
                                  'uv': primitive['TEXCOORD_0'], 'indices': primitive['indices'],
                                  'joints': np.zeros((len(blade_positions), 4), dtype=np.uint8),
                                  'weights': np.tile([255, 0, 0, 0], (len(blade_positions), 1)).astype(np.uint8)}]})
    # Rigid sheath at left hip: an explicit Kiva art placement, not a recovered
    # actor attachment. Source static geometry/UV are retained before this pose.
    glb = models['sheath']
    mesh = next(glb.meshes())
    primitive = mesh['primitives'][0]
    theta = -.15
    z_rotation = np.array([[np.cos(theta), -np.sin(theta), 0], [np.sin(theta), np.cos(theta), 0], [0, 0, 1]])
    sheath_conversion = np.eye(4)
    sheath_conversion[:3, :3] = scale * z_rotation @ rotation.T
    sheath_conversion[:3, 3] = [-.22, .9, .1]
    sheath_positions = points(sheath_conversion, glb.posed(mesh, primitive))
    require(sheath_positions[:, 1].min() > -.05, 'authored sheath penetrates ground at rest')
    parts.append({'name': 'naraka::sheath::authored_hip', 'blade': False,
                  'bones': [('naraka::sheath::rigid_adapter', 1, np.eye(4), np.eye(4))],
                  'primitives': [{'material': material('sheath', glb, mesh, primitive),
                                  'positions': sheath_positions, 'normals': normals(sheath_conversion, posed_normals(glb, mesh, primitive)),
                                  'uv': primitive['TEXCOORD_0'], 'indices': primitive['indices'],
                                  'joints': np.zeros((len(sheath_positions), 4), dtype=np.uint8),
                                  'weights': np.tile([255, 0, 0, 0], (len(sheath_positions), 1)).astype(np.uint8)}]})

    source_records = [{'role': role, 'file': filename, 'sha256': digest, 'packed': role != 'ornament'} for role, (filename, digest) in SOURCES.items()]
    hair_source = json.loads((export / 'character/xinsu_reconstructed_binding.source.json').read_text())
    identity = json.loads((export.parent / 'identity/target_parts.json').read_text())
    provenance = {'format': 'KIVAAV03', 'height_m': 1.73, 'source_yaw_degrees': 180,
                  'max_texture_size': max_texture_size, 'source_floor': floor, 'source_top_complete_body_head_hair': top,
                  'source_scale': scale, 'conversion': conversion.tolist(),
                  'bone_names_prefix': 'naraka::', 'source_animation_name_matching': False,
                  'matrix_contract': "POSITION'=C*POSITION; IBM'=IBM*C^-1; sourceRestModel'=C*originalBoneWorld. Original imported pose may differ from bind pose; do not replace rest with inverse(IBM).",
                  'sources': source_records,
                  'face': 'Generic source base face with sampled source hand skin tone and authored neutral brown eyes/charcoal lashes. LiHuowang1415608 facial parameters/1400595 makeup are NOT applied.',
                  'hair': 'Original geometry/UV/weights/IBMs; six uniquely evidenced reconstructed external parent links, not serialized original SMR palettes. RGB data maps suppressed, only source alpha used with neutral black material.',
                  'shader': 'Source BaseColor/alpha only; no guessed source normal/packed channels, no source anisotropic/SH/FX/cloth/morph shader. Neutral Kiva PBR adaptations.',
                  'weapon': 'Original four-bone current-pose LBS baked into rigid blade with tassels. Authored hilt center and+Y axis; original skin/IBMs retained in source manifest.',
                  'blade_grip_source': grip.tolist(), 'blade_conversion': blade_conversion.tolist(),
                  'sheath': 'Original static source geometry with authored Kiva left-hip placement, following pelvis structural delta.',
                  'ornament': 'Original Guidao source retained as evidence but not packed: dynamic FX shader is not reconstructed.',
                  'structural_driver_adaptation': 'Original l22/current imported source anchor proportions; not the YYSLS rest pose. Source ancestor maps to23 existing structural deltas, preserving each palette original source rest model.'}
    writer = Writer()
    header = json.dumps(provenance, separators=(',', ':')).encode()
    writer.write('I', len(header))
    writer.data.extend(header)
    writer.write('H', len(DRIVERS))
    for (name, _), anchor in zip(DRIVERS, anchors, strict=True):
        writer.string(name)
        writer.write('3f', *anchor)
    writer.write('H', len(textures))
    for name, srgb, raw in textures:
        writer.string(name)
        writer.write('BI', srgb, len(raw))
        writer.data.extend(raw)
    writer.write('H', len(materials))
    for name, base, normal, surface, alpha, response, double in materials:
        writer.string(name)
        writer.write('hhhB11f', base, normal, surface, alpha, *response)
        writer.write('B', double)
    writer.write('H', len(parts))
    for part in parts:
        writer.string(part['name'])
        writer.write('BH', part['blade'], len(part['bones']))
        for name, driver, inverse, rest in part['bones']:
            writer.string(name)
            writer.write('H16f16f', driver, *inverse.T.reshape(-1), *rest.T.reshape(-1))
        writer.write('H', len(part['primitives']))
        for p in part['primitives']:
            writer.write('HII', p['material'], len(p['positions']), len(p['indices']))
            for position, normal, uv, joint, weight in zip(p['positions'], p['normals'], p['uv'], p['joints'], p['weights'], strict=True):
                writer.write('8f8B', *position, *normal, *uv, *joint, *weight)
            writer.data.extend(p['indices'].astype('<u4').tobytes())
    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_bytes(writer.data)
    binary = verify_binary(output)
    report = {'binary': binary, 'source_pose_checks': errors,
              'maximum_float_C_LBS_error': max(e['float_C_source_LBS_error_max'] for e in errors),
              'maximum_serialized_weights255_error_m': max(e['serialized_f32_weights255_error_max_m'] for e in errors),
              'normalization': {'complete_source_bounds': [full.min(0).tolist(), full.max(0).tolist()],
                                'complete_parts': ['outfit', 'hair', 'generic_source_face'],
                                'scale': scale, 'C': conversion.tolist(), 'driver_anchors': anchors.tolist(),
                                'left_hand_negative_X': bool(anchors[10, 0] < 0), 'right_hand_positive_X': bool(anchors[14, 0] > 0),
                                'toe_forward_negative_Z': True},
              'grip': {'authored_source_hilt_center': grip.tolist(), 'source_axis': '+Z', 'converted_axis': '+Y',
                       'grip_origin_error': float(abs(points(blade_conversion, grip[None, :])).max()),
                       'C_blade_rigid': blade_conversion.tolist(), 'baked_source_bone_count': len(source_rig),
                       'bounds': [blade_positions.min(0).tolist(), blade_positions.max(0).tolist()]},
              'sheath': {'authored': True, 'conversion': sheath_conversion.tolist(), 'bounds': [sheath_positions.min(0).tolist(), sheath_positions.max(0).tolist()]},
              'skin_tone': skin_evidence, 'neutral_eyes': eye_evidence,
              'checks': ['pinned_source_hashes', 'source_four_weights_and_indices', 'all_vertex_float_C_source_LBS_equivalence',
                         'actual_serialized_f32_weights255_source_LBS_error', 'source_rest_and_IBM_finite_nonsingular', 'weights_exact_sum255',
                         'driver_order23', 'left_right_and_toe_facing', 'full_body_head_hair_height1.73_floor0',
                         'source_UV_and_finite_unit_normals', 'material_texture_and_doubleSided_flags', 'PNG_size_and_decode',
                         'rigid_blade_pose_bake_and_grip_axis', 'authored_sheath_rest_above_ground', 'exact_binary_EOF'],
              'browser_or_screenshot_test': False}
    source_manifest = {'provenance': provenance, 'source_export_manifest_sha256': sha((export / 'manifest.json').read_bytes()),
                       'identity': identity, 'source_textures': texture_evidence,
                       'original_weapon_four_bone_rest_and_IBMs': source_rig,
                       'hair_reconstruction': {'source_manifest_sha256': sha((export / 'character/xinsu_reconstructed_binding.source.json').read_bytes()),
                                               'threshold': hair_source['matching_threshold'], 'external_parent_links': hair_source['external_parent_links'],
                                               'slots': [d['binding']['unique_matching_slots'] for d in hair_source['draws']]},
                       'source_face_records_sha256': sha((export.parent / 'identity/face_identity_records.json').read_bytes()),
                       'output': binary, 'limits': provenance['face'] + ' ' + provenance['hair'] + ' ' + provenance['shader'],
                       'game_reads': False, 'runtime_external_paths_or_downloads': False}
    (output.parent / 'conversion-report.json').write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
    (output.parent / 'source-manifest.json').write_text(json.dumps(source_manifest, ensure_ascii=False, indent=2) + '\n')
    print(json.dumps({'output': str(output), 'bytes': binary['bytes'], 'sha256': binary['sha256'],
                      'parts': binary['parts'], 'vertices': binary['vertices'], 'triangles': binary['triangles'],
                      'textures': binary['textures'], 'max_float_C_error': report['maximum_float_C_LBS_error'],
                      'max_weights255_error_m': report['maximum_serialized_weights255_error_m']}))


def self_test(binary=None):
    values = np.array([[1., 0, 0, 0], [.25, .25, .25, .25], [.999, .001, 0, 0], [.4, .3, .2, .1]])
    q = quantize_weights(values)
    require((q.astype(int).sum(1) == 255).all() and (q[values == 0] == 0).all(), 'weights255 edge cases')
    # A genuinely posed source joint (rest != inverse IBM) must survive C;
    # testing only bind cancellation would miss the original pose loss.
    c = np.diag([-.9, .9, -.9, 1.])
    c[1, 3] = -.02
    original = np.eye(4)
    original[:3, 3] = [.2, 1.2, .3]
    inverse = np.eye(4)
    inverse[:3, 3] = [-.1, -.8, -.2]
    vertex = np.array([[.2, .3, .4]])
    expected = points(c, points(original @ inverse, vertex))
    actual = points((c @ original) @ (inverse @ np.linalg.inv(c)), points(c, vertex))
    require(np.allclose(actual, expected, atol=1e-12), 'source posed rest/IBM conversion regression')
    print('weights255 and non-bind source-pose conversion checks passed')
    if binary is not None:
        corrupt_binary_checks(binary)


def corrupt_binary_checks(path):
    """Reject actual-artifact corruptions through the complete format reader."""
    verify_binary(path)
    raw = path.read_bytes()
    scan = Reader(raw)
    scan.bytes(8)
    scan.bytes(scan.read('I')[0])
    for _ in range(scan.read('H')[0]):
        scan.string()
        scan.bytes(12)
    for _ in range(scan.read('H')[0]):
        scan.string()
        scan.bytes(1)
        scan.bytes(scan.read('I')[0])
    material_count = scan.read('H')[0]
    double_offset = None
    for _ in range(material_count):
        scan.string()
        scan.bytes(struct.calcsize('<hhhB11f'))
        double_offset = scan.at if double_offset is None else double_offset
        scan.bytes(1)
    scan.read('H')
    scan.string()
    _, bones = scan.read('BH')
    rest_offset = None
    for _ in range(bones):
        scan.string()
        scan.bytes(2 + 64)
        rest_offset = scan.at if rest_offset is None else rest_offset
        scan.bytes(64)
    scan.read('H')
    _, vertices, _ = scan.read('HII')
    first_vertex = scan.at
    index_offset = first_vertex + vertices * 40
    cases = {'magic': (0, b'X'), 'double_sided_flag': (double_offset, b'\x02'),
             'nonfinite_source_rest': (rest_offset, struct.pack('<f', float('nan'))),
             'joint_range': (first_vertex + 32, b'\xff'),
             'weights255': (first_vertex + 36, bytes([(raw[first_vertex + 36] + 1) % 256])),
             'triangle_range': (index_offset, struct.pack('<I', vertices))}
    failures = []
    for name, (at, replacement) in cases.items():
        modified = bytearray(raw)
        modified[at:at + len(replacement)] = replacement
        try:
            verify_binary(Path('<corrupt-memory>'), bytes(modified))
        except ValueError:
            failures.append(name)
        else:
            raise ValueError(f'corrupt avatar accepted: {name}')
    for name, modified in [('truncated', raw[:-1]), ('trailing', raw + b'\x00')]:
        try:
            verify_binary(Path('<corrupt-memory>'), modified)
        except ValueError:
            failures.append(name)
        else:
            raise ValueError(f'corrupt avatar accepted: {name}')
    print('actual binary rejection controls passed: ' + ', '.join(failures))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path)
    parser.add_argument('--output', type=Path, default=Path(__file__).resolve().parents[1] / 'assets/lihuowang/character.bin')
    parser.add_argument('--max-texture-size', type=int, default=1024)
    parser.add_argument('--verify', type=Path)
    parser.add_argument('--self-test', action='store_true')
    args = parser.parse_args()
    if args.self_test:
        self_test(args.verify)
    elif args.verify:
        print(json.dumps(verify_binary(args.verify), ensure_ascii=False))
    else:
        if args.source is None:
            parser.error('--source is required')
        build(args.source.resolve(), args.output, args.max_texture_size)


if __name__ == '__main__':
    main()
