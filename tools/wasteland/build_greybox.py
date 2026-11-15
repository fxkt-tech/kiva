"""Build an original v11 greybox and bind it to Kiva's extracted reference rig.

Run in a separate Blender background process. No old game character mesh is
imported. Garment/body/hair geometry is authored here; the head is adapted from
the documented official Blender CC0 anatomy base.
"""
from __future__ import annotations

import argparse
import json
import math
import sys
from pathlib import Path
from types import SimpleNamespace

import bmesh
import bpy
from mathutils import Matrix, Vector

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import import_reference

PI = math.pi
B = Matrix(((1, 0, 0, 0), (0, 0, -1, 0), (0, 1, 0, 0), (0, 0, 0, 1)))
CHARACTER = None
RIG = None
NODES = {}
MATERIALS = []
MESHES = []
STAGE = 'greybox'


def require(condition, message):
    if not condition:
        raise ValueError(message)


def v(value):
    return Vector(value)


def p(name):
    return Matrix(NODES[name]['source_rest_game']).translation.copy()


def remove_character_object(name):
    obj = bpy.data.objects.get(name)
    if obj:
        if obj in MESHES:
            MESHES.remove(obj)
        bpy.data.objects.remove(obj, do_unlink=True)


def surface_api():
    return SimpleNamespace(mesh=mesh, tube=tube, loft=loft, strand=strand, p=p,
                           blend_weights=blend_weights, torso_weight=torso_weight,
                           remove=remove_character_object)


def normalized_weights(values):
    values = sorted(((name, value) for name, value in values.items() if value > 1e-8), key=lambda a: -a[1])[:4]
    total = sum(weight for _, weight in values)
    require(total > 0, 'unweighted vertex')
    return {name: weight / total for name, weight in values}


def blend_weights(a, b, t):
    out = {}
    for name, value in a.items():
        out[name] = out.get(name, 0) + value * (1 - t)
    for name, value in b.items():
        out[name] = out.get(name, 0) + value * t
    return normalized_weights(out)


def torso_weight(y):
    levels = [(0.89, 'gMan Pelvis'), (0.97, 'gMan Spine'), (1.07, 'gMan Spine1'),
              (1.207, 'gMan Spine2'), (1.414, 'gMan Neck')]
    if y <= levels[0][0]:
        return {levels[0][1]: 1}
    for i in range(len(levels) - 1):
        ya, a = levels[i]
        yb, b = levels[i + 1]
        if y <= yb:
            return blend_weights({a: 1}, {b: 1}, (y - ya) / (yb - ya))
    return {'gMan Neck': 1}


def material(name, color, roughness=0.75, metallic=0):
    mat = bpy.data.materials.new(name)
    mat.diffuse_color = (*color, 1)
    mat.use_nodes = True
    shader = mat.node_tree.nodes.get('Principled BSDF')
    shader.inputs['Base Color'].default_value = (*color, 1)
    shader.inputs['Roughness'].default_value = roughness
    shader.inputs['Metallic'].default_value = metallic
    return mat


def mesh(name, vertices, faces, weights, mat=0, solid=0.0, smooth=True):
    weights = list(weights)
    data = bpy.data.meshes.new(name)
    data.from_pydata([tuple(B @ v(point)) for point in vertices], [], faces)
    data.update()
    bm = bmesh.new()
    bm.from_mesh(data)
    bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    if solid:
        # Give exposed hems a narrow real rim rather than double every cloth
        # triangle with Solidify. The garment remains a thin double-sided shell.
        bm.normal_update()
        bm.verts.ensure_lookup_table()
        boundary = [edge for edge in bm.edges if edge.is_boundary]
        duplicates = {}
        for edge in boundary:
            for vert in edge.verts:
                if vert not in duplicates:
                    duplicates[vert] = bm.verts.new(vert.co - vert.normal * solid)
                    weights.append(weights[vert.index])
        for edge in boundary:
            a, b = edge.verts
            bm.faces.new((a, b, duplicates[b], duplicates[a]))
        bmesh.ops.recalc_face_normals(bm, faces=bm.faces)
    bm.to_mesh(data)
    bm.free()
    obj = bpy.data.objects.new(name, data)
    CHARACTER.objects.link(obj)
    data.materials.append(MATERIALS[mat])
    for polygon in data.polygons:
        polygon.use_smooth = smooth
    groups = {}
    for index, values in enumerate(weights):
        for bone, weight in normalized_weights(values).items():
            if bone not in groups:
                groups[bone] = obj.vertex_groups.new(name=bone)
            groups[bone].add([index], weight, 'REPLACE')
    modifier = obj.modifiers.new('Own mesh bind - native reference rig', 'ARMATURE')
    modifier.object = RIG
    modifier.use_deform_preserve_volume = False
    obj['original_geometry'] = 'Procedurally authored v11 greybox; no source character mesh copied.'
    obj['bind_contract'] = 'Own inverse(bind) from Blender edit rest; original AV03 IBM is not copied.'
    MESHES.append(obj)
    return obj


def loft(name, rings, weights, segments=24, mat=0, caps=True, ripple=0.0, solid=0):
    """Horizontal elliptical loops; rings are (x,y,z,width,depth)."""
    verts, faces, skin = [], [], []
    for row, (x, y, z, rx, rz) in enumerate(rings):
        for i in range(segments):
            a = 2 * PI * i / segments
            fold = 1 + ripple * math.cos(a * 6 + row * 0.31)
            verts.append((x + rx * math.cos(a) * fold, y, z + rz * math.sin(a) * fold))
            skin.append(weights[row])
    for row in range(len(rings) - 1):
        for i in range(segments):
            a, b = row * segments + i, row * segments + (i + 1) % segments
            faces.append((a, b, b + segments, a + segments))
    if caps:
        faces.extend([tuple(reversed(range(segments))), tuple((len(rings) - 1) * segments + i for i in range(segments))])
    return mesh(name, verts, faces, skin, mat, solid)


def tube(name, points, widths, depths, weights, segments=16, mat=0, caps=True, solid=0, ripple=0):
    verts, faces, skin = [], [], []
    points = [v(point) for point in points]
    for row, center in enumerate(points):
        tangent = points[min(row + 1, len(points) - 1)] - points[max(row - 1, 0)]
        tangent.normalize()
        frontal = v((0, 0, -1))
        if abs(tangent.dot(frontal)) > 0.94:
            frontal = v((0, 1, 0))
        frontal = (frontal - tangent * tangent.dot(frontal)).normalized()
        across = tangent.cross(frontal).normalized()
        for i in range(segments):
            a = 2 * PI * i / segments
            fold = 1 + ripple * math.cos(5 * a + row * 0.4)
            verts.append(center + across * math.cos(a) * widths[row] * fold + frontal * math.sin(a) * depths[row] * fold)
            skin.append(weights[row])
    for row in range(len(points) - 1):
        for i in range(segments):
            a, b = row * segments + i, row * segments + (i + 1) % segments
            faces.append((a, a + segments, b + segments, b))
    if caps:
        faces.append(tuple(range(segments)))
        if caps != 'start':
            faces.append(tuple(reversed([(len(points) - 1) * segments + i for i in range(segments)])))
    return mesh(name, verts, faces, skin, mat, solid)


