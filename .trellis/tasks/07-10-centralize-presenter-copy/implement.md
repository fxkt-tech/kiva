# Implementation Plan: Selectable KivDB Presenters

## Gate 0 — Preserve Context

- [ ] Record `git status`; preserve the user's `next-env.d.ts` change and the
  current preview-v2 layout/card work.
- [ ] Re-read frontend programmatic-video specs and shared guides.
- [ ] Confirm current KivDB game/export inventory before the direct migration.

Rollback point: no new source changes.

## Gate 1 — Presenter Definitions and KivDB Data

- [ ] Add `src/core/presenter-definition.ts` with definition/snapshot types,
  exhaustive line contracts, validation, and snapshot creation.
- [ ] Add exhaustive validator tests for IDs, names, avatars, timestamps,
  semantic keys, placeholders, voice statuses, filenames, and seats 1–12.
- [ ] Create `kivdb/presenters.json` with complete independent definitions for:
  - `night_watch` / `守夜人`, using the approved atmospheric copy;
  - `judge` / `法官`, using concise traditional moderator copy.
- [ ] Mark rewritten/mismatched audio pending, preserve only matching ready
  player prompts, and leave unavailable mappings null.

Expected files:

```text
kivdb/presenters.json
src/core/presenter-definition.ts
src/core/__tests__/presenter-definition.test.ts
```

Validation:

```bash
pnpm exec vitest run src/core/__tests__/presenter-definition.test.ts
pnpm typecheck
```

Rollback: remove additive definition/data/test files.

## Gate 2 — Library, Selection, and Game Snapshot

- [ ] Extend `LibraryRecord`, repository reads/writes, atomic `saveAll()`,
  diagnostics, actions, seed tooling, and test memory adapters with presenters.
- [ ] Add a Presenters Library tab and editor for identity, templates, and all
  standard/per-seat voice mappings; validate saves through the shared contract.
- [ ] Add enabled presenter data to the home page and `NewGameDialog`.
- [ ] Add one required presenter selector shared by preset/random modes.
- [ ] Submit and validate `presenterId` in both creation actions.
- [ ] Pass the selected definition into game creation and snapshot it on
  `Game.presenter`; keep presets presenter-agnostic.
- [ ] Add tests for selection, unknown/disabled rejection, and snapshot
  isolation after library edits.
- [ ] Directly add the `night_watch` snapshot to every checked-in
  `kivdb/games/*.json`; add strict tests for missing presenter data.

Expected files include:

```text
src/core/game.ts
src/core/__tests__/game-creation.test.ts
src/server/library-repository.ts
src/server/library-actions.ts
src/server/game-actions.ts
src/server/game-repository.ts
src/server/__tests__/library-repository.test.ts
src/server/__tests__/library-actions.test.ts
src/server/__tests__/game-actions.test.ts
src/components/home/new-game-dialog.tsx
src/components/home/new-game-dialog.test.tsx
src/app/actions.ts
src/app/page.tsx
src/scripts/seed-library.ts
kivdb/games/*.json
```

Validation:

```bash
pnpm exec vitest run src/core/__tests__/game-creation.test.ts src/server/__tests__/library-repository.test.ts src/server/__tests__/library-actions.test.ts src/components/home/new-game-dialog.test.tsx
pnpm typecheck
```

Rollback: restore the prior library/game schema together; do not leave games
that reference definitions the repository does not load.

## Gate 3 — Deep Presenter Resolution and Playback

- [ ] Add `src/core/presenter.ts` with event-to-key selection, typed variables,
  literal template rendering, transcript ownership, and ready/pending handling.
- [ ] Test every event/variant against both initial presenters and verify their
  wording differs while semantic values remain correct.
- [ ] Extend `PlaybackItem` with resolved presenter identity, avatar,
  transcript speaker, and presenter cue.
- [ ] Require the Game Presenter snapshot in playback compilation.
- [ ] Preserve player-authored speech/last words and resolve all presenter
  narration/prompts through the selected snapshot.
- [ ] Stop using event-presenter prose as playback narration; update all test
  callers and fixtures.

