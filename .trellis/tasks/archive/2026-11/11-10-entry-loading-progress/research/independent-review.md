# Independent review

## Contract review before owner handoff

- Reviewed task PRD/design/implementation plan, check.jsonl specs, runtime-contract.md, startup-evidence.md and research/baseline.json. No implementation files modified during this phase.
- Actual bridge/loader regressions: `node --test scripts/input-unit.test.cjs scripts/loader-unit.test.cjs` passed 53/53 while implementation owners were finishing. This is an intermediate result, not the final quality gate.
- `git diff --check` passed at this point.
- Source inspection of installed Bevy 0.20.1 confirms `SpecializedMaterialPipelineCache` is shared across material types; its specialization path requires prepared mesh/material assets. Checking this cache covers both StandardMaterial and the extended ForestMaterial rather than only one material type.
- `render_system` runs the render graph, submits commands and presents before returning. `ViewTarget::needs_present()` reads the attachment's usage flag, which remains set after presentation. The observer after `RenderSystems::Render` and before Cleanup therefore observes a submitted/presented frame rather than merely an allocated target.
- Hidden companion bodies are excluded through render-visible entities. An empty pipeline iterator or empty visible mesh set is not treated as ready. Shader-not-loaded/import-not-yet-available remains pending; shader processing/module creation errors report failure.
- Startup progress cannot enable input. Final `kivaInput.ready()` remains the input authority; real ready overrides failures and ignores delayed progress/errors. Capture uses pointerlockchange plus focus, with pending/rejection/blur feedback and no Promise-based activation.
- No confirmed implementation defect found in this phase. Rust tests, full gates, final artifact audit and browser results remain pending owner handoff/root validation.

## Evidence boundary

Render submission, prepared pipelines and actual output attachment usage establish the selected startup boundary. They do not constitute a GPU readback, appearance acceptance, or proof of the unique cause of the user's hardware-specific first-click behavior. Final browser acceptance belongs to the fixed Trunk artifact, separately from the mixed-version baseline observation.

## Findings fixed after owner handoff

1. **Unfinished WASM response on initialization rejection** — `web/loader.js` kept its original fetch reader open when init rejected without consuming the wrapped response. A real ReadableStream reproduction returned failed with `canceled=false`, two pulls, and the original body still locked. Added a failing regression, then canceled the reader on rejection. Cleanup rejection is consumed so both the thrown error and TrunkApplicationError retain the original cause. The regression verifies cancellation and absence of a false initialization-complete phase.
2. **Trunk placeholder collision in Started event payload** — the first temporary artifact contained `{detail: kiva-b5d10f034fa45658_bg.wasm}` because Trunk also substituted the JavaScript object shorthand `{wasm}`. It caused a ReferenceError instead of dispatching the compatibility event. Changed the pattern to `{detail: { wasm }}` and extended the regression to interpolate and execute the actual pattern. It failed before the fix and now verifies `detail.wasm`, bindings, and a nested public base path.
3. **Unreliable length for CORS-filtered responses** — a CORS response can expose Content-Length while hiding Content-Encoding. Treating the missing header as proof of unencoded data could show a false percentage. Added a failing streamed-response regression and kept this case indeterminate; explicitly available identity encoding remains comparable. Same-origin download measurement is unchanged.

## Final verification at browser handoff

- `make check`: passed. Formatting/native all-target Clippy, 217 Rust unit tests, 9 integration tests, Node suite and WASM cargo check passed. Subsequent changes were confined to loader/config/tests; the full Node suite was rerun after them: **82 passed**.
- WASM Clippy: final frozen Rust source passed `cargo clippy --locked --target wasm32-unknown-unknown -- -D warnings`, as reported by the engine owner and recorded in engine-readiness.md. Reviewer made no Rust edits.
- Final `CARGO_NET_OFFLINE=true trunk build --dist /tmp/kiva-entry-loading-dist.BBkvqj`: passed. Build log: `/tmp/kiva-entry-loading-trunk.log`; full make log: `/tmp/kiva-entry-loading-make-check.log`. This exact dist is retained for root's browser fixture.
- Final emitted initializer has the correct Started event payload and no unresolved built-in placeholders. Packaged startup/loader/input scripts exactly match final source bytes. Generated JS/WASM preload SHA-384 digests match their files and retain anonymous crossorigin.
- Compared against research/baseline.json: protected assets, Cargo.lock, Cargo.toml, simulation, gameplay, camera, Cargo.lock SHA-256, main branch and baseline HEAD unchanged.
- Task JSON and JSONL parse; every referenced file exists (8 entries in each context manifest). All 23 changed/task Markdown local link targets exist. `git diff --check` passed.
- No remaining confirmed implementation defects. Fixed-artifact browser checks are owned by the main session and are not claimed by this review.
