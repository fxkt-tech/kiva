import { describe, expect, it, vi } from "vitest";
import type { PlaybackItem, PlaybackScenePlayer } from "@/core/playback";
import { createShotFrame } from "../shot-engine/director";
import type { SkinRenderInput } from "../show-theme";
import { mansionMurderTheme } from "./mansion-murder";

describe("mansion murder theme", () => {
  it("renders backgrounds with the case bar", () => {
    const ctx = fakeContext();

    mansionMurderTheme.draw.background(input(ctx, scene({ kind: "phase" })));
    mansionMurderTheme.draw.topBar(input(ctx, scene({ kind: "phase" })));

    expect(textCalls(ctx)).toContain("MANSION MURDER");
    expect(textCalls(ctx)).toContain("Title");
  });

  it("uses day and night background images by phase", () => {
    const dayContext = fakeContext();
    const nightContext = fakeContext();

    mansionMurderTheme.draw.background(
      input(dayContext, scene({ phase: "day" }), {
        backgroundImages: {
          day: image({ width: 1672, height: 941 }),
          night: image({ width: 1200, height: 900 }),
        },
      }),
    );
    mansionMurderTheme.draw.background(
      input(nightContext, scene({ phase: "night" }), {
        backgroundImages: {
          day: image({ width: 1672, height: 941 }),
          night: image({ width: 1200, height: 900 }),
        },
      }),
    );

    expect(dayContext.drawImage).toHaveBeenCalledWith(
      expect.objectContaining({ width: 1672 }),
      expect.any(Number),
      expect.any(Number),
      expect.any(Number),
      expect.any(Number),
      0,
      0,
      1920,
      1080,
    );
    expect(nightContext.drawImage).toHaveBeenCalledWith(
      expect.objectContaining({ width: 1200 }),
      expect.any(Number),
      expect.any(Number),
      expect.any(Number),
      expect.any(Number),
      0,
      0,
      1920,
      1080,
    );
  });

  it("renders compact seat track items on both sides", () => {
    const ctx = fakeContext();
    const renderInput = input(ctx, scene());

    renderInput.frame.layout.seatSlots.forEach((slot) => {
      mansionMurderTheme.draw.seat({ ...renderInput, slot });
    });

    expect(textCalls(ctx)).toContain("Seat 1");
    expect(textCalls(ctx)).toContain("Seat 4");
    expect(textCalls(ctx)).toContain("秦川");
    expect(textCalls(ctx)).toContain("身份：狼人");
  });

  it("renders player avatar images when they are loaded", () => {
    const ctx = fakeContext();
    const avatar = "/kivdb-assets/characters/avatar_qinchuan.png";
    const renderInput = input(ctx, scene({
      players: [player({ avatar })],
    }), {
      avatarImages: {
        [avatar]: image({ width: 1200, height: 1200 }),
      },
    });

    mansionMurderTheme.draw.seat({
      ...renderInput,
      slot: renderInput.frame.layout.seatSlots[0]!,
    });

    expect(ctx.drawImage).toHaveBeenCalledWith(
      expect.objectContaining({ width: 1200 }),
      0,
      0,
      1200,
      1200,
      262,
      150,
      168,
      168,
    );
  });

  it("renders speech scenes with speaker focus and subtitles", () => {
    const ctx = fakeContext();
    const speechScene = scene({ kind: "speech", players: [player({ highlighted: true })] });

    mansionMurderTheme.draw.speechShot(input(ctx, speechScene));
    mansionMurderTheme.draw.subtitle(input(ctx, speechScene));

    expect(textCalls(ctx)).toContain("SUSPECT STATEMENT");
    expect(textCalls(ctx)).toContain("秦川");
    expect(textCalls(ctx)).toContain("Text");
  });

  it("renders vote scenes as a pending vote board without lines", () => {
    const ctx = fakeContext();

    mansionMurderTheme.draw.voteShot(
      input(ctx, scene({
        kind: "vote",
        title: "放逐投票",
        text: "1 号 秦川 投给 2 号 林夏。",
        details: [],
      })),
    );

    expect(textCalls(ctx)).toContain("放逐投票");
    expect(textCalls(ctx)).toContain("投票明细");
    expect(textCalls(ctx)).toContain("票型统计");
    expect(textCalls(ctx)).toContain("等待所有玩家完成投票");
    expect(ctx.lineTo).not.toHaveBeenCalled();
  });

  it("renders vote resolutions with visible vote rows", () => {
    const ctx = fakeContext();

    mansionMurderTheme.draw.resolutionShot(
      input(ctx, scene({
        kind: "resolution",
        phase: "vote",
        title: "投票结算",
        text: "2 号 林夏出局。",
        details: ["1 号 秦川 -> 2 号 林夏"],
      })),
    );

    expect(textCalls(ctx)).toContain("投票结算");
    expect(textCalls(ctx)).toContain("1 号 秦川");
    expect(textCalls(ctx)).toContain("2 号 林夏");
    expect(textCalls(ctx)).toContain("放逐出局");
    expect(textCalls(ctx)).toContain("1 票");
  });

  it("renders pk vote resolutions with visible vote rows", () => {
    const ctx = fakeContext();

    mansionMurderTheme.draw.resolutionShot(
      input(ctx, scene({
        kind: "resolution",
        phase: "vote",
        title: "PK 结算",
        text: "无人出局，进入下一轮。",
        details: ["1 号 秦川 -> 2 号 林夏"],
      })),
    );

    expect(textCalls(ctx)).toContain("PK 结算");
    expect(textCalls(ctx)).toContain("1 号 秦川");
    expect(textCalls(ctx)).toContain("2 号 林夏");
    expect(textCalls(ctx)).toContain("当前状态");
  });
});

