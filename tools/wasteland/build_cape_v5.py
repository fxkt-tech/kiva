"""Cape v5: neutral hanging cloak rebuilt from the v03 FRONT / RIGHT SIDE / BACK sheet.

A real garment, not view-facing reliefs.  Cloth layers are surfaces wrapped round the body:
angle theta runs round the wearer (0 = back centre, +90 = the wearer's right, 180 = front)
and each column hangs from the neck/shoulder line down to that layer's hem.  Hem lengths
and tails are measured from the sheet: the back view supplies the back, the side view the
flanks, the front view the shoulder flaps.  World: +X wearer's right, +Y forward, +Z up, m.

  A  scarf   : stacked rolls round the neck, drooping into a cowl at the back
  B  outer   : shoulder cape, short over the shoulders/front, long down the back centre
  C  inner   : long cloth under B, wraps the flanks, carries the burnt tails
"""
from __future__ import annotations

import argparse
import json
import math
import sys
from pathlib import Path

import bpy
import numpy as np

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import cape_v4_common as C
import cape_v5_measure as M

S = 1.15            # millimetres per sheet pixel (collar ~140 mm across, cloak ~1.07 m long)
MM = 0.001
ZREF = 1010         # sheet row that sits at z = 0
N = 3.0             # superellipse exponent of the plan section
PARAMS = {
    'relief_gain': 0.8,
    'bands': [(2.5, 8, 0.0), (7, 26, 12.0), (20, 70, 42.0)],
    'clip': 26.0,
    'pleat_mm': 26.0,
    'collar_tilt_deg': 13.0,
}
# plan section by sheet row (view rows + 47): half-width a, back depth b, front depth f, in sheet px
SECTION_ROWS = [97, 130, 170, 205, 240, 270, 347, 647, 747, 847, 947, 997]
SECTION_B = {'a': [62, 98, 146, 176, 190, 191, 184, 182, 172, 146, 100, 72], 'b': [60, 78, 98, 110, 117, 120, 121, 118, 112, 98, 76, 60],
             'f': [60, 72, 82, 86, 86, 84, 62, 24, 20, 20, 20, 20]}
# outer cape hem by angle (deg -> view row): long down the back, short cap over the shoulders, flaps in front
HEM_B_RIGHT = [(0, 700), (18, 690), (28, 600), (34, 520), (40, 420), (47, 345), (55, 312), (68, 300), (90, 232), (120, 205), (150, 165), (172, 150)]
HEM_B_LEFT = [(0, 700), (18, 680), (28, 580), (34, 500), (40, 400), (47, 330), (55, 296), (68, 286), (90, 225), (120, 215), (150, 190), (158, 160)]
# scarf rolls: front row, back row, half-width, back depth, front depth (view rows / px)
# scarf folds: front row, back row, row at the shoulders, half-width, back depth, front depth (view rows / px)
SCARF = [(58, 54, 56, 66, 68, 64), (78, 88, 66, 90, 98, 78), (100, 128, 78, 112, 120, 84), (122, 168, 92, 126, 130, 86),
         (144, 204, 106, 124, 124, 80), (152, 210, 116, 100, 98, 68)]


def smooth1d(a, k):
    ker = np.hanning(2 * k + 1)
    ker /= ker.sum()
    return np.convolve(np.pad(a, k, mode='edge'), ker, mode='valid')


def relief(view, p):
    """Fold relief (mm) from shading; ember light is avoided by reading the blue channel."""
    img, mask = view['img'], view['mask']
    cool = img[..., 2]
    core = C.blur(((view['warm'] > 95) & (img[..., 1] > 105)).astype(np.float32), 2.0) > 0.10
    valid = (mask & ~core).astype(np.float32)
    s = np.where(valid > 0, cool, C.masked_blur(cool, valid, 7))
    inside = mask.astype(np.float32)
    far = C.masked_blur(s, inside, 60)
    s = np.where(mask, s, far) / np.maximum(far, 8.0)
    calm = 1 - 0.85 * C.smoothstep(0.05, 0.45, C.blur((view['warm'] > 60).astype(np.float32), 12))
    rel = np.zeros_like(s)
    for i, (lo, hi, gain) in enumerate(p['bands']):
        rel += (C.masked_blur(s, inside, lo) - C.masked_blur(s, inside, hi)) * gain * (calm if i < 2 else 0.5 + 0.5 * calm)
    return np.clip(rel, -p['clip'], p['clip']) * C.smoothstep(0.5, 1.0, C.blur(inside, 5))


