import { describe, expect, it } from "vitest";
import {
  createGameScriptSnapshot,
  legacyGameScriptSnapshot,
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

  it("provides a validated legacy compatibility snapshot", () => {
    const legacy = legacyGameScriptSnapshot();
    expect(legacy.presentation.styleKey).toBe("legacy_v1");
    expect(validateGameScriptSnapshot(legacy)).toEqual(legacy);
  });
});
