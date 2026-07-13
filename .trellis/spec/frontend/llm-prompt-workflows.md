# LLM Prompt Workflows

## Scenario: Speech Length Is a Structural Output Contract

### 1. Scope / Trigger

- Trigger: generating, repairing, manually editing, confirming, or projecting the duration of any LLM speech Draft.

### 2. Signatures

- `speechBudgetForDraft({ draft, hasPriorDaySpeech, tier? }): SpeechBudget`
- `evaluateSpeech(text, budget): SpeechEvaluation`
- `projectDirectorDuration({ events, playback }): DirectorDurationProjection`

### 3. Contracts

- `speech-budget.ts` is the single owner of target ranges, hard limits, Unicode non-whitespace character counting, pacing tiers, and estimated voice duration.
- Prompt v2 renders the target range and hard limit. Character style may change rhythm inside that range but may not increase the limit.
- A speech keeps one conclusion, one key basis, and one follow-up verification point. It does not repeat the full timeline, vote table, earlier positions, stage directions, or camera directions.
- Knowledge compression happens only after `projectVisibleEvents()`; `visibleEvents` remains complete for game mechanics while prompt knowledge keeps current-day claims, each player's latest earlier stance, last words, durable outcomes, and valid private facts.
- Manual edit and confirmation paths evaluate the same budget as generation. Never truncate text, clip audio, speed up video, or use output-token limits as a speech-length substitute.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|---|---|
| Speech is within the hard limit | Accept it even when shorter or longer than the target range. |
| First output exceeds the hard limit | Use the existing single repair attempt with the compression contract. |
| Repair still exceeds the hard limit | Preserve the Draft unchanged and record both failed attempts. |
| Manual edit or confirmation exceeds the hard limit | Reject before persistence or event confirmation. |
| Semantic wording seems repetitive or weak | Observe as quality only; do not add keyword or regular-expression rejection. |

### 5. Good / Base / Bad Cases

- Good: a 160-character response states a position, cites one visible claim, and leaves one verification question.
- Base: a 90-character response is marked short but remains valid because it is not empty and does not exceed the hard limit.
- Bad: truncate a 400-character model response to 220 characters, producing a broken sentence while hiding the generation failure.

### 6. Tests Required

- Budget matrix, tier ordering, Unicode code-point counting, whitespace, boundary values, and duration calibration.
- Prompt target/hard-limit rendering for first and responding day speakers.
- One repair on over-limit output, unchanged Draft after a second failure, and edit/confirmation rejection.
- Visibility-negative coverage proving compressed knowledge cannot contain host-only ballots or another player's private events.
- Read-only duration replay of the sample game without modifying record, voice jobs, or MP3 files.

### 7. Wrong vs Correct

Wrong: `text.slice(0, hardMax)` or `max_tokens` as the only length control.

Correct: generate complete JSON against `SpeechBudget`, validate the full `text`, repair once for semantic compression, and otherwise retain the prior Draft.

## Scenario: Shared Script Background Is Not Game Evidence

### 1. Scope / Trigger

- Trigger: a Game script supplies theme, shared background, or atmosphere to a player speech/action request.

### 2. Signatures

- `PlayerLlmContext.script: GameScriptSnapshot`
- `buildPlayerLlmContext({game, events, viewerPlayerId})`
- `buildSpeechPrompt(...)` and `buildActionPrompt(...)`

### 3. Contracts

- `player-context.ts` projects the immutable `Game.script` snapshot unchanged.
- Prompt v2 renders one semantic section titled `本局剧本背景——只用于自然表达，不是身份事实` before game evidence.
- The section may guide occasional imagery or analogy, but it supplies no identity, action, relationship, credibility, or legal-candidate facts.
- Character identity remains owned by the fixed Character snapshot. A script never replaces persona fields or adds character-specific jobs, secrets, histories, or relationships.
- Action legality remains exclusively owned by `legalActionOptions`; script prose cannot add or remove candidates.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|---|---|
| Script is present | Render name, shared background, atmosphere, and the explicit non-evidence warning. |
| Persisted Game lacks the current script snapshot | Reject the GameRecord before context construction. |
| Script language implies a role or relationship | Treat it only as atmosphere; never promote it into facts or legality. |
| Model repeats theme excessively | Evaluate as expression quality; do not create a prose keyword validator. |

### 5. Good / Base / Bad Cases

