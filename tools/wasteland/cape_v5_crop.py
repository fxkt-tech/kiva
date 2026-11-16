"""Gridded zoom of one sheet view in view-local pixel coordinates, optionally with traced lines drawn on top."""
import argparse, json, sys
from pathlib import Path
import numpy as np
sys.path.insert(0, str(Path(__file__).resolve().parent))
import cape_v4_common as C
import cape_v5_measure as M

ap = argparse.ArgumentParser()
ap.add_argument('--sheet', required=True)
ap.add_argument('--output', required=True)
ap.add_argument('--view', required=True)
ap.add_argument('--box', required=True, help='x0,y0,x1,y1 in view pixels')
ap.add_argument('--scale', type=int, default=3)
ap.add_argument('--name', required=True)
ap.add_argument('--trace', default='')
ap.add_argument('--gamma', type=float, default=0.65)
a = ap.parse_args(sys.argv[sys.argv.index('--') + 1:])
view = M.measure(a.sheet)['views'][a.view]
x0, y0, x1, y1 = map(int, a.box.split(','))
s = a.scale
img = np.clip(255.0 * (view['img'][y0:y1, x0:x1] / 255.0) ** a.gamma, 0, 255)
big = np.repeat(np.repeat(img, s, 0), s, 1).copy()
for y in range(y0, y1):
    if y % 20 == 0:
        row = (y - y0) * s
        big[row] = (255, 255, 0) if y % 100 == 0 else big[row] * 0.6 + np.array([0, 100, 100])
for x in range(x0, x1):
    if x % 20 == 0:
        col = (x - x0) * s
        big[:, col] = (255, 255, 0) if x % 100 == 0 else big[:, col] * 0.6 + np.array([0, 100, 100])
if a.trace:
    PAL = [(0, 255, 255), (255, 0, 255), (0, 255, 0), (90, 170, 255), (255, 255, 255), (170, 255, 120), (255, 130, 255)]
    lines = json.loads(Path(a.trace).read_text())[a.view]
    for i, (key, pts) in enumerate(lines.items()):
        col = PAL[i % len(PAL)]
        for (xa, ya), (xb, yb) in zip(pts[:-1], pts[1:]):
            n = int(max(abs(xb - xa), abs(yb - ya)) * s) + 1
            for t in np.linspace(0, 1, n):
                px, py = int(round((xa + (xb - xa) * t - x0) * s)), int(round((ya + (yb - ya) * t - y0) * s))
                if 0 <= px < big.shape[1] - 1 and 0 <= py < big.shape[0] - 1:
                    big[py:py + 2, px:px + 2] = col
        px, py = int((pts[0][0] - x0) * s), int((pts[0][1] - y0) * s)
        if 3 <= px < big.shape[1] - 4 and 3 <= py < big.shape[0] - 4:
            big[py - 3:py + 4, px - 3:px + 4] = (255, 255, 255)
C.write_png(Path(a.output) / 'analysis' / f'{a.name}.png', big)
