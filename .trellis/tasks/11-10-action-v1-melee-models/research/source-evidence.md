# Research: Melee source identity, complementary geometry and static adaptation

- Query: Can frozen Fulangmao 2310478 and ZhexingCuishan 2311638 provide complete deterministic melee skins, especially a defensible Zhexing blade assembly without guessing numeric-texture/shader behavior?
- Scope: internal; read-only inspection of frozen prepared metadata, records, GLBs and existing conversion code. No game installation or web sources used.
- Date: 2026-11-10

## Findings

### Decision supported by source evidence

Use Fulangmao's two original skinned draws. For Zhexing use the configured main draw plus all four **complementary non-FX regions** in one explicitly authored Kiva static assembly; omit both independent FX draws. The prepared role names `source_variant_input_01..04` are preservation labels, not proof that four complete weapon meshes are mutually exclusive alternatives.

Source configuration proves the regions belong together but does **not** provide a shader-free assembled current/default star shape. Keep original vertices, UVs and bound base textures, bake real main-palette LBS, and describe any region rotation/translation/scale and final +Y grip conversion as Kiva adaptation. Exact chosen matrices and dimensions belong in `research/runtime-contract.md` and the runtime conversion report. Do not claim a recovered original/default/random star result. Root approved this authored assembly boundary during implementation.

### Files found

All paths below are relative to `assets/prepared/naraka/` unless otherwise stated.

- `fulangmao/{README.md,portable_manifest.json}` — frozen package scope, all local file hashes and two-draw inventory.
- `fulangmao/source-evidence/{identity_manifest.json,target_parts.json,records/*.bin}` — original skin → battle skin → WeaponTemp identity chain.
- `fulangmao/source-evidence/additional/fulangmao_main.source.json` — two original renderers, original palette order, source paths and encoding audit.
- `fulangmao/source-evidence/additional/main_3583192603019751291.single-indices.u32.bin` — exact 448 original single indices, not invented four-weight data.
- `fulangmao/source-evidence/model-checks/fulangmao_main.cpu-verification.json` — original Unity LBS/current pose and per-palette perturbation comparisons.
- `zhexingcuishan/{README.md,portable_manifest.json}` — seven isolated source inputs; preparation deliberately makes no default-shape claim.
- `zhexingcuishan/source-evidence/{identity_manifest.json,target_parts.json,records/*.bin}` — exact identity, RareSkin1022, shape/color/effect record links.
- `zhexingcuishan/source-evidence/additional/initial_probe.json` — original hierarchy, 26 LOD renderers, assistant mappings, original materials and serialized prefab state.
- `zhexingcuishan/source-evidence/additional/controller_source.json` — material groups, renderer assistants, presentation sync bone and dissolution renderer sets.
- `zhexingcuishan/source-evidence/additional/rule_source.{json,raw}` — exact rule tree/bytes; 20 shape branches, shader application groups and head/bounds controls.
- `zhexingcuishan/source-evidence/additional/*.layout.json`, `*.gpu.bin`, `*.npy` — original stream layouts and numeric texture precision, with no asserted vertex-shader interpretation.
- `zhexingcuishan/models/*.glb` — one 2-slot skin plus six genuinely static inputs; source hierarchy retained independently in every GLB.
- Both `source-evidence/weapon-export.original.json` — original material slot bindings, source object hashes and export provenance.

### Original identities and hash anchors

Original source serialization reports Unity **2019.4.41f2**. The original weapon classes are source art identities: Fulangmao is recorded as 阔刀 and Zhexing as 斩马刀; using them for Kiva hammer/chainaxe is the user's explicit appearance mapping, not a simulation weapon-class change.

| Package | Skin record → battle part | Original battle prefab |
| --- | --- | --- |
| Fulangmao | 2310478 → 3112349 | `assets/res/prefab/weapon_prefab/blade/wp_blade_chuanmaolv1.prefab` |
| ZhexingCuishan | 2311638 → 3112012 | `assets/res/prefab/weapon_prefab/saber/wp_saber_xingtian.prefab` |

