# Role Library Design

## Purpose

The next major foundation is the role library. Its purpose is not just to store role names. It must make the content-generation path stable:

`role definitions + character definitions + preset -> game player snapshots -> LLM prompts -> structured events`

The system must support adding more Werewolf roles later without rewriting game history, corrupting old games, or mixing gameplay mechanics with AI character personality.

## Core Decision

The system will split "role" into two separate libraries:

- **RoleDefinition**: a game identity such as werewolf, seer, witch, villager, hunter, guard, or idiot. This controls faction, mechanics, visibility rules, action timing, and role-specific prompt instructions.
- **CharacterDefinition**: an AI player persona such as Qin Chuan, Lin Xia, or Zhou Zhi. This controls name, speaking style, reasoning style, persona, and default model binding.

This split is required because the same AI persona can play different game identities in different games, and the same game identity can be performed by different AI personas.

The game record will store immutable **PlayerSnapshot** objects produced from both libraries at game creation time. Old games will remain stable if the libraries change later.

## Scope

This spec covers the first role-library phase:

- Define persistent data models for role definitions, character definitions, and game presets.
- Seed the current 6-player setup through scripts, not UI.
- Create new games from a preset instead of hard-coded `createSeedGame` data.
- Preserve per-game snapshots.
- Compose LLM prompts from both character persona and role identity.
- Introduce a clear role-mechanic adapter boundary for current roles.
- Keep the current 6-player complete game flow working.

This phase does not add new roles beyond the current four identities. Hunter, guard, idiot, sheriff, and other roles are intentionally deferred until the library boundary is proven by the existing flow.

## Data Model

### RoleDefinition

Long-lived game identity definition.

Fields:

- `id`: stable identifier, for example `werewolf`, `seer`, `witch`, `villager`.
- `name`: display name, for example `狼人`.
- `faction`: `wolves` or `good`.
- `team`: role grouping used by slaughter-side win conditions, for example `wolf`, `god`, `villager`.
- `mechanicKey`: stable mechanism identifier, for example `wolf_kill`, `seer_check`, `witch_medicine`, `none`.
- `systemPrompt`: role-specific identity prompt.
- `actionPrompt`: optional role-specific action prompt used when generating action drafts.
- `visibilityRules`: symbolic knowledge rules, for example `own_role`, `wolf_teammates`, `witch_medicines`.
- `nightOrder`: optional number used by night action ordering.
- `defaultModelBinding`: optional model binding if this role should prefer a model.
- `enabled`: whether the role can be selected by presets.
- `createdAt`
- `updatedAt`

Role definitions are not copied directly into event logs. They are snapshotted into players when a game is created.

### CharacterDefinition

Long-lived AI performer definition.

Fields:

- `id`: stable identifier, for example `qin_chuan`.
- `name`
- `avatar`: optional asset reference for later preview/replay use.
- `tags`: optional labels for filtering.
- `persona`
- `speakingStyle`
- `reasoningStyle`
- `systemPrompt`: character-specific prompt about voice, personality, and decision style.
- `defaultModelBinding`: optional model binding preferred by this character.
- `enabled`
- `createdAt`
- `updatedAt`

Character definitions do not contain gameplay identity. They are reusable across roles and games.

### GamePreset

Creation recipe for a game.

Fields:

- `id`: stable identifier, for example `six_player_standard`.
- `name`
- `rulesetId`: points to the ruleset used by this preset.
- `playerCount`
- `roleIds`: ordered or multiset role selection, for example `["werewolf", "werewolf", "seer", "witch", "villager", "villager"]`.
- `characterIds`: selected AI characters for seats.
- `seatAssignments`: optional explicit mapping of seat number to character id and role id.
- `enabled`
- `createdAt`
- `updatedAt`

For the first phase, the default preset can use deterministic seat assignments matching the current test game. Later, randomization and host-adjusted setup can be added.

### PlayerSnapshot

Per-game immutable player copy.

Fields:

- `playerId`
- `seatNo`
- `characterSourceId`
- `roleSourceId`
- `name`
- `avatar`
- `persona`
- `speakingStyle`
- `reasoningStyle`
- `characterSystemPromptSnapshot`
- `roleSystemPromptSnapshot`
- `roleActionPromptSnapshot`
- `gameRole`
- `roleName`
- `faction`
- `team`
- `mechanicKey`
- `initialPrivateKnowledge`
- `modelBindingSnapshot`

The model binding is resolved at snapshot time with this priority:

1. Explicit preset seat assignment model binding, if present.
2. Character default model binding, if present.
3. Role default model binding, if present.
4. System default model binding.

This priority keeps "model binding belongs to performer" as the default, while still allowing a preset or role to override it later.

## Storage

The first implementation will use repository-backed JSON files consistent with the current local game storage.

