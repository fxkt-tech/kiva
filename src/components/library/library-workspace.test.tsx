import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { diagnoseCharacter, diagnosePreset, diagnoseRole } from "@/core/library-diagnostics";
import { seedCharacters } from "@/seeds/characters";
import { seedPresets } from "@/seeds/presets";
import { seedRoles } from "@/seeds/roles";
import type { LibraryActionsRecord } from "@/server/library-actions";
import { LibraryList } from "./library-list";
import { LibraryWorkspace } from "./library-workspace";

describe("LibraryWorkspace", () => {
  it("renders tabs, object list, detail region, and validation region", () => {
    const html = renderToStaticMarkup(
      React.createElement(LibraryWorkspace, {
        activeTab: "roles",
        selectedId: "werewolf",
        library: libraryFixture(),
      }),
    );

    expect(html).toContain("Kiva Library");
    expect(html).toContain("Roles");
    expect(html).toContain("Characters");
    expect(html).toContain("Presets");
    expect(html).toContain("狼人");
    expect(html).toContain("Role editor");
    expect(html).toContain("Validation");
  });

  it("marks the selected item in the active list", () => {
    const html = renderToStaticMarkup(
      React.createElement(LibraryWorkspace, {
        activeTab: "characters",
        selectedId: "lin_xia",
        library: libraryFixture(),
      }),
    );

    expect(html).toContain('aria-current="page"');
    expect(html).toContain("林夏");
    expect(html).toContain("Character editor");
  });

  it("selects the first item when selectedId is missing or invalid", () => {
    const withoutId = renderToStaticMarkup(
      React.createElement(LibraryWorkspace, {
        activeTab: "presets",
        selectedId: null,
        library: libraryFixture(),
      }),
    );
    const withInvalidId = renderToStaticMarkup(
      React.createElement(LibraryWorkspace, {
        activeTab: "presets",
        selectedId: "missing",
        library: libraryFixture(),
      }),
    );

    expect(withoutId).toContain("Preset editor");
    expect(withoutId).toContain("6人狼人杀试运行");
    expect(withInvalidId).toContain("Preset editor");
    expect(withInvalidId).toContain("6人狼人杀试运行");
  });

  it("URL-encodes special characters in object list links", () => {
    const html = renderToStaticMarkup(
      React.createElement(LibraryList, {
        tab: "roles",
        selectedId: null,
        items: [
          {
            id: "role&/#/?id=1",
            name: "Special role",
            enabled: true,
            meta: "custom",
            valid: true,
          },
        ],
      }),
    );

    expect(html).toContain(
      'href="/library?tab=roles&amp;id=role%26%2F%23%2F%3Fid%3D1"',
    );
    expect(html).not.toContain('href="/library?tab=roles&amp;id=role&amp;');
  });
});

function libraryFixture(): LibraryActionsRecord {
  return {
    roles: seedRoles,
    characters: seedCharacters,
    presets: seedPresets,
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
    },
  };
}