function input(
  ctx: CanvasRenderingContext2D,
  scene: PlaybackItem,
  overrides: Partial<SkinRenderInput["frame"]> = {},
): SkinRenderInput {
  const frame = createShotFrame({
    scene,
    items: [scene],
    timeMs: scene.startsAtMs,
    backgroundImages: { day: null, night: null },
    avatarImages: {},
  });

  return {
    ctx,
    frame: { ...frame, ...overrides },
    skin: mansionMurderTheme,
  };
}

function scene(overrides: Partial<PlaybackItem> = {}): PlaybackItem {
  return {
    index: 1,
    phase: "day",
    kind: "announcement",
    title: "Title",
    text: "Text",
    details: ["Detail"],
    durationMs: 2000,
    startsAtMs: 0,
    players: [
      player({ seatNo: 1, name: "秦川", highlighted: true }),
      player({ seatNo: 2, name: "林夏" }),
      player({ seatNo: 3, name: "周知" }),
      player({ seatNo: 4, name: "夏宇" }),
      player({ seatNo: 5, name: "陈墨" }),
      player({ seatNo: 6, name: "顾清妍", status: "dead" }),
    ],
    ...overrides,
  };
}

function player(overrides: Partial<PlaybackScenePlayer> = {}): PlaybackScenePlayer {
  return {
    playerId: `player_${overrides.seatNo ?? 1}` as PlaybackScenePlayer["playerId"],
    seatNo: 1,
    name: "秦川",
    avatar: null,
    roleName: "狼人",
    status: "alive",
    highlighted: false,
    ...overrides,
  };
}

function textCalls(ctx: CanvasRenderingContext2D): string[] {
  return vi.mocked(ctx.fillText).mock.calls.map((call) => String(call[0]));
}

function fakeContext(): CanvasRenderingContext2D {
  return {
    canvas: { width: 1920, height: 1080 },
    fillRect: vi.fn(),
    strokeRect: vi.fn(),
    fillText: vi.fn(),
    measureText: vi.fn((text: string) => ({ width: text.length * 12 })),
    beginPath: vi.fn(),
    moveTo: vi.fn(),
    lineTo: vi.fn(),
    quadraticCurveTo: vi.fn(),
    closePath: vi.fn(),
    fill: vi.fn(),
    stroke: vi.fn(),
    arc: vi.fn(),
    clip: vi.fn(),
    drawImage: vi.fn(),
    save: vi.fn(),
    restore: vi.fn(),
    createLinearGradient: vi.fn(() => ({ addColorStop: vi.fn() })),
    globalAlpha: 1,
  } as unknown as CanvasRenderingContext2D;
}

function image(input: {
  readonly width: number;
  readonly height: number;
}): HTMLImageElement {
  return input as HTMLImageElement;
}
