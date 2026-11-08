# Authored League models

Production heroes use Jarvan IV's original appearance and PROJECT: Ashe.
The four minion assets provide blue/order and red/chaos melee and ranged units.
Models, textures, portraits and ability icons are served locally; the game does
not fetch external model services.

Source assets are Riot Games game content, distributed here through the public
Khada model CDN, CommunityDragon and Riot Data Dragon. The precise source URLs,
source SHA-256 values, prepared SHA-256 values and byte sizes are recorded in
[`assets/league-models.json`](../league-models.json). Models were retrieved on
2026-11-08; icons use Data Dragon 16.22.1. The original texture imagery remains
embedded in each GLB. Source files and prepared files intentionally have distinct
hashes. The terrain source is pinned to CommunityDragon 16.22 (the same retrieved
bytes), rather than its moving `latest` alias.

The Khada model and loading-portrait CDN URLs do not expose version archives.
Their pinned hashes reject an upstream replacement; they do not guarantee the
upstream retains these bytes forever. The checked-in local assets are the
reproducible runtime source and continue to work without those services.

Run from the repository root:

```sh
# Verify every packaged model, texture and icon without network access.
python3 -B scripts/import-league-assets.py

# Rebuild the exact pinned output. Changed upstream content fails verification.
python3 -B scripts/import-league-assets.py --download
```

The importer uses the Python standard library. It removes primitives explicitly
marked invisible by source material metadata (PROJECT: Ashe's alternate head and
drone), compacts each visible primitive to its indexed vertex set, removes unused
binary storage, and normalizes the skeleton root's planar motion. Vertex normals,
UVs, joint weights, inverse bind poses, source clips and embedded textures remain
authored data. No League-specific format reader is shipped in the game.

The indexed-vertex compaction matters to model picking: Bevy's joint bounds use
vertex attributes, including unreferenced vertices. Keeping the source's shared
attribute streams would make invisible alternate parts influence visible bounds.

`client::imported_models` owns the model catalog, asynchronous scene replacement,
clip selection and sampled playback. Imported clips are paused and explicitly
sampled from the simulation presentation clock. They never trigger attacks,
damage, movement, respawn or cooldowns. The procedural models remain the loading
and failure fallback; a complete authored scene replaces only their model
subtrees, preserving ground markers and shield visuals.

Khada's scene uses glTF +Z forward and includes its authored negative-X scale.
The client preserves that scale and rotates the outer model to the game's -Z
forward. Existing `UnitVisual` remains the only world-position and airborne/leap
height owner. Death clips are fitted into a 0.65 second fall and retain their
final pose until the actual simulation respawn.

The terrain image is a painted Summoner's Rift tile, not a second map. Selected
grass and stone UV patches texture the existing square and single lane. Collision,
spawns, pathfinding, camera bounds and minimap geometry remain derived from
`MapDefinition`.
