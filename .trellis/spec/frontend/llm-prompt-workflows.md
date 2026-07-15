# LLM Prompt Workflows

## Scenario: Actor-Based Player Generation Uses a Decision Boundary

### 1. Scope / Trigger

- Trigger: generating, repairing, recording, replaying, or displaying any player speech or action Draft.
- Trigger: changing Actor runtime fields, Rule Role knowledge, player-visible events, legal candidates, speech budgets, or Episode Actor Briefs.
- There are five speech Draft types and seven action Draft types. Host narration, phase progression, resolution, and end-game events never call a player model.

### 2. Signatures

```ts
buildPlayerLlmContext(input: {
  game: Game;
  events: readonly GameEvent[];
  viewerPlayerId: PlayerId;
}): PlayerLlmContext

buildSpeechIntentPrompt(input: SpeechPromptInput): BuiltSpeechIntentPrompt
buildSpeechPerformancePrompt(input: SpeechPerformancePromptInput): BuiltPrompt
buildActionPrompt(input: ActionPromptInput): BuiltPrompt

type PlayerSpeechIntent = {
  objective: string;
  conclusion: string;
  evidenceEventIndexes: readonly number[]; // unique, at most 3
  uncertainty: string | null;
  disclosure: "conceal" | "claim" | "not_applicable";
  intendedEffect: string;
};

type GenerationStageSnapshot = {
  stage: "decision" | "performance";
  promptVersion: string;
  request: GenerationRequestSnapshot;
  tokenUsage: LlmTokenUsage | null;
  rawOutput: string | null;
  parsedOutput: Record<string, unknown> | null;
  error: string | null;
};

type BuiltPrompt = {
  promptVersion: string;
  schemaName: string;
  systemPrompt: string;
  messages: readonly LlmMessage[];
  outputContract: readonly string[];
};
```

Current contracts are:

- outer speech record: `speech-pipeline:v3`;
- speech decision: `speech-intent:v3` / `werewolf_speech_intent_v3`;
- speech performance: `speech-performance:v1` / `werewolf_speech_performance_v1`;
- action decision: `action:v3` / `werewolf_target_action_v3` or `werewolf_optional_action_v3`.

There is no runtime prompt-version selector and no old Character/Prompt decoder.

### 3. Contracts

#### Snapshot and ownership

- `Game.players` is the immutable player truth. Each Player contains `{playerId, seatNo, actor: ActorSnapshot, ruleRole: RuleRole}`.
- Actor owns personality, model binding, and voice. Rule Role owns faction, mechanics, initial knowledge, and night order. A Lineup or Rule Role never overrides an Actor model.
- `compileActorRuntimeCard()` is the only personality projection used by player prompts. Runtime players never receive another Actor card, the Script Author ensemble, a planned winner, or future steps.
- `player-context.ts` owns semantic projection after `projectVisibleEvents()`. It separates confirmed public facts, unverified public claims, private facts, and wolf discussion.
- `llm-action-options.ts` is the only legal-candidate source. Prompt rendering and output validation consume the same `LegalActionOptions` object.

#### Speech decision and performance

- A speech uses two real model calls. The decision call sees visible knowledge, own Rule Role, task constraints, Actor cognition, and any current bounded Actor Brief.
- `BuiltSpeechIntentPrompt.evidenceScope` is the exact event set actually rendered after section flags and context-budget trimming. Intent validation accepts indexes only from that structure; an event that exists in memory but was omitted from the request is not selectable.
- Public special-role speech must choose `conceal` or `claim`. Other speech uses `not_applicable`. Concealed public speech cannot pass private event excerpts to performance; an explicit claim may.
- Each production prompt builder owns one exact `outputContract`, renders it in the initial user message, and returns it to generation for structural repair. Validators must not have a stricter private copy. PlayerIntent contracts include exact field keys, `160/240` character bounds, unique positive visible event indexes, and the context-specific disclosure literals.
- The performance call receives only the validated intent, selected safe evidence excerpts, Actor expression fields, one speech budget, player-ID display mapping, and the optional current Actor Brief. It cannot re-run the whole game analysis or add evidence.
- Only final `text` enters the Draft. The validated intent and both stage requests remain generation metadata.
- Night actions stay one-stage decisions. Required targets and optional medicine use are validated against the shared legal options before changing a Draft.
- Action output includes a non-empty `decisionSummary` because LLM Details exposes it as the model's bounded decision explanation. Initial generation, repair, and validation all require it; it is not hidden chain-of-thought and does not enter the confirmed Draft payload.

