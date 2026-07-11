import { describe, expect, it } from "vitest";
import {
  isAutoConfirmDraftEnabled,
  isCompletedLlmGeneration,
  shouldScheduleAutoConfirmDraft,
} from "./llm-generation-notification";

describe("LLM generation notifications", () => {
  it("recognizes a new generation for the request started by this page", () => {
    const pending = JSON.stringify({
      draftId: "draft_1",
      previousGenerationId: "generation_1",
    });

    expect(
      isCompletedLlmGeneration(pending, "draft_1", "generation_2"),
    ).toBe(true);
  });

  it("does not report an unchanged or unrelated generation", () => {
    const pending = JSON.stringify({
      draftId: "draft_1",
      previousGenerationId: "generation_1",
    });

    expect(
      isCompletedLlmGeneration(pending, "draft_1", "generation_1"),
    ).toBe(false);
    expect(
      isCompletedLlmGeneration(pending, "draft_2", "generation_2"),
    ).toBe(false);
    expect(isCompletedLlmGeneration("invalid", "draft_1", "generation_2"))
      .toBe(false);
  });

  it("defaults auto-confirm to disabled and only accepts the stored true value", () => {
    expect(isAutoConfirmDraftEnabled(null)).toBe(false);
    expect(isAutoConfirmDraftEnabled("false")).toBe(false);
    expect(isAutoConfirmDraftEnabled("invalid")).toBe(false);
    expect(isAutoConfirmDraftEnabled("true")).toBe(true);
  });

  it("schedules auto-confirm only when the control is enabled and the draft is ready", () => {
    expect(shouldScheduleAutoConfirmDraft(true, true)).toBe(true);
    expect(shouldScheduleAutoConfirmDraft(true, false)).toBe(false);
    expect(shouldScheduleAutoConfirmDraft(false, true)).toBe(false);
    expect(shouldScheduleAutoConfirmDraft(false, false)).toBe(false);
  });
});
