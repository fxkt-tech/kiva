import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { RecordingWorkspace } from "./recording-workspace";

describe("RecordingWorkspace", () => {
  it("renders a clean 16:9 preview frame with recording actions", () => {
    const html = renderToStaticMarkup(
      React.createElement(RecordingWorkspace, {
        cleanPreviewHref: "/games/game_1/preview?controls=0",
        previewHref: "/games/game_1/preview",
        title: "6人狼人杀试运行",
      }),
    );

    expect(html).toContain("6人狼人杀试运行");
    expect(html).toContain("Recording studio");
    expect(html).toContain("src=\"/games/game_1/preview?controls=0\"");
    expect(html).toContain("aspect-video");
    expect(html).toContain("Start recording");
    expect(html).toContain("Open clean preview");
    expect(html).toContain("Back to preview");
  });
});
