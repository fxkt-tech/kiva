import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import type { PlaybackItem, PlaybackScenePlayer } from "@/core/playback";
import { createHtmlFrameViewModel } from "../composition/frame-view-model";
import {
  HtmlPlaybackStage,
  seatCardLayout,
  STAGE_BACKGROUND_IMAGE_CLASS_NAME,
  stageHeaderTitle,
  stageOpacity,
} from "./html-playback-stage";

describe("preview v2 stage transitions", () => {
  it("stays fully opaque between scenes using the same background", () => {
    const scenes = [item({ phase: "day" }), item({ phase: "speech" })];

    expect(stageOpacity(shot(scenes, 1), 0.25, 0.5)).toBe(1);
  });

  it("fades in and out only when the background switches between day and night", () => {
    const scenes = [
      item({ phase: "speech" }),
      item({ phase: "night" }),
      item({ phase: "day" }),
    ];

    expect(stageOpacity(shot(scenes, 1), 0.25, 1)).toBeCloseTo(0.37);
    expect(stageOpacity(shot(scenes, 1), 1, 0.5)).toBeCloseTo(0.58);
  });

  it("does not fade at the beginning or end of playback", () => {
    const scenes = [item({ phase: "night" })];

    expect(stageOpacity(shot(scenes, 0), 0, 0)).toBe(1);
  });
});

describe("preview v2 stage background", () => {
  it("shows the source background at full color without darkening or scanlines", () => {
    const html = renderStage(item());

    expect(STAGE_BACKGROUND_IMAGE_CLASS_NAME).toBe(
      "h-full w-full object-cover",
    );
    expect(html).not.toContain("opacity-70");
    expect(html).not.toContain("saturate-[0.72]");
    expect(html).not.toContain("repeating-linear-gradient");
    expect(html).not.toContain("rgba(4,7,5,0.18)");
    expect(html).not.toContain("rgba(2,4,3,0.08)");
  });
});

describe("preview v2 stage header", () => {
  it("shows the game title", () => {
    expect(stageHeaderTitle("周五欢乐局")).toBe("周五欢乐局");
  });

  it("shows the current scene title as the subtitle", () => {
    const html = renderStage(item({ title: "第 1 夜开始" }));

    expect(html).toContain("第 1 夜开始");
  });

  it("uses a neutral fallback when the game title is empty", () => {
    expect(stageHeaderTitle("")).toBe("未命名游戏");
  });
});

describe("preview v2 visual stage placeholder", () => {
  it("keeps only its grid slot without visual treatment or displacement", () => {
    const html = renderStage(item());
    const stage = html.match(/<section aria-label="视觉舞台"[^>]*><\/section>/u)?.[0];

    expect(stage).toBe(
      '<section aria-label="视觉舞台" class="col-start-2 row-start-2 min-h-0"></section>',
    );
    expect(html).toContain("gap-x-6 gap-y-4");
  });

  it("renders a structured action in the shared visual stage", () => {
    const players = [player(1, "守卫"), player(2, "村民")];
    const html = renderStage(item({
      players,
      stage: {
        kind: "action",
        action: "protect",
        actor: { kind: "player", playerId: players[0]!.playerId },
        targetPlayerId: players[1]!.playerId,
        result: "selected",
      },
    }));

    expect(html).toContain("守护");
    expect(html).toContain("已守护");
    expect(html).toContain("1号 玩家1");
    expect(html).toContain("2号 玩家2");
  });

  it("renders night, vote, and game resolution variants", () => {
    const players = [player(1, "狼人"), player(2, "村民")];
    const night = renderStage(item({
      players,
      stage: { kind: "night_result", deaths: [] },
    }));
    const vote = renderStage(item({
      players,
      stage: {
        kind: "vote_result",
        voteType: "pk",
        outcome: "tied",
        exiledPlayerId: null,
        candidates: [
          { playerId: players[0]!.playerId, votes: 4 },
          { playerId: players[1]!.playerId, votes: 4 },
        ],
        abstentions: 1,
      },
    }));
    const game = renderStage(item({
      players,
      stage: {
        kind: "game_result",
        winner: "wolves",
        reason: "all_gods_dead",
      },
    }));

    expect(night).toContain("平安夜");
    expect(vote).toContain("PK 结算");
    expect(vote).toContain("弃票 1");
    expect(game).toContain("狼人阵营胜利");
    expect(game).toContain("所有神职出局");
  });
});

