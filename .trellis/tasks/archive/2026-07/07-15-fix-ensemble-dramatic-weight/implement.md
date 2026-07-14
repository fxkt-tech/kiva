# Script Author controlled progression — Implementation Plan

## 1. Lock the State and One-Step Core Contracts

- Add failing tests for the exact `ready` state decoder shape.
- Extract a one-task `advanceEpisodeAuthor()` boundary from the existing author
  loop without duplicating task selection, prompts, repair, or checkpoint code.
- Test `ready` after one semantic task, direct completion after the final task,
  complete-workspace assembly without another LLM call, and unchanged
  `authorEpisodeScript()` full-run behavior.
- Preserve internal beat splitting as bounded recovery inside one semantic
  advance.

## 2. Make Each Background Job Advance Once

- Extend resumable workspace selection to `ready`.
- Make `startEpisodeScriptGeneration()` accept the rendered expected job ID,
  compare it under the game lock, and return `null` for a stale/duplicate
  submission.
- Change `runEpisodeScriptGeneration()` to call the one-step core API and write
  either `ready` or `review` under the existing job guard.
- Keep failure checkpointing and append-only request history unchanged.
- Retain `generateEpisodeScript()` as a full synchronous wrapper by looping
  guarded start/run transitions until terminal review/failed state.
- Add game-action regressions for stepwise progress, completion, stale calls,
  duplicate submissions, retry/resume, and current full-generation behavior.

## 3. Guard the Shared Server Action

- Update `generateEpisodeScriptAction()` to accept `expectedJobId: string |
  null`, call the guarded start method, and register the deferred runner only
  when a fresh job ID is returned.
- Pass each rendered state's current identity from the idle, ready, failed,
  generating-restart, and review-regenerate forms.
- Search every action/method callsite and update tests/types together; do not
  add a second automatic-only mutation path.

## 4. Add the Default-Off Auto-Continue Control

- Add a Script-specific client component with the independent
  `kiva:auto-continue-episode-author` localStorage key, strict preference
  parser, 1-second delay, and pure scheduling predicate.
- Reuse the Editor icon-button styles and enabled/disabled icon convention with
  dynamic `aria-label`, `aria-pressed`, and `title`.
- Schedule the shared generation action only for an enabled `ready` snapshot;
  clear the timer when disabled, identity/state changes, or the component
  unmounts.
- Place the stable control in the Script workspace header so it remains usable
  during an in-flight task. Do not schedule idle start, failed retry, or review
  regeneration.
- Add focused preference/scheduling tests and a static accessibility rendering
  assertion.

## 5. Render the Durable Ready State

- Add the `EpisodeWorkspace` ready branch with persisted next-task progress,
  request history, explanatory copy, and the manual “生成下一步” form.
- Retain the current left-aligned generating container and independently
  centered spinner.
- Verify the polling component stops naturally when refresh reveals ready,
  failed, or review.
- Extend Script page tests for idle, generating, ready, failed, and review
  controls without weakening the existing alignment regression.

## 6. Verify Cross-Layer Contracts and Update Specs

- Run focused core, server, client, and Script page tests while iterating.
- Review the full data flow from local preference to guarded Server Action to
  game lock to one-step author checkpoint and final persisted state.
- Update `.trellis/spec/frontend/episode-script-workflows.md` with the ready
  state, one-task deferred runner, guarded expected job ID, and browser-only
  automation contract after implementation is verified.
- Preserve all pre-existing user changes and exclude `kivdb/games/` from edits
  and any future commit.

## Validation Commands

```bash
pnpm test -- src/core/__tests__/episode-script.test.ts
pnpm test -- src/server/__tests__/game-actions.test.ts
pnpm test -- src/components/script/episode-auto-continue.test.ts 'src/app/games/[gameId]/script/page.test.tsx'
pnpm typecheck
pnpm test
pnpm build
git diff --check
```

If the final client test filename differs, use the implemented focused path
while retaining the same preference and scheduling coverage.

## Risk and Rollback Points

- `src/core/episode-author.ts` is the highest algorithmic risk: extracting the
  loop must not change task order, retry/repair behavior, prompt scope, or final
  assembly. Keep the full-run regression as the compatibility oracle.
- `src/server/game-actions.ts` is the concurrency boundary: expected job ID
  validation must happen inside `withGameLock`, and no deferred provider call
  may be registered for a null/stale start result.
- `src/core/episode-script.ts` is a persisted strict decoder. A rollback must
  account for already-written `ready` records.
- The Script client timer is advisory only; correctness must continue to come
  from the locked server transition.
- Keep rollback slices separable: core/state, server/action, then UI automation.
  Removing only the timer should leave safe manual one-step progression.

## Review Gate Before Implementation

- PRD has no unresolved product questions and records default-off, independent
  browser preference, explicit initial/retry/regenerate actions, and
  non-cancellation of in-flight requests.
- `design.md` defines the one-task core result, durable ready state, guarded
  expected-job transition, browser timer, and rollback implications.
- User reviews these artifacts before implementation resumes.
