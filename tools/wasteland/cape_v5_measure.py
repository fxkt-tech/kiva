"""Measure the v03 three-view sheet: split the views, mask cloak / torso / glow, write traced crops.

Importable: measure(path) returns per-view dictionaries in sheet pixels.
"""
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

import numpy as np

sys.path.insert(0, str(Path(__file__).resolve().parent))
import cape_v4_common as C

VIEWS = ('front', 'side', 'back')
LABEL_ROWS = 60          # FRONT / RIGHT SIDE / BACK captions


def measure(path):
    img = C.load_rgb8(path)
    h, w, _ = img.shape
    r, g, b = img[..., 0], img[..., 1], img[..., 2]
    lum = 0.299 * r + 0.587 * g + 0.114 * b
    border = np.concatenate([img[70:, :12].reshape(-1, 3), img[70:, -12:].reshape(-1, 3), img[-12:].reshape(-1, 3)])
    bg = np.median(border, axis=0)
    bg_lum = float(0.299 * bg[0] + 0.587 * bg[1] + 0.114 * bg[2])
    warm = r - b
    cloak = ((lum < bg_lum - 20) | (warm > 48))
    cloak[:LABEL_ROWS] = False
    cloak = C.blur(cloak.astype(np.float32), 1.0) > 0.5
    cols = cloak.sum(0)
    filled = cols > 6
    runs, start = [], None
    for x in range(w):
        if filled[x] and start is None:
            start = x
        if (not filled[x] or x == w - 1) and start is not None:
            if x - start > 60:
                runs.append((start, x))
            start = None
    # merge runs separated by small gaps (front view has two hanging sides)
    merged = [runs[0]]
    for a, b2 in runs[1:]:
        if a - merged[-1][1] < 150 and len(merged) + (len(runs) - runs.index((a, b2))) > 3:
            merged[-1] = (merged[-1][0], b2)
        else:
            merged.append((a, b2))
    views = {}
    for name, (x0, x1) in zip(VIEWS, merged):
        x0, x1 = max(0, x0 - 24), min(w, x1 + 24)
        sub = cloak[:, x0:x1]
        keep = C.largest_component(sub)
        # keep every sizeable component (front view is two hanging halves joined at the scarf)
        ys = np.nonzero(sub.any(1))[0]
        y0, y1 = max(0, int(ys.min()) - 16), min(h, int(ys.max()) + 16)
        mask = sub[y0:y1]
        # lit folds can be as bright as the backdrop: fill enclosed holes and close notches in the scarf
        mask = ~C.largest_component(~mask)
        grown = C.blur(mask.astype(np.float32), 4) > 0.2
        closed = C.blur(grown.astype(np.float32), 4) > 0.8
        mask = mask | (closed & (np.arange(mask.shape[0])[:, None] < 230))
        rgb = img[y0:y1, x0:x1]
        lum_v = lum[y0:y1, x0:x1]
        warm_v = warm[y0:y1, x0:x1]
        sat = rgb.max(-1) - rgb.min(-1)
        torso = (~mask) & (np.abs(lum_v - bg_lum) > 7) & (sat < 26) & (lum_v > bg_lum - 20)
        torso = C.blur(torso.astype(np.float32), 2.5) > 0.55
        views[name] = {'box': (x0, y0, x1, y1), 'img': rgb, 'mask': mask, 'torso': torso, 'warm': warm_v,
                       'ember': (rgb[..., 0] > 150) & (warm_v > 70), 'hot': (rgb[..., 0] > 215) & (rgb[..., 1] > 120) & (warm_v > 110)}
    return {'bg': bg, 'size': (w, h), 'views': views}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('--sheet', required=True)
    ap.add_argument('--output', required=True)
    args = ap.parse_args(sys.argv[sys.argv.index('--') + 1:])
    out = Path(args.output) / 'analysis'
    data = measure(args.sheet)
    summary = {'background': [float(v) for v in data['bg']]}
    for name, v in data['views'].items():
        x0, y0, x1, y1 = v['box']
        mask = v['mask']
        ys, xs = np.nonzero(mask)
        summary[name] = {'box': [x0, y0, x1, y1], 'cloak_rows': [int(ys.min()), int(ys.max())], 'cloak_cols': [int(xs.min()), int(xs.max())],
                         'cloak_px': int(mask.sum()), 'torso_px': int(v['torso'].sum()), 'ember_px': int(v['ember'].sum())}
        lifted = np.clip(255.0 * (v['img'] / 255.0) ** 0.6, 0, 255)
        scale = 2
        big = np.repeat(np.repeat(lifted, scale, 0), scale, 1).copy()
        hh, ww = mask.shape
        for y in range(0, hh, 20):
            big[y * scale] = (255, 255, 0) if y % 100 == 0 else big[y * scale] * 0.6 + np.array([0, 90, 90])
        for x in range(0, ww, 20):
            big[:, x * scale] = (255, 255, 0) if x % 100 == 0 else big[:, x * scale] * 0.6 + np.array([0, 90, 90])
        C.write_png(out / f'view-{name}.png', big)
        seg = np.zeros((hh, ww, 3))
        seg[mask] = (70, 70, 70)
        seg[v['torso']] = (60, 110, 200)
        seg[v['ember'] & mask] = (255, 140, 0)
        C.write_png(out / f'seg-{name}.png', np.repeat(np.repeat(seg, scale, 0), scale, 1))
        widths = {}
        for y in range(0, hh, 20):
            row = np.nonzero(mask[y])[0]
            if len(row):
                widths[y] = [int(row[0]), int(row[-1]), int(len(row))]
        summary[name]['rows'] = widths
    (out / 'three-view-measure.json').write_text(json.dumps(summary))
    for name in VIEWS:
        s = summary[name]
        print(name, s['box'], 'rows', s['cloak_rows'], 'cols', s['cloak_cols'], 'cloak', s['cloak_px'], 'torso', s['torso_px'], 'ember', s['ember_px'])


if __name__ == '__main__':
    main()
