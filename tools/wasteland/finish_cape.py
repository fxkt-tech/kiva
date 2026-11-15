"""Finish only the rebuilt cape with direct PNG cloth maps and matched fire edges.

Keeps separate editable high-detail and control-cage runtime-source files. Source
body geometry is never modified; authoring and runtime selection are explicit.
"""
from __future__ import annotations

import argparse
import json
import math
import sys
from pathlib import Path

import bpy
import numpy as np
from mathutils import Vector
from mathutils.bvhtree import BVHTree

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import build_greybox as builder
import refine_cape as cape
import texture_character as textures
from review_cape import normalize_cape_materials

PI = math.pi


def cloth_maps(directory, size=2048):
    """Actual authored atlas pixels, not live procedural nodes or projected UV."""
    directory.mkdir(exist_ok=True, parents=True)
    yy, xx = np.mgrid[0:size, 0:size].astype(np.float32)
    u, v = xx / (size - 1), yy / (size - 1)
    rgba = np.ones((size, size, 4), dtype=np.float32)
    pigment = 0.88 + .055 * np.sin(9 * u + 4 * v) + .04 * np.sin(31 * u - 17 * v) * np.sin(11 * v + 6 * u)
    pigment += .018 * np.sin(141 * u + np.sin(29 * v)) * np.sin(119 * v)
    # Geometric cloth panels own different atlas rectangles. Only paired
    # Solidify inner faces share texels; no differential AO is baked over them.
    main = (u < .66).astype(np.float32)
    cu = np.clip((u - .02) / .62, 0, 1)
    cv = np.clip((v - .02) / .96, 0, 1)
    su = np.clip((u - .68) / .30, 0, 1)
    sv = np.where(v < .5, (v - .02) / .45, (v - .52) / .46)
    edge = main * np.exp(-np.minimum(cu, 1 - cu) / .012)
    edge += (1 - main) * (np.exp(-np.minimum(su, 1 - su) / .02) + np.exp(-np.abs(sv - 1) / .035))
    warp = np.sin(2 * PI * (u * 220 + .18 * np.sin(v * 87)))
    weft = np.sin(2 * PI * (v * 420 + .12 * np.sin(u * 101)))
    weave = .5 + .5 * warp * weft
    burn = main * np.clip((cv - .78) / .22, 0, 1) ** 1.7
    # Charcoal cloth, restrained abrasion at real cut/hem UV boundaries.
    ridge = np.exp(-((cu - (.24 + .17 * cv - .10 * np.sin(PI * cv))) / .020) ** 2)
    ridge += .7 * np.exp(-((cu - (.73 - .08 * cv + .075 * np.sin(PI * cv))) / .024) ** 2)
    fibers = np.exp(-(np.sin(180 * u + 9 * v + 2 * np.sin(13 * v)) / .14) ** 2)
    fibers *= .4 + .6 * np.sin(19 * v + 7 * u) ** 2
    base = np.maximum(.004, .0185 * pigment + .0011 * weave + .0032 * edge + .0028 * ridge * main + .0018 * fibers - .009 * burn)
    for channel, multiplier in enumerate((1.00, .96, .92)):
        linear = base * multiplier
        rgba[:, :, channel] = np.where(linear <= .0031308, linear * 12.92, 1.055 * linear ** (1 / 2.4) - .055)
    result = {}
    img = textures.image('cape_detail_base', size, srgb=True)
    img.pixels.foreach_set(rgba.ravel())
    textures.save_image(img, directory / 'cape_detail_base.png')
    result['base'] = img
    # Tangent detail from crossing warp/weft. Macro folds remain actual mesh.
    nx = (.12 * np.cos(2 * PI * u * 220) + .035 * np.cos(2 * PI * u * 620)) * (.40 + .60 * (.5 + .5 * weft))
    ny = (.10 * np.cos(2 * PI * v * 420) + .03 * np.cos(2 * PI * v * 980)) * (.40 + .60 * (.5 + .5 * warp))
    nz = np.ones_like(nx)
    length = np.sqrt(nx * nx + ny * ny + nz * nz)
    rgba[:, :, 0] = nx / length * .5 + .5
    rgba[:, :, 1] = ny / length * .5 + .5
    rgba[:, :, 2] = nz / length * .5 + .5
    img = textures.image('cape_detail_normal', size)
    img.pixels.foreach_set(rgba.ravel())
    textures.save_image(img, directory / 'cape_detail_normal.png')
    result['normal'] = img
    rgba[:, :, 0] = 1  # Explicit unoccluded AO, not a claimed geometry bake.
    rgba[:, :, 1] = np.clip(.84 + .07 * burn - .06 * edge + .035 * weave + .025 * np.sin(u * 29 + v * 11), .70, .96)
    rgba[:, :, 2] = 0
    img = textures.image('cape_detail_orm', size)
    img.pixels.foreach_set(rgba.ravel())
    textures.save_image(img, directory / 'cape_detail_orm.png')
    result['orm'] = img
    rgba[:, :, :3] = 0
    # A few quiet scorch fissures live in the cloth; the dedicated edge below
    # carries active fire. The base cloth is opaque and its UV never scrolls.
    vein = np.exp(-((np.sin(cu * 37 + 2 * np.sin(cv * 7))) / .14) ** 2)
    glow = main * np.clip((cv - .84) / .16, 0, 1) ** 3 * vein * .025
    rgba[:, :, 0], rgba[:, :, 1], rgba[:, :, 2] = glow, glow * .055, glow * .006
    img = textures.image('cape_detail_emission', size, srgb=True)
    img.pixels.foreach_set(rgba.ravel())
    textures.save_image(img, directory / 'cape_detail_emission.png')
    result['emission'] = img
    # Generated float previews and exported PNG sampling must use identical
    # color-space data. Reload the actual files before assigning shader nodes.
    for key, generated in list(result.items()):
        loaded = bpy.data.images.load(str(directory / ('cape_detail_' + key + '.png')), check_existing=False)
        loaded.name = 'CapeDetail_FILE_' + key
        loaded.colorspace_settings.name = 'sRGB' if key in ('base', 'emission') else 'Non-Color'
        loaded.filepath = '//textures/cape_detail_' + key + '.png'
        loaded.pack()
        result[key] = loaded
        bpy.data.images.remove(generated, do_unlink=True)
    return result


