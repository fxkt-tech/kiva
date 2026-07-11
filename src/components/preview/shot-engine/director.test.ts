import { describe, expect, it } from "vitest";
import type { PlaybackItem, PlaybackScenePlayer } from "@/core/playback";
import { createShotClock, createShotFrame } from "./director";

describe("preview shot director", () => {
  it("derives active speaker and subtitle cue for speech scenes", () => {
    const frame = createShotFrame({
      scene: scene({
        kind: "speech",
        text: "我是4号林夏。这里先说重点。",
        players: [player(1), player(4, { highlighted: true })],
      }),
      items: [],
      timeMs: 500,
      backgroundImages: { day: null, night: null },
      avatarImages: {},
    });

    expect(frame.activePlayer?.seatNo).toBe(4);
    expect(frame.highlightedPlayers.map((player) => player.seatNo)).toEqual([4]);
    expect(frame.subtitle?.speaker).toBe("4 号 玩家4");
  });

  it("keeps non-speech highlights without promoting an active speaker", () => {
    const frame = createShotFrame({
      scene: scene({
        kind: "resolution",
        players: [player(2, { highlighted: true }), player(3)],
      }),
      items: [],
      timeMs: 1000,
      backgroundImages: { day: null, night: null },
      avatarImages: {},
    });

    expect(frame.activePlayer).toBeNull();
    expect(frame.highlightedPlayers.map((player) => player.seatNo)).toEqual([2]);
    expect(frame.subtitle).toBeNull();
  });

  it("clamps shot clock progress", () => {
    const shotClock = createShotClock(scene({ startsAtMs: 1000, durationMs: 2000 }), 5000);

    expect(shotClock.sceneMs).toBe(2000);
    expect(shotClock.progress).toBe(1);
  });
});

function scene(overrides: Partial<PlaybackItem> = {}): PlaybackItem {
  return {
    index: 1,
    phase: "day",
    kind: "announcement",
    title: "Title",
    text: "Text",
    details: [],
    durationMs: 2000,
    startsAtMs: 0,
    players: [],
    presenterName: "守夜人",
    presenterAvatar: null,
    transcriptSpeaker: "presenter",
    presenterCue: { copyKey: "fallback.announcement", text: "Text", voiceFile: null },
    ...overrides,
  };
}

function player(
  seatNo: number,
  overrides: Partial<PlaybackScenePlayer> = {},
): PlaybackScenePlayer {
  return {
    playerId: `player_${seatNo}` as PlaybackScenePlayer["playerId"],
    seatNo,
    name: `玩家${seatNo}`,
    avatar: null,
    roleName: "平民",
    status: "alive",
    highlighted: false,
    ...overrides,
  };
}
