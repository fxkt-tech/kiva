import { describe, expect, it } from "vitest";
import {
  createGameScriptSnapshot,
  validateGameScriptDefinitions,
  validateGameScriptSnapshot,
} from "../game-script";
import { seedScripts } from "@/seeds/scripts";

describe("game script definitions", () => {
  it("validates and snapshots the seeded script", () => {
    expect(validateGameScriptDefinitions(seedScripts)).toHaveLength(1);
    const snapshot = createGameScriptSnapshot(seedScripts[0]!);
    expect(snapshot.scriptSourceId).toBe("midnight_archive");
    expect(validateGameScriptSnapshot(snapshot)).toEqual(snapshot);
    expect(snapshot).not.toBe(seedScripts[0]);
    expect(snapshot.presentation).not.toBe(seedScripts[0]!.presentation);
  });

  it("rejects unsafe assets and duplicate ids", () => {
    const base = seedScripts[0]!;
    expect(() =>
      validateGameScriptDefinitions([
        base,
        { ...base, presentation: { ...base.presentation, coverImage: "../secret.png" } },
      ]),
    ).toThrow(/Duplicate game script/);
    expect(() =>
      validateGameScriptDefinitions([
        { ...base, presentation: { ...base.presentation, coverImage: "../secret.png" } },
      ]),
    ).toThrow(/safe internal PNG/);
  });

  it("rejects Preview assets and removed visual styles", () => {
    const snapshot = createGameScriptSnapshot(seedScripts[0]!);
    expect(() =>
      validateGameScriptSnapshot({
        ...snapshot,
        presentation: {
          ...snapshot.presentation,
          dayBackground: "/kivdb-assets/preview/day-background.png",
        },
      }),
    ).toThrow("safe internal PNG asset");
    expect(() =>
      validateGameScriptSnapshot({
        ...snapshot,
        presentation: { ...snapshot.presentation, styleKey: "legacy_v1" },
      }),
    ).toThrow("styleKey is invalid");
  });
});
