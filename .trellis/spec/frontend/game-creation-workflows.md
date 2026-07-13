# Game Creation Workflows

## Scenario: New Game Run Mode Is a Persisted Snapshot

### 1. Scope / Trigger

- Trigger: creating, loading, listing, routing, or advancing a Game whose director selects autonomous game play or pre-authored scripted play.

### 2. Signatures

- `type GameRunMode = "game" | "scripted"`
- `parseGameRunMode(value: unknown): GameRunMode`
- `createGameFromPreset({ ..., runMode }): Game`
- `createGameFromPresetHomeAction(presetId, formData)` with `formData.runMode`
- `createGameFromSeatAssignmentsAction(formData)` with `formData.runMode`
- `validateGameRecord(value): GameRecord`

### 3. Contracts

- `Game.runMode` is immutable creation-time state. UI labels and routes consume the persisted value; they do not infer mode from theme, events, URL, or the presence of future episode data.
- New Game treats `runMode` and seat setup (`preset | random`) as independent choices.
- Both creation forms submit `runMode`; the Server Action parses it with the shared core parser and passes it through library-actions and game-actions.
- `game` routes to `/games/:id/editor`. `scripted` routes to `/games/:id/script`.
- `GameRecord.schemaVersion` and every current Game, Ruleset, Presenter, Game Script, Player, generation, voice-artifact, and Episode-state field are required. Repository reads validate the exact current keys and never inject defaults into persisted JSON.
- The current board is the 12-player ruleset. A six-player board or any incomplete/extra persisted shape is unsupported.
- Scripted Games advance only through the approved Episode plan; ordinary Game mode continues through the autonomous planner.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|---|---|
| Form value is `game` or `scripted` | Persist exactly that value. |
| Form value is missing or unsupported | Reject creation; do not silently choose a mode. |
| Persisted JSON omits `schemaVersion`, `runMode`, script, presenter, rules, Player fields, or Episode state | Reject record loading. |
| Persisted JSON contains an unsupported schema, mode, alias, extra field, or board shape | Reject record loading. |
| Scripted Game has no approved episode | Reject ordinary advancement and keep events/Draft empty. |
| Game-mode Game opens the scripted preparation route | Redirect to Editor. |
| Scripted Game opens Editor | Redirect to scripted preparation. |

### 5. Good / Base / Bad Cases

- Good: select Scripted + Random seats, persist both dimensions, then land on the locked script preparation page.
- Base: load a complete current GameRecord and route exclusively from its explicit `runMode`.
- Bad: infer missing fields, accept an old Player alias/six-player board, use the reusable theme template as evidence that a Game is scripted, or let a scripted Game fall through to `planNextDraft()`.

### 6. Tests Required

- Core parser accepts both modes and rejects missing or invalid values.
- Game creation snapshots the selected mode; repository tests round-trip one complete current record and reject every removed field/alias/schema shape.
- Server tests prove scripted creation persists and `continueGame()` rejects without adding events or a Draft.
- New Game component tests explain both flows and the pre-approval lock.
- Full test, typecheck, production build, and diff-check remain quality gates.

### 7. Wrong vs Correct

Wrong: `const mode = raw.runMode ?? "game"` or `const scripted = Boolean(game.script)`.

Correct: reject incomplete persisted records, then branch only on the validated `game.runMode`; Episode approval remains a separate required persisted state.
