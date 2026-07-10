# HTML Video Preview v2 — Technical Design

## 1. Decision Summary

Build a new HTML video pipeline beside the Pixi pipeline:

~~~text
GameRecord
  -> compilePublicPlayback()
  -> VideoCompositionInput (immutable JSON)
  -> KivaVideoComposition
       -> current frame -> timeMs
       -> existing shot-engine derivation
       -> HtmlPlaybackStage
       -> optional AudioCue sequences
  -> Remotion Player (interactive preview)
  -> Remotion Renderer (deterministic MP4 export)
~~~

The existing '/games/[gameId]/preview', 'PlaybackStage', Pixi renderer, and
browser MediaRecorder path remain intact. V2 has an independent route,
components, APIs, and export storage.

## 2. First-Principles Constraints

1. The reviewed frame and exported frame must come from the same React component
   tree. A second export-only renderer would recreate the current drift problem.
2. Video time is an input, not elapsed wall-clock time. Every animation value is
   derived from 'frame / fps' or 'timeMs'.
3. Export input is immutable. A render must not observe game edits or mutable
   assets halfway through.
4. HTML is responsible for layout. No element participates simultaneously in a
   CSS layout flow and a second manual positioning system unless it is an
   intentional visual overlay.
5. Audio is a timeline of optional cues. Video rendering does not know whether a
   cue came from a system prompt, future TTS, music, or an effect.
6. The first runtime is one writable host. Process-local serialization plus
   filesystem persistence is enough; distributed infrastructure is not.

## 3. Module Boundaries

### 3.1 Renderer-neutral composition contracts

Create 'src/components/preview-v2/composition/types.ts'.

~~~ts
type VideoSpec = {
  width: 1920;
  height: 1080;
  fps: 30;
  codec: "h264";
  audioCodec: "aac";
  pixelFormat: "yuv420p";
};

type CompositionAssets = {
  fontUrl: string;
  dayBackgroundUrl: string | null;
  nightBackgroundUrl: string | null;
  avatarUrls: Readonly<Record<string, string>>;
};

type AudioCue = {
  id: string;
  kind: "system-voice" | "player-voice" | "music" | "effect";
  src: string;
  startsAtMs: number;
  durationMs: number | null;
  trimStartMs: number;
  volume: number;
};

type VideoCompositionInput = {
  schemaVersion: 1;
  gameId: string;
  gameTitle: string;
  items: readonly PlaybackItem[];
  assets: CompositionAssets;
  audioCues: readonly AudioCue[];
};
~~~

These objects are JSON-serializable. They must not contain 'HTMLImageElement',
DOM nodes, repository instances, callbacks, or absolute filesystem paths.
Runtime validation owns the JSON boundary. Because the repository has no
validation library today, use small explicit decoders/type guards rather than
adding Zod only for this contract.

### 3.2 Shared frame composition

Create 'KivaVideoComposition', used unchanged by both '@remotion/player' and
'@remotion/renderer'.

- 'useCurrentFrame()' and 'useVideoConfig()' produce
  'timeMs = frame * 1000 / fps'.
- 'playbackIndexAtMs()' selects the active 'PlaybackItem'.
- Existing shot-engine functions derive the clock, active/highlighted players,
  and subtitle window. Renderer-only Pixi geometry and 'HTMLImageElement'
  assets are not copied into v2.
- 'HtmlPlaybackStage' receives a serializable frame view model and asset URLs.
- Audio cues become Remotion 'Sequence' + 'Html5Audio' nodes. An empty cue list
  is valid.

The composition duration is:

~~~text
max(1, ceil(playbackTotalDurationMs(items) * 30 / 1000))
~~~

No 'setTimeout', 'setInterval', 'requestAnimationFrame', time-based CSS
animation, or live 'HTMLAudioElement.currentTime' may determine exported state.

### 3.3 HTML stage

Create feature-scoped components under
'src/components/preview-v2/stage/':

- 'html-playback-stage.tsx': fixed 1920x1080 root and scene dispatch.
- 'stage-header.tsx': current scene title only.
- 'seat-track.tsx' and 'seat-card.tsx': six stable slots per side.
- 'subtitle-band.tsx': narrator avatar, identity, and deterministic narration.
- 'stage-copy.ts': exhaustive presenter/player attribution and semantic fallback
  narration for scenes without text.
