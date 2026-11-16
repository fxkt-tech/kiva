"""Drape the concept-v11 scarf and flame-feather cape with a real cloth simulation.

Pattern pieces are authored flat (true metres, feather hem cut into the outline),
triangulated with constrained Delaunay, placed loosely around the rest-pose body
and simulated against a collider made from the unchanged character meshes. The
flat pattern is kept as the cloth rest shape and as UV, so later textures follow
the real cut. Run only in an independent background Blender process.
"""
from __future__ import annotations

import argparse
import json
import math
import random
import sys
import time
from pathlib import Path

import bmesh
import bpy
import numpy as np
from mathutils import Vector, geometry
from mathutils.bvhtree import BVHTree

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import refine_cape as rc  # noqa: E402  (old-cape removal and preservation fingerprints)

PI = math.pi
NECK = Vector((0.0, -0.012))  # neck axis in Blender XY (character faces +Y, right = +X)


def require(test, message):
    if not test:
        raise ValueError(message)


def smooth(a, b, x):
    t = min(1.0, max(0.0, (x - a) / (b - a)))
    return t * t * (3 - 2 * t)


# --------------------------------------------------------------------------- pattern

SPINES = []  # filled by cape_pattern: per-lobe spine polyline in (x, depth) pattern metres


def lobe_outline(root_l, root_r, length, bend, rng, serrate=True, tip_curl=0.0):
    """One flame-feather lobe in (a, s) pattern units; returns left-root -> tip -> right-root.

    a is metres across the pattern at the feather root line, s is metres downward.
    The spine sweeps sideways with ``bend`` and the edges carry a few barb notches
    near the tip so the hem dissolves instead of ending in smooth paddles.
    """
    a0, s0 = root_l
    a1, s1 = root_r
    width = a1 - a0
    steps = max(14, int(length / 0.008))
    wave = rng.uniform(0.010, 0.022) * rng.choice((-1, 1))
    def spine(t):
        # Flame-like S sweep: lateral bend grows toward the tip, plus a soft wave.
        x = (a0 + a1) / 2 + bend * t ** 1.7 + tip_curl * t ** 4 + wave * math.sin(1.6 * PI * t) * t
        s = (s0 + s1) / 2 + length * t
        return x, s
    def half(t):
        # Full feather body, long taper, needle tip; never wider than its root.
        return width / 2 * (1 - t ** 2.0) ** 0.85 * (1 - 0.12 * t)
    left, right = [], []
    notches_l = sorted(rng.uniform(0.45, 0.85) for _ in range(rng.randint(0, 2))) if serrate else []
    notches_r = sorted(rng.uniform(0.45, 0.85) for _ in range(rng.randint(0, 2))) if serrate else []
    for k in range(steps + 1):
        t = k / steps
        x, s = spine(t)
        # Tangent to offset perpendicular.
        x2, s2 = spine(min(1, t + 1e-3))
        x1, s1_ = spine(max(0, t - 1e-3))
        tx, ts = x2 - x1, s2 - s1_
        n = math.hypot(tx, ts) or 1
        nx, ns = ts / n, -tx / n
        h = half(t)
        hl = h * (1 - sum(0.45 * math.exp(-((t - c) / 0.014) ** 2) * (1 if t > c else 0.3) for c in notches_l))
        hr = h * (1 - sum(0.45 * math.exp(-((t - c) / 0.014) ** 2) * (1 if t > c else 0.3) for c in notches_r))
        left.append((x - nx * hl, s - ns * hl))
        right.append((x + nx * hr, s + ns * hr))
    # Blend roots exactly onto the supplied root points.
    left[0] = (a0, s0)
    right[0] = (a1, s1)
    SPINES.append({'points': [spine(k / steps) for k in range(steps + 1)], 'root_width': width, 'length': length})
    return left + right[::-1][1:]