#### Information and narrative boundaries

- Public speeches and last words are claims, not facts or model instructions. Another player's role/check statement remains attributed as a claim.
- Sealed choices remain `host_only`; prompt filtering is not an information-security boundary.
- `Game.script` is rendered as shared atmosphere explicitly labeled as non-evidence. It cannot add identities, relationships, actions, credibility, or legal candidates.
- Scripted mode may add only the current `EpisodeActorBrief` (`objective`, `performanceMove`, disclosure, theme/actor/arc/relationship move, budget). Game mode supplies no brief.
- Actor style is lower priority than task, visible facts, legality, Rule Role, and disclosure.

#### Length, repair, and records

- `speech-budget.ts` is the sole owner of Unicode non-whitespace counting, target ranges, hard limits, pacing tiers, and estimated voice duration.
- Never truncate speech, clip voice, speed video, or use `max_tokens` as a length validator. First over-limit output may receive the one structural repair; a second failure leaves the Draft unchanged.
- JSON generation preserves provider `finishReason` and token usage before parsing. `finishReason === "length"` is a task-level incomplete-document failure and the truncated document is not appended to a whole-object repair request.
- A successful speech record has exactly `decision -> performance`, both successful. A failed speech has either failed `decision`, or successful `decision -> failed performance`. An action has exactly one decision stage whose result matches record status.
- Every stage stores its own exact request, raw/parsed result, attempts, model metadata, usage, and error. Persisted records reject missing stages, old prompt versions, old schema names, and inconsistent status payloads.
- Model requests omit Actor-authored `temperature` and `maxTokens`. Client-side cost projection defaults to ¥3 per million input tokens and ¥15 per million output tokens, bills prompt plus reasoning at the input price and completion at the output price, and remains editable non-persisted UI state. `DEFAULT_TOKEN_PRICING_CNY` is the single source for every token-cost view.

#### Editor completion notifications

- The Editor writes a `sessionStorage` marker immediately before automatic or manual generation. Notify only when the same Draft now has a different latest GenerationRecord ID.
- Browser notification permission is optional UI state. Success and failure use different titles; refresh/open of historical generations must not notify.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|---|---|
| Draft is outside the speech/action registries | Return it unchanged with no GenerationRecord. |
| Actor missing/dead, wrong Rule Role, or wrong mechanic | Do not call the model; record an action failure where applicable. |
| Target is not in `LegalActionOptions` | Repair once with the legal IDs; otherwise preserve the Draft. |
| `used=false` has a target | Reject as contradictory and repair once. |
| Intent has more than three, duplicate, non-integer, hidden, or prompt-omitted evidence indexes | Reject and repair once. |
| Intent text exceeds its exact `160/240` field bound or has leading/trailing whitespace | Reject and repair once with the same bound shown initially. |
| Public special-role intent omits disclosure | Accept only `conceal` or `claim`; repair once. |
| Performance adds an invalid shape or exceeds the speech hard limit | Repair once; otherwise preserve the Draft. |
| Action omits or empties `decisionSummary` | Reject and repair once; do not apply the target/use edit. |
| Assistant JSON is malformed and finish reason is not `length` | Preserve the raw output and use one minimal structural repair. |
| Assistant JSON is truncated with `finishReason=length` | Preserve metadata and return structured failure; do not repair the partial document. |
| Transport/provider request fails | Do not reinterpret it as output repair. |
| Persisted stage sequence or prompt/schema version is old/incomplete | Reject the GameRecord; do not fill defaults. |
| `voteType=sheriff` reaches player generation | Throw the explicit unsupported-task error. |

