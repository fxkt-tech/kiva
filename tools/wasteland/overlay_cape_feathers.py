"""Draw the traced feather edges over the concept for checking. Start of each edge is a white dot."""
import argparse, json, sys
from pathlib import Path
import numpy as np
sys.path.insert(0, str(Path(__file__).resolve().parent))
import cape_v4_common as C

ap = argparse.ArgumentParser()
ap.add_argument('--concept', required=True)
ap.add_argument('--output', required=True)
ap.add_argument('--boxes', default='a:100,540,480,1040;b:440,540,820,1040;c:100,1000,480,1536;d:440,1000,820,1400')
ap.add_argument('--scale', type=int, default=2)
a = ap.parse_args(sys.argv[sys.argv.index('--') + 1:])
out = Path(a.output)
img = np.clip(255.0 * (C.load_rgb8(a.concept) / 255.0) ** 0.6, 0, 255)
edges = json.loads((out / 'feathers.json').read_text())
PAL = [(0, 255, 255), (255, 0, 255), (0, 255, 0), (80, 160, 255), (255, 255, 255), (160, 255, 120), (255, 120, 255), (120, 220, 255)]
legend = []
for spec in a.boxes.split(';'):
    name, box = spec.split(':')
    x0, y0, x1, y1 = map(int, box.split(','))
    s = a.scale
    big = np.repeat(np.repeat(img[y0:y1, x0:x1], s, 0), s, 1).copy()
    for y in range(y0, y1):
        if y % 100 == 0:
            big[(y - y0) * s] = (255, 255, 0)
    for x in range(x0, x1):
        if x % 100 == 0:
            big[:, (x - x0) * s] = (255, 255, 0)
    for i, (key, pts) in enumerate(edges.items()):
        col = PAL[i % len(PAL)]
        if name == 'a':
            legend.append((key, col))
        for (xa, ya), (xb, yb) in zip(pts[:-1], pts[1:]):
            n = int(max(abs(xb - xa), abs(yb - ya)) * s) + 1
            for t in np.linspace(0, 1, n):
                px = int(round((xa + (xb - xa) * t - x0) * s))
                py = int(round((ya + (yb - ya) * t - y0) * s))
                if 0 <= px < big.shape[1] - 1 and 0 <= py < big.shape[0] - 1:
                    big[py:py + 2, px:px + 2] = col
        px, py = int((pts[0][0] - x0) * s), int((pts[0][1] - y0) * s)
        if 3 <= px < big.shape[1] - 4 and 3 <= py < big.shape[0] - 4:
            big[py - 3:py + 4, px - 3:px + 4] = (255, 255, 255)
    C.write_png(out / 'analysis' / f'overlay-{name}.png', big)
print('LEGEND', legend)
