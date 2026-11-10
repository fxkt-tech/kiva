# Research: prepared model integration evidence

- Query: Which frozen prepared-model facts determine the female AV03 conversion, retargeting, current-pose weapon bake, material reuse, and local weapon axes for M01–M06?
- Scope: internal; actual GLB buffers, portable manifests, source evidence, current converter/runtime, and project specs.
- Date: 2026-11-10

## Findings

### Files found and source limits

| Path | Evidence / purpose |
| --- | --- |
| `assets/lihuowang/character.bin`, `README.md`, `source-manifest.json` | Existing AV03 brother, independent 23 anchors, current-pose palettes, and explicit Kiva adaptations; no new brother export required. |
| `assets/prepared/naraka/bailingmiao/models/{outfit,lianmu,generic_female_face}.glb` | Frozen female geometry, own palettes, actual current hierarchy, UVs, materials and embedded PNGs. |
| `assets/prepared/naraka/bailingmiao/source-evidence/origin-female_rig.json` | BodySize90005 / `l22_female_skeletonAvatar`, original female transforms; source metadata records Unity2019.4.41f2. |
| `assets/prepared/naraka/bailingmiao/source-evidence/components/*.source.json` | Original component / hair binding reconstruction evidence; not a runtime input. |
| `assets/prepared/naraka/yinbingluo/models/yinbingluo_main.glb` | Two real skinned renderers, original two material base maps, original implicit single-weight indices. |
| `assets/prepared/naraka/yizhiying/models/*.glb` | Nine-slot main bow; independent static quiver and arrowbag. |
| `assets/prepared/naraka/yizhiying/related/models/wp_bow_wuzhengong_arrow_asset_space.glb` | Related 635-vertex / 658-triangle arrow; no recovered equipped-part or material binding. |
| Each package's `portable_manifest.json`, `README.md`, `source-evidence/weapon-export.original.json` | Relative-path hashes, identity/configuration, exact map bindings and explicit reconstruction limits. |
| `scripts/import-naraka.py` | Proven AV03 reader/writer, actual LBS, coordinate conversion, palette drivers and quantization. |
| `scripts/import-bailingmiao.py`, `scripts/prepare-naraka-weapons.py` | Frozen-package verification and source preparation; `--verify` does not re-export. |
| `src/client/avatar.rs`, `stickman.rs`, `companion.rs`, `weapons.rs` | Target-proportion retarget, attachments, temporary companion lifetime, and local shooting-source contract. |

Prepared READMEs are preparation-state documentation: their “not integrated” text describes that earlier deliverable, not a prohibition on this task. Actual limitations remain relevant: female own skeleton; no FacePreset1416403 morph; no original cosmetic/dye/SH/cloth/hair shader; no original firing/drawing animation; unknown packed/normal/em/ID channels must not be guessed. See `assets/prepared/naraka/README.md:14`, `:16`, `:18`; `bailingmiao/README.md:5`, `:7`, `:9`; weapon READMEs `:3`, `:5`, `:11`.

### Frozen identities and hashes

Identity chain: outfit3402139 → ActorTemp1210776 → BodySize90005 → `l22_female_skeletonAvatar` → ActorVisualPart11602 (not NewJoy3402140); Lianmu HairItem100408 / HairVisual100471. Gun2311228 → WeaponTemp3110620. Bow2310557 → WeaponTemp3112195/3112196/3112197. Package README line anchors above and `assets/prepared/naraka/README.md:9` through `:12` establish these.

| Frozen model | SHA256 |
| --- | --- |
| female outfit | `3606106f8f8f55e76eab8063f733725f221363c50faa4945469d98feaf3bd8f2` |
| lianmu | `fddd7ce6283909ff017fc6c0c27d37eb57db8c2f8147529b84b95559971b8d05` |
| generic female face | `5053332a25a1291465077e389b6f15844c48ad13c1310e03490f63fe466c02a8` |
| yinbingluo main | `b2d29661f95e4783921ad0d0ff7bcd25788c7964f93110ff31abf2e5a38003c5` |
| yizhiying main | `5662d347923ff5fc8bf584a4290af47c88f37d1f5052b4b8a2acb05cfd0d06ee` |
| yizhiying quiver | `75d13676df9283c4a1aea4347a857585aa3a7a92a03d8178895d3d00d9e04ff1` |
| yizhiying arrowbag | `42171ac417e29058217f017a4d59725099fe00a7e829cca252facfcd66737f54` |

