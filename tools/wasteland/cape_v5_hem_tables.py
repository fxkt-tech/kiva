"""Write the extracted hem curves to an .npz the build reads (per-column, view-local rows)."""
import json, sys
from pathlib import Path
import numpy as np

args = sys.argv[sys.argv.index('--') + 1:]
data = json.loads(Path(args[0]).read_text())
out = {}
for name, view in data.items():
    hem = view['hems']
    x = np.arange(len(hem['outer']))
    out[f'{name}_x'] = x
    for layer in ('outer', 'mid', 'inner'):
        # keep local dips and tips: light despike only
        a = np.array(hem[layer], np.float32)
        for _ in range(2):
            med = np.copy(a)
            for i in range(1, len(a) - 1):
                med[i] = np.median(a[i - 1:i + 2])
            a = med
        out[f'{name}_{layer}'] = a
np.savez('art/wasteland/cape-v5/hem-tables.npz', **out)
print({k: (float(v.min()), float(v.max())) for k, v in out.items() if not k.endswith('_x')})
