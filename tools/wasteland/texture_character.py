"""Create real packed UV atlases/PBR images from the refined character source.

Runs only in an independent background Blender. The three opaque materials
have unique packed UVs; the dedicated fire fringe intentionally repeats a tile.
Neither the greybox nor the refined source is overwritten.
"""
from __future__ import annotations

import argparse
import collections
import hashlib
import json
import math
import sys
from pathlib import Path

import bmesh
import bpy
import numpy as np
from mathutils import Matrix

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import build_greybox as builder

ROLES = ('skin', 'cloth', 'charcoal', 'fire_fringe')


def require(test, message):
    if not test:
        raise ValueError(message)


def select_only(objects):
    bpy.ops.object.mode_set(mode='OBJECT') if bpy.context.object and bpy.context.object.mode != 'OBJECT' else None
    bpy.ops.object.select_all(action='DESELECT')
    for obj in objects:
        obj.hide_set(False)
        obj.select_set(True)
    bpy.context.view_layer.objects.active = objects[0]


def assign_role(obj):
    name = obj.name.lower()
    if obj.get('kiva_surface_effect') == 1:
        return 3
    mat = obj.data.materials[0].name.lower()
    if 'skin' in mat:
        return 0
    if 'charcoal' in mat or ('hair' in name and 'ember' in name):
        return 2
    return 1  # Bird relief/old ember guides retain their source copper in bake.


def author_source_materials(character, fire):
    """Restrained actual bake sources, later replaced by PNG-only materials."""
    cache = {}
    for obj in character.objects:
        if obj.type != 'MESH' or obj == fire:
            continue
        name = obj.name.lower()
        original = obj.data.materials[0]
        base_role = assign_role(obj)
        if '.sclera.' in name:
            feature = 'sclera'
        elif '.iris.' in name:
            feature = 'iris'
        elif 'hair' in name and 'ember' in name:
            feature = 'static_hair_ember'
        elif name.startswith('refined hair'):
            feature = 'hair'
        elif base_role == 0:
            feature = 'skin'
        elif base_role == 2:
            feature = 'dark_cloth' if 'scarf' in name or 'cape' in name or 'sleeve' in name else 'dark_rubber'
        else:
            feature = 'cloth' if 'grey cloth' in original.name.lower() else 'copper_embroidery'
        key = (original.name, feature)
        if key not in cache:
            mat = original.copy()
            mat.name = original.name + '_bake_' + feature
            shader = mat.node_tree.nodes.get('Principled BSDF')
            shader.inputs['Metallic'].default_value = 0
            if feature == 'sclera':
                shader.inputs['Base Color'].default_value = (0.60, 0.56, 0.49, 1)
                shader.inputs['Roughness'].default_value = 0.38
            elif feature == 'iris':
                shader.inputs['Base Color'].default_value = (0.027, 0.016, 0.009, 1)
                shader.inputs['Roughness'].default_value = 0.35
            elif feature == 'static_hair_ember':
                shader.inputs['Emission Color'].default_value = (0.35, 0.045, 0.005, 1)
                shader.inputs['Emission Strength'].default_value = 1.0
            else:
                roughness = {'hair': 0.48, 'dark_cloth': 0.88, 'dark_rubber': 0.72, 'skin': 0.68,
                             'cloth': 0.86, 'copper_embroidery': 0.80}[feature]
                shader.inputs['Roughness'].default_value = roughness
                if feature in ('skin', 'cloth', 'dark_cloth', 'hair'):
                    base = tuple(shader.inputs['Base Color'].default_value)
                    if feature == 'skin':
                        base = (0.31, 0.225, 0.17, 1)
                    nodes, links = mat.node_tree.nodes, mat.node_tree.links
                    noise = nodes.new('ShaderNodeTexNoise')
                    noise.name = 'Actual atlas pigment and restrained wear source'
                    noise.inputs['Scale'].default_value = 11 if feature != 'skin' else 20
                    noise.inputs['Detail'].default_value = 3
                    geometry = nodes.new('ShaderNodeNewGeometry')
                    links.new(geometry.outputs['Position'], noise.inputs['Vector'])
                    ramp = nodes.new('ShaderNodeValToRGB')
                    low, high = (0.84, 1.10) if feature != 'skin' else (0.94, 1.05)
                    ramp.color_ramp.elements[0].color = tuple(c * low for c in base[:3]) + (1,)
                    ramp.color_ramp.elements[1].color = tuple(c * high for c in base[:3]) + (1,)
                    links.new(noise.outputs['Fac'], ramp.inputs['Fac'])
                    links.new(ramp.outputs['Color'], shader.inputs['Base Color'])
                    if feature == 'skin':
                        pores = nodes.new('ShaderNodeTexNoise')
                        pores.inputs['Scale'].default_value = 420
                        pores.inputs['Detail'].default_value = 2
                        links.new(geometry.outputs['Position'], pores.inputs['Vector'])
                        bump = nodes.new('ShaderNodeBump')
                        bump.inputs['Strength'].default_value = 0.12
                        bump.inputs['Distance'].default_value = 0.00008
                        links.new(pores.outputs['Fac'], bump.inputs['Height'])
                        links.new(bump.outputs['Normal'], shader.inputs['Normal'])
            cache[key] = mat
        obj.data.materials[0] = cache[key]


