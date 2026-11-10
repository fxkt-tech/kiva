# Independent melee review

## Review scope and source findings

Review baseline: `main` at `f1da207ef081f4d736a2af4330d2c646d8f62fd8`. No commit, push, user development-service operation, or external message was performed.

- Read check.jsonl, PRD/design/implementation plan, client/model/combat/cross-layer/reuse specs, original prepared READMEs, source-evidence, runtime contract and baseline manifest.
- Original controller materialGroupConfigs links blade/back/head/handle groups to distinct LOD0 renderer assistants; both FX inputs belong to group4. This supports including main plus four complementary regions and excluding FX. The source controller does not establish an executed default static arrangement.
- `source_lbs` independently accumulates all actual source bone transforms times original inverse binds and weights before the rigid bake. Fulangmao retains five native source slots and the original 448-vertex UInt32 single-index / implicit-weight1 attachment. Zhexing retains its native two-slot main and four originally static regions.
- Authored assembly uses positive-determinant transforms after original current LBS / static hierarchy. Topology and UVs remain original; materials come from source base PNG and prepared neutral factors. Unknown Float/packed/normal channels are not treated as color/ORM or geometric displacement. Source HeadPosition is explicitly an authored design reference rather than a recovered Actor transform or executed star rule.
- `WeaponModel` owns four bounded pair+mode caches and common WP01 read/spawn paths. Held and flying chainaxe spawn the same cache handles. Remote origins/axes and companion mounts retain the prior data path.
- Update ordering places `avatar::sync` and `stickman::sync_attachments` before `weapons::sync_tether`. Tether composes the current local wrist attachment with the current PlayerView Transform; it does not read a stale GlobalTransform or write simulation state.
- Initial frozen-content audit: old five runtime tree OIDs match baseline; no tracked/untracked changes within those runtime directories; no prepared diff; Cargo.lock SHA256 matches baseline.

## Final findings

No production defects or unresolved spec drift were found. The reviewer made no production-code edits.

Read-only feedback resolved before final handoff:

- Root removed the redundant template/TBD PRD prefix and synced the canonical client/model/combat specs plus architecture to actual four-slot cache and wrist/tether scheduling.
- Export owner finalized exact dimensions/assembly matrices and source limits in runtime-contract. Conversion reports now compare every actual source base RGB/alpha pixel, and connect actual f32 part positions through a <=2mm proximity graph. Nearest triangle vertices give an upper bound on surface gap; no exact triangle intersection or topology welding is claimed. CPU part-color side projection was inspected separately and shows complete connected silhouettes and sensible grip placement, without claiming original shader or rendered game acceptance.
- Runtime owner added actual Sniper HitStop freeze/resume, moving-root zero-step/multiple-fixed-step tether, and paused body replacement while axe already away. Tests preserve RigPose, physical position, handle reuse and held/flying visibility. Compact HUD primary names retain class and away-state information through captions/accessible labels.

## Verification

All final commands completed successfully after export/runtime owners released their files:

| Gate | Result |
| --- | --- |
| `make check` | PASS: rustfmt; native all-target Clippy; 211 Rust unit + 9 integration tests; 65 Node tests; WASM target check |
| `cargo clippy --locked --target wasm32-unknown-unknown -- -D warnings` | PASS |
| `CARGO_NET_OFFLINE=true trunk build --dist <temporary-directory>` | PASS; distribution handed to root at `/tmp/kiva-melee-review-dist.NSta78` for browser fixture and cleanup |
| `uv run --offline --with numpy --with pillow python scripts/import-melee-models.py --verify` | PASS: both packages, all eight files byte-identical; all seven malformed-input variants rejected per binary |
| Original `prepare-naraka-weapons.py --verify` for both packages | PASS: Fulangmao 24 frozen files/1 GLB/9 PNGs; Zhexing 122 files/7 GLBs/20 PNGs, original skin/static inventories preserved |
| Final frozen baseline and scope audit | PASS: main HEAD remains f1da207; old five runtime directories and tree OIDs, all prepared files, simulation/gameplay, camera, Cargo manifests/lock and old helpers unchanged |
| Task JSON/JSONL | PASS: parse and file references exist |
| `git diff --check` | PASS |

Final binary SHA256:

- Fulangmao: `104f01a3ced11c443ba2ea8f650d38fb9a92729bcea11c5e08ad7987f2f722c0`.
- ZhexingCuishan: `8828682dd2a818c37043d51b3ba166974acb7fd2e502f27e0e55e2b4d40c7dc1`.

## Remaining acceptance limits

Rendered in-game visibility/materials/feel are outside these headless/source checks. Prior full-game Chromium/SwiftShader renders only background and HUD at the old baseline as well; this reviewer does not claim a rendered visual pass. Root owns any final browser fixture observation and cleanup. The explicitly authored Zhexing arrangement and unknown shader/star/Float/FX limits are accepted scope and remain documented.
