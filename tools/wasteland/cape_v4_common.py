"""Shared numpy/PNG helpers for the cape-v4 concept-matched build (background Blender only)."""
from __future__ import annotations

import struct
import zlib
from pathlib import Path

import bpy
import numpy as np

MM = 0.001  # one concept pixel is authored as one millimetre


def write_png(path, rgb):
    rgb = np.asarray(rgb)
    if rgb.ndim == 2:
        rgb = np.repeat(rgb[..., None], 3, 2)
    rgb = np.ascontiguousarray(np.clip(rgb, 0, 255).astype(np.uint8))
    h, w, _ = rgb.shape
    raw = b''.join(b'\x00' + rgb[y].tobytes() for y in range(h))

    def chunk(tag, data):
        return struct.pack('>I', len(data)) + tag + data + struct.pack('>I', zlib.crc32(tag + data) & 0xffffffff)

    Path(path).write_bytes(b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', w, h, 8, 2, 0, 0, 0))
                           + chunk(b'IDAT', zlib.compress(raw, 6)) + chunk(b'IEND', b''))


def load_rgb8(path):
    """Raw 8-bit display values, top row first."""
    img = bpy.data.images.load(str(path), check_existing=False)
    img.colorspace_settings.name = 'Non-Color'
    w, h = img.size
    px = np.empty(w * h * 4, dtype=np.float32)
    img.pixels.foreach_get(px)
    bpy.data.images.remove(img)
    return (px.reshape(h, w, 4)[::-1, :, :3] * 255.0).round()


def blur(a, r):
    """Three box passes ~ gaussian of sigma ~= r."""
    out = a.astype(np.float32)
    if r <= 0:
        return out
    k = max(1, int(round(r * 0.58)))
    for _ in range(3):
        for axis in (0, 1):
            pad = [(k + 1, k) if i == axis else (0, 0) for i in range(2)]
            c = np.cumsum(np.pad(out, pad, mode='edge'), axis=axis, dtype=np.float64)
            n = out.shape[axis]
            hi = np.take(c, np.arange(2 * k + 1, 2 * k + 1 + n), axis=axis)
            lo = np.take(c, np.arange(0, n), axis=axis)
            out = ((hi - lo) / (2 * k + 1)).astype(np.float32)
    return out


def masked_blur(a, weight, r):
    """Normalised convolution: blur only where weight > 0, fill elsewhere."""
    num = blur(a * weight, r)
    den = blur(weight.astype(np.float32), r)
    return num / np.maximum(den, 1e-4)


def largest_component(mask):
    h, w = mask.shape
    label = np.zeros((h, w), np.int32)
    best, best_n, cur = 0, 0, 0
    ys, xs = np.nonzero(mask)
    for sy, sx in zip(ys.tolist(), xs.tolist()):
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


def bilinear(field, xs, ys):
    h, w = field.shape
    xs = np.clip(xs, 0, w - 1.001)
    ys = np.clip(ys, 0, h - 1.001)
    x0 = np.floor(xs).astype(int)
    y0 = np.floor(ys).astype(int)
    fx = xs - x0
    fy = ys - y0
    return (field[y0, x0] * (1 - fx) * (1 - fy) + field[y0, x0 + 1] * fx * (1 - fy)
            + field[y0 + 1, x0] * (1 - fx) * fy + field[y0 + 1, x0 + 1] * fx * fy)


def smoothstep(a, b, x):
    t = np.clip((x - a) / (b - a), 0, 1)
    return t * t * (3 - 2 * t)
