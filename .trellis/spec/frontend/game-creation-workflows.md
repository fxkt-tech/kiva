# Game Creation Workflows

## Scenario: A Validated Lineup Becomes One Immutable Twelve-Player Game

### 1. Scope / Trigger

- Trigger: loading or editing catalog content, creating a Game from a saved/custom Lineup, loading a GameRecord, or routing by run mode.
- The board is fixed at 12 seats. The Actor pool is extensible and a Game may select any 12 distinct enabled Actors.

### 2. Signatures

```ts
type Lineup = {
  id: string;
  name: string;
  rulesetId: "classic_twelve";
  seats: readonly {
    seatNo: number;
    ruleRoleId: RuleRoleId;
    actorId: string;
  }[]; // exactly 12
  enabled: boolean;
  revision: number;
};

createGameFromLineup(input: {
  gameId: GameId;
  title: string;
  createdAt: string;
  ruleset: Ruleset;
  lineup: Lineup;
  presenter: PresenterDefinition;
  script: GameScriptDefinition;
  actors: readonly ActorDefinition[];
  runMode: "game" | "scripted";
}): Game

ContentCatalog.load(): Promise<ContentCatalogSnapshot>
ContentCatalog.save(data: ContentCatalogData): Promise<void>
ContentCatalog.update(change): Promise<Result>
validateGameRecord(value): GameRecord // schemaVersion 2 only
```

### 3. Contracts

- `Lineup.seats` is the only seat-assignment truth. There are no parallel role IDs, Actor IDs, Preset arrays, or seat-level model overrides.
- A legal Lineup has seat numbers 1–12 exactly once, 12 distinct enabled Actor references, and the current `classic_twelve` Rule Role composition.
- Game creation sorts seats, snapshots each complete Actor plus its code-owned Rule Role, and assigns stable Player IDs `p1`–`p12`. Existing Games never reread the live Actor catalog.
- `Game.players` is the sole Rule Role truth. `role_assigned` events are private notifications and must exactly match the corresponding immutable Player snapshot; replay never replaces snapshot roles with event payloads.
- Qin Chuan has one library invariant: exactly one original Actor is `qin_chuan` / `秦川`. No creation path requires that Actor, gives it a fixed seat, or fills it into a custom cast.
- `Game.runMode` is explicit immutable creation state. `game` routes to `/games/:id/editor`; `scripted` routes to `/games/:id/script` and starts with `{status:"idle"}`. Creation itself never starts Script Author.
- One enabled Presenter and one selected enabled Script are resolved and deep-snapshotted into every new Game.
- `ContentCatalog` owns the four editable collections (`actors.json`, `lineups.json`, `presenters.json`, `scripts.json`) and exposes code-owned Rule Roles read-only. Its public `load`, `save`, and `update` methods own the directory lock; callers never compose unlocked multi-file reads or read-modify-write sequences.
- Catalog save validates the complete cross-referenced snapshot before replacing any file. `update` serializes revision checks with persistence so concurrent edits cannot both accept the same revision.
- GameRecord schema 2 is a clean break. Missing, extra, old Character/Role/Preset fields or a non-current schema are rejected; there is no normalization/migration path.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|---|---|
| Lineup does not contain exactly seats 1–12 | Reject before Game creation. |
| Actor ID is unknown, disabled, or repeated | Reject; never fill from a default cast. |
| Rule Role is unknown or composition differs from the Ruleset | Reject; Studio cannot invent a mechanic. |
| Saved Lineup is disabled | Reject new Game creation from it. |
| Presenter enabled count is not exactly one | Reject the complete catalog and Game creation. |
| No enabled Script or selected Script is unknown/disabled | Reject without saving a partial Game. |
| Concurrent Actor/Lineup edit submits a stale revision | Reject with expected/received revision; retain the winner. |
| `role_assigned` event differs from `Game.players` | Reject the GameRecord as divergent truth. |
| `runMode` is missing/invalid | Reject; do not infer from URL, script, or Episode state. |
| Persisted record is old/incomplete/has aliases or extra fields | Reject; do not inject defaults. |
| Scripted Game has no approved Episode and tries ordinary advancement | Reject while retaining empty events/Draft. |

### 5. Good / Base / Bad Cases

- Good: choose 12 enabled Actors that omit Qin Chuan, assign the legal six Rule Roles, snapshot them, and author the exact resulting Game.
- Good: edit the Actor Library after Game creation; the existing Game still uses its original Actor/model/voice snapshot.
- Base: choose a saved enabled Lineup and the first enabled Script, then route from the explicit run mode.
- Bad: start from a fixed 12-person seed and silently replace only names at runtime.
- Bad: treat `role_assigned` events as a second mutable role source.
- Bad: read four catalog files without one catalog-owned lock or let two revision-2 edits both win.

### 6. Tests Required

- Lineup tests cover exact seat/cardinality, unique Actors, references, enabled state, and each Rule Role count.
- Creation tests prove arbitrary selected casts (including one without Qin Chuan), deep Actor/Rule Role/model snapshots, mode routing, and idle scripted state.
- Qin tests prove the stable original identity exists exactly once in the library and is otherwise ordinary across all six Rule Roles.
- Repository tests round-trip schema 2 and reject old fields, incomplete Players, mismatched role notifications, unsupported schemas, and extra keys.
- Content Catalog tests cover four-file persistence, complete pre-write validation, code-owned Rule Roles, production pool constraints, and concurrent revision serialization.
- UI tests cover saved Lineup and custom/random 12-Actor creation with independent Script/run-mode choices.
- Quality gates: `pnpm typecheck`, `pnpm test`, `pnpm build`, and `git diff --check`.

### 7. Wrong vs Correct

Wrong:

```ts
const actors = defaultCharacters.slice(0, 12);
const role = latestRoleAssignedEvent(playerId) ?? player.role;
const mode = raw.runMode ?? "game";
```

Correct:

```ts
const catalog = await contentCatalog.load();
const lineup = requireEnabledLineup(catalog, lineupId);
const game = createGameFromLineup({
  lineup,
  actors: catalog.actors,
  presenter: requireSoleEnabledPresenter(catalog),
  script: requireEnabledScript(catalog, scriptId),
  runMode: parseGameRunMode(raw.runMode),
  // ids/timestamps/ruleset omitted here
});
```

Validate and snapshot one exact cast once; every Author, runtime, preview, and export consumer reads that immutable Game.