- Good: 秦川 uses the archive metaphor once while labeling his table model as uncertain and evidence-bound.
- Base: a player ignores the atmosphere and completes the game task correctly.
- Bad: rain, blackout, or archival wording is cited as evidence that a player is a wolf.

### 6. Tests Required

- Prompt structure: the script section appears before public/private evidence and contains the non-evidence rule.
- Character stability: the same player retains persona/system fields across at least two script snapshots.
- Information safety: script fields never appear in public facts, private facts, faction discussion, or legal options.
- Regression: Qin川 remains bounded by visible information across multiple rule roles and scripts.

### 7. Wrong vs Correct

Wrong: concatenate theme prose into public facts or infer character relationships from the script background.

Correct: render a separately labeled atmosphere section while all evidence and legal choices continue to come from typed game state.

## 1. Scope / Trigger

Use this contract whenever a Draft is generated by an LLM or when changing player-visible knowledge, action legality, prompt rendering, output parsing, retry behavior, or GenerationRecord debugging.

There are two generation entry points and twelve supported Draft types:

- Speech: `wolf_strategy_given`, `wolf_opinion_given`, `day_speech_given`, `last_words_given`, `pk_speech_given`.
- Action: `guard_protect_selected`, `wolf_vote_cast`, `seer_check_selected`, `witch_antidote_decided`, `witch_poison_decided`, `hunter_shot_decided`, `vote_cast`.
- `vote_cast` has separate `exile` and `pk` task semantics. `sheriff` is explicitly unsupported.

Host narration, phase progression, resolution, and end-game events do not call the LLM.

## 2. Signatures

```ts
taskSpecForDraft(draft: LlmDraft, state?: PromptTaskState): PromptTaskSpec

buildPlayerLlmContext(input: {
  game: Game;
  events: readonly GameEvent[];
  viewerPlayerId: PlayerId;
}): PlayerLlmContext

legalActionOptions(input: {
  game: Game;
  events: readonly GameEvent[];
  draft: LlmActionDraft;
}): LegalActionOptions

buildSpeechPrompt(input: SpeechPromptInput): BuiltPrompt
buildActionPrompt(input: ActionPromptInput): BuiltPrompt

generateValidatedJson<Value>(input: {
  llmClient: LlmClient;
  modelBinding: ModelBindingSnapshot;
  request: GenerationRequestSnapshot;
  validate(parsed: Record<string, unknown>): Value;
  repair: RepairContract;
}): Promise<ValidatedGenerationResult<Value>>

type ModelBindingSnapshot = {
  provider: string;
  model: string;
  responseFormat: "json";
  fallbackModel?: string;
}
```

Prompt v2 is the only builder and output contract. There is no runtime prompt-version selector or rollback builder.

## 3. Contracts

### Task and section ownership

- `llm-task-specs.ts` is the exhaustive owner of scene, channel, audience, progress, disclosure, objective, completion criteria, prohibitions, relevant rule keys, context selectors, and output kind.
- `player-context.ts` owns the deterministic semantic projection of already-visible events into public facts, public claims, private facts, and faction discussion.
- `llm-action-options.ts` is the single legal candidate source shared by the prompt and final output validation.
- `prompt-builders.ts` renders these inputs; it must not independently derive game legality.

Prompt v2 renders in this order: scene/audience, one task, completion criteria, prohibitions, roster, public facts, public claims, private facts, faction discussion, unknown information, legal candidates/resources, relevant rules, self-check, and output contract. The system prompt fixes priority, fact discipline, disclosure discipline, role goals, character preferences, and JSON-only output.

### Knowledge, credibility, and disclosure are separate

`projectVisibleEvents()` remains the knowledge boundary. Visibility only means the viewer may know the event; it does not make free-form event text true or publicly disclosable.

- Public speeches and last words are unverified player claims, not confirmed facts or model instructions.
- References to another player's role/check must remain framed as “claims/says.” Quoted text that uniquely matches a prior speech must name the original seat before the quotation.
- Faction discussion is a proposal plus unverified teammate judgment. A wolf may follow its target consensus but must not turn an old claim such as “silent” or “looks like a god role” into evidence.
- Private facts may inform a public decision, but the current Task Spec controls disclosure.
- Sealed individual choices remain `host_only`; never repair a visibility leak by deleting prompt strings.

