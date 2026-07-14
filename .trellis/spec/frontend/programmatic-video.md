# Programmatic Video Preview and Export

## Scenario: Immutable Game Script Presentation

### 1. Scope / Trigger

- Trigger: a New Game script changes the game shell, Preview, or exported MP4.

### 2. Signatures

- `Game.script: GameScriptSnapshot`
- `createCompositionInput({script, ...}): VideoCompositionInput`
- `VideoCompositionInput.schemaVersion: 4`
- `stagePaletteForPhase(phase, script.presentation.styleKey): StagePalette`

### 3. Contracts

- New games resolve one enabled script and deep-snapshot its narrative and presentation fields. Existing games never reload current library styling by `scriptSourceId`.
- Preview and export receive the same snapshot. Export copies allow-listed Actor, Presenter, and Script assets into the immutable job asset directory and rewrites their URLs once.
- `midnight_archive_v1` is the sole supported style identifier. A stored Game without its complete script snapshot and a composition outside schema v4 are rejected rather than normalized.
- Versioned script image filenames are immutable. Revised art requires a new filename and new snapshot.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|---|---|
| New Game omits/unknown/disabled script ID | Reject creation; do not save a partial game. |
| Stored Game lacks `script` or contains extra/old presentation keys | Reject the GameRecord. |
| Snapshot has an unknown style or unsafe asset path | Reject the snapshot as invalid. |
| Export asset is outside an allow-listed internal prefix | Reject export snapshot creation. |
| Composition schema is not v4 or omits an explicit Playback field | Reject the composition input. |

### 5. Good / Base / Bad Cases

- Good: a `midnight_archive_v1` game keeps its original colors and backgrounds after the script library is edited.
- Base: a complete current Game uses its snapshotted script for Preview and export.
- Bad: Preview fills a missing script, accepts a Preview background path, or looks up `scripts.json` during render.

### 6. Tests Required

- Unit: script definition validation, deep snapshot isolation, single-style dispatch, schema-v3 rejection, and exact Playback-item validation.
- Repository: a new Game round-trips its full script snapshot; incomplete or old snapshots are rejected without rewriting disk.
- Export: script backgrounds and both avatar prefix families are copied and rewritten.
- Browser/render: the selected script appears in New Game, Editor, Preview, and a 1920x1080 MP4 frame without console errors.

### 7. Wrong vs Correct

Wrong: store only `scriptId` and reload mutable presentation data for Preview/export.

Correct: save `GameScriptSnapshot` on Game creation and pass that same value through both rendering paths.

## 1. Scope / Trigger

Use this contract when adding or changing an HTML playback preview, Remotion
composition, audio cue, export job, or video-export API. It prevents preview and
MP4 output from becoming two independent renderers with different layout or
timing behavior.

## 2. Signatures

- Route: `GET /games/:gameId/preview?focus=current`
- Create/list: `POST|GET /api/games/:gameId/preview-v2/exports`
- Status: `GET /api/preview-v2/exports/:jobId`
- Actions: `POST /api/preview-v2/exports/:jobId/{cancel|retry}`
- Files: `GET /api/preview-v2/exports/:jobId/{download|assets/*}`
- Player voice jobs: `POST|GET /api/games/:gameId/voice-jobs`
- Voice job actions: `POST /api/games/:gameId/voice-jobs/:jobId/{cancel|retry}`
- Voice assets: `GET /api/games/:gameId/voice/:eventId`
- Presenter clips: `GET /api/presenters/:presenterId/voice/:file`
- Composition: `KivaVideoComposition(input: VideoCompositionInput)`
- Timeline: `AudioCue {kind, src, startsAtMs, durationMs, trimStartMs, volume}`
- Scene navigation: `sceneStartFrame(startsAtMs, fps) -> integer frame`
- Frame copy: `copyCurrentFrame(playerElement, dependencies) -> Promise<void>`
- Stage palette: `stagePaletteForPhase(day|night) -> StagePalette`
- Identity palette: `roleIdentityTone(roleName, day|night) -> tone`
- Transcript identity: `transcriptSpeakerIdentity(presentation) -> identity`
- Transcript mentions:
  `transcriptTextSegments(text, speakerKind, scene.players) -> segments[]`
