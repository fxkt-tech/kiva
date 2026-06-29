import { describe, expect, it } from "vitest";
import type { PlaybackScenePlayer } from "@/core/playback";
import { createCinematicShotLayout } from "./shot-layout";

describe("cinematic shot layout", () => {
  it("places compact seats 1-3 left and 4-6 right", () => {
    const layout = createCinematicShotLayout([
      player(6),
      player(1),
      player(4),
      player(2),
      player(5),
      player(3),
    ]);

    expect(layout.seatSlots.map((slot) => [slot.player.seatNo, slot.side]))
      .toEqual([
        [1, "left"],
        [2, "left"],
        [3, "left"],
        [4, "right"],
        [5, "right"],
        [6, "right"],
      ]);
    expect(layout.seatSlots[0]?.rect.x).toBeLessThan(layout.mainStage.x);
    expect(layout.seatSlots[3]?.rect.x).toBeGreaterThan(layout.mainStage.x);
  });

  it("keeps shot regions inside the 1920x1080 canvas without overlap between main stage and subtitle", () => {
    const layout = createCinematicShotLayout([player(1)]);

    for (const rect of [
      layout.safe,
      layout.topBar,
      layout.mainStage,
      layout.portrait,
      layout.eventPanel,
      layout.subtitle,
    ]) {
      expect(rect.x).toBeGreaterThanOrEqual(0);
      expect(rect.y).toBeGreaterThanOrEqual(0);
      expect(rect.x + rect.width).toBeLessThanOrEqual(1920);
      expect(rect.y + rect.height).toBeLessThanOrEqual(1080);
    }

    expect(layout.subtitle.y).toBeGreaterThan(
      layout.mainStage.y + layout.mainStage.height,
    );
  });
});

function player(seatNo: number): PlaybackScenePlayer {
  return {
    playerId: `player_${seatNo}` as PlaybackScenePlayer["playerId"],
    seatNo,
    name: `玩家${seatNo}`,
    avatar: null,
    roleName: "平民",
    status: "alive",
    highlighted: false,
  };
}