def fire_fringe(character, rig, cape=None):
    cape = cape or next(obj for obj in character.objects if obj.name.startswith('Broad asymmetric cape'))
    rows, cols = 37, 29
    vertices, faces, weights, uv = [], [], [], []
    groups = {group.index: group.name for group in cape.vertex_groups}

    def surface(row, col):
        # Bilinear surface sampling with controlled edge extrapolation: every
        # flame point follows the real cloth folds, not a straight chord that
        # disappears inside the opaque base cape.
        r = min(rows - 2, max(0, int(row)))
        c = min(cols - 2, max(0, int(col)))
        tr, tc = row - r, col - c
        a = cape.data.vertices[r * cols + c].co.lerp(cape.data.vertices[r * cols + c + 1].co, tc)
        b = cape.data.vertices[(r + 1) * cols + c].co.lerp(cape.data.vertices[(r + 1) * cols + c + 1].co, tc)
        return a.lerp(b, tr)

    for layer in (-1, 1):
        for lane in range(15):
            u = 0.025 + lane / 14 * 0.95
            col = u * (cols - 1)
            source_col = min(cols - 1, max(0, round(col)))
            top_row = 22 + lane % 4
            influence = {groups[g.group]: g.weight for g in cape.data.vertices[(rows - 1) * cols + source_col].groups}
            base = len(vertices)
            for row in range(8):
                t = row / 7
                source_row = top_row + (rows - 1 - top_row) * t + 0.8 * t ** 4
                du = surface(source_row, col + 0.1) - surface(source_row, col - 0.1)
                dv = surface(source_row + 0.1, col) - surface(source_row - 0.1, col)
                normal = du.cross(dv).normalized()
                width = (0.035 + 0.006 * math.sin(lane * 1.7) ** 2)
                width *= (0.22 + 0.78 * math.sin(math.pi * t)) ** 0.55 * (1 - t) ** 0.30
                width = max(width, 0.00035)
                units_per_column = du.length / 0.2
                drift = math.sin(t * math.pi * 1.5 + lane * 1.8) * 0.006 * t
                for k in range(4):
                    ss = k / 3
                    sample_col = col + ((ss * 2 - 1) * width + drift) / units_per_column
                    point = surface(source_row, sample_col)
                    point += normal * layer * (0.010 + 0.002 * math.sin(ss * math.pi))
                    vertices.append(point)
                    weights.append(influence)
                    uv.append((ss, t))
            for row in range(7):
                for k in range(3):
                    a = base + row * 4 + k
                    faces.append((a, a + 1, a + 5, a + 4))
    data = bpy.data.meshes.new('Dedicated surface-following front and back fire fringe')
    data.from_pydata(vertices, [], faces)
    data.update()
    obj = bpy.data.objects.new('Character_FireFringe_ONLY', data)
    character.objects.link(obj)
    layer = data.uv_layers.new(name='UVMap')
    for polygon in data.polygons:
        polygon.use_smooth = True
        for loop in polygon.loop_indices:
            layer.data[loop].uv = uv[data.loops[loop].vertex_index]
    vgroups = {}
    for index, values in enumerate(weights):
        total = sum(values.values())
        for name, weight in values.items():
            if name not in vgroups:
                vgroups[name] = obj.vertex_groups.new(name=name)
            vgroups[name].add([index], weight / total, 'REPLACE')
    modifier = obj.modifiers.new('Fire fringe own bind - inherits actual cape', 'ARMATURE')
    modifier.object = rig
    obj['kiva_surface_effect'] = 1
    obj['kiva_source_role'] = 'fire_fringe'
    obj['uv_contract'] = 'Intentional repeated tile per ribbon; V follows length. Front/back cloth-following layers, opaque base cape fixed.'
    return obj


