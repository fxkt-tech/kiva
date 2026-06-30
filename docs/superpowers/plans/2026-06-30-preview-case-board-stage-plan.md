# Preview Case Board Stage Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Split the Pixi preview center stage so speech scenes keep the player-focused stage and non-speech scenes render through a unified Case Board stage.

**Architecture:** Add a small pure content-selection helper for Case Board rows, then refactor the Pixi center-stage class into a dispatcher that owns a speech stage and a Case Board stage. Keep `ShotFrame`, playback compilation, subtitles, seat cards, header, and controls unchanged.

**Tech Stack:** Next.js, React, TypeScript, PixiJS v8, Vitest.

---

## File Structure

- Create: `src/components/preview/pixi/case-board-content.ts`
  - Pure helper for deriving Case Board label, primary text, and context rows from `ShotFrame`.
  - No Pixi imports; easy to unit test.
- Create: `src/components/preview/pixi/case-board-content.test.ts`
  - Unit tests for `details` priority, highlighted-player fallback, and neutral fallback.
- Modify: `src/components/preview/pixi/pixi-preview-renderer.ts`
  - Replace `MainShot` with a `CenterStage` dispatcher.
  - Rename existing player-focused `MainShot` behavior to `SpeechStage`.
  - Add `CaseBoardStage` Pixi display class.
  - Remove the non-speech floating `lensOverlay` rectangle.
- No changes:
  - `src/core/playback.ts`
  - `src/components/preview/shot-engine/director.ts`
  - `src/components/preview/playback-stage.tsx`

## Task 1: Add Case Board Content Rules

**Files:**
- Create: `src/components/preview/pixi/case-board-content.ts`
- Create: `src/components/preview/pixi/case-board-content.test.ts`

- [ ] **Step 1: Write failing tests for Case Board content priority**

Create `src/components/preview/pixi/case-board-content.test.ts`:

```ts
import { describe, expect, it } from "vitest";
import type { PlaybackItem, PlaybackScenePlayer } from "@/core/playback";
import type { ShotFrame } from "../shot-engine/types";
import { caseBoardContentForFrame } from "./case-board-content";

describe("caseBoardContentForFrame", () => {
  it("uses scene details before highlighted players", () => {
    const content = caseBoardContentForFrame(frame({
      scene: scene({
        kind: "resolution",
        title: "投票结算",
        text: "放逐出局：2 号 林夏。",
        details: ["1 号 秦川 -> 2 号 林夏", "3 号 周知 -> 弃票"],
        players: [player(2, { highlighted: true, roleName: "狼人" })],
      }),
      highlightedPlayers: [player(2, { highlighted: true, roleName: "狼人" })],
    }));

    expect(content.kindLabel).toBe("RESOLUTION");
    expect(content.title).toBe("投票结算");
    expect(content.body).toBe("放逐出局：2 号 林夏。");
    expect(content.rows).toEqual([
      { tone: "detail", label: "01", text: "1 号 秦川 -> 2 号 林夏" },
      { tone: "detail", label: "02", text: "3 号 周知 -> 弃票" },
    ]);
  });

  it("uses highlighted players when details are empty", () => {
    const highlighted = player(9, {
      highlighted: true,
      name: "苏瑾",
      roleName: "平民",
      status: "dead",
    });

    const content = caseBoardContentForFrame(frame({
      scene: scene({
        kind: "announcement",
        title: "昨夜死讯",
        text: "昨夜死亡：9 号 苏瑾。",
        details: [],
        players: [highlighted],
      }),
      highlightedPlayers: [highlighted],
    }));

    expect(content.rows).toEqual([
      {
        tone: "dead-player",
        label: "09",
        text: "苏瑾",
        meta: "身份：平民",
      },
    ]);
  });

  it("falls back to a neutral public record row", () => {
    const content = caseBoardContentForFrame(frame({
      scene: scene({
        kind: "phase",
        title: "第 1 夜开始",
        text: "夜晚开始。",
        details: [],
        players: [player(1)],
      }),
      highlightedPlayers: [],
    }));

    expect(content.rows).toEqual([
      { tone: "neutral", label: "FILE", text: "PUBLIC RECORD" },
    ]);
  });
});

function scene(overrides: Partial<PlaybackItem> = {}): PlaybackItem {
  return {
    index: 1,
    phase: "night",
    kind: "phase",
    title: "第 1 夜开始",
    text: "夜晚开始。",
    details: [],
    durationMs: 2000,
    startsAtMs: 0,
    players: [],
    ...overrides,
  };
}

function frame(overrides: Partial<ShotFrame> = {}): ShotFrame {
  const baseScene = overrides.scene ?? scene();
  return {
    scene: baseScene,
    items: [baseScene],
    layout: {
      safe: { x: 56, y: 48, width: 1808, height: 984 },
      topBar: { x: 0, y: 0, width: 1920, height: 96 },
      seatSlots: [],
      mainStage: { x: 480, y: 130, width: 960, height: 700 },
      portrait: { x: 524, y: 180, width: 872, height: 520 },
      eventPanel: { x: 520, y: 170, width: 880, height: 620 },
      subtitle: { x: 220, y: 852, width: 1480, height: 168 },
    },
    clock: {
      absoluteMs: 0,
      sceneMs: 0,
      durationMs: 2000,
      progress: 0,
      enterProgress: 0,
      exitProgress: 1,
    },
    players: baseScene.players.map((candidate) => ({
      ...candidate,
      emphasis: candidate.highlighted ? "highlighted" : "normal",
    })),
    activePlayer: null,
    highlightedPlayers: [],
    subtitle: null,
    backgroundImages: { day: null, night: null },
    avatarImages: {},
    ...overrides,
  };
}

function player(
  seatNo: number,
  overrides: Partial<PlaybackScenePlayer> = {},
): PlaybackScenePlayer {
  return {
    playerId: `player_${seatNo}` as PlaybackScenePlayer["playerId"],
    seatNo,
    name: `玩家${seatNo}`,
    avatar: null,
    roleName: "平民",
    status: "alive",
    highlighted: false,
    ...overrides,
  };
}
```