SHA256, computed from local frozen files:

| Relative file | SHA256 |
| --- | --- |
| `fulangmao/portable_manifest.json` | `71bf61d35756e4b655e156d952756640c8e8476bc42315c1fdb4c128d1a57b80` |
| `fulangmao/source-evidence/identity_manifest.json` | `497fdcc2c979a1cc97382bcbc6334abe62391893bb1598682bfa7cd237e99705` |
| `fulangmao/source-evidence/weapon-export.original.json` | `57a05fbe51d5886ec6ec548135dd481ae6b33f72ac7bef5194b18aacbc7419c0` |
| `fulangmao/source-evidence/records/WeaponSkin_WeaponSkin_2310478.bin` | `4563180508a8d61246ca4035aa5e6fffe2e03d7ba63e85228ed8b0bf4bc11e81` |
| `fulangmao/source-evidence/records/WeaponSkin_WeaponBattleSkin_2310478.bin` | `122100f329f6f4cef4611e821f7ebad00723c6d342b6eee21cfc0fb22f29ac63` |
| `fulangmao/source-evidence/records/WeaponConfig_WeaponTemp_3112349.bin` | `03f37ff7b915f30fc7ade8519259d3085adfa06e189c2546a8916838f0c2e9e5` |
| `zhexingcuishan/portable_manifest.json` | `81891e9d41c1fb9333c510c398c022e0389fae1c73c7c480385c9447545fdd7e` |
| `zhexingcuishan/source-evidence/identity_manifest.json` | `b9f18bf2494e02c231949fea9ad3b3f38a43c007ca72d359c52899e740e554c9` |
| `zhexingcuishan/source-evidence/weapon-export.original.json` | `dc5feb19e202c864ce4be25e2c406ad48e2ed1ffaec6c249c64c23d672708fdb` |
| `zhexingcuishan/source-evidence/records/WeaponSkin_WeaponSkin_2311638.bin` | `96aeff1f5a813ef7a0f98dcf7056b7a95d1fb9a3f2366557895a85762ae880cd` |
| `zhexingcuishan/source-evidence/records/WeaponSkin_WeaponBattleSkin_2311638.bin` | `c656c02b96f8f62e35f72c7eff82c31274ce5b71bf478a582cc537aa19ad06cb` |
| `zhexingcuishan/source-evidence/records/WeaponConfig_WeaponTemp_3112012.bin` | `afe38e3f1179f8c9c7882f922013c3df3bfd83ab08b6e2fc78e2067945c6458c` |
| `zhexingcuishan/source-evidence/additional/controller_source.json` | `47c8c299f00b165057e0deb4be5af9aaa48bd4d4c574d77caaf34da3b535d690` |
| `zhexingcuishan/source-evidence/additional/rule_source.json` | `31e6e3318e756e49aecd398476e83b09b1a35c5121d5abd34a80d4131753c5fb` |
| `zhexingcuishan/source-evidence/additional/rule_source.raw` | `14c261be0851b337b86eba6bd29b10d29392e17d7617b9cb56a1caaefa91bc6b` |
| `zhexingcuishan/source-evidence/additional/initial_probe.json` | `1679b424d484afb8d6cc8b7fe3c13dcdbbf89459a068ff7e763fd0bac8bb5d6d` |

### Fulangmao: current LBS and the 448-vertex addendum

`fulangmao_main.glb` SHA256 `cf70d33ee341db0063f30dbda955b264f1320298a2267233b5bfa744244c77a9` contains 17 nodes, two meshes/two skins, 5,437 vertices and 7,838 triangles:

