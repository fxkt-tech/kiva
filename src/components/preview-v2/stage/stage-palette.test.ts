import { describe, expect, it } from "vitest";
import { createGameScriptSnapshot } from "@/core/game-script";
import { seedScripts } from "@/seeds/scripts";
import { stagePaletteForPhase } from "./stage-palette";

const presentation = createGameScriptSnapshot(seedScripts[0]!).presentation;

describe("preview v2 smoke-glass stage palette", () => {
  it("uses the approved translucent night palette", () => {
    expect(stagePaletteForPhase("night", presentation)).toMatchObject({
      majorSurface: "rgba(3,15,17,0.78)",
      cardSurface: "rgba(4,18,20,0.74)",
      text: "#D8C9A7",
      number: "#D8C9A7",
      accent: "#70A4A7",
    });
  });

  it("uses the approved translucent day palette", () => {
    expect(stagePaletteForPhase("day", presentation)).toMatchObject({
      majorSurface: "rgba(16,24,23,0.82)",
      cardSurface: "rgba(20,28,25,0.76)",
      text: "#D8C9A7",
      number: "#D8C9A7",
      accent: "#70A4A7",
    });
  });
});
