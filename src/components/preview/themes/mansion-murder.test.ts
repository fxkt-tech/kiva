import { describe, expect, it, vi } from "vitest";
import type { PlaybackItem, PlaybackScenePlayer } from "@/core/playback";
import { mansionMurderTheme } from "./mansion-murder";

describe("mansion murder theme", () => {
  it("renders phase scenes as chapter cards", () => {
    const ctx = fakeContext();

    mansionMurderTheme.render.renderPhase(input(ctx, scene({ kind: "phase" })));

    expect(textCalls(ctx)).toContain("CASE FILE");
    expect(textCalls(ctx)).toContain("Title");
  });

  it("renders speech scenes with speaker file and transcript", () => {
    const ctx = fakeContext();

    mansionMurderTheme.render.renderSpeech(
      input(ctx, scene({ kind: "speech", players: [player({ highlighted: true })] })),
    );

    expect(textCalls(ctx)).toContain("SUSPECT STATEMENT");
    expect(textCalls(ctx)).toContain("秦川");
    expect(textCalls(ctx)).toContain("Text");
  });

  it("renders vote scenes with evidence strings", () => {
    const ctx = fakeContext();

    mansionMurderTheme.render.renderVote(
      input(ctx, scene({ kind: "vote", players: [player({ highlighted: true }), player({ seatNo: 2, name: "林夏" })] })),
    );

    expect(textCalls(ctx)).toContain("EVIDENCE VOTE");
    expect(ctx.beginPath).toHaveBeenCalled();
    expect(ctx.lineTo).toHaveBeenCalled();
  });

  it("renders announcement scenes as case bulletins", () => {
    const ctx = fakeContext();

    mansionMurderTheme.render.renderAnnouncement(
      input(ctx, scene({ kind: "announcement" })),
    );

    expect(textCalls(ctx)).toContain("CASE BULLETIN");
    expect(textCalls(ctx)).toContain("Title");
  });

  it("renders resolution scenes as case closed reveals", () => {
    const ctx = fakeContext();

    mansionMurderTheme.render.renderResolution(
      input(ctx, scene({ kind: "resolution" })),
    );

    expect(textCalls(ctx)).toContain("CASE STATUS");
    expect(textCalls(ctx)).toContain("Title");
  });
});

function input(ctx: CanvasRenderingContext2D, scene: PlaybackItem) {
  return {
    ctx,
    scene,
    items: [scene],
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
    stroke: vi.fn(),
    arc: vi.fn(),
    save: vi.fn(),
    restore: vi.fn(),
    createLinearGradient: vi.fn(() => ({ addColorStop: vi.fn() })),
  } as unknown as CanvasRenderingContext2D;
}
