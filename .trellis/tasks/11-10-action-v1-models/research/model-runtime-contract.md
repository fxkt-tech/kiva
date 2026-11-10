# Model runtime contract — fixed 2026-11-10

Exports come only from frozen `assets/prepared/naraka`. Run with NumPy/Pillow:
`uv run --offline --with numpy --with pillow python scripts/import-prepared-models.py`.
No runtime GLTF, JSON parser, third-party Rust crate or external source path is needed.

## BaiLingmiao

`assets/bailingmiao/character.bin` is exact existing **KIVAAV03**, with no blade part.
The 23 driver names/order are the existing `import-naraka.py::DRIVERS`; source `gMan` names
are resolved against **female Lianmu** nodes. All palettes retain their own source skeleton,
original source joint order, sourceRestModel and inverse bind, including the female face.
Source +Y is up, +Z is forward. Uniform conversion C = yaw180 × scale plus floor translation;
complete outfit + Lianmu + generic female face current-LBS height becomes 1.73m, feet floor0.
`P'=C P; IBM'=IBM C^-1; sourceRestModel'=C originalBoneWorld`.
Do not replace current rest with inverse IBM. Animation uses the existing 23 motion deltas
and target female anchors, preserving female segments; neither male Li indices nor male face
are used. `naraka::` prefix prevents accidental source clip name matching.
Structural anchors come from Lianmu female reference, while each outfit/face palette retains
its own current matrix (including the source outfit foot/Toe 0.002615-unit difference).
All runtime PNGs have max side1024, sRGB base color. Original GLB neutral Lianmu alpha preview,
baseColorFactor, alpha cutoff and doubleSided are retained. Generic female face uses explicitly
recorded female-body-derived skin tint, neutral GRAY eye PBR tint, and lash PBR tint; no facial preset, makeup, hair SH/dye or packed shader claim.

## Ranged weapons

`assets/yinbingluo/weapon.bin` and `assets/yizhiying/weapon.bin` use **KIVAWP01**.
All integers/floats are little endian; string = u16 UTF8 byte length then bytes.
MAT4 elsewhere means 16 column-major f32. Counts are trusted bundled data but must be checked
against available bytes, palette/material/texture ranges, finite values, and exact EOF.

1. magic8 `KIVAWP01`
2. u32 header length, UTF8 JSON header (offline provenance; runtime skips bytes)
3. u16 texture count. Each: string name, u8 sRGB(1), u32 PNG length, PNG bytes.
4. u16 material count. Each exactly AV03: string name, i16 base texture, i16 normal(-1),
   i16 surface(-1), u8 alpha mask, **11 f32** (roughness, metallic, reflectance,
   anisotropy, clearcoat, clearcoat roughness, transmission, tintR,G,B, cutoff), u8 doubleSided.
5. **6 f32**: primary-hand grip XYZ, secondary/support/draw grip XYZ, in muzzle coordinates.
6. u16 part count. Each: string name, u8 mount (0=muzzle/arrow root; 1=spine_03),
   u16 primitive count. Each primitive: u16 material, u32 vertex count, u32 index count;
   vertices each **8 f32 /32bytes** (positionXYZ, normalXYZ, UV0XY), then u32 triangle indices.

Part mount1 vertices are already centered/rotated/offset in spine_03-relative Kiva coordinates.
Runtime parents these parts under the female `attachment_pose` spine_03 position/rotation,
never under muzzle. This is an explicit Kiva body attachment, not a recovered Actor mount.
Normal and packed/em/SH maps remain disconnected. Actual GLB main RGB/alpha preserved.
All configured geometry is retained. Gun includes both SMRs and true joint1 vertices; bow
includes all9 weighted original bones. Current-pose LBS is baked before rigid axis adaptation;
no original fire/draw deformation or animation is asserted.

### Fixed geometry adaptation

**Yinbingluo:** source forward+Z becomes Kiva-Z, source+Y stays+Y; scale1m/source unit.
Muzzle source point `[0,0.0582,1.01312304]`, near the actual far barrel cross section.
Kiva conversion is diag(-1,+1,-1) around that point. Primary/right grip source `[0,0,0.25]`
becomes `[0,-0.0582,0.76312304]`; support/left source `[0,0.03,0.60]` becomes
`[0,-0.0282,0.41312304]`. Muzzle0 is the existing simulation firing source.
The roughly1.348m source length is preserved. Grip and barrel-center choices are Kiva adaptation.

**Yizhiying:** source+Z (limbs) becomes Kiva+Y, source+Y becomes Kiva+Z, source+X becomes-X,
rotation `[-1,0,0; 0,0,1; 0,1,0]`, uniform scale0.66. Origin translation `[0,0,0.08]`.
Source origin/LEFT grip is Kiva `[0,0,0.08]`; RIGHT draw-hand point `[0,0,0.35]`.
The six serialized grip floats are always RIGHT first, LEFT second.
Arrow forward-Z and source0 are the existing simulation source. Bow visible height≈1.552m.
Quiver and arrowbag are separately current-world baked and placed around spine_03, preserving
geometry/materials. Original related arrow remains frozen/neutral and is not packed into this runtime weapon:
no configured source material or Actor assembly has been recovered for it.

## Reports and verification

Each output directory contains `source-manifest.json`, `conversion-report.json`, README.
Reports use relative paths, deterministic output hashes, exact input GLB/manifest SHA256,
actual skin weights/indices and nonbind matrices, f32/quantized-LBS deviation, normal/UV checks,
bounds and PNG color space/resize evidence. Repeated full builds must be byte-identical;
`--verify` checks source manifest hashes, rebuilds in a temporary directory and compares outputs.
No browser, render or interaction acceptance follows from these numerical checks.
