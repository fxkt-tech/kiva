import { describe, expect, it } from "vitest";
import { stagePaletteForPhase } from "./stage-palette";

describe("preview v2 smoke-glass stage palette", () => {
  it("uses the approved translucent night palette", () => {
    expect(stagePaletteForPhase("night")).toMatchObject({
      majorSurface: "rgba(4,7,13,0.70)",
      cardSurface: "rgba(4,8,14,0.58)",
      text: "#FFF7E7",
      number: "#E5C67C",
      accent: "#F0D084",
    });
  });

  it("uses the approved translucent day palette", () => {
    expect(stagePaletteForPhase("day")).toMatchObject({
      majorSurface: "rgba(16,18,15,0.76)",
      cardSurface: "rgba(13,16,13,0.66)",
      text: "#FFFAF0",
      number: "#F0C565",
      accent: "#F4C45C",
    });
  });
});
