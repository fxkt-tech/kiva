# Show Theme Pack Design

## Problem

The current preview can record the correct canvas, but the visual language is still a tool screen. A publishable werewolf video needs a program identity: stage, scene direction, player packaging, tension, and theme-specific event metaphors.

Different games may use different show themes. The renderer must not hard-code one "dark round table" look. The same gameplay events should be reusable across mansion murder, wilderness survival, cyber trial, or other future show formats.

## Goal

Build a player-stage architecture for canvas playback:

- Keep `PlaybackItem[]` as gameplay facts.
- Make the 6 players the persistent visual subjects.
- Use a left-3 / right-3 stage layout so the center stays available for show moments.
- Add a `ShowTheme` layer that skins the stage and effects without replacing the stage structure.
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

Game events are facts. The player stage is structure. Theme packs are direction.

For example, a `vote` scene always says who voted for whom. The mansion theme may render it as a red string on a detective board. The wilderness theme may render it as an exile mark on a camp map. The underlying scene does not change.

The first visual priority is not the theme background. It is the player ensemble:

- every player has a visible card
- each card has avatar, seat number, name, and status
- dead players remain visible but become gray and stamped
- the current speaker is highlighted
- scene overlays happen around the player stage, not instead of it

## Architecture

```text
GameEvent[]
  -> compilePublicPlayback()
  -> PlaybackItem[]
  -> PlayerStageLayout
  -> PlayerCardLayer
  -> CenterStageLayer
  -> SubtitleLayer
  -> ThemeSkin
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

### PlayerStageLayout

The first implementation supports the 6-player horizontal program layout:

```text
[ P1 ]                         [ P4 ]
[ P2 ]      Center Stage       [ P5 ]
[ P3 ]                         [ P6 ]

             Subtitle Bar
```

This layout is fixed for 6 players:

- left column: seats 1, 2, 3
- right column: seats 4, 5, 6
- center region: phase title, voting result table, death announcement, game end reveal
- bottom region: speech subtitles and narrator-style announcements

The layout engine returns geometry only:

```ts
type StageLayout = {
  playerSlots: readonly PlayerSlot[];
  center: Rect;
  subtitle: Rect;
};
```

It does not draw anything and does not know theme colors.

### PlayerCardLayer

Every scene renders player cards. Cards are the persistent visual anchor of the program.

Card contents:

- avatar placeholder or image
- seat number
- player name
- alive/dead state
- current speaker highlight
- target/result highlight when relevant
- future extension slot for role/player special effects

Dead players:

- card desaturated/gray
- avatar dimmed
- `DEAD` or theme-specific stamp
- still kept in their seat

### CenterStageLayer

The center stage changes by scene kind:

- phase: chapter/title card
- announcement: public event announcement
- vote: vote table and result summary
- resolution: exile/PK/death/game result
- end: winner reveal and role reveal

Vote scenes do not use vote lines. They show the whole vote result in the center:

```text
放逐投票

1号 秦川 -> 3号 周知
2号 林夏 -> 3号 周知
4号 许棠 -> 5号 陈墨

本轮结果
3号 周知 2票
5号 陈墨 1票
弃票 1票
```

### SubtitleLayer

Speech and last words use a bottom subtitle bar:

- speaker name and seat label
- speech text
- legible high-contrast typography
- no scene text body in the center during normal speech

This is the primary way spoken content is presented.

### EffectLayer

First version effects:

- current speaker glow
- dead player gray/stamp
- target/result highlight
- scene fade-in

Future reserved effects:

- player-specific speaking effects
- role-specific overlays
- avatar/illustration reveal
- theme-specific special animation

These are designed as extension points, not implemented in this phase.

### ShowThemePack / ThemeSkin

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

`render` contains skin renderers for the shared stage:

- `renderBackground`
- `renderPlayerCard`
- `renderCenterStage`
- `renderSubtitle`
- `renderEffect`

### ThemeDirector

The director coordinates the layers for every frame:

```ts
renderThemeFrame(ctx, {
  theme,
  scene,
  items,
  layout,
  timeMs,
  sceneTimeMs,
  progress,
});
```

The director computes animation progress and calls layers in order:

1. background
2. player cards
3. center stage
4. subtitle
5. effects

This prevents themes from replacing the entire program structure with a one-off composition.

### CanvasRenderer

Shared low-level drawing helpers:

- `drawTextBlock`
- `drawPanel`
- `drawGlowText`
- `drawPlayerCardShell`
- `drawAvatar`
- `drawStatusStamp`
- `drawVoteResultTable`
- `drawNoise`
- `drawVignette`

Theme renderers call these helpers, but player positions and layer order come from the stage system.

## First Theme: Mansion Murder

Visual identity:

- A rainy mansion / detective board atmosphere.
- Deep black, ink gray, faded paper, blood red, cold blue highlights.
- Player cards look like suspect files.
- Voting is represented by a center evidence/result board, not lines.
- Death is represented by a file blackout and red case mark.
- Game end is a case-closed reveal.

Scene design:

### Title

- Large title: game title or "Mansion Murder Werewolf".
- Subtitle: game id or episode label.
- Six suspect files appear in left/right stage positions.

### Night / Phase

- Dark mansion silhouette.
- Low light through windows.
- Phase title appears like a chapter card.

### Speech

- Current speaker file glows in its fixed slot.
- Other players remain visible in their fixed slots.
- Speech text appears in the bottom subtitle bar.

### Vote

- Player cards remain in left/right positions.
- Center stage shows the full vote table.
- Result summary highlights top vote target or tie.

### Announcement / Death

- Affected player file darkens in its fixed slot.
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

This theme should reuse the same player-stage interfaces. If adding it requires rewriting playback data, changing player layout contracts, or hard-coding theme branches in `PlaybackStage`, the architecture failed.

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
- `renderThemeFrame` calls background, player cards, center stage, subtitle, and effects in order
- 6-player stage layout returns left seats 1-3 and right seats 4-6

Component tests:

- preview renders a 1920x1080 canvas
- controls do not render game scene text in DOM
- recording controls remain outside canvas

Renderer tests:

- smoke test render functions with a fake canvas context
- verify each scene kind calls expected helper operations
- verify speech renders subtitle text through subtitle layer
- verify vote renders center result table instead of vote lines

Manual verification:

- open preview
- confirm mansion theme has a distinct program look
- click `Record`
- confirm playback starts at 0
- confirm recording stops at end
- confirm downloaded WebM contains only canvas content

## Implementation Order

1. Add 6-player stage layout.
2. Extract shared canvas drawing helpers from `playback-stage.tsx`.
3. Add `ShowThemePack` types and registry around shared stage layers.
4. Implement `mansion_murder` skin for player cards, center stage, subtitle, and effects.
5. Wire `PlaybackStage` to render through player stage + theme skin.
6. Add `showThemeId` fallback path without exposing UI.
7. Run tests and manual preview verification.

## Acceptance Criteria

- The preview no longer looks like a plain debug canvas.
- The visible playback canvas has a clear mansion murder program identity.
- All six players have persistent left/right cards.
- Speech uses a bottom subtitle bar and highlighted speaker card.
- Dead players remain visible and gray/stamped.
- Vote scenes use a central vote result board, not connecting lines.
- Recording still captures only canvas content.
- The code has a theme boundary that can support `wilderness_survival` without rewriting `PlaybackStage`.