- 'stage-background.tsx': day/night image and CSS overlays.
- 'stage-theme.ts': renderer-neutral design tokens.

The root uses a fixed grid:

~~~text
+-----------+-------------------------------------+-----------+
|           | Current scene title                 |           |
|           +-------------------------------------+           |
| Seats 1-6 | Empty visual stage                  | Seats 7-12|
| full      | Background atmosphere only          | full      |
| height    +-------------------------------------+ height    |
|           | Avatar | narrator + narration       |           |
+-----------+-------------------------------------+-----------+
~~~

Internal dimensions are always 1920x1080. The page scales the Player as a
single aspect-ratio box; media queries never change the internal composition.

Content rules:

- Seat name: at most two lines; deterministic font-size tier based on string
  class/length, not post-render coordinate correction.
- Role: one line with ellipsis.
- Main title: one line with deterministic truncation.
- The visual stage has no readable content. New visual concepts must mount in
  that reserved slot without changing the surrounding grid.
- Transcript narration is at most three lines per deterministic window.
- Speech uses the active player as narrator. Every non-speech kind uses the
  presenter, including private night actions and resolutions. Empty phase text
  uses explicit phase narration rather than falling back to the title.
- Missing avatar: initials/fallback mark in the same box geometry.
- Eliminated player: reduced saturation/contrast plus explicit status marker;
  never represented by opacity alone.
- Intentional overlays (texture, vignette, focus mark) are absolute and
  'pointer-events: none'; content layout remains Grid/Flex.

Transition values are pure functions of 'ShotClock.enterProgress' and
'exitProgress' and are applied with inline CSS variables/styles. Tailwind owns
static layout and appearance; it does not generate dynamic class names.

### 3.4 Preview page

Add 'src/app/games/[gameId]/preview_v2/page.tsx'.

The server component:

1. Loads the game record.
2. Applies existing 'focus=current' draft-preview behavior for display only.
3. Loads system-voice durations.
4. Builds serializable preview composition input with normal asset route URLs.
5. Renders a client 'PreviewV2Studio'.

'PreviewV2Studio' embeds Remotion 'Player' using 'KivaVideoComposition'. It
provides existing navigation semantics: play, pause, seek, previous/next scene,
reset, space/arrow shortcuts, progress, and time display. It also shows:

- 'Draft included in preview; export uses confirmed events only' when relevant.
- Current export queue/history for the game.
- Generate, cancel, retry, and download actions.

The Player's current frame is the only preview clock. Do not maintain an
independent React millisecond timer.

### 3.5 Remotion entry and styling

Create a dedicated Remotion entry under 'src/video/' that registers one
composition ID, 'KivaPlaybackV2'. The entry imports the same
'KivaVideoComposition' and the v2 Tailwind/CSS entry.

Use the official Remotion Tailwind v4 integration for the Remotion Webpack
bundle. All Remotion packages are pinned to exactly the same version, as
required by Remotion. The Next application continues using its existing
Tailwind setup.

Bundle the Remotion entry lazily once per server process and cache the serve URL.
Do not bundle once per job. Rendering uses 'selectComposition()' and
'renderMedia()' with:

- codec 'h264';
- audio codec 'aac';
- pixel format 'yuv420p';
- 1920x1080, 30 FPS;
- overwrite disabled;
- software encoding for reproducibility in the first release;
- bounded concurrency appropriate for one local workstation;
- 'onProgress' persisted with throttling;
- 'cancelSignal' for active cancellation;
- a silent AAC track only if compatibility testing requires it when no cues
  exist; otherwise a valid video-only MP4 is acceptable.

No FFmpeg command override is planned.

Keep renderer/bundler imports behind server-only modules and Node-runtime route
handlers. Adjust Next server package externalization only as required by the
real Next 16 production-build spike; never let renderer-native packages enter a
client bundle. The production build is part of Gate 3 because development-only
success is insufficient evidence for this boundary.

## 4. Immutable Export Snapshot

The POST endpoint never accepts authoritative playback items from the browser.
It accepts only the game ID/action and reloads the current game record.

At job creation:

1. Load confirmed active events only; ignore draft regardless of page query.
2. Compile playback with the same director audience and voice-duration policy.
3. Build the audio cue timeline.
4. Copy every required mutable asset into the job's 'assets/' directory:
   backgrounds, font, used avatars, and available audio.
