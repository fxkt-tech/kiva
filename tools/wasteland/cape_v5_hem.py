"""Per-column layer hems for the v03 views.

The silhouette gives the longest layer's hem exactly.  Upper layers end on the glowing
burned edges drawn in the sheet, so each visible hem line is read column by column and
tracked as one continuous curve (three layers), then written as a JSON table and a
coloured overlay for checking.
"""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

import numpy as np

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import cape_v4_common as C
import cape_v5_measure as M

LAYERS = ('outer', 'mid', 'inner')
SEEDS = {'outer': (0.36, 0.50), 'mid': (0.56, 0.68), 'inner': (0.92, 1.0)}   # fractions of the cloak's row span


def glow(view):
    warm = view['warm']
    return (((warm > 80) & (warm > C.blur(warm, 4) + 12)) | view['hot']) & view['mask']


def long_ridges(ridge, min_size=45):
    """Drop spark specks: keep only connected glow lines long enough to be a burned edge."""
    h, w = ridge.shape
    seen = np.zeros_like(ridge)
    keep = np.zeros_like(ridge)
    ys, xs = np.nonzero(ridge)
    for sy, sx in zip(ys.tolist(), xs.tolist()):
        if seen[sy, sx]:
            continue
        stack, members = [(sy, sx)], []
        seen[sy, sx] = True
        while stack:
            y, x = stack.pop()
            members.append((y, x))
            for dy in (-1, 0, 1):
                for dx in (-1, 0, 1):
                    ny, nx = y + dy, x + dx
                    if 0 <= ny < h and 0 <= nx < w and ridge[ny, nx] and not seen[ny, nx]:
                        seen[ny, nx] = True
                        stack.append((ny, nx))
        arr = np.array(members)
        if len(arr) >= min_size and (np.ptp(arr[:, 1]) >= 14 or np.ptp(arr[:, 0]) >= 30):
            keep[arr[:, 0], arr[:, 1]] = True
    return keep


def clusters(rows, gap=4, min_px=2):
    if len(rows) == 0:
        return []
    groups = []
    start = rows[0]
    prev = rows[0]
    count = 1
    for r in rows[1:]:
        if r - prev <= gap:
            count += 1
        else:
            groups.append((0.5 * (start + prev), count))
            start, count = r, 1
        prev = r
    groups.append((0.5 * (start + prev), count))
    return [c for c in groups if c[1] >= min_px]


def track(g, env, cols, start_col, seed_row, tolerance=40, lost_limit=10):
    """Follow one burned edge away from its strongest column.  The edge is a finite stroke:
    when it runs out the layer's hem eases off toward the cloth silhouette instead of
    streaking across the whole view."""
    path = {}
    for direction in (1, -1):
        slope = 0.0
        prev = seed_row
        lost = 0
        x = start_col
        while 0 <= x < g.shape[1]:
            if cols[x]:
                cs = clusters(np.nonzero(g[:, x])[0])
                target = prev + slope
                near = [c for c in cs if abs(c[0] - target) < tolerance and c[0] < env[x] - 6]
                if near:
                    best = min(near, key=lambda c: abs(c[0] - target) - 1.0 * min(c[1], 8))
                    prev = 0.7 * best[0] + 0.3 * target
                    slope = 0.75 * slope + 0.25 * np.clip(prev - target, -3, 3)
                    lost = 0
                else:
                    lost += 1
                    if lost > lost_limit:
                        # glide, without crack support, onto the silhouette
                        prev = prev + slope + 0.35 * (env[x] - 8 - prev) * 0.12
                        slope = 0.9 * slope
                    else:
                        prev = min(target, env[x] - 6)
            path[x] = prev
            x += direction
    return np.array([path[x] for x in range(g.shape[1])])


