# Show Theme Pack Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the plain canvas preview with a theme-pack rendering pipeline and a first `mansion_murder` show theme while preserving one-click canvas recording.

**Architecture:** Keep `PlaybackItem[]` as gameplay facts. Move drawing primitives into `canvas-renderer`, add `ShowThemePack` registry/director, and make `PlaybackStage` call `renderThemeFrame()` instead of hard-coded drawing functions.

**Tech Stack:** Next.js, React client component, TypeScript, Canvas 2D API, Vitest.

---

## File Structure

- Create `src/components/preview/canvas-renderer.ts`
  - Shared Canvas 2D helpers: background, text wrapping, panels, glow text, player files, vote lines, vignette/noise.
- Create `src/components/preview/show-theme.ts`
  - Theme types, `DEFAULT_SHOW_THEME_ID`, registry, fallback resolver, and `renderThemeFrame()`.
- Create `src/components/preview/themes/mansion-murder.ts`
  - Mansion murder theme tokens and scene renderers.
- Modify `src/components/preview/playback-stage.tsx`
  - Remove inline scene drawing helpers.
  - Call `renderThemeFrame()` from the selected theme.
- Add `src/components/preview/show-theme.test.ts`
  - Registry and director tests.
- Add `src/components/preview/themes/mansion-murder.test.ts`
  - Smoke tests for each scene kind through a fake canvas context.
- Modify `src/components/preview/playback-stage.test.ts`
  - Ensure preview remains canvas-only and renders with the theme path.

## Task 1: Theme Registry And Director

**Files:**
- Create: `src/components/preview/show-theme.ts`
- Test: `src/components/preview/show-theme.test.ts`

- [ ] **Step 1: Write the failing registry/director tests**

```ts
import { describe, expect, it, vi } from "vitest";
import type { PlaybackItem } from "@/core/playback";
import {
  DEFAULT_SHOW_THEME_ID,
  getShowTheme,
  renderThemeFrame,
  type ShowThemePack,
} from "./show-theme";

describe("show theme registry", () => {
  it("falls back to the default mansion murder theme for unknown ids", () => {
    expect(getShowTheme("missing-theme").id).toBe(DEFAULT_SHOW_THEME_ID);
  });

  it("routes scene rendering through the matching theme renderer", () => {
    const renderSpeech = vi.fn();
    const renderAnnouncement = vi.fn();
    const theme: ShowThemePack = {
      id: "test_theme",
      name: "Test Theme",
      tokens: {
        background: "#000",
        panel: "#111",
        text: "#fff",
        muted: "#777",
        accent: "#0ff",
        danger: "#f00",
        good: "#0f0",
        wolf: "#f44",
      },
      render: {
        renderPhase: vi.fn(),
        renderAnnouncement,
        renderSpeech,
        renderVote: vi.fn(),
        renderResolution: vi.fn(),
        renderEnd: vi.fn(),
      },
    };

    renderThemeFrame(fakeContext(), {
      theme,
      scene: scene({ kind: "speech" }),
      items: [scene({ kind: "speech" })],
      timeMs: 1200,
    });

    expect(renderSpeech).toHaveBeenCalledTimes(1);
    expect(renderAnnouncement).not.toHaveBeenCalled();
  });
});

function scene(overrides: Partial<PlaybackItem> = {}): PlaybackItem {
  return {
    index: 1,
    phase: "speech",
    kind: "announcement",
    title: "Title",
    text: "Text",
    details: [],
    durationMs: 2000,
    startsAtMs: 0,
    players: [],
    ...overrides,
  };
}

function fakeContext(): CanvasRenderingContext2D {
  return {
    canvas: { width: 1920, height: 1080 },
    fillRect: vi.fn(),
    strokeRect: vi.fn(),
    fillText: vi.fn(),
    measureText: vi.fn(() => ({ width: 10 })),
    beginPath: vi.fn(),
    moveTo: vi.fn(),
    lineTo: vi.fn(),
    stroke: vi.fn(),
    arc: vi.fn(),
    save: vi.fn(),
    restore: vi.fn(),
    createLinearGradient: vi.fn(() => ({ addColorStop: vi.fn() })),
  } as unknown as CanvasRenderingContext2D;
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `pnpm vitest run src/components/preview/show-theme.test.ts`

Expected: FAIL because `./show-theme` does not exist.

- [ ] **Step 3: Implement minimal theme registry/director**

Create `src/components/preview/show-theme.ts` with:

```ts
import type { PlaybackItem, PlaybackSceneKind } from "@/core/playback";
import { mansionMurderTheme } from "./themes/mansion-murder";