def envelope(mask):
    """Lowest cloak row per image column (NaN where the column is empty), lightly despeckled."""
    h, w = mask.shape
    rows = np.where(mask.any(0), h - 1 - np.argmax(mask[::-1], axis=0), np.nan)
    out = rows.copy()
    for x in range(1, w - 1):
        trio = rows[x - 1:x + 2]
        if not np.isnan(trio).any():
            out[x] = np.median(trio)
    return out


def sample_env(env, x, fallback):
    xs = np.arange(len(env))
    good = ~np.isnan(env)
    val = np.interp(x, xs[good], env[good])
    inside = (x >= xs[good].min()) & (x <= xs[good].max())
    return np.where(inside, val, fallback)


_FINE = np.arange(60, 1040)


def _smooth_table(values):
    return smooth1d(np.interp(_FINE, SECTION_ROWS, values), 22)


def section(rows, table, shrink=0.0):
    a = np.interp(rows, _FINE, _smooth_table(table['a'])) - shrink
    b = np.interp(rows, _FINE, _smooth_table(table['b'])) - shrink
    f = np.interp(rows, _FINE, _smooth_table(table['f'])) - shrink * 0.5
    return a, b, np.maximum(f, 6.0)


def plan(theta, a, b, f):
    """Point and outward normal on the plan section. theta 0 = back centre, +pi/2 = wearer's right."""
    st, ct = np.sin(theta), np.cos(theta)
    e = 2.0 / N
    depth = np.where(ct >= 0, b, f)
    x = a * np.sign(st) * np.abs(st) ** e
    y = -depth * np.sign(ct) * np.abs(ct) ** e
    nx = np.sign(st) * np.abs(st) ** (2 - e) / a
    ny = -np.sign(ct) * np.abs(ct) ** (2 - e) / depth
    ln = np.maximum(np.hypot(nx, ny), 1e-9)
    return x, y, nx / ln, ny / ln


def grid_mesh(name, pos, valid=None, flip=False, thickness=0.0025, uv=None, quad_mask=None):
    """pos: (rows, cols, 3) metres. valid: (cols,) bool to drop degenerate columns."""
    rows, cols, _ = pos.shape
    ii, jj = np.meshgrid(np.arange(rows - 1), np.arange(cols - 1), indexing='ij')
    keep = np.ones(ii.shape, bool) if valid is None else (valid[jj] & valid[jj + 1])
    if quad_mask is not None:
        keep &= quad_mask
    ii, jj = ii[keep], jj[keep]
    quads = np.stack([ii * cols + jj, (ii + 1) * cols + jj, (ii + 1) * cols + jj + 1, ii * cols + jj + 1], 1)
    if flip:
        quads = quads[:, ::-1]
    mesh = bpy.data.meshes.new(name)
    mesh.vertices.add(rows * cols)
    mesh.vertices.foreach_set('co', pos.reshape(-1, 3).astype(np.float32).ravel())
    mesh.loops.add(quads.size)
    mesh.loops.foreach_set('vertex_index', quads.ravel())
    mesh.polygons.add(len(quads))
    mesh.polygons.foreach_set('loop_start', np.arange(0, quads.size, 4))
    mesh.polygons.foreach_set('loop_total', np.full(len(quads), 4))
    mesh.polygons.foreach_set('use_smooth', np.ones(len(quads), bool))
    mesh.update(calc_edges=True)
    if uv is not None:
        layer = mesh.uv_layers.new(name='UVMap')
        layer.data.foreach_set('uv', uv.reshape(-1, 2)[quads.ravel()].astype(np.float32).ravel())
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.scene.collection.objects.link(obj)
    if thickness:
        solid = obj.modifiers.new('Thickness', 'SOLIDIFY')
        solid.thickness = thickness
        solid.offset = -1
    return obj


def pleats(theta, rows, seed, p):
    """Hanging pleats: none on the shoulders, growing toward the hem, with a slight diagonal sweep."""
    rng = np.random.default_rng(seed)
    out = np.zeros(np.broadcast(theta, rows).shape, np.float32)
    for k, amp in ((3.0, 1.0), (5.0, 0.7), (9.0, 0.45), (17.0, 0.2)):
        out += amp * np.sin(k * theta + rng.uniform(0, 6.28) + rng.uniform(1.6, 3.4) * rows / 300.0
                            + 0.5 * np.sin(2.0 * theta + rng.uniform(0, 6.28)))
    grow = 0.2 + 0.8 * C.smoothstep(150, 620, rows)
    return out * p['pleat_mm'] * 0.7 * grow


