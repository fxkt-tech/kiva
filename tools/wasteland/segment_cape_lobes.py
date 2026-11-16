"""Split the concept silhouette into the cloak body and separate feather lobes.

Glowing torn edges act as barriers; a distance-transform watershed grows one region per lobe.
Manual barrier strokes (BARRIERS) close gaps where an edge is not lit.
"""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parent))
import cape_v4_common as C
import build_cape_v4 as B

BARRIERS: list = []      # [(x0, y0), (x1, y1), ...] polylines in concept pixels
CORE = 6                 # marker = free space further than this from any barrier


def draw_lines(shape, lines, width=1.5):
    ys, xs = np.mgrid[0:shape[0], 0:shape[1]].astype(np.float32)
    out = np.zeros(shape, bool)
    for line in lines:
        for (x0, y0), (x1, y1) in zip(line[:-1], line[1:]):
            pad = 4
            xa, xb = int(min(x0, x1) - pad), int(max(x0, x1) + pad)
            ya, yb = int(min(y0, y1) - pad), int(max(y0, y1) + pad)
            sx, sy = xs[ya:yb, xa:xb], ys[ya:yb, xa:xb]
            dx, dy = x1 - x0, y1 - y0
            t = np.clip(((sx - x0) * dx + (sy - y0) * dy) / max(dx * dx + dy * dy, 1e-6), 0, 1)
            out[ya:yb, xa:xb] |= np.hypot(sx - x0 - t * dx, sy - y0 - t * dy) <= width
    return out


def distance(free, limit=80):
    """Octagonal distance to the nearest non-free pixel, capped."""
    dist = np.zeros(free.shape, np.int32)
    cur = free.copy()
    for i in range(limit):
        if not cur.any():
            break
        dist += cur
        p = np.pad(cur, 1, constant_values=False)
        if i % 2 == 0:
            cur = cur & p[:-2, 1:-1] & p[2:, 1:-1] & p[1:-1, :-2] & p[1:-1, 2:]
        else:
            cur = cur & np.logical_and.reduce([p[a:a + cur.shape[0], b:b + cur.shape[1]] for a in range(3) for b in range(3)])
    return dist


def components(mask, min_area):
    h, w = mask.shape
    label = np.zeros((h, w), np.int32)
    cur = 0
    ys, xs = np.nonzero(mask)
    for sy, sx in zip(ys.tolist(), xs.tolist()):
        if label[sy, sx]:
            continue
        cur += 1
        stack = [(sy, sx)]
        label[sy, sx] = cur
        members = []
        while stack:
            y, x = stack.pop()
            members.append((y, x))
            for ny, nx in ((y - 1, x), (y + 1, x), (y, x - 1), (y, x + 1)):
                if 0 <= ny < h and 0 <= nx < w and mask[ny, nx] and not label[ny, nx]:
                    label[ny, nx] = cur
                    stack.append((ny, nx))
        if len(members) < min_area:
            for y, x in members:
                label[y, x] = -1
            cur -= 1
    label[label < 0] = 0
    return label, cur


def grow(label, allowed):
    """Propagate labels into allowed unlabeled pixels until stable."""
    while True:
        p = np.pad(label, 1)
        todo = allowed & (label == 0)
        if not todo.any():
            return label
        new = label.copy()
        for sl in ((slice(0, -2), slice(1, -1)), (slice(2, None), slice(1, -1)), (slice(1, -1), slice(0, -2)), (slice(1, -1), slice(2, None))):
            nb = p[sl]
            take = todo & (new == 0) & (nb > 0)
            new[take] = nb[take]
        if (new == label).all():
            return label
        label = new


def segment(m, barriers=BARRIERS, core=CORE):
    mask = m['mask']
    warm = m['warm']
    ridge = (warm > 85) & (warm > C.blur(warm, 5) + 14)
    barrier = ridge | m['hot'] | draw_lines(mask.shape, barriers)
    free = mask & ~barrier
    dist = distance(free)
    label, n = components(dist >= core, 40)
    for level in range(core - 1, 0, -1):
        label = grow(label, dist >= level)
    label = grow(label, mask)
    return label, n, barrier


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--concept', required=True)
    ap.add_argument('--output', required=True)
    args = ap.parse_args(sys.argv[sys.argv.index('--') + 1:])
    out = Path(args.output) / 'analysis'
    m = B.measure(args.concept)
    label, n, barrier = segment(m)
    rng = np.random.default_rng(3)
    pal = rng.integers(60, 255, (n + 1, 3)).astype(np.float32)
    pal[0] = 0
    img = pal[label]
    edge = np.zeros(label.shape, bool)
    edge[:-1] |= label[:-1] != label[1:]
    edge[:, :-1] |= label[:, :-1] != label[:, 1:]
    img[edge] = 0
    img[barrier & m['mask']] = (255, 255, 255)
    for y in range(0, B.H, 100):
        img[y] = img[y] * 0.5 + 60
    for x in range(0, B.W, 100):
        img[:, x] = img[:, x] * 0.5 + 60
    C.write_png(out / 'lobes.png', img)
    info = {}
    for i in range(1, n + 1):
        ys, xs = np.nonzero(label == i)
        info[i] = {'area': int(len(ys)), 'cx': int(xs.mean()), 'cy': int(ys.mean()), 'box': [int(xs.min()), int(ys.min()), int(xs.max()), int(ys.max())],
                   'rgb': [int(v) for v in pal[i]]}
    (out / 'lobes.json').write_text(json.dumps(info, indent=1))
    print('REGIONS', n)
    for i, v in info.items():
        print(i, v['area'], v['cx'], v['cy'], v['box'])


if __name__ == '__main__':
    main()
