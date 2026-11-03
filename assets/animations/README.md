# Stickman animation sources

Author: **Quaternius**, with animation contributions credited by the author to
Gonzalo Furnier. These selected Standard assets are **CC0 1.0**; the original
license is preserved in `LICENSE.txt`.

- [Universal Animation Library](https://quaternius.itch.io/universal-animation-library),
  official Standard download, v3.0 (June 16, 2026).
- [Universal Animation Library 2](https://quaternius.itch.io/universal-animation-library-2),
  official Standard download, v2.2 (September 28, 2026).
- [Author's animation preview](https://quaternius.com/animviewer.html).

Downloaded November 3, 2026. Both archives contain `Unreal-Godot/UAL*_Standard.glb`
and a separate `_RM` variant. We use the versions **without root motion**.
No paid Source files, Blender, or third-party conversion service are needed.

## Rebuild

Download the two free Standard ZIPs from the official pages, then run:

```sh
python3 scripts/import-animations.py /path/to/UAL1-Standard.zip /path/to/UAL2-Standard.zip
```

The standard-library-only importer validates matching rest transforms and parent
relationships, samples 16 selected clips at 30 fps, discards unused finger bones,
and retains 23 original bones and their local transforms. No retargeting is done.
Root translation is stripped as an extra guard against applying motion twice.
Both gameplay position and damage timing remain controlled by the combat systems.

`stickman.anim` is 431,703 bytes, embedded at compile time for offline play and
identical native/WASM behavior. The binary format is little endian:

- Magic `KIVAAN01`, then `u16` bone count and clip count.
- Each bone: UTF-8 name (`u16` length), `i16` parent index (`-1` for root),
  and ten `f32` values (translation XYZ, rotation XYZW, scale XYZ).
- Each clip: name, `f32` duration, `u16` frame count, then frames containing seven
  `f32` values per bone (translation XYZ, rotation XYZW). Scale is constant.

`skeleton.rs` samples clips with quaternion interpolation and evaluates the parent
hierarchy. The game binds rigid rods and joint spheres to these bone positions;
the head and sword inherit their respective bone rotations. State transitions
blend from the previous pose, while attacks map source clip phases onto the
existing windup, damage and recovery windows. This is a rigid skeletal character,
without a deforming skinned mesh or a Bevy `AnimationGraph`.