def cape_pattern(width_top, width_hem, root_depth, lobes, seed, flare_len=0.6):
    """Trapezoid cape whose lower edge is a chain of feather lobes.

    ``root_depth(a)`` gives the feather root line depth; ``lobes`` is a list of
    (relative width, length, bend) along the hem, left (-X, a=0) to right (a=1).
    Returns the outline in pattern metres (x across, y = -depth) and the map
    helpers used for 3D placement.
    """
    rng = random.Random(seed)
    SPINES.clear()
    def width_at(s):
        return width_top + (width_hem - width_top) * min(1.0, s / flare_len)
    def to_xy(a, s):
        return ((a - 0.5) * width_at(s), -s)
    total = sum(w for w, _, _ in lobes)
    # Roots along the hem line.
    roots, acc = [], 0.0
    for w, _, _ in lobes:
        roots.append(acc / total)
        acc += w
    roots.append(1.0)
    outline = []
    # Top edge from right to left (counter-clockwise when y up: top right -> top left -> down left side).
    n_top = 60
    for k in range(n_top + 1):
        a = 1 - k / n_top
        outline.append(to_xy(a, 0.0))
    # Left side down to the first root.
    s_end = root_depth(0.0)
    n_side = max(8, int(s_end / 0.01))
    for k in range(1, n_side):
        outline.append(to_xy(0.0, s_end * k / n_side))
    # Hem: lobes left to right.
    for i, (w, length, bend) in enumerate(lobes):
        al, ar = roots[i], roots[i + 1]
        sl, sr = root_depth(al), root_depth(ar)
        # Work in metric coordinates at the root line for the lobe itself.
        pl = to_xy(al, sl)
        pr = to_xy(ar, sr)
        pts = lobe_outline((pl[0], -pl[1]), (pr[0], -pr[1]), length, bend, rng,
                           tip_curl=rng.uniform(-0.25, 0.25) * length * 0.2)
        # Valley between lobes: a short rounded cut upward between them.
        if i > 0:
            outline.pop()  # shared root point
        outline.extend((x, -s) for x, s in pts)
    # Right side back up to the top.
    s_end = root_depth(1.0)
    n_side = max(8, int(s_end / 0.01))
    for k in range(n_side - 1, 0, -1):
        outline.append(to_xy(1.0, s_end * k / n_side))
    width_at.spines = [dict(sp) for sp in SPINES]
    width_at.root_depth = root_depth
    width_at.to_xy = to_xy
    return outline, width_at


def resample(poly, step):
    out = []
    n = len(poly)
    for i in range(n):
        a = Vector(poly[i])
        b = Vector(poly[(i + 1) % n])
        seg = (b - a).length
        k = max(1, int(math.ceil(seg / step)))
        for j in range(k):
            out.append(tuple(a.lerp(b, j / k)))
    # Drop near-duplicates.
    clean = [out[0]]
    for p in out[1:]:
        if (Vector(p) - Vector(clean[-1])).length > step * 0.35:
            clean.append(p)
    if (Vector(clean[0]) - Vector(clean[-1])).length < step * 0.35:
        clean.pop()
    return clean


def point_in_poly(px, py, poly):
    x = np.array([p[0] for p in poly])
    y = np.array([p[1] for p in poly])
    x2, y2 = np.roll(x, -1), np.roll(y, -1)
    inside = np.zeros(px.shape, dtype=bool)
    for xa, ya, xb, yb in zip(x, y, x2, y2):
        cond = ((ya > py) != (yb > py))
        with np.errstate(divide='ignore', invalid='ignore'):
            xi = (xb - xa) * (py - ya) / (yb - ya + 1e-30) + xa
        inside ^= cond & (px < xi)
    return inside


def segment_distance(px, py, poly):
    best = np.full(px.shape, np.inf)
    n = len(poly)
    for i in range(n):
        ax, ay = poly[i]
        bx, by = poly[(i + 1) % n]
        dx, dy = bx - ax, by - ay
        L = dx * dx + dy * dy or 1e-30
        t = np.clip(((px - ax) * dx + (py - ay) * dy) / L, 0, 1)
        d = np.hypot(px - (ax + t * dx), py - (ay + t * dy))
        best = np.minimum(best, d)
    return best


def self_intersects(poly):
    n = len(poly)
    def ccw(a, b, c):
        return (c[1] - a[1]) * (b[0] - a[0]) > (b[1] - a[1]) * (c[0] - a[0])
    segs = [(poly[i], poly[(i + 1) % n]) for i in range(n)]
    for i in range(n):
        a, b = segs[i]
        for j in range(i + 2, n):
            if i == 0 and j == n - 1:
                continue
            c, d = segs[j]
            if ccw(a, c, d) != ccw(b, c, d) and ccw(a, b, c) != ccw(a, b, d):
                return (i, j)
    return None


