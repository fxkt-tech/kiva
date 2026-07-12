import { describe, expect, it } from "vitest";
import { normalizeGameRunMode, parseGameRunMode } from "../game-run-mode";

describe("game run mode", () => {
  it("accepts the two supported modes", () => {
    expect(parseGameRunMode("game")).toBe("game");
    expect(parseGameRunMode("scripted")).toBe("scripted");
  });

  it("defaults only missing historical values to game mode", () => {
    expect(normalizeGameRunMode(undefined)).toBe("game");
    expect(() => normalizeGameRunMode("story")).toThrow(
      "Invalid game run mode: story",
    );
  });
});