- [ ] **Step 2: Run the new test to verify it fails**

Run:

```bash
pnpm vitest run src/components/preview/pixi/case-board-content.test.ts
```

Expected: FAIL because `src/components/preview/pixi/case-board-content.ts` does not exist.

- [ ] **Step 3: Implement the pure Case Board helper**

Create `src/components/preview/pixi/case-board-content.ts`:

```ts
import type { PlaybackSceneKind } from "@/core/playback";
import type { RenderablePlayer, ShotFrame } from "../shot-engine/types";

export type CaseBoardRowTone =
  | "detail"
  | "alive-player"
  | "dead-player"
  | "neutral";

export type CaseBoardRow = {
  readonly tone: CaseBoardRowTone;
  readonly label: string;
  readonly text: string;
  readonly meta?: string;
};

export type CaseBoardContent = {
  readonly kindLabel: string;
  readonly title: string;
  readonly body: string;
  readonly rows: readonly CaseBoardRow[];
};

export function caseBoardContentForFrame(frame: ShotFrame): CaseBoardContent {
  const detailRows = frame.scene.details.map<CaseBoardRow>((detail, index) => ({
    tone: "detail",
    label: String(index + 1).padStart(2, "0"),
    text: detail,
  }));

  const playerRows = frame.highlightedPlayers.map(playerRow);
  const rows = detailRows.length > 0
    ? detailRows
    : playerRows.length > 0
      ? playerRows
      : [{ tone: "neutral", label: "FILE", text: "PUBLIC RECORD" } satisfies CaseBoardRow];

  return {
    kindLabel: kindLabelForScene(frame.scene.kind),
    title: frame.scene.title,
    body: frame.scene.text,
    rows,
  };
}

function playerRow(player: RenderablePlayer): CaseBoardRow {
  return {
    tone: player.status === "dead" ? "dead-player" : "alive-player",
    label: String(player.seatNo).padStart(2, "0"),
    text: player.name,
    meta: `身份：${player.roleName}`,
  };
}

function kindLabelForScene(kind: PlaybackSceneKind): string {
  switch (kind) {
    case "phase":
      return "PHASE";
    case "announcement":
      return "ANNOUNCEMENT";
    case "vote":
      return "VOTE";
    case "resolution":
      return "RESOLUTION";
    case "speech":
      return "SPEECH";
  }
}
```

- [ ] **Step 4: Run the helper test**

Run:

```bash
pnpm vitest run src/components/preview/pixi/case-board-content.test.ts
```

Expected: PASS.

- [ ] **Step 5: Commit the helper**

Run:

```bash
git add src/components/preview/pixi/case-board-content.ts src/components/preview/pixi/case-board-content.test.ts
git commit -m "test: add case board content rules"
```

## Task 2: Refactor Center Stage Dispatch

**Files:**
- Modify: `src/components/preview/pixi/pixi-preview-renderer.ts`

