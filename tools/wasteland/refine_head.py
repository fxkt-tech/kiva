"""Static youthful face refinement for the documented CC0 head topology.

The importable functions return Kiva model-space points (+Y up, -Z forward).
They do not alter the reference rig, bind matrices or source .blend file.
The optional CLI renders an independent head study in background Blender:

blender --background --factory-startup --python refine_head.py -- \
  --source head-source.blend --output art/wasteland/refined
"""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import sys
from pathlib import Path

REFINEMENT_VERSION = 'zhuque-youth-face-v1'
HEAD_BONE = 'gMan Head'


def _gauss(value, center, radius):
    return math.exp(-((value - center) / radius) ** 2)


def _smoothstep(low, high, value):
    t = min(1.0, max(0.0, (value - low) / (high - low)))
    return t * t * (3.0 - 2.0 * t)


def adapt_source_vertex(q, object_name):
    """Adapt one original head/eye vertex into the refined character's rest.

    All five source objects use the same smooth spatial warp, so eyeballs and
    lid sockets stay aligned. The opaque-clay iris presentation correction is
    the sole object-specific displacement; it is not a biological face edit.
    """
    sx, sy, sz = map(float, q)
    front = 1.0 - _smoothstep(-0.075, -0.025, sy)
    lower_face = _gauss(sz, 0.245, 0.070) * front
    neck_fade = _smoothstep(0.16, 0.205, sz)

    # Shorter lower-face proportions around the unchanged eye/skull anchor,
    # preserving the source neck transition rather than translating the head.
    vertical = 0.085 * max(0.0, 0.310 - sz) * lower_face * neck_fade
    # A young cheek has volume; avoid a hollow adult face or a needle chin.
    cheek = _gauss(abs(sx), 0.047, 0.028) * _gauss(sz, 0.283, 0.037) * front
    jaw = _gauss(sz, 0.223, 0.040) * front
    sx *= 1.0 + 0.025 * cheek + 0.018 * jaw
    sy -= 0.0032 * cheek

    # Reduce the base's adult nose/lip projection very slightly. These broad
    # fields preserve connected topology and do not add beauty-filter detail.
    sy += 0.0045 * _gauss(sx, 0, 0.026) * _gauss(sz, 0.275, 0.026) * front
    sy += 0.0012 * _gauss(sx, 0, 0.036) * _gauss(sz, 0.240, 0.016) * front
    sy -= 0.0016 * _gauss(abs(sx), 0.036, 0.025) * _gauss(sz, 0.337, 0.016) * front

    # A small relaxed asymmetric half-smile, not a grin. The height change
    # follows the lip corners and cheek tissue, not a separate drawn mouth.
    corner = _gauss(abs(sx), 0.030, 0.012) * _gauss(sz, 0.237, 0.015) * front
    smile = corner * (0.0037 if sx > 0 else 0.0026)
    # Slightly settle the upper eye region for alert, assured eyes. Applying
    # this to both lid and eye meshes avoids hidden eye/skin intersections.
    eye = _gauss(abs(sx), 0.036, 0.023) * _gauss(sz, 0.320, 0.010) * front
    sz += vertical + smile - 0.0013 * eye
    if object_name == 'GEO-head_animation_realistic':
        # Settle only the actual upper lid over the eye a little; the shared
        # envelope warp above still keeps the globe's centre aligned.
        sz -= 0.0015 * eye

    x = -0.86 * sx
    y = 1.29 + 0.94 * sz
    z = 0.76 * sy + 0.028
    if '.iris.' in object_name:
        # Preserve the previous documented opaque-preview adaptation. The
        # source uses transparent cornea, which the four clay materials omit.
        z -= 0.003
    return x, y, z


def _source_detail(sx, sy, sz, forward=0.0010):
    point = adapt_source_vertex((sx, sy, sz), 'GEO-head_animation_realistic')
    return point[0], point[1], point[2] - forward