export const DEFAULT_SHOW_THEME_ID = "mansion_murder";
export type ShowThemeId = typeof DEFAULT_SHOW_THEME_ID | string;

export type ShowThemeTokens = {
  readonly background: string;
  readonly panel: string;
  readonly text: string;
  readonly muted: string;
  readonly accent: string;
  readonly danger: string;
  readonly good: string;
  readonly wolf: string;
};

export type ThemeRenderInput = {
  readonly ctx: CanvasRenderingContext2D;
  readonly scene: PlaybackItem;
  readonly items: readonly PlaybackItem[];
  readonly timeMs: number;
  readonly sceneTimeMs: number;
  readonly enterProgress: number;
};

export type ShowThemeRenderers = {
  readonly renderPhase: (input: ThemeRenderInput) => void;
  readonly renderAnnouncement: (input: ThemeRenderInput) => void;
  readonly renderSpeech: (input: ThemeRenderInput) => void;
  readonly renderVote: (input: ThemeRenderInput) => void;
  readonly renderResolution: (input: ThemeRenderInput) => void;
  readonly renderEnd: (input: ThemeRenderInput) => void;
};

export type ShowThemePack = {
  readonly id: ShowThemeId;
  readonly name: string;
  readonly tokens: ShowThemeTokens;
  readonly render: ShowThemeRenderers;
};

const themeRegistry: Record<string, ShowThemePack> = {
  [mansionMurderTheme.id]: mansionMurderTheme,
};

export function getShowTheme(themeId: string | null | undefined): ShowThemePack {
  return themeRegistry[themeId ?? ""] ?? themeRegistry[DEFAULT_SHOW_THEME_ID]!;
}

export function renderThemeFrame(
  ctx: CanvasRenderingContext2D,
  input: {
    readonly theme: ShowThemePack;
    readonly scene: PlaybackItem;
    readonly items: readonly PlaybackItem[];
    readonly timeMs: number;
  },
): void {
  const sceneTimeMs = Math.max(0, input.timeMs - input.scene.startsAtMs);
  const enterProgress = Math.min(1, sceneTimeMs / 450);
  const renderInput: ThemeRenderInput = {
    ctx,
    scene: input.scene,
    items: input.items,
    timeMs: input.timeMs,
    sceneTimeMs,
    enterProgress,
  };
  rendererForKind(input.theme, input.scene.kind)(renderInput);
}

function rendererForKind(
  theme: ShowThemePack,
  kind: PlaybackSceneKind,
): (input: ThemeRenderInput) => void {
  switch (kind) {
    case "phase":
      return theme.render.renderPhase;
    case "speech":
      return theme.render.renderSpeech;
    case "vote":
      return theme.render.renderVote;
    case "resolution":
      return theme.render.renderResolution;
    case "announcement":
      return theme.render.renderAnnouncement;
  }
}
```

- [ ] **Step 4: Add temporary mansion theme stub**

Create `src/components/preview/themes/mansion-murder.ts` with renderers that only fill the background and write the scene title. This file will be replaced in Task 3.

- [ ] **Step 5: Run test to verify it passes**

Run: `pnpm vitest run src/components/preview/show-theme.test.ts`

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src/components/preview/show-theme.ts src/components/preview/show-theme.test.ts src/components/preview/themes/mansion-murder.ts
git commit -m "feat: add show theme registry"
```

## Task 2: Canvas Renderer Helpers

**Files:**
- Create: `src/components/preview/canvas-renderer.ts`
- Test: `src/components/preview/canvas-renderer.test.ts`

- [ ] **Step 1: Write helper tests**

Add tests for:

- `drawTextBlock` wraps text when width is exceeded.
- `drawPanel` calls `fillRect` and `strokeRect`.
- `drawPlayerFile` draws seat/name/status text.

- [ ] **Step 2: Run helper tests to verify fail**

Run: `pnpm vitest run src/components/preview/canvas-renderer.test.ts`

Expected: FAIL because helper file does not exist.

- [ ] **Step 3: Implement helpers**

Implement:

- `clearCanvas(ctx, color)`
- `drawTextBlock(ctx, text, x, y, options)`
- `drawPanel(ctx, rect, options)`
- `drawGlowText(ctx, text, x, y, options)`
- `drawPlayerFile(ctx, player, rect, options)`
- `drawVoteLine(ctx, from, to, options)`
- `drawVignette(ctx)`

- [ ] **Step 4: Run helper tests**

Run: `pnpm vitest run src/components/preview/canvas-renderer.test.ts`

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/components/preview/canvas-renderer.ts src/components/preview/canvas-renderer.test.ts
git commit -m "feat: add canvas renderer helpers"
```

## Task 3: Mansion Murder Theme

**Files:**
- Modify: `src/components/preview/themes/mansion-murder.ts`
- Test: `src/components/preview/themes/mansion-murder.test.ts`

- [ ] **Step 1: Write mansion renderer smoke tests**

Test each scene kind:

- phase renders chapter card labels
- speech renders active speaker and transcript
- vote renders red string/evidence treatment
- announcement renders case bulletin
- resolution/end renders case closed treatment

- [ ] **Step 2: Run mansion tests to verify fail**

Run: `pnpm vitest run src/components/preview/themes/mansion-murder.test.ts`

Expected: FAIL because current stub does not draw theme-specific operations.

- [ ] **Step 3: Implement mansion theme renderers**

Use `canvas-renderer` helpers. Required visual treatment:

- dark mansion background with top title bar, vignette, and grain/light bands
- suspect file cards for players
- speech transcript panel
- red evidence-string vote lines
- death/out state as darkened file with red stamp
- game end as case-closed reveal

- [ ] **Step 4: Run mansion tests**

Run: `pnpm vitest run src/components/preview/themes/mansion-murder.test.ts`

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/components/preview/themes/mansion-murder.ts src/components/preview/themes/mansion-murder.test.ts
git commit -m "feat: add mansion murder playback theme"
```

## Task 4: Wire PlaybackStage To Theme Director

**Files:**
- Modify: `src/components/preview/playback-stage.tsx`
- Modify: `src/components/preview/playback-stage.test.ts`

- [ ] **Step 1: Write/update stage tests**

Assert:

- stage renders one 1920x1080 canvas
- scene text is not in DOM
- `Record` is visible
- old links `Open clean preview`, `Open recording studio`, `Start recording` are absent

- [ ] **Step 2: Run stage tests**

Run: `pnpm vitest run src/components/preview/playback-stage.test.ts`

Expected: FAIL until `PlaybackStage` calls `renderThemeFrame`.

- [ ] **Step 3: Replace inline drawing with theme director**

In `PlaybackStage`, compute:

```ts
const theme = getShowTheme(DEFAULT_SHOW_THEME_ID);
renderThemeFrame(context, { theme, scene, items, timeMs });
```

Remove the inline `drawSceneContent`, `drawPlayers`, and related helpers.

- [ ] **Step 4: Run stage tests**

Run: `pnpm vitest run src/components/preview/playback-stage.test.ts`

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/components/preview/playback-stage.tsx src/components/preview/playback-stage.test.ts
git commit -m "refactor: render playback through show theme"
```

## Task 5: Full Verification

**Files:**
- No new files.

- [ ] **Step 1: Run focused preview tests**

Run:

```bash
pnpm vitest run \
  src/components/preview/canvas-renderer.test.ts \
  src/components/preview/show-theme.test.ts \
  src/components/preview/themes/mansion-murder.test.ts \
  src/components/preview/playback-stage.test.ts
```

Expected: PASS.

- [ ] **Step 2: Run full verification**

Run:

```bash
pnpm typecheck
pnpm test
pnpm build
```

Expected: all pass.

- [ ] **Step 3: Manual smoke check**

Start dev server on a free port and open:

```text
/games/<gameId>/preview
```

Confirm:

- visible canvas has mansion murder identity
- `Record` starts full playback from 0
- recording stops at the end
- download link appears
- downloaded WebM contains only canvas content

- [ ] **Step 4: Commit if any verification fixes were needed**

```bash
git status --short
```

Expected: clean.

## Self-Review

- Spec coverage: theme registry, theme director, canvas helpers, mansion murder theme, preview wiring, recording preservation are covered.
- Scope check: MP4, audio, image generation, and theme editor remain out of scope.
- Type consistency: `ShowThemePack`, `ThemeRenderInput`, `PlaybackItem`, and renderer method names are consistent across tasks.
