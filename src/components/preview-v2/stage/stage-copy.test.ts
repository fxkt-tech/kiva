import { describe, expect, it } from "vitest";
import type {
  PlaybackItem,
  PlaybackScenePlayer,
} from "@/core/playback";
import {
  narrationTextForScene,
  transcriptSpeakerIdentity,
  transcriptTextSegments,
  transcriptPresentationForScene,
} from "./stage-copy";

describe("preview v2 stage copy ownership", () => {
  it("uses the selected presenter snapshot identity", () => {
    expect(
      transcriptPresentationForScene(
        item({ presenterName: "法官", presenterAvatar: "judge.png" }),
        player(),
      ).speaker,
    ).toEqual({
      kind: "presenter",
      name: "法官",
      seatNo: null,
      avatar: "judge.png",
      roleName: null,
    });
  });

  it("assigns speech narration to the active player", () => {
    expect(
      transcriptPresentationForScene(
        item({
          kind: "speech",
          phase: "speech",
          transcriptSpeaker: "player",
        }),
        player(),
      ).speaker,
    ).toEqual({
      kind: "player",
      name: "林夏",
      seatNo: 4,
      avatar: "linxia.png",
      roleName: "狼人",
    });
  });

  it("shows the player role as their identity", () => {
    const presentation = transcriptPresentationForScene(
      item({ transcriptSpeaker: "player" }),
      player(),
    );

    expect(transcriptSpeakerIdentity(presentation)).toEqual({
      kind: "role",
      label: "狼人",
    });
  });

  it("shows only presenter identity when the presenter is speaking", () => {
    const presentation = transcriptPresentationForScene(item(), null);

    expect(transcriptSpeakerIdentity(presentation)).toEqual({
      kind: "presenter",
      label: "主理人",
    });
  });

  it("colors player mentions in narration with each player's role", () => {
    const wolf = player();
    const seer = {
      ...player(),
      playerId: "seer" as PlaybackScenePlayer["playerId"],
      seatNo: 12,
      name: "沈岚",
      roleName: "预言家",
    };

    expect(
      transcriptTextSegments(
        "4号林夏查验12号沈岚，林夏认为沈岚可以信任。",
        "presenter",
        [wolf, seer],
      ),
    ).toEqual([
      { text: "4号林夏", roleName: "狼人" },
      { text: "查验", roleName: null },
      { text: "12号沈岚", roleName: "预言家" },
      { text: "，", roleName: null },
      { text: "林夏", roleName: "狼人" },
      { text: "认为", roleName: null },
      { text: "沈岚", roleName: "预言家" },
      { text: "可以信任。", roleName: null },
    ]);
  });

  it("does not color an unrelated larger seat number as a player mention", () => {
    expect(transcriptTextSegments("21号线索", "presenter", [player()])).toEqual([
      { text: "21号线索", roleName: null },
    ]);
  });

  it("does not parse player-authored speech for mentions", () => {
    expect(
      transcriptTextSegments(
        "我觉得4号林夏像狼，林夏今晚可能会动手。",
        "player",
        [player()],
      ),
    ).toEqual([
      {
        text: "我觉得4号林夏像狼，林夏今晚可能会动手。",
        roleName: null,
      },
    ]);
  });

  it("uses only the resolved snapshot transcript", () => {
    const scene = item({
      kind: "phase",
      phase: "night",
      title: "第 1 夜开始",
      text: "天黑请闭眼。",
    });

    expect(narrationTextForScene(scene)).toBe("天黑请闭眼。");
    expect(narrationTextForScene(scene)).not.toBe(scene.title);
  });

  it("does not invent a component-local fallback", () => {
    expect(narrationTextForScene(item({ text: "" }))).toBe("");
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
    presenterName: "守夜人",
    presenterAvatar: null,
    transcriptSpeaker: "presenter",
    presenterCue: { copyKey: "fallback.announcement", text: "Narration" },
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