5. Rewrite composition asset/cue sources to job-scoped HTTP asset URLs.
6. Write 'input.json' atomically, then 'job.json' with 'queued' status.

Copying assets is required for true snapshot semantics. Merely storing current
'/kivdb-assets/...' URLs would allow an overwritten avatar or voice file to
change a queued render.

Persist job asset references as relative API paths. Browser preview resolves
normal application asset paths against its page origin; the server renderer
expands job asset paths against a trusted 'KIVA_RENDER_ORIGIN' configuration
before passing input props to headless Chromium. Do not persist or trust an
arbitrary request 'Host' header as renderer authority. Startup diagnostics must
explain a missing/unreachable render origin.

Proposed storage:

~~~text
KIVA_DATA_DIR/
  exports/
    <gameId>/
      <jobId>/
        job.json
        input.json
        assets/
          font.ttf
          day-background.png
          night-background.png
          avatars/...
          audio/...
        output.mp4
        render.log
~~~

All directories are retained indefinitely.

## 5. Export Job Model

'src/server/video-export/' owns the job contract, repository, snapshot builder,
queue, and Remotion adapter.

~~~ts
type ExportJobStatus =
  | "queued"
  | "preparing"
  | "rendering"
  | "completed"
  | "failed"
  | "canceled"
  | "interrupted";
~~~

A job record contains:

- job ID, game ID, schema version;
- created/started/completed timestamps;
- status and progress from 0 to 1;
- current stage: preparing, rendering, encoding, muxing;
- output metadata when completed;
- typed failure code plus sanitized message;
- retry source job ID when applicable;
- warning list, including unavailable optional audio.

State transitions are centralized in one reducer/transition function and written
atomically through temp-file + rename. Illegal transitions fail loudly.

~~~text
queued -> preparing -> rendering -> completed
   |          |            |
   +----------+------------+-> failed
   +----------+------------+-> canceled

queued/preparing/rendering found after restart -> interrupted
interrupted/failed/canceled -> retry creates a new queued job from the same
immutable snapshot
~~~

Retry creates a new job directory linked to the source job; it does not mutate
history or rebuild from the latest game state. Creating a fresh export is the
operation that snapshots the latest confirmed events.

## 6. Queue and Process Lifecycle

Use a process-global singleton queue to survive Next.js module reloads inside one
process. The queue:

- runs exactly one 'renderMedia()' call at a time;
- returns from POST immediately after durable enqueue;
- updates progress at a throttled interval to avoid excessive filesystem writes;
- holds the active Remotion cancel token;
- starts the next queued job in 'finally';
- catches all job errors so one failure cannot stop the queue.

On first repository/queue access after process start:

- scan persisted jobs;
- mark stale 'preparing' or 'rendering' jobs 'interrupted';
- also mark old 'queued' jobs interrupted rather than silently rendering work
  the operator did not explicitly resume;
- expose Retry in the UI.

This is intentionally not safe for multiple Next.js instances. The constraint is
documented and enforced operationally rather than hidden behind a false lock.

## 7. HTTP Boundaries

Proposed route handlers:

~~~text
POST /api/games/:gameId/preview-v2/exports
GET  /api/games/:gameId/preview-v2/exports
GET  /api/preview-v2/exports/:jobId
POST /api/preview-v2/exports/:jobId/cancel
POST /api/preview-v2/exports/:jobId/retry
GET  /api/preview-v2/exports/:jobId/download
GET  /api/preview-v2/exports/:jobId/assets/:path...
~~~

- POST create returns '202 Accepted' and the durable job projection.
- The client polls the game export list every second while any job is active and
  backs off/stops when all are terminal. SSE is unnecessary for a single host.
- Download is available only for completed jobs and uses an attachment filename
  containing game ID and creation timestamp.
- Asset and download handlers resolve paths from validated job metadata; no raw
  filesystem path comes from a request.
- API projections never expose internal absolute paths or raw stack traces.
- Export handlers explicitly use the Node.js runtime; Edge/serverless execution
  is outside the supported deployment model.

## 8. Audio Extension Framework

'buildAudioTimeline(items, resolver)' is a pure module.

