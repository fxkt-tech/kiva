import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import { seedCharacters } from "@/seeds/characters";
import { seedPresets } from "@/seeds/presets";
import { seedRoles } from "@/seeds/roles";
import { seedPresenters } from "@/seeds/presenters";
import { NewGameDialog, PresenterPicker } from "./new-game-dialog";

describe("NewGameDialog", () => {
  it("renders a dialog trigger without immediately rendering setup forms", () => {
    const html = renderToStaticMarkup(
      React.createElement(NewGameDialog, {
        presets: seedPresets,
        roles: seedRoles,
        characters: seedCharacters,
        presenters: seedPresenters,
      }),
    );

    expect(html).toContain("New game");
    expect(html).not.toContain("Create game");
    expect(html).not.toContain('name="seat.1.roleId"');
  });

  it("renders every enabled presenter choice", () => {
    const html = renderToStaticMarkup(
      React.createElement(PresenterPicker, {
        presenters: seedPresenters,
        selectedPresenterId: seedPresenters[0]!.id,
        onSelectPresenter: () => {},
      }),
    );

    expect(html).toContain("守夜人 · night_watch");
    expect(html).toContain("法官 · judge");
  });
});