- Library route: `GET /library?tab=presenters&id=:presenterId`
- Library write: `savePresenterAction(FormData) -> presenters.json`

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
- A configured day/night background renders at source opacity, saturation, and
  color using only `object-cover`. Do not add darkening gradients, vignettes,
  scanlines, or venetian-blind textures above it. The fallback gradient is used
  only when the selected background asset is absent.
- Stage copy has one semantic owner: the header always owns `gameTitle` as its
  primary line and displays `scene.title` only as the current-stage subtitle;
  do not add an English eyebrow or decorative product label above it. The
  game title is horizontally centered and uses enough line height and vertical
  padding to preserve complete CJK glyphs.
  The visual-stage slot owns structured event actions and outcomes from
  `PlaybackItem.stage`; the transcript card owns narrator identity and narration.
  For player speech, the visual-stage slot also shows the complete
  `PlaybackItem.text`, while the transcript card continues showing the current
  timed subtitle window.
  Speech uses the active player as narrator.
  `phase`, `announcement`, `vote`, and `resolution` use the presenter, including
  director-only night actions.
- Presenter copy has one data owner: `kivdb/presenters.json` stores reusable
  Presenter Definitions. Each definition owns an exhaustive independent script
  and voice map. Every Game selects exactly one enabled definition and stores a
  complete `Game.presenter` snapshot; preview and export must consume that
  snapshot, never reload current library wording for an existing game.
- Presenter Definitions are first-class Library objects. The Presenter editor
  keeps stable `id` immutable and may change `name`, `avatar`, `enabled`, and
  every semantic template.
  Voice is built separately into a versioned presenter manifest; template
  edits do not silently reuse stale per-line audio mappings.
- The Presenter editor loads the selected manifest and renders one audio
  preview control for every semantic line. Single-seat variants expose a 1–12
  selector; multi-clip examples play their declared clips serially with the
  canonical 80ms gap. Starting one preview stops the previous preview, and
  preview-only selectors are excluded from the dirty-form guard. Missing
  manifests disable playback without blocking copy editing.
- A Presenter Definition is not a game Actor. It has no seat, Rule Role,
  faction, action, vote, private knowledge, or win-condition participation.
- Presenter templates use stable semantic keys and validated named
  placeholders. Event-to-key selection and typed placeholder values belong to
  the core presenter resolver; rendering components must not parse event
  payloads or choose copy variants.
- Resolved `PlaybackItem` data explicitly carries presenter identity/avatar,
  transcript ownership, transcript text, and the presenter cue. Player-authored
  speech remains player-owned while its player-addressed prompt remains a
  presenter cue.
- Presenter voice files are selected from the built manifest clip plan, never
  from rendered Chinese titles or prose. Dynamic player variables expand to
  fixed seat/role clips. The asset route serves only files declared by that
  manifest.
- Every MP3 route consumed by Remotion (`presenters`, Game player voice, and
  export snapshot assets) supports a single HTTP byte range: valid Range ->
  `206` with `Accept-Ranges`, `Content-Range`, and exact `Content-Length`;
  unsatisfiable Range -> `416` with `bytes */<size>`. A full GET still advertises
  `Accept-Ranges: bytes`.
- Presenter MP3 filenames contain a SHA-256 fingerprint of the complete voice
  profile and synthesized text. Never replace bytes behind the same immutable
  URL; profile, speed, or copy changes must produce a new filename, update the
  manifest atomically, then remove obsolete files.
