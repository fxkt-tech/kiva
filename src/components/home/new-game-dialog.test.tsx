import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { seedCharacters } from "@/seeds/characters";
import { seedPresets } from "@/seeds/presets";
import { seedRoles } from "@/seeds/roles";
import { seedScripts } from "@/seeds/scripts";
import { NewGameDialog, ScriptPicker } from "./new-game-dialog";

describe("NewGameDialog", () => {
  it("renders a dialog trigger without immediately rendering setup forms", () => {
    const html = renderToStaticMarkup(
      React.createElement(NewGameDialog, {
        presets: seedPresets,
        roles: seedRoles,
        characters: seedCharacters,
        scripts: seedScripts,
      }),
    );

    expect(html).toContain("New game");
    expect(html).not.toContain("Create game");
    expect(html).not.toContain('name="seat.1.roleId"');
  });

  it("renders the selected script with narrative context", () => {
    const html = renderToStaticMarkup(
      React.createElement(ScriptPicker, {
        scripts: seedScripts,
        selectedScriptId: seedScripts[0]!.id,
        onSelectScript: () => {},
      }),
    );

    expect(html).toContain("未明档案");
    expect(html).toContain("东方都市异闻");
    expect(html).toContain("已选择");
  });
});
