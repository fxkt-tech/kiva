# Character-Driven Episode Authoring Design

## 1. Design Decision

Keep the existing two-level authorship boundary:

1. Script Author sees the complete legal trace and creates global ensemble
   direction plus per-speech dramatic beats.
2. Runtime player generation sees only the current beat, that player's own
   snapshotted character profile, and facts visible at that moment.

The redesign makes character identity affect episode structure without letting
global future knowledge leak into final dialogue. It does not introduce a
second rules engine, final-dialogue pre-generation, or an LLM critic.

This is one tightly coupled task rather than a parent/child tree: snapshot
contracts, author output, runtime projection, and Review UI must change together
to produce one independently verifiable behavior.

## 2. Versioned Contracts

New authored candidates use:

- `EPISODE_SCRIPT_SCHEMA_VERSION = 2`
- `EPISODE_AUTHOR_PROMPT_VERSION = "episode-author:v2"`
- schema names `werewolf_episode_outline_v2` and
  `werewolf_episode_beats_v2`

Schema v2 adds the following concepts. Exact naming may be adjusted during
implementation, but their ownership and validation boundaries are fixed.

```ts
type EpisodeCastDirection = {
  playerId: PlayerId;
  dramaticWeight: "primary" | "supporting";
  dramaticFunction: string;
  baseline: string;
  pressure: string;
  change: string;
  payoff: string;
  signatureMoment: {
    stepIndex: number;
    description: string;
  };
};

type EpisodeRelationshipDirection = {
  playerIds: readonly [PlayerId, PlayerId];
  kind: "rivalry" | "alliance" | "contrast" | "trust_shift";
  setup: string;
  development: string;
  payoff: string;
};
```

`EpisodeSpeechBeat` keeps its compiler-owned `budget` and existing author fields,
then adds:

```ts
type CharacterBeatDirection = {
  characterHook: string;
  arcMove: string;
  relationshipMove: string | null;
};
```

`characterHook` explains how the actor's stable traits shape this scene;
`arcMove` identifies setup, pressure, adaptation, or payoff; and
`relationshipMove` describes only the current public interaction. These fields
are direction, not game facts or final dialogue.

Historical schema-v1 snapshots remain readable and executable. The decoder
retains a v1 compatibility branch, exposes empty ensemble direction, and
projects null character-beat additions. Existing persisted snapshots are not
rewritten merely by being read. New authoring always writes v2.

## 3. Character Profile Projection and Input Identity

Add one pure projection owned by episode authoring:

```ts
episodeCharacterProfile(player): {
  playerId, seatNo, name, gameRole,
  persona, speakingStyle, reasoningStyle,
  legacyCharacterPrompt?
}
```

Structured `persona`, `speakingStyle`, and `reasoningStyle` are authoritative.
`characterSystemPromptSnapshot` is used only as the compatibility fallback when
all structured fields are blank, matching runtime prompt-v2 behavior. Avatar,
voice, model, and presentation metadata are excluded from the author profile.

The v2 episode input hash includes the exact projected profile fields, name,
role, ruleset, and immutable Game Script snapshot. Hash comparison is
schema-aware: v1 scripts continue using the legacy hash recipe, while v2
generation, approval, and execution use the character-aware recipe. This avoids
invalidating historical approved scripts while ensuring any authoring-relevant
character change invalidates a v2 candidate.

## 4. Outline Request and Ensemble Output

The outline request continues to include theme, planned result, and compact
legal milestones. It additionally includes:

- one concise profile card per cast member;
- a deterministic performance-opportunity map per player, containing real step
  indexes where the player speaks, acts, is publicly affected, dies, or gives
  last words;
- explicit direction to create complementary primary/supporting functions,
  one signature moment per player, earned character change, and only in-game
  relationships;
- explicit prohibitions on changing the trace, inventing pre-game
  relationships, treating dramaturgy as evidence, or writing final dialogue.

The outline JSON returns `title`, `logline`, `acts`, `castDirections`, and
`relationships`. Parsing and cross-validation require:

- the exact Game player-ID set appears once in `castDirections`;
- every required text field is nonblank;
- each signature step is in that player's opportunity set;
- every relationship references two distinct known players;
- undirected relationship pairs are unique;
- enum values are exhaustive.

Invalid output receives the existing single validated-generation repair attempt.
If it remains invalid, the authoring job becomes failed; it cannot enter Review.
There is no subjective creativity score.