def _unused_tails(hem, theta, seed, depth, spacing=(5.0, 13.0), limit=math.radians(104)):
    """Split the lower hem into separate pointed tails: slits rise between them, each tail tapers."""
    rng = np.random.default_rng(seed)
    cuts = [-limit]
    while cuts[-1] < limit:
        cuts.append(cuts[-1] + math.radians(rng.uniform(*spacing)))
    cuts = np.array(cuts)
    idx = np.clip(np.searchsorted(cuts, theta) - 1, 0, len(cuts) - 2)
    lo, hi = cuts[idx], cuts[idx + 1]
    mid = (lo + hi) / 2 + (hi - lo) * rng.uniform(-0.25, 0.25, len(cuts))[idx]
    half = np.where(theta < mid, mid - lo, hi - mid)
    taper = np.clip(np.abs(theta - mid) / np.maximum(half, 1e-6), 0, 1) ** 1.5
    amount = rng.uniform(0.35, 1.0, len(cuts))[idx] * depth
    low = C.smoothstep(380, 640, hem)                     # only where the cloth is long
    return hem - taper * amount * low


def cloth_layer(name, data, p, hem_fn, theta_lo, theta_hi, top_row, table, shrink, seed, n_theta, n_rows, flare=0.0, top_tuck=0.0):
    theta = np.linspace(theta_lo, theta_hi, n_theta)
    hem = hem_fn(theta)
    if callable(top_row):
        top_row = top_row(theta)
    top_row = np.broadcast_to(np.asarray(top_row, np.float32), theta.shape)
    grid = np.linspace(0.0, 1.0, n_rows)[:, None] * (962.0 - top_row[None, :]) + top_row[None, :]
    rows = np.minimum(grid, hem[None, :])                              # view rows; hem rows collapse
    below = np.minimum(hem[:-1], hem[1:])
    quad_mask = np.minimum(grid[:-1, :-1], grid[:-1, 1:]) < below[None, :] - 0.5
    a, b, f = section(rows + 47, table, shrink)
    if top_tuck:
        # near its cut edge a layer pulls in under the layer above, so no lip shows
        inward = top_tuck * (1 - C.smoothstep(0.0, 110.0, rows - top_row[None, :]))
        a, b, f = a - inward, b - inward, np.maximum(f - inward * 0.6, 6.0)
    th = np.broadcast_to(theta[None, :], rows.shape)
    x, y, nx, ny = plan(th, a, b, f)
    ct, st = np.cos(th), np.sin(th)
    wb = np.clip(ct, 0, 1) ** 2
    ws = np.clip(np.abs(st), 0, 1) ** 2 * (ct > -0.3)
    cb, cs = data['frames']['cb'], data['frames']['cs']
    rb = C.bilinear(data['relief']['back'], cb + x, rows)
    rs = C.bilinear(data['relief']['side'], cs + y, rows)
    fall = (rows - top_row[None, :]) / (962.0 - top_row[None, :])
    off = p['relief_gain'] * (wb * rb + ws * rs) + pleats(th, rows, seed, p) + flare * fall ** 2
    pos = np.stack([(x + nx * off / S) * S * MM, (y + ny * off / S) * S * MM, (ZREF - (rows + 47)) * S * MM], -1)
    arc = (theta - theta_lo) / (theta_hi - theta_lo)
    uv = np.stack([np.broadcast_to(arc[None, :], rows.shape), 1 - fall], -1)
    obj = grid_mesh(name, pos, uv=uv, quad_mask=quad_mask)
    return obj, {'quads': int(quad_mask.sum()), 'hem_rows': [float(hem.min()), float(hem.max())]}