def join_by_role(character, fire):
    objects = sorted([obj for obj in character.objects if obj.type == 'MESH'], key=lambda o: o.name)
    grouped = collections.defaultdict(list)
    provenance = {}
    for component, obj in enumerate(objects):
        role = 3 if obj == fire else assign_role(obj)
        attr = obj.data.attributes.get('kiva_component_id') or obj.data.attributes.new('kiva_component_id', 'INT', 'FACE')
        attr.data.foreach_set('value', [component] * len(obj.data.polygons))
        provenance[str(component)] = {'object': obj.name, 'role': ROLES[role], 'vertices': len(obj.data.vertices)}
        grouped[role].append(obj)
    result = {}
    for role in range(4):
        select_only(grouped[role])
        active = grouped[role][0]
        if len(grouped[role]) > 1:
            bpy.ops.object.join()
        active.name = 'Character_' + ROLES[role]
        active['kiva_source_role'] = ROLES[role]
        active['kiva_source_objects'] = json.dumps([item['object'] for item in provenance.values() if item['role'] == ROLES[role]])
        result[role] = active
    return result, provenance


def clean_degenerate(obj):
    bm = bmesh.new()
    bm.from_mesh(obj.data)
    bad = [face for face in bm.faces if face.calc_area() <= 1e-13]
    count = len(bad)
    if bad:
        bmesh.ops.delete(bm, geom=bad, context='FACES_ONLY')
    bmesh.ops.triangulate(bm, faces=list(bm.faces), quad_method='BEAUTY', ngon_method='BEAUTY')
    bm.to_mesh(obj.data)
    obj.data.update()
    bm.free()
    return count


def area2(a, b, c):
    return (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0])


def triangles_overlap(a, b):
    # Strict separating axes: shared edges/points are allowed, positive-area
    # triangle overlap is not. UVs are float32; tolerance is well below a pixel.
    for triangle in (a, b):
        for i in range(3):
            p, q = triangle[i], triangle[(i + 1) % 3]
            axis = (q[1] - p[1], p[0] - q[0])
            av = [x[0] * axis[0] + x[1] * axis[1] for x in a]
            bv = [x[0] * axis[0] + x[1] * axis[1] for x in b]
            if max(av) <= min(bv) + 1e-12 or max(bv) <= min(av) + 1e-12:
                return False
    return True