def cloth_material(maps):
    mat = bpy.data.materials.new('Zhuque_cape_detail_PBR')
    mat.use_nodes = True
    shader = mat.node_tree.nodes.get('Principled BSDF')
    shader.inputs['Roughness'].default_value = .88
    shader.inputs['Sheen Weight'].default_value = 0
    shader.inputs['Sheen Roughness'].default_value = .90
    shader.inputs['Emission Strength'].default_value = 1.0
    made = {}
    for key, img in maps.items():
        node = mat.node_tree.nodes.new('ShaderNodeTexImage')
        node.image = img
        node.name = 'Direct_' + key
        made[key] = node
    links = mat.node_tree.links
    links.new(made['base'].outputs['Color'], shader.inputs['Base Color'])
    normal = mat.node_tree.nodes.new('ShaderNodeNormalMap')
    normal.inputs['Strength'].default_value = 1.0
    links.new(made['normal'].outputs['Color'], normal.inputs['Color'])
    links.new(normal.outputs['Normal'], shader.inputs['Normal'])
    split = mat.node_tree.nodes.new('ShaderNodeSeparateColor')
    links.new(made['orm'].outputs['Color'], split.inputs['Color'])
    links.new(split.outputs['Green'], shader.inputs['Roughness'])
    links.new(split.outputs['Blue'], shader.inputs['Metallic'])
    links.new(made['emission'].outputs['Color'], shader.inputs['Emission Color'])
    mat['kiva_orm_image'] = maps['orm'].name
    mat['kiva_occlusion_image'] = maps['orm'].name
    mat['kiva_surface_effect'] = 0
    mat['kiva_alpha_mode'] = 0
    mat['kiva_source_role'] = 'cape_detail'
    mat['texture_contract'] = 'Authored 2K PNG cloth pigment, weave normal, roughness, zero metal; ORM.R=1 explicitly unoccluded, not geometry AO bake.'
    mat.use_backface_culling = False
    return mat


def point(u, t):
    x, y, z = cape.cape(u, t)
    return Vector((-x, y, z))


