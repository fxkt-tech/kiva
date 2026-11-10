# Independent model review

Reviewer: dispatched `trellis-check`; first stage read-only while implementation owners work.

## Confirmed finding sent to runtime owner

The first female arm solver preserves segment lengths by clamping unreachable weapon grips.
That leaves the rendered right hand behind the fixed gun grip at steep upward aim.
An independent NumPy evaluation of the actual `stickman.anim` Sword_Idle pose at elapsed
0.31 and the serialized female anchors gives right shoulder
`[0.0972433, 1.1817234, 0.1041157]`, total own arm reach `0.5099939m`.
With the actual gun grip and local firing source `[0, 1.2, -0.55]`, pitch 0.8rad needs
`0.6008134m` (9.082cm gap); pitch 1.0rad needs `0.7234932m` (21.350cm gap).
The original new test covered only 0, +0.35, -0.25rad. Hammer launch raises a target
about 3m relative to its surface, so this must be exercised through real E→Tab state.
The root approved a render-only grounded female torso/leg adaptation; runtime owner
is implementing it. The simulation source, actor root and rules must stay fixed.

The original companion rotation computed the shortest arc from global -Z to the
global aim, then removed actor yaw. That preserves the firing direction but adds
yaw-dependent roll. For actor yaw pi and pitch +0.4rad, model up becomes
`[0,-0.921061,+0.389418]`, opposite the upright weapon up
`[0,+0.921061,-0.389418]`: the imported bow is upside down.
Compute the shortest arc in the actor frame, or compose actor yaw with local
pitch, and test weapon up/right as well as its firing direction across yaw.
Sent to root and runtime owner; runtime owns the fix before final review.

## Final dispositions

- **Steep grip reach fixed by runtime owner.** Own female clavicle rotation uses the
  minimum law-of-cosines arc; pelvis height adjusts within both arms' vertical
  reach intervals and both legs' maximum extension. Own two-segment leg IK keeps
  original foot/toe position and orientation; own arm IK keeps segment lengths.
  Root transform, simulation firing source, timing and physical state stay fixed.
  Reviewed the height interval and leg-rise formulas independently. Tests cover
  actual palette grips, all structural segments and planted feet; a 0.005rad
  pitch sweep from -1 to +1 checks reach and adjacent-pose continuity. Real
  Hammer E→Tab and Chain Q→Tab exercise every visible frame and all three shots.
- **Clavicle threshold snap fixed by runtime owner.** Initial all-or-nothing shrug
  was discontinuous at the reach boundary; minimum-angle rotation tends to zero
  there. The leg pole reuses the prior hip→knee direction to retain bend side.
- **Yaw-dependent weapon roll fixed by runtime owner.** Aim rotation is computed
  after converting direction into the actor frame. Real gun/bow roots now check
  firing, up and right axes at yaw 0, ±pi/2, pi and both upward/downward pitches.
- **Historical documentation drift fixed by root.** The linked older
  `docs/plans/naraka-appearances.md` now identifies its earlier-phase scope and
  points to current default/model/body-only F3 contracts.
- Checker changed only this research report. No additional code/spec finding
  remains after ownership handoff; source-owner fixes were reviewed in place.

## Source and integration review

- Read the task PRD/design/plan, every check.jsonl reference, source evidence and runtime contract.
- Female export uses own Lianmu structural anchors; every outfit/hair/face palette retains
  its own joint order, source rest and IBM. The C/P/IBM/rest equations match actual LBS.
- Female source joint bases are effectively orthogonal: worst normalized-column
  orthogonality errors are 1.123e-6 (outfit), 2.248e-7 (Lianmu), 3.125e-7 (face).
  Runtime Transform decomposition therefore does not hide a material source shear.
- Current-pose gun bake includes both renderers and the original 135 joint1 vertices;
  bow bake includes all nine weighted slots. Both axis adapters have positive determinant.
  Source normals are baked before rigid weapon conversion; female normals retain skinning.
- GLB base factors/PNG/alpha cutoffs/doubleSided are carried through the shared decoder;
  PNGs are explicit sRGB and unknown packed/ID/SH/normal channels stay disconnected.
  Female neutral face/eyes/lashes are clearly recorded adaptations; no original face/shader claim.
- Bow accessories use a separate female spine mount, not the muzzle transform.
- Companion visibility and clocks read CompanionState; source alignment remains
  `state.source_position + interpolated_position - state.position`. F3 is PlayerView-only.
- No simulation/gameplay/Cargo dependency changes were present during the read-only review.
- Root's new model-assets spec and updated architecture/client contracts match the
  exported formats, identities, caches and source limitations at this stage.

## Verification status

- `RUST_TEST_THREADS=2 CARGO_BUILD_JOBS=2 make check`: PASS, format + native Clippy,
  206 library tests + 9 integration tests, 65 Node tests, WASM check.
- `cargo clippy --locked --target wasm32-unknown-unknown -- -D warnings`: PASS.
- Offline runtime exporter `--verify`: PASS all 12 outputs byte-identical;
  truncation/trailing/magic/header and semantic vertex/index/palette/rest/weight
  corruptions rejected as applicable.
- Original female/gun/bow preparation `--verify`: PASS 76/20/33 frozen files.
- Task JSON/JSONL: PASS 3 files and every referenced path. Current documentation
  relative links: PASS 26 links. `git diff --check`: PASS.
- `CARGO_NET_OFFLINE=true trunk build --dist /tmp/kiva-model-gate-Mb9D8P`: PASS
  (Trunk 0.22.1). Dist retained briefly for root's separate startup smoke.
- Cargo.lock unchanged: SHA256
  `f582ba6e933a38619a2f5d29eb5410492b24e4ce7d674d753026b1acdc0d5955`.

Numerical/code checks are not rendered visual acceptance; the documented
software-GPU scene limit still applies. Root performs the separate startup smoke.
