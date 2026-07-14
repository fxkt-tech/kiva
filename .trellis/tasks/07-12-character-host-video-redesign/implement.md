# Implementation plan

## Gate 0 — preserve the baseline

- [ ] Record `git status` and leave unrelated work untouched.
- [ ] Run the focused existing creation, prompt, presenter and composition tests before edits.
- [ ] Retain one current preview frame for visual regression comparison.

## 1 — script domain and persistence

- [ ] Add `src/core/game-script.ts` with definition/snapshot types, validation, deep snapshot creation and the built-in legacy fallback.
- [ ] Add unit tests for valid definitions, duplicate IDs, unsafe asset paths, incomplete narrative/presentation data, invalid colors/style keys, snapshot immutability and legacy fallback.
- [ ] Add `kivdb/scripts.json` and validated `src/seeds/scripts.ts` with `midnight_archive`.
- [x] Make Script a validated `ContentCatalog` collection; add complete-catalog round-trip and atomic-write coverage.
- [ ] Extend seed tooling/tests so `seed:library` persists scripts and reports their count.

Validation checkpoint:

```sh
pnpm vitest run src/core/__tests__/game-script.test.ts src/server/__tests__/library-repository.test.ts src/seeds/__tests__/seed-library.test.ts
```

Rollback point: script library exists but is not yet required by Game.

## 2 — game snapshot, creation and clean break

- [x] Add required `Game.script` and snapshot it in `createGameFromLineup`.
- [x] Reject historical/incomplete Game records; accept only exact current schema 2 snapshots.
- [ ] Extend creation services/actions to accept `scriptId`, require an enabled script, and resolve exactly one enabled presenter without a UI-selected presenter ID.
- [ ] Update temporary/random and preset creation paths, fixtures and action tests.
- [ ] Prove that changing/removing the library definition after creation does not mutate the Game snapshot.
- [ ] Prove an existing stored Game without script still loads.

Validation checkpoint:

```sh
pnpm vitest run src/core/__tests__/game-creation.test.ts src/server/__tests__/game-actions.test.ts src/server/__tests__/game-repository.test.ts src/server/__tests__/library-actions.test.ts
```

Rollback point: new games persist script; downstream UI may still render legacy styling.

## 3 — initial actor pool and single presenter

- [ ] Replace runtime character definitions with the first 12 production-ready Actors and IDs; preserve both the name 「秦川」 and ID `qin_chuan` without treating this initial set as the library ceiling.
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
- [ ] Update interaction/render tests for required Script, saved/custom Lineup submission, and current home cards.

Validation checkpoint:

```sh
pnpm vitest run src/components/home/new-game-dialog.test.tsx src/components/home/new-game-setup.test.ts src/server/__tests__/game-actions.test.ts
```

## 6 — script-aware prompts

- [ ] Add the Game script snapshot to `PlayerLlmContext`.
- [ ] Render one semantic “共同叙事背景、不是身份事实” section in the v2 prompt after scene/task requirements and before game evidence.
- [x] Remove old prompt compatibility and persist only the current staged Prompt v3 contracts.
- [ ] Add tests proving theme/background/atmosphere are included, theme cannot be interpreted as player identity evidence, and stable Actor fields remain unchanged across different scripts.
- [ ] Add a 秦川 regression covering at least three Rule Roles and two script snapshots: his style remains recognizable without claiming hidden information, guaranteed correctness or narrative priority.

Validation checkpoint:

```sh
pnpm vitest run src/core/__tests__/player-context.test.ts src/core/__tests__/prompt-builders.test.ts src/core/__tests__/llm-task-specs.test.ts
```

## 6.5 — Script Author Agent v4

