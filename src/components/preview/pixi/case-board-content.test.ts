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
    expect(content.kindLabel).toBe("投票结算");
    expect(content.title).toBe("投票结算");
    expect(content.hero).toBe("2 号 林夏 出局");
    expect(content.body).toBe("");
    expect(content.metrics).toEqual([]);
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

    expect(content).toMatchObject({
      variant: "death",
      hero: "9 号 苏瑾",
      body: "昨夜死亡",
    });
    expect(content.metrics).toEqual([]);
    expect(content.relation).toBeNull();
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
      kindLabel: "投票结算",
      title: "投票结算",
      hero: "5 号 唐棠 出局",
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
      tone: "danger",
      kindLabel: "终局",
      title: "游戏结束",
      hero: "狼人阵营胜利",
      body: "原因：所有平民出局",
      metrics: [],
      relation: null,
      rows: [],
    });
  });

  it("summarizes phase scenes without repeating phase body text", () => {
    const content = caseBoardContentForFrame(
      frame({
        scene: scene({
          kind: "phase",
          title: "第 1 夜开始",
          text: "",
          details: [],
          players: [player(1)],
        }),
        highlightedPlayers: [],
      }),
    );

    expect(content).toMatchObject({
      variant: "phase",
      tone: "night",
      kindLabel: "夜晚",
      title: "第 1 夜",
      hero: "夜间行动",
      body: "",
    });
    expect(content.metrics).toEqual([]);
    expect(content.relation).toBeNull();
    expect(content.rows).toEqual([]);
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

    expect(content.kindLabel).toBe("投票");
  });

  it("uses highlighted players instead of raw log rows for night actions", () => {
    const actor = player(7, {
      highlighted: true,
      name: "陈墨",
      roleName: "预言家",
    });
    const target = player(12, {
      highlighted: true,
      name: "沈岚",
      roleName: "守卫",
    });

    const content = caseBoardContentForFrame(
      frame({
        scene: scene({
          kind: "announcement",
          title: "预言家查验",
          text: "7 号 陈墨 选择查验 12 号 沈岚。",
          details: ["7 号 陈墨 选择查验 12 号 沈岚。"],
          players: [actor, target],
        }),
        highlightedPlayers: [
          renderablePlayer(actor),
          renderablePlayer(target),
        ],
      }),
    );

    expect(content).toMatchObject({
      variant: "night-action",
      tone: "good",
      kindLabel: "夜间行动",
      title: "预言家查验",
      hero: "12 号 沈岚",
      body: "查验目标",
    });
    expect(content.relation).toEqual({
      actor: { tone: "alive-player", label: "07", name: "陈墨", roleName: "预言家" },
      action: "查验",
      target: { tone: "alive-player", label: "12", name: "沈岚", roleName: "守卫" },
    });
    expect(content.rows).toEqual([]);
  });

  it("keeps seer result as a separate broadcast scene", () => {
    const actor = player(7, {
      highlighted: true,
      name: "陈墨",
      roleName: "预言家",
    });
    const target = player(12, {
      highlighted: true,
      name: "沈岚",
      roleName: "守卫",
    });

    const content = caseBoardContentForFrame(
      frame({
        scene: scene({
          kind: "announcement",
          title: "查验结果",
          text: "7 号 陈墨查验 12 号 沈岚，结果：好人。",
          details: ["7 号 陈墨查验 12 号 沈岚，结果：好人。"],
          players: [actor, target],
        }),
        highlightedPlayers: [
          renderablePlayer(actor),
          renderablePlayer(target),
        ],
      }),
    );

    expect(content).toMatchObject({
      variant: "night-action",
      tone: "good",
      kindLabel: "夜间行动",
      title: "查验结果",
      hero: "阵营：好人",
      body: "12 号 沈岚",
    });
    expect(content.relation).toEqual({
      actor: { tone: "alive-player", label: "07", name: "陈墨", roleName: "预言家" },
      action: "查验",
      target: { tone: "alive-player", label: "12", name: "沈岚", roleName: "守卫" },
      result: "好人",
    });
    expect(content.rows).toEqual([]);
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

    expect(content.kindLabel).toBe("发言");
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
