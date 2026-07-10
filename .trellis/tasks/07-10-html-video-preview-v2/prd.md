# HTML video preview v2

## Goal

Add an independent `/games/[gameId]/preview_v2` experience that renders the
playback stage with React and Tailwind instead of PixiJS, establishing a
deterministic path from the browser preview to a final encoded video without
changing the existing `/preview` implementation.

The product problem is not merely to reproduce the old canvas. The preview and
the exported video must share one visual implementation so layout, typography,
timing, and assets do not drift between what the user reviews and what the
system exports.

## Background

- The existing `/games/[gameId]/preview` route compiles `PlaybackItem` records,
  derives `ShotFrame` from an explicit `timeMs`, renders through PixiJS, and
  records the canvas in real time with `captureStream()` and `MediaRecorder`.
- The existing Pixi renderer mixes layout-engine positioning with manual
  geometry, making HTML-like card and typography layout fragile.
- React, Tailwind CSS, `compilePublicPlayback()`, `createShotFrame()`, system
  voice selection, and voice-duration discovery already exist in the project.
- The new work must be additive. Existing `/preview` behavior remains available
  as the comparison and rollback path.

## Requirements

- Add a new route at `/games/[gameId]/preview_v2`; do not replace, redirect, or
  refactor `/games/[gameId]/preview` as part of the initial delivery.
- Render the 16:9 stage with semantic React components and Tailwind/CSS, using a
  fixed 1920x1080 design coordinate space that scales uniformly for browser
  display.
- Preserve the current narrative composition—left/right seat tracks, top title,
  central stage, subtitle band, and distinct speech/case-board responsibilities—
  but redesign the visual system for HTML rather than reproducing Pixi geometry
  pixel-for-pixel. Retain the existing investigative/mansion tone, background
  assets, and recognizable palette while using CSS layout, typography, depth,
  and responsive text flow deliberately.
- Treat adverse content as a design input: long player names, multi-line titles
  and details, missing avatars, dense rows, and dead-player states must have
  explicit layout behavior rather than corrective coordinate patches.
- Reuse the existing playback compilation and shot-engine domain logic where it
  is renderer-independent; do not duplicate event-to-scene rules.
- Make every visual state a pure function of playback data and an explicit
  timeline position (`timeMs` or frame number). Export correctness must not
  depend on wall-clock timers, `requestAnimationFrame`, CSS animation clocks, or
  live audio playback progress.
- Use the same HTML stage component for interactive preview and frame-based
  video rendering.
- Preserve current draft-focus behavior (`?focus=current`), public/director
  scene selection, system-voice duration adjustment, background images,
  avatars, subtitles, playback navigation, and keyboard controls unless a later
  product decision explicitly changes them.
- `preview_v2?focus=current` may include the current unconfirmed draft for
  director review, matching the existing editor-preview intent. Final export
  always compiles only confirmed active events; the v2 UI must clearly disclose
  when the visible draft is excluded from export. The first delivery does not
  offer a force-export-draft option.
- Fonts and image assets must be loaded before a frame is considered ready for
  capture.
- Video export must use deterministic frames and deterministic audio placement;
  it must not be implemented as real-time browser/tab recording.
- Define an explicit renderer-independent audio timeline. An audio cue identifies
  its source, start time, optional trim/duration, volume, and semantic kind.
  Existing system voices adapt into this timeline; future player TTS, music, and
  sound effects can add cues without changing the video renderer or export-job
  contract.
- Audio is optional. Missing or absent audio cues must produce a valid silent
  MP4 rather than fail the export. Available current system-voice assets should
  be composed at deterministic offsets, but generating new speech, music, or
  effects is not required in the first delivery.
- The first delivery must be an end-to-end vertical slice: HTML preview,
  deterministic frame rendering, deterministic system-voice composition,
  server-side video encoding, export progress, failure reporting, and final
  download. Export is not deferred to a later phase.
- Keep the old Pixi renderer and its recording path untouched until v2 has been
  verified and a separate removal decision is made.
