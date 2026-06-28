# New Game Dialog Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the home page "New game" redirect with an in-page dialog that can create a game from a read-only preset or from an adjustable random six-player setup.

**Architecture:** Home page loads library data alongside games and renders a client dialog component. Random mode uses pure helper functions for six-player fixed role counts and character assignment, while server actions create games from either an existing preset or a temporary unsaved preset.

**Tech Stack:** Next.js App Router server actions, React client component, TypeScript, existing library/game repositories.

---

### Task 1: Random Setup Helpers

**Files:**
- Create: `src/components/home/new-game-setup.ts`
- Test: `src/components/home/new-game-setup.test.ts`

- [ ] **Step 1: Add tests for fixed six-player role counts and validation**

Cover:
- generated setup has two werewolves, one seer, one witch, and two villagers.
- validation rejects changed role counts.
- validation rejects duplicate characters.

- [ ] **Step 2: Implement helpers**

Export:
- `requiredSixPlayerRoleCounts`
- `createRandomSeatSetup`
- `validateSeatSetup`

### Task 2: Server Actions

**Files:**
- Modify: `src/server/library-actions.ts`
- Modify: `src/app/actions.ts`
- Test: `src/server/__tests__/library-actions.test.ts`

- [ ] **Step 1: Add temporary preset creation behavior**

Add `createGameFromTemporaryPreset(preset)` to library actions. It validates against current library data and creates a game without saving the temporary preset.

- [ ] **Step 2: Add home page server actions**

Add:
- `createGameFromPresetHomeAction(presetId)`
- `createGameFromSeatAssignmentsAction(formData)`

Both redirect to the new game editor.

### Task 3: Home Dialog UI

**Files:**
- Create: `src/components/home/new-game-dialog.tsx`
- Modify: `src/app/page.tsx`
- Test: `src/components/home/new-game-dialog.test.tsx`

- [ ] **Step 1: Render New Game as dialog trigger**

Home page should no longer use the old redirect form. It should load library data and pass enabled roles, enabled characters, and creatable presets to the dialog.

- [ ] **Step 2: Implement preset and random modes**

Preset mode renders read-only preset seats and submits the selected preset id. Random mode renders editable role/character selects, a reroll button, validation messages, and submits temporary seat assignments only when valid.

### Task 4: Verification

**Files:**
- No new files.

- [ ] **Step 1: Run focused checks**

Run:

```bash
pnpm typecheck
pnpm vitest run src/components/home/new-game-setup.test.ts src/components/home/new-game-dialog.test.tsx src/server/__tests__/library-actions.test.ts
```

Expected: typecheck passes. Vitest may be blocked by the known local `ERR_REQUIRE_ESM` startup issue; if blocked, verify core helpers with `tsx` one-off commands.
