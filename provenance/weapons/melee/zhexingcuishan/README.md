# 谪星·摧山2311638

Generated from frozen `assets/prepared/naraka/zhexingcuishan` by `scripts/import-melee-models.py`. `weapon.bin` is existing KIVAWP01; every part uses Grip0, +Y length, 1.10m total length. The right grip and unused second grip are zero. Held and flying instances share this asset.

Kiva static assembly of source-proven complementary blade/back/head/handle groups plus original main shaft; both FX inputs excluded.

The four non-FX inputs are complementary original controller groups (back, blade, head, handle), not four interchangeable whole blades. Their per-piece transforms are explicitly authored Kiva static composition, documented in both JSON reports. The head anchor references source rule interval0..99; neither that star rule nor its shape-index8 is executed. This is not a recovered original default/randomized appearance. No Float/packed deformation texture is sampled; the clamped PNG previews are never used as geometry.

Original base RGB/alpha, UV, triangles, cull/cutoff and prepared neutral PBR factors are retained. Normal/packed/flow/ID/Float maps are not guessed as colors or ORM. Original attack/morph, stars, dissolve, spirit-light and cubemap/FX behavior are not restored. Grip, display size and static composition are Kiva adaptations; simulation reach, timing and physics do not change. No browser/rendering acceptance is claimed.

`uv run --offline --with numpy --with pillow python scripts/import-melee-models.py --verify` validates pinned source files and helpers, current-LBS, exact binary geometry, and rebuilds all four files byte for byte in a temporary directory.
