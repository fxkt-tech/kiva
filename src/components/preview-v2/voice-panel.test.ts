import { describe, expect, it } from "vitest";
import { shouldRefreshComposition } from "./voice-panel";

describe("voice composition refresh", () => {
  it("refreshes only when the same job publishes another artifact", () => {
    expect(shouldRefreshComposition(null, { jobId: "a", completedItems: 0 })).toBe(false);
    expect(shouldRefreshComposition(
      { jobId: "a", completedItems: 0 },
      { jobId: "a", completedItems: 0 },
    )).toBe(false);
    expect(shouldRefreshComposition(
      { jobId: "a", completedItems: 0 },
      { jobId: "a", completedItems: 1 },
    )).toBe(true);
    expect(shouldRefreshComposition(
      { jobId: "a", completedItems: 1 },
      { jobId: "b", completedItems: 2 },
    )).toBe(false);
  });
});
