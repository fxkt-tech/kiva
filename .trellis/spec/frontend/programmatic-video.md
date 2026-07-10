# Programmatic Video Preview and Export

## 1. Scope / Trigger

Use this contract when adding or changing an HTML playback preview, Remotion
composition, audio cue, export job, or video-export API. It prevents preview and
MP4 output from becoming two independent renderers with different layout or
timing behavior.

## 2. Signatures

- Route: `GET /games/:gameId/preview_v2?focus=current`
- Create/list: `POST|GET /api/games/:gameId/preview-v2/exports`
- Status: `GET /api/preview-v2/exports/:jobId`
- Actions: `POST /api/preview-v2/exports/:jobId/{cancel|retry}`
- Files: `GET /api/preview-v2/exports/:jobId/{download|assets/*}`
- Composition: `KivaVideoComposition(input: VideoCompositionInput)`
- Timeline: `AudioCue {kind, src, startsAtMs, durationMs, trimStartMs, volume}`

## 3. Contracts

- The React `HtmlPlaybackStage` component tree is shared by Remotion Player and
  server rendering. Frame state is a pure function of input props and frame.
- The desktop preview shell may fit the Player into a `100dvh` workspace, but
  it must scale the Player as one 16:9 box. Viewport size must never alter the
  1920x1080 composition tree, design coordinates, or export metadata.
- The preview shell uses application semantic colors (`bg-background`,
  `bg-surface`, `border-border`, status badge tokens) and follows
  `data-theme`. The composition owns its cinematic palette and must not inherit
  light/dark application theme decisions.
- Atmospheric darkness (background gradient and vignette) belongs inside the
  background layer. It must render before seats, case board, header, and
  subtitles; full-frame texture overlays above content must remain low-opacity.
- Video profile is 1920x1080, 30 FPS, H.264, `yuv420p`, with AAC when cues exist.
- Export creation reloads authoritative game state and snapshots confirmed
  active events plus all mutable assets. A preview draft is never exported.
- Jobs are stored under `KIVA_DATA_DIR/exports/<gameId>/<jobId>` and retained.
- `KIVA_RENDER_ORIGIN` optionally overrides the trusted render origin; default
  is `http://127.0.0.1:3000`.
- Exactly one process-global renderer runs at a time. Later jobs remain queued.
- Every finite audio cue must set `durationMs`. A Remotion `Sequence` without a
  duration remains mounted until the composition ends, so completed
  `<Html5Audio>` elements accumulate and can exhaust Player shared audio tags.

## 4. Validation & Error Matrix

- Unknown game -> `404 game_not_found`.
- No confirmed playback -> `409 empty_playback`.
- Unknown job -> `404 job_not_found`.
- Retry of a non-retriable job -> `409 job_not_retriable`.
- Download before completion -> `409 output_not_ready`.
- Invalid identifiers or asset path segments -> `400` or a sanitized server
  error; filesystem paths and stacks are never returned.
- Output missing the required video profile, or AAC when cues exist -> job
  becomes `failed`; the partial file is removed.

## 5. Good / Base / Bad Cases

- Good: confirmed events and available voices produce a downloadable MP4 with
  H.264 video and AAC audio.
- Base: no audio cues produces a valid video-only MP4.
- Bad: `focus=current` contains only a draft; preview may render it, but the
  export button is disabled and the server still rejects an attempted export.

## 6. Tests Required

- Unit: frame/time rounding, input decoder, audio cue ordering, job transitions,
  repository round-trip, path validation, and singleton service identity.
- Render smoke: probe codec, dimensions, FPS, pixel format, duration, and audio.
- Browser: v2 mounts no canvas, has no console errors, and fixed 16:9 content
  does not overlap at native and scaled sizes.
- Desktop fit: at 1600x900 and 1366x768, document height equals viewport height,
  the Player remains 16:9, and export history scrolls inside its sidebar.
- Theme boundary: test both `light` and `dark`; shell colors must change while
  the stage root and rendered frame remain identical.
- Layer order: at a mid-scene frame, root opacity is `1` and no high-opacity
  vignette is a later sibling of the content grid.
- Audio lifecycle regression: seek across the complete Player timeline and
  assert that the shared `<Html5Audio>` tag limit is never exceeded.
- Integration: a second request stays queued while one render is active;
  queued cancel, retry lineage, download, and restart reconciliation work.

## 7. Wrong vs Correct

Wrong: update visual state from `setTimeout`, CSS animation time, or live audio
progress, then implement a separate export-only layout.

Correct: derive `timeMs` from the Remotion frame, create a renderer-neutral view
model, and render the same `HtmlPlaybackStage` in Player and Renderer. Fit the
outer Player with container-query units rather than adding responsive rules to
the composition. Use semantic application tokens outside the Player instead of
duplicating its hard-coded cinematic colors. Bound each system-voice cue to its
playback scene instead of increasing `numberOfSharedAudioTags` to hide
unbounded mounts. Keep dark vignettes inside the background instead of placing
them over information content. Remotion's Webpack disk cache stays disabled
because the application already caches the completed bundle per process; this
avoids corrupted cache packs during local
concurrent builds.
