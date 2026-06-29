import type { PlaybackScenePlayer } from "@/core/playback";
import type { Rect } from "../canvas-renderer";

export type SeatTrackSide = "left" | "right";

export type SeatTrackSlot = {
  readonly player: PlaybackScenePlayer;
  readonly side: SeatTrackSide;
  readonly rect: Rect;
  readonly anchor: { readonly x: number; readonly y: number };
};

export type ShotLayout = {
  readonly safe: Rect;
  readonly topBar: Rect;
  readonly seatSlots: readonly SeatTrackSlot[];
  readonly mainStage: Rect;
  readonly portrait: Rect;
  readonly eventPanel: Rect;
  readonly subtitle: Rect;
};

const CANVAS_WIDTH = 1920;
const CANVAS_HEIGHT = 1080;

const leftSeatRects: readonly Rect[] = [
  { x: 72, y: 150, width: 350, height: 112 },
  { x: 72, y: 286, width: 350, height: 112 },
  { x: 72, y: 422, width: 350, height: 112 },
];

const rightSeatRects: readonly Rect[] = [
  { x: 1498, y: 150, width: 350, height: 112 },
  { x: 1498, y: 286, width: 350, height: 112 },
  { x: 1498, y: 422, width: 350, height: 112 },
];

export function createCinematicShotLayout(
  players: readonly PlaybackScenePlayer[],
): ShotLayout {
  const sortedPlayers = [...players].sort((left, right) => left.seatNo - right.seatNo);
  const seatSlots = sortedPlayers.map((player) => {
    const side = player.seatNo <= 3 ? "left" as const : "right" as const;
    const rect = side === "left"
      ? leftSeatRects[player.seatNo - 1] ?? leftSeatRects[0]!
      : rightSeatRects[player.seatNo - 4] ?? rightSeatRects[0]!;

    return {
      player,
      side,
      rect,
      anchor: {
        x: rect.x + rect.width / 2,
        y: rect.y + rect.height / 2,
      },
    };
  });

  return {
    safe: { x: 56, y: 48, width: CANVAS_WIDTH - 112, height: CANVAS_HEIGHT - 96 },
    topBar: { x: 0, y: 0, width: CANVAS_WIDTH, height: 96 },
    seatSlots,
    mainStage: { x: 456, y: 132, width: 1008, height: 668 },
    portrait: { x: 560, y: 192, width: 800, height: 438 },
    eventPanel: { x: 560, y: 186, width: 800, height: 520 },
    subtitle: { x: 220, y: 824, width: 1480, height: 188 },
  };
}
