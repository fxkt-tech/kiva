# 最新版单轨清理实施计划

## 0. 基线与保护

- [x] Record the dirty-work baseline and avoid unrelated Preview/framework edits.
- [x] Run focused baseline tests for repository, Episode, Prompt, Game Script, composition, and export.
- [x] Confirm the persisted-data deletion set without deleting it yet.

## 1. 收紧 Game 与 Player 持久化契约

- [x] Add the required GameRecord schema version and strict read/save validation.
- [x] Require all current top-level fields and validate run mode, ruleset, presenter, script, players, generations, voice artifacts, and Episode state without normalization.
- [x] Remove missing-run-mode, missing-script, missing-ruleset-fields, missing-generation-fields, and missing-Episode-state injection.
- [x] Remove Player `profileSourceId` and duplicate `systemPrompt`; add strict current Player snapshot validation.
- [x] Remove six-player compatibility helpers and make the default ruleset a direct 12-player definition.
- [x] Update factories, actions, fixtures, and repository tests to write one complete current record and to reject incomplete records.

Checkpoint:

```sh
pnpm vitest run src/core/__tests__/player.test.ts src/core/__tests__/types.test.ts src/core/__tests__/game-run-mode.test.ts src/core/__tests__/game-creation.test.ts src/server/__tests__/game-repository.test.ts src/server/__tests__/game-actions.test.ts
```

## 2. 删除 Episode schema v1

- [x] Make Episode schema 2 the only type and decoder contract.
- [x] Remove the legacy character projection, legacy hash, schema-conditional fields, and conditional dramaturgy.
- [x] Replace state normalization with strict mode-aware state validation.
- [x] Remove the old Review UI branch and schema-v1 execution tests.
- [x] Remove Local Heuristic episode-v1 handlers and update specs.

Checkpoint:

```sh
pnpm vitest run src/core/__tests__/episode-script.test.ts src/components/script/episode-ensemble-review.test.tsx
```

## 3. 删除 Prompt v1 与旧 Generation 展示

- [x] Delete `prompt-builders-v1.ts`, mode types, runtime env selection, and mode propagation.
- [x] Keep only Prompt v2 schema handlers in Local Heuristic.
- [x] Require structured character fields; remove opaque legacy system-prompt fallback.
- [x] Require `GenerationRecord.request` and remove old reasoning-key aliases/UI copy.
- [x] Replace v1-named current test fixtures with current schema names and delete rollback tests/spec text.

Checkpoint:

```sh
pnpm vitest run src/core/__tests__/prompt-builders.test.ts src/core/__tests__/speech-generation.test.ts src/core/__tests__/action-generation.test.ts src/core/__tests__/generation-record.test.ts src/server/__tests__/llm-runtime.test.ts src/components/editor/llm-generation-details.test.tsx src/components/editor/draft-panel.test.ts
```

## 4. 删除旧 Game Script 与 Composition 输入

- [x] Keep only `midnight_archive_v1`; remove legacy snapshot creation and old asset-path acceptance.
- [x] Require Game Script in composition/frame factories and use a current fixture in Remotion root/tests.
- [x] Decode only composition schema 4.
- [x] Make current PlaybackItem fields explicit and require them in the composition decoder.
- [x] Require script background URLs and remove export fallback to Preview backgrounds.
- [x] Remove old background route entries and obsolete tracked image assets while retaining the font.
- [x] Update Game Script, composition, stage-palette, snapshot, and video-export tests/specs.

Checkpoint:

```sh
pnpm vitest run src/core/__tests__/game-script.test.ts src/components/preview-v2/composition src/components/preview-v2/stage/stage-palette.test.ts src/server/video-export
```

## 5. 全仓收尾

- [x] Search production, tests, and specs for legacy selectors, old schema branches, aliases, and migration wording.
- [x] Classify every remaining `v1`, `fallback`, and `compatible` occurrence against the design inventory.
- [x] Update Trellis specs to describe strict current contracts, preserving unrelated dirty content.
- [x] Run focused cross-layer latest-workflow tests.

## 6. 代码质量门

- [x] Run the Trellis check workflow.
- [x] Run:

```sh
pnpm typecheck
pnpm test
pnpm build
git diff --check
```

- [x] Review the final diff for accidental edits to pre-existing dirty files.

Destructive data deletion is forbidden until every item in this gate passes.

## 7. 删除不兼容数据

- [x] Delete all contents of `kivdb/games/` (records, voice jobs, and voice audio).
- [x] Delete `kivdb/exports/` if present.
- [x] Verify both locations contain no persisted jobs or games.
- [x] Retain current library JSON, script/character/presenter assets, presenter voice data, Preview font, and worker state.
- [x] Rerun `git diff --check` and one repository empty-list smoke test after deletion.

## 8. 收尾

- [x] Run `trellis-update-spec` for the learned latest-only persistence contract.
- [x] Review deletion and dirty-work status with the user-visible handoff.
- [x] Commit only after explicit commit confirmation, then run the Trellis finish workflow.

## Verification

- `pnpm typecheck`: passed.
- `pnpm test`: 75 files and 612 tests passed.
- `pnpm build`: passed with Next.js 16.2.9.
- `git diff --check`: passed before and after data deletion.
- Live `kivdb` repository smoke check: returned zero Games after deletion.