- Edge MVP voice identities are centralized constants: Presenter
  `zh-CN-YunjianNeural`, male Player `zh-CN-YunxiNeural`, and female Player
  `zh-CN-XiaoxiaoNeural`; every profile uses `rate: "+20%"` (1.2x).
  Every template with exactly one scalar seat variable resolves to one of 12
  complete utterance clips and must never concatenate a seat-number clip.
  Multi-player/list templates retain reusable clip composition. Presenter clip
  reuse requires the complete manifest
  profile—not only clip text and filename—to match the current definition.
- Player voice artifacts are immutable and keyed by confirmed `eventId` inside
  the Game directory. Generation is a persisted server job; retry skips
  successful artifacts. Preview may use a silent fallback, but export requires
  every player-authored playback item to have an artifact.
- Player subtitle cues come from real TTS word boundaries aggregated into
  sentence windows. Never switch subtitle windows by `scene progress * count`.
- The canonical speech rhythm is presenter clips, 250 ms handoff silence,
  player audio, and 350 ms tail silence. Presenter clip gaps are 80 ms, and
  voice segments never overlap.
- Never use `scene.title` as transcript fallback. Scenes with empty narration
  use an exhaustive semantic projection: explicit copy for every phase, then
  event details or a kind-specific neutral message for other scene kinds.
- Header, visual-stage slot, and transcript occupy equal-width grid regions
  in the center column. The stage renders a card treatment only when the current
  `PlaybackItem.stage` is non-null or the current item is player speech with
  non-empty text; ordinary narration retains an empty slot. Each six-card seat
  track spans
  their combined full height; its top aligns with the header and its bottom
  aligns with the transcript.
- Seat cards are designed for a 1920x1080 composition viewed after phone-scale
  reduction. Player name, role, and seat number remain the primary text; avoid
  microcopy that becomes illegible when the video is fitted to a phone. The
  large seat number contains digits only, occupies an avatar-height block on
  the outer edge, and the avatar sits on the inner edge. Left and right tracks
  are strict mirrors: avatar, text alignment, seat number, state treatment,
  gradients, and emphasis accents all reverse across the center axis.
  Every mirrored card region must declare the same explicit CSS Grid row; do
  not rely on auto-placement after moving the first region to the last column,
  because later regions will flow into clipped implicit rows.
  The track remains full height; its six fixed-height rows use space-between so
  the first and last cards align with the track edges instead of stretching or
  centering the group. Name and role are enlarged and centered. Dead state must
  reduce the whole card hierarchy (surface, border, accent, avatar, number,
  name, and role opacity) while retaining a recognizable role hue. Active
  speech uses a stronger border plus a wider inner-edge bar and directional
  gold field; both treatments mirror across the center axis and stay behind
  readable content.
- Preview v2 identity colors have one owner in `stage/identity-palette.ts`.
  The approved C smoke-glass identity values are night: wolf `#FF767B`,
  villager `#D0D5D1`, seer `#76E2B6`, witch `#CBA4F4`, guard `#80C8FB`,
  hunter `#E5BD6E`, presenter `#F0D084`; and day: wolf `#FF696E`, villager
  `#E0E1DB`, seer `#59DFA4`, witch `#CF93F5`, guard `#66C1FA`, hunter
  `#E8B44C`, presenter `#F4C45C`. Structural surface, border, shadow, text,
  number, rule, and active-state values live in `stage/stage-palette.ts`.
  Presenter and player names use the phase's shared warm white. Dead state may
  reduce the entire hierarchy, but the role label retains a subdued semantic hue.
- The transcript band reserves one avatar slot, one compact speaker row, and one
  narration block. The speaker row shows the role or gold `主理人` directly and
  never renders an `身份` prefix. Player-owned speech exposes the active player's
  role; presenter-owned narration exposes only `主理人`. Avatar border/glow
  follows that current speaker identity. The narration block's top divider must
  stay directly below the speaker row; never vertically center the narration
  block, because short presenter copy would push the divider downward. For
  presenter-owned narration only,
  `stage-copy.ts` safely segments text using typed `scene.players`: full
  seat/name labels, player names, and seat labels receive the mentioned player's
  role color. Player-authored speech is free text and must return one neutral
  segment without mention recognition.
