# Local champion VFX assets

This package uses the original PROJECT: Ashe and base Jarvan IV textures and
selected static particle meshes from patch **16.22.8348910**. Assets are served
from the game origin. The game does not fetch CommunityDragon or Riot at runtime.

`sources.json` records exact source URLs, Riot bundle byte ranges, decoded chunk
hashes, source files, reviewed emitter parameters, import recipes and output
hashes. `catalog.json` is the generated runtime resource catalog. It includes
explicit blend modes, uniform sprite grids, source image direction and real mesh
bounds/axis conversions. `parameters.json` retains reviewed source emitter
fragments as provenance; it is not a League particle interpreter.

From the repository root:

```sh
# Verify the complete checked-in package without network or optional tools.
python3 -B scripts/import-league-vfx.py

# Rebuild from verified downloaded PNG/SCB/BIN originals.
python3 -B scripts/import-league-vfx.py --source-dir /path/to/originals

# Fetch pinned sources and rebuild. Optional importer-only dependency: pyzstd.
python3 -B scripts/import-league-vfx.py --download --cache /tmp/kiva-vfx-sources
```

The source PNG container SHA-256 and exact decoded RGBA pixel SHA-256 are both
recorded. CommunityDragon's Cloudflare Polish can losslessly re-encode PNGs even
at a fixed patch URL: the verified Q sample changed from 51,822 to 45,043 bytes
with **zero pixel differences**. The importer rejects changed pixel fingerprints
and writes a deterministic RGBA8/filter-0/zlib-9 PNG. Polish also clears unused
RGB beneath alpha-zero pixels: the importer explicitly normalizes those three
unused channels to zero before the fingerprint and output. Every texel with
nonzero alpha and every alpha value remain exact; no resize or recolor occurs.
The original full RGBA fingerprint is retained too. The zlib encoder version is
recorded in the manifest. SCB/BIN
and bundle ranges retain strict byte hashes. All expected outputs are checked
before any product file is replaced.

Meshes are expanded per triangle to preserve face UV seams, face RGB and source
vertex BGRA/alpha. They become ordinary glTF 2.0 meshes with POSITION, NORMAL,
TEXCOORD_0 and COLOR_0. The runtime loads their Mesh0/Primitive0 subassets and
assigns the shared reviewed material. No League binary parser is in the game.

The original standard spear is the explicit exception for legacy face RGB:
all 3,690 face corners carry zero VCP RGB, while vertex RGBA is white and the
selected E particle uses a colored hero atlas. Its reviewed recipe ignores only
this all-zero legacy face-lighting channel, preserving source vertex alpha and
UVs. Other meshes keep their colors. The importer rejects a changed nonzero VCP
source instead of silently discarding newly authored color.

One hundred source League distance units become one world unit. The crystal
arrow and trail rotate source +Y forward to game -Z; the planted standard uses
the source emitter's X half-turn. Other meshes retain Y-up. Actual imported and
original bounds are available through `mesh_spec`. Renderers fit these visuals
to actual simulation carrier/object geometry rather than treating source scale
or source particle lifetime as a rule.

Q symbols use a **2×1** sheet and rocks a **2×2** sheet; their material templates
select frame 0. Nonuniform drone/body/arrow mesh atlases preserve their original
UVs and are not sliced into sprite grids. Original black-backed indicator images
use additive blending. Source color, grid and blend come from `skill_assets`;
callers must not change shared material templates for individual instances.

This is a bounded visual adaptation using original artwork and selected source
parameters. It omits League's complete probability, distortion, flowmap and child
emitter systems. Simulation remains the only owner of casts, hits, projectiles,
shields, terrain, visibility and lifetime. The assets do not add picking geometry.

Primary provenance:

- [Fixed CommunityDragon textures](https://raw.communitydragon.org/16.22/game/)
- [Official Riot release manifest](https://lol.secure.dyn.riotcdn.net/channels/public/releases/6EDAD6B3C9806CB9.manifest)
- [SCB binary reference implementation](https://raw.githubusercontent.com/LeagueToolkit/LeagueToolkit/main/src/LeagueToolkit/Core/Mesh/StaticMesh.cs)
- [SCB face/UV reference implementation](https://raw.githubusercontent.com/LeagueToolkit/LeagueToolkit/main/src/LeagueToolkit/Core/Mesh/StaticMeshFace.cs)

Riot Games owns the original game artwork. This repository records how the local
runtime artifacts were obtained and prepared; it does not infer asset rights
from their public download availability.