def triangulate(outline, h):
    boundary = resample(outline, h)
    hit = self_intersects(boundary)
    require(hit is None, f'pattern outline self-intersects at segments {hit}')
    xs = [p[0] for p in boundary]
    ys = [p[1] for p in boundary]
    dy = h * math.sqrt(3) / 2
    gx, gy = [], []
    row = 0
    y = min(ys) + dy / 2
    while y < max(ys):
        off = (h / 2) if row % 2 else 0
        x = min(xs) + off
        while x < max(xs):
            gx.append(x)
            gy.append(y)
            x += h
        y += dy
        row += 1
    gx, gy = np.array(gx), np.array(gy)
    keep = point_in_poly(gx, gy, boundary)
    keep &= segment_distance(gx, gy, boundary) > h * 0.55
    interior = list(zip(gx[keep].tolist(), gy[keep].tolist()))
    verts = [Vector(p) for p in boundary] + [Vector(p) for p in interior]
    nb = len(boundary)
    edges = [(i, (i + 1) % nb) for i in range(nb)]
    out = geometry.delaunay_2d_cdt(verts, edges, [list(range(nb))], 2, 1e-7)
    vco, _, faces = out[0], out[1], out[2]
    faces = [f for f in faces if len(f) == 3]
    return [tuple(v) for v in vco], faces, nb


def patterns():
    """All flat pattern pieces, in metres. Left/right are the character's (a=0 is -X, left)."""
    def inner_root(a):
        return 0.46 + 0.10 * a  # below the attach line where feathers begin; longer on the right
    # (relative root width, length, sideways sweep): long flame feathers, outer ones sweep outward.
    inner_lobes = [(0.10, 0.30, -0.08), (0.13, 0.40, -0.06), (0.15, 0.48, -0.04), (0.14, 0.52, -0.015),
                   (0.16, 0.56, 0.01), (0.15, 0.58, 0.035), (0.16, 0.54, 0.06), (0.12, 0.42, 0.085)]
    def outer_root(a):
        return 0.30 + 0.12 * a
    outer_lobes = [(0.12, 0.22, -0.05), (0.14, 0.30, -0.04), (0.13, 0.34, -0.02), (0.15, 0.38, 0.0),
                   (0.13, 0.36, 0.02), (0.15, 0.42, 0.04), (0.13, 0.40, 0.05), (0.12, 0.34, 0.06)]
    L, W = 1.55, 0.30
    top, bottom = [], []
    n_len = 120
    for k in range(n_len + 1):
        u = k / n_len
        taper = 1 - 0.25 * smooth(0.85, 1.0, u) - 0.15 * smooth(0.12, 0.0, u)
        top.append((u * L, W / 2 * taper))
        bottom.append((u * L, -W / 2 * taper))
    scarf = bottom + top[::-1]
    return {'cape_inner': cape_pattern(0.58, 1.55, inner_root, inner_lobes, seed=11),
            'cape_outer': cape_pattern(0.66, 1.15, outer_root, outer_lobes, seed=23, flare_len=0.4),
            'scarf': (scarf, (L, W))}


def write_pattern_svg(path):
    pats = patterns()
    parts, ox = [], 0.05
    for name, (outline, _) in pats.items():
        xs = [p[0] for p in outline]
        ys = [p[1] for p in outline]
        dx = ox - min(xs)
        pts = ' '.join(f'{(x + dx) * 500:.1f},{(-y + 0.05) * 500 - min(-v for v in ys) * 500:.1f}' for x, y in outline)
        parts.append(f'<polygon points="{pts}" fill="#333" stroke="#f60" stroke-width="1"/>')
        hit = self_intersects(resample(outline, 0.01))
        parts.append(f'<text x="{ox * 500:.0f}" y="20" fill="#fff" font-size="16">{name} {"BAD " + str(hit) if hit else "ok"}</text>')
        ox += max(xs) - min(xs) + 0.08
    Path(path).write_text(f'<svg xmlns="http://www.w3.org/2000/svg" width="{ox * 500:.0f}" height="900" '
                          f'style="background:#888">{"".join(parts)}</svg>')


# --------------------------------------------------------------------------- scene

