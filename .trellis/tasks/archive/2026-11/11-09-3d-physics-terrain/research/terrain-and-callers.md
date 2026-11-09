# Research: Terrain geometry and 3D migration callers

- Query: Identify the smallest complete seams for movable boxes, a ramp/platform and a smooth small hill; preserve the courtyard's existing content and verify non-flat movement, targeting and display together.
- Scope: Mixed; current repository evidence plus official Avian 0.8 collider documentation. Proposed geometry below is a design recommendation, not existing behavior.
- Date: 2026-11-09

## Findings — current behavior

### Level and visual identity

| Existing owner | Evidence | Migration implication |
|---|---|---|
| `Level` stores prop descriptions, not prop entities | `src/gameplay/mod.rs:14`, `:20`, `:45` | Keep deterministic level descriptions, but spawn an authoritative physics entity for each box / fixed prop and associate its visual children with that entity. |
| Collision discards prop height | `src/gameplay/mod.rs:71`–`:77` | Replace planar `collision_world()` assembly with full three-dimensional collider assembly. |
| Scene copies descriptors then independently spawns meshes | `src/client/scene.rs:54`–`:56`, `:82`–`:85`, `:182` | This cannot continue for dynamic boxes: rendering must read the current pose of the physics entity, not respawn from initial level coordinates. |
| Box straps use absolute positions | `src/client/scene.rs:198`–`:218` | Attach body mesh and all straps as local-space children of one visual root, so tipping or rotating a box moves its entire appearance. Keep cosmetic straps inside the main cuboid bounds. |
| Global floor and rim are display-only | `src/client/scene.rs:104`–`:149` | Ground / boundary geometry must join physics assembly. The current rim is only 0.7m high; a three-dimensional boundary must not silently retain an infinite-height planar clamp. |
| Spawn pad and floor lines are non-solid decoration | `src/client/scene.rs:114`, `:161` | They may remain cosmetic. Do not add tiny colliders to every painted line / strap and create artificial trip hazards. |

### Position, support and display

- `Body` contains two `Vec2` snapshots and a planar `step_displacement()` (`src/simulation/movement.rs:6`–`:43`). `MotionState` independently stores previous/current height and vertical speed (`src/simulation/actions.rs:125`–`:129`). Consolidate the authoritative pose into the physics result; keep any `Body` facade as read-only snapshots and clearly name planar projections.
- `is_grounded()` means exactly world height zero, nonpositive vertical speed and no leap (`src/simulation/actions.rs:152`). Standing on a box or hill therefore cannot work by changing only collision geometry. Grounding must expose current support contact, normal and supported entity, independent of absolute altitude.
- Ordinary landing clamps to zero and refills jumps (`src/simulation/actions.rs:425`–`:431`); cannon flight assigns a world-zero parabola and forcibly lands at zero (`:321`–`:334`). Both must use actual support / collision. A lost support must result in falling; it must not be interpreted as a new landing or a jump refill.
- Player, enemies and health bars reconstruct `Vec3(x,height,z)` separately (`src/client/scene.rs:89`–`:100`; `src/client/combat.rs:174`–`:195`, `:213`–`:230`). Read a common interpolated 3D actor pose instead. Dynamic box visuals need the same pause-aware interpolation policy for both translation and rotation.
- The camera already follows the displayed player's full `Transform` (`src/client/camera.rs:14`–`:21`, `:40`–`:45`); its projection does not need a new height-dependent formula. Update its input source and preserve its order after actor display sync (`src/client/mod.rs:62`–`:71`). Validate actual ascent / descent and support loss, not just a static camera formula.
- Animation derives gait speed from `Body::step_displacement().length()` (`src/client/animation.rs:381`). On migration, explicitly use planar or support-tangent locomotion speed; free-fall displacement must not produce running animation. Physics owns root position; foot IK should not move the authority.

### A mandatory aim / marker contract

