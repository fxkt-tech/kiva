# Episode Script Workflows

## Scenario: Script Author Agent Builds, Approves, and Executes One Locked Plan

### 1. Scope / Trigger

- Trigger: a Game with `runMode: "scripted"` is created, authored, reviewed, approved, advanced, voiced, previewed, or exported.

### 2. Signatures

- `compileEpisodePlan(game): CompiledEpisodePlan`
- `createEpisodeAuthorWorkspace({ game, modelBinding? }): EpisodeAuthorWorkspace`
- `episodeAuthorProgress(workspace): { label, completed, total }`
- `advanceEpisodeAuthor({ game, llmClient, createdAt, modelBinding?, workspace?, onCheckpoint? }): Promise<AdvanceEpisodeAuthorResult>`
- `authorEpisodeScript({ game, llmClient, createdAt, modelBinding?, workspace?, onCheckpoint? }): Promise<AuthorEpisodeScriptResult>`
- `episodeActorProfile(player): EpisodeActorProfile`
- `episodeInputHashForScript(game, script): string`
- `assertEpisodeDramaturgy({ game, plan, castDirections, relationships }): void`
- `assertEpisodeScriptMatchesGame({ game, script }): void`
- `planNextEpisodeDraft({ game, events, script, draftId, createdAt })`
- `actorBriefForStep(script, stepIndex): EpisodeActorBrief | null`
- `<EpisodeEnsembleReview script={script} players={players} />`
- `<EpisodeAutoContinueButton gameId={gameId} expectedJobId={jobId} ready={ready} />`
- `gameActions.startEpisodeScriptGeneration(gameId, expectedJobId): Promise<string | null>`
- `gameActions.runEpisodeScriptGeneration(gameId, jobId): Promise<GameRecord>`
- `generateEpisodeScriptAction(gameId, expectedJobId)` submitted manually or automatically from `/games/:id/script`
- `gameActions.approveEpisodeScript(gameId, expectedJobId, expectedScriptId)`

```ts
type EpisodeAuthorTask =
  | { kind: "story" }
  | { kind: "ensemble" }
  | { kind: "actor_arc"; playerId: PlayerId }
  | { kind: "relationship"; playerIds: readonly [PlayerId, PlayerId] }
  | { kind: "beats"; stepIndexes: readonly number[] };

type EpisodeAuthorWorkspace = {
  inputHash: string;
  modelBinding: ModelBindingSnapshot;
  planStepCount: number;
  speechStepCount: number;
  story: EpisodeStorySpine | null;
  ensemble: EpisodeEnsembleMap | null;
  castDirections: readonly EpisodeCastDirection[];
  relationships: readonly EpisodeRelationshipDirection[];
  beats: readonly Omit<EpisodeSpeechBeat, "budget">[];
};

type EpisodeAuthorRequestRecord = {
  id: string;
  task: EpisodeAuthorTask;
  status: "success" | "failed";
  promptVersion: "episode-author:v4";
  provider: string;
  model: string;
  request: GenerationRequestSnapshot;
  tokenUsage: LlmTokenUsage | null;
  finishReason: string | null;
  rawOutput: string | null;
  parsedOutput: Record<string, unknown> | null;
  error: string | null;
  createdAt: string;
  attempts?: readonly GenerationAttemptSnapshot[];
};

type EpisodeAuthorCheckpoint = {
  workspace: EpisodeAuthorWorkspace;
  request: EpisodeAuthorRequestRecord;
  semanticTaskComplete: boolean;
  hasNextTask: boolean;
};

type AdvanceEpisodeAuthorResult =
  | {
      status: "ready";
      workspace: EpisodeAuthorWorkspace;
      requests: readonly EpisodeAuthorRequestRecord[];
      tokenUsage: LlmTokenUsage | null;
      repaired: boolean;
    }
  | ({ status: "complete" } & AuthorEpisodeScriptResult);

type EpisodeScriptReadyState = {
  status: "ready";
  jobId: string;
  workspace: EpisodeAuthorWorkspace;
  requests: readonly EpisodeAuthorRequestRecord[];
};
```

