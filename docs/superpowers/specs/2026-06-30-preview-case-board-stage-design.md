# Preview Case Board Stage Design

## Goal

Redesign the Pixi preview center stage so non-speech scenes no longer reuse the speech/portrait stage. The current shared `MainShot` creates visual artifacts such as a floating dark rectangle during phase transitions because scenes without an active player are forced through a layout built around player focus.

## Scope

This design covers the Pixi preview stage content inside the center column.

In scope:

- Split center-stage rendering by `PlaybackSceneKind`.
- Keep `speech` scenes on the existing player-focused stage.
- Render `phase`, `announcement`, `vote`, and `resolution` scenes through a new unified Case Board stage.
- Keep the existing subtitle band as the spoken/broadcast caption layer.
- Remove the current floating `lensOverlay` rectangle behavior for non-speech scenes.
- Add tests around the Case Board content selection rules.

Out of scope:

- Reworking left/right seat cards.
- Reworking top header.
- Reworking the playback controls.
- Adding a bespoke vote grid or game-end reveal screen in this pass.
- Changing core event types or playback compilation.

## Current Problem

`MainShot` currently decides whether to show a player portrait by checking:

- `frame.activePlayer`
- `frame.highlightedPlayers[0]`

When no player exists, it falls back to a title-slate mode. That fallback still uses overlay geometry intended for a focused shot, so phase scenes such as `第 1 夜开始` can show detached blocks that do not align with the visible title or subtitle.

The deeper issue is not the alpha or position of one rectangle. It is that different scene kinds have different narrative jobs:

- `speech`: one player owns the center stage.
- `phase`: transition and pacing.
- `announcement`: public information.
- `vote`: decision record.
- `resolution`: outcome and consequences.

The renderer should reflect those jobs directly.

## Proposed Structure

Replace the single-purpose `MainShot` behavior with a center-stage dispatcher:

- `SpeechStage`: player-focused stage for `scene.kind === "speech"`.
- `CaseBoardStage`: evidence-board stage for `phase`, `announcement`, `vote`, and `resolution`.

This can be implemented either by renaming `MainShot` into a dispatcher that owns both internal stage classes, or by replacing it with a new `CenterStage` class. The preferred implementation is `CenterStage`, because the public responsibility becomes clear: choose the right visual stage for the current `ShotFrame`.

## Case Board Layout

The Case Board fills the existing center-stage rectangle. It should not add a small floating card inside the stage.

Layout:

- Full center-stage dark translucent panel with subtle border.
- Left primary column:
  - Small kind label: `PHASE`, `ANNOUNCEMENT`, `VOTE`, or `RESOLUTION`.
  - Large title from `scene.title`.
  - Body text from `scene.text`.
  - A restrained divider or accent line.
- Right context column:
  - Prefer `scene.details` when present.
  - Else show highlighted players.
  - Else show a short neutral status row such as `PUBLIC RECORD`.

Highlighted player rows should use the existing player data:

- Two-digit seat number.
- Player name.
- Role name when useful.
- Muted/dead styling when `status === "dead"`.

Details rows should render the strings directly, with wrapping or fitting so they do not overflow the panel.

## Subtitle Band

The subtitle band remains in place for all scene kinds.

Its role is narrower than the Case Board:

- Case Board: visual structure and summary.
- Subtitle band: spoken/broadcast caption text.

This preserves existing subtitle windowing and avoids moving narration logic into the stage component.

## Testing

Add or update preview tests to cover:

- Non-speech scene kinds route to the Case Board stage rather than the speech portrait stage.
- Case Board chooses `details` before highlighted players.
- Case Board falls back to a neutral status row when no details or highlighted players exist.
- Speech scenes still render through the player-focused stage.

Tests should focus on deterministic renderer decisions and content derivation. They do not need pixel-perfect visual assertions.

## Implementation Notes

- Reuse existing `ShotFrame`, `PlaybackItem`, and `RenderablePlayer` data.
- Keep Pixi v8 scene graph rules: group children in `Container` or `LayoutContainer`; keep `Graphics` and `Text` as leaves.
- Prefer stable child objects updated per frame over rebuilding the whole subtree every render.
- Clear and redraw simple `Graphics` panels as currently done elsewhere in the renderer.
- Do not change playback timing, audio, or event presentation in this pass.