Current input submits a planar point (`src/client/controls.rs:302`, `src/simulation/combat.rs:179`). `Combatant::aim()` returns `Vec2` (`src/simulation/combat.rs:224`, `:277`) and `sync_markers()` places **all target, aim and guard rings at y=0.05** (`src/client/combat.rs:254`–`:280`). Updating actor motion alone would leave rings inside hills or below box tops.

Recommended minimal contract:

1. Input continues to own yaw and requested horizontal distance. It submits an intention, not a client-selected surface or authoritative height.
2. Simulation resolves the request from the actor's actual 3D origin using a bounded surface query. Return an optional `SurfaceHit { entity, position: Vec3, normal: Vec3 }`, with collision layers excluding the requesting actor and combat-only sensors. Validate a walkable normal and capsule clearance at the prospective landing pose.
3. Store the resolved 3D hit in combat state. Use the **same hit** for preview, skill validation and initial launch planning; no independent client terrain-height formula. A miss or an invalid surface must be represented explicitly rather than falling back to y=0.
4. During preview, moving / tipping a box changes its current surface hit. After skill acceptance, commit the selected world-space destination as current cannon tests require (`src/simulation/combat.rs:1046`–`:1087`). New aim must not retarget in flight. If the box subsequently moves away, physical contact determines the real landing; do not teleport or snap the actor to a stale elevated target.
5. Draw rings at `hit.position + hit.normal * small_offset`, orienting their local up axis to `hit.normal`. Target / guard rings use the target / actor support surface (or a downward projection beneath an airborne target), not world zero. An airborne target's projected marker is not evidence that the actor is grounded.

### Combat callers that require three-dimensional queries

| Caller | Current assumption | Required adaptation |
|---|---|---|
| `visible()` at `src/simulation/combat.rs:336` | Planar point sweep against initial static boxes | World-space ray / volume query between actual actor or weapon-origin positions, including current dynamic box poses. |
| Candidate targets at `:484`–`:495` | Planar range and planar visibility; height stored separately | Keep a full 3D target snapshot, use intended 3D range and actual obstruction. Horizontal facing can remain yaw-based. |
| Cannon destination at `:479`–`:482` and launch at `:594`–`:608` | Planar sweep and world-zero effect endpoints | Resolve a support hit; launch and feedback endpoints carry its actual height / normal. |
| Hammer / spin / splash at `:527`, `:554`–`:556`, `:701` | Radius tests ignore height | Use explicit 3D effect volumes; an actor vertically separated by a hill must not be damaged merely because X/Z projections are near. |
| Attack endpoint at `:675`–`:684` | Planar range plus a broad 2.5m vertical threshold | Resolve actual three-dimensional reach and collision while preserving existing attack cooldown / damage semantics. Instant shots can stay instant; physical projectile spawning is unnecessary for this migration. |
| Enemy hit at `:770`–`:774` | Requires player's **absolute** height <1.1 | Compare actual relative reach. Otherwise enemies and player standing together on a 2m hill would incorrectly be unable to fight. |
| Respawn / death | `Body::teleport()` and `MotionState::reset()` own separate pieces | Reset the complete physical pose and velocities, clear stale support and retained contacts, and reset interpolation snapshots together. |

## Proposed terrain and routes

All dimensions in this section are content recommendations that can be tuned without changing the product requirements. No existing prop needs relocation.

### West ramp and platform

- Ramp footprint: x ∈ [-6,-3], z ∈ [1,5.1212], width 3m. Rise from y=0 at z=1 to y=1.5 at z≈5.1212: `run = 1.5 / tan(20°)`.
- Platform: x ∈ [-6,-3], z ∈ [5.1212,8.1212], top y=1.5. Align the ramp's upper vertices exactly with the platform edge.
- This clears the existing boxes around (-8,7) / (-10,6), the stone around (-5,11), the player pad and all negative-z enemy spawns (`src/gameplay/mod.rs:46`–`:66`, `:97`–`:101`).
- A visible flat approach can run from the spawn-pad west edge through (-4.5,0) to the ramp foot (-4.5,1). Use flush paint / markers, not additional small physical obstacles.
- Use a genuine solid wedge and solid platform (shared vertices / descriptor for mesh and convex collider), or one continuous static surface with solid sides. A rotated thin rectangular board alone can leave an exposed lip at the foot or a hidden obstacle below the slope.

