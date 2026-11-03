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
relationships, samples 19 selected clips at 30 fps, discards unused finger bones,
and retains 23 original bones and their local transforms. No retargeting is done.
Root translation is stripped as an extra guard against applying motion twice.
Both gameplay position and damage timing remain controlled by the combat systems.

`stickman.anim` is 578,598 bytes, embedded at compile time for offline play and
identical native/WASM behavior. The binary format is little endian:

- Magic `KIVAAN01`, then `u16` bone count and clip count.
- Each bone: UTF-8 name (`u16` length), `i16` parent index (`-1` for root),
  and ten `f32` values (translation XYZ, rotation XYZW, scale XYZ).
- Each clip: name, `f32` duration, `u16` frame count, then frames containing seven
  `f32` values per bone (translation XYZ, rotation XYZW). Scale is constant.

`skeleton.rs` samples clips with quaternion interpolation and evaluates the parent
hierarchy. The game binds rigid rods and joint spheres to these bone positions;
the head and sword inherit their respective bone rotations. State transitions
blend from the previous pose. This is a rigid skeletal character, without a
deforming skinned mesh or a Bevy `AnimationGraph`.

## Sword choreography

`src/game/animation/attacks.rs` authors model-space sword paths over cropped body
gestures from the free Standard clips. `Sword_Attack`, `Sword_Heavy_Combo`, and
`Sword_Dash` supplement the existing regular attacks. Light/Heavy individual
attacks, Aerial, UpperCut, and GroundPound names visible in the author's viewer
are not all included in the free Standard packages; none of those paid Source
clips are embedded or extracted from the viewer.

The sword paths are authored for this prototype, not reproductions of NARAKA
animation assets:

| Gameplay attack | Choreography |
| --- | --- |
| A / 2A / 3A | Horizontal cut / three alternating cuts / broad finishing sweep |
| B / 2B / 3B | Forward thrust / diagonal cut / overhead heavy cut |
| Dodge A / Dodge B | Advancing sweep / extended pursuit thrust |
| Crouch A | Low horizontal sweep, with planted-foot crouch adjustment |
| Uppercut | Low preparation, full-body rising spin with two ascending hits, then aerial guard |
| Air A / Air B | Aerial crosscut + return cut / raised blade into a two-contact downward plunge |
| Horizontal / vertical focus | Side-loaded heavy sweep / overhead heavy cut |
| Riposte | Forward counterattack followed by a diagonal finishing cut |

Each gesture has separate preparation, contact, follow-through, and guard poses.
All multi-hit paths read the strike windows from `AttackSpec.strikes`: 2A has
three hits, uppercut and riposte have two equal hits, and plunge has an initial
light contact followed by the heavy hit. Damage is budgeted per move and split
by strike shares. Gaps have no damage or clash frames. These hit counts follow
historical references listed in the main README; timings remain prototype tuning.
The aerial left-click performs two opposite cuts, with two 0.08-second strike
windows separated by a 0.08-second return preparation. Each cut deals 6 damage
and has its own per-target hit claim and swing sound. Recovery starts after the
second cut; even a frame crossing both windows retains both hits.

Contact and follow-through use the gameplay strike windows; sword-energy
release begins with the charged pose. Body root translation is still stripped,
grounded gestures keep a supporting foot on the source floor, and gameplay owns
all lunges and jump height. The uppercut uses the turning body gesture from `Sword_Regular_C`, with its
pelvis yaw normalized before blending and a coherent full-rig 360-degree turn
applied afterward. Recovery cancellations into air attacks continue the turn
forward instead of unwinding it. All attacks, focus preparation, idle, and aerial
guard use the right hand only; the left arm retains the source body gesture.
The aerial downward plunge is the exception: two-bone arm IK keeps both hands on
the same sword handle, including recovery and state blends. Grounded crouching
uses separate leg IK.

The animation tests verify finite transforms, single-hand free-arm preservation, the plunge two-hand grip, distinct sword paths,
upward/downward trajectories, and forward-pointing thrusts. To export the actual
sampled poses for external visualization (no window/GPU needed):

```sh
KIVA_ATTACK_PREVIEW=/tmp/kiva-attacks.csv cargo test --locked --lib attack_pose_preview
```

The CSV contains attack name, frame index, time in seconds, bone name, and model
XYZ. `sword_tip` uses the same sword socket as the renderer. It includes the low
crouch adjustment for Crouch A; actor jump height and state blending are excluded.