def strand(name, points, radius, bone='gMan Head', mat=2, segments=7):
    # Smooth the authored control polygon into a real curved surface; straight
    # four-point hair tubes read as giant angular wedges in the silhouette.
    if len(points) >= 3:
        controls = [v(point) for point in points]
        curved = []
        for i in range(len(controls) - 1):
            a, b = controls[max(0, i - 1)], controls[i]
            c, d = controls[i + 1], controls[min(i + 2, len(controls) - 1)]
            for j in range(3):
                t = j / 3
                curved.append(0.5 * ((2 * b) + (-a + c) * t + (2 * a - 5 * b + 4 * c - d) * t * t + (-a + 3 * b - 3 * c + d) * t * t * t))
        curved.append(controls[-1])
        points = curved
    count = len(points)
    widths = [radius * (0.65 + 0.45 * math.sin(PI * i / max(1, count - 1))) * (1 - i / count) + 0.0002 for i in range(count)]
    widths[-1] = 0.0005
    return tube(name, points, widths, [r * 0.48 for r in widths], [{bone: 1}] * count, segments, mat)


def ellipsoid(name, center, scale, bone, mat=0, rings=12, segments=24):
    rows = []
    for j in range(rings + 1):
        a = PI * (0.005 + 0.99 * j / rings)
        rows.append((center[0], center[1] + scale[1] * math.cos(a), center[2], scale[0] * math.sin(a), scale[2] * math.sin(a)))
    return loft(name, rows, [{bone: 1}] * len(rows), segments, mat)


def body_and_top():
    rings = [(0, 0.94, 0.003, 0.116, 0.079), (0, 0.99, 0.002, 0.125, 0.083),
             (0, 1.07, 0.003, 0.124, 0.082), (0, 1.15, 0.005, 0.145, 0.091),
             (0, 1.23, 0.006, 0.164, 0.096), (0, 1.31, 0.008, 0.172, 0.087),
             (0, 1.365, 0.009, 0.15, 0.075), (0, 1.405, 0.008, 0.068, 0.052)]
    loft('Body torso under clothing', rings, [torso_weight(r[1]) for r in rings], 32, 0)
    top = [(0, 0.996, -0.001, 0.134, 0.095), (0, 1.006, -0.001, 0.136, 0.096),
           (0, 1.07, 0.0, 0.134, 0.093), (0, 1.14, 0.003, 0.153, 0.103),
           (0, 1.22, 0.004, 0.173, 0.106), (0, 1.30, 0.006, 0.179, 0.098),
           (0, 1.365, 0.007, 0.157, 0.086), (0, 1.402, 0.006, 0.074, 0.063)]
    loft('Waist length outer top', top, [torso_weight(r[1]) for r in top], 40, 1, False, 0.012, 0.003)
    collar = [(0, 1.397, 0.005, 0.066, 0.057), (0, 1.435, 0.005, 0.059, 0.050), (0, 1.472, 0.005, 0.052, 0.045)]
    loft('Plain dark fitted collar under scarf', collar, [{'gMan Neck': 1}] * 3, 32, 2, False, 0, 0.002)
    neck = [(0, 1.405, 0.006, 0.048, 0.044), (0, 1.445, 0.005, 0.046, 0.043), (0, 1.485, 0.002, 0.05, 0.043)]
    loft('Neck anatomy', neck, [blend_weights({'gMan Neck': 1}, {'gMan Head': 1}, i / 2) for i in range(3)], 28, 0)
    for side in ('L', 'R'):
        shoulder, elbow, wrist = [p(f'gMan {side} {suffix}') for suffix in ('UpperArm', 'Forearm', 'Hand')]
        upper, lower = f'gMan {side} UpperArm', f'gMan {side} Forearm'
        arm_points, arm_weights, radii = [], [], []
        for i in range(17):
            t = i / 16 * 2
            if t <= 1:
                center = shoulder.lerp(elbow, t)
                weight = blend_weights({upper: 1}, {lower: 1}, max(0, (t - 0.7) / 0.6))
                rad = 0.05 - 0.014 * t + 0.003 * math.sin(PI * t)
            else:
                q = t - 1
                center = elbow.lerp(wrist, q)
                weight = blend_weights({upper: 1}, {lower: 1}, min(1, (t - 0.7) / 0.6))
                rad = 0.036 - 0.012 * q + 0.002 * math.sin(PI * q)
            arm_points.append(center)
            arm_weights.append(weight)
            radii.append(rad)
        tube(f'{side} plain fitted inner sleeve', arm_points, radii, [r * 0.85 for r in radii], arm_weights, 20, 2)
        # Exactly two external short sleeve shells, each with one hem loop.
        for tier, end, radius in ((1, 0.55, 0.077), (2, 0.81, 0.065)):
            ts = [0.015, 0.12, end * 0.55, end - 0.025, end]
            points = [shoulder.lerp(elbow, t) for t in ts]
            widths = [radius * (1 - 0.06 * t) for t in ts]
            tube(f'{side} outer short sleeve tier {tier}', points, widths, [r * 0.9 for r in widths],
                 [blend_weights({f'gMan {side} Clavicle': 1}, {upper: 1}, min(1, t / 0.25)) for t in ts],
                 28, 1, 'start', 0.003, 0.012)