Expected files:

```text
src/core/presenter.ts
src/core/__tests__/presenter.test.ts
src/core/playback.ts
src/core/__tests__/playback.test.ts
src/core/__tests__/events.test.ts
src/core/__tests__/advance-planner.test.ts
src/server/__tests__/llm-integration.test.ts
```

Validation:

```bash
pnpm exec vitest run src/core/__tests__/presenter.test.ts src/core/__tests__/playback.test.ts
pnpm typecheck
```

Rollback: revert resolver and playback shape as one unit.

## Gate 4 — Stage, Audio, Route, and Export

- [ ] Make `stage-copy.ts` consume resolved presenter name/avatar/speaker/text;
  delete global presenter identity and hard-coded prose.
- [ ] Apply the shared identity palette to seat role labels and presenter
  identity; keep presenter and player speaker names visually identical, and
  preserve role colors when dead avatars desaturate.
- [ ] Apply the approved C smoke-glass day/night palette to header, seat cards,
  transcript, identity tones, and active-speaker treatment without changing
  layout or filtering the background.
- [ ] Compact seat rows, enlarge and center name/role, reduce the full dead-card
  hierarchy, and add a mirrored inner-edge treatment for the active speaker.
- [ ] Redesign the transcript band with the current-speaker identity value
  inline and no `身份` prefix. Color player mentions only in presenter-owned
  narration; leave player-authored speech unparsed and neutral.
- [ ] Add a Preview v2 copy-current-frame control with native-size PNG clipboard
  output and visible copying/success/failure feedback.
- [ ] Use the presenter's first name glyph when its avatar is null.
- [ ] Replace title/text audio matching with `presenterCue.voiceFile`.
- [ ] Load all presenter definitions in the voice route and replace the static
  allowlist with declared-file authorization.
- [ ] Include presenter avatars in composition asset collection and immutable
  export copying.
- [ ] Update v1/v2 preview callers to compile with `record.game.presenter`.
- [ ] Update snapshot/export tests for presenter identity, pending silence,
  ready prompts, optional assets, and immutability.

Expected files:

```text
src/components/preview-v2/stage/stage-copy.ts
src/components/preview-v2/stage/stage-copy.test.ts
src/components/preview/preview-audio.ts
src/components/preview/preview-audio.test.ts
src/app/kivdb-assets/voice/system/[file]/route.ts
src/components/preview-v2/composition/create-composition-input.ts
src/app/games/[gameId]/preview/page.tsx
src/app/games/[gameId]/preview_v2/page.tsx
src/server/video-export/snapshot-builder.ts
src/server/video-export/video-export.test.ts
```

Validation:

```bash
pnpm exec vitest run src/components/preview-v2/stage/stage-copy.test.ts src/components/preview/preview-audio.test.ts src/server/video-export/video-export.test.ts
pnpm typecheck
```

Rollback: restore stage/audio/route/export consumers together; never leave
mixed title-based and semantic voice selection.

## Gate 5 — Direct Composition Upgrade and Final Verification

- [ ] Bump the composition input schema and require the new playback item
  fields; do not add legacy normalization.
- [ ] Update composition/export repository fixtures for the new shape.
- [ ] Search out all old presenter product prose, global name constants,
  semantic title-to-audio maps, and route allowlists.
- [ ] Update `.trellis/spec/frontend/programmatic-video.md` with selectable
  presenter, snapshot, pending audio, and strict data contracts.
- [ ] Run full checks and fix every failure.

Validation:

```bash
rg -n "PRESENTER_NAME|phaseNarration|titleVoiceFiles|allowedFiles" src
pnpm test
pnpm typecheck
pnpm build
pnpm video:smoke
git diff --check
```

Final review:

- [ ] `守夜人` and `法官` are independently complete and selectable.
- [ ] Both creation modes require a presenter.
- [ ] Every current game contains a stable presenter snapshot.
- [ ] No missing-data or legacy-composition fallback exists.
- [ ] Preview/export use only the selected snapshot.
- [ ] Pending recordings never play stale audio.
- [ ] No unrelated user work was overwritten.
