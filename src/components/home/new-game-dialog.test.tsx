import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { seedCharacters } from "@/seeds/characters";
import { seedPresets } from "@/seeds/presets";
import { seedRoles } from "@/seeds/roles";
import { NewGameDialog } from "./new-game-dialog";

describe("NewGameDialog", () => {
  it("renders a dialog trigger without immediately rendering setup forms", () => {
    const html = renderToStaticMarkup(
      React.createElement(NewGameDialog, {
        presets: seedPresets,
        roles: seedRoles,
        characters: seedCharacters,
      }),
    );

    expect(html).toContain("New game");
    expect(html).not.toContain("Create game");
    expect(html).not.toContain('name="seat.1.roleId"');
  });
});
