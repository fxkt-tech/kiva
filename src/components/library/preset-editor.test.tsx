import React from "react";
import { renderToStaticMarkup } from "react-dom/server";
import { describe, expect, it } from "vitest";
import type { GamePreset } from "@/core/game-preset";
import type { ModelBindingSnapshot } from "@/core/player";
import { seedCharacters } from "@/seeds/characters";
import { seedPresets } from "@/seeds/presets";
import { seedRoles } from "@/seeds/roles";
import { PresetEditor } from "./preset-editor";

const modelBinding = {
  provider: "mock",
  model: "seat-model",
  temperature: 0.3,
  maxTokens: 600,
  responseFormat: "json",
} satisfies ModelBindingSnapshot;

describe("PresetEditor", () => {
  it("renders a seat table and create-game action", () => {
    const html = renderEditor(seedPresets[0]);

    expect(html).toContain("Seat 1");
    expect(html).toContain('name="seat.1.roleId"');
    expect(html).toContain('name="seat.1.characterId"');
    expect(html).toContain("Save preset");
    expect(html).toContain("Create game");
  });

  it("preserves seat model binding overrides in hidden JSON", () => {
    const preset: GamePreset = {
      ...seedPresets[0],
      seatAssignments: seedPresets[0].seatAssignments?.map((seat) =>
        seat.seatNo === 2
          ? { ...seat, modelBindingOverride: modelBinding }
          : seat,
      ) ?? null,
    };

    const html = renderEditor(preset);

    expect(html).toContain('name="seat.2.modelBindingOverride"');
    expect(decodeHtml(html)).toContain(JSON.stringify(modelBinding));
  });

  it("can render editable seats from role and character id lists", () => {
    const preset: GamePreset = {
      ...seedPresets[0],
      playerCount: 2,
      roleIds: ["werewolf", "seer"],
      characterIds: ["qin_chuan", "lin_xia"],
      seatAssignments: null,
    };

    const html = renderEditor(preset);

    expect(html).toContain("Seat 1");
    expect(html).toContain("Seat 2");
    expect(html).toContain('name="seat.2.roleId"');
    expect(html).toContain('name="seat.2.characterId"');
  });
});

function renderEditor(preset: GamePreset): string {
  return renderToStaticMarkup(
    React.createElement(PresetEditor, {
      preset,
      roles: seedRoles,
      characters: seedCharacters,
    }),
  );
}

function decodeHtml(value: string): string {
  return value.replaceAll("&quot;", "\"");
}