`generating`, `ready`, and `failed` Episode states require both `workspace` and `requests`. `review` and `approved` require `requests`. Pre-v4 Author requests and old workspace shapes are not decoded or migrated.

### 3. Contracts

#### Legal plan and final snapshot

- The compiler loops through the existing `planNextDraft()` and `confirmDraftEvent()` path from an empty event log to `game_ended`; it does not implement a second rules engine.
- Script Author receives the already-legal structural trace. The final Episode snapshot is schema v3 and contains `title`, `logline`, `acts`, exactly one `EpisodeCastDirection` per player, unique-pair relationship directions, and exactly one beat per speech step.
- Script Author cannot change actions, votes, deaths, winner, planned payloads, speech budgets, or stable step indexes. Final deterministic assembly attaches compiler-owned budgets and validates all dramaturgy before Review.
- Snapshot identity includes schema version, compiler version, exact Actor/Rule Role/rules/script input hash, stable step indexes, planned payloads, speech budgets, and author metadata. Random Draft/Event IDs never enter the plan.
- `episodeActorProfile()` projects `playerId`, seat, the compiled `ActorAuthorCard`, and the exact code-owned Rule Role snapshot. Portrait, voice, system-prompt text, and player model metadata do not enter authoring.
- Every cast direction contains `dramaticWeight`, `dramaticFunction`, `baseline`, `pressure`, `change`, `payoff`, and a signature moment whose step belongs to that player's deterministic `episodePerformanceOpportunities()` set.
- Relationships may describe `rivalry | alliance | contrast | trust_shift`, but never pre-game history, private deals, evidence, or facts visible to runtime actors.

#### One logical Agent, bounded child tasks

- Script Author is one durable application Agent represented by `EpisodeAuthorWorkspace`. It owns task ordering, partial results, retries, resume, and final assembly; it is not one giant LLM request and does not use accumulated chat history.
- The deterministic task order is `story -> ensemble -> one actor_arc per player -> one relationship per selected pair -> local scene beats -> final validation`.
- `advanceEpisodeAuthor()` executes at most the first incomplete semantic task, then returns `ready` if another task remains or assembles the final snapshot and returns `complete`. A length-truncated beat batch may split into bounded child requests inside that same semantic advance.
- `authorEpisodeScript()` is the full-run compatibility API and loops the same internal task boundary to completion; it must not maintain a second task implementation.
- `story` produces only the title, logline, and a spine of one to four acts.
- `ensemble` sees all concise profiles and performance opportunities, but produces only compact player assignments and one to six relationship seeds. It does not write full arcs.
- `EpisodeCastAssignment.dramaticWeight` is exactly the string `"primary" | "supporting"`. The OpenAI-compatible client requests JSON-object mode rather than a provider-enforced field schema, so both the initial Ensemble prompt and its structural-repair output contract must spell out those two literals from one shared instruction. Localized labels, ordinal numbers, and extra tiers remain invalid.
- Each `actor_arc` request sees one Actor profile with its real Rule Role, its assignment, the story spine, and only relationship seeds touching that player. It produces one complete cast direction.
- Each `relationship` request sees one selected pair, their completed directions, and the structural opportunities relevant to that pair. It produces one complete relationship direction.
- `beats` requests are grouped by local phase/scene and contain at most five speech steps. They receive only current actors, their completed direction, touching completed relationships, local structural context, and at most one earlier authored move per actor.
- A child prompt never embeds the full previous assistant output or the entire workspace. The workspace is structured application state; each prompt projects only the fields needed for its task.
- Every authored free-text field is instructed to stay within 80 Chinese characters. Structural validators, not output-token caps, decide whether a complete response is accepted.
- The Script Author model binding is snapshotted in the workspace from a dedicated code-owned default or an explicit author override. It is independent of every player's Actor model binding.

