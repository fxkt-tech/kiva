# Episode Script Workflows

## Scenario: Scripted Games Author, Approve, and Execute One Locked Plan

### 1. Scope / Trigger

- Trigger: a Game with `runMode: "scripted"` is created, authored, reviewed, approved, advanced, voiced, previewed, or exported.

### 2. Signatures

- `compileEpisodePlan(game): CompiledEpisodePlan`
- `episodeCharacterProfile(player): EpisodeCharacterProfile`
- `episodeInputHashForScript(game, script): string`
- `authorEpisodeScript({ game, llmClient, createdAt }): Promise<AuthorEpisodeScriptResult>`
- `assertEpisodeDramaturgy({ game, plan, castDirections, relationships }): void`
- `assertEpisodeScriptMatchesGame({ game, script }): void`
- `planNextEpisodeDraft({ game, events, script, draftId, createdAt })`
- `actorBriefForStep(script, stepIndex): EpisodeActorBrief | null`
- `<EpisodeEnsembleReview script={script} players={players} />`
- `gameActions.generateEpisodeScript(gameId)`
- `gameActions.approveEpisodeScript(gameId, expectedJobId, expectedScriptId)`

### 3. Contracts

- The compiler loops through the existing `planNextDraft()` and `confirmDraftEvent()` path from an empty event log to `game_ended`; it does not implement a second rules engine.
- Script Author receives the already-legal structural trace. Schema v2 authors `title`, `logline`, `acts`, one `EpisodeCastDirection` per player, zero or more unique-pair `EpisodeRelationshipDirection` records, and one beat per speech step. It cannot change actions, votes, deaths, winner, planned payloads, or speech budgets.
- `episodeCharacterProfile()` projects only `playerId`, seat, name, role, `persona`, `speakingStyle`, and `reasoningStyle`. When all three structured fields are blank it projects the legacy character prompt; avatar, voice, and model metadata do not enter authoring.
- Every cast direction contains `dramaticWeight`, `dramaticFunction`, `baseline`, `pressure`, `change`, `payoff`, and a signature moment whose step index belongs to that player's deterministic `episodePerformanceOpportunities()` set. Primary/supporting weight may differ, but every player has one direction and one real signature opportunity.
- Relationships are dramaturgy emerging inside this game. They may describe `rivalry | alliance | contrast | trust_shift`, but never pre-game history, private deals, evidence, or facts visible to runtime actors.
- Authoring is never one giant output request. It generates one compact outline from all concise profiles plus structural milestones, then requests speech beats in batches of at most 12 using a bounded local trace window. Each beat batch receives only current-batch actor profiles/directions, touching relationships, and at most the latest two earlier moves per actor.
- Schema-v2 speech beats add nonblank `characterHook` and `arcMove` plus nullable `relationshipMove`. These are current performance direction, not final dialogue; final speech still uses the actor's then-visible facts at runtime.
- Each bounded request may retry one provider `Headers Timeout Error`. A second timeout persists failed state; retrying the job starts a new candidate identity.
- New snapshots write schema v2. Their input hash includes the exact projected character profiles, rules, and Game Script. Historical schema-v1 snapshots decode with empty cast/relationship arrays, null character-beat additions, and the legacy hash recipe; reading does not rewrite them.
- Snapshot identity includes schema version, compiler version, cast/rules/theme input hash, stable step indexes, planned payloads, speech budgets, and author metadata. Random Draft/Event IDs never enter the plan.
- `GameRecord.episodeScript` is `idle | generating | review | approved | failed` for scripted games and `null` for game mode. Historical scripted records without state normalize to idle.
- Authoring writes a new job identity before the LLM call. Completion updates state only when that job is still current. Approval carries both job ID and candidate script ID.
- Approved execution derives the cursor from active-event count, calls the normal planner, compares the Draft slot, binds the approved payload, and validates it again before confirmation.
- Structural Drafts are read-only in Editor and never call the player action model. Speech text remains editable/generatable within the approved step budget.
- Approval calls `assertEpisodeScriptMatchesGame()`, which recompiles the legal trace and checks winner, day count, duration, every slot/payload/budget, schema-aware input identity, cast coverage, relationship references, and signature opportunities.
- Player prompts receive only their own snapshotted character profile plus the current `EpisodeActorBrief.characterHook`, `arcMove`, and `relationshipMove`; they never receive the full cast direction, planned winner, future steps, or another player's profile. Visible facts/rules outrank all dramatic direction.
- Script Review renders cast arcs, signature moments, and relationship setup/development/payoff read-only. MVP correction is whole-candidate regeneration, never field-level dramaturgy editing.