def scarf(p):
    """One wide cloth wound round the neck: folds bunch on the shoulders and sag into a cowl at the back."""
    seg, sub = 320, 14
    theta = np.linspace(-math.pi, math.pi, seg, endpoint=False)
    st, ct = np.sin(theta), np.cos(theta)
    rng = np.random.default_rng(3)
    rings = []
    for k, (rf, rb, rside, a, b, f) in enumerate(SCARF):
        wob = (1 + 0.10 * np.sin(2 * theta + rng.uniform(0, 6.28)) + 0.06 * np.sin(5 * theta + rng.uniform(0, 6.28))
               + 0.05 * st)
        x, y, nx, ny = plan(theta, a * wob, b * wob, f * wob)
        lean = 1 + 0.16 * st                          # the whole cowl sits off-centre, as in the sheet
        row = (rside + (rb - rside) * (np.clip(ct, 0, 1) ** 1.6) * lean
               + (rf - rside) * np.clip(-ct, 0, 1) ** 1.6
               + 10 * np.sin(1.0 * theta + k * 2.1) + 5 * np.sin(3 * theta + k * 1.3) + 3 * np.sin(5 * theta + k))
        rings.append((x, y, nx, ny, row))
    out = []
    last = len(rings) - 2
    for k in range(len(rings) - 1):
        u, v = rings[k], rings[k + 1]
        gap = np.hypot(np.hypot(v[0] - u[0], v[1] - u[1]), v[4] - u[4])
        # folds swell and thin round the neck so no two rolls look alike
        vary = 0.55 + 0.45 * np.sin(2 * theta + rng.uniform(0, 6.28)) * np.sin(3 * theta + rng.uniform(0, 6.28)) + 0.3
        ts = np.linspace(0, 1, sub, endpoint=False) if k < last else np.linspace(0, 1, sub + 1)
        for tt in ts:
            off = drop = 0.0
            if k < last:
                if tt < 0.74:
                    roll = math.sin(math.pi * tt / 0.74) ** 0.8
                    off, drop = 0.20 * gap * roll * vary, 0.20 * gap * roll * (tt / 0.74)
                else:
                    crev = math.sin(math.pi * (tt - 0.74) / 0.26) ** 2
                    off, drop = -0.42 * gap * crev * vary, -0.26 * gap * crev
            wr = 1.6 * np.sin(23 * theta + 9 * tt + k) * np.sin(5 * theta + k) * (1 if k < last else 0)
            x = u[0] * (1 - tt) + v[0] * tt + (off + wr) * (u[2] * (1 - tt) + v[2] * tt)
            y = u[1] * (1 - tt) + v[1] * tt + (off + wr) * (u[3] * (1 - tt) + v[3] * tt)
            row = u[4] * (1 - tt) + v[4] * tt + drop
            out.append(np.stack([x * S * MM, y * S * MM, (ZREF - (row + 47)) * S * MM], -1))
    pos = np.stack(out)
    pos = np.concatenate([pos, pos[:, :1]], 1)
    obj = grid_mesh('CapeV5_scarf', pos, thickness=0.0, flip=True)
    sub_mod = obj.modifiers.new('Smooth', 'SUBSURF')
    sub_mod.levels = sub_mod.render_levels = 1
    return obj


def collar(top_z, rx, ry, tilt):
    seg, hgt, wall = 160, 104.0, 7.0
    profile = [(1.5, -hgt), (0.5, -hgt * 0.5), (0.0, -22), (-0.5, -14), (0.3, -12.5), (0.6, -8), (0.3, -3.5), (-1.0, -1.0),
               (-wall * 0.5, 0.0), (-wall + 1.0, -1.0), (-wall, -4), (-wall - 0.3, -14), (-wall, -hgt * 0.5), (-wall + 1, -hgt)]
    up = np.array([0.0, math.sin(tilt), math.cos(tilt)])           # tilted so the front rim is lower
    fwd = np.array([0.0, math.cos(tilt), -math.sin(tilt)])
    right = np.array([1.0, 0.0, 0.0])
    origin = np.array([0.0, 0.0, top_z])
    verts = []
    for i in range(seg):
        th = 2 * math.pi * i / seg
        for dr, hh in profile:
            pos = origin + (right * math.sin(th) * (rx + dr) + fwd * math.cos(th) * (ry + dr) + up * hh) * MM
            verts.append(tuple(pos))
    n = len(profile)
    faces = [(i * n + j, ((i + 1) % seg) * n + j, ((i + 1) % seg) * n + j + 1, i * n + j + 1) for i in range(seg) for j in range(n - 1)]
    mesh = bpy.data.meshes.new('CapeV5_collar')
    mesh.from_pydata(verts, [], faces)
    for poly in mesh.polygons:
        poly.use_smooth = True
    obj = bpy.data.objects.new('CapeV5_collar', mesh)
    bpy.context.scene.collection.objects.link(obj)
    sub = obj.modifiers.new('Smooth', 'SUBSURF')
    sub.levels = sub.render_levels = 2
    return obj


