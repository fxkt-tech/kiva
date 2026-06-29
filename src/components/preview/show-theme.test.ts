import { describe, expect, it, vi } from "vitest";
import type { PlaybackItem, PlaybackScenePlayer } from "@/core/playback";
import {
  DEFAULT_SHOW_THEME_ID,
  getShowTheme,
  renderThemeFrame,
  type ShowThemePack,
} from "./show-theme";

describe("show theme registry", () => {
  it("falls back to the default mansion murder theme for unknown ids", () => {
    expect(getShowTheme("missing-theme").id).toBe(DEFAULT_SHOW_THEME_ID);
  });

  it("renders a frame through the fixed player-stage layers", () => {
    const calls: string[] = [];
    const theme: ShowThemePack = {
      id: "test_theme",
      name: "Test Theme",
      tokens: {
        background: "#000",
        panel: "#111",
        text: "#fff",
        muted: "#777",
        accent: "#0ff",
        danger: "#f00",
        good: "#0f0",
        wolf: "#f44",
      },
      draw: {
        background: vi.fn(() => calls.push("background")),
        topBar: vi.fn(() => calls.push("topBar")),
        seat: vi.fn(() => calls.push("seat")),
        speechShot: vi.fn(() => calls.push("speech")),
        voteShot: vi.fn(() => calls.push("vote")),
        phaseShot: vi.fn(() => calls.push("phase")),
        announcementShot: vi.fn(() => calls.push("announcement")),
        resolutionShot: vi.fn(() => calls.push("resolution")),
        subtitle: vi.fn(() => calls.push("subtitle")),
        effects: vi.fn(() => calls.push("effects")),
      },
    };

    renderThemeFrame(fakeContext(), {
      theme,
      scene: scene({ kind: "speech", players: [player()] }),
      items: [scene({ kind: "speech", players: [player()] })],
      timeMs: 1200,
      avatarImages: {},
    });

    expect(calls).toEqual([
      "background",
      "topBar",
      "seat",
      "speech",
      "subtitle",
      "effects",
    ]);
  });
});

function scene(overrides: Partial<PlaybackItem> = {}): PlaybackItem {
  return {
    index: 1,
    phase: "speech",
    kind: "announcement",
    title: "Title",
    text: "Text",
    details: [],
    durationMs: 2000,
    startsAtMs: 0,
    players: [],
    ...overrides,
  };
}

function player(): PlaybackScenePlayer {
  return {
    playerId: "player_1" as PlaybackScenePlayer["playerId"],
    seatNo: 1,
    name: "秦川",
    avatar: null,
    roleName: "狼人",
    status: "alive" as const,
    highlighted: true,
  };
}

function fakeContext(): CanvasRenderingContext2D {
  return {
    canvas: { width: 1920, height: 1080 },
    fillRect: vi.fn(),
    strokeRect: vi.fn(),
    fillText: vi.fn(),
    measureText: vi.fn(() => ({ width: 10 })),
    beginPath: vi.fn(),
    moveTo: vi.fn(),
    lineTo: vi.fn(),
    fill: vi.fn(),
    stroke: vi.fn(),
    arc: vi.fn(),
    drawImage: vi.fn(),
    save: vi.fn(),
    restore: vi.fn(),
    createLinearGradient: vi.fn(() => ({ addColorStop: vi.fn() })),
  } as unknown as CanvasRenderingContext2D;
}
