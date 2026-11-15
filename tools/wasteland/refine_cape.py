"""Rebuild the v11 cape as editable, layered cloth surfaces in a separate source.

Run only in an independent Blender background process. The existing body,
clothes, hair, rig and their UV/skin values are preserved by component hashes.
This is authoring geometry; production export is a separate explicit action.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import sys
from pathlib import Path

import bmesh
import bpy
from mathutils import Vector

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import build_greybox as builder

PI = math.pi
B = builder.B
NEW = []
RIG = None
COLLECTION = None
REMOVED = {
    'Broad asymmetric cape with tapered feather-fire edge',
    'Refined continuous scarf draped and gathered wrap',
    'Refined scarf soft irregular rolled fold 0',
    'Refined scarf soft irregular rolled fold 1',
    'Scarf broad left shoulder drape',
    'Character_FireFringe_ONLY',
} | {f'Cape tapered ember guide {x}' for x in (0.07, 0.23, 0.43, 0.65, 0.86)}


def require(test, message):
    if not test:
        raise ValueError(message)


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def smooth(a, b, x):
    t = min(1.0, max(0.0, (x - a) / (b - a)))
    return t * t * (3 - 2 * t)


def gauss(x, center, width):
    return math.exp(-((x - center) / width) ** 2)


def fingerprints(objects, component_map, excluding):
    """Index-independent face values include true source positions, UVs and skin."""
    result = {}
    for obj in objects:
        attr = obj.data.attributes.get('kiva_component_id')
        require(attr is not None, 'source mesh missing component provenance: ' + obj.name)
        groups = {g.index: g.name for g in obj.vertex_groups}
        uv = obj.data.uv_layers.active
        values = {}
        for poly in obj.data.polygons:
            component = str(attr.data[poly.index].value)
            if component_map[component]['object'] in excluding:
                continue
            corners = []
            for loop in poly.loop_indices:
                vert = obj.data.vertices[obj.data.loops[loop].vertex_index]
                skin = sorted((groups[g.group], g.weight) for g in vert.groups)
                corners.append((list(vert.co), list(uv.data[loop].uv), skin))
            values.setdefault(component, []).append(corners)
        for component, faces in values.items():
            label = component_map[component]['object']
            result[label] = {'faces': len(faces), 'sha256': hashlib.sha256(json.dumps(faces, separators=(',', ':')).encode()).hexdigest()}
    return result


def remove_old_cape(objects, component_map):
    removed = []
    for obj in objects:
        bm = bmesh.new()
        bm.from_mesh(obj.data)
        layer = bm.faces.layers.int.get('kiva_component_id')
        require(layer is not None, 'missing component FACE attribute')
        targets = [face for face in bm.faces if component_map[str(face[layer])]['object'] in REMOVED]
        if targets:
            removed.extend(sorted({component_map[str(face[layer])]['object'] for face in targets}))
            bmesh.ops.delete(bm, geom=targets, context='FACES')
            bm.to_mesh(obj.data)
            obj.data.update()
        bm.free()
        if not obj.data.polygons:
            bpy.data.objects.remove(obj, do_unlink=True)
    require(set(removed) == REMOVED, 'did not remove exactly the declared old cape components')
    return sorted(removed)


def material(name, color, roughness=0.88):
    mat = builder.material(name, color, roughness)
    shader = mat.node_tree.nodes.get('Principled BSDF')
    shader.inputs['Sheen Weight'].default_value = 0.22
    shader.inputs['Sheen Roughness'].default_value = 0.85
    mat['kiva_alpha_mode'] = 0
    mat['kiva_surface_effect'] = 0
    return mat


def mesh_surface(name, fn, nu, nv, weights, mat, wrap=False, uv_rect=(0.02, 0.02, 0.64, 0.98), subdiv=1, thickness=0.003):
    verts, faces = [], []
    columns = nu if wrap else nu + 1
    for j in range(nv + 1):
        for i in range(columns):
            x, y, z = fn(i / nu, j / nv)
            verts.append(B @ Vector((-x, y, z)))
    for j in range(nv):
        for i in range(nu):
            ni = (i + 1) % columns
            faces.append((j * columns + i, j * columns + ni, (j + 1) * columns + ni, (j + 1) * columns + i))
    data = bpy.data.meshes.new(name + ' - editable quad cage')
    data.from_pydata(verts, [], faces)
    data.update()
    obj = bpy.data.objects.new(name, data)
    COLLECTION.objects.link(obj)
    obj.data.materials.append(mat)
    uv = data.uv_layers.new(name='CapeUV')
    u0, v0, u1, v1 = uv_rect
    for poly in data.polygons:
        j, i = divmod(poly.index, nu)
        coords = ((i / nu, j / nv), ((i + 1) / nu, j / nv), ((i + 1) / nu, (j + 1) / nv), (i / nu, (j + 1) / nv))
        poly.use_smooth = True
        for loop, (u, v) in zip(poly.loop_indices, coords):
            uv.data[loop].uv = (u0 + (u1 - u0) * u, v0 + (v1 - v0) * v)
    groups = {}
    for j in range(nv + 1):
        for i in range(columns):
            for bone, weight in builder.normalized_weights(weights(i / nu, j / nv)).items():
                groups.setdefault(bone, obj.vertex_groups.get(bone) or obj.vertex_groups.new(name=bone)).add([j * columns + i], weight, 'REPLACE')
    if subdiv:
        modifier = obj.modifiers.new('Cloth curvature refinement - authored folds remain in cage', 'SUBSURF')
        modifier.subdivision_type = 'CATMULL_CLARK'
        modifier.levels = subdiv
        modifier.render_levels = subdiv
    if thickness:
        modifier = obj.modifiers.new('Real woven cloth thickness and closed edge', 'SOLIDIFY')
        modifier.thickness = thickness
        modifier.offset = 0
        modifier.use_even_offset = False
        modifier.use_quality_normals = True
    modifier = obj.modifiers.new('Own cloth bind - unchanged character rig', 'ARMATURE')
    modifier.object = RIG
    modifier.use_deform_preserve_volume = False
    obj['kiva_source_role'] = 'cape_detail'
    obj['authoring'] = 'New quad surface and explicit hierarchical folds; not a subdivision of the previous cape.'
    obj['uv_contract'] = 'Separate atlas rectangles for outer cloth panels; Solidify paired inner face deliberately shares fabric UV, edge rims inherit it.'
    NEW.append(obj)
    return obj


def hem(u):
    # Four unequal pointed tails, separated by narrow curved burned cuts.
    base = .53 + .22 * u
    for center, depth, width in ((.075, .34, .11), (.32, .31, .145), (.58, .29, .135), (.83, .23, .12)):
        x = abs((u - center) / width)
        base -= depth * max(0, 1 - x) ** .72
    for center, height, width in ((.19, .08, .017), (.465, .09, .018), (.71, .07, .020), (.95, .035, .015)):
        base += height * gauss(u, center, width)
    base += .0015 * math.sin(39 * u + .6)
    return base


def cape(u, t):
    # Gathered primarily at anatomical right shoulder (mirrored at mesh creation).
    radius = .108 + .226 * smooth(0, 1, t) ** .67
    center = -.068 - .047 * smooth(.12, 1, t)
    x = (2 * u - 1) * radius + center
    x += .102 * smooth(.43, 1, u) ** .8 * gauss(t, .04, .15)
    x += .018 * math.sin(PI * t) * math.sin(PI * u + .7)
    top = 1.443 - .027 * (2 * u - 1) ** 2 + .009 * math.sin(PI * u)
    y = top * (1 - t) + hem(u) * t
    z = .090 + .080 * smooth(0, .15, t) + .090 * smooth(.12, 1, t)
    # Unequal sweeping folds: broad cloth planes turn over along narrow valleys.
    envelope = smooth(.015, .22, t)
    for c, drift, bend, amp, width in ((.24, .17, -.10, .060, .105), (.73, -.08, .075, -.050, .145)):
        line = c + drift * t + bend * math.sin(PI * t)
        w = width * (.55 + .70 * t)
        q = (u - line) / w
        z += amp * envelope * math.tanh(q * 1.9) * math.exp(-q * q * .37)
    # One edge returns softly behind itself; it isn't another full-height tube.
    z += .024 * smooth(.25, 1, t) * gauss(u, .94 - .14 * t, .075)
    # Wide shoulder slack converges to the right support, with staggered medium folds.
    z += .015 * gauss(t, .145 + .19 * (u - .18) ** 2, .045) * math.sin(PI * u)
    z -= .010 * gauss(t, .22 + .13 * (u - .28) ** 2, .035) * math.sin(PI * u)
    for start, stop, origin, slope, amp in ((.08, .47, .22, .45, .009), (.51, .92, .29, -.30, -.008),
                                           (.19, .63, .54, .30, .007)):
        mid = (start + stop) / 2
        length = (stop - start) / 2
        z += amp * gauss(t, origin + slope * (u - mid), .018) * gauss(u, mid, length * .80)
    # Irregular curls only at the cut edges and the tapering tail tips.
    edge = gauss(u, 0, .022) + gauss(u, 1, .022)
    z += edge * (.007 * math.sin(9 * t + .6) + .003 * math.sin(19 * t))
    z += smooth(.83, 1, t) * .007 * math.sin(17 * u + 5 * t)
    x -= .043 * smooth(.78, 1, t) ** 2 * (.70 + .30 * math.cos(15 * u + .4))
    return x, y, z


def yoke(u, t):
    # Only the front semicircle. The back is the continuous main cape; there is
    # no independent back yoke edge that can slice across the shoulder cloth.
    a = -PI + PI * u
    left = max(0, -math.cos(a))
    front = max(0, -math.sin(a))
    rx = .085 * (1 - t) + (.145 + .125 * left) * t
    rz = .070 * (1 - t) + (.112 + .025 * front) * t
    y = 1.447 - .033 * t - (.040 + .092 * left) * t * t
    y += .012 * math.sin(PI * t) * left
    # Broad diagonal folds have an actual return crease, not thin raised piping.
    fold = 0.0
    for center, amp, width, drift in ((.23, .016, .095, .11), (.52, -.013, .085, -.08), (.78, .013, .10, .07)):
        line = center + drift * math.sin(a + .5)
        q = (t - line) / width
        fold += amp * math.tanh(q * 1.8) * math.exp(-q * q * .5)
    radial = fold * (.50 + .50 * left)
    radial += .011 * gauss(t, .40 + .18 * (u - .22), .045) * gauss(u, .28, .20)
    radial -= .009 * gauss(t, .66 - .25 * (u - .62), .055) * gauss(u, .65, .17)
    # Soft outer edge curls inward rather than projecting as a rigid board.
    radial -= .021 * gauss(t, .97, .09) * (.3 + .7 * left)
    y += .006 * gauss(t, .98, .08) * left
    y += .005 * math.sin(3 * a + t * 4) * math.sin(PI * t)
    return (rx + radial) * math.cos(a), y, (rz + radial) * math.sin(a)


def cowl(u, t):
    a = 2 * PI * u
    front = max(0, -math.sin(a))
    # A broad folded textile wall, not separate round scarf tubes.
    base_radius = .085 + .020 * (1 - t) + .004 * math.sin(a * 2 + .4)
    folds = 0.0
    for center, amp, width, drift in ((.16, .017, .16, .10), (.47, .012, .11, -.08), (.77, .015, .13, .06)):
        fold_center = center + drift * math.sin(a + center * 4)
        folds += amp * gauss(t, fold_center, width)
        folds -= amp * .40 * gauss(t, fold_center + width * .75, width * .24)
    rx = base_radius + folds
    rz = base_radius * .77 + folds
    x = rx * math.cos(a)
    z = rz * math.sin(a) + .003
    y = 1.382 + .089 * t - .020 * front + .008 * math.cos(a + .6)
    y += .007 * math.sin(2 * a + t * 2.6) * math.sin(PI * t)
    y += .002 * math.sin(11 * a + t * 5) * (gauss(t, 0, .09) + gauss(t, 1, .09))
    return x, y, z


def make_cape(mat):
    mesh_surface('Cape 02 - shoulder gathered long cloth', cape, 96, 144,
                 lambda u, t: {'gMan Spine2': 1} if t < .12 else {('cape_R' if u < .32 else ('cape_L' if u > .71 else 'cape_C')): 1}, mat)
    mesh_surface('Cape 02 - continuous asymmetric shoulder shawl', yoke, 112, 40,
                 lambda u, t: {'gMan Spine2': .86, 'gMan R Clavicle': .14 * t * max(0, -math.cos(-PI + PI * u)),
                               'gMan Neck': .14 * (1 - t * max(0, -math.cos(-PI + PI * u)))}, mat,
                 wrap=False, uv_rect=(.68, .02, .98, .47), thickness=.0035)
    mesh_surface('Cape 02 - layered compressed neck cowl', cowl, 160, 36,
                 lambda u, t: {'gMan Spine2': .78, 'gMan Neck': .22}, mat,
                 wrap=True, uv_rect=(.68, .52, .98, .98), thickness=.003)


def counts(objects):
    out = []
    graph = bpy.context.evaluated_depsgraph_get()
    for obj in objects:
        evaluated = obj.evaluated_get(graph)
        mesh = evaluated.to_mesh()
        mesh.calc_loop_triangles()
        bm = bmesh.new()
        bm.from_mesh(mesh)
        out.append({'object': obj.name, 'control_vertices': len(obj.data.vertices),
                    'control_triangles': sum(len(f.vertices) - 2 for f in obj.data.polygons),
                    'evaluated_vertices': len(mesh.vertices), 'evaluated_triangles': len(mesh.loop_triangles),
                    'non_manifold_edges': sum(not e.is_manifold for e in bm.edges),
                    'zero_area_faces': sum(f.calc_area() < 1e-13 for f in bm.faces),
                    'finite': all(math.isfinite(c) for v in mesh.vertices for c in v.co)})
        bm.free()
        evaluated.to_mesh_clear()
    return out


def render(path, angle, target=(0, 0, .88), scale=2.0, size=(1000, 1300)):
    scene = bpy.context.scene
    camera = scene.camera
    camera.location = (math.sin(angle) * 4, math.cos(angle) * 4, target[2] + .08)
    camera.rotation_euler = (Vector(target) - camera.location).to_track_quat('-Z', 'Y').to_euler()
    camera.data.ortho_scale = scale
    scene.render.resolution_x, scene.render.resolution_y = size
    scene.render.filepath = str(path)
    bpy.ops.render.render(write_still=True)


def lighting():
    scene = bpy.context.scene
    scene.render.engine = 'CYCLES'
    scene.cycles.samples = 16
    scene.cycles.use_denoising = True
    for obj in scene.objects:
        if obj.type == 'LIGHT':
            obj.data.energy *= .42
    key = bpy.data.lights.new('Cape shape broad grazing key', 'AREA')
    key.energy, key.size = 320, 2.5
    obj = bpy.data.objects.new('Cape shape broad grazing key', key)
    scene.collection.objects.link(obj)
    obj.location = (-2.5, -3, 3.8)
    obj.rotation_euler = (Vector((0, 0, 1.1)) - obj.location).to_track_quat('-Z', 'Y').to_euler()
    scene.world.color = (.15, .15, .15)


def main():
    require(bpy.app.background, 'Use an independent background Blender process')
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--source', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--preview-only', action='store_true')
    parser.add_argument('--poses', type=Path)
    opts = parser.parse_args(sys.argv[sys.argv.index('--') + 1:])
    opts.output.mkdir(exist_ok=True, parents=True)
    source_hash = sha(opts.source)
    bpy.ops.wm.open_mainfile(filepath=str(opts.source))
    global RIG, COLLECTION
    RIG = bpy.data.objects['Zhuque_v11_CompatibleRig']
    builder.RIG = RIG
    reference = json.loads(bpy.data.texts['reference-rig.json'].as_string())
    builder.NODES = {n['name']: n for n in reference['nodes']}
    builder.reset_pose()
    for mat in bpy.data.materials:
        if mat.get('kiva_surface_effect') == 1:
            mat.use_fake_user = True
    old = [o for o in bpy.context.scene.objects if o.type == 'MESH' and any(m.type == 'ARMATURE' and m.object == RIG for m in o.modifiers)]
    component_map = json.loads((opts.source.parent / 'texture-report.json').read_text())['source_components']
    before = fingerprints(old, component_map, REMOVED)
    removed = remove_old_cape(old, component_map)
    remaining = [o for o in bpy.context.scene.objects if o.type == 'MESH' and o.data.attributes.get('kiva_component_id') and any(m.type == 'ARMATURE' and m.object == RIG for m in o.modifiers)]
    after = fingerprints(remaining, component_map, set())
    require(before == after, 'unchanged character components lost original values')
    COLLECTION = remaining[0].users_collection[0]
    mat = material('Cape 02 - neutral cloth shape review', (.19, .18, .17))
    make_cape(mat)
    lighting()
    report = {'stage': 'Detailed editable cape authoring, not a runtime publication', 'source_sha256': source_hash,
              'replaced_components': removed, 'preserved_components': before,
              'preservation_passed': before == after, 'cape': counts(NEW),
              'unchanged_character_triangles': sum(sum(len(p.vertices)-2 for p in o.data.polygons) for o in remaining),
              'limits': ['No runtime cloth physics', 'Paired inner/outer cloth surfaces intentionally share fabric UV', 'Neutral clay review is not final PBR material']}
    for row in report['cape']:
        require(row['finite'] and row['zero_area_faces'] == 0 and row['non_manifold_edges'] == 0, 'invalid closed cape surface ' + row['object'])
    report['cape_control_triangles'] = sum(r['control_triangles'] for r in report['cape'])
    report['cape_evaluated_triangles'] = sum(r['evaluated_triangles'] for r in report['cape'])
    report['full_character_evaluated_triangles'] = report['unchanged_character_triangles'] + report['cape_evaluated_triangles']
    (opts.output / 'cape-report.json').write_text(json.dumps(report, ensure_ascii=False, indent=2)+'\n')
    bpy.ops.wm.save_as_mainfile(filepath=str(opts.output / 'zhuque-cape-detail.blend'), check_existing=False)
    render(opts.output / 'cape-clay-back.png', PI)
    render(opts.output / 'cape-clay-neck.png', -.28, (0, 0, 1.39), .56, (1100, 1000))
    print('FIRST CAPE SHAPE PREVIEWS READY', flush=True)
    if not opts.preview_only:
        render(opts.output / 'cape-clay-front.png', 0)
        render(opts.output / 'cape-clay-side.png', PI / 2)
        render(opts.output / 'cape-clay-back-three-quarter.png', PI * .78)
        render(opts.output / 'cape-clay-hem.png', PI * .92, (-.07, -.22, .45), .74, (1100, 1000))
    require(sha(opts.source) == source_hash, 'source unexpectedly changed')
    print(json.dumps({'output': str(opts.output), 'cape_evaluated_triangles': report['cape_evaluated_triangles']}), flush=True)


if __name__ == '__main__':
    main()
