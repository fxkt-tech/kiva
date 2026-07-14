import { describe, expect, it } from "vitest";
import {
  isAutoContinueEpisodeAuthorEnabled,
  shouldScheduleEpisodeAuthorAdvance,
} from "./episode-auto-continue";

describe("Episode Author auto-continue", () => {
  it("defaults to disabled and only accepts the stored true value", () => {
    expect(isAutoContinueEpisodeAuthorEnabled(null)).toBe(false);
    expect(isAutoContinueEpisodeAuthorEnabled("false")).toBe(false);
    expect(isAutoContinueEpisodeAuthorEnabled("invalid")).toBe(false);
    expect(isAutoContinueEpisodeAuthorEnabled("true")).toBe(true);
  });

  it("schedules only an enabled ready snapshot with a current job ID", () => {
    expect(shouldScheduleEpisodeAuthorAdvance(true, true, "ready_job"))
      .toBe(true);
    expect(shouldScheduleEpisodeAuthorAdvance(true, true, null)).toBe(false);
    expect(shouldScheduleEpisodeAuthorAdvance(true, false, "running_job"))
      .toBe(false);
    expect(shouldScheduleEpisodeAuthorAdvance(false, true, "ready_job"))
      .toBe(false);
  });
});