- [ ] **Step 1: Rename the scene field from `mainShot` to `centerStage`**

In `src/components/preview/pixi/pixi-preview-renderer.ts`, change the `PixiPreviewScene` fields and constructor references:

```ts
  private readonly centerStage: CenterStage;
```

```ts
    this.centerStage = new CenterStage(theme);
```

```ts
    this.centerColumn.addChild(this.centerStage.view, this.subtitle.view);
```

In `update(frame: ShotFrame)`, replace:

```ts
    this.mainShot.update(frame);
```

with:

```ts
    this.centerStage.update(frame);
```

In `renderEmpty()`, replace:

```ts
    this.mainShot.renderEmpty();
```

with:

```ts
    this.centerStage.renderEmpty();
```

- [ ] **Step 2: Rename existing `MainShot` class to `SpeechStage`**

Change:

```ts
class MainShot {
```

to:

```ts
class SpeechStage {
```

Keep its constructor and methods otherwise intact in this step.

- [ ] **Step 3: Add `CenterStage` dispatcher above `SpeechStage`**

Insert this class immediately before `class SpeechStage`:

```ts
class CenterStage {
  readonly view = new LayoutContainer();
  private readonly speechStage: SpeechStage;
  private readonly caseBoardStage: CaseBoardStage;

  constructor(theme: PixiPreviewTheme) {
    this.speechStage = new SpeechStage(theme);
    this.caseBoardStage = new CaseBoardStage(theme);
    this.view.layout = {
      position: "absolute",
      left: 0,
      top: 0,
      width: mainStageWidth(theme),
      height: mainStageHeight(theme),
      alignItems: "stretch",
    };
    this.view.addChild(this.speechStage.view, this.caseBoardStage.view);
  }

  update(frame: ShotFrame): void {
    const isSpeech = frame.scene.kind === "speech";
    this.speechStage.view.visible = isSpeech;
    this.caseBoardStage.view.visible = !isSpeech;

    if (isSpeech) {
      this.speechStage.update(frame);
      return;
    }

    this.caseBoardStage.update(frame);
  }

  renderEmpty(): void {
    this.speechStage.view.visible = false;
    this.caseBoardStage.view.visible = true;
    this.caseBoardStage.renderEmpty();
  }
}
```

Do not add `CaseBoardStage` yet; TypeScript should fail after this step.

- [ ] **Step 4: Run typecheck to verify the expected missing class failure**

Run:

```bash
pnpm typecheck
```

Expected: FAIL with `Cannot find name 'CaseBoardStage'`.

Do not commit this failing intermediate state.

## Task 3: Implement Case Board Pixi Stage

**Files:**
- Modify: `src/components/preview/pixi/pixi-preview-renderer.ts`
- Uses: `src/components/preview/pixi/case-board-content.ts`

- [ ] **Step 1: Import the Case Board helper**

Add this import near the other local imports in `src/components/preview/pixi/pixi-preview-renderer.ts`:

```ts
import {
  caseBoardContentForFrame,
  type CaseBoardRow,
} from "./case-board-content";
```

- [ ] **Step 2: Add the `CaseBoardStage` class**

Insert this class between `CenterStage` and `SpeechStage`:

```ts
class CaseBoardStage {
  readonly view = new LayoutContainer();
  private readonly panel = new Graphics();
  private readonly divider = new Graphics();
  private readonly kindLabel: Text;
  private readonly title: Text;
  private readonly body: Text;
  private readonly rows = [
    new CaseBoardRowView(),
    new CaseBoardRowView(),
    new CaseBoardRowView(),
    new CaseBoardRowView(),
  ];

  constructor(private readonly theme: PixiPreviewTheme) {
    const width = mainStageWidth(theme);
    const height = mainStageHeight(theme);
    this.kindLabel = text("", {
      ...theme.typography.meta,
      fill: theme.colors.brass,
      fontSize: 20,
      fontWeight: "900",
    });
    this.title = text("", {
      ...theme.typography.title,
      fill: theme.colors.text,
      fontSize: 48,
      lineHeight: 58,
      wordWrap: true,
      wordWrapWidth: 500,
    });
    this.body = text("", {
      ...theme.typography.body,
      fill: theme.colors.text,
      fontSize: 30,
      lineHeight: 40,
      wordWrap: true,
      wordWrapWidth: 500,
    });
    this.view.layout = {
      position: "absolute",
      left: 0,
      top: 0,
      width,
      height,
    };
    this.panel.layout = {
      position: "absolute",
      left: 0,
      top: 0,
      width: "100%",
      height: "100%",
    };
    this.kindLabel.position.set(58, 78);
    this.title.position.set(58, 122);
    this.body.position.set(58, 238);
    this.rows.forEach((row, index) => {
      row.view.position.set(570, 96 + index * 112);
    });
    this.view.addChild(
      this.panel,
      this.divider,
      this.kindLabel,
      this.title,
      this.body,
      ...this.rows.map((row) => row.view),
    );
  }

  update(frame: ShotFrame): void {
    const content = caseBoardContentForFrame(frame);
    this.drawPanel();
    this.kindLabel.text = content.kindLabel;
    this.title.text = content.title;
    this.body.text = content.body;
    fitText(this.title, 500, 48, 30);
    content.rows.slice(0, this.rows.length).forEach((row, index) => {
      this.rows[index]?.update(row, this.theme);
    });
    for (let index = content.rows.length; index < this.rows.length; index += 1) {
      this.rows[index]?.clear();
    }
  }

  renderEmpty(): void {
    this.drawPanel();
    this.kindLabel.text = "PREVIEW";
    this.title.text = "等待审讯记录";
    this.body.text = "尚无可播放场景。";
    this.rows[0]?.update(
      { tone: "neutral", label: "FILE", text: "PUBLIC RECORD" },
      this.theme,
    );
    for (let index = 1; index < this.rows.length; index += 1) {
      this.rows[index]?.clear();
    }
  }

  private drawPanel(): void {
    const width = mainStageWidth(this.theme);
    const height = mainStageHeight(this.theme);
    this.panel
      .clear()
      .roundRect(0, 0, width, height, 28)
      .fill({ color: this.theme.colors.black, alpha: 0.26 })
      .roundRect(0, 0, width, height, 28)
      .stroke({ color: this.theme.colors.brass, alpha: 0.20, width: 1 })
      .rect(36, height - 96, width - 72, 1)
      .fill({ color: this.theme.colors.accent, alpha: 0.18 });
    this.divider
      .clear()
      .rect(532, 76, 1, height - 152)
      .fill({ color: this.theme.colors.brass, alpha: 0.18 });
  }
}
```

- [ ] **Step 3: Add `CaseBoardRowView`**

Insert this class immediately after `CaseBoardStage`:

```ts
class CaseBoardRowView {
  readonly view = new Container();
  private readonly panel = new Graphics();
  private readonly label = text("", {
    fill: 0xc2ad72,
    fontFamily: PIXI_PREVIEW_FONT_FAMILY,
    fontSize: 30,
    fontWeight: "900",
  });
  private readonly textValue = text("", {
    fill: 0xece7da,
    fontFamily: PIXI_PREVIEW_FONT_FAMILY,
    fontSize: 28,
    fontWeight: "900",
    lineHeight: 34,
    wordWrap: true,
    wordWrapWidth: 280,
  });
  private readonly meta = text("", {
    fill: 0x9a9488,
    fontFamily: PIXI_PREVIEW_FONT_FAMILY,
    fontSize: 19,
    fontWeight: "700",
    lineHeight: 24,
  });

  constructor() {
    this.label.anchor.set(0.5, 0.5);
    this.label.position.set(48, 48);
    this.textValue.position.set(96, 23);
    this.meta.position.set(96, 58);
    this.view.addChild(this.panel, this.label, this.textValue, this.meta);
  }

  update(row: CaseBoardRow, theme: PixiPreviewTheme): void {
    this.view.visible = true;
    const toneColor = colorForCaseBoardTone(row.tone, theme);
    this.panel
      .clear()
      .roundRect(0, 0, 330, 92, 10)
      .fill({ color: theme.colors.black, alpha: 0.36 })
      .roundRect(0, 0, 330, 92, 10)
      .stroke({ color: toneColor, alpha: 0.28, width: 1 })
      .rect(0, 0, 4, 92)
      .fill({ color: toneColor, alpha: 0.78 });
    this.label.text = row.label;
    this.label.style = { ...this.label.style, fill: toneColor };
    this.textValue.text = row.text;
    this.textValue.style = {
      ...this.textValue.style,
      fill: row.tone === "dead-player" ? theme.colors.muted : theme.colors.text,
    };
    this.meta.text = row.meta ?? "";
    this.meta.visible = Boolean(row.meta);
  }

  clear(): void {
    this.view.visible = false;
    this.panel.clear();
    this.label.text = "";
    this.textValue.text = "";
    this.meta.text = "";
  }
}
```