def build(data, p):
    views = data['views']
    back, side, front = views['back'], views['side'], views['front']

    def collar_centre(view):
        xs = np.nonzero(view['mask'][18:40].any(0))[0]
        return 0.5 * (xs.min() + xs.max()), 0.5 * (xs.max() - xs.min())

    cb, rb = collar_centre(back)
    cs, rs = collar_centre(side)
    cf, rf = collar_centre(front)
    data['frames'] = {'cb': cb, 'cs': cs, 'cf': cf}
    data['relief'] = {'back': relief(back, p), 'side': relief(side, p)}
    env_b, env_s = envelope(back['mask']), envelope(side['mask'])
    e = 2.0 / N

    a_fine, b_fine = _smooth_table(SECTION_B['a']), _smooth_table(SECTION_B['b'])
    edges = data.get('edges')
    tables = np.load(str(Path(__file__).resolve().parent.parent.parent / 'art/wasteland/cape-v5/hem-tables.npz'))

    def measured_hem(theta):
        """Longest cloth: the sheet's own silhouette, column by column, on the narrowing section."""
        st, ct = np.sin(theta), np.cos(theta)
        hem = np.full_like(theta, 700.0)
        for _ in range(4):
            a_h = np.interp(hem + 47, _FINE, a_fine)
            b_h = np.interp(hem + 47, _FINE, b_fine)
            xb = cb + a_h * np.sign(st) * np.abs(st) ** e
            ys = cs - b_h * np.sign(ct) * np.abs(ct) ** e
            hb = np.interp(xb, tables['back_x'], tables['back_inner'])
            hs = np.interp(np.minimum(ys, cs + 12), tables['side_x'], tables['side_inner'])
            w = C.smoothstep(math.radians(50), math.radians(64), np.abs(theta))
            hem = 0.5 * hem + 0.5 * (hb * (1 - w) + hs * w)
        return hem

    def measured_layer(prefix, default):
        def fn(theta):
            st, ct = np.sin(theta), np.cos(theta)
            hem = np.full_like(theta, 600.0)
            for _ in range(3):
                a_h = np.interp(hem + 47, _FINE, a_fine)
                b_h = np.interp(hem + 47, _FINE, b_fine)
                xb = cb + a_h * np.sign(st) * np.abs(st) ** e
                ys = cs - b_h * np.sign(ct) * np.abs(ct) ** e
                hb = np.interp(xb, tables['back_x'], tables[f'back_{prefix}'])
                hs = np.interp(np.minimum(ys, cs + 12), tables['side_x'], tables[f'side_{prefix}'])
                w = C.smoothstep(math.radians(50), math.radians(64), np.abs(theta))
                hem = 0.5 * hem + 0.5 * (hb * (1 - w) + hs * w)
            return np.where(hem < default, hem, default)
        return fn

    def layer_top(theta, outer, mid, phase):
        """A long layer emerges from under the shoulder cape at the front and from under the
        back fall at the back, so its cut edge is never seen in the open chest."""
        front = C.smoothstep(math.radians(42), math.radians(60), np.abs(theta))
        outer_rows, mid_rows = outer(theta), mid(theta)
        hidden = np.minimum(outer_rows, mid_rows) - 55.0
        shown = 150.0 + 14 * np.sin(5.3 * theta + phase) + 8 * np.sin(13.1 * theta + phase * 2.1)
        return hidden * (1 - front) + shown * front

    def ragged(theta, base, phase):
        """A layer's hidden edge is never a straight line: let it wander so any exposed part reads as cloth."""
        return base + 46 * np.sin(3.7 * theta + phase) + 26 * np.sin(9.1 * theta + phase * 2.3) + 14 * np.sin(17.3 * theta + phase * 0.7)

    def hem_inner(theta):
        return measured_hem(theta)

    def hem_mid(theta):
        return measured_layer('mid', 680.0)(theta)

    def hem_outer(theta):
        return measured_layer('outer', 420.0)(theta)

    objs, info = [], {}
    for name, fn, lo, hi, top, shrink, seed, nt, nr, flare, tuck in (
            ('CapeV5_inner', hem_inner, -104, 104, lambda th: layer_top(th, hem_outer, hem_mid, 1.7), 9.0, 11, 1200, 340, 4.0, 34.0),
            ('CapeV5_mid', hem_mid, -104, 104, lambda th: layer_top(th, hem_outer, hem_mid, 4.3), 4.5, 17, 1200, 300, 8.0, 20.0),
            ('CapeV5_outer', hem_outer, -158, 172, 62.0, 0.0, 23, 1100, 300, 12.0, 0.0)):
        obj, rep_ = cloth_layer(name, data, p, fn, math.radians(lo), math.radians(hi), top, SECTION_B, shrink, seed, nt, nr, flare=flare, top_tuck=tuck)
        objs.append(obj)
        info[obj.name] = rep_
    objs.append(scarf(p))
    top_z = (ZREF - (16 + back['box'][1])) * S * MM
    objs.append(collar(top_z, rb * S, rs * S, math.radians(p['collar_tilt_deg'])))
    info['frames'] = data['frames']
    info['collar_radius_px'] = {'back': rb, 'side': rs, 'front': rf}
    return objs, info