def trousers():
    # One connected garment. A shared front-to-back crotch seam splits the
    # last hip loop into two leg loops; no overlapping open thigh cylinders.
    segments = 40
    verts, faces, skin = [], [], []
    hips = [(1.004, 0.134, 0.091), (0.98, 0.14, 0.095), (0.93, 0.164, 0.107), (0.89, 0.174, 0.112)]
    if STAGE == 'refined':
        hips = [(1.004, 0.134, 0.091), (0.99, 0.137, 0.093), (0.965, 0.146, 0.099),
                (0.93, 0.16, 0.106), (0.89, 0.171, 0.111), (0.865, 0.174, 0.112)]
    for y, rx, rz in hips:
        for i in range(segments):
            a = i / segments * 2 * PI
            fold = 1 + 0.018 * math.cos(6 * a)
            if STAGE == 'refined':
                fold = 1 + 0.012 * math.sin(5 * a + 0.4) + 0.010 * math.sin(9 * a + (1.004 - y) * 5)
            verts.append((rx * math.cos(a) * fold, y, 0.006 + rz * math.sin(a) * fold))
            skin.append({'gMan Pelvis': 1})
    for row in range(len(hips) - 1):
        for i in range(segments):
            a, b = row * segments + i, row * segments + (i + 1) % segments
            faces.append((a, b, b + segments, a + segments))
    seam = {}
    fork = {}
    for side in ('L', 'R'):
        sign = -1 if side == 'L' else 1
        loop = []
        for i in range(segments):
            a = i / segments * 2 * PI
            exterior = sign * math.cos(a) > -1e-7
            if exterior:
                point = (0.174 * math.cos(a), 0.85, 0.006 + 0.112 * math.sin(a))
            else:
                point = (0, 0.78 + 0.07 * abs(math.sin(a)), 0.006 + 0.112 * math.sin(a))
            key = tuple(round(c, 7) for c in point)
            if key not in seam:
                seam[key] = len(verts)
                verts.append(point)
                skin.append({'gMan Pelvis': 1})
            loop.append(seam[key])
        fork[side] = loop
    hip_base = (len(hips) - 1) * segments
    for i in range(segments):
        j = (i + 1) % segments
        middle = 2 * PI * (i + 0.5) / segments
        side = 'L' if math.cos(middle) < 0 else 'R'
        faces.append((hip_base + i, hip_base + j, fork[side][j], fork[side][i]))
    for side in ('L', 'R'):
        thigh, knee, ankle = [p(f'gMan {side} {suffix}') for suffix in ('Thigh', 'Calf', 'Foot')]
        upper, lower, foot = [f'gMan {side} {suffix}' for suffix in ('Thigh', 'Calf', 'Foot')]
        previous = fork[side]
        leg_rows = 33 if STAGE == 'refined' else 21
        for i in range(leg_rows):
            t = 0.27 + i / (leg_rows - 1) * 1.73
            if t <= 1:
                center = thigh.lerp(knee, t)
                center.y += 0.012
                weight = blend_weights({upper: 1}, {lower: 1}, max(0, (t - 0.72) / 0.56))
                weight = blend_weights({'gMan Pelvis': 1}, weight, min(1, t / 0.3))
                rx = 0.098 + 0.009 * math.sin((t - 0.27) / 0.73 * PI)
                rz = 0.106 - 0.006 * t
            else:
                q = t - 1
                center = knee.lerp(ankle, q)
                center.y += 0.02 + 0.006 * q
                weight = blend_weights({upper: 1}, {lower: 1}, min(1, (t - 0.72) / 0.56))
                weight = blend_weights(weight, {foot: 1}, max(0, (q - 0.9) * 2))
                rx = 0.10 + 0.008 * q
                rz = 0.10 + 0.003 * q
            current = []
            for k in range(segments):
                a = k / segments * 2 * PI
                fold = 1 + 0.035 * math.cos(6 * a + i * 0.2)
                yy = center.y
                if STAGE == 'refined':
                    # Gravity-aligned soft drape, compressed diagonal knee
                    # folds and a gently unsettled wide hem, all actual shape.
                    drape = 0.050 * math.sin(5 * a + 0.45) + 0.025 * math.sin(9 * a - 0.7)
                    knee_envelope = math.exp(-((center.y - knee.y) / 0.075) ** 2)
                    front = max(0, -math.sin(a))
                    compression = 0.040 * math.sin((center.y - knee.y) * 95 + 1.5 * a) * knee_envelope * (0.3 + 0.7 * front)
                    hem = math.exp(-((center.y - 0.15) / 0.055) ** 2)
                    fold = 1 + drape * (0.5 + 0.5 * min(1, t)) + compression
                    fold += hem * 0.028 * math.sin(center.y * 105 + a * 2)
                    yy += 0.0018 * math.sin(5 * a) * hem
                current.append(len(verts))
                verts.append((center.x + rx * math.cos(a) * fold, yy,
                              center.z + 0.002 + rz * math.sin(a) * fold))
                skin.append(weight)
            for k in range(segments):
                j = (k + 1) % segments
                faces.append((previous[k], previous[j], current[j], current[k]))
            previous = current
    mesh('Continuous wide trousers with sewn crotch - no belt', verts, faces, skin, 1, 0.002)


def hands():
    for side in ('L', 'R'):
        hand = f'gMan {side} Hand'
        wrist = p(hand)
        mcp = [p(f'gMan {side} Finger{digit}') for digit in (1, 2, 3, 4)]
        center = sum(mcp, Vector()) / 4
        forward = (center - wrist).normalized()
        across = (mcp[-1] - mcp[0]).normalized()
        normal = forward.cross(across).normalized()
        across = normal.cross(forward).normalized()
        verts, faces, skin = [], [], []
        for j, t in enumerate((0, 0.2, 0.45, 0.75, 1.0)):
            c = wrist.lerp(center, t)
            width = [0.024, 0.028, 0.035, 0.036, 0.034][j]
            depth = [0.018, 0.020, 0.021, 0.019, 0.015][j]
            for i in range(16):
                a = 2 * PI * i / 16
                verts.append(c + across * math.cos(a) * width + normal * math.sin(a) * depth)
                skin.append({hand: 1})
        for j in range(4):
            for i in range(16):
                a, b = j * 16 + i, j * 16 + (i + 1) % 16
                faces.append((a, a + 16, b + 16, b))
        faces.extend([tuple(range(16)), tuple(reversed(range(64, 80)))])
        mesh(f'{side} anatomical palm', verts, faces, skin, 0)
        for digit in range(5):
            chain = [f'gMan {side} Finger{digit}{suffix}' for suffix in ('', '1', '2')]
            joints = [p(name) for name in chain]
            last = Matrix(NODES[chain[-1]]['source_rest_game']).to_3x3() @ Vector((1, 0, 0))
            joints.append(joints[-1] + last.normalized() * (joints[-1] - joints[-2]).length * 0.72)
            points, radii, skin = [], [], []
            base_radius = [0.012, 0.010, 0.0105, 0.0095, 0.008][digit]
            for segment in range(3):
                for k in range(5):
                    t = k / 5
                    points.append(joints[segment].lerp(joints[segment + 1], t))
                    radii.append(base_radius * (1 - 0.14 * (segment + t)))
                    next_bone = chain[min(segment + 1, 2)]
                    skin.append(blend_weights({chain[segment]: 1}, {next_bone: 1}, max(0, (t - 0.65) / 0.7)))
            points.append(joints[-1])
            radii.append(0.003)
            skin.append({chain[-1]: 1})
            tube(f'{side} finger {digit} three-joint surface', points, radii, [r * 0.9 for r in radii], skin, 10, 0)


def adapt_cc0_head(source):
    require(source.exists(), 'Prepare compact CC0 head-source.blend first; see prepare_head_reference.py')
    with bpy.data.libraries.load(str(source), link=False) as (available, loaded):
        loaded.objects = [name for name in available.objects if name.startswith('GEO-head_animation_realistic')]
    for obj in loaded.objects:
        faces = [tuple(poly.vertices) for poly in obj.data.polygons
                 if all(obj.data.vertices[index].co.z >= 0.16 for index in poly.vertices)]
        used = sorted({index for face in faces for index in face})
        remap = {index: i for i, index in enumerate(used)}
        points, weights = [], []
        for index in used:
            q = obj.data.vertices[index].co
            if STAGE == 'refined':
                import refine_head
                x, y, z = refine_head.adapt_source_vertex(q, obj.name)
            else:
                x = -0.86 * q.x
                if q.z < 0.28:
                    x *= 0.96  # Slightly soften/narrow the lower-face base.
                y = 1.29 + 0.94 * q.z
                z = 0.76 * q.y + 0.028
                if '.iris.' in obj.name:
                    # Expose the iris for the opaque four-material clay view.
                    z -= 0.003
                if q.y < -0.12 and 0.21 < q.z < 0.25:
                    y += 0.0015 * math.exp(-((abs(q.x) - 0.025) / 0.014) ** 2)
            points.append((x, y, z))
            weights.append(blend_weights({'gMan Neck': 1}, {'gMan Head': 1}, min(1, max(0, (y - 1.44) / 0.055))))
        mat = 2 if '.iris.' in obj.name else 0
        new = mesh('CC0 adapted youth ' + obj.name, points,
                   [tuple(remap[index] for index in face) for face in faces], weights, mat)
        new['original_geometry'] = 'Adapted from Blender official Human Base Meshes v1.4.1, CC0; see head-source.provenance.json.'
        if STAGE == 'refined' and ('.iris.' in obj.name or '.sclera.' in obj.name):
            # Dense source eyeballs consume nearly4k faces despite a very small
            # footprint. Reclaim that hidden density for hair/cloth shaping.
            bpy.context.view_layer.objects.active = new
            mod = new.modifiers.new('Eye base topology budget', 'DECIMATE')
            mod.ratio = 0.50
            bpy.ops.object.modifier_move_up(modifier=mod.name)
            bpy.ops.object.modifier_apply(modifier=mod.name)
        bpy.data.objects.remove(obj, do_unlink=True)


