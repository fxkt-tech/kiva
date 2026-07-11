# Programmatic Video Preview and Export

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
- Stage copy has one semantic owner: the header always owns `gameTitle` and
  never displays `scene.title`. The game title is the header's only readable
  line; do not add an English eyebrow or decorative product label above it. The
  game title is horizontally centered and uses enough line height and vertical
  padding to preserve complete CJK glyphs.
  The empty visual-stage slot owns no readable content or visual treatment; the transcript card owns
  narrator identity and narration. Speech uses the active player as narrator.
  `phase`, `announcement`, `vote`, and `resolution` use the presenter, including
  director-only night actions.
- Presenter copy has one data owner: `kivdb/presenters.json` stores reusable
  Presenter Definitions. Each definition owns an exhaustive independent script
  and voice map. Every Game selects exactly one enabled definition and stores a
  complete `Game.presenter` snapshot; preview and export must consume that
  snapshot, never reload current library wording for an existing game.
- Presenter Definitions are first-class Library objects. The Presenters editor
  may change `id`, `name`, `avatar`, `enabled`, every semantic template, and
  every standard/per-seat voice mapping. It must derive keys, variables, and
  the required 1–12 seat shape from `PRESENTER_LINE_VARIABLES`, then submit a
  complete catalog through the same `validatePresenterDefinitions()` boundary
  used for direct KivDB loads. Blank voice filenames decode to null; a
  non-blank filename carries `ready` or `pending`.
- A Presenter Definition is not a Player Character. It has no seat, role,
  faction, action, vote, private knowledge, or win-condition participation.
- Presenter templates use stable semantic keys and validated named
  placeholders. Event-to-key selection and typed placeholder values belong to
  the core presenter resolver; rendering components must not parse event
  payloads or choose copy variants.
- Resolved `PlaybackItem` data explicitly carries presenter identity/avatar,
  transcript ownership, transcript text, and the presenter cue. Player-authored
  speech remains player-owned while its player-addressed prompt remains a
  presenter cue.
- Voice mappings are `ready`, `pending`, or absent. Only `ready` mappings emit
  cues. `pending` exists to document a stale/unrecorded asset without allowing
  subtitle/audio wording drift; switching a verified KivDB mapping to `ready`
  enables it without a source change.
- System voice files are selected from `presenterCue.voiceFile`; never match
  rendered Chinese titles or prose. The asset route authorizes safe basenames
  declared by the Presenter Definition collection rather than maintaining a
  second filename allowlist.
- Never use `scene.title` as transcript fallback. Scenes with empty narration
  use an exhaustive semantic projection: explicit copy for every phase, then
  event details or a kind-specific neutral message for other scene kinds.
- Header, empty visual-stage slot, and transcript occupy equal-width grid regions
  in the center column. Only the header and transcript render card treatments.
  The empty stage has no border, background, shadow, texture, or translation;
  the shared grid gap is its only spacing source. Each six-card seat track spans
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
  is `http://127.0.0.1:3000`.
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
- Presenter definitions: validate unique IDs, complete semantic key sets, exact
  placeholder contracts, safe filenames, ready/pending status, and explicit
  prompt mappings for seats 1–12. Prove two definitions can render one event to
  different copy.
- Game presenter selection: both preset and random creation require an enabled
  presenter ID; the created snapshot remains unchanged after its library
  definition is edited, disabled, or removed.
- Library presenter configuration: assert the tab/list/editor renders both
  definitions; form parsing reconstructs all semantic keys and seats 1–12;
  identity, template, ready/pending, and null mappings survive repository
  round-trip; invalid catalogs do not replace the stored collection.
- Presenter resolution: cover every event and result variant, player speech
  ownership, empty-speech fallback, pending silence, ready prompt audio, and
  missing optional files.
- Header geometry: the game title remains horizontally centered, and CJK glyph
  bounds are not clipped at the top or bottom at native 1920x1080 resolution;
  rendered markup contains no secondary English eyebrow.
- Grid geometry: header, empty visual-stage slot, and transcript have identical
  x and width; the stage section has only grid-positioning classes and no style
  attribute. Both seat tracks align to the header top and transcript bottom and
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
progress, then implement a separate export-only layout.

Correct: derive `timeMs` from the Remotion frame, create a renderer-neutral view
model, and render the same `HtmlPlaybackStage` in Player and Renderer. Fit the
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