describe("preview v2 seat cards", () => {
  it("keeps the full-height track and distributes compact rows with space-between", () => {
    const html = renderStage(item({ players: [player(1, "狼人")] }));

    expect(html).toContain(
      "row-span-3 grid min-h-0 content-between grid-rows-[repeat(6,148px)]",
    );
    expect(html).not.toContain(
      "row-span-3 grid min-h-0 content-center grid-rows-[repeat(6,148px)]",
    );
  });

  it("mirrors avatar, seat number, text, and accent placement", () => {
    expect(seatCardLayout("left")).toMatchObject({
      grid: "grid-cols-[104px_minmax(0,1fr)_112px]",
      avatar: "col-start-3 row-start-1",
      text: "row-start-1 text-center",
      seat:
        "col-start-1 row-start-1 flex h-[112px] items-center justify-start",
      accent: "right-0",
    });
    expect(seatCardLayout("right")).toMatchObject({
      grid: "grid-cols-[112px_minmax(0,1fr)_104px]",
      avatar: "col-start-1 row-start-1",
      text: "row-start-1 text-center",
      seat:
        "col-start-3 row-start-1 flex h-[112px] items-center justify-end",
      accent: "left-0",
    });
  });

  it("renders the full role spectrum and presenter gold in the shared stage", () => {
    const scene = item({
      text: "玩家1查验玩家3。",
      players: [
        player(1, "狼人", { status: "dead", highlighted: true }),
        player(2, "平民"),
        player(3, "预言家", { highlighted: true }),
        player(4, "女巫"),
        player(5, "守卫"),
        player(6, "猎人"),
      ],
    });
    const viewModel = createHtmlFrameViewModel({
      gameTitle: "身份色板测试",
      items: [scene],
      timeMs: 500,
      assets: {
        fontUrl: "font.ttf",
        dayBackgroundUrl: null,
        nightBackgroundUrl: null,
        avatarUrls: {},
      },
    });
    const html = renderToStaticMarkup(
      React.createElement(HtmlPlaybackStage, { viewModel }),
    );

    expect(html).not.toContain("KIVA · GAME RECORD");
    expect(html).toContain('aria-label="说话者信息"');
    expect(html).not.toContain(">身份</span>");
    expect(html).toContain(">主理人</span>");
    expect(html).toMatch(/style="color:#FFFAF0;[^"]*">守夜人<\/div>/u);
    expect(html).toContain("mt-2 line-clamp-3 whitespace-pre-line border-t");
    expect(html).not.toContain("line-clamp-3 self-center whitespace-pre-line");
    expect(html).not.toContain('aria-label="身份信息"');
    expect(html).toContain(">玩家1</span>");
    expect(html).toContain(">玩家3</span>");
    for (const color of [
      "#FF696E",
      "#E0E1DB",
      "#59DFA4",
      "#CF93F5",
      "#66C1FA",
      "#E8B44C",
      "#F4C45C",
    ]) {
      expect(html).toContain(`color:${color}`);
    }
    expect(html).toContain("grayscale-[0.92]");
    expect(html).toContain("opacity-35");
    expect(html).toContain("opacity:0.45");
  });

  it("gives the speaking player a stronger mirrored inner-edge treatment", () => {
    const scene = item({
      kind: "speech",
      transcriptSpeaker: "player",
      text: "我先说说我的判断。",
      players: [player(1, "狼人", { highlighted: true })],
    });
    const viewModel = createHtmlFrameViewModel({
      gameTitle: "发言状态测试",
      items: [scene],
      timeMs: 500,
      assets: {
        fontUrl: "font.ttf",
        dayBackgroundUrl: null,
        nightBackgroundUrl: null,
        avatarUrls: {},
      },
    });
    const html = renderToStaticMarkup(
      React.createElement(HtmlPlaybackStage, { viewModel }),
    );

    expect(html).toContain("box-shadow:0 0 38px rgba(66,42,6,0.45)");
    expect(html).toContain("absolute inset-y-2 w-[9px]");
    expect(html).toContain("rgba(246,197,83,0.22)");
  });
});

function shot(items: readonly PlaybackItem[], sceneIndex: number) {
  return {
    scene: items[sceneIndex]!,
    items,
  };
}

function item(overrides: Partial<PlaybackItem> = {}): PlaybackItem {
  return {
    index: 1,
    phase: "day",
    kind: "announcement",
    title: "Record",
    text: "",
    details: [],
    durationMs: 1000,
    startsAtMs: 0,
    players: [],
    presenterName: "守夜人",
    presenterAvatar: null,
    transcriptSpeaker: "presenter",
    presenterCue: { copyKey: "fallback.announcement", text: "" },
    ...overrides,
  };
}

function player(
  seatNo: number,
  roleName: string,
  overrides: Partial<PlaybackScenePlayer> = {},
): PlaybackScenePlayer {
  return {
    playerId: `player_${seatNo}` as PlaybackScenePlayer["playerId"],
    seatNo,
    name: `玩家${seatNo}`,
    avatar: null,
    roleName,
    status: "alive",
    highlighted: false,
    ...overrides,
  };
}

function renderStage(scene: PlaybackItem): string {
  const viewModel = createHtmlFrameViewModel({
    gameTitle: "布局测试",
    items: [scene],
    timeMs: 500,
    assets: {
      fontUrl: "font.ttf",
      dayBackgroundUrl: null,
      nightBackgroundUrl: null,
      avatarUrls: {},
    },
  });

  return renderToStaticMarkup(
    React.createElement(HtmlPlaybackStage, { viewModel }),
  );
}