- [x] Replace outline/large-batch authoring with one durable `EpisodeAuthorWorkspace` and deterministic `story -> ensemble -> actor_arc -> relationship -> beats -> assembly` task routing.
- [x] Give Script Author a dedicated model binding snapshot independent of all Player character bindings.
- [x] Bound the story to four acts, ensemble selection to six relationship seeds, every free-text field to 80 characters, one character or relationship per request, and one local scene beat batch to at most five steps with one prior move per current actor.
- [x] Persist validated workspace and an append-only request record after every successful or failed child task under the Game job lock.
- [x] Resume matching `generating`/`failed` jobs from the first incomplete task without rerunning completed story, ensemble, character, relationship, or beat work.
- [x] Preserve provider finish reason and usage through JSON parse errors; bypass whole-document repair for length truncation, split beats, and retry non-beat tasks once from their original compact input.
- [x] Render live Agent phase/progress, per-task request details, and preserved partial-result counts on the Script page.
- [x] Reject pre-v4 Author requests/old workspace shapes without migration and persist the final executable Episode snapshot on schema 3.
- [x] Add unit, integration, resume, binding, truncation, and Script-page progress coverage.

Validation checkpoint completed:

```sh
pnpm typecheck
pnpm test       # 77 files, 627 tests
pnpm build
git diff --check
```

## 6.6 — Actor and Library Studio v2

- [x] Set Studio scope: an internal content tool with an extensible Actor pool; every Game selects exactly 12 enabled Actors without a globally required Actor, while unsupported Rule Roles cannot be created in Studio.
- [x] Replace Character/Role ambiguity with exact `Actor` and `RuleRole` terminology throughout domain, UI and prompt contracts.
- [x] Introduce exact Actor Definition v2 plus bounded Runtime/Author card compilers; remove raw Character system prompts and legacy persona/style aliases.
- [x] Make Actor interaction fields reference only reusable functions/conflict axes, never named Actor IDs or permanent pairings; compile ensemble matches from the selected 12.
- [x] Replace editable Rule Role JSON with a code-owned mechanics/knowledge/task registry and read-only diagnostics.
- [x] Replace Player prompt assembly with validated PlayerIntent plus speech-only performance rendering; keep night actions single-stage.
- [x] Update Script Author cards and Episode beat semantics so current dramatic direction cannot prescribe future-aware factual conclusions.
- [x] Replace duplicate Preset arrays with one Rule Role setup/Lineup source and make Actor the only model-binding owner; keep Actor selection a separate exactly-12 Game input.
- [x] Validate and snapshot exactly 12 selected Actors before Script Author starts; preserve 「秦川」 / `qin_chuan` in the Actor Library but do not require that Actor in any Game.
- [x] Give Script Author the real seat-to-Rule Role assignment and legal plan, while proving runtime projections expose only each Actor's own role and visible facts.
- [x] Make Author/runtime/preview/export consumers read the immutable Game cast rather than the live Actor Library.
- [x] Replace per-table repository/actions and independent prompt diagnostics with one deep `ContentCatalog` module used by Studio, Game creation, seeds and tests.
- [x] Rebuild Library navigation/editors as Actors, Scripts, Show, Lineups and read-only Rules; add production Runtime/Author previews plus full-pool/current-cast diagnostics.
- [x] Rewrite current `kivdb` content, Player/Game snapshots, fixtures and tests directly to v2 without a compatibility decoder.

## 7 — shared preview and Remotion styling

- [ ] Read the Remotion best-practices skill before touching composition code.
- [ ] Extend composition input/decoder with Script presentation and Script asset slots; keep the current schema exact.
- [ ] Pass the same script-aware input through browser preview and export snapshot builder.
- [ ] Copy/rewrite cover/background/presenter/player assets from allow-listed internal paths into export job assets.
- [ ] Keep exhaustive renderer/style dispatch for the sole supported `midnight_archive_v1` style.
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
- [ ] Record the accepted A direction and initial Actor pool in the task artifacts/journal.
- [ ] Delete or archive the throwaway A/B/C prototype after its decision has been captured; do not ship its switcher or alternate variants.
