"""Independent, read-only Blender audit for the separate cape detail milestone.

Run with --background --factory-startup. This checker never saves source files.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import sys
from collections import Counter, defaultdict
from pathlib import Path

import bmesh
import bpy
from mathutils import Matrix, Vector
from mathutils.bvhtree import BVHTree

REMOVED = {
    'Broad asymmetric cape with tapered feather-fire edge',
    'Refined continuous scarf draped and gathered wrap',
    'Refined scarf soft irregular rolled fold 0',
    'Refined scarf soft irregular rolled fold 1',
    'Scarf broad left shoulder drape',
    'Character_FireFringe_ONLY',
} | {f'Cape tapered ember guide {x}' for x in (.07, .23, .43, .65, .86)}
ORIGINAL_SHA = 'b9847c7088bcb171e9dbccd8d13bdd42f1b26b7e3aff992a3e67a445ba17f59b'


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def require(test, message):
    if not test:
        raise ValueError(message)


def character_meshes(rig):
    return sorted((o for o in bpy.context.scene.objects if o.type == 'MESH'
                   and any(m.type == 'ARMATURE' and m.object == rig for m in o.modifiers)),
                  key=lambda o: o.name)


def fingerprint_components(objects, source_map, exclude):
    values = defaultdict(list)
    counts = Counter()
    for obj in objects:
        attr = obj.data.attributes.get('kiva_component_id')
        if attr is None:
            continue
        uv = obj.data.uv_layers.active
        groups = {group.index: group.name for group in obj.vertex_groups}
        for poly in obj.data.polygons:
            component = str(attr.data[poly.index].value)
            if component not in source_map:
                continue
            label = source_map[component]['object']
            if label in exclude:
                continue
            corners = []
            for loop in poly.loop_indices:
                vert = obj.data.vertices[obj.data.loops[loop].vertex_index]
                xyz = obj.matrix_world @ vert.co
                skin = sorted((groups[g.group], g.weight) for g in vert.groups)
                corners.append((list(xyz), list(uv.data[loop].uv), skin))
            # Canonicalize loop start only; preserve winding and face arity.
            encoded = [json.dumps(c, separators=(',', ':')) for c in corners]
            start = min(range(len(encoded)), key=lambda i: encoded[i:]+encoded[:i])
            values[label].append('|'.join(encoded[start:]+encoded[:start]))
            counts[label] += 1
    return {name: {'faces': counts[name], 'sha256': hashlib.sha256(
                '\n'.join(sorted(faces)).encode()).hexdigest()}
            for name, faces in sorted(values.items())}


def rig_fingerprint(rig):
    rows = []
    for b in rig.data.bones:
        rows.append((b.name, b.parent.name if b.parent else None, list(b.matrix_local),
                     b.get('kiva_source_rest_game_json'), b.get('kiva_source_correction_json')))
    # Matrix rows need ordinary lists for explicit deterministic serialization.
    return hashlib.sha256(json.dumps([
        (name, parent, [list(row) for row in matrix], rest, correction)
        for name, parent, matrix, rest, correction in rows], separators=(',', ':')).encode()).hexdigest()


def reset(rig):
    for pb in rig.pose.bones:
        pb.rotation_mode = 'QUATERNION'
        pb.matrix_basis = Matrix.Identity(4)
    bpy.context.view_layer.update()


def topology(mesh):
    bm = bmesh.new()
    bm.from_mesh(mesh)
    out = {
        'vertices': len(mesh.vertices), 'faces': len(mesh.polygons),
        'triangles': sum(len(p.vertices)-2 for p in mesh.polygons),
        'quad_faces': sum(len(p.vertices) == 4 for p in mesh.polygons),
        'zero_area_faces': sum(f.calc_area() <= 1e-13 for f in bm.faces),
        'boundary_edges': sum(e.is_boundary for e in bm.edges),
        'non_manifold_edges': sum(not e.is_manifold for e in bm.edges),
        'inconsistent_winding_edges': sum(e.is_manifold and not e.is_contiguous for e in bm.edges),
        'loose_vertices': sum(not v.link_faces for v in bm.verts),
        'absolute_signed_volume_m3': abs(bm.calc_volume(signed=True)),
        'nonfinite_vertices_or_normals': sum(not all(math.isfinite(x) for x in (*v.co, *v.normal)) for v in mesh.vertices),
    }
    bm.free()
    return out


def upper_back_clearance(objects, new_objects, source_map):
    points, faces, face_labels = [], [], []
    labels = {'Refined cropped outer top with tension folds', 'Body torso under clothing',
              'Plain dark fitted collar under scarf', 'Neck anatomy'}
    for obj in objects:
        attr = obj.data.attributes.get('kiva_component_id')
        if attr is None:
            continue
        base = len(points)
        points.extend(obj.matrix_world @ v.co for v in obj.data.vertices)
        for poly in obj.data.polygons:
            info = source_map.get(str(attr.data[poly.index].value))
            if info and info['object'] in labels:
                faces.append(tuple(base+i for i in poly.vertices))
                face_labels.append(info['object'])
    require(faces, 'retained torso surfaces missing for clearance audit')
    tree = BVHTree.FromPolygons(points, faces)
    graph = bpy.context.evaluated_depsgraph_get()
    sampled, negatives, minimum = 0, [], math.inf
    for obj in new_objects:
        if obj.get('kiva_source_role') == 'fire_fringe':
            continue
        evaluated = obj.evaluated_get(graph)
        mesh = evaluated.to_mesh()
        for vert in mesh.vertices:
            q = obj.matrix_world @ vert.co
            # Blender +Z is height; negative Y is the back of the character.
            # Restrict to upper-back cloth, excluding the front yoke/cowl.
            if not (1.10 < q.z < 1.43 and q.y < -.045 and abs(q.x) < .20):
                continue
            location, _normal, _index, _distance = tree.ray_cast(Vector((q.x, -2, q.z)), Vector((0, 1, 0)), 4)
            if location is None or location.y > -.015:
                continue
            clearance = location.y-q.y
            sampled += 1
            minimum = min(minimum, clearance)
            if clearance < -.001:
                negatives.append({'object': obj.name, 'blender_point': list(q), 'clearance_m': clearance,
                                  'target_surface': face_labels[_index], 'cape_vertex_normal': list(vert.normal)})
        evaluated.to_mesh_clear()
    return {'samples': sampled, 'minimum_clearance_m': minimum if sampled else None,
            'points_more_than_1mm_inside_retained_upper_back': len(negatives),
            'negative_points_by_object': dict(Counter(v['object'] for v in negatives)),
            'negative_points_by_target': dict(Counter(v['target_surface'] for v in negatives)),
            'worst_points': sorted(negatives, key=lambda v: v['clearance_m'])[:15],
            'method': 'Back-to-front raycast against retained actual torso/top/collar/neck faces; sampled evaluated cape vertices.'}


def audit(path, expected_components, expected_rig, source_map, poses, repo, runtime=False):
    before_sha = sha(path)
    bpy.ops.wm.open_mainfile(filepath=str(path))
    rig = bpy.data.objects['Zhuque_v11_CompatibleRig']
    reset(rig)
    require(rig_fingerprint(rig) == expected_rig, 'reference rig or sourceRest changed')
    objects = character_meshes(rig)
    retained = fingerprint_components(objects, source_map, REMOVED)
    require(retained == expected_components, 'retained body/clothing/hair face values changed')
    all_original = fingerprint_components(objects, source_map, set())
    require(not (set(all_original) & REMOVED), 'old cape components still present')
    new = [o for o in objects if o.get('kiva_source_role') == 'cape_detail' or o.name.startswith('Cape 02')]
    require(new, 'new cape objects not identified')
    graph = bpy.context.evaluated_depsgraph_get()
    rows, cache = [], {}
    max_weight_error = max_influences = 0
    for obj in new:
        is_fire = obj.get('kiva_source_role') == 'fire_fringe'
        control = topology(obj.data)
        if not runtime:
            require(control['quad_faces'] == control['faces'], 'editable cape cage is not all quad')
        require(control['zero_area_faces'] == 0 and control['loose_vertices'] == 0, 'bad control cloth faces')
        subsurf = [m for m in obj.modifiers if m.type == 'SUBSURF' and m.show_viewport and m.levels > 0]
        thickness = [m for m in obj.modifiers if m.type == 'SOLIDIFY' and m.show_viewport and abs(m.thickness) > 0]
        if runtime:
            require(not subsurf, 'runtime LOD still enables Subsurf')
        evaluated = obj.evaluated_get(graph)
        mesh = evaluated.to_mesh()
        final = topology(mesh)
        require(is_fire or thickness or (runtime and final['non_manifold_edges'] == 0
                             and final['absolute_signed_volume_m3'] > 1e-10),
                'cape lacks modifier or baked volumetric cloth thickness')
        require((is_fire or final['non_manifold_edges'] == 0) and final['zero_area_faces'] == 0
                and final['inconsistent_winding_edges'] == 0 and final['nonfinite_vertices_or_normals'] == 0,
                'invalid evaluated closed cape topology')
        evaluated.to_mesh_clear()
        arm = next(m for m in obj.modifiers if m.type == 'ARMATURE')
        arm.show_viewport = False
        bpy.context.view_layer.update()
        evaluated = obj.evaluated_get(graph)
        mesh = evaluated.to_mesh()
        names = {g.index: g.name for g in obj.vertex_groups}
        cache[obj.name] = [(v.co.copy(), [(names[g.group], g.weight) for g in v.groups if g.weight > 1e-8]) for v in mesh.vertices]
        for _point, weights in cache[obj.name]:
            require(weights and all(n in rig.data.bones for n, _ in weights), 'bad evaluated skin influences')
            max_weight_error = max(max_weight_error, abs(sum(w for _, w in weights)-1))
            max_influences = max(max_influences, len(weights))
        evaluated.to_mesh_clear()
        arm.show_viewport = True
        bpy.context.view_layer.update()
        rows.append({'object': obj.name, 'intentional_thin_fire_surface': is_fire, 'control': control, 'evaluated': final,
                     'effective_subsurf': bool(subsurf), 'thickness_m': [m.thickness for m in thickness]})
    require(max_weight_error < 1e-5 and max_influences <= 4, 'new cape skin not normalized/bounded')
    clearance = upper_back_clearance(objects, new, source_map)
    sys.path.insert(0, str(repo/'tools/wasteland'))
    import build_greybox as builder
    builder.RIG = rig
    builder.NODES = {n['name']: n for n in json.loads(bpy.data.texts['reference-rig.json'].as_string())['nodes']}
    selected = {'HammerMusket:game:run:0.50', 'HammerMusket:PlayerMelee { pair: HammerMusket, stage: One }:tick024'}
    max_lbs_error, vertices_checked = 0, 0
    for pose in poses['poses']:
        if pose['name'] not in selected:
            continue
        builder.apply_runtime_pose(poses, pose)
        graph.update()
        palette = {pb.name: pb.matrix @ rig.data.bones[pb.name].matrix_local.inverted() for pb in rig.pose.bones}
        for obj in new:
            evaluated = obj.evaluated_get(graph)
            mesh = evaluated.to_mesh()
            require(len(mesh.vertices) == len(cache[obj.name]), 'pose changed cloth topology')
            for actual, (point, weights) in zip(mesh.vertices, cache[obj.name]):
                predicted = sum(((palette[name]@point)*weight for name, weight in weights), Vector())
                require(all(math.isfinite(x) for x in actual.co), 'nonfinite actual posed cloth vertex')
                max_lbs_error = max(max_lbs_error, (predicted-actual.co).length)
                vertices_checked += 1
            evaluated.to_mesh_clear()
    require(max_lbs_error < 5e-6, 'actual posed thickness/subdivision mesh disagrees with own-bind LBS')
    reset(rig)
    end_sha = sha(path)
    return {'path': str(path), 'sha256': before_sha, 'sha256_after_audit': end_sha, 'artifact_changed_concurrently': end_sha != before_sha, 'preserved_components': len(retained),
            'retained_components_exact': True, 'rig_sourceRest_exact': True,
            'cape': rows, 'maximum_weight_sum_error': max_weight_error, 'maximum_influences': max_influences,
            'upper_back_clearance': clearance, 'representative_actual_posed_vertices': vertices_checked,
            'maximum_actual_mesh_vs_independent_LBS_error_m': max_lbs_error,
            'limits': 'Clearance/finite skin checks do not replace visual art or all-pose self-intersection review.'}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--repo', type=Path, required=True)
    parser.add_argument('--original', type=Path, required=True)
    parser.add_argument('--source', type=Path, required=True)
    parser.add_argument('--lod', type=Path)
    parser.add_argument('--poses', type=Path, required=True)
    parser.add_argument('--report', type=Path, required=True)
    args = parser.parse_args(sys.argv[sys.argv.index('--')+1:])
    require(bpy.app.background, 'read-only audit requires independent background Blender')
    require(sha(args.original) == ORIGINAL_SHA, 'original textured source no longer matches frozen baseline')
    source_map = json.loads((args.original.parent/'texture-report.json').read_text())['source_components']
    bpy.ops.wm.open_mainfile(filepath=str(args.original))
    rig = bpy.data.objects['Zhuque_v11_CompatibleRig']
    expected_rig = rig_fingerprint(rig)
    expected = fingerprint_components(character_meshes(rig), source_map, REMOVED)
    poses = json.loads(args.poses.read_text())
    reports = [audit(args.source, expected, expected_rig, source_map, poses, args.repo)]
    if args.lod:
        reports.append(audit(args.lod, expected, expected_rig, source_map, poses, args.repo, runtime=True))
    args.report.write_text(json.dumps({'original_sha256': ORIGINAL_SHA, 'artifacts': reports}, indent=2)+'\n')
    print(json.dumps({'reports': str(args.report), 'artifacts': [{k:v for k,v in r.items() if k != 'cape'} for r in reports]}, indent=2))


if __name__ == '__main__':
    main()
