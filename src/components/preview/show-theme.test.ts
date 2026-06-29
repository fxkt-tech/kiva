import { describe, expect, it, vi } from "vitest";
import type { PlaybackItem } from "@/core/playback";
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
      render: {
        renderBackground: vi.fn(() => calls.push("background")),
        renderPlayerCard: vi.fn(() => calls.push("players")),
        renderCenterStage: vi.fn(() => calls.push("center")),
        renderSubtitle: vi.fn(() => calls.push("subtitle")),
        renderEffect: vi.fn(() => calls.push("effect")),
      },
    };

    renderThemeFrame(fakeContext(), {
      theme,
      scene: scene({ kind: "speech" }),
      items: [scene({ kind: "speech" })],
      layout: {
        playerSlots: [],
        center: { x: 0, y: 0, width: 100, height: 100 },
        subtitle: { x: 0, y: 100, width: 100, height: 20 },
      },
      timeMs: 1200,
      avatarImages: {},
    });

    expect(calls).toEqual(["background", "players", "center", "subtitle", "effect"]);
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