For the first release, the resolver adapts
'systemVoiceSourceForScene(scene)' into optional 'system-voice' cues at
'scene.startsAtMs'. Missing files produce a warning and no cue.

Future additions require only new resolvers:

- player TTS: one 'player-voice' cue with generated source/duration;
- background music: long 'music' cues with volume envelopes;
- effects: event-relative 'effect' cues.

The composition maps every cue kind through the same timing primitive. Playback
duration policy remains owned by 'compilePublicPlayback()'; adding TTS later may
supply measured scene durations before timeline starts are finalized.

## 9. Failure Semantics

Fail the job for:

- missing/corrupt immutable input snapshot;
- missing required font/background policy asset;
- invalid composition input;
- Chromium or Remotion launch failure;
- frame render, encode, or mux failure;
- output verification failure.

Do not fail the job for:

- no audio cues;
- a missing optional system voice;
- a missing avatar (use visual fallback).

After 'renderMedia()', verify with 'ffprobe'/Remotion metadata:

- container is readable;
- width 1920, height 1080;
- frame rate 30;
- H.264 video stream exists;
- duration is within one frame of the composition;
- AAC exists when audio cues were present.

Write to a temporary output filename and rename to 'output.mp4' only after
verification. Failed/canceled partial files are never advertised as downloadable
output.

## 10. Compatibility and Rollback

- V1 code and route stay available and remain the default embedded preview.
- V2 adds separate editor/home links.
- Game/event storage schemas do not change.
- Export jobs use their own versioned JSON schema below 'exports/'.
- Rollback is removing v2 links/routes and dependencies; v1 continues to work.
- Existing exported files remain ordinary MP4 files even if v2 is later removed.

## 11. Testing Strategy

Pure unit tests:

- composition input decoder and version rejection;
- frame/time conversion and end-frame rounding;
- audio cue creation, missing audio, ordering, trim, and overlap;
- export state transition matrix;
- snapshot uses confirmed events and excludes draft;
- retry preserves snapshot and links source job;
- path validation and atomic repository round-trip.

Component tests:

- every scene kind renders deterministic semantic markup;
- long-name/title/detail fixtures obey declared truncation/overflow rules;
- missing avatar and dead-player treatment;
- draft exclusion notice and empty playback behavior;
- v2 route/component tree contains no Pixi import or canvas renderer.

Integration tests:

- create -> queued -> completed lifecycle with a fake renderer;
- queue serializes two jobs;
- cancel and retry;
- restart reconciliation to 'interrupted';
- asset/download route authorization by job metadata.

Render smoke test:

Render a short fixture through the real Remotion/Chromium/FFmpeg pipeline, then
probe the MP4 for dimensions, FPS, codec, duration, and optional audio. Keep this
as an explicit integration command rather than making every unit-test run launch
Chromium.

Visual QA:

Capture fixed frames for day/night and all scene kinds at enter/middle/exit
positions, including adverse-content fixtures. Review at native 1920x1080 and
scaled browser size. Validate that preview Player and rendered still match.

## 12. Operational Notes

- Local requirements: supported Node version, writable 'KIVA_DATA_DIR', enough
  disk for permanent snapshots/MP4 files, and a Remotion-supported Chromium and
  compositor/FFmpeg runtime.
- This machine currently has Node 24 and FFmpeg/FFprobe 8.1.1 with libx264, but
  startup diagnostics must report missing production dependencies explicitly.
- Remotion is approved for the current individual-use context. Package versions
  must be exact and synchronized, and license fit must be reconsidered if the
  project later becomes a larger-team or automated commercial service.
- Export duration and disk usage are displayed before enqueue where calculable.

## 13. Rejected Alternatives

### Real-time tab/DOM recording

Rejected because it requires wall-clock playback, can drop frames, needs capture
permission, and cannot guarantee deterministic output.

### DOM-to-canvas on every frame

Rejected because CSS/font/media fidelity and performance are weaker than
Chromium's own frame renderer.

### Separate Playwright screenshot pipeline

Viable but rejected for the first design because Remotion already supplies the
Player/renderer time model, frame rendering, cancellation, progress, audio
composition, and encoding integration. Building all of those directly would
increase project-owned infrastructure.

### Replace v1 immediately

Rejected because side-by-side comparison is the safest way to validate the new
visual and export pipeline.