Portable manifests at research time: female `bdf24f30a81916a3864e2c62f6a29fbeb2975e4fb20c26e4426b46ca1319a7a6` (76 files); gun `3ab476488aef89294c1f95a88e42afbc1e7ea6c03ce14d843a7a54f30582d3fc` (20); bow `fbe98256865375192c3f97d4b2b7a0452e6352bf22dfa59ac3db25e493ed7136` (33). Every listed file's size/SHA matched. Calling existing `verify_package` also passed all three: female35 PNGs/3 GLBs, gun9/1, bow9/4 (includes related arrow). Use separate runtime output directories and record these original hashes; do not regenerate the prepared package.

### AV03 binary and matrix contracts

`scripts/import-naraka.py:260` writes `KIVAAV03`; `:630` onward serializes explicit little endian. Layout:

1. 8-byte magic; `u32` provenance JSON length + bytes.
2. `u16` driver count; each `u16` UTF8 byte length + name + `3f` model anchor.
3. `u16` textures; each name + `u8 srgb` + `u32` byte count + PNG bytes.
4. `u16` materials; each name + `hhhB11f` + `u8 doubleSided`. The signed texture indices are base/normal/surface (-1 absent); `B` is alpha-mask; the floats are roughness, metallic, reflectance, anisotropy, clearcoat, coat roughness, diffuse transmission, tint RGB, cutoff.
5. `u16` parts; each name + `u8 blade` + `u16` palette count. Every palette bone has name + `u16 driver` + column-major `16f IBM` + column-major `16f sourceRestModel`.
6. `u16` primitives; each `u16 material`, `u32 vertex_count`, `u32 index_count`; each vertex `8f8B` = position3/normal3/UV2/joints4/weights4; final indices `u32[]`. Weight bytes sum exactly255. Palette count fits u8 joint slots (female maximum78).

AV02 lacks doubleSided and sourceRestModel; it uses inverse(IBM) as rest. AV03 must retain actual imported rest instead. `scripts/import-naraka.py:514` applies:

```text
P' = C P
IBM' = IBM C^-1
Rest' = C originalJointWorld
posedSourceP = sum(weight[k] * JointWorld[j[k]] * IBM[k] * P)
```

`Glb.model` composes parent world × local column-major matrix (`scripts/import-naraka.py:90`). `Glb.posed` is the authoritative source pose (`:149`). For skinned meshes its world-joint LBS already gives source world coordinates: do not multiply mesh world again. Static parts instead apply node world once. Do not substitute joint0, identity IBMs, inverse(IBM) rest, or raw POSITION bounds for this formula.

Important distinction: nonidentity JointWorld×IBM alone is not evidence of a non-bind animation. For gun/bow these matrices match the mesh-node world within5.84e-7; for face/lianmu within2.32e-6. Their mesh world includes `(x,y,z) → (x,z,-y)`. Outfit really differs per bone from mesh-node world: body/body02 maximum matrix-component deviation1.994454, several cloth groups0.076601, body06_cb0.200051. AV03 sourceRest is therefore essential for the female outfit; keep source palettes even where the current result is nearly rigid.

### Female source coordinates, anchors and proportions

All posed coordinates below are actual LBS source-world coordinates, before Kiva normalization. Source +Y is up, source +Z is face/toe forward, source +X is the character's left. Bone names still use `gMan`; that naming does not make this the male skeleton.

| Component | Nodes / meshes / skins | Vertices / triangles | Posed min XYZ | Posed max XYZ |
| --- | --- | --- | --- | --- |
| outfit | 345 / 16 / 16 | 28693 / 41485 | `[-.5867263,.0031901,-.5225771]` | `[.5867270,1.8077447,.1811041]` |
| lianmu | 94 / 3 / 3 | 11848 / 11221 | `[-.2241437,.8153547,-.6871680]` | `[.2174521,1.8901676,.1303091]` |
| female face | 24 / 3 / 3 | 3260 / 5818 | `[-.0843554,1.5824104,-.0210476]` | `[.0843553,1.7867851,.1190701]` |