Schema-v2 direction fields are compiler-adjacent but never compiler-owned:

```ts
type EpisodeSpeechBeatV2 = {
  characterHook: string;
  arcMove: string;
  relationshipMove: string | null;
};
```

### 4. Validation & Error Matrix

| Condition | Required behavior |
|---|---|
| Dry-run does not reach `game_ended` within 240 steps | Fail authoring; do not create Review state. |
| Author output omits/duplicates a speech step | Use one structural repair; fail if still invalid. |
| Cast direction omits/duplicates a player | Use one structural repair; fail before Review if still invalid. |
| Signature step is not in that player's opportunity set | Use one structural repair; fail before Review if still invalid. |
| Relationship references an unknown/same player or repeats an undirected pair | Use one structural repair; fail before Review if still invalid. |
| V2 speech beat lacks `characterHook` or `arcMove` | Use one structural repair; fail before Review if still invalid. |
| Target duration exceeds 35 minutes | Report invalid; Approve rejects. |
| Author transport/output fails | Persist failed state with actionable error; allow full retry. |
| A request asks for more than 12 speech beats | Regression failure; split the batch before calling the provider. |
| Old job finishes after a newer job starts | Return current state; never overwrite it. |
| Approval identity/profile/hash changed, compiler-owned field differs, or events already exist | Reject approval. |
| Schema-v1 snapshot lacks v2 direction fields | Normalize empty/null compatibility fields and continue with the legacy hash recipe. |
| Runtime slot or planned payload differs | Stop with episode divergence; do not select an alternative. |
| Structural edit is submitted in scripted mode | Reject; only speech `text` may be edited. |
| Voice/Preview/export requested before approval | Reject or route back to Script Review. |

### 5. Good / Base / Bad Cases

- Good: New Game assigns complementary primary/supporting functions from all 12 profiles, gives everyone a valid signature moment, develops central relationships across beats, and executes the unchanged legal plan after director approval.
- Base: no external LLM credentials are present, so the local Script Author deterministically produces structurally complete v2 ensemble direction over the same legal trace.
- Bad: mention an actor ID without its profile in a beat batch, restart its arc in every batch, expose a future payoff to the runtime actor, or alter the trace to manufacture equal screen time.

### 6. Tests Required

- Two compilations of one Game yield identical slots/payloads and end in `game_ended`.
- Captured outline input contains every exact concise profile and valid performance-opportunity list; changing an authoring-relevant profile changes the v2 hash.
- Local author covers every cast member and speech step; cast IDs, relationship pairs, signature indexes, character/arc fields, and invalid hash/schema/identity fail deterministically.
- Authored output preserves every compiled slot, planned payload, winner, and speech budget byte-for-byte.
- Provider-shaped regression throws the exact Headers Timeout error when a request exceeds 12 beats and proves the author completes without producing such a request.
- Beat-request capture asserts all current actors have profiles/directions and no actor has more than two prior moves in one request.
- Server integration round-trips v2 through repository state, rejects stale approval, character-profile changes, and structural edits, and executes the entire approved trace to terminal state.
- Schema-v1 decode/execution tests assert empty ensemble arrays and null v2 beat additions.
- Prompt tests assert own profile plus current Actor Brief direction and absence of planned winner/full script/another player's profile.
- Review render tests assert all cast/relationship direction is visible and no input, textarea, or field-level button exists.
- Editor tests hide action regeneration/fields for locked structural Drafts.
- Voice and export tests reject unapproved scripted records.
- Full `pnpm test`, `pnpm typecheck`, `pnpm build`, and `git diff --check` pass.

### 7. Wrong vs Correct

Wrong: ask the LLM to invent a hundred-event game, send only names/roles and hope runtime tone creates a coherent ensemble, or place the whole future arc in every actor prompt.

Correct: compile a legal trace with the existing planner, author global ensemble direction from immutable profiles and real opportunities, project only the current move into runtime, and derive position from active events.
