# Player Stage Theme Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rebuild canvas preview around a persistent six-player left/right stage, with mansion murder as the first theme skin.

**Architecture:** `PlaybackItem[]` remains the fact model. The renderer first computes `StageLayout` with left seats 1-3 and right seats 4-6, then draws background, player cards, center stage, subtitle, and effects through theme skin functions.

**Tech Stack:** Next.js, React client component, TypeScript, Canvas 2D API, Vitest.

---

## Task 1: Stage Layout

**Files:**
- Create `src/components/preview/stage-layout.ts`
- Create `src/components/preview/stage-layout.test.ts`

- [ ] Write failing tests:
  - six players map seats 1-3 to left column
  - seats 4-6 map to right column
  - center and subtitle rects exist
- [ ] Implement `createSixPlayerStageLayout(players)`.
- [ ] Run `pnpm vitest run src/components/preview/stage-layout.test.ts`.
- [ ] Commit `feat: add player stage layout`.

## Task 2: Renderer Helpers For Player Stage

**Files:**
- Modify `src/components/preview/canvas-renderer.ts`
- Modify `src/components/preview/canvas-renderer.test.ts`

- [ ] Add tests for:
  - avatar placeholder drawing
  - status stamp drawing
  - vote result table drawing
  - subtitle bar drawing
- [ ] Implement:
  - `drawAvatar`
  - `drawStatusStamp`
  - `drawVoteResultTable`
  - `drawSubtitleBar`
- [ ] Run `pnpm vitest run src/components/preview/canvas-renderer.test.ts`.
- [ ] Commit `feat: add player stage canvas helpers`.

## Task 3: Theme API Revision

**Files:**
- Modify `src/components/preview/show-theme.ts`
- Modify `src/components/preview/show-theme.test.ts`

- [ ] Change theme render API from scene-kind renderers to stage-layer renderers:
  - `renderBackground`
  - `renderPlayerCard`
  - `renderCenterStage`
  - `renderSubtitle`
  - `renderEffect`
- [ ] Update tests to assert `renderThemeFrame` calls layers in order.
- [ ] Run `pnpm vitest run src/components/preview/show-theme.test.ts`.
- [ ] Commit `refactor: make show themes skin player stage`.

## Task 4: Mansion Murder Player Stage Skin

**Files:**
- Modify `src/components/preview/themes/mansion-murder.ts`
- Modify `src/components/preview/themes/mansion-murder.test.ts`

- [ ] Update tests:
  - all player names are drawn from fixed cards
  - current speaker is highlighted
  - dead players get gray/dead stamp
  - speech text is drawn through subtitle bar
  - vote scenes draw a center vote table and do not require vote lines
- [ ] Implement mansion murder skin:
  - suspect file cards
  - mansion backdrop
  - center case board
  - bottom transcript subtitle
  - speaker glow/death stamp
- [ ] Run `pnpm vitest run src/components/preview/themes/mansion-murder.test.ts`.
- [ ] Commit `feat: add mansion player stage skin`.

## Task 5: Wire PlaybackStage

**Files:**
- Modify `src/components/preview/playback-stage.tsx`
- Modify `src/components/preview/playback-stage.test.ts`

- [ ] Update tests:
  - preview remains canvas-only
  - no scene text in DOM
  - Record/Play/Reset remain outside canvas
- [ ] Wire `PlaybackStage` to:
  - compute `createSixPlayerStageLayout(scene.players)`
  - call `renderThemeFrame`
  - preserve one-click recording
- [ ] Run `pnpm vitest run src/components/preview/playback-stage.test.ts`.
- [ ] Commit `refactor: render preview through player stage`.

## Task 6: Full Verification

- [ ] Run focused tests:

```bash
pnpm vitest run \
  src/components/preview/stage-layout.test.ts \
  src/components/preview/canvas-renderer.test.ts \
  src/components/preview/show-theme.test.ts \
  src/components/preview/themes/mansion-murder.test.ts \
  src/components/preview/playback-stage.test.ts
```

- [ ] Run full checks:

```bash
pnpm typecheck
pnpm test
pnpm build
```

- [ ] Smoke check preview on a dev port:
  - canvas exists
  - Record exists
  - old recording links are absent
  - `/record` remains 404

## Acceptance Criteria

- Six players are always represented as cards, left 3 and right 3.
- Current speaker card highlights.
- Speech appears in bottom subtitle layer.
- Dead players are visibly gray/stamped.
- Vote scene uses center result board, not connecting lines.
- Theme skin controls look and feel without replacing stage structure.
- Recording still captures only canvas.
