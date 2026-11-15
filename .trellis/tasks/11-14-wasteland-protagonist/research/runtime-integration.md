# Zhuque authored runtime integration

## Approved boundary
New authored mesh exporter and selectable appearance ID3. No simulation/attack-clock changes, no existing visible asset or frozen source overwrite, default Lihuowang unchanged. Game startup still requires all appearances in the existing manifest; viewer retains selected-resource pending/fallback. AV04 is a backward-compatible material extension; legacy layouts stay intact. Explicit auxiliary cape collapse is justified numerically, not guessed.

## Delivered
- Strict source: `art/wasteland/textured/zhuque-v11-textured.blend`, SHA256 `b9847c7088bcb171e9dbccd8d13bdd42f1b26b7e3aff992a3e67a445ba17f59b`.
- `tools/wasteland/export_runtime.py`, `runtime-export.md`: independent background Blender, read-only source, own new bind, strict direct image material interface, bounded PNGs including ORM, actual GLB + runtime proof, atomic publication after validation.
- `assets/characters/zhuque.bin`: AV04, SHA256 `2c15b4cde4bc5d7b4c97cb9dab27f7f24f0195fcc668903f81af02932f63959f`; 39,976 triangles, 4parts, 4materials,16 2K images; no development flags.
- `art/wasteland/runtime/zhuque.glb`, compact mesh JSON, textures, conversion report. Provenance committed-path deliverables at `provenance/characters/zhuque/`.
- General material decoder supports alphaMode/opacity/HDR emission/emissive texture/surfaceEffect; validates malformed fields. Effect module owned by separate worker; hooks register material variants once and sample viewer absolute time.
- Browser ID3 / viewer ID `zhuque`, native F3 cycle, stable old IDs/default. Cache and pending paths extended, existing weapon/attack behavior reused.

## Verification
Strict exporter reads all140 real target snapshots and actual Blender evaluation, then actual GLB JSON/BIN node worlds/skin/IBMs and vertex attributes. No old inverse-bind copied: sourceRest R, new IBM inverse(R), runtime A*inverse(R). Authoring 52core + syntheticroot +3cape auxiliary; runtime52core; explicit `cape_L/C/R -> gMan Spine2`.

Final proof: joint matrix2.61e-6, auxiliary delta7.75e-7; 223,740 actual Blender vertices max0.437mm runtimeLBS error. ActualGLB44,748vertices binderror0.000571mm, UV2.98e-8; 223,740 posedGLB vertices (including actual attacktick024) max0.438mm runtimeLBS error.

Final `make check` passed: native fmt/all-targetClippy, Rust362 main tests +integration tests (2 explicit exporters ignored), Node116, WASMcheck. WASMClippy passed. Pythonpy_compile/Ruff and diffcheck passed. Temporary offline Trunk build passed at `/tmp/kiva-zhuque-dist.VL19KU`; emitted asset hash equals final source and HTML includes Zhuque button. Cargo.lock unchanged. No server started or stopped.

## Limits
No cloth physics/facial expressions. Opaque cloak/hair are static emission; only dedicated transparent fringe has procedural flow/pulse. Actual browser/game visual QA is owned by parent and remains separate from numerical checks. Startup downloads the new required20MB asset along with prior appearances; no new lazy-game loading state was introduced.

Final flame revision strictly re-exported. GLB explicit AO uses existing MR texture/image/sampler indices and exact declared complete ORM PNG; all4 independently read back with hashes in provenance glb-material-report.json. Runtime code unchanged; existing Rust/WASM checks retained.

## Current cape refinement publication (supersedes first-version asset above)

Approved runtime source `art/wasteland/cape-refined/zhuque-cape-runtime-source.blend` SHA964a094925d12fa9770f62fd5b87701a1c41514d68178c7517f71a0217b0c004; explicit `--triangle-budget150000` (formal argument, default remains40000, accepted1..250000). Actual139152tri /7parts /5materials /20actual2KPNGs. Published runtime SHA4ccf4c8392b99c99ce075ace84eabb95cdfef6f478ad4e31e8052f1cfd053c23. Separate editable high source is430128tri; source/LOD shape differences and historical prior baseline recorded in provenance; old source preserved. Sixteen prior body/fire texture-channel PNG hashes exactly unchanged.

Strict proof initially rejected GLB missing actualSolidify geometry because native glTF default `export_apply=false`. Explicit `export_apply=true` applies non-Armature modifiers while keeping skin; rerun proves actualGLB thickness geometry instead of weakening tolerances. All140core/aux matrices pass,460110 actualBlender and460110 actualGLB posed vertices max0.613mm;92022GLB bind/UV vertices pass,5material AO readback exactdeclaredORM. Dedicated capeSolidify inner/outer/rim texel sharing is an explicit fabric exception,ORM.R=1, no differentialAO baked.

Final targeted authoredZhuque actualpalette/weapon-grip regression passed(1test). Isolated offlineTrunkbuild passed at `/tmp/kiva-zhuque-dist.VL19KU`, assethash matches final. Source/high/tool/runtime/GLB/all20PNG provenance hashes independently verified, Pythoncompile/Ruff/diffcheck pass; no Rust/runtime-schema change so prior full makecheck/WASM results retained. Root owns final actualGPU acceptance; numerical proof does not replace it.
