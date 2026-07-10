# HTML Video Preview v2 — Implementation Plan

## Delivery Strategy

Implement one end-to-end vertical slice in dependency order. Keep each step
independently testable and keep v1 runnable after every step. Do not begin visual
polish before a real short MP4 has passed the render smoke test; that test proves
the architecture before significant design effort is invested.

## Gate 0 — Pre-development context

- [ ] Run the Trellis before-development workflow and read all relevant frontend
      specs.
- [ ] Recheck git status and preserve unrelated user changes.
- [ ] Record the approved individual-use Remotion license context and pin the
      terms/version reference used for implementation.
- [ ] Choose and pin one exact version for 'remotion', '@remotion/player',
      '@remotion/renderer', '@remotion/bundler', '@remotion/media', and
      '@remotion/tailwind-v4'.
- [ ] Record required local runtime checks for writable 'KIVA_DATA_DIR' and the
      supported Chromium/compositor/FFmpeg path.

Validation:

~~~sh
git status --short
node --version
ffmpeg -version
ffprobe -version
~~~

Rollback point: no source changes.

## Gate 1 — Composition and audio contracts

- [ ] Add centralized video constants for 1920x1080, 30 FPS, H.264/AAC,
      'yuv420p', and composition ID.
- [ ] Add JSON-safe 'VideoCompositionInput', 'CompositionAssets', and
      'AudioCue' contracts.
- [ ] Add runtime decode/normalization at the persisted/API input boundary.
- [ ] Add pure milliseconds/frame conversion helpers with explicit rounding.
- [ ] Add 'buildAudioTimeline()' and the current system-voice resolver adapter.
- [ ] Make missing audio a warning/empty cue, never a composition error.
- [ ] Add tests for schema version, malformed JSON, frame rounding, cue ordering,
      overlap, missing sources, and silent output input.

Files expected:

~~~text
src/components/preview-v2/composition/types.ts
src/components/preview-v2/composition/video-spec.ts
src/components/preview-v2/composition/timing.ts
src/components/preview-v2/audio/audio-timeline.ts
src/components/preview-v2/audio/audio-timeline.test.ts
~~~

Validation:

~~~sh
pnpm test -- src/components/preview-v2
pnpm typecheck
~~~

Rollback point: contracts are additive and have no route integration.

## Gate 2 — Shared HTML composition skeleton

- [ ] Add 'KivaVideoComposition' driven only by Remotion frame/config and
      'VideoCompositionInput'.
- [ ] Derive the active playback scene and renderer-neutral shot view model from
      existing playback/shot-engine functions.
- [ ] Render optional audio cues as frame-positioned Remotion sequences.
- [ ] Add the fixed-size 'HtmlPlaybackStage' skeleton with header, two seat
      tracks, center dispatcher, and subtitle region.
- [ ] Add required font loading and image primitives that block frame capture
      until ready.
- [ ] Ensure no v2 module imports 'pixi.js', '@pixi/layout', or mounts canvas.
- [ ] Add deterministic markup tests for empty and representative scenes.

Files expected:

~~~text
src/components/preview-v2/composition/kiva-video-composition.tsx
src/components/preview-v2/composition/frame-view-model.ts
src/components/preview-v2/stage/html-playback-stage.tsx
src/components/preview-v2/stage/*.tsx
src/components/preview-v2/stage/stage-theme.ts
~~~

Validation:

~~~sh
rg -n "pixi.js|@pixi/layout|canvas" src/components/preview-v2 src/video
pnpm test -- src/components/preview-v2
pnpm typecheck
~~~

Rollback point: v1 still owns every public preview route.

## Gate 3 — Remotion bundle and real render spike

- [ ] Add the Remotion root/entry and composition registration.
- [ ] Configure Tailwind v4 for the Remotion Webpack bundle.
- [ ] Add a fixture composition with one short scene and zero audio cues.
- [ ] Add a server-only renderer adapter around 'bundle()',
      'selectComposition()', and 'renderMedia()'.
- [ ] Prove the Next 16 production build with renderer/bundler imports isolated
      to server-only modules; add 'serverExternalPackages' entries only when the
      build spike demonstrates they are required.
- [ ] Cache the bundle per process and expose progress/cancellation hooks through
      the adapter rather than leaking Remotion types into job orchestration.
- [ ] Render a real short MP4 before proceeding.
- [ ] Probe the output and assert 1920x1080, 30 FPS, H.264, expected duration,
      and readable MP4 container.
- [ ] Repeat with one audio cue and verify AAC; repeat without a cue and verify a
      valid silent/video-only result according to the chosen compatibility
      policy.

Files expected:

~~~text
src/video/index.ts
src/video/root.tsx
src/video/video.css
src/server/video-export/remotion-renderer.ts
src/scripts/render-preview-v2-smoke.ts
~~~

Validation:

~~~sh
pnpm video:smoke
ffprobe -v error -show_streams -show_format <generated-smoke-file>
pnpm typecheck
~~~

Hard gate: do not continue if preview and rendered fixture cannot use the same
composition component.

Rollback point: remove Remotion dependencies and additive v2 modules; v1 remains
unchanged.

## Gate 4 — Visual system and adverse-content behavior

- [ ] Implement day/night background treatment and investigation/mansion visual
      tokens.
- [ ] Implement stable six-slot seat tracks and redesigned seat cards.
- [ ] Implement speech scene with active portrait/dossier hierarchy.
- [ ] Implement case-board variants for phase, announcement, vote, and
      resolution.
- [ ] Reuse or extract existing pure case-board content projection instead of
      duplicating event interpretation. Preserve v1 behavior through a
      compatibility import if an extraction is necessary.
- [ ] Implement deterministic subtitles and frame-derived enter/exit motion.
- [ ] Encode explicit limits/fallbacks for long names, titles, bodies, detail
      rows, missing avatars, dead players, and dense content.
- [ ] Build fixed visual fixtures covering each scene kind, day/night,
      enter/middle/exit, and adverse content.
- [ ] Compare rendered stills at native 1920x1080 and scaled browser size.

Validation:

~~~sh
pnpm test -- src/components/preview-v2
pnpm video:smoke
pnpm typecheck
~~~

Manual visual gate:

- [ ] No overlap or unintended clipping.
- [ ] Information hierarchy remains readable at 1920x1080 and embedded scale.
- [ ] Repeated render of the same frame is visually identical.
- [ ] Missing optional assets use defined fallbacks.

Rollback point: retain the proven composition skeleton and revert only visual
components/tokens if redesign iteration fails.

## Gate 5 — Durable export repository and immutable snapshots

- [ ] Add branded/validated export job identifiers and versioned job/input
      records.
- [ ] Add centralized job transition reducer with exhaustive status handling.
- [ ] Add filesystem repository below
      'KIVA_DATA_DIR/exports/<gameId>/<jobId>/'.
- [ ] Use atomic temp-file + rename writes for JSON status and final MP4.
- [ ] Build snapshots server-side from confirmed active events only.
- [ ] Copy font, backgrounds, referenced avatars, and available audio into the
      job directory.
- [ ] Rewrite input URLs to validated job asset endpoints.
- [ ] Store optional-audio warnings without failing snapshot creation.
- [ ] Make retry clone the immutable snapshot into a new linked job rather than
      recompiling the current game.
- [ ] Retain every job directory indefinitely; add no cleanup behavior.
- [ ] Add tests for atomic round-trip, invalid records, confirmed-only snapshot,
      draft exclusion, missing avatar/audio, path traversal, and retry lineage.

Files expected:

~~~text
src/server/video-export/types.ts
src/server/video-export/job-state.ts
src/server/video-export/export-repository.ts
src/server/video-export/snapshot-builder.ts
src/server/video-export/*.test.ts
~~~

Validation:

~~~sh
pnpm test -- src/server/video-export
pnpm typecheck
~~~

Rollback point: export data is isolated below 'KIVA_DATA_DIR/exports' and does
not alter game records.

## Gate 6 — Serialized queue and restart behavior

- [ ] Add the process-global queue singleton with concurrency exactly one.
- [ ] Persist queued status before returning to callers.
- [ ] Connect renderer progress with throttled durable updates.
- [ ] Map Remotion render/encode/mux stages into the job projection.
- [ ] Add active and queued cancellation.
- [ ] Catch renderer exceptions and persist sanitized typed failures.
- [ ] Reconcile stale queued/preparing/rendering jobs to 'interrupted' after
      process restart.
- [ ] Start the next job from a 'finally' path after completion, failure, or
      cancellation.
- [ ] Verify completed output with probe metadata before atomic final rename.
- [ ] Test two-job serialization, progress, cancel, failure isolation, restart
      reconciliation, retry, and output verification failure with a fake
      renderer.

Files expected:

~~~text
src/server/video-export/export-queue.ts
src/server/video-export/output-verifier.ts
src/server/video-export/export-service.ts
src/server/video-export/*.test.ts
~~~

Validation:

~~~sh
pnpm test -- src/server/video-export
pnpm typecheck
~~~

Rollback point: queue adapter can be disabled while repository/snapshots remain
inspectable.

## Gate 7 — Export HTTP API

- [ ] Add create/list/status/cancel/retry/download/job-asset route handlers.
- [ ] Mark export route handlers for the Node.js runtime and keep renderer-native
      packages out of all client/Edge module graphs.
- [ ] Make create/retry return 202 after durable enqueue.
- [ ] Load authoritative game state on the server; never trust client playback
      items.
- [ ] Validate game/job IDs and ownership for every route.
- [ ] Stream completed MP4 with a stable attachment filename.
- [ ] Stream only allowlisted snapshot assets resolved from job metadata.
- [ ] Return typed public error projections without absolute paths/stacks.
- [ ] Add route/service tests for successful and invalid transitions, missing
      game/job, cross-game lookup, non-completed download, and traversal input.

Files expected:

~~~text
src/app/api/games/[gameId]/preview-v2/exports/route.ts
src/app/api/preview-v2/exports/[jobId]/route.ts
src/app/api/preview-v2/exports/[jobId]/cancel/route.ts
src/app/api/preview-v2/exports/[jobId]/retry/route.ts
src/app/api/preview-v2/exports/[jobId]/download/route.ts
src/app/api/preview-v2/exports/[jobId]/assets/[...path]/route.ts
~~~

Validation:

~~~sh
pnpm test
pnpm typecheck
~~~

Rollback point: remove v2 API route tree; v1 recording is unaffected.

## Gate 8 — Preview v2 page and controls

- [ ] Add the new server page and reuse existing game loading,
      'focus=current', voice-duration, and playback compilation behavior.
- [ ] Add 'PreviewV2Studio' with the Remotion Player and custom controls.
- [ ] Preserve play/pause, seek, previous/next scene, reset, progress/time,
      space key, and arrow key behavior.
- [ ] Use the Player frame as the sole interactive clock.
- [ ] Show a clear draft-exclusion notice when preview includes current draft.
- [ ] Add create, queue progress, cancel, retry, history, and download UI.
- [ ] Poll once per second only while non-terminal jobs exist; stop polling when
      terminal and on unmount.
- [ ] Disable export for empty confirmed playback and explain why.
- [ ] Keep page controls outside the 1920x1080 composition so they never appear
      in exported frames.

Files expected:

~~~text
src/app/games/[gameId]/preview_v2/page.tsx
src/components/preview-v2/preview-v2-studio.tsx
src/components/preview-v2/playback-controls.tsx
src/components/preview-v2/export-panel.tsx
~~~

Validation:

~~~sh
pnpm test -- src/components/preview-v2
pnpm typecheck
pnpm build
~~~

Manual gate:

- [ ] Preview seeks deterministically.
- [ ] Draft is visible only in preview mode and clearly excluded from export.
- [ ] Refresh shows durable job history/progress.
- [ ] Completed MP4 downloads and plays.

## Gate 9 — Additive navigation

- [ ] Add 'Open Preview v2' beside the editor preview controls without replacing
      the existing iframe or v1 link.
- [ ] Add a separate v2 link per game on the home page.
- [ ] Keep all current '/preview' URLs and semantics unchanged.
- [ ] Add/update markup tests for both links.

Validation:

~~~sh
pnpm test
pnpm typecheck
pnpm build
~~~

Rollback point: remove only the new links; direct v2 route can remain available.

## Gate 10 — Full quality and end-to-end verification

- [ ] Run the complete unit/integration suite.
- [ ] Run type checking and production build.
- [ ] Run a real export from an existing game containing multiple scene kinds.
- [ ] Probe the final MP4 for codec, AAC when cues exist, dimensions, FPS, and
      duration.
- [ ] Compare representative Player frames to rendered stills/video frames.
- [ ] Verify v1 preview playback and WebM recording are unchanged.
- [ ] Interrupt a render/process and verify the job becomes retriable.
- [ ] Verify two requested exports run serially.
- [ ] Verify no automatic deletion occurs and outputs land below
      'KIVA_DATA_DIR/exports'.
- [ ] Run Trellis quality check and resolve every verified finding.

Commands:

~~~sh
pnpm test
pnpm typecheck
pnpm build
pnpm video:smoke
ffprobe -v error -show_streams -show_format <final-export.mp4>
git diff --check
git status --short
~~~

## Risk Register

### Remotion and Next/Tailwind bundler mismatch

Mitigation: prove Gate 3 with the actual shared component before visual work;
pin all Remotion package versions exactly.

### Asset URLs render in Player but fail in headless export

Mitigation: snapshot assets and test job-scoped HTTP routes in the real smoke
render; do not rely on mutable application asset URLs.

### Font loads after capture

Mitigation: composition-level font readiness gate plus rendered-frame smoke
fixture containing known CJK metrics.

### Next development reload creates multiple workers

Mitigation: process-global singleton and idempotent job transition checks. State
explicitly that multi-process deployment is unsupported.

### Long renders block request lifecycle

Mitigation: POST persists/enqueues then returns 202; renderer runs outside the
request promise and progress is polled.

### Permanent storage growth

Accepted product decision: no automatic cleanup. Surface output paths and sizes;
operator manages the directory manually.

### V1 regression through shared extraction

Mitigation: prefer additive v2 adapters. If pure content logic must move, retain
a compatibility re-export and run all existing preview tests before proceeding.

## Final Review Gate

Before 'task.py start':

- [ ] User reviews and approves 'prd.md', 'design.md', and 'implement.md'.
- [ ] No blocking product questions remain.
- [x] Remotion is accepted for the current individual-use context; revisit the
      decision if project usage expands.
- [ ] The first implementation target is Gate 1, not broad visual coding.
