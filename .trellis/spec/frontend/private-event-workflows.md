# Private Event Workflows

## 1. Scope / Trigger

Use this contract when an action spans the event log, LLM generation, Editor review, visibility projection, and director/public Preview. The event union in `src/core/events.ts` is the single payload owner.

## 2. Signatures

- Planning: `planNextDraft(input): DraftEvent | null` derives the next step only from active events.
- Editing: `applyDraftPayloadEdit(draft, edit): DraftEvent` preserves immutable payload fields.
- Visibility: `projectVisibleEvents(events, viewerPlayerId, context)` owns player-facing disclosure.
- Playback: `compilePublicPlayback(events, players, { audience })` owns Preview inclusion.

## 3. Contracts

- Shared discussion events use faction/player visibility and may enter director Preview.
- Sealed intermediate choices use `host_only`; player LLM contexts must consume the normal visibility projection.
- Internal Editor control events may be persisted for state derivation but must be explicitly excluded by `shouldIncludeEvent` when they have no Preview scene.
- Resolution events carry structured source choices, tallies, final result, and resolution method. Renderers format this structure; they do not re-parse display text.

## 4. Validation & Error Matrix

- Actor is dead or has the wrong role → reject in the server action.
- Target is dead or forbidden by the role mechanic → reject in the server action.
- Required human tiebreak is empty → reject confirmation.
- Human tiebreak is outside the tied candidates → reject the edit/confirmation.
- Internal or sealed event reaches public Preview/LLM context → regression failure.

## 5. Good / Base / Bad Cases

- Good: faction discussion → host-only sealed choices → faction resolution with structured details.
- Base: one eligible actor still passes through the same event stages without synthetic discussion.
- Bad: filter sealed choices only in prompt string construction while leaving the underlying event faction-visible.

## 6. Tests Required

- Planner chain test asserts every draft/event transition in order.
- Pure resolution test covers unique result and ties.
- Negative LLM-context test asserts sealed choices are absent.
- Editor/server tests assert legal candidates and reject invalid human edits.
- Playback test asserts internal selection and individual sealed choices are absent while discussion and final resolution are present.
- Public visibility test asserts all private workflow events remain absent.

## 7. Wrong vs Correct

### Wrong

```ts
visibility: { kind: "faction_private", faction: "wolves" }
// Later: remove teammate votes from a prompt string manually.
```

### Correct

```ts
visibility: { kind: "host_only" }
// Player context uses projectVisibleEvents; the sealed vote never enters it.
```