#### Persistence, recovery, and observability

- Before each provider call, a guarded start persists a fresh `jobId`, input-bound workspace, and retained request list. `expectedJobId === null` matches only `idle`; every other authorable state requires its exact current job ID under the Game lock. A stale or duplicate start returns `null` and schedules no runner.
- After every successful or failed child request, `onCheckpoint` persists the validated workspace and appended request under the Game lock. The checkpoint explicitly reports whether the full semantic task is complete and whether another task remains. When both are true, that same repository save writes `status: "ready"`; request success and READY must not be separate writes. A process exit can lose at most the in-flight provider call, never a completed task or its READY transition.
- One deferred `runEpisodeScriptGeneration()` invocation advances at most one semantic task. It persists `ready` when another task remains, or `review` only after the last task and deterministic assembly succeed.
- Starting from `ready`, or retrying `generating`/`failed`, with the same input hash resumes from the first incomplete deterministic task. It must not rerun completed story, ensemble, actor arc, relationship, or beat work.
- Every child call persists an `EpisodeAuthorRequestRecord` containing its semantic task, exact request, raw/parsed output, attempts, provider/model, status, provider finish reason, and provider-returned token usage.
- Request history is append-only across failures, resumes, and whole-candidate regeneration. Game token totals include every retained Script Author request separately from player speech/action requests.
- The Script page renders current Agent phase/progress plus persisted request details while generating, and shows preserved partial-result counts plus a resume action when failed.
- The Script page is a fixed-height tool workspace: the page root uses `h-screen overflow-hidden`, the game header occupies its own fixed row, and the remaining area is a `min-h-0` responsive grid. Desktop renders three bounded columns in order: the narrow `LLM requests` navigator, the widest `LLM Details`, then the right-side 「剧本控制台」 status/action panel. Narrow viewports stack the same three panels in that order. “Locked” describes the pre-approval Game invariant and warning copy, not the control panel's name.
- Status, progress, auto-continue, generate-next/retry/start actions, and immutable Game facts belong only to the control panel. Auto-continue plus the state-specific primary action render inside the shared current-status card footer, not as detached sibling cards below it. Review/approve content may remain below the status card because it acts on the assembled candidate rather than the current Author task.
- Persisted Script Author requests are projected once from the current Episode state and rendered only in the request/detail workspace; do not render request history inside `EpisodeWorkspace` state branches.
- `idle`, `generating`, `ready`, `failed`, and `review` share one `EpisodeWorkspaceStatusCard` visual skeleton. The card always renders the “当前状态” kicker, state badge, title/copy, phase progress bar, and semantic completion for all five request kinds: 故事主轴 (`story`), 群像分工 (`ensemble`), 角色弧线 (`actor_arc`), 关系弧线 (`relationship`), and 场景节拍 (`beats`). State branches may change copy, tone, and available actions, but must not replace the card with a spinner-only or ad-hoc status layout.
- Control-card phase completion is derived from structured workspace artifacts, not request-record counts: retries may append multiple audit requests for one semantic task and must not inflate progress. Total retained requests and tokens belong only to the left `LLM requests` audit header, rendered compactly as `<count> · <tokens> tokens` because the panel title already names the count.
- All three panel bodies own their vertical overflow with `min-h-0` plus `overflow-y-auto`. Request and detail headers remain visible, including when there are zero requests. `LLM requests` renders newest first as compact full-row selection buttons, defaults selection to the newest request, and uses `aria-pressed` for the active row. Each visible row contains only original sequence number, success/failure dot, chronological task label, and selection chevron—in that order. Provider/model/status/task metadata and per-request token counts do not render in the list; they belong to `LLM Details`. The status remains screen-reader text on the row.
- `LLM Details` renders only the selected request's shared Prompt/Token/output content inline; Script must not render a details icon or `<dialog>`. Editor and Timeline may continue using the same shared content inside their existing modal trigger.
- Request growth scrolls only `LLM requests`; long Prompt/output content scrolls only `LLM Details`. Neither may move the control panel or create document-level scrolling.
- The Script page renders `ready` without a spinner, shows the next deterministic phase and a manual “生成下一步” action, and does not silently advance when the browser preference is off.
- The Script page generating-state root explicitly uses `text-left`. In-flight work is indicated by the `GENERATING` badge and its pulsing status dot inside the shared card, not by replacing the card with a centered loading block.
- `GENERATING` means the current semantic task still has an in-flight or incomplete provider request. If a historical generating snapshot already contains a successful current-job request fully applied to its workspace, the page projects it as `READY`; auto-continue and the manual next-step action use that effective state without rewriting the stored audit record.
- The request audit renders newest first while retaining each record's original one-based sequence number and chronological run/batch label. The underlying append-only request array remains chronological.
- A generating or failed task exposes only “重试当前阶段”. It resumes from the persisted workspace and never claims to restart the whole Author run.
- Completion updates state only when `jobId` is still current. The deferred runner checks before its first provider call and again under the Game lock before every checkpoint or final state write.
- Manual and automatic next-step submissions call the same guarded Server Action. Browser timers are advisory; the expected job ID is the correctness boundary for duplicate clicks, timers, refreshes, and tabs.
- Script auto-continue uses the independent browser-global key `kiva:auto-continue-episode-author`. Only `"true"` enables it; missing/unknown values default off. An enabled `ready` snapshot waits one second before submission, and dependency changes or disabling clear the timer.
- Auto-continue never starts `idle`, retries `failed`, or regenerates `review`. Disabling during `generating` does not cancel the in-flight provider call; it prevents a timer after the resulting checkpoint becomes `ready`.