# ---------------------------------------------------------------- review
def setup_render(samples):
    scene = bpy.context.scene
    scene.render.engine = 'CYCLES'
    prefs = bpy.context.preferences.addons['cycles'].preferences
    device = 'CPU'
    for kind in ('OPTIX', 'CUDA'):
        try:
            prefs.compute_device_type = kind
            prefs.get_devices()
            if any(d.type == kind for d in prefs.devices):
                for d in prefs.devices:
                    d.use = d.type == kind
                device = 'GPU'
                break
        except Exception:
            continue
    scene.cycles.device = device
    scene.cycles.samples = samples
    scene.cycles.use_denoising = True
    scene.render.film_transparent = True
    scene.render.image_settings.file_format = 'PNG'
    scene.render.image_settings.color_mode = 'RGBA'
    scene.view_settings.view_transform = 'Standard'
    world = bpy.data.worlds.new('CapeV5 studio')
    world.use_nodes = True
    world.node_tree.nodes['Background'].inputs['Color'].default_value = (0.5, 0.48, 0.47, 1)
    world.node_tree.nodes['Background'].inputs['Strength'].default_value = 0.22
    scene.world = world
    return device


def view_camera(name, view, frames, scale):
    """Orthographic camera reproducing one sheet view. Returns (camera object, rig empty)."""
    x0, y0, x1, y1 = view['box']
    w, h = x1 - x0, y1 - y0
    cam = bpy.data.cameras.new(name)
    cam.type = 'ORTHO'
    cam.ortho_scale = max(w, h) * S * MM
    cam.clip_end = 20
    obj = bpy.data.objects.new(name, cam)
    bpy.context.scene.collection.objects.link(obj)
    zc = (ZREF - (y0 + h / 2)) * S * MM
    off = (w / 2 - frames[name]) * S * MM
    if name == 'cb':
        obj.location, obj.rotation_euler = (off, -4, zc), (math.pi / 2, 0, 0)
    elif name == 'cf':
        obj.location, obj.rotation_euler = (-off, 4, zc), (math.pi / 2, 0, math.pi)
    else:
        obj.location, obj.rotation_euler = (4, off, zc), (math.pi / 2, 0, math.pi / 2)
    return obj, (int(w * scale), int(h * scale))


def key_light(cam_obj):
    from mathutils import Vector
    for old in [o for o in bpy.data.objects if o.name.startswith('CapeV5_light')]:
        bpy.data.objects.remove(old, do_unlink=True)
    basis = cam_obj.matrix_world.to_3x3()
    target = Vector((0, 0, 0.55))
    for i, (offset, energy, size) in enumerate((((-1.4, 1.6, 3.0), 85, 2.4), ((2.2, 0.2, 3.0), 18, 3.0), ((0.2, 2.6, 0.6), 40, 2.0))):
        light = bpy.data.lights.new(f'CapeV5_light{i}', 'AREA')
        light.energy, light.size = energy, size
        obj = bpy.data.objects.new(f'CapeV5_light{i}', light)
        obj.location = target + basis @ Vector(offset)
        obj.rotation_euler = (target - obj.location).to_track_quat('-Z', 'Y').to_euler()
        bpy.context.scene.collection.objects.link(obj)


