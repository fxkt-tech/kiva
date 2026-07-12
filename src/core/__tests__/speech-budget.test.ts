import { describe, expect, it } from "vitest";
import type { DraftEvent } from "../drafts";
import {
  estimateSpeechDurationMs,
  evaluateSpeech,
  speechBudgetForDraft,
  speechBudgetForKey,
  spokenCharacterCount,
} from "../speech-budget";
import type { DraftId, GameId, PlayerId } from "../types";

const gameId = "speech_budget_game" as GameId;
const playerId = "p1" as PlayerId;

describe("speech budget", () => {
  it("owns the normal budget matrix", () => {
    expect(speechBudgetForKey("wolf_strategy")).toMatchObject({
      targetMinCharacters: 110,
      targetMaxCharacters: 150,
      hardMaxCharacters: 180,
    });
    expect(speechBudgetForKey("wolf_opinion").hardMaxCharacters).toBe(120);
    expect(speechBudgetForKey("day_first").hardMaxCharacters).toBe(170);
    expect(speechBudgetForKey("day_response").hardMaxCharacters).toBe(220);
    expect(speechBudgetForKey("last_words").hardMaxCharacters).toBe(240);
    expect(speechBudgetForKey("pk").hardMaxCharacters).toBe(220);
  });

  it("distinguishes first and responding day speakers", () => {
    const draft = daySpeechDraft();
    expect(
      speechBudgetForDraft({ draft, hasPriorDaySpeech: false }).key,
    ).toBe("day_first");
    expect(
      speechBudgetForDraft({ draft, hasPriorDaySpeech: true }).key,
    ).toBe("day_response");
  });

  it("counts Unicode code points while ignoring whitespace", () => {
    expect(spokenCharacterCount("你 好\n🙂")).toBe(3);
    expect(spokenCharacterCount("Ａ\tB")).toBe(2);
    expect(spokenCharacterCount("")).toBe(0);
  });

  it("compresses budgets without crossing target ordering", () => {
    const normal = speechBudgetForKey("day_response");
    const compressed = speechBudgetForKey("day_response", "compressed");
    const critical = speechBudgetForKey("day_response", "critical");

    expect(compressed.targetMaxCharacters).toBeLessThan(
      normal.targetMaxCharacters,
    );
    expect(critical.hardMaxCharacters).toBeLessThan(
      compressed.hardMaxCharacters,
    );
    expect(critical.hardMaxCharacters).toBeGreaterThanOrEqual(
      critical.targetMaxCharacters,
    );
  });

  it("evaluates target, long, and over-limit text", () => {
    const budget = speechBudgetForKey("wolf_opinion");

    expect(evaluateSpeech("字".repeat(80), budget).status).toBe("target");
    expect(evaluateSpeech("字".repeat(110), budget).status).toBe("long");
    expect(evaluateSpeech("字".repeat(121), budget)).toMatchObject({
      characterCount: 121,
      status: "over_limit",
      withinHardLimit: false,
    });
  });

  it("estimates duration from the calibrated speech rate", () => {
    expect(estimateSpeechDurationMs(0)).toBe(0);
    expect(estimateSpeechDurationMs(61)).toBe(10_000);
    expect(estimateSpeechDurationMs(Number.NaN)).toBe(0);
  });
});

function daySpeechDraft(): Extract<
  DraftEvent,
  { type: "day_speech_given" }
> {
  return {
    id: "draft_speech_budget" as DraftId,
    gameId,
    status: "draft",
    type: "day_speech_given",
    phase: "speech",
    actorPlayerId: playerId,
    visibility: { kind: "public" },
    payload: {
      playerId,
      text: "",
      dayNumber: 1,
      round: 1,
    },
    createdAt: "2026-07-12T00:00:00.000Z",
  };
}
