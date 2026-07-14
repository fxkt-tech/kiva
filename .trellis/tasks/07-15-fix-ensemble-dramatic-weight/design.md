# Script Author controlled progression — Technical Design

## Summary

Script Author keeps its deterministic `EpisodeAuthorTask` ordering, prompt
contracts, checkpointing, and final assembly. The execution boundary changes:
one deferred browser-triggered job completes at most one logical author task,
then either enters a durable `ready` state or transitions directly to `review`
when final assembly succeeds.

The Script page exposes a browser-local, default-off auto-continue toggle. A
`ready` page always offers the same guarded “生成下一步” Server Action manually;
when the preference is enabled, the client schedules that action after a short
delay. The browser preference decides whether to submit another job, while the
persisted game state remains the authority on whether a job is allowed.

## Core Author Boundary

Add a one-step core API beside the existing full-author API:

```ts
type AdvanceEpisodeAuthorResult =
  | {
      status: "ready";
      workspace: EpisodeAuthorWorkspace;
      requests: readonly EpisodeAuthorRequestRecord[];
      tokenUsage: LlmTokenUsage | null;
      repaired: boolean;
    }
  | {
      status: "complete";
      script: EpisodeScriptSnapshot;
      workspace: EpisodeAuthorWorkspace;
      requests: readonly EpisodeAuthorRequestRecord[];
      tokenUsage: LlmTokenUsage | null;
      repaired: boolean;
    };

advanceEpisodeAuthor(input): Promise<AdvanceEpisodeAuthorResult>;
```

- `advanceEpisodeAuthor()` validates the plan/workspace, derives the first
  incomplete `EpisodeAuthorTask`, and runs at most that one logical task.
- Length-truncated beat batches may still split into smaller requests inside
  the same logical task. This is bounded recovery, not an extra user-visible
  progression step; every child request remains checkpointed.
- After the task succeeds, the function derives the next task. It returns
  `ready` when work remains, or performs deterministic final assembly and
  returns `complete` when the workspace is complete.
- A complete workspace supplied after a process exit performs assembly without
  another provider call.
- `authorEpisodeScript()` remains the full-run compatibility API. It loops the
  same internal one-step mechanism until `complete`, preserving its current
  result and checkpoint contracts for core tests and non-HTTP callers.
- `EpisodeAuthoringError` continues to carry the latest validated workspace and
  all request records produced by the failed step.

## Persisted State Contract

Extend `EpisodeScriptState` with one exact validated variant:

```ts
{
  status: "ready";
  jobId: string;
  workspace: EpisodeAuthorWorkspace;
  requests: readonly EpisodeAuthorRequestRecord[];
}
```

`jobId` identifies the job that produced this exact ready checkpoint. It is
also the compare-and-swap token for submitting the next task. No automation
preference is stored in the record.

The state flow is:

```text
idle --manual start--> generating --one task--> ready
                                      |             |
                                      |             +--manual/auto next--> generating
                                      +--last task + assembly-------------> review
                                      +--error----------------------------> failed

failed --manual retry--> generating
review --manual regenerate, fresh workspace--> generating
generating --manual guarded restart--> generating with a new jobId
```

- `ready`, `failed`, and `generating` workspaces resume only when their
  `inputHash` matches the current game input.
- Request history stays append-only across task runs, failures, retries, and
  whole-candidate regeneration, matching current behavior.
- `review` and `approved` retain their current snapshot/report contracts.

## Guarded Start and Server Action

Change the start boundary to a compare-and-swap operation:

```ts
startEpisodeScriptGeneration(
  gameId: GameId,
  expectedJobId: string | null,
): Promise<string | null>;
```

- `expectedJobId === null` matches only `idle`.
- Every other authorable state must expose a `jobId`, and the submitted value
  must match it under the game lock.
- A match persists `generating` with a fresh job ID and returns that ID.
- A stale or duplicate submission returns `null` without mutating the record.
  The Server Action registers `after(runEpisodeScriptGeneration)` only for a
  non-null result.
- `approved`, active-event, non-scripted, and other existing invalid states
  remain hard errors rather than stale no-ops.

