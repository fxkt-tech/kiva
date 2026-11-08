# League model and terrain asset research

Research date: 2026-11-08. Research agent: `research_league_assets`.
Scope: actual downloadable assets and Bevy 0.20.1 integration constraints. No product code changed during research.

## Verified acquisition

Khada's public model API resolves the requested models without a local League install:

- <https://modelviewer.lol/api/skins?id=22008>: `PROJECT: Ashe`, alias `ashe`, source `lol`.
- <https://modelviewer.lol/api/skins?id=59000>: `Jarvan IV`, alias `jarvaniv`, source `lol`.
- <https://modelviewer.lol/api/search?query=minion>: blue/order and red/chaos melee/ranged units under source `extras`.

The **uncompressed `model.glb` endpoint exists**. Do not use the `model-compressed.wasm` endpoint for Bevy; despite its extension it is a GLB with compressed geometry. No conversion library is needed for the uncompressed asset.

Downloaded originals are in `/tmp/kiva-model-research-vhw4bdaf/`. Each GLB was parsed as glTF 2.0 with JSON and embedded BIN chunks. All contain embedded images, a skin, mesh geometry and animation channels; none requires a mesh-compression extension. `KHR_materials_unlit` is the only used extension.

| Local file | Source | Bytes | SHA-256 |
| --- | --- | ---: | --- |
| `ashe-22008-model.glb` | <https://cdn.modelviewer.lol/lol/models/ashe/22008/model.glb> | 4,131,740 | `b2502086ebcad8476add2134e2d738db8ab17129280de66b0bb1b68c228562f1` |
| `jarvaniv-59000-model.glb` | <https://cdn.modelviewer.lol/lol/models/jarvaniv/59000/model.glb> | 6,501,724 | `605d25ddc47aec08be4dadd350f7bf2f4e5bfb3ed9697a0ea5e30e359ce18802` |
| `ha_orderminionmelee.glb` | <https://cdn.modelviewer.lol/extras/models/ha_orderminionmelee/0/model.glb> | 808,596 | `00a848ac5ffad3caebac835525809fe3c4ce19355d0a1e2e1ef4c437284542b2` |
| `ha_chaosminionmelee.glb` | <https://cdn.modelviewer.lol/extras/models/ha_chaosminionmelee/0/model.glb> | 703,584 | `983f30868271915c7022d47645269928e6e320d695cd29e2192d4e2d73f412ac` |
| `ha_orderminionranged.glb` | <https://cdn.modelviewer.lol/extras/models/ha_orderminionranged/0/model.glb> | 639,948 | `7766e30f0bb80bb7ce6195d74cea00567af9551ec64b1915d736e0f4a23f756d` |
| `ha_chaosminionranged.glb` | <https://cdn.modelviewer.lol/extras/models/ha_chaosminionranged/0/model.glb> | 606,628 | `88aeb18385678b1838d6c6aaccc721321d467ad55f37d63a6396263fc154f379` |

The HA aliases are the publicly listed original blue/red minion models. They do not change simulation unit roles, team semantics or map rules.

## Animation inventory

### PROJECT: Ashe

85 nodes, 83 skin joints, 1 mesh with 4 primitives, 4 materials, 2 embedded images, 30 clips.

Suggested exact clip mapping:

- idle: `Ashe_Skin08_Idle1.anm` (5 seconds).
- run: `Run_Normal` (1 second); alternatives `Run2`, `Run3`.
- attack: `Ashe_Skin08_Attack1.anm` (2 seconds); alternatives Attack2/3.
- Q: `Ashe_Skin08_Spell1.anm`; `Spell1_In` is a 0.133 second transition and `Ashe_Skin08_Spell1_2.anm` is an alternative.
- W: `Ashe_Skin08_Spell2.anm` (1.167 seconds).
- E: `Spell3` (1.667 seconds).
- R: `Ashe_Skin08_Spell4.anm` / `Spell4_Base` (1.5 seconds).
- death: `Death` (3.333 seconds).
- respawn: `Respawn` (2.667 seconds).