def head_and_hair(head_source):
    head = 'gMan Head'
    adapt_cc0_head(head_source)
    # Scalp under original swept hair clumps, not hair cards/billboards.
    scalp_verts, scalp_faces = [], []
    for row in range(9):
        t = row / 8
        for i in range(40):
            a = 2 * PI * i / 40
            start_y = 1.59 + 0.055 * max(0, -math.sin(a)) * (1 - abs(math.cos(a)))
            y = start_y * (1 - t) + 1.716 * t
            radial = math.sqrt(max(0.003, 1 - ((y - 1.603) / 0.115) ** 2))
            scalp_verts.append((0.087 * radial * math.cos(a), y, 0.007 + 0.083 * radial * math.sin(a)))
    for row in range(8):
        for i in range(40):
            a, b = row * 40 + i, row * 40 + (i + 1) % 40
            scalp_faces.append((a, b, b + 40, a + 40))
    mesh('Black hair shaped hairline scalp volume', scalp_verts, scalp_faces, [{head: 1}] * len(scalp_verts), 2)
    for side in (-1, 1):
        for i in range(8):
            t = i / 7
            root = (side * (0.004 + 0.014 * t), 1.704 + 0.004 * math.sin(t * PI), -0.016 + 0.069 * t)
            middle = (side * (0.031 + 0.027 * t), 1.717 - 0.026 * t, -0.061 + 0.089 * t)
            end = (side * (0.050 + 0.049 * t), 1.611 - 0.055 * t + 0.013 * math.sin(i * 2), -0.081 + 0.108 * t)
            strand(f'Hair {side} curved parted lock {i}', [root, middle, (side * (0.075 + 0.024 * t), 1.669 - 0.045 * t, -0.078 + 0.105 * t), end], 0.0105 - 0.0025 * t, head, 2, 7)
            if i in (0, 4):
                strand(f'Hair {side} ember guide tip {i}', [end, (end[0] + side * 0.009, end[1] - 0.012, end[2] + 0.005)], 0.003, head, 3, 6)
        for i in range(5):
            t = i / 4
            strand(f'Hair {side} short brow framing fringe {i}',
                   [(side * (0.006 + 0.008 * t), 1.709 - 0.005 * t, -0.034 + 0.015 * t),
                    (side * (0.022 + 0.022 * t), 1.672 - 0.013 * t, -0.081 - 0.004 * t),
                    (side * (0.020 + 0.047 * t), 1.617 - 0.016 * t + 0.005 * math.sin(i * 2), -0.091 + 0.026 * t)],
                   0.009 - 0.0015 * t, head, 2, 6)
        for i in range(7):
            t = i / 6
            strand(f'Hair {side} nape layered lock {i}', [(side * (0.025 + 0.04 * t), 1.68 - 0.03 * t, 0.06),
                   (side * (0.055 + 0.025 * t), 1.64 - 0.045 * t, 0.083),
                   (side * (0.045 + 0.04 * t), 1.558 - 0.025 * t, 0.072)], 0.014, head, 2, 8)
    # Small off-axis tufts keep the center part from becoming a helmet shell.
    for i in range(7):
        x = (i - 3) * 0.016
        strand(f'Hair crown broken tuft {i}', [(x, 1.69, 0.019), (x - 0.01, 1.727 - 0.003 * abs(i - 3), 0.005),
                                              (x + 0.012, 1.715, -0.017)], 0.01, head, 2, 7)


def shoes():
    for side in ('L', 'R'):
        foot = f'gMan {side} Foot'
        x = p(foot).x
        # Rocker sole with toe spring and raised heel. Low collar remains below
        # the ankle rather than turning into a military boot.
        verts, faces, skin = [], [], []
        for row, (height, rx, rz) in enumerate(((0.007, 0.067, 0.137), (0.022, 0.073, 0.148), (0.043, 0.071, 0.145), (0.052, 0.064, 0.138))):
            for i in range(40):
                a = i / 40 * 2 * PI
                z = -0.026 + rz * math.sin(a)
                spring = 0.015 * max(0, -math.sin(a)) ** 3
                verts.append((x + rx * math.cos(a), height + spring, z))
                skin.append({foot: 1})
        for row in range(3):
            for i in range(40):
                a, b = row * 40 + i, row * 40 + (i + 1) % 40
                faces.append((a, b, b + 40, a + 40))
        faces.extend([tuple(reversed(range(40))), tuple(120 + i for i in range(40))])
        mesh(f'{side} trail sneaker rocker midsole', verts, faces, skin, 0)
        # Upper follows toe box -> heel collar with three continuous loop rows.
        verts, faces = [], []
        for row in range(5):
            t = row / 4
            for i in range(40):
                a = i / 40 * 2 * PI
                rx = 0.064 * (1 - 0.3 * t)
                rz = 0.135 * (1 - 0.60 * t)
                z = (-0.026 * (1 - t) + 0.045 * t) + rz * math.sin(a)
                y = 0.052 + t * (0.035 + 0.032 * max(0, math.sin(a)))
                verts.append((x + rx * math.cos(a), y, z))
        for row in range(4):
            for i in range(40):
                a, b = row * 40 + i, row * 40 + (i + 1) % 40
                faces.append((a, b, b + 40, a + 40))
        mesh(f'{side} low-cut sneaker upper', verts, faces, [{foot: 1}] * len(verts), 2, 0.002)
        for i in range(5):
            z = -0.075 + i * 0.026
            height = 0.089 + i * 0.003
            tube(f'{side} shoe lace {i}', [(x - 0.027, height, z), (x, height + 0.006, z + 0.012), (x + 0.027, height, z)],
                 [0.002] * 3, [0.002] * 3, [{foot: 1}] * 3, 6, 0)
        for i in range(8):
            a = i / 8 * 2 * PI
            center = (x + 0.066 * math.cos(a), 0.009 + 0.009 * max(0, -math.sin(a)), -0.026 + 0.13 * math.sin(a))
            ellipsoid(f'{side} outsole trail lug {i}', center, (0.014, 0.009, 0.02), foot, 2, 4, 8)
        for sign in (-1, 1):
            strand(f'{side} sneaker side chassis {sign}', [(x + sign * 0.063, 0.049, -0.105),
                   (x + sign * 0.064, 0.07, -0.015), (x + sign * 0.056, 0.075, 0.068)], 0.006, foot, 0, 7)