def collider_from(objects, name):
    graph = bpy.context.evaluated_depsgraph_get()
    bm = bmesh.new()
    for obj in objects:
        ev = obj.evaluated_get(graph)
        mesh = ev.to_mesh()
        mesh.transform(obj.matrix_world)
        bm.from_mesh(mesh)
        ev.to_mesh_clear()
    data = bpy.data.meshes.new(name)
    bm.to_mesh(data)
    bm.free()
    obj = bpy.data.objects.new(name, data)
    bpy.context.scene.collection.objects.link(obj)
    mod = obj.modifiers.new('Collision', 'COLLISION')
    obj.collision.thickness_outer = 0.006
    obj.collision.thickness_inner = 0.02
    obj.collision.cloth_friction = 8.0
    obj.collision.damping = 0.4
    obj.hide_render = True
    return obj


def attach_curve(bvh, phis, radii_x, radii_y, lift, outward, z=1.43):
    """Points just off the shoulder/neck surface along an ellipse around the neck."""
    pts = []
    for phi, rx, ry in zip(phis, radii_x, radii_y):
        xy = NECK + Vector((math.cos(phi) * rx, math.sin(phi) * ry))
        q = Vector((xy.x, xy.y, z))
        hit = bvh.ray_cast(q, Vector((0, 0, -1)), 0.25)
        loc, normal = (hit[0], hit[1]) if hit[0] is not None else bvh.find_nearest(q)[:2]
        if normal.z < 0:
            normal = -normal
        pts.append(loc + normal * outward + Vector((0, 0, lift)))
    print('attach', [tuple(round(c, 3) for c in p) for p in pts], flush=True)
    return pts


PIECES = ('cape_inner', 'cape_outer', 'scarf')


def tag_piece(obj, name):
    attr = obj.data.attributes.new('kiva_cape_piece', 'INT', 'FACE')
    for k in range(len(obj.data.polygons)):
        attr.data[k].value = PIECES.index(name)


def build_piece(name, outline, h, place, pin_test, rest_scale=1.0):
    verts2, faces, nb = triangulate(outline, h)
    mesh = bpy.data.meshes.new(name)
    co3 = [place(x, y) for x, y in verts2]
    mesh.from_pydata([tuple(c) for c in co3], [], faces)
    mesh.update()
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.scene.collection.objects.link(obj)
    uv = mesh.uv_layers.new(name='PatternUV')
    for loop in mesh.loops:
        x, y = verts2[loop.vertex_index]
        uv.data[loop.index].uv = (x, y)  # metres; packed into the atlas later
    pat = mesh.attributes.new('kiva_pattern', 'FLOAT2', 'POINT')
    for i, (x, y) in enumerate(verts2):
        pat.data[i].vector = (x, y)
    edge = mesh.attributes.new('kiva_pattern_edge', 'BOOLEAN', 'POINT')
    for i in range(len(verts2)):
        edge.data[i].value = i < nb
    pin = obj.vertex_groups.new(name='pin')
    pinned = [i for i, (x, y) in enumerate(verts2) if pin_test(x, y, i < nb)]
    if pinned:
        pin.add(pinned, 1.0, 'REPLACE')
    obj.shape_key_add(name='Basis', from_mix=False)
    rest = obj.shape_key_add(name='FlatPatternRest', from_mix=False)
    rest.value = 0.0  # Blender 5 defaults new keys to 1.0; this key only supplies spring rest lengths
    for i, (x, y) in enumerate(verts2):
        rest.data[i].co = (x * rest_scale, 0.0, y * rest_scale)
    for poly in mesh.polygons:
        poly.use_smooth = True
    tag_piece(obj, name)
    return obj, len(pinned)


SCARF_CIRCUMFERENCE, SCARF_HEIGHT = 1.09, 0.44


def place_loop(u, v):
    """u in [0,1) around (from the right side through the back), v metres across the band."""
    th = -2 * PI * u
    r = SCARF_CIRCUMFERENCE / (2 * PI)
    xy = NECK + Vector((math.cos(th) * r * 1.10, math.sin(th) * r * 0.92 + 0.01))
    # Tilted: lower over the front-right shoulder, higher at the back-left.
    tilt = 0.075 * math.cos(th - math.radians(25))
    sag = 0.06 * max(0.0, math.sin(th)) ** 2  # front opens below the chin
    return Vector((xy.x, xy.y, 1.615 + v - tilt - sag))


