import type { PlaybackScenePlayer } from "@/core/playback";
import type { Rect } from "./canvas-renderer";

export type PlayerSlotSide = "left" | "right";

export type PlayerSlot = {
  readonly player: PlaybackScenePlayer;
  readonly side: PlayerSlotSide;
  readonly rect: Rect;
};

export type StageLayout = {
  readonly playerSlots: readonly PlayerSlot[];
  readonly center: Rect;
  readonly subtitle: Rect;
};

const leftRects: readonly Rect[] = [
  { x: 88, y: 150, width: 430, height: 190 },
  { x: 88, y: 382, width: 430, height: 190 },
  { x: 88, y: 614, width: 430, height: 190 },
];

const rightRects: readonly Rect[] = [
  { x: 1402, y: 150, width: 430, height: 190 },
  { x: 1402, y: 382, width: 430, height: 190 },
  { x: 1402, y: 614, width: 430, height: 190 },
];

export function createSixPlayerStageLayout(
  players: readonly PlaybackScenePlayer[],
): StageLayout {
  const sortedPlayers = [...players].sort((left, right) => left.seatNo - right.seatNo);
  const playerSlots = sortedPlayers.map((player) => {
    if (player.seatNo <= 3) {
      return {
        player,
        side: "left" as const,
        rect: leftRects[player.seatNo - 1] ?? leftRects[0]!,
      };
    }

    return {
      player,
      side: "right" as const,
      rect: rightRects[player.seatNo - 4] ?? rightRects[0]!,
    };
  });

  return {
    playerSlots,
    center: { x: 560, y: 150, width: 800, height: 650 },
    subtitle: { x: 120, y: 860, width: 1680, height: 150 },
  };
}
