# 伏浪锚2310478

Generated from frozen `assets/prepared/naraka/fulangmao` by `scripts/import-melee-models.py`. `weapon.bin` is existing KIVAWP01; every part uses Grip0, +Y length, 1.10m total length. The right grip and unused second grip are zero. Held and flying instances share this asset.

Both original configured LOD0 skinned instances, with their actual current source pose.

Both original skinned instances are baked from their actual hierarchy/current-LBS. The five-slot native four-weight body and the 448 original UInt32 single-index/implicit-weight1 attachment are retained.

Original base RGB/alpha, UV, triangles, cull/cutoff and prepared neutral PBR factors are retained. Normal/packed/flow/ID/Float maps are not guessed as colors or ORM. Original attack/morph, stars, dissolve, spirit-light and cubemap/FX behavior are not restored. Grip, display size and static composition are Kiva adaptations; simulation reach, timing and physics do not change. No browser/rendering acceptance is claimed.

`uv run --offline --with numpy --with pillow python scripts/import-melee-models.py --verify` validates pinned source files and helpers, current-LBS, exact binary geometry, and rebuilds all four files byte for byte in a temporary directory.
