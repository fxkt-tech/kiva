"""Concept-matched standalone cape (phase A display model).

The concept image is used as a measured drawing: silhouette, fold relief and ember
positions are traced from it in camera space (1 px = 1 mm).  Colour is never projected.
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

W, H = 1024, 1536

# Traced in concept pixels (x right, y down).
COLLAR_VISIBLE = [(424, 8), (654, 8), (654, 64), (646, 67), (600, 96), (540, 121), (476, 128), (436, 103), (424, 80)]
SCARF = [(432, 75), (400, 72), (370, 90), (345, 120), (330, 170), (332, 250), (350, 300), (380, 325), (425, 355),
         (465, 357), (515, 340), (570, 310), (625, 270), (680, 220), (715, 165), (740, 120), (700, 75), (650, 62),
         (646, 67), (600, 96), (540, 121), (476, 128), (436, 103)]
# Scarf fold crests, traced left to right across the visible half of each loop.
SCARF_CRESTS = [
    [(390, 85), (415, 108), (465, 131), (525, 131), (580, 113), (630, 85), (660, 63)],
    [(360, 113), (390, 143), (450, 170), (515, 170), (580, 150), (640, 118), (685, 85), (700, 75)],
    [(335, 145), (360, 180), (425, 210), (500, 215), (570, 198), (640, 160), (695, 125), (730, 108)],
    [(330, 165), (355, 215), (410, 255), (490, 260), (560, 240), (630, 200), (695, 155), (750, 135)],
    [(348, 235), (385, 280), (450, 310), (510, 305), (570, 280), (640, 235), (700, 180), (760, 145)],
    [(365, 300), (400, 340), (445, 357), (500, 345), (560, 315), (625, 270), (680, 220), (720, 175)],
]
SCARF_DEPTH = [120.0, 133.0, 147.0, 158.0, 166.0, 168.0]   # near-point depth radius of each loop, mm
SCARF_BACK_LIMIT = -0.45     # loop columns with cos(theta) below this face away from the concept camera
COLLAR = {'cx': 539.0, 'cy': 40.0, 'radius': 107.0, 'elevation': math.radians(11.6), 'height': 170.0,
          'wall': 9.0, 'placket_deg': -38.8}


def polygon_mask(points):
    ys, xs = np.mgrid[0:H, 0:W].astype(np.float32)
    inside = np.zeros((H, W), bool)
    n = len(points)
    for i in range(n):
        x0, y0 = points[i]
        x1, y1 = points[(i + 1) % n]
        if y0 == y1:
            continue
        cond = ((y0 <= ys) & (ys < y1)) | ((y1 <= ys) & (ys < y0))
        xi = x0 + (ys - y0) * (x1 - x0) / (y1 - y0)
        inside ^= cond & (xs < xi)
    return inside


# ---------------------------------------------------------------- measurement
def measure(concept):
    img = C.load_rgb8(concept)
    r, g, b = img[..., 0], img[..., 1], img[..., 2]
    lum = 0.299 * r + 0.587 * g + 0.114 * b
    border = np.concatenate([img[:40].reshape(-1, 3), img[-40:].reshape(-1, 3)])
    bg = np.median(border, axis=0)
    bg_lum = float(0.299 * bg[0] + 0.587 * bg[1] + 0.114 * bg[2])
    warm = r - b
    ember = (r > 150) & (warm > 70)
    hot = (r > 215) & (g > 120) & (warm > 110)
    raw = (lum < bg_lum - 22) | (warm > 48)
    solid = C.largest_component(raw)
    outside = C.largest_component(~solid)
    mask = ~outside
    # lit scarf folds are as bright as the backdrop and leave notches; close them above the shoulder line
    grown = C.blur(mask.astype(np.float32), 6) > 0.15
    closed = C.blur(grown.astype(np.float32), 6) > 0.85
    upper = np.arange(H)[:, None] < 300
    mask = mask | (closed & upper)
    return {'img': img, 'lum': lum, 'mask': mask, 'ember': ember, 'hot': hot, 'bg': bg, 'warm': warm}


def relief(m, p):
    """Fold relief in mm read from shading (ridges bright, valleys dark).

    Blue is used because ember light barely reaches it; only white-hot cores are inpainted.
    """
    img, mask = m['img'], m['mask']
    cool = img[..., 2]
    core = C.blur(((m['warm'] > 95) & (img[..., 1] > 105)).astype(np.float32), 2.0) > 0.10
    valid = (mask & ~core).astype(np.float32)
    fill = C.masked_blur(cool, valid, 7)
    s = np.where(valid > 0, cool, fill)
    inside = mask.astype(np.float32)
    mean_far = C.masked_blur(s, inside, 70)
    s = np.where(mask, s, mean_far)
    s = s / np.maximum(mean_far, 8.0)
    rel = np.zeros_like(s)
    calm = 1 - 0.75 * C.smoothstep(0.05, 0.45, C.blur((m['warm'] > 60).astype(np.float32), 14))
    for i, (lo, hi, gain) in enumerate(p['bands']):
        band = (C.masked_blur(s, inside, lo) - C.masked_blur(s, inside, hi)) * gain
        rel += band * (calm if i < 2 else 0.5 + 0.5 * calm)
    rel = np.clip(rel, -p['clip'], p['clip'])
    return rel * mask, s


def smooth_polyline(points, rounds=3):
    pts = np.array(points, dtype=np.float64)
    for _ in range(rounds):
        q = 0.75 * pts[:-1] + 0.25 * pts[1:]
        r = 0.25 * pts[:-1] + 0.75 * pts[1:]
        mid = np.empty((2 * len(q), 2))
        mid[0::2], mid[1::2] = q, r
        pts = np.vstack([pts[:1], mid, pts[-1:]])
    return pts


def feather_sheets(m, p, edges):
    """Independent feathers laid over the cloak like shingles.

    Each traced line is the free, glowing lower-right edge of one feather, from where it
    first shows to its tip.  The feather is a whole leaf lying on the upper-left side of
    that edge: pointed at the root, widest mid-way, narrowing to the tip.  It rises off the
    cloak steadily from root to tip and is tilted so its free edge is the high side, so a
    feather rooted further up always lies over the ones rooted below it.
    """
    mask = m['mask']
    open_air = C.blur((~mask).astype(np.float32), 6)
    ys_full = np.arange(H, dtype=np.float32)[:, None]
    feathers = []
    for index, (key, raw) in enumerate(edges.items()):
        pts = smooth_polyline(raw, 2)
        ex, ey = int(np.clip(pts[-1, 0], 0, W - 1)), int(np.clip(pts[-1, 1], 0, H - 1))
        tip_free = bool(open_air[ey, ex] > 0.08)
        lead = pts[0] - pts[2]
        lead /= max(float(np.hypot(*lead)), 1e-6)
        pts = np.vstack([pts[0] + lead * p['root_px'], pts])          # hidden root upstream of the visible edge
        if tip_free:
            direction = pts[-1] - pts[-3]
            direction /= max(float(np.hypot(*direction)), 1e-6)
            pts = np.vstack([pts, pts[-1] + direction * 35, pts[-1] + direction * 70])
        seg = np.hypot(*(pts[1:] - pts[:-1]).T)
        arc = np.concatenate([[0], np.cumsum(seg)])
        length = arc[-1]
        wmax = min(p['feather_width'], 0.42 * length)
        pad = wmax + 12
        x0 = max(0, int(pts[:, 0].min() - pad))
        x1 = min(W, int(pts[:, 0].max() + pad))
        y0 = max(0, int(pts[:, 1].min() - pad))
        y1 = min(H, int(pts[:, 1].max() + pad))
        ys, xs = np.mgrid[y0:y1, x0:x1].astype(np.float32)
        best = np.full(xs.shape, 1e9, np.float32)
        side = np.zeros(xs.shape, np.float32)
        along = np.zeros(xs.shape, np.float32)
        capped = np.ones(xs.shape, bool)
        last = len(pts) - 2
        for i in range(len(pts) - 1):
            ax, ay = pts[i]
            dx, dy = pts[i + 1] - pts[i]
            ll = max(dx * dx + dy * dy, 1e-9)
            tr = ((xs - ax) * dx + (ys - ay) * dy) / ll
            tc = np.clip(tr, 0, 1)
            dist = np.hypot(xs - ax - tc * dx, ys - ay - tc * dy)
            take = dist < best
            best = np.where(take, dist, best)
            cross = (xs - ax) * dy - (ys - ay) * dx
            side = np.where(take, -np.sign(cross), side)      # + = image upper-left of the hem = the feather
            along = np.where(take, arc[i] + tc * seg[i], along)
            inner = ((tr >= 0) | (i > 0)) & ((tr <= 1) | (i < last))
            capped = np.where(take, ~inner, capped)
        t = along / length
        narrow = C.smoothstep(0.72, 1.0, t) ** 1.4 * (0.5 if tip_free else 1.0)
        width = wmax * np.maximum(C.smoothstep(0.0, 0.32, t) ** 0.7 * (1 - narrow), 0.0)
        teeth = 3.0 * (np.abs(((along / 15.0) % 1.0) - 0.5) * 2 - 0.5) + 1.5 * np.sin(along / 37.0 + index)
        signed = side * best + teeth * np.clip(best / 3.0, 0.3, 1)
        # the hidden upper edge is toothed too, a little coarser
        far = width + 2.5 * np.sin(along / 11.0 + 2 * index) - 2.5
        inside = (signed > 0) & ~capped & (best < far)
        tail = 0.4 + 0.6 * C.smoothstep(700, 1350, ys)
        rise = p['rise_mm_per_px'] * along * C.smoothstep(0.0, 0.18, t)
        tilt = p['tilt_mm_per_px'] * (width * 0.5 - best)
        flare = p['tip_flare_mm'] * tail * t ** 3
        height = np.maximum(rise + tilt + flare, p['min_lift_mm'] * C.smoothstep(0.0, 0.1, t))
        hem = C.smoothstep(0.0, 0.3, 1 - best / np.maximum(width, 1e-3))
        height = height * hem - 2.6 * (1 - hem)
        full_hem = np.zeros((H, W), np.float32)
        full_hem[y0:y1, x0:x1] = np.where(inside, hem, 0)
        d_mid = best - 0.45 * width
        rib = np.exp(-(d_mid / 1.6) ** 2)
        phase = (along - 1.1 * np.abs(d_mid)) / 15.0
        barb = np.exp(-((np.abs((phase % 1.0) - 0.5) - 0.5) / 0.06) ** 2) * (np.abs(d_mid) < width * 0.42)
        full_v = np.zeros((H, W), np.float32)
        full_v[y0:y1, x0:x1] = np.where(inside, np.maximum(rib, 0.7 * barb), 0)
        full_in = np.zeros((H, W), bool)
        full_in[y0:y1, x0:x1] = inside
        full_h = np.zeros((H, W), np.float32)
        full_h[y0:y1, x0:x1] = np.where(inside | (best < far + 4), height, 0)
        feathers.append({'key': key, 'root_y': float(raw[0][1]), 'inside': full_in, 'height': full_h, 'vein': full_v, 'hem': full_hem, 'length_px': float(length), 'width_px': float(wmax), 'tip_free': tip_free})
    return feathers


def apply_tiers(feathers, p):
    """Feathers rooted in the same height band form one layer; upper layers lie over lower ones."""
    bands = p['tier_bands']
    tiers = {}
    for feather in feathers:
        tier = sum(feather['root_y'] >= b for b in bands)
        feather['tier'] = tier
        feather['height'] = feather['height'] + (len(bands) - tier) * p['tier_gap_mm'] * feather['hem']
        tiers.setdefault(tier, []).append(feather['key'])
    for tier in sorted(tiers):
        print('TIER', tier + 1, len(tiers[tier]), tiers[tier])
    return tiers


def flutter(p):
    """Low-frequency flare so separate tail tongues leave the surface by different amounts."""
    rng = np.random.default_rng(11)
    small = rng.normal(size=(H // 64 + 3, W // 64 + 3)).astype(np.float32)
    big = np.kron(small, np.ones((64, 64), np.float32))[:H, :W]
    big = C.blur(big, 30)
    big /= max(float(np.abs(big).max()), 1e-6)
    ys = np.arange(H, dtype=np.float32)[:, None]
    return p['flutter_mm'] * big * C.smoothstep(850, 1400, ys)


def row_profile(mask):
    """Smoothed centre line and half width of the silhouette per image row."""
    xl = np.zeros(H, np.float32)
    xr = np.zeros(H, np.float32)
    have = np.zeros(H, bool)
    for y in range(H):
        xs = np.nonzero(mask[y])[0]
        if len(xs) > 20:
            lo, hi = np.percentile(xs, [1.5, 98.5])
            xl[y], xr[y], have[y] = lo, hi, True
    idx = np.nonzero(have)[0]
    xl = np.interp(np.arange(H), idx, xl[idx])
    xr = np.interp(np.arange(H), idx, xr[idx])
    k = 45
    ker = np.hanning(2 * k + 1)
    ker /= ker.sum()
    xl = np.convolve(np.pad(xl, k, mode='edge'), ker, mode='valid')
    xr = np.convolve(np.pad(xr, k, mode='edge'), ker, mode='valid')
    return (xl + xr) * 0.5, (xr - xl) * 0.5


def macro_depth(m, p):
    """Cloth wrapped round an unseen back: elliptical section per row, flattening as it hangs free."""
    xc, hw = row_profile(m['mask'])
    ys = np.arange(H, dtype=np.float32)[:, None]
    xs = np.arange(W, dtype=np.float32)[None, :]
    t = (xs - xc[:, None]) / (hw[:, None] * 1.05)
    section = np.maximum(0.0, 1 - t * t) ** 0.62
    amp = p['hang_mm'] + (p['body_mm'] - p['hang_mm']) * (1 - C.smoothstep(330, 980, ys))
    z = section * amp
    z = z + p['billow_mm'] * np.sin((ys - 500) / 300.0 + (xs - 500) / 420.0) * C.smoothstep(450, 900, ys)
    z = z + 0.6 * p['billow_mm'] * np.sin((xs - 300) / 95.0 - (ys - 600) / 260.0) * C.smoothstep(600, 1000, ys) * section
    # under the real scarf the cloak continues toward the neck, hidden
    scarf = C.blur(polygon_mask(SCARF).astype(np.float32), 9)
    z = z - p['under_scarf_mm'] * C.smoothstep(0.35, 0.95, scarf)
    return z


def scarf_mesh(relief_mm, p):
    """Cowl of stacked loops round the neck. Visible half follows the traced crests exactly."""
    seg, sub = 320, 14
    loops = []
    thetas = np.linspace(-math.pi, math.pi, seg, endpoint=False)
    for crest, b in zip(SCARF_CRESTS, SCARF_DEPTH):
        pts = np.array(crest, dtype=np.float64)
        xl, xr = pts[0, 0] - 3, pts[-1, 0] + 3
        cx, a = (xl + xr) / 2, (xr - xl) / 2
        st, ct = np.sin(thetas), np.cos(thetas)
        x = cx + a * st
        y_vis = np.interp(x, pts[:, 0], pts[:, 1])
        frac = (st + 1) / 2
        y_hid = pts[0, 1] + (pts[-1, 1] - pts[0, 1]) * frac - 14 * np.abs(ct) - 10 * np.sin(2 * thetas + b)
        w = C.smoothstep(-0.12, 0.12, ct)
        y = y_hid * (1 - w) + y_vis * w
        loops.append({'x': x, 'y': y, 'd': b * ct * np.where(ct > 0, 1.0, 0.86), 'st': st, 'ct': ct})
    c = COLLAR
    st, ct = np.sin(thetas), np.cos(thetas)
    first, last = loops[0], loops[-1]
    inner = {'x': c['cx'] + (c['radius'] + 5) * st, 'y': first['y'] - 14 + 5 * st, 'd': (c['radius'] + 5) * ct, 'st': st, 'ct': ct}
    under = {'x': last['x'] - 34 * st, 'y': last['y'] - 4, 'd': last['d'] - 40 * ct, 'st': st, 'ct': ct}
    stack = [inner] + loops + [under]
    rows = []
    for k in range(len(stack) - 1):
        u, v = stack[k], stack[k + 1]
        gap = float(np.mean(np.hypot(v['y'] - u['y'], v['d'] - u['d'])))
        valley = p['valley'] * gap if 0 < k < len(stack) - 2 else 0.0
        ts = np.linspace(0, 1, sub, endpoint=False) if k < len(stack) - 2 else np.linspace(0, 1, sub + 1)
        mid = 0 < k < len(stack) - 2
        for tt in ts:
            off = lift = 0.0
            if mid:
                if tt < 0.72:
                    roll = math.sin(math.pi * tt / 0.72)
                    off = p['roll'] * gap * roll
                    lift = p['droop'] * gap * roll * (tt / 0.72)
                else:
                    crev = math.sin(math.pi * (tt - 0.72) / 0.28) ** 2
                    off = -valley * crev
                    lift = -0.30 * gap * crev
            e = tt * tt * (3 - 2 * tt) if not mid else tt
            x = u['x'] * (1 - e) + v['x'] * e + off * st
            y = u['y'] * (1 - e) + v['y'] * e + lift
            d = u['d'] * (1 - e) + v['d'] * e + off * ct
            rows.append((x, y, d))
    X = np.stack([r[0] for r in rows])
    Y = np.stack([r[1] for r in rows])
    D = np.stack([r[2] for r in rows])
    # secondary wrinkles on the visible half are read from the concept shading
    vis = C.smoothstep(0.1, 0.45, np.broadcast_to(ct, X.shape))
    wr = C.bilinear(relief_mm, X, Y) * vis * p['scarf_relief']
    X = X + wr * st * 0.0
    D = D + wr
    co = np.stack([(X - W / 2) * C.MM, -D * C.MM, (H - Y) * C.MM], -1).reshape(-1, 3)
    nr = X.shape[0]
    ii, jj = np.meshgrid(np.arange(nr - 1), np.arange(seg), indexing='ij')
    jn = (jj + 1) % seg
    quads = np.stack([ii * seg + jj, (ii + 1) * seg + jj, (ii + 1) * seg + jn, ii * seg + jn], -1).reshape(-1, 4)
    hidden = (ct[jj.ravel()] < SCARF_BACK_LIMIT) & (ct[jn.ravel()] < SCARF_BACK_LIMIT)
    quads = quads[hidden]
    mesh = bpy.data.meshes.new('CapeV4_scarf')
    mesh.vertices.add(len(co))
    mesh.vertices.foreach_set('co', co.ravel())
    mesh.loops.add(quads.size)
    mesh.loops.foreach_set('vertex_index', quads.ravel())
    mesh.polygons.add(len(quads))
    mesh.polygons.foreach_set('loop_start', np.arange(0, quads.size, 4))
    mesh.polygons.foreach_set('loop_total', np.full(len(quads), 4))
    mesh.polygons.foreach_set('use_smooth', np.ones(len(quads), bool))
    mesh.update(calc_edges=True)
    obj = bpy.data.objects.new('CapeV4_scarf', mesh)
    bpy.context.scene.collection.objects.link(obj)
    sub_mod = obj.modifiers.new('Smooth', 'SUBSURF')
    sub_mod.levels = sub_mod.render_levels = 1
    return obj


def collar_mesh(depth_center_mm):
    """Real stand collar: lathe with a rolled rim, tilted to the measured view elevation."""
    c = COLLAR
    R, wall, hgt = c['radius'], c['wall'], c['height']
    profile = [(R + 1.5, -hgt), (R + 0.6, -hgt * 0.6), (R, -24), (R - 0.6, -15), (R + 0.2, -13.5), (R + 0.6, -9),
               (R + 0.3, -4), (R - 1.2, -1.2), (R - wall * 0.5, 0.0), (R - wall + 1.2, -1.2), (R - wall, -4.5),
               (R - wall - 0.4, -14), (R - wall, -hgt * 0.6), (R - wall + 1, -hgt)]
    seg = 192
    e = c['elevation']
    up = np.array([0.0, -math.sin(e), math.cos(e)])
    fwd = np.array([0.0, -math.cos(e), -math.sin(e)])       # toward the camera
    right = np.array([1.0, 0.0, 0.0])
    origin = np.array([(c['cx'] - W / 2) * C.MM, -depth_center_mm * C.MM, (H - c['cy']) * C.MM])
    placket = math.radians(c['placket_deg'])
    rng = np.random.default_rng(7)
    phase = rng.uniform(0, 6.28, 6)
    verts = []
    for i in range(seg):
        th = 2 * math.pi * i / seg
        d = (th - placket + math.pi) % (2 * math.pi) - math.pi
        strip = 1.6 * float(C.smoothstep(0.085, 0.06, abs(d)))
        for j, (r, hh) in enumerate(profile):
            outer = j < 8
            wob = 0.0
            if hh < -20:
                t = min(1.0, (-hh - 20) / 60)
                wob = t * (1.8 * math.sin(3 * th + phase[0]) + 1.2 * math.sin(7 * th + phase[1] + hh * 0.05)
                           + 0.8 * math.sin(13 * th + phase[2]))
            rr = r + (strip if outer and hh < -3 else 0) + wob
            hz = hh + 1.4 * math.sin(2 * th + phase[3]) + 0.8 * math.sin(5 * th + phase[4])
            pos = origin + (right * math.sin(th) * rr + fwd * math.cos(th) * rr + up * hz) * C.MM
            verts.append(pos)
    n = len(profile)
    faces = []
    for i in range(seg):
        a0 = i * n
        b0 = ((i + 1) % seg) * n
        for j in range(n - 1):
            faces.append((a0 + j, b0 + j, b0 + j + 1, a0 + j + 1))
    mesh = bpy.data.meshes.new('CapeV4_collar')
    mesh.from_pydata([tuple(v) for v in verts], [], faces)
    for poly in mesh.polygons:
        poly.use_smooth = True
    mesh.update()
    obj = bpy.data.objects.new('CapeV4_collar', mesh)
    bpy.context.scene.collection.objects.link(obj)
    sub = obj.modifiers.new('Smooth', 'SUBSURF')
    sub.levels = sub.render_levels = 2
    return obj


# ---------------------------------------------------------------- mesh
def sheet_mesh(name, field, depth_mm, step, attrs=None, cut=None, cut_limit=0.6):
    """Grid over field>0.5 with boundary verts snapped to the 0.5 iso-contour."""
    gx = np.arange(0, W - 1, step, dtype=np.float32)
    gy = np.arange(0, H - 1, step, dtype=np.float32)
    X, Y = np.meshgrid(gx, gy)
    F = C.bilinear(field, X, Y)
    cell = (F[:-1, :-1] + F[1:, :-1] + F[:-1, 1:] + F[1:, 1:]) * 0.25 > 0.5
    cell &= (np.minimum(np.minimum(F[:-1, :-1], F[1:, :-1]), np.minimum(F[:-1, 1:], F[1:, 1:])) > 0.2)
    if cut is not None:
        Q = C.bilinear(cut, X, Y)
        corners = np.stack([Q[:-1, :-1], Q[1:, :-1], Q[:-1, 1:], Q[1:, 1:]])
        cell &= (corners.max(0) - corners.min(0)) < cut_limit
    used = np.zeros(F.shape, bool)
    used[:-1, :-1] |= cell
    used[1:, :-1] |= cell
    used[:-1, 1:] |= cell
    used[1:, 1:] |= cell
    index = -np.ones(F.shape, np.int64)
    index[used] = np.arange(int(used.sum()))
    px = X[used].copy()
    py = Y[used].copy()
    # snap boundary vertices toward the contour
    interior = np.zeros(F.shape, bool)
    full = np.zeros((F.shape[0] + 1, F.shape[1] + 1), bool)
    full[1:-1, 1:-1] = cell
    interior = full[:-1, :-1] & full[1:, :-1] & full[:-1, 1:] & full[1:, 1:]
    edge = used & ~interior
    gyf, gxf = np.gradient(field)
    for _ in range(3):
        f = C.bilinear(field, px, py)
        dx = C.bilinear(gxf, px, py)
        dy = C.bilinear(gyf, px, py)
        n2 = dx * dx + dy * dy
        move = np.where(edge[used] & (n2 > 1e-5), (f - 0.5) / np.maximum(n2, 1e-5), 0)
        move = np.clip(move, -step * 0.9 / np.sqrt(np.maximum(n2, 1e-5)), step * 0.9 / np.sqrt(np.maximum(n2, 1e-5)))
        px -= move * dx
        py -= move * dy
    pz = C.bilinear(depth_mm, px, py)
    co = np.stack([(px - W / 2) * C.MM, -pz * C.MM, (H - py) * C.MM], 1)
    cy, cx = np.nonzero(cell)
    quads = np.stack([index[cy, cx], index[cy + 1, cx], index[cy + 1, cx + 1], index[cy, cx + 1]], 1)
    mesh = bpy.data.meshes.new(name)
    mesh.vertices.add(len(co))
    mesh.vertices.foreach_set('co', co.ravel())
    mesh.loops.add(quads.size)
    mesh.loops.foreach_set('vertex_index', quads.ravel())
    mesh.polygons.add(len(quads))
    mesh.polygons.foreach_set('loop_start', np.arange(0, quads.size, 4))
    mesh.polygons.foreach_set('loop_total', np.full(len(quads), 4))
    mesh.polygons.foreach_set('use_smooth', np.ones(len(quads), bool))
    mesh.update(calc_edges=True)
    mesh.validate()
    uv = mesh.uv_layers.new(name='UVMap')
    uvs = np.stack([px / W, 1 - py / H], 1)[quads.ravel()]
    uv.data.foreach_set('uv', uvs.ravel())
    for key, arr in (attrs or {}).items():
        a = mesh.attributes.new(key, 'FLOAT', 'POINT')
        a.data.foreach_set('value', C.bilinear(arr.astype(np.float32), px, py))
    obj = bpy.data.objects.new(name, mesh)
    bpy.context.scene.collection.objects.link(obj)
    return obj


# ---------------------------------------------------------------- review
def clay_material():
    mat = bpy.data.materials.new('CapeV4 clay')
    mat.use_nodes = True
    bsdf = mat.node_tree.nodes['Principled BSDF']
    bsdf.inputs['Base Color'].default_value = (0.42, 0.40, 0.38, 1)
    bsdf.inputs['Roughness'].default_value = 0.75
    return mat


def setup_render(samples=96):
    scene = bpy.context.scene
    scene.render.engine = 'CYCLES'
    prefs = bpy.context.preferences.addons['cycles'].preferences
    device = 'CPU'
    for kind in ('OPTIX', 'CUDA'):
        try:
            prefs.compute_device_type = kind
            prefs.get_devices()
            gpus = [d for d in prefs.devices if d.type == kind]
            if gpus:
                for d in prefs.devices:
                    d.use = d.type == kind
                device = 'GPU'
                break
        except Exception:
            continue
    scene.cycles.device = device
    scene.cycles.samples = samples
    scene.cycles.use_denoising = True
    scene.render.resolution_x = W
    scene.render.resolution_y = H
    scene.render.film_transparent = False
    scene.render.image_settings.file_format = 'PNG'
    scene.view_settings.view_transform = 'Standard'
    return device


def concept_camera():
    cam = bpy.data.cameras.new('ConceptCam')
    cam.type = 'ORTHO'
    cam.ortho_scale = H * C.MM
    cam.clip_end = 20
    obj = bpy.data.objects.new('ConceptCam', cam)
    obj.location = (0, -4, H * C.MM / 2)
    obj.rotation_euler = (math.pi / 2, 0, 0)
    bpy.context.scene.collection.objects.link(obj)
    bpy.context.scene.camera = obj
    return obj


def studio(bg):
    world = bpy.data.worlds.new('CapeV4 studio')
    world.use_nodes = True
    world.node_tree.nodes['Background'].inputs['Color'].default_value = (0.16, 0.13, 0.13, 1)
    world.node_tree.nodes['Background'].inputs['Strength'].default_value = 0.35
    bpy.context.scene.world = world
    def area(name, loc, size, energy, color=(1, 1, 1)):
        light = bpy.data.lights.new(name, 'AREA')
        light.size = size
        light.energy = energy
        light.color = color
        obj = bpy.data.objects.new(name, light)
        obj.location = loc
        bpy.context.scene.collection.objects.link(obj)
        direction = np.array([0, 0, 0.8]) - np.array(loc)
        from mathutils import Vector
        obj.rotation_euler = Vector(direction).to_track_quat('-Z', 'Y').to_euler()
        return obj
    area('Key', (-1.6, -3.0, 2.9), 2.4, 105)
    area('Fill', (2.4, -2.6, 1.2), 3.0, 30)
    area('Top', (0.2, -0.9, 3.6), 1.6, 75)
    # backdrop
    bpy.ops.mesh.primitive_plane_add(size=12, location=(0, 1.2, 0.8), rotation=(math.pi / 2, 0, 0))
    plane = bpy.context.object
    plane.name = 'Backdrop'
    mat = bpy.data.materials.new('Backdrop')
    mat.use_nodes = True
    mat.node_tree.nodes['Principled BSDF'].inputs['Base Color'].default_value = (0.165, 0.127, 0.124, 1)
    mat.node_tree.nodes['Principled BSDF'].inputs['Roughness'].default_value = 1
    plane.data.materials.append(mat)
    return plane


def render(path):
    bpy.context.scene.render.filepath = str(path)
    bpy.ops.render.render(write_still=True)


def side_by_side(concept_rgb, render_path, out_path):
    shot = C.load_rgb8(render_path)
    gap = np.full((H, 12, 3), 20.0)
    C.write_png(out_path, np.concatenate([concept_rgb, gap, shot], 1))


# ---------------------------------------------------------------- main
PARAMS = {
    'step': 1.25,
    'bands': [(3.5, 10, 0.0), (9, 30, 26.0), (24, 80, 130.0)],
    'lip_shift': 3,
    'lip_mm': 20.0,
    'feather_width': 92.0,
    'tier_bands': [650, 850, 1000, 1130],
    'tier_gap_mm': 1.6,
    'feather_step': 1.0,
    'root_px': 40.0,
    'rise_mm_per_px': 0.05,
    'tilt_mm_per_px': 0.05,
    'tip_flare_mm': 9.0,
    'min_lift_mm': 0.9,
    'tuck_px': 26.0,
    'tuck_mm_under': 3.4,
    'lip_sharp': 9.0,
    'lip_reach': 12.0,
    'lip_gain': 9.0,
    'flutter_mm': 24.0,
    'roll': 0.22,
    'droop': 0.16,
    'scarf_mm': 38.0,
    'scarf_front_relief': 0.6,
    'tuck_mm': 30.0,
    'leak': 0.965,
    'top_light': 0.0,
    'clip': 55.0,
    'body_mm': 140.0,
    'hang_mm': 75.0,
    'billow_mm': 16.0,
    'under_scarf_mm': 55.0,
    'valley': 0.75,
    'scarf_relief': 0.5,
}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--concept', required=True)
    ap.add_argument('--output', required=True)
    ap.add_argument('--step', type=float, default=PARAMS['step'])
    ap.add_argument('--samples', type=int, default=96)
    ap.add_argument('--tag', default='clay')
    ap.add_argument('--turn', action='store_true')
    ap.add_argument('--params', default='{}')
    args = ap.parse_args(sys.argv[sys.argv.index('--') + 1:])
    p = dict(PARAMS)
    p.update(json.loads(args.params))
    p['step'] = args.step
    out = Path(args.output)
    for obj in list(bpy.data.objects):
        bpy.data.objects.remove(obj, do_unlink=True)
    m = measure(args.concept)
    rel, shade = relief(m, p)
    edges = json.loads((out / 'feathers.json').read_text())
    collar_cut = polygon_mask(COLLAR_VISIBLE)
    scarf_cut = C.blur(polygon_mask(SCARF).astype(np.float32), 10) > 0.8
    cloth = m['mask'] & ~collar_cut & ~scarf_cut
    field = C.blur(cloth.astype(np.float32), 1.0)
    feathers = feather_sheets(m, p, edges)
    apply_tiers(feathers, p)
    surface = macro_depth(m, p) + flutter(p) + rel
    C.write_png(out / 'analysis' / 'relief.png', 128 + rel * 4)
    clay = clay_material()
    pieces = []
    report = {'pieces': {}}
    body = sheet_mesh('CapeV4_body', field, surface, p['step'])
    body.data.materials.append(clay)
    solid = body.modifiers.new('Thickness', 'SOLIDIFY')
    solid.thickness = 0.0028
    solid.offset = -1
    pieces.append(body)
    report['pieces'][body.name] = {'vertices': len(body.data.vertices), 'quads': len(body.data.polygons)}
    for feather in feathers:
        name = f"CapeV4_feather_{feather['key']}"
        region = feather['inside'] & cloth
        if int(region.sum()) < 200:
            print('SKIP', name)
            continue
        obj = sheet_mesh(name, np.minimum(C.blur(region.astype(np.float32), 1.0), field), surface + feather['height'], p['feather_step'])
        obj.data.materials.append(clay)
        solid = obj.modifiers.new('Thickness', 'SOLIDIFY')
        solid.thickness = 0.0016
        solid.offset = 1
        pieces.append(obj)
        obj['kiva_tier'] = feather['tier'] + 1
        report['pieces'][name] = {'tier': feather['tier'] + 1, 'area_px': int(region.sum()), 'length_mm': round(feather['length_px']), 'max_width_mm': round(feather['width_px']),
                                  'vertices': len(obj.data.vertices), 'quads': len(obj.data.polygons)}
    collar = collar_mesh(0.0)
    collar.data.materials.append(clay)
    scarf = scarf_mesh(rel, p)
    scarf.data.materials.append(clay)
    # the half of the scarf the concept shows: silhouette and rolls read straight from the image
    scarf_poly = polygon_mask(SCARF)
    wrap = C.blur(scarf_poly.astype(np.float32), 9)
    near_collar = C.blur(collar_cut.astype(np.float32), 7)
    front_depth = (surface + (p['under_scarf_mm'] + p['scarf_mm']) * C.smoothstep(0.35, 0.95, wrap)
                   + p['scarf_front_relief'] * rel - p['tuck_mm'] * C.smoothstep(0.02, 0.5, near_collar))
    scarf_region = (C.blur(scarf_poly.astype(np.float32), 8) > 0.12) & (np.arange(H)[:, None] < 372)
    front_field = C.blur((scarf_region & m['mask'] & ~collar_cut).astype(np.float32), 1.0)
    scarf_front = sheet_mesh('CapeV4_scarf_front', front_field, front_depth, p['step'])
    scarf_front.data.materials.append(clay)
    solid = scarf_front.modifiers.new('Thickness', 'SOLIDIFY')
    solid.thickness = 0.0030
    solid.offset = -1
    for obj in (collar, scarf, scarf_front):
        report['pieces'][obj.name] = {'vertices': len(obj.data.vertices), 'quads': len(obj.data.polygons)}
    (out / 'clay-report.json').write_text(json.dumps(report, indent=1))
    device = setup_render(args.samples)
    studio(m['bg'])
    cam = concept_camera()
    bpy.ops.wm.save_as_mainfile(filepath=str(out / 'zhuque-cape-v4-clay.blend'))
    shot = out / f'{args.tag}-concept-view.png'
    render(shot)
    side_by_side(m['img'], shot, out / f'{args.tag}-comparison.png')
    if args.turn:
        pivot = bpy.data.objects.new('Pivot', None)
        pivot.location = (0, -0.08, H * C.MM / 2)
        bpy.context.scene.collection.objects.link(pivot)
        cam.parent = pivot
        cam.location = (0, -4, 0)
        bpy.data.objects['Backdrop'].hide_render = True
        bpy.context.scene.world.node_tree.nodes['Background'].inputs['Color'].default_value = (0.165, 0.127, 0.124, 1)
        shots = []
        for deg in (-60, -30, 30, 60, 180):
            pivot.rotation_euler = (0, 0, math.radians(deg))
            path = out / f'{args.tag}-turn{deg:+04d}.png'
            render(path)
            shots.append(C.load_rgb8(path)[::2, ::2])
        C.write_png(out / f'{args.tag}-turn-sheet.png', np.concatenate(shots, 1))
    # proof of independence: one flat colour per piece, same camera
    if args.turn:
        bpy.data.objects['Pivot'].rotation_euler = (0, 0, 0)
    rng = np.random.default_rng(5)
    for obj in pieces:
        mat = bpy.data.materials.new(obj.name + ' id')
        mat.use_nodes = True
        tier_palette = [(0.85, 0.25, 0.25), (0.9, 0.6, 0.15), (0.85, 0.85, 0.2), (0.25, 0.75, 0.35), (0.2, 0.6, 0.9), (0.6, 0.35, 0.85)]
        base = np.array(tier_palette[(obj.get('kiva_tier', 1) - 1) % len(tier_palette)])
        hue = np.clip(base + rng.uniform(-0.1, 0.1, 3), 0, 1)
        mat.node_tree.nodes['Principled BSDF'].inputs['Base Color'].default_value = (*(hue if obj.name not in ('CapeV4_body', 'CapeV4_base') else (0.4, 0.4, 0.4)), 1)
        obj.data.materials[0] = mat
    render(out / f'{args.tag}-pieces.png')
    print('DEVICE', device, 'pieces', len(pieces), 'verts', sum(len(o.data.vertices) for o in pieces))


if __name__ == '__main__':
    main()
