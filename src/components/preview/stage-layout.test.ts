import { describe, expect, it } from "vitest";
import type { PlaybackScenePlayer } from "@/core/playback";
import { createSixPlayerStageLayout } from "./stage-layout";

describe("stage layout", () => {
  it("places seats 1-3 on the left and seats 4-6 on the right", () => {
    const layout = createSixPlayerStageLayout([
      player(1),
      player(2),
      player(3),
      player(4),
      player(5),
      player(6),
    ]);

    expect(layout.playerSlots.map((slot) => [slot.player.seatNo, slot.side]))
      .toEqual([
        [1, "left"],
        [2, "left"],
        [3, "left"],
        [4, "right"],
        [5, "right"],
        [6, "right"],
      ]);
    expect(layout.playerSlots[0]?.rect.x).toBeLessThan(layout.center.x);
    expect(layout.playerSlots[3]?.rect.x).toBeGreaterThan(layout.center.x);
  });

  it("reserves center stage and bottom subtitle regions", () => {
    const layout = createSixPlayerStageLayout([player(1)]);

    expect(layout.center).toMatchObject({
      x: expect.any(Number),
      y: expect.any(Number),
      width: expect.any(Number),
      height: expect.any(Number),
    });
    expect(layout.subtitle.y).toBeGreaterThan(layout.center.y);
    expect(layout.subtitle.width).toBeGreaterThan(layout.center.width);
  });
});

function player(seatNo: number): PlaybackScenePlayer {
  return {
    playerId: `player_${seatNo}` as PlaybackScenePlayer["playerId"],
    seatNo,
    name: `玩家${seatNo}`,
    status: "alive",
    highlighted: false,
  };
}