def create_face_details(mesh_factory, strand_factory):
    """Add original small brow forms through the builder's skin factories.

    Factories use the established game-space mesh/strand signatures. Material
    2 is existing charcoal; material 0 is skin. No new material or face bone.
    Returns the created objects for standalone reporting/inspection.
    """
    del strand_factory  # Lid form is sculpted in the source topology, not drawn as a raised tube.
    import bpy
    from mathutils import Matrix, Vector

    head_objects = [obj for obj in bpy.data.objects
                    if obj.type == 'MESH' and obj.name.endswith('GEO-head_animation_realistic')
                    and (obj.name.startswith('CC0 adapted youth ') or obj.name.startswith('Refined study '))]
    if len(head_objects) != 1:
        raise ValueError('Create the adapted head mesh before its surface-projected eyebrows')
    head_object = head_objects[0]
    bpy.context.view_layer.update()
    basis = Matrix(((1, 0, 0, 0), (0, 0, -1, 0), (0, 1, 0, 0), (0, 0, 0, 1)))
    inverse_basis = basis.inverted()

    def on_skin(point):
        # Cast from in front of the actual adapted mesh, rather than infer
        # skin depth from the original rest. This also keeps triangles from
        # clipping when the cheek/brow sculpt coefficients are adjusted.
        world_origin = basis @ Vector((point[0], point[1], -0.5))
        local_origin = head_object.matrix_world.inverted() @ world_origin
        local_direction = head_object.matrix_world.inverted().to_3x3() @ Vector((0, -1, 0))
        hit, location, _normal, _index = head_object.ray_cast(local_origin, local_direction)
        if not hit:
            raise ValueError('Brow guide did not intersect its actual head surface')
        game_hit = inverse_basis @ (head_object.matrix_world @ location)
        return point[0], point[1], game_hit.z - 0.0012

    result = []
    brows = [(0.016, -0.140, 0.330), (0.023, -0.139, 0.331),
             (0.031, -0.136, 0.332), (0.040, -0.132, 0.332),
             (0.049, -0.127, 0.330), (0.057, -0.119, 0.326)]
    for side in (-1, 1):
        vertices = []
        for i, (sx, sy, sz) in enumerate(brows):
            # Slight natural asymmetry and a tapered, almost straight brow.
            width = (0.0003, 0.0017, 0.0020, 0.0019, 0.0012, 0.0002)[i]
            center_z = sz + (0.0006 if side < 0 else 0)
            vertices.extend([on_skin(_source_detail(side * sx, sy, center_z - width / 0.94)),
                             on_skin(_source_detail(side * sx, sy, center_z + width / 0.94))])
        faces = [(2 * i, 2 * i + 2, 2 * i + 3, 2 * i + 1) for i in range(len(brows) - 1)]
        result.append(mesh_factory(f'Refined natural eyebrow {side}', vertices, faces,
                                   [{HEAD_BONE: 1}] * len(vertices), mat=2, solid=0.00035))
    return result


