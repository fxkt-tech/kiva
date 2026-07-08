import { describe, expect, it } from "vitest";
import type { PlaybackItem, PlaybackScenePlayer } from "@/core/playback";
import type { RenderablePlayer, ShotFrame } from "../shot-engine/types";
import { caseBoardContentForFrame } from "./case-board-content";

describe("caseBoardContentForFrame", () => {
  it("uses scene details before highlighted players", () => {
    const highlighted = player(2, {
      highlighted: true,
      roleName: "狼人",
    });

    const content = caseBoardContentForFrame(
      frame({
        scene: scene({
          kind: "resolution",
          title: "投票结算",
          text: "放逐出局：2 号 林夏。",
          details: ["1 号 秦川 -> 2 号 林夏", "3 号 周知 -> 弃票"],
          players: [highlighted],
        }),
        highlightedPlayers: [renderablePlayer(highlighted)],
      }),
    );

    expect(content.variant).toBe("vote");
    expect(content.kindLabel).toBe("Resolution / Vote");
    expect(content.title).toBe("投票结算");
    expect(content.body).toBe("");
    expect(content.rows).toEqual([
      {
        tone: "result",
        label: "02",
        text: "2 号 林夏",
        meta: "01",
        count: 1,
      },
      {
        tone: "detail",
        label: "--",
        text: "弃票",
        meta: "03",
        count: 1,
      },
    ]);
  });

  it("uses highlighted players when details are empty", () => {
    const highlighted = player(9, {
      highlighted: true,
      name: "苏瑾",
      roleName: "平民",
      status: "dead",
    });

    const content = caseBoardContentForFrame(
      frame({
        scene: scene({
          kind: "announcement",
          title: "昨夜死讯",
          text: "昨夜死亡：9 号 苏瑾。",
          details: [],
          players: [highlighted],
        }),
        highlightedPlayers: [renderablePlayer(highlighted)],
      }),
    );

    expect(content.rows).toEqual([
      {
        tone: "dead-player",
        label: "09",
        text: "苏瑾",
        meta: "平民",
      },
    ]);
  });

  it("groups vote detail rows by target", () => {
    const content = caseBoardContentForFrame(
      frame({
        scene: scene({
          kind: "resolution",
          title: "投票结算",
          text: "放逐出局：5 号 唐棠。",
          details: [
            "1 号 秦川 -> 5 号 唐棠",
            "2 号 林夏 -> 5 号 唐棠",
            "3 号 周知 -> 弃票",
            "4 号 许砚 -> 8 号 夏宇",
          ],
        }),
      }),
    );

    expect(content).toMatchObject({
      variant: "vote",
      kindLabel: "Resolution / Vote",
      title: "投票结算",
      body: "",
    });
    expect(content.rows).toEqual([
      {
        tone: "result",
        label: "05",
        text: "5 号 唐棠",
        meta: "01  02",
        count: 2,
      },
      {
        tone: "detail",
        label: "--",
        text: "弃票",
        meta: "03",
        count: 1,
      },
      {
        tone: "detail",
        label: "08",
        text: "8 号 夏宇",
        meta: "04",
        count: 1,
      },
    ]);
  });

  it("reduces ending scenes to the winning camp", () => {
    const content = caseBoardContentForFrame(
      frame({
        scene: scene({
          kind: "resolution",
          title: "游戏结束：狼人阵营胜利",
          text: "狼人阵营胜利，原因：所有平民出局。",
          details: ["1 号 秦川：狼人（狼人阵营）"],
        }),
      }),
    );

    expect(content).toEqual({
      variant: "ending",
      kindLabel: "Final / Reveal",
      title: "狼人阵营胜利",
      body: "",
      rows: [],
    });
  });

  it("falls back to a neutral public record row", () => {
    const content = caseBoardContentForFrame(
      frame({
        scene: scene({
          kind: "phase",
          title: "第 1 夜开始",
          text: "夜晚开始。",
          details: [],
          players: [player(1)],
        }),
        highlightedPlayers: [],
      }),
    );

    expect(content.rows).toEqual([
      { tone: "neutral", label: "FILE", text: "PUBLIC RECORD" },
    ]);
  });

  it("labels vote scenes for the case board", () => {
    const content = caseBoardContentForFrame(
      frame({
        scene: scene({
          kind: "vote",
          title: "放逐投票",
          text: "进入本日放逐投票。",
        }),
      }),
    );

    expect(content.kindLabel).toBe("VOTE");
  });

  it("labels speech scenes", () => {
    const content = caseBoardContentForFrame(
      frame({
        scene: scene({
          kind: "speech",
          title: "4 号 林夏发言",
          text: "我是4号林夏。",
        }),
      }),
    );

    expect(content.kindLabel).toBe("SPEECH");
  });
});

function scene(overrides: Partial<PlaybackItem> = {}): PlaybackItem {
  return {
    index: 1,
    phase: "night",
    kind: "phase",
    title: "第 1 夜开始",
    text: "夜晚开始。",
    details: [],
    durationMs: 2000,
    startsAtMs: 0,
    players: [],
    ...overrides,
  };
}

function frame(overrides: Partial<ShotFrame> = {}): ShotFrame {
  const baseScene = overrides.scene ?? scene();

  return {
    gameTitle: overrides.gameTitle ?? "",
    scene: baseScene,
    items: [baseScene],
    layout: {
      safe: { x: 56, y: 48, width: 1808, height: 984 },
      topBar: { x: 0, y: 0, width: 1920, height: 96 },
      seatSlots: [],
      mainStage: { x: 480, y: 130, width: 960, height: 700 },
      portrait: { x: 524, y: 180, width: 872, height: 520 },
      eventPanel: { x: 520, y: 170, width: 880, height: 620 },
      subtitle: { x: 220, y: 852, width: 1480, height: 168 },
    },
    clock: {
      absoluteMs: 0,
      sceneMs: 0,
      durationMs: 2000,
      progress: 0,
      enterProgress: 0,
      exitProgress: 1,
    },
    players: baseScene.players.map((candidate) => ({
      ...candidate,
      emphasis: candidate.highlighted ? "highlighted" : "normal",
    })),
    activePlayer: null,
    highlightedPlayers: [],
    subtitle: null,
    backgroundImages: { day: null, night: null },
    avatarImages: {},
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

function renderablePlayer(
  basePlayer: PlaybackScenePlayer,
  emphasis: RenderablePlayer["emphasis"] = "highlighted",
): RenderablePlayer {
  return {
    ...basePlayer,
    emphasis,
  };
}