- `wp_blade_chuanmaolv1`: 4,989 vertices/7,252 triangles, genuine original explicit four-weight/four-index streams; original skin node order `[3,6,5,4,7]`. Source paths are `MotionRoot/chuanmao_01`, then its descendant `.../chuanmao_04`, `.../chuanmao_03`, `.../chuanmao_02`, and `.../chuanmao_06`, in that order.
- `wp_blade_chuanmaolv1_01`: 448 vertices/586 triangles, genuine original 1-slot skin `[3]`, source `MotionRoot/chuanmao_01`. Original channel13 is UInt32/dimension1, stream2/offset21504/stride4; channel12 is absent. All 448 original indices are 0. Their source byte SHA256 is `7b4fc1a8dbe6b6121f16ada516e2ac27e02964716bacea5fb7d07cf30595948e`.
- Prepared glTF expands only that second stream to `JOINTS_0=(original_index,0,0,0)` and `WEIGHTS_0=(1,0,0,0)`. This is an explicit representation of implicit weight1, not original four-weight input and not a missing-index fallback.
- Frozen CPU audit compares `sum(weight * originalBoneWorld * originalInverseBind * originalVertex)` with GLB at actual imported pose and perturbed bones: max current error `1.4128917125155738e-7`, max perturbed error `1.3198985793216877e-7`; all original palette perturbations pass. Do not replace current source transforms/IBMs by identity-bind cancellation.

### Zhexing: controller → regions → source meshes/materials

Evidence chain: `controller_source.json:30` onward supplies the five group indices and assistant pointers. `rule_source.json:71294`, `75102`, `78910`, `82718`, `86526` names the matching shader groups 刀刃/刀背/人头/刀柄/特效. `initial_probe.json` component entries resolve each assistant's `_renderer`, `gameObjectName` and `lodLvl=0`; its renderer entries resolve original material pointers. `weapon-export.original.json` and original mesh-layout records give exact input mesh IDs and counts.

| Preserved role | Original object | Controller group / source meaning | LOD0 assistant | Original mesh path ID | Original material path ID | Vertices / triangles |
| --- | --- | --- | --- | --- | --- | --- |
| main | `wp_saber_xingtian` | 3 / 刀柄 | 5563197552821685458 | 1013643324579487053 | -6666812944221523090 | 1606 / 2008 |
| variant01 | `wp_saber_xingtian_01` | 1 / 刀背 | -979659002343826867 | -6201183846497327796 | 6959952434485530562 | 3776 / 5054 |
| variant02 | `wp_saber_xingtian_02` | 0 / 刀刃 | 190149220287945186 | 7996638505543494287 | -4220766929683576160 | 3300 / 4544 |
| variant03 | `wp_saber_xingtian_03` | 2 / 人头 | 8572311883007519452 | -7349815062883026250 | -4866712198032441692 | 2950 / 3630 |
| variant04 | `wp_saber_xingtian_04` | 3 / 刀柄 | -3441879327291996011 | -596706628310788450 | 563734185129301229 | 5872 / 7384 |
| FX01 | `wp_saber_xingtian_fx_01` | 4 / 特效 | -3341958396009704472 | 9111225317028525459 | 6063882881538256101 | 1488 / 2072 |
| FX02 | `wp_saber_xingtian_fx_02` | 4 / 特效 | -8221251434265135903 | 4648164346310742518 | -4464184604443950896 | 1264 / 1800 |

Main has native two-slot palette `[node4,node3]`; its preserved current-LBS audit error is exactly zero. The other six inputs are static and must stay static. The original helper's `isFxRenderer=0` appears even on FX named assistants, so explicit controller group and original shader/material bindings are the stronger FX classification evidence.

The second original controller (`controller_source.json:311`) serializes presentationMode1, transition0.5s, sync bone transform `-5866840785647173652`, origin `(-1.200003794e-7,-1.968000470e-5,0.569568992)`, 22 sync assistants and four main-LOD dissolve assistants. Sync/transition/dissolve behavior was not executed by preparation and is not a static attachment formula.

