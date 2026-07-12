import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import {
  diagnoseCharacter,
  diagnosePresenter,
  diagnosePreset,
  diagnoseRole,
} from "@/core/library-diagnostics";
import { seedCharacters } from "@/seeds/characters";
import { seedPresets } from "@/seeds/presets";
import { seedRoles } from "@/seeds/roles";
import { seedPresenters } from "@/seeds/presenters";
import { seedScripts } from "@/seeds/scripts";
import type { LibraryActionsRecord } from "@/server/library-actions";
import { ValidationPanel } from "./validation-panel";

const werewolfRole = seedRoles.find((role) => role.id === "werewolf")!;

describe("ValidationPanel", () => {
  it("renders role diagnostics and prompt preview", () => {
    const html = renderToStaticMarkup(
      React.createElement(ValidationPanel, {
        activeTab: "roles",
        selectedId: "werewolf",
        library: libraryFixture(),
      }),
    );

    expect(html).toContain("Validation");
    expect(html).toContain("valid");
    expect(html).toContain("Built-in role contract locked");
    expect(html).toContain("twelve_player_standard");
    expect(html).toContain("Prompt preview");
    expect(html).toContain(werewolfRole.systemPrompt);
  });

  it("renders preset create-game diagnostics", () => {
    const html = renderToStaticMarkup(
      React.createElement(ValidationPanel, {
        activeTab: "presets",
        selectedId: "twelve_player_standard",
        library: libraryFixture(),
      }),
    );

    expect(html).toContain("Can create game");
    expect(html).toContain("yes");
    expect(html).toContain("模型：");
  });

  it("renders an empty diagnostic state for missing selections", () => {
    const html = renderToStaticMarkup(
      React.createElement(ValidationPanel, {
        activeTab: "characters",
        selectedId: "missing",
        library: libraryFixture(),
      }),
    );

    expect(html).toContain("No diagnostics available.");
  });
});

function libraryFixture(): LibraryActionsRecord {
  return {
    roles: seedRoles,
    characters: seedCharacters,
    presets: seedPresets,
    presenters: seedPresenters,
    scripts: seedScripts,
    diagnostics: {
      roles: Object.fromEntries(
        seedRoles.map((role) => [
          role.id,
          diagnoseRole({
            roles: seedRoles,
            characters: seedCharacters,
            presets: seedPresets,
            role,
          }),
        ]),
      ),
      characters: Object.fromEntries(
        seedCharacters.map((character) => [
          character.id,
          diagnoseCharacter({
            roles: seedRoles,
            characters: seedCharacters,
            presets: seedPresets,
            character,
          }),
        ]),
      ),
      presets: Object.fromEntries(
        seedPresets.map((preset) => [
          preset.id,
          diagnosePreset({
            roles: seedRoles,
            characters: seedCharacters,
            presets: seedPresets,
            preset,
          }),
        ]),
      ),
      presenters: Object.fromEntries(
        seedPresenters.map((presenter) => [
          presenter.id,
          diagnosePresenter({ presenters: seedPresenters, presenter }),
        ]),
      ),
    },
  };
}
