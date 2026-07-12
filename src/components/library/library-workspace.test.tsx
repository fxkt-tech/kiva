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
import { LibraryList } from "./library-list";
import { LibraryWorkspace, selectedEditorKey } from "./library-workspace";

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
    expect(html).toContain("Presenters");
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
    expect(html).toContain("乔可");
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
    expect(withoutId).toContain("12人狼人杀标准局");
    expect(withInvalidId).toContain("Preset editor");
    expect(withInvalidId).toContain("12人狼人杀标准局");
  });

  it("renders a presenter as a configurable library object", () => {
    const html = renderToStaticMarkup(
      React.createElement(LibraryWorkspace, {
        activeTab: "presenters",
        selectedId: "judge",
        library: libraryFixture(),
      }),
    );

    expect(html).toContain("Presenter editor");
    expect(html).toContain("闻舟");
    expect(html).toContain("主持文案");
    expect(html).toContain('name="line.phase.night.template"');
  });

  it("uses the selected object identity as the editor key", () => {
    expect(selectedEditorKey({ kind: "role", id: "werewolf" })).toBe(
      "role:werewolf",
    );
    expect(selectedEditorKey({ kind: "role", id: "seer" })).toBe("role:seer");
    expect(selectedEditorKey({ kind: "character", id: "seer" })).toBe(
      "character:seer",
    );
    expect(selectedEditorKey(null)).toBeNull();
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
