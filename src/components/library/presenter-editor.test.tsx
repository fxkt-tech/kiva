import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { seedPresenters } from "@/seeds/presenters";
import type { PresenterVoiceManifest } from "@/core/presenter-voice";
import nightWatchManifest from "../../../kivdb/presenters/night_watch/manifest.json";
import { PresenterEditor } from "./presenter-editor";

describe("PresenterEditor", () => {
  it("renders identity and every copy template", () => {
    const presenter = seedPresenters[0]!;
    const html = renderToStaticMarkup(
      React.createElement(PresenterEditor, { presenter }),
    );

    expect(html).toContain("Presenter editor");
    expect(html).toContain("守夜人");
    expect(html).toContain("44 lines");
    expect(html).toContain('name="avatar"');
    expect(html).toContain('name="line.phase.night.template"');
    expect(html).toContain("预制 manifest 构建命令统一生成");
    expect(html).not.toContain("voice.file");
    expect(html).toContain("Save presenter");
  });

  it("uses the presenter name glyph when no avatar is configured", () => {
    const html = renderToStaticMarkup(
      React.createElement(PresenterEditor, { presenter: seedPresenters[1]! }),
    );

    expect(html).toContain(">法</div>");
  });

  it("renders an audio preview for every line and seat selectors for variants", () => {
    const html = renderToStaticMarkup(
      React.createElement(PresenterEditor, {
        presenter: seedPresenters[0]!,
        voiceManifest: nightWatchManifest as PresenterVoiceManifest,
      }),
    );

    expect(html.match(/aria-label="播放主持音频"/g)).toHaveLength(44);
    expect(html.match(/aria-label="试听座位"/g)).toHaveLength(15);
  });
});
