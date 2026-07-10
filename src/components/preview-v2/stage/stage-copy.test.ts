import { describe, expect, it } from "vitest";
import type {
  PlaybackItem,
  PlaybackSceneKind,
  PlaybackScenePlayer,
} from "@/core/playback";
import type { Phase } from "@/core/types";
import {
  PRESENTER_NAME,
  narrationTextForScene,
  transcriptPresentationForScene,
} from "./stage-copy";

describe("preview v2 stage copy ownership", () => {
  it.each<PlaybackSceneKind>([
    "phase",
    "announcement",
    "vote",
    "resolution",
  ])("assigns %s narration to the presenter", (kind) => {
    expect(transcriptPresentationForScene(item({ kind }), player()).speaker)
      .toEqual({
        kind: "presenter",
        name: PRESENTER_NAME,
        seatNo: null,
        avatar: null,
      });
  });

  it("assigns speech narration to the active player", () => {
    expect(
      transcriptPresentationForScene(
        item({ kind: "speech", phase: "speech" }),
        player(),
      ).speaker,
    ).toEqual({
      kind: "player",
      name: "林夏",
      seatNo: 4,
      avatar: "linxia.png",
    });
  });

  it("creates semantic phase narration instead of repeating the title", () => {
    const scene = item({
      kind: "phase",
      phase: "night",
      title: "第 1 夜开始",
      text: "",
    });

    expect(narrationTextForScene(scene)).toBe(
      "进入夜晚阶段，请所有玩家确认夜间行动。",
    );
    expect(narrationTextForScene(scene)).not.toBe(scene.title);
  });

  it.each<Phase>([
    "setup",
    "night",
    "day",
    "speech",
    "vote",
    "pk",
    "last_words",
    "ended",
  ])("provides explicit narration for the %s phase", (phase) => {
    const scene = item({
      kind: "phase",
      phase,
      title: `${phase} title`,
      text: "",
    });

    expect(narrationTextForScene(scene)).not.toBe("");
    expect(narrationTextForScene(scene)).not.toBe(scene.title);
  });

  it("uses event details when a non-phase scene has no narration", () => {
    expect(
      narrationTextForScene(
        item({ text: "", details: ["4 号 -> 7 号", "8 号 -> 弃票"] }),
      ),
    ).toBe("4 号 -> 7 号；8 号 -> 弃票");
  });
});

function item(overrides: Partial<PlaybackItem> = {}): PlaybackItem {
  return {
    index: 1,
    phase: "night",
    kind: "announcement",
    title: "Record",
    text: "Narration",
    details: [],
    durationMs: 1000,
    startsAtMs: 0,
    players: [],
    ...overrides,
  };
}

function player(): PlaybackScenePlayer {
  return {
    playerId: "player_4" as PlaybackScenePlayer["playerId"],
    seatNo: 4,
    name: "林夏",
    avatar: "linxia.png",
    roleName: "狼人",
    status: "alive",
    highlighted: true,
  };
}
