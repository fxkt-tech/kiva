"""Finish stage for the concept-matched cape: view-projected detail, HDR embers, sparks, bloom.

The concept is de-lit (large-scale shading divided out) and mapped onto every surface that
faces the concept camera, so weave, wear, feather veins, serrated hems and ember lines come
from the concept itself.  Surfaces facing away keep the script-painted cloth.  Emission is
the concept's measured glow above scorched cloth, stored HDR so an engine bloom can use it;
the bloom here is a showcase post-process only.
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
import texture_cape_v4 as T

W, H = B.W, B.H
STRENGTH = 4.0
SCORCH = np.array([0.065, 0.020, 0.016], np.float32)


def to_linear(v8):
    v = v8 / 255.0
    return np.where(v <= 0.04045, v / 12.92, ((v + 0.055) / 1.055) ** 2.4).astype(np.float32)


def view_maps(tex, m):
    """De-lit albedo and HDR emission in the concept camera plane."""
    lin = to_linear(m['img'])
    warm, mask = m['warm'], m['mask']
    glow = C.blur(C.smoothstep(38, 85, warm), 1.0) * mask
    lum = 0.2126 * lin[..., 0] + 0.7152 * lin[..., 1] + 0.0722 * lin[..., 2]
    inner = C.blur(mask.astype(np.float32), 1.2) > 0.97
    valid = inner & (glow < 0.1)
    shade = C.masked_blur(lum, valid.astype(np.float32), 30)
    level = float(lum[valid].mean())
    shade_n = np.clip(shade / level, 0.35, 2.5)
    blue = lin[..., 2]
    detail = np.clip(blue / np.maximum(C.masked_blur(blue, inner.astype(np.float32), 6), 1e-4), 0.4, 2.2)
    scorch = SCORCH[None, None, :] * detail[..., None]
    albedo = (1 - glow)[..., None] * lin / shade_n[..., None] + glow[..., None] * scorch
    for i in range(3):
        albedo[..., i] = np.where(inner, albedo[..., i], C.masked_blur(albedo[..., i], inner.astype(np.float32), 4))
    emission = np.clip(lin - scorch * shade_n[..., None], 0, None) * glow[..., None]
    sat = C.smoothstep(0.85, 1.0, m['img'][..., 0] / 255.0)
    emission = emission * (1 + sat * (0.4 + 1.2 * lin[..., 1]))[..., None]
    C.write_png(tex / 'cape_v4_view_albedo.png', T.srgb(np.clip(albedo, 0, 0.9)))
    C.write_png(tex / 'cape_v4_view_emission.png', T.srgb(np.clip(emission / STRENGTH, 0, 1)))
    return {'albedo_level': level, 'glow_px': float(glow.sum()), 'select': valid}


def project(mat, tex):
    """Mix the view-projected albedo/emission over the painted cloth on camera-facing surfaces."""
    nt = mat.node_tree
    new, link = nt.nodes.new, nt.links.new
    bsdf = nt.nodes['Principled BSDF']
    painted = bsdf.inputs['Base Color'].links[0].from_socket
    coord = new('ShaderNodeTexCoord')
    sep = new('ShaderNodeSeparateXYZ')
    link(coord.outputs['Object'], sep.inputs[0])
    u = new('ShaderNodeMath')
    u.operation = 'MULTIPLY_ADD'
    u.inputs[1].default_value = 1.0 / (W * C.MM)
    u.inputs[2].default_value = 0.5
    link(sep.outputs['X'], u.inputs[0])
    v = new('ShaderNodeMath')
    v.operation = 'MULTIPLY'
    v.inputs[1].default_value = 1.0 / (H * C.MM)
    link(sep.outputs['Z'], v.inputs[0])
    uv = new('ShaderNodeCombineXYZ')
    link(u.outputs[0], uv.inputs['X'])
    link(v.outputs[0], uv.inputs['Y'])
    nsep = new('ShaderNodeSeparateXYZ')
    link(coord.outputs['Normal'], nsep.inputs[0])
    facing = new('ShaderNodeMath')
    facing.operation = 'MULTIPLY_ADD'
    facing.inputs[1].default_value = -8.0
    facing.inputs[2].default_value = -0.16
    facing.use_clamp = True
    link(nsep.outputs['Y'], facing.inputs[0])
    alb = new('ShaderNodeTexImage')
    alb.image = T.image(tex / 'cape_v4_view_albedo.png', True)
    alb.extension = 'EXTEND'
    link(uv.outputs[0], alb.inputs['Vector'])
    mix = new('ShaderNodeMix')
    mix.data_type = 'RGBA'
    link(facing.outputs[0], mix.inputs[0])
    link(painted, mix.inputs[6])
    link(alb.outputs['Color'], mix.inputs[7])
    link(mix.outputs[2], bsdf.inputs['Base Color'])
    em = new('ShaderNodeTexImage')
    em.image = T.image(tex / 'cape_v4_view_emission.png', True)
    em.extension = 'EXTEND'
    link(uv.outputs[0], em.inputs['Vector'])
    gate = new('ShaderNodeMix')
    gate.data_type = 'RGBA'
    gate.inputs[6].default_value = (0, 0, 0, 1)
    link(facing.outputs[0], gate.inputs[0])
    link(em.outputs['Color'], gate.inputs[7])
    link(gate.outputs[2], bsdf.inputs['Emission Color'])
    bsdf.inputs['Emission Strength'].default_value = STRENGTH
    for node in nt.nodes:
        if node.type == 'NORMAL_MAP':
            node.inputs['Strength'].default_value = 0.8


def sparks(m):
    """Small emissive flecks where the concept shows loose sparks off the cloth."""
    near = C.blur(m['mask'].astype(np.float32), 3) > 0.02
    loose = m['ember'] & ~near
    label_src = loose.copy()
    points = []
    ys, xs = np.nonzero(label_src)
    seen = np.zeros_like(label_src)
    for sy, sx in zip(ys.tolist(), xs.tolist()):
        if seen[sy, sx]:
            continue
        stack, members = [(sy, sx)], []
        seen[sy, sx] = True
        while stack:
            y, x = stack.pop()
            members.append((y, x))
            for ny, nx in ((y - 1, x), (y + 1, x), (y, x - 1), (y, x + 1)):
                if 0 <= ny < H and 0 <= nx < W and label_src[ny, nx] and not seen[ny, nx]:
                    seen[ny, nx] = True
                    stack.append((ny, nx))
        arr = np.array(members, dtype=np.float32)
        if len(arr) <= 40:
            points.append((float(arr[:, 1].mean()), float(arr[:, 0].mean()), max(1.0, math.sqrt(len(arr)))))
    verts, faces = [], []
    rng = np.random.default_rng(4)
    for x, y, size in points:
        r = 0.4 * size * C.MM
        cx, cz = (x - W / 2) * C.MM, (H - y) * C.MM
        cy = -0.02 - rng.uniform(0, 0.12)
        base = len(verts)
        verts += [(cx - r, cy, cz), (cx, cy, cz - r), (cx + r, cy, cz), (cx, cy, cz + r)]
        faces.append((base, base + 1, base + 2, base + 3))
    mesh = bpy.data.meshes.new('CapeV4_sparks')
    mesh.from_pydata(verts, [], faces)
    obj = bpy.data.objects.new('CapeV4_sparks', mesh)
    bpy.context.scene.collection.objects.link(obj)
    mat = bpy.data.materials.new('CapeV4 sparks')
    mat.use_nodes = True
    nt = mat.node_tree
    emit = nt.nodes.new('ShaderNodeEmission')
    emit.inputs['Color'].default_value = (1.0, 0.42, 0.08, 1)
    emit.inputs['Strength'].default_value = 2.5
    nt.links.new(emit.outputs[0], nt.nodes['Material Output'].inputs['Surface'])
    mesh.materials.append(mat)
    return len(points)


def render_hdr(path):
    scene = bpy.context.scene
    scene.render.image_settings.file_format = 'OPEN_EXR'
    scene.render.image_settings.color_depth = '32'
    scene.render.filepath = str(path)
    bpy.ops.render.render(write_still=True)
    img = bpy.data.images.load(str(path), check_existing=False)
    w, h = img.size
    px = np.empty(w * h * 4, dtype=np.float32)
    img.pixels.foreach_get(px)
    bpy.data.images.remove(img)
    return px.reshape(h, w, 4)[::-1].copy()


def bloom(lin, amount=0.3):
    lum = lin.max(-1)
    over = np.clip(lum - 0.9, 0, None)
    scale = np.where(lum > 1e-6, over / np.maximum(lum, 1e-6), 0)[..., None]
    bright = np.clip(lin * scale, 0, 6)
    glow = np.zeros_like(lin)
    for radius, gain in ((2.0, 0.6), (6, 0.45), (18, 0.30), (45, 0.15)):
        glow += gain * np.stack([C.blur(bright[..., i], radius) for i in range(3)], -1)
    return lin + amount * glow


def calibrate(out, m, select, rounds=2):
    """Scale the studio so the lit, non-glowing cloth averages the same as in the concept."""
    scene = bpy.context.scene
    target = to_linear(m['img'])
    target = (0.2126 * target[..., 0] + 0.7152 * target[..., 1] + 0.0722 * target[..., 2])[select].mean()
    samples = scene.cycles.samples
    scene.cycles.samples = 48
    total = 1.0
    for _ in range(rounds):
        lin = render_hdr(out / 'calibrate.exr')[..., :3]
        got = (0.2126 * lin[..., 0] + 0.7152 * lin[..., 1] + 0.0722 * lin[..., 2])[select].mean()
        ratio = float(target / max(got, 1e-6))
        total *= ratio
        for name in ('Key', 'Fill', 'Top'):
            bpy.data.objects[name].data.energy *= ratio
        bg = scene.world.node_tree.nodes['Background'].inputs['Strength']
        bg.default_value *= ratio
    scene.cycles.samples = samples
    (out / 'calibrate.exr').unlink(missing_ok=True)
    return total


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--concept', required=True)
    ap.add_argument('--output', required=True)
    ap.add_argument('--samples', type=int, default=256)
    ap.add_argument('--turn', action='store_true')
    args = ap.parse_args(sys.argv[sys.argv.index('--') + 1:])
    out = Path(args.output)
    tex = out / 'textures'
    bpy.ops.wm.open_mainfile(filepath=str(out / 'zhuque-cape-v4-textured.blend'))
    m = B.measure(args.concept)
    info = view_maps(tex, m)
    select = info.pop('select')
    report = {'emission_strength': STRENGTH, **info}
    for name in ('CapeV4 cloak cloth', 'CapeV4 scarf cloth', 'CapeV4 collar canvas'):
        project(bpy.data.materials[name], tex)
    report['sparks'] = sparks(m)
    device = B.setup_render(args.samples)
    # the concept is lit softly; keep the key gentle so de-lit detail is not shaded twice
    bpy.data.objects['Key'].data.energy = 110
    bpy.data.objects['Fill'].data.energy = 70
    bpy.data.objects['Top'].data.energy = 70
    bpy.context.scene.world.node_tree.nodes['Background'].inputs['Strength'].default_value = 0.45
    report['exposure_scale'] = calibrate(out, m, select)
    bpy.ops.wm.save_as_mainfile(filepath=str(out / 'zhuque-cape-v4.blend'))
    lin = render_hdr(out / 'final-concept-view.exr')[..., :3]
    C.write_png(out / 'final-no-bloom.png', T.srgb(lin))
    final = T.srgb(bloom(lin))
    C.write_png(out / 'final-concept-view.png', final)
    C.write_png(out / 'final-comparison.png', np.concatenate([m['img'], np.full((H, 12, 3), 20.0), final], 1))
    T.pair(m['img'], final, (300, 0, 880, 460), out / 'final-detail-collar.png')
    T.pair(m['img'], final, (250, 430, 800, 930), out / 'final-detail-back.png')
    T.pair(m['img'], final, (120, 900, 720, 1500), out / 'final-detail-tail.png')
    report['mean_abs_diff_8bit'] = float(np.abs(final - m['img']).mean())
    report['mean_abs_diff_inside_8bit'] = float(np.abs(final - m['img'])[m['mask']].mean())
    if args.turn:
        scene = bpy.context.scene
        cam = scene.camera
        pivot = bpy.data.objects.new('Pivot', None)
        pivot.location = (0, -0.08, H * C.MM / 2)
        scene.collection.objects.link(pivot)
        cam.parent = pivot
        cam.location = (0, -4, 0)
        bpy.data.objects['Backdrop'].hide_render = True
        bpy.data.objects['CapeV4_sparks'].hide_render = True
        scene.render.film_transparent = True
        backdrop = np.array([0.165, 0.127, 0.124], np.float32)
        shots = []
        for deg in (-60, -30, 30, 60, 180):
            pivot.rotation_euler = (0, 0, math.radians(deg))
            rgba = render_hdr(out / 'final-turn.exr')
            comp = rgba[..., :3] + (1 - rgba[..., 3:4]) * backdrop
            shot = T.srgb(bloom(comp))
            C.write_png(out / f'final-turn{deg:+04d}.png', shot)
            shots.append(shot[::2, ::2])
        C.write_png(out / 'final-turn-sheet.png', np.concatenate(shots, 1))
        (out / 'final-turn.exr').unlink(missing_ok=True)
    (out / 'final-report.json').write_text(json.dumps(report, indent=1))
    print('DEVICE', device, report)


if __name__ == '__main__':
    main()