- Video profile is 1920x1080, 30 FPS, H.264, `yuv420p`, with AAC when cues exist.
- Export creation reloads authoritative game state and snapshots confirmed
  active events plus all mutable assets. A preview draft is never exported.
- Jobs are stored under `KIVA_DATA_DIR/exports/<gameId>/<jobId>` and retained.
- `KIVA_RENDER_ORIGIN` optionally overrides the trusted render origin; default
  is `http://127.0.0.1:9090`, matching the fixed `pnpm dev` port.
- Exactly one process-global renderer runs at a time. Later jobs remain queued.
- Every finite audio cue must set `durationMs`. A Remotion `Sequence` without a
  duration remains mounted until the composition ends, so completed
  `<Html5Audio>` elements accumulate and can exhaust Player shared audio tags.
- Preview previous/next controls navigate adjacent `PlaybackItem` records: a
  speech record means the adjacent player, and any other record means the
  adjacent game event. Scene starts may be arbitrary milliseconds after voice
  durations are applied. Navigation must seek with `sceneStartFrame()`, which
  rounds up so the selected frame is inside the target record. Generic audio
  timing keeps `millisecondsToFrame()` and its floor semantics; do not change
  that shared conversion to repair scene navigation.
- Preview exposes separate playback and data controls. `回到开头` seeks only to
  frame zero and never reloads or mutates the Game. `刷新游戏数据` calls the
  Next router refresh so the dynamic Preview route reloads the current Game and
  rebuilds its composition from authoritative repository state.
- The Preview v2 `复制当前帧` control rasterizes only the Player container and
  writes one `image/png` ClipboardItem at 1920×1080. It does not seek, pause,
  resume, or otherwise mutate playback. Missing Clipboard API support, a null
  rasterized Blob, or clipboard rejection produces visible failure feedback and
  leaves the current frame unchanged.

## 4. Validation & Error Matrix

- Unknown game -> `404 game_not_found`.
- No confirmed playback -> `409 empty_playback`.
- Unknown job -> `404 job_not_found`.
- Retry of a non-retriable job -> `409 job_not_retriable`.
- Download before completion -> `409 output_not_ready`.
- Invalid identifiers or asset path segments -> `400` or a sanitized server
  error; filesystem paths and stacks are never returned.
- Incomplete presenter templates, invalid placeholders, unsafe MP3 names, or
  incomplete seat maps submitted from Library -> validation error and no
  partial `presenters.json` write.
- Output missing the required video profile, or AAC when cues exist -> job
  becomes `failed`; the partial file is removed.
- Scene start falls between frames -> previous/next seeks to the first frame at
  or after the boundary; it must not remain in or skip to an earlier record.
- Unknown/custom role label -> neutral identity tone; rendering must not fail.
- Presenter scene -> speaker identity is always only `主理人`, regardless of
  highlighted players.
- Player-authored speech -> one neutral segment, even when it contains a known
  player name or seat label.
- Text that contains no recognized player mention -> neutral narration text.
- A larger unrelated number such as `21号` -> must not partially color `1号`.
- Clipboard or ClipboardItem unavailable -> copy-frame reports failure without
  changing playback.
- Rasterizer returns null -> copy-frame reports failure and performs no write.

## 5. Good / Base / Bad Cases

- Good: confirmed events and available voices produce a downloadable MP4 with
  H.264 video and AAC audio.
- Base: no audio cues produces a valid video-only MP4.
- Bad: `focus=current` contains only a draft; preview may render it, but the
  export button is disabled and the server still rejects an attempted export.
- Good: editing a presenter in Library round-trips all 44 templates and voice
  mappings while existing games retain their previous snapshot.