### East small hill

- Peak around (x=16.5,z=8.5), y=2.6. Footprint x ∈ [11,19.5], z ∈ [4,13], comfortably within the 50×50m courtyard. This avoids every current prop's X/Z footprint, including the right pillar at (15,0) and the stone at (17,15).
- Deterministic asymmetric compact bump:

  ```text
  dx = x - 16.5; dz = z - 8.5
  rx = if dx < 0 { 5.5 } else { 3.0 }
  q  = (dx / rx)^2 + (dz / 4.5)^2
  h  = if q < 1 { 2.6 * (1 - q)^2 } else { 0 }
  ```

- The analytic surface and its slope join flat ground continuously: height and first derivative are zero at its edge; the peak also has zero first derivative. Its long west approach has maximum analytic slope around 36°, while the shorter east flank reaches around 53°. With a proposed 45° walkable limit this provides both an accessible summit and a genuinely over-limit sector. These angle values are calculations from the proposed shape, not engine guarantees.
- Start with fixed triangle spacing around 0.5m, refining toward 0.25m if actual capsule traversal or appearance requires it. Use smooth display normals; collision remains the exact shared triangles. Avoid random noise, independent terrain formulas in client and simulation, and stacked cuboids masquerading as a hill.
- A clear approach from the spawn-pad east edge can pass through (11,0), then (11,8.5); ascent proceeds eastwards along z=8.5 from x=11 to the summit. This avoids the boxes at (8,3)/(9,5) and leaves the original enemy exercise area available. Flush route indicators can distinguish the west ramp and east hill without changing input or HUD flows.

### Shared geometry and seam handling

- Prefer a pure-data `TerrainGeometry { vertices: Vec<Vec3>, triangles: Vec<[u32;3]> }` built by gameplay and consumed by simulation / client. It avoids a renderer dependency in simulation. Avian accepts raw buffers directly through `Collider::trimesh_with_config`; the client creates its `Mesh` from those same buffers.
- A single static terrain mesh spanning the flat courtyard and hill eliminates a coplanar hill-base/ground overlap and cross-collider seam. A 0.5m grid across 50m has about 10k vertices / 20k triangles; measure startup and browser performance before increasing global resolution. The ramp/platform may remain separate solid colliders, with their entry and summit seams covered by real traversal tests.
- Do not leave an old flat physical slab coplanar with the terrain mesh's flat triangles. Existing decorative floor thickness can sit below the shared top surface; avoid visible z-fighting if both are drawn.
- Official Avian docs note that triangle meshes are hollow, can require collision margin for robustness, and `FIX_INTERNAL_EDGES` can prevent false contacts at internal edges. That flag does **not** heal edges between separate colliders. Use convex cuboids for movable boxes and keep CCD / penetration testing around thin terrain surfaces.
- A heightfield is also supported, but matching the renderer to its exact grid axes and triangle split must be demonstrated. A raw shared triangle buffer is the simpler initial equality contract; no need to enable mesh-import features just to make terrain colliders.

## Verification scenarios

Use the actual fixed-step simulation with tolerance-based outcome assertions. Extend existing helpers / fixtures rather than writing a second gravity or collision implementation in tests.