def cape_and_scarf():
    # Auxiliary cloth bones are distinct from the immutable 52-joint core.
    bpy.context.view_layer.objects.active = RIG
    RIG.select_set(True)
    bpy.ops.object.mode_set(mode='EDIT')
    for name, x in (('cape_L', -0.18), ('cape_C', 0), ('cape_R', 0.18)):
        bone = RIG.data.edit_bones.new(name)
        bone.head = B @ Vector((x, 1.32, 0.12))
        bone.tail = B @ Vector((x * 1.25, 0.65, 0.24))
        bone.parent = RIG.data.edit_bones['gMan Spine2']
        bone.use_deform = True
    bpy.ops.object.mode_set(mode='OBJECT')
    for turn in range(3):
        points = []
        for i in range(65):
            a = i / 64 * 2 * PI
            rx, rz = 0.107 + turn * 0.01, 0.078 + turn * 0.014
            points.append((rx * math.cos(a), 1.417 - turn * 0.026 + 0.018 * math.sin(a + turn * 0.5), rz * math.sin(a)))
        tube(f'Scarf neck wrap fold {turn + 1}', points, [0.018 + 0.004 * turn] * len(points),
             [0.023] * len(points), [{'gMan Spine2': 0.75, 'gMan Neck': 0.25}] * len(points), 12, 2)
    def cape_point(u, t):
        ends = [(0, 0.45), (0.08, 0.35), (0.17, 0.43), (0.28, 0.32),
                (0.36, 0.56), (0.48, 0.51), (0.60, 0.70), (0.72, 0.66), (0.84, 0.84), (1, 0.89)]
        bottom = ends[-1][1]
        for i in range(len(ends) - 1):
            ua, ya = ends[i]
            ub, yb = ends[i + 1]
            if ua <= u <= ub:
                s = (u - ua) / (ub - ua)
                bottom = ya * (1 - s) + yb * s
                if STAGE == 'refined':
                    # Rounded feather sweeps, not a row of sawtooth cuts.
                    ya0 = ends[max(0, i - 1)][1]
                    yb1 = ends[min(len(ends) - 1, i + 2)][1]
                    bottom = 0.5 * (2 * ya + (-ya0 + yb) * s + (2 * ya0 - 5 * ya + 4 * yb - yb1) * s * s + (-ya0 + 3 * ya - 3 * yb + yb1) * s ** 3)
                break
        width = 0.17 + 0.095 * math.sin(PI * t * 0.73)
        x = (u * 2 - 1) * width - 0.08 * t ** 1.2
        top = 1.397 - 0.045 * abs(2 * u - 1)
        y = top * (1 - t) + bottom * t
        z = 0.095 + 0.18 * t + 0.025 * math.sin(PI * u)
        z += 0.020 * math.sin(u * 6.5 * PI + t * 2.2) * math.sin(PI * t * 0.9)
        if STAGE == 'refined':
            z += 0.009 * math.sin(u * 10 * PI - t * 3) * math.sin(PI * t)
            z += 0.006 * math.sin(t * 7 * PI + u * 5) * (1 - t) ** 2
        x += 0.025 * math.sin(t * PI) * math.sin(u * 2 * PI)
        return x, y, z

    verts, faces, skin = [], [], []
    rows, cols = 37, 29
    for row in range(rows):
        t = row / (rows - 1)
        for col in range(cols):
            u = col / (cols - 1)
            verts.append(cape_point(u, t))
            aux = 'cape_L' if u < 0.33 else ('cape_R' if u > 0.67 else 'cape_C')
            skin.append(blend_weights({'gMan Spine2': 1}, {aux: 1}, min(1, t * 1.6)))
    for row in range(rows - 1):
        for col in range(cols - 1):
            a = row * cols + col
            faces.append((a, a + 1, a + 1 + cols, a + cols))
    mesh('Broad asymmetric cape with tapered feather-fire edge', verts, faces, skin, 2, 0.003)
    # Shoulder drape is a closed-width cloth surface crossing the left collar.
    verts, faces, skin = [], [], []
    for row in range(11):
        t = row / 10
        for col in range(17):
            u = col / 16
            x = -0.075 - 0.19 * u
            y = 1.435 - 0.10 * u - 0.07 * t + 0.008 * math.cos(u * 4 * PI)
            z = -0.085 * (1 - t) + 0.10 * t - 0.02 * u
            if STAGE == 'refined':
                y += 0.006 * math.sin(u * 4 * PI + t * 2) * math.sin(PI * t)
                z += 0.005 * math.sin(t * 3 * PI + u * 5)
            verts.append((x, y, z))
            skin.append({'gMan Spine2': 0.85, 'gMan L Clavicle': 0.15})
    for row in range(10):
        for col in range(16):
            a = row * 17 + col
            faces.append((a, a + 1, a + 18, a + 17))
    mesh('Scarf broad left shoulder drape', verts, faces, skin, 2, 0.003)
    # Narrow, non-emissive orange guides identify the approved tail direction.
    for lane in (0.07, 0.23, 0.43, 0.65, 0.86):
        points = []
        aux = 'cape_L' if lane < 0.33 else ('cape_R' if lane > 0.67 else 'cape_C')
        for i in range(8):
            t = 0.67 + i / 7 * 0.33
            u = lane + 0.012 * math.sin(i * 0.6)
            point = list(cape_point(u, t))
            point[2] += 0.003
            points.append(point)
        tube(f'Cape tapered ember guide {lane}', points, [0.001 + 0.002 * math.sin(PI * i / 8) for i in range(8)],
             [0.001] * 8, [{aux: 1}] * 8, 6, 3)


def bird_motif():
    # Original low-relief bird linework. This is geometric greybox placement,
    # not a final texture, imported SVG or the removed Kiva mark.
    lines = [
        [(-0.012, 1.335), (0.007, 1.354), (0.023, 1.346), (0.007, 1.339)],
        [(0.007, 1.339), (0.016, 1.303), (-0.003, 1.276), (0.007, 1.244), (-0.016, 1.218)],
        [(-0.003, 1.298), (-0.035, 1.314), (-0.07, 1.327), (-0.104, 1.351)],
        [(-0.003, 1.292), (-0.044, 1.296), (-0.083, 1.308), (-0.111, 1.333)],
        [(-0.003, 1.286), (-0.041, 1.28), (-0.08, 1.288), (-0.11, 1.311)],
        [(0.012, 1.307), (0.043, 1.326), (0.077, 1.339), (0.105, 1.364)],
        [(0.01, 1.297), (0.052, 1.309), (0.087, 1.322), (0.12, 1.346)],
        [(0.007, 1.286), (0.052, 1.292), (0.094, 1.307), (0.117, 1.326)],
        [(0.007, 1.247), (-0.021, 1.211), (-0.043, 1.167)],
        [(0.006, 1.252), (0.029, 1.222), (0.012, 1.178)],
    ]
    for index, points in enumerate(lines):
        path = []
        for x, y in points:
            # Follow the cylindrical shirt's front instead of a flat logo card.
            z = -0.105 * math.sqrt(max(0.1, 1 - (x / 0.18) ** 2))
            path.append((x, y, z - 0.003))
        tube(f'Original Zhuque embossed line {index}', path, [0.0018] * len(path), [0.001] * len(path),
             [torso_weight(y) for _, y in points], 6, 3)


