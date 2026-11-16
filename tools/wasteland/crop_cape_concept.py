"""Write a contrast-lifted, gridded, upscaled crop of an image for tracing coordinates."""
import argparse, sys
from pathlib import Path
import numpy as np
sys.path.insert(0, str(Path(__file__).resolve().parent))
import cape_v4_common as C

ap = argparse.ArgumentParser()
ap.add_argument('--image', required=True)
ap.add_argument('--out', required=True)
ap.add_argument('--box', required=True, help='x0,y0,x1,y1')
ap.add_argument('--scale', type=int, default=3)
ap.add_argument('--grid', type=int, default=20)
ap.add_argument('--gamma', type=float, default=0.55)
a = ap.parse_args(sys.argv[sys.argv.index('--') + 1:])
img = C.load_rgb8(a.image)
x0, y0, x1, y1 = map(int, a.box.split(','))
crop = np.clip(255.0 * (img[y0:y1, x0:x1] / 255.0) ** a.gamma, 0, 255)
big = np.repeat(np.repeat(crop, a.scale, 0), a.scale, 1).copy()
for y in range(y0, y1):
    if y % a.grid == 0:
        row = (y - y0) * a.scale
        big[row] = (255, 255, 0) if y % (a.grid * 5) == 0 else big[row] * 0.55 + np.array([0, 110, 110])
for x in range(x0, x1):
    if x % a.grid == 0:
        col = (x - x0) * a.scale
        big[:, col] = (255, 255, 0) if x % (a.grid * 5) == 0 else big[:, col] * 0.55 + np.array([0, 110, 110])
C.write_png(a.out, big)