Union floor=.0031900560617017623, top=1.8901675600341004, height=1.8869775039723986 source units. If choosing the existing explicit1.73m union-height convention, this female's own scale is `.91681008192099`, with `C=diag(-s,+s,-s,1)` and `C[1,3]=-floor*s`. This is a target-size choice, not a recovered original meter measurement. Existing Li scale `.9128533132643297` must not be copied. This C is only a180° Y turn plus uniform scale/floor offset: GLBs have already undergone Unity conversion; do not introduce another X reflection. See `scripts/import-naraka.py:436`, `assets/lihuowang/README.md:23`.

`lianmu.glb` contains all22 unique named structural bones required for the23-driver layout on the original female63-node rig. Node indices and unconverted world positions:

| Driver | Original bone | Lianmu node | XYZ |
| --- | --- | --- | --- |
| root | synthetic origin | — | `[0,0,0]` |
| pelvis | gMan Pelvis | 5 | `[0,.9776640,0]` |
| spine_01 | gMan Spine | 14 | `[0,1.0601946,.0168860]` |
| spine_02 | gMan Spine1 | 15 | `[0,1.1741691,.0200523]` |
| spine_03 | gMan Spine2 | 16 | `[0,1.3231310,.0136853]` |
| neck_01 | gMan Neck | 37 | `[0,1.5493190,-.0216683]` |
| Head | gMan Head | 38 | `[0,1.6375835,-.0013074]` |
| clavicle_l | gMan L Clavicle | 17 | `[.0374503,1.5101849,-.0154913]` |
| upperarm_l | gMan L UpperArm | 18 | `[.1551508,1.4935601,-.0244408]` |
| lowerarm_l | gMan L Forearm | 19 | `[.3500706,1.2710181,-.0401183]` |
| hand_l | gMan L Hand | 20 | `[.4903555,1.0821091,.0705327]` |
| clavicle_r | gMan R Clavicle | 39 | `[-.0374504,1.5101849,-.0154912]` |
| upperarm_r | gMan R UpperArm | 40 | `[-.1551509,1.4935599,-.0244403]` |
| lowerarm_r | gMan R Forearm | 41 | `[-.3500709,1.2710181,-.0401175]` |
| hand_r | gMan R Hand | 42 | `[-.4903554,1.0821091,.0705336]` |
| thigh_l | gMan L Thigh | 6 | `[.0887498,.9776640,0]` |
| calf_l | gMan L Calf | 7 | `[.1462793,.5368404,-.0277875]` |
| foot_l | gMan L Foot | 8 | `[.2036418,.0976098,-.0550204]` |
| ball_l | gMan L Toe0 | 9 | `[.2180262,.0153158,.0764311]` |
| thigh_r | gMan R Thigh | 10 | `[-.0887498,.9776640,0]` |
| calf_r | gMan R Calf | 11 | `[-.1462795,.5368405,-.0277870]` |
| foot_r | gMan R Foot | 12 | `[-.2036422,.0976100,-.0550198]` |
| ball_r | gMan R Toe0 | 13 | `[-.2180262,.0153160,.0764318]` |

Outfit own same-named Head/Hand anchors agree within a few1e-6. Its Foot/Toe Y is+.0026146 above Lianmu rig, so never overwrite all component rest matrices with the common driver skeleton. One common female driver rig drives each component's retained own rest. Existing `driver_for` (`scripts/import-naraka.py:171`) resolves every outfit/hair/face palette slot without an unmapped bone, by nearest exact structural ancestor; isolated female facial nodes fall back to Head, while its gMan Neck can follow Neck. Hair28 slots comprise12+14+2, with reconstructed external links documented in `bailingmiao/README.md:7`.

Runtime pattern: `AvatarAssets::driver_pose` recursively accumulates each target anchor segment; `source_driver_pose` uses source motion delta with target anchor segment, and scales pelvis motion only. `joint_pose` multiplies those deltas into each target Rest'. `attachment_pose` uses the same retargeted wrist position with the authored aiming frame. See `src/client/avatar.rs:372`, `:399`, `:442`, `:467` at research time (concurrent implementation may move these). Extend the foreign-skeleton path currently selected for Lihuowang to the female; direct source.model positions would import another character's proportions. Keep Naraka-prefixed palette names so no incidental name/index match enters the Yanyun source hierarchy.