def uv_report(obj, repeat=False):
    data = obj.data
    require(data.uv_layers.active is not None, f'missing UV0: {obj.name}')
    data.calc_loop_triangles()
    uv = data.uv_layers.active.data
    triangles = []
    minimum = 1.0
    for tri in data.loop_triangles:
        points = [tuple(uv[loop].uv) for loop in tri.loops]
        require(all(math.isfinite(c) and -1e-6 <= c <= 1 + 1e-6 for p in points for c in p), 'UV outside normalized tile')
        area = abs(area2(*points)) / 2
        require(area > 1e-14, f'degenerate UV triangle: {obj.name}, triangle {tri.index}')
        minimum = min(minimum, area)
        triangles.append(points)
    if repeat:
        return {'triangles': len(triangles), 'min_uv_area': minimum, 'overlap': 'Intentional repeat per flame ribbon; exempt from opaque unique atlas rule'}
    grid = collections.defaultdict(list)
    overlaps = []
    for index, tri in enumerate(triangles):
        lo = [max(0, min(63, int(min(p[k] for p in tri) * 64))) for k in range(2)]
        hi = [max(0, min(63, int(max(p[k] for p in tri) * 64))) for k in range(2)]
        cells = [(x, y) for x in range(lo[0], hi[0] + 1) for y in range(lo[1], hi[1] + 1)]
        candidates = {other for cell in cells for other in grid[cell]}
        for other in candidates:
            if triangles_overlap(tri, triangles[other]):
                overlaps.append([other, index])
        for cell in cells:
            grid[cell].append(index)
    return {'triangles': len(triangles), 'min_uv_area': minimum, 'positive_area_overlap_pairs': len(overlaps), 'overlap_pairs': overlaps}