#### Provider truncation and repair

- `LlmGenerateJsonResult` and `LlmOutputParseError` preserve the provider `finishReason` and token usage even when assistant content cannot be parsed.
- When `finishReason === "length"`, `generateValidatedJson()` does not send the truncated JSON back for whole-object repair. It throws the structured failure with the first attempt intact.
- A length-truncated `beats` task is recursively split into smaller contiguous task batches. A single-step beat that still truncates fails the job.
- A length-truncated non-beat task receives at most one concise regeneration from the original bounded task input. The truncated raw output is retained for audit but never embedded in that retry.
- A complete JSON object with an invalid schema may receive the existing one minimal structural repair containing only the validation error and output contract.
- A provider `Headers Timeout Error` may retry once for the same bounded child task. Further transport failure persists failed state.

#### Review and execution

- `GameRecord.episodeScript` is `idle | generating | ready | review | approved | failed` for scripted games and `null` for game mode. Every state-specific key is required and validated exactly.
- New Game creation stops at `idle`; only the explicit Generate form on the Script page starts authoring.
- The browser Server Action compare-and-swaps the rendered expected job ID to `generating`, registers one-task `runEpisodeScriptGeneration()` with Next `after(() => ...)` only after a successful transition, and returns without awaiting the provider call. The synchronous wrapper repeats guarded start/run cycles for tests and non-HTTP callers.
- Approved execution derives the cursor from active-event count, calls the normal planner, compares the Draft slot, binds the approved payload, and validates it before confirmation.
- Structural Drafts are read-only in Editor and never call the player action model. Speech text remains editable/generatable within the approved step budget.
- Runtime player prompts receive only their own `ActorRuntimeCard` plus the current `EpisodeActorBrief.actorHook`, `arcMove`, and `relationshipMove`. They never receive the full ensemble, planned winner, future steps, or another Actor profile. Visible facts and rules outrank all dramatic direction.
- Script Review renders cast arcs, signature moments, and relationship setup/development/payoff read-only. MVP correction is whole-candidate regeneration, not field-level dramaturgy editing.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|---|---|
| Dry-run does not reach `game_ended` within 240 steps | Fail before the first author request. |
| Workspace input hash or plan/speech count differs | Reject resume; do not mix partial work from another plan. |
| Story has more than four acts, ensemble has more than six relationship seeds, or a free-text field exceeds 80 characters | Reject as unbounded output and use the one minimal structural repair. |
| Story/ensemble/actor_arc/relationship output is incomplete or otherwise invalid | Use one minimal structural repair for complete JSON; otherwise fail the bounded task. |
| Ensemble `dramaticWeight` is localized, numeric, or neither `"primary"` nor `"supporting"` | Repair once with the exact two-value string contract; fail the bounded task if the repaired value is still invalid. |
| Cast direction omits/duplicates a player or has an invalid signature step | Fail before Review. |
| Relationship references an unknown/same player or duplicates an undirected pair | Fail before Review. |
| Beat output omits/duplicates a requested speech step | Repair complete JSON once; fail if still invalid. |
| A beat request asks for more than five steps or crosses the local phase | Regression failure; partition before calling the provider. |
| Provider returns malformed JSON with `finishReason=length` | Do not repair the whole JSON; split beats or do one concise original-input retry. |
| Provider omits token usage or finish reason | Persist `null`; never estimate or invent provider metadata. |
| Checkpoint belongs to an obsolete job | Abort without writing the workspace or final state. |
| Submitted expected job ID is stale, duplicated, or null outside `idle` | Return `null`, do not mutate state, and do not register a deferred runner. |
| Process exits after a checkpoint | Retry resumes from persisted workspace and skips completed tasks. |
| Author transport/output fails after bounded retry | Persist failed state with workspace and actionable error; allow resume. |
| One semantic task succeeds and more work remains | Persist exact `ready` state with its producing job ID, workspace, and append-only requests. |
| Process exits immediately after a successful task checkpoint | The already-saved record remains `ready`, because request append and status transition are atomic. |
| Auto-continue preference is absent, false, or invalid | Remain in `ready` until the manual next-step action is submitted. |
| Auto-continue is disabled while a task is in flight | Let the current task reach `ready`, `review`, or `failed`; never submit the following task. |
| Script generating state replaces the shared status card with a spinner-only block | Regression failure; keep the same badge/progress/metrics structure used by `ready`, with `GENERATING` state copy and tone. |
| Script request history grows beyond the viewport | Scroll only the `LLM requests` panel body; keep the page root, all panel headers, details selection, and right controls fixed. |
| Selected Prompt/output exceeds the viewport | Scroll only the `LLM Details` body; never open a modal or grow the document. |
| Episode state has no persisted requests yet | Render `LLM requests` with `0 requests · 0 tokens` and a separate empty `LLM Details` panel; do not remove either workspace column. |
| Historical `generating` snapshot contains a successful current-job request already applied to its workspace | Project it as `READY`; do not show `GENERATING` or a current-stage retry action. |
| A semantic task has failed and been retried | Count its completed workspace artifact once in the control card; retain every attempt in the left request audit. |
| Approval identity/profile/hash changed, compiler-owned field differs, or events exist | Reject approval. |
| Episode snapshot is not schema v3 | Reject; do not normalize or upgrade. |
| Episode author request is not v4 or workspace has an old shape | Reject; no compatibility path exists. |
| Runtime slot or planned payload differs | Stop with episode divergence; do not choose an alternative. |

