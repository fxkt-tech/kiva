# 角色驱动的剧本生成重设计

## Goal

Redesign scripted-mode episode authoring so the planned narrative and each
speech beat deliberately use the cast's snapshotted character identities. The
result should give each player a recognizable dramatic function and distinct
behavior before runtime dialogue generation, while preserving rules legality,
visibility boundaries, and the approved episode structure.

The primary product outcome is a compelling performance with immediately
recognizable personalities. Rules legality and information safety are hard
boundaries; they are necessary constraints rather than substitutes for dramatic
quality.

## Background

- `authorEpisodeScript()` currently gives the outline request only player ID,
  seat, name, and hidden game role. Beat requests identify speakers only by
  `actorPlayerId`; neither request carries `persona`, `speakingStyle`,
  `reasoningStyle`, or `characterSystemPromptSnapshot`.
- Runtime speech generation does use the speaking player's `persona`,
  `speakingStyle`, and `reasoningStyle` together with the current
  `EpisodeActorBrief`. Consequently, final lines may sound distinct even though
  the episode-level allocation of conflicts, questions, and dramatic functions
  is not character-driven.
- The legal trace, actions, votes, deaths, planned winner, and per-step speech
  budgets are compiler-owned and must remain unchanged by Script Author output.
- Episode authoring is intentionally split into one compact outline request and
  speech-beat batches of at most 12 to avoid provider timeouts.

## Requirements

- Episode outline authoring must receive a concise profile for every cast member
  sufficient to distinguish temperament, speaking behavior, and reasoning
  tendencies.
- Script Author must assign every cast member a complementary, episode-long
  dramatic function and character arc. Character distinctiveness must come from
  recurring behavior and progression across scenes, not only surface wording.
- Each character arc must be projected into the relevant speech beats so later
  objectives and stances can develop, challenge, or pay off earlier behavior.
- Arcs must preserve each character's recognizable core while allowing pressure
  to expose a secondary facet, mistake, adaptation, or earned reversal. Sudden
  unsupported personality replacement and repetitive trait catchphrases are
  both invalid authoring outcomes.
- Script Author may create rivalries, alliances, trust shifts, and contrast
  pairings that emerge during the episode from the legal trace and public
  interaction. It must not invent pre-game relationships, private deals, or
  unsupported facts, and authored relationships are never game evidence.
- Dramatic weight may be asymmetric when the legal trace produces primary and
  supporting roles. Every cast member must nevertheless receive at least one
  recognizable signature moment within their available speeches, actions,
  death impact, or last words. Authoring must never alter the trace to equalize
  screen time.
- Every speech-beat batch must receive the relevant actor profiles and use them
  to shape objective, stance, disclosure strategy, and theme hook without
  generating final dialogue.
- Character direction must remain subordinate to confirmed rules, legal trace,
  current actor knowledge, and visibility constraints.
- Runtime dialogue generation must continue combining the approved current-step
  Actor Brief with the speaking player's snapshotted character profile.
- Initial episode authoring remains direction-only: it creates ensemble design,
  relationship development, and speech beats, but not final spoken dialogue.
  Final lines continue to be generated step-by-step from the approved current
  brief and the actor's then-visible facts so actors cannot see future events.
- Script Review must show a read-only ensemble section before approval. For each
  player it exposes the dramatic function, episode-long arc, and principal
  relationship directions in a director-readable form. MVP rejection is whole-
  candidate regeneration rather than field-level dramaturgy editing.
- The episode input identity must change when any character-profile field that
  affects authoring changes.
- Author requests must stay bounded and must not expose the full future script
  to runtime player prompts.
- Existing historical episode snapshots must remain readable unless a migration
  is explicitly justified by the final design.
- Automated approval validation must reject objectively invalid dramaturgy:
  missing or duplicate cast directions, unknown player references, signature
  moments that do not map to available steps, or speech beats missing their
  character-progression fields. Subjective quality such as chemistry,
  distinctiveness, and dramatic excitement remains a director judgment in
  Script Review; the system must not add an LLM-as-critic approval call.

## Acceptance Criteria

- [x] Captured outline requests contain distinct concise profiles for every
      player and instruct the author to assign character-consistent dramatic
      functions without altering the legal trace.
- [x] The authored episode snapshot contains one reviewable dramatic function
      and arc for every cast member, with no missing or duplicate player IDs.
- [x] Every cast direction identifies at least one signature moment mapped to a
      real step available to that player; primary/supporting emphasis does not
      change compiled steps or payloads.
- [x] Script Review renders all cast directions and relationship plans before
      approval; no direction is editable independently in MVP, and regeneration
      replaces the complete candidate.
- [x] Captured beat requests contain the profile of every actor represented in
      that batch and explicitly connect each beat to that actor's tendencies and
      episode-long arc.
- [x] Tests prove two materially different character profiles produce different
      authoring input while identical snapshotted inputs remain deterministic.
- [x] Tests prove actions, votes, deaths, winner, step indexes, payloads, and
      speech budgets remain compiler-owned after character-driven authoring.
- [x] Episode approval rejects a candidate when an authoring-relevant character
      snapshot changes after generation.
- [x] Runtime player prompts contain only the current Actor Brief plus that
      player's own character profile, never future steps, planned winner, or
      another player's private profile.
- [x] Existing beat batching remains at most 12 speech steps per provider
      request and the timeout regression continues to pass.
- [x] Structurally incomplete or invalid dramaturgy either fails validated
      authoring after its single repair attempt or makes the report invalid; in
      both cases approval is impossible. Structurally valid output is never
      rejected by an automated subjective creativity score.

## Out of Scope

- Letting Script Author change legal actions, vote targets, deaths, roles, or
  winner.
- Generating final spoken dialogue during the initial episode-authoring job.
- Exposing hidden cast identities or full episode plans to runtime actors.
- Introducing a dedicated Script Author model configuration or changing player
  model bindings.