def build_loop(name, circumference, height, h, place):
    nu = int(round(circumference / h))
    nv = max(4, int(round(height / h)))
    verts, faces, flat = [], [], []
    for j in range(nv + 1):
        v = -height / 2 + height * j / nv
        for i in range(nu):
            verts.append(tuple(place(i / nu, v)))
            flat.append((circumference * i / nu, v))
    for j in range(nv):
        for i in range(nu):
            ni = (i + 1) % nu
            faces.append((j * nu + i, j * nu + ni, (j + 1) * nu + ni, (j + 1) * nu + i))
    mesh = bpy.data.meshes.new(name)
    mesh.from_pydata(verts, [], faces)
    mesh.update()
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.scene.collection.objects.link(obj)
    uv = mesh.uv_layers.new(name='PatternUV')
    for poly in mesh.polygons:
        j, i = divmod(poly.index, nu)
        for loop, (ci, cj) in zip(poly.loop_indices, ((i, j), (i + 1, j), (i + 1, j + 1), (i, j + 1))):
            uv.data[loop].uv = (circumference * ci / nu, -height / 2 + height * cj / nv)
        poly.use_smooth = True
    pat = mesh.attributes.new('kiva_pattern', 'FLOAT2', 'POINT')
    edge = mesh.attributes.new('kiva_pattern_edge', 'BOOLEAN', 'POINT')
    for k, (x, y) in enumerate(flat):
        pat.data[k].vector = (x, y)
        edge.data[k].value = k < nu or k >= nv * nu
    obj.vertex_groups.new(name='pin')
    obj.shape_key_add(name='Basis', from_mix=False)
    rest = obj.shape_key_add(name='FlatPatternRest', from_mix=False)
    rest.value = 0.0
    r = circumference / (2 * PI)
    for k, (x, y) in enumerate(flat):
        th = x / r
        rest.data[k].co = (math.cos(th) * r, math.sin(th) * r, y)  # true cylinder: same lengths as the flat band
    tag_piece(obj, name)
    return obj


def join(objects, name):
    for o in bpy.context.selected_objects:
        o.select_set(False)
    for o in objects:
        o.select_set(True)
    bpy.context.view_layer.objects.active = objects[0]
    bpy.ops.object.join()
    obj = bpy.context.view_layer.objects.active
    obj.name = name
    return obj


def cloth(obj, frames):
    mod = obj.modifiers.new('Drape', 'CLOTH')
    s = mod.settings
    s.quality = 10
    s.mass = 0.25
    s.air_damping = 2.0
    s.tension_stiffness = 40
    s.compression_stiffness = 40
    s.shear_stiffness = 20
    s.bending_stiffness = 5.0  # heavy cloth; keeps narrow feather tips from curling into ribbons
    s.tension_damping = 8
    s.compression_damping = 8
    s.shear_damping = 8
    s.bending_damping = 0.8
    s.vertex_group_mass = 'pin'
    s.pin_stiffness = 1.0
    s.rest_shape_key = obj.data.shape_keys.key_blocks['FlatPatternRest']
    c = mod.collision_settings
    c.collision_quality = 4
    c.distance_min = 0.004
    c.use_self_collision = True
    c.self_distance_min = 0.0025
    c.self_friction = 6
    c.self_impulse_clamp = 0.0
    mod.point_cache.frame_start = 1
    mod.point_cache.frame_end = frames
    return mod


def wind(strength):
    """Gentle steady backward breeze: gives the concept's flared tail without turbulence."""
    bpy.ops.object.effector_add(type='WIND', location=(0, 0.8, 0.7),
                                rotation=(math.radians(78), 0, 0))  # field +Z -> (0, -0.98, +0.21): backward, slightly up
    field = bpy.context.object
    field.name = 'Cape v3 drape breeze (authoring only)'
    field.field.strength = strength
    field.field.flow = 0.0
    field.field.noise = 0.0
    return field


def simulate(obj, frames):
    scene = bpy.context.scene
    scene.frame_start, scene.frame_end = 1, frames
    t0 = time.time()
    for f in range(1, frames + 1):
        scene.frame_set(f)
        if f % 10 == 0:
            print(f'cloth frame {f}/{frames}  {time.time() - t0:.0f}s', flush=True)
    graph = bpy.context.evaluated_depsgraph_get()
    ev = obj.evaluated_get(graph)
    data = bpy.data.meshes.new_from_object(ev, preserve_all_data_layers=True, depsgraph=graph)
    old = obj.data
    obj.modifiers.clear()
    obj.shape_key_clear()
    obj.data = data
    bpy.data.meshes.remove(old)
    return time.time() - t0