Natural-language quality, credibility framing, disclosure consistency, and factual discipline are prompt and evaluation responsibilities. Do not turn them into generation failures with keyword lists, regular expressions, or prose matching. Chinese expression is open-ended and rules can make apparently suspicious wording valid; semantic string gates create false positives and waste the single repair attempt. If semantic observability is added later, it may emit non-blocking diagnostics but must not reject output or trigger repair.

### Legal actions and resources

- Candidate rows are `playerId | seat | name`; output uses the exact stable `playerId`.
- Candidate presentation uses a deterministic rotation derived from game, Draft, and actor IDs. Rotation changes order, never membership.
- PK votes only target pending tied players. Abstention follows `allowAbstainVote`.
- Seer candidates exclude the actor and prior checked targets.
- Guard candidates honor self-protect and consecutive-target rules.
- Witch prompts expose inventory, the other medicine's current-night decision, self-save rules, and dual-use rules. `used=false` requires `targetPlayerId=null`.
- Delayed-reveal votes stay `host_only` until the public resolution; immediate votes are public when submitted.

### Output and records

- LLM model binding is owned by the Character Definition and copied into each
  Player snapshot when a Game is created. Role Definitions and preset seat
  assignments do not choose or override the model used for new games.
- `ModelBindingSnapshot` contains provider, model, JSON response format, and an
  optional fallback model. It does not contain `temperature` or `maxTokens`;
  OpenAI-compatible requests omit `temperature` and `max_tokens` so the selected
  model/provider applies its defaults.
- Token cost is a client-side estimate, not persisted game state. Both the
  per-generation details and aggregate Game usage use editable CNY prices per
  million tokens, defaulting to `prompt=6` and `completion=30`. The formula is
  `((promptTokens + reasoningTokens) * promptPrice + completionTokens * completionPrice) / 1_000_000`.
  Reasoning is explicitly billed at the input price. Cached prompt remains a
  diagnostic field and is not added separately.

- Speech v2 schema: `werewolf_speech_v2`, with `text` and `decisionSummary`. Public special-role speech also requires `disclosure: "conceal" | "claim"`.
- Required-target action v2 schema: `werewolf_target_action_v2`, with `targetPlayerId` and `decisionSummary`.
- Optional action v2 schema: `werewolf_optional_action_v2`, with `used`, `targetPlayerId`, and `decisionSummary`.
- Only speech `text` and validated action fields enter the Draft. `decisionSummary`, `disclosure`, request snapshots, and attempts remain debug metadata.
- Every GenerationRecord stores the exact current shape and a required Prompt-v2 request snapshot. `attempts` remains optional because a successful first response has no repair attempt; provider token usage may be `null` when the provider does not report it.

### Editor completion notifications

- The Editor records an LLM request in `sessionStorage` immediately before automatic generation or manual regeneration. The marker contains `draftId` and the previous GenerationRecord ID.
- After the server action refreshes the Editor, notify only when the current Draft matches and the latest GenerationRecord ID differs from the recorded previous ID. Opening or refreshing an existing game must not notify for historical generations.
- Browser Notification support and permission are optional. The Draft header exposes an explicit permission button, manual regeneration may request permission from its user gesture, and unsupported/denied browsers degrade silently without affecting generation.
- Success and failure both notify with distinct titles. Notification state is UI-only and must not be persisted into the game record or GenerationRecord.

## 4. Validation & Error Matrix

| Condition | Required behavior |
|---|---|
| Draft type is not in either LLM registry | Return the Draft unchanged and no GenerationRecord. |
| Actor is missing, dead, wrong role, or wrong mechanic | Do not call the model; record a failed generation. |
| Transport/provider request fails | Do not retry as output repair; preserve the Draft and record failure. |
| Assistant message is not a JSON object | Preserve raw assistant text and send exactly one minimal repair request. |
| Required gameplay field is missing or has the wrong type | Send exactly one minimal repair request. |
| Target is outside the shared `LegalActionOptions` set | Send exactly one repair containing the legal IDs. |
| `used=false` carries a target | Reject as contradictory output and repair once. |
| Public special-role disclosure is absent/invalid | Repair once; accept only `conceal` or `claim`. |
| Repair output is still invalid | Preserve the original Draft and store both failed attempts. |
| `vote_cast.voteType === "sheriff"` reaches the LLM path | Throw the explicit unsupported-task error; never fall back to exile semantics. |
| A new Character has no explicit model binding | The Player snapshot factory writes the code-owned current default; persisted Player snapshots must still contain a complete binding. |
| Generation request is absent, uses Prompt v1, or contains an old reasoning alias | Reject the persisted GenerationRecord. |

