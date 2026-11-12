# 羿之影2310557

Generated offline by `scripts/import-prepared-models.py` from frozen `assets/prepared/naraka/yizhiying`. Runtime file: `weapon.bin` (KIVAWP01).

Original current source pose LBS baked rigid; no source fire/draw clips, special shader, emission or packed mapping.

Only source base-color PNGs / alpha and explicit neutral PBR adaptations are used; normal/packed/SH/emission maps are not reinterpreted. Source palette and geometry provenance, grips/body mounts, texture resizing and numerical verification are in the two JSON reports. No browser, rendering or interaction acceptance is claimed.

`uv run --offline --with numpy --with pillow python scripts/import-prepared-models.py --verify` revalidates every frozen source file and rebuilds all outputs to compare byte for byte.
