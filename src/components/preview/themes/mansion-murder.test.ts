import { describe, expect, it, vi } from "vitest";
import type { PlaybackItem, PlaybackScenePlayer } from "@/core/playback";
import { mansionMurderTheme } from "./mansion-murder";

describe("mansion murder theme", () => {
  it("renders backgrounds with the case bar", () => {
    const ctx = fakeContext();

    mansionMurderTheme.render.renderBackground(input(ctx, scene({ kind: "phase" })));

    expect(textCalls(ctx)).toContain("MANSION MURDER");
    expect(textCalls(ctx)).toContain("Title");
  });

  it("renders fixed suspect cards on both sides", () => {
    const ctx = fakeContext();

    mansionMurderTheme.render.renderPlayerCard(input(ctx, scene()));

    expect(textCalls(ctx)).toContain("Seat 1");
    expect(textCalls(ctx)).toContain("Seat 4");
    expect(textCalls(ctx)).toContain("秦川");
  });

  it("renders speech scenes with speaker focus and subtitles", () => {
    const ctx = fakeContext();
    const speechScene = scene({ kind: "speech", players: [player({ highlighted: true })] });

    mansionMurderTheme.render.renderCenterStage(input(ctx, speechScene));
    mansionMurderTheme.render.renderSubtitle(input(ctx, speechScene));

    expect(textCalls(ctx)).toContain("SUSPECT STATEMENT");
    expect(textCalls(ctx)).toContain("秦川");
    expect(textCalls(ctx)).toContain("Text");
  });

  it("renders vote scenes as center result tables without lines", () => {
    const ctx = fakeContext();

    mansionMurderTheme.render.renderCenterStage(
      input(ctx, scene({
        kind: "vote",
        title: "放逐投票",
        text: "1 号 秦川 投给 2 号 林夏。",
        details: [],
      })),
    );

    expect(textCalls(ctx)).toContain("放逐投票");
    expect(textCalls(ctx)).toContain("等待本轮投票结算");
    expect(ctx.lineTo).not.toHaveBeenCalled();
  });

  it("renders vote resolutions with visible vote rows", () => {
    const ctx = fakeContext();

    mansionMurderTheme.render.renderCenterStage(
      input(ctx, scene({
        kind: "resolution",
        phase: "vote",
        title: "投票结算",
        text: "2 号 林夏出局。",
        details: ["1 号 秦川 -> 2 号 林夏"],
      })),
    );

    expect(textCalls(ctx)).toContain("投票结算");
    expect(textCalls(ctx)).toContain("1 号 秦川 -> 2 号 林夏");
    expect(textCalls(ctx)).toContain("本轮结果");
  });

  it("renders pk vote resolutions with visible vote rows", () => {
    const ctx = fakeContext();

    mansionMurderTheme.render.renderCenterStage(
      input(ctx, scene({
        kind: "resolution",
        phase: "vote",
        title: "PK 结算",
        text: "无人出局，进入下一轮。",
        details: ["1 号 秦川 -> 2 号 林夏"],
      })),
    );

    expect(textCalls(ctx)).toContain("PK 结算");
    expect(textCalls(ctx)).toContain("1 号 秦川 -> 2 号 林夏");
    expect(textCalls(ctx)).toContain("本轮结果");
  });
});

function input(ctx: CanvasRenderingContext2D, scene: PlaybackItem) {
  return {
    ctx,
    scene,
    items: [scene],
    layout: {
      playerSlots: scene.players.map((scenePlayer, index) => ({
        player: scenePlayer,
        side: scenePlayer.seatNo <= 3 ? "left" as const : "right" as const,
        rect: {
          x: scenePlayer.seatNo <= 3 ? 88 : 1402,
          y: 150 + (index % 3) * 232,
          width: 430,
          height: 190,
        },
      })),
      center: { x: 560, y: 150, width: 800, height: 650 },
      subtitle: { x: 120, y: 860, width: 1680, height: 150 },
    },
    timeMs: scene.startsAtMs,
    sceneTimeMs: 0,
    enterProgress: 1,
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
      player({ seatNo: 4, name: "许棠" }),
      player({ seatNo: 5, name: "陈墨" }),
      player({ seatNo: 6, name: "沈岚", status: "dead" }),
    ],
    ...overrides,
  };
}

function player(overrides: Partial<PlaybackScenePlayer> = {}): PlaybackScenePlayer {
  return {
    playerId: `player_${overrides.seatNo ?? 1}` as PlaybackScenePlayer["playerId"],
    seatNo: 1,
    name: "秦川",
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
    fill: vi.fn(),
    stroke: vi.fn(),
    arc: vi.fn(),
    save: vi.fn(),
    restore: vi.fn(),
    createLinearGradient: vi.fn(() => ({ addColorStop: vi.fn() })),
  } as unknown as CanvasRenderingContext2D;
}