- [ ] **Step 4: Add row tone color helper**

Add this helper near other renderer helper functions:

```ts
function colorForCaseBoardTone(
  tone: CaseBoardRow["tone"],
  theme: PixiPreviewTheme,
): number {
  switch (tone) {
    case "dead-player":
      return theme.colors.danger;
    case "alive-player":
      return theme.colors.accent;
    case "detail":
      return theme.colors.brass;
    case "neutral":
      return theme.colors.muted;
  }
}
```

- [ ] **Step 5: Run typecheck**

Run:

```bash
pnpm typecheck
```

Expected: PASS.

- [ ] **Step 6: Run preview tests**

Run:

```bash
pnpm vitest run src/components/preview
```

Expected: PASS.

- [ ] **Step 7: Commit center-stage implementation**

Run:

```bash
git add src/components/preview/pixi/pixi-preview-renderer.ts
git commit -m "feat: add pixi case board stage"
```

## Task 4: Add Renderer Decision Tests

**Files:**
- Modify: `src/components/preview/pixi/case-board-content.test.ts`

- [ ] **Step 1: Add tests that lock speech vs non-speech routing input assumptions**

Append these tests inside the existing `describe("caseBoardContentForFrame", ...)` block:

```ts
  it("labels vote scenes for the case board", () => {
    const content = caseBoardContentForFrame(frame({
      scene: scene({
        kind: "vote",
        title: "放逐投票",
        text: "进入本日放逐投票。",
      }),
    }));

    expect(content.kindLabel).toBe("VOTE");
  });

  it("can still label speech but the renderer should not route speech to case board", () => {
    const content = caseBoardContentForFrame(frame({
      scene: scene({
        kind: "speech",
        title: "4 号 林夏发言",
        text: "我是4号林夏。",
      }),
    }));

    expect(content.kindLabel).toBe("SPEECH");
  });
```

These tests document the helper behavior without asserting that speech uses Case Board. The routing rule lives in `CenterStage.update`: `frame.scene.kind === "speech"` selects `SpeechStage`; all other kinds select `CaseBoardStage`.

- [ ] **Step 2: Run the helper tests**

Run:

```bash
pnpm vitest run src/components/preview/pixi/case-board-content.test.ts
```

Expected: PASS.

- [ ] **Step 3: Commit tests**

Run:

```bash
git add src/components/preview/pixi/case-board-content.test.ts
git commit -m "test: cover case board scene labels"
```

## Task 5: Manual Visual Check and Final Verification

**Files:**
- No new files.

- [ ] **Step 1: Run all preview tests**

Run:

```bash
pnpm vitest run src/components/preview
```

Expected: PASS.

- [ ] **Step 2: Run typecheck**

Run:

```bash
pnpm typecheck
```

Expected: PASS.

- [ ] **Step 3: Run full test suite**

Run:

```bash
pnpm test
```

Expected: PASS.

- [ ] **Step 4: Start or reuse the dev server**

Run:

```bash
pnpm dev
```

Expected: Next.js prints a local URL. If another dev server is already running, use the URL shown by Next.js instead of starting a second server.

- [ ] **Step 5: Manually inspect a preview page**

Open a game preview URL such as:

```text
http://localhost:9090/games/b3758264-d9d8-43e2-b494-dbd3cda7f010/preview
```

Expected:

- Phase scenes show a full center Case Board with no floating detached rectangle.
- Announcement/resolution scenes show the Case Board and preserve the subtitle band.
- Speech scenes still show the player-focused center stage.
- Seat cards, top header, and playback controls remain visually unchanged.

- [ ] **Step 6: Commit final verification note if manual fixes were needed**

If Task 5 reveals a needed visual adjustment, make the smallest renderer-only fix, run:

```bash
pnpm vitest run src/components/preview
pnpm typecheck
```

Then commit:

```bash
git add src/components/preview/pixi/pixi-preview-renderer.ts
git commit -m "fix: polish case board stage layout"
```

If no manual fixes are needed, do not create an empty commit.

## Self-Review

- Spec coverage: the plan splits speech and non-speech center rendering, adds a Case Board for phase/announcement/vote/resolution, preserves subtitles, avoids core playback changes, and includes tests for content rules.
- Placeholder scan: no task contains unresolved placeholder instructions.
- Type consistency: `CaseBoardRow`, `CaseBoardContent`, `caseBoardContentForFrame`, `CenterStage`, `SpeechStage`, and `CaseBoardStage` names are defined before use in later tasks.
