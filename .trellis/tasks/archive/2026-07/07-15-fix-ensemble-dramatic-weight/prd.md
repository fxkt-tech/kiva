# Script Author reliability and controlled progression

## Goal

Prevent Script-page Episode authoring from failing at the Ensemble map because
the model invents an unsupported representation for `dramaticWeight`, and keep
the Script Author generating-state text left-aligned. Give users explicit
control over whether Script Author advances to the next authoring task: the
default is to stop after the current task, while an opt-in auto-continue control
can keep the workflow moving.

## Background

- `EpisodeCastAssignment.dramaticWeight` is a strict two-value contract:
  `"primary" | "supporting"` (`src/core/episode-script.ts:50`).
- The Ensemble validator correctly rejects every other value
  (`src/core/episode-author.ts:820`).
- Before this fix, the Ensemble prompt named the field without stating its
  legal values (`src/core/episode-author.ts:533`). The persisted failing request first
  returned Chinese labels and its one repair attempt returned numeric levels
  `1`, `2`, and `3`, ending with
  `castAssignments[0].dramaticWeight is invalid`.
- Provider JSON mode guarantees a JSON object but does not enforce a schema
  enum (`src/core/llm.ts:133`), so the natural-language output contract is the
  active boundary.
- Before the alignment fix, the generating-state root applied `text-center`,
  which was inherited by its progress copy and LLM request history
  (`src/app/games/[gameId]/script/page.tsx:201`). Other Script states use the
  normal left alignment.
- Script Author currently completes every remaining `EpisodeAuthorTask` inside
  one background invocation. Its checkpoint callback persists intermediate
  work, but the browser has no boundary at which it can choose whether to start
  the next task.
- The Editor Draft flow already establishes the desired interaction pattern:
  a browser-local, default-off automation button and a manual action share the
  same guarded transition. Disabling automation does not cancel an in-flight
  request; it prevents the following transition.

## Requirements

- The initial Ensemble request must state that `dramaticWeight` is exactly one
  of the string literals `"primary"` and `"supporting"`.
- The structural-repair output contract must repeat the same allowed values so
  an invalid first response can be repaired without guessing another encoding.
- Keep strict runtime validation. Do not accept or normalize Chinese labels,
  ordinal numbers, or a third dramatic-weight tier.
- Add a deterministic regression test at the Episode-author call seam that
  reproduces the observed failure when the enum contract is absent and passes
  when both legal values are present.
- Existing failed jobs remain recoverable through the current Resume flow,
  which starts again from the incomplete Ensemble task. Do not rewrite stored
  request history or game records.
- The Script Author generating state must explicitly left-align its textual
  content. The loading spinner may remain horizontally centered.
- Add a regression at the `EpisodeWorkspace` rendering seam for the generating
  state so the root cannot silently return to inherited center alignment.
- Add an automation toggle to the Script page using the same visual and
  accessibility conventions as the Editor Draft auto-confirm control.
- The Script automation toggle must default to off. Starting or resuming Script
  Author while it is off generates exactly one logical `EpisodeAuthorTask`,
  checkpoints it, and then waits for the user.
- While Script Author is waiting and more work remains, show a manual
  “生成下一步” action. Enabling automation submits that same guarded action and
  continues one task at a time until the Episode reaches review.
- Turning automation off during an in-flight task must let that task finish and
  persist normally, then stop before the next task. This feature does not abort
  a provider request already in progress.
- Persist enough workflow state to distinguish “one task is currently running”
  from “the latest task finished and the next task is ready”. Refreshing the
  page must preserve that distinction and must not silently advance work.
- Manual and automatic advancement must share the same Server Action and stale
  state guard so duplicate browser timers, clicks, refreshes, or old tabs cannot
  run the same next task concurrently.
- If one task fails, preserve its checkpoint and request history and expose the
  existing retry/resume behavior for that incomplete task. Only transition to
  review after final assembly succeeds.
- Store the automation preference in browser-local storage rather than the game
  record. It is one browser-level preference across games, matching the Editor
  precedent, but uses an independent Script key so enabling Editor Draft
  auto-confirm cannot implicitly enable Script auto-continue; only the visual
  and interaction conventions are shared.
- Keep initial generation, failed-task retry, and whole-candidate regeneration
  explicit. Auto-continue activates only after a task succeeds and persisted
  work is ready for the next task; it must not auto-start an idle game,
  auto-retry failures, or auto-regenerate a review candidate.

## Acceptance Criteria

- [x] The Ensemble request explicitly includes both exact string values
      `"primary"` and `"supporting"` next to the `dramaticWeight` field.
- [x] A repair request after an invalid `dramaticWeight` response includes the
      same two-value contract.
- [x] The regression test drives the real authoring/validation/repair path and
      no longer throws `castAssignments[0].dramaticWeight is invalid`.
- [x] The focused Episode-script tests and TypeScript typecheck pass.
- [x] The generating-state root uses `text-left`, does not use `text-center`,
      and retains the centered loading spinner.
- [x] The Script page rendering regression, full tests, typecheck, and build
      pass after the alignment change.
- [x] The Script automation toggle is off when no preference has been stored.
- [x] With automation off, starting or resuming authoring runs one logical task
      and then renders a durable waiting state with “生成下一步”.
- [x] Clicking “生成下一步” runs exactly the next logical task and cannot create
      duplicate concurrent work from a stale page or repeated submission.
- [x] Enabling automation advances through successive waiting states until the
      completed script reaches review.
- [x] Disabling automation while a task is running allows that task to finish
      but does not submit the following task.
- [x] Auto-continue never starts an idle game, retries a failed task, or
      regenerates a review candidate without the existing manual action.
- [x] Refreshing in a running, waiting, failed, or review state preserves the
      correct controls, progress, checkpoint, and request history.
- [x] Focused action/state/UI regressions, full tests, typecheck, and build pass.

## Out of Scope

- Replacing provider JSON-object mode with provider-specific JSON Schema.
- Changing the two-tier dramatic-weight domain model or UI labels.
- Mutating the existing failed game record under `kivdb/games/`.
- Redesigning Preview-stage title, participant-card, result, or subtitle
  alignment.
- Cancelling or interrupting an LLM provider request already in flight.
- Editing intermediate Script Author output before final review.

## Notes

- The original two fixes and the controlled-progression feature are complete.
  `design.md` and `implement.md` record the cross-layer state, action, UI, and
  validation decisions used by the implementation.