- Keep the existing v1 iframe in the editor. Add a separate `Open Preview v2`
  entry in the editor preview area and a separate v2 link for each game on the
  home page. The v2 page owns its preview, export progress, and download UI.
  Existing `/preview` links retain their current meaning until a later migration
  decision.

## Constraints

- Existing game records and event schemas must remain compatible.
- Browser preview and exported output must use the same scene component tree and
  design tokens.
- The initial output target is a fixed-aspect video; responsive browser scaling
  must not change internal layout decisions.
- The initial export format is MP4 with H.264 video, AAC audio, 1920x1080
  dimensions, 30 FPS, and `yuv420p` pixel format.
- Each export job captures an immutable render-input snapshot when it is created;
  later edits to the game, event log, draft, or asset selection must not alter a
  job already queued or rendering.
- Planning must define export ownership, job lifecycle, output format, audio
  composition, failure behavior, and deployment assumptions before
  implementation begins.
- The first delivery targets the current local/single-host deployment model: the
  Next.js server has writable local storage, export artifacts live below
  `KIVA_DATA_DIR`, and at most one render executes at a time while later jobs
  wait in a local queue.
- Remotion is approved for this personal-use project. All Remotion packages must
  use one exact synchronized version; license fit must be revisited if usage
  later expands beyond the current individual context.
- Distributed workers, Redis, shared object storage, and multi-instance queue
  coordination are unnecessary for the first delivery. Export records must
  still make interrupted work diagnosable and allow the user to retry after a
  process restart.
- Export artifacts and job snapshots are retained indefinitely. The application
  does not automatically delete, rotate, expire, or cap completed exports; the
  operator manages disk usage directly in the export directory.

## Acceptance Criteria

- [ ] `/games/[gameId]/preview` continues to behave as before.
- [ ] `/games/[gameId]/preview_v2` loads the same game and compiled playback
  timeline without importing or mounting PixiJS.
- [ ] Existing editor/home v1 links remain, while separate v2 links open the new
  page without changing the embedded v1 iframe.
- [ ] All supported scene kinds (`phase`, `announcement`, `speech`, `vote`, and
  `resolution`) have deterministic HTML render states at an arbitrary `timeMs`.
- [ ] Seeking to the same timeline position repeatedly produces the same visual
  state after assets are ready.
- [ ] Preview and export consume the same stage component rather than parallel
  HTML implementations.
- [ ] Export creation snapshots confirmed active events and all required mutable
  assets; an unconfirmed draft may appear in `focus=current` preview but never
  enters the exported snapshot.
- [ ] A completed export has fixed dimensions and frame rate, contains the
  available intended system voice at deterministic scene offsets (or a valid
  silent audio result when no cues are available), and can be downloaded from
  the v2 workflow.
- [ ] Export UI exposes durable queued, preparing, rendering, completed, failed,
  canceled, and interrupted states with progress, cancel, retry, and download
  actions appropriate to each state.
- [ ] Two requested exports execute serially; a process restart makes unfinished
  jobs explicitly interrupted and retriable rather than silently losing them.
- [ ] Completed MP4 metadata verifies H.264, 1920x1080, 30 FPS, expected
  duration, `yuv420p`, and AAC when audio cues are present.
- [ ] Export job snapshots and successful files remain below
  `KIVA_DATA_DIR/exports` without automatic deletion or rotation.
- [ ] The audio-timeline contract is tested independently of Remotion rendering
  and accepts future cue kinds without changing existing playback items.
- [ ] Long names, long titles/details, missing avatars, eliminated players, and
  empty playback are covered by tests or explicit visual fixtures without
  overlap or clipping outside defined overflow rules.
- [ ] Type checking, automated tests, and a production build pass.

## Out of Scope

- Replacing or deleting the original `/preview` route.
- Changing game rules, event schemas, event presentation semantics, or LLM
  generation.
- Supporting arbitrary user-authored HTML/CSS as video input.
- Generating player-speech TTS, music, ambience, or new sound effects.