def reset_pose():
    for bone in RIG.pose.bones:
        bone.rotation_mode = 'QUATERNION'
        bone.matrix_basis = Matrix.Identity(4)
    bpy.context.view_layer.update()


def apply_runtime_pose(data, pose):
    reset_pose()
    desired = {}
    for node in NODES.values():
        name = node['name']
        if name not in pose['joints']:
            continue
        require(name in data['rest'], f'missing target reference {name}')
        native_rest = B @ Matrix(data['rest'][name]) @ B.inverted()
        current = B @ Matrix(pose['joints'][name]) @ B.inverted()
        desired[name] = current @ native_rest.inverted() @ RIG.data.bones[name].matrix_local
    # Solve every local basis from the known desired parent matrix, then update
    # once. This avoids inherited-pose errors and 52 expensive graph updates.
    for node in NODES.values():
        name = node['name']
        if name not in desired:
            continue
        bone = RIG.data.bones[name]
        if bone.parent:
            parent = bone.parent
            parent_matrix = desired.get(parent.name, parent.matrix_local)
            basis = bone.convert_local_to_pose(desired[name], bone.matrix_local,
                                               parent_matrix=parent_matrix,
                                               parent_matrix_local=parent.matrix_local, invert=True)
        else:
            basis = bone.convert_local_to_pose(desired[name], bone.matrix_local, invert=True)
        RIG.pose.bones[name].matrix_basis = basis
    bpy.context.view_layer.update()
    return desired


def stress_pose(name):
    reset_pose()
    rotations = {
        'shoulder_reach': [('gMan L UpperArm', (0.55, 0.0, -0.70)), ('gMan R UpperArm', (-0.35, 0.0, 0.55)),
                           ('gMan L Forearm', (0, 0.6, 0.3)), ('gMan R Forearm', (0, -0.7, -0.3))],
        'deep_knee': [('gMan L Thigh', (0.75, 0.1, 0.0)), ('gMan R Thigh', (0.75, -0.1, 0)),
                      ('gMan L Calf', (-1.25, 0, 0)), ('gMan R Calf', (-1.25, 0, 0)), ('gMan Spine', (0.12, 0, 0))],
    }
    for bone, euler in rotations[name]:
        pb = RIG.pose.bones[bone]
        pb.rotation_mode = 'XYZ'
        pb.rotation_euler = euler
    bpy.context.view_layer.update()


def relaxed_review_pose():
    """Authored review pose only; never changes the stored bind/rest matrices."""
    reset_pose()
    for side, sign in (('L', -1), ('R', 1)):
        shoulder, elbow = p(f'gMan {side} UpperArm'), p(f'gMan {side} Forearm')
        source = (elbow - shoulder).normalized()
        target = Vector((sign * math.sin(0.26), -math.cos(0.26), 0.015)).normalized()
        rotation = source.rotation_difference(target).to_matrix().to_4x4()
        delta = Matrix.Translation(shoulder) @ rotation @ Matrix.Translation(-shoulder)
        bone = RIG.data.bones[f'gMan {side} UpperArm']
        RIG.pose.bones[bone.name].matrix = B @ delta @ B.inverted() @ bone.matrix_local
        bpy.context.view_layer.update()


def studio():
    scene = bpy.context.scene
    scene.render.engine = 'CYCLES'
    scene.cycles.samples = 24
    scene.cycles.use_denoising = True
    scene.render.resolution_x = 1000
    scene.render.resolution_y = 1300
    scene.render.resolution_percentage = 100
    scene.render.image_settings.file_format = 'PNG'
    scene.world = bpy.data.worlds.new('Neutral greybox studio')
    scene.world.color = (0.18, 0.18, 0.18)
    scene.view_settings.view_transform = 'AgX'
    scene.render.film_transparent = False
    floor_mat = material('Studio floor', (0.10, 0.11, 0.13), 0.8)
    bpy.ops.mesh.primitive_plane_add(size=200, location=(0, 0, -0.006))
    floor = bpy.context.object
    floor.name = 'Studio ground - not character'
    floor.data.materials.append(floor_mat)
    for name, loc, energy, size, color in (
        ('Key softbox', (3, 4, 5), 500, 4, (1.0, 0.91, 0.82)),
        ('Fill softbox', (-3, 2, 3), 350, 3, (0.77, 0.85, 1.0)),
        ('Rim softbox', (0, -3, 4), 650, 3, (1.0, 0.88, 0.77))):
        light = bpy.data.lights.new(name, 'AREA')
        light.energy = energy
        light.shape = 'DISK'
        light.size = size
        light.color = color
        obj = bpy.data.objects.new(name, light)
        scene.collection.objects.link(obj)
        obj.location = loc
        obj.rotation_euler = (Vector((0, 0, 0.9)) - obj.location).to_track_quat('-Z', 'Y').to_euler()
    camera_data = bpy.data.cameras.new('Review camera')
    camera = bpy.data.objects.new('Review camera', camera_data)
    scene.collection.objects.link(camera)
    camera_data.type = 'ORTHO'
    camera_data.ortho_scale = 2.05
    scene.camera = camera
    camera.location = (0, 4, 1)
    camera.rotation_euler = (Vector((0, 0, 0.87)) - camera.location).to_track_quat('-Z', 'Y').to_euler()


def render(output, name, angle=0, portrait=False, shoe_detail=False):
    scene = bpy.context.scene
    camera = scene.camera
    camera.location = (math.sin(angle) * 4, math.cos(angle) * 4, 1.0)
    target = Vector((0, 0, 0.87))
    if shoe_detail:
        camera.location = (0.75, 1.3, 0.45)
        target = Vector((-0.16, 0.02, 0.075))
        camera.data.ortho_scale = 0.48
        scene.render.resolution_x, scene.render.resolution_y = 1100, 800
    elif portrait:
        camera.location = (0.28, 3, 1.62)
        target = Vector((0, 0, 1.40))
        camera.data.ortho_scale = 0.75
        scene.render.resolution_x, scene.render.resolution_y = 1000, 1000
    else:
        camera.data.ortho_scale = 2.05
        scene.render.resolution_x, scene.render.resolution_y = 1000, 1300
    camera.rotation_euler = (target - camera.location).to_track_quat('-Z', 'Y').to_euler()
    safe_name = ''.join(c if c.isalnum() or c in '-_.' else '_' for c in name)
    scene.render.filepath = str(output / (safe_name + '.png'))
    bpy.ops.render.render(write_still=True)


