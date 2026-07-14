# Content Catalog and Library Studio Workflows

## Scenario: Content Authors Edit Actors Without Editing Game Mechanics

### 1. Scope / Trigger

- Trigger: adding, duplicating, editing, enabling, diagnosing, or previewing an Actor, Lineup, Script, or Presenter in Library Studio.
- Trigger: changing Actor fields, compiler projections, catalog cross-references, or Rule Role presentation.

### 2. Signatures

```ts
validateActorDefinitions(value: unknown): readonly ActorDefinition[]
createActorSnapshot(actor: ActorDefinition): ActorSnapshot
compileActorRuntimeCard(actor): ActorRuntimeCard
compileActorAuthorCard(actor): ActorAuthorCard
diagnoseActorPool(actors): ActorPoolDiagnostics
diagnoseActorSelection({ actors, actorIds }): ActorSelectionDiagnostics
compileActorComparisonMatrix(actors): readonly ActorComparisonRow[]

type ContentCatalogSnapshot = {
  actors: readonly ActorDefinition[];
  lineups: readonly Lineup[];
  presenters: readonly PresenterDefinition[];
  scripts: readonly GameScriptDefinition[];
  ruleRoles: readonly RuleRole[]; // code-owned, read-only
};
```

Actor Definition v2 has exact nested sections: `identity`, `core`, `cognition`, `interaction`, `expression`, and `production`, plus `enabled` and positive `revision`.

### 3. Contracts

- Actor is a reusable person; Rule Role is a game mechanic. No field or UI label may use `role` to mean personality.
- Actor content is structured behavior, not a raw system prompt. Removed fields (`persona`, `speakingStyle`, `reasoningStyle`, `systemPrompt`) are not decoded or aliased.
- Actor interaction/conflict fields are reusable interfaces. They cannot name another Actor ID/name or encode a permanent alliance, enemy, faction, or pre-game relationship.
- `ActorRuntimeCard` contains only fields needed for the current player's decisions/expression. `ActorAuthorCard` contains only fields needed for ensemble/arc design. Author cards exclude model, voice, portrait, low-level prompt text, and future runtime facts.
- Actor production is the only owner of player model binding and voice. Rule Roles and Lineups contain no production override.
- The original Qin Chuan Actor is exactly `qin_chuan` / `秦川`; no alias may claim either half of that identity. Copies are ordinary new Actors with new IDs/names.
- The editable catalog must have at least 12 enabled Actors, at least six distinct enabled Actor voices, exactly one enabled Presenter, and at least one enabled Script. Near-duplicate behavior is a diagnostic, not a hard Game rule.
- Studio tabs are Actors, Lineups, Scripts, Show, and read-only Rules. Preview data must call production compilers and diagnostics; UI code does not rebuild prompt/card projections.
- Stable IDs are immutable in editors. Actor/Lineup writes use optimistic revisions inside `ContentCatalog.update()`.
- Disabling content cannot leave an enabled Lineup referencing an unavailable Actor. Existing Game snapshots are unaffected by all Library edits.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|---|---|
| Actor object has missing, extra, blank, over-limit, or duplicate list fields | Reject the complete catalog write. |
| `qin_chuan` is renamed or `秦川` is assigned to another ID | Reject. |
| Actor text names another Actor ID/name | Reject the catalog dependency. |
| Model binding is absent or configures temperature/maxTokens | Reject; there is no silent fallback. |
| Voice profile is incomplete/invalid | Reject. |
| Fewer than 12 enabled Actors or six distinct voices | Reject as not production-ready. |
| Near-duplicate behavior exceeds the diagnostic threshold | Report the pair/score; do not invalidate a legal Game. |
| Lineup references an unknown/disabled/repeated Actor | Reject the Lineup/catalog. |
| Client submits a stale Actor/Lineup revision | Reject without overwriting the newer edit. |
| User attempts to edit/create a Rule Role in Studio | No write path exists; render the code registry read-only. |

### 5. Good / Base / Bad Cases

- Good: add a new Actor with a distinct table function/voice, inspect both compiled cards, then choose it in a 12-person Lineup.
- Good: duplicate Qin Chuan; the original remains exact while the copy receives a new ID and display name and no special treatment.
- Base: a valid Actor shares one conflict axis with another Actor but differs in cognition, pressure response, cadence, and voice.
- Bad: store a prose system prompt that mixes mechanics, personality, secrets, and output formatting.
- Bad: add an editable “Role prompt” row for an unsupported mechanic.
- Bad: compute a Library-only fake prompt preview that runtime never uses.

### 6. Tests Required

- Exact Actor v2 decoder/snapshot tests, including unknown keys, content bounds, list uniqueness, model/voice validation, and Qin identity invariants.
- Runtime/Author card tests prove field inclusion/exclusion and deep snapshot isolation.
- Named-dependency and near-duplicate diagnostics tests.
- Pool/selection tests cover 12 enabled Actors, six voices, exact selected 12, uniqueness, disabled/unknown references, and soft diversity reporting.
- Content action tests cover add/upsert, duplicate, enable/disable constraints, optimistic revision conflicts, and concurrent updates.
- Studio component tests cover all five workspaces, read-only Rule Roles, production compiler preview, full-pool matrix, selected-cast diagnostics, and stable IDs.
- Quality gates: `pnpm typecheck`, `pnpm test`, `pnpm build`, and `git diff --check`.

### 7. Wrong vs Correct

Wrong:

```ts
const prompt = `${actor.systemPrompt}\n${role.actionPrompt}`;
const model = lineupSeat.modelOverride ?? role.defaultModel;
```

Correct:

```ts
const runtime = compileActorRuntimeCard(player.actor);
const author = compileActorAuthorCard(player.actor);
const model = player.actor.production.modelBinding;
const mechanic = player.ruleRole.mechanicKey;
```

Extend content by adding valid Actors, Scripts, or Lineups. Extend mechanics only by implementing a real Rule Role and its rules/tasks first.