def fire_edges(collection, rig, mat):
    vertices, faces, uv, weights = [], [], [], []
    # A contour-following ember rim plus four unequal tapered flame tongues.
    # All are surface-following on both sides and leave the new broad folds visible.
    def band(u0, u1, top, bottom, cols, rows, side, tongue=False):
        base = len(vertices)
        for j in range(rows + 1):
            s = j / rows
            t = top * (1 - s) + bottom * s
            taper = max(.018, (1 - s) ** .55) if tongue else 1
            mid = (u0 + u1) / 2
            for i in range(cols + 1):
                along = i / cols
                u = mid + ((u0 * (1 - along) + u1 * along) - mid) * taper
                pos = point(u, t)
                du = point(u + .0001, t) - point(u - .0001, t)
                dt = point(u, t + .0001) - point(u, t - .0001)
                normal = du.cross(dt).normalized()
                pos += normal * side * .005
                vertices.append(builder.B @ pos)
                uv.append((along, s))
                weights.append({('cape_R' if u < .32 else ('cape_L' if u > .71 else 'cape_C')): 1})
        for j in range(rows):
            for i in range(cols):
                a = base + j * (cols + 1) + i
                faces.append((a, a + 1, a + cols + 2, a + cols + 1))
    for side in (-1, 1):
        for start, end in ((0, .19), (.19, .465), (.465, .71), (.71, 1)):
            band(start, end, .765, 1.012, 40, 10, side)
        for u, span, top in ((.075, .060, .70), (.32, .058, .75), (.58, .060, .72), (.83, .047, .78)):
            band(u - span, u + span, top, 1.035, 5, 12, side, True)
    mesh = bpy.data.meshes.new('Cape 02 matched fire rim and feather tongues')
    mesh.from_pydata(vertices, [], faces)
    mesh.update()
    obj = bpy.data.objects.new('Cape 02 - contour matched fire ONLY', mesh)
    collection.objects.link(obj)
    mesh.materials.append(mat)
    layer = mesh.uv_layers.new(name='UVMap')
    for poly in mesh.polygons:
        poly.use_smooth = True
        for loop in poly.loop_indices:
            layer.data[loop].uv = uv[mesh.loops[loop].vertex_index]
    for i, values in enumerate(weights):
        for bone, weight in values.items():
            group = obj.vertex_groups.get(bone) or obj.vertex_groups.new(name=bone)
            group.add([i], weight, 'REPLACE')
    modifier = obj.modifiers.new('Own cape fire bind', 'ARMATURE')
    modifier.object = rig
    obj['kiva_surface_effect'] = 1
    obj['kiva_source_role'] = 'fire_fringe'
    obj['uv_contract'] = 'Intentional repeated fire tile per contour segment/tongue; V follows outward root-to-tip.'
    return obj


def evaluated(obj):
    graph = bpy.context.evaluated_depsgraph_get()
    result = obj.evaluated_get(graph)
    return result, result.to_mesh(preserve_all_data_layers=True, depsgraph=graph)


def high_runtime_distance(objects):
    result = []
    for obj in objects:
        mods = [m for m in obj.modifiers if m.type == 'SUBSURF']
        for m in mods:
            m.levels, m.render_levels = 0, 0
        bpy.context.view_layer.update()
        low, mesh = evaluated(obj)
        tree = BVHTree.FromPolygons([v.co for v in mesh.vertices], [tuple(p.vertices) for p in mesh.polygons])
        low_count = sum(len(p.vertices)-2 for p in mesh.polygons)
        low.to_mesh_clear()
        for m in mods:
            m.levels, m.render_levels = 1, 1
        bpy.context.view_layer.update()
        high, mesh = evaluated(obj)
        distances = [tree.find_nearest(v.co)[3] for v in mesh.vertices]
        result.append({'object': obj.name, 'runtime_triangles': low_count,
                       'high_triangles': sum(len(p.vertices)-2 for p in mesh.polygons),
                       'max_high_vertex_to_runtime_surface_m': max(distances),
                       'rms_high_vertex_to_runtime_surface_m': math.sqrt(sum(v*v for v in distances)/len(distances))})
        high.to_mesh_clear()
    return result