def bind(obj, rig):
    """Skin the drape to existing bones: neck/spine at the top, cape helpers below."""
    mesh = obj.data
    pat = mesh.attributes['kiva_pattern']
    piece_face = mesh.attributes['kiva_cape_piece']
    piece = [0] * len(mesh.vertices)
    for poly in mesh.polygons:
        for v in poly.vertices:
            piece[v] = piece_face.data[poly.index].value
    groups = {name: obj.vertex_groups.get(name) or obj.vertex_groups.new(name=name)
              for name in ('gMan Neck', 'gMan Spine2', 'gMan R Clavicle', 'gMan L Clavicle', 'cape_L', 'cape_C', 'cape_R')}
    for name in groups:
        require(name in rig.data.bones, 'missing bone ' + name)
    for v in mesh.vertices:
        x, y = pat.data[v.index].vector
        w = {}
        if PIECES[piece[v.index]] == 'scarf':
            right = smooth(0.08, 0.20, v.co.x)
            left = smooth(-0.08, -0.20, v.co.x)
            w = {'gMan Neck': 0.45, 'gMan Spine2': 0.55 - 0.25 * (right + left),
                 'gMan R Clavicle': 0.25 * right, 'gMan L Clavicle': 0.25 * left}
        else:
            s = -y
            top = 1 - smooth(0.04, 0.38, s)
            lr = smooth(-0.16, 0.16, v.co.x)  # character left (-X) -> right (+X)
            wl, wr = max(0.0, 1 - 2 * lr), max(0.0, 2 * lr - 1)
            wc = 1 - wl - wr
            w = {'gMan Spine2': top, 'cape_L': (1 - top) * wl, 'cape_C': (1 - top) * wc, 'cape_R': (1 - top) * wr}
        total = sum(w.values())
        for name, value in w.items():
            if value > 1e-4:
                groups[name].add([v.index], value / total, 'REPLACE')
    mod = obj.modifiers.new('Own cloth bind - unchanged character rig', 'ARMATURE')
    mod.object = rig
    mod.use_deform_preserve_volume = False
    return mod


# --------------------------------------------------------------------------- render

def clay_material():
    mat = bpy.data.materials.new('Cape v3 neutral clay')
    mat.use_nodes = True
    p = mat.node_tree.nodes['Principled BSDF']
    p.inputs['Base Color'].default_value = (0.2, 0.19, 0.18, 1)
    p.inputs['Roughness'].default_value = 0.85
    return mat


def render(path, angle, target=(0, 0, .88), scale=2.0, size=(1000, 1300)):
    scene = bpy.context.scene
    cam = scene.camera
    cam.location = (math.sin(angle) * 4, math.cos(angle) * 4, target[2] + .08)
    cam.rotation_euler = (Vector(target) - cam.location).to_track_quat('-Z', 'Y').to_euler()
    cam.data.type = 'ORTHO'
    cam.data.ortho_scale = scale
    scene.render.resolution_x, scene.render.resolution_y = size
    scene.render.filepath = str(path)
    bpy.ops.render.render(write_still=True)


def gpu_cycles(samples):
    scene = bpy.context.scene
    scene.render.engine = 'CYCLES'
    prefs = bpy.context.preferences.addons['cycles'].preferences
    for kind in ('OPTIX', 'CUDA'):
        try:
            prefs.compute_device_type = kind
            prefs.get_devices()
            if any(d.type == kind for d in prefs.devices):
                for d in prefs.devices:
                    d.use = d.type == kind
                scene.cycles.device = 'GPU'
                break
        except TypeError:
            continue
    scene.cycles.samples = samples
    scene.cycles.use_denoising = True


# --------------------------------------------------------------------------- main

