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

  it("renders the character model binding as editable fields", () => {
    const character: CharacterDefinition = {
      ...seedCharacters[0],
      defaultModelBinding: modelBinding,
    };

    const html = renderToStaticMarkup(
      React.createElement(CharacterEditor, { character }),
    );

    expect(html).toContain('name="modelBinding.provider"');
    expect(html).toContain('value="mock"');
    expect(html).toContain('name="modelBinding.model"');
    expect(html).toContain('value="character-model"');
    expect(html).toContain('name="modelBinding.fallbackModel"');
    expect(html).not.toContain("temperature");
    expect(html).not.toContain("maxTokens");
  });
});
