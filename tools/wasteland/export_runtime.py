"""Export an authored Zhuque .blend to GLB and bounded KIVAAV04.

Run with a separate Blender --background --factory-startup process. The
authoring file is only read; this script never saves changes back into it.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import struct
import sys
from pathlib import Path

import bpy
from mathutils import Matrix, Vector
from mathutils.kdtree import KDTree

B = Matrix(((1, 0, 0, 0), (0, 0, -1, 0), (0, 1, 0, 0), (0, 0, 0, 1)))
BI = B.inverted()
AUXILIARY_MAP = {name: 'gMan Spine2' for name in ('cape_L', 'cape_C', 'cape_R')}


def require(value, message):
    if not value:
        raise ValueError(message)


def triangle_budget(value):
    try:
        count = int(value)
    except ValueError as error:
        raise argparse.ArgumentTypeError('triangle budget must be an integer') from error
    if not 1 <= count <= 250000:
        raise argparse.ArgumentTypeError('triangle budget must be between 1 and 250000')
    return count


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def rows(matrix):
    return [list(row) for row in matrix]


def matrix_error(a, b):
    return max(abs(a[r][c] - b[r][c]) for r in range(4) for c in range(4))


def f32(value):
    return struct.unpack('<f', struct.pack('<f', value))[0]


class Writer:
    def __init__(self):
        self.data = bytearray()

    def write(self, fmt, *values):
        self.data.extend(struct.pack('<' + fmt, *values))

    def string(self, value):
        raw = value.encode('utf-8')
        require(len(raw) <= 65535, 'string exceeds u16')
        self.write('H', len(raw))
        self.data.extend(raw)

    def matrix(self, matrix):
        self.write('16f', *(matrix[r][c] for c in range(4) for r in range(4)))


def reset(rig):
    for bone in rig.pose.bones:
        bone.matrix_basis = Matrix.Identity(4)
    bpy.context.view_layer.update()


def pose_rig(rig, reference, pose):
    reset(rig)
    desired = {}
    for node in reference['nodes']:
        name = node['name']
        if name not in pose['joints']:
            continue
        original = B @ Matrix(node['source_rest_game']) @ BI
        desired[name] = B @ Matrix(pose['joints'][name]) @ BI @ original.inverted() @ rig.data.bones[name].matrix_local
    for bone in rig.data.bones:
        if bone.name not in desired:
            continue
        if bone.parent:
            parent = bone.parent
            basis = bone.convert_local_to_pose(desired[bone.name], bone.matrix_local,
                                               parent_matrix=desired.get(parent.name, parent.matrix_local),
                                               parent_matrix_local=parent.matrix_local, invert=True)
        else:
            basis = bone.convert_local_to_pose(desired[bone.name], bone.matrix_local, invert=True)
        rig.pose.bones[bone.name].matrix_basis = basis
    bpy.context.view_layer.update()


def quantize(groups):
    values = sorted(groups.items(), key=lambda item: (-item[1], item[0]))[:4]
    total = sum(weight for _, weight in values)
    require(total > 1e-8, 'unweighted vertex')
    scaled = [weight / total * 255 for _, weight in values]
    result = [math.floor(weight) for weight in scaled]
    order = sorted(range(len(values)), key=lambda i: (-(scaled[i] - result[i]), i))
    for i in order[:255 - sum(result)]:
        result[i] += 1
    while len(values) < 4:
        values.append((values[0][0], 0))
        result.append(0)
    require(sum(result) == 255, 'invalid normalized byte weights')
    return [name for name, _ in values], result


def image_source(socket, normal=False, development=False):
    if not socket.is_linked:
        return None
    node = socket.links[0].from_node
    if normal and node.type == 'NORMAL_MAP':
        return image_source(node.inputs['Color'], development=development)
    if development and node.type != 'TEX_IMAGE':
        return None
    require(node.type == 'TEX_IMAGE' and node.image is not None,
            'Bake procedural/complex material nodes to direct ImageTexture before export')
    return node.image


def export_materials(materials, directory, development=False):
    images, textures, records = {}, [], []
    directory.mkdir(parents=True, exist_ok=True)
    def add_image(image, srgb):
        key = (image.as_pointer(), srgb)
        if key not in images:
            require(max(image.size) <= 2048 and min(image.size) > 0, 'texture exceeds 2K budget')
            target = directory / f'texture-{len(textures):02}.png'
            copy = image.copy()
            copy.filepath_raw = str(target)
            copy.file_format = 'PNG'
            copy.save()
            bpy.data.images.remove(copy)
            raw = target.read_bytes()
            require(raw.startswith(b'\x89PNG\r\n\x1a\n') and len(raw) <= 16 * 1024 * 1024, 'invalid PNG export')
            images[key] = len(textures)
            textures.append({'name': image.name, 'srgb': srgb, 'file': str(target), 'raw': raw})
        return images[key]
    for mat in materials:
        require(mat.use_nodes, f'{mat.name}: Principled nodes required')
        shaders = [node for node in mat.node_tree.nodes if node.type == 'BSDF_PRINCIPLED']
        require(len(shaders) == 1, f'{mat.name}: exactly one Principled material')
        shader = shaders[0]
        def texture(socket_name, srgb, normal=False, material_shader=shader):
            image = image_source(material_shader.inputs[socket_name], normal, development)
            if image is None:
                return -1
            return add_image(image, srgb)
        base = texture('Base Color', True)
        normal = texture('Normal', False, True)
        emission = texture('Emission Color', True)
        # ORM is intentionally an explicit authoring contract; do not guess
        # channels of unrelated source images or bake arbitrary node networks.
        orm = -1
        if mat.get('kiva_orm_image'):
            image = bpy.data.images.get(mat['kiva_orm_image'])
            require(image is not None, 'declared ORM image missing')
            orm = add_image(image, False)
        color = [1.0, 1.0, 1.0, 1.0] if base >= 0 else list(shader.inputs['Base Color'].default_value)
        emit = list(shader.inputs['Emission Color'].default_value)[:3]
        strength = float(shader.inputs['Emission Strength'].default_value)
        if emission >= 0:
            emit = [strength] * 3
        else:
            emit = [value * strength for value in emit]
        opacity = float(mat.get('kiva_opacity', shader.inputs['Alpha'].default_value))
        mode = int(mat.get('kiva_alpha_mode', 2 if opacity < 0.999 else 0))
        effect = int(mat.get('kiva_surface_effect', 0))
        require(mode in (0, 1, 2) and 0 <= opacity <= 1, 'invalid alpha mode/opacity')
        require(effect in (0, 1) and (effect == 0 or mode == 2), 'effect must be None or dedicated blend Ember')
        require(all(math.isfinite(value) and 0 <= value <= 1000 for value in emit), 'invalid HDR emissive')
        values = {'name': mat.name, 'base': base, 'normal': normal, 'surface': orm,
                  'roughness': 1.0 if orm >= 0 else float(shader.inputs['Roughness'].default_value),
                  'metallic': 1.0 if orm >= 0 else float(shader.inputs['Metallic'].default_value), 'color': color[:3],
                  'alpha_mode': mode, 'opacity': opacity, 'emissive': emit, 'emissive_texture': emission, 'surface_effect': effect,
                  'double_sided': not mat.use_backface_culling, 'alpha_cutoff': float(mat.get('kiva_alpha_cutoff', 0.5))}
        require(all(math.isfinite(v) for v in (*values['color'], values['roughness'], values['metallic'])), 'nonfinite material')
        require(0 <= values['roughness'] <= 1 and 0 <= values['metallic'] <= 1, 'bounded PBR response')
        records.append(values)
    return textures, records


def mesh_records(objects, rig, palette, allow_projected_uv):
    parts, missing_uv = [], []
    for obj in objects:
        armatures = [modifier for modifier in obj.modifiers if modifier.type == 'ARMATURE']
        require(len(armatures) == 1 and armatures[0].object == rig, f'{obj.name}: own armature required')
        modifier = armatures[0]
        visible = modifier.show_viewport
        modifier.show_viewport = False
        bpy.context.view_layer.update()
        evaluated = obj.evaluated_get(bpy.context.evaluated_depsgraph_get())
        mesh = evaluated.to_mesh(preserve_all_data_layers=True, depsgraph=bpy.context.evaluated_depsgraph_get())
        mesh.calc_loop_triangles()
        uv = mesh.uv_layers.active
        if uv is None:
            require(allow_projected_uv, f'{obj.name}: missing source UV_0; do UV pass or explicitly use --development-projected-uv')
            missing_uv.append(obj.name)
        slots = {group.index: group.name for group in obj.vertex_groups}
        primitives = {}
        normals = obj.matrix_world.to_3x3().inverted().transposed()
        for tri in mesh.loop_triangles:
            material = obj.material_slots[tri.material_index].material
            primitive = primitives.setdefault(material.name, {'material': material.name, 'vertices': [], 'indices': [], '_lookup': {}})
            face_normal = BI.to_3x3() @ normals @ tri.normal
            dominant = max(range(3), key=lambda axis: abs(face_normal[axis]))
            axes = [axis for axis in range(3) if axis != dominant]
            for loop_index in tri.loops:
                vertex = mesh.vertices[mesh.loops[loop_index].vertex_index]
                groups = {}
                for group in vertex.groups:
                    if group.weight <= 1e-8:
                        continue
                    native = slots[group.group]
                    native = AUXILIARY_MAP.get(native, native)
                    require(native in palette, f'{obj.name}: unsupported weighted bone {native}')
                    groups[native] = groups.get(native, 0) + group.weight
                names, weights = quantize(groups)
                point = BI @ obj.matrix_world @ vertex.co
                normal = (BI.to_3x3() @ normals @ mesh.corner_normals[loop_index].vector).normalized()
                texcoord = list(uv.data[loop_index].uv) if uv else [point[axes[0]] * 2 + 0.5, point[axes[1]] * 2 + 0.5]
                # Match Blender's native glTF exporter and the existing GLB
                # runtime converters: PNG sampling uses top-origin V.
                texcoord[1] = 1.0 - texcoord[1]
                record = {'position': [f32(v) for v in point], 'normal': list(normal),
                          'uv': texcoord, 'joints': [palette[name] for name in names],
                          'weights': weights, 'source_vertex': vertex.index}
                key = tuple(record['position'] + record['normal'] + record['uv'] + record['joints'] + weights)
                if key not in primitive['_lookup']:
                    primitive['_lookup'][key] = len(primitive['vertices'])
                    primitive['vertices'].append(record)
                primitive['indices'].append(primitive['_lookup'][key])
        for primitive in primitives.values():
            del primitive['_lookup']
        parts.append({'name': obj.name, 'primitives': list(primitives.values()), 'source_vertices': len(mesh.vertices)})
        evaluated.to_mesh_clear()
        modifier.show_viewport = visible
    bpy.context.view_layer.update()
    return parts, missing_uv


def write_binary(path, header, reference, bones, textures, materials, parts):
    writer = Writer()
    writer.data.extend(b'KIVAAV04')
    encoded = json.dumps(header, ensure_ascii=False, sort_keys=True, separators=(',', ':')).encode()
    writer.write('I', len(encoded))
    writer.data.extend(encoded)
    writer.write('H', len(reference['drivers']))
    for driver in reference['drivers']:
        writer.string(driver['name'])
        writer.write('3f', *driver['position_game'])
    writer.write('H', len(textures))
    for texture in textures:
        writer.string(texture['name'])
        writer.write('BI', int(texture['srgb']), len(texture['raw']))
        writer.data.extend(texture['raw'])
    writer.write('H', len(materials))
    material_indices = {}
    for index, mat in enumerate(materials):
        material_indices[mat['name']] = index
        writer.string(mat['name'])
        writer.write('hhhB11fB', mat['base'], mat['normal'], mat['surface'], int(mat['alpha_mode'] == 1),
                     mat['roughness'], mat['metallic'], 0.5, 0.0, 0.0, 0.0, 0.0,
                     *mat['color'], mat['alpha_cutoff'], int(mat['double_sided']))
        writer.write('B4fhB', mat['alpha_mode'], mat['opacity'], *mat['emissive'], mat['emissive_texture'], mat['surface_effect'])
    writer.write('H', len(parts))
    for part_index, part in enumerate(parts):
        # Retain the explicit complete core once for hand/anatomical metadata;
        # other skins only need their actual weighted palette, not 52 entities.
        selected = list(range(len(bones))) if part_index == 0 else sorted({
            index for primitive in part['primitives'] for vertex in primitive['vertices']
            for index, weight in zip(vertex['joints'], vertex['weights']) if weight})
        remap = {original: local for local, original in enumerate(selected)}
        writer.string('naraka::zhuque::' + part['name'])
        writer.write('BH', 0, len(selected))
        for index in selected:
            bone = bones[index]
            rest = Matrix(bone['source_rest_game'])
            writer.string('naraka::zhuque::' + bone['name'])
            writer.write('H', bone['driver_index'])
            writer.matrix(rest.inverted())
            writer.matrix(rest)
        writer.write('H', len(part['primitives']))
        for primitive in part['primitives']:
            writer.write('HII', material_indices[primitive['material']], len(primitive['vertices']), len(primitive['indices']))
            for vertex in primitive['vertices']:
                indices = [remap[index] if weight else 0 for index, weight in zip(vertex['joints'], vertex['weights'])]
                writer.write('8f8B', *vertex['position'], *vertex['normal'], *vertex['uv'], *indices, *vertex['weights'])
            for index in primitive['indices']:
                writer.write('I', index)
    require(len(writer.data) < 64 * 1024 * 1024, 'runtime file exceeds 64MiB')
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(writer.data)


def verify_skin(objects, rig, reference, bones, parts, poses):
    rests = [Matrix(node['source_rest_game']) for node in bones]
    actual_error, aux_error, matrix_max = 0.0, 0.0, 0.0
    compared = 0
    compared_poses = []
    representative = {0, 7, 11, 27}
    representative.update(i for i, pose in enumerate(poses['poses'])
                          if pose['track'] and pose['time'] > 0.35 and 'tick024' in pose['name'])
    for index, pose in enumerate(poses['poses']):
        pose_rig(rig, reference, pose)
        deltas = [Matrix(pose['joints'][node['name']]) @ rest.inverted() for node, rest in zip(bones, rests)]
        for node, delta in zip(bones, deltas):
            bone = rig.data.bones[node['name']]
            expected = B @ delta @ BI @ bone.matrix_local
            matrix_max = max(matrix_max, matrix_error(expected, rig.pose.bones[node['name']].matrix))
        spine_delta = B @ (Matrix(pose['joints']['gMan Spine2']) @ Matrix(poses['rest']['gMan Spine2']).inverted()) @ BI
        for name in AUXILIARY_MAP:
            bone = rig.data.bones[name]
            current_delta = rig.pose.bones[name].matrix @ bone.matrix_local.inverted()
            aux_error = max(aux_error, matrix_error(spine_delta, current_delta))
        if index not in representative:
            continue
        compared_poses.append(pose['name'])
        for obj, part in zip(objects, parts):
            evaluated = obj.evaluated_get(bpy.context.evaluated_depsgraph_get())
            mesh = evaluated.to_mesh()
            require(len(mesh.vertices) == part['source_vertices'], 'modifier topology changes with pose')
            for primitive in part['primitives']:
                for vertex in primitive['vertices']:
                    point = Vector(vertex['position'])
                    predicted = Vector((0, 0, 0))
                    for joint, weight in zip(vertex['joints'], vertex['weights']):
                        if weight:
                            predicted += (deltas[joint] @ point) * (weight / 255)
                    actual = BI @ obj.matrix_world @ mesh.vertices[vertex['source_vertex']].co
                    actual_error = max(actual_error, (predicted - actual).length)
                    compared += 1
            evaluated.to_mesh_clear()
    reset(rig)
    require(aux_error < 0.00001, 'declared auxiliary collapse is not equivalent in all runtime poses')
    require(matrix_max < 0.00001 and actual_error < 0.005, 'actual Blender/runtime serialized LBS proof failed')
    return {'runtime_snapshots': len(poses['poses']), 'maximum_joint_matrix_error': matrix_max,
            'maximum_auxiliary_collapse_matrix_error': aux_error,
            'actual_blender_vertices_compared': compared, 'actual_blender_compared_poses': compared_poses,
            'maximum_f32_weights255_lbs_error_m': actual_error}


def bind_glb_occlusion(path, materials, textures):
    """Honor explicit ORM without relying on Blender's optional AO node group.

    Keep Blender's texture/image/sampler indices. Its MR packer need not retain
    red when AO has no shader input, so use the declared complete ORM payload.
    Existing buffer views remain untouched; replacements append aligned bytes.
    """
    raw = path.read_bytes()
    chunks, offset = {}, 12
    while offset < len(raw):
        length, kind = struct.unpack_from('<I4s', raw, offset)
        chunks[kind] = raw[offset + 8:offset + 8 + length]
        offset += 8 + length
    doc = json.loads(chunks[b'JSON'])
    binary = bytearray(chunks[b'BIN\0'])
    authored = {material['name']: material for material in materials}
    replaced = {}
    for material in doc['materials']:
        source = authored[material['name']]
        if source['surface'] < 0:
            continue
        mr = material['pbrMetallicRoughness'].get('metallicRoughnessTexture')
        require(mr is not None, 'declared ORM has no exchange MR binding')
        texture = doc['textures'][mr['index']]
        image_index = texture['source']
        image = doc['images'][image_index]
        png = textures[source['surface']]['raw']
        if image_index in replaced:
            require(replaced[image_index] == png, 'conflicting shared exchange ORM image')
        else:
            binary.extend(b'\0' * (-len(binary) % 4))
            view = doc['bufferViews'][image['bufferView']]
            view['byteOffset'], view['byteLength'] = len(binary), len(png)
            binary.extend(png)
            image['mimeType'] = 'image/png'
            replaced[image_index] = png
        material['occlusionTexture'] = dict(mr)
        material['occlusionTexture']['strength'] = 1.0
    doc['buffers'][0]['byteLength'] = len(binary)
    binary.extend(b'\0' * (-len(binary) % 4))
    encoded = json.dumps(doc, separators=(',', ':'), ensure_ascii=False).encode('utf-8')
    encoded += b' ' * (-len(encoded) % 4)
    result = struct.pack('<4sII', b'glTF', 2, 28 + len(encoded) + len(binary))
    result += struct.pack('<I4s', len(encoded), b'JSON') + encoded
    result += struct.pack('<I4s', len(binary), b'BIN\0') + binary
    path.write_bytes(result)


def verify_glb(path, parts, bones, poses, require_uv, materials, textures):
    """Read actual GLB skin/inverse-bind data and check bind-world geometry."""
    raw = path.read_bytes()
    require(struct.unpack_from('<4sII', raw) == (b'glTF', 2, len(raw)), 'invalid exchange GLB')
    at, doc, binary = 12, None, None
    while at < len(raw):
        length, kind = struct.unpack_from('<I4s', raw, at)
        at += 8
        if kind == b'JSON':
            doc = json.loads(raw[at:at + length])
        elif kind == b'BIN\0':
            binary = raw[at:at + length]
        at += length
    require(doc is not None and binary is not None, 'GLB JSON/BIN missing')
    authored_materials = {material['name']: material for material in materials}
    orm_checked = 0
    for material in doc['materials']:
        source = authored_materials[material['name']]
        if source['surface'] < 0:
            continue
        mr = material['pbrMetallicRoughness']['metallicRoughnessTexture']
        ao = material.get('occlusionTexture')
        require(ao is not None and ao['index'] == mr['index'] and ao.get('texCoord', 0) == mr.get('texCoord', 0),
                'GLB AO/MR must share declared ORM texture and coordinates')
        require(ao.get('strength', 1) == 1, 'GLB AO strength changed')
        image = doc['images'][doc['textures'][mr['index']]['source']]
        view = doc['bufferViews'][image['bufferView']]
        start = view.get('byteOffset', 0)
        require(binary[start:start + view['byteLength']] == textures[source['surface']]['raw'],
                'actual GLB embedded ORM differs from declared PNG including R occlusion')
        orm_checked += 1
    def accessor(index):
        entry = doc['accessors'][index]
        view = doc['bufferViews'][entry['bufferView']]
        components = {'SCALAR': 1, 'VEC2': 2, 'VEC3': 3, 'VEC4': 4, 'MAT4': 16}[entry['type']]
        fmt = {5121: 'B', 5123: 'H', 5125: 'I', 5126: 'f'}[entry['componentType']]
        size = struct.calcsize('<' + fmt * components)
        offset = view.get('byteOffset', 0) + entry.get('byteOffset', 0)
        stride = view.get('byteStride', size)
        return [struct.unpack_from('<' + fmt * components, binary, offset + i * stride) for i in range(entry['count'])]
    from mathutils import Quaternion
    worlds = {}
    def walk(index, parent):
        node = doc['nodes'][index]
        if 'matrix' in node:
            values = node['matrix']
            local = Matrix([[values[c * 4 + r] for c in range(4)] for r in range(4)])
        else:
            q = node.get('rotation', [0, 0, 0, 1])
            local = Matrix.LocRotScale(Vector(node.get('translation', [0, 0, 0])),
                                       Quaternion((q[3], q[0], q[1], q[2])), Vector(node.get('scale', [1, 1, 1])))
        worlds[index] = parent @ local
        for child in node.get('children', []):
            walk(child, worlds[index])
    for root in doc['scenes'][doc.get('scene', 0)]['nodes']:
        walk(root, Matrix.Identity(4))
    sampled = [poses['poses'][i] for i in (0, 7, 11, 27)]
    sampled.extend(pose for pose in poses['poses'] if pose['track'] and pose['time'] > 0.35 and 'tick024' in pose['name'])
    rests = {bone['name']: Matrix(bone['source_rest_game']) for bone in bones}
    deltas = [{name: Matrix(pose['joints'][name]) @ rest.inverted() for name, rest in rests.items()}
              for pose in sampled]
    part_sources = {part['name']: [v for primitive in part['primitives'] for v in primitive['vertices']] for part in parts}
    source_vertices = [v for part in parts for primitive in part['primitives'] for v in primitive['vertices']]
    source = [vertex['position'] for vertex in source_vertices]
    tree = KDTree(len(source))
    for index, point in enumerate(source):
        tree.insert(Vector(point), index)
    tree.balance()
    maximum, uv_error, uv_count, count, pose_error, pose_count = 0.0, 0.0, 0, 0, 0.0, 0
    for node in doc['nodes']:
        if 'mesh' not in node:
            continue
        require('skin' in node, 'exchange character mesh lost skin')
        require(node['name'] in part_sources, 'GLB source object mapping lost')
        local_vertices = part_sources[node['name']]
        local_tree = KDTree(len(local_vertices))
        for i, vertex in enumerate(local_vertices):
            local_tree.insert(Vector(vertex['position']), i)
        local_tree.balance()
        skin = doc['skins'][node['skin']]
        names = [AUXILIARY_MAP.get(doc['nodes'][joint]['name'], doc['nodes'][joint]['name']) for joint in skin['joints']]
        require(all(name in rests or name == 'root' for name in names), 'unknown GLB joint mapping')
        binds = accessor(skin['inverseBindMatrices'])
        matrices = [worlds[joint] @ Matrix([[bind[c * 4 + r] for c in range(4)] for r in range(4)])
                    for joint, bind in zip(skin['joints'], binds)]
        for primitive in doc['meshes'][node['mesh']]['primitives']:
            attrs = primitive['attributes']
            uvs = accessor(attrs['TEXCOORD_0']) if 'TEXCOORD_0' in attrs else None
            for vertex_index, (point, indices, weights) in enumerate(zip(accessor(attrs['POSITION']), accessor(attrs['JOINTS_0']), accessor(attrs['WEIGHTS_0']))):
                value = Vector((0, 0, 0))
                for index, weight in zip(indices, weights):
                    value += (matrices[index] @ Vector(point)) * weight
                maximum = max(maximum, tree.find(value)[2])
                candidates = local_tree.find_range(value, 0.00001)
                require(candidates, f"GLB vertex missing from authored {node['name']}: nearest {local_tree.find(value)[2]}m")
                match = min(candidates, key=lambda item: math.dist(uvs[vertex_index], local_vertices[item[1]]['uv']) if uvs else item[2])
                authored = local_vertices[match[1]]
                if uvs:
                    uv_error = max(uv_error, math.dist(uvs[vertex_index], authored['uv']))
                    uv_count += 1
                for delta in deltas:
                    actual = Vector((0, 0, 0))
                    for joint, weight in zip(indices, weights):
                        if weight:
                            actual += (delta.get(names[joint], Matrix.Identity(4)) @ matrices[joint] @ Vector(point)) * weight
                    expected = Vector((0, 0, 0))
                    for joint, weight in zip(authored['joints'], authored['weights']):
                        if weight:
                            expected += (delta[bones[joint]['name']] @ Vector(authored['position'])) * (weight / 255)
                    pose_error = max(pose_error, (actual - expected).length)
                    pose_count += 1
                count += 1
    require(count > 0 and maximum < 0.00001, 'GLB own skin/IBM bind-world geometry proof failed')
    require(not require_uv or uv_count == count, 'GLB lost authored UVs')
    require(uv_count == 0 or uv_error < 0.00002, 'GLB/runtime UV convention mismatch')
    require(pose_error < 0.005, 'actual GLB/runtime posed skin proof failed')
    return {'glb_declared_orm_materials_checked': orm_checked, 'vertices_read_from_actual_glb': count, 'maximum_glb_bind_to_authored_geometry_error_m': maximum,
            'glb_uv_vertices_checked': uv_count, 'maximum_glb_runtime_uv_error': uv_error,
            'glb_posed_vertices_checked': pose_count, 'glb_compared_poses': [pose['name'] for pose in sampled],
            'maximum_glb_runtime_posed_lbs_error_m': pose_error}


def main():
    require(bpy.app.background, 'Run in an independent Blender --background process')
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--blend', type=Path, required=True)
    parser.add_argument('--reference', type=Path, required=True)
    parser.add_argument('--poses', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--runtime', type=Path, required=True)
    parser.add_argument('--triangle-budget', type=triangle_budget, default=40000,
                        help='Maximum evaluated full-character triangles, 1..250000 (default: 40000); explicit production ceiling, not a development bypass')
    parser.add_argument('--development-projected-uv', action='store_true')
    parser.add_argument('--development-material-factors', action='store_true', help='Development only: skip procedural texture links and export Principled factors')
    options = parser.parse_args(sys.argv[sys.argv.index('--') + 1:])
    bpy.ops.wm.open_mainfile(filepath=str(options.blend))
    reference = json.loads(options.reference.read_text(encoding='utf-8'))
    poses = json.loads(options.poses.read_text(encoding='utf-8'))
    require(poses['schema'] == 'kiva-runtime-palette-poses-v1', 'pose schema mismatch')
    rig = bpy.data.objects['Zhuque_v11_CompatibleRig']
    require(matrix_error(rig.matrix_world, Matrix.Identity(4)) < 1e-6, 'authoring rig world must be identity')
    for name, parent in AUXILIARY_MAP.items():
        require(rig.data.bones[name].parent.name == parent, 'auxiliary parent contract changed')
    reset(rig)
    objects = sorted([obj for obj in bpy.context.scene.objects if obj.type == 'MESH'
                      and any(mod.type == 'ARMATURE' and mod.object == rig for mod in obj.modifiers)], key=lambda obj: obj.name)
    require(objects, 'no authored character meshes')
    materials = sorted({slot.material for obj in objects for slot in obj.material_slots if slot.material}, key=lambda mat: mat.name)
    bones = [node for node in reference['nodes'] if node['name'] != 'root']
    require(len(bones) == 52, 'native body joint set changed')
    palette = {bone['name']: index for index, bone in enumerate(bones)}
    parts, missing_uv = mesh_records(objects, rig, palette, options.development_projected_uv)
    triangles = sum(len(primitive['indices']) // 3 for part in parts for primitive in part['primitives'])
    require(triangles <= options.triangle_budget,
            f'evaluated character triangles {triangles} exceed explicit budget {options.triangle_budget}')
    options.output.mkdir(parents=True, exist_ok=True)
    textures, materials = export_materials(materials, options.output / 'textures', options.development_material_factors)
    header = {'format': 'KIVAAV04', 'identity': 'Original Zhuque v11 character', 'height_m': 1.73,
              'source_blend_sha256': sha(options.blend), 'reference_sha256': sha(options.reference),
              'triangle_budget': options.triangle_budget,
              'pose_sha256': sha(options.poses), 'auxiliary_weight_collapse': AUXILIARY_MAP,
              'matrix_contract': 'sourceRest=original target R; OWN inverseBind=R^-1; geometry in game model space. No old IBM copied.',
              'development_projected_uv': missing_uv,
              'development_material_factors_only': options.development_material_factors,
              'limits': 'No cloth physics. Auxiliary cape bones intentionally collapse to actual Spine2 delta; authored auxiliary rig remains in GLB/source.'}
    candidate = options.output / 'zhuque.verified.bin'
    write_binary(candidate, header, reference, bones, textures, materials, parts)
    proof = verify_skin(objects, rig, reference, bones, parts, poses)
    reset(rig)
    bpy.ops.object.select_all(action='DESELECT')
    rig.select_set(True)
    for obj in objects:
        obj.parent = rig
        obj.select_set(True)
    bpy.context.view_layer.objects.active = rig
    glb = options.output / 'zhuque.glb'
    bpy.ops.export_scene.gltf(filepath=str(glb), export_format='GLB', use_selection=True,
                              export_skins=True, export_animations=False, export_yup=True, export_apply=True)
    bind_glb_occlusion(glb, materials, textures)
    glb_proof = verify_glb(glb, parts, bones, poses, not options.development_projected_uv, materials, textures)
    compact = {'schema': 'kiva-authored-mesh-v1', 'header': header, 'bones': bones,
               'materials': materials, 'parts': parts}
    (options.output / 'authored-mesh.json').write_text(json.dumps(compact, separators=(',', ':')) + '\n', encoding='utf-8')
    # Publish only after both proofs pass. Never expose a partial or failed
    # export to the user's running asset watcher.
    options.runtime.parent.mkdir(parents=True, exist_ok=True)
    candidate.replace(options.runtime)
    report = {'source': str(options.blend), 'source_sha256': sha(options.blend), 'runtime': str(options.runtime),
              'runtime_sha256': sha(options.runtime), 'glb_sha256': sha(glb), 'triangles': triangles, 'triangle_budget': options.triangle_budget,
              'parts': len(parts), 'materials': len(materials), 'textures': len(textures),
              'missing_source_uv_development_only': missing_uv, 'auxiliary_collapse': AUXILIARY_MAP,
              **proof, **glb_proof}
    (options.output / 'conversion-report.json').write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    print(json.dumps(report, ensure_ascii=False, indent=2))


if __name__ == '__main__':
    main()
