import { describe, expect, it, vi } from "vitest";
import type { PlaybackScenePlayer } from "@/core/playback";
import {
  clearCanvas,
  drawAvatar,
  drawPanel,
  drawPlayerFile,
  drawStatusStamp,
  drawSubtitleBar,
  drawTextBlock,
  drawVoteResultTable,
  drawVoteLine,
} from "./canvas-renderer";

describe("canvas renderer helpers", () => {
  it("clears the full canvas with the given color", () => {
    const ctx = fakeContext();

    clearCanvas(ctx, "#010203");

    expect(ctx.fillStyle).toBe("#010203");
    expect(ctx.fillRect).toHaveBeenCalledWith(0, 0, 1920, 1080);
  });

  it("wraps text blocks when measured width exceeds max width", () => {
    const ctx = fakeContext({ measuredWidth: 999 });

    drawTextBlock(ctx, "abcdef", 10, 20, {
      color: "#fff",
      font: "20px sans-serif",
      maxWidth: 12,
      lineHeight: 30,
    });

    expect(ctx.fillText).toHaveBeenCalledTimes(6);
  });

  it("draws filled and stroked panels", () => {
    const ctx = fakeContext();

    drawPanel(
      ctx,
      { x: 12, y: 24, width: 320, height: 160 },
      { fill: "#111", stroke: "#222" },
    );

    expect(ctx.fillRect).toHaveBeenCalledWith(12, 24, 320, 160);
    expect(ctx.strokeRect).toHaveBeenCalledWith(12, 24, 320, 160);
  });

  it("draws player file seat, name, and status", () => {
    const ctx = fakeContext();

    drawPlayerFile(
      ctx,
      player({ seatNo: 3, name: "周知", status: "dead", highlighted: true }),
      { x: 100, y: 120, width: 380, height: 86 },
      {
        fill: "#111",
        stroke: "#900",
        text: "#fff",
        muted: "#777",
        statusAlive: "#0f0",
        statusDead: "#f00",
      },
    );

    expect(ctx.fillText).toHaveBeenCalledWith("Seat 3", 120, 166);
    expect(ctx.fillText).toHaveBeenCalledWith("周知", 222, 167);
    expect(ctx.fillText).toHaveBeenCalledWith("dead", 390, 166);
  });

  it("draws vote lines between two points", () => {
    const ctx = fakeContext();

    drawVoteLine(ctx, { x: 10, y: 20 }, { x: 80, y: 90 }, "#b91c1c");

    expect(ctx.beginPath).toHaveBeenCalled();
    expect(ctx.moveTo).toHaveBeenCalledWith(10, 20);
    expect(ctx.lineTo).toHaveBeenCalledWith(80, 90);
    expect(ctx.stroke).toHaveBeenCalled();
  });

  it("draws avatar placeholders with initials", () => {
    const ctx = fakeContext();

    drawAvatar(ctx, "周知", { x: 40, y: 50, radius: 28 }, {
      fill: "#111",
      stroke: "#222",
      text: "#fff",
    });

    expect(ctx.arc).toHaveBeenCalledWith(40, 50, 28, 0, Math.PI * 2);
    expect(ctx.fillText).toHaveBeenCalledWith("周", 28, 60);
  });

  it("draws status stamps", () => {
    const ctx = fakeContext();

    drawStatusStamp(ctx, "DEAD", { x: 80, y: 90, width: 120, height: 44 }, {
      fill: "#300",
      stroke: "#900",
      text: "#fee",
    });

    expect(ctx.fillText).toHaveBeenCalledWith("DEAD", 102, 119);
  });

  it("draws vote result tables", () => {
    const ctx = fakeContext();

    drawVoteResultTable(ctx, {
      title: "放逐投票",
      rows: ["1号 秦川 -> 3号 周知", "2号 林夏 -> 弃票"],
      result: ["3号 周知 1票", "弃票 1票"],
      rect: { x: 100, y: 120, width: 620, height: 360 },
      colors: {
        fill: "#111",
        stroke: "#222",
        title: "#fff",
        text: "#ddd",
        accent: "#f00",
      },
    });

    expect(ctx.fillText).toHaveBeenCalledWith("放逐投票", 132, 174);
    expect(ctx.fillText).toHaveBeenCalledWith("1号 秦川 -> 3号 周知", 132, 232);
    expect(ctx.fillText).toHaveBeenCalledWith("本轮结果", 132, 362);
  });

  it("draws subtitle bars", () => {
    const ctx = fakeContext();

    drawSubtitleBar(ctx, {
      speaker: "1号 秦川",
      text: "我先发言。",
      rect: { x: 120, y: 860, width: 1680, height: 150 },
      colors: {
        fill: "#111",
        stroke: "#222",
        speaker: "#0ff",
        text: "#fff",
      },
    });

    expect(ctx.fillText).toHaveBeenCalledWith("1号 秦川", 156, 916);
    expect(ctx.fillText).toHaveBeenCalledWith("我先发言。", 156, 970);
  });
});

function player(overrides: Partial<PlaybackScenePlayer>): PlaybackScenePlayer {
  return {
    playerId: "player_1" as PlaybackScenePlayer["playerId"],
    seatNo: 1,
    name: "秦川",
    status: "alive",
    highlighted: false,
    ...overrides,
  };
}

function fakeContext({
  measuredWidth = 10,
}: {
  readonly measuredWidth?: number;
} = {}): CanvasRenderingContext2D {
  return {
    canvas: { width: 1920, height: 1080 },
    fillRect: vi.fn(),
    strokeRect: vi.fn(),
    fillText: vi.fn(),
    measureText: vi.fn(() => ({ width: measuredWidth })),
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