### Zhexing: why direct source-space overlay is not complete assembly

The original hierarchy retains `main` local rotation mapping local Y to world -Z; main mesh accessor bounds are `(-0.050283082,-1.005510092,-0.084953852)` to `(0.050283082,0.855923235,0.084953852)`, showing a long narrow shaft. This accessor is pre-skin local space; exporter must use actual current LBS before choosing final dimensions.

All four static regions have root parent, identity local/world matrices and zero translation in `initial_probe.json` (`hierarchy` indices10,14,18,22; lines675,939,1203,1467). Their GLB accessor bounds are:

| Region | Minimum XYZ | Maximum XYZ |
| --- | --- | --- |
| 01 back | (-0.032626223,-0.153618112,-0.125300184) | (0.032626223,0.153618112,0.125300184) |
| 02 blade | (-0.022104714,-0.281369716,-0.097328156) | (0.022104714,0.272197843,0.097328171) |
| 03 head | (-0.120056868,-0.147213474,-0.121360466) | (0.120056875,0.147213593,0.121360466) |
| 04 handle decor | (-0.089922987,-0.096667796,-0.089922994) | (0.089922987,0.096667781,0.089922987) |

Thus source-space overlay leaves all four small region shapes clustered around origin and cannot be called the complete source current shape. `enabled=true` on all original renderers includes all LOD levels and is not proof of visible final composition. Source control uses the same four regional meshes with shape mutation, not a simple choose-one renderer switch.

### Shape rules and original numeric texture limits

`rule_source.json:65241` (`ruleGroups[6]`) contains 20 shape branches. `.ruleDatas[0..4]` are shape index, head position, bounds minimum, bounds size, head rotation. Shape/head/bounds rules have `ruleIndexRange={x:3,y:5}`. First branch (`subRuleStartNumber=0`, end99) gives shape scalar8→9, head `(-0.007568359375,0.024398803711,1.656005859375)`, bounds min `(-0.212585449219,-0.655746459961,0)`, size `(0.436279296875,0.970764160156,2.107177734375)`. These are source rule values; neither this branch nor any other is identified as the item's default.

Each shader group ends with bindings `(rule6,0)→_HeadPosition propertyTarget3`, `(6,1)→_HeadPosition target7`, `(6,2)→_BoundsMin target7`, `(0,0)→_BoundsMin target3`, `(6,3)→_BoundsSize target7`, `(0,0)→_BoundsSize target3`. Numeric property-target enums and interpolation5 are preserved but their executable meaning is not recovered here. Applying head/bounds values as arbitrary object translations/scales would be inference, not recovered shader behavior.

Original non-FX material scalars in `weapon-export.original.json/materials[*].tree.m_SavedProperties`:

| Object | `_SHAPE_MUTATE` | `_EnableMutate` | `_BlendShapeParam` RGBA |
| --- | --- | --- | --- |
| main | 0 | 0 | (0.090187646449,0,256,4537) |
| 01 | 1 | 0 | (0.090139120817,0,256,3776) |
| 02 | 1 | 0 | (0.219864994287,0,256,3300) |
| 03 | 1 | 0 | (0,0,256,2950) |
| 04 | 1 | 0 | (0.047660000622,0,256,5872) |

All have `_RelativeMutate=1`. `_EnableMutate=0` is serialized material state, not evidence that shape mutation is disabled or that raw geometry is assembled. Region03 **really shares** region02's `_DeltaPositionTexture` and `_DeltaNormalTexture`; file-name guesses would produce the wrong relation.

The exact `_DeltaPositionTexture`01 layout is little-endian f16 `(140,256,4)` with original GPU row order/no vertical flip; finite fraction1, min0.0716553/max1. The shared `_ShapeTexture` is original format20, f32 `(40,174,4)`, finite fraction0.88979885, finite extrema approximately -1.75e38/+1.71e38. Its packing/bit interpretation is not established; values cannot safely be read as ordinary geometric displacements. Layouts explicitly say no vertex shader/shape lookup interpretation. Display-only PNGs are clamped previews and cannot replace raw/npy inputs. Any later demonstrated decode must preserve raw rows and independently match source vertices; otherwise use the documented authored assembly.