Suggested files:

- `.kiva-data/roles.json`
- `.kiva-data/characters.json`
- `.kiva-data/presets.json`

Seed source files should live in the repository, for example:

- `src/seeds/roles.ts`
- `src/seeds/characters.ts`
- `src/seeds/presets.ts`

Import scripts should create or update local library data:

- `pnpm seed:roles`
- `pnpm seed:characters`
- `pnpm seed:presets`

No role-library management UI is part of this phase.

## Game Creation Flow

Current game creation is hard-coded through `createSeedGame`. It should move to preset-based creation.

Target flow:

1. Load the selected `GamePreset`.
2. Load referenced role and character definitions.
3. Validate counts and enabled status.
4. Resolve seat assignments.
5. Create immutable `PlayerSnapshot` entries.
6. Create the game record with ruleset and player snapshots.
7. Continue the existing draft/event workflow.

The initial preset should reproduce the current 6-player setup:

- Seat 1 Qin Chuan: werewolf
- Seat 2 Lin Xia: werewolf
- Seat 3 Zhou Zhi: seer
- Seat 4 Xu Tang: witch
- Seat 5 Chen Mo: villager
- Seat 6 Shen Lan: villager

## LLM Prompt Composition

Prompts should use both persona and game role.

The composed system prompt should include:

1. Character system prompt: how this AI player thinks and speaks.
2. Role system prompt: what identity the player has in this game.
3. Safety/visibility rule: only use visible information.
4. Output contract: JSON fields required by the current draft type.

Example composition:

```text
你是周知，表达短句直接，有推进感。
你本局身份是预言家，属于好人阵营。
你只能依据你可见的信息行动和发言。
...
```

Action prompts should include role-specific instructions from `roleActionPromptSnapshot` when present.

Generation records already capture request snapshots. Those snapshots should show enough prompt detail for the host to inspect why a role made a choice.

## Mechanic Adapter Boundary

The first phase should not attempt a fully dynamic rules engine. Instead, it should introduce a small adapter boundary:

```ts
mechanicKey -> {
  buildActionPrompt,
  legalTargets,
  validateOutput,
  applyDraftPayload
}
```

Current mappings:

- `wolf_kill`: `wolf_kill_selected`
- `seer_check`: `seer_check_selected` and `seer_check_result`
- `witch_medicine`: `witch_death_info_shown`, `witch_antidote_decided`, `witch_poison_decided`
- `none`: no night mechanic

Existing rule functions can remain the source of truth. The adapter boundary is mainly to prevent future roles from expanding one giant action-generation function.

## Visibility and Knowledge

Role definitions declare symbolic knowledge rules. Runtime visibility remains event-based and must continue to be enforced by `projectVisibleEvents`.

Initial mappings:

- Werewolf: `own_role`, `wolf_teammates`
- Seer: `own_role`
- Witch: `own_role`, `witch_medicines`
- Villager: `own_role`

The role library must not bypass visibility rules. Prompt building must always consume player-visible context, not host/global event logs.

## Backward Compatibility

Existing game records may not have `characterSourceId`, `roleSourceId`, role prompt snapshots, or mechanic keys.

Repository normalization should tolerate old records by deriving missing fields from existing `gameRole`, `faction`, and player style fields where possible.

Old games should remain readable. New games should use the new preset-based snapshot structure.

## Testing Strategy

Tests should cover:

- Role definitions validate required fields and unique ids.
- Character definitions validate required fields and unique ids.
- Presets reject missing role ids, missing character ids, wrong player counts, and disabled entries.
- Game creation from the default preset reproduces the current 6-player setup.
- Player snapshots remain unchanged after library definitions are modified.
- LLM prompt composition includes both character and role prompt snapshots.
- Visibility remains player-specific after role-library migration.
- The existing complete 6-player game flow still reaches `game_ended`.
- Old game records without new snapshot fields still load.

## Migration Plan

Implementation should be incremental:

1. Add library models and seed data without changing game creation.
2. Add repositories and import scripts.
3. Change game creation to use the default preset.
4. Extend `PlayerSnapshot` and repository normalization.
5. Update prompt composition to include character and role snapshots.
6. Add mechanic adapter boundary around existing role actions.
7. Run full regression across editor, LLM generation, and playback.

## Non-Goals

- No role-library UI in this phase.
- No new playable roles in this phase.
- No full dynamic rule engine in this phase.
- No replay/recording changes in this phase.
- No remote database migration in this phase.

## Open Follow-Up Decisions

These are intentionally deferred until after this phase is implemented:

- Whether presets should randomize roles by default.
- Whether character selection should be random, weighted, or host-authored.
- Whether role definitions should include visual assets for replay.
- How complex future roles should declare custom events.
