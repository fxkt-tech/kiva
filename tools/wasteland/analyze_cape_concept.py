"""Measure the cape concept image: silhouette mask, ember map and gridded zoom tiles.

Runs inside background Blender only for its bundled numpy. Writes PNGs with stdlib zlib.
"""
from __future__ import annotations

import argparse
import json
import struct
import sys
import zlib
from pathlib import Path

import bpy
import numpy as np


def write_png(path, rgb):
    rgb = np.ascontiguousarray(np.clip(rgb, 0, 255).astype(np.uint8))
    h, w, _ = rgb.shape
    raw = b''.join(b'\x00' + rgb[y].tobytes() for y in range(h))
    def chunk(tag, data):
        return struct.pack('>I', len(data)) + tag + data + struct.pack('>I', zlib.crc32(tag + data) & 0xffffffff)
    Path(path).write_bytes(b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', w, h, 8, 2, 0, 0, 0))
                           + chunk(b'IDAT', zlib.compress(raw, 6)) + chunk(b'IEND', b''))


def load(path):
    img = bpy.data.images.load(str(path))
    img.colorspace_settings.name = 'Non-Color'
    w, h = img.size
    px = np.array(img.pixels[:], dtype=np.float32).reshape(h, w, 4)[::-1, :, :3]
    return (px * 255.0).round()


def box_blur(a, r):
    out = a.astype(np.float32)
    for axis in (0, 1):
        c = np.cumsum(np.pad(out, [(r + 1, r) if i == axis else (0, 0) for i in range(out.ndim)], mode='edge'), axis=axis)
        n = out.shape[axis]
        hi = np.take(c, np.arange(2 * r + 1, 2 * r + 1 + n), axis=axis)
        lo = np.take(c, np.arange(0, n), axis=axis)
        out = (hi - lo) / (2 * r + 1)
    return out


def largest_component(mask):
    h, w = mask.shape
    label = np.zeros((h, w), np.int32)
    best, best_n, cur = 0, 0, 0
    ys, xs = np.nonzero(mask)
    for sy, sx in zip(ys, xs):
        if label[sy, sx]:
            continue
        cur += 1
        stack = [(sy, sx)]
        label[sy, sx] = cur
        n = 0
        while stack:
            y, x = stack.pop()
            n += 1
            for ny, nx in ((y - 1, x), (y + 1, x), (y, x - 1), (y, x + 1)):
                if 0 <= ny < h and 0 <= nx < w and mask[ny, nx] and not label[ny, nx]:
                    label[ny, nx] = cur
                    stack.append((ny, nx))
        if n > best_n:
            best, best_n = cur, n
    return label == best


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--concept', required=True)
    ap.add_argument('--output', required=True)
    args = ap.parse_args(sys.argv[sys.argv.index('--') + 1:])
    out = Path(args.output)
    img = load(args.concept)
    h, w, _ = img.shape
    r, g, b = img[..., 0], img[..., 1], img[..., 2]
    lum = 0.299 * r + 0.587 * g + 0.114 * b
    border = np.concatenate([img[:40].reshape(-1, 3), img[-40:].reshape(-1, 3), img[:, :40].reshape(-1, 3), img[:, -40:].reshape(-1, 3)])
    bg = np.median(border, axis=0)
    bg_lum = float(0.299 * bg[0] + 0.587 * bg[1] + 0.114 * bg[2])
    dark = lum < bg_lum - 22
    ember = (r > 150) & (r - b > 70)
    hot = (r > 215) & (g > 120) & (r - b > 110)
    mask = largest_component(dark | hot | ((r - b > 55) & (lum < bg_lum - 8)))
    # fill pinholes: anything not connected to the border background
    inv = ~mask
    seed = np.zeros_like(inv)
    seed[0, :] = seed[-1, :] = seed[:, 0] = seed[:, -1] = True
    outside = largest_component(inv)
    mask = ~outside
    write_png(out / 'mask.png', np.repeat((mask * 255)[..., None], 3, 2))
    em = np.zeros((h, w, 3))
    em[..., 0] = ember * 160 + hot * 95
    em[..., 1] = hot * 255
    em[..., 2] = mask * 70
    write_png(out / 'ember.png', em)
    # low frequency shading cue inside mask
    shade = box_blur(lum, 6)
    write_png(out / 'lum-blur.png', np.repeat(np.clip(shade * 3.2, 0, 255)[..., None], 3, 2))
    rows = {}
    for y in range(0, h, 10):
        xs = np.nonzero(mask[y])[0]
        if len(xs):
            rows[y] = [int(xs[0]), int(xs[-1]), int(len(xs))]
    cols = {}
    for x in range(0, w, 10):
        ys = np.nonzero(mask[:, x])[0]
        if len(ys):
            cols[x] = [int(ys[0]), int(ys[-1])]
    stats = {'size': [w, h], 'background_rgb': [float(v) for v in bg], 'mask_pixels': int(mask.sum()),
             'ember_pixels': int(ember.sum()), 'hot_pixels': int(hot.sum()), 'rows': rows, 'cols': cols}
    (out / 'concept-measure.json').write_text(json.dumps(stats))
    np.save(out / 'mask.npy', mask)
    # gridded 2x tiles, contrast lifted so dark folds are readable
    lifted = np.clip(255.0 * (img / 255.0) ** 0.55, 0, 255)
    tile = 384
    for ty in range(0, h, tile):
        for tx in range(0, w, tile):
            crop = lifted[ty:ty + tile, tx:tx + tile]
            big = np.repeat(np.repeat(crop, 2, 0), 2, 1).copy()
            for gy in range(0, crop.shape[0], 32):
                yy = (ty + gy)
                big[gy * 2, :, :] = (0, 255, 255) if yy % 128 == 0 else big[gy * 2] * 0.6 + np.array([0, 90, 90])
            for gx in range(0, crop.shape[1], 32):
                xx = tx + gx
                big[:, gx * 2, :] = (0, 255, 255) if xx % 128 == 0 else big[:, gx * 2] * 0.6 + np.array([0, 90, 90])
            write_png(out / f'tile-x{tx:04d}-y{ty:04d}.png', big)
    print('BG', bg, 'mask', mask.sum(), 'ember', ember.sum(), 'hot', hot.sum())


main()
