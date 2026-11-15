"""Material stage for the concept-matched cape: script-painted PBR maps on the approved clay.

Textures are painted with numpy; the concept contributes only measured positions
(silhouette, relief, ember locations), never projected colour.
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
import build_cape_v4 as B

W, H = B.W, B.H
TILE_MM = 512.0
TILE_PX = 2048
MACRO_PX = 2048


def srgb(linear):
    linear = np.clip(linear, 0, 1)
    return np.where(linear <= 0.0031308, linear * 12.92, 1.055 * linear ** (1 / 2.4) - 0.055) * 255.0


def fft_noise(n, beta, seed):
    rng = np.random.default_rng(seed)
    fx = np.fft.fftfreq(n)[None, :]
    fy = np.fft.fftfreq(n)[:, None]
    k = np.sqrt(fx * fx + fy * fy)
    k[0, 0] = 1.0
    out = np.real(np.fft.ifft2(np.fft.fft2(rng.normal(size=(n, n))) / k ** beta))
    lo, hi = np.percentile(out, [1, 99])
    return np.clip((out - lo) / (hi - lo), 0, 1).astype(np.float32)


def strokes(n, count, seed):
    """Wrapped squiggly fibre marks and specks."""
    rng = np.random.default_rng(seed)
    mask = np.zeros((n, n), np.float32)
    for _ in range(count):
        x, y = rng.uniform(0, n, 2)
        heading = rng.uniform(0, 2 * math.pi)
        width = rng.uniform(0.9, 1.7)
        gain = rng.uniform(0.45, 1.0)
        for _ in range(rng.integers(2, 6)):
            heading += rng.uniform(-1.0, 1.0)
            length = rng.uniform(8, 34)
            x1, y1 = x + math.cos(heading) * length, y + math.sin(heading) * length
            xa, xb = int(min(x, x1) - 4), int(max(x, x1) + 5)
            ya, yb = int(min(y, y1) - 4), int(max(y, y1) + 5)
            xs = np.arange(xa, xb)
            ys = np.arange(ya, yb)
            gx, gy = np.meshgrid(xs.astype(np.float32), ys.astype(np.float32))
            dx, dy = x1 - x, y1 - y
            t = np.clip(((gx - x) * dx + (gy - y) * dy) / max(dx * dx + dy * dy, 1e-6), 0, 1)
            dist = np.hypot(gx - x - t * dx, gy - y - t * dy)
            val = gain * np.clip(1.4 - dist / width, 0, 1)
            idx = np.ix_(ys % n, xs % n)
            mask[idx] = np.maximum(mask[idx], val)
            x, y = x1, y1
    for _ in range(count * 3):
        x, y = rng.integers(0, n, 2)
        mask[y, x] = max(mask[y, x], rng.uniform(0.4, 1.0))
        mask[y, (x + 1) % n] = max(mask[y, (x + 1) % n], 0.5)
    return np.clip(mask, 0, 1)


def detail_tile(out):
    n = TILE_PX
    pitch = 5                                        # 1.25 mm threads at 0.25 mm / px
    ys, xs = np.mgrid[0:n, 0:n]
    warp_wobble = (fft_noise(n, 1.3, 1) - 0.5) * 9.0
    weft_wobble = (fft_noise(n, 1.3, 2) - 0.5) * 9.0
    fx = ((xs + warp_wobble) / pitch) % 1.0
    fy = ((ys + weft_wobble) / pitch) % 1.0
    parity = ((xs // pitch + ys // pitch) % 2).astype(bool)
    wx = np.cos(math.pi * (fx - 0.5))
    wy = np.cos(math.pi * (fy - 0.5))
    weave = np.where(parity, wx * (0.55 + 0.45 * wy), wy * (0.55 + 0.45 * wx)).astype(np.float32)
    thread = fft_noise(n, 0.6, 3)
    weave = weave * (0.45 + 1.1 * thread)
    fuzz = fft_noise(n, 0.2, 4)
    mottled = fft_noise(n, 1.9, 5)
    scratch = strokes(n, 1000, 6) * (0.25 + 0.75 * C.smoothstep(0.25, 0.6, fft_noise(n, 2.2, 8)))
    pebble = fft_noise(n, 1.0, 9)
    height = 0.8 * weave + 0.5 * fuzz + 0.6 * scratch + 1.2 * mottled + 2.6 * pebble
    gx = (np.roll(height, -1, 1) - np.roll(height, 1, 1)) * 0.5
    gy = (np.roll(height, -1, 0) - np.roll(height, 1, 0)) * 0.5
    k = 3.0
    nx, ny = -gx * k, gy * k
    nz = np.ones_like(nx)
    ln = np.sqrt(nx * nx + ny * ny + nz * nz)
    normal = np.stack([nx / ln, ny / ln, nz / ln], -1) * 0.5 + 0.5
    C.write_png(out / 'cape_v4_detail_normal.png', normal * 255)
    shade = np.clip(0.5 + 0.2 * (weave - 0.5) + 0.6 * (mottled - 0.5) + 0.2 * (fuzz - 0.5) + 0.7 * (pebble - 0.5), 0, 1)
    rough = np.clip(0.35 + 0.5 * fft_noise(n, 1.4, 7) + 0.2 * (1 - weave), 0, 1)
    C.write_png(out / 'cape_v4_detail_mod.png', np.stack([shade, scratch, rough], -1) * 255)


def macro_maps(out, m, p, edges):
    """Concept-plane maps for the cloak body and feathers."""
    rel, _ = B.relief(m, p)
    feathers = B.feather_sheets(m, p, edges)
    B.apply_tiers(feathers, p)
    top = np.full((H, W), -1e9, np.float32)
    vein = np.zeros((H, W), np.float32)
    feathered = np.zeros((H, W), np.float32)
    for feather in feathers:
        take = feather['inside'] & (feather['height'] > top)
        top = np.where(take, feather['height'], top)
        vein = np.where(take, feather['vein'], vein)
        feathered = np.where(take, 1.0, feathered)
    mask = m['mask']
    ys = np.arange(H, dtype=np.float32)[:, None]
    warm = m['warm']
    burn = C.smoothstep(0.04, 0.62, C.blur((warm > 45).astype(np.float32), 6)) * mask
    hot = C.smoothstep(0.1, 0.7, C.blur((warm > 95).astype(np.float32), 4)) * mask
    dust = fft_noise(2048, 2.0, 21)[:H, :W]
    patch = fft_noise(2048, 1.2, 22)[:H, :W]
    ridge = np.clip(rel / 22.0, 0, 1)
    edge = C.smoothstep(0.97, 0.6, C.blur(mask.astype(np.float32), 2.2)) * mask

    def col(r, g, b):
        return np.array([r, g, b], np.float32)

    base = col(0.060, 0.046, 0.041) * (0.6 + 0.8 * dust)[..., None]
    base = base + (ridge * (0.35 + 0.65 * patch))[..., None] * col(0.030, 0.026, 0.022)
    base = base * (1 - 0.18 * feathered * (1 - burn))[..., None]
    scorched = col(0.055, 0.016, 0.012) * (0.6 + 0.8 * patch)[..., None]
    base = base * (1 - 0.88 * burn)[..., None] + scorched * (0.88 * burn)[..., None]
    glowing = col(0.12, 0.030, 0.015)
    base = base * (1 - 0.6 * hot)[..., None] + glowing * (0.6 * hot)[..., None]
    base = base + (vein * (0.008 + 0.07 * burn))[..., None] * col(1.0, 0.42, 0.16)
    frayed = col(0.17, 0.12, 0.085)
    amount = (edge * 0.8 * (1 - burn))[..., None]
    base = base * (1 - amount) + frayed * amount

    v, u = np.mgrid[0:MACRO_PX, 0:MACRO_PX].astype(np.float32)
    sx = (u + 0.5) / MACRO_PX * W - 0.5
    sy = (v + 0.5) / MACRO_PX * H - 0.5
    rgb = np.stack([C.bilinear(base[..., i], sx, sy) for i in range(3)], -1)
    C.write_png(out / 'cape_v4_macro_base.png', srgb(rgb))
    height = 0.5 + 0.5 * vein * feathered - 0.25 * edge
    C.write_png(out / 'cape_v4_macro_height.png', np.clip(C.bilinear(height, sx, sy), 0, 1) * 255)
    C.write_png(out / 'cape_v4_macro_masks.png', np.stack([C.bilinear(burn, sx, sy), C.bilinear(hot, sx, sy), C.bilinear(feathered, sx, sy)], -1) * 255)


# ------------------------------------------------------------------ UVs for the lathe / loop meshes
def grid_uv(obj, seg, wrap_mm, keep=None):
    """UV for a vertex grid laid out as row * seg + column, closed in the column direction."""
    mesh = obj.data
    co = np.empty(len(mesh.vertices) * 3, np.float32)
    mesh.vertices.foreach_get('co', co)
    co = co.reshape(-1, seg, 3) * 1000.0
    rows = co.shape[0]
    perimeter = float(np.mean(np.linalg.norm(co - np.roll(co, -1, 1), axis=2).sum(1)))
    turns = max(1, round(perimeter / wrap_mm)) * wrap_mm
    step = np.linalg.norm(co[1:] - co[:-1], axis=2).mean(1)
    v_mm = np.concatenate([[0], np.cumsum(step)])
    ii, jj = np.meshgrid(np.arange(rows - 1), np.arange(seg), indexing='ij')
    ii, jj = ii.ravel(), jj.ravel()
    if keep is not None:
        ii, jj = ii[keep[jj]], jj[keep[jj]]
    us = np.stack([jj, jj, jj + 1, jj + 1], 1) / seg * turns
    vs = np.stack([v_mm[ii], v_mm[ii + 1], v_mm[ii + 1], v_mm[ii]], 1)
    uv = mesh.uv_layers.get('UVMap') or mesh.uv_layers.new(name='UVMap')
    uv.data.foreach_set('uv', np.stack([us / W, vs / H], -1).astype(np.float32).ravel())
    return perimeter, turns


def collar_uv(obj, seg, wrap_mm):
    """Collar lathe: vertices are column * profile + row, faces (a+j, b+j, b+j+1, a+j+1)."""
    mesh = obj.data
    co = np.empty(len(mesh.vertices) * 3, np.float32)
    mesh.vertices.foreach_get('co', co)
    co = co.reshape(seg, -1, 3) * 1000.0
    n = co.shape[1]
    perimeter = float(np.mean(np.linalg.norm(co - np.roll(co, -1, 0), axis=2).sum(0)))
    turns = max(1, round(perimeter / wrap_mm)) * wrap_mm
    v_mm = np.concatenate([[0], np.cumsum(np.linalg.norm(co[:, 1:] - co[:, :-1], axis=2).mean(0))])
    ii, jj = np.meshgrid(np.arange(seg), np.arange(n - 1), indexing='ij')
    ii, jj = ii.ravel(), jj.ravel()
    us = np.stack([ii, ii + 1, ii + 1, ii], 1) / seg * turns
    vs = np.stack([v_mm[jj], v_mm[jj], v_mm[jj + 1], v_mm[jj + 1]], 1)
    uv = mesh.uv_layers.get('UVMap') or mesh.uv_layers.new(name='UVMap')
    uv.data.foreach_set('uv', np.stack([us / W, vs / H], -1).astype(np.float32).ravel())
    return perimeter, turns


# ------------------------------------------------------------------ materials
def image(path, color):
    img = bpy.data.images.load(str(path), check_existing=True)
    img.colorspace_settings.name = 'sRGB' if color else 'Non-Color'
    return img


def cloth_material(name, tex, macro, scratch, tint, scale=1.0, sheen=0.03):
    mat = bpy.data.materials.new(name)
    mat.use_nodes = True
    nt = mat.node_tree
    bsdf = nt.nodes['Principled BSDF']
    new = nt.nodes.new
    link = nt.links.new
    coord = new('ShaderNodeTexCoord')
    mapping = new('ShaderNodeMapping')
    mapping.inputs['Scale'].default_value = (W / TILE_MM * scale, H / TILE_MM * scale, 1)
    link(coord.outputs['UV'], mapping.inputs['Vector'])
    det = new('ShaderNodeTexImage')
    det.image = image(tex / 'cape_v4_detail_mod.png', False)
    link(mapping.outputs['Vector'], det.inputs['Vector'])
    dn = new('ShaderNodeTexImage')
    dn.image = image(tex / 'cape_v4_detail_normal.png', False)
    link(mapping.outputs['Vector'], dn.inputs['Vector'])
    sep = new('ShaderNodeSeparateColor')
    link(det.outputs['Color'], sep.inputs['Color'])
    shade = new('ShaderNodeMath')
    shade.operation = 'MULTIPLY_ADD'
    shade.inputs[1].default_value = 0.9
    shade.inputs[2].default_value = 0.55
    link(sep.outputs['Red'], shade.inputs[0])
    mul = new('ShaderNodeMix')
    mul.data_type = 'RGBA'
    mul.blend_type = 'MULTIPLY'
    mul.inputs[0].default_value = 1.0
    if macro:
        base = new('ShaderNodeTexImage')
        base.image = image(tex / 'cape_v4_macro_base.png', True)
        link(coord.outputs['UV'], base.inputs['Vector'])
        link(base.outputs['Color'], mul.inputs[6])
    else:
        mul.inputs[6].default_value = (*tint, 1)
    link(shade.outputs[0], mul.inputs[7])
    amount = new('ShaderNodeMath')
    amount.operation = 'MULTIPLY'
    amount.inputs[1].default_value = scratch
    link(sep.outputs['Green'], amount.inputs[0])
    mix = new('ShaderNodeMix')
    mix.data_type = 'RGBA'
    mix.inputs[7].default_value = (0.30, 0.15, 0.08, 1)
    link(amount.outputs[0], mix.inputs[0])
    link(mul.outputs[2], mix.inputs[6])
    link(mix.outputs[2], bsdf.inputs['Base Color'])
    rough = new('ShaderNodeMath')
    rough.operation = 'MULTIPLY_ADD'
    rough.inputs[1].default_value = 0.08
    rough.inputs[2].default_value = 0.90
    link(sep.outputs['Blue'], rough.inputs[0])
    link(rough.outputs[0], bsdf.inputs['Roughness'])
    nm = new('ShaderNodeNormalMap')
    nm.inputs['Strength'].default_value = 1.6
    link(dn.outputs['Color'], nm.inputs['Color'])
    normal = nm.outputs['Normal']
    if macro:
        hgt = new('ShaderNodeTexImage')
        hgt.image = image(tex / 'cape_v4_macro_height.png', False)
        link(coord.outputs['UV'], hgt.inputs['Vector'])
        bump = new('ShaderNodeBump')
        bump.inputs['Distance'].default_value = 0.0007
        link(hgt.outputs['Color'], bump.inputs['Height'])
        link(normal, bump.inputs['Normal'])
        normal = bump.outputs['Normal']
    link(normal, bsdf.inputs['Normal'])
    bsdf.inputs['Sheen Weight'].default_value = sheen
    bsdf.inputs['Sheen Roughness'].default_value = 0.55
    bsdf.inputs['Sheen Tint'].default_value = (0.75, 0.68, 0.62, 1)
    bsdf.inputs['Specular IOR Level'].default_value = 0.08
    return mat


def pair(concept, shot, box, path, scale=2):
    x0, y0, x1, y1 = box
    a = np.repeat(np.repeat(concept[y0:y1, x0:x1], scale, 0), scale, 1)
    b = np.repeat(np.repeat(shot[y0:y1, x0:x1], scale, 0), scale, 1)
    C.write_png(path, np.concatenate([a, np.full((a.shape[0], 8, 3), 20.0), b], 1))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--concept', required=True)
    ap.add_argument('--output', required=True)
    ap.add_argument('--samples', type=int, default=160)
    ap.add_argument('--skip-textures', action='store_true')
    ap.add_argument('--turn', action='store_true')
    args = ap.parse_args(sys.argv[sys.argv.index('--') + 1:])
    out = Path(args.output)
    tex = out / 'textures'
    tex.mkdir(exist_ok=True)
    p = dict(B.PARAMS)
    bpy.ops.wm.open_mainfile(filepath=str(out / 'zhuque-cape-v4-clay.blend'))
    m = B.measure(args.concept)
    if not args.skip_textures:
        detail_tile(tex)
        macro_maps(tex, m, p, json.loads((out / 'feathers.json').read_text()))
    report = {}
    scarf = bpy.data.objects['CapeV4_scarf']
    ct = np.cos(np.linspace(-math.pi, math.pi, 320, endpoint=False))
    keep = (ct < B.SCARF_BACK_LIMIT) & (np.roll(ct, -1) < B.SCARF_BACK_LIMIT)
    report['scarf_uv'] = grid_uv(scarf, 320, TILE_MM, keep)
    collar = bpy.data.objects['CapeV4_collar']
    report['collar_uv'] = collar_uv(collar, 192, TILE_MM)
    cloak = cloth_material('CapeV4 cloak cloth', tex, True, 0.6, None)
    wrap = cloth_material('CapeV4 scarf cloth', tex, False, 1.0, (0.062, 0.047, 0.042), sheen=0.04)
    canvas = cloth_material('CapeV4 collar canvas', tex, False, 0.12, (0.050, 0.041, 0.039), scale=1.6, sheen=0.0)
    for obj in bpy.data.objects:
        if obj.type != 'MESH' or not obj.name.startswith('CapeV4_'):
            continue
        obj.data.materials.clear()
        obj.data.materials.append(wrap if obj.name.startswith('CapeV4_scarf') else canvas if obj is collar else cloak)
    device = B.setup_render(args.samples)
    # fixed studio: emissive backdrop seen only by the camera, contrastier key
    backdrop = bpy.data.objects['Backdrop']
    bmat = bpy.data.materials.new('Backdrop flat')
    bmat.use_nodes = True
    bnt = bmat.node_tree
    emit = bnt.nodes.new('ShaderNodeEmission')
    emit.inputs['Color'].default_value = (0.165, 0.127, 0.124, 1)
    bnt.links.new(emit.outputs[0], bnt.nodes['Material Output'].inputs['Surface'])
    backdrop.data.materials.clear()
    backdrop.data.materials.append(bmat)
    for flag in ('visible_diffuse', 'visible_glossy', 'visible_shadow', 'visible_transmission'):
        setattr(backdrop, flag, False)
    bpy.data.objects['Key'].data.energy = 190
    bpy.data.objects['Fill'].data.energy = 8
    bpy.data.objects['Top'].data.energy = 95
    bpy.context.scene.world.node_tree.nodes['Background'].inputs['Strength'].default_value = 0.1
    bpy.ops.wm.save_as_mainfile(filepath=str(out / 'zhuque-cape-v4-textured.blend'))
    shot = out / 'material-concept-view.png'
    B.render(shot)
    B.side_by_side(m['img'], shot, out / 'material-comparison.png')
    rendered = C.load_rgb8(shot)
    pair(m['img'], rendered, (300, 0, 880, 460), out / 'material-detail-collar.png')
    pair(m['img'], rendered, (250, 430, 760, 900), out / 'material-detail-back.png')
    pair(m['img'], rendered, (120, 900, 720, 1500), out / 'material-detail-tail.png')
    if args.turn:
        cam = bpy.context.scene.camera
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
            path = out / f'material-turn{deg:+04d}.png'
            B.render(path)
            shots.append(C.load_rgb8(path)[::2, ::2])
        C.write_png(out / 'material-turn-sheet.png', np.concatenate(shots, 1))
    (out / 'material-report.json').write_text(json.dumps(report, indent=1))
    print('DEVICE', device, report)


if __name__ == '__main__':
    main()