def numeric_report(reference, output, pose_data):
    runtime_checks = None
    if pose_data and pose_data.exists():
        source = json.loads(pose_data.read_text(encoding='utf-8'))
        require(source['schema'] == 'kiva-runtime-palette-poses-v1', 'unknown runtime pose schema')
        cached = []
        for obj in MESHES:
            names = {group.index: group.name for group in obj.vertex_groups}
            for vert in obj.data.vertices:
                cached.append((vert.co.copy(), [(names[g.group], g.weight) for g in vert.groups]))
        inverse_binds = {bone.name: bone.matrix_local.inverted() for bone in RIG.data.bones}
        maximum_mapping_error, checked_vertices = 0.0, 0
        for pose in source['poses']:
            expected = apply_runtime_pose(source, pose)
            for name, matrix in expected.items():
                actual = RIG.pose.bones[name].matrix
                maximum_mapping_error = max(maximum_mapping_error, import_reference.maximum_difference(actual, matrix))
            palette = {bone.name: bone.matrix @ inverse_binds[bone.name] for bone in RIG.pose.bones}
            for point, weights in cached:
                out = Vector()
                for name, weight in weights:
                    out += (palette[name] @ point) * weight
                require(all(math.isfinite(c) for c in out), f'nonfinite actual Blender palette LBS: {pose["name"]}')
                checked_vertices += 1
        require(maximum_mapping_error < 2e-5, 'actual Blender joints differ from requested runtime rest delta')
        runtime_checks = {'pose_source': 'Actual shared RigPose/AttackReplay + avatar::sync target palette export',
                          'snapshots': len(source['poses']), 'finite_lbs_vertices_checked': checked_vertices,
                          'maximum_actual_joint_matrix_vs_expected_error': maximum_mapping_error,
                          'mapping': 'P=(B*A*B^-1)*(B*R*B^-1)^-1*M; own Blender edit-rest inverse bind'}
    reset_pose()
    depsgraph = bpy.context.evaluated_depsgraph_get()
    max_weight_error, maximum_influences, total_vertices, total_triangles = 0.0, 0, 0, 0
    evaluated_triangles = 0
    max_rest_displacement = 0.0
    baseline = {}
    minimum, maximum = Vector((100, 100, 100)), Vector((-100, -100, -100))
    for obj in MESHES:
        total_vertices += len(obj.data.vertices)
        total_triangles += sum(len(face.vertices) - 2 for face in obj.data.polygons)
        for vert in obj.data.vertices:
            groups = [g for g in vert.groups if g.weight > 1e-8]
            maximum_influences = max(maximum_influences, len(groups))
            max_weight_error = max(max_weight_error, abs(sum(g.weight for g in groups) - 1))
            require(all(math.isfinite(c) for c in vert.co), 'nonfinite authored mesh')
            minimum = Vector(tuple(min(minimum[k], vert.co[k]) for k in range(3)))
            maximum = Vector(tuple(max(maximum[k], vert.co[k]) for k in range(3)))
        # Temporarily disable thickness only: rest Armature must be identity.
        thickness = [m for m in obj.modifiers if m.type != 'ARMATURE']
        for mod in thickness:
            mod.show_viewport = False
        depsgraph.update()
        evaluated = obj.evaluated_get(depsgraph).to_mesh()
        evaluated_triangles += sum(len(face.vertices) - 2 for face in evaluated.polygons)
        baseline[obj.name] = [vert.co.copy() for vert in evaluated.vertices]
        require(len(evaluated.vertices) == len(obj.data.vertices), 'unexpected armature topology change')
        for a, b in zip(evaluated.vertices, obj.data.vertices):
            max_rest_displacement = max(max_rest_displacement, (a.co - b.co).length)
        obj.evaluated_get(depsgraph).to_mesh_clear()
        for mod in thickness:
            mod.show_viewport = True
    require(maximum_influences <= 4 and max_weight_error < 1e-5, 'invalid skin weight budget')
    require(max_rest_displacement < 2e-6, 'new mesh not correctly bound in rest')
    checks = []
    for name in ('shoulder_reach', 'deep_knee'):
        stress_pose(name)
        depsgraph.update()
        positions = []
        max_move = 0
        for obj in MESHES:
            other = [m for m in obj.modifiers if m.type != 'ARMATURE']
            for mod in other:
                mod.show_viewport = False
            depsgraph.update()
            eval_obj = obj.evaluated_get(depsgraph)
            evaluated = eval_obj.to_mesh()
            for vert, rest in zip(evaluated.vertices, baseline[obj.name]):
                require(all(math.isfinite(c) for c in vert.co), f'nonfinite stress pose {name}')
                max_move = max(max_move, (vert.co - rest).length)
                positions.append(vert.co)
            eval_obj.to_mesh_clear()
            for mod in other:
                mod.show_viewport = True
        checks.append({'name': name, 'source': 'Authored bone-space stress pose, not runtime motion', 'maximum_vertex_displacement_m': max_move,
                       'finite_vertices': len(positions)})
    reset_pose()
    require(max(total_triangles, evaluated_triangles) <= 40000,
            f'greybox exceeds agreed triangle budget: base {total_triangles}, evaluated {evaluated_triangles}')
    report = {'milestone': f'Concept-v11 {STAGE} model milestone, not final production or game acceptance',
              'blender_version': bpy.app.version_string,
              'geometry_origin': f'Original garment/body/hair {STAGE} geometry authored by build_greybox.py and surface helpers; head topology adapted from official Blender CC0 base, no old game mesh/texture copied.',
              'stage': STAGE,
              'core_reference_nodes': len(reference['nodes']), 'auxiliary_cape_bones': 3,
              'mesh_objects': len(MESHES), 'vertices': total_vertices, 'base_triangles': total_triangles,
              'evaluated_triangles': evaluated_triangles, 'cloth_shells': 'Real boundary rims, no Solidify duplication',
              'character_materials': len(MATERIALS), 'maximum_skin_influences': maximum_influences,
              'maximum_weight_sum_error': max_weight_error, 'maximum_rest_bind_displacement_m': max_rest_displacement,
              'authored_mesh_bounds_blender': [list(minimum), list(maximum)],
              'reference_body_height_m': reference['height_m'],
              'outer_sleeve_tiers_per_side': 2, 'inner_sleeves': 'Plain fitted solid charcoal, no motif', 'belt': False,
              'stress_checks': checks, 'runtime_pose_input': str(pose_data) if pose_data else None,
              'runtime_checks': runtime_checks,
              'limitations': ['Greybox facial topology, cloth, hair and motif need user review and later refinement.',
                             'Cape uses three preview deformation bones; no cloth physics or final flame shader.',
                             'Runtime visual exports are inspection aids; full game integration/attack trajectory acceptance remains later.']}
    (output / f'{STAGE}-report.json').write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    return report


