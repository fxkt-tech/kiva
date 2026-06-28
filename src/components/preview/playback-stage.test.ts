import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import type { PlaybackItem } from "@/core/playback";
import { PlaybackStage } from "./playback-stage";

describe("PlaybackStage", () => {
  it("initially renders the first public playback item", () => {
    const html = renderStage([
      item({ index: 1, title: "First event", text: "Opening announcement" }),
      item({ index: 2, title: "Second event", text: "Later announcement" }),
    ]);

    expect(html).toContain("width=\"1920\"");
    expect(html).toContain("height=\"1080\"");
    expect(html).toContain("aria-label=\"Playback canvas\"");
    expect(html).not.toContain("Second event");
    expect(html).not.toContain("Later announcement");
  });

  it("renders the waiting state when there are no public playback items", () => {
    const html = renderStage([]);

    expect(html).toContain("No playable scenes");
  });

  it("renders sequence controls and progress", () => {
    const html = renderStage([
      item({ index: 1, title: "First event" }),
      item({ index: 2, title: "Second event", startsAtMs: 2200 }),
      item({ index: 3, title: "Third event", startsAtMs: 4400 }),
    ]);

    expect(html).toContain("Record");
    expect(html).toContain("Play");
    expect(html).toContain("Reset");
    expect(html).toContain("Timeline");
    expect(html).toContain("0:00 / 0:06");
    expect(html).toContain("type=\"range\"");
    expect(html).toContain("1 / 3");
    expect(html).not.toContain("Open clean preview");
    expect(html).not.toContain("Open recording studio");
  });

  it("can hide controls for recording output", () => {
    const html = renderStage(
      [item({ index: 1, title: "Recording frame" })],
      { controls: "hidden" },
    );

    expect(html).toContain("aria-label=\"Playback canvas\"");
    expect(html).not.toContain("Playback controls");
    expect(html).not.toContain("Record");
  });

  it("keeps scene content out of the DOM because video output is canvas-only", () => {
    const html = renderStage([
      item({
        index: 8,
        title: "投票结算",
        text: "放逐出局：2 号 林夏。",
        details: ["1 号 秦川 -> 2 号 林夏"],
        players: [
          player({ seatNo: 1, name: "秦川", highlighted: false }),
          player({ seatNo: 2, name: "林夏", status: "dead", highlighted: true }),
        ],
      }),
    ]);

    expect(html).not.toContain("放逐出局");
    expect(html).not.toContain("秦川");
    expect(html).not.toContain("林夏");
  });

  it("disables playback controls that cannot advance a single-item sequence", () => {
    const html = renderStage([item({ index: 1, title: "Only event" })]);

    expect(html).toContain(">Play</button>");
    expect(html).toContain(">Record</button>");
    expect(html).toContain("disabled");
  });
});

function renderStage(
  items: readonly PlaybackItem[],
  props: Partial<React.ComponentProps<typeof PlaybackStage>> = {},
): string {
  return renderToStaticMarkup(
    React.createElement(PlaybackStage, {
      items,
      ...props,
    }),
  );
}

function item(
  overrides: Partial<PlaybackItem> & Pick<PlaybackItem, "index" | "title">,
): PlaybackItem {
  return {
    durationMs: 2200,
    startsAtMs: 0,
    kind: "announcement",
    phase: "day",
    text: "",
    details: [],
    players: [],
    ...overrides,
  };
}

function player(
  overrides: Partial<PlaybackItem["players"][number]> &
    Pick<PlaybackItem["players"][number], "seatNo" | "name">,
): PlaybackItem["players"][number] {
  return {
    playerId: `player_${overrides.seatNo}` as PlaybackItem["players"][number]["playerId"],
    status: "alive",
    highlighted: false,
    roleName: "狼人",
    ...overrides,
  };
}
