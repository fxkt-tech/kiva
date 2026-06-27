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

    expect(html).toContain("First event");
    expect(html).toContain("Opening announcement");
    expect(html).not.toContain("Second event");
    expect(html).not.toContain("Later announcement");
  });

  it("renders the waiting state when there are no public playback items", () => {
    const html = renderStage([]);

    expect(html).toContain("Waiting for public event");
  });

  it("renders sequence controls and progress", () => {
    const html = renderStage([
      item({ index: 1, title: "First event" }),
      item({ index: 2, title: "Second event" }),
      item({ index: 3, title: "Third event" }),
    ]);

    expect(html).toContain("Prev");
    expect(html).toContain("Play");
    expect(html).toContain("Next");
    expect(html).toContain("Reset");
    expect(html).toContain("Speed");
    expect(html).toContain("0.5x");
    expect(html).toContain("2x");
    expect(html).toContain("1 / 3");
  });

  it("renders scene details and player status inside the stage", () => {
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

    expect(html).toContain("1 号 秦川");
    expect(html).toContain("2 号 林夏");
    expect(html).toContain("Seat 1");
    expect(html).toContain("秦川");
    expect(html).toContain("Seat 2");
    expect(html).toContain("林夏");
    expect(html).toContain("dead");
  });

  it("renders dedicated speech and vote templates", () => {
    const speechHtml = renderStage([
      item({
        index: 2,
        kind: "speech",
        title: "1 号 秦川发言",
        text: "我先报一下我的视角。",
        players: [
          player({ seatNo: 1, name: "秦川", highlighted: true }),
          player({ seatNo: 2, name: "林夏" }),
        ],
      }),
    ]);
    const voteHtml = renderStage([
      item({
        index: 3,
        kind: "vote",
        title: "放逐投票",
        text: "1 号 秦川 投给 2 号 林夏。",
        details: ["1 号 秦川 -> 2 号 林夏"],
      }),
    ]);

    expect(speechHtml).toContain("Speaker");
    expect(speechHtml).toContain("秦川");
    expect(voteHtml).toContain("Vote card");
    expect(voteHtml).toContain("1 号 秦川");
    expect(voteHtml).toContain("2 号 林夏");
  });

  it("disables playback controls that cannot advance a single-item sequence", () => {
    const html = renderStage([item({ index: 1, title: "Only event" })]);

    expect(html).toContain("Only event");
    expect(html).toContain(">Play</button>");
    expect(html).toContain(">Next</button>");
    expect(html).toContain("disabled");
  });
});

function renderStage(items: readonly PlaybackItem[]): string {
  return renderToStaticMarkup(React.createElement(PlaybackStage, { items }));
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
    ...overrides,
  };
}
