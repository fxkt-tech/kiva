import { describe, expect, it, vi } from "vitest";
import type { PlaybackScenePlayer } from "@/core/playback";
import {
  clearCanvas,
  drawPanel,
  drawPlayerFile,
  drawTextBlock,
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
    stroke: vi.fn(),
    arc: vi.fn(),
    save: vi.fn(),
    restore: vi.fn(),
    createLinearGradient: vi.fn(() => ({ addColorStop: vi.fn() })),
  } as unknown as CanvasRenderingContext2D;
}