Every manual start, next, retry, restart, or regenerate form calls the same
Server Action with the state identity rendered by its page. The auto-continue
client calls that exact action with the `ready.jobId`. Two timers/tabs or a
manual and automatic submission can race, but only the first matching
transition receives a new job ID; later calls are harmless no-ops.

`runEpisodeScriptGeneration()` calls `advanceEpisodeAuthor()` once. Its guarded
checkpoint writes remain unchanged. Under the final game lock it writes
`ready` for a step result or `review` for a complete result. The synchronous
`generateEpisodeScript()` wrapper repeatedly performs guarded start/run cycles
until it reaches `review` or `failed`, keeping the existing complete-generation
contract used by tests and non-HTTP callers.

## Script UI and Browser Preference

Add a small client control in the Script workspace header using the Editor
Draft button's existing icon-button styles and on/off icon convention.

- Storage key: `kiva:auto-continue-episode-author`.
- Only the literal stored value `"true"` enables it; missing or unknown values
  mean false, so hydration starts safely in the off state.
- The preference is browser-global across games, independent of
  `kiva:auto-confirm-draft`, and does not require cross-tab `storage` syncing.
- The control stays available while the Script page is idle, generating,
  ready, failed, or in review, so a user can change the preference during an
  in-flight provider call. It schedules work only when `ready=true` and a
  current expected job ID is present.
- A 1-second timer gives the user a visible stopping window. Disabling the
  control, changing the ready job ID, leaving ready state, or unmounting clears
  the timer.
- Enabling it on a ready page schedules the next task. Opening or refreshing a
  ready page with the preference already enabled does the same.
- Idle, failed, and review states never schedule an action. Their existing
  manual start/retry/regenerate controls remain the only trigger.
- Dynamic `aria-label`, `aria-pressed`, and `title` expose the state without
  relying on color or tooltip alone.

The new `ready` branch renders persisted progress and request history without a
spinner, explains that the current step is complete, and shows “生成下一步”. The
existing `EpisodeGeneratingRefresh` continues polling only while a job is
running; its refresh reveals either `ready`, `failed`, or `review`, after which
the auto component decides whether another submission is warranted.

## Failure and Recovery

- Turning auto-continue off never attempts to cancel the current provider
  request. Its checkpoint and terminal `ready`/`review`/`failed` write finish
  normally, but no next timer is created.
- Turning it on during a running task causes the eventual `ready` state to
  schedule the next task; a terminal failure still requires manual retry.
- Provider and validation failures keep the current failed-state semantics,
  workspace, error, and append-only request history.
- A stale job still exits before its first provider call or at its next guarded
  write, so restarting a hung task cannot let the old runner overwrite newer
  progress.

## Compatibility, Rollout, and Rollback

- Existing `idle`, `generating`, `review`, `approved`, and `failed` records
  continue to decode. The additive `ready` variant uses the existing game
  record schema version, consistent with the current strict state-union
  validation approach; no eager data rewrite is needed.
- Existing persisted generating/failed work can resume from its current
  workspace under the new one-task runner.
- The browser preference is absent for existing users, so their first task
  stops in `ready` by default as requested.
- A code rollback must retain read/render compatibility for `ready` records or
  first convert them to an equivalent manually resumable failed state. Older
  code that does not recognize the new strict union member cannot safely read a
  `ready` record; this is the only data rollback consideration.
- The existing user-owned records under `kivdb/games/` are not edited as part
  of implementation or tests.

## Test Strategy

- Core tests prove one-step execution makes exactly one semantic task's worth
  of progress, reports `ready`, and final work reports `complete`; the existing
  full-run author tests remain green.
- State decoder tests round-trip `ready` and reject missing/extra keys.
- Game-action tests cover idle-to-ready, repeated ready-to-ready advancement,
  final review, failed resume, full synchronous generation, stale expected job
  IDs, duplicate submissions, and zero provider calls for a stale runner.
- Pure client-contract tests cover default-off parsing and scheduling only for
  `enabled && ready`.
- Script rendering tests cover the toggle's accessible off state, ready-state
  progress/history/manual action, generating alignment, and unchanged manual
  idle/failed/review actions.
- Full typecheck, tests, production build, and whitespace validation protect
  cross-layer compatibility.