### 5. Good / Base / Bad Cases

- Good: Agent authors a compact story, assigns all 12 players, completes one arc at a time, writes local scene beats, checkpoints each result, and assembles the unchanged legal plan.
- Base: local deterministic author produces the same complete schema-v3 Episode snapshot through the same v4 task pipeline without external credentials.
- Good controlled progression: one guarded browser job completes story and persists `ready`; a manual click or enabled timer submits the matching job ID and advances ensemble exactly once.
- Base preference: a browser with no Script preference stops after every successful task and keeps initial start, failure retry, and candidate regeneration manual.
- Good recovery: the provider truncates a five-beat response; the Agent records that request, splits it into smaller batches, and continues without resending the truncated document.
- Good workspace: thirty persisted requests scroll inside the narrow left audit panel, clicking request 12 replaces request 30 in the wide middle inline details panel, and the right status/actions remain reachable without document scrolling.
- Good audit order: request 30 appears above request 29 while both keep their original sequence numbers and labels.
- Good phase summary: the right control card shows `story`, `ensemble`, `actor_arc`, `relationship`, and `beats` completion while a failed Ensemble attempt remains visible only as an extra left-side audit record.
- Bad: ask for the whole script, 12 full arcs, all relationships, and all beats in one response; append the malformed 8 KB output to a repair request; or restart all prior work after one late failure.
- Bad: bind Script Author to seat 1's Actor model, keep only aggregate token counts, or replace request history when generating another candidate.
- Bad: await the Agent run inside the browser Server Action or treat an in-memory promise as durable job state.
- Bad: let the timer call a separate unguarded endpoint, store auto-continue in the Game record, share the Editor auto-confirm key, or use `generating` to mean both in-flight and waiting.
- Bad: append `EpisodeAuthorRequests` inside every status branch, allow the document to grow with request count, or hide the audit panel when the list is empty.
- Bad: keep a details icon on every request and open a modal, render every request's hidden details DOM, or detach auto-continue/current-stage actions from the status card.
- Bad: label retained-request count as a Script Author phase or use request counts as completion, because retries then make semantic progress exceed its target.
- Bad: append a successful request under `generating`, then rely on a second save to transition to `ready`; a process interruption between those writes leaves a false GENERATING badge.

