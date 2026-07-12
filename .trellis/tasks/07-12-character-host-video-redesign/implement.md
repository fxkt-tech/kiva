# Implementation plan

## Gate 0 — preserve the baseline

- [ ] Record `git status` and leave unrelated work untouched.
- [ ] Run the focused existing creation, prompt, presenter and composition tests before edits.
- [ ] Render or retain one legacy preview frame for compatibility comparison.

## 1 — script domain and persistence

- [ ] Add `src/core/game-script.ts` with definition/snapshot types, validation, deep snapshot creation and the built-in legacy fallback.
- [ ] Add unit tests for valid definitions, duplicate IDs, unsafe asset paths, incomplete narrative/presentation data, invalid colors/style keys, snapshot immutability and legacy fallback.
- [ ] Add `kivdb/scripts.json` and validated `src/seeds/scripts.ts` with `midnight_archive`.
- [ ] Extend `LibraryRecord` and `LibraryRepository` read/write/batch contracts with scripts; add repository round-trip and atomic-write coverage.
- [ ] Extend seed tooling/tests so `seed:library` persists scripts and reports their count.

Validation checkpoint:

```sh
pnpm vitest run src/core/__tests__/game-script.test.ts src/server/__tests__/library-repository.test.ts src/seeds/__tests__/seed-library.test.ts
```

Rollback point: script library exists but is not yet required by Game.

## 2 — game snapshot, creation and compatibility

- [ ] Add required `Game.script` and `CreateGameFromPresetInput.script`; snapshot it in `createGameFromPreset`.
- [ ] Normalize historical records without script to `legacy_v1` in `GameRepository` and validate new snapshots.
- [ ] Extend creation services/actions to accept `scriptId`, require an enabled script, and resolve exactly one enabled presenter without a UI-selected presenter ID.
- [ ] Update temporary/random and preset creation paths, fixtures and action tests.
- [ ] Prove that changing/removing the library definition after creation does not mutate the Game snapshot.
- [ ] Prove an existing stored Game without script still loads.

Validation checkpoint:

```sh
pnpm vitest run src/core/__tests__/game-creation.test.ts src/server/__tests__/game-actions.test.ts src/server/__tests__/game-repository.test.ts src/server/__tests__/library-actions.test.ts
```

Rollback point: new games persist script; downstream UI may still render legacy styling.

## 3 — permanent cast and single presenter

- [ ] Replace runtime character definitions with the locked 12-person cast and IDs; preserve `qin_chuan` ID and update all preset references.
- [ ] Make the character seed a validated projection of the runtime JSON to eliminate biography/voice duplication.
- [ ] Assign at least six verified Chinese base voices plus restrained per-character rhythm parameters.
- [ ] Add a voice smoke command or focused test harness that synthesizes one short line for each distinct base voice before accepting the definitions.
- [ ] Replace presenter library definitions with only `wen_zhou`, exhaustive line copy, avatar and verified voice profile.
- [ ] Update presenter line/manifest tests and build a new immutable presenter voice manifest.
- [ ] Keep old presenter assets/directories readable; do not delete historical files as part of the migration.

Validation checkpoint:

```sh
pnpm vitest run src/seeds/characters.test.ts src/seeds/__tests__/seed-library.test.ts src/core/__tests__/character-definition.test.ts src/core/__tests__/presenter-definition.test.ts src/core/__tests__/presenter.test.ts
pnpm voice:build-presenter
```

Rollback point: data definitions are complete before any UI assumes the new portraits.

## 4 — portrait and script assets

- [ ] Use the locked cast bible to generate one consistent 12-player portrait set plus 闻舟; create contact sheets and inspect small/large crops.
- [ ] Generate or compose the three `未明档案` script assets: cover, daylight archive background and night archive background.
- [ ] Store versioned immutable PNGs under `kivdb/assets/characters`, `kivdb/assets/presenters`, and `kivdb/assets/scripts`.
- [ ] Add allow-listed asset routes for presenter and script files without weakening path validation.
- [ ] Verify all portrait paths load, all faces remain recognizable at 120 px, and no image contains text/faction clues/art-style drift.

Validation checkpoint:

```sh
file kivdb/assets/characters/*_v1.png kivdb/assets/presenters/*_v1.png kivdb/assets/scripts/*_v1.png
```