def main():
    global CHARACTER, RIG, NODES, MATERIALS, STAGE
    require(bpy.app.background, 'Use an independent --background Blender process')
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--reference', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--poses', type=Path)
    parser.add_argument('--head-source', type=Path)
    parser.add_argument('--skip-renders', action='store_true')
    parser.add_argument('--stage', choices=('greybox', 'refined'), default='greybox')
    parser.add_argument('--quick-preview', action='store_true', help='Render a short visual iteration, retaining full numeric checks')
    options = parser.parse_args(sys.argv[sys.argv.index('--') + 1:])
    options.output.mkdir(parents=True, exist_ok=True)
    STAGE = options.stage
    require(not (STAGE == 'refined' and options.output.name == 'greybox'), 'Refined stage must use a separate output folder')
    reference = json.loads(options.reference.read_text(encoding='utf-8'))
    # Reuse the reviewed rig creation, with its native-rest corrections.
    import_reference.create(reference, options.output / f'{STAGE}-rig-source.blend')
    RIG = bpy.data.objects[import_reference.RIG_NAME]
    RIG.name = 'Zhuque_v11_CompatibleRig'
    RIG['purpose'] = f'Concept-v11 editable {STAGE} rig, not final production or full in-game animation acceptance.'
    NODES = {n['name']: n for n in reference['nodes']}
    CHARACTER = bpy.data.collections.new(f'ORIGINAL Zhuque v11 {STAGE}')
    bpy.context.scene.collection.children.link(CHARACTER)
    for collection in bpy.data.collections:
        if collection.name.startswith('Runtime structural anchors'):
            collection.hide_viewport = True
            collection.hide_render = True
    MATERIALS = [material('Clay skin and light structural surfaces', (0.48, 0.46, 0.44), 0.68),
                 material('Clay grey outer cloth', (0.30, 0.29, 0.31), 0.85),
                 material('Clay charcoal scarf hair inner sleeves', (0.065, 0.063, 0.068), 0.82),
                 material('Muted ember shape guide - not final fire', (0.40, 0.10, 0.035), 0.75)]
    if STAGE == 'refined':
        MATERIALS = [material('Refined neutral warm clay skin preview', (0.29, 0.225, 0.185), 0.67),
                     material('Refined grey cloth and rubber preview', (0.085, 0.082, 0.091), 0.83),
                     material('Refined charcoal hair scarf and inner cloth', (0.012, 0.011, 0.014), 0.76),
                     material('Refined subdued bronze ember shape guide', (0.13, 0.039, 0.019), 0.80)]
        for index in (1, 2):
            mat = MATERIALS[index]
            nodes = mat.node_tree.nodes
            links = mat.node_tree.links
            shader = nodes.get('Principled BSDF')
            noise = nodes.new('ShaderNodeTexNoise')
            noise.name = 'PREVIEW ONLY fine textile breakup - not baked PBR'
            noise.inputs['Scale'].default_value = 260
            noise.inputs['Detail'].default_value = 2
            geometry = nodes.new('ShaderNodeNewGeometry')
            links.new(geometry.outputs['Position'], noise.inputs['Vector'])
            bump = nodes.new('ShaderNodeBump')
            bump.inputs['Strength'].default_value = 0.15 if index == 1 else 0.08
            bump.inputs['Distance'].default_value = 0.00035
            links.new(noise.outputs['Fac'], bump.inputs['Height'])
            links.new(bump.outputs['Normal'], shader.inputs['Normal'])
            specular = shader.inputs.get('Specular IOR Level')
            if specular:
                specular.default_value = 0.23
    body_and_top()
    trousers()
    hands()
    if STAGE == 'refined':
        import refine_head
        import refined_surfaces
        api = surface_api()
        refined_surfaces.outer_clothes(api)
        source = options.head_source or options.output.parent / 'greybox/head-source.blend'
        adapt_cc0_head(source)
        refine_head.create_face_details(mesh, strand)
        refined_surfaces.hair(api)
        refined_surfaces.shoes(api)
        cape_and_scarf()
        for turn in range(3):
            remove_character_object(f'Scarf neck wrap fold {turn + 1}')
        refined_surfaces.scarf(api)
        refined_surfaces.bird(api)
    else:
        head_and_hair(options.head_source or options.output / 'head-source.blend')
        shoes()
        cape_and_scarf()
        bird_motif()
    studio()
    for screen in bpy.data.screens:
        for area in screen.areas:
            if area.type == 'VIEW_3D':
                area.spaces.active.shading.color_type = 'MATERIAL'
                area.spaces.active.overlay.show_extras = False
    notes = bpy.data.texts.new(f'{STAGE.upper()}_READ_ME')
    notes.write(f'Concept-v11 {STAGE} milestone. Original clothing/body/hair surfaces; adapted official Blender CC0 anatomy head.\n'
                '52 immutable gMan body/finger joints + reference root + 3 separate preview cape bones.\n'
                'Own mesh bind matrices, no old game mesh, texture or IBM copied.\n'
                'Cloth uses narrow boundary rims, not doubled Solidify surfaces.\n'
                'Runtime snapshots are from real shared Kiva drivers; new bind is driven with target current/rest delta.\n'
                'Neutral file is saved; representative runtime renders and numeric report are adjacent.\n'
                'Face/hair/cape shapes and skin weights are greybox review scope, not final art or full gameplay acceptance.\n')
    report = numeric_report(reference, options.output, options.poses)
    # Save the real editable, neutral scene first; render changes don't replace
    # the bind pose in the deliverable.
    reset_pose()
    bpy.context.view_layer.objects.active = RIG
    for obj in bpy.context.selected_objects:
        obj.select_set(False)
    RIG.select_set(True)
    bpy.ops.wm.save_as_mainfile(filepath=str(options.output / f'zhuque-v11-{STAGE}.blend'), check_existing=False)
    if not options.skip_renders:
        views = [('front', 0), ('back', PI), ('side', PI / 2), ('three-quarter', -PI / 6)]
        if options.quick_preview:
            views = [('front', 0), ('three-quarter', -PI / 6)]
        for name, angle in views:
            render(options.output, name, angle)
        render(options.output, 'head-and-sleeve-detail', portrait=True)
        if STAGE == 'refined':
            render(options.output, 'shoe-structure-detail', shoe_detail=True)
            relaxed_review_pose()
            render(options.output, 'authored-relaxed-review', -PI / 7)
            reset_pose()
        if options.quick_preview:
            print(json.dumps(report, ensure_ascii=False, indent=2))
            return
        for name in ('shoulder_reach', 'deep_knee'):
            stress_pose(name)
            render(options.output, 'stress-' + name, -PI / 7)
        if options.poses and options.poses.exists():
            pose_data = json.loads(options.poses.read_text(encoding='utf-8'))
            require(pose_data['schema'] == 'kiva-runtime-palette-poses-v1', 'unknown runtime pose schema')
            selected_names = ['HammerMusket:game:run:0.50', 'HammerMusket:game:crouch:0.50',
                              'HammerMusket:PlayerMelee { pair: HammerMusket, stage: One }:tick024',
                              'ChainAxeBow:PlayerMelee { pair: ChainAxeBow, stage: One }:tick012']
            render_index = []
            for pose in pose_data['poses']:
                if pose['name'] not in selected_names:
                    continue
                apply_runtime_pose(pose_data, pose)
                render(options.output, 'runtime-' + pose['name'], -PI / 7)
                render_index.append({key: pose[key] for key in ('name', 'pair', 'track', 'time', 'duration') if key in pose})
            (options.output / 'runtime-render-index.json').write_text(json.dumps(render_index, indent=2) + '\n', encoding='utf-8')
    reset_pose()
    print(json.dumps(report, ensure_ascii=False, indent=2))


if __name__ == '__main__':
    main()