def skin_proof(objects, rig, pose_data):
    builder.reset_pose()
    bind = {}
    for obj in objects:
        mod = next(m for m in obj.modifiers if m.type == 'ARMATURE')
        mod.show_viewport = False
        bpy.context.view_layer.update()
        evaluated_obj, mesh = evaluated(obj)
        slots = {g.index: g.name for g in obj.vertex_groups}
        bind[obj.name] = [(v.co.copy(), [(slots[g.group], g.weight) for g in v.groups if g.weight > 1e-8]) for v in mesh.vertices]
        evaluated_obj.to_mesh_clear()
        mod.show_viewport = True
    rows = []
    names = ('HammerMusket:game:run:0.50', 'HammerMusket:PlayerMelee { pair: HammerMusket, stage: One }:tick024',
             'ChainAxeBow:game:crouch:0.50')
    for name in names:
        pose = next(p for p in pose_data['poses'] if p['name'] == name)
        builder.apply_runtime_pose(pose_data, pose)
        transforms = {b.name: b.matrix @ rig.data.bones[b.name].matrix_local.inverted() for b in rig.pose.bones}
        maximum, count = 0.0, 0
        for obj in objects:
            eobj, mesh = evaluated(obj)
            cape.require(len(mesh.vertices) == len(bind[obj.name]), 'pose changed cloth topology')
            for actual, (point, groups) in zip(mesh.vertices, bind[obj.name]):
                expected = sum((transforms[b] @ point * weight for b, weight in groups), Vector())
                maximum = max(maximum, (actual.co - expected).length)
                count += 1
            eobj.to_mesh_clear()
        cape.require(maximum < 3e-6, 'actual cloth own-bind LBS mismatch')
        rows.append({'pose': name, 'actual_evaluated_vertices': count, 'max_lbs_error_m': maximum})
    builder.reset_pose()
    return rows