- Base: a Library voice filename is blank; it persists as null and the scene is
  silent.
- Bad: a Library template drops a required placeholder; the collection is not
  written.
- Good: Next from a player speech seeks to the following player speech; at a
  phase/action boundary it seeks to the following event.
- Base: the target starts exactly on a frame boundary; scene navigation lands
  on that frame.
- Bad: a voice duration creates a `23513ms` boundary and floor conversion lands
  on `23500ms`, leaving the scene index on the previous record.
- Good: the six built-in role labels remain visibly distinct on the dark stage,
  and presenter identity reads as gold beside a neutral speaker name.
- Base: an unknown custom role renders in neutral stone instead of borrowing a
  built-in role color.
- Bad: a dead-player card remains as prominent as a living player, or full-card
  grayscale removes the semantic role distinction.
- Good: `4号林夏查验12号沈岚` keeps identity `主理人`, while the two player
  mentions in presenter narration are colored by 林夏 and 沈岚's respective
  roles.
- Base: player-authored speech mentioning 林夏 stays entirely neutral, and a
  presenter scene shows only `主理人` beside the speaker name.
- Bad: highlighted-player roles are listed as identities, the identity metadata
  consumes a separate row, or role words are inferred from prose.

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
  overlay is a later sibling of the content grid. With a configured background,
  assert the image class contains only full-size `object-cover` layout and no
  opacity, saturation, gradient, vignette, or repeating-linear treatment.
- Copy ownership: cover `phase`, `announcement`, `speech`, `vote`, and
  `resolution`; only speech resolves to a player, every other kind resolves to
  the presenter, the header displays `gameTitle` for every scene, and empty
  narration never falls back to `scene.title`.
- Presenter definitions: validate unique IDs, complete semantic key sets, and
  exact placeholder contracts. Presenter manifests validate declared clip
  files, durations, fixed seats 1–12, and runtime clip plans.
- Game presenter selection: both saved-Lineup and custom-Lineup creation resolve
  the sole enabled Presenter; the created snapshot remains unchanged after its library
  definition is edited, disabled, or removed.
- Library presenter configuration: assert the tab/list/editor renders the
  definition; form parsing reconstructs all semantic keys; identity and
  templates survive repository round-trip; invalid catalogs do not replace the
  stored collection.
- Presenter resolution: cover every event and result variant, player speech
  ownership, empty-speech fallback, pending silence, ready prompt audio, and
  missing optional files.
- Header geometry: the game title remains horizontally centered, and CJK glyph
  bounds are not clipped at the top or bottom at native 1920x1080 resolution;
  rendered markup contains no secondary English eyebrow.
- Grid geometry: header, visual-stage slot, and transcript have identical
  x and width. A non-speech scene without `PlaybackItem.stage` renders the bare
  empty slot; player speech renders its complete text without line clamping,
  while the transcript retains the current subtitle window.
  A structured stage scene may add deterministic inline animation styles. Both
  seat tracks align to the header top and transcript bottom and
  each contains exactly six equal-height slots.
- Seat-card geometry: assert the left/right grid columns, avatar position, text
  alignment, outer seat-number position, inner avatar position, and emphasis
  edge are mirrored. Assert a full-height track with compact fixed rows and
  space-between distribution, centered enlarged name/role, subdued dead state,
  and mirrored active-speaker field. At
  phone-scale playback, player name, role, seat number, and state treatment
  remain distinguishable.
- Identity palette: assert all six built-in role names, descriptive aliases
  such as `白狼王` and `村民`, unknown fallback, both presenter golds, both
  phase palettes, and actual shared-stage markup colors. A dead role label must
  retain its semantic tone.
- Transcript identity: assert player speech projects its role, every presenter
  scene projects only `主理人`, and rendered markup contains no `身份` prefix.
  Presenter and player speaker names must share the same neutral tone and layout.
  The narration block must not use self-centering; its divider stays at the same
  vertical position for short presenter copy and longer player speech.
  Presenter mention segmentation must color full seat/name, name-only, and
  seat-only references while rejecting partial larger-number matches such as
  `21号`; player-authored speech must remain one neutral segment.
