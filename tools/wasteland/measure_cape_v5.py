"""Score a clay render against the sheet: silhouette overlap per view and an outline overlay."""
import argparse, json, sys
from pathlib import Path
import numpy as np
sys.path.insert(0, str(Path(__file__).resolve().parent))
import cape_v4_common as C
import cape_v5_measure as M


def png_rgba(path):
    """Decode an 8-bit RGBA PNG with stdlib only (Blender's own numpy available here)."""
    import struct, zlib
    raw = Path(path).read_bytes()
    pos, idat, w = 8, b'', 0
    while pos < len(raw):
        n = struct.unpack('>I', raw[pos:pos + 4])[0]
        tag = raw[pos + 4:pos + 8]
        data = raw[pos + 8:pos + 8 + n]
        if tag == b'IHDR':
            w, h, depth, color = struct.unpack('>IIBB', data[:10])
        elif tag == b'IDAT':
            idat += data
        pos += 12 + n
    data = np.frombuffer(zlib.decompress(idat), np.uint8).reshape(h, 1 + w * 4)
    rows = np.zeros((h, w * 4), np.uint8)
    prev = np.zeros(w * 4, np.int32)
    for y in range(h):
        f = data[y, 0]
        cur = data[y, 1:].astype(np.int32)
        if f == 1:
            for i in range(4, w * 4):
                cur[i] = (cur[i] + cur[i - 4]) & 255
        elif f == 2:
            cur = (cur + prev) & 255
        elif f == 3:
            for i in range(w * 4):
                left = cur[i - 4] if i >= 4 else 0
                cur[i] = (cur[i] + (left + prev[i]) // 2) & 255
        elif f == 4:
            for i in range(w * 4):
                a_ = cur[i - 4] if i >= 4 else 0
                b_ = prev[i]
                c_ = prev[i - 4] if i >= 4 else 0
                pp = a_ + b_ - c_
                pa, pb, pc = abs(pp - a_), abs(pp - b_), abs(pp - c_)
                pred = a_ if (pa <= pb and pa <= pc) else (b_ if pb <= pc else c_)
                cur[i] = (cur[i] + pred) & 255
        rows[y] = cur
        prev = cur
    return w, h, rows.reshape(h, w, 4)

ap = argparse.ArgumentParser()
ap.add_argument('--sheet', required=True)
ap.add_argument('--output', required=True)
ap.add_argument('--tag', default='clay')
ap.add_argument('--scale', type=int, default=2)
a = ap.parse_args(sys.argv[sys.argv.index('--') + 1:])
out = Path(a.output)
data = M.measure(a.sheet)
report = {}
panels = []
for name in ('front', 'side', 'back'):
    view = data['views'][name]
    ref = view['mask']
    s = a.scale
    # renders are RGBA; read the alpha channel directly
    img_path = out / f'{a.tag}-{name}.png'
    w, h, px = png_rgba(img_path)
    alpha = (px[:ref.shape[0] * s, :ref.shape[1] * s, 3] > 0.5)
    if alpha.shape != (ref.shape[0] * s, ref.shape[1] * s):
        pad = np.zeros((ref.shape[0] * s, ref.shape[1] * s), bool)
        pad[:alpha.shape[0], :alpha.shape[1]] = alpha
        alpha = pad
    small = alpha.reshape(ref.shape[0], s, ref.shape[1], s).mean((1, 3)) > 0.5
    inter = (small & ref).sum()
    union = (small | ref).sum()
    report[name] = {'iou': float(inter / max(union, 1)), 'render_px': int(small.sum()), 'sheet_px': int(ref.sum())}
    # outline overlay: sheet mask edges in magenta, render silhouette edges in cyan
    base = view['img'].copy()
    def edges(m):
        e = np.zeros_like(m)
        e[:-1] |= m[:-1] != m[1:]
        e[:, :-1] |= m[:, :-1] != m[:, 1:]
        return e
    e_ref, e_ren = edges(ref), edges(small)
    big = np.repeat(np.repeat(base, s, 0), s, 1)
    e_ref_b = np.repeat(np.repeat(e_ref, s, 0), s, 1)
    e_ren_b = np.repeat(np.repeat(e_ren, s, 0), s, 1)
    big[e_ref_b] = (255, 0, 255)
    big[e_ren_b] = (0, 255, 255)
    panels.append(big.astype(np.float32))
hmax = max(p.shape[0] for p in panels)
padded = [np.concatenate([p, np.tile(data['bg'], (hmax - p.shape[0], p.shape[1], 1))], 0) for p in panels]
row = padded[0]
for p in padded[1:]:
    row = np.concatenate([row, np.full((hmax, 10, 3), 20.0), p], 1)
C.write_png(out / f'{a.tag}-outline-match.png', row)
(out / f'{a.tag}-match.json').write_text(json.dumps(report, indent=1))
print('MATCH', report)