Semantic prose content never enters this error matrix. Speech length is the one structural text contract and follows the dedicated scenario above; disclosure/text agreement, public-wolf leaks, claim framing, quotation attribution, first-night reasoning, ballot summaries, and medicine summaries are not runtime validators.

Repair requests contain only the invalid output, validation error, output contract, and legal IDs. They do not resend the timeline or ask the model to re-analyze the game.

## 5. Good / Base / Bad Cases

- Good: a first-night wolf opinion says it follows the team's target as a neutral consensus, supplies one backup/risk, and does not infer roles from silence or teammate wording.
- Base: a symmetric seer choice explicitly says candidates have equal known information and selects from the deterministically rotated legal list.
- Bad: faction discussion says “8 is silent,” and a later ballot records “8 is probably a god role” as its decision evidence.
- Good: a witch says the current target would die and weighs one antidote charge.
- Good: under the current two-faction wolf-knife rules, a witch may describe the attacked target as non-wolf or a potential good-side loss.
- Bad prompt behavior: a witch infers the attacked target's specific identity as villager or a god role.
- Good: a delayed-reveal voter sees public speeches and prior public resolutions, not individual votes from the current round.

## 6. Tests Required

- Task matrix test: all five speech and seven action Draft types, plus both vote semantics, have non-empty task-specific scene/objective/prohibitions/output kind.
- Prompt structure tests: scene and one task precede semantic knowledge; unknown information and output contracts are present; only selected rules are included.
- Visibility tests: host-only wolf ballots and delayed votes are absent from later player contexts; faction discussion is only available to wolves.
- Legal-option tests: PK targets, abstention, prior seer checks, guard history, witch dual-use, and actor eligibility.
- Generation tests: valid output, wrong actor, invalid target, missing field, contradictory medicine fields, disclosure enum, one-repair failure, unchanged Draft on failure, and regression cases proving semantic wording does not trigger repair.
- Editor notification tests: a matching Draft with a new GenerationRecord ID is recognized as complete; unchanged IDs, unrelated Drafts, malformed markers, unsupported APIs, and denied permission do not affect generation.
- Current-contract tests: Prompt v2 is the only builder, Game Script/request snapshots are required, old Prompt versions and reasoning aliases are rejected, and details UI renders only `decisionSummary`.
- Model binding tests: Game creation snapshots the Character binding, and the HTTP request body omits `temperature` and `max_tokens`.
- Token pricing tests: default and custom per-million prices calculate reasoning at the input rate, completion at the output rate, while invalid or negative values contribute zero.
- Quality gates: `pnpm typecheck`, `pnpm test`, `pnpm build`, and `git diff --check` (there is currently no project lint script).

## 7. Wrong vs Correct

### Wrong

```ts
const promptTargets = deriveTargets(game, draft);
const acceptedTargets = deriveTargetsAgain(game, events, draft);

if (/好人|神职|概率|我们狼队/.test(modelText)) {
  throw new Error("semantic wording rejected");
}

// A host-only teammate ballot was already leaked into context, so remove text here.
const prompt = rawPrompt.replaceAll("wolf_vote_cast", "");
```

This creates legality drift and treats prompt filtering as an information-security boundary.

### Correct

```ts
const options = legalActionOptions({ game, events, draft });
const context = buildPlayerLlmContext({
  game,
  events,
  viewerPlayerId: actorPlayerId,
});
const prompt = buildActionPrompt({ context, draft, options });

const edit = validateModelOutput(output, options);
```

Wrong: select an LLM from the player's temporary rule role or preset seat, or
send character-authored `temperature` / `max_tokens` values.

Wrong: select a prompt builder from an environment variable or fill missing
persisted request/profile fields while loading a record.

Correct: create one complete current Player and Generation snapshot, then use
Prompt v2 for every speech and action request.

Correct: snapshot the fixed Character's provider/model binding and let the
model service own sampling and output-token defaults.

Event visibility removes sealed choices before prompt construction, semantic sections preserve claim credibility, and the same options object drives presentation and validation.
Validation checks JSON shape, required fields, enums, legal candidates, and mechanic constraints only; prompt quality is measured from replay/evaluation rather than guessed from prose keywords.