- Audio lifecycle regression: seek across the complete Player timeline and
  assert that the shared `<Html5Audio>` tag limit is never exceeded.
- Scene navigation: include a non-frame-aligned boundary such as `23513ms` at
  30 FPS, assert `sceneStartFrame()` returns frame 706, and assert converting
  that frame back to milliseconds resolves to the requested `PlaybackItem`.
- Frame copy: assert native 1920×1080 raster options, PNG ClipboardItem payload,
  null-Blob rejection, and visible shell feedback for copying/success/failure.
- Integration: a second request stays queued while one render is active;
  queued cancel, retry lineage, download, and restart reconciliation work.

## 7. Wrong vs Correct

Wrong: update visual state from `setTimeout`, CSS animation time, or live audio
progress, parse Chinese presenter copy into a visual action, then implement a
separate export-only layout.

Correct: derive `timeMs` from the Remotion frame, project typed GameEvent payloads
into `PlaybackItem.stage`, create a renderer-neutral view model, and render the
same `HtmlPlaybackStage` in Player and Renderer. Fit the
outer Player with container-query units rather than adding responsive rules to
the composition. Use semantic application tokens outside the Player instead of
duplicating its hard-coded cinematic colors. Bound each system-voice cue to its
playback scene instead of increasing `numberOfSharedAudioTags` to hide
unbounded mounts. Keep configured backgrounds unfiltered and free of texture
overlays. Do not put temporary event UI into the reserved
visual stage or let transcript content independently fall back to
`scene.title`; keep the header fixed to `gameTitle`, and resolve narrator and
narration once by semantic ownership.
Do not hard-code a global presenter identity, presenter prose, title-to-audio
map, or static system-voice allowlist in application source. Resolve the
selected `Game.presenter` snapshot once during playback compilation and carry
the result into the immutable composition/export snapshot.

Wrong: let the Library form accept arbitrary semantic keys or editable
placeholder declarations, then cast the payload to `PresenterDefinition`.

Correct: enumerate fields from `PRESENTER_LINE_VARIABLES`, rebuild the entire
catalog at the server boundary, and let `saveAll()` validate it atomically.

Wrong: use floor-based `millisecondsToFrame(scene.startsAtMs, fps)` for
previous/next scene controls; arbitrary MP3 durations can place that frame
before the requested scene.

Correct: use the scene-specific ceil-based `sceneStartFrame()` for navigation,
while retaining floor conversion for audio sequence offsets and trim values.

Wrong: scatter role hex values through seat-card branches or grayscale the
whole dead-player card.

Correct: resolve every role label through `roleIdentityTone()`, use the shared
presenter tone for presenter identity while keeping every speaker name neutral,
and mute only the dead avatar.

Wrong: show an `身份` prefix, list highlighted-player roles in the identity area,
or scan player-authored free text for mentions.

Correct: show exactly one current-speaker identity value beside the name, then
color typed player mentions only when the text belongs to the presenter.
Remotion's Webpack disk cache stays disabled
because the application already caches the completed bundle per process; this
avoids corrupted cache packs during local
concurrent builds.

## Scenario: Export Operations Validate Only Their Current Inputs

### 1. Scope / Trigger

- Trigger: export metadata is listed/reconciled, or a composition snapshot is
  loaded for rendering, retry, download, or asset serving.

### 2. Signatures

- `ExportRepository.list(gameId): Promise<readonly ExportJob[]>` reads and
  validates `job.json` only.
- `ExportRepository.get(gameId, jobId): Promise<ExportJobSnapshot | null>`
  reads both `job.json` and `input.json` and requires the current composition
  schema.

### 3. Contracts