Numerical proof on all three actual female GLBs, using the illustrative1.73m C above: float C/LBS equivalence maximum8.8818e-16; real serialized float32 matrices/positions + existing largest-remainder weights255 maximum vertex error `.001117621258m` (outfit body02). This checks actual retained source pose, not only cancellation in a fabricated bind pose. `scripts/import-naraka.py:157`, `:529`, `:537` provide these existing patterns.

### Material facts, not inferred packed-map use

| GLB | Exact PNGs already embedded as baseColorTexture |
| --- | --- |
| outfit | female_body01_d; ch_f_ming_yulinglong_body02_lv_s16_d; body03_lv_s16_d |
| lianmu | derived hair_common_01_id_other_alpha_only; original ch_f_ming_yulinglong_hair02_lv_s19_d; derived scalp_f_default_d_alpha_only |
| generic female face | None. All three prepared material factors are `[1,1,1,1]`, roughness.75, metallic0. |
| yinbingluo main | wp_flintlock_bingxueniaochong_d; wp_flintlock_bingxueniaochong_01_d |
| yizhiying main / quiver | wp_bow_wuzhengong_d |
| yizhiying arrowbag | wp_bow_wuzhengong_arrow_d |

Embedded PNG bytes hash-identically to corresponding packaged PNGs, including the two clearly named derived alpha masks. Lianmu main/scalp material factor is light-gray `.45`; alpha cutoffs are.5/.9/.7 for main/hair02/scalp. Outfit original Cull Front instances already have winding/normal conversion applied; don't reverse them again. Bow main/quiver carry `doubleSided=true`; gun/arrowbag do not. Preserve these direct GLB facts. All embedded base/alpha images use sRGB; evidence-only normal/packed/em/ID/SH/flow/LUT PNGs are not additional proven base or ORM slots.

The existing Li material function (`scripts/import-naraka.py:491`) deliberately adds sampled **male** hand tint, authored brown eyes and dark lashes. Do not blindly reuse that function for female materials. Root accepted minimal explicitly authored female-neutral skin/eye/lash material tints under M06, without a new eye atlas; such tints are Kiva PBR adaptation and are not present in the prepared face GLB. No dedicated face morph/cosmetics/shader restoration is supported by these files.

### Weapon source pose, axes and bounds

| Model | Vertices / triangles | Source-world posed min XYZ | Source-world posed max XYZ |
| --- | --- | --- | --- |
| gun main combined | 5395 / 7106 | `[-.0627106,-.0790735,-.3352044]` | `[.0482289,.1376469,1.0131230]` |
| bow main | 6218 / 6844 | `[-.1252822,-.1939484,-1.1759041]` | `[.1252824,.3462373,1.1759040]` |
| quiver static | 1862 / 2356 | `[-.0124626,-.1521063,-.7262435]` | `[.1655917,.2416139,.6272474]` |
| arrowbag static | 2148 / 2474 | `[.0295456,-.0412956,-.7090033]` | `[.2160115,.2113296,.4858309]` |

Gun main first mesh4159 vertices/5464 triangles has one slot `bingxueniaochong_01`; second mesh1236/1642 has slots `_01`, `_02` with1101/135 vertices respectively, each implicit weight1. Joint origins are `[0,-.0230577,-.0922010]` and `[-.0384364,.0513282,.0628055]`. Original source single-index format is proven UInt32/dim1 and expanded to normalized glTF four lanes; original four-weight arrays do not exist (`yinbingluo/README.md:7`). Joint1 must survive any parsing/bake.

Gun longitudinal axis is source+Z. Its node inventory has root/body_dummy/MotionRoot/bingxueniaochong_01/_02/effect/LOD; **no named muzzle or grip**. Even `effect` is at zero, so it is not a recovered muzzle coordinate. Source geometry z>.9 has x±.025446, y[.020123,.102508], z[.905888,1.013123]; z>1.0 has x±.003328, y[.090849,.102508], z[1.003754,1.013123] (20 vertices). Thus taking the global zmax as the bore center is not justified by a named marker; it can include sight/ornament. Source grip/muzzle selections and uniform display size must be labeled authored Kiva anchors and checked against geometry. A180° Y rotation maps source+Z to existing local -Z; subtract the chosen muzzle after pose bake so its local position is exactly0.