def isolate_conflicting_charts(obj, overlaps):
    data = obj.data
    data.calc_loop_triangles()
    polygons = sorted({data.loop_triangles[index].polygon_index for pair in overlaps for index in pair})
    layer = data.uv_layers.active.data
    for ordinal, index in enumerate(polygons):
        face = data.polygons[index]
        require(len(face.vertices) == 3, 'targeted chart requires triangulated source')
        points = [data.vertices[i].co for i in face.vertices]
        x = (points[1] - points[0]).normalized()
        normal = (points[1] - points[0]).cross(points[2] - points[0]).normalized()
        y = normal.cross(x)
        coords = [((p - points[0]).dot(x), (p - points[0]).dot(y)) for p in points]
        scale = 0.06 / max(abs(c) for point in coords for c in point)
        offset = (2 + ordinal % 10 * 0.2, 2 + ordinal // 10 * 0.2)
        for loop, point in zip(face.loop_indices, coords):
            layer[loop].uv = (offset[0] + point[0] * scale, offset[1] + point[1] * scale)
    select_only([obj])
    bpy.ops.object.mode_set(mode='EDIT')
    bpy.ops.mesh.select_all(action='SELECT')
    bpy.ops.uv.select_all(action='SELECT')
    bpy.ops.uv.pack_islands(rotate=True, scale=True, margin_method='FRACTION', margin=0.006,
                            merge_overlap=False, shape_method='CONCAVE')
    bpy.ops.object.mode_set(mode='OBJECT')
    return len(polygons)


def unwrap(obj):
    removed = clean_degenerate(obj)
    for angle in (45, 25, 12):
        select_only([obj])
        if not obj.data.uv_layers:
            obj.data.uv_layers.new(name='UVMap')
        bpy.ops.object.mode_set(mode='EDIT')
        bpy.ops.mesh.select_all(action='SELECT')
        bpy.ops.uv.smart_project(angle_limit=math.radians(angle), island_margin=0.006,
                                 margin_method='FRACTION', area_weight=0.5,
                                 correct_aspect=True, scale_to_bounds=True)
        bpy.ops.uv.select_all(action='SELECT')
        bpy.ops.uv.pack_islands(rotate=True, scale=True, margin_method='FRACTION', margin=0.006,
                                merge_overlap=False, shape_method='CONCAVE')
        bpy.ops.object.mode_set(mode='OBJECT')
        report = uv_report(obj)
        if report['positive_area_overlap_pairs'] == 0:
            return {'unwrap_angle_degrees': angle, 'removed_zero_area_source_faces': removed, **report}
    isolated = 0
    for _ in range(4):
        isolated += isolate_conflicting_charts(obj, report['overlap_pairs'])
        report = uv_report(obj)
        if report['positive_area_overlap_pairs'] == 0:
            return {'unwrap_angle_degrees': angle, 'removed_zero_area_source_faces': removed,
                    'isolated_triangle_charts': isolated, **report}
    raise ValueError(f'UV overlap persists after independent packing: {obj.name}: {report}')


def image(name, size, color=(0, 0, 0, 1), srgb=False):
    result = bpy.data.images.new(name, width=size, height=size, alpha=True)
    result.generated_color = color
    result.colorspace_settings.name = 'sRGB' if srgb else 'Non-Color'
    return result


def save_image(img, path):
    img.filepath_raw = str(path)
    img.file_format = 'PNG'
    img.save()
    img.filepath = '//textures/' + path.name
    img.pack()


def bake(obj, kind, size, name, texture_dir):
    print(f'BAKE {obj.name} {kind} {size}x{size}', flush=True)
    img = image(name, size, (0.5, 0.5, 1, 1) if kind == 'NORMAL' else (0, 0, 0, 1), kind in ('DIFFUSE', 'EMIT'))
    for mat in obj.data.materials:
        node = mat.node_tree.nodes.new('ShaderNodeTexImage')
        node.name = 'Active real atlas bake target'
        node.image = img
        mat.node_tree.nodes.active = node
    select_only([obj])
    kwargs = {'type': kind, 'use_clear': True, 'margin': 12, 'margin_type': 'EXTEND', 'uv_layer': 'UVMap'}
    if kind == 'DIFFUSE':
        kwargs['pass_filter'] = {'COLOR'}
    if kind == 'NORMAL':
        kwargs['normal_space'] = 'TANGENT'
    bpy.ops.object.bake(**kwargs)
    save_image(img, texture_dir / (name + '.png'))
    return img


def pixels(img):
    array = np.empty(img.size[0] * img.size[1] * 4, dtype=np.float32)
    img.pixels.foreach_get(array)
    return array.reshape((-1, 4))


def pack_orm(ao, roughness, size, name, directory):
    values = np.ones((size * size, 4), dtype=np.float32)
    values[:, 0] = pixels(ao)[:, 0]
    values[:, 1] = pixels(roughness)[:, 0]
    values[:, 2] = 0  # All current source shaders explicitly have metallic=0.
    result = image(name, size)
    result.pixels.foreach_set(values.ravel())
    save_image(result, directory / (name + '.png'))
    return result


def final_material(role, maps, emission_strength=1):
    mat = bpy.data.materials.new('Zhuque_' + ROLES[role] + '_PBR')
    mat.use_nodes = True
    shader = mat.node_tree.nodes.get('Principled BSDF')
    shader.inputs['Emission Strength'].default_value = emission_strength
    nodes, links = mat.node_tree.nodes, mat.node_tree.links
    made = {}
    for key, img in maps.items():
        node = nodes.new('ShaderNodeTexImage')
        node.name = 'Direct_' + key
        node.image = img
        made[key] = node
    links.new(made['base'].outputs['Color'], shader.inputs['Base Color'])
    normal = nodes.new('ShaderNodeNormalMap')
    links.new(made['normal'].outputs['Color'], normal.inputs['Color'])
    links.new(normal.outputs['Normal'], shader.inputs['Normal'])
    split = nodes.new('ShaderNodeSeparateColor')
    links.new(made['orm'].outputs['Color'], split.inputs['Color'])
    links.new(split.outputs['Green'], shader.inputs['Roughness'])
    links.new(split.outputs['Blue'], shader.inputs['Metallic'])
    links.new(made['emission'].outputs['Color'], shader.inputs['Emission Color'])
    mat['kiva_orm_image'] = maps['orm'].name
    mat['kiva_occlusion_image'] = maps['orm'].name
    mat['kiva_surface_effect'] = 1 if role == 3 else 0
    mat['kiva_alpha_mode'] = 2 if role == 3 else 0
    mat['kiva_source_role'] = ROLES[role]
    mat.use_backface_culling = False
    if role == 3:
        links.new(made['base'].outputs['Alpha'], shader.inputs['Alpha'])
        mat['kiva_opacity'] = 1.0
    return mat


def fire_maps(size, directory, only_colors=False):
    y, x = np.mgrid[0:size, 0:size].astype(np.float32)
    u, v = x / (size - 1), y / size
    width = np.clip(1 - np.abs(u * 2 - 1), 0, 1) ** 0.7
    wave = np.clip(0.50 + 0.23 * np.sin(u * 9 + 1.6 * np.sin(v * 2 * PI))
                   + 0.17 * np.cos(u * 17 - v * 2 * PI) + 0.10 * np.sin(u * 25 + v * 4 * PI), 0, 1)
    core = np.clip((0.5 + 0.5 * np.sin(u * 18 + 2.5 * np.sin(v * 2 * PI)) - 0.78) * 4.5, 0, 1)
    alpha = np.clip(0.30 + wave * 0.85, 0, 1) * np.sqrt(width)
    color = np.empty((size, size, 4), dtype=np.float32)
    color[:, :, 0] = 0.62 + wave * 0.38
    color[:, :, 1] = 0.012 + 0.32 * wave ** 1.3 + 0.55 * core ** 1.5 * width
    color[:, :, 2] = 0.001 + 0.08 * core ** 2 * width
    color[:, :, 3] = alpha
    result = {}
    for name in ('base', 'emission'):
        img = image('fire_fringe_' + name, size, srgb=True)
        img.pixels.foreach_set(color.ravel())
        save_image(img, directory / (img.name + '.png'))
        result[name] = img
    if only_colors:
        return result
    for name, value in (('normal', (0.5, 0.5, 1, 1)), ('orm', (1, 0.85, 0, 1))):
        img = image('fire_fringe_' + name, size, value)
        save_image(img, directory / (img.name + '.png'))
        result[name] = img
    return result


def main():
    require(bpy.app.background, 'Use an independent background Blender process')
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--resolution', type=int, default=2048)
    parser.add_argument('--samples', type=int, default=8)
    parser.add_argument('--unwrap-only', action='store_true')
    parser.add_argument('--poses', type=Path)
    options = parser.parse_args(sys.argv[sys.argv.index('--') + 1:])
    require(options.output.resolve() != options.source.parent.resolve(), 'Use a separate textured output directory')
    options.output.mkdir(parents=True, exist_ok=True)
    texture_dir = options.output / 'textures'
    texture_dir.mkdir(exist_ok=True)
    bpy.ops.wm.open_mainfile(filepath=str(options.source))
    rig = bpy.data.objects['Zhuque_v11_CompatibleRig']
    for bone in rig.pose.bones:
        bone.matrix_basis = Matrix.Identity(4)
    bpy.context.view_layer.update()
    character = bpy.data.collections['ORIGINAL Zhuque v11 refined']
    character.name = 'Zhuque textured character - four surface roles'
    fire = fire_fringe(character, rig)
    author_source_materials(character, fire)
    objects, source_map = join_by_role(character, fire)
    reports = {}
    for role, obj in objects.items():
        reports[ROLES[role]] = uv_report(obj, True) if role == 3 else unwrap(obj)
        print(json.dumps({'uv_role': ROLES[role], 'report': reports[ROLES[role]]}), flush=True)
    triangles = sum(sum(len(face.vertices) - 2 for face in obj.data.polygons) for obj in objects.values())
    require(triangles <= 40000, f'textured character with fringe exceeds budget: {triangles}')
    report = {'stage': 'Textured authoring, not full game acceptance', 'source': options.source.name,
              'source_sha256': hashlib.sha256(options.source.read_bytes()).hexdigest(),
              'character_triangles': triangles, 'material_roles': list(ROLES), 'texture_resolution': options.resolution,
              'uv': reports, 'source_components': source_map,
              'fire_uv_exception': 'Dedicated surface3 only: intentional repeated U/V tile with V along ribbon length; opaque base cape UV is packed and fixed.'}
    (options.output / 'uv-report.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    bpy.ops.wm.save_as_mainfile(filepath=str(options.output / 'zhuque-v11-uv.blend'), check_existing=False)
    if options.unwrap_only:
        return
    scene = bpy.context.scene
    scene.render.engine = 'CYCLES'
    scene.cycles.samples = options.samples
    scene.render.bake.use_selected_to_active = False
    scene.render.bake.use_pass_direct = False
    scene.render.bake.use_pass_indirect = False
    scene.render.bake.use_pass_color = True
    fire.hide_render = True
    floor = bpy.data.objects.get('Studio ground - not character')
    if floor:
        floor.hide_render = True
    for role in range(3):
        obj = objects[role]
        stem = ROLES[role]
        maps = {key: bake(obj, kind, options.resolution, stem + '_' + key, texture_dir)
                for key, kind in (('base', 'DIFFUSE'), ('normal', 'NORMAL'), ('emission', 'EMIT'))}
        ao = bake(obj, 'AO', options.resolution, stem + '_ao_work', texture_dir)
        roughness = bake(obj, 'ROUGHNESS', options.resolution, stem + '_roughness_work', texture_dir)
        maps['orm'] = pack_orm(ao, roughness, options.resolution, stem + '_orm', texture_dir)
        for temporary in (ao, roughness):
            temporary_file = texture_dir / (temporary.name + '.png')
            bpy.data.images.remove(temporary, do_unlink=True)
            temporary_file.unlink(missing_ok=True)
        mat = final_material(role, maps)
        obj.data.materials.clear()
        obj.data.materials.append(mat)
        for face in obj.data.polygons:
            face.material_index = 0
        print('Finished actual PNG atlas role: ' + stem, flush=True)
    fire.data.materials.clear()
    fire.data.materials.append(final_material(3, fire_maps(options.resolution, texture_dir), 5.5))
    fire.hide_render = False
    if floor:
        floor.hide_render = False
    report['textures'] = [{'file': path.name, 'sha256': hashlib.sha256(path.read_bytes()).hexdigest()}
                          for path in sorted(texture_dir.glob('*.png')) if '_work' not in path.name]
    report['bake_contract'] = 'Actual Blender COLOR/NORMAL/EMIT/AO/ROUGHNESS bakes; ORM R=AO/G=roughness/B=0 verified nonmetal source. Fire tile authored periodic-V, flat tangent-normal.'
    (options.output / 'texture-report.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    bpy.ops.wm.save_as_mainfile(filepath=str(options.output / 'zhuque-v11-textured.blend'), check_existing=False)
    if options.poses:
        builder.RIG = rig
        builder.MESHES = list(objects.values())
        metadata = json.loads(bpy.data.texts['reference-rig.json'].as_string())
        builder.NODES = {node['name']: node for node in metadata['nodes']}
        builder.reset_pose()
        builder.render(options.output, 'textured-front')
        builder.relaxed_review_pose()
        builder.render(options.output, 'textured-relaxed-review', -math.pi / 7)
        builder.reset_pose()
        data = json.loads(options.poses.read_text(encoding='utf-8'))
        for pose in data['poses']:
            if pose['name'] == 'HammerMusket:PlayerMelee { pair: HammerMusket, stage: One }:tick024':
                builder.apply_runtime_pose(data, pose)
                builder.render(options.output, 'textured-runtime-hammer-attack', -math.pi / 7)
                break
        builder.reset_pose()
    print(json.dumps({'triangles': triangles, 'role_count': 4, 'output': str(options.output)}), flush=True)


PI = math.pi

if __name__ == '__main__':
    main()