- Export history and restart reconciliation depend only on job metadata.
- Rendering, retrying, and serving snapshot-owned assets require a fully valid
  current-version `input.json`.
- No operation migrates, fills, or reinterprets an invalid composition snapshot.

### 4. Validation & Error Matrix

- Missing `job.json` -> repository entry is absent.
- Invalid `job.json` -> reject as `Invalid export job record`.
- Missing or invalid `input.json` during `list()` -> do not read it; return valid current job metadata.
- Missing `input.json` during `get()` -> entry is absent; malformed, incomplete,
  extra-key, or non-v4 input -> reject it.

### 5. Good / Base / Bad Cases

- Good: current job and composition schemas list and load normally.
- Base: a valid job metadata record can be listed while its render input is
  unavailable; no composition data is consumed by that operation.
- Bad: `list()` partially decodes composition input or `get()` fills missing
  composition fields.

### 6. Tests Required

- Repository regression: create a valid job, make only `input.json` unavailable,
  and assert `list()` returns the current job metadata unchanged.
- Decoder regression: direct snapshot loading rejects malformed, incomplete,
  extra-key, and non-v4 input.

### 7. Wrong vs Correct

Wrong: implement metadata listing by calling the full composition decoder for
every job, or turn a composition decoder failure into field injection.

Correct: decode the smallest contract needed by each operation—job metadata
for listing/reconciliation, full composition only for snapshot consumers.

## Scenario: Isolated Voice and Video Workers

### 1. Scope / Trigger

- Trigger: Edge TTS, Remotion, or FFmpeg work is initiated from Preview while
  the Next.js server must remain responsive.

### 2. Signatures

- `pnpm worker:voice` runs `VoiceJobService.processQueuedOnce()` in a loop.
- `pnpm worker:video` runs `VideoExportService.processQueuedOnce()` in a loop.
- `pnpm dev:all` starts Web, Voice Worker, and Video Worker as separate processes.

### 3. Contracts

- API-owned service singletons use `executeInline: false`; POST persists a
  `queued` job and returns HTTP 202 without executing it.
- Each Worker processes the oldest queued job serially and owns a per-kind PID
  lock under `<KIVA_DATA_DIR>/workers/`.
- Restart keeps `queued` jobs queued. Voice `preparing/generating` and video
  `preparing/rendering` jobs become `interrupted`.
- Cancellation is a persisted state transition observed by the Worker; an
  in-memory cancellation flag is only a same-process optimization.
- Voice status polling updates panel-local state. It calls `router.refresh()`
  only when the same job's completed/skipped item count increases; queued
  and executing statuses poll every 30 seconds and must not cause repeated
  Preview RSC requests or Editor iframe remounts.
- Active video-export statuses also poll every 30 seconds; manual refresh
  remains immediate.
- A Game lock contains PID/token ownership and recovers dead-process or old
  ownerless lock directories. Network synthesis runs outside this lock; only
  the final re-read, immutable-artifact check, publish, and record save belong
  inside the critical section.

### 4. Validation & Error Matrix

- Worker absent -> task remains `queued`; Web remains available.
- Second live Worker of the same kind -> startup fails with the owning PID.
- Stale PID lock -> new Worker replaces it.
- Worker exits during active work -> next initialization marks the job
  `interrupted`; user retries manually.
- Worker dies while holding a Game lock -> the next lock acquisition verifies
  the PID, removes the stale lock, and proceeds.

### 5. Good / Base / Bad Cases

- Good: Web, Voice Worker, and Video Worker run as three processes; rendering
  does not occupy the Next.js event loop.
- Base: only Web runs; users can create and inspect queued tasks.
- Bad: an API route calls Edge TTS or `renderVideo()` after returning 202.

### 6. Tests Required

- Voice integration: persist-only service creates `queued`; one Worker pass
  completes it; a second pass reports no work.
- Lock regression: repository instances serialize live owners, recover a dead
  PID owner, and voice synthesis can independently acquire the Game lock while
  the Edge request is running.
