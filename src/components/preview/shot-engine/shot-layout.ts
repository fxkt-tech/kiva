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
const SIDE_WIDTH = CANVAS_WIDTH / 4;
const CENTER_WIDTH = CANVAS_WIDTH / 2;
const STAGE_TOP = 130;
const STAGE_HEIGHT = 700;
const SEAT_HEIGHT = 210;
const SEAT_GAP = 35;
const SEAT_MARGIN_X = 32;
const SEAT_WIDTH = SIDE_WIDTH - SEAT_MARGIN_X * 2;
const RIGHT_COLUMN_X = SIDE_WIDTH + CENTER_WIDTH;

const leftSeatRects: readonly Rect[] = [
  { x: SEAT_MARGIN_X, y: STAGE_TOP, width: SEAT_WIDTH, height: SEAT_HEIGHT },
  { x: SEAT_MARGIN_X, y: STAGE_TOP + SEAT_HEIGHT + SEAT_GAP, width: SEAT_WIDTH, height: SEAT_HEIGHT },
  { x: SEAT_MARGIN_X, y: STAGE_TOP + (SEAT_HEIGHT + SEAT_GAP) * 2, width: SEAT_WIDTH, height: SEAT_HEIGHT },
];

const rightSeatRects: readonly Rect[] = [
  { x: RIGHT_COLUMN_X + SEAT_MARGIN_X, y: STAGE_TOP, width: SEAT_WIDTH, height: SEAT_HEIGHT },
  { x: RIGHT_COLUMN_X + SEAT_MARGIN_X, y: STAGE_TOP + SEAT_HEIGHT + SEAT_GAP, width: SEAT_WIDTH, height: SEAT_HEIGHT },
  { x: RIGHT_COLUMN_X + SEAT_MARGIN_X, y: STAGE_TOP + (SEAT_HEIGHT + SEAT_GAP) * 2, width: SEAT_WIDTH, height: SEAT_HEIGHT },
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
    mainStage: { x: SIDE_WIDTH, y: STAGE_TOP, width: CENTER_WIDTH, height: STAGE_HEIGHT },
    portrait: { x: SIDE_WIDTH + 44, y: STAGE_TOP + 50, width: CENTER_WIDTH - 88, height: 520 },
    eventPanel: { x: SIDE_WIDTH + 40, y: STAGE_TOP + 40, width: CENTER_WIDTH - 80, height: STAGE_HEIGHT - 80 },
    subtitle: { x: 220, y: 852, width: 1480, height: 168 },
  };
}
