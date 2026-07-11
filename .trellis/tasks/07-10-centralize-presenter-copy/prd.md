# Centralize presenter copy in KivDB

## Goal

Make KivDB the single editable source of truth for a reusable library of
presenters and every line each presenter speaks or shows in preview v2 and
exported videos. Every game selects exactly one presenter, whose identity,
atmospheric Werewolf copy, and system-voice mapping remain stable for that
game.

## Background

- Presenter narration is currently split between
  `src/core/event-presenter.ts` and
  `src/components/preview-v2/stage/stage-copy.ts`.
- System-voice selection is separately hard-coded in
  `src/components/preview/preview-audio.ts`, while MP3 files live under
  `kivdb/assets/voice/system/` and the asset route maintains another filename
  allowlist.
- Preview v2 and export both compile director-audience playback. Export then
  snapshots resolved playback items and copies referenced audio, so catalog
  resolution must happen before composition snapshot creation.
- Existing system voices are optional. Missing audio produces a valid silent
  scene/export rather than failing playback.
- KivDB already models roles, player characters, and presets as reusable
  definition collections. Game creation resolves those definitions into stable
  per-game snapshots.
- A presenter is a separate domain entity: it does not occupy a seat, receive a
  role, belong to a faction, vote, or affect the win condition.
- The approved replacement copy covers phases, night actions, dawn
  announcements, exile outcomes, game-end reasons, semantic fallbacks, and
  currently filtered internal events. Player-authored speeches and last words
  remain player-owned content.

## Requirements

- Add one versioned, human-editable presenter-definition collection under
  `kivdb/`, containing multiple independently selectable presenters.
- Expose Presenter Definitions as first-class objects in the Library UI. Users
  can edit identity, avatar, enabled state, every copy template, and every
  standard/per-seat voice mapping without hand-editing JSON.
- Give each presenter a stable ID, display name, optional avatar, enabled state,
  timestamps, complete copy contract, and optional system-voice mappings.
- Each Presenter Definition owns a complete independent copy and audio catalog;
  presenters do not inherit a shared global script. Every enabled definition
  must pass the same exhaustive semantic-key and placeholder validation.
- Seed two enabled Presenter Definitions:
  - `守夜人`: the approved immersive, suspenseful Werewolf copy;
  - `法官`: a distinct concise, directive, in-person moderator style.
  Both initially use a null avatar, with the stage falling back to the first
  glyph of the presenter name (`守` / `法`).
- Require every newly created game to select one enabled presenter in both
  preset and random-seat creation flows.
- Snapshot the selected presenter into the game so later edits, disabling, or
  deletion of its KivDB definition do not change existing games.
- Directly update every current `kivdb/games/*.json` record with a selected
  presenter snapshot. Do not add a default-presenter fallback, lazy migration,
  or compatibility path for game records without a presenter.
- Give every presenter line a stable semantic key rather than selecting copy by
  matching rendered Chinese titles.
- Store atmospheric text templates and their optional system-voice filename
  mapping together in that manifest.
- Give each declared voice mapping an explicit `ready` or `pending` status.
  `pending` mappings remain documented in KivDB but do not produce audio cues;
  this is required whenever the available recording does not match the approved
  text. Existing player-addressed prompts may remain `ready` where their wording
  is preserved, while all newly rewritten narration starts `pending` until its
  recording is replaced and verified.
- Support validated named placeholders for player labels, player lists,
  factions, roles, and other dynamic event values without evaluating code from
  KivDB.
- Use the approved replacement copy from the preceding conversation for every
  covered variant.
- Treat player-addressed speech, last-word, and PK prompts as presenter copy.
  Store their templates and per-seat optional voice mappings in the same
  manifest for seats 1–12; currently missing seat 7–12 MP3 files remain a
  non-fatal silent-audio case.
- Resolve presenter copy before `PlaybackItem` snapshots are passed to the
  shared HTML preview/Remotion composition, preserving preview/export parity
  and immutable export behavior.
- Give Preview v2 role identity labels one coherent cinematic palette: wolf
  red, villager gray, seer green, witch purple, guard blue, hunter ochre, and
  presenter gold. Presenter and player speaker names share the same neutral
  treatment; semantic color belongs to the identity label and avatar frame.
  Use the approved C “smoke glass” dual palette: translucent cool surfaces and
  brighter identity tones at night, warmer translucent surfaces and corrected
  identity tones by day. The palette must remain legible after phone-scale
  reduction without filtering the source background.
- Compact the six seat cards without shrinking their primary information. Keep
  each side track at full height and distribute the six shorter cards with
  space-between; enlarge and center player name/role, strongly recede dead
  players, and give the active speaker a mirrored inner-edge gold field.
- Redesign the Preview v2 transcript band around avatar, speaker, inline
  identity metadata, and narration. Show the current identity value directly,
  without an `身份` prefix. Only presenter-owned narration colors structured
  player mentions by those players' roles; player-authored speech remains
  unparsed and uses the normal subtitle color throughout.