### Actual base textures and material limitations

Fulangmao source material `wp_blade_chuanmaolv1` binds `_MainTex=wp_blade_chuanmaolv1_d.png` SHA `1caee8ebec51d061f19fefe1eba45a366fc55b1169d24e4e74472a733e560f63`; its 448-vertex addendum binds `wp_blade_chuanmaolv1_01_d.png` SHA `911d70de0f0f9cfec9646b70df2c85e6d0c10f61286f5259b208f312c9853c77`. Both base factors are white. Additional normal, spec/gloss, emissive and transmission bindings exist; those names alone do not prove Bevy channel mappings.

All five non-FX Zhexing materials bind exactly the same `_MainTex=wp_saber_xingtian_d.png` SHA `bcf29e7a1b2957e7bb77e1643f57cbb4b9bd6cf3e3d46f00fa254050ac3ba88a`, but distinct source UVs/material slots. Both FX materials bind `_MainTexture` atlas plus mask, delta textures and a source Cubemap through the original misspelled slot `_CsutomReflTexture`; there is no ordinary source `_MainTex` base-color claim for those FX.

Prepared GLB PBR factors are neutral previews. Runtime sRGB base PNG + scalar PBR is an explicit approximation, not recovered thick liquid, refraction, iridescence, regional mutation, random star color, emitted glow, original FX reflection or dissolution. Unknown packed/ID/normal/SH data must not be guessed into color/ORM. Reusing one decoded base image across Zhexing's five distinct materials is valid caching, not a claim that material responses are identical.

### Existing code patterns and related specs

- `scripts/prepare-naraka-weapons.py:26` records Fulangmao implicit-weight expansion; `:28` distinguishes Zhexing source inputs and unrecovered star behavior; `:34` checks exact configured identity/part IDs; `:95` preserves original texture bindings and flags display-only numeric PNGs at `:106`.
- `scripts/import-prepared-models.py:93` preserves source material roles; `:101` loads baseColorTexture; `:109` explicitly calls scalars neutral prepared PBR. This is the reusable material boundary, not evidence for other source shader channels.
- `fulangmao_main.source.json/coordinates` records only standard source X reflection and UV V reversal in the prepared interchange. Final +Y grip, displayed dimensions, wrist mount, and held/flying rigid bake are new Kiva adaptations. Do not describe the original as unskinned because runtime uses a static bake.
- `.trellis/spec/client/model-assets.md` — current-LBS before rigid weapon bake, bounded AV03/KIVAWP01 reads, frozen source files, explicit material/coordinate adaptation.
- `.trellis/spec/client/combat-presentation.md` — held/flying shared assets, hide held chainaxe while away, snapshot tether, simulation authority and F3 isolation.
- `.trellis/tasks/11-10-action-v1-melee-models/{design.md,implement.md}` — authored static shape boundary and separate exporter/runtime ownership.

## Caveats / Not Found

- No source shader implementation, enum definitions, executable shape lookup formula, original default star seed, or proven complete shader-evaluated pose was found in this portable package. No exact random-star/default-shape restoration is claimed.
- The present research recommendation is source-supported **region selection** plus explicitly authored Kiva **assembly**. Numeric UV/texture correlation, if exporter establishes it, must be separately evidenced before changing that claim.
- Source `HeadPosition`/bounds are rule-controlled shader parameters; they are not independently proven object-space attachment transforms. Presentation sync origin is also not a source grip instruction.
- Browser/game/screenshot visual verification was not performed by this research. Original source remains frozen. This role wrote only this research file and did not inspect role-isolated implement/check manifests, modify code/assets/specs, use Git, or manage services.
