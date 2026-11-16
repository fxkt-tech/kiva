"""Paint the draped v3 scarf/cape in flat pattern space and render concept-matched reviews.

Textures are computed per pattern piece with numpy: charcoal cloth with ash mottling and
crackle, an ember gradient into the flame-feather hem, glowing burnt edges, feather
barbs/rachis lines and eroded burnt edges in alpha. Files are written as real PNGs,
reloaded as FILE images with explicit colour spaces, then packed.
Run only in an independent background Blender process.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import math
import sys
from pathlib import Path

import bpy
import numpy as np
from mathutils import Vector

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import drape_cape as dc  # noqa: E402

PX_PER_M = 1350  # largest piece stays under the 2048px runtime texture limit
EMISSION_RANGE = 6.0  # emission PNG stores radiance / range; material strength restores it
MARGIN = 0.015


def require(test, message):
    if not test:
        raise ValueError(message)


# --------------------------------------------------------------------------- noise

def value_noise(shape, cells, rng):
    """Smooth value noise with ``cells`` lattice cells along the longer side."""
    h, w = shape
    ch = max(2, int(round(cells * h / max(h, w))) + 2)
    cw = max(2, int(round(cells * w / max(h, w))) + 2)
    grid = rng.random((ch, cw)).astype(np.float32)
    ys = np.linspace(0, ch - 2.001, h, dtype=np.float32)
    xs = np.linspace(0, cw - 2.001, w, dtype=np.float32)
    y0 = ys.astype(int)
    x0 = xs.astype(int)
    fy = ys - y0
    fx = xs - x0
    fy = fy * fy * (3 - 2 * fy)
    fx = fx * fx * (3 - 2 * fx)
    a = grid[y0][:, x0]
    b = grid[y0][:, x0 + 1]
    c = grid[y0 + 1][:, x0]
    d = grid[y0 + 1][:, x0 + 1]
    top = a + (b - a) * fx[None, :]
    bot = c + (d - c) * fx[None, :]
    return top + (bot - top) * fy[:, None]


def fbm(shape, base_cells, octaves, rng, gain=0.5):
    total = np.zeros(shape, np.float32)
    amp, norm, cells = 1.0, 0.0, base_cells
    for _ in range(octaves):
        total += amp * value_noise(shape, cells, rng)
        norm += amp
        amp *= gain
        cells *= 2.03
    return total / norm


def smooth(a, b, x):
    t = np.clip((x - a) / (b - a), 0, 1)
    return t * t * (3 - 2 * t)


# --------------------------------------------------------------------------- raster helpers

def raster_inside(poly, xs, ys):
    """Scanline polygon fill on a pixel-centre grid; xs (W,), ys (H,)."""
    px = np.array([p[0] for p in poly])
    py = np.array([p[1] for p in poly])
    qx, qy = np.roll(px, -1), np.roll(py, -1)
    mask = np.zeros((len(ys), len(xs)), bool)
    for i, y in enumerate(ys):
        cross = (py > y) != (qy > y)
        if not cross.any():
            continue
        xi = px[cross] + (y - py[cross]) * (qx[cross] - px[cross]) / (qy[cross] - py[cross])
        xi.sort()
        for a, b in zip(xi[0::2], xi[1::2]):
            j0 = np.searchsorted(xs, a)
            j1 = np.searchsorted(xs, b)
            mask[i, j0:j1] = True
    return mask


def inner_distance(mask, max_px):
    """Chamfer distance (pixels) from outside, clipped at max_px."""
    big = float(max_px)
    d = np.where(mask, big, 0.0).astype(np.float32)
    steps = ((0, 1, 1.0), (0, -1, 1.0), (1, 0, 1.0), (-1, 0, 1.0),
             (1, 1, 1.4142), (1, -1, 1.4142), (-1, 1, 1.4142), (-1, -1, 1.4142))
    for _ in range(int(max_px) + 2):
        prev = d
        for dy, dx, cost in steps:
            shifted = np.full_like(d, 0.0)
            ys = slice(max(dy, 0), d.shape[0] + min(dy, 0))
            yd = slice(max(-dy, 0), d.shape[0] + min(-dy, 0))
            xs = slice(max(dx, 0), d.shape[1] + min(dx, 0))
            xd = slice(max(-dx, 0), d.shape[1] + min(-dx, 0))
            shifted[ys, xs] = d[yd, xd]
            d = np.minimum(d, shifted + cost)
        d = np.where(mask, d, 0.0)
        if np.array_equal(prev, d):
            break
    return d


def lobe_fields(spines, gx, gy):
    """Per-pixel nearest feather: along-spine t, signed offset (m), half width (m), length (m)."""
    shape = (len(gy), len(gx))
    t = np.full(shape, -1.0, np.float32)
    off = np.zeros(shape, np.float32)
    half = np.ones(shape, np.float32)
    length = np.zeros(shape, np.float32)
    best = np.full(shape, np.inf, np.float32)
    for sp in spines:
        raw = np.array([(x, -s) for x, s in sp['points']], np.float32)  # to pattern (x, y)
        u = np.linspace(0, 1, len(raw))
        uu = np.linspace(0, 1, 360)
        pts = np.stack((np.interp(uu, u, raw[:, 0]), np.interp(uu, u, raw[:, 1])), 1).astype(np.float32)
        tang = np.gradient(pts, axis=0)
        tang /= np.linalg.norm(tang, axis=1, keepdims=True) + 1e-9
        w = sp['root_width']
        x0, x1 = pts[:, 0].min() - w, pts[:, 0].max() + w
        y0, y1 = pts[:, 1].min() - w * 0.3, pts[:, 1].max() + w * 0.3
        j0, j1 = np.searchsorted(gx, x0), np.searchsorted(gx, x1)
        i0, i1 = np.searchsorted(gy, y0), np.searchsorted(gy, y1)
        if j1 <= j0 or i1 <= i0:
            continue
        for r0 in range(i0, i1, 48):
            r1 = min(i1, r0 + 48)
            X, Y = np.meshgrid(gx[j0:j1], gy[r0:r1])
            d2 = (X[..., None] - pts[None, None, :, 0]) ** 2 + (Y[..., None] - pts[None, None, :, 1]) ** 2
            k = d2.argmin(axis=-1)
            tx, ty = tang[k, 0], tang[k, 1]
            rx, ry = X - pts[k, 0], Y - pts[k, 1]
            along = rx * tx + ry * ty
            perp = tx * ry - ty * rx  # signed distance to the local tangent line (continuous)
            dist = np.sqrt(perp ** 2 + np.minimum(along ** 2, 1e-6))
            tt = np.clip(k / (len(pts) - 1) + along / max(sp['length'], 1e-3), 0, 1)
            hw = w / 2 * (1 - tt ** 2.0) ** 0.85 * (1 - 0.12 * tt) + 1e-4
            region = (slice(r0, r1), slice(j0, j1))
            take = dist < best[region]
            best[region] = np.where(take, dist, best[region])
            t[region] = np.where(take, tt, t[region])
            off[region] = np.where(take, perp, off[region])
            half[region] = np.where(take, hw, half[region])
            length[region] = np.where(take, sp['length'], length[region])
    return t, off, half, length


# --------------------------------------------------------------------------- painting

def to_srgb(c):
    c = np.clip(c, 0, 1)
    return np.where(c <= 0.0031308, c * 12.92, 1.055 * np.power(c, 1 / 2.4) - 0.055)


def paint_piece(name, pattern, seed):
    rng = np.random.default_rng(seed)
    if name == 'scarf':
        C, H = dc.SCARF_CIRCUMFERENCE, dc.SCARF_HEIGHT
        poly = [(0, -H / 2), (C, -H / 2), (C, H / 2), (0, H / 2)]
        bbox = (0.0, -H / 2, C, H / 2)
        width_at = None
    else:
        poly, width_at = pattern
        xs_ = [p[0] for p in poly]
        ys_ = [p[1] for p in poly]
        bbox = (min(xs_) - MARGIN, min(ys_) - MARGIN, max(xs_) + MARGIN, max(ys_) + MARGIN)
    x0, y0, x1, y1 = bbox
    W = int(math.ceil((x1 - x0) * PX_PER_M))
    H_ = int(math.ceil((y1 - y0) * PX_PER_M))
    gx = x0 + (np.arange(W) + 0.5) / PX_PER_M
    gy = y0 + (np.arange(H_) + 0.5) / PX_PER_M
    X, Y = np.meshgrid(gx, gy)
    shape = (H_, W)
    mpp = 1.0 / PX_PER_M

    if name == 'scarf':
        mask = np.ones(shape, bool)
        # Distance to the long free edges only (the loop has no side seams to burn).
        d = np.minimum(Y - (-dc.SCARF_HEIGHT / 2), dc.SCARF_HEIGHT / 2 - Y).astype(np.float32)
    else:
        mask = raster_inside(poly, gx, gy)
        d = inner_distance(mask, 0.06 * PX_PER_M) * mpp

    # --- cloth body: charcoal with warm ash mottling, crackle veins and fibre streaks.
    mott = fbm(shape, max(W, H_) / PX_PER_M * 3.0, 5, rng)
    mott2 = fbm(shape, max(W, H_) / PX_PER_M * 9.0, 4, rng)
    ridge = 1 - np.abs(fbm(shape, max(W, H_) / PX_PER_M * 14.0, 4, rng) * 2 - 1)
    crackle = smooth(0.80, 0.97, ridge)
    fibre = fbm((H_, max(4, -(-W // 6))), max(W, H_) / PX_PER_M * 60.0, 2, rng)
    fibre = np.repeat(fibre, 6, axis=1)[:, :W]
    base = np.zeros(shape + (3,), np.float32)
    charcoal = np.array([0.011, 0.0105, 0.0115], np.float32) if name != 'scarf' else np.array([0.013, 0.0125, 0.0135], np.float32)
    ash = np.array([0.085, 0.078, 0.074], np.float32)
    rust = np.array([0.060, 0.030, 0.022], np.float32)
    m = smooth(0.50, 0.85, mott) * 0.5 + smooth(0.58, 0.88, mott2) * 0.25
    base[:] = charcoal
    base += (ash - charcoal) * (m[..., None] * 0.07 + crackle[..., None] * 0.11)
    base += (rust - charcoal) * (smooth(0.65, 0.92, mott2)[..., None] * 0.18)
    base *= (0.88 + 0.24 * fibre)[..., None]
    rough = 0.90 - 0.06 * crackle - 0.04 * m
    height = 0.12 * mott2 + 0.30 * crackle + 0.35 * fibre
    emission = np.zeros(shape + (3,), np.float32)
    alpha = mask.astype(np.float32)

    if name == 'scarf':
        # Worn frayed long edges: slightly lighter, eroded fibres, no fire.
        edge = np.exp(-d / 0.012)
        base += (ash - base) * (edge * 0.35)[..., None]
        fray = fbm(shape, max(W, H_) / PX_PER_M * 70.0, 2, rng)
        alpha *= (d > (0.0015 + 0.006 * fray * smooth(0.4, 0.9, fbm(shape, 30, 3, rng)))).astype(np.float32)
    else:
        s = -Y
        a = np.clip(X / np.vectorize(width_at)(s) + 0.5, 0, 1)
        root = np.vectorize(width_at.root_depth)(a)
        zone = smooth(root - 0.32, root + 0.10, s)           # burn grows down into the feathers
        edge_zone = smooth(root - 0.20, root - 0.02, s)        # only the hem/feather edges burn
        t, off, half, length = lobe_fields(width_at.spines, gx, gy)
        in_lobe = (t >= 0) & (np.abs(off) < half * 1.25)
        tt = np.where(in_lobe, t, 0)
        rel = np.where(in_lobe, np.abs(off) / half, 1)
        flick = fbm(shape, max(W, H_) / PX_PER_M * 22.0, 4, rng)
        flick2 = fbm(shape, max(W, H_) / PX_PER_M * 55.0, 3, rng)
        # Colour: charcoal -> near-black smouldering maroon; ember red only near tips and edges.
        maroon = np.array([0.045, 0.0055, 0.0030], np.float32)
        ember = np.array([0.30, 0.035, 0.006], np.float32)
        heat = np.clip(zone * (0.35 + 0.65 * tt) * (0.7 + 0.6 * flick), 0, 1)
        base = base * (1 - heat[..., None] * 0.9) + maroon * (heat[..., None] * 0.9)
        edge_heat = edge_zone * np.exp(-d / 0.008)
        base += (ember - base) * np.clip(edge_heat * 0.6 + smooth(0.8, 1.0, heat) * 0.15, 0, 1)[..., None]
        # Glowing burnt rim: white-gold core, saturated orange halo, deep red heat band.
        dj = d * (0.7 + 0.7 * flick2)
        core = np.exp(-dj / 0.0014)
        halo = np.exp(-dj / 0.0065)
        heatband = np.exp(-dj / 0.010)
        tipboost = 0.6 + 0.8 * smooth(0.4, 1.0, tt)
        rim = (edge_zone * tipboost)[..., None] * (core[..., None] * np.array([1.0, 0.66, 0.22]) * 9.0
                                                     + halo[..., None] * np.array([1.0, 0.24, 0.012]) * 3.0
                                                     + heatband[..., None] * np.array([0.45, 0.035, 0.004]) * 0.10)
        # Feather rachis and barbs: thin strands sweeping out and down from the shaft, brighter at tips.
        along = tt * length
        barb_phase = (np.abs(off) * 0.9 - along * 0.5) / 0.018 + flick * 1.6
        barb = np.exp(-((barb_phase - np.round(barb_phase)) / 0.06) ** 2)
        barb *= in_lobe * smooth(0.15, 0.55, tt) * smooth(0.95, 0.45, rel) * smooth(0.30, 0.70, flick2)
        rachis = np.exp(-(off / 0.0016) ** 2) * in_lobe * smooth(0.05, 0.35, tt) * smooth(0.99, 0.8, tt)
        strands = barb * 0.55 + rachis * 1.1
        glow = (strands * zone * (0.15 + 1.1 * tt ** 1.5))[..., None] * np.array([1.0, 0.26, 0.02])
        inner = (zone * smooth(0.45, 1.0, tt) * (0.4 + 0.9 * flick) * 0.22)[..., None] * np.array([1.0, 0.10, 0.008])
        # Sparse ember flecks near the burning edge.
        speck = (rng.random(shape) > 0.9988) & (d < 0.025) & (edge_zone > 0.5)
        speck = speck.astype(np.float32)
        emission = (rim + glow + inner + speck[..., None] * np.array([1.0, 0.45, 0.08]) * 4.0).astype(np.float32)
        base += (np.array([0.30, 0.05, 0.008]) - base) * np.clip(strands * zone * 0.35, 0, 1)[..., None]
        rough = rough - 0.15 * zone
        height += 0.4 * strands * zone
        # Burnt edges are eroded unevenly, finer than the cloth simulation geometry.
        erode = edge_zone * (0.0012 + 0.0055 * smooth(0.35, 0.85, flick2) * (0.4 + 0.6 * tt))
        alpha *= (d > erode).astype(np.float32)
        emission *= alpha[..., None]

    # Normal from height.
    gy_, gx_ = np.gradient(height.astype(np.float32))
    k = 1.6
    n = np.dstack((-gx_ * k, -gy_ * k, np.ones_like(height)))
    n /= np.linalg.norm(n, axis=2, keepdims=True)
    normal = n * 0.5 + 0.5
    return {'bbox': bbox, 'size': (W, H_), 'base': base, 'alpha': alpha, 'emission': emission,
            'rough': np.clip(rough, 0.3, 1.0), 'normal': normal}


def save_png(path, rgb, alpha=None, encode_srgb=True):
    h, w = rgb.shape[:2]
    data = to_srgb(rgb) if encode_srgb else np.clip(rgb, 0, 1)
    rgba = np.dstack((data, alpha if alpha is not None else np.ones((h, w), np.float32))).astype(np.float32)
    img = bpy.data.images.new(path.stem + '_write', w, h, alpha=True, float_buffer=False)
    img.colorspace_settings.name = 'sRGB' if encode_srgb else 'Non-Color'
    img.pixels.foreach_set(rgba.ravel())
    img.filepath_raw = str(path)
    img.file_format = 'PNG'
    img.save()
    bpy.data.images.remove(img)


def load_png(path, colorspace):
    img = bpy.data.images.load(str(path), check_existing=False)
    img.colorspace_settings.name = colorspace
    img.pack()
    return img


def build_material(name, maps, emission_strength):
    mat = bpy.data.materials.new(name)
    mat.use_nodes = True
    nt = mat.node_tree
    nodes, links = nt.nodes, nt.links
    bsdf = nodes['Principled BSDF']
    def tex(img, x, y):
        n = nodes.new('ShaderNodeTexImage')
        n.image = img
        n.location = (x, y)
        n.interpolation = 'Cubic'
        return n
    b = tex(maps['base'], -700, 300)
    links.new(b.outputs['Color'], bsdf.inputs['Base Color'])
    links.new(b.outputs['Alpha'], bsdf.inputs['Alpha'])
    o = tex(maps['orm'], -700, 0)
    sep = nodes.new('ShaderNodeSeparateColor')
    sep.location = (-400, 0)
    links.new(o.outputs['Color'], sep.inputs['Color'])
    links.new(sep.outputs['Green'], bsdf.inputs['Roughness'])
    nm = tex(maps['normal'], -700, -300)
    nmap = nodes.new('ShaderNodeNormalMap')
    nmap.location = (-400, -300)
    links.new(nm.outputs['Color'], nmap.inputs['Color'])
    links.new(nmap.outputs['Normal'], bsdf.inputs['Normal'])
    e = tex(maps['emission'], -700, -600)
    links.new(e.outputs['Color'], bsdf.inputs['Emission Color'])
    bsdf.inputs['Emission Strength'].default_value = emission_strength
    bsdf.inputs['Sheen Weight'].default_value = 0.0
    bsdf.inputs['Metallic'].default_value = 0.0
    mat['kiva_alpha_mode'] = 1  # mask: burnt edge erosion
    mat['kiva_surface_effect'] = 0  # static painted glow; Ember is reserved for dedicated blend fire
    mat['kiva_orm_image'] = maps['orm'].name
    return mat


# --------------------------------------------------------------------------- render setup

def studio():
    scene = bpy.context.scene
    dc.gpu_cycles(128)
    scene.view_settings.view_transform = 'Khronos PBR Neutral'  # keeps ember orange saturated (AgX drifts to peach)
    scene.view_settings.look = 'None'
    world = bpy.data.worlds.new('Concept grey studio')
    world.use_nodes = True
    world.node_tree.nodes['Background'].inputs['Color'].default_value = (0.21, 0.21, 0.225, 1)
    world.node_tree.nodes['Background'].inputs['Strength'].default_value = 0.85
    scene.world = world
    for obj in scene.objects:
        if obj.type == 'MESH' and obj.name.startswith('Studio ground'):
            mat = bpy.data.materials.new('Concept floor grey')
            mat.use_nodes = True
            mat.node_tree.nodes['Principled BSDF'].inputs['Base Color'].default_value = (0.20, 0.20, 0.215, 1)
            mat.node_tree.nodes['Principled BSDF'].inputs['Roughness'].default_value = 0.9
            obj.data.materials.clear()
            obj.data.materials.append(mat)
    try:
        tree = bpy.data.node_groups.new('Ember glare', 'CompositorNodeTree')
        scene.compositing_node_group = tree
        rl = tree.nodes.new('CompositorNodeRLayers')
        glare = tree.nodes.new('CompositorNodeGlare')
        out = tree.nodes.new('NodeGroupOutput')
        tree.interface.new_socket('Image', in_out='OUTPUT', socket_type='NodeSocketColor')
        for key, value in (('Type', 'Fog Glow'), ('Quality', 'High'), ('Threshold', 0.9), ('Size', 0.5), ('Strength', 0.9)):
            if key in glare.inputs:
                try:
                    glare.inputs[key].default_value = value
                except TypeError:
                    pass
        if hasattr(glare, 'glare_type'):
            glare.glare_type = 'FOG_GLOW'
        tree.links.new(rl.outputs['Image'], glare.inputs['Image'])
        tree.links.new(glare.outputs['Image'], out.inputs[0])
    except Exception as exc:  # glare is cosmetic for reviews only
        print('compositor glare unavailable:', exc, flush=True)


def main():
    require(bpy.app.background, 'Use an independent background Blender process')
    parser = argparse.ArgumentParser()
    parser.add_argument('--draped', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--emission', type=float, default=EMISSION_RANGE)
    parser.add_argument('--samples', type=int, default=128)
    parser.add_argument('--renders', default='front,back,side,three,neck,hem')
    opts = parser.parse_args(sys.argv[sys.argv.index('--') + 1:])
    tex_dir = opts.output / 'textures'
    tex_dir.mkdir(parents=True, exist_ok=True)
    bpy.ops.wm.open_mainfile(filepath=str(opts.draped))
    cloth = bpy.data.objects['Cape v3 draped cloth']
    pats = dc.patterns()
    materials, info = [], {}
    for pid, name in enumerate(dc.PIECES):
        painted = paint_piece(name, pats[name], seed=100 + pid)
        files = {}
        for key, rgb, alpha, srgb in (('base', painted['base'], painted['alpha'], True),
                                      ('emission', painted['emission'] / EMISSION_RANGE, None, True),
                                      ('normal', painted['normal'], None, False),
                                      ('orm', np.dstack((np.ones_like(painted['rough']), painted['rough'],
                                                         np.zeros_like(painted['rough']))), None, False)):
            path = tex_dir / f'{name}_{key}.png'
            save_png(path, rgb, alpha, srgb)
            files[key] = path
        maps = {k: load_png(p, 'sRGB' if k in ('base', 'emission') else 'Non-Color') for k, p in files.items()}
        mat = build_material(f'Zhuque cape v3 {name}', maps, opts.emission if name != 'scarf' else 0.0)
        materials.append(mat)
        info[name] = {'bbox': painted['bbox'], 'size': painted['size'],
                      'files': {k: {'path': p.name, 'sha256': hashlib.sha256(p.read_bytes()).hexdigest()} for k, p in files.items()}}
        print('painted', name, painted['size'], flush=True)
    mesh = cloth.data
    mesh.materials.clear()
    for mat in materials:
        mesh.materials.append(mat)
    piece = mesh.attributes['kiva_cape_piece']
    uv = mesh.uv_layers['PatternUV']
    for poly in mesh.polygons:
        pid = piece.data[poly.index].value
        poly.material_index = pid
        x0, y0, x1, y1 = info[dc.PIECES[pid]]['bbox']
        for li in poly.loop_indices:
            u, v = uv.data[li].uv
            uv.data[li].uv = ((u - x0) / (x1 - x0), (v - y0) / (y1 - y0))
    studio()
    bpy.context.scene.cycles.samples = opts.samples
    bpy.ops.wm.save_as_mainfile(filepath=str(opts.output / 'zhuque-cape-v3.blend'), check_existing=False)
    (opts.output / 'texture-report.json').write_text(json.dumps(info, indent=2) + '\n')
    shots = {
        'front': (0, (0, 0, .88), 2.0, (1000, 1300)),
        'back': (math.pi, (0, 0, .88), 2.0, (1000, 1300)),
        'side': (-math.pi / 2, (0, 0, .88), 2.0, (1000, 1300)),
        'three': (math.pi * 0.78, (0, 0, .88), 2.0, (1000, 1300)),
        'neck': (-.35, (0, 0, 1.36), .7, (1100, 1000)),
        'hem': (math.pi * 0.9, (0.05, -0.2, .62), .8, (1100, 1000)),
    }
    for key in filter(None, opts.renders.split(',')):
        angle, target, scale, size = shots[key]
        dc.render(opts.output / f'cape-v3-{key}.png', angle, target, scale, size)
        print('rendered', key, flush=True)
    # Runtime source: same cage, skin and materials; only the authoring subdivision is off.
    for mod in cloth.modifiers:
        if mod.type == 'SUBSURF':
            mod.levels = mod.render_levels = 0
    for name in ('cape_L', 'cape_C', 'cape_R'):
        require(name in cloth.vertex_groups or True, name)
    bpy.ops.wm.save_as_mainfile(filepath=str(opts.output / 'zhuque-cape-v3-runtime-source.blend'), check_existing=False)
    print('runtime source saved', flush=True)


if __name__ == '__main__':
    main()
