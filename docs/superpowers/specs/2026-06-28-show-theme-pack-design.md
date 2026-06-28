# Show Theme Pack Design

## Problem

The current preview can record the correct canvas, but the visual language is still a tool screen. A publishable werewolf video needs a program identity: stage, scene direction, player packaging, tension, and theme-specific event metaphors.

Different games may use different show themes. The renderer must not hard-code one "dark round table" look. The same gameplay events should be reusable across mansion murder, wilderness survival, cyber trial, or other future show formats.

## Goal

Build a theme-pack architecture for canvas playback:

- Keep `PlaybackItem[]` as gameplay facts.
- Add a `ShowTheme` layer that controls visual direction only.
- Support a first high-quality theme: `mansion_murder`.
- Add a second contrasting theme later: `wilderness_survival`, to prove the architecture is not one-off.
- Preserve the current one-click canvas recording flow.

## Non-Goals

- No theme editor UI in this phase.
- No MP4 export in this phase.
- No audio, voiceover, or sound effects in this phase.
- No generative image dependency in the first implementation.
- No change to game rules, event log semantics, or LLM logic.

## Core Principle

Game events are facts. Theme packs are direction.

For example, a `vote` scene always says who voted for whom. The mansion theme may render it as a red string on a detective board. The wilderness theme may render it as an exile mark on a camp map. The underlying scene does not change.

## Architecture

```text
GameEvent[]
  -> compilePublicPlayback()
  -> PlaybackItem[]
  -> ShowThemePack
  -> ThemeDirector
  -> CanvasRenderer
  -> canvas.captureStream()
```

### PlaybackItem

`PlaybackItem` remains the stable public playback fact shape:

- `kind`
- `phase`
- `title`
- `text`
- `details`
- `durationMs`
- `startsAtMs`
- `players`

No theme fields should be added directly to `PlaybackItem`.

### ShowThemePack

Each theme pack owns the full program packaging:

```ts
type ShowThemePack = {
  id: ShowThemeId;
  name: string;
  tokens: ShowThemeTokens;
  render: ShowThemeRenderers;
};
```

`tokens` contains visual constants:

- background colors
- accent colors
- danger colors
- good/wolf faction colors
- typography sizes
- panel geometry
- glow/shadow intensity
- grain/noise settings

`render` contains scene renderers:

- `renderTitle`
- `renderPhase`
- `renderAnnouncement`
- `renderSpeech`
- `renderVote`
- `renderResolution`
- `renderEnd`

### ThemeDirector

The director chooses the renderer for a scene:

```ts
renderThemeFrame(ctx, {
  theme,
  scene,
  items,
  timeMs,
  sceneTimeMs,
  progress,
});
```

The director also computes animation progress:

- scene enter progress
- scene local time
- vote reveal progress
- speaker spotlight progress
- end reveal progress

This prevents individual renderers from re-implementing time math.

### CanvasRenderer

Shared low-level drawing helpers:

- `drawTextBlock`
- `drawPanel`
- `drawGlowText`
- `drawPlayerBadge`
- `drawSeatGrid`
- `drawVoteLine`
- `drawNoise`
- `drawVignette`

Theme renderers call these helpers, but decide layout and metaphor.

## First Theme: Mansion Murder

Visual identity:

- A rainy mansion / detective board atmosphere.
- Deep black, ink gray, faded paper, blood red, cold blue highlights.
- Player cards look like suspect files.
- Voting is represented by evidence strings and stamped cards.
- Death is represented by a file blackout and red case mark.
- Game end is a case-closed reveal.

Scene design:

### Title

- Large title: game title or "Mansion Murder Werewolf".
- Subtitle: game id or episode label.
- Six suspect cards fan in.

### Night / Phase

- Dark mansion silhouette.
- Low light through windows.
- Phase title appears like a chapter card.

### Speech

- Current speaker appears as the active suspect file.
- Other players are smaller files on the side.
- Speech text appears as transcript paper.

### Vote

- Player cards laid out as suspect files.
- Vote lines appear as red evidence string.
- Target card receives stacked vote markers.

### Announcement / Death

- Affected player file darkens.
- Red "DEAD" / out marker.
- Announcement text appears as a case bulletin.

### Resolution / End

- Case board composition.
- Winner revealed with large faction treatment.
- Revealed roles shown as file tags.

## Second Theme: Wilderness Survival

Purpose: validate that the system supports a totally different stage metaphor.

Visual identity:

- Camp map, firelight, terrain contours, emergency marker colors.
- Players are expedition members.
- Votes are exile trail marks.
- Death/out state is a darkened camp marker.

This theme should reuse the same renderer interfaces. If adding it requires rewriting playback data or hard-coding theme branches in `PlaybackStage`, the architecture failed.

## Data Model

First implementation can use a default theme constant in preview:

```ts
const DEFAULT_SHOW_THEME_ID = "mansion_murder";
```

Next data step:

```ts
game.showThemeId?: ShowThemeId;
```

If missing, fallback to `mansion_murder`.

Theme config can be added later:

```ts
game.showThemeConfig?: {
  titleOverride?: string;
  subtitle?: string;
  variant?: string;
};
```

No theme editor UI in this phase.

## UI Behavior

Preview page remains simple:

- canvas
- timeline scrubber
- `Record`
- `Play`
- `Reset`
- download link after recording

No separate recording page.
No clean preview link.
No screen capture flow.

## Testing

Core tests:

- theme registry returns `mansion_murder`
- unknown theme falls back to default
- `renderThemeFrame` chooses the renderer matching scene kind

Component tests:

- preview renders a 1920x1080 canvas
- controls do not render game scene text in DOM
- recording controls remain outside canvas

Renderer tests:

- smoke test render functions with a fake canvas context
- verify each scene kind calls expected helper operations

Manual verification:

- open preview
- confirm mansion theme has a distinct program look
- click `Record`
- confirm playback starts at 0
- confirm recording stops at end
- confirm downloaded WebM contains only canvas content

## Implementation Order

1. Extract shared canvas drawing helpers from `playback-stage.tsx`.
2. Add `ShowThemePack` types and registry.
3. Move current plain renderer into a `basic` internal theme only if needed as a fallback.
4. Implement `mansion_murder` theme.
5. Wire `PlaybackStage` to render through theme registry.
6. Add `showThemeId` fallback path without exposing UI.
7. Run tests and manual preview verification.

## Acceptance Criteria

- The preview no longer looks like a plain debug canvas.
- The visible playback canvas has a clear mansion murder program identity.
- Scene types have meaningfully different layouts.
- Recording still captures only canvas content.
- The code has a theme boundary that can support `wilderness_survival` without rewriting `PlaybackStage`.