### 6. Tests Required

- Two compilations of one Game yield identical slots/payloads and end in `game_ended`.
- Task-capture tests assert one bounded story request, one compact ensemble request, one actor_arc request per player, one relationship request per seed, and local beat batches of at most five.
- An Ensemble contract regression makes the first response use localized weights and makes repair fall back to numeric weights unless the request contains both exact string literals; assert the initial and repair requests share the contract and the repaired task succeeds.
- Cardinality tests reject more than four acts, more than six relationship seeds, or a free-text field longer than 80 characters before that output can enter later prompts.
- Prompt-scope tests assert every child request has only the profiles/directions/local history required by its task and at most one prior move per actor.
- Binding tests prove Script Author uses its dedicated snapshot and does not inherit any player's provider/model.
- Truncation tests preserve raw output, finish reason, and usage; skip whole-JSON repair; split beat batches; and perform only one concise original-input retry for non-beat tasks.
- Checkpoint integration tests fail after partial actor-arc completion, persist workspace/request history, resume the same job, and skip every completed task.
- A simulated process interruption immediately after the first successful checkpoint still reloads `ready` with the completed workspace and request record.
- One-step core tests prove story and ensemble advance separately, while a complete workspace assembles without another provider call and the full-run API remains compatible.
- Ready-state decoder tests round-trip only the exact `status`, `jobId`, `workspace`, and `requests` keys.
- Guarded action tests prove duplicate/stale starts return `null`, old runners make zero provider calls when already stale, request history appends across ready steps, and the synchronous wrapper still reaches review.
- Request records round-trip exact tasks, prompts, outputs, attempts, finish reasons, and token usage; failure plus resume retains all spend.
- Local author covers every cast member and speech step; invalid cast IDs, pairs, signature indexes, hash/schema/identity fail deterministically.
- Script page tests render Agent progress, per-task request details, preserved partial-result counts, and resume copy.
- Auto-continue tests assert only stored `"true"` enables the control and scheduling requires enabled + ready + a current job ID. Script rendering tests assert the off-state accessibility label and the durable manual ready action.
- Ready- and generating-state rendering tests assert the shared “当前状态” card, exact state badge, progressbar, all five semantic request-kind labels, and artifact-derived completion values. They also assert “已保留请求” is absent from the control card. Generating remains left-aligned and does not render the old centered spinner block.
- Script request/detail rendering tests assert newest-first buttons, newest default selection, one inline selected detail payload, `aria-pressed`, no `<dialog>`, and separate zero-request placeholders. Row assertions require the status dot before the visible label and reject provider/model metadata or per-request tokens inside the button. Browser checks click an older row and assert the inline detail title/content changes. Manual viewport checks cover desktop three-column and narrow three-panel stacked layouts with panel-local scrolling.
- Ready-, generating-, failed-, and idle-state rendering tests assert auto-continue and the applicable start/next/retry action are descendants of the shared current-status card.
- Script rendering tests assert newest-first request order, effective READY for a settled historical generating snapshot, and “重试当前阶段” without any whole-run restart copy.
- Creation/action tests require idle creation, deferred execution, stale-job zero-call exit, and approval identity checks.
- Prompt tests assert only the current Actor Brief reaches player generation and future/full ensemble data does not.
- Full `pnpm typecheck`, `pnpm test`, `pnpm build`, and `git diff --check` pass.