Materials `Head_MAT` and `Drone_MAT` have `extras.visible=false`. `Body_MAT` and `Head2_MAT` are visible. These flags are Khada material metadata, not standard glTF visibility; a plain Bevy import would otherwise render both heads and the normally hidden drone. Remove hidden primitives in the offline asset preparation step.

### Jarvan IV base

64 nodes, 62 skin joints, 1 mesh/primitive/material/image, 24 clips.

- idle: `Idle1` (10.167 seconds).
- run: `Run` (1 second); `Run3` is a 0.645 second alternative.
- attack: `Attack1` (1.333 seconds), Attack2/3 and Crit available.
- Q/W/E/R: `Spell1` (1.733), `Spell2` (1.467), `Spell3` (0.984), `Spell4` (2.0).
- death: `Death` (6.2 seconds).
- no dedicated respawn clip: transition directly to idle after the authoritative respawn.

### Minions

All four contain `Idle1`, `Run`, `Attack1`, `Death`; melee models have additional attack variations. Blue melee has 43 joints/19 clips, red melee 43 joints/18 clips, blue ranged 43 joints/10 clips, red ranged 47 joints/11 clips. Geometry is approximately 755–925 vertices per minion, so their mesh complexity is small.

### Coordinate and timing constraints

Each source scene has `Meshes` and `Skeleton` roots with scale `[-0.0085, 0.0085, 0.0085]`. Retain this authored conversion. The outer `UnitVisual` remains the sole world-position/heading/airborne owner. Do not add animation root motion to simulation coordinates.

Clips are source durations, **not** attack/skill rules. Sample them using the existing `PresentationFrame`/simulation tick and authoritative windup/backswing/cast durations. Pause must preserve displayed transforms. Death should hold its final pose until the actual respawn; Jarvan's 6.2 second source death is longer than the current 3 second respawn delay, so fit the visible death transition to the available simulated lifetime rather than extending respawn to finish a clip.

## Picking and skin bounds

Bevy 0.20.1 already provides dynamic skin bounds:

- `bevy_gltf::GltfSkinnedMeshBoundsPolicy::Dynamic` generates `Mesh::skinned_mesh_bounds()` and adds `DynamicSkinnedMeshBounds`.
- `bevy_mesh::skinning::SkinnedMeshBounds` contains per-joint `JointAabb` values and their joint indices.
- The render transform for each such model-space joint bound is `joint_global_transform.affine() * inverse_bindpose[joint_index]`.

For the existing part-based picking contract, project each joint bound separately using these actual last-displayed joint transforms. Using one whole skinned mesh AABB would restore the large empty click area around an extended spear or bow. Keep the existing 6 CSS pixel tolerance, life/visibility gates, candidate ordering and label exclusion.

**Important source-data trap:** Ashe's 4 primitives share a 5,999-vertex attribute accessor; each has its own indices. Bevy's `SkinnedMeshBounds::from_mesh` scans vertex attributes, including vertices not referenced by a primitive's indices. Offline preparation should compact/reindex each primitive's attribute arrays to its referenced vertices, in addition to dropping hidden primitives. Otherwise even a visible body mesh's bounds could include the invisible head/drone vertices.

Relevant verified local library source:

- `$CARGO_HOME/registry/src/.../bevy_mesh-0.20.1/src/skinning.rs`: `SkinnedMeshBounds::from_mesh`, `entity_aabb_from_skinned_mesh_bounds`.
- `bevy_gltf-0.20.1/src/loader/mod.rs`: dynamic bounds generation and component insertion.
- `bevy_camera-0.20.1/src/visibility/mod.rs`: `DynamicSkinnedMeshBounds`, post-transform bounds update.

## Terrain texture

Actually downloaded and visually inspected:

- <https://raw.communitydragon.org/16.22/game/assets/maps/kitpieces/summoners_rift/textures/grnd_terrain_a.png>
- Local `/tmp/kiva-model-research-vhw4bdaf/grnd_terrain_a.png`, 1,647,378 bytes.
- The initial `latest` download and explicitly verified `16.22` URL share SHA-256 `158b147a0e734b6b438037c5d7ab05e6dd3226839bda5fa532298894fe2578f0`. The manifest now uses the fixed release URL. Khada model/loading-portrait endpoints have no version archive; hashes detect replacement but cannot guarantee perpetual upstream availability. Runtime uses the checked-in originals.
- It contains original Summoner's Rift painted grass, stone roadway and dark outer-bank areas in a large ground tile; it is not a seamless square grass texture.
- Preserve the actual `MapDefinition` geometry/navigation. Apply selected texture UV regions to its ground and road mesh; do not substitute the full Summoner's Rift layout or paint impassable-looking banks across walkable midlane.

CDragon's Cloudflare returned error 1010 for Python's default/simple UA. An ordinary complete browser UA and referer successfully fetched the public image:

```text
User-Agent: Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36
Accept: image/avif,image/webp,image/apng,image/svg+xml,image/*,*/*;q=0.8
Referer: https://raw.communitydragon.org/
```

## Proposed integration boundary

1. An offline import script downloads pinned URLs, validates source SHA-256, removes hidden primitives and compacts vertex streams. Keep a provenance manifest with source and prepared hashes.
2. Production assets are local under `assets/`; Trunk must copy the asset directory. No runtime CDN dependency.
3. A private client module owns model catalog/handles, clip mappings, async scene attachment and animation sampling. Simulation/gameplay definitions remain unchanged.
4. Original rigid rig remains for rendererless/raw simulation test actors without a production model identity, not as an extra visible skeleton on top of imported heroes.
5. Share displayed root, corpse lifecycle, pause/final-frame semantics, and picking rules with existing systems; model readiness must not spawn duplicate skeletons or leave orphan entities after reselection.

## Primary references

- <https://docs.modelviewer.lol/debugging-model-pipeline>
- <https://docs.modelviewer.lol/skeleton-and-animation>
- <https://modelviewer.lol/>
- Downloader path example: <https://github.com/Xiaochr/LOLCodexPet/blob/main/skills/build-lol-3d-pet/scripts/modelviewer_assets.py> (read as source only; no pet skill invoked).

## Required verification before accepting implementation

Native load/sampling test with actual packaged GLBs, both hero and all minion clip mappings, finite transforms, hidden primitive exclusion, per-joint picking including extended weapons, authoritative corpse/respawn and reset cleanup. Then WASM build and actual WebGPU screenshots for both heroes, lane/minions, corpse and HUD at 1280×800 and 800×600/DPR2. Existing healthbar/picking/reselection regressions remain applicable. Research success is not a claim that this runtime integration has already passed.

## Implemented preparation and measured poses

The importer and local production assets have now been implemented. Source/prepared digests are in `assets/league-models.json` and `scripts/import-league-assets.py` rebuilds the exact pinned result. All 17 packaged model/terrain/icon files passed offline digest verification. Main-agent shared native Clippy and make-check gates passed after the fixture fix; final browser verification belongs to the parent task.

An independent standard-math CPU evaluation of the actual prepared GLB's last animation samples, node TRS hierarchy, inverse bind matrices and vertex joint weights confirmed the hero death endpoints remain visible above the ground:

- PROJECT Ashe Death body Y bounds `[-0.051151, 0.318821]`, visible head `[-0.006503, 0.193340]` before outer profile scale.
- Jarvan IV Death Y bounds `[-0.117515, 0.386062]` before outer profile scale.
- Source minion Death clips sink fully below the ground; minions still follow the existing immediate gameplay cleanup and do not acquire hero corpse behavior.

The same evaluation measured Ashe Idle head top at 1.617m and minion Idle tops at 1.07–1.15m. Final outer scales were confirmed with the main agent: Jarvan 1.1, Ashe 1.2, melee minions 1.0, ranged minions 1.1. These preserve the earlier approximately 2m hero / 1.2m minion framing rather than enlarging minions close to hero size. Final visible orientation still requires the parent's screenshots.