## 5. Beat Requests and Cross-Batch Continuity

Speech steps remain chronological batches of at most 12. Each request carries:

- the existing bounded local legal-trace window;
- concise profiles and `EpisodeCastDirection` records only for actors in the
  current batch;
- relationship directions touching those actors;
- a bounded summary of already-authored earlier moves for those actors, at most
  the latest two per actor, so batches continue an arc instead of restarting it;
- the exact speech step indexes and compiler-owned budgets.

The result must cover the batch exactly once and return existing beat fields plus
`characterHook`, `arcMove`, and nullable `relationshipMove`. Beat authoring may
shape objective, stance, and theme expression, but it cannot modify step slots,
planned payloads, budgets, roles, actions, deaths, or winner.

## 6. Snapshot and Approval Validation

Validation is layered:

1. JSON parsing validates field shapes and closed enums.
2. Author-result validation compares cast IDs, relationship IDs, signature
   opportunities, and beat indexes with the current Game and compiled plan.
3. Snapshot validation enforces schema-local invariants and retains v1 support.
4. `episodeScriptReport()` reports deterministic approval blockers such as an
   over-duration episode or incomplete v2 dramaturgy.
5. Approval rechecks the schema-aware input hash and current cast binding.

Subjective judgments—chemistry, originality, emotional impact, or whether an
arc is sufficiently exciting—are deliberately absent from automated approval.
The director makes that decision from the Review UI.

## 7. Runtime Actor Projection

`actorBriefForStep()` adds only the current beat's `characterHook`, `arcMove`,
and `relationshipMove`. It does not expose the full cast direction, future arc
stages, planned winner, future steps, another player's profile, or hidden
relationship plan.

The existing runtime prompt remains the final authority order:

1. current scene and task;
2. visible facts, legal rules, and disclosure boundary;
3. role/faction objective;
4. own character identity;
5. current dramatic direction.

If dramatic direction conflicts with visible facts or rules, the player prompt
must ignore it. The actual line is still generated automatically when the new
speech Draft reaches Editor and remains individually reviewable/regenerable.

Schema-v1 briefs retain current behavior and simply omit the new direction.

## 8. Script Review UI

Add a pure, testable ensemble section to Script Review, above the detailed step
list:

- one card per player with seat/name, dramatic function, baseline-to-payoff arc,
  and signature moment;
- a relationship list showing the two participants, relationship kind, setup,
  development, and payoff;
- clear primary/supporting language supplied by the author, without pretending
  screen time is equal.

The section is read-only. The existing whole-candidate Regenerate and Approve
actions remain the only MVP controls. Schema-v1 candidates show a neutral
compatibility message instead of fabricated direction.

## 9. Local Heuristic and External Provider Behavior

`LocalHeuristicLlmClient` must understand both v2 schema names so development
without external credentials still reaches Review. Its v2 output is
deterministic, covers the exact cast and beat IDs parsed from the request, and
produces structurally distinct directions using the supplied names/profile
snippets. It is a functional fallback, not the quality benchmark.

External-provider requests retain JSON-object mode and the existing single
repair mechanism. The first player's model binding remains the current Script
Author model selection for this task; introducing a dedicated author model is a
separate concern.

## 10. Compatibility and Rollback

- Do not migrate stored v1 candidates or approved scripts.
- Do not change the legal compiler or event payloads.
- Do not add final dialogue to the episode snapshot.
- Do not alter Preview, voice, or export contracts; they continue consuming
  confirmed events.
- A rollback can stop producing schema v2 while retaining the v1 decoder. No
  confirmed game events or media require rollback because authoring occurs
  before approval/execution.

## 11. Verification Strategy

Automated checks cover request capture, exact ID validation, signature-step
binding, relationship validation, character-aware hash invalidation, v1 decode,
batch-size limits, compiler-owned fields, current-only Actor Brief projection,
Review rendering, typecheck, build, and diff cleanliness.

A manual quality pass generates one real 12-character episode and checks:

- every character is recognizable from function and behavior without relying
  on their displayed name;
- primary and supporting roles are intentional rather than uniform;
- every player has a plausible signature moment;
- at least the central relationships have setup, escalation, and payoff;
- changes are earned and do not replace the baseline personality;
- no dramaturgy invents game facts or changes the legal result.