def main():
    require(bpy.app.background, 'Use an independent background Blender process')
    parser = argparse.ArgumentParser()
    parser.add_argument('--source', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--spacing', type=float, default=0.012)
    parser.add_argument('--frames', type=int, default=80)
    parser.add_argument('--only', default='')
    parser.add_argument('--pattern-preview', action='store_true')
    parser.add_argument('--wind', type=float, default=0.0)
    parser.add_argument('--skip-renders', action='store_true')
    parser.add_argument('--lean', type=float, default=0.25,
                        help='backward gravity component (fraction of g) giving the concept tail flare')
    opts = parser.parse_args(sys.argv[sys.argv.index('--') + 1:])
    opts.output.mkdir(parents=True, exist_ok=True)
    if opts.pattern_preview:
        write_pattern_svg(opts.output / 'patterns.svg')
        return
    bpy.ops.wm.open_mainfile(filepath=str(opts.source))
    rig = bpy.data.objects['Zhuque_v11_CompatibleRig']
    rc.RIG = rig
    rc.builder.RIG = rig
    reference = json.loads(bpy.data.texts['reference-rig.json'].as_string())
    rc.builder.NODES = {n['name']: n for n in reference['nodes']}
    rc.builder.reset_pose()
    old = [o for o in bpy.context.scene.objects if o.type == 'MESH' and any(m.type == 'ARMATURE' and m.object == rig for m in o.modifiers)]
    component_map = json.loads((opts.source.parent / 'texture-report.json').read_text())['source_components']
    before = rc.fingerprints(old, component_map, rc.REMOVED)
    rc.remove_old_cape(old, component_map)
    body = [o for o in bpy.context.scene.objects if o.type == 'MESH' and o.data.attributes.get('kiva_component_id')
            and any(m.type == 'ARMATURE' and m.object == rig for m in o.modifiers)]
    require(rc.fingerprints(body, component_map, set()) == before, 'preserved character changed')
    rc.builder.relaxed_review_pose()  # concept arms-down pose; cape weights avoid the arm bones
    collider = collider_from(body, 'Cape v3 drape collider (not exported)')
    cm = collider.data
    print('collider', len(cm.vertices), len(cm.polygons), [round(min(v.co[i] for v in cm.vertices),3) for i in range(3)], [round(max(v.co[i] for v in cm.vertices),3) for i in range(3)], flush=True)
    bvh = BVHTree.FromPolygons([tuple(v.co) for v in cm.vertices], [tuple(p.vertices) for p in cm.polygons])

    h = opts.spacing
    pieces = []
    pats = patterns()
    inner, inner_w = pats['cape_inner']
    outer, outer_w = pats['cape_outer']
    scarf, (L, W) = pats['scarf']

    # ---- inner long cape: back of neck to right shoulder, flame feathers to the calf.
    def radii(degs, base, shoulder):
        return [base + (shoulder - base) * smooth(-45, 10, d) for d in degs]
    deg_inner = list(np.linspace(-150, -15, 11))
    attach_inner = attach_curve(bvh, [math.radians(d) for d in deg_inner], radii(deg_inner, 0.15, 0.18),
                                [0.07] * 11, 0.004, 0.010)

    def make_place(attach, width_at, degs, hem0, hem1, drop=0.97, clear=0.30):
        n = len(attach)
        phi0, phi1 = math.radians(degs[0]), math.radians(degs[-1])
        hem0, hem1 = math.radians(hem0), math.radians(hem1)
        def at(a):
            f = a * (n - 1)
            i = min(n - 2, int(f))
            return attach[i].lerp(attach[i + 1], f - i)
        def place(x, y):
            s = -y
            a = min(1.0, max(0.0, x / width_at(s) + 0.5))
            top = at(a)
            k = smooth(0.0, 0.40, s)
            phi = (hem0 + (hem1 - hem0) * a)
            hang = NECK + Vector((math.cos(phi) * clear * 1.15, math.sin(phi) * clear))
            xy = top.xy.lerp(hang, k)
            return Vector((xy.x, xy.y, top.z - s * drop))
        return place

    place_inner = make_place(attach_inner, inner_w, deg_inner, -160, -8, clear=0.29)
    obj, npin = build_piece('cape_inner', inner, h, place_inner, lambda x, y, b: b and y > -1e-4)
    pieces.append(obj)

    # ---- outer layer: from the left shoulder blade over the right shoulder (mantle), feathers to the hip.
    deg_outer = list(np.linspace(-125, 18, 13))
    attach_outer = attach_curve(bvh, [math.radians(d) for d in deg_outer], radii(deg_outer, 0.145, 0.195),
                                [0.068 + 0.03 * smooth(-10, 40, d) for d in deg_outer], 0.012, 0.022)
    place_outer = make_place(attach_outer, outer_w, deg_outer, -140, 25, clear=0.33)
    obj, _ = build_piece('cape_outer', outer, h, place_outer, lambda x, y, b: b and y > -1e-4)
    pieces.append(obj)

    # ---- scarf: a closed, loosely tilted cowl loop dropped over the collar so it bunches on its own.
    obj = build_loop('scarf', SCARF_CIRCUMFERENCE, SCARF_HEIGHT, h, place_loop)

    pieces.append(obj)

    if opts.only:
        keep = set(opts.only.split(','))
        for o in list(pieces):
            if o.name not in keep:
                pieces.remove(o)
                bpy.data.objects.remove(o)

    # Each piece keeps its own shape keys; join requires matching keys so sim separately per piece set.
    sim = join(pieces, 'Cape v3 draped cloth') if len(pieces) > 1 else pieces[0]
    sim.data.shape_keys.key_blocks['FlatPatternRest']  # survives join for identical key names
    for key in sim.data.shape_keys.key_blocks:
        key.value = 0.0
    cloth(sim, opts.frames)
    breeze = wind(opts.wind) if opts.wind else None
    # Authoring-only drape bias: a slight backward gravity component reproduces the concept sheet's
    # flared tail in a single static pose. Recorded in the report; runtime animation is unaffected.
    bpy.context.scene.gravity = (0.0, -9.81 * opts.lean, -9.81)
    print('cloth vertices', len(sim.data.vertices), 'triangles', len(sim.data.polygons), flush=True)
    seconds = simulate(sim, opts.frames)
    if breeze:
        bpy.data.objects.remove(breeze)
    sim.vertex_groups.remove(sim.vertex_groups['pin'])
    bind(sim, rig)
    sim.data.materials.clear()
    sim.data.materials.append(clay_material())
    sim.modifiers.new('Cloth thickness', 'SOLIDIFY').thickness = 0.0025
    sub = sim.modifiers.new('Subdivision', 'SUBSURF')
    sub.levels = sub.render_levels = 1
    sim.modifiers.move(2, 1)  # Armature -> Subdivision -> Solidify
    bpy.data.objects.remove(collider)
    bpy.context.scene.gravity = (0.0, 0.0, -9.81)
    graph = bpy.context.evaluated_depsgraph_get()
    ev = sim.evaluated_get(graph).to_mesh()
    ev.calc_loop_triangles()
    report = {
        'stage': 'concept-v11 scarf + flame-feather cape, cloth-simulated authoring source (not a runtime publication)',
        'method': 'flat pattern pieces (constrained Delaunay) draped by Blender cloth against the unchanged character in the relaxed review pose; flat pattern kept as rest shape and UV',
        'spacing_m': h, 'frames': opts.frames, 'lean_backward_gravity_fraction': opts.lean,
        'sim_seconds': round(seconds, 1),
        'control_vertices': len(sim.data.vertices), 'control_triangles': len(sim.data.polygons),
        'evaluated_triangles': len(ev.loop_triangles),
        'pieces': {name: sum(1 for a in sim.data.attributes['kiva_cape_piece'].data if a.value == i) for i, name in enumerate(PIECES)},
        'preserved_character_components': len(before),
        'limits': ['Drape authored in relaxed arms-down review pose; arm bones are not in the cape weights, so the rest A-pose may show arm/cape overlap',
                   'Backward gravity bias is an authoring stylisation matching the concept flare, not runtime physics',
                   'Runtime LOD/GLB/AV04 export not yet run for this cape'],
    }
    sim.evaluated_get(graph).to_mesh_clear()
    (opts.output / 'drape-report.json').write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
    bpy.ops.wm.save_as_mainfile(filepath=str(opts.output / 'cape-v3-draped.blend'), check_existing=False)
    gpu_cycles(48)
    if opts.skip_renders:
        print(json.dumps({'sim_seconds': round(seconds), 'pins': npin}), flush=True)
        return
    render(opts.output / 'clay-back.png', PI)
    render(opts.output / 'clay-front.png', 0)
    render(opts.output / 'clay-side.png', -PI / 2)
    render(opts.output / 'clay-side-r.png', PI / 2)
    render(opts.output / 'clay-neck.png', -.35, (0, 0, 1.36), .7, (1100, 1000))
    print(json.dumps({'sim_seconds': round(seconds), 'pins': npin}), flush=True)


if __name__ == '__main__':
    main()