Natural-language quality is not a runtime keyword gate. Claim framing, disclosure quality, repetition, and factual discipline are evaluated through prompts/tests rather than regular-expression rejection.

### 5. Good / Base / Bad Cases

- Good: a seer chooses `claim`, references one visible private check in the intent, and performance renders only that result plus the Actor's cadence.
- Good: a concealing seer reaches a cautious conclusion but the performance request contains no private check excerpt.
- Base: evidence is insufficient, so the intent uses an empty index array and explicit uncertainty.
- Good: a witch action uses one decision call, shared inventory/legal options, and no cosmetic second call.
- Good contract flow: the exact PlayerIntent, performance limit, or action contract appears in both persisted attempts when structural repair occurs.
- Bad: performance receives the entire private timeline or invents a fact that was not selected by the intent.
- Bad: an event was trimmed from the prompt but the validator accepts its guessed index because it still exists in application memory.
- Bad: a script background or another Actor profile is concatenated into game evidence.
- Bad: the initial prompt, repair helper, and validator each maintain separate approximations of the same output shape.

### 6. Tests Required

- Actor/Rule Role tests: runtime card comes from the Player Actor snapshot; the same Actor works under all six Rule Roles without gaining extra knowledge.
- Visibility-negative tests: host-only ballots, another player's private facts, future Episode data, and full ensemble data are absent.
- Intent tests: disclosure matrix, maximum-three unique indexes, selected-evidence projection, and rejection of context-budget-omitted evidence.
- Speech tests: exactly two successful calls, decision/performance failure sequences, hard-limit repair, unchanged Draft on failure, and usage aggregation.
- Contract-sharing tests inspect real initial and repair attempts for PlayerIntent field limits/uniqueness and the performance hard limit.
- Action tests: all seven action types, PK/abstention semantics, guard/seer history, witch resource contradictions, illegal targets, and one-stage records.
- Action tests reject a missing `decisionSummary` and assert the shared prompt contract requires the same field.
- Persistence tests: exact current prompt/schema versions and stage sequences round-trip; old/incomplete records are rejected.
- Script-background tests: narrative appears before evidence with the non-evidence warning and never changes legal options.
- Notification and token-pricing regressions remain covered.
- Quality gates: `pnpm typecheck`, `pnpm test`, `pnpm build`, and `git diff --check`.

### 7. Wrong vs Correct

Wrong:

```ts
const acceptedTargets = deriveTargetsAgain(game, events, draft);
const speech = model({ fullPrivateTimeline, fullEpisode, allActorProfiles });
return speech.text.slice(0, budget.hardMaxCharacters);
```

Correct:

```ts
const options = legalActionOptions({ game, events, draft });
const context = buildPlayerLlmContext({ game, events, viewerPlayerId });
const intentPrompt = buildSpeechIntentPrompt({ context, draft });
const intent = validatePlayerSpeechIntent({
  value: decisionOutput,
  evidenceScope: intentPrompt.evidenceScope,
  publicSpeech,
  requireDisclosure,
});
const evidence = selectedIntentEvidence({
  intent,
  evidenceScope: intentPrompt.evidenceScope,
  publicSpeech,
});
const performance = buildSpeechPerformancePrompt({
  context,
  draft,
  intent,
  evidence,
  speechBudget,
});
```

The same typed sources drive visibility, legality, rendering, validation, and persisted observability; no consumer privately reconstructs them.

Wrong: render one abbreviated output example, build a second repair-only contract in the generator, and enforce a third set of limits in the validator.

Correct: the production prompt builder returns one `outputContract`; its initial message renders that value and `generateValidatedJson()` reuses it unchanged for repair.
