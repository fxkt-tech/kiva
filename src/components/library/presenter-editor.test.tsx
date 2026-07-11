import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { seedPresenters } from "@/seeds/presenters";
import { PresenterEditor } from "./presenter-editor";

describe("PresenterEditor", () => {
  it("renders identity, every copy template, and voice configuration", () => {
    const presenter = seedPresenters[0]!;
    const html = renderToStaticMarkup(
      React.createElement(PresenterEditor, { presenter }),
    );

    expect(html).toContain("Presenter editor");
    expect(html).toContain("守夜人");
    expect(html).toContain("44 lines");
    expect(html).toContain('name="avatar"');
    expect(html).toContain('name="line.phase.night.template"');
    expect(html).toContain('name="line.phase.night.voice.file"');
    expect(html).toContain('name="line.prompt.speech.voice.1.file"');
    expect(html).toContain('name="line.prompt.speech.voice.12.status"');
    expect(html).toContain("Save presenter");
  });

  it("uses the presenter name glyph when no avatar is configured", () => {
    const html = renderToStaticMarkup(
      React.createElement(PresenterEditor, { presenter: seedPresenters[1]! }),
    );

    expect(html).toContain(">法</div>");
  });
});
