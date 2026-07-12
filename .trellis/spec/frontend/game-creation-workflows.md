# Game Creation Workflows

## Scenario: New Game Run Mode Is a Persisted Snapshot

### 1. Scope / Trigger

- Trigger: creating, loading, listing, routing, or advancing a Game whose director selects autonomous game play or pre-authored scripted play.

### 2. Signatures

- `type GameRunMode = "game" | "scripted"`
- `parseGameRunMode(value: unknown): GameRunMode`
- `normalizeGameRunMode(value: unknown): GameRunMode`
- `createGameFromPreset({ ..., runMode? }): Game`
- `createGameFromPresetHomeAction(presetId, formData)` with `formData.runMode`
- `createGameFromSeatAssignmentsAction(formData)` with `formData.runMode`

### 3. Contracts

- `Game.runMode` is immutable creation-time state. UI labels and routes consume the persisted value; they do not infer mode from theme, events, URL, or the presence of future episode data.
- New Game treats `runMode` and seat setup (`preset | random`) as independent choices.
- Both creation forms submit `runMode`; the Server Action parses it with the shared core parser and passes it through library-actions and game-actions.
- `game` routes to `/games/:id/editor`. `scripted` routes to `/games/:id/script`.
- A historical record with a missing field normalizes to `game` in memory without rewriting storage. Any present unsupported value is rejected.
- Until episode approval is implemented, `continueGame()` rejects every scripted Game before inspecting or planning a Draft.

### 4. Validation & Error Matrix

| Condition | Required behavior |
|---|---|
| Form value is `game` or `scripted` | Persist exactly that value. |
| Form value is missing or unsupported | Reject creation; do not silently choose a mode. |
| Historical JSON omits `runMode` | Normalize to `game` on read/list only. |
| Persisted JSON contains an unsupported mode | Reject record loading. |
| Scripted Game has no approved episode | Reject ordinary advancement and keep events/Draft empty. |
| Game-mode Game opens the scripted preparation route | Redirect to Editor. |
| Scripted Game opens Editor | Redirect to scripted preparation. |

### 5. Good / Base / Bad Cases

- Good: select Scripted + Random seats, persist both dimensions, then land on the locked script preparation page.
- Base: load an old Game and see it labeled Game mode without a migration write.
- Bad: use the reusable theme template as evidence that a Game is scripted, or let a scripted Game fall through to `planNextDraft()`.

### 6. Tests Required

- Core parser accepts both modes, defaults only `undefined`, and rejects invalid values.
- Game creation snapshots scripted mode; repository compatibility injects game mode for old records.
- Server tests prove scripted creation persists and `continueGame()` rejects without adding events or a Draft.
- New Game component tests explain both flows and the pre-approval lock.
- Full test, typecheck, production build, and diff-check remain quality gates.

### 7. Wrong vs Correct

Wrong: `const scripted = Boolean(game.script)` because every Game has a reusable theme script snapshot.

Correct: branch only on `game.runMode`, while future episode approval remains a separate persisted state.