def _head_study(source, output, skip_renders):
    """Create real study meshes; imports Blender only for this optional CLI."""
    import bpy
    from mathutils import Matrix, Vector

    if not bpy.app.background:
        raise RuntimeError('Head study must run in an independent background Blender process')
    if not source.is_file():
        raise FileNotFoundError(source)
    bpy.ops.wm.read_factory_settings(use_empty=True)
    output.mkdir(parents=True, exist_ok=True)
    basis = Matrix(((1, 0, 0, 0), (0, 0, -1, 0), (0, 1, 0, 0), (0, 0, 0, 1)))
    materials = []
    for name, color in [('Study warm grey skin', (0.30, 0.28, 0.26)),
                        ('Unused cloth reference', (0.30, 0.29, 0.30)),
                        ('Study charcoal iris and brows', (0.055, 0.050, 0.047))]:
        material = bpy.data.materials.new(name)
        material.diffuse_color = (*color, 1)
        material.use_nodes = True
        shader = material.node_tree.nodes.get('Principled BSDF')
        shader.inputs['Base Color'].default_value = (*color, 1)
        shader.inputs['Roughness'].default_value = 0.72
        materials.append(material)
    made = []

    def make_mesh(name, vertices, faces, weights=None, mat=0, solid=0.0):
        del weights
        data = bpy.data.meshes.new(name)
        data.from_pydata([basis @ Vector(point) for point in vertices], [], faces)
        data.update()
        obj = bpy.data.objects.new(name, data)
        bpy.context.scene.collection.objects.link(obj)
        data.materials.append(materials[mat])
        for polygon in data.polygons:
            polygon.use_smooth = True
        if solid:
            # Study-only rim counts are separately evaluated in its report.
            modifier = obj.modifiers.new('Study eyebrow thickness', 'SOLIDIFY')
            modifier.thickness = solid
        made.append(obj)
        return obj

    with bpy.data.libraries.load(str(source), link=False) as (available, loaded):
        loaded.objects = [name for name in available.objects if name.startswith('GEO-head_animation_realistic')]
    if len(loaded.objects) != 5:
        raise ValueError('Expected the documented head, two sclera and two iris source meshes')
    maximum_delta = 0.0
    transformed_points = []
    for obj in loaded.objects:
        source_vertices = obj.data.vertices
        faces = [tuple(poly.vertices) for poly in obj.data.polygons
                 if all(source_vertices[i].co.z >= 0.16 for i in poly.vertices)]
        used = sorted({i for face in faces for i in face})
        remap = {old: new for new, old in enumerate(used)}
        points = []
        for i in used:
            q = source_vertices[i].co
            point = adapt_source_vertex(q, obj.name)
            baseline = (-0.86 * q.x, 1.29 + 0.94 * q.z,
                        0.76 * q.y + 0.028 - (0.003 if '.iris.' in obj.name else 0))
            maximum_delta = max(maximum_delta, math.dist(point, baseline))
            if not all(math.isfinite(v) for v in point):
                raise ValueError('Nonfinite head vertex')
            points.append(point)
        transformed_points.extend(points)
        make_mesh('Refined study ' + obj.name, points,
                  [tuple(remap[i] for i in face) for face in faces],
                  mat=2 if '.iris.' in obj.name else 0)
        bpy.data.objects.remove(obj, do_unlink=True)
    before_details = len(made)
    create_face_details(make_mesh, None)
    detail_objects = made[before_details:]

    scene = bpy.context.scene
    scene.render.engine = 'CYCLES'
    scene.cycles.samples = 48
    scene.render.resolution_x = 800
    scene.render.resolution_y = 1000
    scene.render.resolution_percentage = 100
    scene.world = bpy.data.worlds.new('Head study world')
    scene.world.color = (0.025, 0.025, 0.025)
    scene.view_settings.exposure = -0.7
    target = Vector((0, -0.01, 1.582))
    for name, location, energy, size in [('Key', (0.45, 0.65, 2.0), 30, 0.55),
                                         ('Fill', (-0.65, 0.45, 1.7), 9, 0.8),
                                         ('Rim', (0.2, -0.5, 1.9), 24, 0.4)]:
        data = bpy.data.lights.new(name, 'AREA')
        data.energy, data.shape, data.size = energy, 'DISK', size
        lamp = bpy.data.objects.new(name, data)
        scene.collection.objects.link(lamp)
        lamp.location = location
        lamp.rotation_euler = (target - lamp.location).to_track_quat('-Z', 'Y').to_euler()
    camera_data = bpy.data.cameras.new('Head study camera')
    camera = bpy.data.objects.new('Head study camera', camera_data)
    scene.collection.objects.link(camera)
    scene.camera = camera
    camera_data.type = 'ORTHO'
    camera_data.ortho_scale = 0.32
    camera.location = (0, 0.85, 1.582)
    camera.rotation_euler = (target - camera.location).to_track_quat('-Z', 'Y').to_euler()
    bpy.ops.wm.save_as_mainfile(filepath=str(output / 'head-study.blend'), check_existing=False)
    if not skip_renders:
        for name, location in [('head-front', (0, 0.85, 1.582)),
                               ('head-three-quarter', (0.50, 0.78, 1.59))]:
            camera.location = location
            camera.rotation_euler = (target - camera.location).to_track_quat('-Z', 'Y').to_euler()
            scene.render.filepath = str(output / (name + '.png'))
            bpy.ops.render.render(write_still=True)
    depsgraph = bpy.context.evaluated_depsgraph_get()
    triangles = 0
    detail_triangles = 0
    for obj in made:
        evaluated = obj.evaluated_get(depsgraph)
        evaluated_mesh = evaluated.to_mesh()
        evaluated_mesh.calc_loop_triangles()
        count = len(evaluated_mesh.loop_triangles)
        triangles += count
        if obj in detail_objects:
            detail_triangles += count
        evaluated.to_mesh_clear()
    if detail_triangles > 500:
        raise ValueError('Head study detail geometry exceeded its 500-triangle allowance')
    report = {'version': REFINEMENT_VERSION, 'source': str(source),
              'source_sha256': hashlib.sha256(source.read_bytes()).hexdigest(),
              'coordinate_contract': 'game meters,+Yup,-Zforward; study applies the reviewed Blender basis',
              'source_topology_reused': True, 'facial_animation_system': False,
              'head_and_eyes_objects': before_details, 'original_detail_objects': len(detail_objects),
              'evaluated_triangles': triangles, 'study_detail_triangles': detail_triangles,
              'maximum_vertex_change_from_base_map_m': maximum_delta,
              'sculpt_changes': ['Lower-face shortening with unchanged skull/eye anchor',
                                 'Slight cheek volume and softened adult nose projection',
                                 'Relaxed asymmetric lip-corner lift',
                                 'Lower, flatter tapered eyebrows and settled upper eyelids'],
              'public_api': {'adapt_source_vertex': '(source_xyz, original_object_name) -> game_model_xyz',
                             'create_face_details': '(mesh_factory, strand_factory) -> two brow mesh objects; requires adapted head already in scene; strand factory retained for compatibility'},
              'brow_placement': 'Ray-projected to actual adapted head surface,1.2mm forward hair-height offset; no guessing depth',
              'game_bounds': [[min(point[i] for point in transformed_points) for i in range(3)],
                              [max(point[i] for point in transformed_points) for i in range(3)]],
              'limits': ['Static head study only; fullbody rig/skin validation is performed by the builder.',
                         'Simple clay/charcoal preview, no final skin textures or eye shader.',
                         'Static eyelid form is part of adapted head topology; brows are separate low-profile strips.']}
    (output / 'head-study.report.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print(json.dumps(report, indent=2))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--skip-renders', action='store_true')
    options = parser.parse_args(sys.argv[sys.argv.index('--') + 1:])
    _head_study(options.source, options.output, options.skip_renders)


if __name__ == '__main__':
    main()