Manual gate: inspect one contact sheet and one 1920×1080 stage crop before proceeding.

## 5 — New Game and game-specific application styling

- [ ] Add the explicit script picker to `NewGameDialog`; show cover, name, theme, background excerpt and atmosphere tags, with `midnight_archive` selected by default.
- [ ] Remove presenter selection from New Game and pass `scriptId` through both preset and random forms.
- [ ] Add shared script-style projection helpers/CSS variables; do not let individual components parse script JSON.
- [ ] Add script cover/name/accent to home game cards.
- [ ] Keep the Editor on its original global tool theme; do not apply script backgrounds, colors, or labels there.
- [ ] Place the Editor game header above Draft in the right column, leaving the left Preview/Timeline column at full available height.
- [ ] Keep Library pages on the global application theme.
- [ ] Update interaction/render tests for one-script default selection, required script, preset/random submission and legacy home cards.

Validation checkpoint:

```sh
pnpm vitest run src/components/home/new-game-dialog.test.tsx src/components/home/new-game-setup.test.ts src/server/__tests__/game-actions.test.ts
```

## 6 — script-aware prompts

- [ ] Add the Game script snapshot to `PlayerLlmContext`.
- [ ] Render one semantic “共同叙事背景、不是身份事实” section in the v2 prompt after scene/task requirements and before game evidence.
- [ ] Keep v1 prompt compatibility unchanged unless the context type requires a neutral projection.
- [ ] Add tests proving theme/background/atmosphere are included, theme cannot be interpreted as player identity evidence, and fixed character fields remain unchanged across different scripts.
- [ ] Add a Qin川 regression covering at least three rule identities and two script snapshots: his style remains recognizable without claiming hidden information or guaranteed correctness.

Validation checkpoint:

```sh
pnpm vitest run src/core/__tests__/player-context.test.ts src/core/__tests__/prompt-builders.test.ts src/core/__tests__/llm-task-specs.test.ts
```

## 7 — shared preview and Remotion styling

- [ ] Read the Remotion best-practices skill before touching composition code.
- [ ] Extend composition input/decoder with script presentation and script asset slots; keep schema compatibility explicit.
- [ ] Pass the same script-aware input through browser preview and export snapshot builder.
- [ ] Copy/rewrite cover/background/presenter/player assets from allow-listed internal paths into export job assets.
- [ ] Add exhaustive renderer/style dispatch for `legacy_v1` and `midnight_archive_v1`.
- [ ] Implement A-direction palette, typography, archive rules, stamped results and event emphasis in the shared `HtmlPlaybackStage` without introducing a second export renderer.
- [ ] Keep background images as direct `object-cover` layers with no opacity/filter overlay, per the active video spec.
- [ ] Preserve current seat geometry, transcript ownership, audio timing, player subtitle cues and role identity semantics.
- [ ] Update composition, stage, snapshot and video-export tests for both legacy and `未明档案` inputs.

Validation checkpoint:

```sh
pnpm vitest run src/components/preview-v2 src/components/preview/shot-engine src/server/video-export
pnpm video:smoke
```

Manual gate: capture representative frames for player speech, presenter narration, night action, vote result, death/safe night and game result at native 1920×1080 plus phone-scaled preview.

## 8 — integrated quality gate

- [ ] Run the Trellis check workflow and verify requirements against implementation.
- [ ] Run the full automated suite:

```sh
pnpm typecheck
pnpm test
pnpm build
git diff --check
```

- [ ] Create a new `未明档案` game through both preset and random modes.
- [ ] Confirm the Game record stores script, presenter and player snapshots.
- [ ] Generate at least one speech, inspect the Prompt script section, synthesize voice, open Preview and export MP4.
- [ ] Confirm an old Game record still opens with legacy styling.
- [ ] Review that no management Library page inherits per-script styling.

## 9 — knowledge and cleanup

- [ ] Update `.trellis/spec/frontend/programmatic-video.md` with the Game script snapshot and asset-dispatch contract.
- [ ] Update `.trellis/spec/frontend/llm-prompt-workflows.md` with the script-background semantic section and non-evidence rule.
- [ ] Record the accepted A direction and permanent cast in the task artifacts/journal.
- [ ] Delete or archive the throwaway A/B/C prototype after its decision has been captured; do not ship its switcher or alternate variants.
