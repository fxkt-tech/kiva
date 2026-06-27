import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { RecordingControls } from "./recording-controls";

describe("RecordingControls", () => {
  it("renders clean preview and browser recording actions", () => {
    const html = renderToStaticMarkup(
      React.createElement(RecordingControls, {
        cleanPreviewHref: "/games/game_1/preview?controls=0",
      }),
    );

    expect(html).toContain("Open clean preview");
    expect(html).toContain("/games/game_1/preview?controls=0");
    expect(html).toContain("Start recording");
    expect(html).toContain("Stop");
    expect(html).not.toContain("Download WebM");
  });
});