### 7. Wrong vs Correct

Wrong: `generateJson({ prompt: fullPlan + allProfiles + everyRequirement })`, then on parse failure send `fullPlan + truncatedOutput + "please repair"`.

Correct: persist one `EpisodeAuthorWorkspace`, derive the next deterministic bounded task, project only task-local context, checkpoint its result, and assemble the final snapshot in code.

Wrong: infer recovery from request history text or begin a fresh candidate after every task failure.

Correct: validate the structured workspace against the current plan hash, find the first missing task, and resume from there while retaining the append-only audit log.

Wrong: persist a successful checkpoint as `generating`, return from the provider call, then perform a second save for `ready`.

Correct: mark terminal semantic checkpoints with `semanticTaskComplete`/`hasNextTask` and atomically append the request, workspace, and READY transition in one locked save.

Wrong: treat every malformed assistant response as repairable JSON.

Correct: branch on provider finish reason. Repair a complete but structurally invalid object once; split or concisely regenerate when the provider reports length truncation.

Wrong: name `dramaticWeight` in the Ensemble shape and assume `schemaName` or JSON-object mode tells the provider which values are legal.

Correct: render the shared `dramaticWeight` instruction with the exact strings `"primary"` and `"supporting"` in both the initial request and structural-repair contract, while keeping runtime validation strict.

Wrong: make `generating` a centered spinner card while `ready` uses the approved status-card hierarchy.

Correct: render both states through `EpisodeWorkspaceStatusCard`; communicate in-flight work with the badge tone/dot while retaining phase progress and persisted-result metrics.

Wrong: show `requests.length` as a control-card phase metric or derive phase completion by counting `request.task.kind` records.

Correct: derive all five phase values from `EpisodeAuthorWorkspace`; reserve `requests.length` and token totals for the left audit panel where retries are intentionally preserved.

Wrong: render request history inside `idle`, `generating`, `ready`, `failed`, and `review` branches, then open each request's details in a modal.

Correct: project requests once at the page boundary, render one narrow selectable left audit panel plus one wide middle inline detail panel beside the right control panel, and give each panel body its own bounded overflow.

Wrong: render auto-continue, “生成下一步”, or “重试当前阶段” as separate cards below current status.

Correct: pass the state-specific controls into `EpisodeWorkspaceStatusCard` and render them together in its bordered footer so status and available action remain one unit.

Wrong: let one deferred browser job loop through every remaining `EpisodeAuthorTask`, then attempt to stop that server-side loop with a browser-local toggle.

Correct: persist `ready` after one semantic task and let the browser choose whether to submit the same expected-job guarded action for the next task.

Wrong: trust timer cleanup alone to prevent two tabs or a manual/automatic race from advancing twice.

Correct: compare the rendered expected job ID under the Game lock, return `null` for stale submissions, and schedule a runner only after a fresh `generating` state is persisted.