- Keep the currently empty center stage as a layout slot only. It has no border,
  surface, shadow, texture, or positional effect until a dedicated stage design
  is approved; header/stage/subtitle spacing comes only from the shared grid.
- Display configured day/night background images at their source opacity,
  saturation, and color. Do not add darkening gradients, vignettes, scanlines,
  or venetian-blind textures; use the fallback gradient only when no image exists.
- Add a Preview v2 `复制当前帧` control that rasterizes the current Player frame
  as a native 1920×1080 PNG and writes it to the system clipboard without
  seeking or changing timeline state. Show copying, success, and failure states.
- Remove duplicate presenter prose and duplicate semantic audio maps from
  application source. Code may retain neutral error diagnostics but must not
  retain a second product-copy catalog.
- Validate the manifest at its load boundary with actionable errors for unknown
  schema versions, missing keys, invalid placeholders, unsafe audio filenames,
  and malformed values.
- Keep player-authored speech/last-word text unchanged and keep missing audio
  non-fatal.
- Preserve the current `KIVA_DATA_DIR` deployment boundary; the default
  presenter collection lives in `kivdb/`, and a configured data directory owns
  its corresponding definitions and voice assets.
- Keep existing exported job snapshots immutable after creation.
- Upgrade persisted playback/composition data directly. The current KivDB has
  no historical export-job snapshots, so legacy composition-schema decoding is
  not required for this delivery.

## Acceptance Criteria

- [ ] One KivDB collection contains multiple presenter definitions; each has a
  stable identity, all required semantic text variants, placeholder contracts,
  and optional system-voice mappings.
- [ ] Two presenter definitions may resolve the same game event to different
  wording and voice files without application-source changes.
- [ ] KivDB initially contains selectable `守夜人` and `法官` definitions with
  complete, independently validated catalogs and visibly distinct wording.
- [ ] The Library includes a Presenters tab whose editor round-trips identity,
  all semantic templates, and ready/pending/null voice mappings through the
  same strict KivDB validator used at load time.
- [ ] The new-game dialog lists enabled presenters and requires one selection
  for both preset and random-seat creation.
- [ ] A created game stores the selected presenter snapshot, and preview v1,
  preview v2, and export use that snapshot rather than the current library
  definition.
- [ ] Editing, disabling, or removing a presenter definition does not alter an
  existing game's presenter identity, copy, or voice mapping.
- [ ] Every current KivDB game record is explicitly migrated to a selected
  presenter snapshot; loading a non-migrated record fails instead of silently
  selecting a presenter.
- [ ] Phase, night-action, dawn, exile, game-end, and fallback scenes render the
  approved presenter copy with correct dynamic player/faction values.
- [ ] Speech, last-word, and PK player prompts for seats 1–12 resolve through
  the same manifest; available voice assets are selected, and unavailable
  per-seat assets degrade to silence.
- [ ] Player speeches and last words remain attributed to the player and retain
  their authored text.
- [ ] Preview v2 and a newly created export snapshot contain identical resolved
  presenter text and audio sources for the same confirmed game state.
- [ ] Seat role labels visibly distinguish all six built-in roles, presenter
  identity uses gold, all speaker names share the same neutral treatment, and
  dead-player treatment reduces the whole card's prominence without erasing
  the role color. Seat-card name and role are centered and larger; the active
  speaker is clearly distinguishable on either mirrored side.
- [ ] The transcript speaker row shows the identity value directly without an
  `身份` label or separate row. Player scenes show the player's role; presenter
  scenes show only `主理人`. Player names/seat labels in presenter narration use
  those players' role colors, while player-authored speech is never parsed for
  highlights.
- [ ] Existing export snapshots do not change when the KivDB manifest or voice
  assets change later.
- [ ] Missing optional MP3 files yield warnings/silent scenes, while malformed
  or incomplete presenter manifests fail at the data-loading boundary with a
  clear error.
- [ ] `pending` voice mappings never produce an audio cue, so newly approved
  subtitles cannot play stale recordings; changing a verified mapping to
  `ready` in KivDB enables it without a source-code change.
- [ ] No presenter product prose or semantic title-to-audio map remains
  duplicated in `event-presenter.ts`, `stage-copy.ts`, `preview-audio.ts`, or the
  voice asset route.
- [ ] Unit tests cover manifest decoding, placeholder validation/rendering,
  semantic variant selection, missing audio, player-copy ownership, and
  preview/export snapshot parity.
- [ ] Full tests, type checking, and the preview-v2 render smoke check pass.

## Out of Scope

- Generating or recording replacement MP3 files.
- Changing player-authored speech or last-word content.
- Changing export codecs, dimensions, frame rate, or job lifecycle.
- Supporting legacy game records or legacy composition snapshots that predate
  this presenter model.