def extract(view, name):
    mask = view['mask']
    h, w = mask.shape
    g = long_ridges(glow(view))
    env = np.where(mask.any(0), h - 1 - np.argmax(mask[::-1], axis=0), np.nan)
    ys_any = np.nonzero(mask.any(1))[0]
    top, bottom = ys_any.min(), ys_any.max()
    env_f = np.nan_to_num(env, nan=bottom)
    cols = ~np.isnan(env)
    result = {}
    inner = np.convolve(np.pad(np.array(env_f, np.float32), 4, mode='edge'), np.hanning(9) / np.hanning(9).sum(), 'valid')
    result['inner'] = inner
    # the sheet's layer hems are one family of diagonal sweeps; trace each burning edge and
    # put them in order by how high they sit, so every edge becomes one layer hem
    g_t = g.copy()
    traces = []
    for _ in range(4):
        if g_t.sum() < 60:
            break
        weights = g_t.sum(0)
        start_col = int(np.argmax(weights))
        seed_row = float(np.nonzero(g_t[:, start_col])[0].mean())
        path = track(g_t, env_f, cols, start_col, seed_row)
        support = np.convolve(np.clip(weights, 0, 1), np.ones(25) / 25, 'valid')
        pad = (w - len(support)) // 2
        support = np.pad(support, (pad, w - len(support) - pad), mode='edge')
        # erase this stroke so the next trace finds another edge
        take = support > 0.02
        for x in np.nonzero(take)[0]:
            lo = max(0, int(path[x]) - 26)
            hi = min(h, int(path[x]) + 26)
            g_t[lo:hi, x] = False
        if support.max() > 0.05:
            traces.append((path, support))
    if traces:
        traces.sort(key=lambda t: float(np.mean(t[0])))
        outer = traces[0][0]
        mid = traces[min(1, len(traces) - 1)][0]
    else:
        outer = np.clip(inner - 260, top + 60, env_f - 6)
        mid = np.clip(inner - 130, top + 60, env_f - 6)
    k = 4
    outer = np.convolve(np.pad(outer, k, mode='edge'), np.hanning(2 * k + 1) / np.hanning(2 * k + 1).sum(), 'valid')
    mid = np.convolve(np.pad(mid, k, mode='edge'), np.hanning(2 * k + 1) / np.hanning(2 * k + 1).sum(), 'valid')
    result['outer'] = np.clip(outer, top + 60, env_f - 6).astype(np.float32)
    result['mid'] = np.clip(mid, top + 60, env_f - 6).astype(np.float32)
    order = [np.minimum(result['outer'], result['mid'] - 10), np.minimum(result['mid'], result['inner'] - 10), result['inner']]
    result = dict(zip(LAYERS, order))
    return {'env': env, 'tops': [int(top), int(bottom)], 'hems': {k: v.tolist() for k, v in result.items()}, 'glow': g}


def overlay(view, data, path, name, scale=2):
    img = np.clip(255.0 * (view['img'] / 255.0) ** 0.65, 0, 255)
    big = np.repeat(np.repeat(img, scale, 0), scale, 1).copy()
    colors = {'outer': (0, 255, 255), 'mid': (0, 255, 0), 'inner': (255, 0, 255)}
    for layer, rows in data['hems'].items():
        for x, y in enumerate(rows):
            if not np.isfinite(y):
                continue
            yy = int(round(y * scale))
            xx = int(round(x * scale))
            if 0 <= yy < big.shape[0] - scale and 0 <= xx < big.shape[1] - scale:
                big[yy:yy + scale, xx:xx + scale] = colors[layer]
    C.write_png(path, big)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--sheet', required=True)
    ap.add_argument('--output', required=True)
    args = ap.parse_args(sys.argv[sys.argv.index('--') + 1:])
    out = Path(args.output) / 'analysis'
    data = M.measure(args.sheet)
    report = {}
    for name in ('back', 'side', 'front'):
        view = data['views'][name]
        got = extract(view, name)
        overlay(view, got, out / f'hem-{name}.png', name)
        report[name] = {'tops': got['tops'], 'hems': got['hems']}
        print(name, 'rows', got['tops'], {k: (round(min(v), 1), round(max(v), 1)) for k, v in got['hems'].items()})
    (Path(args.output) / 'hem-tables.json').write_text(json.dumps(report))


if __name__ == '__main__':
    main()
