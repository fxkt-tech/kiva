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
    phase: "day",
    text: "",
    ...overrides,
  };
}
