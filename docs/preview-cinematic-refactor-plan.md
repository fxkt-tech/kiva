# Preview Cinematic Refactor Plan

## Goal

Refactor the preview renderer from a single `mansion_murder` canvas theme into a reusable werewolf playback shot engine with skin-specific visual rendering.

The target visual direction is cinematic werewolf replay:

- Canvas-only output, so exported recordings include the complete style.
- A stable six-seat track remains visible, but low-priority players are compact.
- The current event owns the main shot.
- Speech scenes use a central speaker portrait and segmented subtitle window.
- Vote, resolution, phase, and announcement scenes use specialized shot templates.
- Theme skins replace art direction and drawing style, not the narrative shot structure.
- Animation stays lightweight: scene progress, fade, slide, pulse, zoom, shake, staged reveal.

## Non-Goals

- Do not move scene visuals into DOM/CSS overlays, because recording uses `canvas.captureStream()`.
- Do not build a generic renderer for arbitrary games.
- Do not build a JSON or no-code theme DSL in this pass.
- Do not require finished portrait assets before the refactor works.
- Do not rewrite playback compilation unless a missing render signal is discovered.

## Current Problems

- `src/components/preview/themes/mansion-murder.ts` owns layout decisions, theme styling, scene selection, and low-level drawing details in one file.
- `ShowThemeRenderers` is organized as render passes, but the theme still decides too much about shot structure.
- Player cards are full-size and always visually dominant, so the current event does not read as the main focus.
- The subtitle bar renders long text as a large information box instead of a cinematic subtitle layer.
- `canvas-renderer.ts` has only primitive helpers, making richer visual work repetitive.
- `StageLayout` is hard-coded around large side cards instead of compact tracks plus central shot regions.

## Target Architecture

### 1. Playback Shot Engine

Add a code-level engine under `src/components/preview/shot-engine/`.

Suggested files:

- `types.ts`
- `director.ts`
- `shot-layout.ts`
- `animation.ts`
- `subtitles.ts`
- `players.ts`
- `render-frame.ts`

Core concepts:

```ts
type ShotKind =
  | "phase"
  | "announcement"
  | "speech"
  | "vote"
  | "resolution";

type ShotFrame = {
  readonly scene: PlaybackItem;
  readonly items: readonly PlaybackItem[];
  readonly layout: ShotLayout;
  readonly clock: ShotClock;
  readonly players: readonly RenderablePlayer[];
  readonly activePlayer: RenderablePlayer | null;
  readonly highlightedPlayers: readonly RenderablePlayer[];
  readonly subtitle: SubtitleCue | null;
};

type ShotClock = {
  readonly absoluteMs: number;
  readonly sceneMs: number;
  readonly durationMs: number;
  readonly progress: number;
  readonly enterProgress: number;
  readonly exitProgress: number;
};
```

The engine chooses the shot and prepares derived data. It should not contain mansion-specific colors or drawing style.

### 2. Shot Layout

Replace the current large-card `StageLayout` with a cinematic `ShotLayout`.

Required regions:

- `safe`: full video safe area.
- `topBar`: title, phase, scene index.
- `seatTrackLeft`: compact seats 1-3.
- `seatTrackRight`: compact seats 4-6.
- `mainStage`: central event area.
- `portrait`: current speaker or target portrait region.
- `eventPanel`: vote/result/announcement content region.
- `subtitle`: bottom subtitle safe area.

Keep 1920x1080 as the canonical design coordinate system.

### 3. Theme Skin Interface

Replace `ShowThemePack.render` with a skin interface that draws shared shot parts.

Suggested shape:

```ts
type PreviewThemeSkin = {
  readonly id: ShowThemeId;
  readonly name: string;
  readonly tokens: ThemeTokens;
  readonly assets: ThemeAssetSlots;
  readonly draw: {
    readonly background: (input: SkinRenderInput) => void;
    readonly topBar: (input: SkinRenderInput) => void;
    readonly seat: (input: SkinSeatInput) => void;
    readonly speechShot: (input: SkinRenderInput) => void;
    readonly voteShot: (input: SkinRenderInput) => void;
    readonly phaseShot: (input: SkinRenderInput) => void;
    readonly announcementShot: (input: SkinRenderInput) => void;
    readonly resolutionShot: (input: SkinRenderInput) => void;
    readonly subtitle: (input: SkinSubtitleInput) => void;
    readonly effects: (input: SkinRenderInput) => void;
  };
};
```

The shared renderer calls these in a fixed order:

1. Background
2. Top bar
3. Compact seat tracks
4. Shot body by `scene.kind`
5. Subtitle
6. Effects

This keeps themes replaceable while preserving the cinematic shot grammar.

### 4. Canvas Drawing Helpers

Expand `canvas-renderer.ts` or split it into focused modules.

Needed helpers:

- `drawRoundedRect`
- `drawGradientRect`
- `drawImageCover`
- `drawImageCoverRounded`
- `drawImageCoverCircle`
- `drawSoftShadow`
- `drawTextBlock` with max lines and ellipsis support
- `drawFittedText`
- `drawPill`
- `drawRuleLine`
- `drawNoiseOverlay` if cheap enough
- `withAlpha`
- `withClip`
- `withShadow`

Keep helpers low-level and testable. Do not put werewolf concepts in these helpers.

### 5. Subtitle Cues

Add `subtitles.ts`.

Responsibilities:

- Split Chinese-heavy text by punctuation first, then length.
- Group into windows of 2-3 lines.
- Select the active window from `scene.progress`.
- Preserve speaker label separately from subtitle text.

First-pass behavior:

- Speech scenes: subtitle cue windows progress across the scene duration.
- Non-speech scenes: short title/text can remain static.
- No audio timestamp dependency.

### 6. Animation Helpers

Add `animation.ts`.

Keep the API simple:

- `clamp01`
- `easeOutCubic`
- `easeInOutCubic`
- `fadeIn(progress)`
- `slideIn(progress, distance)`
- `pulse(progress, amount)`
- `zoom(progress, from, to)`
- `shake(progress, amplitude)`
- `stagedReveal(progress, index, count)`

Do not add a timeline/keyframe DSL in this pass.

## Mansion Murder Skin Redesign

`mansion_murder` becomes the first skin implemented against the new shot engine.

Visual rules:

- Background stays mansion/day-night themed, but dimmed enough to support readable foregrounds.
- Top bar becomes quieter and more cinematic.
- Seat tracks become compact dossiers, not large cards.
- Current speaker becomes a central portrait shot:
  - Use `avatar` as enlarged portrait when available.
  - Use a high-quality abstract silhouette/fallback when missing.
  - Show seat number, player name, status, and visible role in a restrained overlay.
- Death state should feel decisive:
  - Muted seat.
  - Red status mark.
  - Optional one-shot impact/shake in death/resolution scenes.
- Good/wolf role colors should be accents, not large color blocks.
- Subtitle should be a lower cinematic band with 2-3 lines, not a large transcript box.

## Implementation Sequence

### Step 1: Lock Current Behavior With Tests

Files:

- `src/components/preview/show-theme.test.ts`
- `src/components/preview/stage-layout.test.ts`
- `src/components/preview/themes/mansion-murder.test.ts`

Add or adjust tests so the current render frame contract is explicit before moving it.

Verify:

```sh
pnpm vitest run src/components/preview/show-theme.test.ts src/components/preview/themes/mansion-murder.test.ts
```

### Step 2: Add Shot Layout

Create:

- `src/components/preview/shot-engine/shot-layout.ts`
- `src/components/preview/shot-engine/shot-layout.test.ts`

Implement 1920x1080 cinematic regions and six compact seat slots.

Acceptance:

- Seats 1-3 map left, 4-6 map right.
- Main stage and subtitle do not overlap.
- All regions stay inside canvas bounds.

### Step 3: Add Shot Clock, Player Derivation, and Director

Create:

- `src/components/preview/shot-engine/types.ts`
- `src/components/preview/shot-engine/director.ts`
- `src/components/preview/shot-engine/players.ts`
- related tests

Acceptance:

- `speech` scenes expose exactly one `activePlayer` when a highlighted speaker exists.
- `announcement`, `resolution`, and `vote` scenes expose highlighted targets.
- `progress`, `enterProgress`, and `exitProgress` are clamped.

### Step 4: Add Subtitle Cue System

Create:

- `src/components/preview/shot-engine/subtitles.ts`
- `src/components/preview/shot-engine/subtitles.test.ts`

Acceptance:

- Long Chinese text splits into readable windows.
- Active window advances with scene progress.
- Short text remains one cue.
- Empty text returns `null`.

### Step 5: Add Animation Helpers

Create:

- `src/components/preview/shot-engine/animation.ts`
- `src/components/preview/shot-engine/animation.test.ts`

Acceptance:

- Helpers clamp safely.
- Easing outputs are deterministic.
- Staged reveal can be tested without rendering.

### Step 6: Expand Canvas Primitives

Modify:

- `src/components/preview/canvas-renderer.ts`
- `src/components/preview/canvas-renderer.test.ts`

Acceptance:

- Existing helper tests keep passing.
- New helpers have tests for expected canvas operations.
- No theme-specific types enter this file.

### Step 7: Introduce Skin-Based Frame Renderer

Modify or replace:

- `src/components/preview/show-theme.ts`

Create:

- `src/components/preview/shot-engine/render-frame.ts`

Acceptance:

- `renderThemeFrame` still exports the public API used by `playback-stage.tsx`.
- Internally, it builds a `ShotFrame` and calls skin draw methods in fixed order.
- Existing `PlaybackStage` does not need to know about shot engine details.

### Step 8: Port Mansion Murder To Skin Interface

Modify:

- `src/components/preview/themes/mansion-murder.ts`
- `src/components/preview/themes/mansion-murder.test.ts`

Acceptance:

- Theme no longer owns global shot structure.
- It only draws mansion-specific background, seats, shot bodies, subtitle, and effects.
- Tests assert that speech scenes draw speaker portrait/fallback and subtitle cue.
- Tests assert that dead players render compact muted seats.

### Step 9: Visual Smoke Test

Run the app and inspect real canvas output.

Suggested checks:

- Open a preview page with several speech scenes.
- Check desktop 16:9 canvas.
- Confirm no blank canvas.
- Confirm text does not overlap.
- Confirm subtitles show current windows, not full transcript.
- Confirm recording still captures the full scene.

Verification commands:

```sh
pnpm vitest run src/components/preview
pnpm lint
pnpm typecheck
```

Use the repo's actual package scripts if these names differ.

## Suggested Commit Slices

1. `test: lock preview render contracts`
2. `refactor: add preview shot layout engine`
3. `feat: add preview subtitle cues and animation helpers`
4. `refactor: render preview frames through theme skins`
5. `refactor: port mansion murder to cinematic skin`
6. `test: cover cinematic preview rendering`

## Risks

- Canvas text measurement differs between environments, so tests should assert behavior and draw calls, not exact pixel output.
- Long Chinese speeches can still overflow if cue sizing is too optimistic; subtitle helpers need max-line behavior.
- Avatar images may be missing or low-resolution; fallback portrait must be designed as a first-class path.
- Too much abstraction too early can slow the skin work; keep the engine focused on werewolf replay shots.
- Existing screenshots may look different even if behavior is correct; visual smoke testing is required.

## Definition of Done

- Preview remains canvas-only.
- Existing playback controls and recording still work.
- `mansion_murder` renders through the new skin interface.
- Speech scenes show compact seat tracks, central speaker portrait/fallback, and segmented subtitles.
- Vote/resolution/announcement scenes use the shared shot structure.
- New layout, subtitle, animation, and director behavior is covered by tests.
- Real browser preview has been smoke-tested for readability and non-overlap.