1. Geometry: finite vertices, valid nondegenerate consistently wound triangles, expected positive hill peak, flat zero-height perimeter, and both below-limit / above-limit face normals. Verify every old prop and all spawns remain clear of both terrain zones.
2. Support: jump to a box / platform, settle and stand for several seconds, jump again, then walk off and fall. Push the supporting box away with a second actor and verify support loss without a false grounded frame or a jump refill.
3. Ramp: walk uphill / downhill / diagonally, stop for several seconds, cross the base and summit seams, perform both jumps from mid-slope, and drop from a platform edge. Track penetration and contact normals rather than only final height.
4. Hill: traverse the west approach through the summit to a gentler descent; test diagonal movement, crest crossing, stopped posture, jump / landing on different normals, and refusal / sliding on the east steep sector. Repeat at different render-frame rates while retaining the same fixed tick.
5. Dynamic boxes: push, apply an off-center force to tip, settle a two-layer stack and remove its lower support. On sloped ground, verify gravity / friction movement is physically consistent with the configured material.
6. Aim / combat: fire-cannon previews on ramp, hill and box tops; marker position follows the surface and marker normal matches it. Move a box to add / remove obstruction; verify high-to-low / low-to-high attacks and that equal-altitude actors can attack above world y=1.1. Preserve frozen in-flight cannon aim and original hammer-link target behavior.
7. Display: query actor / box display snapshots and confirm they interpolate the physics pose; standing actor and camera rise/fall together while the player retains the expected screen anchor. Marker rings do not float above / disappear into terrain.
8. Pause: freeze during falling, a settling stack and standing on a moving support; advance ordinary frames while paused and verify poses, velocities and jump state do not change. Resume without a gravity backlog or stale input replay.
9. Existing suites that need adaptation: movement fixed-step fixtures (`src/simulation/movement.rs:140`), combat arena fixtures (`src/simulation/combat.rs:846`), cannon collision / committed-marker tests (`:1013`, `:1046`), headless scene fixture (`src/client/combat.rs:401`), actor animation fixture (`src/client/animation.rs:413`), level / spawn tests (`src/gameplay/mod.rs:121`, `:135`). Headless suites must run the real physical schedule, not only the old `FixedUpdate` pair.

## External references

- [Avian 0.8 Collider constructors](https://docs.rs/avian3d/0.8.0/avian3d/collision/collider/struct.Collider.html): raw-buffer triangle mesh / convex hull / cuboid / heightfield constructors, dynamic-shape advice, collision margins and internal-edge caveat.
- [Raw triangle mesh with preprocessing flags](https://docs.rs/avian3d/0.8.0/avian3d/collision/collider/struct.Collider.html#method.trimesh_with_config).
- [Heightfield axis / grid contract](https://docs.rs/avian3d/0.8.0/avian3d/collision/collider/struct.Collider.html#method.heightfield).
- [Avian 0.8 overview](https://docs.rs/avian3d/0.8.0/avian3d/): spatial queries, rigid bodies and schedule overview. Detailed controller / fixed-step integration is owned by the separate integration research topic.

## Related specs

- `.trellis/spec/gameplay/index.md`: deterministic level data, shared content geometry, open spawn and current actor assembly.
- `.trellis/spec/simulation/index.md`: authority, fixed-step consumption, action / combat cleanup and required regressions.
- `.trellis/spec/client/index.md`: read-only animation, same display pose for camera and actor, pause snapshots and asset reuse.
- `.trellis/spec/guides/cross-layer-thinking-guide.md`: data / timing ownership and input lifecycle.
- `.trellis/spec/guides/code-reuse-thinking-guide.md`: preserve a single source for geometry and positions.

## Caveats / Not Found

- This is research only. No terrain, physics component or caller has been implemented or experimentally certified; all existing dirty product edits are preserved.
- Walkable slope angle, density / friction, mesh spacing and contact tolerances need real physics tests plus browser hand-feel acceptance. A mathematically smooth height function does not itself certify smooth capsule traversal over its sampled triangle mesh.
- Capsule origin (center versus feet), support-coordinate convention and interpolation authority must be chosen once in the main design and applied to aim, weapon origin, health bars, spawn and respawn together.
- When a character's collision shape shortens for crouch, retain the world-space feet / support point; standing requires headroom validation. Pose-only crouching under a full-height capsule would not provide physically consistent clearance beneath an overhang.
- The visible 0.7m arena rim is a migration boundary: full 3D collision permits jumping over it unless the content or out-of-bounds behavior is deliberately defined. Retaining an independent planar clamp would reintroduce the same infinite-height-wall problem.
- Shared terrain geometry does not imply feet can blindly be snapped to its analytic height. Dynamic boxes, overhangs, ceilings and support removal still require actual contact / shape queries.
