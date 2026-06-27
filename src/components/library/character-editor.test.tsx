import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import type { CharacterDefinition } from "@/core/character-definition";
import type { ModelBindingSnapshot } from "@/core/player";
import { seedCharacters } from "@/seeds/characters";
import { CharacterEditor } from "./character-editor";

const modelBinding = {
  provider: "mock",
  model: "character-model",
  temperature: 0.5,
  maxTokens: 700,
  responseFormat: "json",
} satisfies ModelBindingSnapshot;

describe("CharacterEditor", () => {
  it("renders editable character fields", () => {
    const html = renderToStaticMarkup(
      React.createElement(CharacterEditor, { character: seedCharacters[0] }),
    );

    expect(html).toContain('name="persona"');
    expect(html).toContain('name="speakingStyle"');
    expect(html).toContain('name="reasoningStyle"');
    expect(html).toContain('name="systemPrompt"');
    expect(html).toContain("Save character");
  });

  it("does not render role-only fields", () => {
    const html = renderToStaticMarkup(
      React.createElement(CharacterEditor, { character: seedCharacters[0] }),
    );

    expect(html).not.toContain('name="mechanicKey"');
    expect(html).not.toContain('name="faction"');
    expect(html).not.toContain('name="team"');
  });

  it("preserves default model binding in hidden JSON", () => {
    const character: CharacterDefinition = {
      ...seedCharacters[0],
      defaultModelBinding: modelBinding,
    };

    const html = renderToStaticMarkup(
      React.createElement(CharacterEditor, { character }),
    );

    expect(html).toContain('name="defaultModelBinding"');
    expect(decodeHtml(html)).toContain(JSON.stringify(modelBinding));
  });
});

function decodeHtml(value: string): string {
  return value.replaceAll("&quot;", "\"");
}