def main():
    cape.require(bpy.app.background, 'Independent background Blender required')
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path, required=True)
    parser.add_argument('--original', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--poses', type=Path, required=True)
    parser.add_argument('--skip-renders', action='store_true')
    opts = parser.parse_args(sys.argv[sys.argv.index('--') + 1:])
    opts.output.mkdir(exist_ok=True, parents=True)
    bpy.ops.wm.open_mainfile(filepath=str(opts.source))
    rig = bpy.data.objects['Zhuque_v11_CompatibleRig']
    builder.RIG = rig
    reference = json.loads(bpy.data.texts['reference-rig.json'].as_string())
    builder.NODES = {n['name']: n for n in reference['nodes']}
    builder.reset_pose()
    objects = sorted([o for o in bpy.context.scene.objects if o.type == 'MESH' and o.get('kiva_source_role') == 'cape_detail'], key=lambda o:o.name)
    cape.require(len(objects) == 3, 'expected three new cape cloth panels')
    maps = cloth_maps(opts.output / 'textures')
    mat = cloth_material(maps)
    for obj in objects:
        obj.data.materials.clear()
        obj.data.materials.append(mat)
    fire_mat = next((m for m in bpy.data.materials if m.get('kiva_surface_effect') == 1), None)
    if fire_mat is None:
        with bpy.data.libraries.load(str(opts.original), link=False) as (available, loaded):
            loaded.materials = [name for name in available.materials if 'fire_fringe' in name.lower()]
        fire_mat = next((m for m in loaded.materials if m and m.get('kiva_surface_effect') == 1), None)
    cape.require(fire_mat is not None, 'original dedicated fire material missing')
    for old in list(bpy.context.scene.objects):
        if old.type == 'MESH' and old.get('kiva_source_role') == 'fire_fringe':
            bpy.data.objects.remove(old, do_unlink=True)
    fire_mat = fire_mat.copy()
    fire_mat.name = 'Zhuque_cape_fire_PBR'
    for node in fire_mat.node_tree.nodes:
        if node.type == 'TEX_IMAGE' and node.name.startswith('Direct_'):
            key = node.name[len('Direct_'):]
            path = opts.original.parent / 'textures' / ('fire_fringe_' + key + '.png')
            loaded = bpy.data.images.load(str(path), check_existing=False)
            loaded.colorspace_settings.name = 'sRGB' if key in ('base', 'emission') else 'Non-Color'
            loaded.pack()
            node.image = loaded
            if key == 'orm':
                fire_mat['kiva_orm_image'] = loaded.name
                fire_mat['kiva_occlusion_image'] = loaded.name
    fire = fire_edges(objects[0].users_collection[0], rig, fire_mat)
    normalize_cape_materials()
    shapes = high_runtime_distance(objects)
    print('HIGH/CONTROL-CAGE DISTANCE COMPLETE', json.dumps(shapes), flush=True)
    # Actual skin proof on the precise runtime-source geometry, not just bone matrices.
    for obj in objects:
        for m in obj.modifiers:
            if m.type == 'SUBSURF':
                m.levels = m.render_levels = 0
    bpy.context.view_layer.update()
    poses = json.loads(opts.poses.read_text())
    proof = skin_proof(objects + [fire], rig, poses)
    print('ACTUAL RUNTIME-SOURCE CLOTH LBS PROOF COMPLETE', json.dumps(proof), flush=True)
    original_map = json.loads((opts.original.parent / 'texture-report.json').read_text())['source_components']
    body = [o for o in bpy.context.scene.objects if o.type == 'MESH' and o.data.attributes.get('kiva_component_id')]
    current = cape.fingerprints(body, original_map, set())
    report = json.loads((opts.output / 'cape-report.json').read_text())
    cape.require(current == report['preserved_components'], 'finishing changed original character values')
    runtime = opts.output / 'zhuque-cape-runtime-source.blend'
    bpy.ops.wm.save_as_mainfile(filepath=str(runtime), check_existing=False)
    runtime_counts = cape.counts(objects + [fire])
    for obj in objects:
        for m in obj.modifiers:
            if m.type == 'SUBSURF':
                m.levels = m.render_levels = 1
    bpy.context.view_layer.update()
    author = opts.output / 'zhuque-cape-detail.blend'
    bpy.ops.wm.save_as_mainfile(filepath=str(author), check_existing=False)
    report.update({'stage': 'Detailed cape authoring and separate runtime-source; not yet published runtime',
                   'source_original_sha256': cape.sha(opts.original), 'author_source_sha256': cape.sha(author),
                   'runtime_source_sha256': cape.sha(runtime), 'runtime_cape': runtime_counts,
                   'runtime_full_triangles': report['unchanged_character_triangles'] + sum(r['evaluated_triangles'] for r in runtime_counts),
                   'fire_triangles': sum(r['evaluated_triangles'] for r in runtime_counts if r['object'] == fire.name),
                   'high_full_triangles': report['unchanged_character_triangles'] + report['cape_evaluated_triangles'] + sum(r['evaluated_triangles'] for r in runtime_counts if r['object'] == fire.name),
                   'shape_comparison': shapes, 'runtime_source_actual_skin_proof': proof,
                   'textures': [{'file': p.name, 'sha256': cape.sha(p), 'size': [2048,2048]} for p in sorted((opts.output/'textures').glob('*.png'))],
                   'material_notes': 'Independent actual FILE/packed 2K PNG cape base/normal/ORM/emission. sRGB base encoded from linear ~0.015–0.025 cloth, normal Strength1, no unsupported sheen preview. ORM.R=1 unoccluded AO; no differential AO bake. Existing body3 materials/12PNGs untouched; dedicated fire reuses exact prior4PNG bytes with new coverage.',
                   'uv_notes': 'Three cloth panel rectangles do not overlap each other. Paired Solidify inner/outer and rims intentionally share same undifferentiated fabric texels. No claim of globally unique cape atlas; no differential AO baked into shared regions.',
                   'orientation': 'Anatomical right shoulder/game+X; front image left, back image right long tail.',
                   'cowl_connection': 'Lower cowl intentionally tucks into existing collar under the shoulder/main cloth; source is not globally intersection-free. Inspect actual neck and side views.'})
    report['full_character_evaluated_triangles'] = report['high_full_triangles']
    (opts.output/'cape-report.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
    print('FINAL SOURCES SAVED', json.dumps({'author':str(author),'runtime':str(runtime),'runtime_triangles':report['runtime_full_triangles']}), flush=True)
    if opts.skip_renders:
        return
    cape.render(opts.output/'cape-final-back.png',PI)
    cape.render(opts.output/'cape-final-front.png',0)
    cape.render(opts.output/'cape-final-side.png',PI/2)
    cape.render(opts.output/'cape-final-back-three-quarter.png',PI*.78)
    cape.render(opts.output/'cape-final-neck.png',-.28,(0,0,1.39),.56,(1100,1000))
    cape.render(opts.output/'cape-final-hem.png',PI*.92,(.10,-.22,.46),.75,(1100,1000))
    builder.apply_runtime_pose(poses,next(p for p in poses['poses'] if 'HammerMusket:PlayerMelee' in p['name'] and 'tick024' in p['name']))
    cape.render(opts.output/'cape-runtime-hammer-attack.png',-PI/7)
    builder.apply_runtime_pose(poses,next(p for p in poses['poses'] if p['name']=='ChainAxeBow:game:crouch:0.50'))
    cape.render(opts.output/'cape-runtime-crouch.png',-PI/7)
    builder.reset_pose()


if __name__ == '__main__':
    main()
