# Research: startup loader evidence and first-click observation

- Query: Exact Trunk 0.22.1 download progress extension points; an agent-runnable feedback loop against the actual initial entry, with no post-ready sleep.
- Scope: mixed (local generated loader/input, actual browser, official version-pinned Trunk source).
- Date: 2026-11-10

## Findings

### Files found

- `index.html`: synchronous startup/input bridge scripts before Trunk's Rust module; original entry button is hidden.
- `dist/index.html:26`: actual generated main initializer imports hashed wasm-bindgen module, calls `init({module_or_path: wasmUrl})`, exports `window.wasmBindings`, dispatches `TrunkApplicationStarted` with `{detail:{wasm}}`.
- `dist/kiva-83b0dbbcece0999.js:2427`: wasm-bindgen consumes a `Response`, checks HTTP status, uses native `WebAssembly.instantiateStreaming`, and has the MIME fallback.
- `dist/kiva-83b0dbbcece0999.js:2496`: generated init accepts a Response/Promise through `module_or_path`; it fetches itself only for URL/string/Request arguments.
- `web/input.js` (original lines 78-114): ready alone exposes entry; `enter()` requests lock but does not rerender pending state.
- `web/startup.js` (original lines 3-30): loading/failed/ready strings, readiness wins, no download progress.
- `.trellis/spec/client/input.md`: trustworthy pointer lock still requires actual `pointerlockchange` plus focus, not Promise resolution.
- `/tmp/kiva-startup-evidence/baseline.cjs`: reproducible actual-browser probe; owns/cleans its Chromium instance, reads user server at 3221 without managing it.
- `/tmp/kiva-startup-evidence/headed-report.json`, `headed-run.log`: initial UI baseline and lock rejection evidence.
- `/tmp/kiva-startup-evidence/baseline-report.json`, `baseline-run.log`: later headless intermediate UI run (live devserver picked up concurrent UI edits).
- `/tmp/kiva-startup-evidence/trunk-{rust,output,initializer,guide-initializer}-v0.22.1.*`: version-pinned official source snapshots for loader claims below.

### Runnable feedback loop and actual output

Command (uses installed Playwright plus existing Chromium/temporary system libraries):

```sh
node /tmp/kiva-startup-evidence/baseline.cjs --assert
# Headed mode used for original baseline:
KIVA_QA_HEADED=1 node /tmp/kiva-startup-evidence/baseline.cjs --assert
```

The script opens a fresh context with cache disabled, observes `DOMContentLoaded`, readiness, trusted click, actual native `requestPointerLock`, Promise/events, and long tasks. It clicks immediately after observing ready; it inserts no 5-second stabilization delay. The assertions require the loading button visible+disabled and the capture-pending button disabled. A browser failure is separately recorded.

Initial headed run used original entry UI and exited 1:

```text
LOADING t=149.8 startup=loading hidden=true disabled=false
READY observed t=6199.3 startup=ready hidden=false disabled=false
trusted click t=12847.9
requestPointerLock t=12848.4 hidden=false disabled=false
pointerlock-promise-rejected t=12882.2
  The root document of this element is not valid for pointer lock.
pointerlockerror t=12932.6, retry message visible
```

These captured DOM values deterministically violate visible/disabled loading and immediate pending feedback. The run itself exited on the active-state timeout after a real capture rejection, before its final contract assertions; do not call that timeout a failing loading-progress assertion. It does **not** establish that the user's hardware suffers the same rejection or reproduce their exact performance latency. Long tasks were 456ms, 2319ms, 1133ms and 69ms. Rust's `kiva-ready` occurred at 4420.6ms within a long task; the polling observer ran at 6199.3ms. An externally observed ready time can therefore be later than its synchronous dispatch.

Headless rerun succeeded with no added post-ready delay, exit 0:

```text
DOMContentLoaded t=41.1 loading button visible=true disabled=true
TrunkApplicationStarted t=2234.6
kiva-ready t=3988.1
trusted click t=5863.8
requestPointerLock t=5864.4 disabled=true text=正在进入…
pointerlock Promise resolved t=5908.3
pointerlockchange t=5910.3
active observed t=5926.8
CONTRACT initialButtonVisible=true initialButtonDisabled=true pendingCaptureDisabled=true
```

**Version distinction:** the user's existing Trunk watcher served concurrent UI work during this second run. It is an intermediate integration observation, with new button/pending UI and the old Rust Startup readiness boundary. It is not an unmodified baseline or final engine validation. The 46.5ms click-to-lock success in headless Chromium makes the headed root-document failure an environment-dependent observation; do not diagnose the user's bug from it. GPU async pipeline counters were zero because this runtime uses another pipeline path; those zero counters prove no render readiness property.

### Exact Trunk 0.22.1 extension points

Installed `trunk --version` is `trunk 0.22.1`. Prefer the actual generated initializer and version-pinned source over current prose docs.

