# Character-Driven Episode Authoring Implementation Plan

## 1. Add Character-Aware Episode Contracts

- Add failing core tests for schema-v2 cast directions, relationships,
  character-beat fields, signature-step validation, exact player coverage, and
  schema-v1 compatibility.
- Add the v2 types and schema-aware validator in `src/core/episode-script.ts`.
- Centralize the concise episode character-profile projection and the legacy/v2
  input-hash recipes; update generation, approval, and execution comparisons to
  choose the recipe from the snapshot schema.
- Keep the legal trace, planned payloads, and speech budgets byte-for-byte equal
  to the pre-author compiled plan in regression tests.

Validation:

```bash
pnpm vitest run src/core/__tests__/episode-script.test.ts
```

Rollback point: v2 contracts and tests compile without changing author requests.

## 2. Generate and Validate Ensemble Direction

- Add request-capture tests proving the outline contains every concise character
  profile and deterministic performance opportunity.
- Upgrade `src/core/episode-author.ts` to prompt/parse outline schema v2 with
  exact cast directions and valid relationship pairs.
- Extend the repair contract with exact player IDs, allowed signature steps, and
  relationship rules.
- Teach `LocalHeuristicLlmClient` to return deterministic structurally complete
  v2 ensemble output while retaining v1 handlers.
- Test missing/duplicate players, unknown relationships, invalid signature
  indexes, and character-profile input changes.

Validation:

```bash
pnpm vitest run src/core/__tests__/episode-script.test.ts
```

Rollback point: one generated candidate reaches Review with valid ensemble data.

## 3. Make Speech Beats Continue Character Arcs

- Add request-capture tests for current-batch actor profiles, cast directions,
  touching relationships, and bounded prior moves.
- Upgrade beat schema v2 and parser with `characterHook`, `arcMove`, and nullable
  `relationshipMove` while preserving exact step coverage and 12-step batches.
- Merge authored fields into v2 snapshot steps without allowing changes to
  compiler-owned slots, payloads, or budgets.
- Extend timeout/batch regressions and Local Heuristic beat output.

Validation:

```bash
pnpm vitest run src/core/__tests__/episode-script.test.ts
```

Rollback point: full character-driven candidate passes snapshot/report checks.

## 4. Project Only Current Dramatic Direction at Runtime

- Extend `EpisodeActorBrief` and `actorBriefForStep()` with only the current
  character/arc/relationship move.
- Update prompt rendering to label these as current performance direction and
  keep visible facts/rules higher priority.
- Add prompt and server tests proving own profile + current move are present,
  while full cast directions, future moves, planned winner, and another player's
  profile are absent.
- Confirm schema-v1 execution still produces the legacy brief.

Validation:

```bash
pnpm vitest run \
  src/core/__tests__/prompt-builders.test.ts \
  src/core/__tests__/episode-script.test.ts \
  src/server/__tests__/game-actions.test.ts
```

Rollback point: approved v2 and historical v1 scripts both execute through the
existing Draft-generation path.

## 5. Add Read-Only Ensemble Review

- Extract a pure Review component for cast cards and relationship rows.
- Render it for schema-v2 review candidates using Game player snapshots for
  seat/name labels; show a compatibility message for v1.
- Keep whole-candidate Regenerate and Approve behavior unchanged and add render
  tests for all players, signature moments, relationship direction, and absence
  of field-level editing controls.

Validation:

```bash
pnpm vitest run src/components/script
```

Rollback point: Review UI can be removed without changing stored candidates or
runtime execution.

## 6. Full Quality Gate and Documentation

- Update `.trellis/spec/frontend/episode-script-workflows.md` with the v2
  dramaturgy, compatibility, validation, and runtime-projection contracts.
- Run the focused suites, full test suite, typecheck, production build, and diff
  check.
- Generate one real 12-character candidate and perform the manual quality rubric
  from `design.md`; record any provider-specific failure before approval.

Validation:

```bash
pnpm test
pnpm typecheck
pnpm build
git diff --check
```

## Review Gates

- Do not start UI work until one v2 candidate round-trips through repository
  validation.
- Do not approve implementation completion unless compiler-owned trace fields
  are unchanged and schema-v1 compatibility tests pass.
- If request size or provider timeout regresses, reduce profile/prior-move
  projection before increasing batch size or timeout.

## Completion Evidence

- `pnpm test`: 75 files, 630 tests passed.
- `pnpm typecheck`: passed.
- `pnpm build`: production build passed.
- `git diff --check`: passed.
- Server integration generated a 12-character schema-v2 candidate with the
  local author, persisted and reloaded it, approved it, and executed the locked
  trace through `game_ended`.
- External-provider dramatic quality remains a Director Review judgment; no
  subjective LLM critic was added.
