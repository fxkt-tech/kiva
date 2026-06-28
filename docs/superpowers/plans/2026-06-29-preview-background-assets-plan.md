# Preview Background Assets Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Use the provided day/night images as Preview canvas backgrounds and store them under `kivdb`.

**Architecture:** Move images into `kivdb/assets/preview`, expose them through a small Next route, preload them in `PlaybackStage`, and let the show theme choose the image by scene phase. Keep the existing gradient background as a fallback.

**Tech Stack:** Next.js route handlers, React canvas, TypeScript, existing preview theme architecture.

---

### Task 1: Move Assets

**Files:**
- Move: `背景图-白天.png` -> `kivdb/assets/preview/day-background.png`
- Move: `背景图-夜晚.png` -> `kivdb/assets/preview/night-background.png`

- [ ] **Step 1: Move and rename the images**

Use normal filesystem moves and keep the files in `kivdb/assets/preview`.

### Task 2: Serve Assets

**Files:**
- Create: `src/app/kivdb-assets/preview/[file]/route.ts`

- [ ] **Step 1: Add route handler**

Serve only `day-background.png` and `night-background.png` from `kivdb/assets/preview`.

### Task 3: Draw Backgrounds

**Files:**
- Modify: `src/components/preview/show-theme.ts`
- Modify: `src/components/preview/playback-stage.tsx`
- Modify: `src/components/preview/themes/mansion-murder.ts`

- [ ] **Step 1: Add loaded background images to theme input**

`PlaybackStage` preloads day/night images and passes them to `renderThemeFrame`.

- [ ] **Step 2: Draw day/night image backgrounds**

`mansion-murder` chooses night image for `scene.phase === "night"` and day image otherwise. It draws the image with cover crop and falls back to the old gradient if the image is not loaded.

### Task 4: Verification

Run:

```bash
pnpm typecheck
pnpm build
```

Expected: both pass.