1. `data-initializer="web/module.js"` is supported. The module **default exports a function** returning `{onStart,onProgress,onComplete,onSuccess,onFailure}`. `onProgress` receives `{current,total}`, `onSuccess` receives wasm, `onFailure` receives the reason. The file is copied/minified/hashed and preloaded by Trunk (`rust/mod.rs:746-768`, `rust/output.rs:159-175`).
2. Its `total` comes from `sourceSize`, injected from filesystem metadata of the final WASM (`rust/mod.rs:742`, `initializer.js:18`). It is a true decoded WASM-byte denominator, not network wire bytes. The version-pinned guide's statement that missing Content-Length produces total=0 is stale relative to this exact implementation.
3. The built-in initializer wraps only the WASM fetch in a `ReadableStream`/`Response`, preserving headers/status/statusText, and gives that Response to wasm-bindgen. It does not globally intercept fetch. Cache hits still produce a body stream and measured chunk bytes.
4. Important built-in failure caveat: `reader.read().then(...)` lacks a rejection handler/`controller.error`; an interrupted stream can leave init pending and an unhandled rejection. EOF forces `current=total` rather than reporting the actually accumulated current. Initialization failure calls `onFailure` and returns the reason (does not reject), allowing the default `TrunkApplicationStarted` dispatch after failure. These details favor a small explicit loader when rigorous failure propagation is required.
5. `data-loader-shim` is **worker-only**; setting it for main fails the build (`rust/mod.rs:203-208`). It is not the entrypoint customization API.
6. `[build] pattern_script` replaces the main injected initializer and supplies literal placeholders `{base}`, `{js}`, `{wasm}`, `{crossorigin}` (`rust/output.rs:76-98`). There is no built-in `{size}`/WASM-size parameter in this custom pattern path. Default integrity/preload injection still occurs unless independently customized.
7. Custom parameters can start with `@` to load file contents, but substitution iterates a HashMap (`rust/output.rs:38-49`). Avoid putting built-in placeholders inside an `@file` expansion: replacement order may leave them unresolved. Direct literal `pattern_script` containing `{base}{js}`/`{base}{wasm}` safely interpolates the actual hashed outputs.
8. Default output has **no `__trunkReady` contract**. Preserve `window.wasmBindings` and `TrunkApplicationStarted` detail if replacing it; surface actual errors as `TrunkApplicationError`/startup failure. Only Rust engine readiness enables entry.

### Recommended narrowly scoped mechanism

Use a dedicated loader called by a literal Trunk `pattern_script`: fetch only the passed WASM URL; validate the HTTP status; wrap only its response body, report accumulated `value.byteLength`, preserve native streaming through `init({module_or_path: wrappedResponse})`, and call `controller.error(error)` on stream failures. The loader must propagate failure to the startup bridge and not signal engine readiness.

Without injected build metadata, trust Content-Length as the decoded-byte denominator only when no Content-Encoding (or identity) makes it comparable to the decoded fetch stream. Missing, nonnumeric, zero, or compressed lengths should report unknown total/bytes without a percentage; completed download transitions to an indeterminate initialization stage. If measured current exceeds a claimed total, invalidate the denominator. A fetched cached Response should follow the same measurement path. Never make download 100% mean engine ready, and never use a timer percentage.

### External references (primary, version-pinned)

- [Trunk 0.22.1 Rust pipeline](https://github.com/trunk-rs/trunk/blob/v0.22.1/src/pipelines/rust/mod.rs): initializer discovery, worker-only shim, final WASM metadata.
- [Trunk 0.22.1 output generator](https://github.com/trunk-rs/trunk/blob/v0.22.1/src/pipelines/rust/output.rs): interpolation and exact default main loader.
- [Trunk 0.22.1 initializer implementation](https://github.com/trunk-rs/trunk/blob/v0.22.1/src/pipelines/rust/initializer.js): actual byte/progress/error behavior.
- [Trunk 0.22.1 initializer guide](https://github.com/trunk-rs/trunk/blob/v0.22.1/guide/src/advanced/initializer.md): callback module shape; its Content-Length description conflicts with implementation.

### Related specs / design

`.trellis/spec/client/index.md`, `.trellis/spec/client/input.md`; task `prd.md` L01-L06 and `design.md` startup bridge/readiness boundaries. Bevy readiness APIs belong to the engine owner's separate `runtime-contract.md` research.

## Caveats / Not Found

- The user's exact first-click latency was not reproduced. The observed long tasks explain why Startup dispatch alone is an inadequate proof of immediately observable readiness, but they do not identify a unique hardware/root cause.
- The existing read-only devserver serves a moving working tree. Final validation should use root's isolated temporary Trunk build and record that exact artifact.
- A real browser lock success remains separate from whether the scene/render pipelines are fully usable; the shader/readiness implementation needs its own verification.
- `https://trunkrs.dev/assets/` redirected to unrelated content during lookup; no claims rely on that domain. Official version-tagged GitHub source was used instead.
- Temporary diagnostic artifacts are not durable project files; the important timings, commands, source contracts and caveats are persisted here.
- Root stopped additional live-server baseline runs/loader fixtures after the mixed-version observation. UI owner supplies focused loader tests; root owns final immutable Trunk artifact browser validation. No further fixture was run against the newly added `web/loader.js` by this research agent.