def shoot(path, cam_obj, size, bg):
    scene = bpy.context.scene
    scene.camera = cam_obj
    scene.render.resolution_x, scene.render.resolution_y = size
    bpy.context.view_layer.update()
    key_light(cam_obj)
    scene.render.filepath = str(path)
    bpy.ops.render.render(write_still=True)
    img = bpy.data.images.load(str(path), check_existing=False)
    w, h = img.size
    px = np.empty(w * h * 4, dtype=np.float32)
    img.pixels.foreach_get(px)
    bpy.data.images.remove(img)
    rgba = px.reshape(h, w, 4)[::-1]
    return rgba[..., :3] * 255 * rgba[..., 3:4] + (1 - rgba[..., 3:4]) * bg


def compare(data, cams, out, tag, scale):
    panels = []
    for name, key in (('front', 'cf'), ('side', 'cs'), ('back', 'cb')):
        view = data['views'][name]
        cam, size = cams[key]
        shot = shoot(out / f'{tag}-{name}.png', cam, size, data['bg'])
        ref = view['img']
        k = int(round(scale))
        ref = np.repeat(np.repeat(ref, k, 0), k, 1)[:shot.shape[0], :shot.shape[1]]
        hh = min(ref.shape[0], shot.shape[0])
        panels.append(np.concatenate([ref[:hh], np.full((hh, 6, 3), 20.0), shot[:hh, :ref.shape[1]]], 1))
    hmax = max(pn.shape[0] for pn in panels)
    padded = [np.concatenate([pn, np.tile(data['bg'], (hmax - pn.shape[0], pn.shape[1], 1))], 0) for pn in panels]
    row = padded[0]
    for pn in padded[1:]:
        row = np.concatenate([row, np.full((hmax, 24, 3), 20.0), pn], 1)
    C.write_png(out / f'{tag}-three-views.png', row)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--sheet', required=True)
    ap.add_argument('--output', required=True)
    ap.add_argument('--samples', type=int, default=96)
    ap.add_argument('--params', default='{}')
    ap.add_argument('--hem-tables', default='')
    args = ap.parse_args(sys.argv[sys.argv.index('--') + 1:])
    p = dict(PARAMS)
    p.update(json.loads(args.params))
    out = Path(args.output)
    for obj in list(bpy.data.objects):
        bpy.data.objects.remove(obj, do_unlink=True)
    data = M.measure(args.sheet)
    objs, info = build(data, p)
    clay = bpy.data.materials.new('CapeV5 clay')
    clay.use_nodes = True
    bsdf = clay.node_tree.nodes['Principled BSDF']
    bsdf.inputs['Base Color'].default_value = (0.42, 0.40, 0.38, 1)
    bsdf.inputs['Roughness'].default_value = 0.75
    for obj in objs:
        obj.data.materials.append(clay)
    device = setup_render(args.samples)
    scale = 2
    cams = {key: view_camera(key, data['views'][name], info['frames'], scale) for name, key in (('front', 'cf'), ('side', 'cs'), ('back', 'cb'))}
    bpy.ops.wm.save_as_mainfile(filepath=str(out / 'cape-v5-clay.blend'))
    compare(data, cams, out, 'clay', scale)
    # rear three-quarter check, the angle of the original isolated concept
    from mathutils import Vector
    cam = bpy.data.cameras.new('turn')
    cam.type = 'ORTHO'
    cam.ortho_scale = 1.25
    cam.clip_end = 20
    turn = bpy.data.objects.new('turn', cam)
    bpy.context.scene.collection.objects.link(turn)
    shots = []
    for deg in (35, 145, 215, 325):
        ang = math.radians(deg)
        turn.location = (4 * math.sin(ang), -4 * math.cos(ang), 0.75)
        turn.rotation_euler = (Vector((0, 0, 0.55)) - turn.location).to_track_quat('-Z', 'Y').to_euler()
        shots.append(shoot(out / f'clay-turn{deg:03d}.png', turn, (700, 1100), data['bg']))
    C.write_png(out / 'clay-turn-sheet.png', np.concatenate(shots, 1))
    info['objects'] = {o.name: len(o.data.vertices) for o in objs}
    (out / 'clay-report.json').write_text(json.dumps(info, indent=1, default=float))
    print('DEVICE', device, info['objects'])


if __name__ == '__main__':
    main()