- Video persistence: API-mode initialization leaves a queued export unchanged.
- Full suite must prove existing inline test adapters remain deterministic.

### 7. Wrong vs Correct

Wrong: use promises, timers, or an in-memory queue inside Next.js and call it
background work.

Correct: persist the command in Next.js and execute CPU/network-heavy work in
a separately started process with filesystem-visible state transitions.

Wrong: hold the Game lock around an external TTS request. This makes Editor
state planning wait for network latency and turns a crashed Worker into a
permanent page hang when the lock has no owner metadata.

Correct: synthesize to a unique temporary file without the Game lock, then
acquire the lock briefly, re-read the Game, skip an existing immutable
artifact, and atomically publish the new artifact.

## Scenario: Structured Event Visuals on the Preview Stage

### 1. Scope / Trigger

- Trigger: a confirmed action or resolution must render in the center stage in
  both director Preview and exported video.

### 2. Signatures

- `PlaybackItem.stage: StagePresentation | null`.
- `stagePresentationForEvent(event: GameEvent): StagePresentation | null`.
- `StageEventVisual({shot, assets, palette, phase})` renders the shared stage.
- `StagePresentation.kind` is one of `action`, `night_result`, `vote_result`,
  or `game_result`.

### 3. Contracts

- Audience filtering runs before stage projection. Renderer code never decides
  whether a private GameEvent is safe to show.
- Director playback includes `night_resolved`; public playback excludes it and
  receives only the later public `death_announced` result.
- Sealed wolf ballots remain excluded. `wolf_vote_resolved` projects one team
  action with a synthetic wolves actor.
- `exile_resolved.voteTable` is deterministically aggregated into candidate
  totals and an abstention count. The stage does not render every ballot line.
- Game result renders only winner and reason; player identities remain owned by
  the persistent seat tracks in director Preview.
- Stage animation is a pure function of `ShotClock.sceneMs/durationMs`. No CSS
  keyframes, timers, randomness, DOM measurement, or live audio state is used.
- Every Playback item carries `stage` explicitly. `stage: null` is the current
  representation for no structured event visual; player speech still renders
  its complete `text`, while other null-stage items render an empty center slot.

### 4. Validation & Error Matrix

- Missing `stage` -> reject composition input; `stage: null` is valid, with
  complete player speech in the center and an empty center slot for other items.
- Unknown `stage.kind`, action, result, winner, or reason -> reject composition
  input as `Invalid video composition input`.
- Player reference absent from the scene snapshot -> render a safe fallback
  participant instead of throwing.
- Empty night deaths -> render `平安夜`.
- Null medicine target with `skipped` -> render `未使用`.
- Null wolf target -> render `未确定`.

### 5. Good / Base / Bad Cases

- Good: a director-only seer result renders actor, target, and the typed
  `good|wolves` result in Preview and MP4 at the same frame.
- Base: ordinary speech has `stage: null`, shows its complete text in the center,
  and keeps the timed subtitle window in the transcript band.
- Bad: public playback contains a guard, wolf, seer, witch, or night-resolution
  stage projected from a private event.

### 6. Tests Required

- Playback unit tests cover guard, wolf resolution, seer selection/result,
  witch used/skipped, night death/peace, exile/PK aggregation, and both winners.
- Privacy tests assert private night stages are absent publicly, present for the
  director, and sealed ballots are absent for both.
- Decoder tests accept missing and valid stage data and reject unknown variants.
- Shared-stage render tests cover action, night, vote, and game variants plus
  the no-stage placeholder.
- Production build and browser Preview must show no type or console errors.

### 7. Wrong vs Correct

Wrong: infer player IDs, action kinds, or outcomes from `title`, `text`, or
localized presenter templates inside React.

Correct: project typed GameEvent payloads once during playback compilation,
carry the closed `StagePresentation` union through the immutable composition,
and resolve player display data from the scene snapshot.