Bow palette order is exactly `bow_gong_001`, `005`, `002`, `009`, `003`, `0010`, `004`, `011`, `008` (full prefix on each). All nine are weighted; summed vertex weights by slot are approximately `[681.254,722.495,1181.885,1037.950,684.712,753.704,568,568,20]`. String-center bone `bow_gong_008` is `[~0,.3307989,0]`; main central bone `bow_gong_001` `[0,-.0029936,0]`; tip bones004/011 `[~0,.2779374,-.7688426]` / `[~0,.2787955,.7729236]`. This geometry places the string on source+Y behind the grip and the long span on source±Z. A useful proper rotation for existing local -Z firing is Rx(+90°), `(x,y,z)→(x,-z,y)`: span becomes vertical and rear string stays local+Z. Rx(-90°) puts string on local -Z and reverses the intended fore/aft. This rotation is a Kiva axis adaptation, not recovered source actor assembly.

Quiver/arrowbag roots are separate original standalone prefab scenes with identity root transforms. Their large source-Z extents cannot be treated as already assembled locations on the bow/character. Attach each with explicit independent Kiva offsets/scale and declare that adaptation; original assembly offset was not recovered. Related arrow remains unbound neutral geometry, not a fourth configured part (`yizhiying/README.md:9`).

Faithful static bake: evaluate original hierarchy and every source skin slot as `Glb.posed`; apply the chosen uniform rotation/scale/translation to that posed result; transform and normalize the actual normals through the source pose then the adapter (all current weapon bone scales are approximately1). Preserve source UVs/indices/material selection. Store original GLB/skin metadata hashes and make a numeric equality check for baked vertices against transformed source LBS, including the135 gun joint1 vertices and20 string-center-weighted bow vertices. `scripts/import-naraka.py:400` shows the current posed-normal helper; `:560` uses the same full-pose bake before making Li's rigid blade. A static render mesh is an explicit current-pose bake, not a claim that the original weapon lacked bones or dynamic string behavior.

Existing public display convention is ranged forward local -Z, muzzle/arrow source local origin (`src/client/weapons.rs:84`, `.trellis/spec/client/combat-presentation.md:23`). Companion sync places that origin at interpolated simulation source_position and rotates -Z toward aim (`src/client/companion.rs:188`). Bone-attached player weapons use the retarget wrist and ranged Ry(π) frame (`src/client/stickman.rs:22`). Baking a different source origin without compensating the visual root breaks event/muzzle equality. Keep geometry adapter, local source marker, and hand anchor mathematically consistent; do not move simulation source_position to fit a model.

### Related specs and checks

- `.trellis/spec/client/index.md:46`: AV03 rest, original LBS, own character bone lengths, shared retargeted hand, and no guessed packed channels.
- `.trellis/spec/client/index.md:29`, `:56`, `:61`: display only; no physical/camera scale changes; independent female rig.
- `.trellis/spec/client/combat-presentation.md:21`, `:23`, `:26`, `:41`: companion owns its rig; original source/event alignment; F3 body-only; actor versus source_owner distinction.
- `docs/architecture.md`: client → gameplay → simulation; model assets and animation clocks stay client-owned.
- Task `prd.md` M02/M05/M06 and `design.md`: frozen prepared sources, runtime output in new asset directories, own female proportions, no original face/shader reconstruction.

Existing verification was run with `/home/asuka/kiva-asset-research/naraka/.venv/bin/python` (NumPy2.5.4); system python3 lacks NumPy. This interpreter location is only local tooling availability, not a runtime or portable-validation dependency. Runtime assets must remain reproducible from the repository prepared package with NumPy/Pillow available elsewhere.

## Caveats / Not Found

- No external documentation was needed; all numeric facts above come from repository GLBs and scripts. No internet source, original game directory, browser, game service, or screenshot was used.
- Exact original muzzle/grip semantic markers, original bow/accessory actor assembly, unknown packed-map channel semantics and original dedicated female face/shader parameters were not recovered. Authored anchors/material tints must be recorded as such.
- Cited runtime line numbers reflect the files while concurrent owners were editing; function names identify the stable contracts. This research is source evidence, not a replacement for the exporter's final `model-runtime-contract.md`.
- Offline hashes, LBS and format checks passed. They do not prove final framing, hair clipping, neutral-face appearance, weapon size, or hand placement in rendered gameplay; the main task must record visual validation separately.
