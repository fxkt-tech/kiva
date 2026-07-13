import { describe, expect, it } from "vitest";
import { parseGameRunMode } from "../game-run-mode";

describe("game run mode", () => {
  it("accepts the two supported modes", () => {
    expect(parseGameRunMode("game")).toBe("game");
    expect(parseGameRunMode("scripted")).toBe("scripted");
  });

  it("rejects missing and unsupported modes", () => {
    expect(() => parseGameRunMode(undefined)).toThrow(
      "Invalid game run mode: undefined",
    );
    expect(() => parseGameRunMode("story")).toThrow(
      "Invalid game run mode: story",
    );
  });
});
