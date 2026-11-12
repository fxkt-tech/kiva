# Stickman animation sources

Author: **Quaternius**, with animation contributions credited by the author to
**Gonzalo Furnier**. The selected free Standard assets are **CC0 1.0**; the original
license is preserved byte-for-byte in [LICENSE.txt](LICENSE.txt).

- [Universal Animation Library](https://quaternius.itch.io/universal-animation-library),
  official Standard download, v3.0 (June 16, 2026).
- [Universal Animation Library 2](https://quaternius.itch.io/universal-animation-library-2),
  official Standard download, v2.2 (September 28, 2026).
- [Author's animation preview](https://quaternius.com/animviewer.html).

The assets were downloaded November 3, 2026 for this repository's `act` branch.
This port preserves `stickman.anim` and the license from commit
`bd3e3443cfb2922c5cbf6a682cb75bb1091fea12` without modifying their bytes.
The binary is **578,598 bytes**, SHA-256
`b2116f1ee9dc527dfd776753d07cf97674fadc4fc61567699f60d46bd800e918`.

Both source archives contain `Unreal-Godot/UAL*_Standard.glb` and a separate
`_RM` variant. We use the versions **without root motion**. No paid Source files,
Blender, or third-party conversion service are required.

## Rebuild

Download the two free Standard ZIPs from the official pages, then run:

```sh
python3 scripts/import-animations.py /path/to/UAL1-Standard.zip /path/to/UAL2-Standard.zip
```

The standard-library-only importer checks matching rest transforms and parent
relationships, samples 19 selected clips at 30 fps, discards unused finger bones,
and retains 23 original bones and their local transforms. It does not retarget.
Root translation is stripped so character movement is not applied twice.

## Runtime integration

`src/client/skeleton.rs` embeds the asset at compile time, decodes little-endian
values explicitly, interpolates local transforms, and evaluates the parent
hierarchy. Its model basis rotates the source facing direction by π around Y
and applies a uniform scale of 1.1, preserving the original `act` implementation.

`src/client/animation.rs` selects and blends body clips from authoritative action
and combat state. `src/client/stickman.rs` binds reusable rigid rods, joints, the
head and weapon socket to those evaluated bone poses. This is a rigid skeletal
character; it does not use skin weights or a Bevy `AnimationGraph`.

The source includes idle, walk, jog, jump, roll, block, hit, death and sword-body
gestures. The current paired weapons reuse suitable body gestures; the asset
does not claim dedicated authored hammer, axe, sniper or cannon animations.
Position, jump height, collision, damage and linked-skill timing remain owned by
`src/simulation/`. Imported clips only drive the displayed body and weapon pose.

## Binary format

- Magic `KIVAAN01`, then `u16` bone count and clip count.
- Each bone: UTF-8 name (`u16` length), `i16` parent index (`-1` for root),
  and ten `f32` values: translation XYZ, rotation XYZW, scale XYZ.
- Each clip: name, `f32` duration, `u16` frame count, then frames containing
  seven `f32` values per bone: translation XYZ, rotation XYZW. Scale is constant.

The runtime reads only this trusted bundled asset. Native and WASM use the same
data and decoder. Headless tests cover all 19 clips, all 23 bone bindings, finite
poses, stripped root motion, looping, one-shot endpoints and transition blending.
